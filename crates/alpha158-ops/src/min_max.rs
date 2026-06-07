//! Rolling Max/Min — Monotonic Deque, O(n) 摊还
//!
//! 内部使用 f64 精度, 输入输出为 f32

use crate::RollingOperator;
use std::collections::VecDeque;

// ── Rolling Max ──

pub struct RollingMax {
    pub window: usize,
}

#[derive(Clone)]
pub struct MaxDequeState {
    pub deque: VecDeque<(usize, f64)>,
}

impl Default for MaxDequeState {
    fn default() -> Self {
        Self {
            deque: VecDeque::new(),
        }
    }
}

impl RollingOperator for RollingMax {
    type State = MaxDequeState;
    fn window(&self) -> usize {
        self.window
    }

    fn push(&self, state: &mut MaxDequeState, new_val: f32, _old_val: f32, pos: usize) {
        let new_f64 = new_val as f64;
        while let Some(&front) = state.deque.front() {
            if front.0 + self.window <= pos {
                state.deque.pop_front();
            } else {
                break;
            }
        }
        while let Some(&back) = state.deque.back() {
            if back.1 <= new_f64 {
                state.deque.pop_back();
            } else {
                break;
            }
        }
        state.deque.push_back((pos, new_f64));
    }

    #[inline]
    fn result(&self, state: &MaxDequeState) -> f32 {
        state.deque.front().map_or(f32::NAN, |&(_, v)| v as f32)
    }
}

/// 批量 rolling max, O(n)
pub fn rolling_max(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
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
        while let Some(&back) = deque.back() {
            if back.1 <= val {
                deque.pop_back();
            } else {
                break;
            }
        }
        deque.push_back((i, val));
        out[i] = deque.front().unwrap().1 as f32;
    }
}

// ── Rolling Min ──

pub struct RollingMin {
    pub window: usize,
}

#[derive(Clone)]
pub struct MinDequeState {
    pub deque: VecDeque<(usize, f64)>,
}

impl Default for MinDequeState {
    fn default() -> Self {
        Self {
            deque: VecDeque::new(),
        }
    }
}

impl RollingOperator for RollingMin {
    type State = MinDequeState;
    fn window(&self) -> usize {
        self.window
    }

    fn push(&self, state: &mut MinDequeState, new_val: f32, _old_val: f32, pos: usize) {
        let new_f64 = new_val as f64;
        while let Some(&front) = state.deque.front() {
            if front.0 + self.window <= pos {
                state.deque.pop_front();
            } else {
                break;
            }
        }
        while let Some(&back) = state.deque.back() {
            if back.1 >= new_f64 {
                state.deque.pop_back();
            } else {
                break;
            }
        }
        state.deque.push_back((pos, new_f64));
    }

    #[inline]
    fn result(&self, state: &MinDequeState) -> f32 {
        state.deque.front().map_or(f32::NAN, |&(_, v)| v as f32)
    }
}

/// 批量 rolling min, O(n)
pub fn rolling_min(data: &[f32], window: usize, out: &mut [f32]) {
    let n = data.len();
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
        while let Some(&back) = deque.back() {
            if back.1 >= val {
                deque.pop_back();
            } else {
                break;
            }
        }
        deque.push_back((i, val));
        out[i] = deque.front().unwrap().1 as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_max() {
        let data = [1.0, 3.0, 2.0, 5.0, 4.0];
        let mut out = [0.0; 5];
        rolling_max(&data, 3, &mut out);
        assert_eq!(out[2], 3.0);
        assert_eq!(out[3], 5.0);
        assert_eq!(out[4], 5.0);
    }

    #[test]
    fn test_rolling_min() {
        let data = [3.0, 1.0, 4.0, 2.0, 5.0];
        let mut out = [0.0; 5];
        rolling_min(&data, 3, &mut out);
        assert_eq!(out[2], 1.0);
        assert_eq!(out[3], 1.0);
        assert_eq!(out[4], 2.0);
    }
}
