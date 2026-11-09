//! 纳秒级 UTC 时间戳。

/// 自 Unix 纪元起的纳秒数（UTC）。所有时间统一用它；`i64` 可表示到 2262 年。
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnixNanos(pub i64);

impl UnixNanos {
    /// Unix 纪元。
    pub const EPOCH: Self = Self(0);

    /// 每秒纳秒数。
    pub const NANOS_PER_SEC: i64 = 1_000_000_000;

    /// 原始纳秒数。
    pub const fn as_nanos(self) -> i64 {
        self.0
    }

    /// 由秒构造。
    pub fn from_secs(secs: i64) -> Option<Self> {
        secs.checked_mul(Self::NANOS_PER_SEC).map(Self)
    }

    /// 由毫秒构造（交易所时间戳常用毫秒）。
    pub fn from_millis(millis: i64) -> Option<Self> {
        millis.checked_mul(1_000_000).map(Self)
    }

    /// 由微秒构造。
    pub fn from_micros(micros: i64) -> Option<Self> {
        micros.checked_mul(1_000).map(Self)
    }

    /// 加上一段纳秒（可为负）。
    pub fn checked_add_nanos(self, nanos: i64) -> Option<Self> {
        self.0.checked_add(nanos).map(Self)
    }

    /// 与更早时间点的差（纳秒，可为负）。
    pub fn nanos_since(self, earlier: Self) -> Option<i64> {
        self.0.checked_sub(earlier.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(UnixNanos::from_secs(2), Some(UnixNanos(2_000_000_000)));
        assert_eq!(
            UnixNanos::from_millis(1_700_000_000_123).map(UnixNanos::as_nanos),
            Some(1_700_000_000_123_000_000)
        );
        assert_eq!(UnixNanos::from_secs(i64::MAX), None);
        assert_eq!(UnixNanos(10).nanos_since(UnixNanos(3)), Some(7));
        assert_eq!(UnixNanos(i64::MAX).checked_add_nanos(1), None);
    }
}
