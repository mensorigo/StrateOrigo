//! so-cli 入口：解析参数、初始化日志、分发命令。

mod args;
mod dispatch;
mod record;

use std::process::ExitCode;

use clap::Parser;
use so_telemetry::{ConsoleOutput, LogContext, TelemetryConfig};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let cli = args::Cli::parse();
    let telemetry = TelemetryConfig {
        context: LogContext::new("so-cli"),
        log_dir: None,
        console: ConsoleOutput::Human,
        default_filter: cli.log.clone(),
    };
    let _guard = match so_telemetry::init(&telemetry) {
        Ok(guard) => guard,
        Err(e) => {
            #[allow(clippy::print_stderr)] // 日志系统初始化失败时只能直接写 stderr
            {
                eprintln!("so-cli: {e}");
            }
            return ExitCode::FAILURE;
        }
    };
    dispatch::run(cli.command)
}
