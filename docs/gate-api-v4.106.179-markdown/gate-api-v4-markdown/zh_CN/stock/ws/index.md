---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate Stock WebSocket v1.0.0

Gate 提供 TradFi 现货（美股 / 港股 / 韩股）WebSocket 推送接口。行情、盘口、K 线等数据变更时，会第一时间推送到已订阅的客户端。

## 服务器地址

- `wss://fx-ws.gateio.ws/v4/ws/stock`

## 支持市场

服务同时支持 **美股**（`exchange=us`）、**港股**（`exchange=hk`）与 **韩股**（`exchange=kr`）。订阅参数不变，客户端可通过推送字段 `exchange` 区分市场。

</Block>

<Block>

## 变更日志

**v1.0.0**

2026-06-30

- 初次对外发布 Stock WebSocket 文档
- 支持频道：`stock.tickers`、`stock.orderbooks`、`stock.kline`、`stock.orders`、`stock.position`
- 支持多市场（us / hk / kr）及盘前 / 盘后行情

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用频道（例如行情、订单簿等）都支持 3 种不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据推送。

2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据推送。

3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）

### 请求参数

每个请求都遵循通用格式，其中包含 `id`、`time`、`channel`、`event` 和 `payload`。

| 名称        | 类型    | 必选 | 描述                                      |
|-----------| ------- |----|-----------------------------------------|
| `id`      | Integer | 否  | 请求 ID，用于追踪请求和响应，对应的响应中会携带相同的 ID           |
| `time`    | Integer | 否  | 请求时间                                    |
| `channel` | String  | 是  | 请求 subscribe/unsubscribe 频道             |
| `auth`    | Object  | 否  | 请求身份验证信息，请参阅身份验证部分了解详细信息                |
| `event`   | String  | 是  | 请求 event (subscribe/unsubscribe/update) |
| `payload` | Any   | 是  | 请求详细参数                                  |

### 推送参数

与请求类似，推送参数遵循以下通用格式，其中包含：`time`、`channel`、`event`、`error` 和 `result`。

| 名称      | 类型    | 必选 | 描述                      |
| --------- | ------- | ---- | ------------------------- |
| `time`    | Integer | 是   | 推送时间                  |
| `channel` | String  | 是   | 推送频道                  |
| `event`   | String  | 是   | 推送频道事件 (update/all) |
| `error`   | Object  | 否   | 推送错误                  |
| `result`  | Any     | 是   | 返回来自服务端的新数据通知或对客户端请求的响应。如果有错误返回则 `error` 不为空，没有错误则此字段为空。 |
| `conn_id` | String  | 否   | 当前连接 ID               |

注意：如果它是服务端发起的数据更新通知，那么 `result` 的类型是基于 channel 的，不同 channel 的 `result` 类型有所不同。

但如果是对客户端订阅请求的响应，那么 `result` 为固定的 `{"status": "success"}`。
验证订阅请求是否成功，您只需要检查 `error` 字段是否为空即可，不需要再解析 `result` 字段。

为了简单起见，下面的频道（channel）描述只给出对应频道的 payload 格式。

### 错误

如果出现错误，您会收到 error 字段，其中包含错误代码和错误的类型。

| Code | Message |
|------|---------|
| 40000 | Bad Request |
| 40100 | Unauthorized |
| 40101 | Authentication failed |
| 40201 | Invalid channel name |
| 40202 | Invalid event type |
| 40205 | Invalid payload |
| 42901 | Rate limit exceeded |
| 50000 | Internal Server Error |

</Block>

<Block>

## 鉴权

::: warning
注意: 您使用的 GateAPIv4 密钥对必须至少启用读取权限。如果启用了密钥的白名单，则您的出站 IP 地址必须在密钥的 IP 白名单中。
:::

<Example>

```python
# example WebSocket signature calculation implementation in Python
import hmac, hashlib, json, time


def gen_sign(channel, event, timestamp):
    # GateAPIv4 key pair
    api_key = 'YOUR_API_KEY'
    api_secret = 'YOUR_API_SECRET'

    s = 'channel=%s&event=%s&time=%d' % (channel, event, timestamp)
    sign = hmac.new(api_secret.encode('utf-8'), s.encode('utf-8'), hashlib.sha512).hexdigest()
    return {'method': 'api_key', 'KEY': api_key, 'SIGN': sign}


request = {
    'id': int(time.time() * 1e6),
    'time': int(time.time()),
    'channel': 'stock.orders',
    'event': 'subscribe',
    'payload': ['!all']
}
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
print(json.dumps(request))
```

</Example>

如果频道是私有的，例如 `stock.orders`、`stock.position`，客户端请求需要携带身份验证信息。

身份验证通过请求正文中的 `auth` 字段发送，格式如下:

<a id="schema_auth"></a>

| 名称     | 类型   | 描述                |
| -------- | ------ | ------------------- |
| `method` | String | 验证方式: `api_key` |
| `KEY`    | String | apiKey 的值（`api_key` 方式） |
| `SIGN`   | String | 签名结果            |

WebSocket 认证使用与 Gate APIv4 API 相同的签名计算方法，即 `HexEncode(HMAC_SHA512(secret, signature_string))`，但有以下区别:

1. 签名字符串拼接方式：`channel=<channel>&event=<event>&time=<time>`，其中 `<channel>`、`<event>`、`<time>` 是对应的请求信息
2. 身份验证信息在请求正文中的 `auth` 字段中发送。

您可以登录 Gate 控制台获取 Gate APIv4 的 api_key 和 secret。

</Block>

<Block>

## Websocket连接管理

- Gate websocket 使用协议层 ping/pong 消息。服务器会发起 ping 操作。如果客户端没有回复，客户端将被断开。
  服务器会每 20s 发送一次协议的 ping 消息，如果 60s 没有收到协议的 pong 消息，则会断开连接。
  [protocol layer ping/pong](https://tools.ietf.org/html/rfc6455)
- App 客户端建连时建议同时携带 Header `version` 与 `X-Gate-Device-Type`（详见服务端实现说明）
- 断线后需重新订阅所有频道；`conn_id` 会变化

</Block>

<Block>

## Ping/Pong

`stock.ping`

Ping/Pong 检查服务器/客户端连接。

应用层保活请使用频道 `stock.ping` 发起 `subscribe` 请求；成功响应中 `channel` 会改写为 `stock.pong`。

<Example>

例子：

```python
import time
import json
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
ws.send(json.dumps({
    "id": 0,
    "time": int(time.time()),
    "channel": "stock.ping",
    "event": "subscribe",
    "payload": None
}))
print(ws.recv())
```

操作返回 JSON 结构如下：

```json
{
   "id": 0,
   "time": 1768361398154,
   "channel": "stock.pong",
   "conn_id": "9b065ce1-6c3c-4174-9092-5cc799e5c089",
   "event": "subscribe"
}
```

</Example>

</Block>

<Block>

# 行情频道

`stock.tickers`

`stock.tickers` 推送美股 / 港股 / 韩股现货行情快照与成交增量更新。

**推送类型**: `incremental`

**更新频率**: `200ms`（可配置）

</Block>

<Block>

## 客户端订阅

<Example>

例子：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "stock.tickers",
    "event": "subscribe",
    "payload": {
       "markets": ["AAPL", "00700"]
    }
}))
print(ws.recv())
```

</Example>

格式:

| 名称          | 类型       | 必选 | 描述   |
|-------------|----------| ---- |------|
| `payload`   | `object` | 是   | 请求参数 |
| »»`markets` | `array`  | 是   | 标的列表 |

退订时在 `markets` 中包含 `unsub_all` 可清除本连接在该频道的全部订阅。

::: tip
不需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
   "time": 1710000001111,
   "channel": "stock.tickers",
   "event": "update",
   "result": [
      {
         "symbol": "AAPL",
         "symbol_name": "AAPL",
         "highest_price": "271.03",
         "lowest_price": "267.05",
         "last_price": "237.9084",
         "today_open_price": "267.575",
         "last_today_close_price": "270.15",
         "prev_close_price": "270.15",
         "status": "open",
         "status_code": 3,
         "open_cal": 1,
         "market_cal": 1,
         "change": "-11.93",
         "price_change_rate": "-11.93",
         "change_amount": "-32.2416",
         "high_24h": "272.50",
         "low_24h": "266.80",
         "fx_usdt": "237.9084",
         "exchange": "us",
         "quote_currency": "USD",
         "quote_currency_precision": 6,
         "precision": 6
      }
   ]
}
```

</Example>

Result format:

| 名称               | 类型             | 描述                     |
| ------------------ |----------------|------------------------|
| `result`           | array          | Ticker 数组           |
| »» `symbol`    | string         | 交易品种代码                  |
| »» `symbol_name`    | string         | 品种展示名称                 |
| »» `last_price`   | string         | 最新价（常规交易时段）                 |
| »» `today_open_price`    | string         | 今日开盘价                 |
| »» `last_today_close_price`     | string | 昨日收盘价                 |
| »» `prev_close_price`    | string         | 上一交易日盘中最新价；取不到为 `"0"`                 |
| »» `highest_price` / `lowest_price`     | string | 今日最高 / 最低价                 |
| »» `high_24h` / `low_24h`     | string | 全交易时段（含盘前/盘后）最高 / 最低价                 |
| »» `change` / `price_change_rate`     | string | 涨跌幅，已乘以 100（如 `-11.93` 表示 -11.93%）                 |
| »» `change_utc0` / `change_utc8`     | string | 涨跌幅，UTC0 / UTC+8 口径，与 `change` 同值                 |
| »» `change_amount`     | string | 涨跌额                 |
| »» `status`    | string         | 交易开关：`open` / `closed`                 |
| »» `status_code`    | int         | 市场时段：盘前 `1`、盘后 `2`、盘中 `3`、休市 `4`；内部撮合 `gt_lp` 美股为 `5`、港股/韩股为 `6`。判断盘前/盘中/盘后请依赖本字段，勿仅看 `status`                 |
| »» `open_cal`    | int         | 是否计算盘中今日盈亏：`status_code=3` 时为 `1`，否则为 `0`                 |
| »» `market_cal`    | int         | 是否计算盘前/盘后/盘中今日盈亏：`status_code` 为 `1`、`2`、`3` 时为 `1`，否则为 `0`                 |
| »» `pre_market_last_price` / `pre_market_change_amount` / `pre_market_change`    | string         | 盘前最新价及涨跌（有数据时推送）                 |
| »» `after_market_last_price` / `after_market_change_amount` / `after_market_change`    | string         | 盘后最新价及涨跌（有数据时推送）                 |
| »» `price_limit_status`    | string         | 涨跌停状态：`limit_up` 涨停、`limit_down` 跌停；未触及涨跌停为空字符串 `""`                 |
| »» `total_market_cap` / `price_earn_ratio`    | string         | 总市值 / 市盈率                 |
| »» `fx_rate`    | string         | 汇率；美股固定 `"1"`，港股/韩股为对应 USD 汇率（6 位小数）                 |
| »» `fx_usdt`    | string         | 折合 USDT 价格                 |
| »» `exchange`    | string         | 市场类型：`us` / `hk` / `kr`                 |
| »» `quote_currency`    | string         | 报价货币，如 `USD`、`HKD`、`KRW`                 |
| »» `quote_currency_precision`    | int         | 报价货币精度                 |
| »» `precision`    | int         | 展示用小数位精度                 |


</Block>

<Block>

# K 线频道

`stock.kline`

提供 K 线信息推送。

**推送类型**: `incremental`

**更新频率**: `200ms`（可配置）

</Block>

<Block>

## 客户端订阅

<Example>

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "stock.kline",
    "event": "subscribe",
    "payload": ["1m", "AAPL", "false"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称           | 类型            | 必选 | 描述                         |
|--------------| --------------- | ---- |----------------------------|
| `payload`    | `Array[String]` | 是   | 订阅参数：`[周期, 标的, 是否包含盘前盘后K线]` |
| » `interval` | String          | 是   | K 线周期                   |
| » `symbol`   | String          | 是   | 交易品种名称                     |
| » 第三参   | String          | 否   | `"true"` / `"false"`，缺省 `"false"`；仅美股有效                     |

#### Enumerated Values

| 属性     | 值   |
| -------- |-----|
| interval | 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M, 30d 等 |

::: tip
不需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
   "time": 1758988800123,
   "channel": "stock.kline",
   "event": "update",
   "result": {
      "t": 1758988800,
      "n": "1m_AAPL",
      "v": "1234567",
      "c": "237.50",
      "h": "238.20",
      "l": "236.80",
      "o": "237.00",
      "a": "293456789.12",
      "w": false,
      "m": "earlyHours",
      "exchange": "us"
   }
}
```

</Example>

数据结构:

| 名称       | 类型      | 描述                              |
|----------|---------|---------------------------------|
| `result` | Object   | K 线数据                         |
| `t`      | Integer | K 线时间（窗口起点）                 |
| `o` / `c` / `h` / `l`      | String  | 开 / 收 / 高 / 低                             |
| `v`      | String | 成交量                            |
| `a`      | String  | 成交额                              |
| `n`      | String  | 订阅名称，格式为 `<interval>_<symbol>` |
| `w`      | bool    | 窗口是否已关闭             |
| `m`      | String    | 可选。盘前 `earlyHours`、盘后 `afterHours`、盘中 `open`、收盘 `closed`             |
| `exchange`      | String    | 市场类型：`us` / `hk` / `kr`             |

</Block>

<Block>

# 盘口频道

`stock.orderbooks`

推送买卖盘深度；支持外部报价与内部 ME 报价混合排序。

**推送类型**: `continuous`

**更新频率**: `200ms`（可配置）

</Block>

<Block>

## 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "stock.orderbooks",
    "event": "subscribe",
    "payload": ["AAPL"]
}))
print(ws.recv())
```

</Example>

数据结构:

| 名称        | 类型      | 必选 | 描述                         |
|-----------|---------| ---- | ---------------------------- |
| `payload`   | `array` | 是   | 标的列表 |


::: tip
不需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
  "time": 1710000001000,
  "channel": "stock.orderbooks",
  "event": "update",
  "result": {
    "t": 1778385059478,
    "ut": 1778385059478,
    "symbol": "TSLA",
    "mode": 1,
    "bids": ["427.98", "427.97"],
    "asks": ["428.01", "428.02"],
    "exchange": "us"
  }
}
```

</Example>

推送参数:

| 名称     | 类型      | 描述                     |
| -------- |---------|------------------------|
| `result` | Object   | 盘口对象                     |
| »`t` / `ut`     | Integer | 时间戳（毫秒） |
| »`symbol`     | String  | 交易品种代码                 |
| »`bids` / `asks`     | Array  | 买盘 / 卖盘价格字符串数组（买盘降序、卖盘升序）                 |
| »`mode`    | int  | 报价模式                |
| »`exchange`    | String  | 市场类型：`us` / `hk` / `kr`                |

</Block>

<Block>

# 订单频道

`stock.orders`

提供用户订单更新推送。

**推送类型**: `continuous`

**更新频率**: `real-time`

</Block>

<Block>

## 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
request = {
    "time": int(time.time()),
    "channel": "stock.orders",
    "event": "subscribe",
    "payload": ["!all"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

`payload` 传 `["!all"]` 表示订阅全部订单。

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
  "time": 1778251077642,
  "channel": "stock.orders",
  "event": "update",
  "result": {
    "id": "72057594037927938",
    "client_order_id": "123",
    "create_time": 1778251022557,
    "update_time": 1778251022557,
    "user_id": 1,
    "symbol": "AAPL",
    "side": 1,
    "type": 1,
    "price": "150.00",
    "qty": "1",
    "filled_qty": "0.4",
    "filled_value": "60.00",
    "status": 4,
    "finish_as": 7,
    "exchange": "us",
    "precision": 6,
    "volume_precision": 0
  }
}
```

</Example>

推送参数:

| 名称               | 类型            | 描述                                         |
|------------------|---------------|--------------------------------------------|
| `result`         | Object | 订单对象                                    |
| » `id`       | string        | 订单 ID                                    |
| » `symbol`         | string        | 股票代码                             |
| » `side`           | int           | 订单方向（整型枚举）                         |
| » `type`             | int    | 订单类型（整型枚举）                                               |
| » `price` / `qty`          | string        | 委托价 / 委托量                                       |
| » `filled_qty` / `filled_value`    | string        | 已成交量 / 已成交额                                      |
| » `status`           | int    | 订单状态 **OrderStatus**（整型枚举）          |
| » `finish_as`        | int    | 终态原因 **OrderFinishAs**（整型枚举）                       |
| » `exchange`         | string           | 市场类型：`us` / `hk` / `kr`              |
| » `precision` / `volume_precision`        | int           | 价格 / 数量小数位（服务端注入）                        |

**OrderStatus 主要取值**:

| 数值  | 说明    |
| --- | ----- |
| `1` | 新订单   |
| `2` | 部分成交  |
| `3` | 全部成交  |
| `4` | 撤单成功  |
| `5` | 下单被拒绝 |
| `6` | 撤单被拒绝 |

</Block>

<Block>

# 用户仓位频道

`stock.position`

提供用户持仓更新推送。

**推送类型**: `continuous`

**更新频率**: `real-time`

</Block>

<Block>

## 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/stock")
request = {
    "time": int(time.time()),
    "channel": "stock.position",
    "event": "subscribe",
    "payload": ["!all"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
  "time": 1778251077642,
  "channel": "stock.position",
  "event": "update",
  "result": {
    "user_id": 1,
    "symbol": "AAPL",
    "type": 2,
    "change": "100",
    "qty": "500",
    "sell_qty": "0",
    "sell_value": "0",
    "buy_qty": "100",
    "buy_value": "15000",
    "yesterday_qty": "400",
    "avg_entry_price": "148.50",
    "precision": 6,
    "volume_precision": 0,
    "exchange": "us",
    "quote_currency": "USD",
    "fx_rate": "1"
  }
}
```

</Example>


推送参数:

| 名称            | 类型              | 描述                            |
|---------------|-----------------|-------------------------------|
| `result`      | Object | 仓位对象                       |
| »`user_id` | uint32           | 用户 ID                       |
| »`symbol` | string          | 股票代码                |
| »`type` | int             | 事件类型： `1` 卖出、`2` 买入 |
| »`change` / `qty` | string          | 变更数量 / 总持仓量                          |
| »`sell_qty` / `buy_qty` 等 | string          | 当日卖出 / 买入量及金额                  |
| »`yesterday_qty`   | string | 昨日收盘量                  |
| »`avg_entry_price` | string | 持仓均价            |
| »`exchange`         | string | 市场类型：`us` / `hk` / `kr` |
| »`quote_currency`   | string | 报价货币 |
| »`fx_rate`          | string | 汇率；美股固定 `"1"`，港股 / 韩股为对应 USD 汇率（6 位小数） |
| »`precision` / `volume_precision`        | int    | 价格 / 数量小数位 |

</Block>
