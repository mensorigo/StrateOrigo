//! 清洗（docs/06 §5）：只加标记，不删原始数据。
//!
//! 结构：`flags`（CFlags 位图）→ 行级规则纯函数分文件（`seq` C1–C3、
//! `outlier` C5/C6、`book` C4/C13、`clock` C7/C8）→ `report`（区间汇总、
//! 质量评级、每日报告）。
//!
//! 原则（docs/06 §5.1）：
//! - 清洗输出 = 规范化数据 + `flags` 标记列 + `quality_intervals` 区间表；
//! - **真实插针不删除**，只标记（回测须看到尾部风险）；
//! - 规则版本 `clean_version` 计入数据集哈希。
//!
//! 行级规则是纯函数（算法层不做 I/O，04 I9）；区间汇总与落盘在
//! [`report`] 与 [`crate::pipeline`]（编排层不做算法）。

pub mod book;
pub mod clock;
pub mod engine;
pub mod flags;
pub mod outlier;
pub mod report;
pub mod seq;

pub use flags::CFlags;

/// 清洗规则版本（计数据集哈希；规则语义变化时递增，不回改）。
pub const CLEAN_VERSION: &str = "c1";

/// 行级清洗统计（每表每品种汇总，进质量报告）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CleanStats {
    /// C1 标记行数。
    pub c1_duplicate: u64,
    /// C2 标记行数（缺口帧）。
    pub c2_seq_gap: u64,
    /// C3 标记行数（乱序）。
    pub c3_out_of_order: u64,
    /// C4 标记行数（交叉盘口）。
    pub c4_crossed: u64,
    /// C5 标记行数（插针，**未删**）。
    pub c5_outlier: u64,
    /// C6 剔除行数（非法值；这是唯一会被剔除的类别）。
    pub c6_invalid: u64,
    /// C7 标记行数（时钟异常）。
    pub c7_clock: u64,
    /// C8 stale 区间数。
    pub c8_stale_intervals: u64,
    /// C13 标记行数（成交越界）。
    pub c13_trade_out_of_book: u64,
    /// 总行数（各表分计前的原始计数）。
    pub total_rows: u64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn clean_version_format() {
        assert!(super::CLEAN_VERSION.starts_with('c'));
    }
}
