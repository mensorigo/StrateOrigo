//! 定时 REST 快照拉取（docs/06 §3.2 A6）。
//!
//! 每 N 分钟拉一次深度快照（`with_id=true`，供清洗阶段校验重建簿 C9）；
//! 顺带记录品种元数据（`/contracts`）。网络层由调用方注入（reqwest），
//! 本模块只负责 URL 构造、响应解析编排与 Journal 落盘。

use so_adapter_gate::{parse_contracts, parse_order_book_rest};
use so_core::UnixNanos;

use crate::error::RecorderError;
use crate::journal::{JournalRow, JournalWriter};

/// REST 快照端点构造器。
#[derive(Debug, Clone)]
pub struct RestUrls {
    /// REST 基址（如 `https://api.gateio.ws/api/v4`）。
    pub base: String,
}

impl RestUrls {
    /// 构造。
    pub fn new(base: impl Into<String>) -> Self {
        Self { base: base.into() }
    }

    /// 合约列表（品种元数据）。
    pub fn contracts(&self, settle: &str) -> String {
        format!("{}/futures/{settle}/contracts", self.base)
    }

    /// 深度快照（六步法第 1 步：`with_id=true` 拿 baseID）。
    pub fn order_book(&self, settle: &str, contract: &str, limit: u32) -> String {
        format!(
            "{}/futures/{settle}/order_book?contract={contract}&limit={limit}&with_id=true",
            self.base
        )
    }

    /// 品种元数据快照（单合约）。
    pub fn contract(&self, settle: &str, contract: &str) -> String {
        format!("{}/futures/{settle}/contracts/{contract}", self.base)
    }
}

/// 一次深度快照拉取的结果摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotOutcome {
    /// 品种。
    pub symbol: String,
    /// 快照 ID（baseID）。
    pub snapshot_id: u64,
    /// 买侧档位数。
    pub bid_levels: usize,
    /// 卖侧档位数。
    pub ask_levels: usize,
    /// 本地接收时间。
    pub ts_recv: UnixNanos,
}

/// 处理一次 REST 深度快照响应：解析 → 写 Journal（原始帧入档，A1）。
///
/// `raw` 是响应体原文；解析失败仍写 Journal 并返回错误（帧已落盘）。
pub fn record_order_book_snapshot(
    journal: &mut JournalWriter,
    symbol: &str,
    raw: &str,
    ts_recv: UnixNanos,
    price_scale: so_core::Scale,
) -> Result<SnapshotOutcome, RecorderError> {
    // 先落盘原始帧（docs/06 §3.2 A1：先存原始帧，再解析）
    journal.write(&JournalRow::new(
        ts_recv,
        "rest.order_book",
        symbol,
        "snapshot",
        raw,
    ))?;
    let snap = parse_order_book_rest(raw, ts_recv, price_scale)
        .map_err(|e| RecorderError::Parse { code: e.code() })?;
    Ok(SnapshotOutcome {
        symbol: symbol.to_owned(),
        snapshot_id: snap.snapshot_id,
        bid_levels: snap.bids.as_slice().len(),
        ask_levels: snap.asks.as_slice().len(),
        ts_recv,
    })
}

/// 处理一次合约元数据响应：解析（只验证可解析性）→ 写 Journal。
pub fn record_contracts(
    journal: &mut JournalWriter,
    raw: &str,
    ts_recv: UnixNanos,
) -> Result<usize, RecorderError> {
    journal.write(&JournalRow::new(
        ts_recv,
        "rest.contracts",
        "*",
        "snapshot",
        raw,
    ))?;
    let contracts = parse_contracts(raw).map_err(|e| RecorderError::Parse { code: e.code() })?;
    Ok(contracts.len())
}

/// 快照定时器：判断是否到了下一次快照时间（单调推进，测试注入时钟）。
#[derive(Debug, Clone, Copy)]
pub struct SnapshotTimer {
    /// 间隔（纳秒）。
    interval: i64,
    /// 下次快照时间（纳秒）。
    next_at: i64,
}

impl SnapshotTimer {
    /// 新建定时器（首次立即触发）。
    pub fn new(interval_secs: u64, now: i64) -> Self {
        Self {
            interval: (interval_secs as i64).saturating_mul(1_000_000_000),
            next_at: now,
        }
    }

    /// 是否到期。
    pub fn due(&self, now: i64) -> bool {
        now >= self.next_at
    }

    /// 领取一次到期（推进下次时间）；未到期返回 false。
    pub fn take(&mut self, now: i64) -> bool {
        if !self.due(now) {
            return false;
        }
        self.next_at = now.saturating_add(self.interval);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("so-recorder-snap-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn url_shapes() {
        let u = RestUrls::new("https://api.gateio.ws/api/v4");
        assert_eq!(
            u.order_book("usdt", "BTC_USDT", 100),
            "https://api.gateio.ws/api/v4/futures/usdt/order_book?contract=BTC_USDT&limit=100&with_id=true"
        );
        assert_eq!(
            u.contracts("usdt"),
            "https://api.gateio.ws/api/v4/futures/usdt/contracts"
        );
        assert_eq!(
            u.contract("usdt", "ETH_USDT"),
            "https://api.gateio.ws/api/v4/futures/usdt/contracts/ETH_USDT"
        );
    }

    #[test]
    fn snapshot_written_to_journal_then_parsed() {
        let dir = temp_dir("record");
        let mut j = JournalWriter::new(&dir, "gate", "snapshot").unwrap();
        let raw = r#"{"id":123456,"current":1.0,"update":1.0,
            "asks":[{"p":"1.52","s":"100"}],
            "bids":[{"p":"1.17","s":"150"}]}"#;
        let out = record_order_book_snapshot(
            &mut j,
            "BTC_USDT",
            raw,
            UnixNanos(1_700_000_000_000_000_000),
            so_core::Scale::new(2).unwrap(),
        )
        .unwrap();
        assert_eq!(out.snapshot_id, 123456);
        assert_eq!(out.bid_levels, 1);
        assert_eq!(out.ask_levels, 1);
        assert_eq!(j.stats().total_rows, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snapshot_parse_failure_still_journals() {
        let dir = temp_dir("bad");
        let mut j = JournalWriter::new(&dir, "gate", "snapshot").unwrap();
        let err = record_order_book_snapshot(
            &mut j,
            "BTC_USDT",
            r#"{"broken"#,
            UnixNanos(1_700_000_000_000_000_000),
            so_core::Scale::new(2).unwrap(),
        )
        .unwrap_err();
        assert_eq!(err.code(), "RecorderParse");
        // 原始帧已入 Journal（A1）
        assert_eq!(j.stats().total_rows, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn contracts_recorded() {
        let dir = temp_dir("contracts");
        let mut j = JournalWriter::new(&dir, "gate", "snapshot").unwrap();
        let raw = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1",
            "order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053",
            "funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01",
            "status":"trading","enable_decimal":false,"orders_limit":50}]"#;
        let n = record_contracts(&mut j, raw, UnixNanos(1_700_000_000_000_000_000)).unwrap();
        assert_eq!(n, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snapshot_timer_fires_then_intervals() {
        let mut t = SnapshotTimer::new(300, 1000);
        assert!(t.due(1000));
        assert!(t.take(1000)); // 首次立即
        assert!(!t.due(1000 + 299 * 1_000_000_000));
        assert!(t.take(1000 + 300 * 1_000_000_000));
        // 领取后重新计时
        assert!(!t.due(1000 + 300 * 1_000_000_000 + 1));
    }
}
