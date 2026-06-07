//! Alpha158 增量计算缓存

pub mod incremental;
pub mod window_cache;

pub use incremental::IncrementalState;
pub use window_cache::WindowBuffer;
