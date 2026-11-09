//! 运行模式 [`RunMode`] 与账户类型 [`AccountKind`]，以及 ADR-0001 的账户/策略数量规则。

use core::fmt;
use core::str::FromStr;

/// 账户类型。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AccountKind {
    /// 真实账户。
    Live,
    /// 交易所模拟环境账户（OKX 模拟交易 / Gate 测试网）。
    Demo,
    /// 本地虚拟账户（回测 / 本地模拟盘）。
    Sim,
}

/// 运行模式（docs/01 §7.1）。
///
/// 这里的方法只用于**启动时的配置校验**（ADR-0001、docs/04 S8/S9）；
/// 内核代码中不得出现按模式分支。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RunMode {
    /// 历史数据回放。
    Backtest,
    /// 本地模拟盘：实时公开行情 + 本地撮合。
    Paper,
    /// 交易所模拟环境。
    Demo,
    /// 实盘。
    Live,
}

impl RunMode {
    /// 全部模式。
    pub const ALL: [Self; 4] = [Self::Backtest, Self::Paper, Self::Demo, Self::Live];

    /// 配置中的写法。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Backtest => "backtest",
            Self::Paper => "paper",
            Self::Demo => "demo",
            Self::Live => "live",
        }
    }

    /// 告警与 CLI 中的前缀（docs/04 S10）。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Backtest => "[BACKTEST]",
            Self::Paper => "[PAPER]",
            Self::Demo => "[DEMO]",
            Self::Live => "[LIVE]",
        }
    }

    /// 该模式使用的账户类型。
    pub const fn account_kind(self) -> AccountKind {
        match self {
            Self::Backtest | Self::Paper => AccountKind::Sim,
            Self::Demo => AccountKind::Demo,
            Self::Live => AccountKind::Live,
        }
    }

    /// 是否允许加载 API 密钥。`backtest` / `paper` 一律不允许（docs/04 S8）。
    pub const fn may_load_keys(self) -> bool {
        matches!(self, Self::Demo | Self::Live)
    }

    /// 每个交易所允许的账户数；`None` 表示不限（虚拟账户）（ADR-0001）。
    pub const fn max_accounts_per_venue(self) -> Option<u16> {
        match self {
            Self::Backtest | Self::Paper => None,
            Self::Demo | Self::Live => Some(1),
        }
    }

    /// 每个账户允许的策略数：任何模式都是 1（ADR-0001）。
    pub const fn max_strategies_per_account(self) -> u16 {
        1
    }
}

impl fmt::Display for RunMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 无法识别的运行模式字符串。
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("未知运行模式（允许 backtest / paper / demo / live）")]
pub struct ParseRunModeError;

impl FromStr for RunMode {
    type Err = ParseRunModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backtest" => Ok(Self::Backtest),
            "paper" => Ok(Self::Paper),
            "demo" => Ok(Self::Demo),
            "live" => Ok(Self::Live),
            _ => Err(ParseRunModeError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_round_trip() {
        for m in RunMode::ALL {
            assert_eq!(m.as_str().parse::<RunMode>(), Ok(m));
        }
        assert_eq!("LIVE".parse::<RunMode>(), Err(ParseRunModeError));
    }

    #[test]
    fn adr_0001_rules() {
        assert!(!RunMode::Backtest.may_load_keys());
        assert!(!RunMode::Paper.may_load_keys());
        assert!(RunMode::Demo.may_load_keys());
        assert!(RunMode::Live.may_load_keys());
        assert_eq!(RunMode::Live.max_accounts_per_venue(), Some(1));
        assert_eq!(RunMode::Paper.max_accounts_per_venue(), None);
        for m in RunMode::ALL {
            assert_eq!(m.max_strategies_per_account(), 1);
        }
        assert_eq!(RunMode::Paper.account_kind(), AccountKind::Sim);
    }
}
