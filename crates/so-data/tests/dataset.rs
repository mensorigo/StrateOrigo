//! 数据集清单端到端测试（docs/06 §7）：pipeline 产物 → 扫描 → 清单 → 哈希。

use std::path::Path;

use so_core::UnixNanos;
use so_data::dataset::scan_dataset;
use so_data::dataset_out::write_manifest;
use so_data::pipeline::{NormalizeRequest, normalize_day};
use so_journal::{JournalRow, JournalWriter};

/// 造一天的产物（与 tests/clean.rs 同款数据流）。
fn make_day(root: &std::path::Path) {
    let raw = root.join("raw");
    let ts = 1_700_000_000_000_000_000_i64;
    let contracts = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1","order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053","funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01","status":"trading","enable_decimal":false,"orders_limit":50}]"#;
    let mut w = JournalWriter::new(&raw, "gate", "A").unwrap();
    w.write(&JournalRow::new(
        UnixNanos(ts),
        "rest.contracts",
        "*",
        "snapshot",
        contracts,
    ))
    .unwrap();
    let obu = r#"{"result":{"t":1700000000000,"s":"BTC_USDT","U":100,"u":101,"b":[{"p":"100.0","s":"1"}],"a":[]}}"#;
    w.write(&JournalRow::new(
        UnixNanos(ts + 1),
        "futures.order_book_update",
        "BTC_USDT",
        "A",
        obu,
    ))
    .unwrap();
    w.finish().unwrap();
    let req = NormalizeRequest {
        raw_dir: &raw,
        out_root: &root.join("parquet"),
        venue: "gate",
        date: "2023-11-14",
    };
    let _ = normalize_day(&req).unwrap();
}

/// 扫描 → 清单 → 哈希稳定；文件名与哈希一致；toml 含 files_digest。
#[test]
fn dataset_scan_manifest_hash_roundtrip() {
    let root = std::env::temp_dir().join("so-data-dataset");
    let _ = std::fs::remove_dir_all(&root);
    make_day(&root);
    let n_root = root.join("parquet").join("n1");
    let venues = vec!["gate".to_owned()];

    let scanned = scan_dataset(&n_root, &venues, &["2023-11-14"]).unwrap();
    assert!(scanned.symbols.contains("BTC_USDT"));
    assert!(scanned.tables.iter().any(|t| t == "book_delta"));

    let m1 = scanned.clone().into_manifest(
        "test-ds".into(),
        "A".into(),
        "n1".into(),
        "c1".into(),
        "2023-11-14T00:00:00Z".into(),
        "2023-11-14T23:59:59Z".into(),
    );
    let m2 = scanned.into_manifest(
        "test-ds".into(),
        "A".into(),
        "n1".into(),
        "c1".into(),
        "2023-11-14T00:00:00Z".into(),
        "2023-11-14T23:59:59Z".into(),
    );
    let (hash, path) = write_manifest(&m1).unwrap();
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("{hash}.toml")
    );
    // 同清单重扫再写：哈希不变（幂等）。
    let (hash2, _) = write_manifest(&m2).unwrap();
    assert_eq!(hash, hash2, "同一数据集两次生成哈希必须一致");
    // toml 落盘含关键字段。
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("name = \"test-ds\""));
    assert!(text.contains("files_digest = \"sha256:"));
    assert!(text.contains("clean_version = \"c1\""));
    // 少一个文件（删掉 quality_rebuild）→ 新哈希。
    std::fs::remove_file(
        n_root
            .join("quality_rebuild")
            .join("gate")
            .join("2023-11-14.parquet"),
    )
    .unwrap();
    let scanned3 = scan_dataset(&n_root, &venues, &["2023-11-14"]).unwrap();
    let m3 = scanned3.into_manifest(
        "test-ds".into(),
        "A".into(),
        "n1".into(),
        "c1".into(),
        "2023-11-14T00:00:00Z".into(),
        "2023-11-14T23:59:59Z".into(),
    );
    assert_ne!(
        so_data::dataset::dataset_hash(&m3),
        hash,
        "文件集变化必须改哈希"
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = Path::new("unused");
}

/// 缺数据表的单元拒绝入集（"半个数据集"防护）。
#[test]
fn scan_rejects_empty_venue_date() {
    let root = std::env::temp_dir().join("so-data-dataset-empty");
    let _ = std::fs::remove_dir_all(&root);
    let n_root = root.join("n1");
    std::fs::create_dir_all(n_root.join("gate").join("trade")).unwrap();
    let err = scan_dataset(&n_root, &["gate".to_owned()], &["2023-11-14"]).unwrap_err();
    assert_eq!(err.code(), "DataUsage");
    let _ = std::fs::remove_dir_all(&root);
}
