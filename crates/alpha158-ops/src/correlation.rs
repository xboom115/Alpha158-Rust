//! Rolling Correlation — 对齐 Qlib Corr(x, y, d)
//!
//! Qlib 实现: pandas rolling.corr() + 零方差边界处理
//! 当 x 或 y 的滚动 std < 2e-05 时, 结果设为 NaN

use crate::RollingOperator;

pub struct RollingCorr {
    pub window: usize,
}

#[derive(Default, Clone)]
pub struct CorrState {
    pub sx: f32,
    pub sy: f32,
    pub sxx: f32,
    pub syy: f32,
    pub sxy: f32,
}

impl RollingOperator for RollingCorr {
    type State = CorrState;
    fn window(&self) -> usize {
        self.window
    }

    #[inline]
    fn push(&self, state: &mut CorrState, new_x: f32, old_x: f32, pos: usize) {
        // 这个算子需要双变量, 用 compute_batch_corr 代替
        if pos >= self.window {
            state.sx -= old_x;
            state.sxx -= old_x * old_x;
        }
        state.sx += new_x;
        state.sxx += new_x * new_x;
    }

    #[inline]
    fn result(&self, _state: &CorrState) -> f32 {
        0.0
    }
}

/// 批量 rolling correlation, O(n)
///
/// 对齐 Qlib: 当 x 或 y 的滚动 std < 2e-05 时, 结果设为 NaN
pub fn rolling_corr(
    x: &[f32],
    y: &[f32],
    window: usize,
    out: &mut [f32],
) {
    let n = x.len();
    assert_eq!(n, y.len());
    if n == 0 {
        return;
    }

    let mut sx = 0.0f64;
    let mut sy = 0.0f64;
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    let mut sxy = 0.0f64;
    let _w = window as f64;

    for i in 0..n {
        if i >= window {
            let ox = x[i - window] as f64;
            let oy = y[i - window] as f64;
            sx -= ox;
            sy -= oy;
            sxx -= ox * ox;
            syy -= oy * oy;
            sxy -= ox * oy;
        }
        let xi = x[i] as f64;
        let yi = y[i] as f64;
        sx += xi;
        sy += yi;
        sxx += xi * xi;
        syy += yi * yi;
        sxy += xi * yi;

        {
            let count = if i < window - 1 { i + 1 } else { window };
            if count < 2 {
                out[i] = f32::NAN;
                continue;
            }
            let c = count as f64;
            let inv_c = 1.0 / c;
            let var_x = sxx * inv_c - (sx * inv_c) * (sx * inv_c);
            let var_y = syy * inv_c - (sy * inv_c) * (sy * inv_c);
            let std_x = var_x.max(0.0).sqrt();
            let std_y = var_y.max(0.0).sqrt();

            if std_x < 2e-05 || std_y < 2e-05 {
                out[i] = f32::NAN;
            } else {
                let cov = sxy * inv_c - (sx * inv_c) * (sy * inv_c);
                out[i] = (cov / (std_x * std_y)) as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_corr() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut out = [0.0; 5];
        rolling_corr(&x, &y, 3, &mut out);
        assert!((out[4] - 1.0).abs() < 0.01);
    }
}
