//! so-data：数据层（docs/06）。
//!
//! 结构：`error`（DataError）→ `journal_in`（Raw Journal 读取）→
//! `contracts`（品种元数据）→ `normalize`（统一事件）→ `parquet_out`
//! （Parquet 写出）→ `pipeline`（按日端到端编排）。
//! 存储布局与数据集哈希见 docs/06 §6–§7。
//!
//! 本 crate 是**冷路径**（本地批处理工具），允许堆分配与阻塞 I/O。

pub mod contracts;
pub mod error;
pub mod instrument_out;
pub mod journal_in;
pub mod normalize;
pub mod parquet_out;
pub mod pipeline;

pub use error::DataError;
