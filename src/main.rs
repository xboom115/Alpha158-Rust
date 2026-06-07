//! Alpha158 CLI — 命令行入口

use alpha158_core::pipeline::PipelineConfig;
use clap::Parser;

#[derive(Parser)]
#[command(name = "alpha158", about = "Alpha158 因子计算引擎 (Rust)")]
struct Cli {
    /// 输入 Parquet 文件路径
    #[arg(short, long)]
    input: String,

    /// 输出 Parquet 文件路径
    #[arg(short, long, default_value = "alpha158_output.parquet")]
    output: String,

    /// 线程数 (0 = 自动检测)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// 输出最近 N 个交易日 (0 = 全部)
    /// 实际读取天数 = N + 60 (因子计算所需滚动窗口)
    #[arg(long, default_value_t = 0)]
    stock_day: usize,

    /// 日期输出为 "YYYY-MM-DD" 字符串 (兼容 ParquetViewer)
    #[arg(long, default_value_t = false)]
    date_as_string: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .init();

    let cli = Cli::parse();

    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()?;
        tracing::info!("线程数: {}", cli.threads);
    } else {
        tracing::info!("线程数: 自动检测");
    }

    let config = PipelineConfig {
        stock_day: cli.stock_day,
        date_as_string: cli.date_as_string,
    };

    alpha158_core::pipeline::run_full(&cli.input, &cli.output, &config)?;
    Ok(())
}
