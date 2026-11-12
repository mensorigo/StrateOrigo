//! Journal 行模型（docs/06 §3.1）。
//!
//! 每行一条 JSONL：原始帧原文 + `ts_recv` + 连接 ID + 频道 + 品种。
//! **先存原始帧，再解析**（docs/06 §3.2 A1）；解析失败只记日志，帧已落盘。

use so_core::UnixNanos;

use crate::error::JournalError;

/// Journal 行：一条 WS 帧的录制记录。
///
/// `ts_recv` 以 i64 纳秒序列化（[`UnixNanos`] 未实现 serde）；
/// 读取方按同样约定反序列化（docs/06 §4.1 全表 i64 纳秒）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalRow {
    /// 本地接收时间（收到帧第一时间打点，docs/06 §3.2 A2）。
    pub ts_recv: i64,
    /// 频道名。
    pub channel: String,
    /// 品种（Gate 合约名）。
    pub symbol: String,
    /// 连接标识（A/B 流去重与延迟统计用，docs/06 §3.2 A3）。
    pub conn_id: String,
    /// 原始帧文本（WS 文本消息原样）。
    pub raw: String,
}

impl JournalRow {
    /// 构造一行。
    pub fn new(
        ts_recv: UnixNanos,
        channel: impl Into<String>,
        symbol: impl Into<String>,
        conn_id: impl Into<String>,
        raw: impl Into<String>,
    ) -> Self {
        Self {
            ts_recv: ts_recv.0,
            channel: channel.into(),
            symbol: symbol.into(),
            conn_id: conn_id.into(),
            raw: raw.into(),
        }
    }

    /// 序列化为 JSONL 行（含尾部换行）。
    pub fn to_jsonl(&self) -> Result<String, JournalError> {
        let mut s = serde_json::to_string(self).map_err(|e| JournalError::Row {
            stage: "encode",
            source: e,
        })?;
        s.push('\n');
        Ok(s)
    }

    /// 从一行 JSONL 反序列化（不含换行）。
    pub fn from_jsonl(line: &str) -> Result<Self, JournalError> {
        serde_json::from_str(line).map_err(|e| JournalError::Row {
            stage: "decode",
            source: e,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_roundtrip() {
        let row = JournalRow::new(
            UnixNanos(1_700_000_000_000_000_000),
            "futures.trades",
            "BTC_USDT",
            "A",
            r#"{"result":[]}"#,
        );
        let line = row.to_jsonl().unwrap();
        assert!(line.ends_with('\n'));
        let back = JournalRow::from_jsonl(line.trim_end()).unwrap();
        assert_eq!(back, row);
    }
}
