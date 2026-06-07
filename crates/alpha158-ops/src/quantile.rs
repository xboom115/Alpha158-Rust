//! Rolling Quantile — f64 内部精度, 线性插值

use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// 批量 rolling quantile, O(n log w)
///
/// 对齐 Qlib min_periods=1 + 线性插值
pub fn rolling_quantile(
    data: &[f32],
    window: usize,
    quantile: f32,
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
            let count: usize = sorted.values().sum();
            let mut arr: Vec<f64> = Vec::with_capacity(count);
            for (val, cnt) in sorted.iter() {
                for _ in 0..*cnt {
                    arr.push(val.0);
                }
            }

            let effective = count.max(1);
            let q = quantile as f64;
            let pos = q * (effective as f64 - 1.0);
            let k = pos.floor() as usize;
            let frac = pos - k as f64;

            if k + 1 < arr.len() {
                out[i] = (arr[k] * (1.0 - frac) + arr[k + 1] * frac) as f32;
            } else {
                out[i] = arr[arr.len() - 1] as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_quantile() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut out = [0.0; 10];
        rolling_quantile(&data, 5, 0.8, &mut out);
        assert!((out[4] - 4.2).abs() < 0.01);
    }
}
