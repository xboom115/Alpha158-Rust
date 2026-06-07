//! 增量计算状态 — 支持 T+1 更新

use crate::window_cache::WindowBuffer;

/// 每只股票的增量计算状态
pub struct IncrementalState {
    pub code: String,
    pub last_date: i32,
    /// 原始数据环形缓冲 (容量 = max_window = 60)
    pub open: WindowBuffer,
    pub high: WindowBuffer,
    pub low: WindowBuffer,
    pub close: WindowBuffer,
    pub volume: WindowBuffer,
    pub vwap: WindowBuffer,
}

impl IncrementalState {
    pub fn new(code: String, max_window: usize) -> Self {
        Self {
            code,
            last_date: 0,
            open: WindowBuffer::new(max_window),
            high: WindowBuffer::new(max_window),
            low: WindowBuffer::new(max_window),
            close: WindowBuffer::new(max_window),
            volume: WindowBuffer::new(max_window),
            vwap: WindowBuffer::new(max_window),
        }
    }

    /// 推入新的一天数据
    pub fn push_day(
        &mut self,
        date: i32,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        vwap: f32,
    ) {
        self.last_date = date;
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
        self.volume.push(volume);
        self.vwap.push(vwap);
    }

    /// 从缓冲区构建 StockSlice (用于全量重算)
    pub fn to_arrays(&self) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
        (
            self.open.as_slice(),
            self.high.as_slice(),
            self.low.as_slice(),
            self.close.as_slice(),
            self.volume.as_slice(),
            self.vwap.as_slice(),
        )
    }
}
