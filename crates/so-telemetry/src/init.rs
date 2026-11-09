//! 全局日志初始化：组合过滤器、文件层、控制台层。每个进程调用一次。

use std::io;

use tracing_appender::non_blocking::{ErrorCounter, WorkerGuard};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::{ConsoleOutput, TelemetryConfig};
use crate::error::TelemetryError;
use crate::file::file_writer;
use crate::json::json_layer;

/// 读取日志过滤规则的环境变量。
pub const FILTER_ENV: &str = "SO_LOG";

/// 持有它期间后台写线程保持运行；drop 时把缓冲区刷到文件。必须在 `main` 中保留到退出。
#[must_use = "drop 后日志文件不再写入"]
#[derive(Debug)]
pub struct TelemetryGuard {
    file: Option<(WorkerGuard, ErrorCounter)>,
}

impl TelemetryGuard {
    /// 因缓冲区满而被丢弃的日志行数（应作为指标 `log_dropped_total` 上报）。
    pub fn dropped_lines(&self) -> usize {
        self.file
            .as_ref()
            .map_or(0, |(_, counter)| counter.dropped_lines())
    }
}

/// 初始化全局日志。
pub fn init(config: &TelemetryConfig) -> Result<TelemetryGuard, TelemetryError> {
    let filter = match EnvFilter::try_from_env(FILTER_ENV) {
        Ok(filter) => filter,
        Err(_) => EnvFilter::try_new(&config.default_filter)
            .map_err(|e| TelemetryError::Filter(e.to_string()))?,
    };

    let (file_layer, file_guard) = match &config.log_dir {
        Some(dir) => {
            let (writer, guard, counter) = file_writer(dir.clone(), &config.context.instance)?;
            let layer = json_layer(writer, &config.context);
            (Some(layer), Some((guard, counter)))
        }
        None => (None, None),
    };

    let human = (config.console == ConsoleOutput::Human).then(|| {
        tracing_subscriber::fmt::layer()
            .with_writer(io::stderr)
            .with_target(false)
            .compact()
    });
    let json_console =
        (config.console == ConsoleOutput::Json).then(|| json_layer(io::stderr, &config.context));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(human)
        .with(json_console)
        .try_init()?;

    Ok(TelemetryGuard { file: file_guard })
}
