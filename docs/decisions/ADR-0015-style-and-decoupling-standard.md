# ADR-0015：编码风格、错误码与模块解耦的规范化

- 状态：**已接受**（2026-11-11；日志规则编号 `L1–L6` 已于 2026-11-12 改为 `G1–G6`，见 [ADR-0017](ADR-0017-rule-id-disambiguation.md)；第 1–8 项按现状固化并写入 docs/04，第 9 项由 [ADR-0016](ADR-0016-journal-crate-extraction.md) 落定为「新建 `so-journal`」并已实施）
- 日期：2026-11-11
- 触发：M0–M2 已落地 8 个 crate（so-core / so-model / so-telemetry / so-adapter-gate /
  so-book / so-recorder / so-data / so-py + apps/so-cli），但 docs/04 的规范只覆盖到
  "文件划分（F）"和"错误处理（E1–E5）"两档，且 E2 的示例本身就不符合其他 crate 的实际命名。
  评审发现以下需要**一次性定死**的问题：错误码命名不统一、错误 → 日志字段不统一、
  CLI 退出码无表、实现方法（枚举表、请求结构体、编排分层）无约定、分层与依赖只有一张
  mermaid 图没有可校验的规则、文档与代码存在多处不一致（仓库树、模块名、悬空引用）。

## 背景：现状核查（2026-11-11，逐 crate 读取）

### 1. 错误码命名存在两套风格

| crate | 错误枚举 | `code()` 实际取值 | 风格 |
|---|---|---|---|
| so-core | `FixedError` | `FixedOverflow` `FixedDivisionByZero` `FixedInvalidScale` `FixedInvalidStep` `FixedParse` `FixedExcessPrecision` | ✅ `Fixed` 前缀（2026-11-11 由旧码改名） |
| so-model | `SymbolError` | `SymbolTooLong` `SymbolInvalidChar` `SymbolEmpty` | ✅ `Symbol` 前缀 |
| so-book | `BookError` | `BookGap` `BookStaleSnapshot` `BookOverCapacity` `BookCrossed` | ✅ `Book` 前缀 |
| so-adapter-gate | `GateError` | `GateBadShape` `GateFixed` `GateTsOverflow` `GateSymbol` | ✅ `Gate` 前缀 |
| so-recorder | `RecorderError` | `RecorderConfig` … `RecorderIo`（6 个；Journal 失败转发 `Journal*`） | ✅ `Recorder` 前缀 |
| so-data | `DataError` | `DataJournal` `DataParse` `DataParquet` `DataUsage` | ✅ `Data` 前缀 |
| so-journal | `JournalError` | `JournalOpen` `JournalWrite` `JournalCodec` `JournalRow` `JournalNotRolled` `JournalChecksum` | ✅ `Journal` 前缀（2026-11-11 新建） |
| so-telemetry | `TelemetryError` | `TelemetryFilter` `TelemetryFile` `TelemetryInit` | ✅ 已补 `code()`（2026-11-11） |

6 个域里 5 个已经是 `<域前缀><变体>`，只有最基础的 so-core 不是。
docs/04 E2 的示例写的正是 `ExcessPrecision`——**规范示例与规范本身矛盾**。（两项已于 2026-11-11 修正。）

### 2. 错误 → 日志字段不统一（同为 so-cli 内相邻文件）

```rust
// data_normalize.rs:67 —— 带稳定错误码 ✅
tracing::error!(code = e.code(), venue, date, "normalize 失败");
// data_normalize.rs:27 —— 只带 Display 文本（中文，程序不可判别）❌
tracing::error!(date, error = %e, "扫描 Raw Journal 目录失败");
// data_download.rs:54 —— 同上 ❌
tracing::error!(error = %e, "创建 tokio 运行时失败");
```

### 3. CLI 退出码无规范

`dispatch.rs` 定义了 `EXIT_NOT_IMPLEMENTED = 3` 并注释"与参数错误（clap 用 2）区分"，
而 `ExitCode::FAILURE`（=1）散落在其余 6 个文件里；这张表从未写进文档。

### 4. 实现方法已有事实标准但未成文

`RunMode` 的 `as_str()/label()/ALL/from_str()/may_load_keys()` 枚举表模式、
`Scale::new()` 校验后构造的 newtype 模式、`NormalizeRequest` + `NormalizeStats` 的
"请求结构体 + 统计结构体"模式、`normalize.rs`（纯算法）/ `pipeline.rs`（编排）/
`parquet_out.rs`（序列化）的三层分工——都在用，但没有一条规则说"必须这样"。

### 5. 分层与解耦只有图，没有规则

docs/03 §1 给了一张 mermaid 依赖图和两条文字约束，没有层级概念，没有可机检的依赖表；
`so-data → so-recorder`（读 `JournalRow` + 复用 `Dedup`）这类**数据层反向依赖进程层**的边，
图上根本画不出来。

### 6. 文档与代码不一致（逐条核实）

| 文档位置 | 文档写法 | 代码实际 |
|---|---|---|
| 03 §1 仓库树 | `apps/so-recorder/` | `crates/so-recorder/` |
| 03 §1 仓库树 | `python/strateorigo/` | `python/src/strateorigo/`（src 布局） |
| 03 §10 | `so-data::journal` 负责 Journal 读写 | 写在 `so-recorder::journal`，读在 `so-data::journal_in` |
| 03 §10 | `so-data::store` | 实际是 `parquet_out` / `instrument_out` / `pipeline` |
| 04 §5.3（被 configs 注释引用） | 引用 §5.3 | 当时 §5 没有子编号，引用悬空 |

## 决策

### 1. 规则编号体系（固化）

规则用 `<字母><数字>` 编号：`H` 热路径、`D` 确定性、`F` 文件划分、`E` 错误、
`I` 实现方法（新）、`A` 分层解耦（新）、`L` 日志（新）、`T` 测试、`S` 安全。
编号**只增、不改、不复用**；废弃规则标删除线并注明替代项。
**代码注释只引用编号**（`docs/04 H8`），不引用章节号或行号——章节会重排，行号会失效。

### 2. 错误码命名（固化）

`code()` 恒为 `<域前缀><变体>` 的 PascalCase 拼接，前缀取 crate 名去掉 `so-` 后的
首字母大写形式；so-core 与 so-model 按"类型域"命名为 `Fixed` / `Symbol`；
适配器按所名（`Gate`、`Okx`）。码**全局唯一**（Python / gRPC / 审计只看这一个字符串）、
**只增不改名**；改名等同破坏接口，须新 ADR。

后果：`FixedError` 的 6 个码需要改名（`Overflow → FixedOverflow` 等）。
这是代码改动，**本次只落规定**，改名列入 `docs/pending-items.md` 由 M2 收尾执行。

### 3. 错误对象的形状（固化）

- `code()` 不带参数（码本身不含数据）；`Copy` 的错误枚举用 `const fn code(self)`，
  非 `Copy` 用 `fn code(&self)`。
- 上下文用**命名字段**（`Self::Parse { channel, symbol }`），不拼字符串；
  仅当载荷是外部工具/OS 的原文时才用裸 `String`（如 `DataError::Journal(String)`），
  且该 `String` 只出现在 `Display` 里，不参与 `code()`。
- `Display` 中文面向人；程序判别一律用 `code()`。

### 4. 错误 → 日志（固化为新规则 L4）

记录错误时**必须**带 `code` 字段：`tracing::error!(code = e.code(), …)`。
面向人的可读详情用字段 `error`，且只在不便拆字段时使用。
§2 列出的三处 `error = %e` 属待清理项（列入 pending-items）。

### 5. CLI 退出码（固化，E8）

| 码 | 含义 |
|---|---|
| 0 | 成功 |
| 1 | 运行时失败（I/O、配置、外部接口） |
| 2 | 参数错误（clap 约定，不可复用） |
| 3 | 命令尚未实现（`dispatch.rs` 既有常量） |
| 4+ | 预留给"部分成功 / 需人工介入"类结果 |

`so-cli` 不引入 `anyhow`：错误在 `main` 内用 `tracing::error!(code = …)` 记录后
返回 `ExitCode`（现状即如此，固化之）。

### 6. Python 边界（固化 + 过渡期）

现状：`so-py` 抛 `ValueError`，消息前缀为 `` `{code}: {中文}` ``。
docs/04 E3 原文写"so-py 映射为具体 Python 异常类型"，与实现不符。
**过渡期（M3 前）**：以"错误码作消息前缀"为准，改文档不改代码；
**M4 前**：在 `python/src/strateorigo/errors.py` 建立 `{code: ExceptionType}` 映射表，
把前缀解析为具体异常，并与 Rust 侧错误码登记表同步维护（由测试断言两侧集合一致）。

### 7. 实现方法（固化为新规则 I1–I13）与日志字段命名（L1–L6，现为 G1–G6）

见 docs/04 §3.4 / §3.6。要点：枚举表模式、newtype 受限构造、请求/统计结构体、
编排层不做算法（算法层不做 I/O）、宏做编译期注册、trait 只作端口、
日志字段 `snake_case` + 单位写进名字、基础 crate 不因单个功能增依赖。

### 8. 分层与依赖解耦（固化为新规则 A1–A7）

给每个 crate 一个**依赖层级 `层0`–`层5`**（**刻意不用 L 前缀**：L1/L2/L3 已被
风控执行位置、订单簿深度、回测保真度占用），依赖只能由高编号层指向低编号层：

| 层 | crate |
|---|---|
| `层0` | so-core |
| `层1` | so-model、so-telemetry、so-py |
| `层2` | so-adapter-gate、so-book、so-journal、so-strategy、so-signal、so-exec、so-portfolio |
| `层3` | so-risk、so-oms、so-engine、so-backtest、so-data、so-recorder |
| `层4` | so-adapters/*（含连接/下单的适配器）、so-lab、so-proto |
| `层5` | apps/*（so-cli、so-trader、so-guardian） |

附 CI 断言脚本 `scripts/check-deps.ps1`（读 `cargo metadata`，按层级表与例外表判边）；
`so-data → so-recorder` 这条**同层依赖**在例外表中显式登记为"待消除"（见第 9 项）。

### 9. journal 格式的归属（✅ 已由 ADR-0016 落定为选项 2）

现状：`JournalRow` 与 Journal 写入在 `so-recorder`，读取在 `so-data`，
于是 **`so-data` 依赖 `so-recorder`**——数据层依赖录制进程层，是当前唯一的"违和依赖边"。
三个选项：

1. **下沉到 so-core**（`so-core::journal`）。零新增 crate，但 so-core 是热路径 crate，
   Journal 的 JSONL/zstd 逻辑属冷路径，会污染 so-core 的"零依赖"性质。
2. **新建 `crates/so-journal`**（推荐）。只含 `JournalRow`、帧格式、读写、sha256 校验、
   `Dedup`；`so-recorder` 与 `so-data` 都依赖它，两者互相不依赖。层级 `层2`。
3. **保持现状**，在例外表中登记并注明理由。成本最低，但数据层将长期依赖一个
   带 tokio/reqwest 的重 crate（编译时间与依赖面都吃亏）。

**已选方案 2**（新建 `so-journal`），实施记录见 [ADR-0016](ADR-0016-journal-crate-extraction.md)。

## 影响

- `docs/04`：新增 §0 速查；§3.1 补"静默丢弃视为 bug"；§3.3 扩充为 E1–E9 并内嵌
  错误码登记表与 Python 映射表；新增 §3.4（I1–I13）、§3.5（A1–A7，含层级表与例外表）、
  §3.6（L1–L6，2026-11-12 改为 G1–G6）；§5 加子编号（修好悬空的 §5.3 引用）；S10 补 `[REC]` 与 `mode = recorder`。
- `docs/03`：§1 仓库树与代码对齐并标层级、依赖图加分组；§10 改为模块职责表并更正模块名；
  §18.3 补事件字段命名规则。
- `docs/06` §9.1、`docs/07` §8、`docs/README.md`、`docs/pending-items.md`、`CLAUDE.md` 同步。
- 代码改动（2026-11-11 已完成）：`FixedError` 改名、错误日志补 `code`、`TelemetryError::code()`、
  `scripts/check-deps.ps1`、`so-journal` 下沉（ADR-0016）。Python 映射表仍待 M4 前落地。
