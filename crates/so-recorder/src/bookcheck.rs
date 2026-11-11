//! 订单簿重建验证（docs/05 M2 验收第一条 / docs/06 C9）。
//!
//! 从 Raw Journal 回放 `futures.order_book_update` 增量，用
//! [`GateBookBuilder`]（六步法）逐品种重建订单簿；在每个
//! `rest.order_book` 快照时点比对**前 N 档**，统计一致率。
//!
//! 比对时点：快照行到达时增量可能已越过 `snapshot_id`，因此设为待
//! 比对目标，在**首次应用完 `last_id ≥ snapshot_id` 的增量帧后**比对
//! （穿越帧整帧应用，少量越过属可接受近似），随后以该快照重新锚定
//! （`apply_snapshot`）——与实盘 Gap 重建路径同构。
//!
//! A/B 双流同一帧都会落 Journal（frame.rs 先落盘再去重），回放端
//! 用 [`Dedup`] 以同样键去重，保证重建确定性。

use std::collections::HashMap;

use so_adapter_gate::{ObuLevel, RawObu, parse_obu, parse_order_book_rest};
use so_book::{BookError, GateBookBuilder, Level};
use so_core::{InstrumentId, Price, Qty, Scale, UnixNanos};
use so_model::BookDelta;

use crate::dedup::{Dedup, dedup_key_from_frame};
use crate::journal::JournalRow;

/// 比对的前 N 档（验收标准：N=20）。
pub const COMPARE_DEPTH: usize = 20;

/// 单品种重建统计。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolStats {
    /// 已应用增量帧数（去重后）。
    pub frames: u64,
    /// 比对快照数。
    pub comparisons: u64,
    /// 完全一致的快照数（前 N 档两侧逐档相等）。
    pub matched: u64,
    /// 逐档比对总数与一致档数。
    pub levels: u64,
    pub levels_matched: u64,
    /// 序列缺口次数（含未比对即被新快照覆盖的丢失目标）。
    pub gaps: u64,
    /// 快照过旧次数。
    pub stale: u64,
    /// 解析失败帧数（原始帧仍在 Journal，A1）。
    pub parse_failed: u64,
    /// 缺精度元数据而跳过的帧数。
    pub no_scale: u64,
}

impl SymbolStats {
    /// 快照级一致率（0..1；无比对时为 1.0）。
    pub fn snapshot_rate(&self) -> f64 {
        if self.comparisons == 0 {
            return 1.0;
        }
        self.matched as f64 / self.comparisons as f64
    }

    /// 档级一致率（0..1；无比对时为 1.0）。
    pub fn level_rate(&self) -> f64 {
        if self.levels == 0 {
            return 1.0;
        }
        self.levels_matched as f64 / self.levels as f64
    }
}

/// 一个待比对的快照目标（到达时尚未比对）。
#[derive(Debug)]
struct PendingSnapshot {
    snapshot_id: u64,
    bids: Vec<(Price, Qty)>,
    asks: Vec<(Price, Qty)>,
}

/// 单品种回放器。
#[derive(Debug)]
struct SymbolReplay {
    builder: GateBookBuilder,
    pending: Option<PendingSnapshot>,
    /// 是否已应用首个快照（锚定）。
    started: bool,
    stats: SymbolStats,
}

impl SymbolReplay {
    fn new() -> Self {
        Self {
            builder: GateBookBuilder::new(),
            pending: None,
            started: false,
            stats: SymbolStats::default(),
        }
    }

    /// 应用一帧（多档、同 U/u）：逐档 `BookDelta`；空帧走 `apply_empty`。
    fn apply_frame(&mut self, inst: InstrumentId, obu: &RawObu) {
        self.stats.frames = self.stats.frames.saturating_add(1);
        if obu.levels.as_slice().is_empty() && !obu.full_replace {
            if let Err(e) = self.builder.apply_empty(obu.first_id, obu.last_id) {
                self.count_error(&e);
            }
            return;
        }
        // 逐档应用；每档 delta 携带整帧的 U/u（同区间，档间顺序无关：
        // 档位按价格独立覆盖，乱序应用结果一致）。
        for lv in obu.levels.as_slice() {
            let d = BookDelta {
                inst,
                ts_exch: obu.ts_exch,
                ts_recv: obu.ts_recv,
                first_id: obu.first_id,
                last_id: obu.last_id,
                full_replace: obu.full_replace,
                side: lv.side,
                price: lv.price,
                qty: lv.qty,
            };
            if let Err(e) = self.builder.apply_delta(&d) {
                self.count_error(&e);
                if e.code() == "BookStaleSnapshot" {
                    // 锚定快照早于当前 WS 流（录制启动时序差）：本地簿
                    // 从未有效，丢弃积压增量，等下一个快照重锚。
                    self.started = false;
                }
                break; // Gap/StaleSnapshot：整帧已失效
            }
        }
    }

    /// 帧应用**前**检查待比对快照：若帧起始 ID 已越过快照 id，簿状态
    /// 正好等于快照时点（穿越帧未应用），在此比对。
    ///
    /// 簇无效（Gap 作废）时不比对——无效簿与快照的差异是数据缺失的
    /// 结果而非重建错误；此时快照用作重锚（六步法第 1 步），计 Gap。
    fn maybe_compare(&mut self, incoming_first: u64) {
        let Some(p) = self.pending.take() else {
            return;
        };
        if p.snapshot_id >= incoming_first && incoming_first != 0 {
            self.pending = Some(p);
            return;
        }
        if !self.builder.book().is_valid() {
            self.stats.gaps = self.stats.gaps.saturating_add(1);
            if self
                .builder
                .apply_snapshot(p.snapshot_id, &p.bids, &p.asks)
                .is_err()
            {
                self.stats.stale = self.stats.stale.saturating_add(1);
            }
            return;
        }
        self.compare(p);
    }

    /// 比对前 N 档。**比对不重锚**：簿仍有效就继续 Streaming（旧锚点
    /// 的增量还在连续流，重锚反而把 builder 打回 Buffering，让后续
    /// 增量全部 StaleSnapshot）。簿无效路径在 [`Self::maybe_compare`]。
    fn compare(&mut self, p: PendingSnapshot) {
        self.stats.comparisons = self.stats.comparisons.saturating_add(1);
        let bid_ok = compare_side(self.builder.book().bids().levels(), &p.bids);
        let ask_ok = compare_side(self.builder.book().asks().levels(), &p.asks);
        let n_total = (p.bids.len() + p.asks.len()).min(COMPARE_DEPTH * 2);
        let n_ok = (bid_ok + ask_ok).min(n_total);
        self.stats.levels = self.stats.levels.saturating_add(n_total as u64);
        self.stats.levels_matched = self.stats.levels_matched.saturating_add(n_ok as u64);
        if n_ok == n_total {
            self.stats.matched = self.stats.matched.saturating_add(1);
        }
    }

    fn count_error(&mut self, e: &BookError) {
        match e.code() {
            "BookGap" => self.stats.gaps = self.stats.gaps.saturating_add(1),
            "BookStaleSnapshot" => self.stats.stale = self.stats.stale.saturating_add(1),
            _ => {}
        }
    }
}

/// Journal 回放重建验证器（多品种）。冷路径工具，允许堆分配。
#[derive(Debug)]
pub struct BookCheck {
    symbols: HashMap<String, SymbolReplay>,
    scales: HashMap<String, Scale>,
    dedup: Dedup,
}

impl BookCheck {
    /// 构造；`scales` 为品种 → 价格精度（合约元数据来源）。
    pub fn new(scales: HashMap<String, Scale>) -> Self {
        Self {
            symbols: HashMap::new(),
            scales,
            dedup: Dedup::new(1_000_000),
        }
    }

    /// 喂一行 Journal；返回该行是否与本检查相关（增量或快照行）。
    pub fn on_row(&mut self, row: &JournalRow) -> bool {
        match row.channel.as_str() {
            "futures.order_book_update" => self.on_obu(row),
            "rest.order_book" => self.on_rest_snapshot(row),
            _ => false,
        }
    }

    /// 全部品种统计（按品种名排序）。
    pub fn stats(&self) -> Vec<(String, SymbolStats)> {
        let mut v: Vec<(String, SymbolStats)> = self
            .symbols
            .iter()
            .map(|(k, r)| (k.clone(), r.stats.clone()))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// 处理一帧增量：去重 → 解析 → 逐档应用 → 比对检查。
    fn on_obu(&mut self, row: &JournalRow) -> bool {
        let key = dedup_key_from_frame(&row.channel, &row.symbol, &row.raw);
        if !self.dedup.first_seen(&key) {
            return true; // 重复帧：已消费但不重复应用
        }
        let replay = self
            .symbols
            .entry(row.symbol.clone())
            .or_insert_with(SymbolReplay::new);
        let Some(scale) = self.scales.get(&row.symbol).copied() else {
            replay.stats.no_scale = replay.stats.no_scale.saturating_add(1);
            return true;
        };
        let ts_recv = UnixNanos(row.ts_recv);
        let Some(result) = extract_result(&row.raw) else {
            replay.stats.parse_failed = replay.stats.parse_failed.saturating_add(1);
            return true;
        };
        match parse_obu(&result, ts_recv, scale) {
            Ok(obu) => {
                replay.maybe_compare(obu.first_id);
                replay.apply_frame(InstrumentId(0), &obu);
            }
            Err(_) => {
                replay.stats.parse_failed = replay.stats.parse_failed.saturating_add(1);
            }
        }
        true
    }

    /// 处理一行 REST 快照：解析 → 设为待比对目标（覆盖旧目标即记 Gap）。
    fn on_rest_snapshot(&mut self, row: &JournalRow) -> bool {
        let replay = self
            .symbols
            .entry(row.symbol.clone())
            .or_insert_with(SymbolReplay::new);
        let Some(scale) = self.scales.get(&row.symbol).copied() else {
            replay.stats.no_scale = replay.stats.no_scale.saturating_add(1);
            return true;
        };
        let Ok(snap) = parse_order_book_rest(&row.raw, UnixNanos(row.ts_recv), scale) else {
            replay.stats.parse_failed = replay.stats.parse_failed.saturating_add(1);
            return true;
        };
        if replay.pending.is_some() {
            // 旧目标在到达比对点前被覆盖：快照间增量断档。
            replay.stats.gaps = replay.stats.gaps.saturating_add(1);
        }
        if !replay.started || !replay.builder.book().is_valid() {
            // 首个快照直接锚定；Gap 作废后的快照是重建锚点（六步法第 1 步）。
            replay.start_from(snap);
        } else {
            replay.pending = Some(PendingSnapshot {
                snapshot_id: snap.snapshot_id,
                bids: level_pairs(snap.bids.as_slice()),
                asks: level_pairs(snap.asks.as_slice()),
            });
        }
        true
    }
}

impl SymbolReplay {
    /// 用快照锚定（首个，或 Gap 后重建）。
    fn start_from(&mut self, snap: so_adapter_gate::RawSnapshot) {
        let LevelPairs(bids, asks) = level_pairs_all(&snap);
        match self.builder.apply_snapshot(snap.snapshot_id, &bids, &asks) {
            Ok(()) => self.started = true,
            Err(_) => {
                // 交叉快照（BookCrossed）：跳过，等下一个快照。
                self.stats.stale = self.stats.stale.saturating_add(1);
            }
        }
    }
}

/// 单侧前 N 档一致数。
fn compare_side(rebuilt: &[Level], snapshot: &[(Price, Qty)]) -> usize {
    snapshot
        .iter()
        .take(COMPARE_DEPTH)
        .zip(rebuilt.iter())
        .filter(|((p, q), lv)| lv.price == *p && lv.qty == *q)
        .count()
}

/// 从 raw 提取 `result`（浅解析）。
fn extract_result(raw: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    v.get("result").and_then(|r| serde_json::to_string(r).ok())
}

/// 档位列表 → (Price, Qty) 对。
fn level_pairs(levels: &[ObuLevel]) -> Vec<(Price, Qty)> {
    levels.iter().map(|l| (l.price, l.qty)).collect()
}

/// 快照 → (bids, asks) 档位对。
fn level_pairs_all(snap: &so_adapter_gate::RawSnapshot) -> LevelPairs {
    LevelPairs(
        snap.bids
            .as_slice()
            .iter()
            .map(|l| (l.price, l.qty))
            .collect(),
        snap.asks
            .as_slice()
            .iter()
            .map(|l| (l.price, l.qty))
            .collect(),
    )
}

/// 快照双侧档位（type alias 回避 clippy::type_complexity）。
struct LevelPairs(Vec<(Price, Qty)>, Vec<(Price, Qty)>);
