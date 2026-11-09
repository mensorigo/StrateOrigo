---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---
<Block>

# Gate Futures WebSocket v4.0.0

Gate 提供简单而强大的 WebSocket API，将 Gate BTC、USDT 和 USD1 结算的永续合约交易状态集成到您的业务或应用程序中。

我们在 `Python` 和 `Golang` 中有语言绑定，将来还会有更多！您可以在右侧的深色区域中查看代码示例，并且可以通过右上角的选项卡切换示例的编程语言

## 服务地址

我们提供 BTC、USDT 和 USD1 结算永续合约交易服务器地址，您可以根据自己的情况选择其中之一。

### BTC Contract

地址列表:

- 线上交易: `wss://fx-ws.gateio.ws/v4/ws/btc`
- 模拟盘交易: `wss://fx-ws-testnet.gateio.ws/v4/ws/btc`

### USDT Contract

地址列表:

- 线上交易: `wss://fx-ws.gateio.ws/v4/ws/usdt`
- 线上SBE: `wss://fx-ws.gateio.ws/v4/ws/usdt/sbe`
- 模拟盘交易: `wss://ws-testnet.gate.com/v4/ws/futures/usdt`

### USD1 Contract

地址列表:

- 线上交易: `wss://fx-ws.gateio.ws/v4/ws/usd1`
- 合约示例: `BTC_USD1`

::: tip
建议使用SBE以获取更快的行情和更小的带宽成本
:::

::: warning
如果你使用老的服务地址(`wss://fx-ws.gateio.ws/v4/ws`
或 `wss://fx-ws-testnet.gateio.ws/v4/ws`), 将默认是 BTC 结算的 websocket 服务.
:::

</Block>

<Block>

## 变更日志

<Example>

WebSocket 应用示例

```python
# !/usr/bin/env python
# coding: utf-8

import hashlib
import hmac
import json
import logging
import time
import threading

from websocket import WebSocketApp

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

event = threading.Event()

class GateWebSocketApp(WebSocketApp):

  def __init__(self, url, api_key, api_secret, **kwargs):
    super(GateWebSocketApp, self).__init__(url, **kwargs)
    self._api_key = api_key
    self._api_secret = api_secret

  def _send_ping(self):
    while not event.wait(10):
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
  ws.subscribe("futures.tickers", ['BTC_USDT'], False)

# custom header
custom_headers = {
    "X-Gate-Size-Decimal": "1"
}

if __name__ == "__main__":
  logging.basicConfig(format="%(asctime)s - %(message)s", level=logging.DEBUG)
  app = GateWebSocketApp("wss://fx-ws.gateio.ws/v4/ws/usdt",
                         "YOUR_API_KEY",
                         "YOUR_API_SECRET",
                         on_open=on_open,
                         on_message=on_message,
                         header=custom_headers)
  app.run_forever(ping_interval=5)
```

</Example>

<Example>

WebSocket 应用示例

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
	"net/http"
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
	u := url.URL{Scheme: "wss", Host: "fx-ws.gateio.ws", Path: "/v4/ws/usdt"}
	websocket.DefaultDialer.TLSClientConfig = &tls.Config{RootCAs: nil, InsecureSkipVerify: true}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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
	orderBookMsg := NewMsg("futures.order_book", "subscribe", t, []string{"BTC_USDT"})
	err = orderBookMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe positions
	positionsMsg := NewMsg("futures.positions", "subscribe", t, []string{"USERID", "BTC_USDT"})
	positionsMsg.sign()
	err = positionsMsg.send(c)
	if err != nil {
		panic(err)
	}

	select {}
}
```

</Example>

2026-10-22

- 补充 `futures.adl_warning` 的功能说明、三种市场级风险状态的含义，以及其与单个用户仓位和实际 ADL 事件的关系。

2026-10-16

- 新增订单频道 `futures.orders_v2`，支持实盘和模拟盘，仅限 USDT 结算合约 API WebSocket 的 UTA3.0 用户。该 JSON 频道推送订单创建、成交、改单和撤单事件，包含订单管理系统（OMS）和撮合引擎（ME）的毫秒时间戳，不含清算信息。

2026-10-14

- 新增公开频道 `futures.rpi_order_book`，`futures.rpi_obu` 推送 RPI 深度全量快照

2026-08-25

- 新增市场级 ADL 风险预警公共频道 `futures.adl_warning`

2026-08-20

- 新增频道 `futures.contract_info`，推送合约精度信息（张乘数、下单价格精度、标记价格精度）与风险限额档位变更
- 频道 `futures.public_liquidates` 调整为聚合推送：同一合约在一个聚合周期内的多笔强平订单将合并为数组批量推送，不再丢弃周期内的其他强平订单


2026-08-19

- 新增 USD1 结算永续合约 WebSocket 支持，线上地址为 `wss://fx-ws.gateio.ws/v4/ws/usd1`，`settle` 值使用小写 `usd1`

2026-04-14

- 部分频道支持SBE数据推送: `futures.trades`、`futures.obu`、`futures.book_ticker`、`futures.tickers`、`futures.candlesticks`、`futures.order_book`、`futures.order_book_update`、`futures.usertrades`、`futures.positions`、`futures.orders`。
- 具体的使用查看SBE 数据推送章节

2026-03-30

- `futures.order_place` 下单请求：`iceberg` 字段说明修正为类型 `string`（原文档为 `int64`）、可选（原文档为必填）

2026-02-09

- 模拟盘部分频道支持SBE数据推送
- 正式环境实装另行同步

2026-02-04

- `futures.obu` 模拟盘新增立即的首次快照推送，该次快照推送将会在订阅请求的响应之前推送。此行为与之前快照在订阅请求的响应之后推送不同，请注意该行为变更。
- 正式环境实装另行同步

2026-01-07

- `futures.orders` 新增字段 `market_order_slip_ratio` (市价单的预设滑点比例)
- `futures.order_place` `futures.order_batch_place` 入参新增字段`market_order_slip_ratio` (允许自定义市价单的最大滑点)

2025-12-09

- 合约的张数支持小数，所有的推送的张数、成交量等均改为字符串（可能是小数）。
- 如何对接websocket的小数支持：

  - 在请求链接websocket时，加入：`"X-Gate-Size-Decimal": "1"` 的header即可，推送的张数、成交量等将会是字符串（可能是小数）。
  - 如果在链接websocket时，不加入：`"X-Gate-Size-Decimal": "1"` 的header，推送将保持原始的字段类型（整形）。
  - `请尽快切换到字符串的推送支持，后续整形的推送将会下线(下线时间会另行通知)。`
  - 如果出现小数的size，用户如果还是在使用整形的推送，那么推送出来的size将会是向下取整，例如`1.1`、`1.5`、`1.7`的size推送出去都将会是`1`。
- 将影响到以下的推送频道，和对应的字段。


  | 频道                      | 字段                                              |
  | ------------------------- | ------------------------------------------------- |
  | futures.trades            | size                                              |
  | futures.tickers           | total\_size                                       |
  | futures.book_ticker       | A B                                               |
  | futures.order_book_update | a.s、b.s                                          |
  | futures.order_book        | a.s、b.s                                          |
  | futures.obu               | size 会有小数                                     |
  | futures.candlesticks      | v                                                 |
  | futures.liquidates        | left、size、 order_size                           |
  | futures.public_liquidates | size                                              |
  | futures.contract_stats    | long\_liq\_size、short\_liq\_size、open\_interest |
  | futures.orders            | iceberg、left、size                               |
  | futures.usertrades        | size                                              |
  | futures.auto_deleverages  | position\_size、trade\_size                       |
  | futures.positions         | size                                              |
  | futures.autoorders        | position\_size、 trade\_size                      |
- 将影响到以下的API请求频道，主要涉及张数，成交量等字段。

  | 频道                      | 字段        |
    | ------------------------- |-----------|
  |futures.order_place           | size、left |
  | futures.order_batch_place          | size      |
  | futures.order_cancel       | size、left     |
  | futures.order_cancel_cp | size、left   |
  | futures.order_amend       | size、left     |
  | futures.order_list | size、left   |
  | futures.order_status | size、left   |


2025-09-25

- 新增 `仓位 Adl 排名频道` 文档

2025-05-22

- 新增深度频道V2文档说明
- `futures.order_book_update` 新增字段 `full`

2025-04-25

- 账户交易API新增通道 `futures.order_cancel_ids`
- `futures.order_book` 和 `futures.order_book_update` 新增深度档位字段 `l`

2025-04-18

- 补充文档代码示例

<!-- 2025-04-14

- 账户交易API新增通道 `futures.order_cancel_ids`

2025-04-10

- Websocket API 概述新增 `timeUnit=microsecond` 的说明 -->

2025-03-24

- 修复了深度频道部分文档的错误描述
- 修复了订单频道部分文档的错误描述

2025-03-21

- 更新了频道 `futures.orders` 文档, 新增了 `update_id`, `update_time`, `biz_info`,	`stop_profit_price` 和 `stop_loss_price` 等字段的说明

2025-03-12

- 合约统计信息频道增加 `contract` 字段
- 更新账户交易 API，新增了 `x-gate-exptime` 字段
- 修复了账户交易 API 部分文档描述性错误

2025-02-19

- 新增频道 `futures.public_liquidates` 用于推送合约强平订单的快照信息

2025-02-10

- 更新账户交易 API，新增了 `x_in_time`, `x_out_time`, `conn_trace_id`, `trace_id` 字段
- `futures.order_place`, `futures.order_batch_place`, `futures.order_cancel`, `futures.order_cancel_cp` 和 `futures.order_amend` 新增了 `x_gate_ratelimit_requests_remain`, `x_gate_ratelimit_limit` 和 `x_gat_ratelimit_reset_timestamp` 字段

2024-11-18

- 在频道 `futures.order_book_update` 移除 `10` 档位 `1000ms` 推送间隔支持

2023-09-21

- 在频道`futures.trades`推送参数中新增`is_internal`字段

2023-08-18

- 添加 WebSocket API 操作
- WebSocket API 允许通过 WebSocket 连接创建、取消、修改、查询订单。

2023-07-07

- 在频道`futures.order_book_update`中添加新的间隔“20ms”，请注意，`20ms` 的间隔仅支持 `20` 档位

2023-06-20

- 在频道 `futures.positions` 增加`update_id` 字段

2022-12-22

- 在频道 `futures.autoorders` 初始结构中添加新字段 `auto_size`，将字段详细信息添加到 http api

2022-11-22

- 在通用的返回结果中添加新字段“time_ms”，以表示创建消息的时间

2022-08-11

- 在频道`futures.autoorders`通知中添加新字段`text`
- 在频道`futures.tickers`通知中添加新字段`low_24h`和`high_24h`

2022-04-15

- 在频道`futures.balances`通知中添加新字段 `currency`

2021-03-31

- 在频道`futures.book_ticker`和`futures.order_book`推送中添加毫秒字段`t`

2021-03-10

- 添加新的订单簿频道 `futures.book_ticker` 以实时推送最佳卖价/买价
- 添加新的订单簿频道 `futures.order_book_update` 以与用户推送订单簿更改
  指定更新频率
- 添加本地订单簿维护文档

2021-03-01

- 在通用的返回结果中添加以`_ms`结尾的新毫秒精度时间戳
- 在`futures.book` `all`通知中添加新字段`id`

2020-8-08

- 添加完整代码 demo(golang, python)

2020-8-07

- 添加`futures.autoorders`频道

2020-7-07

- 添加`futures.order_book`频道

2020-4-30

- 添加`futures.position`频道

2019-11-06

- 新增 USDT 结算永续合约的 websocket 推送
- 为`futures.tickers`添加`volume_24h_base`字段、`volume_24h_settle`字段、`volume_24h_quote`字段
- 删除旧服务器地址（`wss://fx-ws.gateio.ws/v4/ws` 或 `wss://fx-ws-testnet.gateio.ws/v4/ws`）

::: tip
如果您使用旧的服务器地址（`wss://fx-ws.gateio.ws/v4/ws` 或 `wss://fx-ws-testnet.gateio.ws/v4/ws`），我们
将为您使用 BTC 结算永续合约的 websocket 推送
:::

2019-10-22

- 添加应用层 ping/pong 消息

2019-04-30

- 添加`index`和`mark` `futures.candlesticks` 订阅
- 为`futures.tickers`添加`funding_rate_indicative`字段
- 为`futures.orders`添加 is_reduce_only 和状态字段

2019-02-13

- 更改 webSocket 基本 url
- 为`futures.tickers`添加`volume_24h_usd`字段和`volume_24h_btc`字段

2019-01-11

- 添加`futures.position_closes` 和 `futures.balance` 订阅
- 删除频道 `futures.auto_deleverages` 和`futures.liquidates`的 finish_time 字段
- 为频道 `futures.auto_deleverages` 和`futures.liquidates` 添加 `time`字段

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用 订阅频道/channel（例如`ticker`、`order_book`等）都支持一些不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据通知。
2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据通知。
3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）。
4. **`all`**

如果有新订阅的数据（所有数据）可用，服务器将向客户端发送通知。

### 请求

每个请求都遵循通用格式，其中包含`time`、`channel`、`event`和`payload`。


| 名称      | 类型    | 必选 | 描述                                                          |
| --------- | ------- | ---- | ------------------------------------------------------------- |
| `id`      | Integer | 否   | 可选的请求 ID，将由服务器发回，以帮助您识别服务器响应哪个请求 |
| `time`    | Integer | 是   | 请求时间                                                      |
| `channel` | String  | 是   | 请求 subscribe/unsubscribe 频道                               |
| `auth`    | String  | 否   | 请求身份验证信息，请参阅身份验证部分了解详细信息              |
| `event`   | String  | 是   | 请求`event` (subscribe/unsubscribe/update/all/api)            |
| `payload` | Array   | 是   | 请求详细参数                                                  |

### 响应

与请求类似，响应遵循以下通用格式，其中包含： `time`, `channel`, `event`
, `error` 和 `result`.


| 名称      | 类型    | 必选 | 描述                                                                                                     |
| --------- | ------- | ---- | -------------------------------------------------------------------------------------------------------- |
| `time`    | Integer | 是   | 响应时间                                                                                                 |
| `time_ms` | Integer | 是   | 毫秒响应时间                                                                                             |
| `channel` | String  | 是   | 响应频道                                                                                                 |
| `event`   | String  | 是   | 响应频道事件 (update/all)                                                                                |
| `error`   | Object  | 是   | 响应错误                                                                                                 |
| `result`  | Any     | 是   | 返回来自服务端的新数据通知 或 对客户端请求的响应。如果有错误返回则`error` 不为空，没有错误则此字段为空。 |

注意：如果它是服务端发起的数据更新通知 那么 `result` 的类型是基于 channel 的，不同 channel 的 `result` 类型有所不同。

但如果是对客户端订阅请求的响应，那么 `result` 为固定的 `{"status": "success"}`。
验证订阅请求是否成功，您只需要检查 `error` 字段是否为空即可，不需要再解析 `result` 字段。

<!-- 如果需要得到微秒级的响应时间，请在 URL 里添加参数 `timeUnit=microsecond`，之后服务返回的数据里会新增 `time_us` 字段，该字段返回精确到微秒的时间戳。
`账户交易 API` 模块下的频道不支持此功能，因为已有字段 `x_out_time` 返回微秒级时间戳。

URL 示例：`wss://fx-ws.gateio.ws/v4/ws/usdt?timeUnit=microsecond`。 -->

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

如果频道是私有的，则请求体需要携带认证信息，
例如`futures.usertrades`

WebSocket 认证使用与 HTTP API 相同的签名计算方法，但具有
以下差异：

1. 签名字符串拼接方式：`channel=<channel>&event=<event>&time=<time>`,
   其中`<channel>`、`<event>`、`<time>`是对应的请求信息
2. 身份验证信息在请求正文中的`auth`字段中发送。

<Example>

代码示例

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
    'channel': 'futures.orders',
    'event': 'subscribe',
    'payload': ["20011", "BTC_USDT"]
}
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
print(json.dumps(request))
```

</Example>

<Example>

代码示例

```go
package main

import (
	"crypto/hmac"
	"crypto/sha512"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"time"
)

func genSign(channel, event string, timestamp int64) map[string]string {
	apiKey := "YOUR_API_KEY"
	apiSecret := "YOUR_API_SECRET"

	s := fmt.Sprintf("channel=%s&event=%s&time=%d", channel, event, timestamp)
	h := hmac.New(sha512.New, []byte(apiSecret))
	h.Write([]byte(s))
	sign := hex.EncodeToString(h.Sum(nil))

	return map[string]string{
		"method": "api_key",
		"KEY":    apiKey,
		"SIGN":   sign,
	}
}

func main() {
	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"id":      time.Now().UnixNano() / 1e3,
		"time":    timestamp,
		"channel": "futures.orders",
		"event":   "subscribe",
		"payload": []string{"20011", "BTC_USDT"},
	}
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	jsonBytes, _ := json.Marshal(request)
	fmt.Println(string(jsonBytes))
}
```

</Example>

您可以登录账户获取永续合约账户的 api_key 和 secret。


| 名称     | 类型   | 描述               |
| -------- | ------ | ------------------ |
| `method` | String | 验证方式:`api_key` |
| `KEY`    | String | apiKey 的值        |
| `SIGN`   | String | 签名结果           |

</Block>

<Block>

# SBE 数据推送

</Block>

<Block>

## 对接SBE

- 使用地址,在现有的地址后添加/sbe：
  - prod: `wss://fx-ws.gateio.ws/v4/ws/usdt/sbe`
  - testnet: `wss://ws-testnet.gate.com/v4/ws/futures/usdt/sbe`
- `schema`地址：
  - `prod`: [gate_fex_ws_prod_latest.xml](https://github.com/gate/gatews/blob/master/sbe/schemas/prod/gate_fex_ws_latest.xml)
  - `testnet`: [gate_fex_ws_testnet_latest.xml](https://github.com/gate/gatews/blob/master/sbe/schemas/testnet/gate_fex_ws_testnet_latest.xml)
- 如果需要指定`sbe_schema_id`，则通过`query`的形式传入`sbe_schema_id`的参数，例如：`wss://fx-ws.gateio.ws/v4/ws/usdt/sbe?sbe_schema_id=1`
  - 目前支持的`sbe_schema_id`为`0`和`1`；`sbe_schema_id`为`0`用于客户端测试`sbe schema`不兼容升级的逻辑
  - 不传入`sbe_schema_id`则默认使用最新的schema版本
  - 传入不合法的`sbe_schema_id`在连接之后会返回系统通知，并将`sbe_schema_id`调整为最新的schema版本
  - 传入旧版本的`sbe_schema_id`在连接之后会返回系统通知，提醒更新新版本的`SBE` `schema`，依旧使用客户端指定的旧版本`schema`

<Example>

无效的sbe_schema_id的系统通知

```json
{
  "time": 1770600979,
  "time_ms": 1770600979609,
  "channel": "futures.system",
  "event": "update",
  "result": {
    "type": "invalid_sbe_schema_id",
    "msg": "Your sbe_schema_id '011' does not exist, it has been adjusted to the default sbe_schema_id '1'."
  }
}
```

过时的sbe_schema_id的系统通知

```json
{
  "time": 1770601096,
  "time_ms": 1770601096665,
  "channel": "futures.system",
  "event": "update",
  "result": {
    "type": "outdated_sbe_schema_id",
    "msg": "Your sbe_schema_id '0' is outdated, please upgrade to the latest version '1'."
  }
}
```

</Example>
    
</Block>

<Block>

## SBE使用说明

- 使用`JSON`进行请求和首次响应；使用`SBE`作为数据推送；
- 同一条连接上同时存在`JSON`和`SBE`的消息，请使用opcode来区分数据：`opcode为1代表JSON`，`opcode为2代表SBE`。
- SBE 的解码：
- **MessageHeader**：每条 SBE 二进制帧均为「MessageHeader + 消息体」。Header 中包含 `blockLength`、`templateId`、**`schemaId`**、`version`，解码时**必须先读 Header**，再根据 `schemaId` 和 `templateId` 选择对应 Schema 与消息类型解码消息体。
- **解码流程建议**：
    1. 读取 MessageHeader（固定长度），得到 `schemaId`、`templateId`、`blockLength`、`version`。
    2. 根据 `schemaId` 选择解码器：`0` → 使用旧版本进行解码；`1` → 使用新版本进行解码。
    3. 根据 `templateId` 确定具体消息类型（如 PublicTrade、OrderBook、Bbo 等），再按该 Schema 的布局解码消息体。
- 使用 SBE 时，**仅可订阅以下频道**，其余频道不支持 SBE 推送。后续将扩展到其余频道。
  - 订阅不支持SBE的频道时，将返回订阅失败的消息

| 通道名                                 | 说明             |
|-------------------------------------|----------------|
| `futures.trades`                    | 公共成交           |
| `futures.order_book`                | 订单簿（深度）        |
| `futures.rpi_order_book`            | RPI 订单簿（深度）    |
| `futures.order_book_update`         | 订单簿增量更新        |
| `futures.book_ticker`               | 最优买卖（BBO）      |
| `futures.obu`                       | 订单簿增量（OBU）     |
| `futures.rpi_obu`                   | RPI 订单簿增量（OBU） |
| `futures.candlesticks`              | K 线            |
| `futures.tickers`                   | 行情             |
| `futures.usertrades`                | 用户成交           |
| `futures.positions`                 | 持仓             |
| `futures.orders`                    | 订单             |


<Example>

不支持sbe将返回订阅失败：

```json
{
  "time": 1770603321,
  "time_ms": 1770603321767,
  "conn_id": "57a8765578ea837e",
  "trace_id": "3c75ba05569b3b292a2f36cfdd90d868",
  "channel": "futures.autoorders",
  "event": "subscribe",
  "payload": [
    "15760812",
    "!all"
  ],
  "error": {
    "code": 2,
    "message": "channel futures.autoorders does not support SBE"
  },
  "result": {
    "status": "fail"
  }
}
```

</Example>


</Block>

<Block>

# System API

**提供系统状态检查，如 ping/pong.**

</Block>

<Block>

## Ping/Pong

**检查服务器/客户端连接.**

**Gate websocket 使用协议层 ping/pong 消息。服务器会发起 ping 操作。如果客户端没有回复，客户端将被断开。**

[websocket rfc 协议](https://tools.ietf.org/html/rfc6455)

**如果想主动检测连接状态，可以发送应用层 ping 消息，并接收 pong 消息。**

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.ping"}')
print(ws.recv())
```

</Example>

<Example>

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	ping := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.ping",
	}

	msg, err := json.Marshal(ping)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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

`futures.ping`操作返回 JSON 结构如下：

```json
{
  "time": 1545404023,
  "time_ms": 1545404023123,
  "channel": "futures.pong",
  "event": "",
  "result": null
}
```

</Example>

### 请求参数

- 频道

  `futures.ping`

</Block>

<Block>

## 服务升级通知

服务在即将关闭进行升级时，会向当前连接主动推送一条系统通知，客户端收到后应尽快重连。

**频道（Channel）：** `futures.system`

**事件（Event）：** `update`

当 `channel` 为 `futures.system`、`event` 为 `update` 且 `result.type` 为 `upgrade` 时，客户端可将该消息识别为服务升级通知。

**`result` 格式（SystemNotifyDTO）：**

| 字段   | 类型   | 说明                   |
| ------ | ------ | ---------------------- |
| `type` | String | 通知类型，如 `upgrade` |
| `msg`  | String | 提示文案               |
| `data` | Object | 可选，扩展数据         |

**示例（服务升级）：**

```json
{
  "time": 1782378978,
  "time_ms": 1782378978107,
  "channel": "futures.system",
  "event": "update",
  "result": {
    "type": "upgrade",
    "msg": "The connection will soon be closed for a service upgrade. Please reconnect."
  }
}
```

</Block>

<Block>

# ticker 频道

**`ticker`是合约状态的高级概述。它向你展示了最高的，
最低的、最后的交易价格。它还包括每日交易量和价格等信息**

</Block>

<Block>

## 订阅操作

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.tickers","event": "subscribe", "payload" : ["BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.tickers","event":"subscribe","payload":["BTC_USDT"]}`

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

上面的订阅请求返回 JSON 结构如下：

```json
{
  "time": 1545404023,
  "time_ms": 1545404023123,
  "channel": "futures.tickers",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅永续合约 24hr 价格变动情况.**

### 请求参数

- channel

  `futures.tickers`
- event

  `subscribe`
- params


| 名称      | 类型  | 必选 | 描述     |
| --------- | ----- | ---- | -------- |
| `payload` | Array | 是   | 合约列表 |

</Block>

<Block>

## ticker 推送

<Example>

```json
{
  "time": 1541659086,
  "time_ms": 1541659086123,
  "channel": "futures.tickers",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT",
      "last": "118.4",
      "change_percentage": "0.77",
      "change_from": "24h",
      "change_price": "0.91",
      "funding_rate": "-0.000114",
      "funding_rate_indicative": "0.01875",
      "mark_price": "118.35",
      "index_price": "118.36",
      "total_size": "73648.5",
      "volume_24h": "745487577",
      "volume_24h_btc": "117",
      "volume_24h_usd": "419950",
      "quanto_base_rate": "",
      "volume_24h_quote": "1665006",
      "volume_24h_settle": "178",
      "volume_24h_base": "5526",
      "low_24h": "99.2",
      "high_24h": "132.5",
      "price_type": "last",
      "t": 1541659086123
    }
  ]
}
```

</Example>

**永续合约 24hr 价格变动情况推送**

### 推送参数

- channel

  `futures.tickers`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称                      | 类型   | 描述                                                                                                  |
| ------------------------- | ------ | ----------------------------------------------------------------------------------------------------- |
| `contract`                | String | 合约名称                                                                                              |
| `last`                    | String | 最新成交价                                                                                            |
| `change_percentage`       | String | 涨跌幅                                                                                                |
| `funding_rate`            | String | 资金费率                                                                                              |
| `funding_rate_indicative` | String | 下一周期预测资金费率（已弃用，改用funding_rate）                                                      |
| `mark_price`              | String | 标记价格                                                                                              |
| `index_price`             | String | 指数价格                                                                                              |
| `total_size`              | String | 总数量                                                                                                |
| `volume_24h`              | String | 24 小时成交量                                                                                         |
| `quanto_base_rate`        | String | 双币种合约中基础货币与结算货币的汇率。不存在于其他类型的合同中                                        |
| `volume_24h_btc`          | String | 近 24 小时 BTC 交易量（已弃用，请使用`volume_24h_base`、`volume_24h_quote`、`volume_24h_settle`代替） |
| `volume_24h_usd`          | String | 近 24 小时美元交易量（已弃用，请使用`volume_24h_base`、`volume_24h_quote`、`volume_24h_settle` 代替） |
| `volume_24h_quote`        | String | 近 24 小时交易量，以计价货币计                                                                        |
| `volume_24h_settle`       | String | 近 24 小时交易量，以结算货币计                                                                        |
| `volume_24h_base`         | String | 近 24 小时交易量，以基础货币计                                                                        |
| `low_24h`                 | String | 近 24 小时最低交易价                                                                                  |
| `high_24h`                | String | 近 24 小时最高交易价                                                                                  |
  | `change_from`             | String | 涨跌幅计算周期，如 `24h`                                                                              |
  | `change_price`            | String | 涨跌额                                                                                                |
  | `price_type`              | String | 价格类型：`last`（最新价）、`mark`（标记价）、`index`（指数价）                                       |
  | `t`                       | Integer | 行情生成时间戳（毫秒）                                                                                |
  | `event_type`              | String | 连接后首次推送为 `snapshot`，实时推送通常不返回该字段                                                 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": 123456,
  "channel": "futures.tickers",
  "event": "unsubscribe",
  "payload": ["BTC_USDT"]
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.tickers",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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
  "time": 1545404900,
  "time_ms": 1545404900123,
  "channel": "futures.tickers",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅**

### 请求参数

- channel

  `futures.tickers`
- event

  `unsubscribe`

</Block>

<Block>

# 公有成交频道

**每当 Gate 发生交易时，该频道都会发送交易消息。它包括价格、金额、时间和类型等交易信息**

</Block>

<Block>

## 公有成交订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.trades","event": "subscribe", "payload" : ["BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.trades","event":"subscribe","payload":["BTC_USDT"]}`

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
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.trades",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅公有成交更新通知**

### 请求参数

- channel

  `futures.trades`
- event

  `subscribe`
- params


| 名称      | 类型  | 必选 | 描述     |
| --------- | ----- | ---- | -------- |
| `payload` | Array | 是   | 合约列表 |

</Block>

<Block>

## 公有成交推送

<Example>

size 正数表示买家，负数表示卖家

```json
{
  "channel": "futures.trades",
  "event": "update",
  "time": 1541503698,
  "time_ms": 1541503698123,
  "result": [
    {
      "size": "-108.5",
      "id": 27753479,
      "create_time": 1545136464,
      "create_time_ms": 1545136464123,
      "price": "96.4",
      "contract": "BTC_USDT",
      "is_internal": true
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`size` 为整型（小数张数向下取整）：

```json
{
  "channel": "futures.trades",
  "event": "update",
  "time": 1541503698,
  "time_ms": 1541503698123,
  "result": [
    {
      "size": -108,
      "id": 27753479,
      "create_time": 1545136464,
      "create_time_ms": 1545136464123,
      "price": "96.4",
      "contract": "BTC_USDT",
      "is_internal": true
    }
  ]
}
```

</Example>

**通知最新交易更新**

### 推送参数

- channel

  `futures.trades`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称             | 类型       | 描述                                                                                                                                                                             |
| ---------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `contract`       | string     | 合约名称                                                                                                                                                                         |
| `size`           | string/int | 交易数量                                                                                                                                                                         |
| `id`             | int        | 交易 ID                                                                                                                                                                          |
| `create_time`    | int        | 交易消息创建时间                                                                                                                                                                 |
| `create_time_ms` | int        | 交易消息创建时间（以毫秒为单位）                                                                                                                                                 |
| `price`          | string     | 交易价格                                                                                                                                                                         |
| `is_internal`    | bool       | 是否为内部成交。内部成交是指保险资金和 ADL 用户对强平指令的接管。由于不是市场深度上的正常撮合，交易价格可能会出现偏差，不会记录在 K 线上。如果不是内部交易，则该字段不会被返回。 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.trades", "event": "subscribe", "payload" : ["BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.trades","event":"subscribe","payload":["BTC_USDT"]}`

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
  "time": 1545404900,
  "time_ms": 1545404900123,
  "channel": "futures.trades",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅公有成交更新通知**

### 请求参数

- channel

  `futures.trades`
- event

  `unsubscribe`

</Block>

<Block>

# 深度频道

**`order_book` 频道允许您跟踪 Gate 订单簿深度的状态。它以价格聚合的方式提供，可自定义精度。**

共有以下订单簿渠道可供订阅:

- `futures.order_book`

  全量频道，定期使用`all`推送完整的有限级别订单簿.
- `futures.rpi_order_book`

  RPI 全量频道，定期使用`all`推送有限级别订单簿。价格合并精度只支持 `0`.
- `futures.book_ticker`

  实时推送最佳买价和卖价.
- `futures.order_book_update`

  以指定的更新频率向用户订单簿推送订单簿的更新内容.

::: warning
不建议通过`futures.order_book`接收订单簿更新，使用 `futures.order_book_update` 可以以更少的流量提供更及时的更新
:::

如何维护本地订单簿:

1. 订阅 `futures.order_book_update` 并指定级别和更新频率，例如
   `["BTC_USDT", "100ms", "100"]` 每 100ms 推送 BTC_USDT 订单簿前 100 个级别的更新
2. 缓存 WebSocket 通知。每个通知都使用“U”和“u”来告诉第一个和最后一个
   自上次通知以来更新 ID。
3. 使用 REST API 检索基本订单簿，并确保记录了订单簿 ID（参考
   如下面的“baseID”）
   例如`https://api.gateio.ws/api/v4/futures/usdt/order_book?contract=BTC_USDT&limit=10&with_id=true`
   获取 BTC_USDT 的 10 级基础订单簿
4. 迭代缓存的 WebSocket 通知，找到第一个包含 baseID 的通知，
   即 `U <= baseId+1` 和 `u >= baseId+1`，然后开始从中消费。请注意，`size`为
   通知都是全量的 size，即应该使用它们覆盖替换原始的`size`。
   如果 size 等于 0，则从订单簿中删除价格。
5. 转储所有满足 `u < baseID+1` 的通知。如果 `baseID+1 < 第一个通知 U`，则
   意味着当前的基本订单簿落后于通知。从步骤 3 开始检索更新的内容基本订单簿。
6. 如果后续发现满足 `U > baseID+1` 的通知，则说明有更新
   丢失的。从步骤 3 重建本地订单簿。

注意：

* 即使 WebSocket 推送中的 `a`, `b` 或者  `asks`, `bids` 为空，用户也需要更新本地订单簿的 `id` 和 `timestamp`，以确保本地订单簿与服务器保持一致。
* WebSocket 订阅的 `level`（档位数）应与 REST 快照的 `limit` 一致，否则可能导致增量更新无法覆盖快照档位，造成本地订单簿错位或不完整。

</Block>

<Block>

## 深度全量更新频道

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.order_book","event": "subscribe", "payload" : ["BTC_USDT", "20", "0"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book","event":"subscribe","payload":["BTC_USDT","20","0"]}`

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

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.order_book",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅深度.**

### 请求参数

- channel

  `futures.order_book`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述                            |
| ---------- | ------ | ---- | ------------------------------- |
| `contract` | String | 是   | 合约名称                        |
| `limit`    | String | 是   | 深度层级: 100, 50, 20, 10, 5, 1 |
| `interval` | String | 是   | 价格合并精度: "0"               |

</Block>

<Block>

## 全量深度推送

<Example>

```json
{
  "channel": "futures.order_book",
  "event": "all",
  "time": 1541500161,
  "time_ms": 1541500161123,
  "result": {
    "t": 1541500161123,
    "contract": "BTC_USDT",
    "id": 93973511,
    "asks": [
      {
        "p": "97.1",
        "s": "2245.5"
      },
      {
        "p": "97.0",
        "s": "1200"
      }
    ],
    "bids": [
      {
        "p": "96.9",
        "s": "3100.2"
      },
      {
        "p": "96.8",
        "s": "0"
      }
    ],
    "l": "20"
  }
}
```

</Example>

**全量深度更新推送**

### 推送参数

- channel

  `futures.order_book`
- event

  `all`
- params


| 名称         | 类型    | 描述                                         |
| ------------ | ------- | -------------------------------------------- |
| `result`     | object  | 深度信息                                     |
| »`t`        | Integer | 深度生成时间戳（以毫秒为单位）               |
| »`contract` | String  | 合约名称                                     |
| »`id`       | Integer | 深度 ID                                      |
| »`asks`     | Array   | 深度卖方的档位列表                           |
| »»`p`      | String  | 档位价格                                     |
| »»`s`      | String  | 档位的数量                                   |
| »`bids`     | Array   | 深度买方的档位列表                           |
| »»`p`      | String  | 档位价格                                     |
| »»`s`      | String  | 档位的数量                                   |
| »`l`        | String  | 深度层级（例如 100 即代表 100 层的深度更新） |

</Block>

<Block>

## 全量深度取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.order_book","event": "unsubscribe", "payload" : ["BTC_USDT", "20", "0"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book","event":"unsubscribe","payload":["BTC_USDT","20","0"]}`

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
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.order_book",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅指定市场的深度**

### 请求参数

- channel

  `futures.order_book`
- event

  `unsubscribe`

</Block>

<Block>

## 最佳买卖价订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.book_ticker","event": "subscribe", "payload" : ["BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.book_ticker","event":"subscribe","payload":["BTC_USDT"]}`

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
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.book_ticker",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅深度最佳买卖价**

### 请求参数

- channel

  `futures.book_ticker`
- event

  `subscribe`
- params

  `payload`是一个包含合约市场的列表.

</Block>

<Block>

## 最佳买卖价的推送

如果 `a` 为空字符串，则表示空买价；如果 `b` 为空字符串，则表示空卖价。

<Example>

```json
{
  "time": 1615366379,
  "time_ms": 1615366379123,
  "channel": "futures.book_ticker",
  "event": "update",
  "result": {
    "t": 1615366379123,
    "u": 2517661076,
    "s": "BTC_USDT",
    "b": "54696.6",
    "B": "37000.5",
    "a": "54696.7",
    "A": "47061.2"
  }
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`B`、`A` 为整型：

```json
{
  "time": 1615366379,
  "time_ms": 1615366379123,
  "channel": "futures.book_ticker",
  "event": "update",
  "result": {
    "t": 1615366379123,
    "u": 2517661076,
    "s": "BTC_USDT",
    "b": "54696.6",
    "B": 37000,
    "a": "54696.7",
    "A": 47061
  }
}
```

</Example>

**最新买卖价的推送**

### 推送参数

- channel

  `futures.book_ticker`
- event

  `update`
- params


| 名称     | 类型           | 描述                                       |
| -------- | -------------- | ------------------------------------------ |
| `result` | object         | 深度的最佳买卖价                           |
| »`t`    | Integer        | 最佳买卖价行情生成的时间戳（以毫秒为单位） |
| »`u`    | Integer        | 深度的 ID                                  |
| »`s`    | String         | 合约名称                                   |
| »`b`    | String         | 最佳买方的价格，如果没有买方，则为空串     |
| »`B`    | String/Integer | 最佳买方的数量，如果没有买方，则为 0       |
| »`a`    | String         | 最佳卖方的价格，如果没有卖方，则为空串     |
| »`A`    | String/Integer | 最佳卖方的数量，如果没有卖方，则为 0       |

</Block>

<Block>

## 最佳买卖价的取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.book_ticker","event": "unsubscribe", "payload" : ["BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.book_ticker","event":"unsubscribe","payload":["BTC_USDT"]}`

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
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.book_ticker",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消指定合约市场的最佳买卖订阅**

### 请求

- channel

  `futures.book_ticker`
- event

  `unsubscribe`

</Block>

<Block>

## 合约深度更新推送订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.order_book_update","event": "subscribe", "payload" : ["BTC_USDT", "100ms", "100"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book_update","event":"subscribe","payload":["BTC_USDT", "100ms", "100"]}`

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
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.order_book_update",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅深度更新推送**

### 请求参数

- channel

  `futures.order_book_update`
- event

  `subscribe`
- params


| 名称        | 类型   | 必选 | 描述                                                                      |
| ----------- | ------ | ---- | ------------------------------------------------------------------------- |
| `contract`  | String | 是   | 合约名称                                                                  |
| `frequency` | String | 是   | 更新频率,`20ms` or `100ms`                                                |
| `level`     | String | 否   | 可选的深度层级。允许以下层级：`100`、`50`、`20`；`20ms`频率 只支持 `20`层 |

</Block>

<Block>

## 深度更新推送

<Example>

```json
{
  "time": 1615366381,
  "time_ms": 1615366381123,
  "channel": "futures.order_book_update",
  "event": "update",
  "result": {
    "t": 1615366381417,
    "s": "BTC_USDT",
    "U": 2517661101,
    "u": 2517661113,
    "b": [
      {
        "p": "54672.1",
        "s": "0"
      },
      {
        "p": "54664.5",
        "s": "58794.5"
      }
    ],
    "a": [
      {
        "p": "54743.6",
        "s": "0"
      },
      {
        "p": "54742",
        "s": "95.2"
      }
    ],
    "l": "100"
  }
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，档位 `s` 为整型：

```json
{
  "time": 1615366381,
  "time_ms": 1615366381123,
  "channel": "futures.order_book_update",
  "event": "update",
  "result": {
    "t": 1615366381417,
    "s": "BTC_USDT",
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
    ],
    "l": "100"
  }
}
```

</Example>

**深度更新推送**

### 推送参数

- channel

  `futures.order_book_update`
- event

  `update`
- params


| 名称     | 类型           | 描述                                                                                                                                                             |
| -------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `result` | Object         | 自上次更新以来发生变更要价和出价                                                                                                                                 |
| »`t`    | Integer        | 订单簿生成时间戳（以毫秒为单位）                                                                                                                                 |
| »`full` | Boolean        | `true` 代表全量的深度（假设订阅 `level` 为 100，推送出来则是 100 层的深度）；用户接收到之后需要替换本地深度；`false` 代表增量的深度，为 `false` 时，不传输该字段 |
| »`s`    | String         | 合约名称                                                                                                                                                         |
| »`U`    | Integer        | 本次更新开始的订单簿更新 ID                                                                                                                                      |
| »`u`    | Integer        | 本次更新结束的订单簿更新 ID                                                                                                                                      |
| »`b`    | Array          | 买方变动列表                                                                                                                                                     |
| »»`p`  | String         | 变更的档位价格                                                                                                                                                   |
| »»`s`  | String/Integer | 档位的待成交数量。如果为 0，则从订单簿中删除该价格                                                                                                               |
| »`a`    | Array          | 卖方变动列表                                                                                                                                                     |
| »»`p`  | String         | 变更的档位价格                                                                                                                                                   |
| »»`s`  | String/Integer | 档位的待成交数量。如果为 0，则从订单簿中删除该价格                                                                                                               |
| »`l`    | String         | 深度层级（例如 100 即代表 100 层的深度更新）                                                                                                                     |

</Block>

<Block>

## 深度更新取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.order_book_update", "event": "unsubscribe", "payload" : ["BTC_USDT", "100ms"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book_update","event":"unsubscribe","payload":["BTC_USDT", "100ms"]}`

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
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.order_book_update",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消指定合约的市场的深度更新订阅**

### 请求参数

- channel

  `futures.order_book_update`
- event

  `unsubscribe`

</Block>

<Block>

# RPI 深度全量更新频道

`futures.rpi_order_book`

推送 RPI 订单簿的全量快照。本频道是公开频道。

订阅参数和推送字段与 `futures.order_book` 相同。价格合并精度只支持 `"0"`。推送 `event` 为 `all`。

SBE 复用 `orderBook` 模板（templateId=`4`），帧里的 `channel` 为 `futures.rpi_order_book`。

</Block>

<Block>

## RPI 深度全量订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.rpi_order_book","event": "subscribe", "payload" : ["BTC_USDT", "20", "0"]}')
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
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.rpi_order_book","event":"subscribe","payload":["BTC_USDT","20","0"]}`

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

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545405058,
  "time_ms": 1545405058123,
  "channel": "futures.rpi_order_book",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅 RPI 深度.**

### 请求参数

- channel

  `futures.rpi_order_book`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述                            |
| ---------- | ------ | ---- | ------------------------------- |
| `contract` | String | 是   | 合约名称                        |
| `limit`    | String | 是   | 深度层级: 100, 50, 20, 10, 5    |
| `interval` | String | 是   | 价格合并精度，只支持 `"0"`      |

</Block>

<Block>

## RPI 全量深度推送

<Example>

```json
{
  "channel": "futures.rpi_order_book",
  "event": "all",
  "time": 1541500161,
  "time_ms": 1541500161123,
  "result": {
    "t": 1541500161123,
    "contract": "BTC_USDT",
    "id": 93973511,
    "asks": [
      {
        "p": "97.1",
        "s": "2245.5"
      }
    ],
    "bids": [
      {
        "p": "96.9",
        "s": "3100.2"
      }
    ],
    "l": "20"
  }
}
```

</Example>

**RPI 全量深度更新推送**

### 推送参数

- channel

  `futures.rpi_order_book`
- event

  `all`
- params


| 名称         | 类型    | 描述                                         |
| ------------ | ------- | -------------------------------------------- |
| `result`     | object  | 深度信息                                     |
| »`t`         | Integer | 深度生成时间戳（以毫秒为单位）               |
| »`contract`  | String  | 合约名称                                     |
| »`id`        | Integer | 深度 ID                                      |
| »`asks`      | Array   | 深度卖方的档位列表                           |
| »»`p`        | String  | 档位价格                                     |
| »»`s`        | String  | 档位的数量                                   |
| »`bids`      | Array   | 深度买方的档位列表                           |
| »»`p`        | String  | 档位价格                                     |
| »»`s`        | String  | 档位的数量                                   |
| »`l`         | String  | 深度层级（例如 20 即代表 20 层的深度更新）   |

</Block>

<Block>

## RPI 全量深度取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.rpi_order_book","event": "unsubscribe", "payload" : ["BTC_USDT", "20", "0"]}')
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
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.rpi_order_book","event":"unsubscribe","payload":["BTC_USDT","20","0"]}`

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

**取消订阅指定市场的 RPI 深度**

### 请求参数

- channel

  `futures.rpi_order_book`
- event

  `unsubscribe`

</Block>

<Block>

# 深度频道V2

**提供一种更快更新的获取深度信息的方法.**

`futures.obu` 与 `futures.rpi_obu` 都使用这一套本地深度维护方式。`futures.rpi_obu` 见下方 [RPI 深度频道V2](#rpi-深度频道v2)。

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
    "ob.BTC_USDT.400"
  ],
  "error": {
    "code": 2,
    "message": "Alert sub ob.BTC_USDT.400"
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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.obu",
        "event": "subscribe", "payload" : ["ob.BTC_USDT.400"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.obu", "event": "subscribe", "payload" : ["ob.BTC_USDT.400"]}`

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
    "ob.BTC_USDT.400"
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

  `payload`是一个包含流名称的列表. 格式为: ob.{symbol}.{level}; 例如 ob.BTC_USDT.400、ob.BTC_USDT.50

  其中`level`枚举为：400、50；更新频率: 400档为100ms；50档为20ms；

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
		"s": "ob.BTC_USDT.400",
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
		"s": "ob.BTC_USDT.400",
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

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT.400"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT.400"]}`

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
  "payload": ["ob.BTC_USDT.400"],
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消指定合约的市场的深度频道V2订阅**

### 请求参数

- channel

  `futures.obu`
- event

  `unsubscribe`

</Block>

<Block>

# RPI 深度频道V2

`futures.rpi_obu`

推送 RPI 订单簿的全量和增量。本频道是公开频道。

消息结构、本地深度维护方式与 [深度频道V2](#深度频道v2) 相同。流名称前缀为 `rpi_ob.`。

SBE 复用 `obu` 模板（templateId=`3`），帧里的 `channel` 为 `futures.rpi_obu`。

</Block>

<Block>

## RPI 深度频道V2订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.rpi_obu",
        "event": "subscribe", "payload" : ["rpi_ob.BTC_USDT.400"]}')
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
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.rpi_obu", "event": "subscribe", "payload" : ["rpi_ob.BTC_USDT.400"]}`

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
  "channel": "futures.rpi_obu",
  "event": "subscribe",
  "payload": [
    "rpi_ob.BTC_USDT.400"
  ],
  "result": {
    "status": "success"
  }
}
```

</Example>

### 请求参数

- channel

  `futures.rpi_obu`
- event

  `subscribe`
- params

  `payload` 是一个包含流名称的列表. 格式为: `rpi_ob.{symbol}.{level}`；例如 `rpi_ob.BTC_USDT.400`、`rpi_ob.BTC_USDT.50`

  其中 `level` 枚举为：400、50；更新频率: 400 档为 100ms；50 档为 20ms。

订阅成功后，首条推送为全量（`full=true`），之后为增量。同一合约的同一深度流，一个链接只允许订阅一次。

</Block>

<Block>

## RPI 深度v2订阅推送

<Example>

全量推送示例:

```json
{
	"channel": "futures.rpi_obu",
	"result": {
		"t": 1743673026995,
		"full": true,
		"s": "rpi_ob.BTC_USDT.400",
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
	"channel": "futures.rpi_obu",
	"result": {
		"t": 1743673027017,
		"s": "rpi_ob.BTC_USDT.400",
		"U": 79072179674,
		"u": 79072179694,
		"b": [
			["83702.2", "62"],
			["83702.1", "0"]
		]
	},
	"time_ms": 1743673027020
}
```

</Example>

**RPI 深度频道V2的消息推送**

### 推送参数

- channel

  `futures.rpi_obu`
- event

  `update`
- params


| 名称                | 类型                    | 描述                                                                       |
| ------------------- | ----------------------- | -------------------------------------------------------------------------- |
| `result`            | Object                  | 自上次更新以来发生变更的卖单和买单                                         |
| »`t`                | Integer                 | 订单簿生成时间戳（以毫秒为单位）                                           |
| »`full`             | Boolean                 | `true` 代表全量的深度；`false` 代表增量的深度，为 `false` 时，不传输该字段 |
| »`s`                | String                  | 深度流的名称，例如 `rpi_ob.BTC_USDT.400`                                   |
| »`U`                | Integer                 | 本次更新开始的订单簿更新 ID                                                |
| »`u`                | Integer                 | 本次更新结束的订单簿更新 ID                                                |
| »`b`                | `Array[OrderBookArray]` | 自上次更新以来的 bids 更新                                                 |
| »» OrderBookArray   | `Array[String]`         | [Price, Amount] 数组对, Amount=0 应当从本地深度中移除                      |
| »`a`                | `Array[OrderBookArray]` | 自上次更新以来的 asks 更新                                                 |
| »» OrderBookArray   | `Array[String]`         | [Price, Amount] 数组对, Amount=0 应当从本地深度中移除                      |

</Block>

<Block>

## RPI 深度频道V2取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.rpi_obu", "event": "unsubscribe", "payload" : ["rpi_ob.BTC_USDT.400"]}')
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
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "futures.rpi_obu", "event": "unsubscribe", "payload" : ["rpi_ob.BTC_USDT.400"]}`

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

**取消指定合约的 RPI 深度频道V2订阅**

### 请求参数

- channel

  `futures.rpi_obu`
- event

  `unsubscribe`

</Block>

<Block>

# K 线频道

**提供一种访问 K 线信息的方法.**

</Block>

<Block>

## K 线订阅

<strong><em>如果在`contract`前面加上`mark_`，则将订阅合约的标记价格 K 线；如果
前缀为“index\_”，将订阅指数价格 K 线.</em></strong>

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.candlesticks","event": "subscribe", "payload" : ["1m", "BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.candlesticks","event":"subscribe","payload":["1m", "BTC_USDT"]}`

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
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.candlesticks",
  "event": "subscribe",
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


| 名称       | 类型   | 描述                                                                     |
| ---------- | ------ | ------------------------------------------------------------------------ |
| `interval` | String | Interval : "10s", "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "7d" |
| `contract` | String | 合约名称                                                                 |

</Block>

<Block>

## k 线消息推送

<Example>

```json
{
  "time": 1542162490,
  "time_ms": 1542162490123,
  "channel": "futures.candlesticks",
  "event": "update",
  "result": [
    {
      "t": 1545129300,
      "v": "27525555.5",
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USDT",
      "a": "314732.87412",
      "w": false
    },
    {
      "t": 1545129300,
      "v": "27525556",
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USDT",
      "a": "314732.87412",
      "w": true
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`v` 为整型：

```json
{
  "time": 1542162490,
  "time_ms": 1542162490123,
  "channel": "futures.candlesticks",
  "event": "update",
  "result": [
    {
      "t": 1545129300,
      "v": 27525555,
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USDT",
      "a": "314732.87412",
      "w": false
    }
  ]
}
```

</Example>

**k 线的消息推送**

### 推送参数

- channel

  `futures.candlesticks`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称 | 类型           | 描述                                                                      |
| ---- | -------------- | ------------------------------------------------------------------------- |
  | `t`  | Integer        | 时间                                                                      |
| `o`  | String         | 开盘价格                                                                  |
| `c`  | String         | 收盘价格                                                                  |
| `h`  | String         | 最高价格                                                                  |
| `l`  | String         | 最低价格                                                                  |
| `v`  | String/Integer | 成交量                                                                    |
| `n`  | String         | 合约名称                                                                  |
| `a`  | String         | 成交原始币种数量                                                          |
| `w`  | Boolean        | `true` 表示窗口已关闭。注：可能会缺失 `true` 的消息，但不影响数据的完整性 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.candlesticks", "event": "unsubscribe", "payload" : ["1m", "BTC_USDT"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.candlesticks","event":"unsubscribe","payload":["1m", "BTC_USDT"]}`

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
  "time": 1545445847,
  "time_ms": 1545445847123,
  "channel": "futures.candlesticks",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅指定市场 K 线信息**

### 请求参数

- channel

  `futures.candlesticks`
- event

  `unsubscribe`

</Block>

<Block>

# 公共强平订单频道

**提供一种接收Gate强平订单信息的方式。同一合约在一个聚合周期（默认1秒）内产生的强平订单将聚合为数组批量推送；当单个合约缓存的强平订单数量达到上限（默认20笔）时，将立即提前推送**

</Block>

<Block>

## 公共强平订单订阅

如果您想订阅所有合约中的强平订单推送，请在订阅请求列表中使用 `!all`

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": 123456,
    "channel": "futures.public_liquidates",
    "event": "subscribe",
    "payload": ["BTC_USDT","BTC_USDT"],
}
ws.send(json.dumps(req))
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
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.public_liquidates",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT", "BTC_USDT"},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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
    "time": 1545459681,
    "time_ms": 1545459681123,
    "channel": "futures.public_liquidates",
    "event": "subscribe",
    "result": {
        "status": "success"
    }
}
```

</Example>

**订阅公共强平订单推送**

### 请求参数

- channel

  `futures.public_liquidates`
- event

  `subscribe`
- params


  | 名称       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `contract` | String | 是   | 合约名称列表 |

</Block>

<Block>

## 公共强平订单推送

<Example>

```json
{
    "channel": "futures.public_liquidates",
    "event": "update",
    "time": 1541505434,
    "time_ms": 1541505434123,
    "result": [
        {
        "price": "215.1",
        "size": "-124.5",
        "time": 1541486601123,
        "contract": "BTC_USDT"
        },
        {
        "price": "215.2",
        "size": "-10.5",
        "time": 1541486601456,
        "contract": "BTC_USDT"
        }
    ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`size` 为整型：

```json
{
    "channel": "futures.public_liquidates",
    "event": "update",
    "time": 1541505434,
    "time_ms": 1541505434123,
    "result": [
        {
        "price": "215.1",
        "size": -124,
        "time": 1541486601123,
        "contract": "BTC_USDT"
        },
        {
        "price": "215.2",
        "size": -10,
        "time": 1541486601456,
        "contract": "BTC_USDT"
        }
    ]
}
```

</Example>

**推送公共强制平仓更新**

### 推送参数

- channel

  `futures.public_liquidates`
- event

  `update`
- params


  | 名称     | 类型  | 描述                                                       |
  | -------- | ----- | ---------------------------------------------------------- |
  | `result` | Array | 同一合约在聚合周期内的全部强平订单列表，可能包含多笔记录 |


  | 名称       | 类型           | 描述                     |
  | ---------- | -------------- | ------------------------ |
  | `price`    | Float          | 订单价格                 |
  | `size`     | String/Integer | 强平订单数量             |
  | `time`     | Integer        | 强平时间（以毫秒为单位） |
  | `contract` | String         | 合约名称                 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": 123456,
    "channel": "futures.public_liquidates",
    "event": "unsubscribe",
    "payload": ["BTC_USDT"],
}
ws.send(json.dumps(req))
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
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.public_liquidates",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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
    "time": 1545459681,
    "time_ms": 1545459681123,
     "channel": "futures.public_liquidates",
     "event": "unsubscribe",
     "result": {
        "status": "success"
     }
}
```

</Example>

**取消订阅公共强平订单更新**

### 请求参数

- channel

  `futures.public_liquidates`
- event

  `unsubscribe`

</Block>

<Block>

# 合约信息频道

**contract_info 通道允许您接收合约精度信息（张乘数、下单价格精度、标记价格精度）与风险限额档位的变更推送**

</Block>

<Block>

## 合约信息订阅

如果您想订阅所有合约的信息推送，请在订阅请求列表中使用 `!all`

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": 123456,
    "channel": "futures.contract_info",
    "event": "subscribe",
    "payload": ["BTC_USDT"],
}
ws.send(json.dumps(req))
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
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.contract_info",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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
    "time": 1545459681,
    "time_ms": 1545459681123,
    "channel": "futures.contract_info",
    "event": "subscribe",
    "result": {
        "status": "success"
    }
}
```

</Example>

**订阅合约信息推送**

### 请求参数

- channel

  `futures.contract_info`
- event

  `subscribe`
- params


  | 名称       | 类型   | 必选 | 描述         |
  | ---------- | ------ | ---- | ------------ |
  | `contract` | String | 是   | 合约名称列表 |

</Block>

<Block>

## 合约信息推送

订阅成功后，若服务端已有该合约的缓存快照，将立即推送一条 `event` 为 `all` 的全量消息；此后当合约精度信息或风险限额档位发生变更时，将推送 `event` 为 `update` 的消息。仅精度信息变更时，`update` 消息可能不携带 `risk_limit_tiers` 字段。

<Example>

订阅成功后的首次推送（event=all）：

```json
{
    "channel": "futures.contract_info",
    "event": "all",
    "time": 1541505434,
    "time_ms": 1541505434123,
    "result": {
        "contract": "BTC_USDT",
        "quanto_multiplier": "0.0001",
        "order_price_round": "0.1",
        "mark_price_round": "0.01",
        "risk_limit_tiers": [
            {
                "tier": 1,
                "risk_limit": "1000000",
                "initial_rate": "0.01",
                "maintenance_rate": "0.005",
                "leverage_max": "100",
                "contract": "BTC_USDT",
                "deduction": "0"
            }
        ]
    }
}
```

</Example>

<Example>

合约信息变更推送（event=update）：

```json
{
    "channel": "futures.contract_info",
    "event": "update",
    "time": 1541505434,
    "time_ms": 1541505434123,
    "result": {
        "contract": "BTC_USDT",
        "quanto_multiplier": "0.0001",
        "order_price_round": "0.5",
        "mark_price_round": "0.01"
    }
}
```

</Example>

**推送合约信息更新**

### 推送参数

- channel

  `futures.contract_info`
- event

  `all` / `update`
- params


  | 名称     | 类型   | 描述                                                    |
  | -------- | ------ | ------------------------------------------------------- |
  | `result` | Object | 合约信息对象，event 为 all 时表示当前快照，update 时表示变更后的最新信息 |


  | 名称                | 类型   | 描述                                                       |
  | ------------------- | ------ | ---------------------------------------------------------- |
  | `contract`          | String | 合约名称                                                   |
  | `quanto_multiplier` | String | 张乘数                                                     |
  | `order_price_round` | String | 下单价格精度                                               |
  | `mark_price_round`  | String | 标记价格精度                                               |
  | `risk_limit_tiers`  | Array  | 风险限额档位列表，仅精度信息变更时可能不携带               |


  `risk_limit_tiers` 字段说明：

  | 名称               | 类型    | 描述           |
  | ------------------ | ------- | -------------- |
  | `tier`             | Integer | 档位           |
  | `risk_limit`       | String  | 风险限额       |
  | `initial_rate`     | String  | 初始保证金率   |
  | `maintenance_rate` | String  | 维持保证金率   |
  | `leverage_max`     | String  | 最大杠杆倍数   |
  | `contract`         | String  | 合约名称       |
  | `deduction`        | String  | 维持保证金速算额 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": 123456,
    "channel": "futures.contract_info",
    "event": "unsubscribe",
    "payload": ["BTC_USDT"],
}
ws.send(json.dumps(req))
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
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.contract_info",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
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
    "time": 1545459681,
    "time_ms": 1545459681123,
     "channel": "futures.contract_info",
     "event": "unsubscribe",
     "result": {
        "status": "success"
     }
}
```

</Example>

**取消订阅合约信息更新**

### 请求参数

- channel

  `futures.contract_info`
- event

  `unsubscribe`

</Block>

<Block>

# 合约统计信息频道

**contract_stats 通道允许您获取合约统计信息**

</Block>

<Block>

## 订阅操作

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.contract_stats","event": "subscribe", "payload" : ["BTC_USDT","1m"]}')
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	message := `{
		"time": 123456,
		"channel": "futures.contract_stats",
		"event": "subscribe",
		"payload": ["BTC_USDT", "1m"]
	}`

	err = conn.WriteMessage(websocket.TextMessage, []byte(message))
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的订阅请求返回 JSON 结构如下:

```json
{
  "time": 1545404023,
  "time_ms": 1545404023123,
  "channel": "futures.contract_stats",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅合约统计信息**

### 请求参数

- channel

  `futures.contract_stats`
- event

  `subscribe`
- params


  | 名称       | 类型   | 必选 | 描述                                                                    |
  | ---------- | ------ | ---- | ----------------------------------------------------------------------- |
  | `contract` | String | Yes  | 合约名称                                                                |
  | `interval` | String | Yes  | Interval : "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "3d", "7d" |

</Block>

<Block>

## contract_stats 推送

<Example>

```json
{
  "time": 1541659086,
  "time_ms": 1541659086123,
  "channel": "futures.contract_stats",
  "event": "update",
  "result": [
    {
      "time": 1603865400,
      "contract":"BTC_USDT",
      "lsr_taker": 100,
      "lsr_account": 0.5,
      "long_liq_size": "0",
      "short_liq_size": "0",
      "open_interest": "124724",
      "short_liq_usd": 0,
      "mark_price": "8865",
      "top_lsr_size": 1.02,
      "short_liq_amount": 0,
      "long_liq_amount": 0,
      "open_interest_usd": 1511,
      "top_lsr_account": 1.5,
      "long_liq_usd": 0
    }
  ]
}
```

</Example>

**合约统计信息推送**

### 推送参数

- channel

  `futures.contract_stats`
- event

  `update`
- params


  | 名称     | 类型  | 描述             |
  | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |


  | 名称                | 类型           | 描述                   |
  | ------------------- | -------------- | ---------------------- |
  | `time`              | Integer        | 统计时间（时间戳）     |
  | `contract`          | string         | 合约名称               |
  | `mark_price`        | Float          | 当前标记价格           |
  | `lsr_taker`         | Float          | 多空吃单比             |
  | `lsr_account`       | Float          | 多空持仓用户比         |
  | `long_liq_size`     | string/Integer | 做多爆仓量（张）       |
  | `long_liq_amount`   | Float          | 做多爆仓量（交易币种） |
  | `long_liq_usd`      | Float          | 做多爆仓量（计价币种） |
  | `short_liq_size`    | string/Integer | 做空爆仓量（张）       |
  | `short_liq_amount`  | Float          | 做空爆仓量（交易币种） |
  | `short_liq_usd`     | Float          | 做空爆仓量（计价币种） |
  | `open_interest`     | string/Integer | 总持仓量（张）         |
  | `open_interest_usd` | Float          | 总持仓量（计价币种）   |
  | `top_lsr_account`   | Float          | 大户多空账户比         |
  | `top_lsr_size`      | String/Float   | 大户多空持仓比         |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": 123456,
    "channel": "futures.contract_stats",
    "event": "unsubscribe",
    "payload": ["BTC_USDT","1m"]
}
ws.send(json.dumps(req))
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

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := `{
		"time": 123456,
		"channel": "futures.contract_stats",
		"event": "unsubscribe",
		"payload": ["BTC_USDT", "1m"]
	}`

	err = conn.WriteMessage(websocket.TextMessage, []byte(req))
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的请求返回 JSON 结构如下:

```json
{
  "time": 1545404900,
  "time_ms": 1545404900123,
  "channel": "futures.contract_stats",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅**

### 请求参数

- channel

  `futures.contract_stats`
- event

  `unsubscribe`
- params


  | 名称       | 类型   | 必选 | 描述                                                                    |
  | ---------- | ------ | ---- | ----------------------------------------------------------------------- |
  | `contract` | String | Yes  | 合约名称                                                                |
  | `interval` | String | Yes  | Interval : "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "3d", "7d" |

**注意：`contract`为`unsub_all`，表示全部取消**

</Block>

<Block>

# ADL 风险预警频道

**公共频道 `futures.adl_warning` 用于推送市场级 ADL 风险预警。**

该频道无需鉴权，仅支持 JSON 连接。

市场级自动减仓（ADL）风险预警用于帮助用户了解某个合约市场发生 ADL 的风险变化。平台根据保险基金在该市场的持仓情况及对应市场限额判断风险状态，对外仅提供状态结果，不公开底层具体阈值和计算公式。

用户需主动建立 WebSocket 连接并订阅对应频道或合约后才会收到推送；未订阅或断开连接后不会收到推送。

### 状态说明

| 状态 | 对外含义 |
| ---- | -------- |
| `normal` | 市场发生 ADL 的概率较小，但突发的大幅市场波动可能产生大量强平单并触发 ADL。 |
| `warning` | 市场发生 ADL 的概率较大，表示保险基金承接强平单的剩余能力有限，该市场接下来新产生的强平单很可能通过 ADL 执行。 |
| `adl_risk` | 保险基金已无法再承接该市场的强平单，新产生的强平单只能通过 ADL 执行。该状态本身不代表 ADL 已经发生。 |

### 补充说明

1. `normal`、`warning`、`adl_risk` 是市场级风险状态，不代表单个用户仓位一定会或不会被 ADL。
2. `adl_risk` 不等同于 ADL 已经发生，也不代表所有用户都会受到 ADL 影响。
3. 该状态仅用于帮助用户了解市场风险变化，不构成交易建议。
4. 用户基于该状态进行减仓、加仓、撤单、调整策略等操作，应自行承担对应交易结果。
5. 如需查询历史强平单记录，请参考[查询强平委托历史](https://www.gate.com/docs/developers/apiv4/zh_CN/futures/#查询强平委托历史)。

</Block>

<Block>

## ADL 风险预警订阅

<Example>

订阅一个或多个合约：

```json
{
  "time": 1781234567,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["BTC_USDT", "ETH_USDT"]
}
```

订阅当前连接结算币种下的全部合约：

```json
{
  "time": 1781234567,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["!all"]
}
```

订阅命中的合约当前全部为 `normal` 时，服务端只返回订阅响应：

```json
{
  "time": 1781234567,
  "time_ms": 1781234567123,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["BTC_USDT", "ETH_USDT"],
  "result": {
    "status": "success"
  }
}
```

</Example>

### 请求参数

- channel

  `futures.adl_warning`
- event

  `subscribe`
- params

  | 参数       | 类型   | 是否必需 | 描述                                                                     |
  | ---------- | ------ | -------- | ------------------------------------------------------------------------ |
  | `contract` | String | 是       | 合约名称。支持传入多个合约；仅传入 `!all` 时订阅当前结算币种下的全部合约 |

`payload` 不能为空。`!all` 不能与具体合约名称混用，指定的合约必须属于当前连接的结算币种。重复的合约名称会被忽略。

如果订阅命中的合约已经处于 `warning` 或 `adl_risk`，对应的 `event=all` 首次快照会先于订阅响应发送。

</Block>

<Block>

## ADL 风险预警推送

<Example>

订阅后的首次风险快照：

```json
{
  "time": 1781234567,
  "time_ms": 1781234567123,
  "channel": "futures.adl_warning",
  "event": "all",
  "result": {
    "contract": "BTC_USDT",
    "settle": "usdt",
    "state": "warning",
    "update_time": 1781234567000
  }
}
```

风险状态推送：

```json
{
  "time": 1781234568,
  "time_ms": 1781234568123,
  "channel": "futures.adl_warning",
  "event": "update",
  "result": {
    "contract": "BTC_USDT",
    "settle": "usdt",
    "state": "adl_risk",
    "update_time": 1781234567000
  }
}
```

恢复状态推送：

```json
{
  "time": 1781234578,
  "time_ms": 1781234578123,
  "channel": "futures.adl_warning",
  "event": "update",
  "result": {
    "contract": "BTC_USDT",
    "settle": "usdt",
    "state": "normal",
    "update_time": 1781234577000
  }
}
```

</Example>

### 推送参数

- channel

  `futures.adl_warning`
- event

  首次风险快照为 `all`，后续推送为 `update`
- params

  | 字段          | 类型    | 描述                                                         |
  | ------------- | ------- | ------------------------------------------------------------ |
  | `contract`    | String  | 合约名称                                                     |
  | `settle`      | String  | 小写结算币种，例如 `btc`、`usdt` 或 `usd1`                   |
  | `state`       | String  | 市场级 ADL 风险状态：`normal`、`warning` 或 `adl_risk`，详见[状态说明](#状态说明)               |
  | `update_time` | Integer | 风险状态的计算时间，Unix 毫秒                                |

`warning` 和 `adl_risk` 状态持续期间每秒推送一次。恢复为 `normal` 时只推送一次，不会重复推送。首次订阅时处于 `normal` 的合约不会发送首次快照。

</Block>

<Block>

## 取消 ADL 风险预警订阅

<Example>

```json
{
  "time": 1781234579,
  "channel": "futures.adl_warning",
  "event": "unsubscribe",
  "payload": ["BTC_USDT", "ETH_USDT"]
}
```

服务端返回：

```json
{
  "time": 1781234579,
  "time_ms": 1781234579123,
  "channel": "futures.adl_warning",
  "event": "unsubscribe",
  "payload": ["BTC_USDT", "ETH_USDT"],
  "result": {
    "status": "success"
  }
}
```

</Example>

### 请求参数

- channel

  `futures.adl_warning`
- event

  `unsubscribe`
- params

  与订阅使用相同的 `payload` 规则。使用 `["!all"]` 取消全部合约订阅。

</Block>

<Block>

# 订单频道

**提供接收用户订单的推送**

::: warning
需要认证.
:::

</Block>

<Block>

## 订单订阅

如果您想订阅所有合约中的订单更新，请在合约列表中使用 `!all`。

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.orders",
  "event": "subscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.orders",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.orders",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅订单更新推送**

### 请求参数

- channel

  `futures.orders`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 订单推送

<Example>

```json
{
  "channel": "futures.orders",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "contract": "BTC_USDT",
      "create_time": 1628736847,
      "create_time_ms": 1628736847325,
      "fill_price": "40000.4",
      "finish_as": "filled",
      "finish_time": 1628736848,
      "finish_time_ms": 1628736848321,
      "iceberg": "0",
      "id": 4872460,
      "id_string": "4872460",
      "is_close": false,
      "is_liq": false,
      "is_reduce_only": false,
      "is_voucher": false,
      "left": "0",
      "mkfr": -0.00025,
      "price": "40000.4",
      "refr": 0,
      "refu": 0,
      "size": "1.5",
      "status": "finished",
      "text": "api",
      "tif": "gtc",
      "tkfr": 0.0005,
      "user": "110xxxxx",
      "update_id": 1,
      "update_time": 1541505434123,
      "biz_info": "-",
      "stp_act": "-",
      "stp_id": "0",
      "amend_text": "-",
      "role": "taker",
      "fee": 0.0009290592,
      "point_fee": 0,
      "bbo": "-",
      "pos_margin_mode": "cross",
      "leverage": "10",
      "stop_loss_price": "",
      "stop_profit_price": "",
      "market_order_slip_ratio": "0.03"
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`size`、`left`、`iceberg` 为整型：

```json
{
  "channel": "futures.orders",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "contract": "BTC_USDT",
      "create_time": 1628736847,
      "create_time_ms": 1628736847325,
      "fill_price": "40000.4",
      "finish_as": "filled",
      "finish_time": 1628736848,
      "finish_time_ms": 1628736848321,
      "iceberg": 0,
      "id": 4872460,
      "id_string": "4872460",
      "is_close": false,
      "is_liq": false,
      "is_reduce_only": false,
      "left": 0,
      "mkfr": -0.00025,
      "price": "40000.4",
      "size": 1,
      "status": "finished",
      "text": "api",
      "tif": "gtc",
      "tkfr": 0.0005,
      "user": "110xxxxx",
      "update_id": 1,
      "update_time": 1541505434123,
      "market_order_slip_ratio": "0.03"
    }
  ]
}
```

</Example>

**下单、更新或完成时通知用户订单信息**

### 推送参数

- channel

  `futures.orders`
- event

  `update`
- params

  `推送结果参数含义请参考http接口.`


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称                | 类型           | 描述                                                                                                                                                                                                                                                                                                                  |
| ------------------- | -------------- |---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `create_time`       | Integer        | 订单创建时间（已弃用）                                                                                                                                                                                                                                                                                                         |
| `create_time_ms`    | Integer        | 订单创建时间戳（以毫秒为单位）                                                                                                                                                                                                                                                                                                     |
| `fill_price`        | String         | 订单成交价格                                                                                                                                                                                                                                                                                                              |
| `finish_as`         | String         | 订单结束方式，包括：<br><br>- open: 等待处理<br>- filled: 完全成交<br>- cancelled: 用户撤销<br>- liquidate_cancelled: 爆仓撤销<br>- small: 订单数量太小<br>- depth_not_enough: 深度不足导致撤单<br>- trader_not_enough: 对手方不足导致撤单<br>- ioc: 未立即成交，因为 tif 设置为 ioc<br>- poc: 未满足挂单策略，因为 tif 设置为 poc<br>- fok: 未立即完全成交，因为 tif 设置为 fok<br>- stp: 订单发生自成交限制而被撤销  |
| `iceberg`           | String/Integer | 冰山下单显示的数量，不指定或传 0 都默认为普通下单。目前不支持全部冰山。                                                                                                                                                                                                                                                                               |
| `id`                | Integer        | 订单 ID                                                                                                                                                                                                                                                                                                               |
| `is_close`          | Bool           | 该订单是否为 close position                                                                                                                                                                                                                                                                                               |
| `is_liq`            | Bool           | 该订单是否为 liquidation                                                                                                                                                                                                                                                                                                  |
| `left`              | String/Integer | 剩余可交易数量                                                                                                                                                                                                                                                                                                             |
| `mkfr`              | Float          | Maker 费用                                                                                                                                                                                                                                                                                                            |
| `is_reduce_only`    | Bool           | 该订单是否为 reduce-only                                                                                                                                                                                                                                                                                                  |
| `status`            | String         | 订单状态<br />- open: 等待交易<br />- finished: 完成                                                                                                                                                                                                                                                                          |
| `tkfr`              | Float          | taker 费用                                                                                                                                                                                                                                                                                                            |
| `price`             | String         | 订单价格。 0 表示市价订单，tif 设置为 ioc                                                                                                                                                                                                                                                                                          |
| `refu`              | Integer        | 推荐用户 ID                                                                                                                                                                                                                                                                                                             |
| `refr`              | Float          |                                                                                                                                                                                                                                                                                                                     |
| `size`              | String/Integer | 订单大小。指定正数进行出价，指定负数进行询问                                                                                                                                                                                                                                                                                              |
| `text`              | String         | 用户定义的信息                                                                                                                                                                                                                                                                                                             |
| `tif`               | String         | 有效时间<br/>- gtc：GoodTillCancelled <br/>- ioc：ImmediateOrCancelled，仅接受者<br/>- poc：PendingOrCancelled，只进行后订单，始终享受挂单费用<br/>- fok： FillOrKill，完全填充或不填充<br/>type=market 时仅支持 ioc 和 fok                                                                                                                                    |
| `finish_time`       | Integer        | 订单结束 unix 时间戳（以秒为单位），未结束订单此字段返回0                                                                                                                                                                                                                                                                                    |
| `finish_time_ms`    | Integer        | 订单结束 unix 时间戳（以毫秒为单位），未结束订单此字段返回0                                                                                                                                                                                                                                                                                   |
| `user`              | String         | 用户 ID                                                                                                                                                                                                                                                                                                               |
| `contract`          | String         | 合约名称                                                                                                                                                                                                                                                                                                                |
| `stp_id`            | String         | 同一 stp_id 组内的用户之间的订单不允许自交易<br />1.如果匹配的两个订单的 stp_id 非零且相等，则不会被执行。而是根据 taker 的 stp_act 执行相应的策略。<br />2.对于未设置 STP 组的订单，stp_id 默认返回 0                                                                                                                                                                                  |
| `stp_act`           | String         | 自我交易预防行动。用户可以通过该字段设置自我交易防范策略<br />1.用户加入 STP Group 后，可以通过 stp_act 来限制用户的自我交易防范策略。如果不传 stp_act，则默认为 cn 策略。<br />2.当用户没有加入 STP 组时，传递 stp_act 参数时会返回错误。<br />3.如果用户下单时没有使用'stp_act'，'stp_act'将返回'-'<br />- cn: 取消最新订单，取消新订单并保留旧订单<br />- co: 取消最旧订单，取消旧订单并保留新订单<br />- cb：取消两者，新旧订单都会被取消                               |
| `amend_text`        | String         | 用户修改订单时备注的自定义数据                                                                                                                                                                                                                                                                                                     |
| `update_id`         | Integer        | 更新id                                                                                                                                                                                                                                                                                                                |
| `update_time`       | Integer        | 更新时间 (毫秒时间戳)                                                                                                                                                                                                                                                                                                        |
| `biz_info`          | String         | 用户可以备注这次修改的信息                                                                                                                                                                                                                                                                                                       |
| `stop_profit_price` | String         | 止盈价格                                                                                                                                                                                                                                                                                                                |
| `stop_loss_price`   | String         | 止损价格                                                                                                                                                                                                                                                                                                                |
| `market_order_slip_ratio`   | String         | 该笔市价单的预设的最大滑点比率（不代表实际的滑点），0.03代表3%的最大滑点                                                                                                                                                                                                                                                                             |
| `id_string`                 | String         | 订单 ID 字符串形式                                                                                                                                                                                                                                                                                                                  |
| `is_voucher`                | Bool           | 是否体验金订单                                                                                                                                                                                                                                                                                                                      |
| `role`                      | String         | 成交角色                                                                                                                                                                                                                                                                                                                            |
| `fee`                       | Float          | 手续费                                                                                                                                                                                                                                                                                                                              |
| `point_fee`                 | Float          | 点卡手续费                                                                                                                                                                                                                                                                                                                          |
| `bbo`                       | String         | BBO 下单选项                                                                                                                                                                                                                                                                                                                        |
| `pos_margin_mode`           | String         | 仓位保证金模式                                                                                                                                                                                                                                                                                                                      |
| `leverage`                  | String         | 杠杆（字符串）                                                                                                                                                                                                                                                                                                                      |
| `stop_profit`               | Object         | 止盈扩展信息，含 `tp`/`op`/`type`                                                                                                                                                                                                                                                                                                   |
| `stop_loss`                 | Object         | 止损扩展信息，含 `tp`/`op`/`type`                                                                                                                                                                                                                                                                                                   |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.orders",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.orders",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx", // replace with your API key
			"SIGN":   "xxxx", // replace with your generated signature
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.orders",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅订单更新通知**

### 请求参数

- channel

  `futures.orders`
- event

  `unsubscribe`

</Block>

<Block>

# 订单频道 V2

**支持环境：USDT 合约实盘和模拟盘。**

- 实盘：`wss://fx-ws.gateio.ws/v4/ws/usdt`
- 模拟盘：`wss://ws-testnet.gate.com/v4/ws/futures/usdt`

`futures.orders_v2` 实时推送撮合阶段、清算前的订单创建、成交、改单和撤销事件。仅支持 USDT 结算合约 API WebSocket 的 UTA3.0 用户，采用 JSON 编码；不支持 PB 和 SBE 订阅。

本频道需要认证，以已认证用户决定推送的订单，并检查该用户的 UTA3.0 资格。`payload` 中提供的用户 ID 不能用于切换到其他账户。

本频道不推送清算订单及清算信息。保留费率字段 `mkfr`，省略 `fee`、`point_fee`、`role`、`refu`、`refr`、`tkfr` 和 `update_time`；ME 时间使用 `update_time_ms`。

同一连接可以独立订阅 `futures.orders` 和 `futures.orders_v2`。不能根据两个频道的消息到达先后推断事件顺序。账户迁移至 UTA3.0 后，重新订阅即可使用本频道。

</Block>

<Block>

## 订单 V2 订阅

请求使用 `channel: "futures.orders_v2"`、`event: "subscribe"`；根据认证章节，使用本频道名称、事件和请求时间生成签名。

<Example>

```json
{
  "time": 1628736847,
  "channel": "futures.orders_v2",
  "event": "subscribe",
  "payload": [
    "BTC_USDT"
  ],
  "auth": {
    "method": "api_key",
    "KEY": "xxxx",
    "SIGN": "xxxx"
  }
}
```

</Example>

<Example>

```json
{
  "time": 1628736847,
  "time_ms": 1628736847320,
  "channel": "futures.orders_v2",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

订阅参数：

| 字段 | 类型 | 必选 | 描述 |
| --- | --- | --- | --- |
| `payload` | Array[String] | 是 | 合约列表，可在首项携带用户 ID。例如 `["BTC_USDT"]`、`["20011", "BTC_USDT", "ETH_USDT"]` 或 `["!all"]`。 |

列表中的单项也可以包含以逗号分隔的合约，例如 `"BTC_USDT,ETH_USDT"`。`!all` 表示订阅全部合约。多次订阅会累加合约；重复合约或与 `!all` 重叠的订阅不会重复推送。整批参数校验通过后才修改订阅，非法合约会使整个请求失败。

订阅成功只返回确认消息，不发送历史订单快照。订阅后的第一条消息可能是已有订单的更新。断线期间错过的事件不会补发。

订阅错误：

| 错误码 | 说明 |
| --- | --- |
| `2` | 非法用户 ID、非法合约、非 UTA3.0 账户或不支持的 PB/SBE 编码。 |
| `3` | 服务无法查询账户资格。 |
| `4` | 未认证或认证失败。 |

</Block>

<Block>

## 订单 V2 推送

全部订单事件均使用 `event: "update"`，`result` 为订单对象数组。

<Example>

```json
{
  "channel": "futures.orders_v2",
  "event": "update",
  "time": 1628736848,
  "time_ms": 1628736848325,
  "result": [
    {
      "contract": "BTC_USDT",
      "create_time": 1628736847,
      "create_time_ms": 1628736847325,
      "fill_price": "40000.4",
      "finish_as": "filled",
      "finish_time": 1628736848,
      "finish_time_ms": 1628736848321,
      "iceberg": "0",
      "id": 4872460,
      "id_string": "4872460",
      "is_close": false,
      "is_liq": false,
      "is_reduce_only": false,
      "is_voucher": false,
      "left": "0",
      "mkfr": -0.00025,
      "price": "40000.4",
      "size": "1.5",
      "status": "finished",
      "text": "api",
      "tif": "gtc",
      "user": "20011",
      "update_id": 1,
      "update_time_ms": 1628736848320,
      "biz_info": "-",
      "stp_act": "-",
      "stp_id": "0",
      "amend_text": "-",
      "bbo": "-",
      "pos_margin_mode": "cross",
      "leverage": "10",
      "stop_loss_price": "",
      "stop_profit_price": "",
      "market_order_slip_ratio": "0.03"
    }
  ]
}
```

</Example>

`size`、`left`、`iceberg`、`price`、`fill_price` 始终为十进制字符串，保留小数数量；是否设置 `X-Gate-Size-Decimal: 1` 不影响本频道的这些字段类型。

`create_time_ms` 来自 OMS。订单创建事件的 `update_time_ms` 为订单到达 ME 的时间，后续成交、改单和撤销事件则为 ME 更新订单的时间。“创建事件”不等于新订阅连接收到的第一条消息。后续事件的 `update_time_ms - create_time_ms` 包含订单存续时间，不能用来表示本次改单或撤销的 OMS→ME 延迟。

外层 `time` 和 `time_ms` 分别为 WS 消息封装的秒和毫秒时间戳；`finish_time` 和 `finish_time_ms` 沿用上游事件日志时间。这些时间不能替代 ME 的 `update_time_ms`。缺少有效正整数 OMS/ME 毫秒时间的消息不会推送，服务不会用本地时间补齐。

订单字段：

| 字段 | 类型 | 描述 |
| --- | --- | --- |
| `contract` | String | 合约名称。 |
| `create_time` | Integer | OMS 订单创建时间，单位为秒。 |
| `create_time_ms` | Integer | OMS 订单创建时间，单位为毫秒。 |
| `fill_price` | String | 订单成交价格，以十进制字符串表示。 |
| `finish_as` | String | 创建事件为 `_new`，未结束订单的更新事件为 `_update`。结束事件返回原因，例如 `filled`（完全成交）、`cancelled`（撤销）、`ioc`、`poc`、`fok` 或 `stp`；其他原因与 `futures.orders` 一致。 |
| `finish_time` | Integer | 上游订单事件日志时间，单位为秒，不表示清算完成时间。 |
| `finish_time_ms` | Integer | 上游订单事件日志时间，单位为毫秒，不表示 ME 更新时间。 |
| `iceberg` | String | 冰山订单显示数量，以十进制字符串表示；普通订单为 `"0"`。 |
| `id` | Integer | 订单 ID；客户端无法精确保留大整数时请使用 `id_string`。 |
| `id_string` | String | 字符串形式的订单 ID。 |
| `is_close` | Boolean | 是否为平仓订单。 |
| `is_liq` | Boolean | 本频道始终为 `false`，不推送清算订单。 |
| `is_reduce_only` | Boolean | 是否为只减仓订单。 |
| `is_voucher` | Boolean | 是否为体验金订单。 |
| `left` | String | 订单剩余数量，以十进制字符串表示。 |
| `mkfr` | Float | Maker 费率，不表示清算后的手续费金额。 |
| `price` | String | 订单价格，以十进制字符串表示；市价订单为 `"0"`。 |
| `size` | String | 订单数量，以十进制字符串表示；正数为买入，负数为卖出。 |
| `status` | String | 订单状态：`open`（等待交易）或 `finished`（已结束）。 |
| `text` | String | 用户定义的订单信息。 |
| `tif` | String | 订单有效期：`gtc`（成交前持续有效）、`ioc`（立即成交，否则取消）、`poc`（只挂单）或 `fok`（立即全部成交，否则取消）。 |
| `user` | String | 用户 ID。 |
| `update_id` | Integer | 订单更新 ID。 |
| `update_time_ms` | Integer | ME 毫秒时间戳。创建事件为订单到达 ME 的时间；成交、改单或撤销事件为 ME 更新订单的时间。 |
| `biz_info` | String | 本次订单更新附带的业务信息。 |
| `stp_act` | String | 自成交防护策略：`cn`（取消最新订单）、`co`（取消最旧订单）、`cb`（全部取消）；未设置为 `-`。 |
| `stp_id` | String | 自成交防护组 ID；未设置为 `"0"`。 |
| `amend_text` | String | 用户改单时附带的自定义信息。 |
| `bbo` | String | BBO 下单选项；未设置为 `-`。 |
| `pos_margin_mode` | String | 仓位保证金模式：`cross`（全仓）或 `isolated`（逐仓）。 |
| `leverage` | String | 杠杆倍数；上游未提供时为空字符串。 |
| `stop_loss_price` | String | 止损价格；未设置时为空字符串。 |
| `stop_profit_price` | String | 止盈价格；未设置时为空字符串。 |
| `market_order_slip_ratio` | String | 市价单预设最大滑点比例；`"0.03"` 表示 3%，不代表实际滑点。 |

</Block>

<Block>

## 订单 V2 取消订阅

使用 `event: "unsubscribe"`，参数规则与订阅一致。`payload: ["!all"]` 清空当前连接对 `futures.orders_v2` 的全部订阅。取消订阅不重新查询 UTA3.0 资格。

当前连接已订阅的合约即使下架，也仍可取消订阅；若请求中包含当前连接未订阅的未知合约，则整批请求失败，原有订阅保持不变。

<Example>

```json
{
  "time": 1628736849,
  "channel": "futures.orders_v2",
  "event": "unsubscribe",
  "payload": [
    "!all"
  ],
  "auth": {
    "method": "api_key",
    "KEY": "xxxx",
    "SIGN": "xxxx"
  }
}
```

</Example>

<Example>

```json
{
  "time": 1628736849,
  "time_ms": 1628736849123,
  "channel": "futures.orders_v2",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

</Block>

<Block>

# 用户私有成交频道

**提供接收用户交易的方式**

::: warning
需要认证
:::

</Block>

<Block>

## 用户私有成交订阅

如果您想订阅所有的市场交易更新，请在请求参数列表中使用`!all`。

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.usertrades",
  "event": "subscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.usertrades",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.usertrades",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅私有成交更新**

### 请求参数

- channel

  `futures.usertrades`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 用户私有成交推送

<Example>

```json
{
  "time": 1543205083,
  "time_ms": 1543205083123,
  "channel": "futures.usertrades",
  "event": "update",
  "result": [
    {
      "id": "3335259",
      "create_time": 1628736848,
      "create_time_ms": 1628736848321,
      "contract": "BTC_USDT",
      "order_id": "4872460",
      "size": "1",
      "price": "40000.4",
      "role": "maker",
      "text": "api",
      "fee": 0.0009290592,
      "point_fee": 0,
      "amend_text": "amend",
      "biz_info": "-",
      "close_size": "0"
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`size` 为整型：

```json
{
  "time": 1543205083,
  "time_ms": 1543205083123,
  "channel": "futures.usertrades",
  "event": "update",
  "result": [
    {
      "id": "3335259",
      "create_time": 1628736848,
      "create_time_ms": 1628736848321,
      "contract": "BTC_USDT",
      "order_id": "4872460",
      "size": 1,
      "price": "40000.4",
      "role": "maker",
      "text": "api",
      "fee": 0.0009290592,
      "point_fee": 0,
      "amend_text": "amend",
      "biz_info": "-",
      "close_size": "0"
    }
  ]
}
```

</Example>

**推送用户私有成交更新**

### 推送参数

- channel

  `futures.usertrades`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称             | 类型           | 描述                     |
| ---------------- | -------------- | ------------------------ |
| `contract`       | String         | 合约名称                 |
| `create_time`    | Integer        | 创建时间                 |
| `create_time_ms` | Integer        | 创建时间（以毫秒为单位） |
| `id`             | String         | 交易 ID                  |
| `order_id`       | String         | 订单 ID                  |
| `price`          | String         | 交易价格                 |
| `size`           | String/Integer | 交易数量                 |
| `role`           | String         | 用户角色 (maker/taker)   |
| `text`           | String         | 订单自定义信息，用户可以用该字段设置自定义 ID，用户自定义字段必须满足以下条件：<br><br>1. 必须以 `t-` 开头<br>2. 不计算 `t-` ，长度不能超过 28 字节<br>3. 输入内容只能包含数字、字母、下划线(_)、中划线(-) 或者点(.)<br><br>除用户自定义信息以外，以下为内部保留字段，标识订单来源:<br><br>- web: 网页<br>- api: API 调用<br>- app: 移动端<br>- auto_deleveraging: 自动减仓<br>- liquidation: 老经典模式仓位强制平仓<br>- liq-xxx: a. 新经典模式仓位强制平仓，包含逐仓、单向全仓、双向全仓非对冲仓位强平。 b. 统一账户单币种保证金模式逐仓强制平仓<br>- hedge-liq-xxx: 新经典模式双向全仓对冲部分强制平仓，即同时平多空仓位<br>- pm_liquidate: 统一账户跨币种保证金模式强制平仓<br>- comb_margin_liquidate: 统一账户组合保证金模式强制平仓<br>- scm_liquidate: 统一账户单币种保证金模式仓位强制平仓<br>- insurance: 保险<br>- clear: 合约下架清退 |
| `fee`            | Float          | 手续费                   |
| `point_fee`      | Float          | 点卡手续费               |
| `amend_text`     | String         | 改单备注                 |
| `biz_info`       | String         | 业务备注信息             |
| `close_size`     | String         | 平仓数量                 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.usertrades",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.usertrades",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.usertrades",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消私有成交订阅**

### 请求参数

- channel

  `futures.usertrades`
- event

  `unsubscribe`

</Block>

<Block>

# 强制平仓频道

**提供一种接收用户强制平仓信息的方式**

::: warning
需要认证
:::

</Block>

<Block>

## 清算订阅

如果您想订阅所有合约中的强制平仓推送，请在订阅请求列表中使用 `!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.liquidates",
  "event": "subscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.liquidates",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.liquidates",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户强制平仓推送**

### 请求参数

- channel

  `futures.liquidates`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 强制平仓推送

<Example>

```json
{
  "channel": "futures.liquidates",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "entry_price": 209,
      "fill_price": 215.1,
      "left": "0",
      "leverage": 0.0,
      "liq_price": 213,
      "margin": 0.007816722941,
      "mark_price": 213,
      "order_id": 4093362,
      "order_price": 215.1,
      "order_size": "-124",
      "size": "-124.5",
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USDT",
      "user": "1040xxxx"
    }
  ]
}
```

</Example>

**推送强制平仓更新**

### 推送参数

- channel

  `futures.liquidates`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称          | 类型           | 描述                 |
| ------------- | -------------- | -------------------- |
| `entry_price` | Float          | 平均入场价           |
| `fill_price`  | Float          | 平均执行价格         |
| `leverage`    | Float          | 杠杆大小             |
| `liq_price`   | Float          | 清算价格             |
| `margin`      | Float          | Margin               |
| `mark_price`  | Float          | 标记价格             |
| `order_id`    | Integer        | 订单 ID              |
| `order_price` | Float          | 订单价格             |
| `left`        | String/Integer | 订单未完成数量       |
| `order_size`  | String/Integer | 原始订单数量         |
| `size`        | String/Integer | 强平数量             |
| `time`        | Integer        | 时间                 |
| `time_ms`     | Integer        | 时间（以毫秒为单位） |
| `user`        | String         | 用户 ID              |
| `contract`    | String         | 合约名称             |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.liquidates",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.liquidates",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.liquidates",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅清算更新**

### 请求参数

- channel

  `futures.liquidates`
- event

  `unsubscribe`

</Block>

<Block>

# 自动减仓频道

**提供一种接收用户自动减仓信息的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 自动减仓订阅

如果您想订阅所有合约的自动减仓更新，请在请求参数列表中使用`!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.auto_deleverages",
  "event": "subscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.auto_deleverages",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.auto_deleverages",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户自动减仓更新**

### 请求参数

- channel

  `futures.auto_deleverages`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 自动减仓推送

<Example>

```json
{
  "channel": "futures.auto_deleverages",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "entry_price": 209,
      "fill_price": 215.1,
      "position_size": "10",
      "trade_size": "10",
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USDT",
      "user": "1040",
      "pos_margin_mode": "cross",
      "leverage": "10"
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`position_size`、`trade_size` 为整型：

```json
{
  "channel": "futures.auto_deleverages",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "entry_price": 209,
      "fill_price": 215.1,
      "position_size": 10,
      "trade_size": 10,
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USDT",
      "user": "1040",
      "pos_margin_mode": "cross",
      "leverage": "10"
    }
  ]
}
```

</Example>

**自动减仓消息**

### 推送参数

- channel

  `futures.auto_deleverages`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称            | 类型    | 描述                 |
| --------------- | ------- | -------------------- |
| `entry_price`   | Float   | 入场价格             |
| `fill_price`    | Float   | 执行价格             |
| `position_size` | String/Integer | 持仓规模             |
| `trade_size`    | String/Integer | 交易数量             |
| `pos_margin_mode` | String | 仓位保证金模式       |
| `leverage`      | String | 杠杆                 |
| `time`          | Integer | 时间                 |
| `time_ms`       | Integer | 时间（以毫秒为单位） |
| `user`          | String  | 用户 ID              |
| `contract`      | String  | 合约名称             |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.auto_deleverages",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.auto_deleverages",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.auto_deleverages",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅自动减仓**

### 请求参数

- channel

  `futures.auto_deleverages`
- event

  `unsubscribe`

</Block>

<Block>

# 平仓频道

**提供一种接收用户仓位平仓信息的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 平仓订阅

如果您想订阅所有合约的平仓更新，请在合约列表中使用 `!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
  "time": int(time.time()),
  "channel": "futures.position_closes",
  "event": "subscribe",
  "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_closes",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.position_closes",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户平仓信息更新**

### 请求参数

- channel

  `futures.position_closes`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 平仓信息推送

<Example>

```json
{
  "channel": "futures.position_closes",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "contract": "BTC_USDT",
      "pnl": -0.000624354791,
      "side": "long",
      "text": "web",
      "time": 1547198562,
      "time_ms": 1547198562123,
      "user": "211xxxx"
    }
  ]
}
```

</Example>

**平仓信息推送**

### 推送参数

- channel

  `futures.position_closes`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称       | 类型    | 描述                 |
| ---------- | ------- | -------------------- |
| `contract` | String  | 合约名称             |
| `pnl`      | Number  | 利润损失             |
| `side`     | String  | 方向 (long or short) |
| `text`     | String  | 附带信息             |
| `time`     | Integer | 时间                 |
| `time_ms`  | Integer | 时间（以毫秒为单位） |
| `user`     | String  | 用户 ID              |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.position_closes",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_closes",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.position_closes",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅平仓更新**

### 请求参数

- channel

  `futures.position_closes`
- event

  `unsubscribe`

</Block>

<Block>

# 余额频道

**提供一种接收用户余额信息的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 余额信息订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.balances",
		"event":   "subscribe",
		"payload": []interface{}{"20011"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.balances",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户余额更新**

### 请求参数

- channel

  `futures.balances`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述    |
| ---------- | ------ | ---- | ------- |
| `user id ` | String | 是   | 用户 ID |

</Block>

<Block>

## 余额更新推送

<Example>

```json
{
  "channel": "futures.balances",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "balance": 9.998739899488,
      "change": -0.000002074115,
      "text": "BTC_USDT:3914424",
      "time": 1547199246,
      "time_ms": 1547199246123,
      "type": "fee",
      "user": "211xxx",
      "currency": "btc"
    }
  ]
}
```

</Example>

**通知余额更新信息**

### 推送参数

- channel

  `futures.balances`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称       | 类型    | 描述                 |
| ---------- | ------- | -------------------- |
| `balance`  | Number  | 余额最终数量         |
| `change`   | Number  | 余额变化数量         |
| `text`     | String  | 附带信息             |
| `time`     | Integer | 时间                 |
| `time_ms`  | Integer | 时间（以毫秒为单位） |
| `type`     | String  | 变更类型: <br/> dnw: 出入金 <br/> pnl：盈亏 <br/> refr: 推荐人费用 <br/> fund: 资金费用 <br/> cross_settle: 统一账户结算 <br/> point_dnw: 点卡出入金 <br/> point_fee: 点卡手续费 <br/> point_refr: 点卡推荐人费用 <br/> bonus_dnw: 体验金出入金 <br/> pv_dnw: 仓位体验券充值提现 <br/> fee: 手续费                 |
| `user`     | String  | 用户 ID              |
| `currency` | String  | 币种                 |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.balances",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.balances",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

</Block>

<Block>

# 降低风险率频道

**推送用户降低风险率信息**

::: warning
需要认证
:::

</Block>

<Block>

## 降低风险率订阅

如果您想订阅所有合约的降低风险率更新，请在合约列表中使用 `!all`。

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.reduce_risk_limits",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.reduce_risk_limits",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx", // replace with your API key
			"SIGN":   "xxxx", // replace with your generated signature
		},
	}

	msg, err := json.Marshal(req)
	if err != nil {
		log.Fatal("json marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Fatal("write error:", err)
	}

	_, resp, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("read error:", err)
	}

	fmt.Println(string(resp))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.reduce_risk_limits",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户降低风险率更新**

### 请求参数

- channel

  `futures.reduce_risk_limits`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `user id ` | String | 是   | 用户 ID  |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 降低风险率推送

<Example>

```json
{
  "time": 1551858330,
  "time_ms": 1551858330123,
  "channel": "futures.reduce_risk_limits",
  "event": "update",
  "result": [
    {
      "cancel_orders": 0,
      "contract": "BTC_USDT",
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

**通知降低风险限制更新**

### 推送参数

- channel

  `futures.reduce_risk_limits`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称               | 类型    | 描述                 |
| ------------------ | ------- | -------------------- |
| `cancel_orders`    | Integer | Cancel orders        |
| `contract`         | String  | 合约名称             |
| `leverage_max`     | Integer | 最大杠杆             |
| `liq_price`        | Float   | 清算价格             |
| `maintenance_rate` | Float   | Maintenance rate     |
| `risk_limit`       | Integer | 风险限额             |
| `time`             | Integer | 时间                 |
| `time_ms`          | Integer | 时间（以毫秒为单位） |
| `user`             | String  | 用户 ID              |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.reduce_risk_limits",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("Dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.reduce_risk_limits",
		"event":   "unsubscribe",
		"payload": []string{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal("JSON Marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal("Write error:", err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("Read error:", err)
	}
	fmt.Println(string(msg))
}

```

</Example>

**退订降低风险限制更新**

### 请求参数

- channel

  `futures.reduce_risk_limits`
- event

  `unsubscribe`

</Block>

<Block>

# 仓位频道

**提供一种接收用户仓位信息的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 仓位订阅

如果您想订阅所有合约的持仓更新，请在合约列表中使用 `!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.positions",
    "event": "subscribe",
    "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("Dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.positions",
		"event":   "subscribe",
		"payload": []string{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal("JSON Marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal("Write error:", err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("Read error:", err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.positions",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户仓位更新**

### 请求参数

- channel

  `futures.positions`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述                                     |
| ---------- | ------ | ---- | ---------------------------------------- |
| `user id ` | String | 是   | 用户 ID （该字段已废弃，仅做占位符使用） |
| `contract` | String | 是   | 合约名称                                 |

</Block>

<Block>

## 仓位信息推送

<Example>

```json
{
  "time": 1588212926,
  "time_ms": 1588212926123,
  "channel": "futures.positions",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT",
      "cross_leverage_limit": 0,
      "entry_price": 40000.36666661111,
      "history_pnl": -0.000108569505,
      "history_point": 0,
      "last_close_pnl": -0.000050123368,
      "leverage": 0,
      "leverage_max": 100,
      "liq_price": 0.1,
      "maintenance_rate": 0.005,
      "margin": 49.999890611186,
      "mode": "single",
      "realised_pnl": -1.25e-8,
      "realised_point": 0,
      "risk_limit": 100,
      "size": "3",
      "time": 1628736848,
      "time_ms": 1628736848321,
      "user": "110xxxxx",
      "update_id": 170919,
      "pos_margin_mode": "cross",
      "lever": "10"
    }
  ]
}
```

</Example>

<Example>

未携带 `X-Gate-Size-Decimal: 1` 请求头时，`size` 为整型：

```json
{
  "time": 1588212926,
  "time_ms": 1588212926123,
  "channel": "futures.positions",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USDT",
      "cross_leverage_limit": 0,
      "entry_price": 40000.36666661111,
      "history_pnl": -0.000108569505,
      "history_point": 0,
      "last_close_pnl": -0.000050123368,
      "leverage": 0,
      "leverage_max": 100,
      "liq_price": 0.1,
      "maintenance_rate": 0.005,
      "margin": 49.999890611186,
      "mode": "single",
      "realised_pnl": -1.25e-8,
      "realised_point": 0,
      "risk_limit": 100,
      "size": 3,
      "time": 1628736848,
      "time_ms": 1628736848321,
      "user": "110xxxxx",
      "update_id": 170919,
      "pos_margin_mode": "cross",
      "lever": "10"
    }
  ]
}
```

</Example>

**推送仓位更新.**

### 推送参数

- channel

  `futures.positions`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称                   | 类型           | 描述                       |
| ---------------------- | -------------- |--------------------------|
| `contract`             | String         | 合约名称                     |
| `cross_leverage_limit` | Float          | 全仓模式下的杠杆倍数               |
| `entry_price`          | Float          | 开仓价格                     |
| `history_pnl`          | Float          | 已平仓的仓位总盈亏                |
| `history_point`        | Float          | 已平仓的点卡总盈亏                |
| `last_close_pnl`       | Float          | 最近一次平仓的盈亏                |
| `leverage`             | Integer        | 杠杆倍数，0代表全仓，正数代表逐仓        |
| `leverage_max`         | Integer        | 当前风险限额下，允许的最大杠杆倍数        |
| `liq_price`            | Float          | 已废弃                      |
| `maintenance_rate`     | Float          | 当前风险限额下，维持保证金比例          |
| `margin`               | Float          | 保证金                      |
| `realised_pnl`         | Float          | 已实现盈亏                    |
| `realised_point`       | Float          | 点卡已实现盈亏                  |
| `risk_limit`           | Integer        | 风险限额                     |
| `size`                 | String/Integer | 合约 size                  |
| `time`                 | Integer        | 更新 unix 时间戳              |
| `time_ms`              | Integer        | 更新 unix 时间戳（以毫秒为单位）      |
| `user`                 | String         | 用户 ID                    |
| `update_id`            | Integer        | 消息序列号，每次推送 order 之后会自增 1 |
| `pos_margin_mode`      | String         | 仓位保证金模式（逐仓/全仓等）           |
| `lever`                | String         | 杠杆信息（逐步替代 `leverage` 等字段）  |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.positions",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.positions",
		"event":   "unsubscribe",
		"payload": []string{"20011", "BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal(err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal(err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.positions",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅仓位更新**

### 请求参数

- channel

  `futures.positions`
- event

  `unsubscribe`

</Block>

<Block>

# 仓位 Adl 排名频道

**推送用户仓位 adl 排名信息的频道**

::: warning
需要认证
:::

</Block>

<Block>

## 仓位 adl 订阅

如果您想订阅所有合约的 adl 更新，请在合约列表中使用 `!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.position_adl_rank",
    "event": "subscribe",
    "payload": ["BTC_USDT"],
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

代码示例

```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("Dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_adl_rank",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal("JSON Marshal error:", err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal("Write error:", err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal("Read error:", err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.position_adl_rank",
  "event": "subscribe",
  "payload": [
    "BTC_USDT"
  ],
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户仓位 adl 更新**

### 请求参数

- channel

  `futures.position_adl_rank`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 仓位 adl 信息推送

<Example>

```json
{
    "time": 1588212926,
    "time_ms": 1588212926123,
    "channel": "futures.position_adl_rank",
    "event": "update",
    "result": [
        {
            "contract": "BTC_USDT",
            "mode": "single",
            "rank_division": 1,
            "time_ms": 1588212926119,
            "user_id": 2124426495
        }
    ]
}
```

</Example>

**推送仓位更新.**

### 推送参数

- channel

  `futures.position_adl_rank`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称            | 类型    | 描述                                                                                |
| --------------- | ------- | ----------------------------------------------------------------------------------- |
| `contract`      | String  | 合约名称                                                                            |
| `mode`          | String  | 持仓模式                                                                            |
| `rank_division` | Integer | 排名区间（1～6，1为最优先被ADL的区间，5为最靠后被ADL的区间。6:爆仓中不参与ADL排序） |
| `time_ms`       | Integer | 消息推送的毫秒时间戳                                                                |
| `user_id`       | Integer | 用户 ID                                                                             |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.position_adl_rank",
    "event": "unsubscribe",
    "payload": ["BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_adl_rank",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal(err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal(err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.position_adl_rank",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅仓位更新**

### 请求参数

- channel

  `futures.position_adl_rank`
- event

  `unsubscribe`

</Block>

<Block>

# 自动订单频道

**提供一种接收用户自动订单信息的方法**

::: warning
需要认证
:::

</Block>

<Block>

## 自动订单订阅

如果您想订阅所有合约中的自动订单更新，请在合约列表中使用`!all`

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.autoorders",
    "event": "subscribe",
    "payload": ["BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.autoorders",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal(err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal(err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.autoorders",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**订阅用户自动订单更新**

### 请求参数

- channel

  `futures.autoorders`
- event

  `subscribe`
- params


| 名称       | 类型   | 必选 | 描述     |
| ---------- | ------ | ---- | -------- |
| `contract` | String | 是   | 合约名称 |

</Block>

<Block>

## 自动订单消息推送

<Example>

```json
{
  "time": 1596798126,
  "time_ms": 1596798126123,
  "channel": "futures.autoorders",
  "event": "update",
  "result": [
    {
      "user": 123456,
      "trigger": {
        "strategy_type": 0,
        "price_type": 0,
        "price": "10000",
        "rule": 2,
        "expiration": 86400
      },
      "initial": {
        "contract": "BTC_USDT",
        "size": "10",
        "price": "10000",
        "tif": "gtc",
        "text": "web",
        "iceberg": "0",
        "is_close": false,
        "is_reduce_only": false,
        "auto_size": ""
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
      },
      "order_type": "close-long-order",
      "id_string": "9256",
      "me_order_id": "213867453823",
      "finish_time": 0,
      "finish_as": "",
      "parent_id": 0,
      "stop_profit_price": "",
      "stop_loss_price": "",
      "batch_id": "",
      "pos_margin_mode": "cross",
      "position_mode": "single",
      "leverage": "10",
      "mmr_trigger": {
        "rate": ""
      }
    }
  ]
}
```

</Example>

**通知自动订单更新**

### 推送参数

- channel

  `futures.autoorders`
- event

  `update`
- params


| 名称     | 类型  | 描述             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |


| 名称            | 类型    | 描述                             |
| --------------- | ------- | -------------------------------- |
| `user`          | Number  | 用户 ID                          |
| `trigger`       | Object  |                                  |
| `initial`       | Object  |                                  |
| `id`            | Number  | 自动订单 ID                      |
| `trade_id`      | Number  | 交易 ID                          |
| `status`        | String  | 订单状态                         |
| `reason`        | String  | 变更原因                         |
| `create_time`   | Number  | 创建时间                         |
| `name`          | String  | 名称                             |
| `is_stop_order` | boolean | 是否停止                         |
| `stop_trigger`  | Object  |                                  |
| `order_type`    | String  | 止盈/止损类型，详情参见 http api |
| `me_order_id`   | String  | 订单止盈/止损对应订单 ID         |
| `id_string`     | String  | 自动订单 ID 字符串形式           |
| `finish_time`   | Number  | 结束时间                         |
| `finish_as`     | String  | 结束方式                         |
| `parent_id`     | Number  | 父订单 ID                        |
| `stop_profit_price` | String | 止盈价                       |
| `stop_loss_price`   | String | 止损价                       |
| `batch_id`      | String  | 批次 ID                          |
| `pos_margin_mode` | String | 仓位保证金模式                  |
| `position_mode` | String  | 持仓模式                         |
| `leverage`      | String  | 杠杆                             |
| `mmr_trigger`   | Object  | MMR 触发配置                     |

</Block>

<Block>

## 取消订阅

<Example>

代码示例

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
req = {
    "time": int(time.time()),
    "channel": "futures.autoorders",
    "event": "unsubscribe",
    "payload": ["BTC_USDT"],
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

代码示例

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
  "time"
	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":     time.Now().Unix(),
		"channel": "futures.autoorders",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USDT"},
		"auth": map[string]string{
			"method": "api_key",
			"KEY":    "xxxx",
			"SIGN":   "xxxx",
		},
	}

	reqBytes, err := json.Marshal(req)
	if err != nil {
		log.Fatal(err)
	}

	err = conn.WriteMessage(websocket.TextMessage, reqBytes)
	if err != nil {
		log.Fatal(err)
	}

	_, msg, err := conn.ReadMessage()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(string(msg))
}

```

</Example>

<Example>

上面的命令返回 JSON 结构如下:

```json
{
  "time": 1545459681,
  "time_ms": 1545459681123,
  "channel": "futures.autoorders",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅自动订单更新**

### 请求参数

- channel

  `futures.autoorders`
- event

  `unsubscribe`

</Block>

<Block>

# 账户交易 API

</Block>

<Block>

## Websocket 交易 API

WebSocket API 允许通过 WebSocket 连接下单、取消、修改、查询订单.

</Block>

<Block>

### Websocket API 客户端请求

<Example>

代码示例

```python
#!/usr/bin/python

import time
import json
import hmac
import hashlib
import websocket
import threading


API_KEY = "xxxxx"
SECRET = "xxxxx"
WS_URL = "wss://fx-ws.gateio.ws/v4/ws/usdt"
CHANNEL_LOGIN = "futures.login"
CHANNEL_ORDER_PLACE = "futures.order_place"

def get_ts():
    return int(time.time())

def get_ts_ms():
    return int(time.time() * 1000)

def get_signature(secret, channel, request_param_bytes, ts):
    key = f"api\n{channel}\n{request_param_bytes.decode()}\n{ts}"
    return hmac.new(secret.encode(), key.encode(), hashlib.sha512).hexdigest()

def build_login_request():
    ts = get_ts()
    req_id = f"{get_ts_ms()}-1"
    request_param = b""

    sign = get_signature(SECRET, CHANNEL_LOGIN, request_param, ts)

    payload = {
        "api_key": API_KEY,
        "signature": sign,
        "timestamp": str(ts),
        "req_id": req_id
    }

    return {
        "time": ts,
        "channel": CHANNEL_LOGIN,
        "event": "api",
        "payload": payload
    }


def build_order_request():
    ts = get_ts()
    req_id = f"{get_ts_ms()}-2"
    order_param = {
        "contract": "BTC_USDT",
        "size": 6024,
        "iceberg": 0,
        "price": "3765",
        "tif": "gtc",
        "text": "t-my-custom-id",
        "stp_act": "-"
    }


    payload = {
        "req_id": req_id,
        "req_param": order_param
    }

    return {
        "time": ts,
        "channel": CHANNEL_ORDER_PLACE,
        "event": "api",
        "payload": payload
    }


def on_message(ws, message):
    print(f"recv: {message}")

def on_error(ws, error):
    print(f"error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("connection closed")

def on_open(ws):
    print("WebSocket opened")

    login_payload = build_login_request()
    print("login payload:", login_payload)
    ws.send(json.dumps(login_payload))

    def delayed_order():
        time.sleep(2)
        order_payload = build_order_request()
        print("order payload:", order_payload)
        ws.send(json.dumps(order_payload))

    threading.Thread(target=delayed_order).start()

custom_headers = {
    "X-Gate-Size-Decimal": "1"
}

if __name__ == "__main__":
    ws = websocket.WebSocketApp(
        WS_URL,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open,
        header=custom_headers
    )
    ws.run_forever()

```

</Example>

<Example>

代码示例

```go
package main

import (
	"crypto/hmac"
	"crypto/sha512"
	"crypto/tls"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
	"time"

	"github.com/gorilla/websocket"
)

func GetApiSignature(secret, channel string, requestParam []byte, ts int64) string {
	hash := hmac.New(sha512.New, []byte(secret))
	key := fmt.Sprintf("%s\n%s\n%s\n%d", "api", channel, string(requestParam), ts)
	hash.Write([]byte(key))
	return hex.EncodeToString(hash.Sum(nil))
}

func main() {

	// 1. login
	apiKey := "xxxxx"
	secret := "xxxxx"
	requestParam := ""
	channel := "futures.login"
	ts := time.Now().Unix()
	requestId := fmt.Sprintf("%d-%d", time.Now().UnixMilli(), 1)

	req := ApiRequest{
		Time:    ts,
		Channel: "futures.login",
		Event:   "api",
		Payload: ApiPayload{
			ApiKey:       apiKey,
			Signature:    GetApiSignature(secret, channel, []byte(requestParam), ts),
			Timestamp:    strconv.FormatInt(ts, 10),
			RequestId:    requestId,
			RequestParam: []byte(requestParam),
		},
	}

	fmt.Println(GetApiSignature(secret, channel, []byte(requestParam), ts))
	marshal, _ := json.Marshal(req)
	fmt.Println(string(marshal))

	// connect the ws
	u := url.URL{Scheme: "wss", Host: "fx-ws.gateio.ws", Path: "/v4/ws/usdt"}
	websocket.DefaultDialer.TLSClientConfig = &tls.Config{RootCAs: nil, InsecureSkipVerify: true}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

	err = c.WriteMessage(websocket.TextMessage, marshal)
	if err != nil {
		panic(err)
	}
	time.Sleep(2 * time.Second)

	//ws create an order
	orderParam := OrderParam{
		Contract: "BTC_USDT",
		Size:     "6024",
		Iceberg:  0,
		Price:    "3765",
		Tif:      "gtc",
		Text:     "t-my-custom-id",
		StpAct:   false,
	}
	orderParamBytes, _ := json.Marshal(orderParam)

	order_place := ApiRequest{
		Time:    ts,
		Channel: "futures.order_place",
		Event:   "api",
		Payload: ApiPayload{
			RequestId:    requestId,
			RequestParam: []byte(orderParamBytes),
		},
	}
	orderReqByte, _ := json.Marshal(order_place)
	err = c.WriteMessage(websocket.TextMessage, orderReqByte)

	if err != nil {
		panic(err)
	}

	select {}
}

type ApiRequest struct {
	App     string     `json:"app,omitempty"`
	Time    int64      `json:"time"`
	Id      *int64     `json:"id,omitempty"`
	Channel string     `json:"channel"`
	Event   string     `json:"event"`
	Payload ApiPayload `json:"payload"`
}
type ApiPayload struct {
	ApiKey       string          `json:"api_key,omitempty"`
	Signature    string          `json:"signature,omitempty"`
	Timestamp    string          `json:"timestamp,omitempty"`
	RequestId    string          `json:"req_id,omitempty"`
	RequestParam json.RawMessage `json:"req_param,omitempty"`
}

type OrderParam struct {
	Contract   string `json:"contract"`
	Size       int64  `json:"size,omitempty"`
	Iceberg    int64  `json:"iceberg,omitempty"`
	Price      string `json:"price,omitempty"`
	Close      bool   `json:"close,omitempty"`
	ReduceOnly bool   `json:"reduce_only,omitempty"`
	Tif        string `json:"tif,omitempty"`
	Text       string `json:"text,omitempty"`
	AutoSize   string `json:"auto_size,omitempty"`
	StpAct     bool   `json:"stp_act,omitempty"`
}

```

</Example>

<Example>

```json
{
  "time": 1680772890,
  "channel": "futures.order_place",
  "event": "api",
  "payload": {
    "req_id": "xxxx",
    "req_param": {
      "contract": "BTC_USDT",
      "size": "10",
      "price": "80048.240000",
      "tif": "gtc",
      "text": "t-my-custom-id"
    }
  }
}
```

</Example>

客户端发起的 api 请求遵循通用的 JSON 格式，
包含以下字段:


| 名称          | 类型    | 必选 | 描述                                                                                                                       |
| ------------- | ------- | ---- | -------------------------------------------------------------------------------------------------------------------------- |
| `time`        | Integer | 是   | 请求时间（以秒为单位）。请求时间和服务器时间之间的差距不得超过 60 秒                                                       |
| `id`          | Integer | 否   | 可选的请求 ID，将由服务器发回，以帮助您识别服务器响应哪个请求                                                              |
| `channel`     | String  | 是   | 要访问的 WebSocket 频道                                                                                                    |
| `event`       | String  | 是   | 固定为`api`                                                                                                                |
| `payload`     | Object  | 是   | 可选请求详细参数                                                                                                           |
| »`req_id`    | String  | 是   | 消息的唯一标识符由客户端提供，将在响应消息中返回，用于标识相应的请求。                                                     |
| »`timestamp` | String  | 是   | 签名时间（秒）                                                                                                             |
| »`api_key`   | String  | 是   | Gate APIv4 APIKey                                                                                                          |
| »`signature` | String  | 是   | 使用 GateAPIv4 密钥和请求信息生成的身份验证签名，<br />详细信息请参见[Websocket API 身份验证](#Websocket API 身份验证)部分 |
| »`req_param` | []Byte  | 是   | 请求 api 参数                                                                                                              |

请注意，`payload.req_param` 的类型是与频道（`channel`字段）绑定的，频道不同`payload.req_param`
的字段也不同，以 `futures.order_place` 为例，`payload.req_param` 与
apiv4 [/futures/{settle}/orders](https://www.gate.com/docs/developers/apiv4/en/#create-a-futures-order)。
例如，您可以对 BTC_USDT 下限价单

</Block>

<Block>

### Websocket API 服务响应

<Example>

服务器回声确认响应示例（目前仅在下单请求中有回声响应）

```json
{
  "request_id": "request-id-1",
  "ack": true,
  "header": {
    "response_time": "1681195121499",
    "status": "200",
    "channel": "futures.order_place",
    "event": "api",
    "client_id": "::1-0x140031563c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gate_ratelimit_reset_timestamp": 1681195121499
  },
  "data": {
    "result": {
      "req_id": "request-id-1",
      "req_param": {
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31503.280000",
        "tif": "gtc",
        "text": "t-my-custom-id"
      }
    }
  }
}
```

服务器 API 响应示例

```json
{
  "request_id": "request-id-1",
  "ack": false,
  "header": {
    "response_time": "1681195121639",
    "status": "200",
    "channel": "futures.order_place",
    "event": "api",
    "client_id": "::1-0x140031563c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gate_ratelimit_reset_timestamp": 1681195121639
  },
  "data": {
    "result": {
      "id": 74046511,
      "user": 6790020,
      "create_time": 1681195121.754,
      "finish_time": 1681195121.754,
      "finish_as": "filled",
      "status": "finished",
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31503.3",
      "tif": "gtc",
      "fill_price": "31500",
      "text": "t-my-custom-id",
      "tkfr": "0.0003",
      "mkfr": "0",
      "stp_id": 2,
      "stp_act": "cn",
      "amend_text": "-"
    }
  }
}
```

</Example>

服务器响应包括对客户端请求的 ack 响应和 api 结果消息推送。
服务器响应遵循通用的 JSON 格式，其中包含以下字段:


| 名称                                 | 类型    | 描述                                                                                                                                                                   |
| ------------------------------------ | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                                          |
| `header`                             | Map     | 响应元信息                                                                                                                                                             |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                                                     |
| »`channel`                          | String  | 请求频道                                                                                                                                                               |
| »`event`                            | String  | 请求`event`                                                                                                                                                            |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                                        |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                                      |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                                      |
| »`x_gate_ratelimit_requests_remain` | Integer | (仅涉及下单/改单/撤单)当前时间窗口剩余可用请求数(为0不展示)                                                                                                            |
| »`x_gate_ratelimit_limit`           | Integer | (仅涉及下单/改单/撤单)当前频率限制上限(为0不展示)                                                                                                                      |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | (仅涉及下单/改单/撤单)已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒） |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                                 |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                                              |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                                                  |
| `data`                               | Object  | 请求响应的数据                                                                                                                                                         |
| »`result`                           | Object  | 如果这是 ack 响应，则结果是请求的`payload`，否则结果是 api 的响应                                                                                                      |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                                                     |
| »»`label`                          | String  | 错误类型                                                                                                                                                               |
| »»`message`                        | String  | 详细错误信息                                                                                                                                                           |

</Block>

<Block>

### 错误

<Example>

错误响应通知示例

```json
{
  "request_id": "request-id-1",
  "ack": false,
  "header": {
    "response_time": "1681195360034",
    "status": "401",
    "channel": "futures.login",
    "event": "api",
    "client_id": "::1-0x140001a2600",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "errs": {
      "label": "INVALID_KEY",
      "message": "Invalid key provided"
    }
  }
}
```

Error 推送示例（限速）

```json
{
  "request_id": "xxxx",
  "header": {
    "response_time": "1677816784084",
    "status": "429",
    "channel": "futures.order_place",
    "event": "api",
    "client_id": "::1-0x14002ba2300",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_limit": 100,
    "x_gate_ratelimit_reset_timestamp": 1677816785084
  },
  "data": {
    "errs": {
      "label": "TOO_MANY_REQUESTS",
      "message": "Request Rate limit Exceeded (311)"
    }
  }
}
```

</Example>

错误响应详情具有以下格式:

<a id="schema_error"></a>


| 名称      | 类型   | 描述         |
| --------- | ------ | ------------ |
| `label`   | String | 错误类型     |
| `message` | String | 详细错误信息 |

限频相关的错误码说明:


| 错误码 | 描述             |
| ------ | ---------------- |
| `100`  | 限流内部异常错误 |
| `311`  | 合约限流         |
| `312`  | 合约成交比率限流 |

</Block>

<Block>

## 登录

::: warning
注意：您使用的 GateAPIv4 密钥对必须具有合约账户对应的权限（例如：order-place 频道必须具有合约账户写入权限），
如果启用了密钥的白名单，则您的出站 IP 地址必须在密钥的 IP 白名单中.
:::

</Block>

<Block>

### 登录请求

客户端 API 请求

<Example>

代码示例

```python
import hmac
import hashlib
import json
import time
import websocket
import ssl

def get_api_signature(secret, channel, request_param, ts):
    key = f"api\n{channel}\n{request_param}\n{ts}"
    hash_object = hmac.new(secret.encode(), key.encode(), hashlib.sha512)
    return hash_object.hexdigest()

class ApiPayload:
    def __init__(self, api_key, signature, timestamp, req_id, request_param):
        self.api_key = api_key
        self.signature = signature
        self.timestamp = timestamp
        self.req_id = req_id
        self.request_param = request_param

class ApiRequest:
    def __init__(self, ts, channel, event, payload):
        self.time = ts
        self.channel = channel
        self.event = event
        self.payload = payload

def main():
    api_key = "YOUR_API_KEY"
    secret = "YOUR_API_SECRET"
    request_param = ""
    channel = "futures.login"
    ts = int(time.time())
    request_id = f"{int(time.time() * 1000)}-1"

    payload = ApiPayload(
        api_key=api_key,
        signature=get_api_signature(secret, channel, request_param, ts),
        timestamp=str(ts),
        req_id=request_id,
        request_param=request_param
    )

    req = ApiRequest(ts=ts, channel=channel, event="api", payload=payload)

    print(get_api_signature(secret, channel, request_param, ts))

    req_json = json.dumps(req, default=lambda o: o.__dict__)
    print(req_json)

    # Connect to WebSocket
    ws_url = "wss://fx-ws.gateio.ws/v4/ws/usdt"  # Replace with your WebSocket URL
    websocket.enableTrace(False)
    ws = websocket.create_connection(ws_url, sslopt={"cert_reqs": ssl.CERT_NONE})

    # Function to receive messages
    def recv_messages():
        while True:
            try:
                message = ws.recv()
                print(f"recv: {message}")
            except Exception as e:
                print(f"Error receiving message: {e}")
                ws.close()
                break

    # Start receiving messages in a separate thread
    import threading
    receive_thread = threading.Thread(target=recv_messages)
    receive_thread.start()

    # Send the request
    ws.send(req_json)

    # Keep the main thread running
    receive_thread.join()

if __name__ == "__main__":
    main()
```

</Example>

<Example>

代码示例

```go
package main

import (
	"crypto/hmac"
	"crypto/sha512"
	"crypto/tls"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/gorilla/websocket"
	"net/url"
	"strconv"
	"time"
)

func GetApiSignature(secret, channel string, requestParam []byte, ts int64) string {
	hash := hmac.New(sha512.New, []byte(secret))
	key := fmt.Sprintf("%s\n%s\n%s\n%d", "api", channel, string(requestParam), ts)
	hash.Write([]byte(key))
	return hex.EncodeToString(hash.Sum(nil))
}

// example WebSocket signature calculation implementation in go
func main() {
	apiKey := "YOUR_API_KEY"
	secret := "YOUR_API_SECRET"
	requestParam := ""
	channel := "futures.login"
	ts := time.Now().Unix()
	requestId := fmt.Sprintf("%d-%d", time.Now().UnixMilli(), 1)

	req := ApiRequest{
		Time:    ts,
		Channel: channel,
		Event:   "api",
		Payload: ApiPayload{
			ApiKey:       apiKey,
			Signature:    GetApiSignature(secret, channel, []byte(requestParam), ts),
			Timestamp:    strconv.FormatInt(ts, 10),
			RequestId:    requestId,
			RequestParam: []byte(requestParam),
		},
	}

	fmt.Println(GetApiSignature(secret, channel, []byte(requestParam), ts))

	marshal, _ := json.Marshal(req)
	fmt.Println(string(marshal))

	// connect the ws
	u := url.URL{Scheme: "wss", Host: "fx-ws.gateio.ws", Path: "/v4/ws/usdt"}
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

	err = c.WriteMessage(websocket.TextMessage, marshal)
	if err != nil {
		panic(err)
	}

	select {}
}

type ApiRequest struct {
	App     string     `json:"app,omitempty"`
	Time    int64      `json:"time"`
	Id      *int64     `json:"id,omitempty"`
	Channel string     `json:"channel"`
	Event   string     `json:"event"`
	Payload ApiPayload `json:"payload"`
}
type ApiPayload struct {
	ApiKey       string          `json:"api_key,omitempty"`
	Signature    string          `json:"signature,omitempty"`
	Timestamp    string          `json:"timestamp,omitempty"`
	RequestId    string          `json:"req_id,omitempty"`
	RequestParam json.RawMessage `json:"req_param,omitempty"`
}
```

</Example>

<Example>

请求示例

```json
{
  "time": 1681984544,
  "channel": "futures.login",
  "event": "api",
  "payload": {
    "api_key": "ea83fad2604399da16bf97e6eea772a6",
    "signature": "6fa3824c8141f2b2283108558ec50966d7caf749bf04a3b604652325b50b47d2343d569d848373d58e65c49d9622ba2e73dc25797abef11c9f20c07da741591e",
    "timestamp": "1681984544",
    "req_id": "request-1"
  }
}
```

</Example>

`payload`参数:


| 名称         | 类型     | 必选 | 描述                                                                              |
| ------------ | -------- | ---- | --------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 请求 id，将由服务器发回，以帮助您识别服务器响应哪个请求，<br />它与外部的`id`不同 |
| `api_key`    | `string` | 是   | Apiv4 key                                                                         |
| `req_header` | `object` | 是   | Apiv4 自定义 header                                                               |
| `signature`  | `string` | 是   | Apiv4 签名                                                                        |
| `timestamp`  | `string` | 是   | Unix 时间戳（以秒为单位）                                                         |

WebSocket api 操作认证使用与 Gate APIv4 API 相同的签名计算方法，即
`HexEncode(HMAC_SHA512(secret,signature_string))`，但有以下区别：

1. 签名字符串拼接方式：`<event>\n<channel>\n<req_param>\n<timestamp>`,
   其中`<event>`、`<channel>`、`<req_param>`、`<timestamp>`是对应的请求信息
2. `login`频道中的 `req_param`始终为空字符串
3. 身份验证信息在请求正文中的`payload`字段中发送。

</Block>

<Block>

### 登录响应

<Example>

登录响应示例

```json
{
  "request_id": "request-1",
  "header": {
    "response_time": "1681985856666",
    "status": "200",
    "channel": "futures.login",
    "event": "api",
    "clientId": "",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": {
      "api_key": "ea83fad2604399da16bf97e6eea772a6",
      "uid": "110284739"
    }
  }
}
```

</Example>

响应参数:


| 名称              | 类型    | 描述                                                              |
| ----------------- | ------- | ----------------------------------------------------------------- |
| `request_id`      | String  | 对应的请求 ID                                                     |
| `header`          | Map     | 响应元信息                                                        |
| »`response_time` | String  | 响应发送时间(毫秒)                                                |
| »`channel`       | String  | 请求频道                                                          |
| »`event`         | String  | 请求`event`                                                       |
| »`client_id`     | String  | 唯一的客户端 ID                                                   |
| »`x_in_time`     | Integer | ws 接收请求的时间（以微秒为单位）                                 |
| »`x_out_time`    | Integer | ws 返回响应的时间（以微秒为单位）                                 |
| »`conn_id`       | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）            |
| »`conn_trace_id` | String  | 与客户端建立连接的TraceId                                         |
| »`trace_id`      | String  | 执行下单操作的TraceId                                             |
| `data`            | Object  | 请求响应的数据                                                    |
| »`result`        | Object  | 如果这是 ack 响应，则结果是请求的`payload`，否则结果是 api 的响应 |
| »»`api_key`     | String  | 登录成功的 apikey                                                 |
| »»`uid`         | String  | 登录成功的用户 ID                                                 |
| »`errs`          | Object  | 仅当请求失败时可用                                                |
| »»`label`       | String  | 错误类型                                                          |
| »»`message`     | String  | 详细错误信息                                                      |

</Block>

<Block>

## 下单

`futures.order_place`

您可以通过此频道进行下单操作.

**本频道和以下的 APIV4 功能相同:**

```json
POST /futures/{
  settle
}/orders
```

</Block>

<Block>

### 下单请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

api_order = {
      "contract": "BTC_USDT",
      "size": 10,
      "price": "31503.280000",
      "tif": "gtc",
      "text": "t-my-custom-id"
    }

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(json.dumps(
    {"time": int(time.time()),
    "channel": "futures.order_place",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_order
    }}
))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
    "crypto/hmac"
    "crypto/sha512"
    "crypto/tls"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "github.com/gorilla/websocket"
	"net/http"
    "net/url"
    "strconv"
    "time"
)

func main() {

	// warn: you should login first before order

	// connect the ws
	u := url.URL{Scheme: "ws", Host: "xx.xx.xxx.xx:xxx", Path: "xxx"}
	websocket.DefaultDialer.TLSClientConfig = &tls.Config{RootCAs: nil, InsecureSkipVerify: true}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

	err = c.WriteMessage(websocket.TextMessage, marshal)

	//ws create an order
	orderParam := orderParam{
		Contract: "BTC_USDT",
		size:     6024,
		Iceberg:  0,
		Price:    "3765",
		Tif:      "gtc",
		Text:     "t-my-custom-id",
		Stp_act:  "-",
	}
	orderParamBytes, _ := json.Marshal(orderParam)
	requestId := fmt.Sprintf("%d-%d", time.Now().UnixMilli(), 1)
	order_place := ApiRequest{
		Time:    time.Now().Unix(),
		Channel: "futures.order_place",
		Event:   "api",
		Payload: ApiPayload{
			RequestId:    requestId,
			RequestParam: []byte(orderParamBytes),
		},
	}
	orderReqByte, _ := json.Marshal(order_place)
	err = c.WriteMessage(websocket.TextMessage, orderReqByte)

	if err != nil {
		panic(err)
	}

	select {}
}
```

</Example>

<Example>

请求示例

```json
{
  "time": 1681195484,
  "channel": "futures.order_place",
  "event": "api",
  "payload": {
    "req_id": "request-id-1",
    "req_param": {
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31503.280000",
      "tif": "gtc",
      "text": "t-my-custom-id"
    }
  }
}
```

</Example>

`payload`参数:


| 名称         | 类型     | 必选 | 描述                                                                                                                 |
| ------------ | -------- | ---- | -------------------------------------------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同                                    |
| `req_param`  | `object` | 是   | 使用 api 下单参数； api 下单参数详细信息[api](https://www.gate.com/docs/developers/apiv4/en/#create-a-futures-order) |
| `req_header` | `object` | 否   | Apiv4 自定义请求头                                                                                                   |

`req_param` API 订单模型的 JSON 字节数据:


| 字段          | 类型     | 必选 | 描述                                                                                            |
| ------------- | -------- | ---- | ----------------------------------------------------------------------------------------------- |
| `contract`    | `string` | 是   | 合约                                                                                            |
| `size`        | `int64`  | 是   | 订单大小。指定正数进行出价，指定负数进行询问                                                    |
| `iceberg`     | `string`  | 否   | 冰山订单的显示尺寸。0 表示非冰山。请注意，您需要支付隐藏尺寸的接受者费用                        |
| `price`       | `string` | 否   | 订单价格。0 表示市价订单，`tif`设置为`ioc`                                                      |
| `close`       | `bool`   | 否   | 设置为`true`平仓，`size`设置为 0                                                                |
| `reduce_only` | `bool`   | 否   | 设置为`true`仅减少订单                                                                          |
| `tif`         | `string` | 否   | 有效时间                                                                                        |
| `text`        | `string` | 否   | 用户定义的信息。如果不为空，则必须遵循以下规则：                                                |
| `auto_size`   | `string` | 否   | 将侧面设置为关闭双模式位置。`close_long`闭合长边；而`close_short`短的。注意`size`还需要设置为 0 |
| `stp_act`     | `string` | 否   | 自我交易预防行动。用户可以通过该字段设置自助交易防范策略                                        |
| `market_order_slip_ratio`   | `string` | 否        | 该笔市价单的预设的最大滑点比率（不代表实际的滑点），0.03代表3%的最大滑点                                                                                                                                                                                                                                                                              |

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

#### 详细描述

**tif**: 有效时间

- gtc：取消前有效
- ioc: ImmediateOrCancelled, 仅接受者
- poc：PendingOrCancelled，仅发布订单，始终享受挂单费用
- fok：FillOrKill，完全填充或不填充

**text**: 订单自定义信息，用户可以用该字段设置自定义 ID，用户自定义字段必须满足以下条件：

- 必须以 t- 开头
- 不计算 t- ，长度不能超过 28 字节
- 输入内容只能包含数字、字母、下划线(\_)、中划线(-) 或者点(.)
- 不填，默认 `apiv4-ws`,来自 ws
- web：来自网络
- api：来自 API
- app：来自手机
- auto_deleveraging：来自 ADL
- liquidation：来自清算
- insurance：来自保险

**stp_act**：自我交易预防行动。用户可以通过该字段设置自助交易防范策略
用户加入后`STP Group`，他可以通过`stp_act`限制用户的自我交易防范策略。如果`stp_act`不通过则默认为`cn`策略。
当用户没有加入时`STP group`，传递参数时会返回错误`stp_act`。
如果用户下单时没有使用`stp_act`，`stp_act`将返回`-`

- cn：取消最新订单，取消新订单并保留旧订单
- co：取消最旧的订单，取消旧订单并保留新订单
- cb：取消两者，新旧订单都会被取消

</Block>

<Block>

### 订单请求回声消息

<Example>

订单确认回声通知示例

```json
{
  "request_id": "request-id-1",
  "ack": true,
  "header": {
    "response_time": "1681195484268",
    "status": "200",
    "channel": "futures.order_place",
    "event": "api",
    "client_id": "::1-0x140001a2600",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "req_id": "request-id-1",
      "req_header": null,
      "req_param": {
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31503.280000",
        "tif": "gtc",
        "text": "t-my-custom-id"
      }
    }
  }
}
```

</Example>

</Block>

<Block>

### 下单结果通知

<Example>

响应返回示例

```json
{
  "request_id": "request-id-1",
  "ack": false,
  "header": {
    "response_time": "1681195484360",
    "status": "200",
    "channel": "futures.order_place",
    "event": "api",
    "client_id": "::1-0x140001a2600",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": 74046514,
      "user": 6790020,
      "create_time": 1681195484.462,
      "finish_time": 1681195484.462,
      "finish_as": "filled",
      "status": "finished",
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31503.3",
      "tif": "gtc",
      "fill_price": "31500",
      "text": "t-my-custom-id",
      "tkfr": "0.0003",
      "mkfr": "0",
      "stp_id": 2,
      "stp_act": "cn",
      "amend_text": "-"
    }
  }
}
```

</Example>

下单时返回订单信息
响应参数:


| 名称                                 | 类型    | 描述                                                                                                                                                                                            |
| ------------------------------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                                                                   |
| `ack`                                | Bool    | "ack"消息的返回表示 WebSocket 的确认消息(目前在下单接口中存在)。<br />如果`ack`为 false(false 该字段不会出现在响应中)，则说明该消息是响应消息，可以判断请求是否成功<br / >通过检查`data.errs`。 |
| `header`                             | Map     | 响应元信息                                                                                                                                                                                      |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                                                                              |
| »`channel`                          | String  | 请求频道                                                                                                                                                                                        |
| »`event`                            | String  | 请求`event`                                                                                                                                                                                     |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                                                                 |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                                                               |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                                                               |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                                                                       |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                                                                           |
| »`x_gate_ratelimit_requests_remain` | Integer | 当前时间窗口剩余可用请求数(为0不展示)                                                                                                                                                           |
| »`x_gate_ratelimit_limit`           | Integer | 当前频率限制上限(为0不展示)                                                                                                                                                                     |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒）                                                |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                                                          |
| `data`                               | Object  | 请求响应的数据                                                                                                                                                                                  |
| »`result`                           | Object  | 如果这是 ack 响应，则结果是请求的`payload`，否则结果是 api 的响应                                                                                                                               |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                                                                              |
| »»`label`                          | String  | 错误类型                                                                                                                                                                                        |
| »»`message`                        | String  | 详细错误信息                                                                                                                                                                                    |

</Block>

<Block>

## 批量下单

`futures.order_batch_place`

您可以通过该频道批量下单

**本频道和以下的 APIV4 功能相同:**

```json
POST /futures/{
  settle
}/batch_orders
```

</Block>

<Block>

### 批量下单请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

api_order=[
      {
        "contract": "BTC_USDT",
        "size": 10,
        "price": "31403.180000",
        "tif": "gtc",
        "text": "t-my-custom-id"
      }
    ]

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_batch_place",
    "event": "api",
    "payload": {
        "header":{
            "x-gate-channel-id":"xxxx",
        },
        "req_id": "1ewq-3123w-5",
        "req_param": api_order
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	apiOrder := []map[string]interface{}{
		{
			"contract": "BTC_USDT",
			"size":     10,
			"price":    "31403.180000",
			"tif":      "gtc",
			"text":     "t-my-custom-id",
		},
	}

	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_batch_place",
		"event":   "api",
		"payload": map[string]interface{}{
			"header": map[string]interface{}{
				"x-gate-channel-id": "xxxx",
			},
			"req_id":    "1ewq-3123w-5",
			"req_param": apiOrder,
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}


```

</Example>

<Example>

请求示例

```json
{
  "time": 1681196536,
  "channel": "futures.order_batch_place",
  "event": "api",
  "payload": {
    "req_id": "request-id-6",
    "req_param": [
      {
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31403.180000",
        "tif": "gtc",
        "text": "t-my-custom-id"
      }
    ]
  }
}
```

</Example>

`payload` 参数:


| 名称         | 类型     | 必选 | 描述                                                                                                                                 |
| ------------ | -------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `req_id`     | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同                                                    |
| `req_param`  | `object` | 是   | 参考 api 批量下单的请求数组； api 批量下单详情[api](https://www.gate.com/docs/developers/apiv4/en/#create-a-batch-of-futures-orders) |
| `req_header` | `object` | 否   | Apiv4 自定义 header                                                                                                                  |

`req_param` API 订单模型的 JSON 字节数据可以参考单个下单，是多个单个下单数组，详情参考下单

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

</Block>

<Block>

### 批量下单确认通知

<Example>

确认通知示例

```json
{
  "request_id": "request-id-6",
  "ack": true,
  "header": {
    "response_time": "1681196536283",
    "status": "200",
    "channel": "futures.order_batch_place",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "req_id": "request-id-6",
      "req_header": null,
      "req_param": [
        {
          "contract": "BTC_USDT",
          "size": "10",
          "price": "31403.180000",
          "tif": "gtc",
          "text": "t-my-custom-id"
        }
      ]
    }
  }
}
```

</Example>

</Block>

<Block>

### 批量下单结果通知

<Example>

响应返回示例

```json
{
  "request_id": "request-id-6",
  "ack": false,
  "header": {
    "response_time": "1681196536532",
    "status": "200",
    "channel": "futures.order_batch_place",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": [
      {
        "succeeded": true,
        "id": 74046545,
        "user": 6790020,
        "create_time": 1681196536.592,
        "status": "open",
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31403.2",
        "tif": "gtc",
        "left": "10",
        "fill_price": "0",
        "text": "t-my-custom-id",
        "tkfr": "0.0003",
        "mkfr": "0"
      }
    ]
  }
}
```

</Example>

批量下单订单信息返回

响应参数:


| 名称                                 | 类型    | 描述                                                                                                                                                                                            |
| ------------------------------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                                                                   |
| `ack`                                | Bool    | "ack"消息的返回表示 WebSocket 的确认消息(目前在下单接口中存在)。<br />如果`ack`为 false(false 该字段不会出现在响应中)，则说明该消息是响应消息，可以判断请求是否成功<br / >通过检查`data.errs`。 |
| `header`                             | Map     | 响应元信息                                                                                                                                                                                      |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                                                                              |
| »`channel`                          | String  | 请求频道                                                                                                                                                                                        |
| »`event`                            | String  | 请求`event`                                                                                                                                                                                     |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                                                                 |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                                                               |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                                                               |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                                                          |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                                                                       |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                                                                           |
| »`x_gate_ratelimit_requests_remain` | Integer | 当前时间窗口剩余可用请求数(为0不展示)                                                                                                                                                           |
| »`x_gate_ratelimit_limit`           | Integer | 当前频率限制上限(为0不展示)                                                                                                                                                                     |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒）                                                |
| `data`                               | Object  | 请求响应的数据                                                                                                                                                                                  |
| »`result`                           | Object  | 如果这是 ack 响应，则结果是请求的`payload`，否则结果是 api 的响应                                                                                                                               |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                                                                              |
| »»`label`                          | String  | 错误类型                                                                                                                                                                                        |
| »»`message`                        | String  | 详细错误信息                                                                                                                                                                                    |

</Block>

<Block>

## 订单取消

`futures.order_cancel`

您可以通过此频道取消订单

**本频道和以下的 APIV4 功能相同:**

```json
DELETE /futures/{
  settle
}/orders/{
  order_id
}
```

</Block>

<Block>

### 订单取消请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
api_cancel_order = {
      "order_id": "74046514"
    }
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_cancel",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_cancel_order
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	apiCancelOrder := map[string]interface{}{
		"order_id": "74046514",
	}

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_cancel",
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1ewq-3123w-5",
			"req_param": toJSONString(apiCancelOrder),
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

func toJSONString(data interface{}) string {
	bytes, _ := json.Marshal(data)
	return string(bytes)
}

```

</Example>

<Example>

订单取消请求示例

```json
{
  "time": 1681195485,
  "channel": "futures.order_cancel",
  "event": "api",
  "payload": {
    "req_id": "request-id-5",
    "req_param": {
      "order_id": "74046514"
    }
  }
}
```

</Example>

`payload` 参数:


| 名称         | 类型     | 必选 | 描述                                                                                              |
| ------------ | -------- | ---- | ------------------------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同                 |
| `req_param`  | `object` | 是   | API 取消订单，详情至[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order-2) |
| `req_header` | `object` | 否   | Apiv4 自定义请求头                                                                                |

`req_param` API 订单模型的 JSON 字节数据:


| 字段       | 类型     | 必选 | 描述                                                                         |
| ---------- | -------- | ---- | ---------------------------------------------------------------------------- |
| `order_id` | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即`text` 字段）。 |

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

</Block>

<Block>

### 订单取消通知

<Example>

订单取消返回示例

```json
{
  "request_id": "request-id-5",
  "header": {
    "response_time": "1681196536282",
    "status": "200",
    "channel": "futures.order_cancel",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": 74046543,
      "user": 6790020,
      "create_time": 1681196535.01,
      "finish_time": 1681196536.343,
      "finish_as": "cancelled",
      "status": "finished",
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31303.2",
      "tif": "gtc",
      "left": "10",
      "fill_price": "0",
      "text": "t-my-custom-id",
      "tkfr": "0.0003",
      "mkfr": "0",
      "stp_id": 2,
      "stp_act": "cn",
      "amend_text": "-"
    }
  }
}
```

</Example>

响应参数:


| 名称                                 | 类型    | 描述                                                                                                                                             |
| ------------------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                    |
| `header`                             | Map     | 响应元信息                                                                                                                                       |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                               |
| »`channel`                          | String  | 请求频道                                                                                                                                         |
| »`event`                            | String  | 请求`event`                                                                                                                                      |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                  |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                           |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                        |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                            |
| »`x_gate_ratelimit_requests_remain` | Integer | 当前时间窗口剩余可用请求数(为0不展示)                                                                                                            |
| »`x_gate_ratelimit_limit`           | Integer | 当前频率限制上限(为0不展示)                                                                                                                      |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒） |
| `data`                               | Object  | 请求响应的数据                                                                                                                                   |
| »`result`                           | Object  | 订单取消参数，详情至[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order-2)                                                |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                               |
| »»`label`                          | String  | 错误类型                                                                                                                                         |
| »»`message`                        | String  | 详细错误信息                                                                                                                                     |

</Block>

<Block>

## 取消所有 ID 列表内的订单

您可以使用此频道`futures.order_cancel_ids`取消所有 ID 列表内的订单。

可以指定多个不同的订单id。一次请求最多只能撤销 20 条记录

**以下是 API 的功能:**

```json
POST /futures/{settle}/batch_cancel_orders
```

</Block>

<Block>

### 取消所有 ID 列表内的订单请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
cancelWithIdsParam = ["1694883366","123"]
ws.send(json.dumps({
    "time":int(time.time()),
    "channel":"futures.order_cancel_ids",
    "event":"api",
    "payload":{
        "req_id":"test_1",
        "req_param": cancelWithIdsParam
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	timestamp := time.Now().Unix()
	cancelWithIdsParam := []string{"1694883366", "123"}
	channel := "futures.order_cancel_ids"

	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	payload := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": cancelWithIdsParam,
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

```

</Example>

<Example>

客户端请求示例

```json
{
  "time": 1681986208,
  "channel": "futures.order_cancel_ids",
  "event": "api",
  "payload": {
    "req_id": "request-9",
    "req_param": [
      "1700664343",
      "123"
    ]
  }
}
```

</Example>

Payload 格式:


| 字段         | 类型     | 必选 | 描述                                                                                |
| ------------ | -------- | ---- | ----------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。 |
| `req_param`  | `array`  | 是   | 订单 ID 列表                                                                        |
| `req_header` | `object` | 否   | apiv4 自定义 header                                                                 |

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

</Block>

<Block>

### 取消所有 ID 列表内的订单推送

<Example>

取消订单推送示例

```json
{
  "request_id": "request-9",
  "header": {
    "response_time": "1681986208564",
    "status": "200",
    "channel": "futures.order_cancel_ids",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": [
      {
        "id": "1694883366",
        "user_id": 111,
        "succeeded": true
      },
      {
        "id": "123",
        "user_id": 111,
        "message": "ORDER_NOT_FOUND"
      }
    ]
  }
}
```

</Example>

推送格式:


| 字段              | 类型    | 描述                                                                                                                                                                       |
| ----------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`      | String  | 消息的唯一标识符                                                                                                                                                           |
| `header`          | Map     | 响应的元信息                                                                                                                                                               |
| »`response_time` | String  | 响应发送时间（以毫秒为单位）                                                                                                                                               |
| »`channel`       | String  | 请求的频道                                                                                                                                                                 |
| »`event`         | String  | 请求事件                                                                                                                                                                   |
| »`client_id`     | String  | 唯一客户端标识 ID                                                                                                                                                          |
| »`x_in_time`     | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                                          |
| »`x_out_time`    | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                                          |
| »`conn_id`       | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                                     |
| »`conn_trace_id` | String  | 与客户端建立连接的TraceId                                                                                                                                                  |
| »`trace_id`      | String  | 执行下单操作的TraceId                                                                                                                                                      |
| `data`            | Object  | 请求响应的数据                                                                                                                                                             |
| »`result`        | Object  | 响应详见[api](https://www.gate.com/docs/developers/apiv4/zh_CN/#%E6%89%B9%E9%87%8F%E6%92%A4%E9%94%80%E6%8C%87%E5%AE%9A-id-%E7%9A%84%E8%AE%A2%E5%8D%95%E5%88%97%E8%A1%A8-2) |
| »`errs`          | Object  | 只有在请求失败时才可用                                                                                                                                                     |
| »»`label`       | String  | 以字符串格式表示错误类型                                                                                                                                                   |
| »»`message`     | String  | 错误信息详情                                                                                                                                                               |

</Block>

<Block>

## 取消匹配的未结束订单

`futures.order_cancel_cp`

您可以通过此渠道取消所有匹配的未结束的订单

**本频道和以下的 APIV4 功能相同:**

```json
DELETE /futures/{
  settle
}/orders
```

</Block>

<Block>

### 请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
api_cancel_all_order = {
      "contract": "BTC_USDT",
      "side": "bid"
    }
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_cancel_cp",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_cancel_all_order
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	apiCancelAllOrder := map[string]interface{}{
		"contract": "BTC_USDT",
		"side":     "bid",
	}

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_cancel_cp",
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1ewq-3123w-5",
			"req_param": toJSONString(apiCancelAllOrder),
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

func toJSONString(data interface{}) string {
	bytes, _ := json.Marshal(data)
	return string(bytes)
}

```

</Example>

<Example>

客户请求示例

```json
{
  "time": 1681196537,
  "channel": "futures.order_cancel_cp",
  "event": "api",
  "payload": {
    "req_id": "request-id-7",
    "req_param": {
      "contract": "BTC_USDT",
      "side": "bid"
    }
  }
}
```

</Example>

`payload` 参数:


| 名称         | 类型     | 必选 | 描述                                                                                       |
| ------------ | -------- | ---- | ------------------------------------------------------------------------------------------ |
| `req_id`     | string   | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同          |
| `req_param`  | object   | 是   | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-matched) |
| `req_header` | `object` | 否   | Apiv4 自定义请求头                                                                         |

`req_param` API 订单模型的 JSON 字节数据:


| 字段       | 类型     | 必选 | 描述                                               |
| ---------- | -------- | ---- | -------------------------------------------------- |
| `contract` | `string` | 是   | 合约                                               |
| `side`     | `string` | 否   | 所有出价或要价。如果没有特别说明，两者都包括在内。 |

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

</Block>

<Block>

### 响应结果

<Example>

订单取消返回示例

```json
{
  "request_id": "request-id-7",
  "header": {
    "response_time": "1681196537567",
    "status": "200",
    "channel": "futures.order_cancel_cp",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": [
      {
        "id": 74046545,
        "user": 6790020,
        "create_time": 1681196536.592,
        "finish_time": 1681196537.626,
        "finish_as": "cancelled",
        "status": "finished",
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31403.2",
        "tif": "gtc",
        "left": "10",
        "fill_price": "0",
        "text": "t-my-custom-id",
        "tkfr": "0.0003",
        "mkfr": "0",
        "stp_id": 2,
        "stp_act": "cn",
        "amend_text": "-"
      }
    ]
  }
}
```

</Example>

响应参数:


| 名称                                 | 类型    | 描述                                                                                                                                             |
| ------------------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                    |
| `header`                             | Map     | 响应元信息                                                                                                                                       |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                               |
| »`channel`                          | String  | 请求频道                                                                                                                                         |
| »`event`                            | String  | 请求`event`                                                                                                                                      |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                  |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                           |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                        |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                            |
| »`x_gate_ratelimit_requests_remain` | Integer | 当前时间窗口剩余可用请求数(为0不展示)                                                                                                            |
| »`x_gate_ratelimit_limit`           | Integer | 当前频率限制上限(为0不展示)                                                                                                                      |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒） |
| `data`                               | Object  | 请求响应的数据                                                                                                                                   |
| »`result`                           | Object  | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-matched)                                                       |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                               |
| »»`label`                          | String  | 错误类型                                                                                                                                         |
| »»`message`                        | String  | 详细错误信息                                                                                                                                     |

</Block>

<Block>

## 修改订单

`futures.order_amend`

您可以通过此频道修改未结束的订单

**本频道和以下的 APIV4 功能相同:**

```
PUT /futures/{settle}/orders/{order_id}
```

</Block>

<Block>

### 请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
api_amend_order = {
      "order_id": "74046543",
      "price": "31303.180000"
    }
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_amend",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_amend_order
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	apiAmendOrder := map[string]interface{}{
		"order_id": "74046543",
		"price":    "31303.180000",
	}

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_amend",
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1ewq-3123w-5",
			"req_param": toJSONString(apiAmendOrder),
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

func toJSONString(data interface{}) string {
	bytes, _ := json.Marshal(data)
	return string(bytes)
}

```

</Example>

<Example>

客户请求示例

```json
{
  "time": 1681196536,
  "channel": "futures.order_amend",
  "event": "api",
  "payload": {
    "req_id": "request-id-4",
    "req_param": {
      "order_id": "74046543",
      "price": "31303.180000"
    }
  }
}
```

</Example>

`payload` 参数:


| 名称         | 类型     | 必选 | 描述                                                                                           |
| ------------ | -------- | ---- | ---------------------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同              |
| `req_param`  | `object` | 是   | API 修改订单参数，详情至[api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order-2) |
| `req_header` | `object` | 否   | Apiv4 自定义请求头                                                                             |

`req_param` API 订单模型的 JSON 字节数据:


| 字段         | 类型     | 必选 | 描述                                                                         |
| ------------ | -------- | ---- | ---------------------------------------------------------------------------- |
| `order_id`   | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即`text` 字段）。 |
| `size`       | `int64`  | 否   | 必选。交易数量，正数为买入，负数为卖出。平仓委托则设置为 0。                 |
| `price`      | `string` | 否   | 价格                                                                         |
| `amend_text` | `int64`  | 否   | 修改订单时的自定义信息                                                       |

`req_header` 自定义 header 数据:


| 字段             | 类型     | 必选 | 描述                                                                       |
| ---------------- | -------- | ---- | -------------------------------------------------------------------------- |
| `x-gate-exptime` | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝 |

#### 详细描述

=> _size_: 新订单尺寸，包括已填充部分。

- 如果新尺寸小于或等于已填充尺寸，订单将被取消。
- 订单面必须与原始面相同。
- 平仓订单大小无法更改。
- 对于仅减少订单，增加尺寸可能会导致其他仅减少订单被取消。
- 如果价格不变，减少数量不会改变其在订单簿中的优先级，而增加数量则会以当前价格将其移至最后。

</Block>

<Block>

### 响应结果

<Example>

订单修改返回示例

```json
{
  "request_id": "request-id-4",
  "header": {
    "response_time": "1681196536251",
    "status": "200",
    "channel": "futures.order_amend",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 99,
    "x_gate_ratelimit_limit": 100,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": 74046543,
      "user": 6790020,
      "create_time": 1681196535.01,
      "status": "open",
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31303.2",
      "tif": "gtc",
      "left": "10",
      "fill_price": "0",
      "text": "t-my-custom-id",
      "tkfr": "0.0003",
      "mkfr": "0",
      "stp_id": 2,
      "stp_act": "cn",
      "amend_text": "-"
    }
  }
}
```

</Example>

响应参数:


| 名称                                 | 类型    | 描述                                                                                                                                             |
| ------------------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `request_id`                         | String  | 对应的请求 ID                                                                                                                                    |
| `header`                             | Map     | 响应元信息                                                                                                                                       |
| »`response_time`                    | String  | 响应发送时间(毫秒)                                                                                                                               |
| »`channel`                          | String  | 请求频道                                                                                                                                         |
| »`event`                            | String  | 请求`event`                                                                                                                                      |
| »`client_id`                        | String  | 唯一的客户端 ID                                                                                                                                  |
| »`x_in_time`                        | Integer | ws 接收请求的时间（以微秒为单位）                                                                                                                |
| »`x_out_time`                       | Integer | ws 返回响应的时间（以微秒为单位）                                                                                                                |
| »`conn_id`                          | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                           |
| »`conn_trace_id`                    | String  | 与客户端建立连接的TraceId                                                                                                                        |
| »`trace_id`                         | String  | 执行下单操作的TraceId                                                                                                                            |
| »`x_gate_ratelimit_requests_remain` | Integer | 当前时间窗口剩余可用请求数(为0不展示)                                                                                                            |
| »`x_gate_ratelimit_limit`           | Integer | 当前频率限制上限(为0不展示)                                                                                                                      |
| »`x_gat_ratelimit_reset_timestamp`  | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒） |
| `data`                               | Object  | 请求响应的数据                                                                                                                                   |
| »`result`                           | Object  | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order-2)                                                                     |
| »`errs`                             | Object  | 仅当请求失败时可用                                                                                                                               |
| »»`label`                          | String  | 错误类型                                                                                                                                         |
| »»`message`                        | String  | 详细错误信息                                                                                                                                     |

</Block>

<Block>

## 获取订单列表

`futures.order_list`

您可以通过此频道获取订单列表

**本频道和以下的 APIV4 功能相同:**

```
GET /futures/{settle}/orders
```

</Block>

<Block>

### 订单列表请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
api_list_order = {
      "contract": "BTC_USDT",
      "status": "open"
    }
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_list",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_list_order
    }
}
))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	apiListOrder := map[string]interface{}{
		"contract": "BTC_USDT",
		"status":   "open",
	}

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_list",
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1ewq-3123w-5",
			"req_param": toJSONString(apiListOrder),
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

func toJSONString(data interface{}) string {
	bytes, _ := json.Marshal(data)
	return string(bytes)
}

```

</Example>

<Example>

客户请求示例

```json
{
  "time": 1681196535,
  "channel": "futures.order_list",
  "event": "api",
  "payload": {
    "req_id": "request-id-3",
    "req_param": {
      "contract": "BTC_USDT",
      "status": "open"
    }
  }
}
```

</Example>

`payload` 参数:


| 名称        | 类型     | 必选 | 描述                                                                                                  |
| ----------- | -------- | ---- | ----------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同                     |
| `req_param` | `object` | 是   | API 请求订单列表参数，详情至[api](https://www.gate.com/docs/developers/apiv4/en/#list-futures-orders) |

`req_param` API 订单模型的 JSON 字节数据:


| 字段       | 类型     | 必选 | 描述                                                    |
| ---------- | -------- | ---- | ------------------------------------------------------- |
| `contract` | `string` | 否   | 合约标识，如果指定则只返回该合约相关数据                |
| `status`   | `string` | 是   | 只列出具有此状态的订单                                  |
| `limit`    | `int`    | 否   | 单个列表中返回的最大记录数                              |
| `offset`   | `int`    | 否   | 列表偏移量，从 0 开始                                   |
| `last_id`  | `string` | 否   | 使用先前列表查询结果中最后一条记录的`id` 指定列表起始点 |

</Block>

<Block>

### 订单列表响应

<Example>

订单列表返回示例

```json
{
  "request_id": "request-id-3",
  "header": {
    "response_time": "1681196536017",
    "status": "200",
    "channel": "futures.order_list",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": [
      {
        "id": 74046543,
        "user": 6790020,
        "create_time": 1681196535.01,
        "finish_time": 1681196535.01,
        "update_time": 1681196535.01,
        "finish_as": "filled",
        "status": "open",
        "contract": "BTC_USDT",
        "size": "10",
        "price": "31403.2",
        "tif": "gtc",
        "left": "10",
        "fill_price": "0",
        "text": "t-my-custom-id",
        "tkfr": "0.0003",
        "mkfr": "0",
        "stp_id": 2,
        "stp_act": "cn",
        "amend_text": "-"
      }
    ]
  }
}
```

</Example>

响应参数:


| 名称              | 类型    | 描述                                                                            |
| ----------------- | ------- | ------------------------------------------------------------------------------- |
| `request_id`      | String  | 对应的请求 ID                                                                   |
| `header`          | Map     | 响应元信息                                                                      |
| »`response_time` | String  | 响应发送时间(毫秒)                                                              |
| »`channel`       | String  | 请求频道                                                                        |
| »`event`         | String  | 请求`event`                                                                     |
| »`client_id`     | String  | 唯一的客户端 ID                                                                 |
| »`x_in_time`     | Integer | ws 接收请求的时间（以微秒为单位）                                               |
| »`x_out_time`    | Integer | ws 返回响应的时间（以微秒为单位）                                               |
| »`conn_id`       | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                          |
| »`conn_trace_id` | String  | 与客户端建立连接的TraceId                                                       |
| »`trace_id`      | String  | 执行下单操作的TraceId                                                           |
| `data`            | Object  | 请求响应的数据                                                                  |
| »`result`        | Object  | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#list-futures-orders) |
| »`errs`          | Object  | 仅当请求失败时可用                                                              |
| »»`label`       | String  | 错误类型                                                                        |
| »»`message`     | String  | 详细错误信息                                                                    |

</Block>

<Block>

## 查询订单详情

`futures.order_status`

您可以通过该频道查询订单详情

**本频道和以下的 APIV4 功能相同:**

```
GET /futures/{settle}/orders/{order_id}
```

</Block>

<Block>

### 订单详情请求

<Example>

代码示例：请求前要先登录

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")

api_status_order = {
      "order_id": "74046543"
    }

ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "futures.order_status",
    "event": "api",
    "payload": {
        "req_id": "1ewq-3123w-5",
        "req_param": api_status_order
    }
}))

print(ws.recv())
```

</Example>

<Example>

代码示例：请求前要先登录

```go

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	ws, _, err := websocket.DefaultDialer.Dial(url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer ws.Close()

	apiStatusOrder := map[string]interface{}{
		"order_id": "74046543",
	}

	payload := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.order_status",
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1ewq-3123w-5",
			"req_param": toJSONString(apiStatusOrder),
		},
	}

	msg, _ := json.Marshal(payload)
	err = ws.WriteMessage(websocket.TextMessage, msg)
	if err != nil {
		log.Println("write:", err)
		return
	}

	_, message, err := ws.ReadMessage()
	if err != nil {
		log.Println("read:", err)
		return
	}
	fmt.Println(string(message))
}

func toJSONString(data interface{}) string {
	bytes, _ := json.Marshal(data)
	return string(bytes)
}

```

</Example>

<Example>

客户请求示例

```json
{
  "time": 1681196535,
  "channel": "futures.order_status",
  "event": "api",
  "payload": {
    "req_id": "request-id-2",
    "req_param": {
      "order_id": "74046543"
    }
  }
}
```

</Example>

`payload` 参数:


| 名称        | 类型     | 必选 | 描述                                                                              |
| ----------- | -------- | ---- | --------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 请求 id，服务器会发回，帮助你识别服务器响应的是哪个请求，<br />它与外部的`id`不同 |
| `req_param` | `object` | 是   | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order-2)  |

req_param` API 订单模型的 JSON 字节数据:


| 字段       | 类型     | 必选 | 描述                                                                         |
| ---------- | -------- | ---- | ---------------------------------------------------------------------------- |
| `order_id` | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即`text` 字段）。 |

</Block>

<Block>

### 订单详情响应

<Example>

订单详情返回示例

```json
{
  "request_id": "request-id-2",
  "header": {
    "response_time": "1681196535985",
    "status": "200",
    "channel": "futures.order_status",
    "event": "api",
    "client_id": "::1-0x14002cfa0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": {
      "id": 74046543,
      "user": 6790020,
      "create_time": 1681196535.01,
      "status": "open",
      "contract": "BTC_USDT",
      "size": "10",
      "price": "31403.2",
      "tif": "gtc",
      "left": "10",
      "fill_price": "0",
      "text": "t-my-custom-id",
      "tkfr": "0.0003",
      "mkfr": "0",
      "stp_id": 2,
      "stp_act": "cn",
      "amend_text": "-"
    }
  }
}
```

</Example>

响应参数:


| 名称              | 类型    | 描述                                                                             |
| ----------------- | ------- | -------------------------------------------------------------------------------- |
| `request_id`      | String  | 对应的请求 ID                                                                    |
| `header`          | Map     | 响应元信息                                                                       |
| »`response_time` | String  | 响应发送时间(毫秒)                                                               |
| »`channel`       | String  | 请求频道                                                                         |
| »`event`         | String  | 请求`event`                                                                      |
| »`client_id`     | String  | 唯一的客户端 ID                                                                  |
| »`x_in_time`     | Integer | ws 接收请求的时间（以微秒为单位）                                                |
| »`x_out_time`    | Integer | ws 返回响应的时间（以微秒为单位）                                                |
| »`conn_id`       | String  | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                           |
| »`conn_trace_id` | String  | 与客户端建立连接的TraceId                                                        |
| »`trace_id`      | String  | 执行下单操作的TraceId                                                            |
| `data`            | Object  | 请求响应的数据                                                                   |
| »`result`        | Object  | 详情至[api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order-2) |
| »`errs`          | Object  | 仅当请求失败时可用                                                               |
| »»`label`       | String  | 错误类型                                                                         |
| »»`message`     | String  | 详细错误信息                                                                     |

</Block>
