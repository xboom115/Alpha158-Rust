//! Alpha158 内存管理 — 线程本地 ScratchPad 池化

mod pool;
mod scratch;

pub use pool::{acquire_scratch, release_scratch, scratch, ScratchGuard};
pub use scratch::{RollingBuffers, ScratchPad};
