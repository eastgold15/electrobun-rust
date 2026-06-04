//! ASAR 格式实现 — 打包和读取
//!
//! 格式：`[文件数据拼接][JSON header][4字节header大小(uint32 LE)]`
//! header 是 JSON 对象，包含文件树结构，每个文件记录在数据段中的 offset 和 size。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::c_char;
use std::path::Path;
use walkdir::WalkDir;

/// ASAR 节点 — 可以是文件或目录
/// - 有 `files` → 目录
/// - 有 `offset` → 文件（offset 是数据段内的偏移量，字符串类型）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AsarNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unpacked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<HashMap<String, AsarNode>>,
}

/// ASAR 顶层 header
#[derive(Debug, Serialize, Deserialize)]
pub struct AsarRoot {
    pub files: HashMap<String, AsarNode>,
}

/// 打开的 ASAR 归档
pub struct AsarArchive {
    file: File,
    header: AsarRoot,
    _header_size: u64,
}

/// # Errors
///
/// 当以下情况发生时返回错误：
/// 1. 无法打开指定路径的文件
/// 2. 无法获取文件元数据或文件大小过小
/// 3. 文件指针定位（seek）失败
/// 4. 无法读取文件数据
/// 5. ASAR header 大小无效或超出文件范围
/// 6. 解析 JSON 格式的 header 失败
pub fn open_asar(path: &Path) -> Result<AsarArchive, String> {
    let mut file = File::open(path).map_err(|e| format!("无法打开 ASAR 文件: {}", e))?;
    let file_len = file
        .metadata()
        .map_err(|e| format!("无法获取文件大小: {}", e))?
        .len();

    if file_len < 8 {
        return Err("ASAR 文件太小".to_string());
    }

    // 读取末尾 4 字节 header 大小
    let mut size_buf = [0u8; 8];
    file.seek(SeekFrom::End(-4))
        .map_err(|e| format!("seek 失败: {}", e))?;
    file.read_exact(&mut size_buf[..4])
        .map_err(|e| format!("读取 header 大小失败: {}", e))?;

    let header_size = u32::from_le_bytes(
        size_buf[..4]
            .try_into()
            .expect("size_buf[..4] should always be 4 bytes"),
    ) as u64;

    // 如果最高位为 1，实际用 8 字节
    let (real_header_size, pad) = if (header_size & 0x8000_0000) != 0 {
        let _high = header_size & 0x7fff_ffff;
        file.seek(SeekFrom::End(-8))
            .map_err(|e| format!("seek 8字节失败: {}", e))?;
        file.read_exact(&mut size_buf)
            .map_err(|e| format!("读取 8 字节 header 大小失败: {}", e))?;
        (u64::from_le_bytes(size_buf), 8)
    } else {
        (header_size, 4)
    };

    if real_header_size == 0 || real_header_size > file_len - pad {
        return Err("header 大小无效".to_string());
    }

    let header_offset = file_len - real_header_size - pad;

    // 读取 JSON header
    let mut header_json = vec![0u8; real_header_size as usize];
    file.seek(SeekFrom::Start(header_offset))
        .map_err(|e| format!("seek header 失败: {}", e))?;
    file.read_exact(&mut header_json)
        .map_err(|e| format!("读取 header 失败: {}", e))?;

    let header: AsarRoot = serde_json::from_slice(&header_json)
        .map_err(|e| format!("解析 ASAR header JSON 失败: {}", e))?;

    Ok(AsarArchive {
        file,
        header,
        _header_size: real_header_size,
    })
}

/// 从 ASAR 归档中读取指定文件
///
/// # 错误
///
/// 返回 `Err(String)` — 如果文件不存在、是目录模式、或 offset/数据读取失败。
pub fn read_file(archive: &mut AsarArchive, filename: &str) -> Result<Vec<u8>, String> {
    let entry = resolve_path(&archive.header.files, filename)?;

    if entry.files.is_some() {
        return Err(format!("'{}' 是目录", filename));
    }

    if entry.unpacked == Some(true) {
        return Err(format!("'{}' 是 unpacked 模式", filename));
    }

    let offset: u64 = entry
        .offset
        .as_ref()
        .ok_or_else(|| format!("'{}' 没有 offset", filename))?
        .parse()
        .map_err(|_| format!("offset 格式无效: {:?}", entry.offset))?;

    let size = entry.size as usize;
    let mut buf = vec![0u8; size];

    archive
        .file
        .seek(SeekFrom::Start(offset))
        .map_err(|e| format!("seek 失败: {}", e))?;
    archive
        .file
        .read_exact(&mut buf)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    Ok(buf)
}

/// 在文件树中按 '/' 分隔路径查找节点
fn resolve_path<'a>(
    files: &'a HashMap<String, AsarNode>,
    path: &str,
) -> Result<&'a AsarNode, String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = files;

    for (i, part) in parts.iter().enumerate() {
        let node = current
            .get(*part)
            .ok_or_else(|| format!("路径 '{}' 在 '{}' 处未找到", path, part))?;

        if i == parts.len() - 1 {
            return Ok(node);
        }

        // 中间节点必须是目录
        current = node
            .files
            .as_ref()
            .ok_or_else(|| format!("'{}' 不是目录", part))?;
    }

    Err("空路径".to_string())
}

/// 递归计算所有 packed 文件的总大小
#[allow(dead_code)]
fn calc_total_size(files: &HashMap<String, AsarNode>) -> u64 {
    let mut total = 0u64;
    for node in files.values() {
        if let Some(child_files) = &node.files {
            total += calc_total_size(child_files);
        } else if node.unpacked != Some(true) {
            total += node.size;
        }
    }
    total
}

/// 递归分配 offset，按键排序保证确定性
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

/// 在 ASAR 文件树中插入路径，构造树结构
fn ensure_path<'a>(
    files: &'a mut HashMap<String, AsarNode>,
    path: &str,
) -> &'a mut HashMap<String, AsarNode> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
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
                let node = map.get_mut(*part).expect("key was just verified");
                if node.files.is_none() {
                    node.files = Some(HashMap::new());
                }
                current = node.files.as_mut().expect("files was set to Some earlier");
            }
        }
        // 返回最后一个父节点所在的 map
        if parts.len() <= 1 {
            return files;
        }
        let last_parent = &parts[..parts.len() - 1];
        let mut cur = files;
        for p in last_parent {
            cur = cur
                .get_mut(*p)
                .expect("value should exist")
                .files
                .as_mut()
                .expect("files was set to Some earlier");
        }
        cur
    }
}

/// 检查文件树中是否有 unpacked 节点
fn has_unpacked_nodes(files: &HashMap<String, AsarNode>) -> bool {
    for node in files.values() {
        if node.unpacked == Some(true) {
            return true;
        }
        if let Some(ref f) = node.files {
            if has_unpacked_nodes(f) {
                return true;
            }
        }
    }
    false
}

/// ASAR 打包：遍历目录写入 ASAR 文件
///
/// # 错误
///
/// 返回 `Err(String)` — 如果源路径不是目录、文件读取失败、或写入输出文件失败。
pub fn pack_directory(
    source_dir: &Path,
    output_path: &Path,
    unpack_patterns: &[String],
) -> Result<(), String> {
    if !source_dir.is_dir() {
        return Err(format!("源路径不是目录: {}", source_dir.display()));
    }

    let mut root: HashMap<String, AsarNode> = HashMap::new();
    let mut file_entries: Vec<(String, std::path::PathBuf)> = Vec::new(); // (asar_path, fs_path)

    // 遍历目录
    for entry in WalkDir::new(source_dir).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
        let entry = entry.map_err(|e| format!("读取目录失败: {}", e))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .map_err(|e| format!("路径前缀错误: {}", e))?;
        let relative_str = relative
            .to_str()
            .ok_or_else(|| format!("无效 UTF-8 路径: {}", relative.display()))?;
        // Windows paths use \ but ASAR always uses /
        let relative_str = relative_str.replace('\\', "/");
        let relative_str = relative_str.as_str(); // ← 加这一行
        if relative_str.is_empty() {
            continue;
        }

        if entry.file_type().is_dir() {
            // 确保目录树中存在
            ensure_path(&mut root, relative_str);
        } else if entry.file_type().is_file() {
            let is_unpacked = unpack_patterns.iter().any(|p| {
                glob::Pattern::new(p)
                    .ok()
                    .map(|pat| pat.matches(relative_str))
                    .unwrap_or(false)
            });
            let meta = entry.metadata().map_err(|e| format!("metadata: {}", e))?;
            let size = meta.len();

            // 在文件树中插入
            let parent = if let Some(pos) = relative_str.rfind('/') {
                ensure_path(&mut root, &relative_str[..pos]);
                let parts: Vec<&str> = relative_str.split('/').collect();
                let parents = &parts[..parts.len() - 1];
                let mut cur = &mut root;
                for p in parents {
                    cur = cur
                        .get_mut(*p)
                        .expect("value should exist")
                        .files
                        .as_mut()
                        .expect("files was set to Some earlier");
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

    // 写入输出
    let mut output = File::create(output_path).map_err(|e| format!("创建输出文件失败: {}", e))?;

    // 写入文件数据段
    for fs_path in file_entries.iter().map(|(_, p)| p) {
        let data =
            fs::read(fs_path).map_err(|e| format!("读取 {} 失败: {}", fs_path.display(), e))?;
        output
            .write_all(&data)
            .map_err(|e| format!("写入数据失败: {}", e))?;
    }

    // 先检查是否有 unpacked 文件（root 在此之后被移动）
    let has_unpacked = has_unpacked_nodes(&root);

    // 写入 JSON header
    let header = AsarRoot { files: root };
    let header_json =
        serde_json::to_vec(&header).map_err(|e| format!("序列化 header 失败: {}", e))?;
    output
        .write_all(&header_json)
        .map_err(|e| format!("写入 header 失败: {}", e))?;

    // 写入 header 大小
    let hs = header_json.len() as u32;
    output
        .write_all(&hs.to_le_bytes())
        .map_err(|e| format!("写入 header 大小失败: {}", e))?;

    if has_unpacked {
        let unpacked_path_str = format!("{}.unpacked", output_path.display());
        let unpacked_path = Path::new(&unpacked_path_str);

        for entry in WalkDir::new(source_dir).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
            let entry = entry.map_err(|e| format!("遍历 unpacked 失败: {}", e))?;
            let path = entry.path();
            let relative = path.strip_prefix(source_dir).map_err(|_| "prefix")?;
            let rel_str = relative.to_str().ok_or("utf8")?;
            if rel_str.is_empty() {
                continue;
            }

            // 检查是否需要 unpack
            let needs_unpack = unpack_patterns.iter().any(|p| {
                glob::Pattern::new(p)
                    .ok()
                    .map(|pat| pat.matches(rel_str))
                    .unwrap_or(false)
            });

            if entry.file_type().is_file() && needs_unpack {
                let dest = unpacked_path.join(rel_str);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("创建 unpacked 目录失败: {}", e))?;
                }
                fs::copy(path, &dest).map_err(|e| format!("复制 unpacked 文件失败: {}", e))?;
            }
        }
    }

    Ok(())
}

// ──────────── C FFI 导出 ────────────

/// 打开 ASAR 文件，返回不透明指针
///
/// # Safety
///
/// `path` 必须是有效的、以 null 结尾的 C 字符串。
pub unsafe extern "C" fn asar_open(path: *const c_char) -> *mut std::ffi::c_void {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(c_str) = CStr::from_ptr(path).to_str() else {
        return std::ptr::null_mut();
    };
    let Ok(archive) = open_asar(Path::new(c_str)) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(archive)) as *mut std::ffi::c_void
}

/// 从 ASAR 中读取文件，成功返回 0，失败返回 -1
///
/// # Safety
///
/// 调用者必须确保 `archive_ptr` 是 `asar_open` 返回的有效指针，
/// `filename` 是有效的 null 结尾 C 字符串，
/// `out_buf` 和 `out_len` 指向可写的内存区域。
pub unsafe extern "C" fn asar_read_file(
    archive_ptr: *mut std::ffi::c_void,
    filename: *const c_char,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if archive_ptr.is_null() || filename.is_null() || out_buf.is_null() || out_len.is_null() {
        return -1;
    }
    let archive = &mut *(archive_ptr as *mut AsarArchive);
    let Ok(c_str) = CStr::from_ptr(filename).to_str() else {
        return -1;
    };
    let Ok(data) = read_file(archive, c_str) else {
        return -1;
    };
    let leaked = data.leak();
    *out_buf = leaked.as_mut_ptr();
    *out_len = leaked.len();
    0
}

/// 释放 `asar_read_file` 分配的缓冲区
///
/// # Safety
///
/// `buf` 必须是 `asar_read_file` 通过 `out_buf` 返回的指针。
pub unsafe extern "C" fn asar_free_buffer(buf: *mut u8, len: usize) {
    if !buf.is_null() && len > 0 {
        let _ = Vec::from_raw_parts(buf, len, len);
    }
}

/// 关闭 ASAR 归档
///
/// # Safety
///
/// `archive_ptr` 必须是 `asar_open` 返回的有效指针，且不可重复关闭。
pub unsafe extern "C" fn asar_close(archive_ptr: *mut std::ffi::c_void) {
    if !archive_ptr.is_null() {
        let _ = Box::from_raw(archive_ptr as *mut AsarArchive);
    }
}
