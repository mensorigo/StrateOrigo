//! `so-cli` 的退出码与错误日志约定（docs/04 E8、G4）。
//!
//! CLI 是**边界层**：它把库错误收敛成退出码，并保证每条错误日志都带 `code`。
//! 库代码不该关心退出码，所以这套约定放在 `apps/` 而不是任何 `crates/`。

use std::io;

/// 退出码（docs/04 E8）。**数值本身是文档规定的**，不得自定义其他数字。
///
/// - `0` 成功
/// - `1` 运行时失败（I/O、配置、外部接口）
/// - `2` 参数错误（与 clap 的约定一致，手工校验参数时也用它，不可复用）
/// - `3` 命令尚未实现
/// - `4+` 预留给"部分成功 / 需人工介入"
pub const EXIT_OK: u8 = 0;
/// 运行时失败（I/O、配置、外部接口）。
pub const EXIT_FAILED: u8 = 1;
/// 参数错误。**2 是 clap 的约定、不可复用**（[04 E8](04-dev-constraints.md)）。
///
/// 手工校验的**同一类输入错误**（未知子命令取值、日期格式不合法等）也归 2：
/// 用户改了命令行就能通过，与 clap 报错在语义上是同一类。反之，
/// 运行期才发现的失败——配置文件读不到/解析不了、数据不存在——一律归 1，
/// 不要因为"看起来像配置问题"就塞进 2（那会让 2 失去"参数错"的含义）。
pub const EXIT_USAGE: u8 = 2;
/// 命令尚未实现。
pub const EXIT_NOT_IMPLEMENTED: u8 = 3;

/// CLI 边界错误码（docs/04 E2，前缀 `Cli`）。
///
/// 用枚举而非字符串字面量：调用点写 `CliCode::Usage.as_str()`，码只在这里**定义一次**
/// （枚举表模式 [04 §3.4 I1](04-dev-constraints.md)）。散落的 `code = "CliUsage"`
/// 字面量与 docs/04 §3.3 的登记表之间没有任何机检，改一边就会静默失配——
/// 这正是 E2 要消灭的那类问题。
///
/// `Cli*` 码覆盖**标准库与 clap 的失败**：它们没有 `code()` 可转发（不像各域的
/// `thiserror` 枚举），只能由边界层就地映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCode {
    /// 找不到文件（配置、数据目录、Journal）。
    NotFound,
    /// 权限不足。
    PermissionDenied,
    /// 目标已存在。
    AlreadyExists,
    /// 输入格式非法。
    InvalidInput,
    /// 超时。
    TimedOut,
    /// 连接类失败。
    Connection,
    /// 其他 I/O 失败（兜底）。
    Io,
    /// 命令行参数不合法（**手工校验**的参数问题；clap 自己的报错不经这里）。
    Usage,
    /// 该查询条件没有数据。
    NoData,
    /// 异步运行时等进程内基础设施创建失败。
    Runtime,
}

impl CliCode {
    /// 全部码。**仅供测试遍历**（`#[cfg(test)]`）：它存在的意义是"新增变体时
    /// 测试自动覆盖到"，生产代码里没有消费者，所以不进非测试构建。
    ///
    /// 与 `RunMode::ALL`（`so-core`，生产代码也用）不同，这里刻意不为了消除
    /// `dead_code` 而在生产路径上找一处牵强的调用点。
    #[cfg(test)]
    pub const ALL: [Self; 10] = [
        Self::NotFound,
        Self::PermissionDenied,
        Self::AlreadyExists,
        Self::InvalidInput,
        Self::TimedOut,
        Self::Connection,
        Self::Io,
        Self::Usage,
        Self::NoData,
        Self::Runtime,
    ];

    /// 稳定英文错误码。
    ///
    /// 用 `match` 返回 `&'static str` 而不是 `static` 数组按索引取（[04 §3.4 I7](04-dev-constraints.md)）：
    /// 将来新增变体时 `match` 会编译报错，数组按索引取不会。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "CliNotFound",
            Self::PermissionDenied => "CliPermissionDenied",
            Self::AlreadyExists => "CliAlreadyExists",
            Self::InvalidInput => "CliInvalidInput",
            Self::TimedOut => "CliTimedOut",
            Self::Connection => "CliConnection",
            Self::Io => "CliIo",
            Self::Usage => "CliUsage",
            Self::NoData => "CliNoData",
            Self::Runtime => "CliRuntime",
        }
    }
}

/// `std::io::Error` → CLI 边界码。
///
/// 让调用点不必各自 match `ErrorKind`；拿不到具体原因时退到 `CliIo`。
pub fn io_code(e: &io::Error) -> CliCode {
    use io::ErrorKind as K;
    match e.kind() {
        K::NotFound => CliCode::NotFound,
        K::PermissionDenied => CliCode::PermissionDenied,
        K::AlreadyExists => CliCode::AlreadyExists,
        K::InvalidInput | K::InvalidData => CliCode::InvalidInput,
        K::TimedOut => CliCode::TimedOut,
        K::ConnectionRefused | K::ConnectionReset | K::ConnectionAborted => CliCode::Connection,
        _ => CliCode::Io,
    }
}

/// 记录一条带 `code` 的错误日志（docs/04 G4）。
///
/// 只写 `error = %e` 而不给 `code` 会让日志无法被程序聚合与告警，
/// 因此**所有**错误日志都经这里，字符串消息只保留给不方便拆字段的 I/O 原文。
pub fn log_io_error(code: CliCode, message: &'static str, e: &io::Error) {
    tracing::error!(code = code.as_str(), error = %e, "{message}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每类 I/O 失败都有带域前缀的码，且不会退化成裸 `Io`（docs/04 E2）。
    #[test]
    fn io_codes_are_prefixed() {
        let cases = [
            io::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidData,
            io::ErrorKind::TimedOut,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::Other,
        ];
        for kind in cases {
            let code = io_code(&io::Error::new(kind, "x")).as_str();
            assert!(code.starts_with("Cli"), "{kind:?} 的码缺域前缀：{code}");
            assert!(code.len() > 4, "{kind:?} 退化成裸域码：{code}");
        }
    }

    /// 全部码都带 `Cli` 前缀且互不相同（docs/04 E2：码全局唯一）。
    ///
    /// 枚举表模式的意义在此：`ALL` 与 `as_str()` 由同一处维护，漏一个就失败。
    #[test]
    fn all_cli_codes_are_prefixed_and_unique() {
        let mut seen = Vec::new();
        for c in CliCode::ALL {
            let s = c.as_str();
            assert!(s.starts_with("Cli"), "缺域前缀：{s}");
            assert!(s.len() > 4, "退化成裸域码：{s}");
            assert!(!seen.contains(&s), "码重复：{s}");
            seen.push(s);
        }
        assert_eq!(seen.len(), CliCode::ALL.len());
    }

    /// 退出码取 docs/04 E8 规定的值：0/1/2/3 各有固定含义。
    #[test]
    fn exit_codes_match_e8() {
        assert_eq!(EXIT_OK, 0);
        assert_eq!(EXIT_FAILED, 1);
        assert_eq!(EXIT_USAGE, 2, "参数错误必须与 clap 约定一致");
        assert_eq!(EXIT_NOT_IMPLEMENTED, 3);
    }
}
