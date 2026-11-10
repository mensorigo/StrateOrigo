//! 解析基准（docs/05 M1 验收：单核 ≥ 50 万条深度消息/秒）。
//!
//! 读一个或多个 Raw Journal 文件（`.journal.zst`），逐行解 JournalRow，
//! 把 raw 中的 `result` 提取后交给 so-adapter-gate 的解析函数，
//! 统计各频道条数与总吞吐。
//!
//! 价格精度来自 Gate 合约元数据（启动时 REST 拉一次，与录制器快照任务
//! 同源）；网络不可用时回退固定 scale 4 并提示（低精度品种会计为失败）。
//!
//! 用法：bench-parse <file.zst>... [--json]
//! 输出人类可读表；--json 输出机器可读 JSON（VPS 验收报告用）。
//!
//! 与 zread 相同的读取策略：容忍崩溃残留的不完整尾帧（M2 normalize 同此）。

#![allow(clippy::print_stdout, clippy::print_stderr)] // 基准工具：输出 stdout，告警 stderr

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

use so_core::{Price, Qty, Scale, UnixNanos};
use so_recorder::JournalRow;

/// 各频道解析结果计数。
#[derive(Default)]
struct Counts {
    /// 成功解析的行数。
    parsed: u64,
    /// 解析失败（BadShape / 精度溢出等）的行数——帧仍在 Journal，只计数。
    failed: u64,
    /// 品种缺元数据（scale 未知）而跳过的行数。
    skipped: u64,
    /// 失败行按 (频道, 品种) 的分布（诊断用，不进 JSON）。
    failed_by: HashMap<String, u64>,
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let json = args.peek().is_some_and(|a| a == "--json");
    if json {
        args.next();
    }
    let files: Vec<String> = args.collect();
    if files.is_empty() {
        eprintln!("usage: bench-parse [--json] <file.zst>...");
        std::process::exit(2);
    }
    let (scales, scale_source) = load_scales();
    let mut counts = Counts::default();
    let mut rows: u64 = 0;
    let start = Instant::now();
    for file in &files {
        run_file(file, &scales, &mut counts, &mut rows);
    }
    let elapsed = start.elapsed();
    let per_sec = (rows as f64 / elapsed.as_secs_f64()).round() as u64;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "rows": rows,
                "parsed": counts.parsed,
                "failed": counts.failed,
                "skipped": counts.skipped,
                "scale_source": scale_source,
                "elapsed_ms": elapsed.as_millis() as u64,
                "rows_per_sec": per_sec,
            })
        );
    } else {
        println!("行数       {rows}");
        println!("解析成功   {}", counts.parsed);
        println!("解析失败   {}", counts.failed);
        println!("缺元数据   {}", counts.skipped);
        println!("精度来源   {scale_source}");
        println!("耗时       {:.3}s", elapsed.as_secs_f64());
        println!("吞吐       {per_sec} 行/秒");
        if !counts.failed_by.is_empty() {
            println!("失败分布（频道×品种 → 条数）：");
            let mut v: Vec<(&String, &u64)> = counts.failed_by.iter().collect();
            v.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
            for (k, n) in v.iter().take(15) {
                println!("    {k}: {n}");
            }
        }
    }
}

/// 启动时拉合约元数据建立 `symbol → scale`；带 3 次重试，全部失败回退固定 4。
fn load_scales() -> (HashMap<String, Scale>, &'static str) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let Some(rt) = (match rt {
        Ok(rt) => Some(rt),
        Err(e) => {
            eprintln!("tokio 运行时创建失败：{e}，回退 scale 4");
            None
        }
    }) else {
        return (HashMap::new(), "fallback(4)");
    };
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();
    let Ok(http) = http else {
        return (HashMap::new(), "fallback(4)");
    };
    let url = "https://api.gateio.ws/api/v4/futures/usdt/contracts";
    let raw = rt.block_on(fetch_with_retry(&http, url));
    let Some(raw) = raw else {
        eprintln!("合约元数据拉取失败（含重试），回退 scale 4");
        return (HashMap::new(), "fallback(4)");
    };
    match so_adapter_gate::parse_contracts(&raw) {
        Ok(list) => {
            let map: HashMap<String, Scale> = list
                .into_iter()
                .filter_map(|c| scale_from_tick(&c.order_price_round).map(|s| (c.name, s)))
                .collect();
            if map.is_empty() {
                eprintln!("合约元数据解析为空，回退 scale 4");
                return (HashMap::new(), "fallback(4)");
            }
            (map, "rest")
        }
        Err(e) => {
            eprintln!("合约元数据解析失败（{}），回退 scale 4", e.code());
            (HashMap::new(), "fallback(4)")
        }
    }
}

/// 带重试的 GET（间隔 1s，最多 3 次）。带 Size-Decimal 头
/// （不带此头 size 字段形态不同，docs/venues/gate.md §2.4）。
async fn fetch_with_retry(http: &reqwest::Client, url: &str) -> Option<String> {
    for attempt in 1..=3 {
        if let Ok(r) = http
            .get(url)
            .header("Accept", "application/json")
            .header("X-Gate-Size-Decimal", "1")
            .send()
            .await
            && r.status().is_success()
            && let Ok(text) = r.text().await
        {
            return Some(text);
        }
        eprintln!("元数据拉取尝试 {attempt}/3 失败");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    None
}

/// `order_price_round` 字符串 → 精度（与 snapshot_pull 同逻辑）。
fn scale_from_tick(tick: &str) -> Option<Scale> {
    let decimals = tick
        .split_once('.')
        .map_or(0, |(_, frac)| frac.trim_end_matches('0').len());
    u8::try_from(decimals).ok().and_then(|d| Scale::new(d).ok())
}

/// 解压并解析一个 Journal 文件（尾帧截断按可恢复部分继续）。
fn run_file(path: &str, scales: &HashMap<String, Scale>, counts: &mut Counts, rows: &mut u64) {
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
        eprintln!("{path}: 尾部截断（按可恢复部分基准）");
    }
    let text = String::from_utf8_lossy(&all);
    for line in text.lines() {
        *rows = rows.saturating_add(1);
        match parse_line(line, scales) {
            LineResult::Parsed => counts.parsed = counts.parsed.saturating_add(1),
            LineResult::Failed => {
                counts.failed = counts.failed.saturating_add(1);
                let key = serde_json::from_str::<JournalRow>(line)
                    .map(|r| format!("{}×{}", r.channel, r.symbol))
                    .unwrap_or_else(|_| "<非 JSON 行>".to_owned());
                *counts.failed_by.entry(key).or_insert(0) += 1;
            }
            LineResult::NoScale => counts.skipped = counts.skipped.saturating_add(1),
        }
    }
}

/// 一行解析结果。
enum LineResult {
    /// 成功。
    Parsed,
    /// 数据帧但解析失败。
    Failed,
    /// 品种缺 scale 元数据（元数据拉取失败时）。
    NoScale,
}

/// 解析一行：JournalRow → 频道分发 → so-adapter-gate 解析函数。
fn parse_line(line: &str, scales: &HashMap<String, Scale>) -> LineResult {
    let Ok(row) = serde_json::from_str::<JournalRow>(line) else {
        return LineResult::Failed;
    };
    let ts_recv = UnixNanos(row.ts_recv);
    let Some(scale) = scales.get(&row.symbol).copied() else {
        return LineResult::NoScale;
    };
    let raw: serde_json::Value = match serde_json::from_str(&row.raw) {
        Ok(v) => v,
        Err(_) => return LineResult::Failed,
    };
    let Some(result) = raw.get("result") else {
        return LineResult::Failed; // 订阅确认 / pong 等非数据帧（数据面不该出现）
    };
    let Some(result_json) = serde_json::to_string(result).ok() else {
        return LineResult::Failed;
    };
    let ok = match row.channel.as_str() {
        "futures.order_book_update" => {
            so_adapter_gate::parse_obu(&result_json, ts_recv, scale).is_ok()
        }
        "futures.book_ticker" => so_adapter_gate::parse_bbo(&result_json, ts_recv, scale).is_ok(),
        "futures.trades" => {
            let mut out = [so_adapter_gate::RawTrade {
                ts_recv,
                ts_exch: UnixNanos(0),
                trade_id: 0,
                price: Price::ZERO,
                qty: Qty::ZERO,
                aggressor: so_model::Side::Bid,
                internal: false,
            }; 32];
            so_adapter_gate::parse_trade(&result_json, ts_recv, scale, &mut out).is_ok()
        }
        "futures.tickers" => so_adapter_gate::parse_ticker(&result_json, ts_recv, scale).is_ok(),
        _ => true, // 其他频道不计失败（M2 normalize 扩展）
    };
    if ok {
        LineResult::Parsed
    } else {
        LineResult::Failed
    }
}
