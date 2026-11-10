//! Telegram 告警（docs/02 §Telegram、docs/03 §479；ADR-0013）。
//!
//! 直接调用 Bot API（复用 reqwest，不引 teloxide）：`POST /bot{token}/sendMessage`。
//! 发送侧是独立后台任务 + 无界队列：调用方只做非阻塞入队，永不阻塞录制。
//! 重试（ADR-0013）：每条最多 5 次指数退避（429 按 `retry_after`），耗尽后
//! 记 ERROR 日志丢弃。消息前缀 `[REC]`（docs/04 S10；录制器是数据面进程）。

use std::time::Duration;

use tokio::sync::{mpsc, watch};

use crate::error::RecorderError;
use crate::silence::SilenceEvent;

/// 单条消息最大尝试次数（含首次）。
const MAX_ATTEMPTS: u32 = 5;

/// 退避起始值；每次翻倍，封顶 60s。
const BACKOFF_START: Duration = Duration::from_secs(1);

/// 退避封顶。
const BACKOFF_MAX: Duration = Duration::from_secs(60);

/// 告警句柄：克隆廉价；未配置时为禁用态（发送是 no-op）。
#[derive(Debug, Clone, Default)]
pub struct AlertHandle {
    tx: Option<mpsc::UnboundedSender<String>>,
}

impl AlertHandle {
    /// 禁用句柄（无 Telegram 配置时使用）。
    pub fn disabled() -> Self {
        Self { tx: None }
    }

    /// 是否已启用。
    pub fn is_enabled(&self) -> bool {
        self.tx.is_some()
    }

    /// 非阻塞入队一条告警；队列无界，不会失败（发送结果由后台任务负责）。
    pub fn send(&self, text: &str) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(text.to_owned());
        }
    }
}

/// 按配置启动告警后台任务，返回句柄。
///
/// token 缺失（环境变量未设）或 `chat_id` 为空时返回禁用句柄并告警一次——
/// 录制不因告警未配置而失败（告警同时进日志，docs/03 §595）。
pub fn start_alert_task(
    config: &crate::config::AlertConfig,
    shutdown: watch::Receiver<bool>,
) -> AlertHandle {
    let token = std::env::var(&config.bot_token_env)
        .ok()
        .filter(|t| !t.is_empty());
    if token.is_none() || config.chat_id.is_empty() {
        tracing::warn!(
            env = %config.bot_token_env,
            "Telegram 告警未配置（缺 token 或 chat_id），仅日志告警"
        );
        return AlertHandle::disabled();
    }
    let (tx, rx) = mpsc::unbounded_channel();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            tracing::error!(
                code = "RecorderAlert",
                "Telegram HTTP 客户端初始化失败：{e}"
            );
        });
    let http = match http {
        Ok(h) => h,
        Err(()) => return AlertHandle::disabled(),
    };
    tokio::spawn(run_alert_task(
        rx,
        http,
        token.unwrap_or_default(),
        config.chat_id.clone(),
        shutdown,
    ));
    AlertHandle { tx: Some(tx) }
}

/// 后台发送循环：逐条发送；关闭信号到达时尽力清空队列（每条只试一次）。
async fn run_alert_task(
    mut rx: mpsc::UnboundedReceiver<String>,
    http: reqwest::Client,
    token: String,
    chat_id: String,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        let msg = tokio::select! {
            _ = shutdown.changed() => {
                drain_best_effort(&mut rx, &http, &token, &chat_id).await;
                return;
            }
            m = rx.recv() => match m {
                Some(m) => m,
                None => return, // 所有句柄已释放
            },
        };
        if let Err(e) = send_with_retry(&http, &msg, &token, &chat_id).await {
            tracing::error!(code = e.code(), "Telegram 告警丢弃（重试耗尽）");
        }
    }
}

/// 关闭时清空队列：每条只尝试一次，不再重试（不阻塞收尾）。
async fn drain_best_effort(
    rx: &mut mpsc::UnboundedReceiver<String>,
    http: &reqwest::Client,
    token: &str,
    chat_id: &str,
) {
    while let Ok(msg) = rx.try_recv() {
        if send_once(http, token, chat_id, &msg).await.is_err() {
            tracing::error!("Telegram 告警丢弃（关闭清空，单次发送失败）");
        }
    }
}

/// 带重试的发送（ADR-0013：最多 5 次，指数退避，429 按 retry_after）。
async fn send_with_retry(
    http: &reqwest::Client,
    text: &str,
    token: &str,
    chat_id: &str,
) -> Result<(), RecorderError> {
    let mut delay = BACKOFF_START;
    for attempt in 1..=MAX_ATTEMPTS {
        match send_once(http, token, chat_id, text).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt == MAX_ATTEMPTS => {
                return Err(RecorderError::Alert(e.describe()));
            }
            Err(e) => {
                tracing::warn!(
                    attempt,
                    reason = e.describe(),
                    "Telegram 发送失败，退避重试"
                );
                let wait = e.retry_after.unwrap_or(delay);
                tokio::time::sleep(wait).await;
                delay = (delay * 2).min(BACKOFF_MAX);
            }
        }
    }
    unreachable!("循环必然在 MAX_ATTEMPTS 内返回")
}

/// 单次 API 调用。
async fn send_once(
    http: &reqwest::Client,
    token: &str,
    chat_id: &str,
    text: &str,
) -> Result<(), SendError> {
    let body = serde_json::json!({ "chat_id": chat_id, "text": text });
    let resp = http
        .post(api_url(token))
        .json(&body)
        .send()
        .await
        .map_err(|e| SendError::new(format!("请求失败：{e}")))?;
    let status = resp.status().as_u16();
    let payload = resp
        .text()
        .await
        .map_err(|e| SendError::new(format!("读取响应失败：{e}")))?;
    parse_send_response(status, &payload)
}

/// Bot API URL。
fn api_url(token: &str) -> String {
    format!("https://api.telegram.org/bot{token}/sendMessage")
}

/// 单次发送失败（携带 429 的 retry_after 供退避）。
#[derive(Debug, Clone)]
struct SendError {
    reason: String,
    retry_after: Option<Duration>,
}

impl SendError {
    fn new(reason: String) -> Self {
        Self {
            reason,
            retry_after: None,
        }
    }

    fn describe(&self) -> String {
        self.reason.clone()
    }
}

/// 解析 Bot API 响应：`ok == true` 成功；否则取 `parameters.retry_after`（429）。
fn parse_send_response(status: u16, body: &str) -> Result<(), SendError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SendError::new(format!("响应非 JSON：{e}")))?;
    if value
        .get("ok")
        .is_some_and(|v| v.as_bool().unwrap_or(false))
    {
        return Ok(());
    }
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("未知错误");
    let retry_after = value
        .pointer("/parameters/retry_after")
        .and_then(|v| v.as_u64())
        .map(Duration::from_secs);
    Err(SendError {
        reason: format!("HTTP {status}：{description}"),
        retry_after,
    })
}

/// 静默事件 → 告警文本（`[REC]` 前缀，docs/04 S10）。
pub fn silence_alert_text(ev: &SilenceEvent) -> String {
    match ev {
        SilenceEvent::WentSilent(k) => {
            format!(
                "[REC] 静默断流：{} {}（超过阈值，docs/06 A5）",
                k.channel, k.symbol
            )
        }
        SilenceEvent::Recovered(k) => {
            format!("[REC] 断流恢复：{} {}", k.channel, k.symbol)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_shape() {
        assert_eq!(
            api_url("TOK"),
            "https://api.telegram.org/botTOK/sendMessage"
        );
    }

    #[test]
    fn response_parsing() {
        assert!(parse_send_response(200, r#"{"ok":true,"result":{}}"#).is_ok());
        let err = parse_send_response(
            429,
            r#"{"ok":false,"error_code":429,"description":"Too Many Requests","parameters":{"retry_after":30}}"#,
        )
        .unwrap_err();
        assert_eq!(err.retry_after, Some(Duration::from_secs(30)));
        assert!(err.describe().contains("Too Many Requests"));
        let err = parse_send_response(
            400,
            r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#,
        )
        .unwrap_err();
        assert_eq!(err.retry_after, None);
        assert!(err.describe().contains("chat not found"));
        // 非 JSON
        assert!(parse_send_response(200, "<html>").is_err());
    }

    #[test]
    fn silence_text_has_rec_prefix() {
        use crate::silence::SilenceKey;
        let t = silence_alert_text(&SilenceEvent::WentSilent(SilenceKey::new(
            "futures.trades",
            "BTC_USDT",
        )));
        assert!(t.starts_with("[REC] "));
        assert!(t.contains("futures.trades"));
        assert!(t.contains("BTC_USDT"));
        let t = silence_alert_text(&SilenceEvent::Recovered(SilenceKey::new("c", "s")));
        assert!(t.starts_with("[REC] 断流恢复"));
    }

    #[test]
    fn disabled_handle_is_noop() {
        let h = AlertHandle::disabled();
        assert!(!h.is_enabled());
        h.send("不应 panic"); // 无接收端也不报错
    }
}
