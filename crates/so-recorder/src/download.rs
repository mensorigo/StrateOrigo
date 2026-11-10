//! Gate 官方 REST 历史数据下载（docs/06 §2 / A10，`so-cli data download`）。
//!
//! 三个公共端点（无需鉴权，单接口 200r/10s，按 IP）：
//! - `GET /futures/{settle}/trades`：逐笔（`from`/`to` 秒级窗口 + `limit`）
//! - `GET /futures/{settle}/candlesticks`：K 线（单次最多 2000 根）
//! - `GET /futures/{settle}/funding_rate`：资金费历史
//!
//! 输出为 JSONL（每行一条原始 JSON 对象），不可变追加；
//! 每个文件配 `manifest.json` 记录来源 URL、下载时间与游标（断点续传）。

use std::time::Duration;

/// 下载频道：决定端点与游标字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadChannel {
    /// 逐笔成交（游标 = 最后一条的 `create_time_ms`）。
    Trades,
    /// K 线（游标 = 最后一根的 `t`）。
    Candles,
    /// 资金费率历史（游标 = 最后一条的 `t`）。
    Funding,
}

impl DownloadChannel {
    /// CLI 参数名（`--channel trades|candles|funding`）。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "trades" => Some(Self::Trades),
            "candles" => Some(Self::Candles),
            "funding" => Some(Self::Funding),
            _ => None,
        }
    }

    /// 端点路径段。
    fn path(self) -> &'static str {
        match self {
            Self::Trades => "trades",
            Self::Candles => "candlesticks",
            Self::Funding => "funding_rate",
        }
    }

    /// 单页最大条数（官方文档：trades/funding 1000；candlesticks 2000）。
    fn page_limit(self) -> u32 {
        match self {
            Self::Trades | Self::Funding => 1000,
            Self::Candles => 2000,
        }
    }
}

/// 一次下载请求的 URL 构造。
pub(crate) fn build_url(
    base: &str,
    settle: &str,
    channel: DownloadChannel,
    contract: &str,
    from_secs: i64,
    to_secs: i64,
) -> String {
    let limit = channel.page_limit();
    let path = channel.path();
    if matches!(channel, DownloadChannel::Candles) {
        // K 线端点用 from/to/interval；interval 由调用方按窗口宽度选择
        format!(
            "{base}/futures/{settle}/{path}?contract={contract}&from={from_secs}&to={to_secs}&limit={limit}"
        )
    } else {
        format!(
            "{base}/futures/{settle}/{path}?contract={contract}&from={from_secs}&to={to_secs}&limit={limit}"
        )
    }
}

/// 一页响应的时间覆盖范围与条数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PageRange {
    /// 最早时间戳（秒）。
    pub min_ts: i64,
    /// 最晚时间戳（秒）。
    pub max_ts: i64,
    /// 条数。
    pub count: usize,
}

/// 统计一页响应的时间覆盖范围（端点排序不同，取全页 min/max 而非末条）。
///
/// trades 用 `create_time`（浮点，毫秒精度）；candles/funding 用 `t`（整数秒）。
/// 返回 None 表示响应为空数组或无法解析。
pub(crate) fn page_range(channel: DownloadChannel, page: &str) -> Option<PageRange> {
    let value = serde_json::from_str::<serde_json::Value>(page).ok()?;
    let arr = value.as_array()?;
    if arr.is_empty() {
        return None;
    }
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let mut count = 0usize;
    for item in arr {
        let ts: Option<i64> = match channel {
            DownloadChannel::Trades => item
                .get("create_time")
                .and_then(|v| v.as_f64())
                .map(|t| t as i64),
            DownloadChannel::Candles | DownloadChannel::Funding => {
                item.get("t").and_then(|v| v.as_i64())
            }
        };
        if let Some(t) = ts {
            min = min.min(t);
            max = max.max(t);
            count += 1;
        }
    }
    if count == 0 {
        return None;
    }
    Some(PageRange {
        min_ts: min,
        max_ts: max,
        count,
    })
}

/// 限速：公共接口 200r/10s（docs/venues/gate.md §5.2），取 80% 余量。
pub const REQUEST_INTERVAL: Duration = Duration::from_millis(50);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_names() {
        assert_eq!(
            DownloadChannel::from_name("trades"),
            Some(DownloadChannel::Trades)
        );
        assert_eq!(
            DownloadChannel::from_name("candles"),
            Some(DownloadChannel::Candles)
        );
        assert_eq!(
            DownloadChannel::from_name("funding"),
            Some(DownloadChannel::Funding)
        );
        assert_eq!(DownloadChannel::from_name("tickers"), None);
    }

    #[test]
    fn url_shapes() {
        let base = "https://api.gateio.ws/api/v4";
        let u = build_url(base, "usdt", DownloadChannel::Trades, "BTC_USDT", 100, 200);
        assert_eq!(
            u,
            "https://api.gateio.ws/api/v4/futures/usdt/trades?contract=BTC_USDT&from=100&to=200&limit=1000"
        );
        let u = build_url(base, "usdt", DownloadChannel::Candles, "BTC_USDT", 100, 200);
        assert!(u.contains("candlesticks"));
        assert!(u.contains("limit=2000"));
    }

    #[test]
    fn page_range_covers_whole_page() {
        // trades 降序（最新在前）：min/max 应取全页而非末条
        let trades = r#"[{"id":2,"create_time":150.5},{"id":1,"create_time":149.5}]"#;
        let r = page_range(DownloadChannel::Trades, trades).unwrap();
        assert_eq!((r.min_ts, r.max_ts, r.count), (149, 150, 2));
        // funding 降序
        let funding = r#"[{"r":"0.1","t":200},{"r":"0.1","t":100}]"#;
        let r = page_range(DownloadChannel::Funding, funding).unwrap();
        assert_eq!((r.min_ts, r.max_ts), (100, 200));
        // candles 升序
        let candles = r#"[{"t":100},{"t":200}]"#;
        let r = page_range(DownloadChannel::Candles, candles).unwrap();
        assert_eq!((r.min_ts, r.max_ts), (100, 200));
        assert_eq!(page_range(DownloadChannel::Funding, "[]"), None);
        assert_eq!(page_range(DownloadChannel::Funding, "not json"), None);
    }
}
