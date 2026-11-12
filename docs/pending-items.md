# 未完成与暂后事项记录

> 快照时间：2026-11-11（ADR-0015 文档规范化 + ADR-0016 重构之后）。每完成一项请把该行改为 ✅ 并注明完成日期；
> 新出现的暂后事项按同样格式追加。里程碑总体进度见 [05-roadmap.md](05-roadmap.md)。

## 1. M1 未完成（阻塞项）

| # | 事项 | 阻塞原因 | 解锁后要做什么 |
|---|------|----------|----------------|
| 1 | VPS 实测验收（docs/05 M1 验收标准） | 需部署 VPS；规则要求**当次明确授权**才连接/部署 | 连续录制 20 品种 **1 天**（原 7 天，2026-11-10 调整）；静默/缺口检测告警实收验证；解析基准 ≥ 50 万条/秒（工具 `bench-parse` 已就绪，d0d7336）；存储量与延迟分布报告 |
| 2 | Telegram 告警**成功路径**实收验证 | 需真实 bot token 与 chat_id（用户提供，不落盘） | 本地已验证失败路径（假 token → 401 → 退避重试 → 丢弃，005c017 冒烟）；VPS 验收时用真实 token 确认告警到达 |

## 2. ADR-0015 带来的代码整改

规则已写入 [04](04-dev-constraints.md)。第 7–10、12 项**已于 2026-11-11 完成**（见各行 ✅）；
第 11 项（Python 异常映射）按计划留到 M4 前。

| # | 事项 | 依据 | 具体动作 |
|---|------|------|----------|
| 7 | ✅ 2026-11-11 `FixedError` 错误码加域前缀 | 04 E2 | `Overflow → FixedOverflow`、`DivisionByZero → FixedDivisionByZero`、`InvalidScale → FixedInvalidScale`、`InvalidStep → FixedInvalidStep`、`Parse → FixedParse`、`ExcessPrecision → FixedExcessPrecision`；同步 `crates/so-core/src/error.rs` 的 `code()`、`crates/so-py/src/version.rs` 的文档示例、04 §3.3 登记表 |
| 8 | ✅ 2026-11-11 错误日志补 `code` 字段 | 04 L4 | 把只记 `error = %e` 的地方改为 `code = e.code()`：至少 `apps/so-cli/src/data_normalize.rs`（读配置失败、扫描目录失败）、`apps/so-cli/src/data_download.rs`（tokio 运行时创建失败）；改完全仓复查一遍 |
| 9 | ✅ 2026-11-11 依赖分层机检脚本 | 04 A7 | 新增 `scripts/check-deps.ps1`：读 `cargo metadata`，按 04 §3.5 层级表与例外表判定依赖边，出现表外边即非零退出；接入 `scripts/check.ps1` |
| 10 | ✅ 2026-11-11 journal 归属下沉（`so-journal`） | [ADR-0016](decisions/ADR-0016-journal-crate-extraction.md) | 已建 `crates/so-journal`（层2：`JournalRow`、帧格式、读写、sha256、`Dedup`、`JournalError`）；`so-recorder` 与 `so-data` 各自依赖它，同层依赖已消除。`so-data` 生产依赖图 250 → 95 个 crate；Parquet 输出逐字节不变（聚合哈希 `94d1ffb0…`） |
| 11 | Python 异常映射落地 | 04 E3 | 新建 `python/src/strateorigo/errors.py`（`{code: ExceptionType}` 映射 + 前缀解析）；加测试断言映射表与 Rust 侧错误码登记表集合一致 |
| 12 | ✅ 2026-11-11 `TelemetryError` 补 `code()` | 04 E2 | 登记表已登记 `TelemetryFilter` / `TelemetryFile` / `TelemetryInit`，代码尚未实现 `code()`；实现后 so-telemetry 的错误才能用于日志聚合（04 L4） |

### 2.1 本次整改中新发现、待处理

| # | 事项 | 依据 | 说明 |
|---|------|------|------|
| 14 | `so-py/src/version.rs` 的 `pyo3` 绑定未同步 `Python` 侧异常映射 | 04 E3 | 现状消息前缀已随错误码改为 `FixedParse:` / `FixedExcessPrecision:`（`python/tests/test_bindings.py` 已同步）；类型映射仍待第 11 项一并落地 |
| 15 | `scripts/check-deps.ps1` 的层级表解析依赖 04 §3.5 的 markdown 形状 | 04 A7 | 表格被重排（如换成非 `层N` 标签、多列合并）会让解析失效并报"没解析出任何层级登记"；改 §3.5 表格后须跑一次该脚本 |

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
