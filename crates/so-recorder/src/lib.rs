//! so-recorder：行情录制（docs/06 §3）。
//!
//! 结构：`config`（录制配置）→ `journal`（Raw Journal 行模型与滚动）
//! → `ws`（协议层：订阅、心跳、重连参数）→ `dedup`（A/B 流去重）
//! → `silence`（静默断流检测）→ `alert`（Telegram 告警）
//! → `snapshot`（REST 定时快照）→ `connection`（单次会话）
//! → `frame`（帧处理）→ `run`（组装）。`timefmt` 是 UTC 日期格式化与 ts_recv 时钟。
//!
//! 本 crate 是**冷路径进程**（VPS 独立运行，docs/06 §3.2 A8），可以用 tokio。
//! 行情解析复用 so-adapter-gate。订单簿重建验证（docs/06 C9）已归位
//! `so-data::bookcheck`（随清洗链路走，ADR-0016 下沉手法）。
//!
//! 本模块树是录制进程的实现；`book-check` 离线分析工具见
//! `crates/so-data/examples/`。

pub mod alert;
pub mod config;
pub mod connection;
pub mod download;
pub mod download_run;
pub mod error;
pub mod frame;
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
// Journal 行模型、写入器、去重、sha256、日期格式化都由 `so-journal` 提供
// （层2 叶子 crate，ADR-0016）。这里重新导出，让本 crate 内的
// `crate::journal::…` 调用点保持可读，且不重复实现文件格式。
pub use so_journal::{JournalRow, JournalStats, JournalWriter, sha256_file};

/// Journal 相关类型的兼容路径（crate 内调用点沿用 `crate::journal::X`）。
pub mod journal {
    pub use so_journal::{JournalRow, JournalStats, JournalWriter};
}

/// A/B 去重（crate 内调用点沿用 `crate::dedup::X`）。
pub mod dedup {
    pub use so_journal::{Dedup, DedupKey, dedup_key_from_frame};
}

/// 文件校验和（crate 内调用点沿用 `crate::hash::X`）。
pub mod hash {
    pub use so_journal::sha256_file;
}
pub use run::run_recorder;
