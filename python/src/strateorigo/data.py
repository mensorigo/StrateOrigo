"""数据访问（docs/06 §8）：按数据集哈希加载、as-of join。

`load` 返回文件清单 dict，Python 侧用 Polars `scan_parquet` 组装
LazyFrame；零拷贝 Arrow 直传在 M4 随 `backtest.run()` 上线。
`asof_join` 把多品种/多交易所的表按时间对齐（只用"当时已知"的
数据，防止未来函数）。
"""

import polars as pl

from strateorigo._core import load

__all__ = ["asof_join", "load"]


def asof_join(
    left: pl.LazyFrame,
    right: pl.LazyFrame,
    *,
    left_on: str = "ts_recv",
    right_on: str = "ts_recv",
    by: str | list[str] | None = None,
    suffix: str = "_right",
) -> pl.LazyFrame:
    """按时间 as-of join（docs/06 §8：只用当时已知的数据）。

    对 left 的每一行，在 right 中找 `right_on <= left_on` 的最近一行
    （向前填充，不向后看）。`by` 指定分组键（如 `["symbol"]`），
    同组才匹配。

    参数
    ----
    left : 左表 LazyFrame（如 trade）。
    right : 右表 LazyFrame（如 bar 或另一品种）。
    left_on : 左表时间列名。
    right_on : 右表时间列名。
    by : 分组键（同组才匹配）。
    suffix : 右表非键列的重名后缀。

    Returns
    -------
    pl.LazyFrame
        左表每行附右表最近已知行（无匹配则 null）。
    """
    by_list: list[str] | None
    if by is None:
        by_list = None
    elif isinstance(by, str):
        by_list = [by]
    else:
        by_list = list(by)

    return left.join_asof(
        right,
        left_on=left_on,
        right_on=right_on,
        by=by_list,
        suffix=suffix,
    )
