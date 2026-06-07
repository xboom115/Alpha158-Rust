//! Rolling Std — 前缀和法, O(n), 数值稳定
//!
//! 对齐 Qlib: ddof=1 (样本标准差) + min_periods=1

use crate::RollingOperator;

pub struct RollingStd {
    pub window: usize,
}

#[derive(Clone)]
pub struct WelfordState {
    pub mean: f32,
    pub m2: f32,
}

impl Default for WelfordState {
    fn default() -> Self {
        Self {
            mean: 0.0,
            m2: 0.0,
        }
    }
}

impl RollingOperator for RollingStd {
    type State = WelfordState;
    fn window(&self) -> usize {
        self.window
    }

    #[inline]
    fn push(&self, state: &mut WelfordState, new_val: f32, old_val: f32, pos: usize) {
        let w = self.window as f32;
        if pos < self.window {
            let n = (pos + 1) as f32;
            let delta = new_val - state.mean;
            state.mean += delta / n;
            let delta2 = new_val - state.mean;
            state.m2 += delta * delta2;
        } else {
            let delta_old = old_val - state.mean;
            state.mean -= delta_old / w;
            let delta_old2 = old_val - state.mean;
            state.m2 -= delta_old * delta_old2;
            let delta_new = new_val - state.mean;
            state.mean += delta_new / w;
            let delta_new2 = new_val - state.mean;
            state.m2 += delta_new * delta_new2;
        }
    }

    #[inline]
    fn result(&self, state: &WelfordState) -> f32 {
        (state.m2.max(0.0) / (self.window as f32 - 1.0)).sqrt()
    }
}

/// 批量 rolling std, O(n)
///
/// 对齐 Qlib: ddof=1 + min_periods=1
pub fn rolling_std(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
    if n == 0 {
        return;
    }

    let mut cumsum = vec![0.0f64; n + 1];
    let mut cumsum_sq = vec![0.0f64; n + 1];
    for i in 0..n {
        let x = data[i] as f64;
        cumsum[i + 1] = cumsum[i] + x;
        cumsum_sq[i + 1] = cumsum_sq[i] + x * x;
    }

    for i in 0..n {
        let count = if i < window { i + 1 } else { window };
        let start = if i < window { 0 } else { i + 1 - window };
        let sum_x = cumsum[i + 1] - cumsum[start];
        let sum_x2 = cumsum_sq[i + 1] - cumsum_sq[start];
        let c = count as f64;
        if count < 2 {
            out[i] = 0.0;
        } else {
            let var = (sum_x2 - sum_x * sum_x / c) / (c - 1.0);
            out[i] = var.max(0.0).sqrt() as f32;
        }
    }
}
