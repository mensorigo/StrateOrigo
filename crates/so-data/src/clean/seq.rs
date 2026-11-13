//! 序列与顺序类清洗规则：C1 去重 / C2 缺口 / C3 乱序（docs/06 §5.2）。
//!
//! 全部是**纯函数**：输入行 + 少量前序状态，输出标记，不做 I/O
//! （[04 §3.4 I8/I9](../../docs/04-dev-constraints.md)：编排层不做算法，算法层不做 I/O）。
//! C2 的区间语义（缺口到下一次快照重建之间订单簿无效）由调用方
//! [`crate::clean`] 汇总成 `quality_intervals`，本模块只标行。

use crate::normalize::{BboRow, BookDeltaRow, TradeRow};

/// C1：成交重复（相同 `trade_id`）。
///
/// A/B 双连接去重在 Journal 层做过帧级去重（`Dedup`），但帧内多条成交
/// 之间、以及跨日边界的同 id 仍可能漏网——这里是行级兜底。
/// `seen` 由调用方按 `(品种, 日期)` 生命周期持有。
pub fn c1_duplicate_trade(trade_id: u64, seen: &mut std::collections::HashSet<u64>) -> bool {
    !seen.insert(trade_id)
}

/// C2：深度序列缺口（`seq_prev + 1 != seq_first` 且非首帧）。
///
/// Gate 规则：下一帧 `U` 应等于上一帧 `u + 1`（[docs/venues/gate.md §3.2](../../docs/venues/gate.md)）。
/// 首帧（`seq_prev == 0`，即会话开始/刚重连）不算缺口——没有前序可断。
/// `full=true` 全量替换帧由 normalize 阶段前置判断，此处只看增量衔接。
pub fn c2_seq_gap(row: &BookDeltaRow) -> bool {
    row.seq_prev != 0 && row.seq_first != row.seq_prev + 1
}

/// C3：深度帧时间戳乱序（同品种内 `ts_exch` 非单调）。
///
/// 保留接收顺序（不改数据），只标记。`prev_ts_exch` 是该品种上一帧
/// 的 `ts_exch`（调用方按品种跟踪）。
pub fn c3_ts_out_of_order(ts_exch: i64, prev_ts_exch: i64) -> bool {
    ts_exch < prev_ts_exch
}

/// BBO 行的 C3 判定（按 `seq` 前后帧比较 ts_exch）。
pub fn c3_bbo_out_of_order(row: &BboRow, prev_ts_exch: i64) -> bool {
    row.ts_exch < prev_ts_exch
}

/// Trade 行的 C3 判定。
pub fn c3_trade_out_of_order(row: &TradeRow, prev_ts_exch: i64) -> bool {
    row.ts_exch < prev_ts_exch
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obu(prev: u64, first: u64) -> BookDeltaRow {
        BookDeltaRow {
            ts_exch: 1_000,
            ts_recv: 1_001,
            seq_first: first,
            seq_last: first + 1,
            seq_prev: prev,
            side: 0,
            price: 100,
            qty: 1,
            conn_id: "A".into(),
        }
    }

    /// C2：首帧（prev=0）不算缺口；衔接正常不算；断号算。
    #[test]
    fn c2_gap_rules() {
        assert!(!c2_seq_gap(&obu(0, 10)), "首帧不算缺口");
        assert!(!c2_seq_gap(&obu(9, 10)), "正常衔接");
        assert!(c2_seq_gap(&obu(8, 10)), "断号 8→10 算缺口");
        // 首帧行本身标 prev=0，之后正常
        assert!(!c2_seq_gap(&obu(11, 12)));
    }

    /// C1：同一 trade_id 第二次出现返回 true（重复）。
    #[test]
    fn c1_duplicate_detection() {
        let mut seen = std::collections::HashSet::new();
        assert!(!c1_duplicate_trade(7, &mut seen));
        assert!(c1_duplicate_trade(7, &mut seen));
        assert!(!c1_duplicate_trade(8, &mut seen));
    }

    /// C3：ts_exch 回退（哪怕只差 1ns）都算乱序；相等不算。
    #[test]
    fn c3_order_rules() {
        assert!(!c3_ts_out_of_order(1_000, 999));
        assert!(!c3_ts_out_of_order(1_000, 1_000));
        assert!(c3_ts_out_of_order(999, 1_000));
    }
}
