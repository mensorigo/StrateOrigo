//! 数据集读回：按清单哈希解析 `data/datasets/<hash>.toml` 并定位文件。
//!
//! 读取端与 [`crate::dataset_out`] 的写端共用同一套字段定义（写入用
//! `render_toml`，读取用 `toml` crate 反序列化）；解析后**复算哈希并
//! 与文件名比对**——清单被手改一个字节都会被拒绝（E 类原则：数据完整性
//! 问题宁可报错不可静默）。文件列表是 `(表, 品种, 日期, 绝对路径)`，
//! 供 Python 侧 LazyFrame 扫描与 Rust 回测的多表归并共用（I8：本模块
//! 只做 I/O 与解析，不含聚合算法）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::dataset::{DatasetManifest, dataset_hash};
use crate::dataset_out::DATASETS_DIR;
use crate::error::DataError;

/// toml 反序列化视图（`files` 计数行只是展示，反序列化时忽略）。
#[derive(Debug, Deserialize)]
struct ManifestToml {
    name: String,
    normalize_version: String,
    clean_version: String,
    min_grade: String,
    venues: Vec<String>,
    symbols: Vec<String>,
    tables: Vec<String>,
    start: String,
    end: String,
    files_digest: String,
}

/// 数据集引用（按哈希解析的产物）。
#[derive(Debug, Clone)]
pub struct DatasetRef {
    /// 数据集哈希（= 清单文件名）。
    pub hash: String,
    /// 清单内容。
    pub manifest: DatasetManifest,
    /// 数据根目录（清单所在仓库根的 `data/parquet/{normalize_version}`）。
    pub root: PathBuf,
    /// `(表, 品种, 日期, 文件绝对路径)`，按 表→品种→日期 排序。
    pub files: Vec<(String, String, String, PathBuf)>,
}

/// 按数据集哈希解析清单并定位全部文件。
///
/// `workspace_root` 是仓库根（含 `data/` 的目录）。清单相对路径的基准
/// 是 `data/parquet/{normalize_version}`，与写入端一致。
pub fn load_dataset(workspace_root: &Path, hash: &str) -> Result<DatasetRef, DataError> {
    let manifest = read_manifest(workspace_root, hash)?;
    let root = workspace_root
        .join("data")
        .join("parquet")
        .join(&manifest.normalize_version);
    let files = locate_files(&root, &manifest)?;
    Ok(DatasetRef {
        hash: hash.to_owned(),
        manifest,
        root,
        files,
    })
}

/// 读清单文本 → toml → 复算哈希校验（防手改）。
fn read_manifest(workspace_root: &Path, hash: &str) -> Result<DatasetManifest, DataError> {
    let path = workspace_root
        .join(DATASETS_DIR)
        .join(format!("{hash}.toml"));
    let text = std::fs::read_to_string(&path).map_err(|e| {
        DataError::Usage(format!(
            "数据集清单不存在或不可读：{}（{e}）",
            path.display()
        ))
    })?;
    let raw: ManifestToml =
        toml::from_str(&text).map_err(|e| DataError::Usage(format!("清单解析失败：{e}")))?;
    let manifest = DatasetManifest {
        name: raw.name,
        normalize_version: raw.normalize_version,
        clean_version: raw.clean_version,
        min_grade: raw.min_grade,
        venues: raw.venues,
        symbols: raw.symbols,
        tables: raw.tables,
        start: raw.start,
        end: raw.end,
        files: BTreeMap::new(), // files 表不参与哈希（正典文本只含 files_digest）
        files_digest: raw.files_digest,
    };
    let actual = dataset_hash(&manifest);
    if actual != hash {
        return Err(DataError::Usage(format!(
            "清单内容与文件名不符：文件名 {hash}，复算 {actual}（清单可能被手改）"
        )));
    }
    Ok(manifest)
}

/// 按清单的 venues × tables 定位 parquet 文件（跳过 quality_* 表）。
fn locate_files(
    root: &Path,
    manifest: &DatasetManifest,
) -> Result<Vec<(String, String, String, PathBuf)>, DataError> {
    let mut files = Vec::new();
    for venue in &manifest.venues {
        for table in &manifest.tables {
            if table.starts_with("quality_") {
                continue; // 区间/验证表不是行情数据，回测不读
            }
            collect_table_files(&root.join(venue).join(table), table, &mut files);
        }
    }
    files.sort_by(|a, b| (&a.0, &a.1, &a.2).cmp(&(&b.0, &b.1, &b.2)));
    Ok(files)
}

/// 遍历单表的品种目录，收集 `(表, 品种, 日期, 路径)`。
fn collect_table_files(
    tdir: &Path,
    table: &str,
    files: &mut Vec<(String, String, String, PathBuf)>,
) {
    let Ok(rd) = std::fs::read_dir(tdir) else {
        return;
    };
    for sym_dir in rd.flatten() {
        if !sym_dir.path().is_dir() {
            continue;
        }
        let Ok(date_rd) = std::fs::read_dir(sym_dir.path()) else {
            continue;
        };
        let Some(symbol) = sym_dir.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        for date_file in date_rd.flatten() {
            let p = date_file.path();
            if p.extension().and_then(|s| s.to_str()) != Some("parquet") {
                continue;
            }
            let Some(date) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            files.push((table.to_owned(), symbol.clone(), date.to_owned(), p));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 写 → 读往返：字段一致、文件可定位、哈希稳定。
    #[test]
    fn write_then_load_roundtrip() {
        let tmp = std::env::temp_dir().join("so-data-dataset-in");
        let _ = std::fs::remove_dir_all(&tmp);
        let n_root = tmp.join("data").join("parquet").join("n1");
        let f = n_root.join("gate").join("trade").join("BTC_USDT");
        std::fs::create_dir_all(&f).unwrap();
        std::fs::write(f.join("2023-11-14.parquet"), b"x").unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "gate/trade/BTC_USDT/2023-11-14.parquet".to_owned(),
            "aa".to_owned(),
        );
        let m = DatasetManifest {
            name: "rt".into(),
            normalize_version: "n1".into(),
            clean_version: "c1".into(),
            min_grade: "A".into(),
            venues: vec!["gate".into()],
            symbols: vec!["BTC_USDT".into()],
            tables: vec!["trade".into()],
            start: "2023-11-14T00:00:00Z".into(),
            end: "2023-11-14T23:59:59Z".into(),
            files,
            files_digest: "sha256:beef".into(),
        };
        let (hash, _) = crate::dataset_out::write_manifest_at(&m, &tmp).unwrap();

        let loaded = load_dataset(&tmp, &hash).unwrap();
        assert_eq!(loaded.manifest.name, "rt");
        assert_eq!(loaded.manifest.min_grade, "A");
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(loaded.files[0].0, "trade");
        assert_eq!(loaded.files[0].1, "BTC_USDT");
        assert!(loaded.files[0].3.ends_with("2023-11-14.parquet"));

        // 手改清单 → 哈希不符被拒。
        let path = tmp.join(DATASETS_DIR).join(format!("{hash}.toml"));
        let mut text = std::fs::read_to_string(&path).unwrap();
        text = text.replace("rt", "evil");
        std::fs::write(&path, text).unwrap();
        let err = load_dataset(&tmp, &hash).unwrap_err();
        assert_eq!(err.code(), "DataUsage");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 哈希不存在 → Usage 错误（不是 panic）。
    #[test]
    fn missing_dataset_rejected() {
        let tmp = std::env::temp_dir().join("so-data-dataset-in-miss");
        let err = load_dataset(&tmp, "deadbeefdeadbeefdeadbeefdeadbeef").unwrap_err();
        assert_eq!(err.code(), "DataUsage");
    }
}
