//! 质量扫描：读回 normalize 产物，产出每 `(venue, symbol)` 的评级输入。
//!
//! `so-cli data check` 的算法部分（读 Parquet + 累计 flags/区间）在此，
//! CLI 只做编排（04 I8：编排层不做算法，算法层不做 I/O——本模块是
//! "读取 + 汇总"的数据层工具，输入输出都是纯数据，无展示逻辑）。

use std::collections::BTreeMap;
use std::path::Path;

use arrow::datatypes::DataType;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::clean::CFlags;
use crate::clean::report::QualityInput;
use crate::error::DataError;

/// 一个 `(venue, symbol)` 单元的评级输入（键为 `(venue, symbol)`）。
pub type DayQuality = BTreeMap<(String, String), QualityInput>;

/// 一天全部交易所的扫描结果。
#[derive(Debug, Default)]
pub struct ScanResult {
    /// `(venue, symbol) → 评级输入`（BTreeMap：输出顺序确定）。
    pub cells: DayQuality,
    /// quality_intervals 行数（报告展示）。
    pub intervals: usize,
}

/// 扫描 `{n_root}/{venue}/...` 与 `{n_root}/quality_intervals/...`。
///
/// `n_root` 是 normalize 版本根（如 `data/parquet/n1`）。某交易所当天
/// 无数据（无表文件也无区间文件）时不出现在结果里。
pub fn scan_day(n_root: &Path, date: &str) -> Result<ScanResult, DataError> {
    let mut out = ScanResult::default();
    let venues = list_venues(n_root)?;
    for venue in venues {
        absorb_intervals(n_root, &venue, date, &mut out)?;
        absorb_rebuild(n_root, &venue, date, &mut out)?;
        scan_tables(n_root, &venue, date, &mut out)?;
    }
    Ok(out)
}

/// 列出版本根下的交易所目录。
fn list_venues(n_root: &Path) -> Result<Vec<String>, DataError> {
    let mut out = Vec::new();
    for e in
        std::fs::read_dir(n_root).map_err(|e| DataError::Parquet(format!("读目录失败：{e}")))?
    {
        let e = e.map_err(|e| DataError::Parquet(format!("读目录失败：{e}")))?;
        if e.path().is_dir()
            && let Some(name) = e.file_name().to_str()
            && name != "quality_intervals"
            && name != "quality_rebuild"
        {
            out.push(name.to_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// 读 quality_intervals 并入单元（C2/C8 计入缺口）。
fn absorb_intervals(
    n_root: &Path,
    venue: &str,
    date: &str,
    out: &mut ScanResult,
) -> Result<(), DataError> {
    let path = n_root
        .join("quality_intervals")
        .join(venue)
        .join(format!("{date}.parquet"));
    let Ok(file) = std::fs::File::open(&path) else {
        return Ok(()); // 无文件 = 当天无区间级异常
    };
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| DataError::Parquet(e.to_string()))?
        .build()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    for b in reader {
        let b = b.map_err(|e| DataError::Parquet(e.to_string()))?;
        let sym = string_col(&b, "symbol")?;
        let start = i64_col(&b, "ts_start")?;
        let end = i64_col(&b, "ts_end")?;
        let flags = u32_col(&b, "flags")?;
        for i in 0..b.num_rows() {
            let symbol = sym.value(i).to_owned();
            let cell = out.cells.entry((venue.to_owned(), symbol)).or_default();
            let f = CFlags::from_bits(flags.value(i));
            if f.intersects(CFlags::SEQ_GAP.union(CFlags::STALE)) {
                cell.gap_count += 1;
                cell.gap_total_ns += end.value(i) - start.value(i);
            }
            out.intervals += 1;
        }
    }
    Ok(())
}

/// 读 quality_rebuild 表填 `rebuild_match_rate`（无比对样本的品种保持 None，
/// 评级按 1.0 处理——docs/06 §5.3）。
fn absorb_rebuild(
    n_root: &Path,
    venue: &str,
    date: &str,
    out: &mut ScanResult,
) -> Result<(), DataError> {
    let path = n_root
        .join("quality_rebuild")
        .join(venue)
        .join(format!("{date}.parquet"));
    let Ok(file) = std::fs::File::open(&path) else {
        return Ok(()); // 无文件 = 当天无 REST 快照（如纯 WS 录制日）
    };
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| DataError::Parquet(e.to_string()))?
        .build()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    for b in reader {
        let b = b.map_err(|e| DataError::Parquet(e.to_string()))?;
        let sym = string_col(&b, "symbol")?;
        let comparisons = u64_col(&b, "comparisons")?;
        let rate = f64_col(&b, "snapshot_rate")?;
        for i in 0..b.num_rows() {
            if comparisons.value(i) == 0 {
                continue; // 无比对样本：rebuild_match_rate 保持 None
            }
            let cell = out
                .cells
                .entry((venue.to_owned(), sym.value(i).to_owned()))
                .or_default();
            cell.rebuild_match_rate = Some(rate.value(i));
        }
    }
    Ok(())
}

/// 扫四张数据表的 flags 列。
fn scan_tables(
    n_root: &Path,
    venue: &str,
    date: &str,
    out: &mut ScanResult,
) -> Result<(), DataError> {
    for table in ["book_delta", "trade", "bbo", "mark_price"] {
        let tdir = n_root.join(venue).join(table);
        let Ok(rd) = std::fs::read_dir(&tdir) else {
            continue;
        };
        for sym_dir in rd.flatten() {
            let path = sym_dir.path().join(format!("{date}.parquet"));
            let Ok(file) = std::fs::File::open(&path) else {
                continue;
            };
            let Some(symbol) = sym_dir.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let reader = ParquetRecordBatchReaderBuilder::try_new(file)
                .map_err(|e| DataError::Parquet(e.to_string()))?
                .build()
                .map_err(|e| DataError::Parquet(e.to_string()))?;
            for b in reader {
                let b = b.map_err(|e| DataError::Parquet(e.to_string()))?;
                let flags = u32_col(&b, "flags").map_err(|e| {
                    DataError::Parquet(format!("{table}/{symbol} 缺 flags 列：{e}"))
                })?;
                let cell = out
                    .cells
                    .entry((venue.to_owned(), symbol.to_owned()))
                    .or_default();
                for i in 0..b.num_rows() {
                    cell.actual_rows += 1;
                    let f = CFlags::from_bits(flags.value(i));
                    if f.intersects(
                        CFlags::TS_OUT_OF_ORDER
                            .union(CFlags::PRICE_OUTLIER)
                            .union(CFlags::CLOCK_ANOMALY)
                            .union(CFlags::TRADE_OUT_OF_BOOK),
                    ) {
                        cell.anomaly_rows += 1;
                    }
                }
            }
        }
    }
    Ok(())
}

fn string_col<'a>(
    b: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::StringArray, DataError> {
    b.column_by_name(name)
        .and_then(|c| {
            (c.data_type() == &DataType::Utf8)
                .then(|| c.as_any().downcast_ref::<arrow::array::StringArray>())
                .flatten()
        })
        .ok_or_else(|| DataError::Parquet(format!("{name} 列缺失或类型不符")))
}

fn i64_col<'a>(
    b: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::Int64Array, DataError> {
    b.column_by_name(name)
        .and_then(|c| {
            (c.data_type() == &DataType::Int64)
                .then(|| c.as_any().downcast_ref::<arrow::array::Int64Array>())
                .flatten()
        })
        .ok_or_else(|| DataError::Parquet(format!("{name} 列缺失或类型不符")))
}

fn u32_col<'a>(
    b: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::UInt32Array, DataError> {
    b.column_by_name(name)
        .and_then(|c| {
            (c.data_type() == &DataType::UInt32)
                .then(|| c.as_any().downcast_ref::<arrow::array::UInt32Array>())
                .flatten()
        })
        .ok_or_else(|| DataError::Parquet(format!("{name} 列缺失或类型不符")))
}

fn u64_col<'a>(
    b: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::UInt64Array, DataError> {
    b.column_by_name(name)
        .and_then(|c| {
            (c.data_type() == &DataType::UInt64)
                .then(|| c.as_any().downcast_ref::<arrow::array::UInt64Array>())
                .flatten()
        })
        .ok_or_else(|| DataError::Parquet(format!("{name} 列缺失或类型不符")))
}

fn f64_col<'a>(
    b: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a arrow::array::Float64Array, DataError> {
    b.column_by_name(name)
        .and_then(|c| {
            (c.data_type() == &DataType::Float64)
                .then(|| c.as_any().downcast_ref::<arrow::array::Float64Array>())
                .flatten()
        })
        .ok_or_else(|| DataError::Parquet(format!("{name} 列缺失或类型不符")))
}
