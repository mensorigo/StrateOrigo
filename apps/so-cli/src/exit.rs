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
/// 参数错误（含手工校验的参数问题，与 clap 一致）。
pub const EXIT_USAGE: u8 = 2;
/// 命令尚未实现。
pub const EXIT_NOT_IMPLEMENTED: u8 = 3;

/// I/O 失败的稳定错误码。
///
/// `std::io::Error` 是标准库类型，无法给它加 `code()`，因此在边界处统一成
/// 带域前缀的 CLI 码（docs/04 E2），而不是让调用点各自拼字符串。
pub fn io_code(e: &io::Error) -> &'static str {
    use io::ErrorKind as K;
    match e.kind() {
        K::NotFound => "CliNotFound",
        K::PermissionDenied => "CliPermissionDenied",
        K::AlreadyExists => "CliAlreadyExists",
        K::InvalidInput | K::InvalidData => "CliInvalidInput",
        K::TimedOut => "CliTimedOut",
        K::ConnectionRefused | K::ConnectionReset | K::ConnectionAborted => "CliConnection",
        _ => "CliIo",
    }
}

/// 记录一条带 `code` 的错误日志（docs/04 G4）。
///
/// 只写 `error = %e` 而不给 `code` 会让日志无法被程序聚合与告警，
/// 因此**所有**错误日志都经这里，字符串消息只保留给不方便拆字段的 I/O 原文。
pub fn log_io_error(code: &'static str, message: &'static str, e: &io::Error) {
    tracing::error!(code, error = %e, "{message}");
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
            let code = io_code(&io::Error::new(kind, "x"));
            assert!(code.starts_with("Cli"), "{kind:?} 的码缺域前缀：{code}");
            assert!(code.len() > 4, "{kind:?} 退化成裸域码：{code}");
        }
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
