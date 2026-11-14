//! BookCheck 集成测试（docs/05 M2 验收：重建簿 vs REST 快照）。
//!
//! 从 so-recorder 迁来（C9 归位 so-data，2026-11-13）；数据流不变。

use std::collections::HashMap;

use so_core::Scale;
use so_data::bookcheck::{BookCheck, SymbolStats};
use so_journal::JournalRow;

fn row(channel: &str, symbol: &str, raw: &str, ts_recv: i64) -> JournalRow {
    JournalRow {
        ts_recv,
        channel: channel.to_owned(),
        symbol: symbol.to_owned(),
        conn_id: "A".to_owned(),
        raw: raw.to_owned(),
    }
}

fn obu_raw(first: u64, last: u64, bids: &str, asks: &str) -> String {
    format!(r#"{{"result":{{"t":1,"s":"BTC_USDT","U":{first},"u":{last},"b":{bids},"a":{asks}}}}}"#)
}

fn snap_raw(id: u64, bids: &str, asks: &str) -> String {
    format!(r#"{{"id":{id},"current":1.0,"update":1.0,"bids":{bids},"asks":{asks}}}"#)
}

fn checker() -> BookCheck {
    let mut scales = HashMap::new();
    scales.insert("BTC_USDT".to_owned(), Scale::new(1).unwrap());
    BookCheck::new(scales)
}

fn first_stats(c: &BookCheck) -> SymbolStats {
    c.stats().first().unwrap().1.clone()
}

#[test]
fn rebuild_matches_snapshot() {
    let mut c = checker();
    // 快照 id=100：买 100@1，卖 101@1
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            100,
            r#"[{"p":"100.0","s":"1"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        100,
    ));
    // 增量 U=100 u=102：覆盖买一为 2
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(100, 102, r#"[{"p":"100.0","s":"2"}]"#, "[]"),
        110,
    ));
    // 新快照 id=102 到达（含买一 100@2）
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            102,
            r#"[{"p":"100.0","s":"2"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        120,
    ));
    // 增量越过 102 → 触发比对
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(103, 104, r#"[{"p":"101.0","s":"3"}]"#, "[]"),
        130,
    ));
    let s = first_stats(&c);
    assert_eq!(s.comparisons, 1);
    assert_eq!(s.matched, 1);
    assert_eq!(s.level_rate(), 1.0);
    // 比对时间窗记录（C9 区间用）
    assert_eq!(s.first_cmp_ts, 120);
    assert_eq!(s.last_cmp_ts, 120);
    // 重锚后增量继续应用
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(105, 106, "[]", r#"[{"p":"102.0","s":"1"}]"#),
        140,
    ));
    let s2 = first_stats(&c);
    assert_eq!(s2.gaps, 0);
}

#[test]
fn mismatch_detected_when_level_drifts() {
    let mut c = checker();
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            100,
            r#"[{"p":"100.0","s":"1"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        100,
    ));
    // 增量把买一改为 2，但新快照记录 9（重建错误/数据异常模拟）
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(100, 101, r#"[{"p":"100.0","s":"2"}]"#, "[]"),
        110,
    ));
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            101,
            r#"[{"p":"100.0","s":"9"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        120,
    ));
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(102, 103, "[]", "[]"),
        130,
    ));
    let s = first_stats(&c);
    assert_eq!(s.comparisons, 1);
    assert_eq!(s.matched, 0);
    assert_eq!(s.levels, 2);
    assert_eq!(s.levels_matched, 1); // 卖侧一致、买侧不一致
}

#[test]
fn duplicate_frames_applied_once() {
    let mut c = checker();
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            100,
            r#"[{"p":"100.0","s":"1"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        100,
    ));
    let frame = obu_raw(100, 101, r#"[{"p":"100.0","s":"5"}]"#, "[]");
    c.on_row(&row("futures.order_book_update", "BTC_USDT", &frame, 110));
    // 同帧从 B 流再来：不重复应用
    let mut b = row("futures.order_book_update", "BTC_USDT", &frame, 120);
    b.conn_id = "B".to_owned();
    c.on_row(&b);
    assert_eq!(first_stats(&c).frames, 1);
}

#[test]
fn gap_counted_when_snapshot_target_lost() {
    let mut c = checker();
    let snap = snap_raw(
        100,
        r#"[{"p":"100.0","s":"1"}]"#,
        r#"[{"p":"101.0","s":"1"}]"#,
    );
    // 首快照锚定
    c.on_row(&row("rest.order_book", "BTC_USDT", &snap, 100));
    // 衔接增量（进入 Streaming）
    c.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(100, 101, r#"[{"p":"100.0","s":"1"}]"#, "[]"),
        110,
    ));
    // 快照 id=101 设为待比对目标；没有增量越过它，下一个快照直接覆盖 → 记 1 次 Gap
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            101,
            r#"[{"p":"100.0","s":"1"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        120,
    ));
    c.on_row(&row(
        "rest.order_book",
        "BTC_USDT",
        &snap_raw(
            200,
            r#"[{"p":"100.0","s":"1"}]"#,
            r#"[{"p":"101.0","s":"1"}]"#,
        ),
        200,
    ));
    assert_eq!(first_stats(&c).gaps, 1);
    // 被覆盖的目标未比对
    assert_eq!(first_stats(&c).comparisons, 0);
}

#[test]
fn other_channels_ignored() {
    let mut c = checker();
    assert!(!c.on_row(&row(
        "futures.trades",
        "BTC_USDT",
        r#"{"result":[{"id":1}]}"#,
        1,
    )));
    assert!(c.stats().is_empty());
}
