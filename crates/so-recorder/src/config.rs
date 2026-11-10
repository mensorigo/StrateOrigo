//! 录制配置（`configs/recording.toml`，docs/06 §3.3）。

use serde::{Deserialize, Serialize};

use crate::error::RecorderError;

/// 录制配置根。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// 交易所与品种列表。
    pub venues: Vec<VenueConfig>,
    /// 录制频道列表（全部品种订阅同一组）。
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
    /// Journal 输出根目录（默认 `data/raw`，docs/06 §6）。
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    /// 滚动周期秒（默认 3600 = 小时滚动）。
    #[serde(default = "default_roll_secs")]
    pub roll_secs: u64,
    /// 深度快照定时拉取间隔秒（docs/06 §3.2 A6）。
    #[serde(default = "default_snapshot_secs")]
    pub snapshot_secs: u64,
    /// Telegram 告警（可选）。
    pub alert: Option<AlertConfig>,
}

/// 一个交易所的录制目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    /// 交易所 ID（当前只支持 `gate`）。
    pub venue: String,
    /// WS 地址（默认 Gate 合约实盘 `wss://fx-ws.gateio.ws/v4/ws/usdt`）。
    pub ws_url: String,
    /// REST 基址（默认 `https://api.gateio.ws/api/v4`）。
    pub rest_url: String,
    /// 结算币种路径段（Gate REST 用，默认 `usdt`）。
    #[serde(default = "default_settle")]
    pub settle: String,
    /// 品种列表（Gate 合约名，如 `BTC_USDT`）。
    pub symbols: Vec<String>,
    /// 是否启用双连接冗余（默认 true，docs/06 §3.2 A3）。
    #[serde(default = "default_true")]
    pub dual_conn: bool,
}

/// 一个订阅频道。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 频道名，如 `futures.order_book_update`。
    pub channel: String,
    /// 订阅 payload 的附加参数（Gate 深度频道是 `["BTC_USDT","100ms","100"]`）。
    pub params: Vec<String>,
    /// 最大无消息间隔秒（静默断流阈值，docs/06 §3.2 A5）。
    pub max_silence_secs: u64,
    /// 是否在该频道上维护本地订单簿（深度频道为 true）。
    #[serde(default)]
    pub maintain_book: bool,
}

/// Telegram 告警配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Bot token（从环境变量读取，不落配置文件——docs/04 S8）。
    #[serde(skip)]
    pub bot_token_env: String,
    /// Chat ID。
    pub chat_id: String,
}

fn default_data_dir() -> String {
    "data/raw".to_owned()
}

fn default_roll_secs() -> u64 {
    3600
}

fn default_snapshot_secs() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

fn default_settle() -> String {
    "usdt".to_owned()
}

impl RecordingConfig {
    /// 从 TOML 文本解析（容忍 UTF-8 BOM——Windows 编辑器常带 BOM）。
    pub fn parse(text: &str) -> Result<Self, RecorderError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        toml::from_str(text).map_err(|e| RecorderError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_config() {
        let text = r#"
data_dir = "data/raw"
roll_secs = 3600
snapshot_secs = 300

[[venues]]
venue = "gate"
ws_url = "wss://fx-ws.gateio.ws/v4/ws/usdt"
rest_url = "https://api.gateio.ws/api/v4"
symbols = ["BTC_USDT", "ETH_USDT"]
dual_conn = true

[[channels]]
channel = "futures.order_book_update"
params = ["100ms", "100"]
max_silence_secs = 5
maintain_book = true

[[channels]]
channel = "futures.trades"
params = []
max_silence_secs = 60
"#;
        let cfg = RecordingConfig::parse(text).unwrap();
        assert_eq!(cfg.venues.len(), 1);
        assert_eq!(cfg.venues[0].symbols.len(), 2);
        assert_eq!(cfg.channels.len(), 2);
        assert!(cfg.channels[0].maintain_book);
        assert_eq!(cfg.roll_secs, 3600);
        assert_eq!(cfg.snapshot_secs, 300);
    }

    #[test]
    fn defaults_apply() {
        let text = r#"
[[venues]]
venue = "gate"
ws_url = "wss://fx-ws.gateio.ws/v4/ws/usdt"
rest_url = "https://api.gateio.ws/api/v4"
symbols = ["BTC_USDT"]
"#;
        let cfg = RecordingConfig::parse(text).unwrap();
        assert!(cfg.venues[0].dual_conn); // 默认双连接
        assert_eq!(cfg.venues[0].settle, "usdt"); // 默认结算币种
        assert_eq!(cfg.data_dir, "data/raw");
        assert_eq!(cfg.roll_secs, 3600);
        assert_eq!(cfg.channels.len(), 0);
    }

    #[test]
    fn rejects_bad_config() {
        let err = RecordingConfig::parse("not = [valid").unwrap_err();
        assert_eq!(err.code(), "RecorderConfig");
    }

    #[test]
    fn tolerates_utf8_bom() {
        // Windows 编辑器常写 BOM；配置文件必须可读（实测踩坑 2026-11-10）
        let text = "\u{feff}\n[[venues]]\nvenue = \"gate\"\nws_url = \"wss://x\"\nrest_url = \"https://x\"\nsymbols = []\n";
        let cfg = RecordingConfig::parse(text).unwrap();
        assert_eq!(cfg.venues.len(), 1);
    }
}
