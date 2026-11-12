//! Journal 滚动写入器（docs/06 §3.1）。
//!
//! 布局：`{data_dir}/{venue}/{YYYY-MM-DD}/{HH}.{conn}.journal.zst` + `.sha256`。
//! 一个文件对应一小时 + 一条连接；跨小时自动滚动。
//! 优雅关闭时写 sha256 边车，覆盖**整个文件**（含此前重连会话追加的 zstd 帧），
//! 保证同步端可以按边车校验完整内容（docs/06 §3.2 A9）。

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::date::format_utc_date;
use crate::error::JournalError;
use crate::row::JournalRow;

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
    pub fn new(data_dir: &Path, venue: &str, conn_id: &str) -> Result<Self, JournalError> {
        let dir = data_dir.join(venue);
        fs::create_dir_all(&dir).map_err(|e| JournalError::Open {
            path: dir.display().to_string(),
            source: e,
        })?;
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
    pub fn write(&mut self, row: &JournalRow) -> Result<(), JournalError> {
        let hour = row.ts_recv / 3_600_000_000_000;
        if self.writer.is_none() || hour != self.hour_start {
            self.roll(hour)?;
        }
        let line = row.to_jsonl()?;
        let Some(enc) = self.writer.as_mut() else {
            return Err(JournalError::NotRolled);
        };
        enc.write_all(line.as_bytes())
            .map_err(|e| JournalError::Write {
                path: self.path_for(self.hour_start).display().to_string(),
                bytes: line.len() as u64,
                source: e,
            })?;
        self.rows = self.rows.saturating_add(1);
        self.total_rows = self.total_rows.saturating_add(1);
        self.bytes = self.bytes.saturating_add(line.len() as u64);
        Ok(())
    }

    /// 滚动到新小时（关闭旧文件 + 打开新文件）。
    pub fn roll(&mut self, new_hour: i64) -> Result<(), JournalError> {
        self.finish_current()?;
        self.hour_start = new_hour;
        let path = self.path_for(new_hour);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| JournalError::Open {
                path: parent.display().to_string(),
                source: e,
            })?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| JournalError::Open {
                path: path.display().to_string(),
                source: e,
            })?;
        let buf = BufWriter::new(file);
        let enc = zstd::stream::write::Encoder::new(buf, 3).map_err(|e| JournalError::Codec {
            stage: "init",
            source: e,
        })?;
        self.writer = Some(enc);
        self.rows = 0;
        self.bytes = 0;
        Ok(())
    }

    /// 优雅关闭（进程退出前调用）。
    pub fn finish(&mut self) -> Result<(), JournalError> {
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
    fn finish_current(&mut self) -> Result<(), JournalError> {
        let Some(enc) = self.writer.take() else {
            return Ok(());
        };
        enc.finish().map_err(|e| JournalError::Codec {
            stage: "finish",
            source: e,
        })?;
        let path = self.path_for(self.hour_start);
        let hex = crate::hash::sha256_file(&path)?;
        let sha_path = path.with_extension("zst.sha256");
        fs::write(&sha_path, format!("{hex}  {}\n", path.display())).map_err(|e| {
            JournalError::Write {
                path: sha_path.display().to_string(),
                bytes: 0,
                source: e,
            }
        })?;
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
        // best-effort：Drop 中忽略错误（进程退出路径上无处上报）。
        let _ = self.finish_current();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::UnixNanos;

    /// 一次写入 + 关闭：行可读、sha256 边车存在且覆盖整个文件。
    #[test]
    fn write_and_checksum_sidecar() {
        let dir = std::env::temp_dir().join("so-journal-writer");
        let _ = std::fs::remove_dir_all(&dir);
        let ts = 1_700_000_000_000_000_000_i64;
        let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
        w.write(&JournalRow::new(
            UnixNanos(ts),
            "futures.trades",
            "BTC_USDT",
            "A",
            "line1",
        ))
        .unwrap();
        w.finish().unwrap();

        let path = w.path_for(ts / 3_600_000_000_000);
        let sha_path = path.with_extension("zst.sha256");
        let sidecar = std::fs::read_to_string(&sha_path).unwrap();
        let want = crate::hash::sha256_file(&path).unwrap();
        assert!(sidecar.starts_with(&want), "边车哈希须等于文件实际哈希");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 重连追加：同一小时两次会话 = 两个 zstd 帧，边车覆盖整个文件。
    #[test]
    fn reconnect_appends_frames_and_hashes_whole_file() {
        let dir = std::env::temp_dir().join("so-journal-append");
        let _ = std::fs::remove_dir_all(&dir);
        let ts = 1_700_000_000_000_000_000_i64;
        let hour = ts / 3_600_000_000_000;

        let mut w1 = JournalWriter::new(&dir, "gate", "A").unwrap();
        w1.write(&JournalRow::new(UnixNanos(ts), "c", "S", "A", "line1"))
            .unwrap();
        w1.finish().unwrap();
        let after_first = crate::hash::sha256_file(&w1.path_for(hour)).unwrap();

        let mut w2 = JournalWriter::new(&dir, "gate", "A").unwrap();
        w2.roll(hour).unwrap();
        w2.write(&JournalRow::new(UnixNanos(ts + 1), "c", "S", "A", "line2"))
            .unwrap();
        w2.finish().unwrap();
        let after_second = crate::hash::sha256_file(&w2.path_for(hour)).unwrap();

        assert_ne!(after_first, after_second, "追加后哈希必须变化");
        let sidecar =
            std::fs::read_to_string(w2.path_for(hour).with_extension("zst.sha256")).unwrap();
        assert!(
            sidecar.starts_with(&after_second),
            "边车覆盖追加后的整个文件"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stats_track_rows() {
        let dir = std::env::temp_dir().join("so-journal-stats");
        let _ = std::fs::remove_dir_all(&dir);
        let ts = 1_700_000_000_000_000_000_i64;
        let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
        for i in 0..3 {
            w.write(&JournalRow::new(UnixNanos(ts + i), "c", "S", "A", "x"))
                .unwrap();
        }
        let s = w.stats();
        assert_eq!((s.rows, s.total_rows), (3, 3));
        assert!(s.bytes > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
