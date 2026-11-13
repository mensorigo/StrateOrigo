//! 订单簿一致性清洗：C4 交叉盘口 / C13 成交越界（docs/06 §5.2）。
//!
//! 两者的输入都是"当前最新盘口视图"——由 bbo 行序列隐式携带
//! （每行就是当时的 BBO）。C4 标记**该行**；C13 需要拿成交行与
//! "最近一次 BBO"比对，属于跨表规则，状态由调用方按品种维护。

use crate::normalize::{BboRow, TradeRow};

/// C4：交叉盘口（`bid_px >= ask_px`，任一侧缺失时不判）。
///
/// 单行瞬时交叉只标记；持续 > N ms 的区间判"订单簿无效"由
/// 调用方（区间汇总）处理——本模块只标行。
pub fn c4_crossed(row: &BboRow) -> bool {
    match (row.bid_px, row.ask_px) {
        (Some(b), Some(a)) => b >= a,
        _ => false,
    }
}

/// C13：成交价在当时盘口之外（严格越界：买价高于当时 ask、卖价低于当时 bid）。
///
/// `book` 是**最近一次** BBO（成交与 BBO 之间有时间差，属已知近似；
/// 真正按时间对齐的判定在回测重放里做，这里只做粗筛标记）。
pub fn c13_trade_out_of_book(trade: &TradeRow, book: &BboRow) -> bool {
    // 成交时间必须不早于 BBO 快照（否则拿未来盘口判过去成交——未来函数）。
    if trade.ts_recv < book.ts_recv {
        return false;
    }
    match (book.bid_px, book.ask_px) {
        (Some(b), Some(a)) => trade.price > a || trade.price < b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbo(bid: Option<i64>, ask: Option<i64>) -> BboRow {
        BboRow {
            ts_exch: 1_000,
            ts_recv: 1_000,
            seq: 1,
            bid_px: bid,
            bid_qty: bid.map(|_| 10),
            ask_px: ask,
            ask_qty: ask.map(|_| 10),
            conn_id: "A".into(),
        }
    }

    fn trade(px: i64, ts_recv: i64) -> TradeRow {
        TradeRow {
            ts_exch: ts_recv,
            ts_recv,
            trade_id: 1,
            price: px,
            qty: 1,
            aggressor_side: 0,
            conn_id: "A".into(),
        }
    }

    /// C4：bid >= ask 算交叉；单边缺失不判；正常价差不判。
    #[test]
    fn c4_rules() {
        assert!(c4_crossed(&bbo(Some(101), Some(100)))); // 倒挂
        assert!(c4_crossed(&bbo(Some(100), Some(100)))); // 相切
        assert!(!c4_crossed(&bbo(Some(100), Some(101))));
        assert!(!c4_crossed(&bbo(Some(100), None))); // 单边不判
        assert!(!c4_crossed(&bbo(None, None)));
    }

    /// C13：越界判定 + 未来函数防护（成交早于盘口快照不判）。
    #[test]
    fn c13_rules() {
        let book = bbo(Some(99), Some(101));
        assert!(c13_trade_out_of_book(&trade(102, 1_001), &book)); // 高于 ask
        assert!(c13_trade_out_of_book(&trade(98, 1_001), &book)); // 低于 bid
        assert!(!c13_trade_out_of_book(&trade(100, 1_001), &book)); // 带内
        assert!(!c13_trade_out_of_book(&trade(102, 999), &book)); // 早于快照不判
    }
}
