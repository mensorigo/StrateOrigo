//! 清洗标记位图（docs/06 §5.1–§5.2）。
//!
//! [`CFlags`] 是 `u32` 位图，每条清洗规则一位。规则位**只增不改**：
//! 新规则追加到表尾并递增位号，已发布数据集的位语义永不复用
//! （改位语义 ⇒ 旧 Parquet 的 flags 列含义漂移，违反 E2 的"码只增不改名"精神）。
//!
//! 行级标记直接进 `flags` 列（normalize 加列方案，2026-11-12 拍板）；
//! 区间级异常（缺口、断流、维护等持续一段时间的）另落 `quality_intervals` 表。

/// 清洗标记位图。
///
/// 位与 [docs/06 §5.2](https://) 的 C1–C13 一一对应；位 0–12 为 C1–C13，
/// 位 13 起为补充规则。`0` = 无任何标记（干净行）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct CFlags(pub u32);

impl CFlags {
    /// C1 重复消息（相同 trade_id / 相同 seq；A/B 合并残留）。
    pub const DUPLICATE: Self = Self::bit(0);
    /// C2 序列号缺口（seq_prev != 上一帧 seq_last；区间内订单簿无效）。
    pub const SEQ_GAP: Self = Self::bit(1);
    /// C3 时间戳乱序（ts_exch 非单调；保留接收顺序，只标记）。
    pub const TS_OUT_OF_ORDER: Self = Self::bit(2);
    /// C4 交叉盘口（best_bid >= best_ask）。
    pub const CROSSED_BOOK: Self = Self::bit(3);
    /// C5 价格异常跳变（疑似插针——**只标记不删**，可能真实行情）。
    pub const PRICE_OUTLIER: Self = Self::bit(4);
    /// C6 非法值（价格 <= 0 / 数量 < 0 / 精度不符；**剔除**并计数）。
    pub const INVALID_VALUE: Self = Self::bit(5);
    /// C7 时钟异常（ts_recv - ts_exch 为负或异常大）。
    pub const CLOCK_ANOMALY: Self = Self::bit(6);
    /// C8 断流（频道无消息超过静默阈值；stale 区间起点行标记）。
    pub const STALE: Self = Self::bit(7);
    /// C9 订单簿重建偏差（REST 快照校验前 N 档不一致）。
    pub const BOOK_REBUILD_MISMATCH: Self = Self::bit(8);
    /// C10 交易所维护 / 暂停交易。
    pub const MAINTENANCE: Self = Self::bit(9);
    /// C11 品种元数据变更（精度在变更时刻切换）。
    pub const META_CHANGE: Self = Self::bit(10);
    /// C12 股票公司行为（加密永续暂不触发；位预留）。
    pub const CORPORATE_ACTION: Self = Self::bit(11);
    /// C13 成交价在当时盘口之外。
    pub const TRADE_OUT_OF_BOOK: Self = Self::bit(12);

    /// 由原始 u32 构造（Parquet 读回用）。
    pub const fn from_bits(v: u32) -> Self {
        Self(v)
    }

    /// 原始位图值（Parquet 写出用）。
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// 置入另一组标记。
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// 是否含任一指定标记。
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// 是否干净（无任何标记）。
    pub const fn is_clean(self) -> bool {
        self.0 == 0
    }

    const fn bit(n: u32) -> Self {
        Self(1 << n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 13 条规则位互不重叠，且全部落在 u32 内（C12 预留位也算）。
    #[test]
    fn rule_bits_are_distinct() {
        let all = [
            CFlags::DUPLICATE,
            CFlags::SEQ_GAP,
            CFlags::TS_OUT_OF_ORDER,
            CFlags::CROSSED_BOOK,
            CFlags::PRICE_OUTLIER,
            CFlags::INVALID_VALUE,
            CFlags::CLOCK_ANOMALY,
            CFlags::STALE,
            CFlags::BOOK_REBUILD_MISMATCH,
            CFlags::MAINTENANCE,
            CFlags::META_CHANGE,
            CFlags::CORPORATE_ACTION,
            CFlags::TRADE_OUT_OF_BOOK,
        ];
        let mut seen = 0u32;
        for f in all {
            assert_eq!(f.0 & seen, 0, "规则位重复：{:032b}", f.0);
            seen |= f.0;
        }
        assert_eq!(seen.count_ones(), 13, "应有 13 个规则位（含 C12 预留）");
    }

    /// union / intersects / is_clean 的基本语义。
    #[test]
    fn bitmap_semantics() {
        let a = CFlags::DUPLICATE.union(CFlags::SEQ_GAP);
        assert!(a.intersects(CFlags::DUPLICATE));
        assert!(a.intersects(CFlags::SEQ_GAP));
        assert!(!a.intersects(CFlags::CROSSED_BOOK));
        assert!(!a.is_clean());
        assert!(CFlags::default().is_clean());
        assert_eq!(a.bits(), 0b011);
        assert_eq!(CFlags::from_bits(0b011), a);
    }
}
