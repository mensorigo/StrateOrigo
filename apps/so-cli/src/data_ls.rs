//! `so-cli data ls`：列出规范化输出与数据集清单（docs/06 §6–§7）。
//!
//! 这里只列**本地已存在**的 Parquet 文件与数据集清单，不生成哈希；
//! 哈希与清单生成属 `data check`（M2 后续交付物）。

use std::path::Path;
use std::process::ExitCode;

use crate::exit::EXIT_OK;

use so_data::pipeline::list_parquet;

/// Parquet 输出根目录。
const PARQUET_ROOT: &str = "data/parquet";
/// 数据集清单目录（docs/06 §6）。
const DATASETS_DIR: &str = "data/datasets";

/// 执行 `data ls`。
pub fn run() -> ExitCode {
    let files = list_parquet(Path::new(PARQUET_ROOT));
    if files.is_empty() {
        tracing::warn!(
            root = PARQUET_ROOT,
            "尚无规范化输出（先运行 data normalize）"
        );
    } else {
        for f in &files {
            tracing::info!(path = %f.display(), bytes = std::fs::metadata(f).map(|m| m.len()).unwrap_or(0), "parquet");
        }
    }
    let manifests = list_toml(Path::new(DATASETS_DIR));
    for f in &manifests {
        // 文件名即数据集哈希（docs/06 §7：回测只引用哈希）。
        let hash = f.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        tracing::info!(dataset_hash = hash, path = %f.display(), "dataset");
    }
    tracing::info!(
        parquet = files.len(),
        datasets = manifests.len(),
        "data ls 完成"
    );
    ExitCode::from(EXIT_OK)
}

/// 列出目录下 `.toml` 文件（目录不存在 → 空）。
fn list_toml(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<_> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
        .collect();
    out.sort();
    out
}
