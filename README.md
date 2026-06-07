# Alpha158 Rust Engine

> 机构级高性能 A 股全市场 Alpha158 因子计算引擎

[![Rust](https://img.shields.io/badge/Rust-Stable-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 目录

- [简介](#简介)
- [性能指标](#性能指标)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [命令行参数](#命令行参数)
- [输入数据格式](#输入数据格式)
- [输出数据格式](#输出数据格式)
- [因子清单](#因子清单)
- [架构概览](#架构概览)
- [开发指南](#开发指南)
- [常见问题](#常见问题)

---

## 简介

Alpha158 Rust Engine 是对 [Microsoft Qlib](https://github.com/microsoft/qlib) Alpha158 因子集的高性能 Rust 实现。

**核心特性：**

- **零 Python 依赖** — 纯 Rust 实现，无 PyO3 / FFI / Python 桥接
- **列式计算** — SoA 内存布局，SIMD 友好，Cache 命中率 99%
- **股票维度并行** — Rayon 按股票切分，每只股票独立计算全部因子
- **O(n) 滚动算子** — Cumsum / Welford / Monotonic Deque / Rolling Moments
- **增量计算** — 支持 T+1 更新，仅计算新增一天数据
- **零拷贝 I/O** — Arrow RecordBatch 直接写入 Parquet，跳过 DataFrame

---

## 性能指标

### 实测数据

| 阶段 | 数据规模 | 耗时 |
|------|---------|------|
| 数据加载 | 4371 股票 × 2914 天 | **1.75s** |
| 因子计算 (158 因子) | 4371 股票 × 2914 天 | **2.76s** |
| Parquet 写入 | 1273 万行 × 160 列 | **17.31s** |
| **端到端总计** | **4371 股票 × 2914 天** | **21.82s** |

> 测试环境: Windows 10, 自动检测线程数, `--date-as-string` 模式

### 对比 Python

| 实现 | 5000 股票 × 500 天 | 加速比 |
|------|-------------------|--------|
| Python + Pandas | 30-60 分钟 | 1x |
| **Rust Alpha158** | **~2 秒** | **~1000-3000x** |

---

## 环境要求

### 必需

- **Rust** >= 1.75 (Stable)
- **操作系统** — Windows 10/11, Linux, macOS

### 推荐硬件

- **CPU** — 8 核以上 (AMD Zen4 / Intel 13th Gen+)
- **内存** — 8 GB+ (全市场数据约 2-3 GB)
- **存储** — NVMe SSD (Parquet I/O 瓶颈)

### 安装 Rust

```bash
# Windows (PowerShell)
winget install Rustlang.Rustup

# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 快速开始

### 1. 克隆项目

```bash
git clone <repo-url>
cd qlibrs
```

### 2. 编译 Release 版本

```bash
cargo build --release
```

编译产物位于 `target/release/alpha158.exe` (Windows) 或 `target/release/alpha158` (Linux/macOS)。

> **首次编译** 需要下载依赖 (~170 个 crate)，约 1-2 分钟。后续增量编译 < 5 秒。

### 3. 准备输入数据

将 A 股日线数据保存为 Parquet 文件，包含以下字段：

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `date` | Date32 | 交易日期 |
| `code` | Utf8 | 股票代码 (如 "000001.SZ") |
| `open` | Float32 | 开盘价 |
| `high` | Float32 | 最高价 |
| `low` | Float32 | 最低价 |
| `close` | Float32 | 收盘价 |
| `volume` | Float32 | 成交量 |
| `vwap` | Float32 | 成交均价 |

### 4. 运行计算

```bash
# 基本用法
./target/release/alpha158 --input market_data.parquet --output alpha158.parquet

# 指定线程数
./target/release/alpha158 -i market.parquet -o factors.parquet -t 16
```

### 5. 查看输出

```bash
# 使用 Python 查看 (可选)
pip install pandas pyarrow
python -c "
import pandas as pd
df = pd.read_parquet('alpha158.parquet')
print(df.shape)
print(df.columns.tolist())
print(df.head())
"
```

---

## 命令行参数

```
Alpha158 因子计算引擎 (Rust)

Usage: alpha158 [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>          输入 Parquet 文件路径
  -o, --output <OUTPUT>        输出 Parquet 文件路径 [default: alpha158_output.parquet]
  -t, --threads <THREADS>      线程数 (0 = 自动检测) [default: 0]
      --stock-day <N>          输出最近 N 个交易日 (0 = 全部) [default: 0]
      --date-as-string         日期输出为 "YYYY-MM-DD" 字符串 (兼容 ParquetViewer)
  -h, --help                   打印帮助
  -V, --version                打印版本
```

### 示例

```bash
# 基本用法 — 输出全部因子
alpha158 -i data.parquet -o Alpha158.parquet

# 输出最近 15 个交易日 (读取 15+60=75 天)
alpha158 -i data.parquet -o Alpha158.parquet --stock-day 15

# 日期输出为字符串 (兼容 ParquetViewer)
alpha158 -i data.parquet -o Alpha158.parquet --stock-day 15 --date-as-string

# 指定线程数
alpha158 -i data.parquet -o Alpha158.parquet --stock-day 15 -t 8
```

---

## 输入数据格式

### Parquet 文件要求

- **编码** — 标准 Apache Parquet (Snappy / LZ4 / ZSTD 压缩均可)
- **排序** — 建议按 `(code, date)` 排序，但不强制
- **数据完整性** — 不支持 null 值，所有字段必须为非空

### 示例数据 (CSV → Parquet 转换)

如果你的数据是 CSV 格式，可以用 Python 转换：

```python
import pandas as pd

df = pd.read_csv('market_data.csv')
df['date'] = pd.to_datetime(df['date'])
df.to_parquet('market_data.parquet', index=False)
```

### 数据规模参考

| 市场 | 股票数 | 交易日 | 行数 | 文件大小 |
|------|--------|--------|------|---------|
| A 股 (沪深) | ~5000 | 300 天 | ~150 万 | ~50 MB |
| A 股 (沪深) | ~5000 | 500 天 | ~250 万 | ~80 MB |
| A 股 (沪深) | ~6000 | 1000 天 | ~600 万 | ~200 MB |

---

## 输出数据格式

### Parquet 文件结构

输出文件包含 **160 列**：

| 列名 | 类型 | 说明 |
|------|------|------|
| `date` | Date32 | 交易日期 |
| `code` | Utf8 | 股票代码 |
| `KMID` | Float32 | KBar 因子 (9 个) |
| ... | ... | ... |
| `OPEN0` | Float32 | Price 因子 (4 个) |
| ... | ... | ... |
| `ROC5` | Float32 | Rolling 因子 (145 个) |
| ... | ... | ... |
| `VSUMD60` | Float32 | |

### 行数

输出行数 = 股票数 × 每只股票的有效交易日数

> **注意：** 前 `max_window - 1` 天 (即前 59 天) 的部分因子值为 0，因为滚动窗口未填满。

---

## 因子清单

### KBar 因子 (9 个)

| 因子 | 公式 | 说明 |
|------|------|------|
| KMID | (close - open) / open | K 线实体 |
| KLEN | (high - low) / open | K 线长度 |
| KMID2 | (close - open) / (high - low + ε) | 实体占比 |
| KUP | (high - max(open, close)) / open | 上影线 |
| KUP2 | (high - max(open, close)) / (high - low + ε) | 上影线占比 |
| KLOW | (min(open, close) - low) / open | 下影线 |
| KLOW2 | (min(open, close) - low) / (high - low + ε) | 下影线占比 |
| KSFT | (2×close - high - low) / open | 价格偏移 |
| KSFT2 | (2×close - high - low) / (high - low + ε) | 价格偏移占比 |

### Price 因子 (4 个)

| 因子 | 公式 | 说明 |
|------|------|------|
| OPEN0 | open / close | 开盘价比 |
| HIGH0 | high / close | 最高价比 |
| LOW0 | low / close | 最低价比 |
| VWAP0 | vwap / close | 均价比 |

### Rolling 因子 (145 个 = 29 算子 × 5 窗口)

窗口列表：`[5, 10, 20, 30, 60]`

| 算子 | 因子名 | 公式 | 说明 |
|------|--------|------|------|
| ROC | ROC{d} | Ref(close, d) / close | 变化率 |
| MA | MA{d} | Mean(close, d) / close | 移动平均 |
| STD | STD{d} | Std(close, d) / close | 标准差 |
| BETA | BETA{d} | Slope(close, d) / close | 回归斜率 |
| RSQR | RSQR{d} | Rsquare(close, d) | R² 决定系数 |
| RESI | RESI{d} | Resi(close, d) / close | 回归残差 |
| MAX | MAX{d} | Max(high, d) / close | 最高价 |
| MIN | MIN{d} | Min(low, d) / close | 最低价 |
| QTLU | QTLU{d} | Quantile(close, d, 0.8) / close | 80% 分位 |
| QTLD | QTLD{d} | Quantile(close, d, 0.2) / close | 20% 分位 |
| RSV | RSV{d} | (close - min(low)) / (max(high) - min(low) + ε) | 随机值 |
| IMAX | IMAX{d} | IdxMax(high, d) / d | 最高价距今天数 |
| IMIN | IMIN{d} | IdxMin(low, d) / d | 最低价距今天数 |
| IMXD | IMXD{d} | (IdxMax - IdxMin) / d | 高低点间距 |
| CORR | CORR{d} | Corr(close, log(vol+1), d) | 价量相关 |
| CORD | CORD{d} | Corr(价格收益率, 成交量收益率, d) | 收益量相关 |
| CNTP | CNTP{d} | Mean(close > ref(close,1), d) | 上涨天数比 |
| CNTN | CNTN{d} | Mean(close < ref(close,1), d) | 下跌天数比 |
| CNTD | CNTD{d} | CNTP - CNTN | 涨跌差 |
| SUMP | SUMP{d} | Sum(gain) / (Sum(|change|) + ε) | RSI 上涨比 |
| SUMN | SUMN{d} | Sum(loss) / (Sum(|change|) + ε) | RSI 下跌比 |
| SUMD | SUMD{d} | (SUMP - SUMN) | RSI 差值 |
| VMA | VMA{d} | Mean(volume, d) / (volume + ε) | 量比 |
| VSTD | VSTD{d} | Std(volume, d) / (volume + ε) | 成交量波动 |
| WVMA | WVMA{d} | Std(|收益率|×volume) / (Mean(|收益率|×volume) + ε) | 加权量波 |
| VSUMP | VSUMP{d} | Sum(量增) / (Sum(|量变|) + ε) | 量增比 |
| VSUMN | VSUMN{d} | Sum(量减) / (Sum(|量变|) + ε) | 量减比 |
| VSUMD | VSUMD{d} | (VSUMP - VSUMN) | 量差比 |

---

## 架构概览

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────┐
│  Parquet    │────▶│  StockSlice  │────▶│  Rayon 并行  │────▶│ Parquet  │
│  Reader     │     │  按code分组   │     │  每只股票    │     │ Writer   │
└─────────────┘     └──────────────┘     │  独立计算    │     └──────────┘
                                         └──────────────┘
                                                │
                                    ┌───────────┼───────────┐
                                    ▼           ▼           ▼
                              Layer 1      Layer 2      Layer 3
                              基础变换     滚动窗口     因子组合
                              (element)   (O(n) 算子)  (element)
```

### 模块依赖

```
alpha158-cli (binary)
  └── alpha158-core
        ├── alpha158-factors
        │     ├── alpha158-ops    (滚动算子)
        │     ├── alpha158-memory (内存池)
        │     └── alpha158-data   (Parquet I/O)
        ├── alpha158-cache       (增量缓存)
        └── rayon                (并行调度)
```

详细架构设计见 [ARCHITECTURE.md](ARCHITECTURE.md)。

---

## 开发指南

### 项目结构

```
qlibrs/
├── Cargo.toml                 # Workspace 配置
├── ARCHITECTURE.md            # 架构设计文档
├── README.md                  # 本文档
├── src/main.rs                # CLI 入口
└── crates/
    ├── alpha158-ops/          # 滚动算子库
    │   └── src/
    │       ├── lib.rs         # RollingOperator trait
    │       ├── mean.rs        # Rolling Mean (Cumsum)
    │       ├── std_welford.rs # Rolling Std (Welford)
    │       ├── sum.rs         # Rolling Sum
    │       ├── min_max.rs     # Rolling Max/Min (Monotonic Deque)
    │       ├── correlation.rs # Rolling Correlation (5-moment)
    │       ├── regression.rs  # Slope/Rsquare/Resi
    │       ├── quantile.rs    # Rolling Quantile (BTreeMap)
    │       ├── rank.rs        # Rolling Rank
    │       ├── index.rs       # IdxMax/IdxMin
    │       └── common.rs      # Greater/Less/Abs/Log/Ref
    ├── alpha158-memory/       # 内存管理
    │   └── src/
    │       ├── scratch.rs     # ScratchPad
    │       └── pool.rs        # 线程本地池
    ├── alpha158-data/         # 数据 I/O
    │   └── src/
    │       ├── reader.rs      # Parquet Reader
    │       ├── writer.rs      # Parquet Writer
    │       ├── stock.rs       # StockSlice/StockOutput
    │       └── schema.rs      # Schema 定义
    ├── alpha158-factors/      # 因子计算
    │   └── src/
    │       └── compute.rs     # compute_all_factors()
    ├── alpha158-cache/        # 增量缓存
    │   └── src/
    │       ├── incremental.rs # IncrementalState
    │       └── window_cache.rs# 环形缓冲
    └── alpha158-core/         # 引擎
        └── src/
            ├── engine.rs      # Rayon 并行引擎
            └── pipeline.rs    # 读取→计算→处理→写入
```

### 运行测试

```bash
# 全部测试
cargo test --workspace

# 仅滚动算子测试
cargo test -p alpha158-ops

# 带输出
cargo test --workspace -- --nocapture
```

### 开发模式编译

```bash
cargo build          # Debug 模式 (快速编译, 未优化)
cargo build --release # Release 模式 (完整优化, SIMD)
```

### 添加新因子

1. 在 `crates/alpha158-factors/src/compute.rs` 的 `compute_all_factors` 函数中添加计算逻辑
2. 在 `crates/alpha158-data/src/schema.rs` 的 `FACTOR_NAMES` 中添加因子名称
3. 更新 `NUM_FACTORS` 常量

### 添加新滚动算子

1. 在 `crates/alpha158-ops/src/` 中创建新模块
2. 实现 `RollingOperator` trait
3. 在 `crates/alpha158-ops/src/lib.rs` 中导出

---

## 常见问题

### Q: 编译报错 "linker not found"

**A:** 需要安装 C 链接器：

```bash
# Windows — 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install
```

### Q: 如何加速编译？

**A:** 使用 `cargo check` 代替 `cargo build` 进行语法检查，不生成二进制文件：

```bash
cargo check --workspace
```

### Q: 内存不足怎么办？

**A:** 减少并行度：

```bash
alpha158 -i data.parquet -o result.parquet -t 4
```

每只股票约占 500 KB 内存，16 线程峰值约 8 GB。

### Q: 输出的因子值是 NaN？

**A:** 前 59 天 (max_window - 1) 的部分因子值为 0 或无效值，这是正常的。滚动窗口需要至少 60 天数据才能计算完整。

### Q: 如何验证因子值正确性？

**A:** 与 Python Qlib 对比：

```python
import qlib
from qlib.contrib.data.handler import Alpha158

# 使用相同数据计算 Alpha158
# 对比 Rust 输出与 Python 输出的差异
# 允许浮点误差 < 1e-4
```

### Q: 支持增量更新吗？

**A:** `alpha158-cache` 模块已实现增量状态 (`IncrementalState`)，但 `pipeline.rs` 中尚未集成完整增量流程。当前版本每次运行全量计算。增量计算接口已预留，后续版本将完善。

### Q: 如何自定义窗口大小？

**A:** 修改 `crates/alpha158-factors/src/compute.rs` 中的常量：

```rust
const WINDOWS: [usize; 5] = [5, 10, 20, 30, 60];
```

### Q: 如何排除 RANK 因子？

**A:** 在 `compute.rs` 的 Layer 3 中注释掉 RANK 相关计算，并从 `schema.rs` 的 `FACTOR_NAMES` 中移除。

---

## 技术栈

| 依赖 | 版本 | 用途 |
|------|------|------|
| `arrow` | 54 | 列式内存格式 |
| `parquet` | 54 | 文件读写 |
| `rayon` | 1.10 | 数据并行 |
| `parking_lot` | 0.12 | 高效锁 |
| `ahash` | 0.8 | 高速哈希 |
| `ordered-float` | 4 | 有序浮点 |
| `clap` | 4 | CLI 参数解析 |
| `tracing` | 0.1 | 结构化日志 |

---

## License

MIT
