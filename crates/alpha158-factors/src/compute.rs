//! 全部 Alpha158 因子的计算逻辑
//!
//! 分三层:
//!   Layer 1: 基础变换 (element-wise) — 计算中间变量
//!   Layer 2: 滚动窗口 — 5 个 window × 多个算子
//!   Layer 3: 最终因子组合 — element-wise 组合为 158 列输出
//!
//! 常量: WINDOW_LIST = [5, 10, 20, 30, 60], WINDOW_COUNT = 5

use alpha158_data::StockSlice;
use alpha158_memory::ScratchPad;
use alpha158_ops::common;
use alpha158_ops::correlation::rolling_corr;
use alpha158_ops::index::{rolling_idx_max, rolling_idx_min};
use alpha158_ops::mean::rolling_mean;
use alpha158_ops::min_max::{rolling_max, rolling_min};
use alpha158_ops::quantile::rolling_quantile;
use alpha158_ops::rank::rolling_rank;
use alpha158_ops::regression::rolling_regression;
use alpha158_ops::std_welford::rolling_std;
use alpha158_ops::sum::rolling_sum;

/// 默认窗口列表
const WINDOWS: [usize; 5] = [5, 10, 20, 30, 60];

/// 输出列索引常量
const O_KMID: usize = 0;
const O_KLEN: usize = 1;
const O_KMID2: usize = 2;
const O_KUP: usize = 3;
const O_KUP2: usize = 4;
const O_KLOW: usize = 5;
const O_KLOW2: usize = 6;
const O_KSFT: usize = 7;
const O_KSFT2: usize = 8;
const O_OPEN0: usize = 9;
const O_HIGH0: usize = 10;
const O_LOW0: usize = 11;
const O_VWAP0: usize = 12;

/// 每个 window 的因子起始索引 (相对 rolling 因子)
/// 每个算子贡献 5 列 (5 个 window)
/// 算子顺序: ROC, MA, STD, BETA, RSQR, RESI, MAX, MIN, QTLU, QTLD,
///           RSV, IMAX, IMIN, IMXD, CORR, CORD, CNTP, CNTN, CNTD,
///           SUMP, SUMN, SUMD, VMA, VSTD, WVMA, VSUMP, VSUMN, VSUMD
/// 共 28 个算子 × 5 窗口 = 140 列 + 9 kbar + 4 price = 153
/// 加上 RANK 5 列 = 158
const _NUM_OPS: usize = 30; // 30 个滚动算子 (含 RANK)
const ROLLING_BASE: usize = 13; // 9 kbar + 4 price = 13

/// 计算一只股票的全部 158 个 Alpha158 因子
///
/// 输入: StockSlice (原始 OHLCV 数据)
/// 输出: scratch.output[0..158] 填充完毕
pub fn compute_all_factors(stock: &StockSlice, scratch: &mut ScratchPad) {
    let n = stock.n;
    if n == 0 {
        return;
    }

    // ═══════════════════════════════════════════════════════════════
    // Layer 1: 基础变换 (中间变量)
    // ═══════════════════════════════════════════════════════════════

    // ref(close, 1)
    common::ref_offset(stock.close, 1, &mut scratch.ref_close_1);

    // price_change = close - ref(close, 1)
    common::sub(stock.close, &scratch.ref_close_1, &mut scratch.price_change);

    // abs_change = |price_change|
    common::abs(&scratch.price_change, &mut scratch.abs_change);

    // gain = max(price_change, 0)
    // loss = max(-price_change, 0) = max(ref_close_1 - close, 0)
    common::greater(stock.close, &scratch.ref_close_1, &mut scratch.gain);
    common::greater(&scratch.ref_close_1, stock.close, &mut scratch.loss);

    // log_volume = ln(volume + 1)
    common::log1p(stock.volume, &mut scratch.log_volume);

    // close_return = close / ref(close, 1) (用于 CORR/CORD)
    // 第一天无 ref(close,1), 设为 1.0 避免 inf
    common::div(stock.close, &scratch.ref_close_1, &mut scratch.close_return, 1e-12);
    if n > 0 {
        scratch.close_return[0] = 1.0;
    }

    // vol_ref_1 = ref(volume, 1), log_vol_ratio = ln(volume/vol_ref_1 + 1)
    let mut vol_ref_1 = vec![0.0f32; n];
    common::ref_offset(stock.volume, 1, &mut vol_ref_1);
    let mut vol_ratio = vec![0.0f32; n];
    common::div(stock.volume, &vol_ref_1, &mut vol_ratio, 1e-12);
    if n > 0 {
        vol_ratio[0] = 1.0;
    }
    common::log1p(&vol_ratio, &mut scratch.log_vol_ratio);

    // weighted_vol = |close_return - 1| * volume = |close/ref_close_1 - 1| * volume
    let mut abs_return = vec![0.0f32; n];
    for i in 0..n {
        abs_return[i] = (scratch.close_return[i] - 1.0).abs();
    }
    common::mul(&abs_return, stock.volume, &mut scratch.weighted_vol);

    // vol_change = volume - ref(volume, 1)
    common::sub(stock.volume, &vol_ref_1, &mut scratch.vol_change);
    common::abs(&scratch.vol_change, &mut scratch.abs_vol_change);
    common::greater(stock.volume, &vol_ref_1, &mut scratch.vol_gain);
    common::greater(&vol_ref_1, stock.volume, &mut scratch.vol_loss);

    // up_flag = (close > ref_close_1) as f32, down_flag
    for i in 0..n {
        scratch.up_flag[i] = if stock.close[i] > scratch.ref_close_1[i] {
            1.0
        } else {
            0.0
        };
        scratch.down_flag[i] = if stock.close[i] < scratch.ref_close_1[i] {
            1.0
        } else {
            0.0
        };
    }

    // hl_range = high - low + 1e-12
    common::sub(stock.high, stock.low, &mut scratch.hl_range);
    for i in 0..n {
        scratch.hl_range[i] += 1e-12;
    }

    // greater_oc = max(open, close), less_oc = min(open, close)
    for i in 0..n {
        scratch.greater_oc[i] = stock.open[i].max(stock.close[i]);
        scratch.less_oc[i] = stock.open[i].min(stock.close[i]);
    }

    // two_close_hl = 2*close - high - low
    for i in 0..n {
        scratch.two_close_hl[i] = 2.0 * stock.close[i] - stock.high[i] - stock.low[i];
    }

    // Ref(close, d) for each window
    for (wi, &w) in WINDOWS.iter().enumerate() {
        common::ref_offset(stock.close, w, &mut scratch.ref_close[wi]);
    }

    // ═══════════════════════════════════════════════════════════════
    // Layer 2: 滚动窗口计算 (5 个 window)
    // ═══════════════════════════════════════════════════════════════

    for (wi, &w) in WINDOWS.iter().enumerate() {
        let rb = &mut scratch.rolling[wi];

        // Mean(close, d)
        rolling_mean(stock.close, w, &mut rb.ma);
        // Std(close, d)
        rolling_std(stock.close, w, &mut rb.std);
        // Max(high, d)
        rolling_max(stock.high, w, &mut rb.max);
        // Min(low, d)
        rolling_min(stock.low, w, &mut rb.min);
        // Sum(gain, d)
        rolling_sum(&scratch.gain, w, &mut rb.sum_gain);
        // Sum(loss, d)
        rolling_sum(&scratch.loss, w, &mut rb.sum_loss);
        // Sum(abs_change, d)
        rolling_sum(&scratch.abs_change, w, &mut rb.sum_abs);

        // Corr(close, log_volume, d)
        rolling_corr(stock.close, &scratch.log_volume, w, &mut rb.corr_cv);
        // Corr(close_return, log_vol_ratio, d)
        rolling_corr(
            &scratch.close_return,
            &scratch.log_vol_ratio,
            w,
            &mut rb.corr_rv,
        );

        // Slope/Rsquare/Resi(close, d)
        rolling_regression(stock.close, w, &mut rb.slope, &mut rb.rsquare, &mut rb.residual);

        // IdxMax(high, d), IdxMin(low, d)
        // Qlib: rolling(d).apply(lambda x: x.argmax() + 1), 然后 /d
        rolling_idx_max(stock.high, w, &mut rb.idx_max);
        rolling_idx_min(stock.low, w, &mut rb.idx_min);

        // Quantile(close, d, 0.8) and (close, d, 0.2)
        rolling_quantile(stock.close, w, 0.8, &mut rb.qtl_80);
        rolling_quantile(stock.close, w, 0.2, &mut rb.qtl_20);

        // Rank(close, d)
        rolling_rank(stock.close, w, &mut rb.rank);

        // Mean(up_flag, d), Mean(down_flag, d)
        rolling_mean(&scratch.up_flag, w, &mut rb.mean_up);
        rolling_mean(&scratch.down_flag, w, &mut rb.mean_down);

        // Sum(vol_gain, d), Sum(vol_loss, d), Sum(|vol_change|, d)
        rolling_sum(&scratch.vol_gain, w, &mut rb.sum_vol_gain);
        rolling_sum(&scratch.vol_loss, w, &mut rb.sum_vol_loss);
        rolling_sum(&scratch.abs_vol_change, w, &mut rb.sum_vol_abs);

        // Std(weighted_vol, d), Mean(weighted_vol, d)
        rolling_std(&scratch.weighted_vol, w, &mut rb.std_wvol);
        rolling_mean(&scratch.weighted_vol, w, &mut rb.mean_wvol);

        // Mean(volume, d), Std(volume, d)
        rolling_mean(stock.volume, w, &mut rb.ma_volume);
        rolling_std(stock.volume, w, &mut rb.std_volume);
    }

    // ═══════════════════════════════════════════════════════════════
    // Layer 3: 最终因子组合 (158 列) — f64 精度
    // ═══════════════════════════════════════════════════════════════

    let eps = 1e-12_f64;

    // ── KBar 因子 (9 个) ──
    for i in 0..n {
        let o = stock.open[i] as f64;
        let h = stock.high[i] as f64;
        let l = stock.low[i] as f64;
        let c = stock.close[i] as f64;
        let range = scratch.hl_range[i] as f64;
        let go = scratch.greater_oc[i] as f64;
        let lo = scratch.less_oc[i] as f64;
        let tcl = scratch.two_close_hl[i] as f64;

        scratch.output[O_KMID][i] = ((c - o) / o) as f32;
        scratch.output[O_KLEN][i] = ((h - l) / o) as f32;
        scratch.output[O_KMID2][i] = ((c - o) / range) as f32;
        scratch.output[O_KUP][i] = ((h - go) / o) as f32;
        scratch.output[O_KUP2][i] = ((h - go) / range) as f32;
        scratch.output[O_KLOW][i] = ((lo - l) / o) as f32;
        scratch.output[O_KLOW2][i] = ((lo - l) / range) as f32;
        scratch.output[O_KSFT][i] = (tcl / o) as f32;
        scratch.output[O_KSFT2][i] = (tcl / range) as f32;
    }

    // ── Price 因子 (4 个) ──
    for i in 0..n {
        let c = stock.close[i] as f64;
        scratch.output[O_OPEN0][i] = (stock.open[i] as f64 / c) as f32;
        scratch.output[O_HIGH0][i] = (stock.high[i] as f64 / c) as f32;
        scratch.output[O_LOW0][i] = (stock.low[i] as f64 / c) as f32;
        scratch.output[O_VWAP0][i] = (stock.vwap[i] as f64 / c) as f32;
    }

    // ── Rolling 因子 (30 算子 × 5 窗口 = 150 个) ──
    for (wi, &_w) in WINDOWS.iter().enumerate() {
        let rb = &scratch.rolling[wi];
        let base = ROLLING_BASE + wi;

        for i in 0..n {
            let c = stock.close[i] as f64;
            let v = stock.volume[i] as f64;
            let c_inv = 1.0 / c;
            let v_inv = 1.0 / (v + eps);

            let rc = scratch.ref_close[wi][i] as f64;
            let ma = rb.ma[i] as f64;
            let std = rb.std[i] as f64;
            let slope = rb.slope[i] as f64;
            let rsq = rb.rsquare[i] as f64;
            let resi = rb.residual[i] as f64;
            let max_h = rb.max[i] as f64;
            let min_l = rb.min[i] as f64;
            let qtl80 = rb.qtl_80[i] as f64;
            let qtl20 = rb.qtl_20[i] as f64;
            let rank = rb.rank[i] as f64;
            let idx_max = rb.idx_max[i] as f64;
            let idx_min = rb.idx_min[i] as f64;
            let corr_cv = rb.corr_cv[i] as f64;
            let corr_rv = rb.corr_rv[i] as f64;
            let mean_up = rb.mean_up[i] as f64;
            let mean_down = rb.mean_down[i] as f64;
            let sum_gain = rb.sum_gain[i] as f64;
            let sum_loss = rb.sum_loss[i] as f64;
            let sum_abs = rb.sum_abs[i] as f64 + eps;
            let ma_vol = rb.ma_volume[i] as f64;
            let std_vol = rb.std_volume[i] as f64;
            let std_wvol = rb.std_wvol[i] as f64;
            let mean_wvol = rb.mean_wvol[i] as f64;
            let sum_vg = rb.sum_vol_gain[i] as f64;
            let sum_vl = rb.sum_vol_loss[i] as f64;
            let sum_va = rb.sum_vol_abs[i] as f64 + eps;

            scratch.output[base + 0 * 5][i] = (rc * c_inv) as f32;       // ROC
            scratch.output[base + 1 * 5][i] = (ma * c_inv) as f32;       // MA
            scratch.output[base + 2 * 5][i] = (std * c_inv) as f32;      // STD
            scratch.output[base + 3 * 5][i] = (slope * c_inv) as f32;    // BETA
            scratch.output[base + 4 * 5][i] = rsq as f32;                 // RSQR
            scratch.output[base + 5 * 5][i] = (resi * c_inv) as f32;     // RESI
            scratch.output[base + 6 * 5][i] = (max_h * c_inv) as f32;    // MAX
            scratch.output[base + 7 * 5][i] = (min_l * c_inv) as f32;    // MIN
            scratch.output[base + 8 * 5][i] = (qtl80 * c_inv) as f32;    // QTLU
            scratch.output[base + 9 * 5][i] = (qtl20 * c_inv) as f32;    // QTLD
            scratch.output[base + 10 * 5][i] = rank as f32;               // RANK
            // RSV
            let rsv_denom = max_h - min_l + eps;
            scratch.output[base + 11 * 5][i] = ((c - min_l) / rsv_denom) as f32;
            scratch.output[base + 12 * 5][i] = idx_max as f32;            // IMAX
            scratch.output[base + 13 * 5][i] = idx_min as f32;            // IMIN
            scratch.output[base + 14 * 5][i] = (idx_max - idx_min) as f32; // IMXD
            scratch.output[base + 15 * 5][i] = corr_cv as f32;            // CORR
            scratch.output[base + 16 * 5][i] = corr_rv as f32;            // CORD
            scratch.output[base + 17 * 5][i] = mean_up as f32;            // CNTP
            scratch.output[base + 18 * 5][i] = mean_down as f32;          // CNTN
            scratch.output[base + 19 * 5][i] = (mean_up - mean_down) as f32; // CNTD
            // SUMP, SUMN, SUMD
            scratch.output[base + 20 * 5][i] = (sum_gain / sum_abs) as f32;
            scratch.output[base + 21 * 5][i] = (sum_loss / sum_abs) as f32;
            scratch.output[base + 22 * 5][i] = ((sum_gain - sum_loss) / sum_abs) as f32;
            // VMA, VSTD
            scratch.output[base + 23 * 5][i] = (ma_vol * v_inv) as f32;
            scratch.output[base + 24 * 5][i] = (std_vol * v_inv) as f32;
            // WVMA
            let wvol_denom = mean_wvol + eps;
            scratch.output[base + 25 * 5][i] = (std_wvol / wvol_denom) as f32;
            // VSUMP, VSUMN, VSUMD
            scratch.output[base + 26 * 5][i] = (sum_vg / sum_va) as f32;
            scratch.output[base + 27 * 5][i] = (sum_vl / sum_va) as f32;

            // VSUMDd = (Sum(vol_gain) - Sum(vol_loss)) / (Sum(|vol_change|) + eps)
            scratch.output[base + 28 * 5][i] = ((sum_vg - sum_vl) / sum_va) as f32;

            // RANKd (算子索引 10, 已在上面计算)
        }
    }
}
