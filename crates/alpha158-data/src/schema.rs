//! Schema 定义 — 输入/输出 Parquet Schema

use arrow::datatypes::{DataType, Field, Schema};

/// Alpha158 因子名称列表 (默认配置, 排除 RANK = 158 个)
pub const FACTOR_NAMES: &[&str] = &[
    // KBar (9)
    "KMID", "KLEN", "KMID2", "KUP", "KUP2", "KLOW", "KLOW2", "KSFT", "KSFT2",
    // Price (4)
    "OPEN0", "HIGH0", "LOW0", "VWAP0",
    // ROC (5)
    "ROC5", "ROC10", "ROC20", "ROC30", "ROC60",
    // MA (5)
    "MA5", "MA10", "MA20", "MA30", "MA60",
    // STD (5)
    "STD5", "STD10", "STD20", "STD30", "STD60",
    // BETA (5)
    "BETA5", "BETA10", "BETA20", "BETA30", "BETA60",
    // RSQR (5)
    "RSQR5", "RSQR10", "RSQR20", "RSQR30", "RSQR60",
    // RESI (5)
    "RESI5", "RESI10", "RESI20", "RESI30", "RESI60",
    // MAX (5)
    "MAX5", "MAX10", "MAX20", "MAX30", "MAX60",
    // MIN (5)
    "MIN5", "MIN10", "MIN20", "MIN30", "MIN60",
    // QTLU (5)
    "QTLU5", "QTLU10", "QTLU20", "QTLU30", "QTLU60",
    // QTLD (5)
    "QTLD5", "QTLD10", "QTLD20", "QTLD30", "QTLD60",
    // RANK (5)
    "RANK5", "RANK10", "RANK20", "RANK30", "RANK60",
    // RSV (5)
    "RSV5", "RSV10", "RSV20", "RSV30", "RSV60",
    // IMAX (5)
    "IMAX5", "IMAX10", "IMAX20", "IMAX30", "IMAX60",
    // IMIN (5)
    "IMIN5", "IMIN10", "IMIN20", "IMIN30", "IMIN60",
    // IMXD (5)
    "IMXD5", "IMXD10", "IMXD20", "IMXD30", "IMXD60",
    // CORR (5)
    "CORR5", "CORR10", "CORR20", "CORR30", "CORR60",
    // CORD (5)
    "CORD5", "CORD10", "CORD20", "CORD30", "CORD60",
    // CNTP (5)
    "CNTP5", "CNTP10", "CNTP20", "CNTP30", "CNTP60",
    // CNTN (5)
    "CNTN5", "CNTN10", "CNTN20", "CNTN30", "CNTN60",
    // CNTD (5)
    "CNTD5", "CNTD10", "CNTD20", "CNTD30", "CNTD60",
    // SUMP (5)
    "SUMP5", "SUMP10", "SUMP20", "SUMP30", "SUMP60",
    // SUMN (5)
    "SUMN5", "SUMN10", "SUMN20", "SUMN30", "SUMN60",
    // SUMD (5)
    "SUMD5", "SUMD10", "SUMD20", "SUMD30", "SUMD60",
    // VMA (5)
    "VMA5", "VMA10", "VMA20", "VMA30", "VMA60",
    // VSTD (5)
    "VSTD5", "VSTD10", "VSTD20", "VSTD30", "VSTD60",
    // WVMA (5)
    "WVMA5", "WVMA10", "WVMA20", "WVMA30", "WVMA60",
    // VSUMP (5)
    "VSUMP5", "VSUMP10", "VSUMP20", "VSUMP30", "VSUMP60",
    // VSUMN (5)
    "VSUMN5", "VSUMN10", "VSUMN20", "VSUMN30", "VSUMN60",
    // VSUMD (5)
    "VSUMD5", "VSUMD10", "VSUMD20", "VSUMD30", "VSUMD60",
];

pub const NUM_FACTORS: usize = 158;

/// 输入 Parquet Schema
pub fn input_schema() -> Schema {
    Schema::new(vec![
        Field::new("date", DataType::Date32, false),
        Field::new("code", DataType::Utf8, false),
        Field::new("open", DataType::Float32, false),
        Field::new("high", DataType::Float32, false),
        Field::new("low", DataType::Float32, false),
        Field::new("close", DataType::Float32, false),
        Field::new("volume", DataType::Float32, false),
        Field::new("vwap", DataType::Float32, false),
    ])
}

/// 输出 Parquet Schema (Date32 日期)
pub fn output_schema() -> Schema {
    let mut fields = vec![
        Field::new("date", DataType::Date32, false),
        Field::new("code", DataType::Utf8, false),
    ];
    for name in FACTOR_NAMES {
        fields.push(Field::new(*name, DataType::Float32, false));
    }
    Schema::new(fields)
}

/// 输出 Parquet Schema (字符串日期, 兼容 ParquetViewer)
pub fn output_schema_string_date() -> Schema {
    let mut fields = vec![
        Field::new("date", DataType::Utf8, false),
        Field::new("code", DataType::Utf8, false),
    ];
    for name in FACTOR_NAMES {
        fields.push(Field::new(*name, DataType::Float32, false));
    }
    Schema::new(fields)
}
