"""strateorigo._core 的类型存根（C 扩展，无运行时内容）。

与 crates/so-py/src/lib.rs 的 #[pymodule] 注册保持一致；
改 Rust 绑定后须同步更新本文件（I1：枚举/注册表模式的两端）。
"""

from typing import TypedDict

class DatasetFile(TypedDict):
    table: str
    symbol: str
    date: str
    path: str

class DatasetInfo(TypedDict):
    hash: str
    name: str
    min_grade: str
    start: str
    end: str
    files: list[DatasetFile]

__version__: str

def parse_price(text: str, decimals: int, /) -> int:
    """精确解析十进制字符串为定点整数（不经过浮点）。

    失败抛 ValueError，消息以稳定错误码开头（FixedParse: 等）。
    """
    ...

def run_mode_rules(mode: str, /) -> tuple[bool, int | None, int]:
    """各运行模式的账户/策略规则（ADR-0001）。

    返回 (可加载密钥, 每交易所最大账户数, 每账户策略数)。
    """
    ...

def load(dataset_hash: str, root: str = ".", /) -> DatasetInfo:
    """按数据集哈希加载清单（docs/06 §8）。

    返回 {hash, name, min_grade, start, end, files: [{table, symbol,
    date, path}]}；文件按 表→品种→日期 排序。失败抛 ValueError，
    消息以 DataUsage 错误码开头。
    """
    ...
