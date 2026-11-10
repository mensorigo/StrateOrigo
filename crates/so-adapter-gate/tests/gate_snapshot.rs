//! Gate 消息解析快照测试：样例取自官方文档 v4.106.179
//! （docs/gate-api-v4.106.179-markdown/，docs/04 §4 快照测试）。
//!
//! 样例 JSON 是官方文档逐字引用；`ts_recv` 由测试固定值模拟 I/O 层打点。

use so_adapter_gate::{GateError, parse_bbo, parse_obu, parse_trade};
use so_core::{Price, Qty, Scale, UnixNanos};
use so_model::Side;

fn sc() -> Scale {
    Scale::new(1).unwrap() // Gate BTC_USDT: order_price_round 0.1
}

const RECV: UnixNanos = UnixNanos(1_700_000_000_000_000_000);

// —— futures.book_ticker（官方样例，docs/venues/gate.md §3.3）——

#[test]
fn book_ticker_official_sample() {
    let json = r#"{
        "t": 1615366379123,
        "u": 2517661076,
        "s": "BTC_USDT",
        "b": "54696.6",
        "B": "37000.5",
        "a": "54696.7",
        "A": "47061.2"
    }"#;
    let bbo = parse_bbo(json, RECV, sc()).unwrap();
    assert_eq!(bbo.update_id, 2_517_661_076);
    assert_eq!(
        bbo.ts_exch,
        UnixNanos::from_millis(1_615_366_379_123).unwrap()
    );
    assert_eq!(bbo.bid, Some(Price(546_966)));
    assert_eq!(bbo.bid_qty, Some(Qty(370_005)));
    assert_eq!(bbo.ask, Some(Price(546_967)));
    assert_eq!(bbo.ask_qty, Some(Qty(470_612)));
}

#[test]
fn book_ticker_empty_side() {
    // 官方：a 为空字符串 = 空卖盘
    let json = r#"{
        "t": 1615366379123, "u": 1, "s": "BTC_USDT",
        "b": "54696.6", "B": "37000.5",
        "a": "", "A": "0"
    }"#;
    let bbo = parse_bbo(json, RECV, sc()).unwrap();
    assert_eq!(bbo.bid, Some(Price(546_966)));
    assert!(bbo.ask.is_none());
    assert!(bbo.ask_qty.is_none());
}

// —— futures.order_book_update（官方样例）——

#[test]
fn obu_official_sample() {
    let json = r#"{
        "t": 1615366381417,
        "s": "BTC_USDT",
        "U": 2517661101,
        "u": 2517661113,
        "b": [ {"p": "54672.1", "s": "0"}, {"p": "54664.5", "s": "58794.5"} ],
        "a": [ {"p": "54743.6", "s": "0"}, {"p": "54742", "s": "95.2"} ],
        "l": "100"
    }"#;
    let obu = parse_obu(json, RECV, sc()).unwrap();
    assert_eq!(obu.first_id, 2_517_661_101);
    assert_eq!(obu.last_id, 2_517_661_113);
    assert!(!obu.full_replace);
    let lv = obu.levels.as_slice();
    assert_eq!(lv.len(), 4);
    // b: 54672.1 qty=0（删档）；54664.5 qty=58794.5
    assert_eq!(lv[0].side, Side::Bid);
    assert_eq!((lv[0].price, lv[0].qty), (Price(546_721), Qty(0)));
    assert_eq!((lv[1].price, lv[1].qty), (Price(546_645), Qty(587_945)));
    // a: 54743.6 qty=0；54742 qty=95.2
    assert_eq!(lv[2].side, Side::Ask);
    assert_eq!((lv[2].price, lv[2].qty), (Price(547_436), Qty(0)));
    assert_eq!((lv[3].price, lv[3].qty), (Price(547_420), Qty(952)));
}

#[test]
fn obu_full_replace_and_empty_sides() {
    // 官方规则 §3.2 第 5 步：b/a 为空也必须更新本地 id —— 空档位数组合法
    let json = r#"{
        "t": 1615366381417, "s": "BTC_USDT",
        "U": 100, "u": 110,
        "full": true,
        "b": [], "a": [], "l": "100"
    }"#;
    let obu = parse_obu(json, RECV, sc()).unwrap();
    assert!(obu.full_replace);
    assert_eq!(obu.levels.as_slice().len(), 0);
    assert_eq!(obu.last_id, 110);
}

#[test]
fn obu_integer_size_accepted() {
    // 不带 X-Gate-Size-Decimal 头时 s 是整型张数（2026-11-10 实测录制样本）
    let json = r#"{
        "t": 1794272504605, "s": "BTC_USDT",
        "U": 130949312629, "u": 130949312695,
        "a": [ {"p": "75202", "s": 4152} ],
        "b": [ {"p": "75201.6", "s": 709} ],
        "l": "100"
    }"#;
    let obu = parse_obu(json, RECV, sc()).unwrap();
    let lv = obu.levels.as_slice();
    assert_eq!(lv.len(), 2);
    // parse_obu 先 b 后 a（与官方样例测试一致）
    assert_eq!(lv[0].side, Side::Bid);
    assert_eq!(lv[0].price, Price(752_016));
    assert_eq!(lv[0].qty, Qty(7_090)); // 709 × 10
    assert_eq!(lv[1].side, Side::Ask);
    assert_eq!(lv[1].price, Price(752_020));
    assert_eq!(lv[1].qty, Qty(41_520)); // 4152 × 10
}

#[test]
fn bbo_and_trade_integer_size_accepted() {
    // book_ticker 的 B/A 与 trades 的 size 同样有整型形态（实测样本）
    // SOL_USDT 价格两位小数，用 scale 2
    let sc2 = Scale::new(2).unwrap();
    let bbo = parse_bbo(
        r#"{"t":1794272504650,"u":44408682894,"s":"SOL_USDT","b":"86.69","B":100,"a":"86.7","A":321}"#,
        RECV,
        sc2,
    )
    .unwrap();
    assert_eq!(bbo.bid_qty, Some(Qty(10_000))); // 100 × 100
    assert_eq!(bbo.ask_qty, Some(Qty(32_100))); // 321 × 100

    // ETH_USDT 价格 2130.8 一位小数，用 scale 1
    let json = r#"[{"id":968914222,"size":0,"create_time":1794272504,"create_time_ms":1794272504707,"price":"2130.8","contract":"ETH_USDT"}]"#;
    let mut out = [so_adapter_gate::RawTrade {
        ts_recv: UnixNanos(0),
        ts_exch: UnixNanos(0),
        trade_id: 0,
        price: Price::ZERO,
        qty: Qty::ZERO,
        aggressor: Side::Bid,
        internal: false,
    }; 4];
    let n = parse_trade(json, RECV, sc(), &mut out).unwrap();
    assert_eq!(n, 1);
    assert_eq!(out[0].trade_id, 968_914_222);
    assert_eq!(out[0].qty, Qty(0));
}

// —— futures.trades（官方样例；size 正 = 主动买）——

#[test]
fn trades_official_sample() {
    let json = r#"[
        {
            "size": "-108.5",
            "id": 27753479,
            "create_time": 1545136464,
            "create_time_ms": 1545136464123,
            "price": "96.4",
            "contract": "BTC_USDT",
            "is_internal": true
        }
    ]"#;
    let mut out = [so_adapter_gate::RawTrade {
        ts_recv: UnixNanos(0),
        ts_exch: UnixNanos(0),
        trade_id: 0,
        price: Price::ZERO,
        qty: Qty::ZERO,
        aggressor: Side::Bid,
        internal: false,
    }; 4];
    let n = parse_trade(json, RECV, sc(), &mut out).unwrap();
    assert_eq!(n, 1);
    assert_eq!(out[0].trade_id, 27_753_479);
    assert_eq!(out[0].price, Price(964));
    assert_eq!(out[0].qty, Qty(1_085)); // 负号 → 绝对量
    assert_eq!(out[0].aggressor, Side::Ask); // 负 = 主动卖
    assert!(out[0].internal);
}

#[test]
fn trades_positive_size_is_buy_aggressor() {
    let json = r#"[
        { "size": "0.5", "id": 1, "create_time_ms": 1545136464123,
          "price": "96.4", "contract": "BTC_USDT" }
    ]"#;
    let mut out = [so_adapter_gate::RawTrade {
        ts_recv: UnixNanos(0),
        ts_exch: UnixNanos(0),
        trade_id: 0,
        price: Price::ZERO,
        qty: Qty::ZERO,
        aggressor: Side::Ask,
        internal: false,
    }; 4];
    let n = parse_trade(json, RECV, sc(), &mut out).unwrap();
    assert_eq!(n, 1);
    assert_eq!(out[0].aggressor, Side::Bid);
    assert!(!out[0].internal); // is_internal 缺省
}

// —— futures.tickers（官方样例：标记价 + 资金费）——

#[test]
fn tickers_official_sample() {
    // 标记价/指数价精度独立于 tick（Gate 标记价为计算值，两位小数常见），
    // 此处用 scale 2；品种实际精度由 configs/instruments 快照决定。
    let sc2 = Scale::new(2).unwrap();
    let json = r#"[
        {
            "contract": "BTC_USDT",
            "last": "118.4",
            "change_percentage": "0.77",
            "funding_rate": "-0.000114",
            "mark_price": "118.35",
            "index_price": "118.36",
            "total_size": "73648.5",
            "volume_24h": "745487577",
            "low_24h": "99.2",
            "high_24h": "132.5",
            "t": 1541659086123
        }
    ]"#;
    let t = so_adapter_gate::parse_ticker(json, RECV, sc2).unwrap();
    assert_eq!(t.mark, Price(11_835));
    assert_eq!(t.index, Some(Price(11_836)));
    assert_eq!(t.rate_e8, -11_400); // -0.000114 × 10^8
}

// —— 错误路径 ——

#[test]
fn rejects_bad_shapes() {
    assert_eq!(
        parse_bbo(r#"{"unexpected": 1}"#, RECV, sc())
            .unwrap_err()
            .code(),
        "GateBadShape"
    );
    // 多出精度不接受静默截断（so-core ExcessPrecision 透传）
    assert_eq!(
        parse_obu(
            r#"{ "t": 1, "s": "BTC_USDT", "U": 1, "u": 2,
                 "b": [{"p": "54672.11", "s": "1"}], "a": [] }"#,
            RECV,
            sc()
        )
        .unwrap_err()
        .code(),
        "GateFixed"
    );
    // 精度不匹配但数字不同：54672.1 在 scale 1 合法；错误的是 0.005
    assert_eq!(
        parse_bbo(
            r#"{ "t": 1, "u": 1, "s": "X", "b": "0.005", "B": "1", "a": "1", "A": "1" }"#,
            RECV,
            sc()
        )
        .unwrap_err()
        .code(),
        "GateFixed"
    );
}

#[test]
fn funding_rate_helper() {
    assert_eq!(so_adapter_gate::parse_funding("0.01875"), Ok(1_875_000));
    assert_eq!(so_adapter_gate::parse_funding("-0.000114"), Ok(-11_400));
    assert!(so_adapter_gate::parse_funding("abc").is_err());
}

#[allow(dead_code)]
fn _gate_error_codes_are_stable(e: GateError) -> &'static str {
    e.code()
}
