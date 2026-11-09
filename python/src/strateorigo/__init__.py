"""strateorigo：研究环境包（入口模块只做重新导出，docs/04 F4）。"""

from strateorigo._core import parse_price, run_mode_rules

__version__ = "0.1.0"

__all__ = ["__version__", "parse_price", "run_mode_rules"]
