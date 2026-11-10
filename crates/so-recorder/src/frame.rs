//! 帧处理：Journal 落盘 → A/B 去重 → 静默刷新；静默 tick 与告警日志。
//!
//! 处理顺序固定为**先存原始帧，再解析**（docs/06 §3.2 A1）：
//! 解析失败只记日志，帧已落盘。

use std::sync::{Arc, Mutex};

use so_core::UnixNanos;

use crate::alert::{AlertHandle, silence_alert_text};
use crate::dedup::dedup_key_from_frame;
use crate::error::RecorderError;
use crate::journal::{JournalRow, JournalWriter};
use crate::run::SharedState;
use crate::silence::SilenceEvent;
use crate::timefmt::now_nanos;
use crate::ws::{EnvelopeFrame, FrameKind, is_pong, is_upgrade_notice};

/// 处理一帧；升级通知返回错误触发重连。
pub(crate) fn handle_frame(
    journal: &mut JournalWriter,
    shared: &Arc<Mutex<SharedState>>,
    alert: &AlertHandle,
    conn_id: &str,
    text: &str,
    ts_recv: UnixNanos,
) -> Result<(), RecorderError> {
    let Some(env) = EnvelopeFrame::parse(text) else {
        return Ok(()); // 协议外帧（pong 等），不入 Journal。
    };
    if is_pong(&env) {
        return Ok(());
    }
    // 服务升级通知 → 立即重连（docs/venues/gate.md §2.3）。
    if is_upgrade_notice(&env) {
        return Err(RecorderError::Ws("服务升级通知，主动重连".into()));
    }
    match env.kind() {
        FrameKind::Update => {
            // 数据帧：品种从 result 提取（Gate 各频道字段名不同，浅解析）。
            let symbol = extract_symbol(&env).unwrap_or_else(|| "*".to_owned());
            // 先落盘（A1：先存原始帧，再解析）。
            journal.write(&JournalRow::new(
                ts_recv,
                &env.channel,
                symbol.as_str(),
                conn_id,
                text,
            ))?;
            let mut st = shared
                .lock()
                .map_err(|_| RecorderError::Ws("共享状态锁中毒".into()))?;
            // 去重（A/B 流）。
            let key = dedup_key_from_frame(&env.channel, &symbol, text);
            if st.dedup.first_seen(&key) {
                // 首见：刷新静默计时（重复帧不刷新——另一条连接活着不代表本连接活着）。
                let events = st.silence.on_message(&env.channel, &symbol, ts_recv.0);
                log_silence_events(&events, alert);
            }
            Ok(())
        }
        FrameKind::Error(e) => {
            tracing::error!(code = e.code, channel = %env.channel, "订阅失败");
            Ok(()) // 订阅失败不致命：其余频道继续录制。
        }
        _ => Ok(()), // 确认帧不入 Journal。
    }
}

/// 静默 tick：检测 + 日志 + Telegram 告警。
pub(crate) fn handle_silence_tick(
    shared: &Arc<Mutex<SharedState>>,
    alert: &AlertHandle,
) -> Result<(), RecorderError> {
    let now = now_nanos().0;
    let mut st = shared
        .lock()
        .map_err(|_| RecorderError::Ws("共享状态锁中毒".into()))?;
    let events = st.silence.check(now);
    log_silence_events(&events, alert);
    Ok(())
}

/// 静默事件 → 日志 + Telegram 告警（告警由后台任务发送，这里只入队）。
pub(crate) fn log_silence_events(events: &[SilenceEvent], alert: &AlertHandle) {
    for ev in events {
        match ev {
            SilenceEvent::WentSilent(k) => {
                tracing::error!(channel = %k.channel, symbol = %k.symbol, "静默断流（A5）");
            }
            SilenceEvent::Recovered(k) => {
                tracing::info!(channel = %k.channel, symbol = %k.symbol, "断流恢复");
            }
        }
        alert.send(&silence_alert_text(ev));
    }
}

/// 从信封 result 提取品种（Gate：合约频道 `s`/`contract`，现货 `currency_pair`）。
fn extract_symbol(env: &EnvelopeFrame) -> Option<String> {
    let r = env.result.as_ref()?;
    if let Some(s) = r.get("s").and_then(|v| v.as_str()) {
        return Some(s.to_owned());
    }
    if let Some(s) = r.get("contract").and_then(|v| v.as_str()) {
        return Some(s.to_owned());
    }
    if let Some(s) = r.get("currency_pair").and_then(|v| v.as_str()) {
        return Some(s.to_owned());
    }
    // 数组体（trades）：取首元素的字段。
    if let Some(arr) = r.as_array()
        && let Some(first) = arr.first()
        && let Some(s) = first.get("contract").and_then(|v| v.as_str())
    {
        return Some(s.to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("so-recorder-frame-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// 禁用告警句柄（测试不需要 Telegram）。
    fn no_alert() -> AlertHandle {
        AlertHandle::disabled()
    }

    #[test]
    fn update_frame_journaled_and_deduped() {
        let dir = temp_dir("frame");
        let mut journal = JournalWriter::new(&dir, "gate", "A").unwrap();
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let frame = r#"{"time":1,"channel":"futures.trades","event":"update","error":null,"result":[{"id":27753479,"contract":"BTC_USDT"}]}"#;
        let ts = UnixNanos(1_700_000_000_000_000_000);
        handle_frame(&mut journal, &shared, &no_alert(), "A", frame, ts).unwrap();
        assert_eq!(journal.stats().total_rows, 1);
        // 同帧再处理：Journal 记录（原始帧不可变，两连接都落盘），
        // 但去重器只记一次。
        handle_frame(&mut journal, &shared, &no_alert(), "A", frame, ts).unwrap();
        assert_eq!(journal.stats().total_rows, 2);
        {
            let st = shared.lock().unwrap();
            assert_eq!(st.dedup.len(), 1);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn subscribe_ack_not_journaled() {
        let dir = temp_dir("ack");
        let mut journal = JournalWriter::new(&dir, "gate", "A").unwrap();
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let frame = r#"{"time":1,"channel":"futures.trades","event":"subscribe","error":null,"result":{"status":"success"}}"#;
        handle_frame(&mut journal, &shared, &no_alert(), "A", frame, UnixNanos(1)).unwrap();
        assert_eq!(journal.stats().total_rows, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upgrade_notice_triggers_reconnect_error() {
        let dir = temp_dir("upgrade");
        let mut journal = JournalWriter::new(&dir, "gate", "A").unwrap();
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let frame =
            r#"{"time":1,"channel":"futures.system","event":"update","result":{"type":"upgrade"}}"#;
        let err =
            handle_frame(&mut journal, &shared, &no_alert(), "A", frame, UnixNanos(1)).unwrap_err();
        assert_eq!(err.code(), "RecorderWs");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn symbol_extraction() {
        let env = EnvelopeFrame::parse(
            r#"{"channel":"futures.order_book_update","event":"update","result":{"s":"BTC_USDT"}}"#,
        )
        .unwrap();
        assert_eq!(extract_symbol(&env).as_deref(), Some("BTC_USDT"));
        let env = EnvelopeFrame::parse(
            r#"{"channel":"futures.trades","event":"update","result":[{"contract":"ETH_USDT"}]}"#,
        )
        .unwrap();
        assert_eq!(extract_symbol(&env).as_deref(), Some("ETH_USDT"));
    }
}
