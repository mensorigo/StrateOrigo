# OKX 接入参考（API v5）

> 整理自官方文档英文版（抓取于 2026-11-09；本地快照 `docs/okx-api-v5-snapshot/okx-api-v5-en.md`，SHA256 见快照 README）。
> 本文是工程速查，字段语义以官方文档为准；写代码前与本地快照核对，官网为准。
> 范围：M1 行情接入全覆盖；交易部分（§6）只列概要，**M6 写交易前必须补齐并重新核对**。

## 1. 服务地址与运行环境

| 项 | 值 |
|---|---|
| REST 实盘 | `https://www.okx.com`（2026-05-20 起新增专用域名 `https://openapi.okx.com`，官方建议 REST 优先使用；两者完全等价） |
| REST 模拟（demo） | `https://www.okx.com` + 请求头 `x-simulated-trading: 1` |
| 公共 WS | `wss://ws.okx.com/ws/v5/public`（端口 443；**8443 端口将退役**，官方已发 64010 通知，新代码一律用 443） |
| 私有 WS | `wss://ws.okx.com/ws/v5/private` |
| 业务 WS | `wss://ws.okx.com/ws/v5/business` |
| demo 公共/私有/业务 WS | `wss://wspap.okx.com/ws/v5/{public\|private\|business}` |
| SBE 二进制行情 | `wss://ws.okx.com/ws/v5/public-sbe`（XML 模板见官方文档 SBE 节）；M1 先 JSON，SBE 列为后续优化 |
| 数据中心 | 官方文档未明示机房位置；业内普遍认为撮合在阿里云香港。**VPS 选址以实测 RTT 为准**（05 M1） |
| demo Key | 模拟盘 API Key 在"交易→模拟交易→个人中心"单独创建，与实盘 Key 不通用 |

## 2. WebSocket 通用规则

### 2.1 消息格式

```json
// 订阅（公共频道无需 auth）
{ "id": "1512", "op": "subscribe",
  "args": [ { "channel": "books", "instId": "BTC-USDT-SWAP" } ] }
// 确认 / 数据推送 / 错误
{ "id": "1512", "event": "subscribe", "arg": {...}, "connId": "a4d3ae55" }
{ "arg": {...}, "action": "snapshot" | "update", "data": [...] }   // 推送
{ "id": "1512", "event": "error", "code": "60012", "msg": "...", "connId": "..." }
```

- `id`：客户端自定义请求标识（≤32 字符字母数字），原样返回，用于关联请求与响应。
- 批量订阅：`args` 可含多个频道参数，总长度 ≤ 64 KB。
- 服务器主动断连前 60 秒发 `event: "notice"`、`code: 64008`（服务升级）——**收到即主动重连**。
- `connId` 唯一标识连接，重连后变化；按 `connId` 区分 A/B 双流。

### 2.2 连接与订阅限额

| 限制 | 值 | 维度 |
|---|---|---|
| 建连频率 | 3 次/秒 | IP |
| subscribe/unsubscribe/login 请求 | 合计 480 次/小时 | 每连接 |
| 私有频道连接数 | 每频道每子账户 30 条（orders/account/positions/balance/风险/greeks） | 子账户 |
| 单品种挂单数上限 | 500（限价/市价/post-only/IOC/FOK/TP/SL 及其触发单均计入） | 账户 |
| 全账户挂单数上限 | 4,000 | 账户 |
| taker 单撮合 maker 数上限 | 1,000（超限部分撤销，FOK 整单撤销） | — |

- 超连接数限制时，最新订阅被拒：先收到正常 subscribe 确认，随后 `channel-conn-count-error`。

### 2.3 心跳与断线

- **服务器**每 20 秒发 WS ping（opcode 9）；60 秒内未收到客户端 pong 即断连。客户端收到 ping 须尽快回 pong（opcode 10，回传相同 payload）。
- **服务器 30 秒无数据可推送也会主动断连**（close frame 4004 `No data received in 30s`）——行情频道自身有约 60 秒空推送机制（见 §3.1），正常不会触发。
- 客户端可随时发 `ping`（文本帧），服务器回 `pong`——健康探测用应用层 ping。
- close frame 语义表：4001 登录失败、4002 非法请求、4003 订阅量超限、4004 无数据、4006 异常断连、4007 Key 变更、4008/4009 订阅频道数超限。

### 2.4 私有频道鉴权（WS login）

```json
{ "op": "login",
  "args": [ { "apiKey": "...", "passphrase": "...", "timestamp": "1538054050",
              "sign": "Base64(HMAC_SHA256(timestamp + 'GET' + '/users/self/verify', secret))" } ] }
```

- `timestamp` 为**秒**级 Unix 时间；`method` 固定 `GET`；`requestPath` 固定 `/users/self/verify`。
- 登录失败错误码：60009；一连接只允许一次 login（60031）。

## 3. 永续合约（USDT 结算）行情频道

M1 优先接入。品种命名 `BTC-USDT-SWAP`（连字符，**不是** Gate 的下划线）。

### 3.1 深度频道（重点：频道选型 + seqId 校验）

| 频道 | 模式 | 推送频率 | 内容 |
|---|---|---|---|
| `books` | 快照 + 增量 | 增量 100ms | 400 档 |
| `books-l2-tbt` | 快照 + 增量 | **增量 10ms** | 400 档逐笔 |
| `books50-l2-tbt` | 快照 + 增量 | **增量 10ms** | 50 档逐笔 |
| `bbo-tbt` | 快照 | **10ms** | 最优买卖 1 档（无 `action` 字段） |
| `books5` | 快照 | 100ms | 5 档（快照式，无增量） |
| `books-rpi` | 快照 + 增量 | 100ms | 有机单+RPI 合并深度（M1 不用） |

**推送数据格式**（`books` / `books-l2-tbt` / `books50-l2-tbt`）：

```json
{ "arg": { "channel": "books", "instId": "BTC-USDT" },
  "action": "snapshot",            // snapshot | update
  "data": [ {
    "asks": [ ["8476.98","415","0","13"], ... ],   // [价格, 数量, 订单数, 虚拟数量RPI]
    "bids": [ ... ],
    "ts": "1597026383085",          // 毫秒
    "checksum": 0,                  // ★已废弃，恒为 0
    "prevSeqId": -1,                // 快照恒为 -1
    "seqId": 123456 } ] }
```

**seqId 校验规则（M1 核心，对应 03 §5）**：

1. 每个品种独立序列；同一频道多连接收到的 seqId 集合相同（A/B 双流去重依据）。
2. 正常增量：`prevSeqId == 上一条 seqId`，且 `seqId > prevSeqId`。
3. **60 秒左右无深度变化**：增量频道推送空消息 `asks:[], bids:[]`，且 `seqId == prevSeqId`（活跃探测，不是错误）；快照式频道推送最新全量。
4. **维护后序列重置**：可能出现 `seqId < prevSeqId`，之后恢复正常递增——处理为"重置"而不是缺口。
5. `checksum` 字段**已废弃**（恒 0，官方明确不再用于完整性校验）。**这与 docs/03 §5 中"OKX 校验和"的旧描述不符，以官方当前文档为准：只用 seqId/prevSeqId 校验**。
6. 同连接同品种**不能同时订阅** `books-l2-tbt` 与 `books50-l2-tbt`/`books`（错误码 64004）。

**本地订单簿维护（官方流程）**：首条 `action: "snapshot"` 全量替换 → 后续 `update` 增量应用（`[px, sz, ...]` 中 **sz 为该档位最新总量，覆盖而非累加**；sz=0 删档）→ `prevSeqId` 断裂 ⇒ 标记 Gap、重新订阅（OKX 无 REST 快照对齐步骤，直接重新订阅拿新 snapshot）。

### 3.2 逐笔成交 `trades`

```json
{ "data": [ { "instId": "BTC-USDT", "tradeId": "130639474", "px": "42219.9",
    "sz": "0.12060306", "side": "buy", "ts": "1630048897897",
    "count": "3", "source": "0", "seqId": 1234 } ] }
```

- **聚合推送**：一条消息可聚合多笔成交；`tradeId` 是**聚合内最后一笔**的 ID，`count` 为聚合笔数——与 Gate 逐笔不同，**OKX 无法从 trades 频道还原逐笔明细**（HF 回测需要逐笔时用 REST `trades-history` 或 Tardis 数据对照）。
- `side` 为 **taker 方向**（buy=主动买）。
- `source`：`0` 普通单、`1` ELP 流动性计划单。
- SWAP 的 `sz` 单位为**张**（合约数），换算需 `ctVal`。

### 3.3 其他行情频道

| 频道 | 速度 | 说明 |
|---|---|---|
| `tickers` | 100ms（有事件才推） | BBO、24h 量、`sodUtc0/8` |
| `mark-price` | 变化时 200ms，无变化 10s | `markPx` |
| `funding-rate` | 30–90s | 见 §3.4 |
| `open-interest` | — | OI（`oi`、`oiCcy`、`oiUsd`） |
| `liquidation-orders` | 事件驱动 | 订阅按 `instType: SWAP`；`details` 数组含 `bkPx/sz/posSide/ts`；**不代表全所强平总量**（抽样推送） |
| `ADL warning channel` | 事件驱动 | ADL 排名预警（私有） |
| `candlesticks` | — | K 线 1s–1M；interval 表见官方文档 |

### 3.4 资金费率频道 `funding-rate`（S2 策略核心数据）

关键字段：`fundingRate`（预测值）、`fundingTime`/`nextFundingTime`（结算时刻，ms）、`minFundingRate`/`maxFundingRate`（±0.375% 上限）、`premium`（溢价指数）、`settState`（processing/settled）+ `settFundingRate`（**实际结算费率**——回测要用这个而不是预测值）、`method: current_period`。

- 结算间隔通常 8 小时，但**可变**——用 `nextFundingTime - fundingTime` 差值确定实际间隔。
- REST：`GET /api/v5/public/funding-rate`（10 次/2s，IP+品种）、`GET /api/v5/public/funding-rate-history`（3 个月，10 次/2s）。

### 3.5 品种静态信息（`GET /api/v5/public/instruments`，`VenueCapabilities` 与品种元数据来源）

限频 20 次/2s（IP+instType）。关键字段：

| 字段 | 含义 | 用途 |
|---|---|---|
| `instId` | 如 `BTC-USDT-SWAP` | InstrumentId 映射 |
| `ctType` | `linear`（U 本位）/ `inverse`（币本位） | 品种分类；反向合约 P&L 非线性 |
| `ctVal` / `ctMult` / `ctValCcy` | 一张合约面值 / 乘数 / 面值币种 | **张数 ↔ 币数换算**（linear: 名义 = sz × ctVal × markPx） |
| `settleCcy` | 保证金/结算币 | S2 现货-永续对腿匹配 |
| `tickSz` | 价格最小变动 | `Price::round_to_step` |
| `lotSz` / `minSz` | 数量步长 / 最小下单量 | sz 必须是 lotSz 倍数（违规 51121）且 ≥ minSz |
| `maxLmtSz` / `maxMktSz` | 单笔限价/市价最大量（市价为 USDT 计） | 风控 RK-L1 |
| `lever` | 品种杠杆上限（账户实际可用更低） | 风控 |
| `state` | `live / suspend / post_only / preopen / test / settling...` | 只交易 `live`；`post_only` 只收 post-only 单 |
| `listTime` / `expTime` | 上市/下架时间 | 时点元数据（06 §4.2） |
| `initPxLmtPct` / `floatPxLmtPct` / `maxPxLmtPct` | 价格限制带（上市初 10 分钟/常态/硬上限） | 价格带风控；精确值用 `GET /api/v5/public/price-limit` |
| `upcChg` | **即将生效的参数变更**（tickSz/minSz/maxMktSz + 生效时间） | 时点元数据必须覆盖 |
| `instFamily` / `uly` | 品种族（如 `BTC-USDT`）/ 指数 | 订阅与查询参数；`uly` 已被 instFamily 取代（错误码 64000） |

## 4. 现货（次要，S2 需要时接）

- 频道与合约共用同一 WS 与消息框架（`instId: BTC-USDT`）。
- 深度频道同 §3.1（现货同样支持 `books`/`books-l2-tbt`/`bbo-tbt`）。
- `trades`：`px/sz` 十进制字符串；`sz` 单位为 base 币数量（**无"张"概念**）。
- instruments：`baseCcy/quoteCcy`、`lotSz/minSz` 为币数量。

## 5. REST 要点

### 5.1 认证（私有接口）

请求头：`OK-ACCESS-KEY`、`OK-ACCESS-SIGN`、`OK-ACCESS-TIMESTAMP`（**毫秒 ISO 8601**，如 `2020-12-08T09:08:57.715Z`）、`OK-ACCESS-PASSPHRASE`；demo 另加 `x-simulated-trading: 1`。

签名：`Base64(HMAC_SHA256(timestamp + method + requestPath + body, secret))`。

- `method` 大写；GET 的 body 为空字符串（**query string 计入 requestPath**，如 `/api/v5/account/balance?ccy=BTC`）。
- 时间偏差 > 30 秒拒收（错误 50102）——**下单前先 `GET /api/v5/public/time` 校时**；chrony offset < 1ms 的启动检查（04 §8）仍然保留。

### 5.2 限频（速率以文档"Rate Limits"节为准）

| 接口 | 限制 | 维度 |
|---|---|---|
| 下单/撤单/改单（单笔） | 60 次/2s | 品种（REST 与 WS **共享**额度） |
| 批量下单/改单 | 300 单/2s | 品种 |
| 公共行情（instruments 等） | 20 次/2s | IP + instType |
| funding-rate | 10 次/2s | IP + 品种 |
| WS 订阅请求 | 480 次/小时 | 连接 |

- **REST 与 WS 下单共享同一限额**——RK-L1 风控令牌桶必须合并两通道计数。
- **子账户级限频**：下单+改单合计 1,000 次/2s（错误码 50061）；VIP5+ 按成交率分层可升至 10,000 次/2s（成交率 = 成交量 USDT / 加权请求数，含 symbol multiplier）。ADR-0001 一账户一策略下，此限制按子账户独立计算。
- 改单并发上限：每单同时最多 3 个在途 amend（错误码 51513）。
- `GET /api/v5/trade/account-rate-limit` 查询当前实际限频与成交率（每日 08:00 UTC 更新）。

### 5.3 错误处理

非 2xx 或业务失败返回 `{"code": "...", "msg": "..."}`；批量接口每单用 `sCode/sMsg`。**程序判别用数字错误码**（REST 50000–59999、WS 60000–64010，附录见快照 1,735,183 / 1,846,287 偏移处）。常用：

- 通用：50011 限频、50013 系统繁忙、50004 请求超时（**不代表订单成败**——进入 Unknown 对账流程，04 §8）
- 认证：50102 时间戳偏差、50113 签名无效
- 订单：51008 余额不足、51121 数量非 lotSz 倍数、51119 价格非 tickSz 倍数、51016 品种不存在、51513 改单并发超限、51603 订单不存在
- WS：60009 登录失败、64004 频道互斥、64008 服务升级将断连、64010 8443 端口将退役
- 存在**子错误码**格式（如 `51008_1000`）——错误枚举匹配主码前先按 `_` 截断。

### 5.4 自成交防护（STP）

下单参数 `stpMode`：`cancel_maker`（默认）/ `cancel_taker` / `cancel_both`（不支持 FOK）。账户级默认 `acctStpMode` 可在网页端设置，下单参数可覆盖。对应 03 §7 自成交防护行——M6 交易接入时确认账户级配置。

## 6. 交易接口概要（M6 前必须补齐细节并重新核对）

| 项 | 概要 |
|---|---|
| 下单 | `POST /api/v5/trade/order`；WS 下单 `op: "order"`（REST 与 WS 共享限频）；批量 20 单 |
| 订单类型 | `market / limit / post_only / fok / ioc / optimal_limit_ioc / mmp` |
| `clOrdId` | 客户端订单 ID，≤32 字符字母数字——`ClientOrderId` 编码（03 §3）映射到此 |
| 改单 | `POST /api/v5/trade/amend-order`（每单在途 amend ≤ 3） |
| 撤单 | `POST /api/v5/trade/cancel-order`、批量、`POST /api/v5/trade/cancel-all-orders` |
| **Dead Man's Switch** | `POST /api/v5/trade/cancel-all-after`（超时毫秒数，到期自动撤单；续期即重发）——so-guardian RK-L3 的 DMS 方案用此接口 |
| 超时保护 | 请求头/WS 参数 `expTime`：服务器时间超过后丢弃请求（不处理），防网络延迟导致的迟到单 |
| 订单状态推送 | 私有频道 `orders`（全生命周期，含 `state`、`avgPx`、`accFillSz`、`fee`）、`fills` 成交频道 |
| 持仓模式 | `net` 单向 / `longShort` 双向（`POST /api/v5/account/set-position-mode`） |
| 对账 | `GET /api/v5/trade/orders-pending`、`GET /api/v5/account/positions`、`GET /api/v5/account/balance`、bills 明细（7 天/3 个月/2021 起） |

## 7. 对设计文档的影响（实现时核对）

1. **`so-book` 的 OKX 校验只用 `seqId`/`prevSeqId`**，无校验和（docs/03 §5 的"OKX 校验和"描述过时，需修订）；序列重置（seqId 变小）与缺口（prevSeqId 断裂）要区分处理。
2. **深度增量 `sz` 是覆盖量**（该档最新总量，非增量），与 Gate 相同；但 OKX 无 Gate 式 REST 快照-增量衔接六步法，重订订阅拿 snapshot 即可。
3. **`trades` 频道是聚合推送**（tradeId 为聚合内最后一笔、count 为笔数）——06 §4.1 `trade` 表的 `trade_id` 语义在 OKX 上要按聚合记录，HF 逐笔需求用 REST `trades-history`/Tardis 补。
4. **REST 与 WS 下单共享限频**，RK-L1 风控令牌桶（03 §7）必须合并计数；另有子账户级 1,000 次/2s 总闸与成交率分层（VIP5+）。
5. **合约"张"换算**：`ctVal`（linear 为 base 币数量）——`Qty` 统一用币数量，适配器负责换算（同 Gate `quanto_multiplier`）。
6. `expTime` 请求级超时 + `50004` 超时错误 = OKX 版的"Unknown 状态对账"路径（03 §6），设计已覆盖。
7. 服务器 30s 无数据主动断连（close 4004）+ 60s 空推送心跳——`so-recorder` 的静默断流检测（06 §3.2 A5）阈值要与此对齐。
8. 8443 端口将退役：连接 URL 固定用 443（`wss://ws.okx.com/ws/v5/...`，不带 `:8443`）。
9. `funding-rate` 的**实际结算值是 `settFundingRate`**（`settState: settled` 后），回测资金费结算（03 §8.2）用它，不是预测值 `fundingRate`。
