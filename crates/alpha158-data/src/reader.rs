//! Parquet Reader — 读取 Parquet 文件, 按 code 分组为 StockSlice
//!
//! 支持 Float32 / Float64 / Int32 / Int64 列, 自动转换为 f32.

use crate::stock::StockSlice;
use ahash::AHashMap;
use arrow::array::{
    Array, Date32Array, Float32Array, Float64Array, Int32Array, Int64Array, StringArray,
};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;

/// 已排序的股票数据 (按 code 分组, 每只股票内按 date 排序)
pub struct StockData {
    pub codes: Vec<String>,
    pub dates: Vec<Vec<i32>>,
    pub open: Vec<Vec<f32>>,
    pub high: Vec<Vec<f32>>,
    pub low: Vec<Vec<f32>>,
    pub close: Vec<Vec<f32>>,
    pub volume: Vec<Vec<f32>>,
    pub vwap: Vec<Vec<f32>>,
}

impl StockData {
    /// 获取第 i 只股票的 StockSlice
    pub fn slice(&self, i: usize) -> StockSlice<'_> {
        StockSlice {
            code: &self.codes[i],
            n: self.dates[i].len(),
            dates: &self.dates[i],
            open: &self.open[i],
            high: &self.high[i],
            low: &self.low[i],
            close: &self.close[i],
            volume: &self.volume[i],
            vwap: &self.vwap[i],
        }
    }

    pub fn num_stocks(&self) -> usize {
        self.codes.len()
    }
}

/// 从 RecordBatch 的第 col_idx 列提取 f32 值, 支持多种数值类型
fn extract_f32_column(batch: &RecordBatch, col_idx: usize) -> Result<Vec<f32>, String> {
    let col = batch.column(col_idx);
    let n = batch.num_rows();

    // 尝试 Float32
    if let Some(arr) = col.as_any().downcast_ref::<Float32Array>() {
        return Ok((0..n).map(|i| arr.value(i)).collect());
    }
    // 尝试 Float64
    if let Some(arr) = col.as_any().downcast_ref::<Float64Array>() {
        return Ok((0..n).map(|i| arr.value(i) as f32).collect());
    }
    // 尝试 Int32
    if let Some(arr) = col.as_any().downcast_ref::<Int32Array>() {
        return Ok((0..n).map(|i| arr.value(i) as f32).collect());
    }
    // 尝试 Int64
    if let Some(arr) = col.as_any().downcast_ref::<Int64Array>() {
        return Ok((0..n).map(|i| arr.value(i) as f32).collect());
    }

    Err(format!(
        "column {} has unsupported type: {:?}",
        col_idx,
        col.data_type()
    ))
}

/// 日期字符串转 Date32 (days since 1970-01-01)
fn date_str_to_date32(s: &str) -> i32 {
    // 支持格式: "2024-01-15", "20240115", "2024/01/15"
    let s = s.trim();
    let (y, m, d) = if s.len() == 10 && s.as_bytes()[4] == b'-' {
        // "2024-01-15"
        let y: i32 = s[0..4].parse().unwrap_or(1970);
        let m: i32 = s[5..7].parse().unwrap_or(1);
        let d: i32 = s[8..10].parse().unwrap_or(1);
        (y, m, d)
    } else if s.len() == 10 && s.as_bytes()[4] == b'/' {
        // "2024/01/15"
        let y: i32 = s[0..4].parse().unwrap_or(1970);
        let m: i32 = s[5..7].parse().unwrap_or(1);
        let d: i32 = s[8..10].parse().unwrap_or(1);
        (y, m, d)
    } else if s.len() == 8 {
        // "20240115"
        let y: i32 = s[0..4].parse().unwrap_or(1970);
        let m: i32 = s[4..6].parse().unwrap_or(1);
        let d: i32 = s[6..8].parse().unwrap_or(1);
        (y, m, d)
    } else {
        return 0;
    };
    // 转为 days since 1970-01-01 (与 Arrow Date32 一致)
    let mut days = (y - 1970) * 365 + (y - 1969) / 4 - (y - 1901) / 100 + (y - 1601) / 400;
    let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    days += month_days[(m - 1) as usize] + d - 1;
    if m > 2 && (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) {
        days += 1;
    }
    days
}

/// 从 RecordBatch 的第 col_idx 列提取 Date32 值
/// 支持: Date32, Int32, Utf8 (字符串日期)
fn extract_date32_column(batch: &RecordBatch, col_idx: usize) -> Result<Vec<i32>, String> {
    let col = batch.column(col_idx);
    let n = batch.num_rows();

    // Date32
    if let Some(arr) = col.as_any().downcast_ref::<Date32Array>() {
        return Ok((0..n).map(|i| arr.value(i)).collect());
    }
    // Int32
    if let Some(arr) = col.as_any().downcast_ref::<Int32Array>() {
        return Ok((0..n).map(|i| arr.value(i)).collect());
    }
    // Int64
    if let Some(arr) = col.as_any().downcast_ref::<arrow::array::Int64Array>() {
        return Ok((0..n).map(|i| arr.value(i) as i32).collect());
    }
    // Utf8 (字符串日期: "2024-01-15", "20240115", "2024/01/15")
    if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
        return Ok((0..n).map(|i| date_str_to_date32(arr.value(i))).collect());
    }

    Err(format!(
        "column {} unsupported date type: {:?}",
        col_idx,
        col.data_type()
    ))
}

/// 按列名查找列索引 (支持单个名称)
fn find_column(schema: &arrow::datatypes::Schema, name: &str) -> Option<usize> {
    schema
        .fields()
        .iter()
        .position(|f| f.name().eq_ignore_ascii_case(name))
}

/// 按多个别名查找列索引 (返回第一个匹配)
fn find_column_any(
    schema: &arrow::datatypes::Schema,
    aliases: &[&str],
) -> Option<usize> {
    for alias in aliases {
        if let Some(idx) = find_column(schema, alias) {
            return Some(idx);
        }
    }
    None
}

/// 从 Parquet 文件读取数据并按 code 分组
///
/// 支持的列名别名:
///   日期: date, Date, DATE, trade_date, datetime
///   代码: code, Code, stock, Stock, symbol, ticker, instrument
///   开盘: open, Open, OPEN
///   最高: high, High, HIGH
///   最低: low, Low, LOW
///   收盘: close, Close, CLOSE
///   成交量: volume, Volume, VOL, vol
///   均价: vwap, VWAP, avg_price
///
/// `max_days`: 如果指定, 每只股票只保留最后 N 天数据
pub fn read_parquet(
    path: &str,
    max_days: Option<usize>,
) -> Result<StockData, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let schema = builder.schema().clone();
    let reader = builder.build()?;

    // 按列名别名查找索引
    let date_idx = find_column_any(&schema, &["date", "Date", "DATE", "trade_date", "datetime"])
        .ok_or("missing date column (tried: date, trade_date, datetime)")?;
    let code_idx = find_column_any(&schema, &["code", "Code", "stock", "Stock", "symbol", "ticker", "instrument"])
        .ok_or("missing code column (tried: code, stock, symbol, ticker, instrument)")?;
    let open_idx = find_column_any(&schema, &["open", "Open", "OPEN"])
        .ok_or("missing open column")?;
    let high_idx = find_column_any(&schema, &["high", "High", "HIGH"])
        .ok_or("missing high column")?;
    let low_idx = find_column_any(&schema, &["low", "Low", "LOW"])
        .ok_or("missing low column")?;
    let close_idx = find_column_any(&schema, &["close", "Close", "CLOSE"])
        .ok_or("missing close column")?;
    let volume_idx = find_column_any(&schema, &["volume", "Volume", "VOL", "vol"])
        .ok_or("missing volume column")?;
    let vwap_idx = find_column_any(&schema, &["vwap", "VWAP", "avg_price", "AvgPrice"])
        .ok_or("missing vwap column (tried: vwap, VWAP, avg_price)")?;

    let mut all_dates = Vec::new();
    let mut all_codes = Vec::new();
    let mut all_open = Vec::new();
    let mut all_high = Vec::new();
    let mut all_low = Vec::new();
    let mut all_close = Vec::new();
    let mut all_volume = Vec::new();
    let mut all_vwap = Vec::new();

    for batch_result in reader {
        let batch: RecordBatch = batch_result?;

        let codes_col: &StringArray = batch
            .column(code_idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or("code column is not Utf8")?;

        let dates_vec = extract_date32_column(&batch, date_idx)?;
        let open_vec = extract_f32_column(&batch, open_idx)?;
        let high_vec = extract_f32_column(&batch, high_idx)?;
        let low_vec = extract_f32_column(&batch, low_idx)?;
        let close_vec = extract_f32_column(&batch, close_idx)?;
        let volume_vec = extract_f32_column(&batch, volume_idx)?;
        let vwap_vec = extract_f32_column(&batch, vwap_idx)?;

        for i in 0..batch.num_rows() {
            all_dates.push(dates_vec[i]);
            all_codes.push(codes_col.value(i).to_string());
            all_open.push(open_vec[i]);
            all_high.push(high_vec[i]);
            all_low.push(low_vec[i]);
            all_close.push(close_vec[i]);
            all_volume.push(volume_vec[i]);
            all_vwap.push(vwap_vec[i]);
        }
    }

    // 按 code 分组
    let mut groups: AHashMap<String, Vec<usize>> = AHashMap::new();
    for (i, code) in all_codes.iter().enumerate() {
        groups.entry(code.clone()).or_default().push(i);
    }

    let mut codes: Vec<String> = groups.keys().cloned().collect();
    codes.sort();

    // 每只股票内按日期排序, 然后截取最后 max_days 天
    for indices in groups.values_mut() {
        indices.sort_by_key(|&i| all_dates[i]);
        if let Some(max_d) = max_days {
            if indices.len() > max_d {
                let skip = indices.len() - max_d;
                *indices = indices[skip..].to_vec();
            }
        }
    }

    let mut dates = Vec::with_capacity(codes.len());
    let mut open = Vec::with_capacity(codes.len());
    let mut high = Vec::with_capacity(codes.len());
    let mut low = Vec::with_capacity(codes.len());
    let mut close = Vec::with_capacity(codes.len());
    let mut volume = Vec::with_capacity(codes.len());
    let mut vwap = Vec::with_capacity(codes.len());

    for code in &codes {
        let indices = &groups[code];
        dates.push(indices.iter().map(|&i| all_dates[i]).collect());
        open.push(indices.iter().map(|&i| all_open[i]).collect());
        high.push(indices.iter().map(|&i| all_high[i]).collect());
        low.push(indices.iter().map(|&i| all_low[i]).collect());
        close.push(indices.iter().map(|&i| all_close[i]).collect());
        volume.push(indices.iter().map(|&i| all_volume[i]).collect());
        vwap.push(indices.iter().map(|&i| all_vwap[i]).collect());
    }

    Ok(StockData {
        codes,
        dates,
        open,
        high,
        low,
        close,
        volume,
        vwap,
    })
}
