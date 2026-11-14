//! 命令分发：已实现的命令在这里调用对应模块；尚未实现的命令提示计划里程碑（docs/05）。

use std::process::ExitCode;

use crate::exit::{EXIT_NOT_IMPLEMENTED, EXIT_OK};

use so_core::RunMode;

use crate::args::{Command, DataCommand};

/// 执行命令。
pub fn run(command: Command) -> ExitCode {
    let milestone = match command {
        Command::Info => {
            print_info();
            return ExitCode::from(EXIT_OK);
        }
        Command::Record { config, venue } => return crate::record::run(&config, venue.as_deref()),
        Command::Data(DataCommand::Download {
            venue,
            symbol,
            from,
            to,
            channel,
        }) => {
            return crate::data_download::run_download(
                &venue,
                &symbol,
                &from,
                to.as_deref(),
                &channel,
            );
        }
        Command::Data(DataCommand::Normalize { date }) => {
            return crate::data_normalize::run(&date);
        }
        Command::Data(DataCommand::Check {
            date,
            dataset,
            name,
        }) => {
            return crate::data_check::run(&date, dataset, name.as_deref());
        }
        Command::Data(DataCommand::Bar { date, kind, value }) => {
            return crate::data_bar::run(&date, &kind, value);
        }
        Command::Data(DataCommand::Ls) => return crate::data_ls::run(),
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
