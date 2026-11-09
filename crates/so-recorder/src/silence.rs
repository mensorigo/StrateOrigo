//! 静默断流检测（docs/06 §3.2 A5）。
//!
//! 每个 (频道, 品种) 设“最大无消息间隔”；超时视为断流，触发重连并标记缺口。
//! 检测由定时器驱动（不依赖消息到达），时间源由调用方注入（测试用假时钟）。

use std::collections::HashMap;

/// 频道静默监控键。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SilenceKey {
    /// 频道名。
    pub channel: String,
    /// 品种。
    pub symbol: String,
}

impl SilenceKey {
    /// 构造。
    pub fn new(channel: impl Into<String>, symbol: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            symbol: symbol.into(),
        }
    }
}

/// 单个频道的静默状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilenceState {
    /// 最后一条消息的本地接收时间（纳秒）。
    pub last_seen: i64,
    /// 阈值（纳秒）。
    pub limit: i64,
    /// 是否已处于断流告警状态（避免重复告警）。
    pub alerting: bool,
}

/// 静默断流检测器。
#[derive(Debug, Default)]
pub struct SilenceDetector {
    entries: HashMap<SilenceKey, SilenceState>,
}

/// 一次检测的结果事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SilenceEvent {
    /// 某频道首次超过静默阈值（应告警 + 触发重连，docs/06 §3.2 A5）。
    WentSilent(SilenceKey),
    /// 某频道恢复（消息恢复到达）。
    Recovered(SilenceKey),
}

impl SilenceDetector {
    /// 注册一个监控（开播时调用）。
    pub fn watch(&mut self, channel: &str, symbol: &str, limit_nanos: i64, now: i64) {
        let key = SilenceKey::new(channel, symbol);
        self.entries.insert(
            key,
            SilenceState {
                last_seen: now,
                limit: limit_nanos,
                alerting: false,
            },
        );
    }

    /// 记录一次消息到达（刷新时间戳；从断流恢复时产生 `Recovered`）。
    pub fn on_message(&mut self, channel: &str, symbol: &str, now: i64) -> Vec<SilenceEvent> {
        let key = SilenceKey::new(channel, symbol);
        let Some(st) = self.entries.get_mut(&key) else {
            return Vec::new();
        };
        st.last_seen = now;
        if st.alerting {
            st.alerting = false;
            return vec![SilenceEvent::Recovered(key)];
        }
        Vec::new()
    }

    /// 移除监控（退订/断开时调用）。
    pub fn unwatch(&mut self, channel: &str, symbol: &str) {
        self.entries.remove(&SilenceKey::new(channel, symbol));
    }

    /// 定时检测：返回本次状态变化事件。
    pub fn check(&mut self, now: i64) -> Vec<SilenceEvent> {
        let mut events = Vec::new();
        for (key, st) in self.entries.iter_mut() {
            let silent = now.saturating_sub(st.last_seen) > st.limit;
            if silent && !st.alerting {
                st.alerting = true;
                events.push(SilenceEvent::WentSilent(key.clone()));
            }
        }
        events
    }

    /// 监控条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 查询某频道状态（只读）。
    pub fn state(&self, channel: &str, symbol: &str) -> Option<SilenceState> {
        self.entries.get(&SilenceKey::new(channel, symbol)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEC: i64 = 1_000_000_000;

    #[test]
    fn silent_then_recover() {
        let mut d = SilenceDetector::default();
        d.watch("futures.trades", "BTC_USDT", 5 * SEC, 0);
        // 4s 无消息：未超阈值
        assert!(d.check(4 * SEC).is_empty());
        // 6s 无消息：断流
        let ev = d.check(6 * SEC);
        assert_eq!(
            ev,
            vec![SilenceEvent::WentSilent(SilenceKey::new(
                "futures.trades",
                "BTC_USDT"
            ))]
        );
        // 重复检测不重复告警
        assert!(d.check(7 * SEC).is_empty());
        // 消息恢复
        let ev = d.on_message("futures.trades", "BTC_USDT", 8 * SEC);
        assert_eq!(
            ev,
            vec![SilenceEvent::Recovered(SilenceKey::new(
                "futures.trades",
                "BTC_USDT"
            ))]
        );
        // 恢复后正常消息不再产生事件
        assert!(
            d.on_message("futures.trades", "BTC_USDT", 9 * SEC)
                .is_empty()
        );
    }

    #[test]
    fn per_symbol_independent() {
        let mut d = SilenceDetector::default();
        d.watch("futures.trades", "BTC_USDT", 5 * SEC, 0);
        d.watch("futures.trades", "ETH_USDT", 5 * SEC, 0);
        d.on_message("futures.trades", "ETH_USDT", 3 * SEC);
        let ev = d.check(6 * SEC);
        assert_eq!(ev.len(), 1);
        assert_eq!(
            ev[0],
            SilenceEvent::WentSilent(SilenceKey::new("futures.trades", "BTC_USDT"))
        );
    }

    #[test]
    fn unwatch_stops_monitoring() {
        let mut d = SilenceDetector::default();
        d.watch("futures.trades", "S", 5 * SEC, 0);
        d.unwatch("futures.trades", "S");
        assert!(d.check(100 * SEC).is_empty());
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn message_to_unknown_channel_ignored() {
        let mut d = SilenceDetector::default();
        assert!(d.on_message("x", "y", 0).is_empty());
    }
}
