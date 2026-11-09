//! 日志配置：实例身份 [`LogContext`]、控制台输出方式、整体配置 [`TelemetryConfig`]。

use std::path::PathBuf;

use serde_json::Value;
use so_core::RunMode;

/// 每条记录都带的上下文（实例身份）。
#[derive(Clone, Debug, Default)]
pub struct LogContext {
    /// 实例名，如 `live-okx-funding`、`paper-lab`、`so-cli`。
    pub instance: String,
    /// 运行模式；CLI 等非交易进程为 `None`。
    pub mode: Option<RunMode>,
    /// 交易所（实盘实例固定一个）。
    pub venue: Option<String>,
    /// 账户（实盘实例固定一个；模拟盘多账户时由事件字段携带）。
    pub account: Option<String>,
    /// 策略实例（同上）。
    pub strategy: Option<String>,
}

impl LogContext {
    /// 只有实例名的上下文。
    pub fn new(instance: impl Into<String>) -> Self {
        Self {
            instance: instance.into(),
            ..Self::default()
        }
    }

    /// 写入每条 JSON 记录的固定字段（只包含有值的项）。
    pub(crate) fn static_fields(&self) -> Vec<(&'static str, Value)> {
        let mut fields = vec![("instance", Value::from(self.instance.as_str()))];
        if let Some(mode) = self.mode {
            fields.push(("mode", Value::from(mode.as_str())));
        }
        for (key, value) in [
            ("venue", &self.venue),
            ("account", &self.account),
            ("strategy", &self.strategy),
        ] {
            if let Some(v) = value {
                fields.push((key, Value::from(v.as_str())));
            }
        }
        fields
    }
}

/// 控制台输出方式。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConsoleOutput {
    /// 不输出到控制台（VPS 上的服务进程）。
    Off,
    /// 人类可读文本到 stderr（本地 CLI）。
    Human,
    /// JSON Lines 到 stderr。
    Json,
}

/// 日志配置。
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// 实例身份。
    pub context: LogContext,
    /// 日志文件目录；`None` 表示不写文件。
    pub log_dir: Option<PathBuf>,
    /// 控制台输出。
    pub console: ConsoleOutput,
    /// 没有设置 `SO_LOG` 时的过滤规则，如 `"info"`、`"warn,so_trader=info"`。
    pub default_filter: String,
}
