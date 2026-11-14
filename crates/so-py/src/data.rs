//! `data`：数据集加载绑定（docs/06 §8 `strateorigo.data.load`）。
//!
//! 当前实现返回**文件清单**（表/品种/日期/路径），Python 侧用 Polars
//! `scan_parquet` 自行组装 LazyFrame；Arrow C Stream 零拷贝直传留到
//! M4 与 `backtest.run()` 一起上（届时才有真实消费者，避免过早做
//! 零拷贝管道）。错误映射只在边界做（04 E6）：`DataError.code()` →
//! Python `ValueError`，消息带码前缀。

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use so_data::dataset_in::load_dataset;

/// `strateorigo.data.load(dataset_hash, root) -> dict`。
///
/// 参数 `root` 是仓库根（含 `data/` 的目录，默认当前工作目录）。
/// 返回 `{hash, name, min_grade, start, end, files: [{table, symbol,
/// date, path}]}`；文件按 表→品种→日期 排序。
#[pyfunction]
#[pyo3(signature = (dataset_hash, root = "."))]
pub fn load<'py>(py: Python<'py>, dataset_hash: &str, root: &str) -> PyResult<Bound<'py, PyDict>> {
    let workspace = std::path::Path::new(root);
    let loaded = load_dataset(workspace, dataset_hash)
        .map_err(|e| PyValueError::new_err(format!("{}: {e}", e.code())))?;
    let out = PyDict::new(py);
    out.set_item("hash", &loaded.hash)?;
    out.set_item("name", &loaded.manifest.name)?;
    out.set_item("min_grade", &loaded.manifest.min_grade)?;
    out.set_item("start", &loaded.manifest.start)?;
    out.set_item("end", &loaded.manifest.end)?;
    let files = PyList::empty(py);
    for (table, symbol, date, path) in &loaded.files {
        let d = PyDict::new(py);
        d.set_item("table", table)?;
        d.set_item("symbol", symbol)?;
        d.set_item("date", date)?;
        d.set_item("path", path.to_string_lossy().to_string())?;
        files.append(d)?;
    }
    out.set_item("files", files)?;
    Ok(out)
}

/// 在父模块注册 `load`（顶层函数，不是子模块）。
///
/// 单文件扩展（.pyd）无法给 pyright 提供子模块存根路径；
/// 顶层函数 + 单个 `_core.pyi` 存根是类型检查可用的最小结构
/// （Python 侧 `strateorigo.data.load` 命名空间由 `data.py` 维护）。
pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    parent.add_function(wrap_pyfunction!(load, parent)?)?;
    Ok(())
}
