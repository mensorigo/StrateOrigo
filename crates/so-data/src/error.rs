//! so-data 错误类型（docs/04 E1/E2）。

/// 数据层错误。
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    /// Journal 读取失败。
    #[error("Journal 读取失败：{0}")]
    Journal(String),
    /// 帧解析失败（原始帧保留在 Journal，仅计数）。
    #[error("帧解析失败：code={code}")]
    Parse {
        /// 稳定错误码（来源适配器或 recorder）。
        code: String,
    },
    /// Parquet 写出失败。
    #[error("Parquet 写出失败：{0}")]
    Parquet(String),
    /// 输出表与频道不匹配等用法错误。
    #[error("用法错误：{0}")]
    Usage(String),
}

impl DataError {
    /// 稳定英文错误码（docs/04 E2）。
    pub fn code(&self) -> &'static str {
        match self {
            Self::Journal(_) => "DataJournal",
            Self::Parse { .. } => "DataParse",
            Self::Parquet(_) => "DataParquet",
            Self::Usage(_) => "DataUsage",
        }
    }
}
