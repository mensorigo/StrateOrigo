//! 时钟端口。热路径只能通过 [`Clock`] 取时间（docs/04 H7）。
//!
//! 实现：`LiveClock`（实盘/模拟盘，后续在 so-engine 中基于 quanta 实现）、
//! `SimClock`（回测，so-backtest），以及这里的 [`ManualClock`]（测试与工具）。

use core::cell::Cell;

use crate::time::UnixNanos;

/// 时钟端口。
pub trait Clock {
    /// 当前时间。
    fn now(&self) -> UnixNanos;
}

/// 手动推进的时钟，用于测试。单线程使用（不是 `Sync`）。
#[derive(Debug, Default)]
pub struct ManualClock {
    now: Cell<UnixNanos>,
}

impl ManualClock {
    /// 从给定时间开始。
    pub const fn new(start: UnixNanos) -> Self {
        Self {
            now: Cell::new(start),
        }
    }

    /// 设置当前时间。
    pub fn set(&self, t: UnixNanos) {
        self.now.set(t);
    }

    /// 前进 `nanos` 纳秒；溢出时保持不变并返回 `false`。
    pub fn advance(&self, nanos: i64) -> bool {
        match self.now.get().checked_add_nanos(nanos) {
            Some(t) => {
                self.now.set(t);
                true
            }
            None => false,
        }
    }
}

impl Clock for ManualClock {
    fn now(&self) -> UnixNanos {
        self.now.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock() {
        let c = ManualClock::new(UnixNanos(100));
        assert!(c.advance(50));
        assert_eq!(c.now(), UnixNanos(150));
        c.set(UnixNanos(i64::MAX));
        assert!(!c.advance(1));
        assert_eq!(c.now(), UnixNanos(i64::MAX));
    }
}
