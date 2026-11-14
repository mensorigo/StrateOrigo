//! `so-cli data bar`：从 trade Parquet 生成 Bar（docs/06 §8）。
//!
//! 算法在 [`so_data::bar`] / [`so_data::bar_pipeline`]，本模块只做
//! 参数解析、目录编排与日志（04 I9：编排层不做算法）。

use std::path::Path;
use std::process::ExitCode;
use std::str::FromStr;

use crate::exit::{CliCode, EXIT_FAILED, EXIT_OK};
use so_data::bar::{BarCfg, BarKind};
use so_data::bar_pipeline::generate_day_bars;
use so_data::pipeline::NORMALIZE_VERSION;

/// Parquet 输出根目录（与 data_normalize 一致）。
const PARQUET_ROOT: &str = "data/parquet";

/// 执行 `data bar --date YYYY-MM-DD --kind time|volume|notional --value N`。
pub fn run(date: &str, kind_str: &str, value: i64) -> ExitCode {
    let kind = match BarKind::from_str(kind_str) {
        Ok(k) => k,
        Err(e) => {
            tracing::error!(code = CliCode::Usage.as_str(), "{e}");
            return ExitCode::from(EXIT_FAILED);
        }
    };
    if value <= 0 {
        tracing::error!(code = CliCode::Usage.as_str(), "value 必须为正整数");
        return ExitCode::from(EXIT_FAILED);
    }
    let cfg = match kind {
        BarKind::Time => BarCfg::time(value),
        BarKind::Volume => BarCfg::volume(value),
        BarKind::Notional => BarCfg::notional(value),
    };

    let n_root = Path::new(PARQUET_ROOT).join(NORMALIZE_VERSION);
    match generate_day_bars(&n_root, date, &cfg) {
        Ok(stats) => {
            tracing::info!(
                date,
                kind = kind.as_str(),
                value,
                trade_rows = stats.trade_rows,
                bars = stats.bars,
                "Bar 生成完成"
            );
            ExitCode::from(EXIT_OK)
        }
        Err(e) => {
            tracing::error!(code = e.code(), date, "Bar 生成失败");
            ExitCode::from(EXIT_FAILED)
        }
    }
}
