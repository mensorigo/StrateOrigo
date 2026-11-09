//! 现货频道解析（次要：S2 需要时启用，docs/venues/gate.md §4）。
//!
//! 现货与合约消息框架相同，差异只在：
//! - 深度档位是 `[price, amount]` **数组对**（合约是 `{p, s}` 对象）；
//! - 成交方向是 `side: "buy"/"sell"` 字符串（合约是 size 符号）。
//!
//! 本模块只实现档位数组对的适配；其余复用合约解析。

use so_core::{Price, Qty};
use so_model::Side;

use crate::error::GateError;
use crate::types::ObuLevel;

/// 现货 `spot.order_book_update` 的档位数组对 `[price, amount]` → [`ObuLevel`]。
///
/// 现货推送体 `b`/`a` 是 `Vec<[String; 2]>`，与合约的对象格式不同（两套解析，docs/venues/gate.md §7.3）。
pub fn parse_spot_level(
    pair: &[String],
    side: Side,
    scale: so_core::Scale,
) -> Result<ObuLevel, GateError> {
    let price = pair.first().ok_or(GateError::BadShape)?;
    let qty = pair.get(1).ok_or(GateError::BadShape)?;
    Ok(ObuLevel {
        side,
        price: Price::parse(price, scale)?,
        qty: Qty::parse(qty, scale)?,
    })
}

/// 现货 `spot.trades` 的方向字段。
pub fn spot_side(side: &str) -> Option<Side> {
    match side {
        "buy" => Some(Side::Bid), // buy = 吃单方是买方
        "sell" => Some(Side::Ask),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sc() -> so_core::Scale {
        so_core::Scale::new(2).unwrap()
    }

    #[test]
    fn parses_array_pair_level() {
        let lv = parse_spot_level(
            &["54672.1".to_owned(), "58794.5".to_owned()],
            Side::Bid,
            sc(),
        )
        .unwrap();
        assert_eq!(lv.price, Price(5_467_210));
        assert_eq!(lv.qty, Qty(5_879_450));
    }

    #[test]
    fn rejects_short_pair() {
        let lv = parse_spot_level(&["1".to_owned()], Side::Ask, sc());
        assert_eq!(lv, Err(GateError::BadShape));
    }

    #[test]
    fn side_mapping() {
        assert_eq!(spot_side("buy"), Some(Side::Bid));
        assert_eq!(spot_side("sell"), Some(Side::Ask));
        assert_eq!(spot_side("other"), None);
    }
}
