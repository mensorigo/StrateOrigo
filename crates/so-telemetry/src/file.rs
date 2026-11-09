//! 日志文件写入器：按天滚动、非阻塞、缓冲区满时丢弃并计数。

use std::path::PathBuf;

use tracing_appender::non_blocking::{ErrorCounter, NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use crate::error::TelemetryError;

/// 创建 `<dir>/<instance>.<YYYY-MM-DD>.jsonl` 写入器。
///
/// 返回的 `WorkerGuard` 必须保留到进程退出，drop 时把缓冲区刷到文件。
pub fn file_writer(
    dir: PathBuf,
    instance: &str,
) -> Result<(NonBlocking, WorkerGuard, ErrorCounter), TelemetryError> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(instance)
        .filename_suffix("jsonl")
        .build(dir)?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let counter = writer.error_counter();
    Ok((writer, guard, counter))
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;
    use crate::{LogContext, json_layer};

    #[test]
    fn creates_daily_jsonl() {
        let dir = std::env::temp_dir().join(format!("so-telemetry-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (writer, guard, counter) = file_writer(dir.clone(), "paper-lab").unwrap();
        let layer = json_layer(writer, &LogContext::new("paper-lab"));
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || tracing::info!("写入文件"));
        drop(guard); // 刷盘

        let files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(files.len(), 1);
        let name = files[0].file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            name.starts_with("paper-lab.") && name.ends_with(".jsonl"),
            "{name}"
        );
        let content = std::fs::read_to_string(&files[0]).unwrap();
        assert!(content.contains("\"msg\":\"写入文件\""));
        assert_eq!(counter.dropped_lines(), 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
