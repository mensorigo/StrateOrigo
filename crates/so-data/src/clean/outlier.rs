//! 价格异常类清洗规则：C5 插针 / C6 非法值（docs/06 §5.2）。
//!
//! **C5 只标记不删**：真实的极端行情（闪崩、插针）必须保留，
//! 否则回测会系统性低估尾部风险（docs/06 §5.1 原则）。
//! "与其他交易所交叉比对"留到 OKX 适配器之后（单所时代只做
//! 滚动中位价偏离 + 短时回归判定；2026-11-12 拍板）。
//!
//! 指标计算用 `f64`（[04 H5](../../docs/04-dev-constraints.md) 允许指标用浮点）；
//! 但**判定不影响存储值**——价格仍以定点整数原样落盘。

use crate::clean::flags::CFlags;
use crate::normalize::BboRow;

/// C5/C6 参数（docs/06 §5.2 的 k 值与窗口；调参属研究期，先给合理初值）。
#[derive(Debug, Clone, Copy)]
pub struct OutlierCfg {
    /// 滚动窗口内样本数（按 tick 数计，不用时间——时间窗对稀疏品种失效）。
    pub window: usize,
    /// 偏离阈值：相对滚动中位价的偏离超过 k 倍滚动 MAD 判异常。
    pub k_mad: f64,
    /// "短时间内回归"窗口：异常后 N 笔内回到判定带内才算插针，否则只记统计。
    pub revert_ticks: usize,
}

impl Default for OutlierCfg {
    fn default() -> Self {
        Self {
            window: 200,
            k_mad: 10.0,
            revert_ticks: 20,
        }
    }
}

/// 滚动中位价 / MAD 状态（每品种一个）。
#[derive(Debug)]
pub struct MedianState {
    buf: std::collections::VecDeque<i64>,
    cfg: OutlierCfg,
    /// 待观察的异常：`(行号, 候选时刻的中位价, 剩余观察笔数)`。
    pending: Option<(usize, i64, usize)>,
    /// 候选时刻的 MAD（观察窗内基准不更新）。
    last_mad: f64,
    tick_counter: usize,
}

impl MedianState {
    /// 构造。
    pub fn new(cfg: OutlierCfg) -> Self {
        Self {
            buf: std::collections::VecDeque::with_capacity(cfg.window),
            cfg,
            pending: None,
            last_mad: 0.0,
            tick_counter: 0,
        }
    }

    /// 喂一笔成交价，返回**本次确认归属到候选行**的 C5 标记。
    ///
    /// 判定（两步）：价偏离滚动中位价超过 `k_mad × MAD` ⇒ **候选**；
    /// 候选后 `revert_ticks` 笔内价格回到候选时刻的判定带内 ⇒ 确认插针
    /// （短时回归是插针与真实趋势跳变的区分点；不回归按趋势处理，不标）。
    ///
    /// **标记归属候选行而非回归行**——插针发生在候选那一刻。
    /// 归属方式：返回 `(flagged_row_index, flags)`；`flagged_row_index`
    /// 是调用方 `on_trade_px` 的**调用序号**（从 1 起），调用方用它回填
    /// 行缓冲里的对应行。历史不足窗口一半时不判定（样本太少，MAD 不稳）。
    pub fn on_trade_px(&mut self, px: i64) -> (usize, CFlags) {
        self.tick_counter += 1;
        let this_row = self.tick_counter;
        let mut out: Option<(usize, CFlags)> = None;

        // 已有候选：检查是否在观察窗内回归。
        if let Some((idx, med_at_candidate, remaining)) = self.pending {
            let mad_at_candidate = self.last_mad; // 观察窗内不更新 med/mad 基准
            if remaining == 0 {
                // 观察耗尽未回归 ⇒ 按趋势处理，撤销候选（判定状态不落盘）。
                self.pending = None;
            } else if is_within_band(px, med_at_candidate, mad_at_candidate, self.cfg.k_mad) {
                // 回归 ⇒ 确认：标记归属候选行（idx），当前回归行不标。
                out = Some((idx, CFlags::PRICE_OUTLIER));
                self.pending = None;
            } else {
                self.pending = Some((idx, med_at_candidate, remaining - 1));
            }
        }

        if self.pending.is_none() && self.buf.len() >= self.cfg.window / 2 {
            let med = median_of(&self.buf);
            let mad = mad_of(&self.buf, med);
            self.last_mad = mad;
            if mad > 0.0 && !is_within_band(px, med, mad, self.cfg.k_mad) {
                self.pending = Some((this_row, med, self.cfg.revert_ticks));
            }
        }

        // 维护滚动窗口（候选期间也推进——窗口反映最新市场状态）。
        self.buf.push_back(px);
        if self.buf.len() > self.cfg.window {
            self.buf.pop_front();
        }
        out.unwrap_or((this_row, CFlags::default()))
    }
}

/// C6：非法值判定（价格 <= 0 / 数量 < 0）。精度不符在解析层已拒（Scale）。
pub fn c6_invalid(price: i64, qty: i64) -> bool {
    price <= 0 || qty < 0
}

/// C6 对 BBO 行：任一存在的档位非法。
pub fn c6_invalid_bbo(row: &BboRow) -> bool {
    let bad_bid = row.bid_px.is_some_and(|p| p <= 0) || row.bid_qty.is_some_and(|q| q < 0);
    let bad_ask = row.ask_px.is_some_and(|p| p <= 0) || row.ask_qty.is_some_and(|q| q < 0);
    bad_bid || bad_ask
}

fn is_within_band(px: i64, med: i64, mad: f64, k: f64) -> bool {
    if mad <= 0.0 {
        return true; // 无离散度时不可判，宁可不标
    }
    let dev = (px as f64 - med as f64).abs();
    dev <= k * mad
}

fn median_of(buf: &std::collections::VecDeque<i64>) -> i64 {
    let mut v: Vec<i64> = buf.iter().copied().collect();
    v.sort_unstable();
    v[v.len() / 2]
}

fn mad_of(buf: &std::collections::VecDeque<i64>, med: i64) -> f64 {
    let mut devs: Vec<f64> = buf.iter().map(|&x| (x as f64 - med as f64).abs()).collect();
    devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    devs[devs.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C6：零价/负量非法；正常值合法。
    #[test]
    fn c6_rules() {
        assert!(c6_invalid(0, 1));
        assert!(c6_invalid(-5, 1));
        assert!(c6_invalid(100, -1));
        assert!(!c6_invalid(100, 1));
    }

    /// C5：构造"稳定价格 + 短时插针 + 回归"序列，插针行应被标记。
    #[test]
    fn c5_spike_with_revert_is_flagged() {
        let cfg = OutlierCfg {
            window: 20,
            k_mad: 8.0,
            revert_ticks: 5,
        };
        let mut st = MedianState::new(cfg);
        // 100 附近窄幅波动 20 笔（>= window/2，开始判定）。
        for i in 0..20 {
            let _ = st.on_trade_px(100 + (i % 3) as i64);
        }
        // 插针：第 21 笔打到 150（远超 8×MAD）⇒ 候选；第 22 笔回归 ⇒ 确认。
        let (r1, f1) = st.on_trade_px(150);
        let (r2, f2) = st.on_trade_px(101);
        // 接口约定：确认发生在回归那笔的返回值里，行号指向候选行（第 21 笔）。
        assert!(
            !f1.intersects(CFlags::PRICE_OUTLIER),
            "候选当时未确认，第 21 笔返回无标记"
        );
        assert_eq!(r1, 21);
        assert!(
            f2.intersects(CFlags::PRICE_OUTLIER),
            "确认标记在回归笔的返回里"
        );
        assert_eq!(r2, 21, "但行号归属候选行（插针那笔）");
    }

    /// C5：真实趋势跳变（不回归）不标记——保留尾部行情。
    #[test]
    fn c5_trend_shift_not_flagged() {
        let cfg = OutlierCfg {
            window: 20,
            k_mad: 8.0,
            revert_ticks: 5,
        };
        let mut st = MedianState::new(cfg);
        for i in 0..20 {
            let _ = st.on_trade_px(100 + (i % 3) as i64);
        }
        // 跳到 150 并**停留**（趋势，不是插针）。
        let _ = st.on_trade_px(150);
        for _ in 0..6 {
            let (_row, f) = st.on_trade_px(150);
            assert!(!f.intersects(CFlags::PRICE_OUTLIER));
        }
    }
}
