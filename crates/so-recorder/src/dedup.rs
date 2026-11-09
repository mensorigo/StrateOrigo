//! A/B 双流去重（docs/06 §3.2 A3/C1）。
//!
//! 双连接冗余时同一消息会从两条连接到达；按 `(channel, symbol, 去重键)` 判重。
//! 深度用 `(U, u)` 区间 + 档位内容哈希，成交用 `trade_id`，其余用整帧哈希。
//! 缓存有界（环形缓冲），防止长跑内存增长。

use std::collections::HashSet;

/// 去重键：频道语义决定粒度（C1：相同 `trade_id`、相同 `seq`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DedupKey {
    /// 频道名。
    pub channel: String,
    /// 品种。
    pub symbol: String,
    /// 键值（trade_id / seq 区间 / 帧哈希十六进制）。
    pub key: String,
}

impl DedupKey {
    /// 构造。
    pub fn new(
        channel: impl Into<String>,
        symbol: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            channel: channel.into(),
            symbol: symbol.into(),
            key: key.into(),
        }
    }
}

/// 有界去重器：先到先得（A/B 流随机到达顺序不影响结果）。
#[derive(Debug)]
pub struct Dedup {
    seen: HashSet<String>,
    /// 插入顺序（环形，用于淘汰最旧）。
    order: Vec<String>,
    next: usize,
    capacity: usize,
}

impl Dedup {
    /// 容量按“最大重复间隔 × 消息速率”估算；默认 100 万条。
    pub fn new(capacity: usize) -> Self {
        Self {
            seen: HashSet::new(),
            order: Vec::new(),
            next: 0,
            capacity: capacity.max(1),
        }
    }

    /// 是否首次见到（true = 保留，false = 重复丢弃）。
    pub fn first_seen(&mut self, key: &DedupKey) -> bool {
        let flat = format!("{}|{}|{}", key.channel, key.symbol, key.key);
        if self.seen.contains(&flat) {
            return false;
        }
        if self.order.len() < self.capacity {
            self.order.push(flat.clone());
        } else {
            // 淘汰最旧（环形）。
            let Some(slot) = self.order.get_mut(self.next) else {
                // 不可达：next < capacity ≤ order.len()
                return true;
            };
            let old = std::mem::replace(slot, flat.clone());
            self.seen.remove(&old);
            self.next = (self.next + 1) % self.capacity;
        }
        self.seen.insert(flat);
        true
    }

    /// 当前缓存条数。
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// 从原始帧提取去重键（解析失败时退化为整帧哈希——宁可重复也不丢）。
pub fn dedup_key_from_frame(channel: &str, symbol: &str, raw: &str) -> DedupKey {
    let key = match channel {
        // 成交：trade_id（数字字段；OKX 同名字段）
        c if c.ends_with("trades") => extract_json_num_field(raw, &["id", "tradeId", "trade_id"])
            .or_else(|| extract_json_str_field(raw, &["id", "tradeId", "trade_id"]))
            .unwrap_or_else(|| hash_hex(raw)),
        // 深度增量：seqId 区间
        c if c.contains("order_book") => extract_seq_pair(raw).unwrap_or_else(|| hash_hex(raw)),
        // 其余频道：整帧哈希
        _ => hash_hex(raw),
    };
    DedupKey::new(channel, symbol, key)
}

/// 深度帧的 `(seqId, prevSeqId)` / `(U, u)` 区间键。
fn extract_seq_pair(raw: &str) -> Option<String> {
    let first = extract_json_num_field(raw, &["U", "seqId"])?;
    let last = extract_json_num_field(raw, &["u", "seqId"]).unwrap_or_else(|| first.clone());
    Some(format!("{first}-{last}"))
}

/// 提取 JSON 顶层字符串字段（浅解析，失败返回 None）。
fn extract_json_str_field(raw: &str, names: &[&str]) -> Option<String> {
    for name in names {
        let needle = format!("\"{name}\":");
        if let Some(pos) = raw.find(&needle) {
            let rest = raw.get(pos + needle.len()..)?;
            let rest = rest.trim_start();
            if let Some(s) = rest.strip_prefix('"') {
                let end = s.find('"')?;
                return Some(s.get(..end)?.to_owned());
            }
        }
    }
    None
}

/// 提取 JSON 顶层数字字段（浅解析，失败返回 None）。
fn extract_json_num_field(raw: &str, names: &[&str]) -> Option<String> {
    for name in names {
        let needle = format!("\"{name}\":");
        if let Some(pos) = raw.find(&needle) {
            let rest = raw.get(pos + needle.len()..)?;
            let rest = rest.trim_start();
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            let num = rest.get(..end)?;
            if !num.is_empty() {
                return Some(num.to_owned());
            }
        }
    }
    None
}

/// 帧文本 → 十六进制短哈希（去重键的退化路径）。
fn hash_hex(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_first_seen_then_reject() {
        let mut d = Dedup::new(16);
        let k = DedupKey::new("futures.trades", "BTC_USDT", "123");
        assert!(d.first_seen(&k));
        assert!(!d.first_seen(&k));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn dedup_evicts_oldest() {
        let mut d = Dedup::new(4);
        for i in 0..10 {
            d.first_seen(&DedupKey::new("c", "s", format!("k{i}")));
        }
        // 容量 4：最早 6 个已被淘汰，可再次插入
        assert!(d.first_seen(&DedupKey::new("c", "s", "k0")));
        assert!(!d.first_seen(&DedupKey::new("c", "s", "k9")));
    }

    #[test]
    fn trades_key_uses_trade_id() {
        let k = dedup_key_from_frame(
            "futures.trades",
            "BTC_USDT",
            r#"{"id":27753479,"price":"96.4"}"#,
        );
        assert_eq!(k.key, "27753479");
    }

    #[test]
    fn book_key_uses_seq_pair() {
        let k = dedup_key_from_frame(
            "futures.order_book_update",
            "BTC_USDT",
            r#"{"U":2517661101,"u":2517661113}"#,
        );
        assert_eq!(k.key, "2517661101-2517661113");
    }

    #[test]
    fn unparsable_falls_back_to_hash() {
        let k = dedup_key_from_frame("futures.trades", "S", "not json");
        assert_eq!(k.key.len(), 16); // 十六进制短哈希
        let k2 = dedup_key_from_frame("futures.trades", "S", "not json");
        assert_eq!(k.key, k2.key); // 确定性
    }
}
