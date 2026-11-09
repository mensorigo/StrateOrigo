//! 定点算术：带取整的整数除法、精度转换、乘法、除法。中间结果用 `i128`。

use core::cmp::Ordering;

use super::scale::{Rounding, Scale};
use crate::error::FixedError;

/// `10^0 ..= 10^36`。两个 `Scale` 相乘后的精度最大为 36，`10^36` 仍在 `i128` 范围内。
// 编译期计算：若溢出会直接编译失败，不会在运行时 panic。
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
const POW10: [i128; 37] = {
    let mut table = [1_i128; 37];
    let mut i = 1;
    while i < 37 {
        table[i] = table[i - 1] * 10;
        i += 1;
    }
    table
};

/// `10^exp`；`exp > 36` 返回溢出。
pub(super) fn pow10(exp: u32) -> Result<i128, FixedError> {
    POW10.get(exp as usize).copied().ok_or(FixedError::Overflow)
}

/// `i128` 收窄为 `i64`。
pub(super) fn narrow(v: i128) -> Result<i64, FixedError> {
    i64::try_from(v).map_err(|_| FixedError::Overflow)
}

/// `n / d`，按 `mode` 取整。
pub fn div_round(n: i128, d: i128, mode: Rounding) -> Result<i128, FixedError> {
    if d == 0 {
        return Err(FixedError::DivisionByZero);
    }
    let q = n.checked_div(d).ok_or(FixedError::Overflow)?;
    let r = n.checked_rem(d).ok_or(FixedError::Overflow)?;
    if r == 0 {
        return Ok(q);
    }
    // r != 0 ⇒ n != 0，真实商的符号由 n、d 的符号决定。
    let negative = (n < 0) != (d < 0);
    let away_from_zero = match mode {
        Rounding::TowardZero => false,
        Rounding::Floor => negative,
        Rounding::Ceil => !negative,
        Rounding::HalfEven => {
            let twice_r = r
                .unsigned_abs()
                .checked_mul(2)
                .ok_or(FixedError::Overflow)?;
            match twice_r.cmp(&d.unsigned_abs()) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => (q & 1) != 0,
            }
        }
    };
    if !away_from_zero {
        return Ok(q);
    }
    if negative {
        q.checked_sub(1)
    } else {
        q.checked_add(1)
    }
    .ok_or(FixedError::Overflow)
}

fn rescale_wide(v: i128, from: u32, to: u32, mode: Rounding) -> Result<i64, FixedError> {
    let factor = pow10(to.abs_diff(from))?;
    let out = if to >= from {
        v.checked_mul(factor).ok_or(FixedError::Overflow)?
    } else {
        div_round(v, factor, mode)?
    };
    narrow(out)
}

/// 把 `raw`（精度 `from`）转换为精度 `to`；降低精度时按 `mode` 取整。
pub fn rescale(raw: i64, from: Scale, to: Scale, mode: Rounding) -> Result<i64, FixedError> {
    rescale_wide(
        i128::from(raw),
        u32::from(from.decimals()),
        u32::from(to.decimals()),
        mode,
    )
}

/// 定点乘法：`a(sa) × b(sb)`，结果精度为 `out`。
pub fn mul(
    a: i64,
    sa: Scale,
    b: i64,
    sb: Scale,
    out: Scale,
    mode: Rounding,
) -> Result<i64, FixedError> {
    let product = i128::from(a)
        .checked_mul(i128::from(b))
        .ok_or(FixedError::Overflow)?;
    let from = u32::from(sa.decimals())
        .checked_add(u32::from(sb.decimals()))
        .ok_or(FixedError::Overflow)?;
    rescale_wide(product, from, u32::from(out.decimals()), mode)
}

/// 定点除法：`a(sa) ÷ b(sb)`，结果精度为 `out`。
pub fn div(
    a: i64,
    sa: Scale,
    b: i64,
    sb: Scale,
    out: Scale,
    mode: Rounding,
) -> Result<i64, FixedError> {
    if b == 0 {
        return Err(FixedError::DivisionByZero);
    }
    // 结果 raw = a × 10^(out + sb - sa) / b
    let up = u32::from(out.decimals())
        .checked_add(u32::from(sb.decimals()))
        .ok_or(FixedError::Overflow)?;
    let down = u32::from(sa.decimals());
    let factor = pow10(up.abs_diff(down))?;
    let (num, den) = if up >= down {
        let num = i128::from(a)
            .checked_mul(factor)
            .ok_or(FixedError::Overflow)?;
        (num, i128::from(b))
    } else {
        let den = i128::from(b)
            .checked_mul(factor)
            .ok_or(FixedError::Overflow)?;
        (i128::from(a), den)
    };
    narrow(div_round(num, den, mode)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc(d: u8) -> Scale {
        Scale::new(d).unwrap()
    }

    #[test]
    fn rounding_modes() {
        use Rounding::*;
        let cases: [(i128, i128, Rounding, i128); 16] = [
            (5, 10, HalfEven, 0),
            (15, 10, HalfEven, 2),
            (25, 10, HalfEven, 2),
            (-5, 10, HalfEven, 0),
            (-15, 10, HalfEven, -2),
            (16, 10, HalfEven, 2),
            (14, 10, HalfEven, 1),
            (7, 2, Floor, 3),
            (-7, 2, Floor, -4),
            (7, 2, Ceil, 4),
            (-7, 2, Ceil, -3),
            (7, 2, TowardZero, 3),
            (-7, 2, TowardZero, -3),
            (7, -2, Floor, -4),
            (-7, -2, Ceil, 4),
            (6, 3, HalfEven, 2),
        ];
        for (n, d, mode, want) in cases {
            assert_eq!(div_round(n, d, mode), Ok(want), "{n}/{d} {mode:?}");
        }
        assert_eq!(div_round(1, 0, Floor), Err(FixedError::DivisionByZero));
    }

    #[test]
    fn division() {
        // 1 ÷ 3，scale 4 → 0.3333 / 0.3334
        assert_eq!(div(1, sc(0), 3, sc(0), sc(4), Rounding::Floor), Ok(3_333));
        assert_eq!(div(1, sc(0), 3, sc(0), sc(4), Rounding::Ceil), Ok(3_334));
        // 1.50 ÷ 0.5 = 3（scale 0）
        assert_eq!(div(150, sc(2), 5, sc(1), sc(0), Rounding::Floor), Ok(3));
        assert_eq!(
            div(1, sc(0), 0, sc(0), sc(0), Rounding::Floor),
            Err(FixedError::DivisionByZero)
        );
    }

    #[test]
    fn multiplication_and_rescale() {
        // 1.5（scale 1）× 2.25（scale 2）= 3.375 → scale 2 五成双 3.38
        assert_eq!(
            mul(15, sc(1), 225, sc(2), sc(2), Rounding::HalfEven),
            Ok(338)
        );
        assert_eq!(rescale(12_345, sc(3), sc(1), Rounding::Floor), Ok(123));
        assert_eq!(rescale(-12_345, sc(3), sc(1), Rounding::Floor), Ok(-124));
        assert_eq!(rescale(7, sc(0), sc(3), Rounding::Floor), Ok(7_000));
    }
}
