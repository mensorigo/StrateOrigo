//! 命令分发：已实现的命令在这里调用对应模块；尚未实现的命令提示计划里程碑（docs/05）。

use std::process::ExitCode;

use so_core::RunMode;

use crate::args::Command;

/// "尚未实现"的退出码，与参数错误（clap 用 2）区分。
const EXIT_NOT_IMPLEMENTED: u8 = 3;

/// 执行命令。
pub fn run(command: Command) -> ExitCode {
    let milestone = match command {
        Command::Info => {
            print_info();
            return ExitCode::SUCCESS;
        }
        Command::Record { config, venue } => return crate::record::run(&config, venue.as_deref()),
        Command::Data(_) => "M1 / M2",
        Command::Bt { .. } | Command::Exp(_) => "M3",
        Command::Strategy { .. } => "M4",
        Command::Trader(_) | Command::Watch { .. } | Command::Logs(_) => "M5",
        Command::Kill(_) => "M6",
    };
    tracing::error!(milestone, "该命令尚未实现");
    ExitCode::from(EXIT_NOT_IMPLEMENTED)
}

#[allow(clippy::print_stdout)] // CLI 的输出就是 stdout
fn print_info() {
    println!("so-cli {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!(
        "{:<10} {:<12} {:<10} {:<14}",
        "模式", "标识", "可加载密钥", "每交易所账户数"
    );
    for mode in RunMode::ALL {
        let accounts = mode
            .max_accounts_per_venue()
            .map_or_else(|| "不限".to_owned(), |n| n.to_string());
        let keys = if mode.may_load_keys() { "是" } else { "否" };
        println!(
            "{:<12} {:<14} {:<14} {:<14}",
            mode.as_str(),
            mode.label(),
            keys,
            accounts
        );
    }
    println!();
    println!("每个账户只运行一个策略（ADR-0001）。");
}
