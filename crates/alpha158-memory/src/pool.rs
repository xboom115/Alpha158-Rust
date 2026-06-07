//! 线程本地 ScratchPad 池 — 无锁复用, 避免频繁 malloc

use crate::scratch::ScratchPad;
use std::cell::RefCell;

thread_local! {
    static LOCAL_POOL: RefCell<Vec<ScratchPad>> = const { RefCell::new(Vec::new()) };
}

/// 从线程本地池获取 ScratchPad, 没有则新建
pub fn acquire_scratch(n: usize) -> ScratchPad {
    LOCAL_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if let Some(mut pad) = pool.pop() {
            pad.resize(n);
            pad
        } else {
            ScratchPad::new(n)
        }
    })
}

/// 归还 ScratchPad 到线程本地池
pub fn release_scratch(pad: ScratchPad) {
    LOCAL_POOL.with(|pool| {
        pool.borrow_mut().push(pad);
    });
}

/// RAII 守卫 — 自动归还 ScratchPad
pub struct ScratchGuard {
    pad: Option<ScratchPad>,
}

impl std::ops::Deref for ScratchGuard {
    type Target = ScratchPad;
    fn deref(&self) -> &Self::Target {
        self.pad.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for ScratchGuard {
    fn deref_mut(&mut self) -> &mut ScratchPad {
        self.pad.as_mut().unwrap()
    }
}

impl Drop for ScratchGuard {
    fn drop(&mut self) {
        if let Some(pad) = self.pad.take() {
            release_scratch(pad);
        }
    }
}

/// 获取 RAII 守卫
pub fn scratch(n: usize) -> ScratchGuard {
    ScratchGuard {
        pad: Some(acquire_scratch(n)),
    }
}
