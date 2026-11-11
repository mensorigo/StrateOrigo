//! `instrument` 时点表写出（docs/06 §4.1）。从 `parquet_out` 拆出，
//! 保持每个文件只负责一件事（docs/04 F2）；全局一张表，不按品种分目录。

use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;

use crate::error::DataError;
use crate::normalize::InstrumentRow;

/// 写 `instrument/{YYYY-MM-DD}.parquet`（zstd 压缩，按 `valid_from` 升序）。
pub fn write_instrument(
    out_dir: &Path,
    rows: &[InstrumentRow],
    date: &str,
) -> Result<usize, DataError> {
    let n = rows.len();
    if n == 0 {
        return Ok(0);
    }
    let schema = instrument_schema();
    let batch = build_batch(rows, schema.clone())?;
    let dir = out_dir.join("instrument");
    std::fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let file = std::fs::File::create(dir.join(format!("{date}.parquet")))
        .map_err(|e| DataError::Parquet(format!("创建失败：{e}")))?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    writer
        .write(&batch)
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    writer
        .close()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    Ok(n)
}

/// instrument 行 → RecordBatch（排序 + 列构造，与写出 I/O 分离）。
fn build_batch(rows: &[InstrumentRow], schema: Arc<Schema>) -> Result<RecordBatch, DataError> {
    let mut sorted: Vec<&InstrumentRow> = rows.iter().collect();
    sorted.sort_by(|a, b| (a.valid_from, &a.symbol).cmp(&(b.valid_from, &b.symbol)));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.valid_from),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.valid_to),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.venue.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.symbol.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.inst_type.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.tick_size.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.lot_size.as_str()),
        )),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.min_notional))),
        Arc::new(arrow::array::Int32Array::from_iter_values(
            sorted.iter().map(|r| r.price_scale),
        )),
        Arc::new(arrow::array::Int32Array::from_iter_values(
            sorted.iter().map(|r| r.qty_scale),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.status.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.contract_size.as_str()),
        )),
    ];
    RecordBatch::try_new(schema, cols)
        .map_err(|e| DataError::Parquet(format!("instrument batch：{e}")))
}

/// `instrument` 表 schema。
fn instrument_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("valid_from", DataType::Int64, false),
        Field::new("valid_to", DataType::Int64, false),
        Field::new("venue", DataType::Utf8, false),
        Field::new("symbol", DataType::Utf8, false),
        Field::new("inst_type", DataType::Utf8, false),
        Field::new("tick_size", DataType::Utf8, false),
        Field::new("lot_size", DataType::Utf8, false),
        Field::new("min_notional", DataType::Int64, true),
        Field::new("price_scale", DataType::Int32, false),
        Field::new("qty_scale", DataType::Int32, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("contract_size", DataType::Utf8, false),
    ]))
}
