//! 定点整数 → 十进制文本显示（冷路径使用：日志、报表、CLI）。

use core::fmt;

use super::arith::pow10;
use super::scale::Scale;

/// 定点数的十进制显示，总是输出 `scale` 位小数。
#[derive(Copy, Clone, Debug)]
pub struct DecimalDisplay {
    raw: i64,
    scale: Scale,
}

/// 以给定精度显示 `raw`。
pub const fn display(raw: i64, scale: Scale) -> DecimalDisplay {
    DecimalDisplay { raw, scale }
}

impl fmt::Display for DecimalDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimals = self.scale.decimals();
        let factor = pow10(u32::from(decimals))
            .ok()
            .and_then(|v| u64::try_from(v).ok())
            .ok_or(fmt::Error)?;
        let abs = self.raw.unsigned_abs();
        let int_part = abs.checked_div(factor).ok_or(fmt::Error)?;
        let frac_part = abs.checked_rem(factor).ok_or(fmt::Error)?;
        if self.raw < 0 {
            f.write_str("-")?;
        }
        if decimals == 0 {
            write!(f, "{int_part}")
        } else {
            let width = usize::from(decimals);
            write!(f, "{int_part}.{frac_part:0width$}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc(d: u8) -> Scale {
        Scale::new(d).unwrap()
    }

    #[test]
    fn display_formats() {
        assert_eq!(display(12_345, sc(2)).to_string(), "123.45");
        assert_eq!(display(-5, sc(2)).to_string(), "-0.05");
        assert_eq!(display(0, sc(3)).to_string(), "0.000");
        assert_eq!(display(7, sc(0)).to_string(), "7");
        assert_eq!(
            display(i64::MIN, sc(18)).to_string(),
            "-9.223372036854775808"
        );
    }
}
