//! 时间类清洗：C7 时钟异常 / C8 断流（docs/06 §5.2）。
//!
//! C7 逐行判定（纯函数）；C8 是**区间级**——静默期结束才知道
//! 发生了断流，本模块维护状态、由调用方在行流中逐帧喂入并取区间。

use crate::clean::flags::CFlags;
use crate::normalize::BboRow;

/// C7 参数。
#[derive(Debug, Clone, Copy)]
pub struct ClockCfg {
    /// `ts_recv - ts_exch` 低于此值（纳秒）视为时钟倒挂（负值自然落入）。
    pub min_lag_ns: i64,
    /// 高于此值（纳秒，默认 10 秒）视为异常大（交易所时钟漂移或重放）。
    pub max_lag_ns: i64,
}

impl Default for ClockCfg {
    fn default() -> Self {
        Self {
            min_lag_ns: 0,
            max_lag_ns: 10_000_000_000,
        }
    }
}

/// C7：`ts_recv - ts_exch` 为负或异常大。
pub fn c7_clock_anomaly(ts_recv: i64, ts_exch: i64, cfg: &ClockCfg) -> bool {
    let lag = ts_recv - ts_exch;
    lag < cfg.min_lag_ns || lag > cfg.max_lag_ns
}

/// C8 断流检测状态（每 `(品种, 频道)` 一个）。
///
/// 阈值语义与录制器静默检测一致（`max_silence_secs`）：
/// 超过阈值无消息 ⇒ stale 区间从**上一条消息**开始计；
/// 下一条消息到达时区间闭合，起点行标 [`CFlags::STALE`]。
#[derive(Debug)]
pub struct StaleState {
    max_silence_ns: i64,
    last_ts_recv: Option<i64>,
}

impl StaleState {
    /// 构造。`max_silence_secs` 与录制配置同名同义。
    pub fn new(max_silence_secs: i64) -> Self {
        Self {
            max_silence_ns: max_silence_secs * 1_000_000_000,
            last_ts_recv: None,
        }
    }

    /// 喂一行的时间戳；若与上一行的间隔超阈值，返回 `(区间起点, 区间终点)`。
    ///
    /// 区间用 `ts_recv` 表达（数据侧唯一的接收时钟）。
    pub fn on_ts(&mut self, ts_recv: i64) -> Option<(i64, i64)> {
        let prev = self.last_ts_recv.replace(ts_recv);
        match prev {
            Some(p) if ts_recv - p > self.max_silence_ns => Some((p, ts_recv)),
            _ => None,
        }
    }
}

/// C8 + C7 对 BBO 行的组合判定（调用方便利）。
pub fn check_bbo(row: &BboRow, cfg: &ClockCfg) -> CFlags {
    let mut f = CFlags::default();
    if c7_clock_anomaly(row.ts_recv, row.ts_exch, cfg) {
        f = f.union(CFlags::CLOCK_ANOMALY);
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS: i64 = 1_000_000_000;

    /// C7：负 lag 与超 10s lag 异常；正常 lag 不判。
    #[test]
    fn c7_rules() {
        let cfg = ClockCfg::default();
        assert!(c7_clock_anomaly(1_000, 2_000, &cfg)); // lag 负
        assert!(c7_clock_anomaly(20 * NS, NS, &cfg)); // lag 19s > 10s
        assert!(!c7_clock_anomaly(1_500_000_000, 1_000_000_000, &cfg)); // 0.5s 正常
    }

    /// C8：间隔超阈值返回区间；正常间隔不返回；区间起点是上一条消息。
    #[test]
    fn c8_interval_semantics() {
        let mut st = StaleState::new(5);
        assert_eq!(st.on_ts(NS), None, "首条无前序");
        assert_eq!(st.on_ts(3 * NS), None, "2s < 5s 阈值");
        let interval = st.on_ts(20 * NS);
        assert_eq!(
            interval,
            Some((3 * NS, 20 * NS)),
            "17s > 5s，区间从上一条开始"
        );
        assert_eq!(st.on_ts(21 * NS), None, "区间闭合后恢复正常");
    }
}
