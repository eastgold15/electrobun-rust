//! rust-asar CLI — ASAR 打包工具
//!
//! 兼容命令：

#![allow(clippy::expect_used)]
//!   rust-asar pack <source_dir> <output.asar> [--unpack pattern ...]
//!
//! 注意：此文件自包含 ASAR 打包逻辑，不依赖 lib.rs（cdylib）。
//! lib.rs 中的 asar 模块只包含读取逻辑和 C FFI 导出。

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use walkdir::WalkDir;

/// ASAR 节点
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AsarNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset: Option<String>,
    #[serde(default)]
    size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unpacked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    files: Option<HashMap<String, AsarNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AsarRoot {
    files: HashMap<String, AsarNode>,
}

/// ASAR 打包 CLI
#[derive(Parser)]
#[command(name = "rust-asar", version = "0.1.0", about = "ASAR 打包工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// 将目录打包为 ASAR 文件
    Pack {
        /// 源目录路径
        source: PathBuf,
        /// 输出 ASAR 文件路径
        output: PathBuf,
        /// 不打包的文件模式（可重复指定）
        #[arg(long = "unpack", default_values = &[""])]
        unpack: Vec<String>,
    },
}

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Pack {
            source,
            output,
            unpack,
        } => {
            let patterns: Vec<String> = unpack.into_iter().filter(|p| !p.is_empty()).collect();
            if let Err(e) = pack_directory(&source, &output, &patterns) {
                error!("ASAR 打包失败: {}", e);
                std::process::exit(1);
            }
            info!("已创建 ASAR: {}", output.display());
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// ASAR 打包逻辑
// ═══════════════════════════════════════════════════════════════

/// 在文件树中确保路径存在
fn ensure_path<'a>(
    files: &'a mut HashMap<String, AsarNode>,
    path: &str,
) -> &'a mut HashMap<String, AsarNode> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return files;
    }

    // 插入所有中间目录
    let mut current: *mut HashMap<String, AsarNode> = files;
    unsafe {
        for (i, part) in parts.iter().enumerate() {
            let map = &mut *current;
            if !map.contains_key(*part) {
                map.insert(
                    part.to_string(),
                    AsarNode {
                        offset: None,
                        size: 0,
                        unpacked: None,
                        files: if i < parts.len() - 1 {
                            Some(HashMap::new())
                        } else {
                            None
                        },
                    },
                );
            }
            if i < parts.len() - 1 {
                let node = map.get_mut(*part).expect("value should exist");
                if node.files.is_none() {
                    node.files = Some(HashMap::new());
                }
                current = node.files.as_mut().expect("value should exist");
            }
        }
    }

    // 返回最后一个父级
    if parts.len() <= 1 {
        return files;
    }
    let parents = &parts[..parts.len() - 1];
    let mut cur = files;
    for p in parents {
        cur = cur.get_mut(*p).expect("value should exist").files.as_mut().expect("value should exist");
    }
    cur
}

/// 递归分配 offset
fn assign_offsets(files: &mut HashMap<String, AsarNode>, base: &mut u64) {
    let mut names: Vec<String> = files.keys().cloned().collect();
    names.sort();
    for name in names {
        let node = files.get_mut(&name).expect("value should exist");
        if node.files.is_some() {
            if let Some(ref mut child_files) = node.files {
                assign_offsets(child_files, base);
            }
        } else if node.unpacked != Some(true) {
            node.offset = Some(base.to_string());
            *base += node.size;
        }
    }
}

/// 检查是否有 unpacked 节点
fn has_unpacked(files: &HashMap<String, AsarNode>) -> bool {
    for node in files.values() {
        if node.unpacked == Some(true) {
            return true;
        }
        if let Some(ref f) = node.files {
            if has_unpacked(f) {
                return true;
            }
        }
    }
    false
}

/// 打包目录为 ASAR 文件
fn pack_directory(
    source_dir: &Path,
    output_path: &Path,
    unpack_patterns: &[String],
) -> Result<(), String> {
    if !source_dir.is_dir() {
        return Err(format!("源路径不是目录: {}", source_dir.display()));
    }

    let mut root: HashMap<String, AsarNode> = HashMap::new();
    let mut file_entries: Vec<(String, PathBuf)> = Vec::new(); // (asar_path, fs_path)

    // 遍历目录构建文件树
    for entry in WalkDir::new(source_dir).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
        let entry = entry.map_err(|e| format!("读取目录失败: {}", e))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .map_err(|e| format!("路径前缀错误: {}", e))?;
        let relative_str = relative.to_str().ok_or_else(|| {
            format!("无效 UTF-8 路径: {}", relative.display())
        })?;
        if relative_str.is_empty() {
            continue;
        }

        if entry.file_type().is_dir() {
            ensure_path(&mut root, relative_str);
        } else if entry.file_type().is_file() {
            let is_unpacked = unpack_patterns.iter().any(|p| {
                glob::Pattern::new(p).ok()
                    .map(|pat| pat.matches(relative_str))
                    .unwrap_or(false)
            });
            let meta = entry.metadata().map_err(|e| format!("metadata: {}", e))?;
            let size = meta.len();

            // 插入到父目录
            let parent = if let Some(pos) = relative_str.rfind('/') {
                ensure_path(&mut root, &relative_str[..pos]);
                let parts: Vec<&str> = relative_str.split('/').collect();
                let parents = &parts[..parts.len() - 1];
                let mut cur = &mut root;
                for p in parents {
                    cur = cur.get_mut(*p).expect("value should exist").files.as_mut().expect("value should exist");
                }
                cur
            } else {
                &mut root
            };

            let fname = relative_str.rsplit('/').next().unwrap_or(relative_str);
            parent.insert(
                fname.to_string(),
                AsarNode {
                    offset: None,
                    size,
                    unpacked: if is_unpacked { Some(true) } else { None },
                    files: None,
                },
            );

            if !is_unpacked {
                file_entries.push((relative_str.to_string(), path.to_path_buf()));
            }
        }
    }

    // 分配 offsets
    let mut base = 0u64;
    assign_offsets(&mut root, &mut base);

    // 写入 ASAR
    let mut output = fs::File::create(output_path)
        .map_err(|e| format!("创建输出文件失败: {}", e))?;

    // 写入文件数据段
    for fs_path in file_entries.iter().map(|(_, p)| p) {
        let data = fs::read(fs_path)
            .map_err(|e| format!("读取 {} 失败: {}", fs_path.display(), e))?;
        output.write_all(&data).map_err(|e| format!("写入数据失败: {}", e))?;
    }

    // 检查 unpacked
    let needs_unpacked = has_unpacked(&root);

    // 写入 JSON header
    let header = AsarRoot { files: root };
    let header_json = serde_json::to_vec(&header)
        .map_err(|e| format!("序列化 header 失败: {}", e))?;
    output.write_all(&header_json)
        .map_err(|e| format!("写入 header 失败: {}", e))?;

    // 写入 header 大小
    let hs = header_json.len() as u32;
    output.write_all(&hs.to_le_bytes())
        .map_err(|e| format!("写入 header 大小失败: {}", e))?;

    // 复制 unpacked 文件
    if needs_unpacked {
        let unpacked_dir = format!("{}.unpacked", output_path.display());
        let unpacked_path = Path::new(&unpacked_dir);

        for entry in WalkDir::new(source_dir).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
            let entry = entry.map_err(|e| format!("遍历 unpacked 失败: {}", e))?;
            let path = entry.path();
            let relative = path.strip_prefix(source_dir).map_err(|_| "prefix")?;
            let rel_str = relative.to_str().ok_or("utf8")?;
            if rel_str.is_empty() {
                continue;
            }

            let needs = unpack_patterns.iter().any(|p| {
                glob::Pattern::new(p).ok()
                    .map(|pat| pat.matches(rel_str))
                    .unwrap_or(false)
            });

            if entry.file_type().is_file() && needs {
                let dest = unpacked_path.join(rel_str);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("创建 unpacked 目录失败: {}", e))?;
                }
                fs::copy(path, &dest)
                    .map_err(|e| format!("复制 unpacked 文件失败: {}", e))?;
            }
        }
    }

    Ok(())
}
