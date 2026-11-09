# Futures WS 文档示例代码改动说明

## 改动范围

- `docs/futures/ws/zh_CN/index.md`
- `docs/futures/ws/en/index.md`

## 改动原则（已确认）

1. **仅修改 fenced 代码块**（\`\`\`python / \`\`\`go / \`\`\`json 等）内的内容
2. **不修改正文**：服务器地址列表、changelog、参数说明表格、解释性文字（如 `ob.BTC_USDT.400` 格式说明）保持原样
3. **WS 地址**：代码块内所有 WebSocket 连接 URL 统一为线上 USDT 地址  
   `wss://fx-ws.gateio.ws/v4/ws/usdt`
4. **合约 symbol**：代码块内示例 payload / 示例 JSON 中的 `BTC_USD`、`ETH_USD` 等统一为 `BTC_USDT`  
   - 已是 `BTC_USDT` 的不改  
   - `ob.BTC_USDT.*` 等流名称保持不变

## 替换规则

| 类型 | 原值（示例） | 新值 |
|------|-------------|------|
| WS URL | `wss://fx-ws-testnet.gateio.ws/v4/ws/btc` | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| WS URL | `wss://fx-ws-testnet.gateio.ws/v4/ws/usdt` | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| WS URL | `wss://ws-testnet.gate.com/v4/ws/futures/usdt` | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| WS URL | `wss://fx-ws.gateio.ws/v4/ws/btc` | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| WS URL | 含 `/sbe` 的 testnet/其他地址（仅代码块） | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| Symbol | `ETH_USD` | `BTC_USDT` |
| Symbol | `BTC_USD` | `BTC_USDT` |

## 不改动的内容

- 文档开头「线上交易 / 模拟盘」地址列表
- SBE 章节正文中的 prod/testnet 地址说明
- 参数表、频道说明等非代码块文字
- 已是 `BTC_USDT` 的 symbol 与 `ob.BTC_USDT.*` 流名

## 执行方式

使用脚本 `scripts/update_futures_ws_examples.py` 按代码块边界处理，避免误改正文。

Symbol 替换使用 `\b(?:ETH_USD|BTC_USD)\b` 词边界匹配，避免把已有的 `BTC_USDT` 误替换为 `BTC_USDTT`。

## 执行结果

- 已更新 `zh_CN/index.md`、`en/index.md`
- 正文中的 testnet / SBE 地址列表未改动
- 代码块内 WS URL 与示例 symbol 已全部校验通过

## 2026-06-03 推送示例同步（fex-ws）

依据 `fex-ws` 中 JSON 推送 DTO（`DecimalSizeJson` / `IntSizeJson`，忽略 SBE）更新：

- 各频道推送 JSON 示例补全缺失字段（如 `futures.tickers` 的 `change_from`、`change_price`、`price_type`、`t`；`futures.orders` 的 `id_string`、`pos_margin_mode`、`market_order_slip_ratio` 等）
- 张数相关字段：默认示例按 `X-Gate-Size-Decimal: 1` 使用字符串；另增「未带该 header」时的整型示例块
- 修正类型：`fill_price`/`price` 等为字符串；`futures.liquidates` 补充 `order_size`
- 脚本：`scripts/sync_futures_ws_push_from_fex_ws.py`（可重复执行部分替换）
