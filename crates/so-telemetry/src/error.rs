//! 日志初始化错误类型。

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
