//! bsdiff CLI — 二进制差分工具
//!
//! 兼容命令：
//!   bsdiff <old_file> <new_file> <patch_file> [--use-zstd]

use clap::Parser;
use std::path::PathBuf;
use tracing::error;

/// 二进制差分工具
#[derive(Parser)]
#[command(name = "bsdiff", version = "0.1.0", about = "生成二进制差分补丁")]
struct Cli {
    /// 旧文件路径
    old: PathBuf,
    /// 新文件路径
    new: PathBuf,
    /// 输出补丁文件路径
    patch: PathBuf,
    /// 使用 zstd 压缩补丁
    #[arg(long = "use-zstd")]
    use_zstd: bool,
}

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    if let Err(e) = run_bsdiff(&cli.old, &cli.new, &cli.patch, cli.use_zstd) {
        error!("bsdiff 失败: {}", e);
        std::process::exit(1);
    }
}

fn run_bsdiff(old: &PathBuf, new: &PathBuf, patch: &PathBuf, use_zstd: bool) -> Result<(), String> {
    let old_data = std::fs::read(old).map_err(|e| format!("读取旧文件失败: {}", e))?;
    let new_data = std::fs::read(new).map_err(|e| format!("读取新文件失败: {}", e))?;

    // 生成 bsdiff 补丁
    let mut patch_data = Vec::new();
    bsdiff::diff(&old_data, &new_data, &mut patch_data)
        .map_err(|e| format!("生成补丁失败: {}", e))?;

    // 如果启用 zstd 压缩
    let final_data = if use_zstd {
        zstd::encode_all(&patch_data[..], 3)
            .map_err(|e| format!("压缩补丁失败: {}", e))?
    } else {
        patch_data
    };

    std::fs::write(patch, &final_data)
        .map_err(|e| format!("写入补丁文件失败: {}", e))?;

    Ok(())
}
