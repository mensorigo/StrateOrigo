//! 单侧订单档位数组：按价格排序的定长存储 + 二分查找。
//!
//! Bid 侧价格**降序**（最优在前），Ask 侧**升序**（最优在前）。
//! 所有操作无分配；满容量时 `set` 返回 `OverCapacity`（深档更新被拒即视为
//! 数据不完整，调用方应重建——正常行情下档位数远低于容量）。

use so_core::Price;
use so_model::Side;

use crate::error::BookError;

/// 单侧容量：Gate level ≤ 100、OKX books 400 档，1024 留足余量。
pub const SIDE_CAPACITY: usize = 1024;

/// 一档：价格 + 最新总量（0 = 该档不存在，删除后压缩）。
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Level {
    /// 档位价格。
    pub price: Price,
    /// 最新总量（绝对量，覆盖语义）。
    pub qty: so_core::Qty,
}

/// 单侧深度。`levels[..len]` 按价格有序：Bid 降序、Ask 升序。
#[derive(Copy, Clone, Debug)]
pub struct BookSide {
    side: Side,
    levels: [Level; SIDE_CAPACITY],
    len: usize,
}

impl BookSide {
    /// 空侧。
    pub const fn empty(side_kind: Side) -> Self {
        Self {
            side: side_kind,
            levels: [const {
                Level {
                    price: Price::ZERO,
                    qty: so_core::Qty::ZERO,
                }
            }; SIDE_CAPACITY],
            len: 0,
        }
    }

    /// 侧别。
    pub const fn side(&self) -> Side {
        self.side
    }

    /// 有效档位数。
    pub const fn len(&self) -> usize {
        self.len
    }

    /// 是否为空。
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 有序档位切片（最优在前）。
    pub fn levels(&self) -> &[Level] {
        self.levels.get(..self.len).unwrap_or(&[])
    }

    /// 最优档（买一/卖一）。
    pub fn best(&self) -> Option<Level> {
        self.levels().first().copied()
    }

    /// 第 `depth` 档（0 起）。
    pub fn level_at(&self, depth: usize) -> Option<Level> {
        self.levels().get(depth).copied()
    }

    /// 前 `depth` 档总量累加（docs/03 §5 指定深度累积量）。
    pub fn cumulative_qty(&self, depth: usize) -> so_core::Qty {
        let mut acc = so_core::Qty::ZERO;
        let end = self.len.min(depth);
        let Some(slice) = self.levels.get(..end) else {
            return acc;
        };
        for lv in slice {
            // 饱和累加：热路径不 panic，总量溢出只影响视图精度（docs/04 H8）。
            acc = so_core::Qty(acc.0.saturating_add(lv.qty.0));
        }
        acc
    }

    /// 二分查找价格档位；返回 `Ok(idx)`（命中）或 `Err(ins_point)`。
    fn search(&self, price: Price) -> Result<usize, usize> {
        // 统一为升序视角：Bid（降序存储）比较 lv vs price，
        // Ask（升序存储）比较 price vs lv——两种情况下
        // "Less ⇒ 插入点 ≤ mid、Greater ⇒ 插入点 > mid" 同构。
        let mut lo = 0usize;
        let mut hi = self.len;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let Some(lv) = self.levels.get(mid) else {
                return Err(self.len);
            };
            let cmp = match self.side {
                Side::Bid => lv.price.0.cmp(&price.0),
                Side::Ask => price.0.cmp(&lv.price.0),
            };
            match cmp {
                core::cmp::Ordering::Equal => return Ok(mid),
                core::cmp::Ordering::Less => hi = mid,
                core::cmp::Ordering::Greater => lo = mid.saturating_add(1),
            }
        }
        Err(lo)
    }

    /// 应用一档覆盖更新：设置/更新/删除该价格档位。
    pub fn set(&mut self, price: Price, qty: so_core::Qty) -> Result<(), BookError> {
        if qty.is_zero() {
            return self.remove(price);
        }
        match self.search(price) {
            Ok(idx) => {
                if let Some(lv) = self.levels.get_mut(idx) {
                    lv.qty = qty;
                }
                Ok(())
            }
            Err(pos) => {
                if self.len >= SIDE_CAPACITY {
                    return Err(BookError::OverCapacity(SIDE_CAPACITY));
                }
                // [..pos) 保留，[pos..len) 后移一格，[pos] 写入。
                // 切片取 [pos..len+1]（len+1 ≤ 容量已由上方检查保证），
                // 使 copy_within 的目的端 1 落在切片内。
                let n = self.len.saturating_add(1);
                let tail = self.levels.get_mut(pos..n).ok_or(
                    // pos ≤ len 恒成立；此分支不可达。
                    BookError::OverCapacity(SIDE_CAPACITY),
                )?;
                let tail_len = tail.len().saturating_sub(1);
                tail.copy_within(0..tail_len, 1);
                self.len = n;
                if let Some(lv) = self.levels.get_mut(pos) {
                    *lv = Level { price, qty };
                }
                Ok(())
            }
        }
    }

    /// 删除价格档位（不存在则幂等成功）。
    pub fn remove(&mut self, price: Price) -> Result<(), BookError> {
        if let Ok(idx) = self.search(price) {
            // [idx+1..len) 前移一格。
            let src = idx.saturating_add(1);
            if src < self.len {
                let tail_len = self.len - src;
                let tail = self
                    .levels
                    .get_mut(idx..self.len)
                    .ok_or(BookError::OverCapacity(SIDE_CAPACITY))?;
                tail.copy_within(1..1 + tail_len, 0);
            }
            self.len = self.len.saturating_sub(1);
        }
        Ok(())
    }

    /// 清空（快照重建 / 全量替换前调用）。
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// 从无序档位列表全量重建（快照应用路径）。
    pub fn load(&mut self, levels: &[(Price, so_core::Qty)]) -> Result<(), BookError> {
        self.clear();
        for (price, qty) in levels {
            self.set(*price, *qty)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::Qty;

    fn bids() -> BookSide {
        BookSide::empty(Side::Bid)
    }

    #[test]
    fn bid_side_stays_sorted_desc() {
        let mut b = bids();
        b.set(Price(100), Qty(1)).unwrap();
        b.set(Price(102), Qty(2)).unwrap();
        b.set(Price(101), Qty(3)).unwrap();
        let prices: Vec<i64> = b.levels().iter().map(|l| l.price.0).collect();
        assert_eq!(prices, vec![102, 101, 100]);
        assert_eq!(
            b.best(),
            Some(Level {
                price: Price(102),
                qty: Qty(2)
            })
        );
    }

    #[test]
    fn ask_side_stays_sorted_asc() {
        let mut a = BookSide::empty(Side::Ask);
        a.set(Price(105), Qty(1)).unwrap();
        a.set(Price(103), Qty(2)).unwrap();
        a.set(Price(104), Qty(3)).unwrap();
        let prices: Vec<i64> = a.levels().iter().map(|l| l.price.0).collect();
        assert_eq!(prices, vec![103, 104, 105]);
    }

    #[test]
    fn overwrite_and_remove() {
        let mut b = bids();
        b.set(Price(100), Qty(1)).unwrap();
        b.set(Price(100), Qty(5)).unwrap(); // 覆盖
        assert_eq!(b.best().unwrap().qty, Qty(5));
        b.remove(Price(100)).unwrap();
        assert!(b.is_empty());
        b.remove(Price(100)).unwrap(); // 幂等
    }

    #[test]
    fn qty_zero_removes_level() {
        let mut b = bids();
        b.set(Price(100), Qty(1)).unwrap();
        b.set(Price(100), Qty::ZERO).unwrap(); // 删档语义
        assert!(b.is_empty());
    }

    #[test]
    fn cumulative_qty_depth() {
        let mut b = bids();
        b.set(Price(100), Qty(1)).unwrap();
        b.set(Price(99), Qty(2)).unwrap();
        b.set(Price(98), Qty(4)).unwrap();
        assert_eq!(b.cumulative_qty(0), Qty(0));
        assert_eq!(b.cumulative_qty(1), Qty(1));
        assert_eq!(b.cumulative_qty(2), Qty(3));
        assert_eq!(b.cumulative_qty(10), Qty(7));
    }

    #[test]
    fn load_replaces_contents() {
        let mut b = bids();
        b.set(Price(50), Qty(9)).unwrap();
        b.load(&[(Price(100), Qty(1)), (Price(99), Qty(2))])
            .unwrap();
        assert_eq!(b.len(), 2);
        assert_eq!(b.best().unwrap().price, Price(100));
    }
}
