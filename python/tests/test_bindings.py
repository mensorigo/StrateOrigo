"""strateorigo 包的基础测试：绑定可用、运行模式规则与 ADR-0001 一致。"""

import pytest

from strateorigo import parse_price, run_mode_rules


def test_parse_price_exact() -> None:
    price_raw = 650005  # 65000.5，精度 1
    assert parse_price("65000.5", 1) == price_raw
    negative_raw = -5  # -0.05，精度 2
    assert parse_price("-0.05", 2) == negative_raw
    with pytest.raises(ValueError, match="FixedExcessPrecision:"):
        parse_price("1.234", 2)
    with pytest.raises(ValueError, match="FixedParse:"):
        parse_price("1e5", 2)


@pytest.mark.parametrize(
    ("mode", "may_keys", "max_acc"),
    [
        ("backtest", False, None),
        ("paper", False, None),
        ("demo", True, 1),
        ("live", True, 1),
    ],
)
def test_run_mode_rules_adr_0001(mode: str, may_keys: bool, max_acc: int | None) -> None:
    may_keys_got, max_acc_got, max_strats = run_mode_rules(mode)
    assert may_keys_got is may_keys
    assert max_acc_got == max_acc
    assert max_strats == 1  # 任何模式：一个账户一个策略


def test_run_mode_rules_invalid() -> None:
    with pytest.raises(ValueError, match="UnknownRunMode:"):
        run_mode_rules("LIVE")
