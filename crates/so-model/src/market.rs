//! 公共行情事件（docs/03 §3）。全部固定大小、`Copy`，无堆分配。

use so_core::{InstrumentId, Price, Qty, UnixNanos, VenueId};

/// 行情事件。每个事件带两个时间戳：`ts_exch`（交易所）与 `ts_recv`（本地接收）；
/// 回测按 `ts_recv`（经延迟模型）推进（docs/04 §9）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MarketEvent {
    /// 最优买卖价（Gate `futures.book_ticker` / OKX `bbo-tbt`）。
    Bbo(Bbo),
    /// 深度增量（档位级变更，含全量替换标记）。
    BookDelta(BookDelta),
    /// 逐笔成交。
    Trade(Trade),
    /// 标记价格。
    MarkPrice(MarkPrice),
    /// 资金费率。
    Funding(Funding),
    /// 行情连接状态变化（Gap 期间风控强制 ReduceOnly，docs/03 §5）。
    FeedStatus(FeedStatus),
}

/// 最优买卖价。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bbo {
    /// 品种。
    pub inst: InstrumentId,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 订单簿更新 ID（Gate `u`；OKX 无对应字段时为 0）。
    pub update_id: u64,
    /// 买价；无买单时为 `None`（Gate 空盘口）。
    pub bid: Option<Price>,
    /// 买量。
    pub bid_qty: Option<Qty>,
    /// 卖价；无卖单时为 `None`。
    pub ask: Option<Price>,
    /// 卖量。
    pub ask_qty: Option<Qty>,
}

/// 深度增量：单个档位的价格与最新总量（覆盖而非累加，Gate 官方语义）。
///
/// 一条 WS 消息含多个档位 ⇒ 适配器产出多个 `BookDelta`（每档一个），
/// 共享同一 `first_id`/`last_id` 区间。全量替换（Gate `full: true`）由
/// [`BookDelta::is_full_replace`] 标记。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BookDelta {
    /// 品种。
    pub inst: InstrumentId,
    /// 交易所时间（订单簿生成时间）。
    pub ts_exch: UnixNanos,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 本条消息首个更新 ID（Gate `U`）。
    pub first_id: u64,
    /// 本条消息最后更新 ID（Gate `u`）。
    pub last_id: u64,
    /// 全量替换标记（Gate `full: true`；缺省为增量）。
    pub full_replace: bool,
    /// 侧（买/卖）。
    pub side: Side,
    /// 档位价格。
    pub price: Price,
    /// 该档位最新总量；0 = 删档。
    pub qty: Qty,
}

impl BookDelta {
    /// 是否为全量替换。
    pub const fn is_full_replace(&self) -> bool {
        self.full_replace
    }
}

/// 方向。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// 买方。
    Bid,
    /// 卖方。
    Ask,
}

/// 逐笔成交。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Trade {
    /// 品种。
    pub inst: InstrumentId,
    /// 交易所时间（成交时间）。
    pub ts_exch: UnixNanos,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 交易所成交 ID（去重键，docs/03 §6）。
    pub trade_id: u64,
    /// 成交价。
    pub price: Price,
    /// 成交量（带符号时正=主动买；统一转绝对量 + `aggressor`）。
    pub qty: Qty,
    /// 主动方方向。
    pub aggressor: Side,
    /// 保险基金/ADL 内部成交（Gate `is_internal`；不入 K 线，回测须排除）。
    pub internal: bool,
}

/// 标记价格（可附指数价）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarkPrice {
    /// 品种。
    pub inst: InstrumentId,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 标记价。
    pub mark: Price,
    /// 指数价（Gate tickers 1s 粒度附带；无则 `None`）。
    pub index: Option<Price>,
}

/// 资金费率。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Funding {
    /// 品种。
    pub inst: InstrumentId,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 费率 ×10^8（docs/03 §3 `rate_e8`）。
    pub rate_e8: i64,
    /// 下次结算时间。
    pub next_ts: UnixNanos,
}

/// 行情连接状态（静默断流检测输出，docs/06 §3.2 A5）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FeedStatus {
    /// 交易所。
    pub venue: VenueId,
    /// 状态。
    pub status: FeedState,
}

/// 连接状态。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FeedState {
    /// 正常。
    Connected,
    /// 序列缺口（订单簿无效，直到重建）。
    Gap,
    /// 数据陈旧（超过静默阈值）。
    Stale,
    /// 断开。
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> (UnixNanos, UnixNanos) {
        (UnixNanos(1000), UnixNanos(2000))
    }

    #[test]
    fn book_delta_full_replace_flag() {
        let (exch, recv) = ts();
        let delta = BookDelta {
            inst: InstrumentId(0),
            ts_exch: exch,
            ts_recv: recv,
            first_id: 1,
            last_id: 2,
            full_replace: true,
            side: Side::Bid,
            price: Price(100),
            qty: Qty(5),
        };
        assert!(delta.is_full_replace());
    }

    #[test]
    fn bbo_allows_empty_side() {
        let (exch, recv) = ts();
        let bbo = Bbo {
            inst: InstrumentId(0),
            ts_exch: exch,
            ts_recv: recv,
            update_id: 9,
            bid: None,
            bid_qty: None,
            ask: Some(Price(101)),
            ask_qty: Some(Qty(3)),
        };
        assert!(bbo.bid.is_none());
        assert_eq!(bbo.ask, Some(Price(101)));
    }
}
