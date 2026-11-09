//! 合约（USDT 永续）行情推送 → [`Raw*`] 中间表示的解析函数。
//!
//! 输入是 WS 信封中 `result` 字段的原始 JSON 文本；时间戳参数 `ts_recv`
//! 由 I/O 线程收到帧的第一时间打上（docs/06 §3.2 A2），解析函数不读时钟
//! （docs/04 H7）。数量字符串可能带负号（trades size 语义），价格不带。

use so_core::{Price, Qty, UnixNanos};
use so_model::Side;

use crate::error::GateError;
use crate::msg;
use crate::types::{LevelList, ObuLevel, RATE_E8_SCALE, RawBbo, RawObu, RawTicker, RawTrade};

/// 毫秒 → UnixNanos。
fn ms_to_nanos(ms: i64) -> Result<UnixNanos, GateError> {
    UnixNanos::from_millis(ms).ok_or(GateError::TsOverflow)
}

/// `futures.book_ticker`：`{"t","u","s","b","B","a","A"}` → [`RawBbo`]。
///
/// 数量字段（`B`/`A`）在 serde 模型中命名 `bs`/`as_`（`as` 是关键字）。
/// 价格/数量解析用的 scale 由调用方传入（品种元数据），典型为
/// Gate 合约 `order_price_round` 的精度与 `quanto_multiplier` 精度。
pub fn parse_bbo(
    result_json: &str,
    ts_recv: UnixNanos,
    scale: so_core::Scale,
) -> Result<RawBbo, GateError> {
    let m: msg::BookTickerMsg =
        serde_json::from_str(result_json).map_err(|_| GateError::BadShape)?;
    let ts_exch = ms_to_nanos(m.t)?;
    let (bid, bid_qty) = opt_price_qty(&m.b, &m.bid_qty, scale)?;
    let (ask, ask_qty) = opt_price_qty(&m.a, &m.ask_qty, scale)?;
    Ok(RawBbo {
        ts_recv,
        ts_exch,
        update_id: m.u,
        bid,
        bid_qty,
        ask,
        ask_qty,
    })
}

/// 空字符串 → None；否则解析价格 + 数量对。
fn opt_price_qty(
    price: &str,
    qty: &str,
    scale: so_core::Scale,
) -> Result<(Option<Price>, Option<Qty>), GateError> {
    if price.is_empty() {
        return Ok((None, None));
    }
    let p = Price::parse(price, scale)?;
    let q = Qty::parse(qty, scale)?;
    Ok((Some(p), Some(q)))
}

/// `futures.order_book_update`：`{"t","s","U","u","b","a"[,"full"]}` → [`RawObu`]。
///
/// 档位超过容量（256）时返回 `BadShape`——按 Gap 处理并重建（docs/06 C2）。
pub fn parse_obu(
    result_json: &str,
    ts_recv: UnixNanos,
    scale: so_core::Scale,
) -> Result<RawObu, GateError> {
    let m: msg::ObuMsg = serde_json::from_str(result_json).map_err(|_| GateError::BadShape)?;
    let ts_exch = ms_to_nanos(m.t)?;
    let mut levels = LevelList::EMPTY;
    for (levels_side, msgs) in [(Side::Bid, &m.b), (Side::Ask, &m.a)] {
        for lv in msgs.iter() {
            let price = Price::parse(&lv.p, scale)?;
            let qty = Qty::parse(&lv.s, scale)?;
            if !levels.push(ObuLevel {
                side: levels_side,
                price,
                qty,
            }) {
                return Err(GateError::BadShape); // 档位超容量 → Gap 重建
            }
        }
    }
    Ok(RawObu {
        ts_recv,
        ts_exch,
        first_id: m.first_id,
        last_id: m.last_id,
        full_replace: m.full,
        levels,
    })
}

/// `futures.trades`：`[{"id","create_time_ms","price","size","contract","is_internal"}]` → 逐笔。
///
/// `size` 符号语义：正 = 主动买、负 = 主动卖（docs/venues/gate.md §3.1）。
/// 每次推送可能有多笔；写入调用方缓冲，返回写入条数。
pub fn parse_trade(
    result_json: &str,
    ts_recv: UnixNanos,
    scale: so_core::Scale,
    out: &mut [RawTrade],
) -> Result<usize, GateError> {
    let msgs: Vec<msg::TradeMsg> =
        serde_json::from_str(result_json).map_err(|_| GateError::BadShape)?;
    let mut n = 0;
    for m in &msgs {
        if n >= out.len() {
            break; // 缓冲不足：截断由调用方计数（冷路径），解析不失败
        }
        let ts_exch = ms_to_nanos(m.create_time_ms)?;
        let signed = Qty::parse(&m.size, scale)?;
        let (qty, aggressor) = if signed.is_negative() {
            (signed.checked_neg()?, Side::Ask)
        } else {
            (signed, Side::Bid)
        };
        let Some(slot) = out.get_mut(n) else {
            break; // 缓冲不足：截断由调用方计数（冷路径），解析不失败
        };
        *slot = RawTrade {
            ts_recv,
            ts_exch,
            trade_id: m.id,
            price: Price::parse(&m.price, scale)?,
            qty,
            aggressor,
            internal: m.is_internal,
        };
        n = n.saturating_add(1);
    }
    Ok(n)
}

/// `futures.tickers`：`[{"contract","mark_price","index_price","funding_rate","t"}]` → 标记价 + 资金费。
///
/// 返回首条（tickers 是全所 1s 轮询，每条推送通常只含订阅品种）。
pub fn parse_ticker(
    result_json: &str,
    ts_recv: UnixNanos,
    scale: so_core::Scale,
) -> Result<RawTicker, GateError> {
    let msgs: Vec<msg::TickerMsg> =
        serde_json::from_str(result_json).map_err(|_| GateError::BadShape)?;
    let m = msgs.first().ok_or(GateError::BadShape)?;
    let ts_exch = ms_to_nanos(m.t)?;
    let index = if m.index_price.is_empty() {
        None
    } else {
        Some(Price::parse(&m.index_price, scale)?)
    };
    let rate_raw = so_core::fixed::parse_decimal(&m.funding_rate, RATE_E8_SCALE)?;
    Ok(RawTicker {
        ts_recv,
        ts_exch,
        mark: Price::parse(&m.mark_price, scale)?,
        index,
        rate_e8: rate_raw,
    })
}

/// 资金费率字符串 → rate_e8（暴露给 so-recorder 的 REST 解析复用）。
pub fn parse_funding(rate: &str) -> Result<i64, GateError> {
    so_core::fixed::parse_decimal(rate, RATE_E8_SCALE).map_err(GateError::from)
}
