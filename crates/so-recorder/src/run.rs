//! 录制主循环组装：双连接编排与重连退避（docs/06 §3.1/§3.2）。
//!
//! `run_recorder` 为 A/B 两条连接各 spawn 一个 `run_connection`，
//! 共享一个去重器与静默检测（`Arc<Mutex>`，冷路径允许）。
//! 单次会话的收帧细节见 [`crate::connection`]，帧处理见 [`crate::frame`]。

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::{Instant, sleep_until};

use crate::config::{RecordingConfig, VenueConfig};
use crate::connection::run_connection_once;
use crate::dedup::Dedup;
use crate::error::RecorderError;
use crate::silence::SilenceDetector;
use crate::timefmt::now_nanos;

/// 共享状态：A/B 流共用的去重器与静默检测。
#[derive(Debug, Default)]
pub struct SharedState {
    /// 去重器（docs/06 §3.2 A3）。
    pub dedup: Dedup,
    /// 静默检测（docs/06 §3.2 A5）。
    pub silence: SilenceDetector,
}

/// 运行整个录制器（阻塞直到 ctrl-c）。
///
/// A/B 双连接各一个任务；`dual_conn=false` 时只跑 A。
pub async fn run_recorder(
    config: &RecordingConfig,
    venue: &VenueConfig,
) -> Result<(), RecorderError> {
    let data_dir = Path::new(&config.data_dir);
    let shared = Arc::new(Mutex::new(SharedState::default()));

    // 注册静默监控（全部 频道×品种 组合）。
    {
        let mut st = shared
            .lock()
            .map_err(|_| RecorderError::Ws("共享状态锁中毒".into()))?;
        for ch in &config.channels {
            for sym in &venue.symbols {
                st.silence.watch(
                    &ch.channel,
                    sym,
                    (ch.max_silence_secs as i64) * 1_000_000_000,
                    now_nanos().0,
                );
            }
        }
    }

    let mut handles = Vec::new();
    let conn_ids: &[&str] = if venue.dual_conn { &["A", "B"] } else { &["A"] };
    for conn_id in conn_ids.iter().copied() {
        let cfg = config.clone();
        let venue_cfg = venue.clone();
        let shared = Arc::clone(&shared);
        let data_dir = data_dir.to_owned();
        let conn = String::from(conn_id);
        handles.push(tokio::spawn(async move {
            run_connection(&cfg, &venue_cfg, &data_dir, &conn, shared).await
        }));
    }

    // 等待 ctrl-c，然后等待任务结束。
    let _ = tokio::signal::ctrl_c().await;
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

/// 一条连接的完整生命周期（含无限重连）。
pub async fn run_connection(
    config: &RecordingConfig,
    venue: &VenueConfig,
    data_dir: &Path,
    conn_id: &str,
    shared: Arc<Mutex<SharedState>>,
) -> Result<(), RecorderError> {
    let mut attempt: u32 = 0;
    loop {
        match run_connection_once(config, venue, data_dir, conn_id, &shared).await {
            Ok(()) => {
                // 正常返回只有 ctrl-c 关闭，退出循环。
                return Ok(());
            }
            Err(e) => {
                let code = e.code();
                tracing::warn!(
                    code = code,
                    conn = conn_id,
                    attempt = attempt,
                    "连接结束，退避后重连"
                );
            }
        }
        attempt = attempt.saturating_add(1);
        let backoff = crate::ws::backoff_ms(attempt);
        sleep_until(Instant::now() + Duration::from_millis(backoff)).await;
    }
}
