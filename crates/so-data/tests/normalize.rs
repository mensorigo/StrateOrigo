//! so-data 集成测试（docs/05 M2：normalize 确定性、Parquet 往返）。

use std::collections::HashMap;

use so_core::Scale;
use so_data::normalize::Normalizer;
use so_data::parquet_out::write_all;
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

fn trade_raw(id: u64, price: &str, size: &str) -> String {
    format!(
        r#"{{"result":[{{"id":{id},"create_time_ms":1,"price":"{price}","size":"{size}","contract":"BTC_USDT","is_internal":false}}]}}"#
    )
}

fn scales() -> HashMap<String, Scale> {
    let mut m = HashMap::new();
    m.insert("BTC_USDT".to_owned(), Scale::new(1).unwrap());
    m
}

#[test]
fn obu_frame_splits_into_per_level_rows() {
    let mut n = Normalizer::new(scales());
    let raw = obu_raw(
        100,
        102,
        r#"[{"p":"100.0","s":"1"},{"p":"99.5","s":"2"}]"#,
        r#"[{"p":"101.0","s":"3"}]"#,
    );
    assert!(n.on_row(&row("futures.order_book_update", "BTC_USDT", &raw, 10)));
    let rows = &n.tables.book_delta["BTC_USDT"];
    assert_eq!(rows.len(), 3);
    // 首行 seq_prev = 0（首帧）
    assert_eq!(rows[0].seq_prev, 0);
    assert_eq!((rows[0].seq_first, rows[0].seq_last), (100, 102));
    assert_eq!(rows[0].price, 1000); // scale 1：100.0 → 1000
    // 第二帧：seq_prev = 上一帧 seq_last
    let raw2 = obu_raw(103, 104, r#"[{"p":"98.0","s":"9"}]"#, "[]");
    n.on_row(&row("futures.order_book_update", "BTC_USDT", &raw2, 20));
    let rows = &n.tables.book_delta["BTC_USDT"];
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[3].seq_prev, 102);
}

#[test]
fn trade_frame_parses_and_signs() {
    let mut n = Normalizer::new(scales());
    // size 正 = 主动买（bid），负 = 主动卖（ask）
    n.on_row(&row(
        "futures.trades",
        "BTC_USDT",
        &trade_raw(1, "100.5", "3"),
        10,
    ));
    n.on_row(&row(
        "futures.trades",
        "BTC_USDT",
        &trade_raw(2, "100.0", "-2"),
        20,
    ));
    let rows = &n.tables.trade["BTC_USDT"];
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].aggressor_side, 0);
    assert_eq!(rows[0].price, 1005);
    assert_eq!(rows[1].aggressor_side, 1);
    assert_eq!(rows[1].qty, 20); // scale 1：-2 → 20；负号转方向，qty 恒正
}

#[test]
fn normalize_is_deterministic() {
    let input = [
        row(
            "futures.order_book_update",
            "BTC_USDT",
            &obu_raw(1, 2, r#"[{"p":"100.0","s":"1"}]"#, "[]"),
            10,
        ),
        row(
            "futures.trades",
            "BTC_USDT",
            &trade_raw(1, "100.5", "3"),
            11,
        ),
        row(
            "futures.order_book_update",
            "BTC_USDT",
            &obu_raw(3, 4, "[]", r#"[{"p":"101.0","s":"1"}]"#),
            12,
        ),
    ];
    let mut a = Normalizer::new(scales());
    let mut b = Normalizer::new(scales());
    for r in &input {
        a.on_row(r);
        b.on_row(r);
    }
    assert_eq!(a.tables.book_delta, b.tables.book_delta);
    assert_eq!(a.tables.trade, b.tables.trade);
}

#[test]
fn parquet_roundtrip_row_counts() {
    let mut n = Normalizer::new(scales());
    n.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        &obu_raw(1, 2, r#"[{"p":"100.0","s":"1"}]"#, "[]"),
        10,
    ));
    n.on_row(&row(
        "futures.trades",
        "BTC_USDT",
        &trade_raw(1, "100.5", "3"),
        11,
    ));
    let dir = std::env::temp_dir().join("so-data-test-roundtrip");
    let _ = std::fs::remove_dir_all(&dir);
    let written = write_all(&dir, &n.tables, &scales(), "2023-11-14").unwrap();
    assert_eq!(written, 2);
    assert!(
        dir.join("book_delta")
            .join("BTC_USDT")
            .join("2023-11-14.parquet")
            .exists()
    );
    assert!(
        dir.join("trade")
            .join("BTC_USDT")
            .join("2023-11-14.parquet")
            .exists()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unreadable_frames_counted_not_dropped_silently() {
    let mut n = Normalizer::new(scales());
    n.on_row(&row(
        "futures.order_book_update",
        "BTC_USDT",
        r#"{"result":{"broken"#,
        10,
    ));
    n.on_row(&row(
        "futures.trades",
        "XXX_USDT",
        &trade_raw(1, "1", "1"),
        11,
    ));
    assert_eq!(n.parse_failed, 1);
    assert_eq!(n.no_scale, 1);
}
