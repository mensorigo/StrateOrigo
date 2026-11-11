//! `rest.contracts` → 品种精度 + `instrument` 时点行（docs/06 §4.2）。
//!
//! 精度取 `order_price_round`（价格 tick）的小数位；normalize 沿用
//! so-adapter-gate 的单 scale 解析接口，价格/数量都以该精度存储，
//! 因此文件元数据 `price_scale` / `qty_scale` 同值。
//! 崩溃残留的截断行按"扫描完整对象"恢复，原始文件本身不可变。

use std::collections::HashMap;

use so_adapter_gate::parse_contracts;
use so_adapter_gate::rest::ContractMsg;
use so_core::Scale;

use crate::normalize::InstrumentRow;

/// 从若干条 contracts 响应（按 ts_recv 升序）建精度映射与 instrument 行。
///
/// 后到的快照覆盖同品种精度（point-in-time）；`valid_to` 指向下一次快照，
/// 末条为 0（表示"至今有效"）。截断行扫描其中完整的合约对象恢复
/// （docs/06 §3.1：原始文件不可变，尾帧可恢复部分继续用）。
pub fn parse_contract_snapshots(
    raws: &[(i64, String)],
    venue: &str,
) -> (HashMap<String, Scale>, Vec<InstrumentRow>, u64) {
    let mut scales = HashMap::new();
    let mut by_symbol: HashMap<String, Vec<InstrumentRow>> = HashMap::new();
    let mut failed = 0_u64;
    for (ts, raw) in raws {
        let Some(list) = parse_contracts(raw)
            .ok()
            .or_else(|| recover_contract_objects(raw))
        else {
            failed = failed.saturating_add(1);
            continue;
        };
        for c in &list {
            if let Some(s) = scale_from_tick(&c.order_price_round) {
                scales.insert(c.name.clone(), s);
                by_symbol
                    .entry(c.name.clone())
                    .or_default()
                    .push(instrument_row(c, *ts, venue, s.decimals()));
            }
        }
    }
    let mut rows = Vec::new();
    for series in by_symbol.values_mut() {
        series.sort_by_key(|r| r.valid_from);
        for i in 0..series.len() {
            let next = series.get(i + 1).map_or(0, |r| r.valid_from);
            if let Some(r) = series.get_mut(i) {
                r.valid_to = next;
            }
        }
        rows.extend(series.iter().cloned());
    }
    rows.sort_by(|a, b| (a.valid_from, &a.symbol).cmp(&(b.valid_from, &b.symbol)));
    (scales, rows, failed)
}

/// 截断的 contracts 数组：逐段扫描完整的 `{...}` 对象反序列化。
fn recover_contract_objects(raw: &str) -> Option<Vec<ContractMsg>> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in raw.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let s = start.take()?;
                    let obj = raw.get(s..=i)?;
                    let c: ContractMsg = serde_json::from_str(obj).ok()?;
                    out.push(c);
                }
            }
            _ => {}
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

/// 一条合约元数据 → instrument 行（字符串列保持原文，不做精度换算）。
fn instrument_row(c: &ContractMsg, ts: i64, venue: &str, scale: u8) -> InstrumentRow {
    InstrumentRow {
        valid_from: ts,
        valid_to: 0,
        venue: venue.to_owned(),
        symbol: c.name.clone(),
        inst_type: "perpetual".to_owned(),
        tick_size: c.order_price_round.clone(),
        lot_size: c.order_size_min.clone(),
        min_notional: None,
        price_scale: i32::from(scale),
        qty_scale: i32::from(scale),
        status: c.status.clone(),
        contract_size: c.quanto_multiplier.clone(),
    }
}

/// `"0.0001"` → [`Scale`]（4）；无小数点 = 整数。
pub fn scale_from_tick(tick: &str) -> Option<Scale> {
    let decimals = tick
        .split_once('.')
        .map_or(0, |(_, frac)| frac.trim_end_matches('0').len());
    u8::try_from(decimals).ok().and_then(|d| Scale::new(d).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = r#"[{"name":"BTC_USDT","quanto_multiplier":"0.0001","order_price_round":"0.1","order_size_min":"1","order_size_max":"1000000","funding_rate":"0.002053","funding_interval":28800,"funding_next_apply":1610035200,"mark_price_round":"0.01","status":"trading","enable_decimal":false,"orders_limit":50}]"#;

    #[test]
    fn tick_to_scale() {
        assert_eq!(scale_from_tick("0.1").map(Scale::decimals), Some(1));
        assert_eq!(scale_from_tick("0.0001").map(Scale::decimals), Some(4));
        assert_eq!(scale_from_tick("1").map(Scale::decimals), Some(0));
        assert_eq!(scale_from_tick("0.0000000000000000001"), None);
    }

    #[test]
    fn point_in_time_rows() {
        let raws = vec![(10, RAW.to_owned()), (20, RAW.to_owned())];
        let (scales, rows, failed) = parse_contract_snapshots(&raws, "gate");
        assert_eq!(scales["BTC_USDT"].decimals(), 1);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].valid_from, 10);
        assert_eq!(rows[0].valid_to, 20);
        assert_eq!(rows[1].valid_to, 0);
        assert_eq!(failed, 0);
    }

    #[test]
    fn truncated_contracts_recovers_complete_objects() {
        // 只切掉数组尾部 `]`：对象完整但整体 JSON 不合法（模拟尾帧截断）。
        let cut = RAW.get(..RAW.len() - 1).unwrap();
        let (scales, rows, failed) = parse_contract_snapshots(&[(1, cut.to_owned())], "gate");
        assert_eq!(failed, 0);
        assert_eq!(scales["BTC_USDT"].decimals(), 1);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn bad_contracts_counted() {
        let (scales, rows, failed) = parse_contract_snapshots(&[(1, "{broken".to_owned())], "gate");
        assert!(scales.is_empty());
        assert!(rows.is_empty());
        assert_eq!(failed, 1);
    }
}
