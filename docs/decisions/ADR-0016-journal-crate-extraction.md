# ADR-0016：Journal 文件格式下沉重构为 `so-journal` crate

- 状态：**已接受并已实施**（2026-11-11）
- 日期：2026-11-11
- 触发：执行 [ADR-0015](ADR-0015-style-and-decoupling-standard.md) 第 9 项（journal 格式归属）
- 取代：ADR-0015 第 9 项（该项列的三个选项在此落定为选项 2）

## 背景

Raw Journal 的**写入**在 `so-recorder`、**读取**在 `so-data`，于是数据层依赖了录制进程层：

```
so-data ──依赖──> so-recorder        # 同层依赖（层3 → 层3），违反 04 §3.5 A1
```

房地产业式的"共用格式定义"本应下沉，但当时格式定义与写入器一起被留在 `so-recorder`
（一个带 `tokio` / `reqwest` / `tokio-tungstenite` 的**进程型** crate）。

### 实测代价（2026-11-11）

口径：`cargo tree -p so-data -e normal --prefix none | grep -E '^[a-zA-Z]' | sed 's/ v[0-9].*//' | sort -u | wc -l`
（唯一 **crate 名**，不区分同一 crate 的多个版本；消除前的数字由 `git worktree add 75df20a` 实测得到，
不是估算——初稿曾误写为 250，系早期用不同口径测得，2026-11-12 已按同一命令复测纠正。）

| 指标 | 消除前 | 消除后 |
|---|---|---|
| `so-data` 生产依赖图的唯一 crate 数 | 204 | **95** |
| 其中 `tokio` / `hyper` / `reqwest` / `tokio-tungstenite` / `rustls` 的行数 | 37 | **0** |
| `so-data` 图上是否出现 `so-recorder` | 是 | 否 |

即：数据层为了拿一个 `JournalRow`，连整个 WebSocket 网络栈（37 个依赖行）一起编译。
而 `so-data` 的**生产代码**只用到 `JournalRow`（`JournalWriter` 仅出现在 `#[cfg(test)]`）。

### 为什么必须处理（而不是"忍一下"）

1. **编译与依赖面**：改一行 `so-data` 要重编 204 个 crate 的依赖图；CI 时间与磁盘都是实测成本。
2. **分层原则自相矛盾**：`so-data` 是纯数据变换（可离线、可在研究域跑），
   `so-recorder` 是长驻采集进程（连 VPS、连交易所）。让前者依赖后者，
   等于说"离线清洗数据需要网络栈"——这与 [04 §3.5 A1](04-dev-constraints.md) 的层号设计直接冲突。
3. **例外表会变成永久豁免**：ADR-0015 允许把它登记为例外行，但"登记后没人再动"是这类表的常态，
   而 A7 的机检脚本恰好能强制"例外消失后必须删行"。

## 决策

新建 **`crates/so-journal`（层2）**，只放 Journal 的**文件格式与读写**，不引入网络/异步依赖：

| 模块 | 内容 | 来源 |
|---|---|---|
| `row.rs` | `JournalRow` 行模型 + `to_jsonl` / `from_jsonl` | 原 `so-recorder::journal` |
| `writer.rs` | `JournalWriter` / `JournalStats`：按小时滚动 + zstd 多帧 + sha256 边车 | 原 `so-recorder::journal` |
| `date.rs` | `format_utc_date` / `format_utc_time`（Howard Hinnant 换算） | 原 `so-recorder::timefmt` |
| `hash.rs` | `sha256_file`（流式） | 原 `so-recorder::hash` |
| `dedup.rs` | `Dedup` / `DedupKey` / `dedup_key_from_frame`（A/B 双流去重） | 原 `so-recorder::dedup` |
| `error.rs` | `JournalError`，码前缀 `Journal*`（`Open`/`Write`/`Codec`/`Row`/`NotRolled`/`Checksum`） | 新建（原用 `RecorderError::Journal(String)`） |

依赖：`so-core` + `thiserror` + `serde` + `serde_json` + `sha2` + `zstd`——**全是冷路径形态**，没有 `tokio`。

### 消除的特例：日期格式化为什么也一起下沉

`format_utc_date` 决定文件路径（`{venue}/{YYYY-MM-DD}/{HH}.{conn}.journal.zst`），
是**格式的一部分**：读端要按同样的规则推算路径。留在 `so-recorder` 会让读端重新实现一遍换算，
两份实现迟早漂移。进程时钟 `now_nanos()` **不下沉**——它属于采集进程，不属于文件格式。

### 写入端与读端的接口

- `so-recorder` 保留同名模块作为**重新导出壳**（`crate::journal` / `crate::dedup` / `crate::hash`），
  20+ 处既有调用点无需改动；新增代码应直接 `use so_journal::…`。
- `so-recorder::RecorderError::Journal` 改为 `Journal(#[from] JournalError)`，
  `code()` **转发** `so_journal` 的码而不是折叠成 `RecorderJournal`——
  折叠会让"文件打开失败"与"zstd 编码失败"在日志里无法区分（04 E2）。

## 后果

- **`so-data` 与 `so-recorder` 之间不再有依赖边**，04 §3.5 同层例外表回到空表。
- `so-journal` 的提取让 `so-data` 的生产依赖图从 204 个 crate 降到 95 个（−53%）。
- `so-data` 的 Parquet 输出**逐字节不变**（41 个文件 sha256 全部一致，聚合哈希
  `94d1ffb0a772096d94b6f7f2b35bd242cd14a04bba30a0a16bb3a1d85ef38ea0` 前后相同）——
  证明这是纯粹的重构，没有语义改动。
- 新增规则约束：`so-journal` 是**叶子 crate**，不得新增网络/异步/GUI 依赖；
  由 `scripts/check-deps.ps1` 的层级检查与 `deny.toml` 共同兜底。
- 文档同步：docs/03 §1 仓库树与依赖图、§10 模块职责表、docs/04 §3.3 错误码登记表。

## 未采用方案（ADR-0015 第 9 项列的另两项）

1. **下沉到 `so-core`**：`so-core` 当时的生产依赖恰好只有 `thiserror`（编译期，零运行时代码）。
   Journal 需要 `zstd`/`serde_json`/`sha2`/`serde` 四个真实依赖，且属冷路径；
   把它们塞进**每个 crate 都要依赖**的层0，等于让热路径 crate 无条件背冷路径依赖（违反 A1/I13）。
2. **保持现状并登记例外**：接受 204 个 crate 的构建代价与一条永久例外，
   且掩盖"离线数据清洗依赖网络栈"这个架构信号。成本最低但把问题固化。
