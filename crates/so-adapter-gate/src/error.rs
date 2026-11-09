//! Gate 适配器错误类型（docs/04 E1/E2）。

/// Gate 行情解析错误。
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GateError {
    /// JSON 结构不符合预期（缺字段 / 类型错误）。
    #[error("消息结构与 Gate 文档不符")]
    BadShape,
    /// 十进制字段无法精确解析为定点（含错误码前缀）。
    #[error("定点解析失败：{code}")]
    Fixed {
        /// so-core 错误码。
        code: &'static str,
    },
    /// 毫秒时间戳超出 UnixNanos 表示范围。
    #[error("时间戳溢出")]
    TsOverflow,
    /// 品种符号非法。
    #[error("品种符号非法：{code}")]
    Symbol {
        /// so-model 错误码。
        code: &'static str,
    },
}

impl From<so_core::FixedError> for GateError {
    fn from(e: so_core::FixedError) -> Self {
        Self::Fixed { code: e.code() }
    }
}

impl From<so_model::SymbolError> for GateError {
    fn from(e: so_model::SymbolError) -> Self {
        Self::Symbol { code: e.code() }
    }
}

impl GateError {
    /// 稳定英文错误码（docs/04 E2）。
    pub const fn code(self) -> &'static str {
        match self {
            Self::BadShape => "GateBadShape",
            Self::Fixed { .. } => "GateFixed",
            Self::TsOverflow => "GateTsOverflow",
            Self::Symbol { .. } => "GateSymbol",
        }
    }
}
