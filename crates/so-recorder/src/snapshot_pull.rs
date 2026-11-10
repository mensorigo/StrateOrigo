//! REST 拉取任务：启动时合约元数据，之后按定时器拉深度快照（docs/06 §3.2 A6）。
//!
//! 快照与元数据都先写 Raw Journal 再解析（A1）；网络失败只记日志，
//! 下个周期继续（录制进程不因单次 REST 失败退出）。

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use so_adapter_gate::parse_contracts;
use so_core::Scale;
use tokio::sync::watch;
use tokio::time::interval;

use crate::config::{RecordingConfig, VenueConfig};
use crate::error::RecorderError;
use crate::journal::JournalWriter;
use crate::snapshot::{RestUrls, SnapshotTimer, record_contracts, record_order_book_snapshot};
use crate::timefmt::now_nanos;

/// 快照任务：启动时拉合约元数据，之后每 `snapshot_secs` 拉一次深度快照。
pub async fn run_snapshot_task(
    config: &RecordingConfig,
    venue: &VenueConfig,
    data_dir: &Path,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), RecorderError> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| RecorderError::Rest(format!("HTTP 客户端初始化失败：{e}")))?;
    let urls = RestUrls::new(&venue.rest_url);
    let settle = venue.settle.as_str();
    let mut journal = JournalWriter::new(data_dir, &venue.venue, "snapshot")?;

    // 启动时拉一次合约元数据（价格精度用于解析校验；失败不阻塞深度录制）。
    let scales = fetch_scales(&http, &urls, settle, &mut journal).await;

    let mut timer = SnapshotTimer::new(config.snapshot_secs, now_nanos().0);
    let mut tick = interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = shutdown.changed() => return Ok(()),
            _ = tick.tick() => {
                let now = now_nanos().0;
                if !timer.take(now) {
                    continue;
                }
                for sym in &venue.symbols {
                    let scale = scales.get(sym).copied().unwrap_or(Scale::ZERO);
                    pull_one(&http, &urls, settle, sym, scale, &mut journal).await;
                }
            }
        }
    }
}

/// 拉合约元数据并建立 `品种 → 价格精度` 映射；失败时返回空映射。
async fn fetch_scales(
    http: &reqwest::Client,
    urls: &RestUrls,
    settle: &str,
    journal: &mut JournalWriter,
) -> HashMap<String, Scale> {
    let url = urls.contracts(settle);
    let raw = match fetch_text(http, &url).await {
        Ok(raw) => raw,
        Err(e) => {
            tracing::warn!(code = e.code(), url = url.as_str(), "拉取合约元数据失败");
            return HashMap::new();
        }
    };
    let ts = now_nanos();
    if let Err(e) = record_contracts(journal, &raw, ts) {
        tracing::warn!(code = e.code(), "合约元数据解析失败（原始帧已入 Journal）");
    }
    parse_contracts(&raw).map_or_else(
        |_| HashMap::new(),
        |list| {
            list.into_iter()
                .filter_map(|c| scale_from_tick(&c.order_price_round).map(|s| (c.name, s)))
                .collect()
        },
    )
}

/// 拉一个品种的深度快照并写入 Journal。
async fn pull_one(
    http: &reqwest::Client,
    urls: &RestUrls,
    settle: &str,
    symbol: &str,
    scale: Scale,
    journal: &mut JournalWriter,
) {
    let url = urls.order_book(settle, symbol, 100);
    let raw = match fetch_text(http, &url).await {
        Ok(raw) => raw,
        Err(e) => {
            tracing::warn!(code = e.code(), symbol, "拉取深度快照失败");
            return;
        }
    };
    let ts = now_nanos();
    match record_order_book_snapshot(journal, symbol, &raw, ts, scale) {
        Ok(out) => tracing::debug!(
            symbol = out.symbol.as_str(),
            snapshot_id = out.snapshot_id,
            bids = out.bid_levels,
            asks = out.ask_levels,
            "深度快照已写入 Journal"
        ),
        Err(e) => tracing::warn!(
            code = e.code(),
            symbol,
            "快照解析失败（原始帧已入 Journal）"
        ),
    }
}

/// GET 文本；非 2xx 视为错误。带 Size-Decimal 头（docs/venues/gate.md §2.4）。
async fn fetch_text(http: &reqwest::Client, url: &str) -> Result<String, RecorderError> {
    let resp = http
        .get(url)
        .header("Accept", "application/json")
        .header("X-Gate-Size-Decimal", "1")
        .send()
        .await
        .map_err(|e| RecorderError::Rest(format!("GET 失败：{e}")))?;
    if !resp.status().is_success() {
        return Err(RecorderError::Rest(format!("HTTP {}", resp.status())));
    }
    resp.text()
        .await
        .map_err(|e| RecorderError::Rest(format!("读取响应失败：{e}")))
}

/// 最小价格单位字符串 → 价格精度（`"0.0001"` → 4，`"1"` → 0）。
fn scale_from_tick(tick: &str) -> Option<Scale> {
    let decimals = tick
        .split_once('.')
        .map_or(0, |(_, frac)| frac.trim_end_matches('0').len());
    u8::try_from(decimals).ok().and_then(|d| Scale::new(d).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_to_scale() {
        assert_eq!(scale_from_tick("0.1").map(Scale::decimals), Some(1));
        assert_eq!(scale_from_tick("0.0001").map(Scale::decimals), Some(4));
        assert_eq!(scale_from_tick("0.0010").map(Scale::decimals), Some(3));
        assert_eq!(scale_from_tick("1").map(Scale::decimals), Some(0));
        assert_eq!(scale_from_tick("0.0000000000000000001"), None); // 超过 18 位
    }
}
