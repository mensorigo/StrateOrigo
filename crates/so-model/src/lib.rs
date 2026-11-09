//! 领域模型（入口文件：只做模块声明与重新导出，docs/04 F4）。
//!
//! 本 crate 属于**热路径**：事件结构全部为固定大小、`Copy` 或零分配借用，
//! 不含 `String`/`Vec` 字段——变长数据（深度档位数组）由 so-book 的预分配缓冲承接。

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
    )
)]

pub mod event;
pub mod market;
pub mod symbol;

pub use event::Event;
pub use market::{Bbo, BookDelta, FeedStatus, Funding, MarkPrice, Side, Trade};
pub use symbol::{Symbol, SymbolError};
