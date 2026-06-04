//! bspatch CLI — 二进制补丁应用工具
//!
//! 兼容命令：
//!   bspatch <old_file> <new_file> <patch_file>
//!
//! 自动检测补丁是否为 zstd 压缩（通过检查前 4 字节 magic number）。

use clap::Parser;
use std::io::Cursor;
use std::path::PathBuf;
use tracing::error;

/// 二进制补丁应用工具
#[derive(Parser)]
#[command(name = "bspatch", version = "0.1.0", about = "应用二进制差分补丁")]
struct Cli {
    /// 旧文件路径
    old: PathBuf,
    /// 新文件（输出）路径
    new: PathBuf,
    /// 补丁文件路径
    patch: PathBuf,
}

/// zstd magic number: 0xFD2FB528
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    if let Err(e) = run_bspatch(&cli.old, &cli.new, &cli.patch) {
        error!("bspatch 失败: {}", e);
        std::process::exit(1);
    }
}

fn run_bspatch(old: &PathBuf, new: &PathBuf, patch: &PathBuf) -> Result<(), String> {
    let old_data = std::fs::read(old).map_err(|e| format!("读取旧文件失败: {}", e))?;
    let patch_data = std::fs::read(patch).map_err(|e| format!("读取补丁文件失败: {}", e))?;

    // 检测是否为 zstd 压缩补丁
    let decompressed = if patch_data.len() > 4 && patch_data[..4] == ZSTD_MAGIC {
        zstd::decode_all(&patch_data[..]).map_err(|e| format!("解压缩补丁失败: {}", e))?
    } else {
        patch_data
    };

    let mut output = Vec::new();
    let mut patch_cursor = Cursor::new(decompressed);
    bsdiff::patch(&old_data, &mut patch_cursor, &mut output)
        .map_err(|e| format!("应用补丁失败: {}", e))?;

    std::fs::write(new, &output).map_err(|e| format!("写入新文件失败: {}", e))?;

    Ok(())
}
