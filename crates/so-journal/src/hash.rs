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
        let n = file.read(&mut buf).map_err(|e| JournalError::Write {
            path: path.display().to_string(),
            bytes: 0,
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
}
