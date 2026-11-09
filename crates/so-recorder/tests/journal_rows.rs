//! Journal 行序列化与滚动写入（docs/06 §3.1）。

use std::fs;

use so_core::UnixNanos;
use so_recorder::{JournalRow, JournalWriter};

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("so-recorder-journal-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn journal_row_jsonl_roundtrip() {
    let row = JournalRow::new(
        UnixNanos(1_700_000_000_000_000_000),
        "futures.trades",
        "BTC_USDT",
        "A",
        r#"{"raw":"frame"}"#,
    );
    let jsonl = row.to_jsonl().unwrap();
    assert!(jsonl.ends_with('\n'));
    // 反序列化往返一致
    let back: JournalRow = serde_json::from_str(jsonl.trim()).unwrap();
    assert_eq!(back, row);
}

#[test]
fn writer_rolls_and_writes_sha() {
    let dir = temp_dir("roll");
    let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
    // 2023-11-14 22:13:20 UTC = 1700000000
    w.write(&JournalRow::new(
        UnixNanos(1_700_000_000_000_000_000),
        "c",
        "S",
        "A",
        "line1",
    ))
    .unwrap();
    // 2023-11-14 23:13:20 UTC（下一小时 = +3600 秒 = +3.6e12 纳秒）
    w.write(&JournalRow::new(
        UnixNanos(1_700_000_000_000_000_000 + 3_600_000_000_000),
        "c",
        "S",
        "A",
        "line2",
    ))
    .unwrap();
    w.finish().unwrap();
    // 两个文件 + 两个 sha256
    let d1 = dir.join("gate").join("2023-11-14");
    assert!(d1.join("22.A.journal.zst").exists());
    assert!(d1.join("22.A.journal.zst.sha256").exists());
    assert!(d1.join("23.A.journal.zst").exists());
    assert_eq!(w.stats().total_rows, 2);
    let _ = fs::remove_dir_all(&dir);
}
