# ADR-0012：WebPKI 根证书包的许可证（CDLA-Permissive-2.0）

- 状态：**已决策**（2026-11-09，采纳方案 1：白名单加 CDLA-Permissive-2.0）
- 日期：2026-11-09
- 触发：M1 `so-recorder` 引入 `tokio-tungstenite`（`rustls-tls-webpki-roots`）与
  `reqwest`（`rustls` → `rustls-platform-verifier`）后，`cargo deny check` 的
  licenses 阶段失败：`webpki-root-certs` / `webpki-roots` 声明
  `CDLA-Permissive-2.0`，不在 M0 白名单内。

## 背景

`cargo deny check` 的 licenses 白名单（M0 定，deny.toml）：
MIT / Apache-2.0 / Apache-2.0 WITH LLVM-exception / BSD-2-Clause /
BSD-3-Clause / ISC / Zlib / Unicode-3.0。

`webpki-roots` 系列 crate 是 rustls 生态的 WebPKI 根证书集：**代码**
是 Apache-2.0，内嵌的**根证书数据**由各 CA 提供，许可证标为
CDLA-Permissive-2.0（Community Data License Agreement，数据许可证）。
cargo-deny 按 crate 级 `license` 字段判定，因此整包被判为 CDLA。

影响面（当前依赖树）：
- `tokio-tungstenite 0.28.0` → `webpki-roots 0.26.11`（so-recorder WS）
- `reqwest 0.13.5` → `rustls-platform-verifier 0.7.1` → `webpki-root-certs 1.0.11`（so-recorder REST）

## 选项

1. **白名单加 CDLA-Permissive-2.0**（推荐）。CDLA-Permissive 是宽松数据
   许可证：允许使用、修改、再分发，无 copyleft 要求；与 MIT/Apache 精神
   一致。根证书数据本来就是公开基础设施（浏览器/OS 同源分发）。
2. 改用 `rustls-tls-native-roots`（读 OS 证书库）。绕开 webpki-roots，
   但行为依赖运行平台：VPS（Linux）需 `ca-certificates` 包；且
   tokio-tungstenite 0.28 的 native-roots feature 走 `rustls-native-certs`，
   增加了平台差异与测试盲区。
3. 自签 pin。为交易所固定证书/pin——交易所会轮换证书，pin 会导致周期性
   断连，且违背"不凭记忆写规则"原则（需持续跟踪各所证书变更）。
4. 不接 TLS（明文 ws://）。交易所只提供 wss://，不可行。

## 决策

**已决策（用户 2026-11-09 拍板）**：选 1——`deny.toml` 的 licenses 白名单
增加 `CDLA-Permissive-2.0`，并在注释中说明它只可能来自根证书数据包
（代码仍是 Apache-2.0）。

理由：CDLA-Permissive 对使用/修改/分发均无回授要求；根证书是公共事实数据；
白名单其余条款保持严格——新增任何**代码**许可证仍需单独 ADR。

## 影响

- `deny.toml`：licenses.allow 增加一项。
- 无代码影响。
- 若未来 cargo-deny 引入 `[licenses]` 的 `unused-allowed-license` 之类
  更严策略，需要复查本决策。
