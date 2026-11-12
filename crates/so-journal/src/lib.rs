//! Raw Journal 的帧格式与读写（docs/06 §3.1、§3.2 A1/A3/A9）。
//!
//! 本 crate 是**冷路径叶子 crate**：只做「一行一条 JSONL 帧」的模型、按小时滚动的
//! zstd 多帧写入、A/B 双流去重与文件校验和。录制端（`so-recorder`）与数据端
//! （`so-data`）都依赖它，两者互不依赖（docs/04 A1，ADR-0016）。
//!
//! 设计约束：**不引入网络、异步、数据库依赖**。Journal 是纯文件格式；
//! 一旦这里出现 tokio/reqwest，抽取它的理由（避免数据层背采集进程的依赖图）就失效了。

pub mod date;
pub mod dedup;
pub mod error;
pub mod hash;
pub mod row;
pub mod writer;

pub use date::{format_utc_date, format_utc_time};
pub use dedup::{Dedup, DedupKey, dedup_key_from_frame};
pub use error::JournalError;
pub use hash::sha256_file;
pub use row::JournalRow;
pub use writer::{JournalStats, JournalWriter};
