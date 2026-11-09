use thiserror::Error;

/// 定点运算与解析错误。热路径中所有可能失败的运算都返回它，而不是 panic。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum FixedError {
    /// 结果超出 `i64` 范围。
    #[error("定点运算溢出")]
    Overflow,
    /// 除数为零。
    #[error("除数为零")]
    DivisionByZero,
    /// 精度超出 `0..=18`。
    #[error("无效精度 {0}（允许 0..=18）")]
    InvalidScale(u8),
    /// 取整步长（tick / lot）必须为正。
    #[error("取整步长必须为正")]
    InvalidStep,
    /// 不是合法的十进制数字串。
    #[error("无法解析的十进制数")]
    Parse,
    /// 小数位多于精度且多出的位不为零（不做静默截断）。
    #[error("小数位超出精度")]
    ExcessPrecision,
}

impl FixedError {
    /// 稳定的英文错误码，用于跨语言匹配（Python 异常、日志、审计）；
    /// `Display` 的中文消息只面向人，不作为程序判别依据。
    pub const fn code(self) -> &'static str {
        match self {
            Self::Overflow => "Overflow",
            Self::DivisionByZero => "DivisionByZero",
            Self::InvalidScale(_) => "InvalidScale",
            Self::InvalidStep => "InvalidStep",
            Self::Parse => "Parse",
            Self::ExcessPrecision => "ExcessPrecision",
        }
    }
}
