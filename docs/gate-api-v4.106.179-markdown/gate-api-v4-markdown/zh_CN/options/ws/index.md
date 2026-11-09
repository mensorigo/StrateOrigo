---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate 期权 WebSocket v4.0.0

Gate 提供了一个简单而强大 Websocket 频道来集成 Gate 底层选项 交易状态进入您的业务或应用程序。

我们在`Python`中有语言绑定，将来还会有更多！您可以在右侧的深色区域中查看代码示例，并且可以通过右上角的选项卡切换示例的编程语言。

## 服务器地址

以下是我们提供的期权交易服务器地址，您可以根据您的情况选择其中之一。

地址列表:

- 线上交易: `wss://op-ws.gateio.live/v4/ws`
- 模拟盘交易: `wss://op-ws-testnet.gateio.live/v4/ws`

</Block>

<Block>

## 变更日志

<Example>

```python
# !/usr/bin/env python
# coding: utf-8

import hashlib
import hmac
import json
import logging
import time

from websocket import WebSocketApp

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class GateWebSocketApp(WebSocketApp):

   def __init__(self, url, api_key, api_secret, **kwargs):
      super(GateWebSocketApp, self).__init__(url, **kwargs)
      self._api_key = api_key
      self._api_secret = api_secret

   def _send_ping(self, interval, event, payload):
      while not event.wait(interval):
         self.last_ping_tm = time.time()
         if self.sock:
            try:
               self.sock.ping(payload)
            except Exception as ex:
               logger.warning("send_ping routine terminated: {}".format(ex))
               break
            try:
               self._request("options.ping", auth_required=False)
            except Exception as e:
               raise e

   def _request(self, channel, event=None, payload=None, auth_required=True):
      current_time = int(time.time())
      data = {
         "time": current_time,
         "channel": channel,
         "event": event,
         "payload": payload,
      }
      if auth_required:
         message = 'channel=%s&event=%s&time=%d' % (channel, event, current_time)
         data['auth'] = {
            "method": "api_key",
            "KEY": self._api_key,
            "SIGN": self.get_sign(message),
         }
      data = json.dumps(data)
      logger.info('request: %s', data)
      self.send(data)

   def get_sign(self, message):
      h = hmac.new(self._api_secret.encode("utf8"), message.encode("utf8"), hashlib.sha512)
      return h.hexdigest()

   def subscribe(self, channel, payload=None, auth_required=True):
      self._request(channel, "subscribe", payload, auth_required)

   def unsubscribe(self, channel, payload=None, auth_required=True):
        self._request(channel, "unsubscribe", payload, auth_required)


def on_message(ws, message):
    # type: (GateWebSocketApp, str) -> None
    # handle message received
    logger.info("message received from server: {}".format(message))


def on_open(ws):
    # type: (GateWebSocketApp) -> None
    # subscribe to channels interested
    logger.info('websocket connected')
    ws.subscribe("options.contract_tickers", ['BTC_USDT-20211231-59800-C'], False)


if __name__ == "__main__":
    logging.basicConfig(format="%(asctime)s - %(message)s", level=logging.DEBUG)
    app = GateWebSocketApp("wss://op-ws.gateio.live/v4/ws",
                           "YOUR_API_KEY",
                           "YOUR_API_SECRET",
                           on_open=on_open,
                           on_message=on_message)
    app.run_forever(ping_interval=5)
```

</Example>

<Example>

```go
package main

import (
	"crypto/hmac"
	"crypto/sha512"
	"crypto/tls"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/url"
	"time"

	"github.com/gorilla/websocket"
)

type Msg struct {
	Time    int64    `json:"time"`
	Channel string   `json:"channel"`
	Event   string   `json:"event"`
	Payload []string `json:"payload"`
	Auth    *Auth    `json:"auth"`
}

type Auth struct {
	Method string `json:"method"`
	KEY    string `json:"KEY"`
	SIGN   string `json:"SIGN"`
}

const (
	Key    = "YOUR_API_KEY"
	Secret = "YOUR_API_SECRETY"
)

func sign(channel, event string, t int64) string {
	message := fmt.Sprintf("channel=%s&event=%s&time=%d", channel, event, t)
	h2 := hmac.New(sha512.New, []byte(Secret))
	io.WriteString(h2, message)
	return hex.EncodeToString(h2.Sum(nil))
}

func (msg *Msg) sign() {
	signStr := sign(msg.Channel, msg.Event, msg.Time)
	msg.Auth = &Auth{
		Method: "api_key",
		KEY:    Key,
		SIGN:   signStr,
	}
}

func (msg *Msg) send(c *websocket.Conn) error {
	msgByte, err := json.Marshal(msg)
	if err != nil {
		return err
	}
	return c.WriteMessage(websocket.TextMessage, msgByte)
}

func NewMsg(channel, event string, t int64, payload []string) *Msg {
	return &Msg{
		Time:    t,
		Channel: channel,
		Event:   event,
		Payload: payload,
	}
}

func main() {
	u := url.URL{Scheme: "wss", Host: "op-ws.gateio.live", Path: "/v4/ws"}
	websocket.DefaultDialer.TLSClientConfig = &tls.Config{RootCAs: nil, InsecureSkipVerify: true}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		panic(err)
	}
	c.SetPingHandler(nil)

	// read msg
	go func() {
		for {
			_, message, err := c.ReadMessage()
			if err != nil {
				c.Close()
				panic(err)
			}
			fmt.Printf("recv: %s\n", message)
		}
	}()

	t := time.Now().Unix()
	pingMsg := NewMsg("options.ping", "", t, []string{})
	err = pingMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe order book
	orderBookMsg := NewMsg("options.order_book", "subscribe", t, []string{"BTC_USDT-20211231-59800-C"})
	err = orderBookMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe positions
	positionsMsg := NewMsg("options.positions", "subscribe", t, []string{"USERID", "BTC_USDT-20211231-59800-C"})
	positionsMsg.sign()
	err = positionsMsg.send(c)
	if err != nil {
		panic(err)
	}

	select {}
}
```

</Example>

2026.05-12

- 多个频道新增按标的订阅：在 `subscribe` / `unsubscribe` 中可使用 JSON 对象作为 `payload`，通过 `underlyings` 与可选的 `contracts` 指定标的及期权合约范围；服务端将各标的展开为该标的下全部挂牌期权合约，并与显式 `contracts` 取并集去重。
- 适用频道：`options.contract_tickers`、`options.contract_candlesticks`、`options.book_ticker`、`options.trades`、`options.mark_prices`、`options.contracts`、`options.settlements`、`options.orders`、`options.usertrades`、`options.positions`、`options.liquidates`、`options.position_closes`、`options.user_settlements`。
- 单次订阅受 `underlyings` 数量与展开后合约总数上限约束；超出上限时 `error.code` 为 `2`，`message` 通常包含 `too many`。

2025.12-18

- `options.orders` 更新推送参数说明

2025.11-19

- `options.positions` 更新订阅说明

2024.11-28

- `options.usertrades` 新增字段 `fee`,`text`

2021-12-28

- 初次发版

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用 频道（例如行情、订单簿等）都支持 4 种不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据推送。

2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据推送。

3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）

4. **`all`**

   如果有新订阅的数据（所有数据）可用，服务器将向客户端发送推送。

### 请求参数

每个请求都遵循通用格式，其中包含 `time`, `channel`, `event` 和 `payload`。

| 名称      | 类型    | 必选 | 描述                                             |
| --------- | ------- | ---- | ------------------------------------------------ |
| `time`    | Integer | 是   | 请求时间                                         |
| `channel` | String  | 是   | 请求 subscribe/unsubscribe 频道                  |
| `auth`    | String  | 否   | 请求身份验证信息，请参阅身份验证部分了解详细信息 |
| `event`   | String  | 是   | 请求 event (subscribe/unsubscribe/update/all)    |
| `payload` | Array / Object | 是   | 请求详细参数；部分频道支持对象格式按标的订阅，见「按标的订阅（对象 payload）」与各频道说明 |

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

| Code | Message                   |
| ---- | ------------------------- |
| `1`  | `invalid argument struct` |
| `2`  | `invalid argument`        |
| `3`  | `service error`           |
| `4`  | `authentication fail`     |

### 按标的订阅（对象 payload）

已支持本方式的频道，其 `subscribe` / `unsubscribe` 除字符串数组外，还可使用 JSON 对象；对象中须包含频道专有字段（如有），并与下列通用字段组合使用。

| 名称          | 类型            | 必选     | 描述                                                                 |
| ------------- | --------------- | -------- | -------------------------------------------------------------------- |
| `underlyings` | `Array[String]` | 条件必选 | 期权标的名称（如 `BTC_USDT`）。须为服务端支持的标的。与 `contracts` 至少之一非空。 |
| `contracts`   | `Array[String]` | 条件必选 | 显式期权合约名称。与各 `underlying` 展开得到的合约合并去重。         |

**公共频道：** `options.contract_tickers`、`options.contract_candlesticks`、`options.book_ticker`、`options.trades`、`options.mark_prices`、`options.contracts`、`options.settlements`。

**私有频道（须鉴权）：** `options.orders`、`options.usertrades`、`options.positions`、`options.liquidates`、`options.position_closes`、`options.user_settlements`。

单次订阅的 `underlyings` 个数及展开后的合约总数存在服务端上限；超出时 `error` 不为空，`code` 为 `2`，`message` 通常包含 `too many`。

私有频道中原有的 `["<userId>", "!all"]` 等数组格式仍可用，并可与对象格式在同一连接内共存、分别维护订阅范围。

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
    'channel': 'options.orders',
    'event': 'subscribe',
    'payload': ["1001", "BTC_USDT-20211231-59800-C"]
}
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
print(json.dumps(request))
```

</Example>

如果频道是私有的，例如，客户端请求需要携带身份验证信息。例如： `options.orders`检索用户订单更新的频道。

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

您可以登录账户获取永续合约账户的 api_key 和 secret。

</Block>

<Block>

# System API

提供系统状态检查，如`ping/pong`。

</Block>

<Block>

## Ping/Pong

`options.ping`

Ping/Pong
检查服务器/客户端连接.

Gate websocket 使用协议层 ping/pong 消息。服务器会发起 ping 操作。如果客户端没有回复，客户端将被断开。
[protocol layer ping/pong](https://tools.ietf.org/html/rfc6455)
如果想主动检测连接状态，可以发送应用层 ping 消息，并接收 pong 消息。

<Example>

例子：

```python
import time
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send('{"time": %d, "channel": "options.ping"}'% int(time.time()))
print(ws.recv())
```

操作返回 JSON 结构如下：

```json
{
  "time": 1630566602,
  "channel": "options.pong",
  "event": "",
  "error": null,
  "result": null
}
```

</Example>

</Block>

<Block>

# 合约行情频道

`options.contract_tickers`

`tickers`是行情的高级概述。它显示最新交易价格、最佳卖出价格、最佳买入价格、指数价格等信息。

**推送类型**: `incremental`

**更新频率**: `1s`

</Block>

<Block>

## 客户订阅

<Example>

例子：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.contract_tickers",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211231-59800-C"]
}))
print(ws.recv())
```

</Example>

格式:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；除上文「按标的订阅」中的 `underlyings` / `contracts` 外，本频道另支持：

| 名称          | 类型   | 必选 | 描述 |
| ------------- | ------ | ---- | ---- |
| `change_from` | String | 否   | 统计口径，如 `utc0`、`24h` |

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.contract_tickers",
  "event": "subscribe",
  "payload": {
    "change_from": "utc0",
    "underlyings": ["BTC_USDT"]
  }
}
```

```json
{
  "time": 1746754200,
  "channel": "options.contract_tickers",
  "event": "subscribe",
  "payload": {
    "change_from": "24h",
    "underlyings": ["BTC_USDT"],
    "contracts": ["ETH_USDT-20260213-3000-C"]
  }
}
```

```json
{
  "time": 1746754260,
  "channel": "options.contract_tickers",
  "event": "unsubscribe",
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的合同不会被覆盖。

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
  "time": 1630576352,
  "channel": "options.contract_tickers",
  "event": "update",
  "result": {
    "name": "BTC_USDT-20211231-59800-P",
    "last_price": "11349.5",
    "mark_price": "11170.19",
    "index_price": "",
    "position_size": 993,
    "bid1_price": "10611.7",
    "bid1_size": 100,
    "ask1_price": "11728.7",
    "ask1_size": 100,
    "vega": "34.8731",
    "theta": "-72.80588",
    "rho": "-28.53331",
    "gamma": "0.00003",
    "delta": "-0.78311",
    "mark_iv": "0.86695",
    "bid_iv": "0.65481",
    "ask_iv": "0.88145",
    "leverage": "3.5541112718136"
  }
}
```

</Example>

Result format:

| 名称               | 类型           | 描述                                                    |
| ------------------ | -------------- | ------------------------------------------------------- |
| `result`           | Object         | Ticker Object                                           |
| »» `name`          | string         | 标的名称                                                |
| »»` last_price`    | string         | 最新成交价                                              |
| »» `mark_price`    | string         | 当前标记价格                                            |
| »» `index_price`   | string         | 当前指数价格                                            |
| »» `ask1_size`     | integer(int64) | 最佳卖出大小                                            |
| »» `ask1_price`    | string         | 最佳卖出价格                                            |
| »» `bid1_size`     | integer(int64) | 最佳买入大小                                            |
| »» `bid1_price`    | string         | 最佳买入价格                                            |
| »» `position_size` | integer(int64) | 当前多头头寸总规模                                      |
| »» `mark_iv`       | string         | 隐含波动率                                              |
| »» `bid_iv`        | string         | 买入隐含波动率                                          |
| »» `ask_iv`        | string         | 卖出隐含波动率                                          |
| »» `leverage`      | string         | 当前杠杆。公式: `underlying_price / mark_price * delta` |
| »» `delta`         | string         | Delta                                                   |
| »» `gamma`         | string         | Gamma                                                   |

</Block>

<Block>

# 标的资产行情通道

`options.ul_tickers`

底层计时器显示了买入交易、卖出交易以及标的资产的指数价格。

**推送类型**: `incremental`

**推送频率**: `1s`

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.ul_tickers",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

格式:

| 名称      | 类型            | 必选 | 描述      |
| --------- | --------------- | ---- | --------- |
| `payload` | `Array[String]` | 是   | 标的 列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的合同不会被覆盖。

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
  "time": 1630576352,
  "channel": "options.ul_tickers",
  "event": "update",
  "result": {
    "trade_put": 800,
    "trade_call": 41700,
    "index_price": "50695.43",
    "name": "BTC_USDT"
  }
}
```

</Example>

推送参数:

| 字段           | 类型           | 描述                                               |
| -------------- | -------------- | -------------------------------------------------- |
| `result`       | Object         | 报价对象                                           |
| »`name`        | String         | 标的物名称                                         |
| »`trade_put`   | integer(int64) | 过去 24 小时内的看跌期权交易总量（单位：合约数量） |
| »`trade_call`  | integer(int64) | 过去 24 小时内的看涨期权交易总量（单位：合约数量） |
| »`index_price` | string         | 指数价格（报价货币）                               |

</Block>

<Block>

# 公开合约交易频道

`options.trades`

该频道在 Gate 发生交易时发送交易消息。它包括交易的详细信息，如价格、数量和时间。

**推送类型**: `continuous`

**更新频率**: `real-time`

</Block>

<Block>

## 客户订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.trades",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211231-59800-C"]
}))
print(ws.recv())
```

</Example>

请求参数:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象，字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.trades",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT", "ETH_USDT"]
  }
}
```

```json
{
  "time": 1746754260,
  "channel": "options.trades",
  "event": "unsubscribe",
  "payload": {
    "underlyings": ["ETH_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。除非明确取消订阅，否则先前订阅的合约不会被覆盖。
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
  "time": 1630576356,
  "channel": "options.trades",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT-20211231-59800-C",
      "create_time": 1639144526,
      "id": 12279,
      "price": 997.8,
      "size": -100,
      "create_time_ms": 1639144526597,
      "underlying": "BTC_USDT"
    }
  ]
}
```

</Example>

请注意，公开交易频道只会通推送交易中的接收方（taker）。下方的私人用户交易频道将推送所有与用户相关的交易。

推送参数:

| 名称              | 类型   | 描述                                |
| ----------------- | ------ | ----------------------------------- |
| `result`          | Array  | 交易数组                            |
| »`contract`       | String | 期权合约名称                        |
| »`size`           | int    | T 交易量                            |
| »`id`             | int    | 交易 ID                             |
| »`create_time`    | int    | 交易时间（交易发生的时间）          |
| »`create_time_ms` | int    | 交易时间，毫秒精确到小数点后 3 位。 |
| »`price`          | Float  | 交易价格                            |
| »`underlying`     | String | 标的物名称                          |

</Block>

<Block>

# 公开基础交易频道

`options.ul_trades`

该频道在 Gate 上每次发生交易时都会发送交易消息，其中包括交易的详细信息，如价格、数量和时间。这些消息涵盖了所有合约交易数据，而不仅仅是基础交易数据。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.ul_trades",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称      | 类型            | 必选 | 描述      |
| --------- | --------------- | ---- | --------- |
| `payload` | `Array[String]` | 是   | 标的 列表 |

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

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
  "time": 1630576356,
  "channel": "options.ul_trades",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT-20211231-59800-C",
      "create_time": 1639144526,
      "id": 12279,
      "price": 997.8,
      "size": -100,
      "create_time_ms": 1639144526597,
      "underlying": "BTC_USDT",
      "is_call": true
    }
  ]
}
```

</Example>

推送参数:

| 名称               | 类型   | 描述                            |
| ------------------ | ------ | ------------------------------- |
| `result`           | Array  | 交易数组                        |
| »`contract`        | String | 期权合约名称                    |
| »`size`            | int    | 交易量                          |
| »`id`              | int    | 交易 ID                         |
| »`create_time`     | int    | 交易时间                        |
| » `create_time_ms` | int    | 交易时间，毫秒精确到 3 位小数。 |
| »`price`           | Float  | 交易价格                        |
| »`underlying`      | String | 标的物名称                      |
| »`is_call`         | Bool   | 是: CALL，否:PUT                |

</Block>

<Block>

# 基础资产价格频道

`options.ul_price`

该频道发送基础资产价格更新消息。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.ul_price",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称      | 类型            | 必选 | 描述      |
| --------- | --------------- | ---- | --------- |
| `payload` | `Array[String]` | 是   | 标的 列表 |

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

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
  "time": 1630576356,
  "channel": "options.ul_price",
  "event": "update",
  "result": {
    "underlying": "BTC_USDT",
    "price": 49653.24,
    "time": 1639143988,
    "time_ms": 1639143988931
  }
}
```

</Example>

推送参数:

| 名称          | 类型   | 描述                                   |
| ------------- | ------ | -------------------------------------- |
| `result`      | Object | 价格更新结果                           |
| »`underlying` | String | 期权标的物名称                         |
| »`price`      | Float  | 标的物价格                             |
| »`time`       | int    | 更新时间（来自 Gate 引擎的时间）       |
| »`time_ms`    | int    | 更新时间（毫秒，来自 Gate 引擎的时间） |

</Block>

<Block>

# 标记价格频道

`options.mark_prices`

该频道发送标记价格更新消息。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.mark_prices",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211231-59800-P"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象，字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.mark_prices",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

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
  "time": 1630576356,
  "channel": "options.mark_prices",
  "event": "update",
  "result": {
    "contract": "BTC_USDT-20211231-59800-P",
    "price": 11021.27,
    "time": 1639143401,
    "time_ms": 1639143401676
  }
}
```

</Example>

推送参数:

| 名称        | 类型   | 描述             |
| ----------- | ------ | ---------------- |
| `result`    | Object | 标记价格对象     |
| »`contract` | String | 期权合约名称     |
| »`price`    | Float  | 标的物价格       |
| »`time`     | int    | 更新时间         |
| »`time_ms`  | int    | 更新时间（毫秒） |

</Block>

<Block>

# 结算频道

`options.settlements`

该频道发送合约结算更新消息。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.settlements",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211130-55000-P"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象，字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.settlements",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

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
  "time": 1630576356,
  "channel": "options.settlements",
  "event": "update",
  "result": {
    "contract": "BTC_USDT-20211130-55000-P",
    "orderbook_id": 2,
    "position_size": 1,
    "profit": 0.5,
    "settle_price": 70000,
    "strike_price": 65000,
    "tag": "WEEK",
    "trade_id": 1,
    "trade_size": 1,
    "underlying": "BTC_USDT",
    "time": 1639051907,
    "time_ms": 1639051907000
  }
}
```

</Example>

推送参数:

| 名称            | 类型   | 描述                           |
| --------------- | ------ | ------------------------------ |
| `result`        | Object | 结算对象                       |
| » time          | Int    | 配置的最后更改时间（结算时间） |
| » time_ms       | Int    | 配置的最后更改时间（毫秒）     |
| » contract      | string | 合约名称                       |
| » profit        | string | 每个大小的结算利润             |
| » settle_price  | string | 结算价格                       |
| » strike_price  | Int    | 行权价格                       |
| » orderbook_id  | Int    | 当前订单簿 ID                  |
| » position_size | Int    | 当前总多头持仓规模             |
| » tag           | String | 结算标签                       |
| » trade_id      | int    | 当前交易 ID                    |
| » trade_size    | Int    | 历史累计交易规模               |
| » underlying    | String | 标的物名称                     |

</Block>

<Block>

# 合约频道

`options.contracts`

该频道发送合约更新消息。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.contracts",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211130-50000-P"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象，字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.contracts",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

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
  "time": 1630576356,
  "channel": "options.contracts",
  "event": "update",
  "result": {
    "contract": "BTC_USDT-20211130-50000-P",
    "create_time": 1637917026,
    "expiration_time": 1638230400,
    "init_margin_high": 0.15,
    "init_margin_low": 0.1,
    "is_call": false,
    "maint_margin_base": 0.075,
    "maker_fee_rate": 0.0004,
    "mark_price_round": 0.1,
    "min_balance_short": 0.5,
    "min_order_margin": 0.1,
    "multiplier": 0.0001,
    "order_price_deviate": 0,
    "order_price_round": 0.1,
    "order_size_max": 1,
    "order_size_min": 10,
    "orders_limit": 100000,
    "ref_discount_rate": 0.1,
    "ref_rebate_rate": 0,
    "strike_price": 50000,
    "tag": "WEEK",
    "taker_fee_rate": 0.0004,
    "underlying": "BTC_USDT",
    "time": 1639051907,
    "time_ms": 1639051907000
  }
}
```

</Example>

推送参数:

| 名称                  | 类型           | 描述                                                                                                                                                       |
| --------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `result`              | Object         | 合约对象                                                                                                                                                   |
| » contract            | string         | 期权合约                                                                                                                                                   |
| » tag                 | string         | 标记                                                                                                                                                       |
| » create_time         | integer(int64) | 合约创建时间                                                                                                                                               |
| » expiration_time     | integer(int64) | 合约到期时间                                                                                                                                               |
| » init_margin_high    | float          | 初始头寸保证金上限                                                                                                                                         |
| » init_margin_low     | float          | 初始头寸保证金下限                                                                                                                                         |
| » is_call             | boolean        | `true` 表示看涨期权，而 `false` 表示看跌期权                                                                                                               |
| » maint_margin_base   | float          | 头寸维持保证金基数                                                                                                                                         |
| » multiplier          | string         | 用于从开票货币转换为结算货币的乘数                                                                                                                         |
| » underlying          | string         | 标的                                                                                                                                                       |
| » maker_fee_rate      | string         | Maker 手续费率，负值表示返点                                                                                                                               |
| » taker_fee_rate      | string         | Taker 手续费率                                                                                                                                             |
| » order_price_round   | string         | 最小订单价格增量                                                                                                                                           |
| » mark_price_round    | string         | M 最小标记价格增量                                                                                                                                         |
| » order_size_min      | integer(int64) | 合约允许的最小订单规模                                                                                                                                     |
| » order_size_max      | integer(int64) | 合约允许的最大订单规模                                                                                                                                     |
| » order_price_deviate | string         | 订单价格与当前指数价格之间的偏差。如果订单价格表示为`order_price`，则必须满足以下条件：`abs(order_price - mark_price) <= mark_price * order_price_deviate` |
| » ref_discount_rate   | string         | 推荐费率折扣                                                                                                                                               |
| » ref_rebate_rate     | string         | 推荐人佣金比例                                                                                                                                             |
| » orders_limit        | integer        | 最大开放订单数量                                                                                                                                           |
| » min_balance_short   | float          | 未完成订单的余额保证金                                                                                                                                     |
| » min_order_margin    | Float          | 未完成订单的订单保证金                                                                                                                                     |
| » strike_price        | float          | 行权价格                                                                                                                                                   |
| » time                | Int64          | 消息创建时间                                                                                                                                               |
| » time_ms             | Int64          | 消息创建时间（毫秒）                                                                                                                                       |

</Block>

<Block>

# 合约 K 线频道

`options.contract_candlesticks`

提供了一种访问 K 线信息的方式。

**推送类型**: `incremental`

**更新频率**: `2s`

</Block>

<Block>

## 客户端订阅

<b><i>如果在 `contract` 前加上 `mark_`，将订阅合约的标记价格 K 线；如果在 `contract` 前加上 `index_`，将订阅指数价格 K 线。</i></b>

<Example>

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.contract_candlesticks",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["10s", "BTC_USDT-20211231-59800-C"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称         | 类型            | 必选 | 描述                                 |
| ------------ | --------------- | ---- | ------------------------------------ |
| `payload`    | `Array[String]` | 是   | 订阅参数。从左到右，`interval`, `cp` |
| » `interval` | String          | 是   | K 线数据点间隔                       |
| » `contract` | String          | 是   | 期权合约名称                         |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象。`interval` **必填**；并与上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）组合使用。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.contract_candlesticks",
  "event": "subscribe",
  "payload": {
    "interval": "1m",
    "underlyings": ["BTC_USDT"]
  }
}
```

```json
{
  "time": 1746754200,
  "channel": "options.contract_candlesticks",
  "event": "subscribe",
  "payload": {
    "interval": "5m",
    "underlyings": ["BTC_USDT", "ETH_USDT"],
    "contracts": ["BTC_USDT-20260213-60000-C"]
  }
}
```

```json
{
  "time": 1746754260,
  "channel": "options.contract_candlesticks",
  "event": "unsubscribe",
  "payload": {
    "interval": "1m",
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

#### Enumerated Values

| 属性     | 值  |
| -------- | --- |
| interval | 10s |
| interval | 1m  |
| interval | 5m  |
| interval | 15m |
| interval | 30m |
| interval | 1h  |
| interval | 4h  |
| interval | 8h  |
| interval | 1d  |
| interval | 7d  |

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
  "time": 1630650451,
  "channel": "options.contract_candlesticks",
  "event": "update",
  "result": [
    {
      "t": 1639039260,
      "v": 100,
      "c": "1041.4",
      "h": "1041.4",
      "l": "1041.4",
      "o": "1041.4",
      "a": "0",
      "n": "10s_BTC_USDT-20211231-59800-C"
    }
  ]
}
```

</Example>

数据结构:

| 名称     | 类型    | 描述                                 |
| -------- | ------- | ------------------------------------ |
| `result` | Array   | K 线数据数组                         |
| `t`      | Integer | 以秒为单位的 Unix 时间戳             |
| `o`      | String  | 开盘价                               |
| `c`      | String  | 收盘价                               |
| `h`      | String  | 最高价                               |
| `l`      | String  | 最低价                               |
| `v`      | Integer | 总成交量                             |
| `a`      | String  | 数量                                 |
| `n`      | String  | 订阅的名称，格式为 `<interval>_<cp>` |

</Block>

<Block>

# K 线频道

`options.ul_candlesticks`

提供了一种访问图表下层 K 线信息的方式。

**推送类型**: `continuous`

**更新频率**: `2s`

</Block>

<Block>

## 客户端订阅

<b><i>如果在`contract`前加上`mark_`，将订阅合约的标记价格 K 线；如果在`contract`前加上`index_`，将订阅指数价格 K 线</i></b>

<Example>

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.ul_candlesticks",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["10s", "BTC_USDT"]
}))
print(ws.recv())
```

</Example>

数据格式:

| 名称         | 类型            | 必选 | 描述                                  |
| ------------ | --------------- | ---- | ------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数。从左到右， `interval`, `cp` |
| » `interval` | String          | 是   | K 线数据点间隔                        |
| » `contract` | String          | 是   | 期权合约名称                          |

#### 枚举

| 属性     | 值  |
| -------- | --- |
| interval | 10s |
| interval | 1m  |
| interval | 5m  |
| interval | 15m |
| interval | 30m |
| interval | 1h  |
| interval | 4h  |
| interval | 8h  |
| interval | 1d  |
| interval | 7d  |

要订阅多个合约或使用不同的时间间隔，只需发送多个具有不同参数的订阅请求。

::: tip
不需要认证。
:::

</Block>

<Block>

## 服务端推送

<Example>

推送示例

```json
{
  "time": 1630650451,
  "channel": "options.ul_candlesticks",
  "event": "update",
  "result": [
    {
      "t": 1639039260,
      "v": 100,
      "c": "1041.4",
      "h": "1041.4",
      "l": "1041.4",
      "o": "1041.4",
      "a": "0",
      "n": "10s_BTC_USDT"
    }
  ]
}
```

</Example>

推送参数:

| 名称     | 类型    | 描述                                 |
| -------- | ------- | ------------------------------------ |
| `result` | Array   | k 线数组                             |
| `t`      | Integer | 以秒为单位的 UNIX 时间戳             |
| `o`      | String  | 开盘价                               |
| `c`      | String  | 收盘价                               |
| `h`      | String  | 最高价                               |
| `l`      | String  | 最低价                               |
| `v`      | Integer | 总成交量                             |
| `a`      | String  | 数量                                 |
| `n`      | String  | 订阅的名称，格式为 `<interval>_<cp>` |

</Block>

<Block>

# 订单簿频道

**`order_book` 通道允许您跟踪 Gate 订单簿深度的状态。它以价格聚合的方式提供，可自定义精度。**

有三个不同的订单簿订阅频道：

- `options.order_book`

传统的频道，使用`all`来推送完整的有限级别订单簿，使用`update`来发送每个订单簿变动事件。

- `options.book_ticker`

  实时推送最佳买入和卖出价格。

- `options.order_book_update`

  按照用户指定的更新频率推送订单簿变动。

::: warning
不建议使用`options.order_book`，而是使用`options.order_book_update`，这样可以提供更及时的更新，并减少网络流量。
:::

如何维护本地订单簿:

1. 订阅`options.order_book_update`，指定级别和更新频率。例如，["BTC_USDT-20211130-50000-C", "1000ms", "10"]表示每 1 秒推送`BTC_USDT`订单簿中前 10 个级别的更新。
2. 缓存`WebSocket`推送。每个推送使用`U`和`u`来告知自上次推送以来的第一个和最后一个更新 ID。
3. 使用 REST API 检索基础订单簿，并确保订单簿 ID 被记录下来（在下面称为`baseID`）
   例如. `https://api.gateio.ws/api/v4/options/order_book?contract=BTC_USDT-20211130-50000-C&limit=10&with_id=true`
   检索`BTC_USDT`的 10 级基础订单簿
4. 遍历缓存的`WebSocket`推送，找到第一个包含`baseID`的推送，即`U <= baseID + 1`且`u >= baseID + 1`，然后从该推送开始消费。请注意，推送中的数量都是绝对值。使用它们来替换相应价格的原始数量。如果数量等于 0，则从订单簿中删除该价格。
5. 转储所有满足`u < baseID+1`的推送。如果`baseID+1 < 第一个推送的U`，则表示当前基础订单簿落后于推送。从步骤 3 开始获取更新的基础订单簿。
6. 如果找到任何满足 U > baseID+1 的后续推送，意味着有一些更新丢失了。从步骤 3 中重新构建本地订单簿。

您可以在以下示例应用程序中找到实现上述方法的示例 [SDK GitHub repository](https://github.com/gate/gatews)

</Block>

<Block>

## 最优买卖价

`options.book_ticker`

**推送类型**: `continuous`

**更新频率**: `real-time`

</Block>

<Block>

### 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.book_ticker",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211130-50000-C"]
}))
print(ws.recv())
```

</Example>

数据结构:

| 名称      | 类型            | 必选 | 描述     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | 是   | 合约列表 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象，字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.book_ticker",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

```json
{
  "time": 1746754200,
  "channel": "options.book_ticker",
  "event": "subscribe",
  "payload": {
    "underlyings": ["BTC_USDT"],
    "contracts": ["ETH_USDT-20260213-3000-P"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

::: tip
不需要认证
:::

</Block>

<Block>

## 服务端推送

如果`a`是空字符串，则表示没有卖单；如果`b`是空字符串，则表示没有买单。

<Example>

推送示例

```json
{
  "time": 1630650452,
  "channel": "options.book_ticker",
  "event": "update",
  "result": {
    "t": 1615366379123,
    "u": 2517661076,
    "s": "BTC_USDT-20211130-50000-C",
    "b": "54696.6",
    "B": 37000,
    "a": "54696.7",
    "A": 47061
  }
}
```

</Example>

推送参数:

| 名称     | 类型    | 描述                                       |
| -------- | ------- | ------------------------------------------ |
| `result` | object  | 订单簿行情对象                             |
| » `t`    | Integer | 订单簿更新时间（毫秒）                     |
| » `u`    | Integer | 订单簿更新 ID                              |
| » `s`    | String  | 合约名称                                   |
| » `b`    | String  | 最佳买入价格。如果没有买单，则为空字符串。 |
| » `B`    | Integer | 最佳买入数量。如果没有买单，则为 0。       |
| » `a`    | String  | 最佳卖出价格。如果没有卖单，则为空字符串。 |
| » `A`    | Integer | 最佳卖出数量。如果没有卖单，则为 0。       |

</Block>

<Block>

## 深度增量更新频道

`options.order_book_update`

**推送类型**: `continuous`

**更新频率**: `100ms`

</Block>

<Block>

### 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.order_book_update",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211130-50000-C", "1000ms", "20"]
}))
print(ws.recv())
```

</Example>

数据结构:

| 名称         | 类型            | 必选 | 描述                                           |
| ------------ | --------------- | ---- | ---------------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数，从左到右， `contract`, `interval`    |
| » `contract` | String          | 是   | 合约名称                                       |
| » `interval` | String          | 是   | 推送更新速度                                   |
| » `level`    | String          | 否   | 感兴趣的可选级别。只有在其中的更新才会被推送。 |

#### 枚举

| 属性     | 值     |
| -------- | ------ |
| interval | 100ms  |
| interval | 1000ms |

| 属性  | 值  |
| ----- | --- |
| level | 5   |
| level | 10  |
| level | 20  |
| level | 50  |

您可以多次订阅/取消订阅。之前订阅的合约不会被覆盖，除非明确取消订阅。

::: tip
不需要认证。
:::

</Block>

<Block>

### 服务端推送

<Example>

推送示例

```json
{
  "time": 1630650445,
  "channel": "options.order_book_update",
  "event": "update",
  "result": {
    "t": 1615366381417,
    "s": "BTC_USDT-20211130-50000-C",
    "U": 2517661101,
    "u": 2517661113,
    "b": [
      {
        "p": "54672.1",
        "s": 0
      },
      {
        "p": "54664.5",
        "s": 58794
      }
    ],
    "a": [
      {
        "p": "54743.6",
        "s": 0
      },
      {
        "p": "54742",
        "s": 95
      }
    ]
  }
}
```

</Example>

推送参数:

| 名称     | 类型    | 描述                                                 |
| -------- | ------- | ---------------------------------------------------- |
| `result` | object  | 自上次更新以来的卖出和买入变动。                     |
| »`t`     | Integer | 订单簿更新时间（毫秒）。                             |
| »`s`     | Integer | 合约名称                                             |
| »`U`     | Integer | 自上次更新以来的首个订单簿更新 ID。                  |
| »`u`     | Integer | 自上次更新以来的最后一个订单簿更新 ID。              |
| »`b`     | String  | 变更的买单                                           |
| »»`p`    | String  | 变更价格                                             |
| »»`s`    | String  | 变更后的绝对数量值。如果为 0，则从订单簿中移除该价格 |
| »`a`     | String  | 变更的买单                                           |
| »»`p`    | String  | 变更的价格                                           |
| »»`s`    | String  | 变更后的绝对数量值。如果为 0，则从订单簿中移除该价格 |

</Block>

<Block>

## 深度全量更新频道

`options.order_book`

**推送类型**: `continuous`

**更新频率**: `250ms`

</Block>

<Block>

### 客户端订阅

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "options.order_book",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT-20211130-50000-C", "20", "0"]
}))
print(ws.recv())
```

</Example>

数据结构:

| 名称       | 类型   | 必选 | 描述                         |
| ---------- | ------ | ---- | ---------------------------- |
| `contract` | String | 是   | 合约名称                     |
| `limit`    | String | 是   | 法定限制：50，20，10，5，1。 |
| `accuracy` | String | 是   | 目前只支持"0"。              |

#### 枚举

| 属性  | 值  |
| ----- | --- |
| level | 5   |
| level | 10  |
| level | 20  |

| 属性     | 值  |
| -------- | --- |
| accuracy | 0   |

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

::: tip
不需要认证
:::

</Block>

<Block>

### 服务端推送

<Example>

推送示例

```json
{
  "time": 1630650445,
  "channel": "options.order_book",
  "event": "all",
  "result": {
    "t": 1541500161123,
    "contract": "BTC_USDT-20211130-50000-C",
    "id": 93973511,
    "asks": [
      {
        "p": "97.1",
        "s": 2245
      },
      {
        "p": "97.1",
        "s": 2245
      }
    ],
    "bids": [
      {
        "p": "97.1",
        "s": 2245
      },
      {
        "p": "97.1",
        "s": 2245
      }
    ]
  }
}
```

Or

```json
{
  "channel": "options.order_book",
  "event": "update",
  "time": 1630650445,
  "result": [
    {
      "p": "49525.6",
      "s": 7726,
      "c": "BTC_USDT-20211130-50000-C",
      "id": 93973511
    }
  ]
}
```

</Example>

推送参数:

| 名称     | 类型    | 描述                                                                         |
| -------- | ------- | ---------------------------------------------------------------------------- |
| `result` | Array   | 结果                                                                         |
| »`c`     | String  | 期权合约名称                                                                 |
| »`s`     | Integer | 这个数字是最终值，计算得出的值。正数表示多头（买入），负数表示空头（卖出）。 |
| »`p`     | String  | 订单簿价格                                                                   |
| »`id`    | Integer | 价格订单簿 ID                                                                |

</Block>

<Block>

# 订单频道

`options.orders`

提供了一种接收用户已关闭订单的方式。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.orders",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001","BTC_USDT-20211130-65000-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据结构:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 合于期权名称 |

#### 对象格式（按标的）

除上述 `[userId, contract, ...]` 数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。可与 `["<userId>", "!all"]` 等形式在同一连接内共存。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.orders",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

```json
{
  "time": 1746754200,
  "channel": "options.orders",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"],
    "contracts": ["ETH_USDT-20260213-3000-C"]
  }
}
```

```json
{
  "time": 1746754260,
  "channel": "options.orders",
  "event": "unsubscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

如果您想订阅所有合约中的所有订单更新，您可以在合约列表中包含`!all`。

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
  "time": 1630654851,
  "channel": "options.orders",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT-20211130-65000-C",
      "create_time": 1637897000,
      "fill_price": 0,
      "finish_as": "cancelled",
      "iceberg": 0,
      "id": 106,
      "is_close": false,
      "is_liq": false,
      "is_reduce_only": false,
      "left": -10,
      "mkfr": 0.0004,
      "price": 15000,
      "refr": 0,
      "refu": 0,
      "size": -10,
      "status": "finished",
      "text": "web",
      "tif": "gtc",
      "tkfr": 0.0004,
      "underlying": "BTC_USDT",
      "user": "9xxx",
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

已更新的订单列表。请注意，可能会在一条推送中更新多个合约的订单。

推送参数:

| 名称             | 类型            | 描述                                                                                                                                                                                                                                       |
| ---------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `result`         | `Array[Object]` | 更新的订单列表                                                                                                                                                                                                                             |
| » id             | integer(int64)  | 期权订单 ID                                                                                                                                                                                                                                |
| » user           | String          | 用户 ID                                                                                                                                                                                                                                    |
| » create_time    | integer(int64)  | 订单创建时间                                                                                                                                                                                                                               |
| » finish_as      | string          | 订单如何完成：- filled：全部成交 - cancelled：手动取消 - liquidated：因清算而取消 - ioc：即时生效订单，立即完成 - auto_deleveraged：被自动减仓完成 - reduce_only：因为设置了"只减仓"，所以取消了订单 - position_closed：因为持仓平仓而取消 |
| » status         | string          | 订单状态 - `open`: 等待成交 - `finished`: 完成                                                                                                                                                                                             |
| » contract       | string          | 合约名称                                                                                                                                                                                                                                   |
| » size           | integer(int64)  | 订单大小。使用正数表示买入报价，使用负数表示卖出报价。                                                                                                                                                                                     |
| » iceberg        | integer(int64)  | 显示冰山订单的显示大小。非冰山订单显示大小为 0。请注意，您需要支付隐藏数量的吃单方手续费。                                                                                                                                                 |
| » price          | string          | 订单价格。若以`tif`设置为`ioc`的市价单，则价格为 0。                                                                                                                                                                                       |
| » is_close       | boolean         | 该订单是否用于平仓操作。                                                                                                                                                                                                                   |
| » is_reduce_only | boolean         | 该订单是否为"只减仓"订单。                                                                                                                                                                                                                 |
| » is_liq         | boolean         | 该订单是否用于清算操作。                                                                                                                                                                                                                   |
| » tif            | string          | 有效期限 - gtc: 长期有效 - ioc: 即时成交或取消, 仅吃单 - poc: 待定或已取消，仅减仓                                                                                                                                                         |
| » left           | integer(int64)  | 待交易的剩余数量                                                                                                                                                                                                                           |
| » fill_price     | string          | 订单成交价格                                                                                                                                                                                                                               |
| » tkfr           | Float           | 吃单费用                                                                                                                                                                                                                                   |
| » mkfr           | Float           | 挂单费用                                                                                                                                                                                                                                   |
| » refu           | integer         | 推荐人 ID                                                                                                                                                                                                                                  |
| » refr           | Float           | 推荐人返利                                                                                                                                                                                                                                 |
| » underlying     | String          | 标的物名称                                                                                                                                                                                                                                 |
| » time           | int             | 创建时间                                                                                                                                                                                                                                   |
| » time_ms        | Int             | 创建时间（毫秒）                                                                                                                                                                                                                           |
| » text        | String             | 订单自定义信息，用户可以用该字段设置自定义 ID，用户自定义字段必须满足以下条件：<br />1. 必须以 t- 开头<br />2. 不计算 t- ，长度不能超过 28 字节<br />3. 输入内容只能包含数字、字母、下划线(_)、中划线(-) 或者点(.)<br />除用户自定义信息以外，以下为内部保留 field，标识订单来源:<br />- web: 网页<br />- api: API 调用<br />- app: 移动端<br />- auto_deleveraging: 自动减仓<br />- liquidation: 强制平仓<br />- insurance: 保险              |

</Block>

<Block>

# 用户交易频道

`options.usertrades`

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.usertrades",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001", "BTC_USDT-20211216-44800-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 期权合约名称 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。可与 `!all` 等数组订阅在同一连接内共存。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.usertrades",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

如果您想订阅所有合约中的用户交易更新，可以在合约列表中包含`!all`。

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
  "time": 1639144214,
  "channel": "options.usertrades",
  "event": "update",
  "result": [
    {
      "id": "1",
      "underlying": "BTC_USDT",
      "order": "557940",
      "contract": "BTC_USDT-20211216-44800-C",
      "create_time": 1639144214,
      "create_time_ms": 1639144214583,
      "price": "4999",
      "role": "taker",
      "size": -1,
      "fee": "0.001",
      "text": "t-xer01sax4yu"
    }
  ]
}
```

</Example>

已更新的用户交易列表。请注意，可能会在一条推送中更新多个合约的交易记录。

推送参数:

| 名称              | 类型    | 描述                      |
| ----------------- | ------- | ------------------------- |
| `result`          | Array   | 结果                      |
| »`contract`       | String  | 期权合约名称              |
| »`create_time`    | Integer | 创建时间                  |
| »`create_time_ms` | Integer | 创建时间（毫秒）          |
| »`id`             | String  | 交易 id                   |
| »`order`          | String  | 订单 Id                   |
| »`price`          | String  | 交易价格                  |
| »`size`           | Integer | 交易量                    |
| »`role`           | String  | 用户角色（挂单方/吃单方） |
| »`fee`            | String  | 手续费             |
| »`text`           | String  | 用户自定义信息 |

#### 枚举

| 属性 | 值    |
| ---- | ----- |
| role | maker |
| role | taker |

</Block>

<Block>

# 清算频道

`options.liquidates`

提供了一种接收用户清算信息的方式。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.liquidates",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001", "BTC_USDT-20211130-50000-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 期权合约名称 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.liquidates",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

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
  "channel": "options.liquidates",
  "event": "update",
  "time": 1630654851,
  "result": [
    {
      "user": "1xxxx",
      "init_margin": 1190,
      "maint_margin": 1042.5,
      "order_margin": 0,
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

Result format:

| 名称            | 类型   | 描述                   |
| --------------- | ------ | ---------------------- |
| `result`        | Array  | 结果                   |
| »`time`         | int64  | 平仓时间               |
| »`time`         | int64  | 平仓时间（毫秒）       |
| »`user`         | string | 用户 id                |
| »`init_margin`  | float  | 初始持仓保证金         |
| »`maint_margin` | float  | 持仓维持保证金         |
| »`order_margin` | float  | 未完成订单的委托保证金 |

</Block>

<Block>

# 用户结算频道

`options.user_settlements`

提供了一种接收用户结算信息的方式。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.user_settlements",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001", "BTC_USDT-20211130-65000-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 期权合约名称 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.user_settlements",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

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
  "channel": "options.user_settlements",
  "event": "update",
  "time": 1639051907,
  "result": [
    {
      "contract": "BTC_USDT-20211130-65000-C",
      "realised_pnl": -13.028,
      "settle_price": 70000,
      "settle_profit": 5,
      "size": 10,
      "strike_price": 65000,
      "underlying": "BTC_USDT",
      "user": "9xxx",
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

| 名称             | 类型    | 描述             |
| ---------------- | ------- | ---------------- |
| `result`         | Array   | 结果             |
| »`realised_pnl`  | Float   | 实现盈亏         |
| »`settle_price`  | Float   | 结算价格         |
| »`settle_profit` | Integer | 每份结算利润     |
| »`strike_price`  | float   | 行权价格         |
| »`underlying`    | string  | 标的物名称       |
| »`size`          | Integer | 交易量           |
| »`time`          | Integer | 结算时间         |
| »`time_ms`       | Integer | 结算时间（毫秒） |
| »`user`          | String  | 用户 id          |
| »`contract`      | String  | 期权合约名称     |

</Block>

<Block>

# 持仓平仓频道

`options.position_closes`

提供了一种接收用户平仓信息的方式。

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

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.position_closes",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001", "BTC_USDT-20211130-50000-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 期权合约名称 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.position_closes",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

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
  "channel": "options.position_closes",
  "event": "update",
  "time": 1630654851,
  "result": [
    {
      "contract": "BTC_USDT-20211130-50000-C",
      "pnl": -0.0056,
      "settle_size": 0,
      "side": "long",
      "text": "web",
      "underlying": "BTC_USDT",
      "user": "11xxxxx",
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

推送参数:

| 名称          | 类型    | 描述                 |
| ------------- | ------- | -------------------- |
| `result`      | Array   | 数组结果             |
| »`contract`   | String  | 期权合约名称         |
| »`pnl`        | Float   | 利润与损失           |
| »`side`       | String  | 持仓方向，多头或空头 |
| »`text`       | String  | 平仓订单的文本       |
| »`time`       | Integer | 平仓时间             |
| »`time_ms`    | Integer | 平仓时间（毫秒）     |
| »`user`       | String  | 用户 id              |
| »`underlying` | string  | 标的唔名称           |

#### 枚举值

| 属性 | 值    |
| ---- | ----- |
| side | long  |
| side | Short |

</Block>

<Block>

# 余额频道

`options.balances`

提供一种接收用户余额信息的方式。

**推送类型**: `continuous`

**更新频率**: `real-time`

</Block>

<Block>

## 客户端订阅

<Example>

代码示例

```python
import json
import time
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.balances",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述    |
| ---------- | ------ | ---- | ------- |
| `user id ` | String | 是   | 用户 id |

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
  "channel": "options.balances",
  "event": "update",
  "time": 1630654851,
  "result": [
    {
      "balance": 60.79009,
      "change": -0.5,
      "text": "BTC_USDT-20211130-55000-P",
      "type": "set",
      "user": "11xxxx",
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

推送参数:

| 名称       | 类型    | 描述                 |
| ---------- | ------- | -------------------- |
| `result`   | Array   | 结果对象数组         |
| »`balance` | Float   | 变化后的余额         |
| »`change`  | Float   | 变化大小             |
| »`text`    | String  | 余额变动消息         |
| »`time`    | Integer | 余额变动时间         |
| »`time_ms` | Integer | 余额变动时间（毫秒） |
| »`type`    | String  | 类型                 |
| »`user`    | String  | 用户 id              |

</Block>

<Block>

# 持仓频道

`options.positions`

提供了一种接收用户持仓信息的方式。

**推送类型**: `continuous`

**更新频率**: `real-time`

如果您想订阅所有的持仓更新，请在列表中使用 `!all`

</Block>

<Block>

## 客户端订阅

<Example>

代码示例

```python
import json
import time
from websocket import create_connection

ws = create_connection("wss://op-ws.gateio.live/v4/ws")
request = {
    "time": int(time.time()),
    "channel": "options.positions",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1001", "BTC_USDT-20211130-65000-C"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

数据格式:

| 名称       | 类型   | 必选 | 描述         |
| ---------- | ------ | ---- | ------------ |
| `user id ` | String | 是   | 用户 id      |
| `contract` | String | 是   | 期权合约名称, `!all` - 表示订阅所有期权合约 |

#### 对象格式（按标的）

除上述数组外，还可在 `payload` 中使用对象；须携带[鉴权](#鉴权)。对象字段为上文「按标的订阅」中的 `underlyings` / `contracts`（至少其一非空）。可与 `!all` 等数组订阅在同一连接内共存。

<Example>

```json
{
  "time": 1746754200,
  "channel": "options.positions",
  "event": "subscribe",
  "auth": { "method": "api_key", "KEY": "<api_key>", "SIGN": "<sign>" },
  "payload": {
    "underlyings": ["BTC_USDT"]
  }
}
```

</Example>

您可以多次订阅/取消订阅。之前订阅的合约除非明确取消订阅，否则不会被覆盖。

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
  "time": 1630654851,
  "channel": "options.positions",
  "event": "update",
  "error": null,
  "result": [
    {
      "entry_price": 0,
      "realised_pnl": -13.028,
      "size": 0,
      "contract": "BTC_USDT-20211130-65000-C",
      "user": "9010",
      "time": 1639051907,
      "time_ms": 1639051907000
    }
  ]
}
```

</Example>

推送参数:

| 名称            | 类型    | 描述                           |
| --------------- | ------- | ------------------------------ |
| `result`        | Array   | Array of objects               |
| »`contract`     | String  | 期权合约名称                   |
| »`entry_price`  | Float   | 入场价格                       |
| »`realised_pnl` | Float   | 已实现盈亏                     |
| »`size`         | Integer | 合约大小                       |
| »`time`         | Integer | 更新的 UNIX 时间戳             |
| »`time_ms`      | Integer | 以毫秒为单位的更新 UNIX 时间戳 |
| »`user`         | String  | 用户 id                        |

</Block>
