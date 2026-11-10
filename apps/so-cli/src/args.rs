//! 命令行参数定义（clap）。只描述命令结构，不含执行逻辑。

use clap::{Args, Parser, Subcommand};

/// StrateOrigo 命令行工具。
#[derive(Debug, Parser)]
#[command(name = "so-cli", version, about)]
pub struct Cli {
    /// 日志过滤规则（EnvFilter 语法）；环境变量 SO_LOG 优先。
    #[arg(long, global = true, default_value = "warn")]
    pub log: String,

    #[command(subcommand)]
    pub command: Command,
}

/// 顶层命令。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// 显示版本、工具链与运行模式规则。
    Info,
    /// 数据：下载、规范化、清洗、质量检查、数据集列表。
    #[command(subcommand)]
    Data(DataCommand),
    /// 回测：结果写入实验库。
    Bt {
        /// 策略名，如 mm-basic。
        strategy: String,
        /// 策略配置文件。
        #[arg(short, long)]
        config: String,
        /// 数据集哈希。
        #[arg(long)]
        dataset: String,
    },
    /// 实验库查询。
    #[command(subcommand)]
    Exp(ExpCommand),
    /// 策略生命周期迁移（检查晋级门槛）。
    Strategy {
        /// 策略与版本，如 mm-basic@0.3.1。
        target: String,
        /// 目标阶段，如 paper。
        #[arg(long)]
        to: String,
        /// 依据的实验 run_id。
        #[arg(long)]
        evidence: String,
    },
    /// 经 gRPC 控制 so-trader 实例。
    #[command(subcommand)]
    Trader(TraderCommand),
    /// 终端实时监控界面。
    Watch {
        /// 实例名；不填则显示全部实例（实盘与模拟盘用不同颜色区分）。
        instance: Option<String>,
    },
    /// 启动行情录制（so-recorder 内核，docs/06 §3）。
    Record {
        /// 录制配置文件（configs/recording.toml）。
        #[arg(short, long, default_value = "configs/recording.toml")]
        config: String,
        /// 只录制指定交易所（配置中的 venue ID）；不填则录制全部。
        #[arg(long)]
        venue: Option<String>,
    },
    /// 查看实例运行日志。
    Logs(LogsArgs),
    /// Kill Switch：撤全部挂单并让策略进入 Halt。
    Kill(KillArgs),
}

/// `data` 子命令。
#[derive(Debug, Subcommand)]
pub enum DataCommand {
    /// 下载交易所官方历史数据（docs/06 A10：断点续传 + 来源记录）。
    Download {
        /// 交易所与市场：gate-usdt（当前支持）。
        venue: String,
        /// 品种，如 BTC_USDT。
        symbol: String,
        /// 开始日期 YYYY-MM-DD。
        #[arg(long)]
        from: String,
        /// 结束日期 YYYY-MM-DD（含）；不填 = 到今天。
        #[arg(long)]
        to: Option<String>,
        /// 频道：trades | candles | funding。
        #[arg(long)]
        channel: String,
    },
    /// Raw Journal → Parquet。
    Normalize {
        /// 日期 YYYY-MM-DD。
        #[arg(long)]
        date: String,
    },
    /// 清洗与质量检查。
    Check {
        /// 日期 YYYY-MM-DD。
        #[arg(long)]
        date: String,
    },
    /// 数据集列表与哈希。
    Ls,
}

/// `exp` 子命令。
#[derive(Debug, Subcommand)]
pub enum ExpCommand {
    /// 列出实验。
    List {
        /// 按策略过滤。
        #[arg(long)]
        strategy: Option<String>,
    },
    /// 查看实验详情。
    Show {
        /// 实验 run_id。
        run_id: String,
    },
    /// 比较两个实验。
    Compare {
        /// 实验 A。
        a: String,
        /// 实验 B。
        b: String,
    },
}

/// `trader` 子命令。控制命令必须指定实例（docs/04 S11）。
#[derive(Debug, Subcommand)]
pub enum TraderCommand {
    /// 实例状态。
    Status {
        /// 实例名；不填则列出全部实例。
        instance: Option<String>,
    },
    /// 启动策略。
    Start(InstanceArgs),
    /// 停止策略（平仓后退出）。
    Stop(InstanceArgs),
    /// 只减仓。
    ReduceOnly(InstanceArgs),
    /// 撤销全部挂单。
    CancelAll(InstanceArgs),
}

/// 必须指定的目标实例。
#[derive(Debug, Args)]
pub struct InstanceArgs {
    /// 实例名，如 live-okx-funding。
    pub instance: String,
    /// 对 live 实例的写操作需二次确认。
    #[arg(long)]
    pub confirm: bool,
}

/// `logs` 参数（docs/03 §18.6）。
#[derive(Debug, Args)]
pub struct LogsArgs {
    /// 实例名。
    pub instance: String,
    /// 最低级别。
    #[arg(long, default_value = "info")]
    pub level: String,
    /// 时间范围，如 1h、30m。
    #[arg(long, default_value = "1h")]
    pub since: String,
    /// 按文本过滤。
    #[arg(long)]
    pub grep: Option<String>,
    /// 持续跟踪新日志。
    #[arg(long)]
    pub follow: bool,
}

/// `kill` 参数：必须指定一个实例，或显式使用 `--all-live`（docs/04 S11）。
#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("target").required(true).args(["instance", "all_live"])))]
pub struct KillArgs {
    /// 目标实例。
    pub instance: Option<String>,
    /// 作用于全部实盘实例。
    #[arg(long)]
    pub all_live: bool,
    /// 同时平掉全部仓位。
    #[arg(long)]
    pub flatten: bool,
    /// 二次确认。
    #[arg(long)]
    pub confirm: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn kill_requires_explicit_target() {
        assert!(Cli::try_parse_from(["so-cli", "kill", "--confirm"]).is_err());
        assert!(Cli::try_parse_from(["so-cli", "kill", "live-okx-funding", "--confirm"]).is_ok());
        assert!(Cli::try_parse_from(["so-cli", "kill", "--all-live", "--confirm"]).is_ok());
        assert!(Cli::try_parse_from(["so-cli", "kill", "x", "--all-live"]).is_err());
    }

    #[test]
    fn trader_control_requires_instance() {
        assert!(Cli::try_parse_from(["so-cli", "trader", "cancel-all"]).is_err());
        assert!(Cli::try_parse_from(["so-cli", "trader", "cancel-all", "paper-lab"]).is_ok());
    }

    #[test]
    fn record_defaults_to_recording_toml() {
        let cli = Cli::try_parse_from(["so-cli", "record"]).unwrap();
        match cli.command {
            Command::Record { config, venue } => {
                assert_eq!(config, "configs/recording.toml");
                assert!(venue.is_none());
            }
            _ => panic!("应为 Record 命令"),
        }
        assert!(
            Cli::try_parse_from(["so-cli", "record", "-c", "other.toml", "--venue", "gate"])
                .is_ok()
        );
    }
}
