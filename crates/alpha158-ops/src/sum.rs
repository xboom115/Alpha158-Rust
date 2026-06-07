//! Rolling Sum — Cumulative Sum, O(n)
//!
//! 对齐 Qlib min_periods=1, 使用 f64 累加

use crate::RollingOperator;

pub struct RollingSum {
    pub window: usize,
}

#[derive(Default, Clone)]
pub struct SumState {
    pub sum: f32,
}

impl RollingOperator for RollingSum {
    type State = SumState;
    fn window(&self) -> usize {
        self.window
    }

    #[inline]
    fn push(&self, state: &mut SumState, new_val: f32, old_val: f32, _pos: usize) {
        state.sum += new_val - old_val;
    }

    #[inline]
    fn result(&self, state: &SumState) -> f32 {
        state.sum
    }
}

/// 批量 rolling sum, O(n)
///
/// 使用 f64 前缀和, 对齐 Qlib min_periods=1
pub fn rolling_sum(data: &[f32], window: usize, out: &mut [f32]) {
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
            out[i] = cumsum[i + 1] as f32;
        } else {
            out[i] = (cumsum[i + 1] - cumsum[i + 1 - window]) as f32;
        }
    }
}
