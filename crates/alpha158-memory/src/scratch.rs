//! ScratchPad — 每只股票的计算工作区, 预分配, 复用

/// 单个 window 的滚动窗口中间结果缓冲
#[derive(Clone)]
pub struct RollingBuffers {
    pub ma: Vec<f32>,
    pub std: Vec<f32>,
    pub max: Vec<f32>,
    pub min: Vec<f32>,
    pub sum_gain: Vec<f32>,
    pub sum_loss: Vec<f32>,
    pub sum_abs: Vec<f32>,
    pub corr_cv: Vec<f32>,
    pub corr_rv: Vec<f32>,
    pub slope: Vec<f32>,
    pub rsquare: Vec<f32>,
    pub residual: Vec<f32>,
    pub idx_max: Vec<f32>,
    pub idx_min: Vec<f32>,
    pub qtl_80: Vec<f32>,
    pub qtl_20: Vec<f32>,
    pub rank: Vec<f32>,
    pub mean_up: Vec<f32>,
    pub mean_down: Vec<f32>,
    pub sum_vol_gain: Vec<f32>,
    pub sum_vol_loss: Vec<f32>,
    pub sum_vol_abs: Vec<f32>,
    pub std_wvol: Vec<f32>,
    pub mean_wvol: Vec<f32>,
    pub ma_volume: Vec<f32>,
    pub std_volume: Vec<f32>,
}

impl RollingBuffers {
    pub fn new(n: usize) -> Self {
        Self {
            ma: vec![0.0; n],
            std: vec![0.0; n],
            max: vec![0.0; n],
            min: vec![0.0; n],
            sum_gain: vec![0.0; n],
            sum_loss: vec![0.0; n],
            sum_abs: vec![0.0; n],
            corr_cv: vec![0.0; n],
            corr_rv: vec![0.0; n],
            slope: vec![0.0; n],
            rsquare: vec![0.0; n],
            residual: vec![0.0; n],
            idx_max: vec![0.0; n],
            idx_min: vec![0.0; n],
            qtl_80: vec![0.0; n],
            qtl_20: vec![0.0; n],
            rank: vec![0.0; n],
            mean_up: vec![0.0; n],
            mean_down: vec![0.0; n],
            sum_vol_gain: vec![0.0; n],
            sum_vol_loss: vec![0.0; n],
            sum_vol_abs: vec![0.0; n],
            std_wvol: vec![0.0; n],
            mean_wvol: vec![0.0; n],
            ma_volume: vec![0.0; n],
            std_volume: vec![0.0; n],
        }
    }

    pub fn resize(&mut self, n: usize) {
        let bufs = [
            &mut self.ma,
            &mut self.std,
            &mut self.max,
            &mut self.min,
            &mut self.sum_gain,
            &mut self.sum_loss,
            &mut self.sum_abs,
            &mut self.corr_cv,
            &mut self.corr_rv,
            &mut self.slope,
            &mut self.rsquare,
            &mut self.residual,
            &mut self.idx_max,
            &mut self.idx_min,
            &mut self.qtl_80,
            &mut self.qtl_20,
            &mut self.rank,
            &mut self.mean_up,
            &mut self.mean_down,
            &mut self.sum_vol_gain,
            &mut self.sum_vol_loss,
            &mut self.sum_vol_abs,
            &mut self.std_wvol,
            &mut self.mean_wvol,
            &mut self.ma_volume,
            &mut self.std_volume,
        ];
        for buf in bufs {
            buf.resize(n, 0.0);
        }
    }
}

/// 每只股票的计算工作区
pub struct ScratchPad {
    /// Layer 1 中间变量 (命名缓冲)
    pub ref_close_1: Vec<f32>,
    pub price_change: Vec<f32>,
    pub abs_change: Vec<f32>,
    pub gain: Vec<f32>,
    pub loss: Vec<f32>,
    pub log_volume: Vec<f32>,
    pub close_return: Vec<f32>,
    pub log_vol_ratio: Vec<f32>,
    pub weighted_vol: Vec<f32>,
    pub vol_change: Vec<f32>,
    pub vol_gain: Vec<f32>,
    pub vol_loss: Vec<f32>,
    pub up_flag: Vec<f32>,
    pub down_flag: Vec<f32>,
    pub hl_range: Vec<f32>,
    pub greater_oc: Vec<f32>,
    pub less_oc: Vec<f32>,
    pub two_close_hl: Vec<f32>,
    pub abs_vol_change: Vec<f32>,

    /// Ref(close, d) per window
    pub ref_close: [Vec<f32>; 5],

    /// Layer 2 滚动窗口结果
    pub rolling: [RollingBuffers; 5],

    /// Layer 3 最终输出: 158 列
    pub output: Vec<Vec<f32>>,

    /// 当前行数
    pub n: usize,
}

impl ScratchPad {
    pub fn new(n: usize) -> Self {
        Self {
            ref_close_1: vec![0.0; n],
            price_change: vec![0.0; n],
            abs_change: vec![0.0; n],
            gain: vec![0.0; n],
            loss: vec![0.0; n],
            log_volume: vec![0.0; n],
            close_return: vec![0.0; n],
            log_vol_ratio: vec![0.0; n],
            weighted_vol: vec![0.0; n],
            vol_change: vec![0.0; n],
            vol_gain: vec![0.0; n],
            vol_loss: vec![0.0; n],
            up_flag: vec![0.0; n],
            down_flag: vec![0.0; n],
            hl_range: vec![0.0; n],
            greater_oc: vec![0.0; n],
            less_oc: vec![0.0; n],
            two_close_hl: vec![0.0; n],
            abs_vol_change: vec![0.0; n],
            ref_close: [
                vec![0.0; n],
                vec![0.0; n],
                vec![0.0; n],
                vec![0.0; n],
                vec![0.0; n],
            ],
            rolling: [
                RollingBuffers::new(n),
                RollingBuffers::new(n),
                RollingBuffers::new(n),
                RollingBuffers::new(n),
                RollingBuffers::new(n),
            ],
            output: vec![vec![0.0; n]; 158],
            n,
        }
    }

    pub fn resize(&mut self, n: usize) {
        self.n = n;
        let bufs = [
            &mut self.ref_close_1,
            &mut self.price_change,
            &mut self.abs_change,
            &mut self.gain,
            &mut self.loss,
            &mut self.log_volume,
            &mut self.close_return,
            &mut self.log_vol_ratio,
            &mut self.weighted_vol,
            &mut self.vol_change,
            &mut self.vol_gain,
            &mut self.vol_loss,
            &mut self.up_flag,
            &mut self.down_flag,
            &mut self.hl_range,
            &mut self.greater_oc,
            &mut self.less_oc,
            &mut self.two_close_hl,
            &mut self.abs_vol_change,
        ];
        for buf in bufs {
            buf.resize(n, 0.0);
        }
        for rc in &mut self.ref_close {
            rc.resize(n, 0.0);
        }
        for rb in &mut self.rolling {
            rb.resize(n);
        }
        for col in &mut self.output {
            col.resize(n, 0.0);
        }
    }
}
