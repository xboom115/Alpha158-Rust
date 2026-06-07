//! Alpha158 滚动窗口算子库
//!
//! 所有算子实现 O(n) 或 O(n log w) 时间复杂度, 支持 SIMD auto-vectorization.

pub mod common;
pub mod correlation;
pub mod index;
pub mod mean;
pub mod min_max;
pub mod quantile;
pub mod rank;
pub mod regression;
pub mod std_welford;
pub mod sum;

/// 滚动窗口算子 trait
pub trait RollingOperator: Send + Sync {
    /// 算子内部状态
    type State: Default + Clone;

    /// 窗口大小
    fn window(&self) -> usize;

    /// 推入新值, 移除旧值, 更新状态
    fn push(&self, state: &mut Self::State, new_val: f32, old_val: f32, pos: usize);

    /// 从状态获取当前结果
    fn result(&self, state: &Self::State) -> f32;

    /// 批量计算 (默认: 逐元素)
    fn compute_batch(&self, data: &[f32], out: &mut [f32]) {
        let w = self.window();
        let mut state = Self::State::default();
        for i in 0..data.len() {
            if i >= w {
                self.push(&mut state, data[i], data[i - w], i);
            } else {
                self.push(&mut state, data[i], 0.0, i);
            }
            if i >= w - 1 {
                out[i] = self.result(&state);
            }
        }
    }
}
