//! 逐笔 → Bar 聚合（docs/06 §4.1 `bar` 表、§8 派生数据）。
//!
//! 三类 Bar：时间 Bar（固定周期分桶）、成交量 Bar（累计 qty 达阈值
//! 分桶）、成交额 Bar（累计 notional 达阈值分桶）。服务端实时聚合与
//! 本研究代码为**同一份实现**（docs/03 §11）——本模块只做纯计算
//! （I8），调用方负责读 Parquet 与写出。
//!
//! 价格/数量是定点整数（与 trade 表同 scale）；notional 用 i128 防
//! 溢出（price × qty 在极端品种下超 i64）。vwap = notional / volume，
//! 保留 6 位小数（研究精度足够，不追求与交易所逐位一致）。

use crate::clean::CFlags;
use crate::normalize::TradeRow;

/// Bar 类型（docs/06 §8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarKind {
    /// 时间 Bar：固定周期（如 1m = 60_000_000_000 ns）。
    Time,
    /// 成交量 Bar：累计 qty 达阈值分桶。
    Volume,
    /// 成交额 Bar：累计 notional 达阈值分桶。
    Notional,
}

impl BarKind {
    /// 枚举表模式（04 I1）：as_str / FromStr。
    pub const ALL: &'static [BarKind] = &[BarKind::Time, BarKind::Volume, BarKind::Notional];

    pub fn as_str(self) -> &'static str {
        match self {
            BarKind::Time => "time",
            BarKind::Volume => "volume",
            BarKind::Notional => "notional",
        }
    }
}

impl std::str::FromStr for BarKind {
    type Err = UnknownBarKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|k| k.as_str() == s)
            .ok_or(UnknownBarKind)
    }
}

/// 无法识别的 Bar 类型字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("未知 Bar 类型（允许 time / volume / notional）")]
pub struct UnknownBarKind;

/// 一根 Bar（docs/06 §4.1 `bar` schema）。
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub ts_open: i64,
    pub ts_close: i64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume: i64,
    pub notional: i128,
    pub trades: u32,
    pub vwap: f64,
}

/// Bar 生成参数。
#[derive(Debug, Clone, Copy)]
pub struct BarCfg {
    pub kind: BarKind,
    /// 时间 Bar 的周期（ns）；成交量/成交额 Bar 忽略。
    pub interval_ns: i64,
    /// 成交量/成交额 Bar 的阈值（定点整数，与 trade 同 scale）。
    pub threshold: i64,
}

impl BarCfg {
    /// 时间 Bar 构造（interval_ns 必须为正）。
    pub fn time(interval_ns: i64) -> Self {
        Self {
            kind: BarKind::Time,
            interval_ns,
            threshold: 0,
        }
    }

    /// 成交量 Bar 构造（threshold 必须为正）。
    pub fn volume(threshold: i64) -> Self {
        Self {
            kind: BarKind::Volume,
            interval_ns: 0,
            threshold,
        }
    }

    /// 成交额 Bar 构造（threshold 必须为正）。
    pub fn notional(threshold: i64) -> Self {
        Self {
            kind: BarKind::Notional,
            interval_ns: 0,
            threshold,
        }
    }
}

/// 从 trade 行生成 Bar 序列（按 ts_exch 排序后聚合）。
///
/// `trades` 是清洗后的 `[(TradeRow, CFlags)]`——C6 剔除已在清洗阶段
/// 完成，这里直接用。返回的 Bar 按 `ts_open` 升序。
pub fn generate_bars(trades: &[(TradeRow, CFlags)], cfg: &BarCfg) -> Vec<Bar> {
    if trades.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<&(TradeRow, CFlags)> = trades.iter().collect();
    sorted.sort_by_key(|(r, _)| (r.ts_exch, r.trade_id));

    match cfg.kind {
        BarKind::Time => time_bars(&sorted, cfg.interval_ns),
        BarKind::Volume => threshold_bars(&sorted, cfg.threshold, false),
        BarKind::Notional => threshold_bars(&sorted, cfg.threshold, true),
    }
}

/// 时间 Bar：按 interval_ns 分桶（ts_open = 桶起点对齐）。
fn time_bars(sorted: &[&(TradeRow, CFlags)], interval_ns: i64) -> Vec<Bar> {
    let mut bars: Vec<Bar> = Vec::new();
    let mut cur: Option<Bar> = None;
    let mut bucket_start: i64 = 0;
    for (r, _) in sorted {
        let ts = r.ts_exch;
        let aligned = ts - ts.rem_euclid(interval_ns);
        match cur.as_mut() {
            Some(b) if aligned == bucket_start => accumulate(b, r),
            _ => {
                if let Some(b) = cur.take() {
                    bars.push(b);
                }
                bucket_start = aligned;
                cur = Some(new_bar(r, aligned));
            }
        }
    }
    if let Some(b) = cur {
        bars.push(b);
    }
    for b in &mut bars {
        finalize_vwap(b);
    }
    bars
}

/// 成交量/成交额 Bar：累计达阈值后切桶。
fn threshold_bars(sorted: &[&(TradeRow, CFlags)], threshold: i64, use_notional: bool) -> Vec<Bar> {
    let mut bars: Vec<Bar> = Vec::new();
    let mut cur: Option<Bar> = None;
    let mut acc: i128 = 0;
    for (r, _) in sorted {
        match cur.as_mut() {
            Some(b) => {
                accumulate(b, r);
                acc += bucket_acc(r, use_notional);
            }
            None => {
                cur = Some(new_bar(r, r.ts_exch));
                acc = bucket_acc(r, use_notional);
            }
        }
        if acc >= threshold as i128 {
            if let Some(b) = cur.take() {
                bars.push(b);
            }
            cur = None;
            acc = 0;
        }
    }
    if let Some(b) = cur {
        bars.push(b);
    }
    for b in &mut bars {
        finalize_vwap(b);
    }
    bars
}

/// 新桶的首笔（open/high/low/close 都 = 首笔价）。
fn new_bar(r: &TradeRow, ts_open: i64) -> Bar {
    Bar {
        ts_open,
        ts_close: r.ts_exch,
        open: r.price,
        high: r.price,
        low: r.price,
        close: r.price,
        volume: r.qty,
        notional: notional(r.price, r.qty),
        trades: 1,
        vwap: r.price as f64,
    }
}

/// 非首笔累加。
fn accumulate(b: &mut Bar, r: &TradeRow) {
    b.ts_close = r.ts_exch;
    b.high = b.high.max(r.price);
    b.low = b.low.min(r.price);
    b.close = r.price;
    b.volume += r.qty;
    b.notional += notional(r.price, r.qty);
    b.trades += 1;
}

/// 分桶累加值（成交量 Bar 用 qty，成交额 Bar 用 notional）。
fn bucket_acc(r: &TradeRow, use_notional: bool) -> i128 {
    if use_notional {
        notional(r.price, r.qty)
    } else {
        r.qty as i128
    }
}

/// price × qty 用 i128 防溢出（极端品种单笔可能超 i64）。
fn notional(price: i64, qty: i64) -> i128 {
    price as i128 * qty as i128
}

/// vwap = notional / volume（volume=0 时保持 open 作为占位）。
fn finalize_vwap(b: &mut Bar) {
    if b.volume > 0 {
        b.vwap = b.notional as f64 / b.volume as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(ts: i64, price: i64, qty: i64) -> (TradeRow, CFlags) {
        (
            TradeRow {
                ts_exch: ts,
                ts_recv: ts + 100,
                trade_id: ts as u64,
                price,
                qty,
                aggressor_side: 0,
                conn_id: "A".into(),
            },
            CFlags::default(),
        )
    }

    /// 时间 Bar：两笔同桶合并，跨桶切分。
    #[test]
    fn time_bars_merge_and_split() {
        let trades = vec![
            trade(1_000, 100, 2),
            trade(2_000, 105, 3),
            trade(70_000_000_000, 110, 1), // 下一桶
        ];
        let bars = generate_bars(&trades, &BarCfg::time(60_000_000_000));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].open, 100);
        assert_eq!(bars[0].close, 105);
        assert_eq!(bars[0].high, 105);
        assert_eq!(bars[0].low, 100);
        assert_eq!(bars[0].volume, 5);
        assert_eq!(bars[0].trades, 2);
        assert_eq!(bars[1].open, 110);
    }

    /// 成交量 Bar：累计达阈值切桶。
    #[test]
    fn volume_bars_split_on_threshold() {
        let trades = vec![
            trade(1_000, 100, 2),
            trade(2_000, 105, 3), // 累计 5 >= 阈值 5 ⇒ 切
            trade(3_000, 110, 1),
        ];
        let bars = generate_bars(&trades, &BarCfg::volume(5));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].volume, 5);
        assert_eq!(bars[1].volume, 1);
    }

    /// 成交额 Bar：notional 累计达阈值切桶。
    #[test]
    fn notional_bars_split_on_threshold() {
        let trades = vec![trade(1_000, 100, 2), trade(2_000, 105, 3)];
        // notional = 100*2 + 105*3 = 515
        let bars = generate_bars(&trades, &BarCfg::notional(500));
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].notional, 515);
        let bars2 = generate_bars(&trades, &BarCfg::notional(600));
        assert_eq!(bars2.len(), 1); // 未达阈值，单桶
    }

    /// vwap = notional / volume。
    #[test]
    fn vwap_is_notional_over_volume() {
        let trades = vec![trade(1_000, 100, 2), trade(2_000, 200, 1)];
        let bars = generate_bars(&trades, &BarCfg::time(60_000_000_000));
        // notional = 100*2 + 200*1 = 400, volume = 3
        assert!((bars[0].vwap - 400.0 / 3.0).abs() < 1e-9);
    }
}
