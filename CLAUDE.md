# CLAUDE.md

StrateOrigo：Rust + Python 量化交易机器人。重心是**数据获取与清洗 → 策略 → 交易执行与风控**。当前处于 M2 数据清洗与存储阶段（数据采集、订单簿重建验证、Raw Journal → Parquet 已落地）。

**一键检查**：`powershell -ExecutionPolicy Bypass -File scripts/check.ps1`（fmt / clippy / nextest / 文件行数 / deny / ruff / pyright / pytest）。

**规则的唯一事实来源是 [docs/04-dev-constraints.md](docs/04-dev-constraints.md)**；下面的速查只做指针，冲突时以 04 为准（并应删掉本文件的重复描述）。

## 必读
- 总览与优先级：[docs/README.md](docs/README.md)
- **强制规则**：[docs/04-dev-constraints.md](docs/04-dev-constraints.md)（§0 速查：H 热路径 / D 确定性 / F 文件 / E 错误 / I 实现方法 / A 分层解耦 / L 日志 / T 测试 / S 安全 / Agent 协作）
- 仓库结构与依赖分层：[docs/03-module-design.md](docs/03-module-design.md) §1
- 数据：[docs/06-data.md](docs/06-data.md) · 策略：[docs/07-strategies.md](docs/07-strategies.md)
- 当前里程碑与验收标准：[docs/05-roadmap.md](docs/05-roadmap.md) · 待办台账：[docs/pending-items.md](docs/pending-items.md)

## 关键规则速查
- **不做任何 Web 页面**（Web 控制台、Grafana、MLflow UI、JupyterLab、plotly/HTML 报告）。交互只用 `so-cli`、ratatui 终端界面、Telegram。GPUI 桌面端为 P2，M9 之前不得引入 GPUI 依赖；但客户端需要的数据（含 K 线）必须先由服务端接口（VPS 交易域）或 `so-lab`（本地研究域）提供，见 docs/03 §11、docs/04 §5.4。
- 服务端完全无头，服务端二进制不得依赖任何 GUI crate。
- 热路径 crate（so-core / so-book / so-engine / so-risk 的事前风控 / so-oms / so-signal / so-exec / HF 策略）：不分配堆内存、不加锁、不阻塞、不用 tokio、价格数量用定点 `Price`/`Qty`、不 panic、时间只用 `Clock`（docs/04 H1–H10）。
- 回测必须确定性；随机数只用显式种子的 RNG（docs/04 D1–D5）。
- **分层解耦**（docs/04 A1–A7、§3.5）：`层0` so-core → `层1` model/telemetry/py → `层2` 领域组件 → `层3` engine/data/recorder → `层4` adapters/lab/proto → `层5` apps；依赖只能由高编号层指向低编号层，同层依赖须登记例外表。`so-engine` 不得依赖适配器/tokio/网络库；策略 crate 只能依赖 `so-strategy`/`so-signal`/`so-exec`。
- **文件划分**（docs/04 F1–F7）：单文件 ≤ 400 行（超 250 行警告）；每个文件只负责一件事，开头写明职责；解析、计算、显示、I/O、配置、错误分文件；`lib.rs`/`mod.rs`/`__init__.py` 不写逻辑；函数 ≤ 60 行；不准用 `utils`/`helpers`/`common` 这类文件名。
- **错误处理**（docs/04 E1–E9）：每域一个 thiserror 枚举；错误码 `<域前缀><变体>`（如 `DataJournal`、`GateFixed`），登记表在 docs/04 §3.3，**只增不改名**；上下文用命名字段不拼字符串；程序判别只匹配错误码、不匹配中文消息；gRPC/Python 映射只在 so-proto / so-py 边界做；热路径错误只返回或降级，不重试阻塞；退出码只用 0/1/2/3。
- **实现方法**（docs/04 I1–I13）：枚举表模式（`ALL`/`as_str`/`from_str`）；受限构造（私有字段 + `new()` 校验）；请求结构体 + 统计结构体；内核内多态用枚举、边界才用 `dyn`；编排层不做算法、算法层不做 I/O。
- **日志**（docs/04 L1–L6）：热路径写 `LogRecord` 到 SPSC，不调 `tracing` 宏；不按 tick 打 INFO；**记录错误必须带 `code` 字段**；字段 `snake_case`、时间带 `_ts`、耗时带 `_ns`。
- 运行模式 `backtest` / `paper` / `demo` / `live` 共用同一内核，内核中不得出现按模式分支。**一个账户一个策略**；`live`/`demo` 启动时校验"每个交易所 1 个账户、共 1 个策略"；`backtest`/`paper` 不得加载任何密钥（ADR-0001、docs/04 S8–S11）。
- 数据层：原始数据不可变；清洗只加标记不删原始数据；真实插针不得删除；回测只引用数据集哈希；**静默跳过或丢弃的数据视为 bug，必须计数并出现在日志与质量报告里**。
- 不得修改 `configs/env/live*.toml`、`configs/risk/**`、密钥文件；不得连接实盘账户、部署 VPS、推送远程仓库（除非当次明确授权）。
- 写交易所适配器前先把当前官方文档整理到 `docs/venues/`（docs/04 §5.5），不凭记忆写字段名和规则。
- 每次改动后：`cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo nextest run`；Python：`ruff check`、`ruff format`、`pyright`、`pytest`。
- 遇到文档未覆盖的设计决策：写 ADR 到 `docs/decisions/` 并询问，不要自行决定。发现"文档 vs 代码"不一致时先判断哪边对，只改一边或列入 `docs/pending-items.md`，**不要两边各改一半**（docs/04 §5.6）。
