//! Alpha158 因子实现 — 全部 158 个因子的计算
//!
//! 设计: 单个 `compute_all_factors` 函数, 在一只股票的数据上串行计算全部因子.
//! 中间变量只计算一次, 被多个因子共享复用.

mod compute;

pub use compute::compute_all_factors;
