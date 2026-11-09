//! 订单簿错误类型（docs/04 E1/E2）。

/// 订单簿错误。
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BookError {
    /// 序列号不连续，本地订单簿作废，需重新快照（docs/03 §5）。
    #[error("序列缺口：本地 id={local}，收到 U={incoming}")]
    Gap {
        /// 本地已应用到的更新 ID。
        local: u64,
        /// 收到消息的起始 ID。
        incoming: u64,
    },
    /// 快照 ID 与增量序列衔接不上（Gate 六步法第 3 步：baseID+1 < 首个 U）。
    #[error("快照过旧：baseID={base}，首个增量 U={first}")]
    StaleSnapshot {
        /// 快照记录的更新 ID。
        base: u64,
        /// 首条待应用增量的起始 ID。
        first: u64,
    },
    /// 档位容量超限。
    #[error("档位数量超过容量上限 {0}")]
    OverCapacity(usize),
    /// 盘口交叉（买一 ≥ 卖一）——只标记不 panic，由调用方决定是否重建。
    #[error("盘口交叉：bid={bid_raw} >= ask={ask_raw}")]
    Crossed {
        /// 买一价 raw。
        bid_raw: i64,
        /// 卖一价 raw。
        ask_raw: i64,
    },
}

impl BookError {
    /// 稳定英文错误码（docs/04 E2）。
    pub const fn code(self) -> &'static str {
        match self {
            Self::Gap { .. } => "BookGap",
            Self::StaleSnapshot { .. } => "BookStaleSnapshot",
            Self::OverCapacity(_) => "BookOverCapacity",
            Self::Crossed { .. } => "BookCrossed",
        }
    }
}
