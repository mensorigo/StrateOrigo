//! 品种符号：各交易所命名到内部统一符号的转换（docs/06 §4.2）。
//!
//! 内部符号统一为连字符大写（OKX 风格，如 `BTC-USDT-SWAP`）；
//! Gate 的下划线命名（`BTC_USDT`）在此转换。转换不可逆地丢失原拼写，
//! 需要原拼写时用 `venue` 原始值。

use core::fmt;
use core::str::FromStr;

/// 内部统一品种符号（如 `BTC-USDT-SWAP`、`BTC-USDT`）。
///
/// 固定长度内联存储，不分配堆内存（docs/04 H1）：
/// 上限 24 字符覆盖 `XXXXXX-USDT-SWAP` 类命名。
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    bytes: [u8; Self::MAX_LEN],
    len: u8,
}

/// 符号错误。
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SymbolError {
    /// 超出最大长度。
    #[error("品种符号超长（最多 {0} 字节）")]
    TooLong(usize),
    /// 含非法字符（只允许 A-Z、0-9、`-`、`_`）。
    #[error("品种符号含非法字符")]
    InvalidChar,
    /// 空符号。
    #[error("品种符号为空")]
    Empty,
}

impl Symbol {
    /// 内联存储上限。
    pub const MAX_LEN: usize = 24;

    /// 符号文本。
    pub fn as_str(&self) -> &str {
        // 构造已验证为 ASCII，from_utf8 不会失败；失败时返回空串而不是 panic（H6）。
        let end = usize::from(self.len);
        self.bytes
            .get(..end)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or_default()
    }

    /// 从交易所原始符号构造并规范化：大写、下划线转连字符。
    pub fn normalize(venue_text: &str) -> Result<Self, SymbolError> {
        let mut s = Self {
            bytes: [0; Self::MAX_LEN],
            len: 0,
        };
        for &b in venue_text.as_bytes() {
            let c = match b {
                b'a'..=b'z' => b.wrapping_sub(32), // 大写化（H6：非 panic 算术）
                b'A'..=b'Z' | b'0'..=b'9' => b,
                b'_' => b'-', // Gate 下划线 → 统一连字符
                b'-' => b,
                _ => return Err(SymbolError::InvalidChar),
            };
            let idx = usize::from(s.len);
            if idx >= Self::MAX_LEN {
                return Err(SymbolError::TooLong(venue_text.len()));
            }
            // 惩罚路径 lint（H1/H6）：显式边界检查代替索引与自增。
            if let Some(slot) = s.bytes.get_mut(idx) {
                *slot = c;
            }
            s.len = s.len.checked_add(1).unwrap_or(s.len);
        }
        if s.len == 0 {
            return Err(SymbolError::Empty);
        }
        Ok(s)
    }
}

impl FromStr for Symbol {
    type Err = SymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::normalize(s)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.as_str())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl SymbolError {
    /// 稳定英文错误码（docs/04 E2）。
    pub const fn code(self) -> &'static str {
        match self {
            Self::TooLong(_) => "SymbolTooLong",
            Self::InvalidChar => "SymbolInvalidChar",
            Self::Empty => "SymbolEmpty",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_gate_and_okx() {
        assert_eq!(
            Symbol::normalize("BTC_USDT").map(|s| s.as_str().to_owned()),
            Ok("BTC-USDT".to_owned())
        );
        assert_eq!(
            Symbol::normalize("btc-usdt-swap").map(|s| s.as_str().to_owned()),
            Ok("BTC-USDT-SWAP".to_owned())
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(Symbol::normalize(""), Err(SymbolError::Empty));
        assert_eq!(Symbol::normalize("BTC USDT"), Err(SymbolError::InvalidChar));
        assert_eq!(
            Symbol::normalize(&"A".repeat(25)),
            Err(SymbolError::TooLong(25))
        );
    }

    #[test]
    fn round_trip_and_compare() {
        let a = Symbol::normalize("BTC_USDT").unwrap();
        let b = "BTC-USDT".parse::<Symbol>().unwrap();
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "BTC-USDT");
        assert!(a < Symbol::normalize("ETH_USDT").unwrap());
    }
}
