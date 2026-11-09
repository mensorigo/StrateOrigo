//! 解析产物类型：交易所原始字段 → 统一中间表示。
//!
//! 价格/数量在此层解析为**定点 raw + scale**（尚未绑定品种精度——
//! 品种元数据在 so-data::normalize 阶段按 `configs/instruments` 快照
//! 绑定，docs/03 §2）。symbol 保留 Gate 原始拼写（`BTC_USDT`）。

use so_core::{Price, Qty, UnixNanos};

/// `futures.book_ticker` 解析产物。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawBbo {
    /// 本地接收时间（I/O 线程收到帧时打点，docs/06 §3.2 A2）。
    pub ts_recv: UnixNanos,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 订单簿更新 ID。
    pub update_id: u64,
    /// 买价；无买单 = None。
    pub bid: Option<Price>,
    /// 买量。
    pub bid_qty: Option<Qty>,
    /// 卖价；无卖单 = None。
    pub ask: Option<Price>,
    /// 卖量。
    pub ask_qty: Option<Qty>,
}

/// 深度增量解析产物：一条推送的公共头 + 档位数组。
///
/// 档位数组用固定容量避免堆分配（docs/04 H1）；
/// Gate level ≤ 100，容量 128 留余量。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawObu {
    /// 本地接收时间（I/O 线程收到帧时打点，docs/06 §3.2 A2）。
    pub ts_recv: UnixNanos,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 起始更新 ID。
    pub first_id: u64,
    /// 结束更新 ID。
    pub last_id: u64,
    /// 全量替换标记。
    pub full_replace: bool,
    /// 档位（买侧在前、卖侧在后，side 字段区分）。
    pub levels: LevelList,
}

/// 档位列表：栈上定长数组 + 长度。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LevelList {
    /// 档位数据。
    pub buf: [ObuLevel; 256],
    /// 有效长度。
    pub len: usize,
}

/// 单个深度档位。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObuLevel {
    /// 侧。
    pub side: so_model::Side,
    /// 价格。
    pub price: Price,
    /// 最新总量；0 = 删档。
    pub qty: Qty,
}

impl LevelList {
    /// 空列表。
    pub const EMPTY: Self = Self {
        buf: [ObuLevel {
            side: so_model::Side::Bid,
            price: Price::ZERO,
            qty: Qty::ZERO,
        }; 256],
        len: 0,
    };

    /// 追加一档；满则返回 false（调用方决定截断策略：溢出即视为 Gap）。
    pub fn push(&mut self, level: ObuLevel) -> bool {
        let Some(slot) = self.buf.get_mut(self.len) else {
            return false;
        };
        *slot = level;
        self.len = self.len.saturating_add(1);
        true
    }

    /// 有效档位切片。
    pub fn as_slice(&self) -> &[ObuLevel] {
        self.buf.get(..self.len).unwrap_or(&[])
    }
}

/// `futures.trades` 单笔解析产物。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawTrade {
    /// 本地接收时间（I/O 线程收到帧时打点，docs/06 §3.2 A2）。
    pub ts_recv: UnixNanos,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 交易 ID。
    pub trade_id: u64,
    /// 成交价。
    pub price: Price,
    /// 成交量（绝对值）。
    pub qty: Qty,
    /// 主动方：正 size = 买，负 = 卖。
    pub aggressor: so_model::Side,
    /// 内部成交（保险基金/ADL）。
    pub internal: bool,
}

/// `futures.tickers` 解析产物（标记价 + 资金费）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawTicker {
    /// 本地接收时间（I/O 线程收到帧时打点，docs/06 §3.2 A2）。
    pub ts_recv: UnixNanos,
    /// 交易所时间。
    pub ts_exch: UnixNanos,
    /// 标记价 raw（scale = price_scale）。
    pub mark: Price,
    /// 指数价 raw。
    pub index: Option<Price>,
    /// 资金费率 ×10^8（rate_e8，docs/03 §3）。
    pub rate_e8: i64,
}

/// `funding_rate` 字符串（如 `"-0.000114"`）→ rate_e8 的专用精度（8 位小数）。
pub const RATE_E8_SCALE: so_core::Scale = match so_core::Scale::new(8) {
    Ok(s) => s,
    // 8 ≤ MAX_SCALE(18)，此分支不可达；退化值只为让 const 求值成立。
    Err(_) => so_core::Scale::ZERO,
};

impl RawTicker {
    /// 下次结算时间不可从 tickers 获得（需 REST `funding_next_apply`），
    /// 由 so-recorder 的 REST 快照补齐；解析层填 0。
    pub const NO_NEXT_TS: UnixNanos = UnixNanos(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::Price;

    #[test]
    fn level_list_push_and_full() {
        let mut list = LevelList::EMPTY;
        for i in 0..256 {
            assert!(list.push(ObuLevel {
                side: so_model::Side::Bid,
                price: Price(i),
                qty: Qty(1),
            }));
        }
        assert!(!list.push(ObuLevel {
            side: so_model::Side::Ask,
            price: Price(999),
            qty: Qty(1),
        }));
        assert_eq!(list.as_slice().len(), 256);
    }
}
