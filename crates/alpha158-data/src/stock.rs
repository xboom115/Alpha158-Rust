//! StockSlice — 每只股票的列式数据切片

/// 每只股票的原始数据切片 — 列式, 零拷贝引用
#[derive(Clone)]
pub struct StockSlice<'a> {
    pub code: &'a str,
    pub n: usize,
    pub dates: &'a [i32],
    pub open: &'a [f32],
    pub high: &'a [f32],
    pub low: &'a [f32],
    pub close: &'a [f32],
    pub volume: &'a [f32],
    pub vwap: &'a [f32],
}

/// 股票输出 — 158 个因子列
pub struct StockOutput {
    pub code: String,
    pub dates: Vec<i32>,
    pub data: Vec<Vec<f32>>, // 158 列, 每列 Vec<f32>
}
