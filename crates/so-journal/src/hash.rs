//! 文件校验和（docs/06 §3.2 A9：同步前按 sha256 校验）。
//!
//! 冷路径工具：流式读取，64 KiB 缓冲，不把整个文件读入内存。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::JournalError;

/// 流式计算文件 SHA-256，返回小写十六进制。
pub fn sha256_file(path: &Path) -> Result<String, JournalError> {
    let mut file = File::open(path).map_err(|e| JournalError::Open {
        path: path.display().to_string(),
        source: e,
    })?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0_u8; 64 * 1024];
    loop {
        // 读失败不能标成 Write：日志里 "写入失败" 会把排查引向错误方向。
        let n = file.read(&mut buf).map_err(|e| JournalError::Read {
            path: path.display().to_string(),
            source: e,
        })?;
        if n == 0 {
            break;
        }
        let Some(chunk) = buf.get(..n) else {
            break;
        };
        hasher.update(chunk);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// 按 `.sha256` 边车校验一个 Journal 文件（docs/06 §3.2 A9）。
///
/// 同步流程在 `scripts/sync-journal.ps1` 里做过同样的事，但那是 PowerShell 侧；
/// 这里提供 Rust 侧实现，供数据校验命令与将来的 VPS 侧工具复用——
/// 校验逻辑只有一处定义、两种调用方（[04 §3.4 I13](../../docs/04-dev-constraints.md)）。
///
/// 只在**同步与校验**路径调用，不进录制热路径。
pub fn verify_sidecar(path: &Path) -> Result<(), JournalError> {
    let sha_path = path.with_extension("zst.sha256");
    let text = std::fs::read_to_string(&sha_path).map_err(|e| JournalError::Checksum {
        path: sha_path.display().to_string(),
        source: e,
    })?;
    // 边车格式为 `<hex>  <path>`（与 sha256sum 一致）；只取第一段。
    let Some(expected) = text.split_whitespace().next() else {
        return Err(JournalError::Checksum {
            path: sha_path.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, "边车内容为空"),
        });
    };
    let actual = sha256_file(path)?;
    if expected != actual {
        return Err(JournalError::ChecksumMismatch {
            path: path.display().to_string(),
            expected: expected.to_owned(),
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vector() {
        // 空文件的 SHA-256（公开已知值）。
        let dir = std::env::temp_dir().join("so-journal-sha");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("empty.bin");
        std::fs::write(&p, b"").unwrap();
        assert_eq!(
            sha256_file(&p).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // "abc" 的 SHA-256（公开已知值）。
        std::fs::write(&p, b"abc").unwrap();
        assert_eq!(
            sha256_file(&p).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 边车校验：内容一致通过；文件被改动后报 ChecksumMismatch（不是读边车失败）。
    #[test]
    fn verify_sidecar_detects_tampering() {
        let dir = std::env::temp_dir().join("so-journal-verify");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("01.A.journal.zst");
        std::fs::write(&f, b"payload").unwrap();
        let good = sha256_file(&f).unwrap();
        std::fs::write(
            f.with_extension("zst.sha256"),
            format!(
                "{good}  {}
",
                f.display()
            ),
        )
        .unwrap();
        assert!(verify_sidecar(&f).is_ok(), "边车一致时应通过");

        // 内容变了（模拟截断/篡改），边车没变
        std::fs::write(&f, b"payload-tampered").unwrap();
        let err = verify_sidecar(&f).unwrap_err();
        assert_eq!(err.code(), "JournalChecksumMismatch");
        assert!(err.to_string().contains(&good), "报错要带上期望值");

        // 边车缺失 → Checksum（读边车失败），与上面区分
        std::fs::remove_file(f.with_extension("zst.sha256")).unwrap();
        assert_eq!(verify_sidecar(&f).unwrap_err().code(), "JournalChecksum");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
