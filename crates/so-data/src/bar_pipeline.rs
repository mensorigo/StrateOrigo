//! Bar 生成编排：读 trade Parquet → 聚合 → 写 Bar Parquet（docs/06 §8）。
//!
//! 输入是 normalize 产物（`{n_root}/{venue}/trade/{symbol}/{date}.parquet`），
//! 输出落 [`crate::bar_out::write_bars`]。算法在 [`crate::bar`]（I8：
//! 编排层不做算法，算法层不做 I/O）。

use std::path::Path;

use arrow::array::Int64Array;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::bar::{BarCfg, generate_bars};
use crate::bar_out::write_bars;
use crate::clean::CFlags;
use crate::error::DataError;
use crate::normalize::TradeRow;

/// Bar 生成统计。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BarStats {
    /// 读入的 trade 行数。
    pub trade_rows: u64,
    /// 生成的 Bar 数。
    pub bars: u64,
}

/// 从 trade Parquet 生成 Bar 并写出（遍历全部交易所）。
///
/// `n_root` 是 normalize 版本根（如 `data/parquet/n1`）。遍历
/// `{n_root}/{venue}/trade/{symbol}/{date}.parquet`，对每个品种按 `cfg`
/// 聚合并写出。返回统计。
pub fn generate_day_bars(n_root: &Path, date: &str, cfg: &BarCfg) -> Result<BarStats, DataError> {
    let mut stats = BarStats::default();
    for venue in list_venues(n_root)? {
        generate_venue_bars(n_root, &venue, date, cfg, &mut stats)?;
    }
    Ok(stats)
}

/// 列出版本根下的交易所目录（排除 quality_* / bar 派生目录）。
fn list_venues(n_root: &Path) -> Result<Vec<String>, DataError> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(n_root) else {
        return Ok(out);
    };
    for e in rd.flatten() {
        if e.path().is_dir()
            && let Some(name) = e.file_name().to_str()
            && !name.starts_with("quality_")
            && name != "bar"
        {
            out.push(name.to_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// 单交易所的 Bar 生成（遍历品种目录）。
fn generate_venue_bars(
    n_root: &Path,
    venue: &str,
    date: &str,
    cfg: &BarCfg,
    stats: &mut BarStats,
) -> Result<(), DataError> {
    let trade_dir = n_root.join(venue).join("trade");
    let Ok(rd) = std::fs::read_dir(&trade_dir) else {
        return Ok(()); // 无 trade 目录 = 当天无数据
    };
    for sym_dir in rd.flatten() {
        if !sym_dir.path().is_dir() {
            continue;
        }
        let Some(symbol) = sym_dir.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let path = sym_dir.path().join(format!("{date}.parquet"));
        let Ok(file) = std::fs::File::open(&path) else {
            continue;
        };
        let trades = read_trades(&file)?;
        stats.trade_rows += trades.len() as u64;
        let bars = generate_bars(&trades, cfg);
        let cfg_value = match cfg.kind {
            crate::bar::BarKind::Time => cfg.interval_ns,
            crate::bar::BarKind::Volume | crate::bar::BarKind::Notional => cfg.threshold,
        };
        let n = write_bars(n_root, venue, cfg.kind, cfg_value, &symbol, date, &bars)?;
        stats.bars += n as u64;
    }
    Ok(())
}

/// 读 trade Parquet → `[(TradeRow, CFlags)]`（flags 默认 0；Bar 聚合不
/// 过滤 C5/C13——这些是标记不是剔除，真实插针要进 Bar 的 high/low）。
fn read_trades(file: &std::fs::File) -> Result<Vec<(TradeRow, CFlags)>, DataError> {
    let clone = file
        .try_clone()
        .map_err(|e| DataError::Parquet(format!("克隆文件句柄失败：{e}")))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(clone)
        .map_err(|e| DataError::Parquet(e.to_string()))?
        .build()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    let mut out = Vec::new();
    for b in reader {
        let b = b.map_err(|e| DataError::Parquet(e.to_string()))?;
        let ts_exch = i64_col(&b, "ts_exch")?;
        let ts_recv = i64_col(&b, "ts_recv")?;
        let trade_id = u64_col(&b, "trade_id")?;
        let price = i64_col(&b, "price")?;
        let qty = i64_col(&b, "qty")?;
        let side = i32_col(&b, "aggressor_side")?;
        for i in 0..b.num_rows() {
            out.push((
                TradeRow {
                    ts_exch: ts_exch.value(i),
                    ts_recv: ts_recv.value(i),
                    trade_id: trade_id.value(i),
                    price: price.value(i),
                    qty: qty.value(i),
                    aggressor_side: side.value(i),
                    conn_id: String::new(),
                },
                CFlags::default(),
            ));
        }
    }
    Ok(out)
}

/// 列辅助（与 quality_scan 同款，带生命周期）。
fn i64_col<'a>(b: &'a arrow::array::RecordBatch, name: &str) -> Result<&'a Int64Array, DataError> {
    b.column_by_name(name)
        .ok_or_else(|| DataError::Parquet(format!("缺列 {name}")))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| DataError::Parquet(format!("列 {name} 类型不是 Int64")))
}

fn u64_col<'a>(
    b: &'a arrow::array::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::UInt64Array, DataError> {
    b.column_by_name(name)
        .ok_or_else(|| DataError::Parquet(format!("缺列 {name}")))?
        .as_any()
        .downcast_ref::<arrow::array::UInt64Array>()
        .ok_or_else(|| DataError::Parquet(format!("列 {name} 类型不是 UInt64")))
}

fn i32_col<'a>(
    b: &'a arrow::array::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::Int32Array, DataError> {
    b.column_by_name(name)
        .ok_or_else(|| DataError::Parquet(format!("缺列 {name}")))?
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .ok_or_else(|| DataError::Parquet(format!("列 {name} 类型不是 Int32")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::UnixNanos;
    use so_journal::JournalWriter;

    const CONTRACTS: &str = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1","order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053","funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01","status":"trading","enable_decimal":false,"orders_limit":50}]"#;

    /// 端到端：normalize 产物 → Bar 生成 → 落盘。
    #[test]
    fn bar_pipeline_end_to_end() {
        let root = std::env::temp_dir().join("so-data-bar-pipeline");
        let _ = std::fs::remove_dir_all(&root);
        let raw = root.join("raw");
        let ts = 1_700_000_000_000_000_000_i64;
        let mut w = JournalWriter::new(&raw, "gate", "A").unwrap();
        w.write(&so_journal::JournalRow::new(
            UnixNanos(ts),
            "rest.contracts",
            "*",
            "snapshot",
            CONTRACTS,
        ))
        .unwrap();
        // 两笔成交（同 1m 桶）。
        let trade = r#"{"result":[{"id":1,"create_time_ms":1,"price":"100.5","size":"3","contract":"BTC_USDT","is_internal":false},{"id":2,"create_time_ms":2,"price":"101.0","size":"2","contract":"BTC_USDT","is_internal":false}]}"#;
        w.write(&so_journal::JournalRow::new(
            UnixNanos(ts + 2),
            "futures.trades",
            "BTC_USDT",
            "A",
            trade,
        ))
        .unwrap();
        w.finish().unwrap();

        // normalize
        let req = crate::pipeline::NormalizeRequest {
            raw_dir: &raw,
            out_root: &root.join("parquet"),
            venue: "gate",
            date: "2023-11-14",
        };
        crate::pipeline::normalize_day(&req).unwrap();

        // Bar 生成
        let n_root = root
            .join("parquet")
            .join(crate::pipeline::NORMALIZE_VERSION);
        let stats =
            generate_day_bars(&n_root, "2023-11-14", &BarCfg::time(60_000_000_000)).unwrap();
        assert_eq!(stats.trade_rows, 2);
        assert_eq!(stats.bars, 1);
        let bar_path = n_root
            .join("gate")
            .join("bar")
            .join("time")
            .join("60000000000")
            .join("BTC_USDT")
            .join("2023-11-14.parquet");
        assert!(bar_path.exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
