//! rust-zstd CLI — 使用 Rust zstd crate 实现压缩/解压
//!
//! 兼容命令：
//!   rust-zstd compress -i <input> -o <output> [--threads max]
//!   rust-zstd decompress -i <input> -o <output> [--no-timing]

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rust-zstd", version = "0.1.0", about = "zstd 压缩/解压工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 压缩文件
    Compress {
        /// 输入文件路径
        #[arg(short = 'i')]
        input: PathBuf,
        /// 输出文件路径
        #[arg(short = 'o')]
        output: PathBuf,
        /// 线程数（max 表示使用所有核心）
        #[arg(long = "threads", default_value = "1")]
        threads: String,
    },
    /// 解压文件
    Decompress {
        /// 输入文件路径
        #[arg(short = 'i')]
        input: PathBuf,
        /// 输出文件路径
        #[arg(short = 'o')]
        output: PathBuf,
        /// 忽略计时输出（兼容选项，本实现默认无输出）
        #[arg(long = "no-timing")]
        no_timing: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress {
            input,
            output,
            threads,
        } => {
            if let Err(e) = compress(&input, &output, &threads) {
                eprintln!("压缩失败: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Decompress {
            input,
            output,
            no_timing: _,
        } => {
            if let Err(e) = decompress(&input, &output) {
                eprintln!("解压失败: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn compress(input: &PathBuf, output: &PathBuf, threads: &str) -> Result<(), String> {
    let data = std::fs::read(input).map_err(|e| format!("读取输入文件失败: {}", e))?;

    let num_threads = if threads == "max" {
        0 // 0 表示自动检测
    } else {
        threads
            .parse::<u32>()
            .map_err(|_| format!("无效的线程数: {}", threads))?
    };

    // zstd 的 Encoder 支持 multithread 方法（需 zstdmt 特性）
    let mut encoder = zstd::Encoder::new(Vec::new(), 3)
        .map_err(|e| format!("创建 zstd encoder 失败: {}", e))?;

    if num_threads == 0 {
        // 0 = 自动检测 CPU 核心数
        encoder
            .multithread(0u32)
            .map_err(|e| format!("设置多线程失败: {}", e))?;
    } else if num_threads > 1 {
        encoder
            .multithread(num_threads)
            .map_err(|e| format!("设置多线程失败: {}", e))?;
    }

    // 写入数据
    use std::io::Write;
    encoder
        .write_all(&data)
        .map_err(|e| format!("压缩数据失败: {}", e))?;

    let compressed = encoder
        .finish()
        .map_err(|e| format!("完成压缩失败: {}", e))?;

    std::fs::write(output, &compressed)
        .map_err(|e| format!("写入输出文件失败: {}", e))?;

    Ok(())
}

fn decompress(input: &PathBuf, output: &PathBuf) -> Result<(), String> {
    let data = std::fs::read(input).map_err(|e| format!("读取输入文件失败: {}", e))?;

    let mut decoder = zstd::Decoder::new(&data[..])
        .map_err(|e| format!("创建 zstd decoder 失败: {}", e))?;

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("解压数据失败: {}", e))?;

    std::fs::write(output, &decompressed)
        .map_err(|e| format!("写入输出文件失败: {}", e))?;

    Ok(())
}
