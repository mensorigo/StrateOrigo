//! `so-cli data check`：读当日 Parquet 产出质量评级与每日报告（docs/06 §5.3）。
//!
//! 输入是 normalize 产物；扫描算法在 [`so_data::quality_scan`]，本模块
//! 只做目录编排、日志与报告落盘（编排层不做算法，04 I9）。
//! Telegram 摘要延后到最终整体验证阶段（2026-11-13 拍板）。

use std::path::Path;
use std::process::ExitCode;

use crate::exit::{CliCode, EXIT_FAILED, EXIT_OK};
use so_data::clean::report::{DailyReport, Grade};
use so_data::pipeline::NORMALIZE_VERSION;
use so_data::quality_scan::scan_day;

/// Parquet 输出根目录（与 data_normalize 一致）。
const PARQUET_ROOT: &str = "data/parquet";
/// 质量报告目录（docs/06 §6）。
const QUALITY_DIR: &str = "data/quality";

/// 执行 `data check --date YYYY-MM-DD`。
pub fn run(date: &str) -> ExitCode {
    let n_root = Path::new(PARQUET_ROOT).join(NORMALIZE_VERSION);
    let scan = match scan_day(&n_root, date) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(code = e.code(), date, "质量扫描失败");
            return ExitCode::from(EXIT_FAILED);
        }
    };
    if scan.cells.is_empty() {
        tracing::error!(
            code = CliCode::NoData.as_str(),
            date,
            "该日期无数据，先运行 data normalize"
        );
        return ExitCode::from(EXIT_FAILED);
    }
    let mut grades: Vec<(String, Grade)> = scan
        .cells
        .iter()
        .map(|((venue, symbol), q)| {
            let g = so_data::clean::report::grade(q);
            tracing::info!(
                venue,
                symbol,
                grade = g.as_str(),
                actual_rows = q.actual_rows,
                gap_count = q.gap_count,
                "质量评级"
            );
            (format!("{venue}/{symbol}"), g)
        })
        .collect();
    grades.sort_by(|a, b| a.0.cmp(&b.0));

    let stats = so_data::clean::CleanStats::default();
    let report = DailyReport {
        venue: "all",
        date,
        grades,
        stats: &stats,
        intervals: scan.intervals,
    };
    let dir = Path::new(QUALITY_DIR);
    if let Err(e) = std::fs::create_dir_all(dir)
        .and_then(|_| std::fs::write(dir.join(format!("{date}.md")), report.render()))
    {
        tracing::error!(code = CliCode::Io.as_str(), "写质量报告失败：{e}");
        return ExitCode::from(EXIT_FAILED);
    }
    tracing::info!(date, path = %dir.join(format!("{date}.md")).display(), "质量报告已写出");
    ExitCode::from(EXIT_OK)
}
