//! Gate 序列衔接与快照-增量同步（docs/venues/gate.md §3.2 官方六步法 + docs/03 §5）。
//!
//! [`GateSequencer`] 校验 U/u 连续性；[`GateBookBuilder`] 组合订单簿与同步状态机
//! （等快照 → 等衔接增量 → 流式增量 → 缺口重建）。

use so_model::BookDelta;

use crate::book::OrderBook;
use crate::error::BookError;

/// Gate 增量同步状态机。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SyncPhase {
    /// 等待快照（未重建过）。
    AwaitingSnapshot,
    /// 有快照（baseID），等第一条 `U ≤ baseID+1 ≤ u` 的增量。
    Buffering { base_id: u64 },
    /// 流式应用增量。
    Streaming,
}

/// Gate 订单簿构建器：封装六步法的快照-增量衔接 + U/u 校验 + 全量替换。
#[derive(Debug)]
pub struct GateBookBuilder {
    book: OrderBook,
    phase: SyncPhase,
}

impl GateBookBuilder {
    /// 新构建器（等待快照）。
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(),
            phase: SyncPhase::AwaitingSnapshot,
        }
    }

    /// 只读视图。
    pub const fn book(&self) -> &OrderBook {
        &self.book
    }

    /// 是否已有可用订单簿（快照已应用且衔接完成或正在流式更新）。
    pub const fn is_ready(&self) -> bool {
        matches!(self.phase, SyncPhase::Streaming) && self.book.is_valid()
    }

    /// 应用 REST 快照（六步法第 1 步）。`snapshot_id` 即 `with_id=true` 返回的 `id`。
    ///
    /// 任意时刻可调用（重连/重建路径）；清掉缓冲状态回到 Buffering。
    pub fn apply_snapshot(
        &mut self,
        snapshot_id: u64,
        bids: &[(so_core::Price, so_core::Qty)],
        asks: &[(so_core::Price, so_core::Qty)],
    ) -> Result<(), BookError> {
        self.book.replace(snapshot_id, bids, asks)?;
        self.phase = SyncPhase::Buffering {
            base_id: snapshot_id,
        };
        Ok(())
    }

    /// 应用一条增量（六步法第 2–4 步）。
    ///
    /// - Buffering：丢弃 `u < baseID+1`；`U > baseID+1` ⇒ 快照过旧（`StaleSnapshot`，
    ///   调用方重拉快照）；否则衔接成功进入 Streaming；
    /// - Streaming：要求 `U ≤ last_id+1 ≤ u`；`U > last_id+1` ⇒ `Gap`（订单簿作废）。
    /// - `full: true`：全量替换——直接衔接成功（消息本身即新状态起点）。
    pub fn apply_delta(&mut self, d: &BookDelta) -> Result<(), BookError> {
        if d.is_full_replace() {
            // 重订阅场景：full 消息即新起点（docs/venues/gate.md §4.2 现货同规则）。
            self.book.apply_full(d)?;
            self.phase = SyncPhase::Streaming;
            return Ok(());
        }
        match self.phase {
            SyncPhase::AwaitingSnapshot => {
                // 未做快照：增量无处衔接 ⇒ 忽略（调用方应尽快拉快照）。
                Ok(())
            }
            SyncPhase::Buffering { base_id } => {
                let expect = base_id.saturating_add(1);
                if d.last_id < expect {
                    Ok(()) // 第 3 步：太旧的增量，丢弃
                } else if d.first_id > expect {
                    Err(BookError::StaleSnapshot {
                        base: base_id,
                        first: d.first_id,
                    })
                } else {
                    // 第 2 步：U ≤ baseID+1 ≤ u，衔接成功
                    self.apply_levels(d)?;
                    self.phase = SyncPhase::Streaming;
                    Ok(())
                }
            }
            SyncPhase::Streaming => {
                let expect = self.book.last_id().saturating_add(1);
                if d.first_id > expect {
                    // 第 4 步：缺口 ⇒ 重建
                    self.book.invalidate();
                    Err(BookError::Gap {
                        local: self.book.last_id(),
                        incoming: d.first_id,
                    })
                } else {
                    // U ≤ last_id+1 ≤ u（重叠增量：后到的部分覆盖前面，幂等安全）
                    self.apply_levels(d)?;
                    Ok(())
                }
            }
        }
    }

    /// 档位逐条写入订单簿；档位为空也要推进 ID（第 5 步）。
    fn apply_levels(&mut self, d: &BookDelta) -> Result<(), BookError> {
        // so_model::BookDelta 是单档事件；空档位消息不会走到这（由调用方
        // 对 0 档消息调用 advance）。这里直接应用并推进 ID。
        self.book.apply_level(d)
    }

    /// Gap / StaleSnapshot 后重置：订单簿作废，等新快照。
    pub fn reset(&mut self) {
        self.book.invalidate();
        self.phase = SyncPhase::AwaitingSnapshot;
    }
}

impl Default for GateBookBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Gate 序列号校验器（无快照语义的独立校验，供 recorder 记录前过滤用）。
#[derive(Copy, Clone, Debug, Default)]
pub struct GateSequencer {
    /// 已见到的最后更新 ID。
    last_id: u64,
    /// 是否已初始化（首条不做连续性要求）。
    started: bool,
}

impl GateSequencer {
    /// 校验一条增量：返回 `Err(Gap)` 表示序列不连续。
    pub fn check(&mut self, first_id: u64, last_id: u64) -> Result<(), BookError> {
        if self.started && first_id > self.last_id.saturating_add(1) {
            return Err(BookError::Gap {
                local: self.last_id,
                incoming: first_id,
            });
        }
        self.last_id = last_id;
        self.started = true;
        Ok(())
    }

    /// 重置（重连后首条不判缺）。
    pub fn reset(&mut self) {
        self.started = false;
        self.last_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::{InstrumentId, Price, Qty, UnixNanos};
    use so_model::Side;

    fn d(first: u64, last: u64, side: Side, price: i64, qty: i64) -> BookDelta {
        BookDelta {
            inst: InstrumentId(0),
            ts_exch: UnixNanos(1),
            ts_recv: UnixNanos(2),
            first_id: first,
            last_id: last,
            full_replace: false,
            side,
            price: Price(price),
            qty: Qty(qty),
        }
    }

    #[test]
    fn six_step_flow() {
        let mut g = GateBookBuilder::new();
        // 快照前：增量被忽略
        g.apply_delta(&d(1, 2, Side::Bid, 100, 1)).unwrap();
        assert!(!g.is_ready());
        // 第 1 步：快照 id=10
        g.apply_snapshot(10, &[(Price(100), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap();
        assert!(!g.is_ready());
        // 第 3 步：u=9 < 11 丢弃
        g.apply_delta(&d(5, 9, Side::Bid, 99, 1)).unwrap();
        assert_eq!(g.book().last_id(), 10);
        // 第 3 步（反例）：U=20 > 11 ⇒ 快照过旧
        let err = g.apply_delta(&d(20, 25, Side::Bid, 99, 1)).unwrap_err();
        assert_eq!(err.code(), "BookStaleSnapshot");
        // 重拉快照后衔接：U=10 ≤ 11 ≤ u=12
        g.apply_snapshot(10, &[(Price(100), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap();
        g.apply_delta(&d(10, 12, Side::Bid, 100, 2)).unwrap();
        assert!(g.is_ready());
        assert_eq!(g.book().last_id(), 12);
        assert_eq!(g.book().bids().best().unwrap().qty, Qty(2));
    }

    #[test]
    fn gap_invalidates() {
        let mut g = GateBookBuilder::new();
        g.apply_snapshot(10, &[(Price(100), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap();
        g.apply_delta(&d(10, 12, Side::Bid, 100, 2)).unwrap();
        // U=15 > 12+1 ⇒ Gap
        let err = g.apply_delta(&d(15, 16, Side::Ask, 101, 1)).unwrap_err();
        assert_eq!(err.code(), "BookGap");
        assert!(!g.book().is_valid());
        // reset 后重新等快照
        g.reset();
        assert!(!g.is_ready());
    }

    #[test]
    fn empty_delta_advances_id() {
        let mut g = GateBookBuilder::new();
        g.apply_snapshot(10, &[], &[]).unwrap();
        // 第 5 步：b/a 为空的增量也推进 id——由 advance 语义承载
        g.apply_delta(&d(10, 13, Side::Bid, 100, 0)).unwrap();
        assert_eq!(g.book().last_id(), 13);
    }

    #[test]
    fn full_replace_reanchors() {
        let mut g = GateBookBuilder::new();
        g.apply_snapshot(10, &[(Price(100), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap();
        g.apply_delta(&d(10, 12, Side::Bid, 100, 2)).unwrap();
        // full=true 的消息无需连续性
        let mut full = d(999, 1000, Side::Ask, 102, 1);
        full.full_replace = true;
        g.apply_delta(&full).unwrap();
        assert_eq!(g.book().last_id(), 1000);
        assert!(g.is_ready());
    }

    #[test]
    fn sequencer_detects_gap() {
        let mut s = GateSequencer::default();
        s.check(1, 5).unwrap();
        s.check(5, 8).unwrap(); // 重叠合法
        let err = s.check(20, 21).unwrap_err();
        assert_eq!(err.code(), "BookGap");
        s.reset();
        s.check(100, 101).unwrap(); // reset 后不判缺
    }
}
