//! Raw Journal → 统一事件行（docs/06 §4.1 表 schema）。
//!
//! [`Normalizer`] 收 [`JournalRow`]，按频道分发到 Gate 解析函数，把解析
//! 产物转成各表的**平铺行**（与 Parquet 列一一对应）。行缓冲按表分组，
//! 由调用方定期取走写 Parquet（[`crate::parquet_out`]）。
//!
//! 确定性（docs/05 M2 验收）：同输入重跑逐位一致——行只来自帧内容 +
//! ts_recv，无时钟、无随机。

use std::collections::HashMap;

use so_adapter_gate::{RawTrade, parse_bbo, parse_obu, parse_ticker, parse_trade};
use so_core::{Scale, UnixNanos};
use so_journal::JournalRow;

/// `book_delta` 表一行（docs/06 §4.1）。
///
/// Gate OBU 一帧多档 → 拆多行，行内 `seq_first/seq_last` 同帧头；
/// `seq_prev` 由 Normalizer 按品种跟踪上一帧 `seq_last` 填充（C2 输入）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookDeltaRow {
    pub ts_exch: i64,
    pub ts_recv: i64,
    pub seq_first: u64,
    pub seq_last: u64,
    pub seq_prev: u64,
    pub side: i32, // 0=bid 1=ask
    pub price: i64,
    pub qty: i64,
    pub conn_id: String,
}

/// `trade` 表一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeRow {
    pub ts_exch: i64,
    pub ts_recv: i64,
    pub trade_id: u64,
    pub price: i64,
    pub qty: i64,
    pub aggressor_side: i32, // 0=bid(主动买) 1=ask(主动卖)
    pub conn_id: String,
}

/// `bbo` 表一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BboRow {
    pub ts_exch: i64,
    pub ts_recv: i64,
    pub seq: u64,
    pub bid_px: Option<i64>,
    pub bid_qty: Option<i64>,
    pub ask_px: Option<i64>,
    pub ask_qty: Option<i64>,
    pub conn_id: String,
}

/// `mark_price` 表一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkPriceRow {
    pub ts_exch: i64,
    pub ts_recv: i64,
    pub mark_px: i64,
    pub index_px: Option<i64>,
    pub rate_e8: i64,
    pub conn_id: String,
}

/// `instrument` 时点表一行（docs/06 §4.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentRow {
    pub valid_from: i64,
    pub valid_to: i64,
    pub venue: String,
    pub symbol: String,
    pub inst_type: String,
    pub tick_size: String,
    pub lot_size: String,
    pub min_notional: Option<i64>,
    pub price_scale: i32,
    pub qty_scale: i32,
    pub status: String,
    pub contract_size: String,
}

/// 各表行缓冲（按品种分组）。
#[derive(Debug, Default)]
pub struct Tables {
    /// `品种 → book_delta 行`。
    pub book_delta: HashMap<String, Vec<BookDeltaRow>>,
    /// `品种 → trade 行`。
    pub trade: HashMap<String, Vec<TradeRow>>,
    /// `品种 → bbo 行`。
    pub bbo: HashMap<String, Vec<BboRow>>,
    /// `品种 → mark_price 行`。
    pub mark_price: HashMap<String, Vec<MarkPriceRow>>,
}

/// 单品种序列跟踪状态（seq_prev 填充，C2 输入）。
#[derive(Debug, Default, Clone, Copy)]
struct SeqState {
    /// 上一帧 `seq_last`（无则 0）。
    prev_last: u64,
}

/// Journal 行 → 统一表行的转换器。
#[derive(Debug)]
pub struct Normalizer {
    /// `品种 → 价格精度`（合约元数据来源）。
    scales: HashMap<String, Scale>,
    seq: HashMap<String, SeqState>,
    pub tables: Tables,
    /// 解析失败行数（原始帧仍在 Journal，A1）。
    pub parse_failed: u64,
    /// 缺元数据跳过行数。
    pub no_scale: u64,
}

impl Normalizer {
    /// 构造。
    pub fn new(scales: HashMap<String, Scale>) -> Self {
        Self {
            scales,
            seq: HashMap::new(),
            tables: Tables::default(),
            parse_failed: 0,
            no_scale: 0,
        }
    }

    /// 喂一行 Journal。返回是否为数据帧（被转换或计数）。
    pub fn on_row(&mut self, row: &JournalRow) -> bool {
        let Some(scale) = self.scales.get(&row.symbol).copied() else {
            // rest.contracts 等元数据行不计数；数据频道才计 no_scale。
            if is_data_channel(&row.channel) {
                self.no_scale += 1;
                return true;
            }
            return false;
        };
        let ts_recv = UnixNanos(row.ts_recv);
        let Some(result) = extract_result(&row.raw) else {
            // 无 result 字段（订阅确认/控制帧）不计数；数据频道但 JSON 残缺才算 parse_failed。
            if is_data_channel(&row.channel) {
                self.parse_failed += 1;
                return true;
            }
            return false;
        };
        match row.channel.as_str() {
            "futures.order_book_update" => {
                self.on_obu(row, &result, scale);
                true
            }
            "futures.trades" => {
                self.on_trades(row, &result, scale, ts_recv);
                true
            }
            "futures.book_ticker" => {
                self.on_bbo(row, &result, scale, ts_recv);
                true
            }
            "futures.tickers" => {
                self.on_ticker(row, &result, scale, ts_recv);
                true
            }
            _ => false,
        }
    }

    /// OBU 一帧 → 多行 book_delta（每档一行，同帧头）。
    fn on_obu(&mut self, row: &JournalRow, result: &str, scale: Scale) {
        let Ok(obu) = parse_obu(result, UnixNanos(row.ts_recv), scale) else {
            self.parse_failed += 1;
            return;
        };
        let prev = self.seq_state(&row.symbol, obu.last_id);
        let out = &mut self.tables.book_delta;
        let entry = out.entry(row.symbol.clone()).or_default();
        for lv in obu.levels.as_slice() {
            entry.push(BookDeltaRow {
                ts_exch: obu.ts_exch.0,
                ts_recv: obu.ts_recv.0,
                seq_first: obu.first_id,
                seq_last: obu.last_id,
                seq_prev: prev,
                side: side_code(lv.side),
                price: lv.price.0,
                qty: lv.qty.0,
                conn_id: row.conn_id.clone(),
            });
        }
    }

    /// trades 一帧（数组体）→ 多行 trade。
    fn on_trades(&mut self, row: &JournalRow, result: &str, scale: Scale, ts_recv: UnixNanos) {
        let mut buf = [RawTrade {
            ts_recv,
            ts_exch: UnixNanos(0),
            trade_id: 0,
            price: so_core::Price::ZERO,
            qty: so_core::Qty::ZERO,
            aggressor: so_model::Side::Bid,
            internal: false,
        }; 32];
        let Ok(n) = parse_trade(result, ts_recv, scale, &mut buf) else {
            self.parse_failed += 1;
            return;
        };
        let entry = self.tables.trade.entry(row.symbol.clone()).or_default();
        for t in buf.get(..n).into_iter().flatten() {
            entry.push(TradeRow {
                ts_exch: t.ts_exch.0,
                ts_recv: t.ts_recv.0,
                trade_id: t.trade_id,
                price: t.price.0,
                qty: t.qty.0,
                aggressor_side: side_code(t.aggressor),
                conn_id: row.conn_id.clone(),
            });
        }
    }

    /// book_ticker 一帧 → 一行 bbo。
    fn on_bbo(&mut self, row: &JournalRow, result: &str, scale: Scale, ts_recv: UnixNanos) {
        let Ok(b) = parse_bbo(result, ts_recv, scale) else {
            self.parse_failed += 1;
            return;
        };
        self.tables
            .bbo
            .entry(row.symbol.clone())
            .or_default()
            .push(BboRow {
                ts_exch: b.ts_exch.0,
                ts_recv: b.ts_recv.0,
                seq: b.update_id,
                bid_px: b.bid.map(|p| p.0),
                bid_qty: b.bid_qty.map(|q| q.0),
                ask_px: b.ask.map(|p| p.0),
                ask_qty: b.ask_qty.map(|q| q.0),
                conn_id: row.conn_id.clone(),
            });
    }

    /// tickers 一帧 → 一行 mark_price。
    fn on_ticker(&mut self, row: &JournalRow, result: &str, scale: Scale, ts_recv: UnixNanos) {
        let Ok(t) = parse_ticker(result, ts_recv, scale) else {
            self.parse_failed += 1;
            return;
        };
        self.tables
            .mark_price
            .entry(row.symbol.clone())
            .or_default()
            .push(MarkPriceRow {
                ts_exch: t.ts_exch.0,
                ts_recv: t.ts_recv.0,
                mark_px: t.mark.0,
                index_px: t.index.map(|p| p.0),
                rate_e8: t.rate_e8,
                conn_id: row.conn_id.clone(),
            });
    }

    /// 取并更新品种序列状态（seq_prev = 上一帧 seq_last）。
    fn seq_state(&mut self, symbol: &str, last_id: u64) -> u64 {
        let st = self.seq.entry(symbol.to_owned()).or_default();
        let prev = st.prev_last;
        st.prev_last = last_id;
        prev
    }
}

/// 是否为数据频道（元数据/控制频道不计 no_scale、不做 A/B 去重）。
pub fn is_data_channel(channel: &str) -> bool {
    matches!(
        channel,
        "futures.order_book_update" | "futures.trades" | "futures.book_ticker" | "futures.tickers"
    )
}

/// 从 raw 提取 `result`（浅解析）。
fn extract_result(raw: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    v.get("result").and_then(|r| serde_json::to_string(r).ok())
}

/// Side → 表内整数编码（0=bid 1=ask）。
fn side_code(s: so_model::Side) -> i32 {
    match s {
        so_model::Side::Bid => 0,
        so_model::Side::Ask => 1,
    }
}
