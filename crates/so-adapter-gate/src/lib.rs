//! Gate 交易所适配器（入口文件：只做模块声明与重新导出）。
//!
//! M1 范围：**合约（USDT 永续）公共行情解析**——深度增量、BBO、逐笔、
//! 标记价/资金费（docs/05 M1）。消息格式依据 docs/venues/gate.md（快照
//! docs/gate-api-v4.106.179-markdown/）。
//!
//! 解析函数属**热路径**：不分配堆内存（输出写入调用方提供的缓冲）、
//! 不 panic、错误返回 `GateError`。连接层（tokio-tungstenite、重连、
//! 双流去重）在 so-recorder 阶段实现，不在本 crate。

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

pub mod error;
pub mod msg;
pub mod parse;
pub mod rest;
pub mod spot;
pub mod types;

pub use error::GateError;
pub use parse::{parse_bbo, parse_funding, parse_obu, parse_ticker, parse_trade};
pub use rest::{RawSnapshot, parse_contracts, parse_order_book_rest};
pub use types::{ObuLevel, RawBbo, RawObu, RawTicker, RawTrade};
