//! 表行 → Parquet 写出（docs/06 §4.1 schema、§6 存储布局）。
//!
//! `out_dir` 由调用方预先拼好版本与交易所段（`{root}/{n版本}/{venue}`），
//! 本模块落 `{out_dir}/{table}/{symbol}/{YYYY-MM-DD}.parquet`，zstd 压缩，
//! 按 `ts_recv` 排序。列与 [`crate::normalize`] 行体一一对应。
//!
//! 精度说明：适配器解析 API 是单 scale，价格与数量都按品种 tick 精度
//! 解析存储，因此文件元数据 `price_scale` 与 `qty_scale` 同值——元数据
//! 如实描述所存定点整数；数量"天然精度"与价格精度解耦属后续版本
//! （见 `contracts.rs` 的精度取舍说明）。

use std::collections::HashMap;
use std::fs;
// 仅数据表的 Arrow/Parquet 列构造；instrument 时点表见 instrument_out.rs。
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;

use crate::error::DataError;
use crate::normalize::{BboRow, BookDeltaRow, MarkPriceRow, TradeRow};

/// Parquet 文件元数据键：价格精度。
pub const META_PRICE_SCALE: &str = "price_scale";
/// Parquet 文件元数据键：数量精度。
pub const META_QTY_SCALE: &str = "qty_scale";

/// book_delta 表 schema（docs/06 §4.1）。
pub fn book_delta_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("ts_exch", DataType::Int64, false),
        Field::new("ts_recv", DataType::Int64, false),
        Field::new("seq_first", DataType::UInt64, false),
        Field::new("seq_last", DataType::UInt64, false),
        Field::new("seq_prev", DataType::UInt64, false),
        Field::new("side", DataType::Int32, false),
        Field::new("price", DataType::Int64, false),
        Field::new("qty", DataType::Int64, false),
        Field::new("conn_id", DataType::Utf8, false),
    ]))
}

/// 写 book_delta 行组（单品种单文件）。
pub fn write_book_delta(
    out_dir: &Path,
    symbol: &str,
    rows: &[BookDeltaRow],
    date: &str,
    price_scale: u8,
    qty_scale: u8,
) -> Result<usize, DataError> {
    let n = rows.len();
    if n == 0 {
        return Ok(0);
    }
    let schema = book_delta_schema();
    // 按 ts_recv 排序（docs/06 §6），同刻保序（稳定排序保证确定性）。
    let mut sorted: Vec<&BookDeltaRow> = rows.iter().collect();
    sorted.sort_by_key(|r| (r.ts_recv, r.ts_exch));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_exch),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_recv),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|r| r.seq_first),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|r| r.seq_last),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|r| r.seq_prev),
        )),
        Arc::new(arrow::array::Int32Array::from_iter_values(
            sorted.iter().map(|r| r.side),
        )),
        Arc::new(Int64Array::from_iter_values(sorted.iter().map(|r| r.price))),
        Arc::new(Int64Array::from_iter_values(sorted.iter().map(|r| r.qty))),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.conn_id.as_str()),
        )),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("book_delta batch：{e}")))?;
    write_batch(
        out_dir,
        "book_delta",
        symbol,
        date,
        schema,
        vec![batch],
        price_scale,
        qty_scale,
    )?;
    Ok(n)
}

/// trade 表写出。
pub fn write_trade(
    out_dir: &Path,
    symbol: &str,
    rows: &[TradeRow],
    date: &str,
    price_scale: u8,
    qty_scale: u8,
) -> Result<usize, DataError> {
    let n = rows.len();
    if n == 0 {
        return Ok(0);
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("ts_exch", DataType::Int64, false),
        Field::new("ts_recv", DataType::Int64, false),
        Field::new("trade_id", DataType::UInt64, false),
        Field::new("price", DataType::Int64, false),
        Field::new("qty", DataType::Int64, false),
        Field::new("aggressor_side", DataType::Int32, false),
        Field::new("conn_id", DataType::Utf8, false),
    ]));
    let mut sorted: Vec<&TradeRow> = rows.iter().collect();
    sorted.sort_by_key(|r| (r.ts_recv, r.trade_id));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_exch),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_recv),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|r| r.trade_id),
        )),
        Arc::new(Int64Array::from_iter_values(sorted.iter().map(|r| r.price))),
        Arc::new(Int64Array::from_iter_values(sorted.iter().map(|r| r.qty))),
        Arc::new(arrow::array::Int32Array::from_iter_values(
            sorted.iter().map(|r| r.aggressor_side),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.conn_id.as_str()),
        )),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("trade batch：{e}")))?;
    write_batch(
        out_dir,
        "trade",
        symbol,
        date,
        schema,
        vec![batch],
        price_scale,
        qty_scale,
    )?;
    Ok(n)
}

/// bbo 表写出。
pub fn write_bbo(
    out_dir: &Path,
    symbol: &str,
    rows: &[BboRow],
    date: &str,
    price_scale: u8,
    qty_scale: u8,
) -> Result<usize, DataError> {
    let n = rows.len();
    if n == 0 {
        return Ok(0);
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("ts_exch", DataType::Int64, false),
        Field::new("ts_recv", DataType::Int64, false),
        Field::new("seq", DataType::UInt64, false),
        Field::new("bid_px", DataType::Int64, true),
        Field::new("bid_qty", DataType::Int64, true),
        Field::new("ask_px", DataType::Int64, true),
        Field::new("ask_qty", DataType::Int64, true),
        Field::new("conn_id", DataType::Utf8, false),
    ]));
    let mut sorted: Vec<&BboRow> = rows.iter().collect();
    sorted.sort_by_key(|r| (r.ts_recv, r.seq));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_exch),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_recv),
        )),
        Arc::new(UInt64Array::from_iter_values(sorted.iter().map(|r| r.seq))),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.bid_px))),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.bid_qty))),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.ask_px))),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.ask_qty))),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.conn_id.as_str()),
        )),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("bbo batch：{e}")))?;
    write_batch(
        out_dir,
        "bbo",
        symbol,
        date,
        schema,
        vec![batch],
        price_scale,
        qty_scale,
    )?;
    Ok(n)
}

/// mark_price 表写出。
pub fn write_mark_price(
    out_dir: &Path,
    symbol: &str,
    rows: &[MarkPriceRow],
    date: &str,
    price_scale: u8,
    qty_scale: u8,
) -> Result<usize, DataError> {
    let n = rows.len();
    if n == 0 {
        return Ok(0);
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("ts_exch", DataType::Int64, false),
        Field::new("ts_recv", DataType::Int64, false),
        Field::new("mark_px", DataType::Int64, false),
        Field::new("index_px", DataType::Int64, true),
        Field::new("rate_e8", DataType::Int64, false),
        Field::new("conn_id", DataType::Utf8, false),
    ]));
    let mut sorted: Vec<&MarkPriceRow> = rows.iter().collect();
    sorted.sort_by_key(|r| (r.ts_recv, r.ts_exch));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_exch),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.ts_recv),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.mark_px),
        )),
        Arc::new(Int64Array::from_iter(sorted.iter().map(|r| r.index_px))),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|r| r.rate_e8),
        )),
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|r| r.conn_id.as_str()),
        )),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("mark_price batch：{e}")))?;
    write_batch(
        out_dir,
        "mark_price",
        symbol,
        date,
        schema,
        vec![batch],
        price_scale,
        qty_scale,
    )?;
    Ok(n)
}

/// 落一个批次集合到 `{out_dir}/{table}/{symbol}/{date}.parquet`。
///
/// `date` 由调用方从 ts_recv 推（本工具按天处理，一天一文件）。
#[allow(clippy::too_many_arguments)]
fn write_batch(
    out_dir: &Path,
    table: &str,
    symbol: &str,
    date: &str,
    schema: Arc<Schema>,
    batches: Vec<RecordBatch>,
    price_scale: u8,
    qty_scale: u8,
) -> Result<(), DataError> {
    let dir = out_dir.join(table).join(symbol);
    fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let path = dir.join(format!("{date}.parquet"));
    let file = fs::File::create(&path).map_err(|e| DataError::Parquet(format!("创建失败：{e}")))?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .set_key_value_metadata(Some(vec![
            parquet::file::metadata::KeyValue::new(
                META_PRICE_SCALE.to_owned(),
                Some(price_scale.to_string()),
            ),
            parquet::file::metadata::KeyValue::new(
                META_QTY_SCALE.to_owned(),
                Some(qty_scale.to_string()),
            ),
        ]))
        .build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    for b in &batches {
        writer
            .write(b)
            .map_err(|e| DataError::Parquet(e.to_string()))?;
    }
    writer
        .close()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    Ok(())
}

/// 全部表一次落盘（品种 → 行缓冲 → 各表 parquet）。
///
/// `date` 决定输出文件名与 `normalize_version` 目录（由调用方组装）。
pub fn write_all(
    out_dir: &Path,
    tables: &crate::normalize::Tables,
    scales: &HashMap<String, so_core::Scale>,
    date: &str,
) -> Result<usize, DataError> {
    let mut total = 0;
    for (symbol, rows) in &tables.book_delta {
        let (ps, qs) = scales_for(scales, symbol);
        total += write_book_delta(out_dir, symbol, rows, date, ps, qs)?;
    }
    for (symbol, rows) in &tables.trade {
        let (ps, qs) = scales_for(scales, symbol);
        total += write_trade(out_dir, symbol, rows, date, ps, qs)?;
    }
    for (symbol, rows) in &tables.bbo {
        let (ps, qs) = scales_for(scales, symbol);
        total += write_bbo(out_dir, symbol, rows, date, ps, qs)?;
    }
    for (symbol, rows) in &tables.mark_price {
        let (ps, qs) = scales_for(scales, symbol);
        total += write_mark_price(out_dir, symbol, rows, date, ps, qs)?;
    }
    Ok(total)
}

/// 品种精度（缺元数据回退 4/4 并告警——调用方应先保证元数据齐全）。
///
/// 价格与数量返回同值：解析阶段两者共用品种 tick 精度（见模块注释）。
fn scales_for(scales: &HashMap<String, so_core::Scale>, symbol: &str) -> (u8, u8) {
    match scales.get(symbol) {
        Some(s) => (s.decimals(), s.decimals()),
        None => {
            tracing::warn!(symbol, "缺精度元数据，回退 4/4");
            (4, 4)
        }
    }
}
