"""数据访问（docs/06 §8）：按数据集哈希加载。

`load` 返回文件清单 dict，Python 侧用 Polars `scan_parquet` 组装
LazyFrame；零拷贝 Arrow 直传在 M4 随 `backtest.run()` 上线。
"""

from strateorigo._core import load

__all__ = ["load"]
