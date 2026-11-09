//! so-recorder：行情录制（docs/06 §3）。
//!
//! 结构：`config`（录制配置）→ `journal`（Raw Journal 行模型与滚动）
//! → `ws`（连接层：订阅、心跳、重连）→ `dedup`（A/B 流去重）
//! → `silence`（静默断流检测）→ `snapshot`（REST 定时快照）→ `run`（组装）。
//!
//! 本 crate 是**冷路径进程**（VPS 独立运行，docs/06 §3.2 A8），可以用 tokio。
//! 行情解析复用 so-adapter-gate；订单簿重建复用 so-book。

pub mod config;
pub mod dedup;
pub mod error;
pub mod journal;
pub mod silence;
pub mod snapshot;
pub mod ws;

pub use config::RecordingConfig;
pub use error::RecorderError;
pub use journal::{JournalRow, JournalStats, JournalWriter};
