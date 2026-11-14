//! 数据集清单模型与哈希（docs/06 §7）。
//!
//! 清单 = 字段 + 所含文件摘要。哈希规则：`files` 按**相对路径排序**，
//! 每行 `<sha256>  <相对路径>`（sha256sum 格式），对这串文本再取
//! sha256 ⇒ `files_digest`；数据集哈希 = **清单正典文本的 sha256**
//! （正典文本 = 字段按固定顺序 `key = value` 逐行 + `files_digest` 行，
//! 与磁盘上的 toml 排版无关，保证重写不改哈希）。
//! 回测与实验只引用数据集哈希，不引用"某个目录"（docs/06 §7）。

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::DataError;

/// 数据集清单（docs/06 §7 字段；`files` 由扫描产出，其余由调用方给定）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetManifest {
    /// 人类可读名（不进哈希？——**进**：改名即新数据集，防语义漂移）。
    pub name: String,
    /// 规范化版本（如 `n1`）。
    pub normalize_version: String,
    /// 清洗规则版本（`so_data::clean::CLEAN_VERSION`）。
    pub clean_version: String,
    /// 最低质量评级（`A`/`B`/`C`）。
    pub min_grade: String,
    /// 交易所列表（排序去重）。
    pub venues: Vec<String>,
    /// 品种列表（排序去重）。
    pub symbols: Vec<String>,
    /// 包含的表名（排序去重）。
    pub tables: Vec<String>,
    /// 起止 UTC 时间（RFC3339；取所含文件覆盖范围的 min/max）。
    pub start: String,
    pub end: String,
    /// 相对路径 → 文件 sha256（hex，小写；BTreeMap 保证顺序确定）。
    pub files: BTreeMap<String, String>,
    /// 全部文件摘要的 sha256（`files_digest = "sha256:<hex>"`）。
    pub files_digest: String,
}

/// 扫描一个版本根（如 `data/parquet/n1`）下给定 `(venue, date)` 组合的
/// 全部 Parquet 文件，产出清单骨架（除 name/min_grade 外的字段自动填充）。
///
/// 目录布局（docs/06 §6）：`{root}/{venue}/{table}/{symbol}/{date}.parquet`
/// + `{root}/quality_intervals/{venue}/{date}.parquet`
/// + `{root}/quality_rebuild/{venue}/{date}.parquet`。
///
/// 扫描即校验：某 `(venue, date)` 无任何数据表文件即返回错误（清单必须
/// 描述完整一致的数据，不允许"半个数据集"）。
pub fn scan_dataset(
    root: &Path,
    venues: &[String],
    dates: &[&str],
) -> Result<ScanResult, DataError> {
    let mut res = ScanResult::default();
    for venue in venues {
        for date in dates {
            scan_one(root, venue, date, &mut res)?;
        }
    }
    finalize(&mut res);
    Ok(res)
}

/// 扫描产出（清单字段 + 文件计数）。
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// 各表品种集合。
    pub venues: std::collections::BTreeSet<String>,
    pub symbols: std::collections::BTreeSet<String>,
    pub tables: std::collections::BTreeSet<String>,
    /// 相对路径（相对 `root`）→ sha256 hex。
    pub files: BTreeMap<String, String>,
    /// 区间表与重建表行数（报告用）。
    pub interval_rows: usize,
    pub rebuild_rows: usize,
}

impl ScanResult {
    /// 组装清单（`name`/`min_grade` 由调用方给定；哈希在 `DatasetManifest` 里算）。
    pub fn into_manifest(
        self,
        name: String,
        min_grade: String,
        normalize_version: String,
        clean_version: String,
        start: String,
        end: String,
    ) -> DatasetManifest {
        let files_digest = files_digest_hex(&self.files);
        DatasetManifest {
            name,
            normalize_version,
            clean_version,
            min_grade,
            venues: self.venues.into_iter().collect(),
            symbols: self.symbols.into_iter().collect(),
            tables: self.tables.into_iter().collect(),
            start,
            end,
            files: self.files,
            files_digest,
        }
    }
}

/// 扫描一个 `(venue, date)` 单元。
fn scan_one(root: &Path, venue: &str, date: &str, res: &mut ScanResult) -> Result<(), DataError> {
    let venue_dir = root.join(venue);
    let mut tables_seen = std::collections::BTreeSet::new();
    for table in ["book_delta", "trade", "bbo", "mark_price"] {
        let tdir = venue_dir.join(table);
        let Ok(rd) = std::fs::read_dir(&tdir) else {
            continue;
        };
        for sym_dir in rd.flatten() {
            if !sym_dir.path().is_dir() {
                continue;
            }
            let p = sym_dir.path().join(format!("{date}.parquet"));
            if !p.exists() {
                continue;
            }
            let Some(symbol) = sym_dir.file_name().to_str().map(str::to_owned) else {
                return Err(DataError::Usage(format!(
                    "品种目录名非法：{}",
                    sym_dir.path().display()
                )));
            };
            add_file(root, &p, res)?;
            res.symbols.insert(symbol);
            tables_seen.insert(table.to_owned());
        }
    }
    if tables_seen.is_empty() {
        return Err(DataError::Usage(format!(
            "{venue}/{date} 无数据表文件，不能构成数据集"
        )));
    }
    res.tables.extend(tables_seen);
    res.venues.insert(venue.to_owned());
    // 区间与重建表（存在才收录，空缺不算错——纯 WS 日没有 rebuild 文件）。
    for sub in ["quality_intervals", "quality_rebuild"] {
        let p = root.join(sub).join(venue).join(format!("{date}.parquet"));
        if p.exists() {
            if sub == "quality_intervals" {
                res.interval_rows += 1;
            } else {
                res.rebuild_rows += 1;
            }
            add_file(root, &p, res)?;
        }
    }
    Ok(())
}

/// 累加一个文件：sha256 + 相对路径登记。
fn add_file(root: &Path, p: &Path, res: &mut ScanResult) -> Result<(), DataError> {
    let hex = so_journal::sha256_file(p).map_err(|e| DataError::Journal(e.to_string()))?;
    let rel = p
        .strip_prefix(root)
        .map_err(|_| DataError::Usage(format!("文件不在版本根下：{}", p.display())))?
        .to_string_lossy()
        .replace('\\', "/"); // 清单路径统一正斜杠（跨平台稳定哈希）
    res.files.insert(rel, hex);
    Ok(())
}

/// 汇总扫描结果（排序在 BTreeSet 里已保证，此处预留跨版本字段归一）。
fn finalize(_res: &mut ScanResult) {}

/// `files_digest`：排序路径逐行 `<sha256>  <path>`，再取整体 sha256。
fn files_digest_hex(files: &BTreeMap<String, String>) -> String {
    let mut text = String::new();
    for (path, hex) in files {
        text.push_str(hex);
        text.push_str("  ");
        text.push_str(path);
        text.push('\n');
    }
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// 数据集哈希：清单正典文本的 sha256（前 16 字节 hex，短引用够用且可读）。
pub fn dataset_hash(m: &DatasetManifest) -> String {
    sha256_hex(canonical_text(m).as_bytes())[..32].to_owned()
}

/// 正典文本：字段按固定顺序 `key = value` 逐行 + files_digest。
/// 与磁盘 toml 排版无关——重写清单不改哈希。
fn canonical_text(m: &DatasetManifest) -> String {
    let mut s = String::new();
    s.push_str(&format!("name = {}\n", m.name));
    s.push_str(&format!("normalize_version = {}\n", m.normalize_version));
    s.push_str(&format!("clean_version = {}\n", m.clean_version));
    s.push_str(&format!("min_grade = {}\n", m.min_grade));
    s.push_str(&format!("venues = {:?}\n", m.venues));
    s.push_str(&format!("symbols = {:?}\n", m.symbols));
    s.push_str(&format!("tables = {:?}\n", m.tables));
    s.push_str(&format!("start = {}\n", m.start));
    s.push_str(&format!("end = {}\n", m.end));
    s.push_str(&format!("files_digest = {}\n", m.files_digest));
    s
}

/// 字节串 sha256 hex（小写）。
fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(data);
    let out = h.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// files_digest 对文件集敏感：任一文件变化（路径或哈希）都改摘要。
    #[test]
    fn files_digest_changes_with_content() {
        let mut a = BTreeMap::new();
        a.insert("b/2.parquet".to_owned(), "aa".to_owned());
        let d1 = files_digest_hex(&a);
        a.insert("a/1.parquet".to_owned(), "bb".to_owned());
        let d2 = files_digest_hex(&a);
        assert_ne!(d1, d2, "加文件必须改摘要");
        let mut b = a.clone();
        b.insert("a/1.parquet".to_owned(), "cc".to_owned());
        assert_ne!(d2, files_digest_hex(&b), "内容变化必须改摘要");
    }

    /// 数据集哈希：字段全同 → 同哈希；任一字段变 → 不同哈希。
    #[test]
    fn dataset_hash_deterministic_and_sensitive() {
        let mk = |name: &str, min_grade: &str| DatasetManifest {
            name: name.to_owned(),
            normalize_version: "n1".into(),
            clean_version: "c1".into(),
            min_grade: min_grade.to_owned(),
            venues: vec!["gate".into()],
            symbols: vec!["BTC_USDT".into()],
            tables: vec!["trade".into()],
            start: "2026-11-13T00:00:00Z".into(),
            end: "2026-11-13T00:00:00Z".into(),
            files: BTreeMap::new(),
            files_digest: "sha256:deadbeef".into(),
        };
        let h1 = dataset_hash(&mk("ds", "A"));
        let h2 = dataset_hash(&mk("ds", "A"));
        assert_eq!(h1, h2, "同清单同哈希");
        assert_ne!(h1, dataset_hash(&mk("ds2", "A")), "名字变 → 新哈希");
        assert_ne!(h1, dataset_hash(&mk("ds", "B")), "评级门槛变 → 新哈希");
        // 段长：16 字节 = 32 hex。
        assert_eq!(h1.len(), 32);
    }
}
