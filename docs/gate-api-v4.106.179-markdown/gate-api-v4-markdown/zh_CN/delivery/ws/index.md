---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate Delivery WebSocket v4.0.0

Gate 提供简单而强大的 Websocket API，将 Gate BTC/USDT 交割合约交易状态集成到您的业务或应用程序中。

我们在`Python`中有语言绑定，将来还会有更多！您可以在右侧的深色区域中查看代码示例，并且可以通过右上角的选项卡切换示例的编程语言

## 服务地址

我们提供 BTC/USDT 结算交割合约交易服务器地址，您可以根据自己的情况选择其中之一

### USDT Contract

地址列表:

- 线上交易: `wss://fx-ws.gateio.ws/v4/ws/delivery/usdt`
- 模拟盘交易: `wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt`

### BTC Contract

Base URLs:

- 线上交易: `wss://fx-ws.gateio.ws/v4/ws/delivery/btc`
- 模拟盘交易: `wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/btc`

</Block>

<Block>

## Changelog

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

    def _send_ping(self, interval, event):
        while not event.wait(interval):
            self.last_ping_tm = time.time()
            if self.sock:
                try:
                    self.sock.ping()
                except Exception as ex:
                    logger.warning("send_ping routine terminated: {}".format(ex))
                    break
                try:
                    self._request("futures.ping", auth_required=False)
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
    ws.subscribe("futures.tickers", ['BTC_USDT_20230630'], False)


if __name__ == "__main__":
    logging.basicConfig(format="%(asctime)s - %(message)s", level=logging.DEBUG)
    app = GateWebSocketApp("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt",
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
	u := url.URL{Scheme: "wss", Host: "fx-ws.gateio.ws", Path: "/v4/ws/delivery/usdt"}
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
	pingMsg := NewMsg("futures.ping", "", t, []string{})
	err = pingMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe order book
	orderBookMsg := NewMsg("futures.order_book", "subscribe", t, []string{"BTC_USDT_20230630"})
	err = orderBookMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe positions
	positionsMsg := NewMsg("futures.positions", "subscribe", t, []string{"USERID", "BTC_USDT_20230630"})
	positionsMsg.sign()
	err = positionsMsg.send(c)
	if err != nil {
		panic(err)
	}

	select {}
}
```

</Example>

2026-06-25

- 新增 `futures.obu` 深度频道 V2（OBU），提供更快更新的深度信息推送

2025-03-21

- 修改了所有私有通道订阅参数的说明文档

2021-08-06

- 支持 BTC 交割合约

2020-08-08

- 添加完整代码 demo(golang, python)

2020-08-07

- 新增自动订单通道

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用订阅通道/channel（例如`ticker`、`order_book`等）都支持一些不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据通知。

2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据通知。

3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）。

4. **`all`**

如果有新订阅的数据（所有数据）可用，服务器将向客户端发送通知。

### 请求参数

每个请求都遵循通用格式，其中包含`time`、`channel`、`event`和`payload`。

| 名称      | 类型    | 必选 | 描述                                                          |
| --------- | ------- | ---- | ------------------------------------------------------------- |
| `id`      | Integer | 否   | 可选的请求 ID，将由服务器发回，以帮助您识别服务器响应哪个请求 |
| `time`    | Integer | 是   | 请求时间                                                      |
| `channel` | String  | 是   | 请求 subscribe/unsubscribe 通道                               |
| `auth`    | String  | 否   | 请求身份验证信息，请参阅身份验证部分了解详细信息              |
| `event`   | String  | 是   | 请求 `event` (subscribe/unsubscribe/update/all/api)           |
| `payload` | Array   | 是   | 请求详细参数                                                  |

### 推送参数

与请求类似，响应遵循以下通用格式，其中包含： `time`, `channel`, `event`
, `error` 和 `result`.

| 名称      | 类型    | 必选 | 描述                      |
| --------- | ------- | ---- | ------------------------- |
| `time`    | Integer | 是   | 响应时间                  |
| `time_ms` | Integer | 是   | 毫秒响应时间              |
| `channel` | String  | 是   | 响应通道                  |
| `event`   | String  | 是   | 响应通道事件 (update/all) |
| `error`   | Object  | 是   | 响应错误                  |
| `result`  | Any     | 是   | 返回来自服务端的新数据通知 或 对客户端请求的响应。如果有错误返回则 `error` 不为空，没有错误则此字段为空。 |

注意：如果它是服务端发起的数据更新通知 那么 `result` 的类型是基于 channel 的，不同 channel 的 `result` 类型有所不同。

但如果是对客户端订阅请求的响应，那么 `result` 为固定的 `{"status": "success"}`。
验证订阅请求是否成功，您只需要检查 `error` 字段是否为空即可，不需要再解析 `result` 字段。

为了简单起见，下面的频道（channel）描述只给出对应频道的 payload 格式。

### 错误

如果出现错误，您会收到`error`字段，其中包含错误代码和错误的类型。

| Code | Message                   |
| ---- | ------------------------- |
| `1`  | `invalid argument struct` |
| `2`  | `invalid argument`        |
| `3`  | `service error`           |
| `4`  | `authentication fail`     |

</Block>

<Block>

## 鉴权

如果通道是私有的，则请求体需要携带认证信息，
例如`futures.usertrades`

WebSocket 认证使用与 HTTP API 相同的签名计算方法，但具有
以下差异：

1. 签名字符串拼接方式：`channel=<channel>&event=<event>&time=<time>`,
   其中`<channel>`、`<event>`、`<time>`是对应的请求信息
2. 身份验证信息在请求正文中的`auth`字段中发送。

<Example>

```python
# example WebSocket signature calculation implementation in Python
import hmac, hashlib, time

## api_key method generate secret
secret = 'xxxx'
message = 'channel=%s&event=%s&time=%s' % ('futures.orders', 'subscribe', int(time.time()))
print(hmac.new(secret, message, hashlib.sha512).hexdigest())  ## Generating signature
```

</Example>

您可以登录账户获取交割合约账户的 api_key 和 secret。

| 名称     | 类型   | 描述               |
| -------- | ------ | ------------------ |
| `method` | String | 验证方式:`api_key` |
| `KEY`    | String | apiKey 的值        |
| `SIGN`   | String | 签名结果           |

</Block>

<Block>

# System API

**提供系统状态检查，如 `ping/pong`。**

</Block>

<Block>

## Ping/Pong

**检查服务器/客户端连接。**

**Gate websocket 使用协议层 ping/pong 消息。服务器会发起 ping 操作。如果客户端没有回复，客户端将被断开。**

[websocket rfc 协议](https://tools.ietf.org/html/rfc6455)

**如果想主动检测连接状态，可以发送应用层 ping 消息，并接收 pong 消息。**

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send('{"time" : 123456, "channel" : "futures.ping"}')
print(ws.recv())
```

</Example>

<Example>

`futures.ping`
上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545404023,
  "channel": "futures.pong",
  "event": "",
  "error": null,
  "result": null
}
```

</Example>

### 请求参数

- channel

  `futures.ping`

</Block>

<Block>

# Ticker 频道

**Ticker 是合约的高层概览, 它向您显示最高、最低、最近的交易价格。还包括每日交易量以及价格在过去一天内的波动情况等信息。**

</Block>

<Block>

## 订阅 Ticker

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.tickers", "event": "subscribe", "payload" : ["BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545404023,
  "channel": "futures.tickers",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅交割合约的 Ticker**

### 请求参数

- channel

  `futures.tickers`

- event

  `subscribe`

- params

  | 参数      | 类型  | 必选 | 描述     |
  | --------- | ----- | ---- | -------- |
  | `payload` | Array | 是   | 合约列表 |

</Block>

<Block>

## Tickers 推送

<Example>

```json
{
  "time": 1541659086,
  "channel": "futures.tickers",
  "event": "update",
  "error": null,
  "result": [
    {
      "contract": "BTC_USDT_20230630",
      "last": "118.4",
      "change_percentage": "0.77",
      "funding_rate": "-0.000114",
      "funding_rate_indicative": "0.01875",
      "mark_price": "118.35",
      "index_price": "118.36",
      "total_size": "73648",
      "volume_24h": "745487577",
      "volume_24h_btc": "117",
      "volume_24h_usd": "419950",
      "quanto_base_rate": "",
      "volume_24h_quote": "1665006",
      "volume_24h_settle": "178",
      "volume_24h_base": "5526"
    }
  ]
}
```

</Example>

**推送已订阅的 Tickers**

### 通知

- channel

  `futures.tickers`

- event

  `update`

- params

  | 字段                       | 类型   | 描述                  |
  | -------------------------- | ------ | --------------------- |
  | `result`                   | Array  | 结果                  |
  | »`contract`                | String | 交割合同的名称        |
  | »`last`                    | String | 最新的价格            |
  | »`change_percentage`       | String | 价格的变动百分比      |
  | »`funding_rate`            | String | 资金费率              |
  | »`funding_rate_indicative` | String | 资金费率的指示        |
  | »`mark_price`              | String | 标记价格              |
  | »`index_price`             | String | 指数价格              |
  | »`total_size`              | String | 总交易量大小          |
  | »`volume_24h`              | String | 24 小时成交量         |
  | »`quanto_base_rate`        | String | 合约基准汇率          |
  | »`volume_24h_btc`          | String | 24 小时 BTC 的成交量  |
  | »`volume_24h_usd`          | String | 24 小时 USD 的成交量  |
  | »`volume_24h_quote`        | String | 24 小时报价货币交易量 |
  | »`volume_24h_settle`       | String | 24 小时结算货币交易量 |
  | »`volume_24h_base`         | String | 24 小时基础货币交易量 |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.tickers",
    "event": "unsubscribe",
    "payload": ["BTC_USDT_20230630"]
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545404900,
  "channel": "futures.tickers",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅 Ticker**

### 请求参数

- channel

  `futures.tickers`

- event

  `unsubscribe`

</Block>

<Block>

# 交易频道

**该通道在 Gate 上发生交易时发送交易消息，它包括交易的详细信息，如价格、数量、时间和类型。**

</Block>

<Block>

## 订阅 Trades

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.trades", "event": "subscribe", "payload" : ["BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545405058,
  "channel": "futures.trades",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**Trades 推送**

### 请求参数

- channel

  `futures.trades`

- event

  `subscribe`

- params

  | 参数      | 类型  | 必选 | 描述     |
  | --------- | ----- | ---- | -------- |
  | `payload` | Array | 是   | 合约列表 |

</Block>

<Block>

## Trades 推送

<Example>

正数表示 taker 是 buyer, 复数则是 seller.

```json
{
  "channel": "futures.trades",
  "event": "update",
  "time": 1541503698,
  "result": [
    {
      "size": -108,
      "id": 27753479,
      "create_time": 1545136464,
      "create_time_ms": 1545136464123,
      "price": "96.4",
      "contract": "BTC_USDT_20230630"
    }
  ]
}
```

</Example>

**推送 Trades 更新**

### Notify

- channel

  `futures.trades`

- event

  `update`

- params

  | 字段              | 类型   | 描述                     |
  | ----------------- | ------ | ------------------------ |
  | `result`          | Array  | 结果                     |
  | »`contract`       | String | 交割合约名称             |
  | »`size`           | int    | 交易大小                 |
  | »`id`             | int    | 交易的唯一标识           |
  | »`create_time`    | int    | 交易的创建时间，单位秒   |
  | »`create_time_ms` | int    | 交易的创建时间，单位毫秒 |
  | »`price`          | string | 交易价格                 |

</Block>

<Block>

## 取消订阅

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.trades", "event": "subscribe", "payload" : ["BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545404900,
  "channel": "futures.trades",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅 Trades**

### 请求参数

- channel

  `futures.trades`

- event

  `unsubscribe`

</Block>

<Block>

# 订单簿频道

**`order_book` 通道允许您跟踪 Gate 订单簿深度的状态。它以价格聚合的方式提供，可自定义精度。**

支持三种不同的订单簿订阅通道：

- `futures.order_book`

传统通道，使用 all 推送完整的有限级别订单簿，使用 update 发送每个订单簿变动事件。

- `futures.book_ticker`

实时推送最佳买价和卖价。

- `futures.order_book_update`

推送用户指定更新频率的订单簿变动。

- `futures.obu`

订单簿增量（OBU），提供更快更新的深度信息。

::: warning
不建议通过 futures.order_book 接收订单簿更新。`futures.order_book_update` 可以提供更及时的更新，而且流量更少。
:::

如何维护本地订单簿：

1. 订阅 `futures.order_book_update`，并指定级别和更新频率，例如 `["BTC_USDT_20230630", "1000ms", "10"]` 每秒推送 `BTC_USDT` 订单簿的前 10 个级别的更新。
2. 缓存 WebSocket 通知。每个通知使用 `U` 和 `u` 来告知上次通知以来的第一个和最后一个更新 ID。
3. 使用 REST API 检索基础订单簿，并确保记录了订单簿 ID（在下面被称为 baseID）。例如，https://api.gateio.ws/api/v4/delivery/usdt/order_book?contract=BTC_USDT_20230630&limit=10&with_id=true 可以检索到 `BTC_USDT_20230630` 的前 10 级基础订单簿。
4. 遍历缓存的 WebSocket 通知，并找到第一个包含 baseID 的通知，即 `U <= baseId+1` 且 `u >= baseId+1`，然后从这里开始消费。注意通知中的大小都是绝对值。使用它们来替换相应价格的原始大小。如果大小等于 0，则从订单簿中删除该价格。
5. 删除满足 `u < baseID+1` 的所有通知。如果 `aseID+1 < 第一个通知 U`，则表示当前基础订单簿落后于通知。从步骤 3 开始检索更新的基础订单簿。
6. 如果发现满足 `U > baseID+1` 的任何后续通知，表示已丢失一些更新。从步骤 3 重新构建本地订单簿。

</Block>

<Block>

## 深度全量更新频道

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.order_book", "event": "subscribe", "payload" : ["BTC_USDT_20230630", "20", "0"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.order_book",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅订单簿**

### 请求参数

- channel

  `futures.order_book`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述                                      |
  | ---------- | ------ | ---- | ----------------------------------------- |
  | `contract` | String | 是   | 交割合约名称                              |
  | `limit`    | String | 是   | 限制，合法的限制值：100, 50, 20, 10, 5, 1 |
  | `interval` | String | 是   | 合法的间隔值："0"                         |

</Block>

<Block>

## 传统订单簿通知

<Example>

```json
{
  "channel": "futures.order_book",
  "event": "all",
  "time": 1541500161,
  "time_ms": 1541500161123,
  "result": {
    "t": 1541500161123,
    "contract": "BTC_USDT_20230630",
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

或者

```json
{
  "channel": "futures.order_book",
  "event": "update",
  "time": 1541500167,
  "time_ms": 1541500167123,
  "result": [
    {
      "p": "97.5",
      "s": 6541,
      "contract": "BTC_USDT_20230630",
      "id": 93973512
    }
  ]
}
```

</Example>

**推送订单簿更新信息**

### 通知

- channel

  `futures.order_book`

- event

  `update/all`

- params

  | 字段        | type    | 描述                                                       |
  | ----------- | ------- | ---------------------------------------------------------- |
  | `result`    | Array   | 结果                                                       |
  | »`contract` | String  | 交割合约名称                                               |
  | »`s`        | Integer | 最终计算的结果，正数表示多头（买单），负数表示空头（卖单） |
  | »`p`        | String  | 订单簿价格                                                 |
  | »`id`       | Integer | 订单簿唯一标识                                             |

</Block>

<Block>

## 取消传统订单簿

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.order_book", "event": "unsubscribe", "payload" : ["BTC_USDT_20230630", "20", "0"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.order_book",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅指定合约订单簿**

### 请求参数

- channel

  `futures.order_book`

- event

  `unsubscribe`

</Block>

<Block>

## 最优买卖价

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.book_ticker", "event": "subscribe", "payload" : ["BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.book_ticker",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅订单簿标价**

### 请求参数

- channel

  `futures.book_ticker`

- event

  `subscribe`

- params

  `payload` 是指感兴趣的交割合约列表。

</Block>

<Block>

## 推送订单簿标记信息

如果 `a` 是空字符串，表示卖单为空；如果 `b` 是空字符串，表示买单为空。

<Example>

```json
{
  "time": 1615366379,
  "time_ms": 1615366379123,
  "channel": "futures.book_ticker",
  "event": "update",
  "error": null,
  "result": {
    "t": 1615366379123,
    "u": 2517661076,
    "s": "BTC_USDT_20230630",
    "b": "54696.6",
    "B": 37000,
    "a": "54696.7",
    "A": 47061
  }
}
```

</Example>

**通知合约最佳买/卖单**

### 通知

- channel

  `futures.book_ticker`

- event

  `update`

- params

  | 字段     | 类型    | 描述                                       |
  | -------- | ------- | ------------------------------------------ |
  | `result` | object  | 结果                                       |
  | »`t`     | Integer | 订单簿标价生成的时间戳                     |
  | »`u`     | Integer | 订单簿更新的唯一标识符                     |
  | »`s`     | String  | 交割合约名称                               |
  | »`b`     | String  | 最佳买入价格，如果没有买单，返回空字符串   |
  | »`B`     | Integer | 最佳买入量，如果没有买单，这个值为 0       |
  | »`a`     | String  | 最佳卖出价格，如何没有卖单，它将是空字符串 |
  | »`A`     | Integer | 最佳卖出量，如果没有卖单，它将是 0         |

</Block>

<Block>

## 退订最佳买卖价

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.book_ticker", "event": "unsubscribe", "payload" : ["BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.book_ticker",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订指定合约最佳买卖价**

### 请求参数

- channel

  `futures.book_ticker`

- event

  `unsubscribe`

</Block>

<Block>

## 深度增量更新频道

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/btc")
ws.send(
    '{"time" : 123456, "channel" : "futures.order_book_update", "event": "subscribe", "payload" : ["BTC_USDT_20230630", "1000ms", "20"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.order_book_update",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅订单簿更新**

### 请求参数

- channel

  `futures.order_book_update`

- event

  `subscribe`

- payload

  | 参数        | 类型   | 必选 | 描述                                                                |
  | ----------- | ------ | ---- | ------------------------------------------------------------------- |
  | `contract`  | String | 是   | 交割合约名称                                                        |
  | `frequency` | String | 是   | 更新频率: `100ms` 或者 `1000ms`                                     |
  | `level`     | String | 否   | 可选级别，仅会通知所选择的更新，支持 `100`、`50`、`20`、`10` 或 `5` |

</Block>

<Block>

## 订单簿更新通知

<Example>

```json
{
  "time": 1615366381,
  "time_ms": 1615366381123,
  "channel": "futures.order_book_update",
  "event": "update",
  "error": null,
  "result": {
    "t": 1615366381417,
    "s": "BTC_USDT_20230630",
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

**推送订单簿更新信息**

### 通知

- channel

  `futures.order_book_update`

- event

  `update`

- params

  | 字段     | 类型    | 描述                                         |
  | -------- | ------- | -------------------------------------------- |
  | `result` | object  | 自上次更新以来变更的买单和卖单               |
  | »`t`     | Integer | 订单簿生成的时间戳，单位毫秒                 |
  | »`s`     | String  | 交割合约名称                                 |
  | »`U`     | Integer | 自上次更新以来的第一个订单簿更新 ID          |
  | »`u`     | Integer | 自上次更新以来的最后一个订单簿更新 ID        |
  | »`b`     | String  | 变更的买单                                   |
  | »`p`     | String  | 已变更的价格                                 |
  | »`s`     | String  | 变更绝对值，如果为 0，则从订单簿中删除此价格 |
  | »`a`     | String  | 变更的卖单                                   |
  | »`p`     | String  | 已变更的价格                                 |
  | »`s`     | String  | 变更绝对值，如果为 0，则从订单簿中删除此价格 |

</Block>

<Block>

## 退订订单簿

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/btc")
ws.send(
    '{"time" : 123456, "channel" : "futures.order_book_update", "event": "unsubscribe", "payload" : ["BTC_USDT_20230630", "100ms"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.order_book_update",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订指定交割合约的订单簿**

### 请求参数

- channel

  `futures.order_book_update`

- event

  `unsubscribe`

</Block>

<Block>

# 深度频道V2

**提供一种更快更新的获取深度信息的方法.**

</Block>

<Block>

## 维护本地深度

说明:

1. 全量深度推送（`full=true`）： 当频道推送的深度为全量深度时，需要将该深度数据完整替换本地深度，并将`深度ID`更新为消息中的字段 `u`。服务端可能会重复推送全量深度。
   1. 订阅该频道时，首次推送为全量深度。
2. 增量深度推送（`full=false`）： 增量消息中不会显示 `full` 字段，此时消息包含字段 `U`（深度起始ID）和 `u`（深度结束ID）。
   1. 如果 `U` = 本地`深度ID` + `1`，则表示深度连续更新：
      1. 将本地深度ID替换为消息中的 `u`。
      2. 若更新中的 `a` 和 `b` 不为空，分别按价格更新对应的买、卖深度数量（`level[0]` 为价格，`level[1]` 为数量）。当数量 `level[1]`= "0" 时，需移除对应档位。
   2. 若 `U` ≠ 本地`深度ID` + `1`，则深度数据不连续，需要**取消订阅该市场，并重新订阅**以获取初始化深度。
3. 订阅限制： 针对同一合约的同一深度流，一个链接只允许订阅一次，重复订阅会返回错误。失败示例：

<Example>

```json
{
  "time": 1747391482,
  "time_ms": 1747391482960,
  "id": 1,
  "conn_id": "d9db9373dc5e081e",
  "trace_id": "ee001938590e183db957bd5ba71651c0",
  "channel": "futures.obu",
  "event": "subscribe",
  "payload": [
    "ob.BTC_USDT_20230630.100"
  ],
  "error": {
    "code": 2,
    "message": "Alert sub ob.BTC_USDT_20230630.100"
  },
  "result": {
    "status": "fail"
  }
}
```

</Example>

</Block>

<Block>

## 深度频道V2订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send('{"time" : 123456, "channel" : "futures.obu",
        "event": "subscribe", "payload" : ["ob.BTC_USDT_20230630.100"]}')
print(ws.recv())
```

</Example>

<Example>

代码示例

```go
package main

import (
	"fmt"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.obu", "event": "subscribe", "payload" : ["ob.BTC_USDT_20230630.100"]}`

	err = conn.WriteMessage(websocket.TextMessage, []byte(msg))
	if err != nil {
		log.Fatal("write message error:", err)
	}

	_, message, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read message error:", err)
	}

	fmt.Println(string(message))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1747391482,
  "time_ms": 1747391482384,
  "id": 1,
  "conn_id": "d9db9373dc5e081e",
  "trace_id": "ee001938590e183db957bd5ba71651c0",
  "channel": "futures.obu",
  "event": "subscribe",
  "payload": [
    "ob.BTC_USDT_20230630.100"
  ],
  "result": {
    "status": "success"
  }
}
```

</Example>

### 请求参数

- channel

  `futures.obu`
- event

  `subscribe`
- params

  `payload`是一个包含流名称的列表. 格式为: ob.{contract}.{level}; 例如 ob.BTC_USDT_20230630.100、ob.BTC_USDT_20230630.20

  其中`level`枚举为：100、20；更新频率: 100档为100ms；20档为20ms；

</Block>

<Block>

## 深度v2订阅推送

<Example>

全量推送示例:

```json
{
	"channel": "futures.obu",
	"result": {
		"t": 1743673026995,
		"full": true,
		"s": "ob.BTC_USDT_20230630.100",
		"u": 79072179673,
		"b": [
			["83705.9", "30166"]
		],
		"a": [
			["83706", "4208"]
		]
	},
	"time_ms": 1743673026999
}
```

增量推送示例:

```json
{
	"channel": "futures.obu",
	"result": {
		"t": 1743673027017,
		"s": "ob.BTC_USDT_20230630.100",
		"U": 79072179674,
		"u": 79072179694,
		"b": [
			["83702.2", "62"],
			["83702.1", "0"],
			["83702", "0"],
			["83685.6", "120"],
			["83685", "239"]
		]
	},
	"time_ms": 1743673027020
}
```

</Example>

**深度频道V2的消息推送**

### 推送参数

- channel

  `futures.obu`
- event

  `update`
- params


| 名称                | 类型                    | 描述                                                                       |
| ------------------- | ----------------------- | -------------------------------------------------------------------------- |
| `result`            | Object                  | 自上次更新以来发生变更要价和出价                                           |
| »`t`               | Integer                 | 订单簿生成时间戳（以毫秒为单位）                                           |
| »`full`            | Boolean                 | `true` 代表全量的深度；`false` 代表增量的深度，为 `false` 时，不传输该字段 |
| »`s`               | String                  | 深度流的名称                                                               |
| »`U`               | Integer                 | 本次更新开始的订单簿更新 ID                                                |
| »`u`               | Integer                 | 本次更新结束的订单簿更新 ID                                                |
| »`b`               | `Array[OrderBookArray]` | 自上次更新以来的 bids 更新                                                 |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对,  Amount=0应当从本地深度中移除                      |
| »`a`               | `Array[OrderBookArray]` | 自上次更新以来的 asks 更新                                                 |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对,  Amount=0应当从本地深度中移除                      |

</Block>

<Block>

## 深度频道V2取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT_20230630.100"]}')
print(ws.recv())
```

</Example>

<Example>

代码示例

```go
package main

import (
	"fmt"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT_20230630.100"]}`

	err = conn.WriteMessage(websocket.TextMessage, []byte(msg))
	if err != nil {
		log.Fatal("write message error:", err)
	}

	_, message, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read message error:", err)
	}

	fmt.Println(string(message))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1743673617,
  "time_ms": 1743673617242,
  "id": 1,
  "conn_id": "7b06ff199a98ab0e",
  "trace_id": "8f86e4021a84440e502f73fde5b94918",
  "channel": "futures.obu",
  "event": "unsubscribe",
  "payload": ["ob.BTC_USDT_20230630.100"],
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消指定交割合约的深度频道V2订阅**

### 请求参数

- channel

  `futures.obu`
- event

  `unsubscribe`

</Block>

<Block>

# K 线频道

**提供一种获取 K 线信息的方式**

</Block>

<Block>

## 订阅 K 线

<strong><em>如果合约字段 `contract` 包含 `mark_` 前缀的话, 将订阅合同的标记价格 K 线数据。</em></strong>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.candlesticks", "event": "subscribe", "payload" : ["1m", "BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545445847,
  "channel": "futures.candlesticks",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

### 请求参数

- channel

  `futures.candlesticks`

- event

  `subscribe`

- params

  | 字段       | 类型   | 描述                                                                |
  | ---------- | ------ | ------------------------------------------------------------------- |
  | `interval` | String | 间隔："10s", "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "7d" |
  | `contract` | String | 交割合约名称                                                        |

</Block>

<Block>

## 推送 K 线消息

<Example>

```json
{
  "time": 1542162490,
  "channel": "futures.candlesticks",
  "event": "update",
  "error": null,
  "result": [
    {
      "t": 1545129300,
      "v": 27525555,
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USDT_20230630"
    },
    {
      "t": 1545129300,
      "v": 27525555,
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USDT_20230630"
    }
  ]
}
```

</Example>

**推送 K 线信息**

### 通知

- channel

  `futures.candlesticks`

- event

  `update`

- params

  | 字段     | 类型    | 描述         |
  | -------- | ------- | ------------ |
  | `result` | Array   | 结果         |
  | »`t`     | Integer | 时间         |
  | »`o`     | String  | 开盘价       |
  | »`c`     | String  | 收盘价       |
  | »`h`     | String  | 最高价       |
  | »`l`     | String  | 最低价       |
  | »`v`     | Integer | 成交量       |
  | »`n`     | String  | 交割合约名称 |

</Block>

<Block>

## 取消订阅

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
ws.send(
    '{"time" : 123456, "channel" : "futures.candlesticks", "event": "unsubscribe", "payload" : ["1m", "BTC_USDT_20230630"]}')
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545445847,
  "channel": "futures.candlesticks",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订指定交割合约的 K 线信息**

### 请求参数

- channel

  `futures.candlesticks`

- event

  `unsubscribe`

</Block>

<Block>

# 订单频道

**提供一种获取用户已关闭的订单的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅订单消息

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.orders",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.orders",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户订单消息**

### 请求参数

- channel

  `futures.orders`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 订单推送信息

<Example>

```json
{
  "channel": "futures.orders",
  "event": "update",
  "time": 1541505434,
  "result": [
    {
      "contract": "BTC_USDT_20230630",
      "user": "200XX",
      "create_time": 1545141817,
      "create_time_ms": 1545141817123,
      "fill_price": 4120,
      "finish_as": "filled",
      "iceberg": 0,
      "id": 93282759,
      "is_reduce_only": false,
      "status": "finished",
      "is_close": 0,
      "is_liq": 0,
      "left": 0,
      "mkfr": -0.00025,
      "price": 4120,
      "refu": 0,
      "size": 10,
      "text": "-",
      "tif": "gtc",
      "finish_time": 1545640868,
      "finish_time_ms": 1545640868123,
      "tkfr": 0.00075
    }
  ]
}
```

</Example>

**当订单被下单、更新或完成时通知用户订单信息。**

### 通知

- channel

  `futures.orders`

- event

  `update`

- params

  以下字段的具体意思，请查阅 HTTP 接口。

  | 字段              | 类型    | 描述                 |
  | ----------------- | ------- | -------------------- |
  | `result`          | Array   | 结果                 |
  | »`create_time`    | Integer | 创建时间戳，单位秒   |
  | »`create_time_ms` | Integer | 创建时间戳，单位毫秒 |
  | »`fill_price`     | Float   | 成交价格             |
  | »`finish_as`      | String  | 完成状态             |
  | »`iceberg`        | Integer | 冰山订单             |
  | »`id`             | Integer | 订单唯一标识         |
  | »`is_close`       | Integer | 是否平仓             |
  | »`is_liq`         | Integer | 是否强平             |
  | »`left`           | Integer | 订单剩余数量         |
  | »`mkfr`           | Integer | -                    |
  | »`is_reduce_only` | Bool    | 是否减仓             |
  | »`status`         | String  | 订单状态             |
  | »`tkfr`           | Integer | -                    |
  | »`price`          | Integer | 订单价格             |
  | »`refu`           | Integer | -                    |
  | »`size`           | Integer | 订单大小             |
  | »`text`           | String  | -                    |
  | »`tif`            | String  | -                    |
  | »`finish_time`    | Integer | 完成时间戳，单位秒   |
  | »`finish_time_ms` | Integer | 完成时间戳，单位毫秒 |
  | »`user`           | String  | 用户唯一标识         |
  | »`contract`       | String  | 交割订单名称         |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.orders",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.orders",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订所有交割合约的订单更新数据**

### 请求参数

- channel

  `futures.orders`

- event

  `unsubscribe`

</Block>

<Block>

# 用户交易频道

**提供一种获取用户交易数据的方式。**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅用户交易数据

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.usertrades",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.usertrades",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户交易更新数据**

### 请求参数

- channel

  `futures.usertrades`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 推送用户交易数据

<Example>

```json
{
  "time": 1543205083,
  "channel": "futures.usertrades",
  "event": "update",
  "error": null,
  "result": [
    {
      "contract": "BTC_USDT_20230630",
      "create_time": 1545140672,
      "create_time_ms": 1545140672371,
      "id": "12651269",
      "order_id": "56945246",
      "price": "113.6",
      "size": 10,
      "role": "maker"
    }
  ]
}
```

</Example>

**推送用户交易更新数据**

### 通知

- channel

  `futures.usertrades`

- event

  `update`

- params

  | 字段              | type    | 描述                   |
  | ----------------- | ------- | ---------------------- |
  | `result`          | Array   | 结果                   |
  | »`contract`       | String  | 交割合约名称           |
  | »`create_time`    | Integer | 创建时间               |
  | »`create_time_ms` | Integer | 创建时间               |
  | »`id`             | String  | 交易唯一标识           |
  | »`order_id`       | String  | 订单唯一标识           |
  | »`price`          | String  | 价格                   |
  | »`size`           | Integer | 交易大小               |
  | »`role`           | String  | 用户角色 (maker/taker) |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.usertrades",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.usertrades",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订用户交易更新数据**

### 请求参数

- channel

  `futures.usertrades`

- event

  `unsubscribe`

</Block>

<Block>

# 强平频道

**提供一种获取强平的方式。**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅清算信息

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.liquidates",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.liquidates",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户清算更新**

### 请求

- channel

  `futures.liquidates`

- event

  `subscribe`

- params

  | 参数       | type   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 清算通知

<Example>

```json
{
  "channel": "futures.liquidates",
  "event": "update",
  "time": 1541505434,
  "result": [
    {
      "entry_price": 209,
      "fill_price": 215.1,
      "left": 0,
      "leverage": 0.0,
      "liq_price": 213,
      "margin": 0.007816722941,
      "mark_price": 213,
      "order_id": 4093362,
      "order_price": 215.1,
      "size": -124,
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USDT_20230630",
      "user": "1040"
    }
  ]
}
```

</Example>

**通知清算更新消息**

### 通知

- channel

  `futures.liquidates`

- event

  `update`

- params

  | 字段           | 类型    | 描述           |
  | -------------- | ------- | -------------- |
  | `result`       | Array   | 结果           |
  | »`entry_price` | Float   | 平均入场价格   |
  | »`fill_price`  | Float   | 平均执行价格   |
  | »`leverage`    | Float   | 杠杆           |
  | »`liq_price`   | Float   | 清算价格       |
  | »`margin`      | Float   | 保证金         |
  | »`mark_price`  | Float   | 标记价格       |
  | »`order_id`    | Integer | 订单唯一标识   |
  | »`order_price` | Float   | 佣金价格       |
  | »`left`        | Integer | 订单未成交数量 |
  | »`size`        | Integer | 持仓的原始大小 |
  | »`time`        | Integer | 时间           |
  | »`time_ms`     | Integer | 时间，毫秒     |
  | »`user`        | String  | 用户唯一标识   |
  | »`contract`    | String  | 交割合约名称   |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.liquidates",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.liquidates",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订用户清算更新**

### 请求

- channel

  `futures.liquidates`

- event

  `unsubscribe`

</Block>

<Block>

# 自动减仓频道

::: warning
需要认证
:::

</Block>

<Block>

## 订阅自动减仓更新

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.auto_deleverages",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.auto_deleverages",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户自动检查更新数据**

### 请求

- channel

  `futures.auto_deleverages`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 自动减仓通知

<Example>

```json
{
  "channel": "futures.auto_deleverages",
  "event": "update",
  "time": 1541505434,
  "result": [
    {
      "entry_price": 209,
      "fill_price": 215.1,
      "position_size": 10,
      "trade_size": 10,
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USDT_20230630",
      "user": "1040"
    }
  ]
}
```

</Example>

**通知自动检查更新信息**

### 通知

- channel

  `futures.auto_deleverages`

- event

  `update`

- params

  | 字段             | 类型    | 描述           |
  | ---------------- | ------- | -------------- |
  | `result`         | Array   | 结果           |
  | »`entry_price`   | Float   | 入场价格       |
  | »`fill_price`    | Float   | 执行价格       |
  | »`position_size` | Integer | 持仓大小       |
  | »`trade_size`    | Integer | 交易大小       |
  | »`time`          | Integer | 时间，单位秒   |
  | »`time_ms`       | Integer | 时间，单位毫秒 |
  | »`user`          | String  | 用户唯一标识   |
  | »`contract`      | String  | 交割合约名称   |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.auto_deleverages",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.auto_deleverages",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订自动减仓更新**

### 请求

- channel

  `futures.auto_deleverages`

- event

  `unsubscribe`

</Block>

<Block>

# 平仓信息频道

**提供接收用户平仓的方式**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅平仓更新数据

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.position_closes",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.position_closes",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户平仓更新信息**

### 请求

- channel

  `futures.position_closes`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 平仓通知

<Example>

```json
{
  "channel": "futures.position_closes",
  "event": "update",
  "time": 1541505434,
  "result": [
    {
      "contract": "BTC_USDT_20230630",
      "pnl": -0.000624354791,
      "side": "long",
      "text": "web",
      "time": 1547198562,
      "time_ms": 1547198562123,
      "user": "20011"
    }
  ]
}
```

</Example>

**通知平仓更新**

### Notify

- channel

  `futures.position_closes`

- event

  `update`

- params

  | 字段        | 类型    | 描述             |
  | ----------- | ------- | ---------------- |
  | `result`    | Array   | 结果             |
  | »`contract` | String  | 交割合约名称     |
  | »`pnl`      | Number  | 盈亏金额         |
  | »`side`     | String  | 方向，多头或空头 |
  | »`text`     | String  | 消息内容         |
  | »`time`     | Integer | 时间，单位秒     |
  | »`time_ms`  | Integer | 时间，单位毫秒   |
  | »`user`     | String  | 用户唯一标识     |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.position_closes",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.position_closes",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅仓位平仓更新**

### 请求

- channel

  `futures.position_closes`

- event

  `unsubscribe`

</Block>

<Block>

# 余额频道

**获取用户余额信息**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅余额

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.balances",
    "event": "subscribe",
    "payload": ["20011"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.balances",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户余额更新**

### 请求

- channel

  `futures.balances`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必填 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |

</Block>

<Block>

## 余额通知

<Example>

```json
{
  "channel": "futures.balances",
  "event": "update",
  "time": 1541505434,
  "result": [
    {
      "balance": 9.998739899488,
      "change": -0.000002074115,
      "text": "BTC_USDT_20230630:3914424",
      "time": 1547199246,
      "time_ms": 1547199246123,
      "type": "fee",
      "user": "20011"
    }
  ]
}
```

</Example>

**通知余额更新**

### 通知

- channel

  `futures.balances`

- event

  `update`

- params

  | 字段       | 类型    | 描述           |
  | ---------- | ------- | -------------- |
  | `result`   | Array   | 结果           |
  | »`balance` | Number  | 变更后的余额   |
  | »`change`  | Number  | 变动的金额     |
  | »`text`    | String  | 消息内容       |
  | »`time`    | Integer | 时间，单位秒   |
  | »`time_ms` | Integer | 时间，单位毫秒 |
  | »`type`    | String  | 类型           |
  | »`user`    | String  | 用户唯一标识   |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.balances",
    "event": "unsubscribe",
    "payload": ["20011"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.balances",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

</Block>

<Block>

# 减少风险限额频道

**提供一种接收用户减少风险限额信息的方式。**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅减少风险限额

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.reduce_risk_limits",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.reduce_risk_limits",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户减少风险限额的更新数据 **

### 请求

- channel

  `futures.reduce_risk_limits`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 减少风险限额通知

<Example>

```json
{
  "time": 1551858330,
  "channel": "futures.reduce_risk_limits",
  "event": "update",
  "error": null,
  "result": [
    {
      "cancel_orders": 0,
      "contract": "ETH_USD",
      "leverage_max": 10,
      "liq_price": 136.53,
      "maintenance_rate": 0.09,
      "risk_limit": 450,
      "time": 1551858330,
      "time_ms": 1551858330123,
      "user": "20011"
    }
  ]
}
```

</Example>

**通知减少风险限额更新。**

### 通知

- channel

  `futures.reduce_risk_limits`

- event

  `update`

- params

  | 字段                | 类型   | 描述           |
  | ------------------- | ------ | -------------- |
  | `result`            | Array  | 结果           |
  | »`cancel_orders`    | Number | 取消订单数     |
  | »`contract`         | String | 合约名称       |
  | »`leverage_max`     | Number | 最大杠杆       |
  | »`liq_price`        | Number | 清算价格       |
  | »`maintenance_rate` | Number | 维持保证金率   |
  | »`risk_limit`       | Number | 风险限额       |
  | »`time`             | Number | 时间，单位秒   |
  | »`time_ms`          | Number | 时间，单位毫秒 |
  | »`user`             | String | 用户唯一标识   |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.reduce_risk_limits",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

**Unsubscribe reduce risk limits update.**

### 请求

- channel

  `futures.reduce_risk_limits`

- event

  `unsubscribe`

</Block>

<Block>

# 仓位频道

**提供接受用户仓位信息的方式。**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅仓位信息

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.positions",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.positions",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户的仓位更新数据 **

### 请求

- channel

  `futures.positions`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必填 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 推送仓位更新数据

<Example>

```json
{
  "time": 1588212926,
  "channel": "futures.positions",
  "event": "update",
  "error": null,
  "result": [
    {
      "contract": "BTC_USDT_20230630",
      "entry_price": 5999,
      "history_pnl": 9.99872821972,
      "history_point": -0.02954299895,
      "last_close_pnl": -0.00011406187,
      "leverage": 10,
      "leverage_max": 100,
      "liq_price": 5508.28,
      "maintenance_rate": 0.005,
      "margin": 0.001112608124,
      "realised_pnl": -0.000072631078,
      "realised_point": 0,
      "risk_limit": 100,
      "size": 70,
      "time": 1588212925,
      "time_ms": 1588212925123,
      "user": "10003"
    }
  ]
}
```

</Example>

**通知仓位更新**

### 通知

- channel

  `futures.positions`

- event

  `update`

- params

  | 字段                | 类型   | 描述               |
  | ------------------- | ------ | ------------------ |
  | `result`            | Array  | 结果               |
  | »`contract`         | String | 交割合约名称       |
  | »`entry_price`      | Number | 入场价格           |
  | »`history_pnl`      | Number | 历史盈亏           |
  | »`history_point`    | Number | 历史点数           |
  | »`last_close_pnl`   | Number | 最后平仓盈亏       |
  | »`leverage`         | Number | 杠杆               |
  | »`leverage_max`     | Number | 最大支持的杠杆     |
  | »`liq_price`        | Number | 强平价格           |
  | »`maintenance_rate` | Number | 维持保证金率       |
  | »`margin`           | Number | 保证金             |
  | »`realised_pnl`     | Number | 已实现盈亏         |
  | »`realised_point`   | Number | 已实现点数         |
  | »`risk_limit`       | Number | 风险限额           |
  | »`size`             | Number | 合约大小           |
  | »`time`             | Number | 更新时间，单位秒   |
  | »`time_ms`          | Number | 更新时间，单位毫秒 |
  | »`user`             | String | 用户唯一标识       |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.positions",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.positions",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订仓位更新数据**

### 请求参数

- channel

  `futures.positions`

- event

  `unsubscribe`

</Block>

<Block>

# 自动下单频道

**提供一种获取用户自动下单数据的方式。**

::: warning
需要认证
:::

</Block>

<Block>

## 订阅自动下单

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.autoorders",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.autoorders",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅自动下单更新数据**

### 请求参数

- channel

  `futures.autoorders`

- event

  `subscribe`

- params

  | 参数       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `user id ` | String | 否   | 用户唯一标识 |
  | `contract` | String | 是   | 交割合约名称。!all——订阅所有合约 |

</Block>

<Block>

## 自动下单推送通知

<Example>

```json
{
  "time": 1596798126,
  "channel": "futures.autoorders",
  "event": "update",
  "error": null,
  "result": [
    {
      "user": 1543255,
      "trigger": {
        "strategy_type": 0,
        "price_type": 0,
        "price": "10000",
        "rule": 2,
        "expiration": 86400
      },
      "initial": {
        "contract": "BTC_USDT_20230630",
        "size": 10,
        "price": "10000",
        "tif": "gtc",
        "text": "web",
        "iceberg": 0,
        "is_close": false,
        "is_reduce_only": false
      },
      "id": 9256,
      "trade_id": 0,
      "status": "open",
      "reason": "",
      "create_time": 1596798126,
      "name": "price_autoorders",
      "is_stop_order": false,
      "stop_trigger": {
        "rule": 0,
        "trigger_price": "",
        "order_price": ""
      }
    }
  ]
}
```

</Example>

**通知自动下单更新数据**

### 通知

- channel

  `futures.autoorders`

- event

  `update`

- params

  | 字段             | 类型    | 描述               |
  | ---------------- | ------- | ------------------ |
  | `result`         | Array   | 结果               |
  | »`user`          | Number  | 用户唯一标识       |
  | »`trigger`       | Object  | 触发器             |
  | »`initial`       | Object  | 起势               |
  | »`id`            | Number  | 自动订单的唯一标识 |
  | »`trade_id`      | Number  | 交易唯一标识       |
  | »`status`        | String  | 订单状态           |
  | »`reason`        | String  | 变更原因           |
  | »`create_time`   | Number  | 创建时间           |
  | »`name`          | String  | 名称               |
  | »`is_stop_order` | boolean | 是否停止订单       |
  | »`stop_trigger`  | Object  | 停止触发器         |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/delivery/usdt")
req = {
    "time": 123456,
    "channel": "futures.autoorders",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT_20230630"],
    "auth": {
        "method": "api_key",
        "KEY": "xxxx",
        "SIGN": "xxxx"
    }}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "channel": "futures.autoorders",
  "event": "unsubscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

**退订自动订单数据**

### 请求参数

- channel

  `futures.autoorders`

- event

  `unsubscribe`

</Block>
