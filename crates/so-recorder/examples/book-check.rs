//! 订单簿重建验证（docs/05 M2 验收：重建簿 vs REST 快照，前 20 档一致率 ≥ 99.9%）。
//!
//! 读 Raw Journal（增量 + rest.order_book 快照），用 BookCheck 重建比对，
//! 输出每品种的快照级/档级一致率与缺口计数。
//!
//! 价格精度来源与 bench-parse 相同（REST 合约元数据；失败回退 scale 4）。
//!
//! 用法：book-check <file.zst>... [--json]
//! 退出码：0 = 全部品种快照级一致率 ≥ 99.9%；1 = 有品种不达标；2 = 参数错误。

#![allow(clippy::print_stdout, clippy::print_stderr)] // 报告工具：输出 stdout，告警 stderr

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use so_core::Scale;
use so_recorder::JournalRow;
use so_recorder::bookcheck::{BookCheck, SymbolStats};

/// M2 验收阈值（快照级一致率）。
const THRESHOLD: f64 = 0.999;

fn main() -> std::process::ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    args.retain(|a| !a.starts_with("--"));
    if args.is_empty() {
        eprintln!("usage: book-check [--json] <file.zst>...");
        return std::process::ExitCode::from(2);
    }
    let (scales, scale_source) = load_scales();
    let mut checker = BookCheck::new(scales);
    // 多文件（A/B 双流 + snapshot）按 ts_recv 全局时序归并喂入：
    // 快照锚定与增量衔接都依赖严格时序，按文件分批喂会制造人为 Gap。
    let mut rows: Vec<JournalRow> = Vec::new();
    for file in &args {
        read_file(file, &mut rows);
    }
    rows.sort_by_key(|r| r.ts_recv);
    for row in &rows {
        checker.on_row(row);
    }
    let stats = checker.stats();
    let pass = stats.iter().all(|(_, s)| s.snapshot_rate() >= THRESHOLD);
    if json {
        print_json(&stats, scale_source);
    } else {
        print_table(&stats, scale_source);
    }
    if pass {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// 解压一个 Journal 文件的行追加到 `rows`（尾帧截断按可恢复部分继续）。
fn read_file(path: &str, rows: &mut Vec<JournalRow>) {
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
        if let Ok(row) = serde_json::from_str::<JournalRow>(line) {
            rows.push(row);
        }
    }
}

/// 启动时拉合约元数据建 scale 映射（带 3 次重试；失败回退固定 4）。
fn load_scales() -> (HashMap<String, Scale>, &'static str) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let Ok(rt) = rt else {
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
                return (HashMap::new(), "fallback(4)");
            }
            (map, "rest")
        }
        Err(_) => (HashMap::new(), "fallback(4)"),
    }
}

/// 带重试的 GET（间隔 1s，最多 3 次；带 Size-Decimal 头）。
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

/// `order_price_round` → 精度。
fn scale_from_tick(tick: &str) -> Option<Scale> {
    let decimals = tick
        .split_once('.')
        .map_or(0, |(_, frac)| frac.trim_end_matches('0').len());
    u8::try_from(decimals).ok().and_then(|d| Scale::new(d).ok())
}

/// 人类可读表。
#[allow(clippy::too_many_arguments)]
fn print_table(stats: &[(String, SymbolStats)], scale_source: &str) {
    println!("精度来源   {scale_source}");
    println!(
        "{:<14} {:>8} {:>8} {:>10} {:>10} {:>6} {:>6} {:>8}",
        "品种", "增量帧", "比对数", "快照一致", "档一致", "缺口", "过旧", "解析失败"
    );
    for (symbol, s) in stats {
        println!(
            "{:<14} {:>8} {:>8} {:>9.3}% {:>9.3}% {:>6} {:>6} {:>8}",
            symbol,
            s.frames,
            s.comparisons,
            s.snapshot_rate() * 100.0,
            s.level_rate() * 100.0,
            s.gaps,
            s.stale,
            s.parse_failed
        );
    }
    println!("\n阈值：快照级一致率 ≥ 99.9%（docs/05 M2）；不达标品种退出码 1。");
}

/// 机器可读 JSON。
fn print_json(stats: &[(String, SymbolStats)], scale_source: &str) {
    let map: serde_json::Map<String, serde_json::Value> = stats
        .iter()
        .map(|(k, s)| {
            (
                k.clone(),
                serde_json::json!({
                    "frames": s.frames,
                    "comparisons": s.comparisons,
                    "matched": s.matched,
                    "snapshot_rate": s.snapshot_rate(),
                    "level_rate": s.level_rate(),
                    "levels": s.levels,
                    "levels_matched": s.levels_matched,
                    "gaps": s.gaps,
                    "stale": s.stale,
                    "parse_failed": s.parse_failed,
                    "no_scale": s.no_scale,
                }),
            )
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({ "scale_source": scale_source, "symbols": map })
    );
}
