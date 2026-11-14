//! Bar → Parquet 写出（docs/06 §4.1 `bar` schema、§6 存储布局）。
//!
//! 落 `{n_root}/{venue}/bar/{kind}/{interval_tag}/{symbol}/{date}.parquet`：
//! 时间 Bar 的 interval_tag 是纳秒数（如 `60000000000` = 1m），成交量/
//! 成交额 Bar 的 interval_tag 是阈值（如 `vol100` / `notional1000000`）。
//! 与 trade 同 normalize_version 并存——Bar 是派生数据，随 trade 版本。

use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;

use crate::bar::{Bar, BarKind};
use crate::error::DataError;

/// Bar schema（docs/06 §4.1）。
fn bar_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("ts_open", DataType::Int64, false),
        Field::new("ts_close", DataType::Int64, false),
        Field::new("open", DataType::Int64, false),
        Field::new("high", DataType::Int64, false),
        Field::new("low", DataType::Int64, false),
        Field::new("close", DataType::Int64, false),
        Field::new("volume", DataType::Int64, false),
        // notional 可能超 i64（极端品种累计），用 Decimal128(38, 0) 存定点。
        Field::new("notional", DataType::Decimal128(38, 0), false),
        Field::new("trades", DataType::UInt32, false),
        Field::new("vwap", DataType::Float64, false),
    ]))
}

/// interval 目录标签（时间 Bar 用 ns 数，量/额 Bar 用 `volN` / `notionalN`）。
fn interval_tag(kind: BarKind, cfg_value: i64) -> String {
    match kind {
        BarKind::Time => cfg_value.to_string(),
        BarKind::Volume => format!("vol{cfg_value}"),
        BarKind::Notional => format!("notional{cfg_value}"),
    }
}

/// 写单品种单日的 Bar（无 Bar 时也落空表作凭证）。
pub fn write_bars(
    n_root: &Path,
    venue: &str,
    kind: BarKind,
    cfg_value: i64,
    symbol: &str,
    date: &str,
    bars: &[Bar],
) -> Result<usize, DataError> {
    let dir = n_root
        .join(venue)
        .join("bar")
        .join(kind.as_str())
        .join(interval_tag(kind, cfg_value))
        .join(symbol);
    std::fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let path = dir.join(format!("{date}.parquet"));
    let file =
        std::fs::File::create(&path).map_err(|e| DataError::Parquet(format!("创建失败：{e}")))?;

    let schema = bar_schema();
    let cols = bar_columns(bars);
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("bar batch：{e}")))?;
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
    Ok(bars.len())
}

/// Bar → Arrow 列（notional 用 Decimal128）。
fn bar_columns(bars: &[Bar]) -> Vec<ArrayRef> {
    vec![
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.ts_open))),
        Arc::new(Int64Array::from_iter_values(
            bars.iter().map(|b| b.ts_close),
        )),
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.open))),
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.high))),
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.low))),
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.close))),
        Arc::new(Int64Array::from_iter_values(bars.iter().map(|b| b.volume))),
        Arc::new(
            arrow::array::Decimal128Array::from_iter_values(bars.iter().map(|b| b.notional))
                .with_precision_and_scale(38, 0)
                .expect("precision 38 is valid"),
        ),
        Arc::new(UInt32Array::from_iter_values(bars.iter().map(|b| b.trades))),
        Arc::new(Float64Array::from_iter_values(bars.iter().map(|b| b.vwap))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::BarCfg;
    use crate::clean::CFlags;
    use crate::normalize::TradeRow;

    fn trade(ts: i64, price: i64, qty: i64) -> (TradeRow, CFlags) {
        (
            TradeRow {
                ts_exch: ts,
                ts_recv: ts + 100,
                trade_id: ts as u64,
                price,
                qty,
                aggressor_side: 0,
                conn_id: "A".into(),
            },
            CFlags::default(),
        )
    }

    /// 写出 → 读回行数与内容一致。
    #[test]
    fn bars_roundtrip() {
        let dir = std::env::temp_dir().join("so-data-bar-out");
        let _ = std::fs::remove_dir_all(&dir);
        let trades = vec![trade(1_000, 100, 2), trade(2_000, 105, 3)];
        let bars = crate::bar::generate_bars(&trades, &BarCfg::time(60_000_000_000));
        let n = write_bars(
            &dir,
            "gate",
            BarKind::Time,
            60_000_000_000,
            "BTC_USDT",
            "2026-11-12",
            &bars,
        )
        .unwrap();
        assert_eq!(n, 1);
        let path = dir
            .join("gate")
            .join("bar")
            .join("time")
            .join("60000000000")
            .join("BTC_USDT")
            .join("2026-11-12.parquet");
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 空 Bar 也落表（凭证）。
    #[test]
    fn empty_bars_still_written() {
        let dir = std::env::temp_dir().join("so-data-bar-out-empty");
        let _ = std::fs::remove_dir_all(&dir);
        let n = write_bars(
            &dir,
            "gate",
            BarKind::Volume,
            100,
            "BTC_USDT",
            "2026-11-12",
            &[],
        )
        .unwrap();
        assert_eq!(n, 0);
        let path = dir
            .join("gate")
            .join("bar")
            .join("volume")
            .join("vol100")
            .join("BTC_USDT")
            .join("2026-11-12.parquet");
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
