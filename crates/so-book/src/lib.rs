//! L1/L2 订单簿（docs/03 §5）。
//!
//! 排序数组 + 二分查找实现，预分配固定容量、零运行时分配。
//! 序列校验按交易所规则由调用方封装（Gate U/u 见 [`book`]）；
//! 缺口时订单簿作废，调用方发 `FeedStatus::Gap` 并重新快照。

pub mod book;
pub mod error;
pub mod gate;
pub mod side;

pub use book::OrderBook;
pub use error::BookError;
pub use gate::{GateBookBuilder, GateSequencer};
pub use side::{BookSide, Level};
