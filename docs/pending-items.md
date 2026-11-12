# 未完成与暂后事项记录

> 快照时间：2026-11-12（ADR-0015 规范化 + ADR-0016 重构 + ADR-0017 编号去歧义之后）。每完成一项请把该行改为 ✅ 并注明完成日期；
> 新出现的暂后事项按同样格式追加。里程碑总体进度见 [05-roadmap.md](05-roadmap.md)。

## 1. M1 未完成（阻塞项）

| # | 事项 | 阻塞原因 | 解锁后要做什么 |
|---|------|----------|----------------|
| 1 | VPS 实测验收（docs/05 M1 验收标准） | 需部署 VPS；规则要求**当次明确授权**才连接/部署 | 连续录制 20 品种 **1 天**（原 7 天，2026-11-10 调整）；静默/缺口检测告警实收验证；解析基准 ≥ 50 万条/秒（工具 `bench-parse` 已就绪，d0d7336）；存储量与延迟分布报告 |
| 2 | Telegram 告警**成功路径**实收验证 | **用户指示：留到最后，不阻塞 M1 收尾**（2026-11-12） | 本地已验证失败路径（假 token → 401 → 退避重试 → 丢弃，005c017 冒烟）；成功路径实收推迟到最后的整体验证阶段。M1 验收项 2 中"检测、记录"部分仍随 VPS 录制一并看日志验证，仅"Telegram 实收"这一下留到最后。ADR-0013 的 4 点确认随之同延 |

## 2. ADR-0015 带来的代码整改

规则已写入 [04](04-dev-constraints.md)。第 7–10、12 项**已于 2026-11-11 完成**（见各行 ✅）；
第 11 项（Python 异常映射）按计划留到 M4 前。

| # | 事项 | 依据 | 具体动作 |
|---|------|------|----------|
| 7 | ✅ 2026-11-11 `FixedError` 错误码加域前缀 | 04 E2 | `Overflow → FixedOverflow`、`DivisionByZero → FixedDivisionByZero`、`InvalidScale → FixedInvalidScale`、`InvalidStep → FixedInvalidStep`、`Parse → FixedParse`、`ExcessPrecision → FixedExcessPrecision`；同步 `crates/so-core/src/error.rs` 的 `code()`、`crates/so-py/src/version.rs` 的文档示例、04 §3.3 登记表 |
| 8 | ✅ 2026-11-11 错误日志补 `code` 字段 | 04 G4（原 L4） | 把只记 `error = %e` 的地方改为 `code = e.code()`：至少 `apps/so-cli/src/data_normalize.rs`（读配置失败、扫描目录失败）、`apps/so-cli/src/data_download.rs`（tokio 运行时创建失败）；改完全仓复查一遍 |
| 9 | ✅ 2026-11-11 依赖分层机检脚本 | 04 A7 | 新增 `scripts/check-deps.ps1`：读 `cargo metadata`，按 04 §3.5 层级表与例外表判定依赖边，出现表外边即非零退出；接入 `scripts/check.ps1` |
| 10 | ✅ 2026-11-11 journal 归属下沉（`so-journal`） | [ADR-0016](decisions/ADR-0016-journal-crate-extraction.md) | 已建 `crates/so-journal`（层2：`JournalRow`、帧格式、读写、sha256、`Dedup`、`JournalError`）；`so-recorder` 与 `so-data` 各自依赖它，同层依赖已消除。`so-data` 生产依赖图 204 → 95 个 crate；Parquet 输出逐字节不变（聚合哈希 `94d1ffb0…`） |
| 11 | Python 异常映射落地 | 04 E3 | 新建 `python/src/strateorigo/errors.py`（`{code: ExceptionType}` 映射 + 前缀解析）；加测试断言映射表与 Rust 侧错误码登记表集合一致 |
| 12 | ✅ 2026-11-11 `TelemetryError` 补 `code()` | 04 E2 | 登记表已登记 `TelemetryFilter` / `TelemetryFile` / `TelemetryInit`，代码尚未实现 `code()`；实现后 so-telemetry 的错误才能用于日志聚合（04 G4，原 L4） |

### 2.1 编号去歧义（ADR-0017，2026-11-12 已完成）

| # | 事项 | 依据 | 说明 |
|---|------|------|------|
| 16 | ✅ 2026-11-12 日志规则 `L1–L6` → `G1–G6` | [ADR-0017](decisions/ADR-0017-rule-id-disambiguation.md) | 规则编号里不再有 `L`；语义与编号值一一对应（`L4`→`G4` 仍是"错误日志必带 `code`"） |
| 17 | ✅ 2026-11-12 领域级别加前缀 | 同上 | 风控 `RK-L1/L2/L3`、订单簿 `BK-L1/L2`、回测保真度 `BT-L0/L1/L2`；代码标识符（`book::L1` 视图类型）不变 |
| 18 | ✅ 2026-11-12 mermaid 节点 ID 去规则化 | 同上（附带发现） | `docs/01 §10` 风控图子图 ID 改 `RKL1/2/3`；`docs/07` 两图节点 `L0/L12`→`VEC/EVT`、`G1/G2/G3`→`GATE1/2/3`——后者与新的 `G` 日志规则撞名 |

### 2.2 本次整改中新发现、待处理

| # | 事项 | 依据 | 说明 |
|---|------|------|------|
| 14 | `so-py/src/version.rs` 的 `pyo3` 绑定未同步 `Python` 侧异常映射 | 04 E3 | 现状消息前缀已随错误码改为 `FixedParse:` / `FixedExcessPrecision:`（`python/tests/test_bindings.py` 已同步）；类型映射仍待第 11 项一并落地 |
| 15 | `scripts/check-deps.ps1` 的层级表解析依赖 04 §3.5 的 markdown 形状 | 04 A7 | 表格被重排（如换成非 `层N` 标签、多列合并）会让解析失效并报"没解析出任何层级登记"；改 §3.5 表格后须跑一次该脚本 |
| 19 | ✅ 2026-11-12 `so-py` 层位修正：层1 → **层4** | [ADR-0018](decisions/ADR-0018-universe-and-symbol-selection.md) / 04 §3.5 | PyO3 绑定**是边界**（与 `so-proto` 同类）。原写层1 会让"Python 调 `so-data`/`so-backtest`"（03 §9.1、05 M3）变成 `层1 → 层3` 方向违规。**该缺陷当时机检抓不到**：`so-py` 目前只依赖 `so-core`，层1 恰好合法——违规要等 M3 它依赖 `so-data` 才暴露。图与层级表冲突时以机检一方为准，但**先判断机检是否只是"还没到暴露时机"** |
| 20 | ✅ 2026-11-12 策略 crate `st-*` 登记层级（**层3**） | 同上 | 04 §3.5 表里原先**完全没有** `st-*`，图却把它画在层2 框里：`st-mm-basic` 落地当天 check 1 会直接报"未登记"。现登记为层3（依赖层2 的 `so-strategy`/`so-signal`/`so-exec`）。**根因是解析器只认 `so-*`/`apps*` 前缀，`st-*` 会被静默丢弃**——已修 `check-deps.ps1` |
| 21 | ✅ 2026-11-12 三条规划同层边登记 + 例外表支持"休眠" | 04 A1/A7 | `so-backtest → so-engine`、`so-backtest → so-data`、`so-exec → so-strategy`（均未建）已登记，标 `规划`。`check-deps.ps1` 原判"两端不存在 ⇒ 过期"，会与"未建先登记"直接冲突；现对两端未齐的**规划**行休眠、两端一建立自动转入机检（不需要回头补登记）。`so-lab → so-backtest` 判为**画错**，已从图中删除（读实验库/数据集不需要跑回测） |
| 22 | ✅ 2026-11-12 `so-engine` 的 A3 网络护栏机检 | 04 A3 | `check-deps.ps1` 新增检查 4b：`so-engine` 一旦存在，依赖里出现 `tokio`/`hyper`/`reqwest`/`tokio-tungstenite`/`rustls` 等即失败；未建时跳过。**已用"临时把 `so-recorder` 改名为 `so-engine`"注入验证**，`reqwest`/`tokio`/`tokio-tungstenite` 全部报出，还原后通过 |
| 23 | ✅ 2026-11-12 F1 行数检查纳入 Python | 04 F1 | `check.ps1` 原只扫 `crates`/`apps` 的 `*.rs`，Python 侧形同豁免；现扫 `crates`/`apps`/`python` 的 `*.rs`+`*.py`，并排除 `target`/`.venv`/`__pycache__` |
| 24 | ✅ 2026-11-12 F5 的 Python 阈值与文档对齐 | 04 F5 | 04 F5 写明 ruff `PLR0915` 最多 **40** 条语句，而 `pyproject.toml` 未配 `max-statements`（ruff 默认 50）——文档与配置不一致。已配 `max-statements = 40`，并用 41/40 条语句两个探针验证阈值确为 40 |
| 25 | ✅ 2026-11-12 `04 §5.7` 补"编号只增不改"的例外口径 | ADR-0017 | 04 §5.7 原写"编号只增、不改、不复用"（绝对措辞），与 ADR-0017 已执行的 `L`→`G` 改名自相矛盾。现写明**唯一一次例外**的条件（先写 ADR / 只动字面量 / 一次性改净不留双编号） |
| 26 | ✅ 2026-11-12 L→G 改名**执行不全**的 11 处残留已补 | ADR-0017 / 04 §5.7 | `docs/06 §1` 的"L2 深度增量"、`docs/07 §5.3` 的"L1 风控"、`docs/venues/gate.md` 5 处、`docs/venues/okx.md` 4 处（均为**本项目自己的**风控 `RK-L1` / 订单簿 `BK-L2`）；ADR-0017 声明"已实施"时这些还在。**上次漏掉的原因**：复查用的 grep 模式只覆盖"已改写过的形状"，命中 0 就当成干净（等于用产生结论的假设去验证结论）；**正确做法**是先按形状枚举（`(?<![A-Za-z\-])L\d`）再逐条分类 |
| 27 | ✅ 2026-11-12 录制面三模式已定（`list`/`top`/`all`，默认 `list`） | [ADR-0018](decisions/ADR-0018-universe-and-symbol-selection.md) | 05 M1"前 20"vs 配置 10 个的矛盾随之消解：**默认按 `list` 用当前名单**；20 只是 `top` 模式的一个参数取值，不是验收前提。残留小项：① `unsubscribe` 帧已定义（`ws.rs`）但**无调用点**，做 `top` 时补；② `all` 模式启用前须核实单连接订阅上限并补进 `gate.md`（04 §5.5） |
| 28 | ⚠ 验收方式提醒：违规注入测试**必须每例之间还原** | 本次自省 | 我第一次写的脚本让用例相互累积，输出里满是前例的遗留违规，**看起来"有牙"实则 E/D 两例根本没触发**。教训：验证脚本自身要有"每例干净起点 + 锚点命中断言"，否则测的是自己的 bug |
| 29 | ✅ 2026-11-12 S2 维持跨品类原设计 | 用户拍板"按照原来默认的" | 讨论过"单策略不跨玩法品类"会废除 S2（现货多+永续空）；结论：**不引入该限制**。07 §1 分类维度是优势来源而非市场类型；delta 中性两条腿不可拆（ADR-0001 亦不允许跨进程协调）。已记入 ADR-0018「已讨论并明确不做的事」 |
| 30 | ✅ 2026-11-12 实时触发定为方案 A（判定进内核） | ADR-0018 §5.1 | 用户拍板。冷热分工：每小时全量排名在冷路径（产出定长候选集事件）；热路径只对候选集槽位做阈值判定+滞回（f64 比较、位图置位）。B（独立选品线程）被否：回测里无法重放真实时钟，且 live/backtest 行为分叉违反"内核不得按模式分支"。节拍随 `futures.tickers` 1s 推送；候选集大小 K 为配置项。**实施时**：`so-selector` 的热路径部分须过 H1–H10 逐条核查（同 04 §1 适用 crate 表） |

## 3. 暂后事项（有意推迟，含原因）

| # | 事项 | 来源 | 说明 |
|---|------|------|------|
| 3 | OKX 适配器（so-adapter-okx） | 用户指示："gate 的实现优先，okx 可以后置" | 启动前须先把 OKX 官方文档整理到 `docs/venues/`（04 §5.5），不凭记忆写字段名 |
| 4 | `docs/03 §5` OKX checksum 描述疑似过时 | 前期记录，未核对 | 写 OKX 适配器前整理官方文档时一并核对修订 |
| 5 | `docs/06 §4.1` OKX trades 聚合规则待核对 | 前期记录，未核对 | 同上 |

## 4. 挂起决策（ADR 待用户确认）

| # | 决策 | 状态 | 待确认点 |
|---|------|------|----------|
| 6 | [ADR-0013](decisions/ADR-0013-telegram-alert-transport.md)：Telegram 告警传输与重试策略 | ⏳ 待确认（2026-11-10 提出） | ① 直调 Bot API 而非 teloxide；② 无界队列非阻塞入队；③ 5 次退避重试后丢弃（与 03 §18.1 告警"不可丢"的取舍）；④ 非交易进程前缀用 `[REC]` |
| 13 | [ADR-0015](decisions/ADR-0015-style-and-decoupling-standard.md)：编码风格与解耦规范 | ✅ 全部已定（第 9 项 2026-11-11 由 ADR-0016 落定） | — |

> ADR-0013 的第 ④ 点已在 04 S10 落定：前缀集合为 `[LIVE]`/`[PAPER]`/`[DEMO]`/`[REC]`，录制器日志记 `mode = recorder`。

## 5. 不适用（明确排除，防止误做）

| # | 事项 | 原因 |
|---|------|------|
| — | Web 控制台 / Grafana / JupyterLab 等任何 Web 页面 | docs/04 强制规则（§5.4）：交互只用 so-cli、ratatui、Telegram |
| — | GPUI 桌面端 | P2，M9 前不得引入 GPUI 依赖 |
