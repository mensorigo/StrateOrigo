# CLAUDE.md

StrateOrigo：Rust + Python 量化交易机器人。重心是**数据获取与清洗 → 策略 → 交易执行与风控**。当前处于设计阶段。

## 必读
- 总览与优先级：[docs/README.md](docs/README.md)
- **强制规则**：[docs/04-dev-constraints.md](docs/04-dev-constraints.md)（热路径、确定性、安全、Agent 协作）
- 数据：[docs/06-data.md](docs/06-data.md) · 策略：[docs/07-strategies.md](docs/07-strategies.md)
- 当前里程碑与验收标准：[docs/05-roadmap.md](docs/05-roadmap.md)

## 关键规则速查
- **不做任何 Web 页面**（Web 控制台、Grafana、MLflow UI、JupyterLab、plotly/HTML 报告）。交互只用 `so-cli`、ratatui 终端界面、Telegram。GPUI 桌面端为 P2，M9 之前不得引入 GPUI 依赖；但客户端需要的数据（含 K 线）必须先以 gRPC 接口（VPS 交易域）或 `so-lab`（本地研究域）提供，见 docs/03 §11。
- 服务端完全无头，服务端二进制不得依赖任何 GUI crate。
- 热路径 crate（so-core / so-book / so-engine / so-risk / so-oms / so-signal / so-exec / HF 策略）：不分配堆内存、不加锁、不阻塞、不用 tokio、价格数量用定点 `Price`/`Qty`、不 panic、时间只用 `Clock`。
- 回测必须确定性；随机数只用显式种子的 RNG。
- 运行模式 `backtest` / `paper` / `demo` / `live` 共用同一内核，内核中不得出现按模式分支。**一个账户一个策略**；`live`/`demo` 启动时校验"每个交易所 1 个账户、共 1 个策略"；`backtest`/`paper` 不得加载任何密钥（ADR-0001、docs/04 S8–S11）。
- 数据层：原始数据不可变；清洗只加标记不删原始数据；真实插针不得删除；回测只引用数据集哈希。
- 不得修改 `configs/env/live.toml`、`configs/risk/**`、密钥文件；不得连接实盘账户、部署 VPS、推送远程仓库（除非当次明确授权）。
- 写交易所适配器前先把当前官方文档整理到 `docs/venues/`，不凭记忆写字段名和规则。
- 每次改动后：`cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo nextest run`；Python：`ruff check`、`ruff format`、`pyright`、`pytest`。
- 遇到文档未覆盖的设计决策：写 ADR 到 `docs/decisions/` 并询问，不要自行决定。
