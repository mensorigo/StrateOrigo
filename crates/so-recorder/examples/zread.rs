//! 临时校验工具：解压 Journal 并抽样检查行内容。
//!
//! 容忍崩溃残留的**不完整尾帧**（进程被 kill 时 `finish()` 未执行）：
//! 逐帧解码，解码失败即停，报告已恢复的行数。M2 normalize 读取端同此策略。
//! 传第二参数 = 频道过滤，第三参数 = 打印条数（打印完整 raw 便于核对字段类型）。

#![allow(clippy::print_stdout)] // 校验工具的输出就是 stdout

use std::fs::File;
use std::io::Read;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: zread <file.zst> [channel] [count]");
    let filter = std::env::args().nth(2);
    let count: usize = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let mut buf = Vec::new();
    File::open(&path).unwrap().read_to_end(&mut buf).unwrap();
    let mut decoder = zstd::stream::read::Decoder::new(buf.as_slice()).unwrap();
    let mut all = Vec::new();
    match decoder.read_to_end(&mut all) {
        Ok(_) => {}
        Err(e) => println!("尾部截断（按可恢复部分校验）：{e}"),
    }
    let text = String::from_utf8_lossy(&all);
    let lines: Vec<&str> = text.lines().collect();
    println!("总行数: {}", lines.len());
    let mut shown = 0;
    for l in &lines {
        if let Some(f) = filter.as_deref()
            && !l.contains(&format!("\"channel\":\"{f}\""))
        {
            continue;
        }
        if shown >= count {
            break;
        }
        // 打印完整行（含 raw 原文）
        println!("{l}");
        shown += 1;
    }
    // 各频道行数统计
    let mut stats = std::collections::BTreeMap::new();
    for l in &lines {
        let ch = l
            .split("\"channel\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("?");
        *stats.entry(ch.to_owned()).or_insert(0usize) += 1;
    }
    for (ch, cnt) in stats {
        println!("{ch}: {cnt}");
    }
}
