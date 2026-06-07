//! 环形缓冲区 — 保存最近 max_window 天的原始数据

/// 环形缓冲区, 固定容量, 支持 O(1) push/pop
pub struct WindowBuffer {
    pub capacity: usize,
    pub data: Vec<f32>,
    pub head: usize,
    pub len: usize,
}

impl WindowBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            data: vec![0.0; capacity],
            head: 0,
            len: 0,
        }
    }

    /// 推入一个值, 如果满了则覆盖最旧的
    pub fn push(&mut self, val: f32) {
        self.data[self.head] = val;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// 获取当前窗口数据 (从最旧到最新)
    pub fn as_slice(&self) -> Vec<f32> {
        if self.len < self.capacity {
            // 还没满, 数据从 0 到 len-1
            self.data[..self.len].to_vec()
        } else {
            // 已满, 数据从 head 到 head+capacity-1 (环形)
            let mut result = Vec::with_capacity(self.capacity);
            for i in 0..self.capacity {
                let idx = (self.head + i) % self.capacity;
                result.push(self.data[idx]);
            }
            result
        }
    }

    /// 获取最旧的值
    pub fn oldest(&self) -> f32 {
        if self.len < self.capacity {
            self.data[0]
        } else {
            self.data[self.head]
        }
    }

    /// 获取最新的值
    pub fn newest(&self) -> f32 {
        if self.len == 0 {
            0.0
        } else {
            let idx = if self.head == 0 {
                self.capacity - 1
            } else {
                self.head - 1
            };
            self.data[idx]
        }
    }
}
