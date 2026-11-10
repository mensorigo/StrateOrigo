//! so-recorder：行情录制（docs/06 §3）。
//!
//! 结构：`config`（录制配置）→ `journal`（Raw Journal 行模型与滚动）
//! → `ws`（协议层：订阅、心跳、重连参数）→ `dedup`（A/B 流去重）
//! → `silence`（静默断流检测）→ `alert`（Telegram 告警）
//! → `snapshot`（REST 定时快照）→ `connection`（单次会话）→ `frame`（帧处理）
//! → `run`（组装）。`timefmt` 是 UTC 日期格式化与 ts_recv 时钟。
//!
//! 本 crate 是**冷路径进程**（VPS 独立运行，docs/06 §3.2 A8），可以用 tokio。
//! 行情解析复用 so-adapter-gate；订单簿重建复用 so-book。

pub mod alert;
pub mod config;
pub mod connection;
pub mod dedup;
pub mod download;
pub mod download_run;
pub mod error;
pub mod frame;
pub mod hash;
pub mod journal;
pub mod run;
pub mod silence;
pub mod snapshot;
pub mod snapshot_pull;
pub mod timefmt;
pub mod ws;

pub use alert::AlertHandle;
pub use config::RecordingConfig;
pub use download::DownloadChannel;
pub use download_run::download_history;
pub use error::RecorderError;
pub use journal::{JournalRow, JournalStats, JournalWriter};
pub use run::run_recorder;
