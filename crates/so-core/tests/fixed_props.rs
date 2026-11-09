//! 定点运算性质测试（M0 验收：加减乘除、取整、溢出返回错误）。

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use so_core::fixed::{self, Rounding, Scale};
use so_core::{FixedError, Price, Qty};

const MODES: [Rounding; 4] = [
    Rounding::Floor,
    Rounding::Ceil,
    Rounding::TowardZero,
    Rounding::HalfEven,
];

fn scale() -> impl Strategy<Value = Scale> {
    (0_u8..=18).prop_map(|d| Scale::new(d).unwrap())
}

fn mode() -> impl Strategy<Value = Rounding> {
    prop::sample::select(MODES.to_vec())
}

fn pow10(e: u32) -> i128 {
    10_i128.pow(e)
}

/// 每个性质 10,000 例；失败的最小用例存到 `tests/fixed_props.proptest-regressions`，之后每次都会先重跑。
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 10_000,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/fixed_props.proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    /// 解析(显示(x)) == x：任何 i64、任何精度。
    #[test]
    fn parse_display_round_trip(raw in any::<i64>(), s in scale()) {
        let text = Price(raw).display(s).to_string();
        prop_assert_eq!(Price::parse(&text, s), Ok(Price(raw)));
    }

    /// 加减与 i128 参考结果一致；超出 i64 时返回 Overflow 而不是 panic。
    #[test]
    fn add_sub_match_reference(a in any::<i64>(), b in any::<i64>()) {
        let sum = i128::from(a) + i128::from(b);
        let diff = i128::from(a) - i128::from(b);
        let want_sum = i64::try_from(sum).map(Qty).map_err(|_| FixedError::Overflow);
        let want_diff = i64::try_from(diff).map(Qty).map_err(|_| FixedError::Overflow);
        prop_assert_eq!(Qty(a).checked_add(Qty(b)), want_sum);
        prop_assert_eq!(Qty(a).checked_sub(Qty(b)), want_diff);
    }

    /// 先升精度再降回原精度，值不变（范围选在不会溢出的区间，保证每个样本都真正被检验）。
    #[test]
    fn rescale_up_down_identity(
        raw in -1_000_000_000_i64..1_000_000_000,
        from in 0_u8..=9,
        extra in 0_u8..=9,
        m in mode(),
    ) {
        let (from, to) = (Scale::new(from).unwrap(), Scale::new(from + extra).unwrap());
        let up = fixed::rescale(raw, from, to, m).unwrap();
        prop_assert_eq!(fixed::rescale(up, to, from, m), Ok(raw));
    }

    /// 乘法：Floor 与参考实现一致；各取整方式落在 [Floor, Ceil] 内，且两者相差至多 1 个最小单位。
    #[test]
    fn mul_rounding_bracket(
        a in -1_000_000_000_000_i64..1_000_000_000_000,
        b in -1_000_000_000_000_i64..1_000_000_000_000,
        sa in scale(), sb in scale(), out in scale(),
    ) {
        let product = i128::from(a) * i128::from(b);
        let from = u32::from(sa.decimals() + sb.decimals());
        let o = u32::from(out.decimals());
        let reference_floor = if o >= from {
            product.checked_mul(pow10(o - from))
        } else {
            Some(product.div_euclid(pow10(from - o)))
        };

        let floor = fixed::mul(a, sa, b, sb, out, Rounding::Floor);
        let ceil = fixed::mul(a, sa, b, sb, out, Rounding::Ceil);
        match reference_floor.and_then(|v| i64::try_from(v).ok()) {
            Some(want) => prop_assert_eq!(floor, Ok(want)),
            None => prop_assert!(floor.is_err()),
        }
        if let (Ok(f), Ok(c)) = (floor, ceil) {
            prop_assert!(c - f == 0 || c - f == 1);
            for m in [Rounding::TowardZero, Rounding::HalfEven] {
                let v = fixed::mul(a, sa, b, sb, out, m).unwrap();
                prop_assert!(f <= v && v <= c);
            }
            let tz = fixed::mul(a, sa, b, sb, out, Rounding::TowardZero).unwrap();
            prop_assert_eq!(tz, if product >= 0 { f } else { c });
        }
    }

    /// 除法是精确乘法的逆运算。
    #[test]
    fn div_inverts_exact_mul(
        a in -1_000_000_000_i64..1_000_000_000,
        b in -1_000_000_000_i64..1_000_000_000,
        sa in 0_u8..=9, sb in 0_u8..=9,
        m in mode(),
    ) {
        prop_assume!(b != 0);
        let (sa, sb) = (Scale::new(sa).unwrap(), Scale::new(sb).unwrap());
        let sp = Scale::new(sa.decimals() + sb.decimals()).unwrap();
        let product = fixed::mul(a, sa, b, sb, sp, Rounding::Floor).unwrap();
        prop_assert_eq!(fixed::div(product, sp, b, sb, sa, m), Ok(a));
    }

    /// 除法的取整区间：Floor ≤ 各方式 ≤ Ceil，相差至多 1。
    #[test]
    fn div_rounding_bracket(a in any::<i64>(), b in any::<i64>(), sa in scale(), sb in scale(), out in scale()) {
        let floor = fixed::div(a, sa, b, sb, out, Rounding::Floor);
        let ceil = fixed::div(a, sa, b, sb, out, Rounding::Ceil);
        if b == 0 {
            prop_assert_eq!(floor, Err(FixedError::DivisionByZero));
        } else if let (Ok(f), Ok(c)) = (floor, ceil) {
            prop_assert!(c - f == 0 || c - f == 1);
            for m in [Rounding::TowardZero, Rounding::HalfEven] {
                if let Ok(v) = fixed::div(a, sa, b, sb, out, m) {
                    prop_assert!(f <= v && v <= c);
                }
            }
        }
    }

    /// 取整到 tick / lot：结果是步长的整数倍，Floor ≤ 原值 ≤ Ceil，两者相差 0 或 1 个步长。
    #[test]
    fn round_to_step_properties(raw in -1_000_000_000_000_i64..1_000_000_000_000, step in 1_i64..1_000_000) {
        let x = Price(raw);
        let f = x.round_to_step(Price(step), Rounding::Floor).unwrap();
        let c = x.round_to_step(Price(step), Rounding::Ceil).unwrap();
        prop_assert_eq!(f.raw() % step, 0);
        prop_assert_eq!(c.raw() % step, 0);
        prop_assert!(f <= x && x <= c);
        prop_assert!(c.raw() - f.raw() == 0 || c.raw() - f.raw() == step);
        for m in [Rounding::TowardZero, Rounding::HalfEven] {
            let v = x.round_to_step(Price(step), m).unwrap();
            prop_assert!(v == f || v == c);
        }
    }

    /// 任意输入都不 panic（只允许返回 Ok 或 Err）。
    #[test]
    fn never_panics(
        a in any::<i64>(), b in any::<i64>(),
        sa in scale(), sb in scale(), out in scale(), m in mode(),
        text in ".{0,48}",
    ) {
        let _ = fixed::mul(a, sa, b, sb, out, m);
        let _ = fixed::div(a, sa, b, sb, out, m);
        let _ = fixed::rescale(a, sa, out, m);
        let _ = Price(a).round_to_step(Price(b), m);
        let _ = Qty(a).checked_neg();
        let _ = Qty(a).checked_mul_int(b);
        let _ = Price::parse(&text, sa);
    }
}

#[test]
fn extreme_values_return_errors() {
    let s0 = Scale::new(0).unwrap();
    assert_eq!(
        fixed::mul(i64::MAX, s0, i64::MAX, s0, s0, Rounding::Floor),
        Err(FixedError::Overflow)
    );
    assert_eq!(Qty(i64::MIN).checked_neg(), Err(FixedError::Overflow));
    assert_eq!(
        Price(i64::MAX).checked_add(Price(1)),
        Err(FixedError::Overflow)
    );
    assert_eq!(
        fixed::rescale(i64::MAX, s0, Scale::new(18).unwrap(), Rounding::Floor),
        Err(FixedError::Overflow)
    );
}
