//! Raw Journal 行模型与滚动写入（docs/06 §3.1）。
//!
//! 布局：`{data_dir}/{venue}/{YYYY-MM-DD}/{HH}.{conn}.journal.zst` + `.sha256`。
//! 每行一条 JSONL：原始帧原文 + `ts_recv` + 连接 ID + 频道 + 品种。
//! **先存原始帧，再解析**（docs/06 §3.2 A1）；解析失败只记日志，帧已落盘。

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use so_core::UnixNanos;

use crate::error::RecorderError;
use crate::timefmt::format_utc_date;

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
    pub fn to_jsonl(&self) -> Result<String, RecorderError> {
        let mut s = serde_json::to_string(self)
            .map_err(|e| RecorderError::Journal(format!("序列化 Journal 行失败：{e}")))?;
        s.push('\n');
        Ok(s)
    }
}

/// Journal 滚动写入器：一个文件对应一小时 + 一条连接。
///
/// 手动实现 `Debug`：zstd 编码器无 Debug 实现，只暴露路径与计数。
pub struct JournalWriter {
    dir: PathBuf,
    conn_id: String,
    /// 当前小时索引（Unix 纳秒 / 3.6e12）。
    hour_start: i64,
    writer: Option<zstd::stream::write::Encoder<'static, BufWriter<File>>>,
    /// 已写入行数（当前文件）。
    rows: u64,
    /// 全部已写入行数（本进程生命周期）。
    total_rows: u64,
    /// 全部已写入字节（未压缩，当前文件）。
    bytes: u64,
}

impl std::fmt::Debug for JournalWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JournalWriter")
            .field("dir", &self.dir)
            .field("conn_id", &self.conn_id)
            .field("hour_start", &self.hour_start)
            .field("rows", &self.rows)
            .field("total_rows", &self.total_rows)
            .field("bytes", &self.bytes)
            .finish_non_exhaustive()
    }
}

impl JournalWriter {
    /// 在 `data_dir/{venue}/` 下创建写入器。
    pub fn new(data_dir: &Path, venue: &str, conn_id: &str) -> Result<Self, RecorderError> {
        let dir = data_dir.join(venue);
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            conn_id: conn_id.to_owned(),
            hour_start: 0,
            writer: None,
            rows: 0,
            total_rows: 0,
            bytes: 0,
        })
    }

    /// 写入一行；跨小时自动滚动。
    pub fn write(&mut self, row: &JournalRow) -> Result<(), RecorderError> {
        let hour = row.ts_recv / 3_600_000_000_000;
        if self.writer.is_none() || hour != self.hour_start {
            self.roll(hour)?;
        }
        let line = row.to_jsonl()?;
        let Some(enc) = self.writer.as_mut() else {
            return Err(RecorderError::Journal("写入器未初始化".to_owned()));
        };
        enc.write_all(line.as_bytes())
            .map_err(|e| RecorderError::Journal(format!("写入 Journal 失败：{e}")))?;
        self.rows = self.rows.saturating_add(1);
        self.total_rows = self.total_rows.saturating_add(1);
        self.bytes = self.bytes.saturating_add(line.len() as u64);
        Ok(())
    }

    /// 滚动到新小时（关闭旧文件 + 打开新文件）。
    pub fn roll(&mut self, new_hour: i64) -> Result<(), RecorderError> {
        self.finish_current()?;
        self.hour_start = new_hour;
        let path = self.path_for(new_hour);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| RecorderError::Journal(format!("打开 Journal 失败：{e}")))?;
        let buf = BufWriter::new(file);
        let enc = zstd::stream::write::Encoder::new(buf, 3)
            .map_err(|e| RecorderError::Journal(format!("zstd 初始化失败：{e}")))?;
        self.writer = Some(enc);
        self.rows = 0;
        self.bytes = 0;
        Ok(())
    }

    /// 优雅关闭（进程退出前调用）。
    pub fn finish(&mut self) -> Result<(), RecorderError> {
        self.finish_current()
    }

    /// 当前小时的文件路径。
    pub fn path_for(&self, hour: i64) -> PathBuf {
        let secs = hour.saturating_mul(3600);
        let date = format_utc_date(secs);
        let hh = (secs % 86_400) / 3600;
        self.dir
            .join(date)
            .join(format!("{hh:02}.{}.journal.zst", self.conn_id))
    }

    /// 当前文件统计。
    pub fn stats(&self) -> JournalStats {
        JournalStats {
            rows: self.rows,
            total_rows: self.total_rows,
            bytes: self.bytes,
        }
    }

    /// 关闭当前文件并写 sha256 边车（docs/06 §3.2 A9）。
    ///
    /// 哈希覆盖**整个文件**（含此前重连会话追加的 zstd 帧），
    /// 保证同步端可以按边车校验完整内容。
    fn finish_current(&mut self) -> Result<(), RecorderError> {
        let Some(enc) = self.writer.take() else {
            return Ok(());
        };
        enc.finish()
            .map_err(|e| RecorderError::Journal(format!("zstd 收尾失败：{e}")))?;
        let path = self.path_for(self.hour_start);
        let hex = crate::hash::sha256_file(&path)?;
        let sha_path = path.with_extension("zst.sha256");
        fs::write(&sha_path, format!("{hex}  {}\n", path.display()))
            .map_err(|e| RecorderError::Journal(format!("写 sha256 失败：{e}")))?;
        Ok(())
    }
}

/// Journal 写入统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JournalStats {
    /// 当前文件行数。
    pub rows: u64,
    /// 本进程累计行数。
    pub total_rows: u64,
    /// 当前文件未压缩字节数。
    pub bytes: u64,
}

impl Drop for JournalWriter {
    fn drop(&mut self) {
        // best-effort：Drop 中忽略错误。
        let _ = self.finish_current();
    }
}
