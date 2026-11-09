//! 单次连接会话：连接 → 订阅全部 频道×品种 → 收帧循环。
//!
//! 重连与退避由 [`crate::run::run_connection`] 负责；本模块只跑一次会话，
//! 任何结束（断线/升级/静默错误）都返回 `Err` 交给上层退避。

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::config::{RecordingConfig, VenueConfig};
use crate::error::RecorderError;
use crate::frame::{handle_frame, handle_silence_tick};
use crate::journal::JournalWriter;
use crate::run::SharedState;
use crate::timefmt::now_nanos;
use crate::ws::{RequestFrame, ping_frame};

/// 一次连接会话：连接 → 订阅 → 收帧循环（不返回 `Ok`，直到对端关闭）。
pub(crate) async fn run_connection_once(
    config: &RecordingConfig,
    venue: &VenueConfig,
    data_dir: &Path,
    conn_id: &str,
    shared: &Arc<Mutex<SharedState>>,
) -> Result<(), RecorderError> {
    let mut journal = JournalWriter::new(data_dir, &venue.venue, conn_id)?;
    let (ws, _resp) = tokio_tungstenite::connect_async(&venue.ws_url)
        .await
        .map_err(|e| RecorderError::Ws(format!("连接失败：{e}")))?;
    let mut ws: WebSocketStream<MaybeTlsStream<TcpStream>> = ws;

    // 逐个订阅（docs/venues/gate.md §2.1；订阅帧 time 秒级）。
    let now_secs = now_nanos().0 / 1_000_000_000;
    let mut req_id: u64 = 1;
    for ch in &config.channels {
        for sym in &venue.symbols {
            let mut payload = vec![sym.clone()];
            payload.extend(ch.params.iter().cloned());
            let frame = RequestFrame::subscribe(now_secs, req_id, &ch.channel, payload);
            let text = frame.to_json().map_err(RecorderError::Ws)?;
            ws.send(Message::Text(text.into()))
                .await
                .map_err(|e| RecorderError::Ws(format!("发送订阅失败：{e}")))?;
            req_id = req_id.saturating_add(1);
        }
    }

    // 收帧循环：静默检测 tick 1s；应用层 ping 20s（服务器 60s 无活动会断）。
    let mut silence_tick = interval(Duration::from_secs(1));
    let mut ping_tick = interval(Duration::from_secs(20));
    silence_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    ping_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = silence_tick.tick() => {
                handle_silence_tick(shared)?;
            }
            _ = ping_tick.tick() => {
                let t = now_nanos().0 / 1_000_000_000;
                let _ = ws.send(Message::Text(ping_frame(t).into())).await;
            }
            msg = ws.next() => {
                let Some(msg) = msg else {
                    return Err(RecorderError::Ws("连接关闭".into()));
                };
                let text = match msg.map_err(|e| RecorderError::Ws(format!("接收失败：{e}")))? {
                    Message::Text(t) => t.to_string(),
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => continue,
                    Message::Close(_) => return Err(RecorderError::Ws("对端关闭".into())),
                    _ => continue,
                };
                // ts_recv 第一时间打点（docs/06 §3.2 A2）。
                let ts_recv = now_nanos();
                handle_frame(&mut journal, shared, conn_id, &text, ts_recv)?;
            }
        }
    }
}
