//! 通用 element-wise 操作 — 全部 SIMD 友好, 可 auto-vectorize

/// Greater(a, b) = max(a - b, 0), 无分支
#[inline]
pub fn greater(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = (a[i] - b[i]).max(0.0);
    }
}

/// Less(a, b) = min(a, b)
#[inline]
pub fn less(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i].min(b[i]);
    }
}

/// Abs
#[inline]
pub fn abs(a: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i].abs();
    }
}

/// Log(x + 1)
#[inline]
pub fn log1p(a: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = (a[i] + 1.0).ln();
    }
}

/// Element-wise divide
#[inline]
pub fn div(a: &[f32], b: &[f32], out: &mut [f32], eps: f32) {
    for i in 0..a.len() {
        out[i] = a[i] / (b[i] + eps);
    }
}

/// Element-wise subtract
#[inline]
pub fn sub(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] - b[i];
    }
}

/// Element-wise multiply
#[inline]
pub fn mul(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] * b[i];
    }
}

/// Scalar subtract: out[i] = a[i] - scalar
#[inline]
pub fn sub_scalar(a: &[f32], scalar: f32, out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] - scalar;
    }
}

/// Ref: 向右偏移 n 位, 前 n 个填 0
/// Ref(data, n)[i] = data[i - n] if i >= n else 0.0
#[inline]
pub fn ref_offset(data: &[f32], offset: usize, out: &mut [f32]) {
    let n = data.len();
    for i in 0..n {
        out[i] = if i >= offset { data[i - offset] } else { 0.0 };
    }
}

/// Element-wise: if cond > 0 then a else b
#[inline]
pub fn select(cond: &[f32], a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..cond.len() {
        out[i] = if cond[i] > 0.0 { a[i] } else { b[i] };
    }
}
