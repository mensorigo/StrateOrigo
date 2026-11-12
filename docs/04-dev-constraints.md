# 04 · 开发限制与注意事项

> 本文是**强制规则**。Agent 和人写的代码同样适用。违反"热路径规则"或"安全规则"的改动不得合并。
>
> 本文是规则的**唯一事实来源**：`CLAUDE.md` 与 `docs/README.md` 只做指针与速查，
> 与本文冲突时以本文为准，并应删掉重复描述。
>
> 规则编号：`H` 热路径 · `D` 确定性 · `F` 文件划分 · `E` 错误 · `I` 实现方法 ·
> `A` 分层解耦 · `G` 日志 · `T` 测试 · `S` 安全。编号**只增、不改、不复用**；
> 废弃规则标~~删除线~~并注明替代项。**代码注释只引用编号**（`docs/04 H8`），不引用章节号或行号。
> 规则变更流程（先 ADR、再改本文、最后改引用方）见 §5.7。

## 0. 速查

| 我要做 | 规则 | 一句话 |
|---|---|---|
| 在热路径写代码 | H1–H10 | 不分配、不加锁、不阻塞、不 panic、不用 `f64` |
| 写任何会进回测的代码 | D1–D5 | 相同输入 ⇒ 逐位相同输出 |
| 建新文件 / 新模块 | F1–F7 | ≤ 400 行、一事一文件、入口不写逻辑 |
| 定义错误 | E1–E7 | 每域一个 `thiserror` 枚举 + 稳定英文 `code()`，登记在 §3.3 |
| 抛错 / 记错误 | E8–E9、G4 | 退出码只用 0‑3，库代码不 `unwrap()`；日志**必带** `code` 字段 |
| 选实现方式 | I1–I13 | 枚举表、受限构造、请求结构体、编排层不做算法 |
| 加 crate / 加依赖 | A1–A7、§3.5 | 依赖只能由高编号层指向低编号层，新增边须登记 |
| 打日志 | G1–G6 | 不按 tick 打 INFO，字段 `snake_case` |
| 写测试 | T1–T3 | 测试同样受文件规则约束；数据/回测代码必须有确定性测试 |
| 跨语言边界 | E3、§3.3 | Python 异常与错误码登记表同步维护 |
| 不知道怎么办 | §5.1、§5.7 | 写 ADR 并问人，**不要自行决定** |

## 1. 热路径规则（H）

适用 crate：`so-core`、`so-book`、`so-engine`、`so-risk`（事前风控部分）、`so-oms`、`so-signal`、`so-exec`、`strategies/*`（HF），以及适配器中的解析和编码函数。

| # | 规则 | 原因 / 替代方案 |
|---|---|---|
| H1 | **预热后不分配堆内存**：禁止热路径中的 `Vec::push` 扩容、`String`、`format!`、`Box::new`、`clone()` 大对象 | 用预分配缓冲、`SmallVec`、对象池；用 `dhat` 或计数分配器测试验证 |
| H2 | **不加锁**：禁止 `Mutex`、`RwLock`、`parking_lot` | 用 SPSC 队列（rtrb）传递数据，状态只归单一线程所有 |
| H3 | **不阻塞**：禁止文件 I/O、网络 I/O、`sleep`、同步通道 `recv()` 阻塞等待 | 交给冷路径线程，通过队列异步处理 |
| H4 | **不用 async/tokio**：内核线程是普通 OS 线程的忙轮询 / 自旋 + 退避循环 | 可预测延迟 |
| H5 | **价格与数量不用 `f64`**：一律 `Price` / `Qty` 定点 | 避免累积误差和比较失败；指标/信号计算可以用 `f64` |
| H6 | **不 panic**：禁止 `unwrap()`、`expect()`、可能越界的索引、整数溢出 | 返回 `Result`；clippy 开启 `unwrap_used`、`indexing_slicing`、`arithmetic_side_effects`（热路径 crate） |
| H7 | **不读系统时间**：禁止 `SystemTime::now()`、`Instant::now()` | 只能用 `Clock` trait（`ctx.now()`） |
| H8 | **不用默认 `HashMap`**：SipHash 慢，且迭代顺序随机破坏确定性 | 用 `InstrumentId` 作为数组下标；必须用映射时用确定性哈希且**不依赖迭代顺序** |
| H9 | **日志不格式化**：热路径不调用 `tracing::info!` 等带格式化的宏 | 写固定结构的事件到 SPSC 队列（G1、G3），冷路径线程负责格式化 |
| H10 | **不用 `dyn Trait` 做每事件的分发**（策略入口除外） | 用泛型 / 枚举分发（I4） |

**度量**：每个热路径改动附 criterion 基准；CI 中基准相对 main 回退 > 10% 视为失败。

## 2. 确定性规则（D）

| # | 规则 |
|---|---|
| D1 | 回测结果须逐位可复现：相同 `dataset_hash` + `git_commit` + 参数 + `seed` ⇒ 相同的订单序列与 PnL（快照测试） |
| D2 | 所有随机性来自显式种子的 RNG（`ctx.rng()`）；禁止 `thread_rng()` |
| D3 | 事件排序规则固定：按 `(ts, 事件源优先级, 序号)` 全序排序，并写入文档与测试 |
| D4 | 浮点计算（信号）在回测和实盘使用同一份代码；禁止 `-ffast-math` 类优化 |
| D5 | 有未提交改动（`git dirty`）时运行的实验只能标为草稿，不能作为晋级依据 |

## 3. 编码规范

### 3.1 通用（Rust 与 Python 共同适用）

- 标识符（类型、函数、字段、变量）一律英文；注释与文档用中文。
- 公开项必须有文档注释；注释写**为什么**，不写**是什么**（能从代码看出的不复述）。
- 文件开头第一行写一句话职责（F2）。
- 不写"防御式重复校验"：限制在哪一层定死就只在那里校验（A4），上层只调用不重查；边界只做一次转换（E3）。
- **静默丢弃视为 bug**：任何被跳过、丢弃、回退、失败计数的分支都要有对应统计数据，并最终出现在日志或报告里。
- 禁止 `utils` / `helpers` / `common` 类命名，包括函数名（F7）。

### 3.2 文件与模块划分（F）

| # | 规则 |
|---|---|
| F1 | **单文件行数上限**：源文件（`.rs` / `.py`，含注释与内联测试）**超过 400 行 CI 失败**，超过 250 行给出警告、应当拆分。生成的代码（如 prost 输出）、锁文件、数据文件不计。由 `scripts/check.ps1` 检查，**不设例外清单** |
| F2 | **每个文件只负责一件事**：一个核心类型及其方法，或一组紧密相关的函数。文件开头用 `//!`（Python 用模块 docstring）一句话写明职责；一句话说不清就说明该拆了 |
| F3 | **不同逻辑分文件**：解析、计算、格式化/显示、I/O、配置、错误类型分别放在不同文件；热路径代码与冷路径代码不放在同一文件 |
| F4 | **入口文件不写逻辑**：`lib.rs`、`mod.rs`、`__init__.py` 只做模块声明、重新导出和 crate 级属性 |
| F5 | **函数长度**：单个函数不超过 60 行（clippy `too_many_lines`，阈值见 `clippy.toml`；Python 用 ruff `PLR0915`，最多 40 条语句） |
| F6 | **测试位置**：单元测试可以内联在 `#[cfg(test)] mod tests` 中，但计入行数；测试变多时移到同目录的 `tests.rs` 子模块，或 `tests/` 集成测试 |
| F7 | **按职责拆，不按行数切**：不得出现 `part1` / `part2`、`utils2` 之类按长度机械切分的文件；`utils` / `helpers` / `common` 这类没有明确职责的文件名不允许使用 |

### 3.3 错误处理（E）

Rust 的错误是值（`Result`），"统一"靠以下约定，而不是全局 try/catch：

| # | 规则 |
|---|---|
| E1 | **每个域一个错误枚举**：库 crate 用 `thiserror` 定义本域错误（`so_core::FixedError`，将来的 `OmsError`、`RiskError`、适配器错误各归各的 crate）。禁止字符串当错误类型；`anyhow` 只在二进制内使用，不跨库边界传播 |
| E2 | **跨边界错误带稳定错误码**：每个错误枚举实现 `code()`，取值为 `<域前缀><变体>` 的 PascalCase。域前缀与全部现有码见下方**错误码登记表**；码**全局唯一**、**只增不改名**，改名等同破坏接口、须新 ADR |
| E3 | **映射只发生在边界**：so-proto 把域错误映射为 gRPC status（details 带错误码）；so-py 按下方 Python 映射表映射为异常。映射表在 so-proto / so-py 内各一个文件，与登记表同步维护，并有测试断言两侧集合一致 |
| E4 | **错误分级**：可自动恢复的（重连、重建订单簿、限频退避）记 WARN；需要人工处理的（对账不一致、Unknown 超时、配置校验失败）才是 ERROR（L2） |
| E5 | **热路径错误只能返回或降级**：返回 `Result`、拒绝订单、进入只减仓；不得重试阻塞（H3）、不得 panic（H6） |
| E6 | **`code()` 不带参数**：码本身不含数据，需要上下文的信息放命名字段（E7）。`Copy` 枚举用 `const fn code(self)`，非 `Copy` 用 `fn code(&self)` |
| E7 | **上下文用命名字段而非拼字符串**：`Parse { channel, symbol }` 优于 `Parse(String)`。裸 `String` 只允许承载**外部工具或 OS 的原文**（如 `DataError::Journal(String)`），且只用于 `Display`，不参与 `code()` 判别 |
| E8 | **退出码固定**：所有二进制只用 0 成功 / 1 运行时失败 / 2 参数错误（clap 约定，不可复用）/ 3 命令未实现 / 4+ 预留"部分成功、需人工介入"。不得自定义其他数字 |
| E9 | **库与工具的错误策略不同**：`examples/`、一次性脚本、基准工具可以 `panic!`（失败即失败，便于定位）；`crates/*` 的库代码与 `apps/*` 一律走 E1–E8，**库代码里不得出现 `unwrap()` / `expect()`**（确有不可失败的理由时，行内注释写明） |

**错误码登记表**（唯一事实来源；新增或改动必须同步本表与 Python 映射表）：

| 域 | 错误枚举（模块） | 码前缀 | 现用码 |
|---|---|---|---|
| 定点与解析 | `so_core::FixedError`（`so-core/src/error.rs`） | `Fixed*` | `FixedOverflow`、`FixedDivisionByZero`、`FixedInvalidScale`、`FixedInvalidStep`、`FixedParse`、`FixedExcessPrecision` |
| 符号 | `so_model::SymbolError`（`so-model/src/symbol.rs`） | `Symbol*` | `SymbolTooLong`、`SymbolInvalidChar`、`SymbolEmpty` |
| 订单簿 | `so_book::BookError`（`so-book/src/error.rs`） | `Book*` | `BookGap`、`BookStaleSnapshot`、`BookOverCapacity`、`BookCrossed` |
| 交易所适配器 | `so_adapter_*::*Error`（`so-adapter-gate/src/error.rs`） | `<所名>*` | `GateBadShape`、`GateFixed`、`GateTsOverflow`、`GateSymbol` |
| 录制 | `so_recorder::RecorderError`（`so-recorder/src/error.rs`） | `Recorder*` | `RecorderConfig`、`RecorderWs`、`RecorderRest`、`RecorderParse`、`RecorderAlert`、`RecorderIo`（Journal 失败转发 `Journal*`，不折叠） |
| 数据 | `so_data::DataError`（`so-data/src/error.rs`） | `Data*` | `DataJournal`、`DataParse`、`DataParquet`、`DataUsage` |
| 遥测 | `so_telemetry::TelemetryError`（`so-telemetry/src/error.rs`） | `Telemetry*` | `TelemetryFilter`、`TelemetryFile`、`TelemetryInit` |
| Journal | `so_journal::JournalError`（`so-journal/src/error.rs`） | `Journal*` | `JournalOpen`、`JournalWrite`、`JournalRead`、`JournalCodec`、`JournalRow`、`JournalNotRolled`、`JournalChecksum`、`JournalChecksumMismatch` |
| CLI 边界 | `so-cli` 的 `exit.rs`（`apps/so-cli/src/exit.rs`） | `Cli*` | `CliNotFound`、`CliPermissionDenied`、`CliAlreadyExists`、`CliInvalidInput`、`CliTimedOut`、`CliConnection`、`CliIo`、`CliUsage`、`CliNoData`、`CliRuntime`（标准库 `io::Error` 无 `code()`，在边界统一成 CLI 码） |
| 尚未创建 | `so-engine` / `so-risk` / `so-oms` / `so-backtest` / `so-lab` / `so-proto` | `Engine*` / `Risk*` / `Oms*` / `Backtest*` / `Lab*` / `Proto*` | 建 crate 时在此登记 |

**Python 异常映射**（E3；过渡期状态见 `docs/pending-items.md`）：

| 码族 | 目标 Python 异常 | 现状 |
|---|---|---|
| `Fixed*`、`Symbol*`、`Gate*`、`Data*`、`Journal*`（参数与数据形状类） | `strateorigo.errors.StrateOrigoValueError`（继承 `ValueError`） | 过渡期：`ValueError`，消息以 `` `{code}: {中文}` `` 开头 |
| `Recorder*`、`Telemetry*`、`Engine*`、`Risk*`、`Oms*`（运行时与 I/O 类） | `strateorigo.errors.StrateOrigoRuntimeError`（继承 `RuntimeError`） | 同上 |
| 未登记的码 | 测试失败（禁止未登记的码跨边界） | M4 前落地断言 |

> 程序判别一律读错误码（异常类型或消息前缀），**不匹配中文消息**。

### 3.4 实现方法（I）

以下不是"风格偏好"，是已经发生过分歧的地方，定死后按此选型：

| # | 规则 | 现状示例 |
|---|---|---|
| I1 | **枚举表模式**：有限集合的枚举把"全部取值"和"字符串 ↔ 枚举转换"收在枚举自身的关联方法里：`const ALL: &[Self]`、`as_str()`、`from_str()`、`label()`。禁止在调用侧散写 `match` | `so-core::RunMode` |
| I2 | **受限构造**：带不变量的类型（精度、步长、容量）用 `new()` 校验后构造，字段私有 + 只读访问器；不用 `pub struct Scale(pub u8)` 暴露能构造出非法值的入口 | `Scale::new(u8) -> Result` |
| I3 | **请求结构体 + 统计结构体**：多次传入的参数（≥3 个，或未来会增长）用 `XxxRequest<'a>` 聚合，用 `XxxStats` 回传统计；不用 6 个位置参数 | `NormalizeRequest` / `NormalizeStats` |
| I4 | **内核内多态用枚举分发**，边界用 trait 对象。`dyn Trait` 只出现在（a）策略入口 `Box<dyn Strategy>`、（b）适配器边界、（c）so-py 的 Python 对象 | H10 |
| I5 | **trait 是端口，不是分类法**：trait 描述"跨边界能力"（`Clock`、`MarketDataGateway`），不描述"实现细节相似"；同类实现用泛型或枚举 | `Clock` 的 live / sim 两实现 |
| I6 | **编译期注册用宏**：策略等插件用 `so_strategy::register!("mm-basic", …)` 生成静态注册项，不用运行时字符串查找 | `so-strategy`（M3） |
| I7 | **代码表用 `match` 返回 `&'static str`**，不用 `static` 数组按索引取；`code()`、`as_str()` 均如此（编译期穷尽检查） | E2、I1 |
| I8 | **编排层不做算法，算法层不做 I/O**：一个功能拆成 `<name>.rs`（纯计算，输入→输出，可单测）/ `pipeline.rs`（编排：收集输入、调用算法、落盘）/ `<name>_out.rs`（序列化） | `normalize.rs` / `pipeline.rs` / `parquet_out.rs` |
| I9 | **I/O 集中在少数文件**：文件读写、网络调用不散落在业务函数里；模块内的 I/O 只出现在 `*_in.rs`、`*_out.rs`、`store.rs`、`download.rs` 这类明确命名的文件 | `journal_in.rs` / `instrument_out.rs` |
| I10 | **配置强类型 + 默认值 + 启动校验**：`serde` 反序列化成结构体，不用 `HashMap<String, Value>` 传配置；未知字段报错（`deny_unknown_fields`） | `so-recorder::config` |
| I11 | **ID 与序号用 newtype**，不裸传 `u64`（`InstrumentId`、`ClientOrderId`）；热路径可作数组下标（H8） | `so-core::ids` |
| I12 | **`Display` 只面向人**：可读文本给 `Display`，机器可读给 `code()` 或序列化；**不要用 `Debug` 做任何程序判别或持久化** | E2、G4 |
| I13 | **基础 crate 不因单个功能而增依赖**：`so-core` 保持零第三方依赖（`thiserror` 仅编译期）；为避免把网络/异步/序列化依赖拖进基础 crate，应新建叶子 crate（如 [`so-journal`](03-module-design.md#1-仓库结构)） | ADR-0015 第 9 项、[ADR-0016](decisions/ADR-0016-journal-crate-extraction.md) |

### 3.5 分层与解耦（A）

**层级编号 `层0`–`层5` 指 crate 的依赖层级**，与各领域内部的级别编号无关。
领域级别一律带领域前缀，见 [ADR-0017](decisions/ADR-0017-rule-id-disambiguation.md)：

| 领域级别 | 写法 | 出处 |
|---|---|---|
| 风控执行位置 | `RK-L1` 事前 / `RK-L2` 组合 / `RK-L3` 守护进程 | [03 §7](03-module-design.md#7-so-risk风控) |
| 订单簿深度 | `BK-L1` 一档 / `BK-L2` 多档（行业标准术语） | [03 §1](03-module-design.md#1-仓库结构) |
| 回测保真度 | `BT-L0` / `BT-L1` / `BT-L2` | [03 §8.1](03-module-design.md#8-so-backtest回测) |

所以 `L` 不再是任何规则编号的前缀，规则编号与领域级别不会再撞车。

| 层 | crate |
|---|---|
| `层0` | `so-core` |
| `层1` | `so-model`、`so-telemetry`、`so-py` |
| `层2` | `so-adapter-gate`、`so-book`、`so-journal`、`so-strategy`、`so-signal`、`so-exec`、`so-portfolio` |
| `层3` | `so-risk`、`so-oms`、`so-engine`、`so-backtest`、`so-data`、`so-recorder` |
| `层4` | `so-adapters/*`（`so-adapter-okx`、`so-adapter-ibkr` 等含连接/下单的适配器）、`so-lab`、`so-proto` |
| `层5` | `apps/*`（`so-cli`、`so-trader`、`so-guardian` 等二进制） |

> `so-adapter-gate` 在 `层2` 而非 `层4`：它只依赖 `so-core` + `so-model`，是**纯解析**
> 适配器（热路径、无网络），任何 `层3` 的数据/录制代码都可以直接用它。
> `层4` 留给需要连接与下单能力的适配器——那里才有 `tokio` 这类重依赖。

| # | 规则 |
|---|---|
| A1 | **依赖只能由高编号层指向低编号层**；同层依赖须登记在下表（现状 + 消除计划），不得擅自新增 |
| A2 | **禁止循环依赖**：`cargo` 会在 crate 粒度报错，但模块之间的循环要靠人守（模块内同样禁止） |
| A3 | `so-engine` **不得依赖** `so-adapters/*`、`tokio`、任何网络库、任何 GUI crate；策略 crate 只能依赖 `so-strategy`、`so-signal`、`so-exec` 及其重新导出的类型 |
| A4 | **限制只在定义处校验一次**：模式与密钥规则、精度、限额在各自的属主 crate 校验（`so-core::RunMode`、`so-risk`），上层只调用；禁止同一规则在多处重复实现 |
| A5 | **服务端完全无头**：服务端二进制不得依赖任何 GUI crate（`deny.toml` 的 `[bans]` 强制）；桌面端是独立 workspace，只消费 `so-proto` / `so-lab` 的数据 |
| A6 | **新增依赖须说明理由**：热路径 crate 新增任何依赖、任何 crate 新增网络/异步/GUI 依赖，须在提交信息里写明用途与替代方案 |
| A7 | 依赖表由 `scripts/check-deps.ps1` 对 `cargo metadata` 机检（出现表外依赖边即失败），接入 `scripts/check.ps1` |

**同层依赖例外表**（当前为空；每次新增都要写清消除计划，并接受"上游不存在即失败"的检查）：

| 依赖边 | 层 | 现状 | 消除计划 |
|---|---|---|---|
| （空） | — | — | — |

> `so-data → so-recorder` 曾是这里唯一的例外，已由下沉 `so-journal` 消除（ADR-0016）。

### 3.6 日志（G）

与 [03 §18](03-module-design.md#18-日志与可观测性so-telemetry) 的六类记录配合使用；本节只约束"代码里怎么写"。

| # | 规则 |
|---|---|
| G1 | 热路径不调用 `tracing` 宏：写 `LogRecord` 到 SPSC（H9、03 §18.2）；冷路径与 I/O 线程可直接用宏 |
| G2 | 级别按 03 §18.4：ERROR = 需要人工处理；WARN = 异常但已自动处理；INFO = 状态变化；DEBUG/TRACE = 开发调试。**不按 tick 打 INFO** |
| G3 | 需要给人看的"关键事件"必须**同时**进审计库与日志（启动、重连、缺口、风控动作、生命周期迁移）；日志不是事实来源（03 §18.1） |
| G4 | **记录错误必须带 `code` 字段**：`tracing::error!(code = e.code(), venue, "normalize 失败")`。人的可读详情用 `error` 字段，且仅在不便拆字段时使用；**禁止只写 `error = %e` 而不给 `code`** |
| G5 | **字段命名统一**：`snake_case`；时间字段带 `_ts` 后缀且单位写进名字（`ts_exch`、`ts_recv`、`elapsed_ns`）；数量带语义后缀（`qty`、`notional`、`bytes`、`rows`）；错误码字段固定叫 `code`；异常详情固定叫 `error` |
| G6 | 日志不得包含密钥、签名、Passphrase、完整请求头（S2、03 §18.5）；脱敏用 `***`，不打印长度与前后缀 |

## 4. 测试要求（T）

| # | 规则 |
|---|---|
| T1 | 测试与实现同等要求：同样受 F1–F7 的文件划分约束，命名清晰，不断言实现细节 |
| T2 | **确定性测试优先**：任何数据、回测、编排代码必须有"相同输入两次运行结果一致"的测试（D1） |
| T3 | **修 bug 先写复现测试**，再改代码；修复后该测试留在仓库里 |

| 层级 | 工具 | 覆盖对象 |
|---|---|---|
| 单元测试 | nextest / pytest | 所有模块 |
| 性质测试 | proptest / hypothesis | 定点运算、订单簿、OMS 状态机、撮合模拟器（不变量：仓位守恒、资金守恒、状态只能合法迁移） |
| 快照测试 | insta | 交易所消息解析、回测结果 |
| 回放测试 | 录制的真实交易所消息 | 每个适配器：行情解析、订单回报、断线重连、序列缺口 |
| 对照测试 | hftbacktest | L2 回测器（M3） |
| 基准测试 | criterion | 热路径 |
| 故障注入 | 自研 chaos 测试工具 | 断网、交易所返回 5xx/限频、回报延迟/丢失/乱序、进程被杀、时钟跳变 |
| 模拟盘 | Paper 模式 | 实盘前必经 |

## 5. Agent 协作规则

开发主要由 Agent 完成，以下规则同步写入根目录 `CLAUDE.md`（该文件只放指针与速查，**规则本身以本文为准**）。

### 5.1 工作方式

1. **先读文档再写代码**：任何模块的改动前先读 `docs/03-module-design.md` 对应章节和本文件。
2. **小步提交**：每个任务对应一个可编译、测试通过的提交；不在一个任务里跨多个里程碑。
3. **不确定就停**：遇到文档未覆盖的设计决策，写入 `docs/decisions/`（ADR 格式）并询问，而不是自行决定。

### 5.2 禁止 Agent 改动的内容（需人工完成或 review）

- `configs/env/live*.toml`、`configs/risk/**`；
- 任何密钥、`.env`、部署凭据；
- `so-risk` 的限额默认值和守护进程逻辑（可提出修改建议，由人确认）；
- 删除或放宽测试、基准门槛。

### 5.3 禁止 Agent 执行的操作

连接实盘账户、实盘下单、部署到 VPS、推送到远程仓库（除非当次明确授权）。

### 5.4 不做 Web 页面

不得引入任何 Web 前端、HTTP 看板、浏览器界面或输出 HTML 的报告工具（Grafana、MLflow UI、plotly、JupyterLab 等）。人机交互只用 CLI、终端界面（ratatui）、Telegram；GUI（GPUI）为 P2，未启动前不得引入 GPUI 依赖。客户端需要的所有数据（含 K 线）必须先由服务端接口（`so-proto`，VPS 交易域）或 `so-lab`（本地研究域）提供，见 [03 §11](03-module-design.md#11-控制面与客户端接口预留so-proto)。

### 5.5 交易所 API

实现适配器前，先把当前版本的官方文档要点整理进 `docs/venues/<venue>.md`（端点、签名、限频、序列号规则、错误码），再写代码。**不凭记忆写字段名**。

### 5.6 文档与代码不一致时怎么办

- **规则文档写错了** ⇒ 改文档，并同步 `CLAUDE.md` / `docs/README.md` 的指针；
- **代码写错了** ⇒ 改代码，或列入 `docs/pending-items.md` 并注明依据的规则编号；
- **不要两边各改一半**：改完一处后，全仓搜索该规则的引用（代码注释、configs 注释、ADR）一并核对。
- 规则编号是稳定标识：代码注释只引用编号（`docs/04 H8`），**不引用章节号或行号**——章节会重排，行号会失效。

### 5.7 规则变更

新增规则必须先写 ADR（说明触发现状与取舍），再改本文，最后改引用方；规则编号只增、不改、不复用，废弃规则标~~删除线~~并注明替代项。

## 6. 安全规则（S）

| # | 规则 |
|---|---|
| S1 | API Key **禁止提现权限**，绑定 VPS 出口 IP 白名单；交易 Key 与守护进程 Key 分开 |
| S2 | 密钥不入仓库、不入日志、不入实验库；VPS 上用权限 600 的文件或 `systemd-creds` 加密存储；`.gitignore` 包含 `*.key`、`.env*`、`secrets/` |
| S3 | 私钥/Secret 与 OKX Passphrase 不离开 VPS；交易所支持非对称密钥时优先使用 |
| S4 | 控制面只监听 WireGuard/Tailscale 接口；mTLS 或 Token；写操作审计 |
| S5 | VPS：SSH 仅密钥登录、禁 root 登录、防火墙默认拒绝、自动安全更新（交易时段外重启） |
| S6 | 子账户隔离：**每个实盘策略一个交易所子账户**（ADR-0001），限制单点损失 |
| S7 | 依赖供应链：`cargo deny`、锁文件、不使用 git 依赖（gpui 系列除外，且须锁定 rev 或精确版本） |
| S8 | **模拟盘不加载密钥**：`backtest` / `paper` 模式下，配置加载器遇到 `key_file` 字段或读取密钥目录即报错退出；模拟盘进程以无权读取密钥目录的系统用户运行 |
| S9 | **实盘一账户一策略**：`live` / `demo` 配置启动时校验"每个交易所 1 个账户、共 1 个策略"，不满足拒绝启动；校验逻辑有单元测试 |
| S10 | **模式标识**：所有日志、审计记录、数据库行、Telegram 消息、CLI 输出都带 `mode` 与实例名。**前缀集合是封闭的**：`[LIVE]`、`[PAPER]`、`[DEMO]`（交易域，取值自 `RunMode`），以及 `[REC]`（录制器 `so-recorder`：数据面进程，无 `RunMode`，日志中记 `mode = recorder`）。**新增前缀须改本规则**，禁止就地发明 |
| S11 | **控制命令指定目标**：Kill Switch、暂停、退出、改参数必须显式指定实例；没有"作用于全部实例"的默认值（全局 Kill 用单独命令 `so-cli kill --all-live`）；对 `live` 实例的写操作需二次确认 |

## 7. 合规与市场限制

> 以下为工程层面的提醒，**不构成法律意见**。上线前请确认你所在司法辖区及交易所/券商的最新规则。

### 7.1 加密货币
- 确认所在司法辖区是否允许使用相关交易所及衍生品（部分地区个人使用境外交易所存在法律风险，交易所也会限制特定地区用户）。
- 各交易所的**限频规则**（按 IP、按 UID、下单数/撤单比、订单数量上限）会变化，违规可能被封禁 API。适配器启动时拉取并记录当前限额。
- 自成交、刷量、幌骗（spoofing）等行为违反交易所规则，风控和策略审查中须明确禁止。
- 资金费率、强平机制、ADL（自动减仓）须在回测和风控中建模。

### 7.2 A 股
- **T+1**、涨跌停、最小申报单位（100 股）、集合竞价时段、停复牌。
- 程序化交易须按券商要求**报备**；《证券市场程序化交易管理规定（试行）》及沪深交易所实施细则对高频交易有专门认定标准和额外监管（以最新细则为准）。⇒ **本系统对 A 股只做中低频。**
- miniQMT / QMT 仅支持 Windows，且通常要求运行在券商客户端环境中：通过独立的 Python 桥接进程接入，与 `so-trader` 用本地 gRPC 通信。部分券商要求托管在其指定机房。

### 7.3 美股
- 通过 IBKR 等零售通道接入，延迟为数十毫秒级 ⇒ 中低频。
- Pattern Day Trader 规则（保证金账户日内交易次数与最低权益要求；该规则近年存在修订讨论，**以券商当前执行规则为准**）。
- 卖空规则（Reg SHO、可借券）、盘前盘后流动性、交易时段与夏令时切换。

## 8. 运维注意事项

| 项目 | 规则 |
|---|---|
| 自动重启 | `so-recorder` 可自动重启；`so-trader` **崩溃后不自动重启**，由守护进程撤单并告警，人工确认后重启（防止崩溃循环中反复开仓） |
| 启动检查 | 时钟偏差（chrony offset < 1ms）、品种元数据一致、风控配置哈希、账户仓位与本地记录一致，任一不通过拒绝启动 |
| 交易所维护 | 订阅维护公告；维护窗口前自动 ReduceOnly |
| 日志 | 运行日志 JSON Lines、按天滚动、保留 90 天；审计日志（订单/成交/控制命令）永久保留并备份；级别与字段约定见 03 §18 与 §3.6。新增代码：不按 tick 打 INFO；ERROR 只用于需要人工处理的情况；日志不得包含密钥与签名 |
| 备份 | SQLite 审计库每日备份到异地；Raw Journal 同步回本地 |
| 复盘 | 任何守护进程动作、任何实盘单日亏损超阈值 ⇒ 写复盘到 `docs/incidents/` |
| 资金 | 交易所账户只存放策略所需保证金；定期把利润转出到冷钱包/银行账户 |
| 多实例共存 | 实盘与模拟盘同机运行时：实盘绑定固定核心、高优先级，模拟盘用其余核心并 `nice` 降级；各实例数据目录、数据库、日志分开（`data/<instance>/`）；VPS 资源不足时模拟盘移到另一台机器 |
| 实例单元 | 每个实例一个 systemd 单元 `so-trader@<instance>.service`；实盘单元名以 `live-` 开头，模拟盘以 `paper-` 开头，交易所模拟以 `demo-` 开头 |

## 9. 常见陷阱清单

- 回测使用了未来数据（例如用 K 线收盘价在同一根 K 线内成交、用 `ts_exch` 而不是 `ts_recv` 推进）。
- 订单簿快照与增量拼接时的序列号对齐错误 ⇒ 盘口错误但不报错。
- 忽略部分成交、撤单与成交的竞态 ⇒ 仓位错误。
- 只用 Taker 成交价估计 Maker 策略 ⇒ 严重高估。
- 忽略资金费率、VIP 费率变化、最小下单量和精度取整。
- 参数优化在全样本上做 ⇒ 过拟合；必须 Walk-Forward。
- 多次试验只报告最好的结果 ⇒ 用 DSR / PBO 校正。
- 本地时钟与交易所时钟偏差导致延迟测量错误。
- WebSocket 静默断流（连接仍在但不推数据）⇒ 必须有行情新鲜度检测。
- 交易所返回超时但订单其实已成交 ⇒ Unknown 状态 + 对账。
- 同一规则在两层各写一遍，改一处忘一处（A4）⇒ 只留一个定义处，另一处调用。
- 错误只记 `error = %e` 却没有 `code` ⇒ 日志无法被程序聚合与告警（G4）。
- 数据被静默跳过或丢弃而没有计数 ⇒ 质量报告显示"正常"，实际缺数据（§3.1）。
- 把依赖层级（`层0`–`层5`）与领域级别（`RK-L*`、`BK-L*`、`BT-L*`）混用（§3.5）。
