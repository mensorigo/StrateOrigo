//! L2 订单簿：双侧档位 + 只读视图（docs/03 §5）。
//!
//! 纯数据结构，不读时钟、不管序列号——序列校验在 [`crate::gate`]（交易所规则
//! 各异）。应用 [`so_model::BookDelta`] 由调用方经 [`GateSequencer`] 校验后
//! 逐档调 [`OrderBook::apply_level`]；快照/全量替换走 [`OrderBook::replace`]。

use so_core::{Price, Qty};
use so_model::{BookDelta, Side};

use crate::error::BookError;
use crate::side::{BookSide, Level};

/// 订单簿：预分配双侧各 [`SIDE_CAPACITY`] 档。
#[derive(Copy, Clone, Debug)]
pub struct OrderBook {
    bids: BookSide,
    asks: BookSide,
    /// 已应用到的最后更新 ID（Gate `u`；无快照时为 0）。
    last_id: u64,
    /// 订单簿是否有效（Gap 后为 false，直到重建）。
    valid: bool,
}

impl OrderBook {
    /// 空订单簿。
    pub fn new() -> Self {
        Self {
            bids: BookSide::empty(Side::Bid),
            asks: BookSide::empty(Side::Ask),
            last_id: 0,
            valid: false,
        }
    }

    /// 已应用到的最后更新 ID。
    pub const fn last_id(&self) -> u64 {
        self.last_id
    }

    /// 是否有效（有快照且无未重建缺口）。
    pub const fn is_valid(&self) -> bool {
        self.valid
    }

    /// 标记无效（缺口/陈旧；Gap 期间风控强制 ReduceOnly，docs/03 §5）。
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// —— 只读视图 ——
    ///
    /// 买侧。
    pub const fn bids(&self) -> &BookSide {
        &self.bids
    }

    /// 卖侧。
    pub const fn asks(&self) -> &BookSide {
        &self.asks
    }

    /// BBO：`(bid, ask)`，任一侧空则对应项为 `None`。
    pub fn bbo(&self) -> (Option<Level>, Option<Level>) {
        (self.bids.best(), self.asks.best())
    }

    /// 中间价（bid+ask)/2，raw 单位；一侧缺失时返回 `None`。
    ///
    /// 用 i128 中间量避免 raw 加法溢出；定点除 2 的余数丢弃（半 tick 场景）。
    pub fn mid(&self) -> Option<i64> {
        let (Some(b), Some(a)) = self.bbo() else {
            return None;
        };
        let sum = i128::from(b.price.0) + i128::from(a.price.0);
        i64::try_from(sum / 2).ok()
    }

    /// 微价格 = (bid·askQty + ask·bidQty) / (bidQty+askQty)，raw 单位。
    /// 任一数量为 0 或缺失时退化为中间价。
    pub fn micro(&self) -> Option<i64> {
        let (Some(b), Some(a)) = self.bbo() else {
            return None;
        };
        let bq = i128::from(b.qty.0);
        let aq = i128::from(a.qty.0);
        let denom = bq + aq;
        if denom == 0 {
            return None;
        }
        let num = i128::from(b.price.0) * aq + i128::from(a.price.0) * bq;
        i64::try_from(num / denom).ok()
    }

    /// 盘口失衡 ∈ [-10^7, +10^7]：(bidQty − askQty) / (bidQty + askQty) × 10^7。
    pub fn imbalance_e7(&self) -> Option<i32> {
        let (Some(b), Some(a)) = self.bbo() else {
            return None;
        };
        let bq = i128::from(b.qty.0);
        let aq = i128::from(a.qty.0);
        let denom = bq + aq;
        if denom == 0 {
            return None;
        }
        let v = (bq - aq) * 10_000_000 / denom;
        i32::try_from(v).ok()
    }

    /// —— 写路径 ——
    ///
    /// 应用单档覆盖更新（已过序列校验的增量）。
    pub fn apply_level(&mut self, delta: &BookDelta) -> Result<(), BookError> {
        let side = match delta.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        side.set(delta.price, delta.qty)?;
        self.last_id = delta.last_id;
        Ok(())
    }

    /// 更新最后 ID（消息档位为空时也必须推进，Gate 六步法第 5 步）。
    pub fn advance_id(&mut self, last_id: u64) {
        self.last_id = last_id;
    }

    /// 用快照（无序档位列表）整体重建。清空双侧后逐档写入。
    pub fn replace(
        &mut self,
        snapshot_id: u64,
        bids: &[(Price, Qty)],
        asks: &[(Price, Qty)],
    ) -> Result<(), BookError> {
        self.bids.clear();
        self.asks.clear();
        self.bids.load(bids)?;
        self.asks.load(asks)?;
        self.last_id = snapshot_id;
        self.valid = true;
        self.check_crossed()
    }

    /// 应用全量替换增量（Gate `full: true`）：先清空被替换的消息档位语义——
    /// Gate full 消息仍只带变更档，但官方语义为"替换后状态"；实践中 full 消息
    /// 只在重订阅时出现且档位为空（见 docs/venues/gate.md §4.2），因此只推进 ID。
    pub fn apply_full(&mut self, delta: &BookDelta) -> Result<(), BookError> {
        debug_assert!(delta.is_full_replace());
        self.advance_id(delta.last_id);
        Ok(())
    }

    /// 盘口交叉检查：买一 ≥ 卖一时返回 `Crossed`（调用方决定重建）。
    pub fn check_crossed(&self) -> Result<(), BookError> {
        let (Some(b), Some(a)) = self.bbo() else {
            return Ok(());
        };
        if b.price.0 >= a.price.0 {
            return Err(BookError::Crossed {
                bid_raw: b.price.0,
                ask_raw: a.price.0,
            });
        }
        Ok(())
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::UnixNanos;

    fn delta(side: Side, price: i64, qty: i64, last_id: u64) -> BookDelta {
        BookDelta {
            inst: so_core::InstrumentId(0),
            ts_exch: UnixNanos(1),
            ts_recv: UnixNanos(2),
            first_id: last_id,
            last_id,
            full_replace: false,
            side,
            price: Price(price),
            qty: Qty(qty),
        }
    }

    #[test]
    fn apply_and_bbo() {
        let mut b = OrderBook::new();
        b.apply_level(&delta(Side::Bid, 100, 5, 1)).unwrap();
        b.apply_level(&delta(Side::Bid, 99, 7, 2)).unwrap();
        b.apply_level(&delta(Side::Ask, 101, 3, 3)).unwrap();
        let (bid, ask) = b.bbo();
        assert_eq!(bid.unwrap().price, Price(100));
        assert_eq!(ask.unwrap().price, Price(101));
        assert_eq!(b.last_id(), 3);
        assert!(!b.is_valid()); // 未做快照重建，仍无效
    }

    #[test]
    fn snapshot_replace_then_deltas() {
        let mut b = OrderBook::new();
        b.replace(10, &[(Price(100), Qty(1))], &[(Price(101), Qty(2))])
            .unwrap();
        assert!(b.is_valid());
        b.apply_level(&delta(Side::Bid, 100, Qty::ZERO.raw(), 11))
            .unwrap();
        let (bid, _) = b.bbo();
        assert!(bid.is_none());
        assert_eq!(b.last_id(), 11);
    }

    #[test]
    fn views_mid_micro_imbalance() {
        let mut b = OrderBook::new();
        b.replace(1, &[(Price(100), Qty(3))], &[(Price(102), Qty(1))])
            .unwrap();
        assert_eq!(b.mid(), Some(101));
        // micro = (100·1 + 102·3) / 4 = 101.5 → raw 取整 101
        assert_eq!(b.micro(), Some(101));
        // imbalance = (3-1)/(3+1) = 0.5 → 5_000_000
        assert_eq!(b.imbalance_e7(), Some(5_000_000));
    }

    #[test]
    fn crossed_detection() {
        let mut b = OrderBook::new();
        b.replace(1, &[(Price(102), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap_err();
    }

    #[test]
    fn invalidate_and_rebuild() {
        let mut b = OrderBook::new();
        b.replace(5, &[(Price(100), Qty(1))], &[(Price(101), Qty(1))])
            .unwrap();
        b.invalidate();
        assert!(!b.is_valid());
        b.replace(9, &[], &[]).unwrap();
        assert!(b.is_valid());
    }
}
