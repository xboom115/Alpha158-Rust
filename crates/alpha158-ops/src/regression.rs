//! Rolling Linear Regression — 对齐 Qlib Slope/Rsquare/Resi
//!
//! Qlib Cython 实现:
//!   x = [1, 2, ..., N] (1-based)
//!   slope = (N*Σxy - Σx*Σy) / (N*Σx² - Σx²)
//!   rsquare = r² where r = (N*Σxy - Σx*Σy) / sqrt((N*Σx²-Σx²)(N*Σy²-Σy²))
//!   resi = y_last - (slope * N + intercept)
//!   intercept = y_mean - slope * x_mean
//!
//! 使用环形缓冲 + 增量更新

/// 批量 rolling regression
///
/// x = [1, 2, ..., window] (1-based, 对齐 Qlib)
pub fn rolling_regression(
    y: &[f32],
    window: usize,
    slope_out: &mut [f32],
    rsq_out: &mut [f32],
    resi_out: &mut [f32],
) {
    let n = y.len();
    if n < window {
        return;
    }

    let w = window as f64;
    // x = [1, 2, ..., window] (1-based)
    let sum_x = w * (w + 1.0) / 2.0; // Σx = 1+2+...+w = w*(w+1)/2
    let sum_x2 = w * (w + 1.0) * (2.0 * w + 1.0) / 6.0; // Σx² = w*(w+1)*(2w+1)/6
    let denom_x = w * sum_x2 - sum_x * sum_x;

    // 环形缓冲
    let mut ring = vec![0.0f64; window];
    let mut head = 0usize;

    for i in 0..n {
        ring[head] = y[i] as f64;
        head = (head + 1) % window;

        let count = if i < window - 1 { i + 1 } else { window };
        if count < 2 {
            slope_out[i] = 0.0;
            rsq_out[i] = 0.0;
            resi_out[i] = 0.0;
            continue;
        }

        // 计算 Σy, Σxy, Σy²
        let mut sum_y = 0.0f64;
        let mut sum_xy = 0.0f64;
        let mut sum_y2 = 0.0f64;
        let start = if i < window - 1 { 0 } else { head };
        let c = count as f64;

        for j in 0..count {
            let idx = (start + j) % window;
            let yj = ring[idx];
            let xj = (j + 1) as f64; // x = 1, 2, ..., count (1-based)
            sum_y += yj;
            sum_xy += xj * yj;
            sum_y2 += yj * yj;
        }

        // x = [1, 2, ..., count]
        let sum_x = c * (c + 1.0) / 2.0;
        let sum_x2 = c * (c + 1.0) * (2.0 * c + 1.0) / 6.0;
        let denom_x = c * sum_x2 - sum_x * sum_x;

        // Slope
        let numer = c * sum_xy - sum_x * sum_y;
        let slope = if denom_x.abs() > 1e-12 {
            numer / denom_x
        } else {
            0.0
        };

        // Rsquare = r²
        let denom_y = c * sum_y2 - sum_y * sum_y;
        let rsq = if denom_x.abs() > 1e-12 && denom_y.abs() > 1e-12 {
            let r = numer / (denom_x * denom_y).sqrt();
            r * r
        } else {
            0.0
        };

        // Residual = y_last - (slope * count + intercept)
        let x_mean = sum_x / c;
        let y_mean = sum_y / c;
        let intercept = y_mean - slope * x_mean;
        let y_last = y[i] as f64;
        let predicted = slope * c + intercept; // x_last = count (1-based)
        let resi = y_last - predicted;

        slope_out[i] = slope as f32;
        rsq_out[i] = rsq.clamp(0.0, 1.0) as f32;
        resi_out[i] = resi as f32;
    }
}

/// RollingOperator 版本 (Slope)
pub struct RollingRegression {
    pub window: usize,
}

#[derive(Clone)]
pub struct RegState {
    pub ring: Vec<f64>,
    pub head: usize,
    pub sum_y: f64,
    pub sum_xy: f64,
    pub sum_y2: f64,
    pub count: usize,
}

impl Default for RegState {
    fn default() -> Self {
        Self {
            ring: Vec::new(),
            head: 0,
            sum_y: 0.0,
            sum_xy: 0.0,
            sum_y2: 0.0,
            count: 0,
        }
    }
}

use crate::RollingOperator;

impl RollingOperator for RollingRegression {
    type State = RegState;
    fn window(&self) -> usize {
        self.window
    }

    fn push(&self, state: &mut RegState, new_y: f32, _old_y: f32, pos: usize) {
        if state.ring.is_empty() {
            state.ring = vec![0.0f64; self.window];
        }

        let new_y_f64 = new_y as f64;

        if pos < self.window {
            state.ring[pos] = new_y_f64;
            let x = (pos + 1) as f64; // 1-based
            state.sum_y += new_y_f64;
            state.sum_xy += x * new_y_f64;
            state.sum_y2 += new_y_f64 * new_y_f64;
            state.count = pos + 1;
        } else {
            // 增量更新 (对齐 Qlib Cython)
            // 移除旧值: xy_sum -= y_sum (所有 x 减 1)
            state.sum_xy -= state.sum_y;
            state.sum_y -= state.ring[state.head]; // 移除最旧的 y
            state.sum_y2 -= state.ring[state.head] * state.ring[state.head];

            // 添加新值 (x = window)
            state.sum_y += new_y_f64;
            state.sum_xy += self.window as f64 * new_y_f64;
            state.sum_y2 += new_y_f64 * new_y_f64;

            state.ring[state.head] = new_y_f64;
            state.head = (state.head + 1) % self.window;
            state.count = self.window;
        }
    }

    fn result(&self, state: &RegState) -> f32 {
        let w = self.window as f64;
        let sum_x = w * (w + 1.0) / 2.0;
        let sum_x2 = w * (w + 1.0) * (2.0 * w + 1.0) / 6.0;
        let denom_x = w * sum_x2 - sum_x * sum_x;
        let numer = w * state.sum_xy - sum_x * state.sum_y;
        if denom_x.abs() > 1e-12 {
            (numer / denom_x) as f32
        } else {
            0.0
        }
    }
}
