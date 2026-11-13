//! 清洗编排：把行级规则应用到 [`Tables`]，产出标记列 + 区间表 + 统计。
//!
//! 输入是 normalize 产出的行缓冲；输出是**原行 + `flags` 列**（不删行，
//! C6 非法值行在写出时剔除并计数——docs/06 §5.2 里唯一允许剔除的类别）。
//! 规则全是 [`super`] 各文件的纯函数；本模块只做遍历与状态维护
//! （编排层不做算法，04 I9）。
//!
//! 确定性：无时钟、无随机、无 HashMap 迭代序依赖（品种按名称排序处理）。

use std::collections::HashMap;

use super::clock::{ClockCfg, StaleState};
use super::outlier::{MedianState, OutlierCfg, c6_invalid, c6_invalid_bbo};
use super::report::QualityInterval;
use super::seq::{
    c1_duplicate_trade, c2_seq_gap, c3_bbo_out_of_order, c3_trade_out_of_order, c3_ts_out_of_order,
};
use super::{CFlags, CleanStats};
use crate::normalize::{BboRow, BookDeltaRow, MarkPriceRow, TradeRow};

/// 一天清洗的输出（原行 + flags，按品种分组）。
#[derive(Debug, Default)]
pub struct CleanedTables {
    /// `品种 → (行, 标记)`。
    pub book_delta: HashMap<String, Vec<(BookDeltaRow, CFlags)>>,
    pub trade: HashMap<String, Vec<(TradeRow, CFlags)>>,
    pub bbo: HashMap<String, Vec<(BboRow, CFlags)>>,
    pub mark_price: HashMap<String, Vec<(MarkPriceRow, CFlags)>>,
    /// 区间级异常（C2/C8 等）。
    pub intervals: Vec<QualityInterval>,
    /// 规则计数。
    pub stats: CleanStats,
}

/// 清洗参数（阈值集中在一处；调参改这里，规则语义不动）。
#[derive(Debug, Clone, Copy)]
pub struct CleanCfg {
    pub clock: ClockCfg,
    pub outlier: OutlierCfg,
    /// 静默阈值（秒；与录制配置同名同义，默认取深度频道收紧值）。
    pub max_silence_secs: i64,
}

impl Default for CleanCfg {
    fn default() -> Self {
        Self {
            clock: ClockCfg::default(),
            outlier: OutlierCfg::default(),
            max_silence_secs: 60,
        }
    }
}

/// 执行一天清洗。`venue` 供区间表填充。
pub fn clean_tables(
    tables: &crate::normalize::Tables,
    venue: &str,
    cfg: &CleanCfg,
) -> CleanedTables {
    let mut out = CleanedTables::default();
    clean_book_delta(&tables.book_delta, venue, cfg, &mut out);
    clean_trade(&tables.trade, &tables.bbo, venue, cfg, &mut out);
    clean_bbo(&tables.bbo, venue, cfg, &mut out);
    clean_mark_price(&tables.mark_price, venue, cfg, &mut out);
    out
}

// ---- book_delta：C2 缺口 + C3 乱序 + C7 时钟 ----

fn clean_book_delta(
    src: &HashMap<String, Vec<BookDeltaRow>>,
    venue: &str,
    cfg: &CleanCfg,
    out: &mut CleanedTables,
) {
    for symbol in sorted_keys(src.keys()) {
        let rows = &src[symbol];
        out.stats.total_rows += rows.len() as u64;
        let mut prev_ts_exch = 0i64;
        let mut marked: Vec<(BookDeltaRow, CFlags)> = Vec::with_capacity(rows.len());
        for r in rows {
            let mut f = CFlags::default();
            if c2_seq_gap(r) {
                f = f.union(CFlags::SEQ_GAP);
                out.stats.c2_seq_gap += 1;
                out.intervals.push(QualityInterval {
                    venue: venue.to_owned(),
                    symbol: symbol.to_owned(),
                    channel: "futures.order_book_update".into(),
                    ts_start: r.ts_recv,
                    ts_end: r.ts_recv + 1,
                    flags: CFlags::SEQ_GAP.bits(),
                });
            }
            if c3_ts_out_of_order(r.ts_exch, prev_ts_exch) {
                f = f.union(CFlags::TS_OUT_OF_ORDER);
                out.stats.c3_out_of_order += 1;
            }
            if super::clock::c7_clock_anomaly(r.ts_recv, r.ts_exch, &cfg.clock) {
                f = f.union(CFlags::CLOCK_ANOMALY);
                out.stats.c7_clock += 1;
            }
            prev_ts_exch = r.ts_exch;
            marked.push((r.clone(), f));
        }
        out.book_delta.insert(symbol.to_owned(), marked);
    }
}

// ---- trade：C1 重复 + C3 乱序 + C5 插针 + C6 非法值 + C7 + C13 越界 ----
//
// C13 是跨表规则：成交与"最近一次 BBO"比对。两表各自按 ts_recv 有序，
// 用两指针合并——成交行处理前先把 BBO 指针推进到 `ts_recv <= 成交时刻`
// 的最后一行（未来函数防护在 [`super::book::c13_trade_out_of_book`] 里
// 还有第二道：成交早于快照不判）。

fn clean_trade(
    src: &HashMap<String, Vec<TradeRow>>,
    bbo_src: &HashMap<String, Vec<BboRow>>,
    _venue: &str,
    cfg: &CleanCfg,
    out: &mut CleanedTables,
) {
    for symbol in sorted_keys(src.keys()) {
        let rows = &src[symbol];
        out.stats.total_rows += rows.len() as u64;
        let book_rows = bbo_src.get(symbol).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut book_idx = 0usize;
        let mut seen_ids = std::collections::HashSet::new();
        let mut prev_ts_exch = 0i64;
        let mut med = MedianState::new(cfg.outlier);
        // flags 先按行序累积；C5 确认晚于候选行，用行号回填。
        let mut flags: Vec<CFlags> = Vec::with_capacity(rows.len());
        for r in rows {
            let mut f = CFlags::default();
            if c6_invalid(r.price, r.qty) {
                f = f.union(CFlags::INVALID_VALUE);
                out.stats.c6_invalid += 1;
            }
            if c1_duplicate_trade(r.trade_id, &mut seen_ids) {
                f = f.union(CFlags::DUPLICATE);
                out.stats.c1_duplicate += 1;
            }
            if c3_trade_out_of_order(r, prev_ts_exch) {
                f = f.union(CFlags::TS_OUT_OF_ORDER);
                out.stats.c3_out_of_order += 1;
            }
            if super::clock::c7_clock_anomaly(r.ts_recv, r.ts_exch, &cfg.clock) {
                f = f.union(CFlags::CLOCK_ANOMALY);
                out.stats.c7_clock += 1;
            }
            // C13：推进 BBO 指针到成交时刻可见的最新盘口。
            while book_idx + 1 < book_rows.len() && book_rows[book_idx + 1].ts_recv <= r.ts_recv {
                book_idx += 1;
            }
            if book_idx < book_rows.len()
                && super::book::c13_trade_out_of_book(r, &book_rows[book_idx])
            {
                f = f.union(CFlags::TRADE_OUT_OF_BOOK);
                out.stats.c13_trade_out_of_book += 1;
            }
            let (row_idx, outlier_f) = med.on_trade_px(r.price);
            if outlier_f.intersects(CFlags::PRICE_OUTLIER) {
                // 回填候选行（row_idx 从 1 起，与行号一一对应）。
                if let Some(slot) = flags.get_mut(row_idx - 1) {
                    *slot = slot.union(CFlags::PRICE_OUTLIER);
                    out.stats.c5_outlier += 1;
                }
            }
            prev_ts_exch = r.ts_exch;
            flags.push(f);
        }
        let marked: Vec<(TradeRow, CFlags)> = rows
            .iter()
            .cloned()
            .zip(flags)
            .filter(|(_, f)| !f.intersects(CFlags::INVALID_VALUE))
            .collect();
        out.trade.insert(symbol.to_owned(), marked);
    }
}

// ---- bbo：C4 交叉 + C6 + C7 + C8 断流 ----

fn clean_bbo(
    src: &HashMap<String, Vec<BboRow>>,
    venue: &str,
    cfg: &CleanCfg,
    out: &mut CleanedTables,
) {
    for symbol in sorted_keys(src.keys()) {
        let rows = &src[symbol];
        out.stats.total_rows += rows.len() as u64;
        let mut prev_ts_exch = 0i64;
        let mut stale = StaleState::new(cfg.max_silence_secs);
        let mut marked: Vec<(BboRow, CFlags)> = Vec::with_capacity(rows.len());
        for r in rows {
            let mut f = super::clock::check_bbo(r, &cfg.clock);
            if f.intersects(CFlags::CLOCK_ANOMALY) {
                out.stats.c7_clock += 1;
            }
            if c6_invalid_bbo(r) {
                f = f.union(CFlags::INVALID_VALUE);
                out.stats.c6_invalid += 1;
            }
            if super::book::c4_crossed(r) {
                f = f.union(CFlags::CROSSED_BOOK);
                out.stats.c4_crossed += 1;
            }
            if c3_bbo_out_of_order(r, prev_ts_exch) {
                f = f.union(CFlags::TS_OUT_OF_ORDER);
                out.stats.c3_out_of_order += 1;
            }
            let row_idx = marked.len();
            if let Some((start, end)) = stale.on_ts(r.ts_recv) {
                out.stats.c8_stale_intervals += 1;
                out.intervals.push(QualityInterval {
                    venue: venue.to_owned(),
                    symbol: symbol.to_owned(),
                    channel: "futures.book_ticker".into(),
                    ts_start: start,
                    ts_end: end,
                    flags: CFlags::STALE.bits(),
                });
                // 区间起点行（静默前的最后一条消息）补 STALE 标记。
                if let Some(slot) = marked.get_mut(row_idx - 1) {
                    slot.1 = slot.1.union(CFlags::STALE);
                }
            }
            prev_ts_exch = r.ts_exch;
            marked.push((r.clone(), f));
        }
        out.bbo.insert(symbol.to_owned(), marked);
    }
}

// ---- mark_price：仅 C7（标记价来源本身是交易所计算值）----

fn clean_mark_price(
    src: &HashMap<String, Vec<MarkPriceRow>>,
    _venue: &str,
    cfg: &CleanCfg,
    out: &mut CleanedTables,
) {
    for symbol in sorted_keys(src.keys()) {
        let rows = &src[symbol];
        out.stats.total_rows += rows.len() as u64;
        let mut marked: Vec<(MarkPriceRow, CFlags)> = Vec::with_capacity(rows.len());
        for r in rows {
            let mut f = CFlags::default();
            if super::clock::c7_clock_anomaly(r.ts_recv, r.ts_exch, &cfg.clock) {
                f = f.union(CFlags::CLOCK_ANOMALY);
                out.stats.c7_clock += 1;
            }
            marked.push((r.clone(), f));
        }
        out.mark_price.insert(symbol.to_owned(), marked);
    }
}

/// 排序品种键（确定性：不依赖 HashMap 迭代序，04 H8/D1）。
fn sorted_keys<'a, I>(keys: I) -> Vec<&'a str>
where
    I: Iterator<Item = &'a String>,
{
    let mut v: Vec<&str> = keys.map(|s| s.as_str()).collect();
    v.sort_unstable();
    v
}
