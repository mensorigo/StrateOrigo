# 02 · 技术栈与版本

> **版本基线查询日期：2026-11-08**（crates.io / PyPI / Rust 官方发布通道实时查询）。
> 标注 "lock 时锁定" 的条目当天查询失败，初始化工程时以 `cargo update` / `uv lock` 解析出的版本写入锁文件，并回填到本表。
> **本表是"起点"，实际版本以 `Cargo.lock` / `uv.lock` 为准。**

## 1. 工具链

| 工具 | 版本 | 用途 | 备注 |
|---|---|---|---|
| Rust | **1.99.1**（stable，2026-10-13） | 核心语言 | `rust-toolchain.toml` 固定；edition **2024** |
| Cargo | 1.99.1（与 rustc 同版本号） | 构建 | |
| cargo-nextest | lock 时锁定 | 测试运行器 | `cargo install cargo-nextest --locked` |
| cargo-deny | lock 时锁定 | 许可证/漏洞/重复依赖检查 | CI 必跑 |
| Python | **3.12.x**（本机 3.12.6） | 研究环境 | numpy 2.5 / scipy 1.18 / xgboost 3.4 要求 ≥3.12 |
| uv | 0.12.23（本机） | Python 包/环境管理 | 替代 pip/poetry |
| maturin | **1.16.0** | 构建 PyO3 扩展 | |
| protoc | lock 时锁定 | Protobuf 编译 | 或用 `protox` 纯 Rust 编译，免装 protoc |
| Git | 2.55（本机） | 版本控制 | 需先 `git init` |
| Node / pnpm | 本机已有 24.21 / 12.16 | **不使用** | 无任何 Web 前端 |

**目标平台**

| 平台 | 用途 | 要求 |
|---|---|---|
| Windows 11 x64（MSVC） | 本地开发、数据处理、研究、回测 | 需安装 VS Build Tools（C++ 工作负载）+ Windows SDK |
| Linux x86_64（Ubuntu LTS） | VPS 实盘/模拟盘/录制 | 热路径优化（绑核、`io_uring` 可选）仅在 Linux 启用 |
| macOS | 不作为目标 | — |

## 2. Rust 核心依赖

### 2.1 基础与并发
| Crate | 版本基线 | 用途 | 使用范围 |
|---|---|---|---|
| tokio | 1.54.1 | 异步 I/O 运行时 | **仅 I/O 线程与冷路径**，内核线程禁止 |
| rtrb | 0.4.1 | 无锁 SPSC 环形队列 | 线程间热路径通信 |
| crossbeam | 0.8.5 | MPSC 通道、工具 | 冷路径 / 控制命令 |
| parking_lot | 0.13.0 | 锁 | **仅冷路径** |
| core_affinity | 0.8.3 | 线程绑核 | 内核线程 |
| quanta | 0.13.0 | 低开销高精度时钟（TSC） | LiveClock、延迟测量 |
| mimalloc | 0.1.52 | 全局分配器 | 所有二进制 |
| smallvec | 1.16.5 | 栈上小数组 | 热路径 |
| ahash | 0.8.12 | 快速哈希 | 冷路径 HashMap；热路径优先用数组下标 |

### 2.2 数值与时间
| Crate | 版本基线 | 用途 |
|---|---|---|
| rust_decimal | 1.43.0 | 冷路径会计、对账、报表（热路径用自研定点 `Price`/`Qty`） |
| chrono | 0.4.45 | 日期/交易日历（冷路径） |
| statrs | 0.19.1 | 统计分布（Rust 侧验证指标） |
| rand / rand_distr | 0.10.3 / 0.6.0 | 回测随机模型（必须显式种子） |
| ndarray | 0.17.2 | 特征矩阵（可选） |
| ort | lock 时锁定（2.x） | ONNX 模型推理（MF 策略；HF 优先手写线性/树模型） |

### 2.3 序列化与网络
| Crate | 版本基线 | 用途 |
|---|---|---|
| serde / serde_json | 1.0.229 / 1.0.154 | 配置、冷路径 |
| sonic-rs | 0.5.10 | 高性能 JSON 解析（行情热路径首选） |
| simd-json | 0.19.0 | 备选 JSON 解析器（两者做基准测试后二选一） |
| rkyv | 0.8.18 | 零拷贝序列化（Raw Journal 帧、快照） |
| tokio-tungstenite | 0.28.0 | WebSocket 客户端 |
| fastwebsockets | 0.10.0 | 备选低延迟 WS（基准测试后决定） |
| reqwest | 0.13.5 | REST（下单备用通道、快照、对账） |
| hmac / sha2 | 0.13.0 / 0.11.1 | 交易所签名：OKX 为 HMAC-SHA256 + Base64（另需 Passphrase），Gate 为 HMAC-SHA512（以官方文档为准） |
| base64 | lock 时锁定 | OKX 签名编码 |
| tonic / prost | 0.14.9 / 0.14.4 | 控制面 gRPC（so-cli ↔ so-trader） |
| teloxide 或直接调用 Bot API（reqwest） | lock 时锁定 | Telegram 告警与受限命令 |

### 2.4 数据存储
| Crate | 版本基线 | 用途 |
|---|---|---|
| arrow / parquet | 60.1.0 / 60.1.0 | 列式数据读写 |
| polars（Rust） | 0.55.2 | **可选**；Rust 侧优先直接用 arrow，避免重依赖 |
| zstd | lock 时锁定 | Raw Journal 压缩 |
| rusqlite | 0.41.1（`bundled`） | 订单/成交审计、策略注册表（WAL 模式） |
| redb | 4.5.0 | **可选**：内嵌 KV（状态快照） |
| clickhouse | 0.15.3 | **后期可选**：大规模 tick 查询 |

**存储选型说明**：初期只用 **SQLite（订单、成交、账户流水、审计、实验库）+ Parquet（行情与研究数据）**，VPS 上不额外运行数据库服务。**不使用 Redis 和消息队列**：实时状态在进程内存中，进程间通信用本机 Unix socket / 共享内存。当需要跨多个交易进程集中查询（或多机部署）时，再把交易记录迁移到 PostgreSQL；行情数据量过大时再引入 ClickHouse。

### 2.5 可观测性与工具
| Crate | 版本基线 | 用途 |
|---|---|---|
| tracing / tracing-subscriber | 0.1.44 / 0.3.23 | 结构化日志门面与订阅器（JSON 输出、`EnvFilter`、运行时调级 `reload`）；设计见 03 §18 |
| tracing-appender | 0.2.5 | 非阻塞写入 + 按天滚动日志文件 |
| pyo3-log | lock 时锁定（须兼容 pyo3 0.30） | `so-py` 中把 Rust 日志转给 Python `logging` |
| metrics | 0.24.6 | 指标门面；自研 recorder 写本地 Parquet/SQLite 时序（**不用 Prometheus**） |
| hdrhistogram | 7.6.0 | 延迟直方图 |
| ratatui + crossterm | lock 时锁定 | `so-cli watch` 终端实时监控界面 |
| thiserror / anyhow | 2.0.21 / 1.0.104 | 错误：库用 thiserror，二进制用 anyhow |
| clap | 4.6.7 | CLI |
| config | 0.15.27 | 分层配置（TOML + 环境变量） |
| uuid / ulid | 1.29.1 / 3.0.0 | ID（订单 ID 用自研单调整数 + 前缀，见 03） |

### 2.6 测试与基准
| Crate | 版本基线 | 用途 |
|---|---|---|
| proptest | 1.12.0 | 性质测试（OMS 状态机、撮合、定点运算） |
| insta | 1.49.0 | 快照测试（回测结果、解析结果） |
| criterion | 0.8.2 | 微基准 + 性能回归门槛 |

### 2.7 PyO3 绑定
| Crate | 版本基线 | 用途 |
|---|---|---|
| pyo3 | 0.30.1 | Rust ↔ Python 绑定（`abi3-py312`） |
| arrow 的 pyarrow 互操作（C Data Interface） | 随 arrow 60.1 | 回测结果零拷贝传给 Python |

### 2.8 参考实现（只读参考，不作为依赖）
| 项目 | 版本 | 参考价值 | 许可证注意 |
|---|---|---|---|
| hftbacktest | crate 0.9.4 / PyPI 2.4.4 | BT-L2 回测的延迟与排队模型；**M3 回测内核阶段用作对照基准** | MIT，可借鉴 |
| NautilusTrader | PyPI 1.231.0 / crate 0.67.0 | 领域模型、订单状态机、适配器结构 | **LGPL-3.0**，不得复制代码，只参考设计 |
| barter | 0.14.0 | Rust 事件驱动交易框架设计 | MIT |

## 3. 桌面端（P2，实盘稳定后才开始）

> 以下仅为技术选型的预留记录，M0~M8 期间**不引入**任何 GPUI 依赖。

| Crate | 版本 | 说明 |
|---|---|---|
| **gpui-component** | **=0.9.0**（2026-11-02） | Longbridge GPUI Kit 组件库，60+ 组件（Table、List、Dock、Tabs、Chart、Form、Theme 等） |
| **gpui-pre** | **=0.3.11**（2026-11-02） | gpui-component 0.9.0 **精确锁定**依赖的 GPUI 发行版 |
| gpui-kit-assets / gpui-base | ^0.9.0 | gpui-component 的配套依赖，随之升级 |
| smol | ^2 | gpui-component 使用的异步运行时 |
| tonic（客户端） | 0.14.9 | 连接 `so-trader` 控制面 |

**关键约束**
1. **不要同时依赖 crates.io 上 Zed 官方的 `gpui`（0.2.2）**。gpui-component 用的是 `gpui-pre`，混用会出现两套 GPUI 类型，无法编译。代码里通过 gpui-component 重新导出的 gpui 类型使用。
2. `gpui-component` 与 `gpui-pre` **只能成对升级**，升级单独开分支，跑完桌面端回归清单再合并。
3. 桌面端是**独立的 Cargo workspace**（`apps/so-desktop/`），通过 path 依赖复用 `so-model`、`so-proto`。这样：
   - VPS 构建不编译 GPUI；
   - GPUI 的依赖版本（如 smol、某些图形库）不会约束核心引擎的依赖解析。
4. 专业 K 线（蜡烛 + 成交量 + 指标 + 十字光标 + 缩放平移）预计需在 GPUI canvas 上**自研组件**；先评估 gpui-component 的 Chart 能否满足，再决定。
5. 中文字体：显式打包或指定系统字体（如 Microsoft YaHei / Noto Sans CJK），避免不同平台回退字体不一致。
6. 许可证：初始化时用 `cargo deny check licenses` 确认 gpui-pre / gpui-component 的许可证。

## 4. Python 研究环境

| 包 | 版本基线 | 用途 |
|---|---|---|
| numpy | 2.5.4 | 数值基础（要求 Python ≥3.12） |
| polars | **2.1.0** | 主力数据框（优先于 pandas） |
| pyarrow | 26.0.1 | Parquet、与 Rust 零拷贝 |
| duckdb | lock 时锁定 | SQL 查询 Parquet |
| pandas | 3.1.1 | 仅兼容第三方库时使用 |
| scipy | 1.18.1 | 统计、优化 |
| statsmodels | 0.15.0 | 时间序列、协整、回归 |
| arch | 8.0.0 | GARCH 波动率、**SPA / Reality Check 检验**、Bootstrap |
| scikit-learn | 1.9.1 | 机器学习基础、交叉验证 |
| lightgbm | 4.7.0 | 梯度提升树（主力 ML 模型） |
| xgboost | 3.4.1 | 备选 |
| torch | 2.15.0 | **可选**，深度模型（后期） |
| skl2onnx / onnx | 1.20.0 / 1.23.2 | 模型导出给 Rust |
| numba | 0.68.1 | 自定义特征加速 |
| optuna | 5.0.0 | 参数优化（TPE、多目标、剪枝）；存储用本地 SQLite，不用其 dashboard |
| matplotlib | lock 时锁定 | 静态图（PNG），用于报告与 Notebook |
| ipykernel | lock 时锁定 | VS Code Notebook 内核（**不用浏览器版 JupyterLab**） |
| vectorbt | 1.2.0 | **可选**：向量化快速初筛 |
| pytest / hypothesis | 9.2.0 / 6.170.15 | 测试 |
| ruff | 0.17.4 | lint + format |
| mypy 或 pyright | 2.4.1 / 1.1.414 | 类型检查（二选一，推荐 pyright） |

**不引入**：ccxt（实盘适配器自研，ccxt 只可用于研究阶段拉取低频历史数据，按需安装）；mlflow、plotly、jupyterlab、quantstats 等依赖 Web 界面或输出 HTML 的工具。

**实验追踪**：自研 `strateorigo.registry`，元数据存 SQLite（`data/experiments.db`），曲线/成交等结果存 Parquet，报告为 Markdown + PNG，用 `so-cli exp list/show/compare` 或 Python 查询。

## 5. 基础设施与第三方服务

| 组件 | 选择 | 说明 |
|---|---|---|
| 监控 | `so-cli watch`（ratatui 终端界面，SSH 上直接用）+ 指标落本地时序文件 | 无 Web 看板 |
| 告警 | Telegram Bot（主）/ ntfy（备） | 守护进程与 so-trader 双通道；Telegram 支持白名单用户的 `/status`、`/halt` 等受限命令 |
| 内网 | WireGuard 或 Tailscale | 控制面唯一入口 |
| 进程管理 | systemd | `Restart=on-failure`，交易进程**不自动重启**（见 04） |
| 加密历史 BK-L2 | Tardis.dev（付费） | 补齐录制开始之前的历史盘口 |
| 加密公共历史 | OKX / Gate 官方历史数据下载 | 逐笔 / K 线等（以官网实际提供为准） |
| 美股数据 | Databento / Polygon | 按需 |
| A 股数据 | QMT 内置数据 / Tushare | 按需 |
| 美股券商 | Interactive Brokers（IB Gateway + TWS API） | 中低频 |
| A 股接入 | miniQMT（**仅 Windows**，Python） | 通过桥接进程接入，见 04 §7 |

## 6. 版本管理策略

1. **锁文件入库**：`Cargo.lock`、`apps/so-desktop/Cargo.lock`、`python/uv.lock` 全部提交。
2. **工具链固定**：`rust-toolchain.toml` 写 `channel = "stable"`（避免 rustup 另装一份同版本工具链），真正的版本固定是工作区 `Cargo.toml` 的 `rust-version = "1.99.1"` + `scripts/check.ps1` 校验 rustc 版本；`.python-version` 写 `3.12`。升级 Rust 时三处一起改。
3. **依赖声明**：根 `Cargo.toml` 使用 `[workspace.dependencies]` 统一声明版本，成员 crate 一律 `xxx.workspace = true`。
4. **升级节奏**：每月一个升级窗口，单独分支；升级后必须通过：全量测试 + 基准无回退 + 回测快照一致（或已解释的差异）。
5. **冻结期**：实盘新部署前 7 天内不升级任何依赖。
6. **安全**：CI 跑 `cargo deny check`（advisories、licenses、bans、sources）；Python 端跑 `uv pip audit` 或 `pip-audit`。
7. **最小化依赖**：热路径 crate（`so-core`、`so-book`、`so-engine`、`so-risk`、`so-oms`）新增依赖须在 PR 里写明理由。
