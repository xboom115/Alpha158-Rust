//! Parquet Writer — 直接从 Vec<f32> 构建 Arrow RecordBatch, 零拷贝写入

use crate::schema::{output_schema, output_schema_string_date, NUM_FACTORS};
use crate::stock::StockOutput;
use arrow::array::{ArrayRef, Date32Array, Float32Array, StringArray};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::sync::Arc;

/// Date32 (days since 1970-01-01) 转 "YYYY-MM-DD" 字符串
fn date32_to_string(days: i32) -> String {
    // 1970-01-01 + days
    let _ts = days as i64 * 86400;
    // 简单计算: 直接用 chrono-like 算法
    let d = days as i64;
    // 儒略日 → 年月日
    let jd = d + 2440588;
    let l = jd + 68569;
    let n = 4 * l / 146097;
    let l2 = l - (146097 * n + 3) / 4;
    let y = 4000 * (l2 + 1) / 1461001;
    let l3 = l2 - 1461 * y / 4 + 31;
    let m = 80 * l3 / 2447;
    let day = l3 - 2447 * m / 80;
    let l4 = m / 11;
    let month = m + 2 - 12 * l4;
    let year = 100 * (n - 49) + y + l4;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// 将多个 StockOutput 写入 Parquet 文件
///
/// `date_as_string`: true 时日期输出为 "YYYY-MM-DD" 字符串 (兼容 ParquetViewer)
pub fn write_parquet(
    outputs: &[StockOutput],
    path: &str,
    date_as_string: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_write_batch_size(65536)
        .set_max_row_group_size(65536)
        .set_created_by("alpha158-engine".to_string())
        .set_dictionary_enabled(false) // 禁用字典编码, 兼容 .NET ParquetViewer
        .build();

    let schema = if date_as_string {
        Arc::new(output_schema_string_date())
    } else {
        Arc::new(output_schema())
    };
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;

    // 计算总行数
    let total_rows: usize = outputs.iter().map(|o| o.dates.len()).sum();

    // 构建 date 列
    let date_arr: ArrayRef = if date_as_string {
        let mut dates_vec = Vec::with_capacity(total_rows);
        for output in outputs {
            for &d in &output.dates {
                dates_vec.push(date32_to_string(d));
            }
        }
        Arc::new(StringArray::from(dates_vec))
    } else {
        let mut dates_vec = Vec::with_capacity(total_rows);
        for output in outputs {
            dates_vec.extend_from_slice(&output.dates);
        }
        Arc::new(Date32Array::from(dates_vec))
    };

    // 构建 code 列
    let mut codes_vec = Vec::with_capacity(total_rows);
    for output in outputs {
        for _ in 0..output.dates.len() {
            codes_vec.push(output.code.as_str());
        }
    }
    let code_arr: ArrayRef = Arc::new(StringArray::from(codes_vec));

    // 构建因子列 (直接从 Vec<f32> 转换, 零拷贝)
    let mut columns: Vec<ArrayRef> = vec![date_arr, code_arr];

    for factor_idx in 0..NUM_FACTORS {
        let mut values = Vec::with_capacity(total_rows);
        for output in outputs {
            if factor_idx < output.data.len() {
                values.extend_from_slice(&output.data[factor_idx]);
            } else {
                values.resize(values.len() + output.dates.len(), 0.0);
            }
        }
        columns.push(Arc::new(Float32Array::from(values)));
    }

    let batch = RecordBatch::try_new(schema, columns)?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(())
}
