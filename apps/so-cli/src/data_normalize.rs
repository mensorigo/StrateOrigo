//! `so-cli data normalize`：按日把 Raw Journal 转成 Parquet（docs/05 M2）。
//!
//! 输入目录取 `configs/recording.toml` 的 `data_dir`；输出根目录固定
//! `data/parquet`（docs/06 §6）。交易所列表按日期目录下实际存在的 Journal
//! 推断，避免把"配置里有但当天没录"的交易所算成缺数据。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::exit::{CliCode, EXIT_FAILED, EXIT_OK, io_code, log_io_error};
use so_data::pipeline::{NormalizeRequest, normalize_day};
use so_recorder::RecordingConfig;

/// 录制配置默认路径（与 `record` 命令一致）。
const RECORDING_CONFIG: &str = "configs/recording.toml";
/// Parquet 输出根目录。
const PARQUET_ROOT: &str = "data/parquet";

/// 执行 `data normalize --date YYYY-MM-DD`。
pub fn run(date: &str) -> ExitCode {
    let raw_dir = match load_data_dir() {
        Some(p) => p,
        None => return ExitCode::from(EXIT_FAILED),
    };
    let venues = match venues_for_date(&raw_dir, date) {
        Ok(v) => v,
        Err(e) => {
            log_io_error(io_code(&e), "扫描 Raw Journal 目录失败", &e);
            return ExitCode::from(EXIT_FAILED);
        }
    };
    if venues.is_empty() {
        tracing::error!(code = CliCode::NoData.as_str(), date, raw_dir = %raw_dir.display(), "该日期没有 Raw Journal");
        return ExitCode::from(EXIT_FAILED);
    }
    let out_root = Path::new(PARQUET_ROOT);
    let mut total = 0_u64;
    for venue in &venues {
        let req = NormalizeRequest {
            raw_dir: &raw_dir,
            out_root,
            venue,
            date,
        };
        match normalize_day(&req) {
            Ok((stats, out_dir)) => {
                total = total.saturating_add(stats.event_rows);
                tracing::info!(
                    venue,
                    date,
                    files = stats.files,
                    journal_rows = stats.journal_rows,
                    dropped_lines = stats.dropped_lines,
                    duplicate_rows = stats.duplicate_rows,
                    parse_failed = stats.parse_failed,
                    no_scale = stats.no_scale,
                    contracts_failed = stats.contracts_failed,
                    instrument_rows = stats.instrument_rows,
                    event_rows = stats.event_rows,
                    out_dir = %out_dir.display(),
                    "normalize 完成"
                );
                if stats.dropped_lines > 0 || stats.parse_failed > 0 || stats.contracts_failed > 0 {
                    tracing::warn!(venue, date, "存在不可用帧，详见统计（原始 Journal 未修改）");
                }
            }
            Err(e) => {
                tracing::error!(code = e.code(), venue, date, "normalize 失败");
                return ExitCode::from(EXIT_FAILED);
            }
        }
    }
    tracing::info!(date, event_rows = total, "全部交易所规范化完成");
    ExitCode::from(EXIT_OK)
}

/// 读取录制配置里的 `data_dir`。
fn load_data_dir() -> Option<PathBuf> {
    let text = match std::fs::read_to_string(RECORDING_CONFIG) {
        Ok(t) => t,
        Err(e) => {
            log_io_error(io_code(&e), "读取录制配置失败", &e);
            return None;
        }
    };
    match RecordingConfig::parse(&text) {
        Ok(c) => Some(PathBuf::from(c.data_dir)),
        Err(e) => {
            tracing::error!(path = RECORDING_CONFIG, code = e.code(), "解析录制配置失败");
            None
        }
    }
}

/// 列出该日期下**确实有 Journal 文件**的交易所目录。
fn venues_for_date(raw_dir: &Path, date: &str) -> std::io::Result<Vec<String>> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(raw_dir)? {
        let e = e?;
        if !e.path().is_dir() {
            continue;
        }
        let day = e.path().join(date);
        if day.is_dir()
            && std::fs::read_dir(&day)?.next().is_some()
            && let Some(name) = e.file_name().to_str()
        {
            out.push(name.to_owned());
        }
    }
    out.sort();
    Ok(out)
}
