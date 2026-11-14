//! 数据集清单 toml 落盘（docs/06 §7：`data/datasets/<hash>.toml`）。
//!
//! 序列化即哈希引用契约：文件名 = 数据集哈希；正典文本见 [`crate::dataset`]。
//! 重复生成同内容清单是幂等的（同哈希同内容，覆盖无害）；
//! **同哈希不同内容视为实现 bug**（正典文本决定哈希，不可能发生）。

use std::path::{Path, PathBuf};

use crate::dataset::{DatasetManifest, dataset_hash};
use crate::error::DataError;

/// 数据集清单目录（docs/06 §6）。
pub const DATASETS_DIR: &str = "data/datasets";

/// 写出清单，返回数据集哈希与文件路径。
pub fn write_manifest(m: &DatasetManifest) -> Result<(String, PathBuf), DataError> {
    write_manifest_at(m, Path::new("."))
}

/// 写出清单到指定工作根（测试与嵌套部署用；`write_manifest` 即
/// `write_manifest_at(m, ".")`，落默认 `data/datasets/`）。
pub fn write_manifest_at(
    m: &DatasetManifest,
    workspace_root: &Path,
) -> Result<(String, PathBuf), DataError> {
    let hash = dataset_hash(m);
    let dir = workspace_root.join(DATASETS_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let path = dir.join(format!("{hash}.toml"));
    std::fs::write(&path, render_toml(m))
        .map_err(|e| DataError::Parquet(format!("写清单失败：{e}")))?;
    Ok((hash, path))
}

/// 清单 → toml 文本（docs/06 §7 示例的字段与顺序）。
fn render_toml(m: &DatasetManifest) -> String {
    let mut s = String::new();
    s.push_str(&format!("name = {:?}\n", m.name));
    s.push_str(&format!("normalize_version = {:?}\n", m.normalize_version));
    s.push_str(&format!("clean_version = {:?}\n", m.clean_version));
    s.push_str(&format!("min_grade = {:?}\n", m.min_grade));
    s.push_str(&format!("venues = {:?}\n", m.venues));
    s.push_str(&format!("symbols = {:?}\n", m.symbols));
    s.push_str(&format!("tables = {:?}\n", m.tables));
    s.push_str(&format!("start = {:?}\n", m.start));
    s.push_str(&format!("end = {:?}\n", m.end));
    s.push_str("\n# 下列由工具自动生成\n");
    s.push_str(&format!("files_digest = {:?}\n", m.files_digest));
    s.push_str(&format!("files = {}\n", m.files.len()));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn manifest() -> DatasetManifest {
        let mut files = BTreeMap::new();
        files.insert("gate/trade/BTC_USDT/d.parquet".to_owned(), "aa".into());
        DatasetManifest {
            name: "test-ds".into(),
            normalize_version: "n1".into(),
            clean_version: "c1".into(),
            min_grade: "A".into(),
            venues: vec!["gate".into()],
            symbols: vec!["BTC_USDT".into()],
            tables: vec!["trade".into()],
            start: "2026-11-13T00:00:00Z".into(),
            end: "2026-11-13T23:59:59Z".into(),
            files,
            files_digest: "sha256:beef".into(),
        }
    }

    /// toml 渲染含全部字段；哈希与文件名一致。
    #[test]
    fn render_and_hash_match() {
        let m = manifest();
        let hash = dataset_hash(&m);
        assert_eq!(hash.len(), 32);
        let toml = render_toml(&m);
        assert!(toml.contains("name = \"test-ds\""));
        assert!(toml.contains("files_digest = \"sha256:beef\""));
        assert!(toml.contains("files = 1"));
    }
}
