# Gate 接入参考（API v4）

> 整理自官方文档 **v4.106.179**（2026-11-03 发布；本地快照 `docs/gate-api-v4.106.179-markdown/`）。
> 本文是工程速查，字段语义以官方文档为准；写代码前与本地快照核对，官网为准。
> 范围：M1 行情接入全覆盖；交易部分（§6）只列概要，**M6 写交易前必须补齐并重新核对**。

## 1. 服务地址与数据中心

| 项 | 值 |
|---|---|
| 数据中心 | **AWS 东京 ap-northeast-1**（官方文档"数据中心"一节）。VPS 选址候选以东京为主，香港/新加坡对照 |
| REST 实盘 | `https://api.gateio.ws/api/v4` |
| REST 模拟（testnet） | `https://api-testnet.gateapi.io/api/v4` |
| 合约 REST 备选 | `https://fx-api.gateio.ws/api/v4`（只适用合约） |
| 现货 WS 实盘 | `wss://api.gateio.ws/ws/v4/` |
| 现货 WS 模拟 | `wss://ws-testnet.gate.com/v4/ws/spot` |
| 合约 WS 实盘（USDT 结算） | `wss://fx-ws.gateio.ws/v4/ws/usdt` |
| 合约 WS 模拟 | `wss://ws-testnet.gate.com/v4/ws/futures/usdt` |
| SBE 二进制行情 | 在 WS 地址后加 `/sbe`；官方建议使用（更快、带宽更小），M1 先用 JSON，SBE 列为后续优化 |
| 域名注意 | `gate.io` 已 301 到 **`gate.com`**；`*.gateio.io` 域名已废弃但文档中 WS/REST 仍在用 `gateio.ws`——以本文表格为准 |

## 2. WebSocket 通用规则（现货与合约相同）

### 2.1 消息格式

客户端请求（订阅/退订共用同一 payload 格式）：

```json
{
  "time": 1611541000,          // 秒级；与服务器时差 ≤ 60s
  "id": 123456789,             // 可选请求 ID，原样返回
  "channel": "futures.trades",
  "event": "subscribe",        // subscribe | unsubscribe
  "payload": ["BTC_USDT"],
  "auth": { "method": "api_key", "KEY": "…", "SIGN": "…" }   // 仅私有频道
}
```

服务端响应（订阅确认与数据推送同构，用 `event` 区分）：

```json
{
  "time": 1611541000,
  "time_ms": 1611541000123,    // 2022-11 起全部频道均带毫秒时间戳
  "channel": "futures.trades",
  "event": "update",           // update = 数据推送；subscribe/unsubscribe = 确认
  "error": null,               // 非空即订阅失败；订阅成功只看 error 是否为 null
  "result": …
}
```

错误码（`error.code`）：`1` 请求体格式错、`2` 参数无效、`3` 服务端异常、`4` 鉴权失败。

### 2.2 私有频道鉴权

- 签名串：`channel=<channel>&event=<event>&time=<time>`
- 签名：`HexEncode(HMAC_SHA512(secret, 签名串))`（与 REST 相同算法，不同拼接）
- 放在请求体 `auth` 字段，`method = "api_key"`。
- 需要至少对应产品的**读**权限；开了 IP 白名单则本机出口 IP 必须在白名单内。

### 2.3 心跳与断线

- 协议层 ping/pong 由服务器发起（客户端库通常自动处理）。
- 应用层 `spot.ping` / `futures.ping`：客户端主动探测，收到 `*.pong` 即存活；服务器收到 ping 会重置超时计时器。
- **服务升级通知**：`event=update`、`channel=spot.system|futures.system`、`result.type="upgrade"` → 立即主动重连。
- 连接数限制：**每 IP ≤ 300 条**（REST+WS 总说明）；`so-recorder` 双连接冗余在配额内。

### 2.4 关键请求头

| 头 | 说明 |
|---|---|
| `X-Gate-Size-Decimal: 1` | **必须带**。2026-03（v4.106.0）起合约所有 size 字段改为十进制字符串；不带此头时 size 是**整型张数**（小数被向下取整），WS 推送里 `s`/`size`/`left` 的类型都不同。适配器固定携带 |
| `X-In-Time` / `X-Out-Time`（REST 响应头） | 网关入/出站微秒时间戳，延迟测量用 |
| `X-Gate-RateLimit-Requests-Remain` / `-Limit` / `-Reset-Timestamp` | REST 限流余量（WS 下单响应的 `spot.order_place` 也返回同名字段） |

## 3. 永续合约（USDT 结算）行情频道

M1 优先接入。频道名前缀 `futures.`。

### 3.1 `futures.trades` 公共成交（实时）

```json
{ "result": [{
    "id": 27753479,               // 交易 ID，逐笔去重用
    "create_time": 1545136464,    // 秒
    "create_time_ms": 1545136464123,
    "price": "96.4",
    "size": "-108.5",             // 十进制字符串；正=买方主动（aggressor 是买），负=卖方主动
    "contract": "BTC_USDT",
    "is_internal": true           // 可选；保险基金/ADL 接管强平的内部成交，不入 K 线，回测须排除或单独标记
}]}
```

注意：`size` 符号语义与很多交易所相反方向的记录法（正=主动买）；`is_internal=true` 的成交价格可能偏离盘口，Raw Journal 原样存，清洗规则 C5 类标记。

### 3.2 `futures.order_book_update` 深度增量（20ms / 100ms）

订阅：`payload: ["BTC_USDT", "100ms", "100"]`（合约、level、间隔；**level 应与 REST 快照 limit 一致**）。

```json
{ "result": {
    "t": 1615366381417,           // 订单簿生成时间（毫秒）
    "s": "BTC_USDT",
    "U": 2517661101,              // 本次更新起始 orderbook 更新 ID
    "u": 2517661113,              // 本次更新结束 ID
    "full": true,                 // 可选字段；true=全量替换，缺省=增量
    "b": [ {"p": "54672.1", "s": "0"}, {"p": "54664.5", "s": "58794.5"} ],
    "a": [ {"p": "54743.6", "s": "0"} ],
    "l": "100"                    // 深度层级
}}
```

**本地订单簿维护规则**（官方六步流程，与 03 §5 的 Gate U/u 设计对应）：

1. REST 拉快照：`GET /api/v4/futures/usdt/order_book?contract=BTC_USDT&limit=100&with_id=true`，记 `id` 为 `baseID`；
2. 找到第一个满足 `U ≤ baseID+1 ≤ u` 的缓存通知，从它开始应用；`s` 为**绝对数量**（覆盖而非累加），`s="0"` 删档；
3. 丢弃所有 `u < baseID+1` 的通知；若 `baseID+1 < 首个通知的 U` → 快照过旧，重拉；
4. 后续出现 `U > 本地id+1` → **缺口，重建**；
5. 推送里 `b`/`a` 为空时也必须更新本地 `id` 与 `timestamp`；
6. `size` 字段带 `X-Gate-Size-Decimal: 1` 时是字符串，不带是整数——适配器解析要兼容声明中的两种类型。

**实测（2026-11-10 录制验证）**：

- WS 握手与 REST 请求都必须带 `X-Gate-Size-Decimal: 1`，否则**所有** size 字段（OBU 档位 `s`、book_ticker `B`/`A`、trades `size`、REST 快照档位 `s`）都是整型张数且小数被服务端截断，无法恢复；
- 不存在独立的 `futures.mark_price` 频道（官方频道表无此项）；标记价/指数价/资金费率随 `futures.tickers` 推送（§3.3）。

### 3.3 其他行情频道

| 频道 | 速度 | 说明 |
|---|---|---|
| `futures.order_book` | 100ms/1000ms | 全量有限档；官方**不推荐**（流量大、时效差于增量） |
| `futures.book_ticker` | 实时 | BBO：`t`(ms)、`u`(更新ID)、`b/B/a/A`（价/量） |
| `futures.tickers` | 1s | 含 `funding_rate`、`funding_rate_indicative`（下期预估）、`mark_price`、`index_price` |
| `futures.candlesticks` | — | K 线；interval 见现货表（§4.3），`w=true` 表示窗口关闭（可能缺条，不影响完整性） |
| `futures.book_ticker_v2` / `spot.obu` 等 v2 | — | 深度 V2：`payload ["ob.BTC_USDT.400"]`，全量推送置 `u`，增量要求 `U == 本地id+1` 否则重订阅；同一流同连接只能订阅一次。**M1 不用，列作候选** |

### 3.4 私有频道（M6 用，此处仅列名）

`futures.orders`（订单全生命周期推送，字段同 REST `FuturesOrder`；`update_id` 用于回报排序）、`futures.usertrades`、`futures.positions`（payload 为 `[user, contract]`，user 已废弃仅占位，可用 `!all` 订阅全部）、`futures.balances`、`futures.liquidates`（强平）、`futures.auto_deleverages`（ADL）、`futures.position_closes`、`futures.risk_limit_changed`。

### 3.5 合约静态信息（`GET /futures/{settle}/contracts`，`VenueCapabilities` 与品种元数据来源）

关键字段（均已从 v4.106.179 文档核对）：

| 字段 | 含义 | 用途 |
|---|---|---|
| `name` | 合约名，如 `BTC_USDT`（下划线分隔，**不是** OKX 的连字符） | InstrumentId 映射 |
| `type` | `inverse` 反向 / `direct` 正向 | 品种分类 |
| `quanto_multiplier` | 一张合约的面值（标的币数量） | **张数 ↔ 币数换算**；Gate 合约以"张"计价 |
| `order_price_round` | 委托价最小单位（tick） | `Price::round_to_step` |
| `order_size_min` / `order_size_max` | 张数上下限（字符串） | 风控 RK-L1 |
| `order_price_deviate` | 下单价相对标记价的最大偏移比率 | 价格带风控 |
| `leverage_min` / `leverage_max` | 杠杆范围 | 风控 |
| `maintenance_rate` | 第一档维持保证金率 | 强平价计算 |
| `maker_fee_rate` / `taker_fee_rate` | 费率；**maker 为负代表返佣** | 成本模型 |
| `funding_rate` / `funding_interval` / `funding_next_apply` | 当前资金费率（字符串小数）/ 应用间隔秒 / 下次应用时间 | S2 策略 |
| `funding_rate_limit` | 资金费率上限 | 极端行情建模 |
| `mark_price` / `index_price` / `last_price` | 标记/指数/最新价 | 启动比对 |
| `orders_limit` | 每用户该盘口最大挂单数 | 风控 RK-L1 `max_open_orders` |
| `status` | `prelaunch / trading / delisting / delisted / circuit_breaker` | 只交易 `trading`；`delisting` 只减仓 |
| `in_delisting` | 下架过渡期只允许 reduce-only | 交易过滤 |
| `enable_decimal` | 是否支持小数张数 | size 解析方式 |
| `market_order_slip_ratio` / `market_order_size_max` | 市价单滑点上限/最大张数（0=用 order_size_max） | 市价单路径 |
| `enable_circuit_breaker` | 标记价熔断开关 | 异常行情处理 |

## 4. 现货（次要，S2 需要时接）

频道前缀 `spot.`，消息框架与合约完全相同。URL 见 §1。

### 4.1 公共频道

| 频道 | 速度 | 说明 |
|---|---|---|
| `spot.trades` | 实时 | `side: buy/sell` 是**吃单方**方向；`amount`+`price` 十进制字符串；`stock`/`money`/`trade_mode` 为统一订单簿字段（计价侧统一时实际资产与名义市场不同） |
| `spot.order_book_update` | 20ms/100ms | 订阅 `["BTC_USDT","100ms"]`；`U`/`u` 语义同合约；`b`/`a` 是 `[price, amount]` 数组（**数组对**，合约是对象 `{p,s}`——两套解析） |
| `spot.order_book` | 100ms/1000ms | 全量；`lastUpdateId` |
| `spot.book_ticker` | 10ms | BBO |
| `spot.candlesticks` | 2s | 见 4.3 |
| `spot.tickers` | 1s | 行情概览 |

### 4.2 本地深度维护

与合约一致的六步法（REST 快照 `GET /spot/order_book?…&with_id=true`）。`spot.order_book_update` 首条可能带 `full: true` 直接全量替换。

### 4.3 K 线 interval

`10s, 30s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 7d, 30d`（REST 现货另支持 `1s`）。03 §11 的 K 线约定周期（`1s…1d`）是其子集，兼容。

## 5. REST 要点

### 5.1 认证（私有接口）

请求头：`KEY`、`Timestamp`（秒，与服务器时差 ≤ 60s）、`SIGN`。

签名串（`\n` 连接五段）：

```text
METHOD \n URL路径(含 /api/v4) \n QueryString(未编码,顺序与实际一致) \n HexEncode(SHA512(body)) \n Timestamp
```

`SIGN = HexEncode(HMAC_SHA512(secret, 签名串))`。空 body 用空字符串的 SHA512（文档给出了常量值，适配器启动时自测比对）。

### 5.2 限频（速率以文档"限频规则"节为准）

| 市场/接口 | 限制 | 维度 |
|---|---|---|
| 公共行情 | 单接口 200r/10s | IP |
| 合约下单（批量/单个/改单合计） | 100 r/s | UID |
| 合约撤单（批量/单个合计） | 200 r/s | UID |
| 合约其他 | 200r/10s | UID |
| 现货下单/改单合计 | 10 r/s | UID+市场 |
| 现货撤单 | 200 r/s | UID |
| WS 下单 | 现货 10r/s；合约 100r/s | UID |

- 限速按子账号/主账号**分别**计算。
- **成交率惩罚**（注意！）：24h 内请求 > 86,400 次且零成交 → 下一小时下单限到 10r/10s；成交率 < 1% → 20r/10s。做市类策略（挂撤频繁）必须在 RK-L1 风控里对**成交率**设限，不只是请求速率。
- `so-trader` 的 RK-L1 频率限制取上表的 80%（01 §10）。

### 5.3 错误处理

非 2xx 返回 `{"label": "…", "message": "…"}`。**程序判别用 `label`**（大写下划线稳定枚举，如 `ORDER_NOT_FOUND`、`BALANCE_NOT_ENOUGH`、`TOO_MANY_REQUESTS`——这与 E2 错误码约定同构，`GateError` 枚举直接映射 label）。完整 label 列表见本地文档"异常处理"节；常用：认证类 `INVALID_SIGNATURE / IP_FORBIDDEN / REQUEST_EXPIRED / READ_ONLY`，订单类 `ORDER_NOT_FOUND / ORDER_CLOSED / POC_FILL_IMMEDIATELY / PRICE_TOO_DEVIATED / SIZE_TOO_LARGE`，限频类 `TOO_MANY_REQUESTS`。

### 5.4 自成交防护（STP）

下单参数 `stp_act`：`cn`（默认，取消新单）/ `co` / `cb`。需要先创建 **STP 用户组** 并加入账户，未入组传 `stp_act` 会报错。ADR-0001"一账户一策略"下自成交风险低，但 M6 交易接入时应创建 STP 组并默认 `cn`（对应 03 §7 的自成交防护行）。

## 6. 交易接口概要（M6 前必须补齐细节并重新核对）

| 项 | 概要 |
|---|---|
| 下单 | `POST /futures/{settle}/orders`（`size` 十进制字符串，正=多负=空；`tif: gtc/ioc/poc/fok`；`text` 自定义 ID 以 `t-` 开头 ≤28 字节）；WS 下单 `futures.orders` 频道 `event: subscribe`+`payload`（API 调用走特殊请求格式） |
| 改单 | `POST /futures/{settle}/batch_amend_orders` 等 |
| 撤单 | `DELETE /futures/{settle}/orders/{order_id}`（order_id 可用 `text`） |
| 订单状态 | `status: open/finished`；`finish_as: filled/ioc/poc/liquidate_cancelled/stp/…`（枚举见文档） |
| 持仓模式 | `single` 单向 / `dual` 双向 / `dual_plus` 分仓（`POST /futures/{settle}/set_position_mode`） |
| Dead Man's Switch | `POST /futures/{settle}/price_orders` 为条件单；**倒计时撤单**接口存在（现货 `POST /spot/countdown_cancel_all`，合约同理）——`so-guardian` 的 DMS 方案在 M6 确认具体接口 |
| 模拟环境 | testnet 的 REST/WS 地址见 §1；**testnet API Key 与实盘不通用**，需单独申请（demo 模式配置用） |

## 7. 对设计文档的影响（实现时核对）

1. `so-book` 的 Gate 校验用 `U`/`u` 连续性 + 全量替换（`full=true`），与 03 §5 一致；快照-增量衔接按 §3.2 六步法实现。
2. 合约"张"与币的换算（`quanto_multiplier`）在 `Instrument` 元数据中体现；`Qty` 统一用币数量，适配器负责换算。
3. 两个市场两套深度格式（现货数组对、合约对象），适配器解析层分开文件（F3）。
4. `is_internal=true` 成交不入 K 线（官方行为），回测撮合也应排除——写进 so-backtest 的 SimExchange 规则。
5. 成交率惩罚限频是 Gate 特有风险，RK-L1 风控的频率限制需支持"成交率"维度（M3 风控设计时补充）。
6. SBE 二进制行情（templateId 表见本地文档）列为 M7 后的延迟优化候选；先 JSON。
