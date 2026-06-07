//! Alpha158 数据 I/O — Parquet 读写, StockSlice 管理

pub mod reader;
pub mod schema;
pub mod stock;
pub mod writer;

pub use stock::StockSlice;
