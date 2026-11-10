//! 历史下载编排：分页循环 + manifest 断点续传（docs/06 A10）。
//!
//! 输出布局：`data/vendor/gate-hist/{channel}/{contract}/{YYYY-MM-DD}.jsonl`
//! （每行一条原始 JSON 对象，不可变）+ 同目录 `manifest.json`
//! （来源 URL、下载时间、剩余窗口）。重跑同一命令从 manifest 继续。
//!
//! 分页方向（端点排序不同，实测 2026-11-10）：
//! - trades / funding_rate：**降序**（最新在前），窗口上界 hi 逐步下移；
//! - candlesticks：**升序**（最旧在前），窗口下界 lo 逐步上移。
//!
//! 满页（count == limit）时收窄到本页覆盖的边界秒，重新拉取边界秒（有界重叠，
//! 去重交给 normalize）。`to`/`from` 只接受整数秒（实测小数被拒）。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::download::{DownloadChannel, REQUEST_INTERVAL, build_url, page_range};
use crate::error::RecorderError;
use crate::timefmt::{format_utc_date, now_nanos};

/// 下载进度清单（断点续传：剩余窗口 + 计数 + 来源）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadManifest {
    /// 交易所（gate）。
    pub venue: String,
    /// 频道名。
    pub channel: String,
    /// 品种。
    pub contract: String,
    /// 目标窗口下界（Unix 秒，含）。
    pub from_secs: i64,
    /// 目标窗口上界（Unix 秒，含）。
    pub to_secs: i64,
    /// 剩余窗口下界。
    pub lo: i64,
    /// 剩余窗口上界。
    pub hi: i64,
    /// 已写入行数。
    pub rows: u64,
    /// 最近一次请求 URL（来源记录，A10）。
    pub last_url: String,
    /// 最近一次下载时间（Unix 纳秒）。
    pub last_ts_recv: i64,
}

/// 一次下载任务的结果摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadOutcome {
    /// 本次新增行数。
    pub rows: u64,
    /// 是否已覆盖整个目标窗口。
    pub complete: bool,
}

/// 执行下载（阻塞直到完成或达到终点）。
///
/// `from_secs`/`to_secs` 为目标窗口（含端点）；已有 manifest 时从其剩余窗口继续。
pub async fn download_history(
    rest_base: &str,
    settle: &str,
    channel: DownloadChannel,
    contract: &str,
    from_secs: i64,
    to_secs: i64,
    data_dir: &Path,
) -> Result<DownloadOutcome, RecorderError> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| RecorderError::Rest(format!("HTTP 客户端初始化失败：{e}")))?;
    let dir = output_dir(data_dir, channel, contract);
    fs::create_dir_all(&dir)?;
    let manifest_path = dir.join("manifest.json");
    let mut manifest = read_manifest(&manifest_path, channel, contract, from_secs, to_secs);
    let limit = channel_page_limit(channel) as usize;
    let mut rows_new: u64 = 0;

    while manifest.lo <= manifest.hi {
        let url = build_url(
            rest_base,
            settle,
            channel,
            contract,
            manifest.lo,
            manifest.hi,
        );
        let page = fetch_page(&http, &url).await?;
        let Some(range) = page_range(channel, &page) else {
            // 空页：剩余窗口无数据。
            manifest.lo = manifest.hi.saturating_add(1);
            break;
        };
        let page_rows = append_page(&dir, &page, range.min_ts)?;
        rows_new = rows_new.saturating_add(page_rows);
        let full = range.count >= limit;
        let descending = channel_descending(channel);
        let covered_all = if descending {
            !full || range.min_ts <= manifest.lo
        } else {
            !full || range.max_ts >= manifest.hi
        };
        if covered_all {
            manifest.lo = manifest.hi.saturating_add(1);
        } else if descending {
            manifest.hi = range.min_ts; // 有界重叠：边界秒重取，去重交给 normalize
        } else {
            manifest.lo = range.max_ts;
        }
        manifest.rows = manifest.rows.saturating_add(page_rows);
        manifest.last_url = url.clone();
        manifest.last_ts_recv = now_nanos().0;
        write_manifest(&manifest_path, &manifest)?;
        tokio::time::sleep(REQUEST_INTERVAL).await;
    }

    Ok(DownloadOutcome {
        rows: rows_new,
        complete: manifest.lo > manifest.hi,
    })
}

/// 输出目录：`{data_dir}/vendor/gate-hist/{channel}/{contract}`。
fn output_dir(data_dir: &Path, channel: DownloadChannel, contract: &str) -> PathBuf {
    let name = channel_name(channel);
    data_dir
        .join("vendor")
        .join("gate-hist")
        .join(name)
        .join(contract)
}

/// 频道名（CLI 与目录名）。
fn channel_name(channel: DownloadChannel) -> &'static str {
    match channel {
        DownloadChannel::Trades => "trades",
        DownloadChannel::Candles => "candles",
        DownloadChannel::Funding => "funding",
    }
}

/// 单页最大条数。
fn channel_page_limit(channel: DownloadChannel) -> u32 {
    match channel {
        DownloadChannel::Trades | DownloadChannel::Funding => 1000,
        DownloadChannel::Candles => 2000,
    }
}

/// 是否降序（最新在前）。
fn channel_descending(channel: DownloadChannel) -> bool {
    matches!(channel, DownloadChannel::Trades | DownloadChannel::Funding)
}

/// 读 manifest；不存在时新建（剩余窗口 = 目标窗口）。
fn read_manifest(
    path: &Path,
    channel: DownloadChannel,
    contract: &str,
    from_secs: i64,
    to_secs: i64,
) -> DownloadManifest {
    if let Ok(text) = fs::read_to_string(path)
        && let Ok(m) = serde_json::from_str::<DownloadManifest>(&text)
    {
        return m;
    }
    DownloadManifest {
        venue: "gate".to_owned(),
        channel: channel_name(channel).to_owned(),
        contract: contract.to_owned(),
        from_secs,
        to_secs,
        lo: from_secs,
        hi: to_secs,
        rows: 0,
        last_url: String::new(),
        last_ts_recv: 0,
    }
}

/// 写 manifest（原子替换）。
fn write_manifest(path: &Path, m: &DownloadManifest) -> Result<(), RecorderError> {
    let text = serde_json::to_string_pretty(m)
        .map_err(|e| RecorderError::Rest(format!("manifest 序列化失败：{e}")))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// 一页 JSON 数组 → 按行追加到日期文件；返回新增行数。
fn append_page(dir: &Path, page: &str, last_ts_secs: i64) -> Result<u64, RecorderError> {
    let value = serde_json::from_str::<serde_json::Value>(page)
        .map_err(|e| RecorderError::Rest(format!("响应不是 JSON：{e}")))?;
    let Some(arr) = value.as_array() else {
        return Err(RecorderError::Rest("响应不是数组".into()));
    };
    if arr.is_empty() {
        return Ok(0);
    }
    let path = dir.join(format!("{}.jsonl", format_utc_date(last_ts_secs)));
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let mut written = 0u64;
    for item in arr {
        let line = serde_json::to_string(item)
            .map_err(|e| RecorderError::Rest(format!("行序列化失败：{e}")))?;
        file.write_all(line.as_bytes())
            .map_err(|e| RecorderError::Rest(format!("写入失败：{e}")))?;
        file.write_all(b"\n")
            .map_err(|e| RecorderError::Rest(format!("写入失败：{e}")))?;
        written = written.saturating_add(1);
    }
    Ok(written)
}

/// GET 一页；非 2xx 报错。
async fn fetch_page(http: &reqwest::Client, url: &str) -> Result<String, RecorderError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("so-recorder-dl-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn manifest_roundtrip_and_resume() {
        let dir = temp_dir("manifest");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("manifest.json");
        let m = read_manifest(&path, DownloadChannel::Trades, "BTC_USDT", 100, 200);
        assert_eq!((m.lo, m.hi), (100, 200));
        let mut m = m;
        m.hi = 150; // 已下载 [151, 200]
        write_manifest(&path, &m).unwrap();
        // 重读：剩余窗口保留（断点续传）
        let m2 = read_manifest(&path, DownloadChannel::Trades, "BTC_USDT", 100, 200);
        assert_eq!((m2.lo, m2.hi), (100, 150));
        assert_eq!(m2.contract, "BTC_USDT");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn page_appended_as_jsonl() {
        let dir = temp_dir("page");
        fs::create_dir_all(&dir).unwrap();
        // 1970-01-01 的日期文件
        let n = append_page(&dir, r#"[{"id":1,"t":60},{"id":2,"t":61}]"#, 61).unwrap();
        assert_eq!(n, 2);
        let text = fs::read_to_string(dir.join("1970-01-01.jsonl")).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"id":1,"t":60}"#);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn output_dir_shape() {
        let d = output_dir(Path::new("data"), DownloadChannel::Funding, "BTC_USDT");
        let s = d.to_string_lossy().replace('\\', "/");
        assert_eq!(s, "data/vendor/gate-hist/funding/BTC_USDT");
    }

    #[test]
    fn descending_channels() {
        assert!(channel_descending(DownloadChannel::Trades));
        assert!(channel_descending(DownloadChannel::Funding));
        assert!(!channel_descending(DownloadChannel::Candles));
    }
}
