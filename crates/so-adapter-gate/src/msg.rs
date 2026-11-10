//! Gate WS 消息信封的反序列化模型（serde）。
//!
//! 只声明本适配器消费的字段；Gate 文档规定 `s` 在带
//! `X-Gate-Size-Decimal: 1` 头时是字符串、不带时是整数（或被截断），
//! 适配器固定带该头（docs/venues/gate.md §2.4），数量一律按字符串解析；
//! 但 `is_internal` 等字段可能缺省，全部 `#[serde(default)]`。

use serde::Deserialize;

use crate::flex::string_or_number;

/// WS 信封：订阅确认与数据推送同构，用 `event` 区分（docs/venues/gate.md §2.1）。
#[derive(Debug, Deserialize)]
pub struct Envelope<'a> {
    /// 频道名，如 `futures.order_book_update`。
    pub channel: &'a str,
    /// `update` = 数据推送；`subscribe`/`unsubscribe` = 确认。
    pub event: &'a str,
    /// 错误；订阅确认只看它是否为 null。
    pub error: Option<EnvelopeError>,
    /// 数据体（推送）或结果（确认）；保留原始 JSON 文本由具体频道解析。
    #[serde(borrow)]
    pub result: &'a serde_json::value::RawValue,
}

/// 信封错误对象。
#[derive(Debug, Deserialize)]
pub struct EnvelopeError {
    /// 错误码：1 请求体格式错、2 参数无效、3 服务端异常、4 鉴权失败。
    pub code: i64,
}

/// `futures.book_ticker` 推送体。
#[derive(Debug, Deserialize)]
pub struct BookTickerMsg {
    /// 生成时间（毫秒）。
    pub t: i64,
    /// 订单簿更新 ID。
    pub u: u64,
    /// 合约名。
    pub s: String,
    /// 最优买价；空字符串 = 无买单。
    pub b: String,
    /// 最优买量（带 Size-Decimal 头是字符串，否则是整数张数）。
    #[serde(rename = "B", deserialize_with = "string_or_number")]
    pub bid_qty: String,
    /// 最优卖价。
    pub a: String,
    /// 最优卖量。
    #[serde(rename = "A", deserialize_with = "string_or_number")]
    pub ask_qty: String,
}

/// `futures.order_book_update` 推送体。
#[derive(Debug, Deserialize)]
pub struct ObuMsg {
    /// 订单簿生成时间（毫秒）。
    pub t: i64,
    /// 合约名。
    pub s: String,
    /// 本次更新起始 ID。
    #[serde(rename = "U")]
    pub first_id: u64,
    /// 本次更新结束 ID。
    #[serde(rename = "u")]
    pub last_id: u64,
    /// 全量替换标记；缺省 = 增量。
    #[serde(default)]
    pub full: bool,
    /// 买侧变更。
    #[serde(default)]
    pub b: Vec<ObuLevelMsg>,
    /// 卖侧变更。
    #[serde(default)]
    pub a: Vec<ObuLevelMsg>,
}

/// 深度档位 `{p, s}`（对象格式，合约频道）。
#[derive(Debug, Deserialize)]
pub struct ObuLevelMsg {
    /// 价格。
    pub p: String,
    /// 最新总量（覆盖量）；0 = 删档。可能是整数（不带头时）。
    #[serde(deserialize_with = "string_or_number")]
    pub s: String,
}

/// `futures.trades` 推送体（数组）。
#[derive(Debug, Deserialize)]
pub struct TradeMsg {
    /// 交易 ID（去重键）。
    pub id: u64,
    /// 成交时间（毫秒）。
    pub create_time_ms: i64,
    /// 成交价。
    pub price: String,
    /// 成交量；正 = 主动买，负 = 主动卖。可能是整数（不带头时）。
    #[serde(deserialize_with = "string_or_number")]
    pub size: String,
    /// 合约名。
    #[serde(rename = "contract")]
    pub contract: String,
    /// 保险基金/ADL 内部成交；缺省 = false。
    #[serde(default)]
    pub is_internal: bool,
}

/// `futures.tickers` 推送体（数组）。
#[derive(Debug, Deserialize)]
pub struct TickerMsg {
    /// 合约名。
    #[serde(rename = "contract")]
    pub contract: String,
    /// 标记价。
    pub mark_price: String,
    /// 指数价。
    pub index_price: String,
    /// 当前资金费率（字符串小数）。
    pub funding_rate: String,
    /// 生成时间（毫秒）。
    pub t: i64,
}
