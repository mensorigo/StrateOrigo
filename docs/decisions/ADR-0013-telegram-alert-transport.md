# ADR-0013：Telegram 告警的传输实现与重试策略（so-recorder）

- 状态：**待决策**（2026-11-10 提出，待用户确认）
- 日期：2026-11-10
- 触发：M1 交付物"so-recorder 静默断流 → Telegram 告警"（docs/05 M1）。
  docs/02 §68 只锁定了"teloxide 或直接调用 Bot API（reqwest）"两个候选，
  未覆盖：选型、队列/背压、重试与丢弃策略、非交易进程的消息前缀。

## 背景

- so-recorder 是冷路径进程（VPS 独立运行，docs/06 §3.2 A8），已依赖
  reqwest（REST 快照）与 tokio。
- 告警来源目前只有静默检测事件（docs/06 §3.2 A5）；
  docs/03 §595 要求"告警不靠扫描日志，由代码显式发出"，且告警
  "不可丢（发送失败重试）"。
- docs/04 S10 要求所有 Telegram 消息带模式标识前缀
  （`[LIVE]` / `[PAPER]` / `[DEMO]`）——但 so-recorder 不是交易模式，
  无对应前缀。

## 选项

1. **直接调用 Bot API（reqwest）**（推荐）。零新增依赖；告警只有
   `sendMessage` 一个调用面，用不上 teloxide 的更新轮询/命令分发
   （受限命令属 M6+ so-trader/so-guardian 的需求，届时按需评估）。
   Bot API 是稳定 HTTPS JSON 接口，无鉴权签名复杂度。
2. teloxide。功能全（命令处理、对话流），但引入 30+ 传递依赖
   （teloxide-core、rand 等），对"只发不收"的录制告警是过度设计；
   命令能力在 M6 前用不到，届时引入也来得及。

## 决策（建议方案 1，待确认）

- **传输**：直接调用 Bot API（`POST /bot{token}/sendMessage`，reqwest）。
- **token**：从环境变量读取（配置文件只写变量名，docs/04 S8——密钥不落盘），
  `chat_id` 写配置。
- **队列/背压**：帧处理只做非阻塞入队（`tokio::sync::mpsc::unbounded`），
  发送在独立后台任务。告警量极小（静默事件级），无界队列实际无溢出风险；
  录制路径永不因告警阻塞。
- **重试**：每条消息最多 5 次尝试；指数退避 1s→2s→4s→8s→16s（封顶 60s）；
  HTTP 429 时改用响应 `parameters.retry_after`。耗尽后记 ERROR 日志
  并丢弃（不能阻塞录制进程）。
  注：docs/03 §592 称告警"不可丢"——该表针对交易域告警（so-trader /
  so-guardian，有 SQLite 审计持久化）；录制告警以日志为兜底记录，
  5 次重试后丢弃属有意取舍。
- **关闭收尾**：Ctrl-C 关闭信号到达时尽力清空队列（每条只试一次），
  不阻塞 Journal 收尾。
- **前缀**：非交易进程无 `[LIVE]` 前缀可用，采用 `[REC]` 标识录制器
  （数据面进程）。S10 的字面前缀集合未来可考虑补 `[REC]`。
- **未配置行为**：token 环境变量缺失或 chat_id 为空 → 告警任务不启动，
  记 WARN 日志，静默事件仍写 tracing 日志。录制不因告警未配置而失败。

## 影响

- 新增 `crates/so-recorder/src/alert.rs`（发送任务 + 句柄）；
  `RecorderError` 增加 `Alert` 变体（错误码 `RecorderAlert`）。
- 零新增第三方依赖。
- 若后续 M6 的 so-trader 告警需要命令分发（`/status` `/halt`），
  再单独决策是否引入 teloxide。
