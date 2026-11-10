//! Gate REST 响应解析（冷路径：品种元数据快照 + 深度快照）。
//!
//! 输入是 REST 响应体的原始 JSON 文本。解析产物与 WS 频道共用 [`Raw*`]
//! 中间表示（`ts_recv` 由调用方注入——REST 场景即为请求返回时间）。
//!
//! 官方文档：
//! - `GET /futures/{settle}/contracts`（docs/venues/gate.md §3.5）
//! - `GET /futures/{settle}/order_book?contract=…&with_id=true`（§3.2 第 1 步）

use so_core::{Price, Qty, Scale, UnixNanos};

use crate::error::GateError;
use crate::types::{LevelList, ObuLevel, RawBbo};

/// 合约元数据快照（`GET /futures/{settle}/contracts` 单条）。
#[derive(Debug, serde::Deserialize)]
pub struct ContractMsg {
    /// 合约名，如 `BTC_USDT`。
    pub name: String,
    /// 一张合约的面值（标的币数量），如 `"0.0001"`。
    pub quanto_multiplier: String,
    /// 委托价最小单位（tick），如 `"0.1"`。
    pub order_price_round: String,
    /// 张数下限（字符串）。
    pub order_size_min: String,
    /// 张数上限（字符串）。
    pub order_size_max: String,
    /// 当前资金费率（字符串小数）。
    pub funding_rate: String,
    /// 资金费应用间隔（秒）。
    pub funding_interval: i64,
    /// 下次资金费应用时间（秒）。
    pub funding_next_apply: i64,
    /// 标记价最小单位。
    pub mark_price_round: String,
    /// 品种状态：`prelaunch / trading / delisting / delisted / circuit_breaker`。
    pub status: String,
    /// 是否支持小数张数。
    pub enable_decimal: bool,
    /// 每用户最大挂单数。
    pub orders_limit: i64,
}

/// 深度快照（`GET /futures/{settle}/order_book?with_id=true`）。
#[derive(Debug, serde::Deserialize)]
pub struct OrderBookRestMsg {
    /// 订单簿 ID（即六步法的 `baseID`）。
    pub id: i64,
    /// 响应数据生成时间（秒，浮点）。
    pub current: f64,
    /// 订单簿变更时间（秒，浮点）。
    pub update: f64,
    /// 卖侧（升序）。
    pub asks: Vec<RestLevelMsg>,
    /// 买侧（降序）。
    pub bids: Vec<RestLevelMsg>,
}

/// REST 深度档位（`{p, s}`；带 Size-Decimal 头时 s 是字符串，否则是整数张数）。
#[derive(Debug, serde::Deserialize)]
pub struct RestLevelMsg {
    /// 价格。
    pub p: String,
    /// 总量。
    #[serde(deserialize_with = "crate::flex::string_or_number")]
    pub s: String,
}

/// 解析合约元数据列表（contracts 端点返回数组）。
pub fn parse_contracts(json: &str) -> Result<Vec<ContractMsg>, GateError> {
    serde_json::from_str(json).map_err(|_| GateError::BadShape)
}

/// 解析深度快照（order_book 端点）。
pub fn parse_order_book_rest(
    json: &str,
    ts_recv: UnixNanos,
    scale: Scale,
) -> Result<RawSnapshot, GateError> {
    let m: OrderBookRestMsg = serde_json::from_str(json).map_err(|_| GateError::BadShape)?;
    let mut bids = LevelList::EMPTY;
    let mut asks = LevelList::EMPTY;
    for lv in &m.bids {
        bids.push(ObuLevel {
            side: so_model::Side::Bid,
            price: Price::parse(&lv.p, scale)?,
            qty: Qty::parse(&lv.s, scale)?,
        })
        .ok_or(GateError::BadShape)?;
    }
    for lv in &m.asks {
        asks.push(ObuLevel {
            side: so_model::Side::Ask,
            price: Price::parse(&lv.p, scale)?,
            qty: Qty::parse(&lv.s, scale)?,
        })
        .ok_or(GateError::BadShape)?;
    }
    // update 是秒浮点（如 1623898993.121）→ 纳秒
    let update_ns = (m.update * 1e9) as i64;
    Ok(RawSnapshot {
        ts_recv,
        ts_exch: UnixNanos(update_ns),
        snapshot_id: m.id as u64,
        bids,
        asks,
    })
}

/// REST 深度快照解析产物。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawSnapshot {
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
    /// 交易所时间（`update` 字段）。
    pub ts_exch: UnixNanos,
    /// 快照 ID（六步法 baseID）。
    pub snapshot_id: u64,
    /// 买侧档位。
    pub bids: LevelList,
    /// 卖侧档位。
    pub asks: LevelList,
}

impl RawSnapshot {
    /// 转换为 [`RawBbo`]（快照首档即 BBO）。
    pub fn to_bbo(&self) -> RawBbo {
        let bid = self.bids.as_slice().first();
        let ask = self.asks.as_slice().first();
        RawBbo {
            ts_recv: self.ts_recv,
            ts_exch: self.ts_exch,
            update_id: self.snapshot_id,
            bid: bid.map(|l| l.price),
            bid_qty: bid.map(|l| l.qty),
            ask: ask.map(|l| l.price),
            ask_qty: ask.map(|l| l.qty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECV: UnixNanos = UnixNanos(1_700_000_000_000_000_000);

    #[test]
    fn contracts_official_sample() {
        let json = r#"[{
            "name": "BTC_USDT",
            "type": "direct",
            "quanto_multiplier": "0.0001",
            "order_price_round": "0.1",
            "order_size_min": "1",
            "order_size_max": "1000000",
            "funding_rate": "0.002053",
            "funding_interval": 28800,
            "funding_next_apply": 1610035200,
            "mark_price_round": "0.01",
            "status": "trading",
            "enable_decimal": false,
            "orders_limit": 50
        }]"#;
        let contracts = parse_contracts(json).unwrap();
        assert_eq!(contracts.len(), 1);
        let c = &contracts[0];
        assert_eq!(c.name, "BTC_USDT");
        assert_eq!(c.quanto_multiplier, "0.0001");
        assert_eq!(c.order_price_round, "0.1");
        assert_eq!(c.funding_interval, 28800);
    }

    #[test]
    fn order_book_rest_official_sample() {
        // 官方样例（docs/venues/gate.md §3.2 第 1 步）
        let sc2 = Scale::new(2).unwrap(); // 1.17/1.52 需要 2 位精度
        let json = r#"{
            "id": 123456,
            "current": 1623898993.123,
            "update": 1623898993.121,
            "asks": [
                {"p": "1.52", "s": "100"},
                {"p": "1.53", "s": "40"}
            ],
            "bids": [
                {"p": "1.17", "s": "150"},
                {"p": "1.16", "s": "203"}
            ]
        }"#;
        let snap = parse_order_book_rest(json, RECV, sc2).unwrap();
        assert_eq!(snap.snapshot_id, 123456);
        assert_eq!(snap.bids.as_slice().len(), 2);
        assert_eq!(snap.asks.as_slice().len(), 2);
        // bids 官方排序：降序（1.17 > 1.16）
        assert_eq!(snap.bids.as_slice()[0].price, Price(117));
        assert_eq!(snap.bids.as_slice()[1].price, Price(116));
        assert_eq!(snap.bids.as_slice()[0].qty, Qty(15_000)); // 150 × 100
        // asks 官方排序：升序（1.52 < 1.53）
        assert_eq!(snap.asks.as_slice()[0].price, Price(152));
        assert_eq!(snap.asks.as_slice()[1].price, Price(153));
        assert_eq!(snap.asks.as_slice()[0].qty, Qty(10_000));
    }

    #[test]
    fn order_book_rest_integer_size_accepted() {
        // 不带 X-Gate-Size-Decimal 头时 s 是整型张数（2026-11-10 实测录制样本）
        let sc2 = Scale::new(2).unwrap();
        let json = r#"{
            "id": 14206010871,
            "current": 1794274915.008,
            "update": 1794274914.767,
            "asks": [{"s": 422, "p": "51.43"}],
            "bids": [{"s": 100, "p": "51.42"}]
        }"#;
        let snap = parse_order_book_rest(json, RECV, sc2).unwrap();
        assert_eq!(snap.snapshot_id, 14206010871);
        assert_eq!(snap.asks.as_slice()[0].qty, Qty(42_200)); // 422 × 100
        assert_eq!(snap.bids.as_slice()[0].qty, Qty(10_000));
    }
}
