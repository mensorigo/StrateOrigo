"""strateorigo.data.load 与 asof_join 的端到端测试（docs/06 §8）。

在临时目录造一个最小数据集（清单 + 一个 Parquet 占位），验证
哈希校验、文件清单与错误分支；不依赖真实 normalize 产物。
"""

from pathlib import Path

import polars as pl
import pytest

from strateorigo import data


def _make_dataset(root: Path, name: str = "t") -> str:
    """在 root 下造占位清单（文件名与内容必然不符，用于负路径测试）。

    完整写→读往返（哈希一致的正路径）由 Rust 侧 dataset_in 测试覆盖，
    同一实现两端共用。
    """
    ds_dir = root / "data" / "datasets"
    ds_dir.mkdir(parents=True, exist_ok=True)
    manifest = (
        f'name = "{name}"\n'
        'normalize_version = "n1"\n'
        'clean_version = "c1"\n'
        'min_grade = "A"\n'
        'venues = ["gate"]\n'
        'symbols = ["BTC_USDT"]\n'
        'tables = ["trade"]\n'
        'start = "2023-11-14T00:00:00Z"\n'
        'end = "2023-11-14T23:59:59Z"\n'
        'files_digest = "sha256:0000"\n'
    )
    fake_hash = "0" * 32
    (ds_dir / f"{fake_hash}.toml").write_text(manifest, encoding="utf-8")
    return fake_hash


def test_load_missing_dataset(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="DataUsage"):
        data.load("f" * 32, str(tmp_path))


def test_load_detects_tampered_manifest(tmp_path: Path) -> None:
    fake_hash = _make_dataset(tmp_path)
    # 文件名与内容不符 → 拒绝（手改防护）。
    with pytest.raises(ValueError, match="DataUsage"):
        data.load(fake_hash, str(tmp_path))


def test_load_bad_hash_format(tmp_path: Path) -> None:
    # 不存在 → Usage 错误码稳定。
    with pytest.raises(ValueError, match="DataUsage"):
        data.load("not-a-hash", str(tmp_path))


def test_asof_join_forward_fill() -> None:
    """as-of join：左表每行取右表 ts_recv <= 左表 ts_recv 的最近行。"""
    left = pl.LazyFrame(
        {
            "ts_recv": [100, 200, 300],
            "symbol": ["BTC", "BTC", "BTC"],
        }
    )
    right = pl.LazyFrame(
        {
            "ts_recv": [50, 150, 250],
            "symbol": ["BTC", "BTC", "BTC"],
            "close": [100.0, 105.0, 110.0],
        }
    )
    result = data.asof_join(left, right, by="symbol").collect()
    assert result["close"].to_list() == [100.0, 105.0, 110.0]


def test_asof_join_by_symbol_no_cross_match() -> None:
    """不同 symbol 不匹配（ETH 的左行不拿 BTC 的右行）。"""
    left = pl.LazyFrame(
        {
            "ts_recv": [100],
            "symbol": ["ETH"],
        }
    )
    right = pl.LazyFrame(
        {
            "ts_recv": [50],
            "symbol": ["BTC"],
            "close": [100.0],
        }
    )
    result = data.asof_join(left, right, by="symbol").collect()
    assert result["close"].to_list() == [None]
