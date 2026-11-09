//! so-recorder 错误类型（docs/04 E1/E2）。

/// 录制错误。
#[derive(Debug, thiserror::Error)]
pub enum RecorderError {
    /// 配置文件解析失败。
    #[error("配置解析失败：{0}")]
    Config(String),

    /// WebSocket 连接层错误（连接/发送/接收）。
    #[error("WS 错误：{0}")]
    Ws(String),

    /// REST 请求失败。
    #[error("REST 错误：{0}")]
    Rest(String),

    /// Journal 写入失败。
    #[error("Journal 写入失败：{0}")]
    Journal(String),

    /// 行情帧解析失败（Raw Journal 仍保留原始帧，docs/06 §3.2 A1）。
    #[error("帧解析失败（{code}）：原始帧已入 Journal")]
    Parse {
        /// Gate 适配器错误码。
        code: &'static str,
    },

    /// IO 错误。
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}

impl RecorderError {
    /// 稳定英文错误码（docs/04 E2）。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "RecorderConfig",
            Self::Ws(_) => "RecorderWs",
            Self::Rest(_) => "RecorderRest",
            Self::Journal(_) => "RecorderJournal",
            Self::Parse { .. } => "RecorderParse",
            Self::Io(_) => "RecorderIo",
        }
    }
}
