//! Rolling IdxMax/IdxMin — 对齐 Qlib, f64 内部精度

use std::collections::VecDeque;

/// 批量 rolling idx_max, O(n)
///
/// 对齐 Qlib: rolling(d).apply(lambda x: x.argmax() + 1), 然后 /d
/// 窗口 = d 个元素, 返回 1-based 索引 / d
pub fn rolling_idx_max(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
    let d = window as f32;
    let mut deque: VecDeque<(usize, f64)> = VecDeque::new();

    for i in 0..n {
        let val = data[i] as f64;
        while let Some(&front) = deque.front() {
            if front.0 + window <= i {
                deque.pop_front();
            } else {
                break;
            }
        }
        // 平局时保留第一个 (对齐 numpy argmax)
        while let Some(&back) = deque.back() {
            if back.1 < val {
                deque.pop_back();
            } else {
                break;
            }
        }
        deque.push_back((i, val));

        {
            let max_pos = deque.front().unwrap().0;
            let window_start = if i >= window - 1 { i + 1 - window } else { 0 };
            let argmax_0based = max_pos - window_start;
            let effective_d = if i < window - 1 { (i + 1) as f32 } else { d };
            out[i] = (argmax_0based + 1) as f32 / effective_d;
        }
    }
}

/// 批量 rolling idx_min, O(n)
pub fn rolling_idx_min(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
    let d = window as f32;
    let mut deque: VecDeque<(usize, f64)> = VecDeque::new();

    for i in 0..n {
        let val = data[i] as f64;
        while let Some(&front) = deque.front() {
            if front.0 + window <= i {
                deque.pop_front();
            } else {
                break;
            }
        }
        // 平局时保留第一个 (对齐 numpy argmin)
        while let Some(&back) = deque.back() {
            if back.1 > val {
                deque.pop_back();
            } else {
                break;
            }
        }
        deque.push_back((i, val));

        {
            let min_pos = deque.front().unwrap().0;
            let window_start = if i >= window - 1 { i + 1 - window } else { 0 };
            let argmin_0based = min_pos - window_start;
            let effective_d = if i < window - 1 { (i + 1) as f32 } else { d };
            out[i] = (argmin_0based + 1) as f32 / effective_d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_idx_max() {
        let data = [1.0, 3.0, 2.0, 5.0, 4.0];
        let mut out = [0.0; 5];
        rolling_idx_max(&data, 3, &mut out);
        assert!((out[2] - 2.0 / 3.0).abs() < 0.01);
        assert!((out[3] - 1.0).abs() < 0.01);
        assert!((out[4] - 2.0 / 3.0).abs() < 0.01);
    }
}
