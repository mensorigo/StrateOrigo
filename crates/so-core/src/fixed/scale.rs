//! 精度 [`Scale`] 与取整方式 [`Rounding`] 的定义。

use crate::error::FixedError;

/// 支持的最大小数位数（`10^18` 仍在 `i64` 范围内）。
pub const MAX_SCALE: u8 = 18;

/// 小数位数，范围 `0..=18`。
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scale(u8);

impl Scale {
    /// 创建精度；超过 [`MAX_SCALE`] 返回错误。
    pub const fn new(decimals: u8) -> Result<Self, FixedError> {
        if decimals <= MAX_SCALE {
            Ok(Self(decimals))
        } else {
            Err(FixedError::InvalidScale(decimals))
        }
    }

    /// 小数位数。
    pub const fn decimals(self) -> u8 {
        self.0
    }
}

/// 取整方式。
///
/// 常见用法：买价向下取 tick（`Floor`）、卖价向上取 tick（`Ceil`）、下单数量向零取 lot（`TowardZero`）。
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rounding {
    /// 向负无穷。
    Floor,
    /// 向正无穷。
    Ceil,
    /// 向零（截断）。
    TowardZero,
    /// 四舍六入五成双（银行家舍入）。
    HalfEven,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_bounds() {
        assert_eq!(Scale::new(18).map(Scale::decimals), Ok(18));
        assert_eq!(Scale::new(19), Err(FixedError::InvalidScale(19)));
    }
}
