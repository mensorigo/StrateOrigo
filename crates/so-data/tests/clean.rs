//! so-data 清洗集成测试（docs/05 M2：C 规则端到端、确定性）。
//!
//! 覆盖编排层 [`clean_tables`] 的行流语义：标记归属、区间产出、
//! C6 剔除、C13 未来函数防护、同输入逐位一致（D1）。

use std::collections::HashMap;

use so_data::clean::CFlags;
use so_data::clean::engine::{CleanCfg, clean_tables};
use so_data::normalize::{BboRow, BookDeltaRow, Tables, TradeRow};

fn obu(prev: u64, first: u64, ts: i64) -> BookDeltaRow {
    BookDeltaRow {
        ts_exch: ts,
        ts_recv: ts + 100,
        seq_first: first,
        seq_last: first + 1,
        seq_prev: prev,
        side: 0,
        price: 100,
        qty: 1,
        conn_id: "A".into(),
    }
}

fn tables() -> Tables {
    Tables::default()
}

/// 端到端：缺口帧被标 C2 且落区间；干净帧无标记。
#[test]
fn book_delta_gap_flow() {
    let mut src: HashMap<String, Vec<BookDeltaRow>> = HashMap::new();
    src.insert(
        "BTC_USDT".into(),
        vec![
            obu(0, 10, 1_000),
            obu(11, 12, 2_000),
            obu(13, 14, 3_000),
            obu(11, 13, 4_000),
        ],
    );
    let out = clean_tables(
        &Tables {
            book_delta: src,
            ..tables()
        },
        "gate",
        &CleanCfg::default(),
    );
    let marked = &out.book_delta["BTC_USDT"];
    assert!(!marked[0].1.intersects(CFlags::SEQ_GAP), "首帧不标");
    assert!(!marked[1].1.intersects(CFlags::SEQ_GAP));
    assert!(!marked[2].1.intersects(CFlags::SEQ_GAP));
    assert!(marked[3].1.intersects(CFlags::SEQ_GAP), "13 != 11+1 断号");
    assert_eq!(out.stats.c2_seq_gap, 1);
    assert_eq!(out.intervals.len(), 1);
    assert_eq!(out.intervals[0].venue, "gate");
    assert_eq!(out.intervals[0].flags, CFlags::SEQ_GAP.bits());
}

/// C6 非法行被剔除且计数；C1 重复被标记但保留（docs/06 §5.1 只加标记不删）。
#[test]
fn trade_invalid_removed_duplicate_kept() {
    let t = |id: u64, px: i64, qty: i64, ts: i64| TradeRow {
        ts_exch: ts,
        ts_recv: ts + 100,
        trade_id: id,
        price: px,
        qty,
        aggressor_side: 0,
        conn_id: "A".into(),
    };
    let mut src: HashMap<String, Vec<TradeRow>> = HashMap::new();
    src.insert(
        "BTC_USDT".into(),
        vec![
            t(1, 100, 1, 1_000),
            t(1, 100, 1, 1_100), // 重复 id
            t(2, 0, 1, 1_200),   // 非法价 → 剔除
            t(3, 100, 1, 1_300),
        ],
    );
    let out = clean_tables(
        &Tables {
            trade: src,
            ..tables()
        },
        "gate",
        &CleanCfg::default(),
    );
    let kept = &out.trade["BTC_USDT"];
    assert_eq!(kept.len(), 3, "非法行剔除，其余保留");
    assert!(kept[1].1.intersects(CFlags::DUPLICATE), "重复行保留但标记");
    assert_eq!(out.stats.c6_invalid, 1);
    assert_eq!(out.stats.c1_duplicate, 1);
}

/// C13 端到端：越界成交被标记；早于盘口的成交不判（未来函数防护）。
#[test]
fn trade_out_of_book_flow() {
    let t = |id: u64, px: i64, ts: i64| TradeRow {
        ts_exch: ts,
        ts_recv: ts,
        trade_id: id,
        price: px,
        qty: 1,
        aggressor_side: 0,
        conn_id: "A".into(),
    };
    let b = |seq: u64, bid: i64, ask: i64, ts: i64| BboRow {
        ts_exch: ts,
        ts_recv: ts,
        seq,
        bid_px: Some(bid),
        bid_qty: Some(1),
        ask_px: Some(ask),
        ask_qty: Some(1),
        conn_id: "A".into(),
    };
    let mut trades: HashMap<String, Vec<TradeRow>> = HashMap::new();
    trades.insert(
        "BTC_USDT".into(),
        vec![t(1, 105, 1_500), t(2, 100, 2_500), t(3, 105, 3_500)],
    );
    let mut bbos: HashMap<String, Vec<BboRow>> = HashMap::new();
    bbos.insert(
        "BTC_USDT".into(),
        vec![
            b(1, 99, 101, 1_000),
            b(2, 99, 101, 2_000),
            b(3, 99, 101, 3_000),
        ],
    );
    let out = clean_tables(
        &Tables {
            trade: trades,
            bbo: bbos,
            ..tables()
        },
        "gate",
        &CleanCfg::default(),
    );
    let marked = &out.trade["BTC_USDT"];
    assert!(
        marked[0].1.intersects(CFlags::TRADE_OUT_OF_BOOK),
        "105 高于 ask 101"
    );
    assert!(!marked[1].1.intersects(CFlags::TRADE_OUT_OF_BOOK), "带内");
    assert!(
        marked[2].1.intersects(CFlags::TRADE_OUT_OF_BOOK),
        "3_500 时可见 3_000 的盘口，105 仍越界"
    );
    assert_eq!(out.stats.c13_trade_out_of_book, 2);
}

/// C8 端到端：静默超阈值产生区间，区间起点行（静默前最后一条）标 STALE。
#[test]
fn bbo_stale_flow() {
    let b = |ts: i64| BboRow {
        ts_exch: ts,
        ts_recv: ts,
        seq: 1,
        bid_px: Some(99),
        bid_qty: Some(1),
        ask_px: Some(101),
        ask_qty: Some(1),
        conn_id: "A".into(),
    };
    let mut src: HashMap<String, Vec<BboRow>> = HashMap::new();
    src.insert(
        "BTC_USDT".into(),
        vec![b(1_000), b(2_000), b(70_000_000_000)], // 2s 后静默 70s > 60s
    );
    let out = clean_tables(
        &Tables {
            bbo: src,
            ..tables()
        },
        "gate",
        &CleanCfg::default(),
    );
    let marked = &out.bbo["BTC_USDT"];
    assert!(
        marked[1].1.intersects(CFlags::STALE),
        "区间起点行（静默前最后一条）标 STALE"
    );
    assert!(!marked[0].1.intersects(CFlags::STALE));
    assert!(!marked[2].1.intersects(CFlags::STALE));
    assert_eq!(out.stats.c8_stale_intervals, 1);
    assert_eq!(out.intervals.len(), 1);
    assert_eq!(out.intervals[0].ts_start, 2_000);
    assert_eq!(out.intervals[0].ts_end, 70_000_000_000);
}

/// 同输入两次清洗结果一致（D1）。
#[test]
fn clean_is_deterministic() {
    let mk = || {
        let mut src: HashMap<String, Vec<BboRow>> = HashMap::new();
        src.insert(
            "BTC_USDT".into(),
            (0..50)
                .map(|i| BboRow {
                    ts_exch: 1_000 + i,
                    ts_recv: 1_100 + i,
                    seq: i as u64,
                    bid_px: Some(99),
                    bid_qty: Some(1),
                    ask_px: Some(101),
                    ask_qty: Some(1),
                    conn_id: "A".into(),
                })
                .collect(),
        );
        Tables {
            bbo: src,
            ..tables()
        }
    };
    let a = clean_tables(&mk(), "gate", &CleanCfg::default());
    let b = clean_tables(&mk(), "gate", &CleanCfg::default());
    assert_eq!(a.stats, b.stats);
    let fa: Vec<_> = a.bbo["BTC_USDT"].iter().map(|(_, f)| f.bits()).collect();
    let fb: Vec<_> = b.bbo["BTC_USDT"].iter().map(|(_, f)| f.bits()).collect();
    assert_eq!(fa, fb);
}

/// 质量扫描端到端：pipeline 产物 → scan_day 能读回行数与区间（docs/06 §5.3）。
#[test]
fn scan_day_reads_pipeline_output() {
    use so_core::UnixNanos;
    use so_data::pipeline::{NormalizeRequest, normalize_day};
    use so_data::quality_scan::scan_day;
    use so_journal::JournalWriter;

    let root = std::env::temp_dir().join("so-data-quality-scan");
    let _ = std::fs::remove_dir_all(&root);
    let raw = root.join("raw");
    let ts = 1_700_000_000_000_000_000_i64;
    let contracts = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1","order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053","funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01","status":"trading","enable_decimal":false,"orders_limit":50}]"#;
    let mut w = JournalWriter::new(&raw, "gate", "A").unwrap();
    w.write(&so_journal::JournalRow::new(
        UnixNanos(ts),
        "rest.contracts",
        "*",
        "snapshot",
        contracts,
    ))
    .unwrap();
    // 正常衔接（100→101）+ 一条成交。
    let obu = r#"{"result":{"t":1700000000000,"s":"BTC_USDT","U":100,"u":101,"b":[{"p":"100.0","s":"1"}],"a":[]}}"#;
    w.write(&so_journal::JournalRow::new(
        UnixNanos(ts + 1),
        "futures.order_book_update",
        "BTC_USDT",
        "A",
        obu,
    ))
    .unwrap();
    let trade = r#"{"result":[{"id":1,"create_time_ms":1,"price":"100.5","size":"3","contract":"BTC_USDT","is_internal":false}]}"#;
    w.write(&so_journal::JournalRow::new(
        UnixNanos(ts + 2),
        "futures.trades",
        "BTC_USDT",
        "A",
        trade,
    ))
    .unwrap();
    w.finish().unwrap();
    let req = NormalizeRequest {
        raw_dir: &raw,
        out_root: &root.join("parquet"),
        venue: "gate",
        date: "2023-11-14",
    };
    let (stats, out_dir) = normalize_day(&req).unwrap();
    assert_eq!(stats.event_rows, 2);

    let scan = scan_day(&root.join("parquet").join("n1"), "2023-11-14").unwrap();
    let cell = &scan.cells[&("gate".to_owned(), "BTC_USDT".to_owned())];
    assert_eq!(cell.actual_rows, 2, "book_delta 1 行 + trade 1 行");
    assert_eq!(cell.gap_count, 0, "正常衔接无缺口");
    let _ = out_dir;
    let _ = std::fs::remove_dir_all(&root);
}

/// 未来函数测试（docs/05 M2 验收）：篡改 t 之后的数据，t 及之前的行标记不变。
///
/// 构造两份数据：A 份在第 30 行之后是"正常价格"；B 份在第 30 行之后插入
/// 交叉盘口与断流。前 30 行相同 ⇒ 前 30 行的 flags 必须逐行相同——
/// 任何依赖"未来行"的规则（如 C4/C8 提前回填）都会在这里露馅。
#[test]
fn flags_of_past_rows_do_not_depend_on_future_data() {
    let base_rows = |suffix: bool| {
        let mut rows = Vec::new();
        for i in 0..30 {
            rows.push(BboRow {
                ts_exch: 1_000 + i,
                ts_recv: 1_000 + i,
                seq: i as u64,
                bid_px: Some(99),
                bid_qty: Some(1),
                ask_px: Some(101),
                ask_qty: Some(1),
                conn_id: "A".into(),
            });
        }
        if suffix {
            // 交叉盘口（C4）+ 大间隔（C8）——只应影响第 30 行之后的行。
            rows.push(BboRow {
                ts_exch: 2_000,
                ts_recv: 2_000,
                seq: 30,
                bid_px: Some(200),
                bid_qty: Some(1),
                ask_px: Some(101),
                ask_qty: Some(1),
                conn_id: "A".into(),
            });
            rows.push(BboRow {
                ts_exch: 90_000_000_000,
                ts_recv: 90_000_000_000,
                seq: 31,
                bid_px: Some(99),
                bid_qty: Some(1),
                ask_px: Some(101),
                ask_qty: Some(1),
                conn_id: "A".into(),
            });
        }
        rows
    };
    let mut a: HashMap<String, Vec<BboRow>> = HashMap::new();
    a.insert("BTC_USDT".to_owned(), base_rows(false));
    let mut b: HashMap<String, Vec<BboRow>> = HashMap::new();
    b.insert("BTC_USDT".to_owned(), base_rows(true));
    let out_a = clean_tables(&Tables { bbo: a, ..tables() }, "gate", &CleanCfg::default());
    let out_b = clean_tables(&Tables { bbo: b, ..tables() }, "gate", &CleanCfg::default());
    let fa = &out_a.bbo["BTC_USDT"];
    let fb = &out_b.bbo["BTC_USDT"];
    for i in 0..30 {
        assert_eq!(fa[i].1, fb[i].1, "第 {i} 行标记不应受未来数据影响");
    }
}
