//! 时间工具：UTC 日期/时刻格式化与本地接收时钟（docs/06 §3.1）。
//!
//! 不引入 chrono：日期换算用 Howard Hinnant 的 `civil_from_days` 算法。
//! `now_nanos` 生产环境应替换为 quanta 高精度时钟；此处用 `SystemTime` 兜底。

use std::time::{SystemTime, UNIX_EPOCH};

use so_core::UnixNanos;

/// Unix 秒 → `YYYY-MM-DD`（UTC）。
pub fn format_utc_date(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Unix 秒 → `HH:MM:SS`（UTC）。
pub fn format_utc_time(unix_secs: i64) -> String {
    let s = unix_secs.rem_euclid(86_400);
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

/// 当前 Unix 纳秒（ts_recv 打点）。
pub fn now_nanos() -> UnixNanos {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    UnixNanos(dur.as_nanos() as i64)
}

/// days since 1970-01-01 → (year, month, day)。Howard Hinnant 算法。
fn civil_from_days(z: i64) -> (i64, u64, u64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m as u64, d as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_formatting() {
        assert_eq!(format_utc_date(1_700_000_000), "2023-11-14");
        assert_eq!(format_utc_date(0), "1970-01-01");
        assert_eq!(format_utc_date(951_782_400), "2000-02-29"); // 闰年
        assert_eq!(format_utc_time(1_700_000_000), "22:13:20");
        // 时钟可读且为 2023 年之后
        assert!(now_nanos().0 > 1_700_000_000_000_000_000);
    }
}
