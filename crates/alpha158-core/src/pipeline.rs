//! 完整计算流水线: 读取 → 计算 → 写入

use alpha158_data::reader::read_parquet;
use alpha158_data::stock::StockOutput;
use alpha158_data::writer::write_parquet;
use crate::engine::compute_all;

/// 流水线配置
pub struct PipelineConfig {
    /// 只输出最近 N 天 (0 = 全部)
    pub stock_day: usize,
    /// 日期输出为字符串
    pub date_as_string: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            stock_day: 0,
            date_as_string: false,
        }
    }
}

/// 运行完整的 Alpha158 计算流水线
pub fn run(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    run_full(input_path, output_path, &PipelineConfig::default())
}

/// 运行完整的 Alpha158 计算流水线 (完整配置)
pub fn run_full(
    input_path: &str,
    output_path: &str,
    config: &PipelineConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let t0 = std::time::Instant::now();

    // ── 计算需要读取的天数 ──
    let max_days = if config.stock_day > 0 {
        let needed = config.stock_day + 60; // 60 = 最大滚动窗口
        tracing::info!(
            "StockDay={}, 需要读取最近 {} 天数据 (含 60 天滚动窗口回溯)",
            config.stock_day, needed
        );
        Some(needed)
    } else {
        None
    };

    // ── Step 1: 读取 Parquet ──
    tracing::info!("读取数据: {}", input_path);
    let stocks = read_parquet(input_path, max_days)?;
    tracing::info!(
        "数据加载完成: {} 只股票, 每只 {} 天, 耗时 {:.2}s",
        stocks.num_stocks(),
        if stocks.num_stocks() > 0 { stocks.dates[0].len() } else { 0 },
        t0.elapsed().as_secs_f64()
    );

    // ── Step 2: 并行计算 Alpha158 因子 ──
    let t1 = std::time::Instant::now();
    let outputs = compute_all(&stocks);
    tracing::info!(
        "因子计算完成: {} 只股票, 耗时 {:.2}s",
        outputs.len(),
        t1.elapsed().as_secs_f64()
    );

    // ── Step 3: 截取最近 stock_day 天 ──
    let final_outputs = if config.stock_day > 0 {
        tracing::info!("截取最近 {} 天数据", config.stock_day);
        slice_outputs(outputs, config.stock_day)
    } else {
        outputs
    };

    // ── Step 4: 写入 Parquet ──
    let t2 = std::time::Instant::now();
    tracing::info!("写入结果: {}", output_path);
    write_parquet(&final_outputs, output_path, config.date_as_string)?;
    tracing::info!("写入完成, 耗时 {:.2}s", t2.elapsed().as_secs_f64());

    tracing::info!("总耗时: {:.2}s", t0.elapsed().as_secs_f64());
    Ok(())
}

/// 截取每只股票的最后 N 天数据
fn slice_outputs(outputs: Vec<StockOutput>, last_n: usize) -> Vec<StockOutput> {
    outputs
        .into_iter()
        .map(|mut output| {
            let total = output.dates.len();
            if total > last_n {
                let skip = total - last_n;
                output.dates = output.dates[skip..].to_vec();
                for col in &mut output.data {
                    *col = col[skip..].to_vec();
                }
            }
            output
        })
        .collect()
}
