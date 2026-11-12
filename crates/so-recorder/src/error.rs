//! so-recorder 错误类型（docs/04 E1/E2）。
//!
//! Journal 相关的失败来自 `so-journal`（层 L2），此处用 `#[from]` 转发，
//! 保持本 crate 调用点的 `?` 写法不变（ADR-0016）。

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

    /// Journal 读写失败（转发 `so-journal` 的错误，保留其错误码）。
    #[error("Journal 失败：{0}")]
    Journal(#[from] so_journal::JournalError),

    /// 行情帧解析失败（Raw Journal 仍保留原始帧，docs/06 §3.2 A1）。
    #[error("帧解析失败（{code}）：原始帧已入 Journal")]
    Parse {
        /// Gate 适配器错误码。
        code: &'static str,
    },

    /// Telegram 告警发送失败（重试耗尽后丢弃，ADR-0013）。
    #[error("告警发送失败：{0}")]
    Alert(String),

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
            Self::Journal(e) => journal_code(e),
            Self::Parse { .. } => "RecorderParse",
            Self::Alert(_) => "RecorderAlert",
            Self::Io(_) => "RecorderIo",
        }
    }
}

/// 转发 `so-journal` 自身的错误码（不折叠成 `RecorderJournal`）。
///
/// 码是跨语言判别的依据（docs/04 E2）：把不同失败原因折成一个码会让
/// 「打开失败」与「zstd 失败」在日志里无法区分。
const fn journal_code(e: &so_journal::JournalError) -> &'static str {
    e.code()
}
