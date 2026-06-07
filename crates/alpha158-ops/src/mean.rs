//! Rolling Mean — Cumulative Sum, O(n)
//!
//! 对齐 Qlib min_periods=1, 使用 f64 累加避免精度损失

use crate::RollingOperator;

pub struct RollingMean {
    pub window: usize,
}

#[derive(Default, Clone)]
pub struct MeanState {
    pub sum: f32,
}

impl RollingOperator for RollingMean {
    type State = MeanState;
    fn window(&self) -> usize {
        self.window
    }

    #[inline]
    fn push(&self, state: &mut MeanState, new_val: f32, old_val: f32, _pos: usize) {
        state.sum += new_val - old_val;
    }

    #[inline]
    fn result(&self, state: &MeanState) -> f32 {
        state.sum / self.window as f32
    }
}

/// 批量 rolling mean, O(n)
///
/// 使用 f64 前缀和避免 window=60 时的精度损失
pub fn rolling_mean(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
    if n == 0 {
        return;
    }
    let mut cumsum = vec![0.0f64; n + 1];
    let mut s = 0.0f64;
    for i in 0..n {
        s += data[i] as f64;
        cumsum[i + 1] = s;
    }
    for i in 0..n {
        if i < window {
            out[i] = (cumsum[i + 1] / (i + 1) as f64) as f32;
        } else {
            out[i] = ((cumsum[i + 1] - cumsum[i + 1 - window]) / window as f64) as f32;
        }
    }
}
