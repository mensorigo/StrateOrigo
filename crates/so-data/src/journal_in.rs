//! Raw Journal 读取（docs/06 §3.1）：zstd 多帧 + JSONL → [`JournalRow`]。
//!
//! **流式解码**：zstd 解码器直接包在 `File` 上逐块拉取，压缩字节与解压
//! 后的文本都不整体驻留内存——按天运行时内存占用 ≈ 行缓冲 + 行集合
//! （docs/06 §6：一天一文件，大小与小时文件数线性）。
//!
//! 崩溃残留的**不完整尾帧**按可恢复部分处理：解码报错即停，保留此前
//! 完整行；最后一行若被截断则丢弃并计数（原始文件不可变，不修补）。
//! `rest.contracts` 行即使被截断也尽力恢复——它是当日精度的唯一本地来源。

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use so_journal::JournalRow;

use crate::error::DataError;

/// 一个 Journal 文件的读取结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRead {
    /// 成功反序列化的完整行。
    pub rows: Vec<JournalRow>,
    /// 可恢复的 `rest.contracts` 响应原文（含截断修复后的原文）。
    pub contracts: Vec<(i64, String)>,
    /// 尾部截断或不完整 JSONL 行数。
    pub dropped_lines: u64,
    /// zstd 解码是否因尾帧不完整而中止（区别正常 EOF）。
    pub truncated: bool,
}

/// 读一个 `.journal.zst`：默认 zstd 解码器会连续拼接多个压缩帧，
/// 对应重连会话追加写入的布局（docs/06 §3.1）。
pub fn read_journal(path: &Path) -> Result<JournalRead, DataError> {
    let file = File::open(path)
        .map_err(|e| DataError::Journal(format!("打开 {}: {e}", path.display())))?;
    // Decoder 本身按 zstd 内部块大小缓冲，外层无需再套 BufReader。
    let decoder = zstd::stream::read::Decoder::new(file)
        .map_err(|e| DataError::Journal(format!("zstd 初始化 {}: {e}", path.display())))?;
    let mut reader = BufReader::new(decoder);
    let mut out = JournalRead {
        rows: Vec::new(),
        contracts: Vec::new(),
        dropped_lines: 0,
        truncated: false,
    };
    // 逐行拉取：read_line 返回 0 字节 = 正常 EOF；非 0 但解出残行
    // = 尾帧截断（最后一块解码失败时 zstd 返回 Err，行内容已可用）。
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => consume_line(&line, &mut out),
            Err(e) => {
                // 残行可能是尾帧截断前的最后一段，仍尝试消费。
                if !line.is_empty() {
                    consume_line(line.as_str(), &mut out);
                }
                out.truncated = true;
                let _ = e;
                break;
            }
        }
    }
    Ok(out)
}

/// 消费一行 JSONL：完整行入 `rows`；`rest.contracts` 抽原文；
/// 残缺行尝试修复，失败计 dropped。
fn consume_line(line: &str, out: &mut JournalRead) {
    let line = line.trim_end_matches(['\n', '\r']);
    if line.is_empty() {
        return;
    }
    match serde_json::from_str::<JournalRow>(line) {
        Ok(row) => {
            if row.channel == "rest.contracts" {
                out.contracts.push((row.ts_recv, row.raw.clone()));
            }
            out.rows.push(row);
        }
        Err(_) => {
            if line.contains("\"channel\":\"rest.contracts\"")
                && let Some((ts, raw)) = recover_contracts(line)
            {
                out.contracts.push((ts, raw));
                return;
            }
            out.dropped_lines = out.dropped_lines.saturating_add(1);
        }
    }
}

/// 修复被尾帧截断的 `rest.contracts` 行：补上 `raw` 字符串与外层对象的结尾。
///
/// 原始文件不可变，这里只在内存里补齐引号/花括号；若行尾恰好停在转义
/// 反斜杠上，多补一个引号再试。解析失败返回 `None`（按丢弃计数）。
fn recover_contracts(line: &str) -> Option<(i64, String)> {
    for suffix in ["\"}", "\"\"}", "\\\"}", "\\\"\"}"] {
        let repaired = format!("{line}{suffix}");
        if let Ok(row) = serde_json::from_str::<JournalRow>(&repaired) {
            return Some((row.ts_recv, row.raw));
        }
    }
    None
}

/// 列出某交易所某日目录下的所有 Journal 文件，按文件名排序保证确定性。
pub fn journal_files(raw_dir: &Path, venue: &str, date: &str) -> Result<Vec<PathBuf>, DataError> {
    let dir = raw_dir.join(venue).join(date);
    let rd = std::fs::read_dir(&dir)
        .map_err(|e| DataError::Journal(format!("读取目录 {}: {e}", dir.display())))?;
    let mut paths = Vec::new();
    for entry in rd {
        let entry =
            entry.map_err(|e| DataError::Journal(format!("遍历 {}: {e}", dir.display())))?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("zst") {
            paths.push(p);
        }
    }
    paths.sort();
    Ok(paths)
}

/// 日期字符串格式校验（`YYYY-MM-DD`，不引 chrono）。
pub fn valid_date(s: &str) -> bool {
    let Some((y, rest)) = s.split_once('-') else {
        return false;
    };
    let Some((m, d)) = rest.split_once('-') else {
        return false;
    };
    y.len() == 4
        && m.len() == 2
        && d.len() == 2
        && y.chars().all(|c| c.is_ascii_digit())
        && m.chars().all(|c| c.is_ascii_digit())
        && d.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use so_core::UnixNanos;
    use so_journal::JournalWriter;

    /// 写一个含多帧（模拟重连追加）+ 截断尾帧的 Journal，验证流式读取。
    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("so-data-journal-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn roundtrip_journal_rows() {
        let dir = temp_dir("read");
        let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
        let ts = 1_700_000_000_000_000_000_i64;
        w.write(&JournalRow::new(
            UnixNanos(ts),
            "futures.trades",
            "BTC_USDT",
            "A",
            r#"{"result":[]}"#,
        ))
        .unwrap();
        w.write(&JournalRow::new(
            UnixNanos(ts + 1),
            "rest.contracts",
            "*",
            "snapshot",
            r#"[{"name":"BTC_USDT"}]"#,
        ))
        .unwrap();
        w.finish().unwrap();
        let path = w.path_for(ts / 3_600_000_000_000);
        let got = read_journal(&path).unwrap();
        assert_eq!(got.rows.len(), 2);
        assert_eq!(got.contracts.len(), 1);
        assert_eq!(got.rows[0].channel, "futures.trades");
        assert_eq!(got.dropped_lines, 0);
        assert!(!got.truncated);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reconnect_append_frames_all_read() {
        // 同一文件两次会话 = 两个 zstd 帧；流式解码器必须都读到。
        let dir = temp_dir("multiframe");
        let mut w = JournalWriter::new(&dir, "gate", "A").unwrap();
        let ts = 1_700_000_000_000_000_000_i64;
        let hour = ts / 3_600_000_000_000;
        w.write(&JournalRow::new(
            UnixNanos(ts),
            "futures.trades",
            "BTC_USDT",
            "A",
            r#"{"result":[]}"#,
        ))
        .unwrap();
        w.finish().unwrap();
        let mut w2 = JournalWriter::new(&dir, "gate", "A").unwrap();
        w2.roll(hour).unwrap();
        w2.write(&JournalRow::new(
            UnixNanos(ts + 2),
            "futures.book_ticker",
            "BTC_USDT",
            "A",
            r#"{"result":{}}"#,
        ))
        .unwrap();
        w2.finish().unwrap();
        let path = w2.path_for(hour);
        let got = read_journal(&path).unwrap();
        assert_eq!(got.rows.len(), 2, "多帧必须全部读到");
        assert_eq!(got.rows[1].channel, "futures.book_ticker");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn truncated_contracts_line_is_recovered() {
        let full = JournalRow::new(
            UnixNanos(1_700_000_000_000_000_000),
            "rest.contracts",
            "*",
            "snapshot",
            r#"[{"name":"BTC_USDT","order_price_round":"0.1"},{"name":"ETH_USDT"}]"#,
        )
        .to_jsonl()
        .unwrap();
        let cut = full.trim_end().get(..full.trim_end().len() - 20).unwrap();
        let got = recover_contracts(cut);
        assert!(got.is_some());
        assert!(got.unwrap().1.contains("BTC_USDT"));
    }

    #[test]
    fn date_validation() {
        assert!(valid_date("2026-11-11"));
        assert!(!valid_date("2026-1-11"));
        assert!(!valid_date("not-a-date"));
    }
}
