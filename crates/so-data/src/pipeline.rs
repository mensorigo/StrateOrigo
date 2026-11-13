//! 按日编排：Raw Journal → 统一 Parquet（docs/05 M2、docs/06 §6）。
//!
//! 流程：列出当日 Journal → 读取全部行 + 恢复 contracts → 全局按
//! `ts_recv` 归并 → A/B 去重 → Normalizer → 写 Parquet。
//! 同一天的输入重跑输出逐位一致（文件按固定顺序、行按稳定排序）。

use std::path::{Path, PathBuf};

use so_journal::JournalRow;
use so_journal::{Dedup, dedup_key_from_frame};

use crate::clean;
use crate::clean::engine::CleanCfg;
use crate::contracts::parse_contract_snapshots;
use crate::error::DataError;
use crate::instrument_out::write_instrument;
use crate::intervals_out::write_intervals;
use crate::journal_in::{journal_files, read_journal, valid_date};
use crate::normalize::{Normalizer, is_data_channel};
use crate::parquet_out::write_all;

/// 规范化版本目录名（docs/06 §6：升级后并存）。
pub const NORMALIZE_VERSION: &str = "n1";

/// 一次 normalize 的统计。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NormalizeStats {
    /// Journal 文件数。
    pub files: usize,
    /// 读出的 Journal 行数（含未解析的控制帧）。
    pub journal_rows: u64,
    /// 尾帧截断/不完整行数。
    pub dropped_lines: u64,
    /// A/B 双连接重复帧数。
    pub duplicate_rows: u64,
    /// 解析失败的数据帧数。
    pub parse_failed: u64,
    /// 缺品种元数据而跳过的帧数。
    pub no_scale: u64,
    /// contracts 快照解析失败数。
    pub contracts_failed: u64,
    /// `instrument` 时点行数。
    pub instrument_rows: u64,
    /// 写出的事件行数（四张行情表合计，清洗剔除后）。
    pub event_rows: u64,
    /// 清洗统计（docs/06 §5.2 的规则计数）。
    pub clean: clean::CleanStats,
}

/// normalize 输入参数。
#[derive(Debug, Clone)]
pub struct NormalizeRequest<'a> {
    /// Raw Journal 根目录（如 `data/raw`）。
    pub raw_dir: &'a Path,
    /// Parquet 输出根目录（如 `data/parquet`）。
    pub out_root: &'a Path,
    /// 交易所 ID（如 `gate`）。
    pub venue: &'a str,
    /// UTC 日期 `YYYY-MM-DD`。
    pub date: &'a str,
}

/// 执行一天的规范化；返回统计与实际输出目录。
pub fn normalize_day(req: &NormalizeRequest<'_>) -> Result<(NormalizeStats, PathBuf), DataError> {
    if !valid_date(req.date) {
        return Err(DataError::Usage(format!(
            "日期格式应为 YYYY-MM-DD：{}",
            req.date
        )));
    }
    let files = journal_files(req.raw_dir, req.venue, req.date)?;
    let mut rows: Vec<JournalRow> = Vec::new();
    let mut contracts: Vec<(i64, String)> = Vec::new();
    let mut stats = NormalizeStats {
        files: files.len(),
        ..NormalizeStats::default()
    };
    for f in &files {
        let read = read_journal(f)?;
        stats.journal_rows = stats
            .journal_rows
            .saturating_add(u64::try_from(read.rows.len()).unwrap_or(u64::MAX));
        stats.dropped_lines = stats.dropped_lines.saturating_add(read.dropped_lines);
        rows.extend(read.rows);
        contracts.extend(read.contracts);
    }
    // 严格时序归并：seq_prev、快照锚定、A/B 去重都依赖全局顺序；
    // 同刻保持文件内顺序（稳定排序 → 确定性）。
    rows.sort_by_key(|r| r.ts_recv);
    contracts.sort_by_key(|(ts, _)| *ts);
    let (scales, instrument_rows, contracts_failed) =
        parse_contract_snapshots(&contracts, req.venue);
    stats.contracts_failed = contracts_failed;
    stats.instrument_rows = u64::try_from(instrument_rows.len()).unwrap_or(u64::MAX);

    let mut normalizer = Normalizer::new(scales.clone());
    let mut dedup = Dedup::new(1_000_000);
    for row in &rows {
        if !is_data_channel(&row.channel) {
            continue;
        }
        let key = dedup_key_from_frame(&row.channel, &row.symbol, &row.raw);
        if !dedup.first_seen(&key) {
            stats.duplicate_rows = stats.duplicate_rows.saturating_add(1);
            continue;
        }
        normalizer.on_row(row);
    }
    stats.parse_failed = normalizer.parse_failed;
    stats.no_scale = normalizer.no_scale;

    // 清洗阶段：normalize 后、写盘前。只加标记不删数据（C6 唯一例外，
    // 非法值行剔除并计数——docs/06 §5.2）；区间表与统计一起产出。
    let cleaned = clean::engine::clean_tables(&normalizer.tables, req.venue, &CleanCfg::default());
    stats.clean = cleaned.stats;

    let n_root = req.out_root.join(NORMALIZE_VERSION);
    let out_dir = n_root.join(req.venue);
    let written = write_all(&out_dir, &cleaned, &scales, req.date)?;
    write_instrument(&out_dir, &instrument_rows, req.date)?;
    write_intervals(&n_root, req.venue, req.date, &cleaned.intervals)?;
    stats.event_rows = u64::try_from(written).unwrap_or(u64::MAX);
    Ok((stats, out_dir))
}

/// 供 `so-cli data ls` 使用：列出规范化输出中的 Parquet 文件。
pub fn list_parquet(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_parquet(root, &mut out);
    out.sort();
    out
}

/// 递归收集 `.parquet`（目录不存在时返回空列表）。
fn collect_parquet(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_parquet(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("parquet") {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::UnixNanos;
    use so_journal::JournalWriter;

    const CONTRACTS: &str = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1","order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053","funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01","status":"trading","enable_decimal":false,"orders_limit":50}]"#;

    #[test]
    fn one_day_pipeline_is_deterministic() {
        let root = std::env::temp_dir().join("so-data-pipeline");
        let _ = std::fs::remove_dir_all(&root);
        let raw = root.join("raw");
        let ts = 1_700_000_000_000_000_000_i64;
        let mut w = JournalWriter::new(&raw, "gate", "A").unwrap();
        w.write(&JournalRow::new(
            UnixNanos(ts),
            "rest.contracts",
            "*",
            "snapshot",
            CONTRACTS,
        ))
        .unwrap();
        let obu = r#"{"result":{"t":1700000000000,"s":"BTC_USDT","U":10,"u":11,"b":[{"p":"100.0","s":"1"}],"a":[]}}"#;
        w.write(&JournalRow::new(
            UnixNanos(ts + 1),
            "futures.order_book_update",
            "BTC_USDT",
            "A",
            obu,
        ))
        .unwrap();
        w.finish().unwrap();
        let req = NormalizeRequest {
            raw_dir: &raw,
            out_root: &root.join("parquet"),
            venue: "gate",
            date: "2023-11-14",
        };
        let (s1, out1) = normalize_day(&req).unwrap();
        let (s2, out2) = normalize_day(&req).unwrap();
        assert_eq!(s1, s2);
        assert_eq!(out1, out2);
        assert_eq!(s1.event_rows, 1);
        assert_eq!(s1.instrument_rows, 1);
        let files = list_parquet(&root.join("parquet"));
        assert_eq!(files.len(), 3); // book_delta + instrument + quality_intervals
        let bytes1 = std::fs::read(&files[0]).unwrap();
        let _ = normalize_day(&req).unwrap();
        let bytes2 = std::fs::read(&files[0]).unwrap();
        assert_eq!(bytes1, bytes2); // 逐位一致
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn bad_date_rejected() {
        let root = std::env::temp_dir().join("so-data-pipeline-bad");
        let req = NormalizeRequest {
            raw_dir: &root,
            out_root: &root,
            venue: "gate",
            date: "2026-1-1",
        };
        assert_eq!(normalize_day(&req).unwrap_err().code(), "DataUsage");
    }
}
