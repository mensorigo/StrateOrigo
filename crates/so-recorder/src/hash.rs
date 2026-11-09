//! 文件校验和（docs/06 §3.2 A9：同步前按 sha256 校验）。
//!
//! 冷路径工具：流式读取，64 KiB 缓冲，不把整个文件读入内存。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::RecorderError;

/// 流式计算文件 SHA-256，返回小写十六进制。
pub fn sha256_file(path: &Path) -> Result<String, RecorderError> {
    let mut file =
        File::open(path).map_err(|e| RecorderError::Journal(format!("打开校验目标失败：{e}")))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| RecorderError::Journal(format!("读取校验目标失败：{e}")))?;
        if n == 0 {
            break;
        }
        match buf.get(..n) {
            Some(chunk) => hasher.update(chunk),
            None => break, // 不可达：n ≤ buf.len()
        }
    }
    use sha2::Digest;
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha_matches_known_content() {
        let dir = std::env::temp_dir().join("so-recorder-hash-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"abc").unwrap();
        drop(f);
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert_eq!(
            sha256_file(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_is_error() {
        let err = sha256_file(Path::new("C:/definitely/not/here.bin")).unwrap_err();
        assert_eq!(err.code(), "RecorderJournal");
    }
}
