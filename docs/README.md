# StrateOrigo 开发文档

> Rust + Python 量化交易机器人。
> 文档版本：v0.4（2026-11-11） · 状态：M2 数据清洗与存储进行中（数据采集、订单簿重建验证、Raw Journal → Parquet 已落地）

## 先读哪一篇

| 你的身份 / 任务 | 先读 |
|---|---|
| 第一次接触项目 | 本篇 → [01-architecture.md](01-architecture.md) |
| **要写代码**（人或 Agent） | [04-dev-constraints.md](04-dev-constraints.md)（**强制规则**：§0 速查、错误码登记表、层级表） |
| 不确定某模块放哪 / 谁能依赖谁 | [03-module-design.md](03-module-design.md) §1（仓库结构 + 依赖图） |
| 做数据相关的工作 | [06-data.md](06-data.md)（§9.1 是实现约定） |
| 做策略相关的工作 | [07-strategies.md](07-strategies.md)（§8 是代码规范） |
| 查版本基线 | [02-tech-stack.md](02-tech-stack.md) |
| 查当前该做什么 | [05-roadmap.md](05-roadmap.md) + [pending-items.md](pending-items.md) |
| 查为什么这样定 | [decisions/](decisions/)（ADR） |

## 一句话定位

**一个以交易为核心的量化交易机器人：把数据获取和清洗做扎实，把策略研究和验证做严谨，让同一套 Rust 确定性内核驱动回测、模拟盘和实盘，并由独立风控兜底。**

## 优先级

| 优先级 | 范围 |
|---|---|
| **P0 核心** | 数据获取 → 数据清洗 → 策略研究/回测/验证 → 交易执行 → 风控 |
| **P1 支撑** | 命令行工具（`so-cli`）、终端监控、Telegram 告警、部署运维 |
| **P2 低优先级** | GPUI 桌面端（交易系统实盘稳定之后才开始） |
| **不做** | 任何 Web 页面：Web 控制台、Grafana、MLflow UI、浏览器版 Jupyter、HTML 报告 |

## 文档目录

| 文档 | 内容 |
|---|---|
| [01-architecture.md](01-architecture.md) | 目标与范围、整体架构、进程拓扑、逻辑图、热路径、风控分层 |
| [02-tech-stack.md](02-tech-stack.md) | 技术栈、组件库、版本基线与锁定策略 |
| [03-module-design.md](03-module-design.md) | 仓库结构（含依赖层级）、各模块职责与核心接口、日志与可观测性 |
| [04-dev-constraints.md](04-dev-constraints.md) | **强制规则**：H 热路径 · D 确定性 · F 文件 · E 错误 · I 实现方法 · A 分层解耦 · L 日志 · T 测试 · S 安全 · Agent 协作 |
| [05-roadmap.md](05-roadmap.md) | 里程碑、交付物与验收标准 |
| [06-data.md](06-data.md) | **数据获取与清洗**：数据源、采集、规范化、清洗规则、存储、质量控制、实现约定 |
| [07-strategies.md](07-strategies.md) | **交易策略**：策略类别、研究流程、首批策略、交易判断、成本与执行、风险审核、评估指标 |
| [pending-items.md](pending-items.md) | 未完成 / 暂后 / 挂起决策的台账 |
| [decisions/](decisions/) | 架构决策记录（ADR） |
| [venues/](venues/) | 各交易所 API 要点整理（写适配器前必须先更新，见 [04 §5.5](04-dev-constraints.md)） |

## 规则体系（这套文档怎么用）

1. **`docs/04` 是规则的唯一事实来源**。本文件与 `CLAUDE.md` 只写指针与速查；重复描述一旦与 04 冲突，以 04 为准，并应删掉重复描述。
2. **规则用编号引用，编号只增不改**：`docs/04 H8` 永远指向同一条规则；废弃规则标删除线并注明替代项。因此**代码注释只引用编号，不引用章节号或行号**——章节会重排，行号会失效。
3. **文档与代码不一致时先判断哪边对**：规则文档错了 ⇒ 改文档；代码错了 ⇒ 改代码或列入 [pending-items.md](pending-items.md)。**不要两边各改一半**（详见 [04 §5.6](04-dev-constraints.md)）。
4. **文档未覆盖的设计决策**：写 ADR 到 `docs/decisions/` 并询问，不自行决定（[04 §5.1](04-dev-constraints.md)）。
5. **完成一项待办**：在 [pending-items.md](pending-items.md) 把该行改成 ✅ 并注明日期，不要删行。

## 核心决策摘要

| 决策 | 选择 | 理由 |
|---|---|---|
| 核心语言 | Rust（稳定版 1.99.1，edition 2024） | 低延迟、内存安全、无 GC 抖动 |
| 研究语言 | Python 3.12 | 数据科学生态；与本机环境一致 |
| 回测/实盘一致性 | 同一个确定性事件内核 `so-engine`，只替换数据源/时钟/执行端 | 消除"回测与实盘两套逻辑"的最大误差来源 |
| 高频回测保真度 | L2 订单簿 + 延迟模型 + 排队位置模型 | 没有这三者的高频回测无参考价值 |
| 价格/数量表示 | 定点整数 `i64` + 品种精度 | 避免浮点误差，热路径零分配 |
| 风控 | 三层：事前（引擎内同步）→ 组合（引擎内实时）→ 守护进程（独立进程 + 独立 API Key） | 策略或引擎崩溃时仍能止损 |
| 人机交互 | `so-cli` 命令行 + 终端实时监控（ratatui）+ Telegram 告警与命令 | 无 Web 页面；VPS 上 SSH 即可操作 |
| 实验追踪 | 自研：SQLite 元数据 + Parquet 结果 | 不依赖带 Web UI 的 MLflow；查询用 Python / CLI |
| 研究环境 | VS Code Notebook（ipykernel）+ matplotlib 静态图 | 不用浏览器 |
| 桌面端（P2） | GPUI Kit：`gpui-component =0.9.0` + `gpui-pre =0.3.11` | 实盘稳定后再做 |
| 控制面 | gRPC（tonic），仅经 WireGuard/Tailscale 访问 | 强类型；CLI 与将来的桌面端共用 |
| 市场与频率 | 加密货币：高频/中频；股票：中低频 | 个人接入股票市场的延迟与监管不支持高频 |
| 交易所 | **OKX（主）+ Gate**；首先接入 OKX 永续（SWAP）+ 现货 | 按需求确定；Binance / Bybit 暂不接入 |
| 运行模式 | `backtest` / `paper`（本地模拟盘）/ `demo`（交易所模拟环境）/ `live`，同一内核；实盘与模拟盘可同时运行 | 模拟盘验证策略，demo 验证交易流程，再上实盘 |
| 账户与策略 | **一个账户一个策略**；实盘进程 = 1 个真实账户 + 1 个策略（启动时校验）；多策略验证用模拟盘的多个虚拟账户（ADR-0001） | 对账与归因最简单；进程和资金天然隔离 |
| 多币种 | 同一份代码、每个币种单独参数与单独验证；币数由验证结果决定 | 多币种不自动分散风险，选币本身也是过拟合 |
| **编码规范** | 规则编号 H/D/F/E/I/A/L/T/S（[04](04-dev-constraints.md)）；**唯一事实来源是 04**，其他文档只做指针 | 规范只有一处会改，避免"文档 A 与文档 B 不一致" |
| 错误码 | `<域前缀><变体>`（如 `DataJournal`、`GateFixed`），登记表在 [04 §3.3](04-dev-constraints.md) | 跨语言（Python / gRPC / 审计）程序判别只看这一个字符串 |
| 依赖分层 | `层0` so-core → `层1` model/telemetry/py → `层2` 领域组件（含 `so-adapter-gate`、`so-journal`）→ `层3` engine/data/recorder → `层4` 边界 → `层5` apps，**只能高→低依赖**（[04 §3.5](04-dev-constraints.md)） | 解耦靠可机检的方向约束（`scripts/check-deps.ps1`），不靠自觉 |

## 术语

- **热路径（Hot Path）**：行情到达 → 订单簿更新 → 策略决策 → 事前风控 → 下单发出 这一段，延迟敏感。
- **冷路径**：日志落盘、指标、控制面、持久化、对账等，不影响下单延迟。
- **Tick-to-Trade**：从行情包到达网卡到订单包发出的内部耗时（不含网络往返）。
- **确定性**：相同数据 + 相同代码版本 + 相同参数 + 相同随机种子 ⇒ 逐位相同的结果。
- **依赖层级（`层0`–`层5`）**：见 [04 §3.5](04-dev-constraints.md)。**不要与领域级别缩写混用**：风控的 L1/L2/L3 指执行位置，订单簿的 L1/L2 指深度，回测保真度的 L0/L1/L2 指撮合精细度。
- **域（Domain）**：一个 crate 所拥有并定义其错误类型、配置与规则的范围；[04 E1](04-dev-constraints.md)"每个域一个错误枚举"中的"域"即 crate 级。
