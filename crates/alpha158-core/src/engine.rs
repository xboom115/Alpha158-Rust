//! 并行计算引擎 — Rayon 按股票并行, 每只股票串行计算全部因子

use alpha158_data::reader::StockData;
use alpha158_data::schema::FACTOR_NAMES;
use alpha158_data::stock::StockOutput;
use alpha158_factors::compute_all_factors;
use alpha158_memory::acquire_scratch;
use rayon::prelude::*;

/// 计算全部股票的 Alpha158 因子 (并行)
///
/// 使用 Rayon par_iter, 每只股票独立计算, 线程本地 ScratchPad 复用.
pub fn compute_all(stocks: &StockData) -> Vec<StockOutput> {
    let num_stocks = stocks.num_stocks();

    tracing::info!(
        "开始计算: {} 只股票, {} 个因子",
        num_stocks,
        FACTOR_NAMES.len()
    );

    // 按股票并行计算
    let results: Vec<StockOutput> = (0..num_stocks)
        .into_par_iter()
        .map(|i| {
            let slice = stocks.slice(i);
            let n = slice.n;

            // 从线程本地池获取 ScratchPad
            let mut scratch = acquire_scratch(n);

            // 计算全部因子
            compute_all_factors(&slice, &mut scratch);

            // 提取输出
            let output = StockOutput {
                code: slice.code.to_string(),
                dates: slice.dates.to_vec(),
                data: scratch.output.clone(),
            };

            output
        })
        .collect();

    tracing::info!("计算完成: {} 只股票", results.len());
    results
}
