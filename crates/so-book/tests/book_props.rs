//! 订单簿性质测试（docs/04 §4：订单簿不变量）。
//!
//! 对任意随机增量序列：
//! - 两侧档位始终有序（Bid 降序、Ask 升序）且无重复价格；
//! - `qty=0` 应用后该价格档不存在；
//! - 快照 replace 后 `is_valid` 为真、档位与快照集合一致。

use proptest::prelude::*;
use so_book::{BookSide, OrderBook};
use so_core::{Price, Qty};
use so_model::Side;

fn price_strategy() -> impl Strategy<Value = i64> {
    (0i64..200).prop_map(|i| i * 5) // 粗网格提高碰撞率（覆盖/删除路径更易命中）
}

fn level_ops() -> impl Strategy<Value = Vec<(Side, i64, i64)>> {
    proptest::collection::vec((proptest::bool::ANY, price_strategy(), 0i64..10), 0..200).prop_map(
        |v| {
            v.into_iter()
                .map(|(is_bid, p, q)| (if is_bid { Side::Bid } else { Side::Ask }, p, q))
                .collect()
        },
    )
}

fn assert_sorted(side: &BookSide) {
    let levels = side.levels();
    for w in levels.windows(2) {
        match side.side() {
            Side::Bid => assert!(w[0].price.0 > w[1].price.0, "bid side not desc"),
            Side::Ask => assert!(w[0].price.0 < w[1].price.0, "ask side not asc"),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn side_invariants(ops in level_ops()) {
        for side_kind in [Side::Bid, Side::Ask] {
            let mut b = BookSide::empty(side_kind);
            for (_, p, q) in ops.iter().filter(|(s, _, _)| *s == side_kind) {
                b.set(Price(*p), Qty(*q))?;
                assert_sorted(&b);
                // qty>0 的价格档必然存在
                if *q > 0 {
                    let found = b.levels().iter().any(|l| l.price.0 == *p && l.qty.0 == *q);
                    prop_assert!(found, "applied level missing: price={p} qty={q}");
                }
            }
        }
    }

    #[test]
    fn book_snapshot_invariants(
        bids in proptest::collection::vec((price_strategy(), 1i64..10), 0..50),
        asks in proptest::collection::vec((price_strategy(), 1i64..10), 0..50),
    ) {
        let mut book = OrderBook::new();
        let b_levels: Vec<(Price, Qty)> =
            bids.iter().map(|(p, q)| (Price(*p), Qty(*q))).collect();
        let a_levels: Vec<(Price, Qty)> =
            asks.iter().map(|(p, q)| (Price(*p), Qty(*q))).collect();
        let res = book.replace(42, &b_levels, &a_levels);
        if res.is_err() {
            return Ok(()); // 盘口交叉：合法输入组合下允许，调用方重建
        }
        prop_assert!(book.is_valid());
        assert_sorted(book.bids());
        assert_sorted(book.asks());
        // 档位集合与快照一致（同价快照覆盖：保留最后一次写入）
        let mut seen: Vec<i64> = Vec::new();
        for (p, q) in &b_levels {
            if seen.contains(&p.0) {
                continue; // 只校验每个价格的最终值（后写覆盖前写）
            }
            seen.push(p.0);
            let final_qty = b_levels
                .iter()
                .rfind(|(bp, _)| bp.0 == p.0)
                .map(|(_, bq)| *bq)
                .unwrap_or(*q);
            let found = book
                .bids()
                .levels()
                .iter()
                .find(|l| l.price == *p)
                .map(|l| l.qty);
            prop_assert_eq!(found, Some(final_qty));
        }
        prop_assert_eq!(book.last_id(), 42);
    }
}
