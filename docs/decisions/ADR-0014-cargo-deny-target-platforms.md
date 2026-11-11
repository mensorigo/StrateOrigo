# ADR-0014：cargo-deny 依赖图限定到项目目标平台

- 状态：**已决策**（2026-11-11，用户拍板方案 A）
- 日期：2026-11-11
- 触发：M2 `so-data` 引入 `arrow` / `parquet` 60.1.0 后，`cargo deny check`
  的 licenses 阶段失败：`tiny-keccak 2.0.2` 声明 `CC0-1.0`，不在
  M0 白名单内（deny.toml）。

## 背景

失败链路：

```
so-data → arrow → arrow-array → ahash 0.8
  （仅 [target.'cfg(target_arch = "wasm32")'.dependencies]，
   feature = ["compile-time-rng"]）
→ const-random → const-random-macro → tiny-keccak（CC0-1.0）
```

关键事实：`cargo tree -i tiny-keccak` 逐平台核实，该链**只在
`wasm32-unknown-unknown` 目标**下进入依赖图；项目两个实际目标平台
`x86_64-pc-windows-msvc` 与 `x86_64-unknown-linux-gnu` 下均无此依赖
（非 wasm 上 ahash 用 `runtime-rng` → getrandom，MIT/APL）。
cargo-deny 默认收集所有平台的依赖并统一判许可证，因此被无关平台的
传递依赖卡住。

项目目标平台（docs/02 §1）：Windows 11 x64（开发/研究）与
Linux x86_64（VPS），无浏览器/wasm 目标，未来也不排期。

## 选项

1. **`[graph] targets` 限定到两个目标平台**（采纳）。cargo-deny 原生
   支持按目标过滤依赖图；许可证白名单保持严格不变——新增任何进入
   实际构建的代码许可证仍需单独 ADR。
2. 白名单加 `CC0-1.0`。CC0 本身是宽松的放弃权利声明，但放宽白名单
   是全局动作：未来任何 crate 声明 CC0 都会静默通过，而它可能不止
   服务于 wasm 这一条无关路径。
3. 放弃 arrow 用别的列式库。违背 docs/02 §2.4 版本基线（arrow/parquet
   60.1），成本不可比。

## 决策

**方案 A（已采纳）**：`deny.toml` 的 `[graph]` 增加
`targets = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]`，
并在注释中指向本 ADR。

理由：cargo-deny 检查的应该是"实际会进入构建的代码"的许可证；
对永不构建的平台上的传递依赖放宽全局白名单，是把检查范围问题和
许可证政策问题混为一谈。限定 targets 后，白名单语义反而更严格、
更接近 docs/04 S7 的本意。

## 影响

- `deny.toml`：`[graph]` 增加 `targets` 两项。
- 无代码影响。
- 若未来新增目标平台（如 ARM VPS），须同步更新此列表并复查
  新平台的许可证面。
- `advisories` / `bans` / `sources` 检查同样只覆盖这两个平台——
  与实际部署面一致。
