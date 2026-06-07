//! Rolling Rank — f64 内部精度, 对齐 pandas rank(pct=True)

use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// 批量 rolling rank, O(n log w)
pub fn rolling_rank(
    data: &[f32],
    window: usize,
    out: &mut [f32],
) {
    let n = data.len();
    if n == 0 {
        return;
    }
    let mut sorted: BTreeMap<OrderedFloat<f64>, usize> = BTreeMap::new();

    for i in 0..n {
        if i >= window {
            let old = OrderedFloat(data[i - window] as f64);
            if let Some(count) = sorted.get_mut(&old) {
                if *count <= 1 {
                    sorted.remove(&old);
                } else {
                    *count -= 1;
                }
            }
        }

        *sorted
            .entry(OrderedFloat(data[i] as f64))
            .or_insert(0) += 1;

        {
            let current = OrderedFloat(data[i] as f64);
            let count: usize = sorted.values().sum();
            let mut pos = 1usize;
            let mut avg_rank = 0.0f64;

            for (val, cnt) in sorted.iter() {
                if *val == current {
                    avg_rank = pos as f64 + (*cnt - 1) as f64 / 2.0;
                    break;
                }
                pos += cnt;
            }

            out[i] = (avg_rank / count.max(1) as f64) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_rank() {
        let data = [3.0, 1.0, 4.0, 1.0, 5.0];
        let mut out = [0.0; 5];
        rolling_rank(&data, 3, &mut out);
        assert!((out[2] - 1.0).abs() < 0.01);
        assert!((out[3] - 0.5).abs() < 0.01);
        assert!((out[4] - 1.0).abs() < 0.01);
    }
}
