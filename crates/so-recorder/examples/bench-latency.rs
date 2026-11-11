//! 行情延迟报告（docs/05 M1：`ts_recv − ts_exch` 分布，VPS 选址测试）。
//!
//! 读 Raw Journal，对每条数据帧取交易所时间戳（raw 里的毫秒字段，按频道
//! 不同：obu/bbo/tickers 为 `t`，trades 为 `create_time_ms`），与录制时
//! 打点的 `ts_recv` 相减，输出分频道/分品种的延迟分布（p50/p90/p99/max）
//! 与直方图。
//!
//! 语义注意：
//! - ts_exch 是**交易所生成消息**的时间，ts_recv 是**本机收到**的时间，
//!   差值 = 网络传输 + 交易所出队，不是纯 RTT/2；选址对比看相对值。
//! - 时钟偏移会整体平移分布（VPS 上先过 vps-preflight 的 ≤10ms 检查）。
//! - trades 是数组体，取首笔的 create_time_ms。
//!
//! 用法：bench-latency <file.zst>... [--json]
//! 输出人类可读表（默认）或机器可读 JSON（选址报告用）。

#![allow(clippy::print_stdout, clippy::print_stderr)] // 报告工具：输出 stdout，告警 stderr

use std::fs::File;
use std::io::Read;

use so_recorder::JournalRow;

/// 单频道延迟样本聚合（微秒，i64）。诊断工具保留全部原始样本：
/// 时钟偏移检测与精确分位都需要原始值，有损直方图会把负值场景的
/// 信息全部压掉（一天数据约数百万样本 × 8B ≈ 几十 MB，可接受）。
#[derive(Default)]
struct ChannelStats {
    /// 全部延迟样本（µs）。
    samples: Vec<i64>,
}

impl ChannelStats {
    fn add(&mut self, us: i64) {
        self.samples.push(us);
    }

    /// 排序后缓存。数据文件处理完才调用一次。
    fn sort(&mut self) {
        self.samples.sort_unstable();
    }

    fn n(&self) -> u64 {
        self.samples.len() as u64
    }

    /// 精确分位（µs）。p ∈ (0,1]；线性插值。
    fn percentile(&self, p: f64) -> i64 {
        let len = self.samples.len();
        if len == 0 {
            return 0;
        }
        let idx = ((len as f64 - 1.0) * p).round() as usize;
        self.samples[idx.min(len - 1)]
    }

    fn mean_us(&self) -> i64 {
        if self.samples.is_empty() {
            return 0;
        }
        let sum: i128 = self.samples.iter().map(|&v| i128::from(v)).sum();
        (sum / self.samples.len() as i128) as i64
    }

    fn min_us(&self) -> i64 {
        *self.samples.first().unwrap_or(&0)
    }

    fn max_us(&self) -> i64 {
        *self.samples.last().unwrap_or(&0)
    }

    /// 负样本数（排序后从头线性扫）。
    fn negatives(&self) -> u64 {
        self.samples.iter().filter(|&&v| v < 0).count() as u64
    }
}

/// 频道名归一：同频道聚合；--by-symbol 时按 频道×品种 聚合。
fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let by_symbol = args.iter().any(|a| a == "--by-symbol");
    args.retain(|a| !a.starts_with("--"));
    if args.is_empty() {
        eprintln!("usage: bench-latency [--json] [--by-symbol] <file.zst>...");
        std::process::exit(2);
    }
    let mut stats: std::collections::BTreeMap<String, ChannelStats> = Default::default();
    for file in &args {
        run_file(file, &mut stats, by_symbol);
    }
    for s in stats.values_mut() {
        s.sort();
    }
    // 时钟偏移检测：多数样本为负 → 本机时钟落后。真实网络延迟不为负，
    // 负值主导必来自时钟偏移；按每频道 p5 估计偏移并扣除后再报分布。
    let mut offset_note = String::new();
    let total: u64 = stats.values().map(|s| s.n()).sum();
    let negatives: u64 = stats.values().map(|s| s.negatives()).sum();
    if total > 0 && negatives * 2 > total {
        let mut offsets: Vec<i64> = stats.values().map(|s| s.percentile(0.05)).collect();
        offsets.sort_unstable();
        let med_off = offsets[offsets.len() / 2];
        offset_note = format!(
            "时钟偏移：{:.0}ms（多数样本为负，本机时钟落后；以下为扣除偏移后的分布）",
            med_off as f64 / 1e3
        );
        for s in stats.values_mut() {
            for v in s.samples.iter_mut() {
                *v -= med_off;
            }
            s.samples.sort_unstable();
        }
    }
    if json {
        print_json(&stats, &offset_note);
    } else {
        print_table(&stats, &offset_note);
    }
}

/// 解压一个 Journal 文件并聚合（尾帧截断按可恢复部分继续）。
fn run_file(
    path: &str,
    stats: &mut std::collections::BTreeMap<String, ChannelStats>,
    by_symbol: bool,
) {
    let mut buf = Vec::new();
    File::open(path)
        .unwrap_or_else(|e| panic!("打开 {path} 失败：{e}"))
        .read_to_end(&mut buf)
        .unwrap_or_else(|e| panic!("读取 {path} 失败：{e}"));
    let mut decoder = match zstd::stream::read::Decoder::new(buf.as_slice()) {
        Ok(d) => d,
        Err(e) => panic!("zstd 初始化 {path} 失败：{e}"),
    };
    let mut all = Vec::new();
    if decoder.read_to_end(&mut all).is_err() {
        eprintln!("{path}: 尾部截断（按可恢复部分统计）");
    }
    let text = String::from_utf8_lossy(&all);
    for line in text.lines() {
        let Ok(row) = serde_json::from_str::<JournalRow>(line) else {
            continue;
        };
        let Some(exch_ms) = extract_exch_ms(&row) else {
            continue;
        };
        let exch_ns = i128::from(exch_ms) * 1_000_000;
        let delta_us = ((i128::from(row.ts_recv) - exch_ns) / 1_000) as i64;
        let key = if by_symbol {
            format!("{}×{}", row.channel, row.symbol)
        } else {
            row.channel.clone()
        };
        stats.entry(key).or_default().add(delta_us);
    }
}

/// 从 raw 提取交易所时间戳（毫秒）。浅解析：定位到 `result` 后取字段。
///
/// - obu / bbo / tickers：result 对象内 `t`（毫秒整型）
/// - trades：result 数组首元素的 `create_time_ms`
fn extract_exch_ms(row: &JournalRow) -> Option<i64> {
    let raw: serde_json::Value = serde_json::from_str(&row.raw).ok()?;
    let result = raw.get("result")?;
    let t_field = |v: &serde_json::Value| -> Option<i64> { v.get("t").and_then(|t| t.as_i64()) };
    match result {
        serde_json::Value::Array(arr) => arr
            .first()
            .and_then(|first| first.get("create_time_ms"))
            .and_then(|t| t.as_i64()),
        serde_json::Value::Object(_) => t_field(result),
        _ => None,
    }
}

/// 人类可读表。
fn print_table(stats: &std::collections::BTreeMap<String, ChannelStats>, note: &str) {
    if !note.is_empty() {
        println!("{note}");
    }
    println!(
        "{:<38} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8} {:>7}",
        "频道", "样本", "p50", "p90", "p99", "max", "均值", "负值"
    );
    for (name, s) in stats {
        let f = |us: i64| -> String {
            if us >= 1_000_000 {
                format!("{:.2}s", us as f64 / 1e6)
            } else if us >= 1_000 {
                format!("{:.1}ms", us as f64 / 1e3)
            } else {
                format!("{us}µs")
            }
        };
        println!(
            "{:<38} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8} {:>7}",
            name,
            s.n(),
            f(s.percentile(0.50)),
            f(s.percentile(0.90)),
            f(s.percentile(0.99)),
            f(s.max_us()),
            f(s.mean_us()),
            s.negatives()
        );
    }
    println!(
        "\n语义：ts_recv − ts_exch（交易所生成 → 本机收到；含网络 + 交易所出队，非纯 RTT/2）。"
    );
    println!("负值 = 本机时钟落后于交易所时钟（preflight 的 chrony 检查应拦住大偏移）。");
}

/// 机器可读 JSON。
fn print_json(stats: &std::collections::BTreeMap<String, ChannelStats>, note: &str) {
    let map: serde_json::Map<String, serde_json::Value> = stats
        .iter()
        .map(|(k, s)| {
            (
                k.clone(),
                serde_json::json!({
                    "n": s.n(),
                    "p50_us": s.percentile(0.50),
                    "p90_us": s.percentile(0.90),
                    "p99_us": s.percentile(0.99),
                    "max_us": s.max_us(),
                    "min_us": s.min_us(),
                    "mean_us": s.mean_us(),
                    "negatives": s.negatives(),
                }),
            )
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({ "clock_offset_note": note, "channels": map })
    );
}
