//! Gate WS 协议层（docs/venues/gate.md §2/§3）。
//!
//! 只负责**协议本身**：构造订阅/退订帧、解析信封、识别错误码、
//! 维护 `X-Gate-Size-Decimal` 头要求下的消息语义。网络 I/O 由调用方
//! （tokio-tungstenite）驱动——本模块不依赖 tokio，便于测试。

use serde::{Deserialize, Serialize};

/// WS 客户端请求帧（订阅/退订同构，docs/venues/gate.md §2.1）。
#[derive(Debug, Clone, Serialize)]
pub struct RequestFrame {
    /// 秒级时间（与服务器时差 ≤ 60s）。
    pub time: i64,
    /// 可选请求 ID（原样返回，用于配对确认）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// 频道名。
    pub channel: String,
    /// `subscribe` / `unsubscribe`。
    pub event: String,
    /// 订阅参数（合约名 + 频道附加参数）。
    pub payload: Vec<String>,
}

impl RequestFrame {
    /// 构造订阅帧。
    pub fn subscribe(time: i64, id: u64, channel: &str, payload: Vec<String>) -> Self {
        Self {
            time,
            id: Some(id),
            channel: channel.to_owned(),
            event: "subscribe".to_owned(),
            payload,
        }
    }

    /// 序列化为 WS 文本帧。
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// WS 服务端帧（信封，确认与推送同构）。
#[derive(Debug, Clone, Deserialize)]
pub struct EnvelopeFrame {
    /// 频道名。
    pub channel: String,
    /// `update` = 数据推送；`subscribe`/`unsubscribe` = 确认。
    pub event: String,
    /// 非空即订阅失败（确认帧）。
    pub error: Option<GateWsError>,
    /// 数据体（保留原始 JSON 文本，由频道解析器处理）。
    pub result: Option<serde_json::Value>,
    /// 秒级时间。
    pub time: Option<i64>,
    /// 毫秒时间（2022-11 起全部频道带）。
    pub time_ms: Option<i64>,
}

/// 信封错误对象。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GateWsError {
    /// 错误码：1 请求体格式错、2 参数无效、3 服务端异常、4 鉴权失败。
    pub code: i64,
    /// 错误消息（人读）。
    pub message: Option<String>,
}

/// 帧分类结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameKind {
    /// 数据推送。
    Update,
    /// 订阅确认。
    SubscribeAck,
    /// 退订确认。
    UnsubscribeAck,
    /// 订阅/退订失败。
    Error(GateWsError),
    /// 无法分类（event 缺失/未知）。
    Unknown,
}

impl EnvelopeFrame {
    /// 分类。
    pub fn kind(&self) -> FrameKind {
        if let Some(err) = &self.error {
            return FrameKind::Error(err.clone());
        }
        match self.event.as_str() {
            "update" => FrameKind::Update,
            "subscribe" => FrameKind::SubscribeAck,
            "unsubscribe" => FrameKind::UnsubscribeAck,
            _ => FrameKind::Unknown,
        }
    }

    /// 从 WS 文本帧解析信封；失败返回 None（协议外帧，如 pong）。
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }
}

/// 应用层 ping 帧（`futures.ping`，docs/venues/gate.md §2.3）。
pub fn ping_frame(time: i64) -> String {
    format!(r#"{{"time":{time},"channel":"futures.ping","event":"subscribe","payload":[]}}"#)
}

/// 判断是否为 pong 响应。
pub fn is_pong(env: &EnvelopeFrame) -> bool {
    env.channel == "futures.pong"
}

/// 判断是否为服务升级通知（`event=update`、`channel=*.system`、
/// `result.type="upgrade"` → 立即主动重连，docs/venues/gate.md §2.3）。
pub fn is_upgrade_notice(env: &EnvelopeFrame) -> bool {
    if env.event != "update" || !env.channel.ends_with(".system") {
        return false;
    }
    env.result
        .as_ref()
        .and_then(|r| r.get("type"))
        .and_then(|t| t.as_str())
        == Some("upgrade")
}

/// Gate WS 订阅 URL（合约 USDT 实盘，docs/venues/gate.md §1）。
pub const WS_URL_USDT_LIVE: &str = "wss://fx-ws.gateio.ws/v4/ws/usdt";

/// Gate REST 基址（实盘）。
pub const REST_URL_LIVE: &str = "https://api.gateio.ws/api/v4";

/// 订阅超时时间（秒，请求 id 配对）。
pub const SUBSCRIBE_TIMEOUT_SECS: i64 = 10;

/// 重连退避序列（毫秒）：1s、2s、5s、10s、30s、60s 封顶。
pub const BACKOFF_MS: [u64; 6] = [1000, 2000, 5000, 10_000, 30_000, 60_000];

/// 第 n 次重连的退避（毫秒）；n 从 0 起。
pub fn backoff_ms(attempt: u32) -> u64 {
    let idx = (attempt as usize).min(BACKOFF_MS.len() - 1);
    BACKOFF_MS[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_frame_shape() {
        let f = RequestFrame::subscribe(
            1_700_000_000,
            7,
            "futures.order_book_update",
            vec!["BTC_USDT".to_owned(), "100ms".to_owned(), "100".to_owned()],
        );
        let json = f.to_json().unwrap();
        assert!(json.contains(r#""event":"subscribe""#));
        assert!(json.contains(r#""channel":"futures.order_book_update""#));
        assert!(json.contains(r#""payload":["BTC_USDT","100ms","100"]"#));
        assert!(json.contains(r#""id":7"#));
    }

    #[test]
    fn parse_update_frame() {
        let env = EnvelopeFrame::parse(
            r#"{"time":1611541000,"time_ms":1611541000123,"channel":"futures.trades","event":"update","error":null,"result":[{"id":1}]}"#,
        )
        .unwrap();
        assert_eq!(env.kind(), FrameKind::Update);
        assert_eq!(env.channel, "futures.trades");
        assert_eq!(env.time_ms, Some(1611541000123));
        assert!(env.result.is_some());
    }

    #[test]
    fn parse_subscribe_ack() {
        let env = EnvelopeFrame::parse(
            r#"{"time":1,"channel":"futures.trades","event":"subscribe","error":null,"result":{"status":"success"}}"#,
        )
        .unwrap();
        assert_eq!(env.kind(), FrameKind::SubscribeAck);
    }

    #[test]
    fn parse_error_frame() {
        let env = EnvelopeFrame::parse(
            r#"{"time":1,"channel":"futures.trades","event":"subscribe","error":{"code":2,"message":"invalid param"},"result":null}"#,
        )
        .unwrap();
        match env.kind() {
            FrameKind::Error(e) => assert_eq!(e.code, 2),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn upgrade_notice_detection() {
        let env = EnvelopeFrame::parse(
            r#"{"time":1,"channel":"futures.system","event":"update","result":{"type":"upgrade"}}"#,
        )
        .unwrap();
        assert!(is_upgrade_notice(&env));
        let normal = EnvelopeFrame::parse(
            r#"{"time":1,"channel":"futures.trades","event":"update","result":{"type":"x"}}"#,
        )
        .unwrap();
        assert!(!is_upgrade_notice(&normal));
    }

    #[test]
    fn pong_detection() {
        let env = EnvelopeFrame::parse(r#"{"time":1,"channel":"futures.pong","event":"update"}"#)
            .unwrap();
        assert!(is_pong(&env));
    }

    #[test]
    fn backoff_grows_then_caps() {
        assert_eq!(backoff_ms(0), 1000);
        assert_eq!(backoff_ms(1), 2000);
        assert_eq!(backoff_ms(5), 60_000);
        assert_eq!(backoff_ms(100), 60_000);
    }

    #[test]
    fn non_json_frame_is_none() {
        assert!(EnvelopeFrame::parse("pong").is_none());
    }
}
