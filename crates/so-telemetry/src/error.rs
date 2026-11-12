//! 日志初始化错误类型（docs/04 E1/E2）。

/// 初始化失败。
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// 过滤规则无法解析。
    #[error("日志过滤规则无效：{0}")]
    Filter(String),
    /// 无法创建日志文件。
    #[error("无法创建日志文件：{0}")]
    File(#[from] tracing_appender::rolling::InitError),
    /// 全局日志已初始化过。
    #[error("全局日志初始化失败：{0}")]
    Init(#[from] tracing_subscriber::util::TryInitError),
}

impl TelemetryError {
    /// 稳定英文错误码（docs/04 E2）：`<域前缀><变体>`，本域前缀 `Telemetry`。
    ///
    /// 所有错误类型都提供 `code()`，日志字段 `code` 直接取自它（docs/04 §3.6
    /// G4）——调用点若拿不到码就只能拼字符串，日志无法按码聚合。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Filter(_) => "TelemetryFilter",
            Self::File(_) => "TelemetryFile",
            Self::Init(_) => "TelemetryInit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 码带域前缀（docs/04 E2）。
    ///
    /// 只覆盖可构造的变体：`File`/`Init` 包装的外部错误类型没有公开构造函数，
    /// 拿不到实例；其余变体的前缀由 `code()` 的 match 穷尽性保证。
    #[test]
    fn code_is_prefixed() {
        assert_eq!(
            TelemetryError::Filter(String::new()).code(),
            "TelemetryFilter"
        );
    }
}
