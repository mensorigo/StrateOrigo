//! `so-cli record`：加载录制配置并运行 so-recorder 内核（docs/06 §3）。

use std::process::ExitCode;

use crate::exit::{EXIT_FAILED, EXIT_OK, EXIT_USAGE, io_code, log_io_error};
use so_recorder::{RecordingConfig, run_recorder};

/// 执行 `record` 命令。
pub fn run(config_path: &str, venue_filter: Option<&str>) -> ExitCode {
    let text = match std::fs::read_to_string(config_path) {
        Ok(t) => t,
        Err(e) => {
            log_io_error(io_code(&e), "读取录制配置失败", &e);
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let config = match RecordingConfig::parse(&text) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(path = config_path, code = e.code(), "解析录制配置失败");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    // 交易所过滤：--venue gate 只录制配置中该 venue 的部分。
    let venues: Vec<_> = config
        .venues
        .iter()
        .filter(|v| venue_filter.is_none_or(|f| v.venue == f))
        .collect();
    if venues.is_empty() {
        tracing::error!(
            filter = venue_filter.unwrap_or_default(),
            "没有匹配的交易所配置"
        );
        return ExitCode::from(EXIT_USAGE);
    }
    tracing::info!(
        venues = venues
            .iter()
            .map(|v| v.venue.as_str())
            .collect::<Vec<_>>()
            .join(","),
        data_dir = config.data_dir.as_str(),
        "启动行情录制（Ctrl-C 停止）"
    );
    // 录制内核是 async：CLI 其余命令是同步的，这里按需起一个多线程运行时。
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!(code = "CliRuntime", error = %e, "创建 tokio 运行时失败");
            return ExitCode::from(EXIT_FAILED);
        }
    };
    for venue in venues {
        if let Err(e) = rt.block_on(run_recorder(&config, venue)) {
            tracing::error!(
                code = e.code(),
                venue = venue.venue.as_str(),
                "录制器异常退出"
            );
            return ExitCode::from(EXIT_FAILED);
        }
    }
    ExitCode::from(EXIT_OK)
}
