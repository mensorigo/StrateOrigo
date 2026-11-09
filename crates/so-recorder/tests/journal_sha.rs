//! Journal 边车校验回归：重连后同一小时文件被追加，sha256 必须覆盖整个文件
//! （docs/06 §3.2 A9：同步端按边车校验完整内容）。

use std::fs;

use so_core::UnixNanos;
use so_recorder::hash::sha256_file;
use so_recorder::{JournalRow, JournalWriter};

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("so-recorder-it-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn sha_sidecar_covers_reconnect_append() {
    let dir = temp_dir("sha");
    let ts = UnixNanos(1_700_000_000_000_000_000);

    // 第一次会话写入 line1 并收尾
    let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
    w.write(&JournalRow::new(ts, "c", "S", "A", "line1"))
        .unwrap();
    w.finish().unwrap();

    // 模拟重连：同一小时文件再次打开并追加 line2
    let mut w2 = JournalWriter::new(&dir, "gate", "A").unwrap();
    w2.write(&JournalRow::new(ts, "c", "S", "A", "line2"))
        .unwrap();
    w2.finish().unwrap();

    // 边车哈希 == 整个文件的哈希（覆盖两次会话）
    let path = w2.path_for(ts.0 / 3_600_000_000_000);
    let hex = sha256_file(&path).unwrap();
    let sidecar = fs::read_to_string(path.with_extension("zst.sha256")).unwrap();
    assert!(sidecar.starts_with(&hex), "sidecar={sidecar} hex={hex}");

    let _ = fs::remove_dir_all(&dir);
}
