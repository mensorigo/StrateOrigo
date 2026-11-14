//! C9 订单簿重建验证端到端（docs/06 §5.2 C9、docs/05 M2 验收）。
//!
//! Journal 带 rest.order_book 快照 → pipeline 产出 quality_rebuild 表 →
//! 评级输入的 rebuild_match_rate；不达标品种落 C9 区间（区间路径的
//! 单元测试在 src/pipeline.rs 与 tests/bookcheck.rs 覆盖）。

use std::path::Path;

use so_core::UnixNanos;
use so_data::pipeline::{NormalizeRequest, normalize_day};
use so_data::quality_scan::scan_day;
use so_journal::{JournalRow, JournalWriter};

/// 完全一致的快照 → rebuild_match_rate = 1.0，不落 C9 区间。
#[test]
fn c9_rebuild_through_pipeline() {
    let root = std::env::temp_dir().join("so-data-c9");
    let _ = std::fs::remove_dir_all(&root);
    write_journal(&root);
    let req = NormalizeRequest {
        raw_dir: &root.join("raw"),
        out_root: &root.join("parquet"),
        venue: "gate",
        date: "2023-11-14",
    };
    let _ = normalize_day(&req).unwrap();
    // quality_rebuild 表已落盘。
    let rebuild_path = root
        .join("parquet")
        .join("n1")
        .join("quality_rebuild")
        .join("gate")
        .join("2023-11-14.parquet");
    assert!(rebuild_path.exists(), "quality_rebuild 表应存在");
    // 评级输入带上一致率；完全一致 → 不落 C9 区间。
    let scan = scan_day(&root.join("parquet").join("n1"), "2023-11-14").unwrap();
    let cell = &scan.cells[&("gate".to_owned(), "BTC_USDT".to_owned())];
    assert_eq!(cell.rebuild_match_rate, Some(1.0));
    let _ = std::fs::remove_dir_all(&root);
}

/// 写一段"增量与快照完全一致"的 Journal（contracts + 快照锚定 + 增量 + 触发比对）。
fn write_journal(root: &Path) {
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
    // 快照锚定：买 100@1 / 卖 101@1
    let snap = |id: u64, bid_s: &str| {
        format!(
            r#"{{"id":{id},"current":1.0,"update":1.0,"bids":[{{"p":"100.0","s":"{bid_s}"}}],"asks":[{{"p":"101.0","s":"1"}}]}}"#
        )
    };
    w.write(&JournalRow::new(
        UnixNanos(ts + 1),
        "rest.order_book",
        "BTC_USDT",
        "A",
        snap(100, "1"),
    ))
    .unwrap();
    let obu = |first: u64, last: u64, bid_s: &str| {
        format!(
            r#"{{"result":{{"t":1700000000000,"s":"BTC_USDT","U":{first},"u":{last},"b":[{{"p":"100.0","s":"{bid_s}"}}],"a":[]}}}}"#
        )
    };
    // 增量把买一改为 2；新快照也记录 2 → 应完全一致。
    w.write(&JournalRow::new(
        UnixNanos(ts + 2),
        "futures.order_book_update",
        "BTC_USDT",
        "A",
        obu(100, 101, "2"),
    ))
    .unwrap();
    w.write(&JournalRow::new(
        UnixNanos(ts + 3),
        "rest.order_book",
        "BTC_USDT",
        "A",
        snap(101, "2"),
    ))
    .unwrap();
    // 增量越过 101 → 触发比对。
    w.write(&JournalRow::new(
        UnixNanos(ts + 4),
        "futures.order_book_update",
        "BTC_USDT",
        "A",
        obu(102, 103, "2"),
    ))
    .unwrap();
    w.finish().unwrap();
}
