//! `so-cli data download`：Gate 官方 REST 历史数据下载入口（docs/06 A10）。
//!
//! 其余 `data` 子命令（normalize/check/ls）属 M2，未实现时提示里程碑。

use std::process::ExitCode;

use so_recorder::{DownloadChannel, download_history};

/// Gate REST 基址（docs/venues/gate.md §1）。
const GATE_REST: &str = "https://api.gateio.ws/api/v4";

/// 执行 `data download`。
pub fn run_download(
    venue: &str,
    symbol: &str,
    from: &str,
    to: Option<&str>,
    channel: &str,
) -> ExitCode {
    let Some(ch) = DownloadChannel::from_name(channel) else {
        tracing::error!(channel, "未知频道（支持 trades | candles | funding）");
        return ExitCode::FAILURE;
    };
    if venue != "gate-usdt" {
        tracing::error!(venue, "当前只支持 gate-usdt（OKX 后置）");
        return ExitCode::FAILURE;
    }
    let Some(from_secs) = parse_date_secs(from) else {
        tracing::error!(from, "日期格式应为 YYYY-MM-DD");
        return ExitCode::FAILURE;
    };
    // `to` 取该日结束（次日 00:00，作为不含上界的终点）；不填 = 到今天结束。
    let to_secs = match to {
        Some(t) => match parse_date_secs(t) {
            Some(s) => s.saturating_add(86_400),
            None => {
                tracing::error!(to = t, "日期格式应为 YYYY-MM-DD");
                return ExitCode::FAILURE;
            }
        },
        None => today_end_secs(),
    };
    if from_secs >= to_secs {
        tracing::error!(from, to, "起始日期必须早于结束日期");
        return ExitCode::FAILURE;
    }
    tracing::info!(venue, symbol, channel, from, "开始历史下载");
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!(error = %e, "创建 tokio 运行时失败");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(download_history(
        GATE_REST,
        "usdt",
        ch,
        symbol,
        from_secs,
        to_secs,
        std::path::Path::new("data"),
    )) {
        Ok(out) => {
            tracing::info!(rows = out.rows, complete = out.complete, "下载完成");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!(code = e.code(), "下载失败（已下载部分保留，重跑续传）");
            ExitCode::FAILURE
        }
    }
}

/// `YYYY-MM-DD` → Unix 秒（当日 00:00 UTC）。无 chrono 依赖的手写解析。
fn parse_date_secs(s: &str) -> Option<i64> {
    let (y, rest) = s.split_once('-')?;
    let (m, d) = rest.split_once('-')?;
    let y: i64 = y.parse().ok()?;
    let m: i64 = m.parse().ok()?;
    let d: i64 = d.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    days_from_civil(y, m, d).checked_mul(86_400)
}

/// 今天 UTC 结束（明日 00:00）。
fn today_end_secs() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    (now / 86_400 + 1) * 86_400
}

/// (y, m, d) → 自 1970-01-01 的天数。Howard Hinnant 算法（days_from_civil）。
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parsing() {
        assert_eq!(parse_date_secs("2026-01-01"), Some(1_767_225_600));
        assert_eq!(parse_date_secs("1970-01-01"), Some(0));
        assert_eq!(parse_date_secs("2000-02-29"), Some(951_782_400)); // 闰年
        assert_eq!(parse_date_secs("2026-13-01"), None);
        assert_eq!(parse_date_secs("2026-01-32"), None);
        assert_eq!(parse_date_secs("not-a-date"), None);
    }

    #[test]
    fn today_end_is_midnight_aligned() {
        let t = today_end_secs();
        assert_eq!(t % 86_400, 0);
        assert!(t > 1_700_000_000);
    }
}
