//! 定点值类型 [`Price`]、[`Qty`] 及其方法，以及名义金额计算 [`notional`]。

use super::arith::{div_round, mul, narrow, rescale};
use super::display::{DecimalDisplay, display};
use super::parse::parse_decimal;
use super::scale::{Rounding, Scale};
use crate::error::FixedError;

macro_rules! fixed_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(pub i64);

        impl $name {
            /// 零。
            pub const ZERO: Self = Self(0);

            /// 由原始整数构造。
            pub const fn from_raw(raw: i64) -> Self {
                Self(raw)
            }

            /// 原始整数。
            pub const fn raw(self) -> i64 {
                self.0
            }

            /// 是否为零。
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }

            /// 是否为正。
            pub const fn is_positive(self) -> bool {
                self.0 > 0
            }

            /// 是否为负。
            pub const fn is_negative(self) -> bool {
                self.0 < 0
            }

            /// 加法（同精度）。
            pub fn checked_add(self, rhs: Self) -> Result<Self, FixedError> {
                self.0.checked_add(rhs.0).map(Self).ok_or(FixedError::Overflow)
            }

            /// 减法（同精度）。
            pub fn checked_sub(self, rhs: Self) -> Result<Self, FixedError> {
                self.0.checked_sub(rhs.0).map(Self).ok_or(FixedError::Overflow)
            }

            /// 取反。
            pub fn checked_neg(self) -> Result<Self, FixedError> {
                self.0.checked_neg().map(Self).ok_or(FixedError::Overflow)
            }

            /// 乘以整数（如 tick 数）。
            pub fn checked_mul_int(self, n: i64) -> Result<Self, FixedError> {
                self.0.checked_mul(n).map(Self).ok_or(FixedError::Overflow)
            }

            /// 转换精度。
            pub fn rescale(self, from: Scale, to: Scale, mode: Rounding) -> Result<Self, FixedError> {
                rescale(self.0, from, to, mode).map(Self)
            }

            /// 取整到 `step`（tick size / lot size）的整数倍。`step` 必须为正。
            pub fn round_to_step(self, step: Self, mode: Rounding) -> Result<Self, FixedError> {
                if step.0 <= 0 {
                    return Err(FixedError::InvalidStep);
                }
                let n = div_round(i128::from(self.0), i128::from(step.0), mode)?;
                let v = n.checked_mul(i128::from(step.0)).ok_or(FixedError::Overflow)?;
                narrow(v).map(Self)
            }

            /// 精确解析十进制字符串，见 [`parse_decimal`]。
            pub fn parse(s: &str, scale: Scale) -> Result<Self, FixedError> {
                parse_decimal(s, scale).map(Self)
            }

            /// 以给定精度显示。
            pub const fn display(self, scale: Scale) -> DecimalDisplay {
                display(self.0, scale)
            }
        }
    };
}

fixed_newtype!(
    /// 定点价格：`raw × 10^(-scale)`，scale 由品种决定。允许为负（价差、基差）。
    Price
);

fixed_newtype!(
    /// 定点数量：`raw × 10^(-scale)`，scale 由品种决定。带符号时正为多、负为空。
    Qty
);

/// 名义金额 = 价格 × 数量，结果精度为 `out`，以原始整数返回。
pub fn notional(
    price: Price,
    price_scale: Scale,
    qty: Qty,
    qty_scale: Scale,
    out: Scale,
    mode: Rounding,
) -> Result<i64, FixedError> {
    mul(price.0, price_scale, qty.0, qty_scale, out, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc(d: u8) -> Scale {
        Scale::new(d).unwrap()
    }

    #[test]
    fn notional_example() {
        // 价格 65000.5（scale 1）× 数量 0.003（scale 3）= 195.0015
        let p = Price::parse("65000.5", sc(1)).unwrap();
        let q = Qty::parse("0.003", sc(3)).unwrap();
        assert_eq!(
            notional(p, sc(1), q, sc(3), sc(2), Rounding::Floor),
            Ok(19_500)
        );
        assert_eq!(
            notional(p, sc(1), q, sc(3), sc(4), Rounding::Floor),
            Ok(1_950_015)
        );
    }

    #[test]
    fn tick_rounding() {
        let tick = Price(5); // 0.05（scale 2）
        assert_eq!(
            Price(123).round_to_step(tick, Rounding::Floor),
            Ok(Price(120))
        );
        assert_eq!(
            Price(123).round_to_step(tick, Rounding::Ceil),
            Ok(Price(125))
        );
        assert_eq!(
            Price(123).round_to_step(Price(0), Rounding::Floor),
            Err(FixedError::InvalidStep)
        );
    }
}
