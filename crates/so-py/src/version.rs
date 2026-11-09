//! `run_mode_rules` 与 `parse_price`：把 so-core 的校验规则暴露给研究端。

use std::str::FromStr;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use so_core::RunMode;
/// 注册到 Python 模块。
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_mode_rules, m)?)?;
    m.add_function(wrap_pyfunction!(parse_price, m)?)?;
    Ok(())
}

/// 各运行模式的账户/策略规则（ADR-0001），返回 `(可加载密钥, 每交易所最大账户数, 每账户策略数)`。
#[pyfunction]
fn run_mode_rules(mode: &str) -> PyResult<(bool, Option<u16>, u16)> {
    let m = RunMode::from_str(mode)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("UnknownRunMode: {e}")))?;
    Ok((
        m.may_load_keys(),
        m.max_accounts_per_venue(),
        m.max_strategies_per_account(),
    ))
}

/// 精确解析十进制字符串为定点整数（不经过浮点）。
///
/// 失败时抛出 `ValueError`，消息以稳定错误码开头（如 `Parse:`、`ExcessPrecision:`），
/// 程序判别用错误码，不要匹配中文消息。
#[pyfunction]
fn parse_price(text: &str, decimals: u8) -> PyResult<i64> {
    let raise = |e: so_core::FixedError| {
        pyo3::exceptions::PyValueError::new_err(format!("{}: {e}", e.code()))
    };
    let scale = so_core::Scale::new(decimals).map_err(raise)?;
    so_core::fixed::parse_decimal(text, scale).map_err(raise)
}
