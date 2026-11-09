---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate TradFi WebSocket v1.0.0 
Gate 提供一个简单的tradfi行情推送接口，当行情变更时，会第一时间推送到所订阅的客户端。

## 服务器地址

- `wss://fx-ws.gateio.ws/v4/ws/tradfi`

</Block>

<Block>

## 变更日志

**v0.1.0**

2026-01-14

- 初次上线，支持Gate的TradFi业务相关数据内容推送

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用 频道（例如行情、订单簿等）都支持 3 种不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据推送。

2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据推送。

3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）


### 请求参数

每个请求都遵循通用格式，其中包含 `id`,`time`, `channel`, `event` 和 `payload`。

| 名称        | 类型    | 必选 | 描述                                      |
|-----------| ------- |----|-----------------------------------------|
| `id`      | Integer | 否  | 请求ID,用于追踪请求和响应，对应的响应中会携带一样的ID           |
| `time`    | Integer | 否  | 请求时间                                    |
| `channel` | String  | 是  | 请求 subscribe/unsubscribe 频道             |
| `auth`    | String  | 否  | 请求身份验证信息，请参阅身份验证部分了解详细信息                |
| `event`   | String  | 是  | 请求 event (subscribe/unsubscribe/update) |
| `payload` | Any   | 是  | 请求详细参数                                  |

### 推送参数

与请求类似，推送参数遵循以下通用格式，其中包含： `time`, `channel`, `event`, `error` 和 `result`.

| 名称      | 类型    | 必选 | 描述                      |
| --------- | ------- | ---- | ------------------------- |
| `time`    | Integer | 是   | 推送时间                  |
| `channel` | String  | 是   | 推送频道                  |
| `event`   | String  | 是   | 推送频道事件 (update/all) |
| `error`   | Object  | 是   | 推送错误                  |
| `result`  | Any     | 是   | 返回来自服务端的新数据通知 或 对客户端请求的响应。如果有错误返回则 `error` 不为空，没有错误则此字段为空。 |

注意：如果它是服务端发起的数据更新通知 那么 `result` 的类型是基于 channel 的，不同 channel 的 `result` 类型有所不同。

但如果是对客户端订阅请求的响应，那么 `result` 为固定的 `{"status": "success"}`。
验证订阅请求是否成功，您只需要检查 `error` 字段是否为空即可，不需要再解析 `result` 字段。

为了简单起见，下面的频道（channel）描述只给出对应频道的 payload 格式。

### 错误

如果出现错误，您会收到 error 字段，其中包含错误代码和错误的类型。

| Code | Message |
|------|---------|
| 40000 | Bad Request |
| 40001 | WebSocket upgrade failed |
| 40002 | WebSocket connection closed |
| 40003 | WebSocket message too large |
| 40004 | WebSocket invalid message |
| 40100 | Unauthorized |
| 40101 | Authentication failed |
| 40102 | Invalid credentials |
| 40103 | Token expired |
| 40104 | Invalid JSON format |
| 40200 | Payment Required |
| 40201 | Invalid channel name |
| 40202 | Invalid event type |
| 40203 | Invalid symbol |
| 40204 | Invalid stream key |
| 40205 | Invalid payload |
| 40206 | Insufficient balance |
| 40207 | Invalid order type |
| 40208 | Invalid order side |
| 40209 | Invalid order size |
| 40210 | Invalid order price |
| 40211 | Order size too small |
| 40212 | Order size too large |
| 40213 | Order price too low |
| 40214 | Order price too high |
| 40300 | Forbidden |
| 40301 | Authorization failed |
| 40302 | Account suspended |
| 40303 | Market is closed |
| 40304 | Invalid auth method |
| 40400 | Not Found |
| 40401 | Channel not found |
| 40402 | Stream not found |
| 40403 | Client not found |
| 40404 | Subscription not found |
| 40405 | Order not found |
| 40406 | Position not found |
| 40407 | Balance not found |
| 40408 | Market not found |
| 40500 | Method Not Allowed |
| 40600 | Not Acceptable |
| 40700 | Proxy Authentication Required |
| 40800 | Request Timeout |
| 40900 | Conflict |
| 40901 | Subscription already exists |
| 40902 | Order already exists |
| 40903 | Order already filled |
| 40904 | Order already cancelled |
| 41000 | Gone |
| 41100 | Length Required |
| 41200 | Precondition Failed |
| 41300 | Payload Too Large |
| 41400 | URI Too Long |
| 41500 | Unsupported Media Type |
| 41600 | Range Not Satisfiable |
| 41700 | Expectation Failed |
| 41800 | I'm a teapot |
| 42100 | Misdirected Request |
| 42200 | Unprocessable Entity |
| 42300 | Locked |
| 42400 | Failed Dependency |
| 42500 | Too Early |
| 42600 | Upgrade Required |
| 42800 | Precondition Required |
| 42900 | Too Many Requests |
| 42901 | Rate limit exceeded |
| 43100 | Request Header Fields Too Large |
| 45100 | Unavailable For Legal Reasons |
| 50000 | Internal Server Error |
| 50001 | Database error |
| 50002 | Redis error |
| 50003 | Kafka error |
| 50004 | External API error |
| 50005 | Configuration error |
| 50006 | Metrics error |
| 50007 | Logging error |
| 50100 | Not Implemented |
| 50200 | Bad Gateway |
| 50300 | Service Unavailable |
| 50301 | Service in maintenance mode |
| 50400 | Gateway Timeout |
| 50500 | HTTP Version Not Supported |
| 50600 | Variant Also Negotiates |
| 50700 | Insufficient Storage |
| 50800 | Loop Detected |
| 51000 | Not Extended |
| 51100 | Network Authentication Required |

</Block>

<Block>

## 鉴权

::: warning
注意: 您使用的 GateAPIv4 密钥对必须至少启用选项读取权限， 如果启用了密钥的白名单，则您的出站 IP 地址必须在密钥的 IP 白名单中。
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
    'channel': 'tradfi.orders',
    'event': 'subscribe',
    'payload': []
}
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
print(json.dumps(request))
```

</Example>

如果频道是私有的，例如，客户端请求需要携带身份验证信息。例如： `tradfi.orders` 推送用户订单更新的频道。

身份验证通过请求正文中的`auth`字段发送，格式如下:

<a id="schema_auth"></a>

| 名称     | 类型   | 描述                |
| -------- | ------ | ------------------- |
| `method` | String | 验证方式: `api_key` |
| `KEY`    | String | apiKey 的值         |
| `SIGN`   | String | 签名结果            |

WebSocket 认证使用与 Gate APIv4 API 相同的签名计算方法，即`HexEncode(HMAC_SHA512(secret, signature_string))`, 但有以下区别:

1. 签名字符串拼接方式： `channel=<channel>&event=<event>&time=<time>`, 其中 `<channel>`, `<event>`, `<time>` 是对应的请求信息
2. 身份验证信息在请求正文中的 `auth`字段中发送。

您可以登录 Gate 控制台获取 Gate APIv4 的 api_key 和 secret。

</Block>

<Block>

## Websocket连接管理

- Gate websocket 使用协议层 ping/pong 消息。服务器会发起 ping 操作。如果客户端没有回复，客户端将被断开。
  服务器会每20s发送一次协议的ping消息，如果60s没有收到协议的pong消息，则会断开连接。
  [protocol layer ping/pong](https://tools.ietf.org/html/rfc6455)
- 每个IP最多30个连接数
- 每个IP每分钟最多尝试连接30次
- 每个连接每秒钟最多请求10次

</Block>

<Block>

## Ping/Pong

`tradfi.ping`

Ping/Pong
检查服务器/客户端连接.

如果想主动检测连接状态，可以发送应用层 ping 消息，并接收 pong 消息。

<Example>

例子：

```python
import time
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
ws.send('{"time": %d, "channel": "tradfi.ping"}'% int(time.time()))
print(ws.recv())
```

操作返回 JSON 结构如下：

```json
{
   "time": 1768361398154,
   "channel": "tradfi.pong",
   "conn_id": "9b065ce1-6c3c-4174-9092-5cc799e5c089"
}
```

</Example>

</Block>

<Block>

# tradfi行情频道

`tradfi.tickers`

`tickers`是行情的高级概述。它显示最新交易价格、最佳卖出价格、最佳买入价格、指数价格等信息。

**推送类型**: `incremental`

**更新频率**: `250ms`

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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "tradfi.tickers",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": {
       "markets": ["XAUUSD"]
    }
}))
print(ws.recv())
```

</Example>

格式:

| 名称          | 类型       | 必选 | 描述   |
|-------------|----------| ---- |------|
| `payload`   | `object` | 是   | 请求参数 |
| »»`markets` | `array`  | 是   | 市场列表 |
您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的symbol不会被覆盖。

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
   "time": 1768362181817,
   "channel": "tradfi.tickers",
   "event": "update",
   "result": [
      {
         "timestamp": 1768341600,
         "symbol": "XAUUSD",
         "open_price": "4587.16",
         "last_price": "4622.95",
         "price_change_amount": "35.79",
         "price_change_rate": "0.78",
         "high": "4625.59",
         "low": "4587.07"
      }
   ]
}
```

</Example>

Result format:

| 名称               | 类型             | 描述                     |
| ------------------ |----------------|------------------------|
| `result`           | array          | Ticker array           |
| »» `timestamp`          | string         | 行情产生的时间戳（Unix 时间，单位：秒） |
| »»` symbol`    | string         | 交易品种代码                  |
| »» `open_price`    | string         | 开盘价                 |
| »» `last_price`   | string         | 最新价                 |
| »» `price_change_amount`     | string | 价格变动值（最新价 − 开盘价）                 |
| »» `price_change_rate`    | string         | 价格涨跌幅（单位：百分比）                 |
| »» `high`     | string | 当日最高价                 |
| »» `low`    | string         | 当日最低价                 |


</Block>

<Block>

# tradfi K 线频道

`tradfi.candlesticks`

提供了一种访问 K 线信息的方式。

**推送类型**: `incremental`

**更新频率**: `250ms`

</Block>

<Block>

## 客户端订阅

<Example>

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "tradfi.candlesticks",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1d", "XAUUSD"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称           | 类型            | 必选 | 描述                         |
|--------------| --------------- | ---- |----------------------------|
| `payload`    | `Array[String]` | 是   | 订阅参数。从左到右，`interval`, `symbol` |
| » `interval` | String          | 是   | K 线数据点间隔                   |
| » `symbol`   | String          | 是   | 交易品种名称                     |

#### Enumerated Values

| 属性     | 值   |
| -------- |-----|
| interval | 1m  |
| interval | 5m  |
| interval | 15m |
| interval | 30m |
| interval | 1h  |
| interval | 4h  |
| interval | 1d  |
| interval | 7d  |
| interval | 1M  |

要订阅多个合约或使用不同的间隔，只需发送多个具有不同参数的订阅请求。

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
   "time": 1768362860942,
   "channel": "tradfi.candlesticks",
   "event": "update",
   "result": [
      {
         "t": 1768341600,
         "n": "1d_XAUUSD",
         "v": "0.000000",
         "c": "4627.14",
         "h": "4627.31",
         "l": "4587.07",
         "o": "4587.16",
         "a": "0.000000",
         "w": false
      }
   ]
}
```

</Example>

数据结构:

| 名称       | 类型      | 描述                              |
|----------|---------|---------------------------------|
| `result` | Array   | K 线数据数组                         |
| `t`      | Integer | 以秒为单位的 Unix 时间戳                 |
| `o`      | String  | 开盘价                             |
| `c`      | String  | 收盘价                             |
| `h`      | String  | 最高价                             |
| `l`      | String  | 最低价                             |
| `v`      | Integer | 总成交量                            |
| `a`      | String  | 数量                              |
| `n`      | String  | 订阅的名称，格式为 `<interval>_<symbol>` |
| `w`      | bool    | windows_close, 是否关闭窗口             |

</Block>

<Block>

# 最优买卖价

`tradfi.order_book`

**推送类型**: `continuous`

**更新频率**: `250ms`

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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "tradfi.order_book",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["XAUUSD"]
}))
print(ws.recv())
```

</Example>

数据结构:

| 名称        | 类型      | 必选 | 描述                         |
|-----------|---------| ---- | ---------------------------- |
| `payload`   | `array` | 是   | 市场列表 |


您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

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
  "time": 1768372119511,
  "channel": "tradfi.order_book",
  "event": "update",
  "result": [
    {
      "time": 1768372119504,
      "symbol": "XAUUSD",
      "bid": "4633.24",
      "ask": "4633.33"
    }
  ]
}
```

</Example>

推送参数:

| 名称     | 类型      | 描述                     |
| -------- |---------|------------------------|
| `result` | Array   | 结果                     |
| »`time`     | Integer | 行情产生的时间戳（Unix 时间，单位：秒） |
| »`symbol`     | String  | 交易品种代码                 |
| »`bid`     | String  | 最佳买入价格                 |
| »`ask`    | String  | 最佳的卖出价格                |

</Block>

<Block>

# 订单频道

`tradfi.orders`

提供了一种接收用户订单更新的方式。

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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
request = {
    "time": int(time.time()),
    "channel": "tradfi.orders",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": []
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

订阅成功后会推送全部symbol的订单

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
  "time": 1768373164534,
  "channel": "tradfi.orders",
  "event": "update",
  "result": [
    {
      "order_id": "2536849",
      "gate_uid": 2124580619,
      "symbol": "NZDSEK",
      "side": 2,
      "volume": "0.01",
      "fill_volume": "0",
      "price": "5.29483",
      "price_tp": "0.00000",
      "price_sl": "0.00000",
      "finished": "0",
      "finished_as": "-",
      "time_setup": 1768373164,
      "timestamp": 1768373164491,
      "order_opt_type": 1
    }
  ]
}
```

</Example>

推送参数:

| 名称               | 类型            | 描述                                         |
|------------------|---------------|--------------------------------------------|
| `result`         | `Array[Object]` | 更新的订单列表                                    |
| » order_id       | string        | 订单唯一 ID                                    |
| » gate_uid       | int64         | Gate 用户唯一标识                                |
| » symbol         | string        | 交易品种（如 XAUUSD）                             |
| » side           | int           | 交易方向（1 = 卖出，2 = 买入）                         |
| » volume         | string        | 下单数量                                       |
| » fill_volume    | string        | 已成交数量                                      |
| » price          | string        | 委托价格                                       |
| » price_tp       | string        | 止盈价格（0 表示未设置）                              |
| » price_sl       | string        | 止损价格（0 表示未设置）                              |
| » finished       | string        | 订单是否完成（0 = 未完成，1 = 已完成）                    |
| » finished_as    | string        | 订单完成方式（如成交、撤单，`-` 表示未完成）                   |
| » time_setup     | int64        | 订单创建时间（Unix 时间，秒）                          |
| » timestamp      | int64        | 订单状态更新时间戳（Unix 时间，毫秒）                      |
| » order_opt_type | int          | 订单操作类型  1: 卖 2:买 3:平多 4:平空 5:强制平仓多 6:强制平仓空 |

</Block>

<Block>

# 用户仓位频道

`tradfi.position`

提供了一种接收用户交易的方式。

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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
request = {
    "time": int(time.time()),
    "channel": "tradfi.position",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": []
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

订阅成功后会推送全部symbol的仓位更新

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
  "time": 1768373164545,
  "channel": "tradfi.position",
  "event": "update",
  "result": [
    {
      "position_id": 2536849,
      "gate_uid": 2124580619,
      "side": 2,
      "symbol": "NZDSEK",
      "volume": "0.01",
      "price_open": "5.29483",
      "price_tp": "0.00000",
      "price_sl": "0.00000",
      "time_create": 1768373164491
    }
  ]
}
```

</Example>


推送参数:

| 名称            | 类型              | 描述                            |
|---------------|-----------------|-------------------------------|
| `result`      | `Array[Object]` | 更新的仓位列表                       |
| »`position_id` | int64           | 持仓唯一 ID                       |
| »`gate_uid` | int64           | Gate 用户唯一标识                   |
| »`side` | int             | 持仓方向（1 = 空仓 / 卖出，2 = 多仓 / 买入） |
| »`symbol` | string          | 交易品种（如 NZDSEK）                |
| »`volume` | string          | 持仓数量                          |
| »`price_open` | string          | 开仓价格                          |
| »`price_tp` | string          | 止盈价格（0 表示未设置）                 |
| »`price_sl` | string          | 止损价格（0 表示未设置）                 |
| »`time_create` | number          | 持仓创建时间（Unix 时间，毫秒）            |

</Block>

<Block>

# 用户余额频道

`tradfi.balance`

提供了一种接收用户交易的方式。

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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/tradfi")
request = {
    "time": int(time.time()),
    "channel": "tradfi.balance",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": []
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

订阅成功后会推送全部余额变更

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
  "time": 1768373369028,
  "channel": "tradfi.balance",
  "event": "update",
  "result": [
    {
      "deal_id": "84776",
      "gate_uid": 2124580619,
      "change": "-20",
      "comment": "",
      "timestamp": 1768373368990
    }
  ]
}
```

</Example>


推送参数:

| 名称            | 类型              | 描述                    |
|---------------|-----------------|-----------------------|
| `result`      | `Array[Object]` | 更新的余额信息列表             |
| deal_id | string          | 资金变动记录唯一 ID           |
| gate_uid | int64           | Gate 用户唯一标识           |
| change | string          | 余额变动金额（正数表示增加，负数表示减少） |
| comment | string          | 资金变动备注说明              |
| timestamp | int64          | 余额变动时间戳（Unix 时间，毫秒）   |

</Block>

