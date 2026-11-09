//! 紧凑整数 ID：品种、策略实例、交易所、账户。热路径中用作数组下标。

macro_rules! compact_id {
    ($(#[$meta:meta])* $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(pub $inner);

        impl $name {
            /// 作为数组下标使用（热路径用下标代替 HashMap，docs/04 H8）。
            pub const fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

compact_id!(
    /// 品种 ID，启动时由注册表分配。
    InstrumentId(u32)
);
compact_id!(
    /// 策略实例 ID。
    StrategyId(u16)
);
compact_id!(
    /// 交易所 ID。
    VenueId(u8)
);
compact_id!(
    /// 账户 ID。订单、成交、持仓、资金流水、风控限额都以 `(VenueId, AccountId)` 区分。
    AccountId(u16)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_index() {
        assert_eq!(InstrumentId(7).index(), 7);
        assert_eq!(AccountId(u16::MAX).index(), 65_535);
    }
}
