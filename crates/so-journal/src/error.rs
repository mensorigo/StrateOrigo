//! Journal 域错误类型（docs/04 E1/E2/E7）。

/// Raw Journal 读写错误。
///
/// 上下文用命名字段承载（状态/字节偏移/容量），外部工具原文才用裸 `String`；
/// 错误码前缀 `Journal*`（docs/04 §3.3 登记表）。
#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    /// 打开或创建 Journal 文件失败。
    #[error("打开 Journal {path}：{source}")]
    Open {
        /// 文件路径。
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },

    /// 写入 Journal 失败。
    #[error("写入 Journal {path}（{bytes} 字节）：{source}")]
    Write {
        /// 文件路径。
        path: String,
        /// 本次写入字节数。
        bytes: u64,
        /// 底层 IO 错误。
        source: std::io::Error,
    },

    /// zstd 编解码失败。
    #[error("zstd {stage}：{source}")]
    Codec {
        /// `init` / `encode` / `finish` / `decode`。
        stage: &'static str,
        /// 底层错误。
        source: std::io::Error,
    },

    /// 序列化或反序列化一行失败。
    #[error("Journal 行 {stage}：{source}")]
    Row {
        /// `encode` / `decode`。
        stage: &'static str,
        /// 底层错误。
        source: serde_json::Error,
    },

    /// 写入器未初始化（未滚动到任何小时即写入）。
    #[error("写入器未初始化（未滚动到任何小时）")]
    NotRolled,

    /// 读取校验和边车失败。
    #[error("读取校验和 {path}：{source}")]
    Checksum {
        /// 边车路径。
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
}

impl JournalError {
    /// 稳定英文错误码（docs/04 E2）。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Open { .. } => "JournalOpen",
            Self::Write { .. } => "JournalWrite",
            Self::Codec { .. } => "JournalCodec",
            Self::Row { .. } => "JournalRow",
            Self::NotRolled => "JournalNotRolled",
            Self::Checksum { .. } => "JournalChecksum",
        }
    }
}
