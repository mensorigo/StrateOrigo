//! 质量评级、`quality_intervals` 区间表与每日质量报告（docs/06 §5.3）。
//!
//! 评级规则与条件写死在代码里（docs/06 §5.3 的初始值）；
//! 输出：区间表行结构（Parquet 落盘由 pipeline 编排）+ Markdown 报告文本。

use crate::clean::CleanStats;

/// `quality_intervals` 表一行（docs/06 §5.1；schema 见 pipeline 落盘）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityInterval {
    pub venue: String,
    pub symbol: String,
    pub channel: String,
    /// 区间起点（ts_recv 纳秒，含）。
    pub ts_start: i64,
    /// 区间终点（ts_recv 纳秒，不含；单点事件 = ts_start + 1）。
    pub ts_end: i64,
    /// 触发的规则位图。
    pub flags: u32,
}

/// 单 `(venue, symbol, channel, 日期)` 的质量记录（评级输入）。
#[derive(Debug, Clone, Default)]
pub struct QualityInput {
    /// 该单元预期消息数（用相邻消息中位间隔推算的期望数；不足则覆盖率按 1.0）。
    pub expected_rows: u64,
    /// 实际行数。
    pub actual_rows: u64,
    /// 缺口（C2）区间数与总时长纳秒。
    pub gap_count: u64,
    pub gap_total_ns: i64,
    /// 乱序率（C3 行数 / 总行数）。
    pub out_of_order_rate: f64,
    /// 异常标记行数（C5/C6/C7/C13 合计）。
    pub anomaly_rows: u64,
    /// 重建一致率（C9；无 REST 快照校验样本时为 `None`）。
    pub rebuild_match_rate: Option<f64>,
    /// 延迟分位（`ts_recv - ts_exch` 的 p99，纳秒）。
    pub lag_p99_ns: Option<i64>,
}

/// 质量评级（docs/06 §5.3 的初始条件）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// 可用于 HF 回测与晋级。
    A,
    /// 可用于 MF/LF；HF 回测时自动跳过缺口区间。
    B,
    /// 默认禁止用于回测，需显式 `--allow-grade C`。
    C,
}

impl Grade {
    /// 稳定字符串（报告与 Parquet 落盘用）。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

/// 按初始条件评级（docs/06 §5.3；阈值调整属研究期调参，先按文档值）。
pub fn grade(q: &QualityInput) -> Grade {
    let coverage = if q.expected_rows == 0 {
        1.0
    } else {
        q.actual_rows as f64 / q.expected_rows as f64
    };
    let no_unrecovered_gaps = q.gap_count == 0;
    let rebuild_ok = q.rebuild_match_rate.is_none_or(|r| r >= 0.999);

    if coverage >= 0.9999 && rebuild_ok && no_unrecovered_gaps {
        Grade::A
    } else if coverage >= 0.999 {
        Grade::B
    } else {
        Grade::C
    }
}

/// 每日质量报告（Markdown；`so-cli data check` 与 Telegram 摘要共用此文本）。
#[derive(Debug)]
pub struct DailyReport<'a> {
    pub venue: &'a str,
    pub date: &'a str,
    /// `(品种, 评级)` 汇总。
    pub grades: Vec<(String, Grade)>,
    pub stats: &'a CleanStats,
    /// 区间总数。
    pub intervals: usize,
}

impl DailyReport<'_> {
    /// 生成 Markdown 文本。
    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "# 数据质量报告 · {} · {}\n\n",
            self.venue, self.date
        ));
        s.push_str("| 品种 | 评级 |\n|---|---|\n");
        let mut rows: Vec<_> = self.grades.iter().collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        for (sym, g) in rows {
            s.push_str(&format!("| {sym} | {} |\n", g.as_str()));
        }
        s.push_str(&format!(
            "\n标记统计：C1 重复 {} · C2 缺口 {} · C3 乱序 {} · C4 交叉 {} · \
             C5 插针（保留）{} · C6 剔除 {} · C7 时钟 {} · C8 断流区间 {} · \
             C13 越界 {}\n区间总数：{}\n",
            self.stats.c1_duplicate,
            self.stats.c2_seq_gap,
            self.stats.c3_out_of_order,
            self.stats.c4_crossed,
            self.stats.c5_outlier,
            self.stats.c6_invalid,
            self.stats.c7_clock,
            self.stats.c8_stale_intervals,
            self.stats.c13_trade_out_of_book,
            self.intervals,
        ));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> QualityInput {
        QualityInput {
            expected_rows: 100_000,
            actual_rows: 100_000,
            ..Default::default()
        }
    }

    /// 评级边界（docs/06 §5.3 初始值）：A=99.99%+无缺口+重建达标；
    /// B=99.9%；C=其余。缺口未恢复直接降 A。
    #[test]
    fn grade_boundaries() {
        assert_eq!(grade(&base()), Grade::A);

        // 覆盖率 99.995%：仍 A。
        let q = QualityInput {
            actual_rows: 99_995,
            ..base()
        };
        assert_eq!(grade(&q), Grade::A);

        // 有未恢复缺口：A → B（覆盖率还够 B）。
        let q = QualityInput {
            gap_count: 1,
            ..base()
        };
        assert_eq!(grade(&q), Grade::B);

        // 覆盖率 99.95%：B。
        let q = QualityInput {
            actual_rows: 99_950,
            ..base()
        };
        assert_eq!(grade(&q), Grade::B);

        // 覆盖率 99.5%：C。
        let q = QualityInput {
            actual_rows: 99_500,
            ..base()
        };
        assert_eq!(grade(&q), Grade::C);

        // 重建一致率不足：A → B（ coverage 仍 100%）。
        let q = QualityInput {
            rebuild_match_rate: Some(0.99),
            ..base()
        };
        assert_eq!(grade(&q), Grade::B);

        // 无预期行数（新品种首日）：按 1.0 处理。
        let q = QualityInput {
            expected_rows: 0,
            actual_rows: 0,
            ..base()
        };
        assert_eq!(grade(&q), Grade::A);
    }

    /// 报告文本包含关键节。
    #[test]
    fn report_renders() {
        let stats = CleanStats {
            c1_duplicate: 3,
            c5_outlier: 2,
            ..Default::default()
        };
        let r = DailyReport {
            venue: "gate",
            date: "2026-11-12",
            grades: vec![("BTC_USDT".into(), Grade::A), ("ETH_USDT".into(), Grade::B)],
            stats: &stats,
            intervals: 4,
        };
        let md = r.render();
        assert!(md.contains("# 数据质量报告 · gate · 2026-11-12"));
        assert!(md.contains("| BTC_USDT | A |"));
        assert!(md.contains("| ETH_USDT | B |"));
        assert!(md.contains("C5 插针（保留）2"));
        assert!(md.contains("区间总数：4"));
    }
}
