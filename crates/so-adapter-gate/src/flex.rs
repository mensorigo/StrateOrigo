//! 宽松标量反序列化：同一字段可能是字符串或数字（docs/venues/gate.md §2.4）。
//!
//! Gate 的 size 类字段在请求带 `X-Gate-Size-Decimal: 1` 时是十进制字符串，
//! 不带时是整型张数（且小数被服务端截断）。适配器固定带该头（拿到字符串），
//! 但也要能读历史 Journal / 兜底路径里的数字形态，统一归一为字符串。
//!
//! 归一化规则：字符串原样保留；整数原样转字符串（无小数点）；
//! 浮点用 `Display`（最短表示，如 `1.5`）。

use serde::Deserialize;
use serde::de::Error as _;

/// 内部枚举：untagged 匹配字符串或 JSON 数字。
#[derive(Deserialize)]
#[serde(untagged)]
enum StrOrNum {
    /// 字符串形态（带 Size-Decimal 头）。
    Str(String),
    /// 数字形态（不带头，整型张数）。
    Num(serde_json::Number),
}

/// `deserialize_with` 用的归一化函数：任意标量 → 字符串。
pub(crate) fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match StrOrNum::deserialize(deserializer) {
        Ok(StrOrNum::Str(s)) => Ok(s),
        Ok(StrOrNum::Num(n)) => Ok(n.to_string()),
        Err(e) => Err(D::Error::custom(format!("既非字符串也非数字：{e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Holder {
        #[serde(deserialize_with = "string_or_number")]
        v: String,
    }

    #[test]
    fn string_form_preserved() {
        let h: Holder = serde_json::from_str(r#"{"v":"108.5"}"#).unwrap();
        assert_eq!(h.v, "108.5");
    }

    #[test]
    fn integer_form_normalized() {
        let h: Holder = serde_json::from_str(r#"{"v":4152}"#).unwrap();
        assert_eq!(h.v, "4152");
    }

    #[test]
    fn float_form_normalized() {
        let h: Holder = serde_json::from_str(r#"{"v":1.5}"#).unwrap();
        assert_eq!(h.v, "1.5");
    }

    #[test]
    fn other_types_rejected() {
        assert!(serde_json::from_str::<Holder>(r#"{"v":null}"#).is_err());
        assert!(serde_json::from_str::<Holder>(r#"{"v":[1]}"#).is_err());
    }
}
