//! 日志初始化（docs/03 §18）。入口文件：只做模块声明与重新导出。
//!
//! - **运行日志**：JSON Lines，按天滚动，写入 `<log_dir>/<instance>.<date>.jsonl`；
//!   非阻塞写入，缓冲区满时丢弃并计数（运行日志允许丢失，审计日志不走这里）。
//! - **控制台**：本地 CLI 用人类可读格式输出到 stderr。
//! - 过滤规则来自环境变量 `SO_LOG`（`EnvFilter` 语法），没有时用配置的默认值。
//!
//! 本 crate 属于冷路径；内核线程不得直接调用 `tracing` 宏（docs/04 H9）。

mod config;
mod error;
mod file;
mod init;
mod json;

pub use config::{ConsoleOutput, LogContext, TelemetryConfig};
pub use error::TelemetryError;
pub use file::file_writer;
pub use init::{FILTER_ENV, TelemetryGuard, init};
pub use json::json_layer;
