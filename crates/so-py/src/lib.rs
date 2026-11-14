//! PyO3 绑定入口：`strateorigo._core`。只做模块注册与重新导出。
//!
//! 目前暴露无状态基础能力（版本、运行模式规则）与 `data.load`
//! （数据集加载，docs/06 §8）；回测入口在 M3 加入（docs/05 M3
//! `backtest.run()`）。

use pyo3::prelude::*;

mod data;
mod version;

/// Python 可见的模块。
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    version::register(m)?;
    data::register(m)?;
    Ok(())
}
