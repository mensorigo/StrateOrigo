//! `so-cli data check`：读当日 Parquet 产出质量评级与每日报告（docs/06 §5.3）。
//!
//! 输入是 normalize 产物；扫描算法在 [`so_data::quality_scan`]，本模块
//! 只做目录编排、日志与报告落盘（编排层不做算法，04 I9）。
//! `--dataset` 顺带扫描生成数据集清单（docs/06 §7，M3 回测的哈希引用前置）。
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

/// 执行 `data check --date YYYY-MM-DD [--dataset --name X]`。
pub fn run(date: &str, dataset: bool, name: Option<&str>) -> ExitCode {
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

    if dataset {
        let Some(name) = name else {
            tracing::error!(code = CliCode::Usage.as_str(), "--dataset 需要 --name");
            return ExitCode::from(EXIT_FAILED);
        };
        return build_dataset(&n_root, date, name);
    }
    ExitCode::from(EXIT_OK)
}

/// 扫描当日产物生成数据集清单（编排：字段在这里定，算法在 so-data）。
fn build_dataset(n_root: &Path, date: &str, name: &str) -> ExitCode {
    let venues: Vec<String> = {
        let mut v: Vec<String> = scan_venues(n_root);
        v.sort();
        v
    };
    if venues.is_empty() {
        tracing::error!(code = CliCode::NoData.as_str(), date, "该日期无交易所输出");
        return ExitCode::from(EXIT_FAILED);
    }
    let scanned = match so_data::dataset::scan_dataset(n_root, &venues, &[date]) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(code = e.code(), date, "数据集扫描失败");
            return ExitCode::from(EXIT_FAILED);
        }
    };
    // 评级输入里的最低评级作为 min_grade（docs/06 §7）。
    let min_grade = scan_min_grade(n_root, date, &venues);
    let manifest = scanned.into_manifest(
        name.to_owned(),
        min_grade,
        NORMALIZE_VERSION.to_owned(),
        so_data::clean::CLEAN_VERSION.to_owned(),
        format!("{date}T00:00:00Z"),
        format!("{date}T23:59:59Z"),
    );
    match so_data::dataset_out::write_manifest(&manifest) {
        Ok((hash, path)) => {
            tracing::info!(
                dataset_hash = hash,
                files = manifest.files.len(),
                path = %path.display(),
                "数据集清单已写出"
            );
            ExitCode::from(EXIT_OK)
        }
        Err(e) => {
            tracing::error!(code = e.code(), "清单写出失败");
            ExitCode::from(EXIT_FAILED)
        }
    }
}

/// 版本根下的交易所目录（与 quality_scan::list_venues 同口径）。
fn scan_venues(n_root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(n_root).into_iter().flatten().flatten() {
        if e.path().is_dir()
            && let Some(name) = e.file_name().to_str()
            && !name.starts_with("quality_")
        {
            out.push(name.to_owned());
        }
    }
    out
}

/// 当日全部单元的最低评级（评级逻辑复用 so-data，这里只取 min）。
fn scan_min_grade(n_root: &Path, date: &str, venues: &[String]) -> String {
    let mut min = b'C';
    let mut any = false;
    if let Ok(scan) = scan_day(n_root, date) {
        for ((venue, _), q) in &scan.cells {
            if !venues.contains(venue) {
                continue;
            }
            any = true;
            let g = so_data::clean::report::grade(q);
            match g {
                so_data::clean::report::Grade::A => {}
                so_data::clean::report::Grade::B => min = min.min(b'B'),
                so_data::clean::report::Grade::C => min = b'C',
            }
        }
    }
    if !any {
        return "C".to_owned(); // 无评级输入按最保守处理
    }
    (min as char).to_string()
}
