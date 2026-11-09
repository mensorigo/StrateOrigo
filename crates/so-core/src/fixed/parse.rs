//! 十进制字符串 → 定点整数的精确解析（不经过浮点，不分配内存）。

use super::arith::{narrow, pow10};
use super::scale::Scale;
use crate::error::FixedError;

/// 精确解析十进制字符串（如交易所返回的 `"0.0125"`）。
///
/// - 接受可选的 `+` / `-`、整数部分、可选的 `.` 与小数部分；不接受空白和科学计数法。
/// - 小数位多于 `scale` 时，多出的位必须全为 0，否则返回 [`FixedError::ExcessPrecision`]。
pub fn parse_decimal(s: &str, scale: Scale) -> Result<i64, FixedError> {
    let bytes = s.as_bytes();
    let (negative, digits) = match bytes.split_first() {
        Some((b'-', rest)) => (true, rest),
        Some((b'+', rest)) => (false, rest),
        Some(_) => (false, bytes),
        None => return Err(FixedError::Parse),
    };

    let max_frac = scale.decimals();
    let mut acc: i128 = 0;
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut frac_digits: u8 = 0;

    for &c in digits {
        match c {
            b'0'..=b'9' => {
                seen_digit = true;
                let digit = i128::from(c.wrapping_sub(b'0'));
                if seen_dot {
                    if frac_digits == max_frac {
                        if digit != 0 {
                            return Err(FixedError::ExcessPrecision);
                        }
                        continue;
                    }
                    frac_digits = frac_digits.wrapping_add(1); // < scale ≤ 18，不会溢出
                }
                acc = acc
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(FixedError::Overflow)?;
            }
            b'.' if !seen_dot => seen_dot = true,
            _ => return Err(FixedError::Parse),
        }
    }
    if !seen_digit {
        return Err(FixedError::Parse);
    }

    acc = acc
        .checked_mul(pow10(u32::from(max_frac.abs_diff(frac_digits)))?)
        .ok_or(FixedError::Overflow)?;
    if negative {
        acc = acc.checked_neg().ok_or(FixedError::Overflow)?;
    }
    narrow(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc(d: u8) -> Scale {
        Scale::new(d).unwrap()
    }

    #[test]
    fn parse_valid() {
        assert_eq!(parse_decimal("0.1", sc(2)), Ok(10));
        assert_eq!(parse_decimal("-0.05", sc(2)), Ok(-5));
        assert_eq!(parse_decimal("+100", sc(2)), Ok(10_000));
        assert_eq!(parse_decimal("1.2300", sc(2)), Ok(123));
        assert_eq!(parse_decimal("1.", sc(1)), Ok(10));
        assert_eq!(parse_decimal(".5", sc(1)), Ok(5));
        assert_eq!(parse_decimal("42", sc(0)), Ok(42));
        assert_eq!(parse_decimal("-9223372036854775808", sc(0)), Ok(i64::MIN));
    }

    #[test]
    fn parse_invalid() {
        let bad = [
            "", "-", "+", ".", "-.", "1.2.3", "abc", " 1", "1 ", "1e5", "--1", "1,5",
        ];
        for s in bad {
            assert_eq!(
                parse_decimal(s, sc(2)),
                Err(FixedError::Parse),
                "输入 {s:?}"
            );
        }
        assert_eq!(
            parse_decimal("1.234", sc(2)),
            Err(FixedError::ExcessPrecision)
        );
        assert_eq!(
            parse_decimal("9223372036854775808", sc(0)),
            Err(FixedError::Overflow)
        );
        assert_eq!(
            parse_decimal("99999999999999999999999999999999999999999", sc(0)),
            Err(FixedError::Overflow)
        );
    }
}
