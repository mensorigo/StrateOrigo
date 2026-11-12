//! 本地接收时钟（docs/06 §3.1、§3.2 A2）。
//!
//! 纯格式化函数已下沉到 `so-journal`（写入端与读取端共用同一份换算），
//! 本模块只保留**进程时钟**——它属于采集进程，不属于文件格式。
//! `now_nanos` 生产环境应替换为 quanta 高精度时钟；此处用 `SystemTime` 兜底。

use std::time::{SystemTime, UNIX_EPOCH};

pub use so_journal::{format_utc_date, format_utc_time};

use so_core::UnixNanos;

/// 当前 Unix 纳秒（ts_recv 打点）。
pub fn now_nanos() -> UnixNanos {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    UnixNanos(dur.as_nanos() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_is_readable() {
        // 日期格式化的测试随函数在 so-journal；这里只验证进程时钟可用。
        assert!(now_nanos().0 > 1_700_000_000_000_000_000);
    }
}
