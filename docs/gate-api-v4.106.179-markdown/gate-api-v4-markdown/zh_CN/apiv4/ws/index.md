---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://www.gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Spot WebSocket v4.0.0

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

# pip install -U websocket_client
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
                    self._request("spot.ping", auth_required=False)
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
    # handle whatever message you received
    logger.info("message received from server: {}".format(message))


def on_open(ws):
    # type: (GateWebSocketApp) -> None
    # subscribe to channels interested
    logger.info('websocket connected')
    ws.subscribe("spot.trades", ['BTC_USDT'], False)


if __name__ == "__main__":
    logging.basicConfig(format="%(asctime)s - %(message)s", level=logging.DEBUG)
    app = GateWebSocketApp("wss://api.gateio.ws/ws/v4/",
                           "YOUR_API_KEY",
                           "YOUR_API_SECRET",
                           on_open=on_open,
                           on_message=on_message)
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
	u := url.URL{Scheme: "wss", Host: "api.gateio.ws", Path: "/ws/v4/"}
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
	pingMsg := NewMsg("spot.ping", "", t, []string{})
	err = pingMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe order book
	orderBookMsg := NewMsg("spot.order_book", "subscribe", t, []string{"BTC_USDT", "5", "100ms"})
	err = orderBookMsg.send(c)
	if err != nil {
		panic(err)
	}

	// subscribe positions
	ordersMsg := NewMsg("spot.orders", "subscribe", t, []string{"BTC_USDT"})
	ordersMsg.sign()
	err = ordersMsg.send(c)
	if err != nil {
		panic(err)
	}

	select {}
}
```

</Example>

Gate 提供了一个简单而健壮的 Websocket API 来集成现货交易状态信息为您的业务或应用提供支持。

我们当前使用 `Python` 和 `Golang` 作为代码示例，将来会有更多语言示例支持！

代码示例位于右侧深色区域，您可以点击右上方的标签切换示例的编程语言。

### 服务地址

Websocket 链接地址：

- 线上交易: `wss://api.gateio.ws/ws/v4/`
- 线上交易 SBE: `wss://api.gateio.ws/ws/v4/ws/spot/sbe`
- 模拟盘交易: `wss://ws-testnet.gate.com/v4/ws/spot`
- 模拟盘 SBE: `wss://ws-testnet.gate.com/v4/ws/spot/sbe`

::: tip
建议使用 SBE 以获取更快的行情和更小的带宽成本，具体使用查看 `SBE 数据推送` 章节。
:::

### SDK

我们提供了 WebSocket SDK 来帮助开发者进行业务复用。

SDK 的源代码 在 [gatews](https://github.com/gate/gatews) GitHub 仓库。

### 变更历史

2026-10-14

- 新增公开频道 `spot.rpi_order_book`，`spot.rpi_obu` 推送 RPI 深度全量快照

2026-08-31

- `spot.order_place` 单笔和批量下单新增 `trade_quote`；RESULT 和 FULL 结果在下游返回实际计价资产时输出该字段。
- `spot.order_cancel_cp` 新增 `trade_quote`，用于仅撤销使用指定实际计价资产的订单。
- 成交、用户成交和订单推送新增统一订单簿字段 `stock`、`money` 和 `trade_mode`。
- 本期仅启用计价侧统一交易（`trade_mode=1`）；。
- 订单 `finish_as` 新增 `unified_check_failed`。
- SBE Schema 2 在 publicTrade、userTrade 和 orders 模板中新增统一订单簿字段。

2026-06-09

- 模拟盘新增 SBE（Simple Binary Encoding）二进制行情推送支持，具体使用查看 `SBE 数据推送` 章节。

2026-03-31

- `spot.obu` 模拟盘新增立即的首次快照推送，该次快照推送将会在订阅请求的响应之前推送。此行为与之前快照在订阅请求的响应之后推送不同，请注意该行为变更。

2026-01-07

- `spot.order_place` 入参增加 `slippage` 字段支持
- `spot.orders` 和 `spot.orders_v2` 文档新增 `slippage` 字段

2025-10-21

- 更新 `spot.orders` 和 `spot.orders_v2` 的 `type` 的说明

2025-09-29

- 更新 `spot.balances` 的 `change_type` 的说明

2025-06-24

- 新增深度频道V2文档说明

2025-06-12

- 更新了字段枚举值，涉及 `spot.orders` 和 `spot.orders_v2` 频道

2025-04-18

- 补充文档代码示例

2025-04-10

- `spot.orders` 和 `spot.orders_v2` 文档新增 `filled_amount` 字段说明
<!-- - Websocket API 概述新增 `timeUnit=microsecond` 的说明 -->

2025-03-26

- 修复 `spot.order_place` 的 `auto_repay` 和 `auto_borrow` 字段的文档说明

2025-02-10

- 更新现货账户交易模块，新增了 `x_in_time`, `x_out_time`, `conn_trace_id`, `trace_id` 字段
- 更新现货账户交易模块，新增了 `spot.order_list` 频道
- `spot.order_place` 和 `spot.order_amend` 新增了 `x_gate_ratelimit_requests_remain`, `x_gate_ratelimit_limit` 和 `x_gat_ratelimit_reset_timestamp` 字段

2025-01-08

- `spot.usertrades` 频道新增 `id_market` 字段, 该字段代表交易市场唯一
- `spot.trades` 频道新增 `id_market` 字段, 该字段代表交易市场唯一
- 新增 `spot.trades_v2` 频道。
- 新增 `spot.usertrades_v2` 频道。
- 新增 `spot.orders_v2` 频道。

2024-11-28

- 移除 `1000ms` 更新频率支持在 `spot.order_book_update` 

2024-01-17

- 新增 `w` 窗口关闭标识到 K 线频道更新推送

2023-04-21

- 新增 `freeze`,`freeze_change`,`change_type` 字段到 `spot.cross_balances` 频道推送

2022-12-12

- 新增 `avg_deal_price` 字段到 `spot.orders` 频道推送

2022-12-07

- 新增 `freeze`,`freeze_change`,`change_type` 字段到 `spot.balances` 频道推送

2022-11-22

- 新增消息发送毫秒时间戳 `time_ms` 到全部频道推送

2022-07-05

- 新增 `spot.cross_loan` 频道通知用户用户全仓保证金借款和利息更新

2021-07-23

- 增加 `spot.cross_balances` 频道来通知 全仓杠杆余额（cross margin balance） 更新
- 增加 `text` 字段在 `spot.usertrades` 服务通知中

2021-04-27

- 增加 毫秒时间戳 在 `spot.orders`, `spot.balances`, `spot.margin_balances`
  和 `spot.funding_balances` 服务通知中

2021-03-17

- 增加文档阐述如何维护本地深度
- 增加 毫秒时间戳 `t` 在所有的 order book 频道的 `result` 中

2021-01-26

- 初代版本发布

</Block>

<Block>

## Websocket API 概述

WebSocket 操作分为不同的频道。频道可以是公共的或私有的。 公共频道可以直接订阅，而私有频道需要使用 Gate APIv4
密钥对进行身份验证（有关详细信息，请参阅下面的[身份验证](https://chat.openai.com/c/7b212ca5-e8b9-40df-af12-899e9c902460#authentication)）。

所有频道都支持以下事件：

- **`subscribe`**

​ 由客户端发起。客户端使用此方法告诉服务器它对此频道感兴趣，并要求服务器在频道相关数据更改时通知 新数据。

- **`unsubscribe`**

  由客户端发起。客户端使用此方法告诉服务器它不再对此频道感兴趣，并停止发送任何进一步的频道更新。

- **`update`**

  由服务器发起。服务器使用此方法将更改的数据发送给所有订阅此频道的客户端。客户端不能使用此操作事件。

</Block>

<Block>

### 客户端请求

<Example>

客户端请求示例

```json
{
  "time": 1611541000,
  "id": 123456789,
  "channel": "spot.orders",
  "event": "subscribe",
  "payload": ["BTC_USDT", "GT_USDT"],
  "auth": {
    "method": "api_key",
    "KEY": "xxxx",
    "SIGN": "xxxx"
  }
}
```

</Example>

从客户端发起的 `subscribe` 或 `unsubscribe` 请求都是通用 JSON 格式且包含以下字段：

| 字段      | 类型                 | 必选 | 描述                                                                                |
| --------- | -------------------- | ---- | ----------------------------------------------------------------------------------- |
| `time`    | Integer              | 是   | 精确到秒的请求时间。请求时间和服务器时间之间的间隔不能超过 60 秒。                  |
| `id`      | Integer              | 否   | 请求 id（可选），服务器将此 id 发送回来，用来帮助您确定服务器响应哪个请求           |
| `channel` | String               | 是   | 需要订阅的 websocket 频道                                                           |
| `auth`    | [Auth](#schema_auth) | 否   | 私有频道的身份验证凭证。详见 身份验证（[Authentication](#authentication)） 细节部分 |
| `event`   | String               | 是   | 频道的操作事件。例如 `subscribe`, `unsubscribe`                                     |
| `payload` | Any                  | 否   | 请求详细参数（可选）                                                                |

注意，`payload` 的类型是基于 channel 的，不同 channel 需要的 payload 参数不同。但 `subscribe` 和 `unsubscribe` 的 payload
在同一个 channel 中使用的格式都是相同的。

以 `spot.orders` 频道为例，此频道需要的 payload 格式是一个货币列表。你可以指定 `["BTC_USDT", "ETH_USDT", "etc"]`
作为 `subscribe` 的 payload 以接收关于这些货币对的订单更新通知。然后在 `unsubscribe` payload 中指定 `["etc"]`
来去掉这个货币对的订单更新通知。

为了简单起见，下面的频道（channel）描述只给出对应频道的 payload 格式，但是您需要发送完整的请求来执行频道订阅操作。

</Block>

<Block>

### 服务端响应

<Example>

服务端响应示例

```json
{
  "time": 1611541000,
  "channel": "spot.orders",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

</Example>

服务端响应包括对 客户端请求的响应 和 服务端发起的更新消息通知。
与请求类似，服务端响应与客户端请求格式都是几乎相同的 JSON 格式：

| 字段      | 类型                   | 描述                                                                                                      |
| --------- | ---------------------- | --------------------------------------------------------------------------------------------------------- |
| `time`    | Integer                | 精确到秒的响应时间                                                                                        |
| `time_ms` | Integer                | 精确到毫秒的响应时间                                                                                        |
| `id`      | Integer                | 从客户端请求 payload 中提取的请求 ID （如果请求参数中有的话）                                             |
| `channel` | String                 | WebSocket 频道名称                                                                                        |
| `event`   | String                 | 服务端频道事件（即，`update`）或 用于从客户端发起的请求的 `event`                                         |
| `error`   | [Error](#schema_error) | 如果服务端正常接受客户端的请求，则返回为空；否则，返回请求被拒绝的详情。                                  |
| `result`  | Any                    | 返回来自服务端的新数据通知 或 对客户端请求的响应。如果有错误返回则 `error` 不为空，没有错误则此字段为空。 |

注意：如果它是服务端发起的数据更新通知 那么 `result` 的类型是基于 channel 的，不同 channel 的 `result` 类型有所不同。

但如果是对客户端订阅请求的响应，那么 `result` 为固定的 `{"status": "success"}`。
验证订阅请求是否成功，您只需要检查 `error` 字段是否为空即可，不需要再解析 `result` 字段。

<!-- 如果需要得到微秒级的响应时间，请在 URL 里添加参数 `timeUnit=microsecond`，之后服务返回的数据里会新增 `time_us` 字段，该字段返回精确到微秒的时间戳。
`现货账户交易` 模块下的频道不支持此功能，因为已有字段 `x_out_time` 返回微秒级时间戳。 

URL 示例：`wss://api.gateio.ws/ws/v4/?timeUnit=microsecond`。 -->

为了简单起见，下面的频道（channel）描述只给出对应频道的 payload 格式。

</Block>

<Block>

### Error

错误对象的格式如下:

<a id="schema_error"></a>

| 字段      | 类型    | 描述         |
| --------- | ------- | ------------ |
| `code`    | Integer | 错误码       |
| `message` | String  | 错误详情描述 |

请求异常时，服务端返回的消息会包含一个 `error` 对象，包括对应的错误码和具体的错误信息：

| `code` | `message`                                     |
| ------ | --------------------------------------------- |
| 1      | Invalid request body format（无效请求格式）   |
| 2      | Invalid argument provided（提供的参数无效）   |
| 3      | Server side error happened（服务端发生异常）  |
| 4      | Authentication fail（鉴权失败）       |

</Block>

<Block>

<a id="authentication"></a>

## 鉴权

::: warning
注意：您使用的 GateAPIv4 密钥对至少开了 spot（现货） read（读）权限，
如果启用密钥白名单，则您的出方向 IP 地址必须在密钥的 IP 白名单中。
:::

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
    'channel': 'spot.orders',
    'event': 'subscribe',
    'payload': ["BTC_USDT", "GT_USDT"]
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
		"channel": "spot.orders",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT", "GT_USDT"},
	}
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	jsonBytes, _ := json.Marshal(request)
	fmt.Println(string(jsonBytes))
}

```

</Example>

如果频道是私有的，例如 `spot.orders` 频道用于获取用户订单更新，那么客户端请求需要携带身份验证信息。

身份验证由请求体中的 `auth` 字段发送，格式如下：

<a id="schema_auth"></a>

| 字段     | 类型   | 描述                                           |
| -------- | ------ | ---------------------------------------------- |
| `method` | String | 身份验证方法。目前只接受一个方法 `api_key`     |
| `KEY`    | String | Gate APIv4 用户 key 字符串                     |
| `SIGN`   | String | 使用 GateAPIv4 secret 和请求信息生成的认证签名 |

WebSocket 认证使用与 Gate APIv4 API 相同的签名计算方法，即:
`HexEncode(HMAC_SHA512(secret, signature_string))`，但有以下区别：

1. 签名字符串连接方法: `channel=<channel>&event=<event>&time=<time>`,
   这里的 `<channel>`, `<event>`, `<time>` 对应的请求信息
2. 身份验证信息在 `auth` 字段的请求主体中发送。

您可以登录到控制台查看 Gate APIv4 密钥和秘密.

</Block>

<Block>

# SBE 数据推送

</Block>

<Block>

## 对接SBE

- 使用地址，在现有的地址后添加 `/sbe`：
  - 线上交易: `wss://api.gateio.ws/ws/v4/ws/spot/sbe`
  - testnet: `wss://ws-testnet.gate.com/v4/ws/spot/sbe`
- Schema `1` XML 文件：
  - `testnet`: [gate_spot_ws_latest.xml](https://github.com/gate/gatews/blob/master/sbe/schemas/testnet/gate_spot_ws_latest.xml)
  - 线上交易: [gate_spot_ws_latest.xml](https://github.com/gate/gatews/blob/master/sbe/schemas/prod/gate_spot_ws_latest.xml)
- 上述两个公开 XML 文件当前均为 `schemaId=1`，不能用于解码请求 `sbe_schema_id=2` 后收到的消息。
- Schema `2` 将以独立文件 `gate_spot_ws_v2.0.0.xml` 对外发布。在对应公开目录提供该 XML 文件之前，客户端必须继续使用 Schema `1`，不得请求 `sbe_schema_id=2`。
- Schema `2` XML 文件公开后，可通过 `query` 参数指定 `sbe_schema_id`，例如：`wss://ws-testnet.gate.com/v4/ws/spot/sbe?sbe_schema_id=2`
  - 目前支持的 `sbe_schema_id` 为 `0`、`1` 和 `2`；`sbe_schema_id` 为 `0` 用于客户端测试 `sbe schema` 不兼容升级的逻辑
  - 不传入 `sbe_schema_id` 时默认使用 Schema `1`；需要接收统一订单簿字段时必须显式指定 `sbe_schema_id=2`
  - publicTrade、userTrade 和 orders 模板中的统一订单簿字段（`stock`、`money` 和 `trade_mode`）需要使用 `sbe_schema_id=2`。Schema 0 和 1 保持原有布局。
  - 不支持的 `sbe_schema_id` 会回退到默认 Schema `1`；仍受支持的旧版本继续使用客户端指定的布局

</Block>

<Block>

## SBE使用说明

- 使用 `JSON` 进行请求和首次响应；使用 `SBE` 作为数据推送；
- 同一条连接上同时存在 `JSON` 和 `SBE` 的消息，请使用 opcode 来区分数据：`opcode 为 1 代表 JSON`，`opcode 为 2 代表 SBE`。
- SBE 的解码：
  - **MessageHeader**：每条 SBE 二进制帧均为「MessageHeader + 消息体」。Header 为固定 8 字节（小端序，`uint16` 依次为 `blockLength`、`templateId`、`schemaId`、`version`），解码时**必须先读 Header**，再根据 `schemaId` 和 `templateId` 选择对应 Schema 与消息类型解码消息体。
  - **解码流程建议**：
      1. 读取 MessageHeader（固定 8 字节），得到 `blockLength`、`templateId`、`schemaId`、`version`。
      2. 根据 `schemaId` 选择解码器：`0` → 使用字符串 decimal 的旧布局；`1` → 使用定点 decimal 布局；`2` → 使用增加统一订单簿字段的定点布局。不能用其他 schema 的布局解码当前帧。
      3. 根据 `templateId` 确定具体消息类型（如 publicTrade、orderBook、bbo 等），再按该 Schema 的布局解码消息体。
- 使用 SBE 时，**仅可订阅以下频道**，其余频道不支持 SBE 推送。后续将扩展到其余频道。
  - 订阅不支持 SBE 的频道时，将返回订阅失败的消息

| 通道名                     | templateId | 消息类型         | 说明              |
|-------------------------|------------|--------------|-----------------|
| `spot.tickers`          | 7          | ticker       | 行情              |
| `spot.trades`           | 2          | publicTrade  | 公共成交            |
| `spot.f_trades`         | 2          | publicTrade  | 过滤公共成交        |
| `spot.candlesticks`     | 6          | candlestick  | K 线             |
| `spot.order_book`       | 4          | orderBook    | 订单簿（全量深度）       |
| `spot.rpi_order_book`   | 4          | orderBook    | RPI 订单簿（全量深度）   |
| `spot.order_book_update`| 5          | orderBookUpdate | 订单簿增量更新      |
| `spot.book_ticker`      | 1          | bbo          | 最优买卖价（BBO）      |
| `spot.obu`              | 3          | obu          | 订单簿增量（OBU）      |
| `spot.rpi_obu`          | 3          | obu          | RPI 订单簿增量（OBU）  |
| `spot.usertrades`       | 8          | userTrade    | 用户成交            |
| `spot.orders`           | 9          | orders       | 订单              |
| `spot.balances`         | 10         | balance      | 现货余额            |
| `spot.margin_balances`  | 11         | marginBalance | 杠杆余额           |
| `spot.funding_balances` | 12         | fundingBalance | 理财余额          |
| `spot.cross_balances`   | 13         | crossBalance | 全仓杠杆余额          |
| `spot.cross_loan`       | 14         | crossLoan    | 全仓借贷            |
| `spot.priceorders`      | 15         | priceOrder   | 价格条件单           |

> - 上表 `templateId` 以对应版本的 schema XML 为准。
> - 上述频道的 v2 版本（如 `spot.trades_v2`、`spot.orders_v2`、`spot.usertrades_v2`）复用相同的消息模板，同样支持 SBE 推送。

SBE Schema 2 的统一订单簿新增字段：

| 模板 | 作用域 | SBE 字段 | ID | JSON 字段 | 说明 |
|------|--------|----------|----|-----------|------|
| publicTrade (templateId 2) | 消息 | `tradeMode` | 12 | `trade_mode` | 交易模式 |
| publicTrade (templateId 2) | 消息 | `stock` / `money` | 203 / 204 | `stock` / `money` | 实际基础/计价资产 |
| userTrade (templateId 8) | `trades` 分组 | `tradeMode` | 13 | `trade_mode` | 交易模式 |
| userTrade (templateId 8) | `trades` 分组 | `stock` / `money` | 206 / 207 | `stock` / `money` | 实际基础/计价资产 |
| orders (templateId 9) | `result` 分组 | `tradeMode` | 28 | `trade_mode` | 交易模式 |
| orders (templateId 9) | `result` 分组 | `stock` / `money` | 213 / 214 | `stock` / `money` | 实际基础/计价资产 |

orders 模板在旧 Schema 中已经包含 `finishAs`（Schema 1 和 2 中均为 `id=210`）。`unified_check_failed` 只是新增字符串取值，不要求使用 Schema 2。

`spot.usertrades_v2` 的 JSON 推送仍不包含 `fee`、`fee_currency`、`point_fee` 和 `gt_fee`。为保持二进制兼容，共用的 SBE userTrade 模板会保留固定手续费字段；该频道中这些字段解码为 0 或空字符串。

<Example>

订阅受支持的频道（以 `spot.order_book` 为例）。请求与首次响应均为 JSON，订阅成功后该频道数据以 SBE 二进制帧推送。

客户端请求

```json
{
  "time": 1770603400,
  "id": 123456789,
  "channel": "spot.order_book",
  "event": "subscribe",
  "payload": ["BTC_USDT", "5", "100ms"]
}
```

服务端响应（订阅的 JSON 确认）

```json
{
  "time": 1770603400,
  "time_ms": 1770603400123,
  "channel": "spot.order_book",
  "event": "subscribe",
  "error": null,
  "result": {
    "status": "success"
  }
}
```

订阅成功后，该频道的订单簿数据将以 SBE 二进制帧（`opcode 2`）推送，使用 `templateId` `4`（orderBook）解码。

订阅不支持 SBE 的频道时将返回订阅失败：

```json
{
  "time": 1770603321,
  "time_ms": 1770603321767,
  "conn_id": "57a8765578ea837e",
  "trace_id": "3c75ba05569b3b292a2f36cfdd90d868",
  "channel": "spot.system",
  "event": "subscribe",
  "payload": [
    "20011"
  ],
  "error": {
    "code": 2,
    "message": "channel spot.system does not support SBE"
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

系统 API 用于检索服务元信息，**不**用于订阅。

## 应用层 ping/pong 消息

`spot.ping`

<Example>

代码示例

```python
import time
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send('{"time": %d, "channel" : "spot.ping"}' % int(time.time()))
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	ping := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.ping",
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

响应示例

```json
{
  "time": 1545404023,
  "channel": "spot.pong",
  "event": "",
  "error": null,
  "result": null
}
```

</Example>

检查与服务器的连接是否仍然存活。

这是一种额外的连接可达性检查。服务器使用[协议层的 ping/pong](https://tools.ietf.org/html/rfc6455)
消息来检查客户端是否仍然连接。它不强制要求使用这种方法。如果您使用一些知名的 WebSocket 客户端库，通常不需要关心这个 API。

然而，从客户端的角度来看，这个 API 可以帮助客户端主动检查与服务器的连接是否仍然可达。此外，如果服务器接收到客户端的`spot.ping`
请求，它还将重置客户端的超时计时器。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

## 服务升级通知

服务在即将关闭进行升级时，会向当前连接主动推送一条系统通知，客户端收到后应尽快重连。

**频道（Channel）：** `spot.system`

**事件（Event）：** `update`

当 `channel` 为 `spot.system`、`event` 为 `update` 且 `result.type` 为 `upgrade` 时，客户端可将该消息识别为服务升级通知。

**`result` 格式（SystemNotifyDTO）：**

| 字段   | 类型   | 说明                   |
| ------ | ------ | ---------------------- |
| `type` | String | 通知类型，如 `upgrade` |
| `msg`  | String | 提示文案               |
| `data` | Object | 可选，扩展数据         |

**示例（服务升级）：**

```json
{
  "time": 1784800711,
  "time_ms": 1784800711140,
  "channel": "spot.system",
  "event": "update",
  "result": {
    "type": "upgrade",
    "msg": "The connection will soon be closed for a service upgrade. Please reconnect."
  }
}
```

</Block>

<Block>

# Tickers 频道

`spot.tickers`

**更新速度:** 1000ms

Ticker 是对现货交易状态的高级概览。它显示了最高、最低和最后的交易价格。它还包括每日成交量以及价格在过去一天内的变化情况等信息。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.tickers",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.tickers",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606291803,
  "channel": "spot.tickers",
  "event": "update",
  "result": {
    "currency_pair": "BTC_USDT",
    "last": "19106.55",
    "lowest_ask": "19108.71",
    "highest_bid": "19106.55",
    "change_percentage": "3.66",
    "base_volume": "2811.3042155865",
    "quote_volume": "53441606.52411221454674732293",
    "high_24h": "19417.74",
    "low_24h": "18434.21"
  }
}
```

</Example>

通知结果格式：

| 字段                  | 类型   | 描述                               |
| --------------------- | ------ | ---------------------------------- |
| `result`              | Object | Ticker 对象                        |
| » `currency_pair`     | String | 货币对                             |
| » `last`              | String | 最新成交价                         |
| » `lowest_ask`        | String | 卖方最低价                         |
| » `highest_bid`       | String | 买方最高价                         |
| » `change_percentage` | String | 涨跌百分比，跌用负数标识，如 -7.45 |
| » `base_volume`       | String | 交易货币成交量                     |
| » `quote_volume`      | String | 计价货币成交量                     |
| » `high_24h`          | String | 24 小时最高价                      |
| » `low_24h`           | String | 24 小时最低价                      |

</Block>

<Block>

# 公共成交频道

`spot.trades`

**更新速度:** 实时

该频道在每次发生交易时发送交易消息。它包括交易的详细信息，如价格、数量、时间和类型。

只有吃单方会被通知。

请注意，这是一个公共频道。对于私有交易通知，请参阅下面的 [用户私有成交](#spot-user-trades) 部分。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.trades",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.trades",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606292218,
  "channel": "spot.trades",
  "event": "update",
  "result": {
    "id": 309143071,
    "create_time": 1606292218,
    "create_time_ms": "1606292218213.4578",
    "side": "sell",
    "currency_pair": "BTC_USD",
    "amount": "16.4700000000",
    "price": "0.4705000000",
    "range": "2390902-2390902",
    "id_market": 917144,
    "stock": "BTC",
    "money": "USDC",
    "trade_mode": 1
  }
}
```

</Example>

请注意，公共交易频道只通知交易中的吃单方。下面的私有用户交易频道将通知所有与用户相关的交易。

通知结果格式：

| 字段               | 类型    | 描述                              |
| ------------------ | ------- | --------------------------------- |
| `result`           | Object  | 公共成交详情                      |
| » `id`             | Integer | 成交 ID                           |
| » `create_time`    | Integer | 成交时间，精确到秒                |
| » `create_time_ms` | String  | 成交时间，毫秒精度                |
| » `side`           | String  | 买单或者卖单                      |
| » `currency_pair`  | String  | 交易货币对                        |
| » `amount`         | String  | 交易数量                          |
| » `price`          | String  | 交易价                            |
| » `range`          | String  | 成交范围(格式: "开始 ID-结束 ID") |
| » `id_market`      | Integer | 按市场成交ID |
| » `stock`          | String  | 该笔成交实际使用的基础资产 |
| » `money`          | String  | 该笔成交实际使用的计价资产 |
| » `trade_mode`     | Integer | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一 |

在统一订单簿中，`currency_pair` 仍为订阅的名义市场（例如 `BTC_USD`），`stock` 和 `money` 表示该笔成交实际使用的资产。

#### 字段数据枚举

| Property | Value |
| -------- | ----- |
| side     | buy   |
| side     | sell  |

</Block>

<Block>

# 公共成交频道v2（停止维护）

`spot.trades_v2`

**更新速度:** 实时

该频道在每次发生交易时发送交易消息。它包括交易的详细信息，如价格、数量、时间和类型。

请注意，这是一个公共频道。对于私有交易通知，请参阅下面的 [用户私有成交](#spot-user-trades) 部分。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.trades_v2",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.trades_v2",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606292218,
  "channel": "spot.trades_v2",
  "event": "update",
  "result": {
    "id": 309143071,
    "create_time": 1606292218,
    "create_time_ms": "1606292218213.4578",
    "side": "sell",
    "currency_pair": "BTC_USD",
    "amount": "16.4700000000",
    "price": "0.4705000000",
    "range": "2390902-2390902",
    "id_market": 917144,
    "stock": "BTC",
    "money": "USDC",
    "trade_mode": 1
  }
}
```

</Example>

请注意，公共交易频道只通知交易中的吃单方。下面的私有用户交易频道将通知所有与用户相关的交易。

通知结果格式：

| 字段               | 类型    | 描述                              |
| ------------------ | ------- | --------------------------------- |
| `result`           | Object  | 公共成交详情                      |
| » `id`             | Integer | 成交 ID                           |
| » `create_time`    | Integer | 成交时间，精确到秒                |
| » `create_time_ms` | String  | 成交时间，毫秒精度                |
| » `side`           | String  | 买单或者卖单                      |
| » `currency_pair`  | String  | 交易货币对                        |
| » `amount`         | String  | 交易数量                          |
| » `price`          | String  | 交易价                            |
| » `range`          | String  | 成交范围(格式: "开始 ID-结束 ID") |
| » `id_market`      | Integer | 按市场成交ID |
| » `stock`          | String  | 该笔成交实际使用的基础资产 |
| » `money`          | String  | 该笔成交实际使用的计价资产 |
| » `trade_mode`     | Integer | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一 |

在统一订单簿中，`currency_pair` 仍为订阅的名义市场，`stock` 和 `money` 表示该笔成交实际使用的资产。

#### 字段数据枚举

| Property | Value |
| -------- | ----- |
| side     | buy   |
| side     | sell  |

</Block>

<Block>

# K 线频道

`spot.candlesticks`

**更新速度:** 2000ms

提供 K 线 信息订阅。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.candlesticks",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["1m", "BTC_USDT"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.candlesticks",
		"event":   "subscribe",
		"payload": []string{"1m", "BTC_USDT"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段         | 类型            | 必选 | 描述                                  |
| ------------ | --------------- | ---- | ------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数。从左到右是 `interval`, `cp` |
| » `interval` | String          | 是   | K 线数据点间隔时间                    |
| » `cp`       | String          | 是   | 交易货币对                            |

#### 字段数据枚举

| Property | Value |
| -------- | ----- |
| interval | 10s   |
| interval | 30s   |
| interval | 1m    |
| interval | 3m    |
| interval | 5m    |
| interval | 15m   |
| interval | 30m   |
| interval | 1h    |
| interval | 2h    |
| interval | 4h    |
| interval | 6h    |
| interval | 8h    |
| interval | 12h   |
| interval | 1d    |
| interval | 7d    |
| interval | 30d   |

要订阅多个货币对或具有不同时间间隔的数据，只需发送多个带有不同参数的订阅请求即可。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606292600,
  "channel": "spot.candlesticks",
  "event": "update",
  "result": {
    "t": "1606292580",
    "v": "2362.32035",
    "c": "19128.1",
    "h": "19128.1",
    "l": "19128.1",
    "o": "19128.1",
    "n": "1m_BTC_USDT",
    "a": "3.8283",
    "w": true
  }
}
```

</Example>

通知结果格式：

| 字段     | 类型    | 描述                                                                      |
| -------- | ------- | ------------------------------------------------------------------------- |
| `result` | Object  | K 线数                                                                    |
| »`t`     | String  | Unix 时间戳，精确到秒                                                     |
| »`v`     | String  | 交易量，只有市场行情的 K 线数据里有该值                                   |
| »`c`     | String  | 收盘价                                                                    |
| »`h`     | String  | 最高价                                                                    |
| »`l`     | String  | 最低价                                                                    |
| »`o`     | String  | 开盘价                                                                    |
| »`n`     | String  | 订阅名称，格式为 `<interval>_<cp>`                                        |
| »`a`     | String  | 基础货币交易金额                                                          |
| »`w`     | Boolean | `true` 表示窗口已关闭。注：可能会缺失 `true` 的消息，但不影响数据的完整性 |

</Block>

<Block>

# 订单簿/深度频道

深度有以下订阅频道，分别满足不同的需求。它们是:

- `spot.book_ticker`

  及时推送订阅货币对的最优卖/买（ask/bid）单。

- `spot.order_book_update`

  定时推送 深度增量更新数据，可用于维护本地深度。

- `spot.order_book`

  基于订阅的 level，定时推送深度全量更新

- `spot.rpi_order_book`

  基于订阅的 level，定时推送 RPI 深度全量更新

每个货币对的深度更新都有一个内部更新 ID，该 ID 在每个订单成交后递增 1。深度更新 ID 对应于 REST API
的 `GET /api/v4/spot/order_book` 响应中的`ID` 字段。

如何维护本地订单簿/深度：

1. 以指定的更新频率订阅 `spot.order_book_update`。
   例如 `["BTC_USDT", "100ms"]` 每 100ms 推送 BTC_USDT 订单簿/深度的更新。

2. 缓存 WebSocket 通知。每个通知用 `U` 和 `u` 来分别标识自上条通知的起始 ID 和最新 ID。

3. 使用 REST API 获取基础订单簿/深度，并确保订单簿/深度 ID 被记录（下面称为`baseID`)

   例如使用：`https://api.gateio.ws/api/v4/spot/order_book?currency_pair=BTC_USDT&limit=100&with_id=true`
   获取 10 个 BTC_USDT 基础深度。

4. 迭代缓存的 WebSocket 通知，然后找到第一个包含 baseID 的，
   例如`U <= baseId+1` 且 `u >= baseId+1`，然后开始消费通知。注意 `size`
   在通知里都是绝对值。 使用他们替换对应价格 `price` 中的原始 `size`。
   如果 `size` 等于 0，从订单簿/深度中删除价格。

5. 转储所有满足要求的通知 `u < baseID+1`。 如果 `baseID+1 < first notification U`，那将
   意味着当前基础深度已经落后当前的通知。从第三步重新开始获取最新的基础深度。

6. 如其后有发现任何满足 `U > baseID+1`的通知，那意味着一些更新通知丢失。需要从第三步开始重构本地深度。

[SDK GitHub repository](https://github.com/gate/gatews) 中可以找到实现上述方法的示例的应用程序

</Block>

<Block>

## 最优买卖价

`spot.book_ticker`

**更新速度:** 每 10ms

</Block>

<Block>

### 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.book_ticker",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.book_ticker",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述           |
| --------- | --------------- | ---- | -------------- |
| `payload` | `Array[String]` | 是   | 交易货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

### 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606293275,
  "channel": "spot.book_ticker",
  "event": "update",
  "result": {
    "t": 1606293275123,
    "u": 48733182,
    "s": "BTC_USDT",
    "b": "19177.79",
    "B": "0.0003341504",
    "a": "19179.38",
    "A": "0.09"
  }
}
```

</Example>

通知结果格式：

| 字段     | 类型    | 描述                     |
| -------- | ------- | ------------------------ |
| `result` | Object  | Order book ticker object |
| » `t`    | Integer | 深度更新毫秒时间戳       |
| » `u`    | Integer | 深度更新 ID              |
| » `s`    | String  | 货币对                   |
| » `b`    | String  | 最优买单价               |
| » `B`    | String  | 最优买单数量             |
| » `a`    | String  | 最优卖单价               |
| » `A`    | String  | 最优卖单数量             |

</Block>

<Block>

## 深度增量更新频道

`spot.order_book_update`

**更新速度:** 20ms 或 100ms

</Block>

<Block>

### 客户端订阅

代码示例：

<Example>

代码示例

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.order_book_update",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT", "100ms"]
}))
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.order_book_update",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT", "100ms"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段         | 类型            | 必选 | 描述                                   |
| ------------ | --------------- | ---- | -------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数，从左到右是 `cp`， `interval` |
| » `cp`       | String          | 是   | 货币对                                 |
| » `interval` | String          | 是   | 更新速度: `20ms` 或 `100ms`（`20ms` 对应的深度层级是 `20`，`100ms` 对应的深度层级是 `100`）         |

#### 字段枚举值

| Property | Value  |
| -------- | ------ |
| interval | 20ms  |
| interval | 100ms  |

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

### 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606294781,
  "time_ms": 1606294781236,
  "channel": "spot.order_book_update",
  "event": "update",
  "result": {
    "t": 1606294781123,
    "full": true,
    "l": "100",
    "e": "depthUpdate",
    "E": 1606294781,
    "s": "BTC_USDT",
    "U": 48776301,
    "u": 48776306,
    "b": [
      ["19137.74", "0.0001"],
      ["19088.37", "0"]
    ],
    "a": [["19137.75", "0.6135"]]
  }
}
```

</Example>

通知结果格式：

| 字段              | 类型                    | 描述                                                   |
| ----------------- | ----------------------- | ------------------------------------------------------ |
| `result`          | Object                  | 自从上次更新变化后的卖买单列表（asks/bids）            |
| » `t`             | Integer                 | 深度更新变化毫秒时间戳                                 |
| » `full`          | Boolean                 | `true` 代表全量的深度（假设订阅 100ms，推送出来则是 100 层的深度）；用户接收到之后需要替换本地深度；`false` 代表增量的深度，为 `false` 时，不传输该字段     |
| » `l`             | String                  | 深度层级（例如 100 即代表 100 层的深度更新）            |
| » `e`             | String                  | 忽略这个参数                                           |
| » `E`             | Integer                 | 深度更新时间戳（精确到秒）.已经弃用，建议使用 `t` 字段 |
| » `s`             | String                  | 货币对                                                 |
| » `U`             | Integer                 | 自动上次深度后的第一个深度 ID                          |
| » `u`             | Integer                 | 自动上次深度后的最新深度 ID                            |
| » `b`             | `Array[OrderBookArray]` | 自上次更新以来的 bids 更新，按价格从高到低排序         |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                 |
| » `a`             | `Array[OrderBookArray]` | 自上次更新以来的 asks 更新，按价格从地到高排序         |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                 |

</Block>

<Block>

## 深度全量更新频道

`spot.order_book`

**更新速度:** 1000ms 或 100ms

</Block>

<Block>

### 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.order_book",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT", "5", "100ms"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.order_book",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT", "5", "100ms"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段         | 类型            | 必选 | 描述                                               |
| ------------ | --------------- | ---- | -------------------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数，从左到右分别是 `cp`, `level`, `interval` |
| » `cp`       | String          | 是   | 货币对                                             |
| » `level`    | String          | 是   | 深度层级                                           |
| » `interval` | String          | 是   | 通知更新速度                                       |

#### 字段枚举值

| Property | Value  |
| -------- | ------ |
| level    | 5      |
| level    | 10     |
| level    | 20     |
| level    | 30     |
| level    | 50     |
| level    | 100    |
| interval | 100ms  |
| interval | 1000ms |

::: tip
此频道不需要身份验证
:::

</Block>

<Block>

### 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606295412,
  "time_ms": 1606295412213,
  "channel": "spot.order_book",
  "event": "update",
  "result": {
    "t": 1606295412123,
    "lastUpdateId": 48791820,
    "s": "BTC_USDT",
    "l": "5",
    "bids": [
      ["19079.55", "0.0195"],
      ["19079.07", "0.7341"],
      ["19076.23", "0.00011808"],
      ["19073.9", "0.105"],
      ["19068.83", "0.1009"]
    ],
    "asks": [
      ["19080.24", "0.1638"],
      ["19080.91", "0.1366"],
      ["19080.92", "0.01"],
      ["19081.29", "0.01"],
      ["19083.8", "0.097"]
    ]
  }
}
```

</Example>

通知结果格式：

| 字段              | 类型                    | 描述                                                    |
| ----------------- | ----------------------- | ------------------------------------------------------- |
| `result`          | Object                  | 深度信息        |
| » `t`             | Integer                 | 深度更新毫秒时间戳                                      |
| » `lastUpdateId`  | Integer                 | 深度副本当前的深度 ID                                   |
| » `s`             | String                  | 货币对                                                  |
| » `l`             | String                  | 深度层级（例如 100 即代表 100 层的深度更新）            |
| » `bids`          | `Array[OrderBookArray]` | 在当前快照的 顶层 bids（买单） 数据，按价格从高到低排序 |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                  |
| » `asks`          | `Array[OrderBookArray]` | 在当前快照的 顶层 asks（卖单） 数据，按价格从低到高排序 |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                  |

</Block>

<Block>

# RPI 深度全量更新频道

`spot.rpi_order_book`

推送 RPI 订单簿的全量快照。本频道是公开频道。

订阅参数和推送字段与 `spot.order_book` 相同。同一时刻两条频道的买卖盘可以不同，本频道对齐 RPI 盘口。

**更新速度:** `100ms`、`500ms` 或 `1000ms`

SBE 复用 `orderBook` 模板（templateId=`4`），帧里的 `channel` 为 `spot.rpi_order_book`。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "spot.rpi_order_book",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT", "20", "100ms"]
}))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	subscribe := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "spot.rpi_order_book",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT", "20", "100ms"},
	}

	msg, err := json.Marshal(subscribe)
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

订阅参数格式:

| 字段         | 类型            | 必选 | 描述                                               |
| ------------ | --------------- | ---- | -------------------------------------------------- |
| `payload`    | `Array[String]` | 是   | 订阅参数，从左到右分别是 `cp`, `level`, `interval` |
| » `cp`       | String          | 是   | 货币对                                             |
| » `level`    | String          | 是   | 深度层级                                           |
| » `interval` | String          | 是   | 通知更新速度                                       |

#### 字段枚举值

| Property | Value  |
| -------- | ------ |
| level    | 5      |
| level    | 10     |
| level    | 20     |
| level    | 30     |
| level    | 50     |
| level    | 100    |
| interval | 100ms  |
| interval | 500ms  |
| interval | 1000ms |

取消订阅使用相同的 `payload`，`event` 改为 `unsubscribe`。

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1606295412,
  "time_ms": 1606295412213,
  "channel": "spot.rpi_order_book",
  "event": "update",
  "result": {
    "t": 1606295412123,
    "lastUpdateId": 48791820,
    "s": "BTC_USDT",
    "l": "20",
    "bids": [
      ["19079.55", "0.0195"],
      ["19079.07", "0.7341"]
    ],
    "asks": [
      ["19080.24", "0.1638"],
      ["19080.91", "0.1366"]
    ]
  }
}
```

</Example>

通知结果格式：

| 字段              | 类型                    | 描述                                                    |
| ----------------- | ----------------------- | ------------------------------------------------------- |
| `result`          | Object                  | 深度信息                                                |
| » `t`             | Integer                 | 深度更新毫秒时间戳                                      |
| » `lastUpdateId`  | Integer                 | 深度副本当前的深度 ID                                   |
| » `s`             | String                  | 货币对                                                  |
| » `l`             | String                  | 深度层级（例如 20 即代表 20 层的深度更新）              |
| » `bids`          | `Array[OrderBookArray]` | 在当前快照的顶层 bids（买单）数据，按价格从高到低排序   |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                  |
| » `asks`          | `Array[OrderBookArray]` | 在当前快照的顶层 asks（卖单）数据，按价格从低到高排序   |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对                                  |

买卖档数不超过订阅的 `level`。

</Block>

<Block>

# 深度频道V2

**提供一种更快更新的获取深度信息的方法.**

`spot.obu` 与 `spot.rpi_obu` 都使用这一套本地深度维护方式。`spot.rpi_obu` 见下方 [RPI 深度频道V2](#rpi-深度频道v2)。

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
3. 订阅限制： 针对同一现货的同一深度流，一个链接只允许订阅一次，重复订阅会返回错误。失败示例：

<Example>

```json
{
  "time": 1747391482,
  "time_ms": 1747391482960,
  "id": 1,
  "conn_id": "d9db9373dc5e081e",
  "trace_id": "ee001938590e183db957bd5ba71651c0",
  "channel": "spot.obu",
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

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send('{"time" : 123456, "channel" : "spot.obu",
        "event": "subscribe", "payload" : ["ob.BTC_USDT.50"]}')
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "spot.obu", "event": "subscribe", "payload" : ["ob.BTC_USDT.50"]}`

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
    "time": 1747054611,
    "time_ms": 1747054611614,
    "conn_id": "d7de96c024f2a5b2",
    "trace_id": "e6fd9bdd617fcdb80d0762ffa33e71f6",
    "channel": "spot.obu",
    "event": "subscribe",
    "payload": [
        "ob.BTC_USDT.50"
    ],
    "result": {
        "status": "success"
    }
}
```

</Example>

### 请求参数

- channel

  `spot.obu`
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
    "channel": "spot.obu",
    "result": {
        "t": 1747054612673,
        "full": true,
        "s": "ob.BTC_USDT.50",
        "u": 73777715168,
        "b": [
            ["104027.1","509392"],
            ["104027", "477932"]
        ],
        "a": [
            ["104027.2", "44617"],
            ["104027.4", "39322"]
        ]
    },
    "time_ms": 1747054612848
}
```

增量推送示例:

```json
{
    "channel": "spot.obu",
    "result": {
        "t": 1747054612695,
        "s": "ob.BTC_USDT.50",
        "U": 73777715169,
        "u": 73777715212,
        "b": [
            ["104024.5", "10343"],
            ["104014.5", "509392"]
        ],
        "a": [
            ["104027.2", "0"],
            ["104027.4", "0"]
        ]
    },
    "time_ms": 1747054612925
}
```

</Example>

**深度频道V2的消息推送**

### 推送参数

- channel

  `spot.obu`
- event

  `update`
- params


| 名称                | 类型                    | 描述                                                  |
| ------------------- | ----------------------- | ----------------------------------------------------- |
| `result`            | Object                  | 自上次更新以来发生变更要价和出价                      |
| »`t`               | Integer                 | 订单簿生成时间戳（以毫秒为单位）                      |
| »`full`             | Boolean                 | `true` 代表全量的深度；`false` 代表增量的深度，为 `false` 时，不传输该字段     |
| »`s`               | String                  | 深度流的名称      |
| »`U`               | Integer                 | 本次更新开始的订单簿更新 ID                           |
| »`u`               | Integer                 | 本次更新结束的订单簿更新 ID                           |
| »`b`               | `Array[OrderBookArray]` | 自上次更新以来的 bids 更新                            |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对,  Amount=0应当从本地深度中移除 |
| »`a`               | `Array[OrderBookArray]` | 自上次更新以来的 asks 更新                            |
| »» OrderBookArray | `Array[String]`         | [Price, Amount] 数组对,  Amount=0应当从本地深度中移除 |

</Block>

<Block>

## 深度频道V2取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(
  '{"time" : 123456, "channel" : "spot.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT.50"]}')
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "spot.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT.50"]}`

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
    "channel": "spot.obu",
    "event": "unsubscribe",
    "payload": [
        "ob.BTC_USDT.50"
    ],
    "result": {
        "status": "success"
    }
}
```

</Example>

**取消指定合约的市场的深度频道V2订阅**

### 请求参数

- channel

  `spot.obu`
- event

  `unsubscribe`

</Block>

<Block>

# RPI 深度频道V2

`spot.rpi_obu`

推送 RPI 订单簿的全量和增量。本频道是公开频道。

消息结构、本地深度维护方式与 [深度频道V2](#深度频道v2) 相同。流名称前缀为 `rpi_ob.`。

SBE 复用 `obu` 模板（templateId=`3`），帧里的 `channel` 为 `spot.rpi_obu`。

</Block>

<Block>

## RPI 深度频道V2订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send('{"time" : 123456, "channel" : "spot.rpi_obu",
        "event": "subscribe", "payload" : ["rpi_ob.BTC_USDT.50"]}')
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "spot.rpi_obu", "event": "subscribe", "payload" : ["rpi_ob.BTC_USDT.50"]}`

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
    "time": 1747054611,
    "time_ms": 1747054611614,
    "conn_id": "d7de96c024f2a5b2",
    "trace_id": "e6fd9bdd617fcdb80d0762ffa33e71f6",
    "channel": "spot.rpi_obu",
    "event": "subscribe",
    "payload": [
        "rpi_ob.BTC_USDT.50"
    ],
    "result": {
        "status": "success"
    }
}
```

</Example>

### 请求参数

- channel

  `spot.rpi_obu`
- event

  `subscribe`
- params

  `payload` 是一个包含流名称的列表. 格式为: `rpi_ob.{symbol}.{level}`；例如 `rpi_ob.BTC_USDT.400`、`rpi_ob.BTC_USDT.50`

  其中 `level` 枚举为：400、50；更新频率: 400 档为 100ms；50 档为 20ms。

订阅成功后，首条推送为全量（`full=true`），之后为增量。同一现货的同一深度流，一个链接只允许订阅一次。

</Block>

<Block>

## RPI 深度v2订阅推送

<Example>

全量推送示例:

```json
{
    "channel": "spot.rpi_obu",
    "result": {
        "t": 1747054612673,
        "full": true,
        "s": "rpi_ob.BTC_USDT.50",
        "u": 73777715168,
        "b": [
            ["104027.1", "509392"],
            ["104027", "477932"]
        ],
        "a": [
            ["104027.2", "44617"],
            ["104027.4", "39322"]
        ]
    },
    "time_ms": 1747054612848
}
```

增量推送示例:

```json
{
    "channel": "spot.rpi_obu",
    "result": {
        "t": 1747054612695,
        "s": "rpi_ob.BTC_USDT.50",
        "U": 73777715169,
        "u": 73777715212,
        "b": [
            ["104024.5", "10343"]
        ],
        "a": [
            ["104027.2", "0"]
        ]
    },
    "time_ms": 1747054612925
}
```

</Example>

**RPI 深度频道V2的消息推送**

### 推送参数

- channel

  `spot.rpi_obu`
- event

  `update`
- params


| 名称                | 类型                    | 描述                                                  |
| ------------------- | ----------------------- | ----------------------------------------------------- |
| `result`            | Object                  | 自上次更新以来发生变更的卖单和买单                    |
| »`t`                | Integer                 | 订单簿生成时间戳（以毫秒为单位）                      |
| »`full`             | Boolean                 | `true` 代表全量的深度；`false` 代表增量的深度，为 `false` 时，不传输该字段 |
| »`s`                | String                  | 深度流的名称，例如 `rpi_ob.BTC_USDT.50`               |
| »`U`                | Integer                 | 本次更新开始的订单簿更新 ID                           |
| »`u`                | Integer                 | 本次更新结束的订单簿更新 ID                           |
| »`b`                | `Array[OrderBookArray]` | 自上次更新以来的 bids 更新                            |
| »» OrderBookArray   | `Array[String]`         | [Price, Amount] 数组对, Amount=0 应当从本地深度中移除 |
| »`a`                | `Array[OrderBookArray]` | 自上次更新以来的 asks 更新                            |
| »» OrderBookArray   | `Array[String]`         | [Price, Amount] 数组对, Amount=0 应当从本地深度中移除 |

</Block>

<Block>

## RPI 深度频道V2取消订阅

<Example>

代码示例

```python
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
ws.send(
  '{"time" : 123456, "channel" : "spot.rpi_obu", "event": "unsubscribe", "payload" : ["rpi_ob.BTC_USDT.50"]}')
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time" : 123456, "channel" : "spot.rpi_obu", "event": "unsubscribe", "payload" : ["rpi_ob.BTC_USDT.50"]}`

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

### 请求参数

- channel

  `spot.rpi_obu`
- event

  `unsubscribe`

</Block>

<Block>

# 订单频道

`spot.orders`

**更新速度:** 实时

通知已订阅货币对中创建的订单的更改，包括订单创建、成交、关闭和取消。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.orders",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.orders",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

如果你想订阅全部货币对订单更新，你可以使用`!all`来作为货币对列表参数。

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1694655225,
  "time_ms": 1694655225315,
  "channel": "spot.orders",
  "event": "update",
  "result": [
    {
      "id": "399123456",
      "text": "t-testtext",
      "create_time": "1694655225",
      "update_time": "1694655225",
      "currency_pair": "BTC_USD",
      "stock": "BTC",
      "money": "USDC",
      "trade_mode": 1,
      "type": "limit",
      "account": "spot",
      "side": "sell",
      "amount": "0.0001",
      "price": "26253.3",
      "time_in_force": "gtc",
      "left": "0.0001",
      "filled_total": "0",
      "filled_amount": "812.8",
      "avg_deal_price": "0",
      "fee": "0",
      "fee_currency": "USDC",
      "point_fee": "0",
      "gt_fee": "0",
      "rebated_fee": "0",
      "rebated_fee_currency": "USDC",
      "create_time_ms": "1694655225315",
      "update_time_ms": "1694655225315",
      "user": 3497082,
      "event": "put",
      "stp_id": 0,
      "stp_act": "-",
      "finish_as": "open",
      "biz_info": "-",
      "amend_text": "-",
      "slippage": "0.05"
    }
  ]
}
```

</Example>

更新的订单列表。请注意，可能会在一条通知中更新多个货币对的订单。

在统一订单簿中，`currency_pair` 仍为名义市场，`stock` 和 `money` 表示订单实际使用的资产。

通知结果格式：

| 字段                       | 类型            | 描述                                                                                                                                                                                                                                                                                                                                                                   |
|--------------------------| --------------- |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `result`                 | `Array[Object]` | 更新订单列表                                                                                                                                                                                                                                                                                                                                                               |
| » `id`                   | String          | 订单 ID                                                                                                                                                                                                                                                                                                                                                                |
| » `user`                 | Integer         | 用户 ID                                                                                                                                                                                                                                                                                                                                                                |
| » `text`                 | String          | 用户自定义订单信息                                                                                                                                                                                                                                                                                                                                                            |
| » `create_time`          | String          | 订单创建时间，精确到秒                                                                                                                                                                                                                                                                                                                                                          |
| » `create_time_ms`       | String          | 订单创建时间，精确到毫秒                                                                                                                                                                                                                                                                                                                                                         |
| » `update_time`          | String          | 订单最新更新时间，精确到秒                                                                                                                                                                                                                                                                                                                                                        |
| » `update_time_ms`       | String          | 订单最新更新时间，精确到毫秒                                                                                                                                                                                                                                                                                                                                                       |
| » `event`                | String          | 订单事件<br/> - `put`: 订单创建<br/> - `update`: 订单成交更新<br/> - `finish`: 订单关闭或者取消                                                                                                                                                                                                                                                                                            |
| » `currency_pair`        | String          | 交易货币对                                                                                                                                                                                                                                                                                                                                                                |
| » `stock`                | String          | 订单实际使用的基础资产                                                                                                                                                                                                                                                                                                                          |
| » `money`                | String          | 订单实际使用的计价资产                                                                                                                                                                                                                                                                                                                          |
| » `trade_mode`           | Integer         | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一                                                                                                                                                                                                                               |
| » `type`                 | String          | 订单类型                                                                                                                                                                                                                                                                                                                                                                 |
| » `account`              | String          | 账户类型. spot - 现货账户; margin - 杠杆账户; cross_margin - 全仓杠杆账户; unified - 统一账户                                                                                                                                                                                                                                                                                              |
| » `side`                 | String          | 买单或者卖单                                                                                                                                                                                                                                                                                                                                                               |
| » `amount`               | String          | 交易数量                                                                                                                                                                                                                                                                                                                                                                 |
| » `price`                | String          | 交易价                                                                                                                                                                                                                                                                                                                                                                  |
| » `time_in_force`        | String          | Time in force 策略。<br />\- gtc: GoodTillCancelled<br /> \- ioc: ImmediateOrCancelled ，立即成交或者取消，只吃单不挂单<br /> \- poc: PendingOrCancelled，被动委托，只挂单不吃单                                                                                                                                                                                                                    |
| » `left`                 | String          | 交易货币未成交数量                                                                                                                                                                                                                                                                                                                                                            |
| » `filled_total`         | String          | 已成交总金额                                                                                                                                                                                                                                                                                                                                                               |
| »`filled_amount`         | String          | 货币成交数量                                                                                                                                                                                                                                                                                                                                                               |
| »`avg_deal_price`        | String          | 订单成交均价                                                                                                                                                                                                                                                                                                                                                               |
| » `fee`                  | String          | 成交扣除的手续费                                                                                                                                                                                                                                                                                                                                                             |
| » `fee_currency`         | String          | 手续费计价单位                                                                                                                                                                                                                                                                                                                                                              |
| » `point_fee`            | String          | 手续费抵扣使用的点卡数量                                                                                                                                                                                                                                                                                                                                                         |
| » `gt_fee`               | String          | 手续费抵扣使用的 GT 数量                                                                                                                                                                                                                                                                                                                                                       |
| » `gt_discount`          | Boolean         | 是否开启 GT 抵扣                                                                                                                                                                                                                                                                                                                                                           |
| » `rebated_fee`          | String          | 返还的手续费                                                                                                                                                                                                                                                                                                                                                               |
| » `rebated_fee_currency` | String          | 返还手续费计价单位                                                                                                                                                                                                                                                                                                                                                            |
| »`auto_repay`            | Boolean         | 启用或禁用跨保证金订单生成的自动借贷的自动还款功能。默认情况下禁用。请注意以下事项:<br>1. 该字段仅对跨保证金订单有效。保证金账户不支持为订单设置自动还款 <br/>2. `auto_borrow` 和 `auto_repay` 不能同时在同一订单中设置为 true.                                                                                                                                                                                                                            |
| »`auto_borrow`           | Boolean         | 在杠杆或全仓杠杆交易中使用，以允许在余额不足时自动借入足够的金额                                                                                                                                                                                                                                                                                                                                     |
| »`stp_id`                | Integer         | 具有相同`stp_id`组的用户之间的订单不允许自成交。有关详细信息，请参阅 ApiV4。                                                                                                                                                                                                                                                                                                                        |
| »`stp_act`               | String          | 自成交预防操作。用户可以使用此字段设置自成交预防策略。                                                                                                                                                                                                                                                                                                                                          |
| »`finish_as`             | String          | 订单的完成状态说明如下.<br>\- `open`：等待处理 <br/>\- `filled`：完全成交 <br/>\- `cancelled`：用户撤销<br/>\- `ioc`：未立即成交，因为 tif 设置为 `ioc`<br/>\- `stp`：因自成交预防而取消<br/>\- `poc`：未满足挂单策略，因为 tif 设置为 `poc`<br/>\- `fok`：未立即完全成交，因为 tif 设置为 `fok`<br/>\- `trader_not_enough`：对手方不足导致撤单<br/>\- `depth_not_enough`：深度不足导致撤单<br/>\- `small`：订单数量太小<br/>\- `liquidate_cancelled`：爆仓取消<br/>\- `unified_check_failed`：统一市场校验失败，例如实际交易资产不受支持，或市场/币种处于暂停状态 <br/>\- `-`：未知 |
| »`amend_text`            | String          | 用户在修改订单时添加的自定义数据。                                                                                                                                                                                                                                                                                                                                                    |
| »`slippage`              | String          | 滑点，默认范围为0.0001-0.05，0.05代表5%，代表市价单能接受的最大滑点，不代表实际生效的滑点                                                                                                                                                                                                                                                                                                                |


#### 字段枚举值

| Property      | Value              | Description              |
| ------------- | ------------------ | ------------------ |
| type          | limit              | 现货账户下限价单              |
| type          | market             | 现货账户下市价单              |
| type          | limit_repay        | 统一账户下限价单成交后会触发自动还款        |
| type          | market_repay       | 统一账户下市价单成交后会触发自动还款       |
| type          | limit_borrow       | 统一账户下限价单前余额不足会借款后再下单       |
| type          | market_borrow      | 统一账户下市价单前余额不足会借款后再下单      |
| type          | limit_borrow_repay | 统一账户下限价单前余额不足会借款后再下单，下单成交后会触发自动还款 |
| type          | market_borrow_repay | 统一账户下市价单前余额不足会借款后再下单，下单成交后会触发自动还款 |
| account       | spot               | 现货账户               |
| account       | margin             | 杠杆账户             |
| account       | cross_margin       | 全仓杠杆账户          |
| account       | unified            | 统一账户       |
| side          | buy                | 买单                |
| side          | sell               | 卖单               |
| time_in_force | gtc                | 一直有效，直到用户取消或完全成交                |
| time_in_force | ioc                | 立即成交或者取消，只吃单不挂单                |
| time_in_force | poc                | 被动委托，只挂单不吃单                |

</Block>

<Block>

# 订单频道V2（轻量级通道）

`spot.orders_v2`

**订单频道 `spot.orders` & 订单频道V2 `spot.orders_v2` 区别：**
`spot.orders_v2` 不含手续费信息相关的字段。 具体有 `fee`, `point_fee`, `gt_fee` 和 `rebated_fee`

**更新速度:** 实时

通知已订阅货币对中创建的订单的更改，包括订单创建、成交、关闭和取消。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.orders_v2",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.orders_v2",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

如果你想订阅全部货币对订单更新，你可以使用`!all`来作为货币对列表参数。

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1736238443,
  "time_ms": 1736238443516,
  "channel": "spot.orders_v2",
  "event": "update",
  "result": [
    {
      "id": "769689142776",
      "text": "t-poc1736238443494",
      "create_time": "1736238443",
      "update_time": "1736238443",
      "currency_pair": "BTC_USD",
      "stock": "BTC",
      "money": "USDC",
      "trade_mode": 1,
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "0.78",
      "price": "3.9147",
      "time_in_force": "poc",
      "left": "0",
      "filled_total": "3.053466",
      "filled_amount": "812.8",
      "avg_deal_price": "3.9147",
      "fee_currency": "USDC",
      "gt_discount": true,
      "rebated_fee_currency": "USDC",
      "create_time_ms": "1736238443503",
      "update_time_ms": "1736238443506",
      "user": 3128780,
      "event": "finish",
      "stp_id": 0,
      "stp_act": "-",
      "finish_as": "filled",
      "biz_info": "-",
      "amend_text": "-",
      "slippage": "0.05"
    }
  ]
}
```

</Example>

更新的订单列表。请注意，可能会在一条通知中更新多个货币对的订单。

在统一订单簿中，`currency_pair` 仍为名义市场，`stock` 和 `money` 表示订单实际使用的资产。

通知结果格式：

| 字段                     | 类型            | 描述                                                                                                                                                                                                                            |
| ------------------------ | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `result`                 | `Array[Object]` | 更新订单列表                                                                                                                                                                                                                    |
| » `id`                   | String          | 订单 ID                                                                                                                                                                                                                         |
| » `user`                 | Integer         | 用户 ID                                                                                                                                                                                                                         |
| » `text`                 | String          | 用户自定义订单信息                                                                                                                                                                                                              |
| » `create_time`          | String          | 订单创建时间，精确到秒                                                                                                                                                                                                          |
| » `create_time_ms`       | String          | 订单创建时间，精确到毫秒                                                                                                                                                                                                        |
| » `update_time`          | String          | 订单最新更新时间，精确到秒                                                                                                                                                                                                      |
| » `update_time_ms`       | String          | 订单最新更新时间，精确到毫秒                                                                                                                                                                                                    |
| » `event`                | String          | 订单事件<br/> - `put`: 订单创建<br/> - `update`: 订单成交更新<br/> - `finish`: 订单关闭或者取消                                                                                                                                 |
| » `currency_pair`        | String          | 交易货币对                                                                                                                                                                                                                      |
| » `stock`                | String          | 订单实际使用的基础资产                                                                                                                                                                                                            |
| » `money`                | String          | 订单实际使用的计价资产                                                                                                                                                                                                            |
| » `trade_mode`           | Integer         | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一                                                                                                                                                                                         |
| » `type`                 | String          | 订单类型 |
| » `account`              | String          | 账户类型. spot - 现货账户; margin - 杠杆账户; cross_margin - 全仓杠杆账户; unified - 统一账户                                                                                                                                                        |
| » `side`                 | String          | 买单或者卖单                                                                                                                                                                                                                    |
| » `amount`               | String          | 交易数量                                                                                                                                                                                                                        |
| » `price`                | String          | 交易价                                                                                                                                                                                                                          |
| » `time_in_force`        | String          | Time in force 策略。<br />\- gtc: GoodTillCancelled<br /> \- ioc: ImmediateOrCancelled ，立即成交或者取消，只吃单不挂单<br /> \- poc: PendingOrCancelled，被动委托，只挂单不吃单                                                |
| » `left`                 | String          | 交易货币未成交数量                                                                                                                                                                                                              |
| » `filled_total`         | String          | 已成交总金额                                                                                                                                                                                                                    |
| »`filled_amount`            | String          | 货币成交数量                                                                   |
| »`avg_deal_price`        | String          | 订单成交均价                                                                                                                                                                                                                    |
| » `fee_currency`         | String          | 手续费计价单位                                                                                                                                                                                                                  |
| » `gt_discount`          | Boolean         | 是否开启 GT 抵扣                                                                                                                                                                                                                |
| » `rebated_fee_currency` | String          | 返还手续费计价单位                                                                                                                                                                                                              |
| »`auto_repay`            | Boolean         | 启用或禁用跨保证金订单生成的自动借贷的自动还款功能。默认情况下禁用。请注意以下事项:<br>1. 该字段仅对跨保证金订单有效。保证金账户不支持为订单设置自动还款 <br/>2. `auto_borrow` 和 `auto_repay` 不能同时在同一订单中设置为 true. |
| »`auto_borrow`           | Boolean         | 在杠杆或全仓杠杆交易中使用，以允许在余额不足时自动借入足够的金额                                                                                                                                                                |
| »`stp_id`                | Integer         | 具有相同`stp_id`组的用户之间的订单不允许自成交。有关详细信息，请参阅 ApiV4。                                                                                                                                                    |
| »`stp_act`               | String          | 自成交预防操作。用户可以使用此字段设置自成交预防策略。                                                                                                                                                                          |
| »`finish_as`             | String          | 订单的完成状态说明如下.<br>\- `open`：等待处理 <br/>\- `filled`：完全成交 <br/>\- `cancelled`：用户撤销<br/>\- `ioc`：未立即成交，因为 tif 设置为 `ioc`<br/>\- `stp`：因自成交预防而取消<br/>\- `poc`：未满足挂单策略，因为 tif 设置为 `poc`<br/>\- `fok`：未立即完全成交，因为 tif 设置为 `fok`<br/>\- `trader_not_enough`：对手方不足导致撤单<br/>\- `depth_not_enough`：深度不足导致撤单<br/>\- `small`：订单数量太小<br/>\- `liquidate_cancelled`：爆仓取消<br/>\- `unified_check_failed`：统一市场校验失败，例如实际交易资产不受支持，或市场/币种处于暂停状态 <br/>\- `-`：未知 |
| »`amend_text`            | String          | 用户在修改订单时添加的自定义数据。                                                                                                                                                                                              |
| »`slippage`              | String          | 滑点，默认范围为0.0001-0.05，0.05代表5%，代表市价单能接受的最大滑点，不代表实际生效的滑点                                                                                                                                                                                                                                                                                                                |

#### 字段枚举值

| Property      | Value              | Description              |
| ------------- | ------------------ | ------------------ |
| type          | limit              | 现货账户下限价单              |
| type          | market             | 现货账户下市价单              |
| type          | limit_repay        | 统一账户下限价单成交后会触发自动还款        |
| type          | market_repay       | 统一账户下市价单成交后会触发自动还款       |
| type          | limit_borrow       | 统一账户下限价单前余额不足会借款后再下单       |
| type          | market_borrow      | 统一账户下市价单前余额不足会借款后再下单      |
| type          | limit_borrow_repay | 统一账户下限价单前余额不足会借款后再下单，下单成交后会触发自动还款 |
| type          | market_borrow_repay | 统一账户下市价单前余额不足会借款后再下单，下单成交后会触发自动还款 |
| account       | spot               | 现货账户               |
| account       | margin             | 杠杆账户             |
| account       | cross_margin       | 全仓杠杆账户          |
| account       | unified            | 统一账户       |
| side          | buy                | 买单                |
| side          | sell               | 卖单               |
| time_in_force | gtc                | 一直有效，直到用户取消或完全成交                |
| time_in_force | ioc                | 立即成交或者取消，只吃单不挂单                |
| time_in_force | poc                | 被动委托，只挂单不吃单                |

</Block>

<Block>

<a id="spot-user-trades"></a>

# 用户私有成交频道

`spot.usertrades`

**更新速度:** 实时

通知用户在指定货币对中的个人交易。与`spot.trades` 频道不同，这是一个私有频道，并通知与用户相关的所有交易，不论交易角色（Maker/Taker）如何。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.usertrades",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.usertrades",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

如果你想订阅全部用户合约成交更新，你可以使用 `!all` 来作为合约列表参数。

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1605176741,
  "channel": "spot.usertrades",
  "event": "update",
  "result": [
    {
      "id": 5736713,
      "user_id": 1000001,
      "order_id": "30784428",
      "currency_pair": "BTC_USD",
      "create_time": 1605176741,
      "create_time_ms": "1605176741123.456",
      "side": "sell",
      "amount": "1.00000000",
      "role": "taker",
      "price": "10000.00000000",
      "fee": "0.00200000000000",
      "point_fee": "0",
      "gt_fee": "0",
      "text": "apiv4",
      "id_market": 917144,
      "stock": "BTC",
      "money": "USDC",
      "trade_mode": 1
    }
  ]
}
```

</Example>

更新的用户交易列表。

在统一订单簿中，`currency_pair` 仍为名义市场，`stock` 和 `money` 表示该笔成交实际使用的资产。

通知结果格式：

| 字段               | 类型               | 描述                     |
| ------------------ | ------------------ | ------------------------ |
| `result`           | `Array[UserTrade]` | Updated user trades list |
| » `id`             | Integer            | 成交 ID                  |
| » `user_id`        | Integer            | 用户 ID                  |
| » `order_id`       | String             | 关联的订单 ID            |
| » `currency_pair`  | String             | 交易货币对               |
| » `create_time`    | Integer            | 成交时间，精确到秒       |
| » `create_time_ms` | String             | 成交时间，毫秒精度       |
| » `side`           | String             | 买单或者卖单             |
| » `amount`         | String             | 交易数量                 |
| » `role`           | String             | 交易角色 (maker/taker)   |
| » `price`          | String             | 交易价                   |
| » `fee`            | String             | 成交扣除的手续费         |
| » `fee_currency`   | String             | 手续费计价单位           |
| » `point_fee`      | String             | 手续费抵扣使用的点卡数量 |
| » `gt_fee`         | String             | 手续费抵扣使用的 GT 数量 |
| » `text`           | String             | 用户自定义信息           |
| » `id_market`      | Integer            | 按市场成交ID |
| » `stock`          | String             | 该笔成交实际使用的基础资产 |
| » `money`          | String             | 该笔成交实际使用的计价资产 |
| » `trade_mode`     | Integer            | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一 |

#### 字段枚举值

| Property | Value |
| -------- | ----- |
| side     | buy   |
| side     | sell  |
| role     | maker |
| role     | taker |

</Block>
<Block>

<a id="spot-user-trades"></a>

# 用户私有成交频道V2（轻量级通道）

`spot.usertrades_v2`

**更新速度:** 实时

**用户私有成交频道 `spot.usertrades` & 用户私有成交频道V2 `spot.usertrades_v2` 区别：**
`spot.usertrades_v2` 不含手续费信息相关的字段。 具体有 `fee`, `point_fee`, `gt_fee` 和 `fee_currency`

通知用户在指定货币对中的个人交易。与`spot.trades` 频道不同，这是一个私有频道，并通知与用户相关的所有交易，不论交易角色（Maker/Taker）如何。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.usertrades_v2",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload": ["BTC_USDT"]
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.usertrades_v2",
		"event":   "subscribe",
		"payload": []string{"BTC_USDT"},
	}

	// refer to Authentication section for gen_sign implementation
  request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

订阅参数格式:

| 字段      | 类型            | 必选 | 描述       |
| --------- | --------------- | ---- | ---------- |
| `payload` | `Array[String]` | 是   | 货币对列表 |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

如果你想订阅全部用户合约成交更新，你可以使用 `!all` 来作为合约列表参数。

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1736237480,
  "time_ms": 1736237480397,
  "channel": "spot.usertrades_v2",
  "event": "update",
  "result": [
    {
      "id": 12855056637,
      "user_id": 3128780,
      "order_id": "769683030088",
      "currency_pair": "BTC_USD",
      "create_time": 1736237480,
      "create_time_ms": "1736237480369.211",
      "side": "sell",
      "amount": "32462.7",
      "role": "maker",
      "price": "0.041828",
      "text": "t-poc1736237480359",
      "amend_text": "-",
      "biz_info": "-",
      "id_market": 917144,
      "stock": "BTC",
      "money": "USDC",
      "trade_mode": 1
    }
  ]
}
```

</Example>

更新的用户交易列表。

在统一订单簿中，`currency_pair` 仍为名义市场，`stock` 和 `money` 表示该笔成交实际使用的资产。该轻量频道仍不包含 `fee`、`fee_currency`、`point_fee` 和 `gt_fee`。

通知结果格式：

| 字段                 | 类型               | 描述                       |
|--------------------| ------------------ |--------------------------|
| `result`           | `Array[UserTrade]` | Updated user trades list |
| » `id`             | Integer            | 成交 ID                    |
| » `user_id`        | Integer            | 用户 ID                    |
| » `order_id`       | String             | 关联的订单 ID                 |
| » `currency_pair`  | String             | 交易货币对                    |
| » `create_time`    | Integer            | 成交时间，精确到秒                |
| » `create_time_ms` | String             | 成交时间，毫秒精度                |
| » `side`           | String             | 买单或者卖单                   |
| » `amount`         | String             | 交易数量                     |
| » `role`           | String             | 交易角色 (maker/taker)       |
| » `price`          | String             | 交易价                      |
| » `text`           | String             | 用户自定义信息                  |
| » `biz_info`       | String             | 交易价                      |
| » `id_market`      | Integer             | 按市场成交ID                  |
| » `stock`          | String              | 该笔成交实际使用的基础资产      |
| » `money`          | String              | 该笔成交实际使用的计价资产      |
| » `trade_mode`     | Integer             | 交易模式：`0` 普通市场、`1` 计价侧统一、`2` 基础侧统一、`3` 两侧统一 |

#### 字段枚举值

| Property | Value |
| -------- | ----- |
| side     | buy   |
| side     | sell  |
| role     | maker |
| role     | taker |

</Block>

<Block>

# 现货余额频道

`spot.balances`

**更新速度:** 实时

通知用户现货账户余额更新

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.balances",
    "event": "subscribe",  # "unsubscribe" for unsubscription
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.balances",
		"event":   "subscribe",
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

不需要 payload

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1605248616,
  "time_ms": 1605248616763,
  "channel": "spot.balances",
  "event": "update",
  "result": [
    {
      "timestamp": "1667556323",
      "timestamp_ms": "1667556323730",
      "user": "1000001",
      "currency": "USDT",
      "change": "0",
      "total": "222244.3827652",
      "available": "222244.3827",
      "freeze": "5",
      "freeze_change": "5.000000",
      "change_type": "order-create"
    }
  ]
}
```

</Example>

通知结果格式：

| 字段             | 类型                 | 描述                                                                                                                                                                                                                                                                                            |
| ---------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `result`         | `Array[SpotBalance]` | 新的余额更新列表                                                                                                                                                                                                                                                                                |
| » `timestamp`    | String               | 余额更新时间戳戳，精确到秒                                                                                                                                                                                                                                                                      |
| » `timestamp_ms` | String               | 余额更新时间戳戳，精确到毫秒                                                                                                                                                                                                                                                                    |
| » `user`         | String               | 用户 ID                                                                                                                                                                                                                                                                                         |
| » `currency`     | String               | 变更的货币                                                                                                                                                                                                                                                                                      |
| » `change`       | String               | 变更数量                                                                                                                                                                                                                                                                                        |
| » `total`        | String               | 总现货余额                                                                                                                                                                                                                                                                                      |
| » `available`    | String               | 可用现货余额                                                                                                                                                                                                                                                                                    |
| »`freeze`        | String               | 冻结余额数量                                                                                                                                                                                                                                                                                    |
| »`freeze_change` | String               | 余额锁定的变动金额                                                                                                                                                                                                                                                                              |
| »`change_type`   | String               | 余额变动类型<br/> - `withdraw`: 提现或取消提现 <br/> - `deposit`: 充值 <br/> - `trade-fee-deduct`: 从账户扣出手续费 <br/> - `order-create`: 订单创建 <br/> - `order-match`: 订单成交更新 <br/> - `order-update`: 取消订单或修改订单 <br/> - `margin-transfer`: 现货-逐仓杠杆划转 <br/> - `future-transfer`: 现货-永续合约划转 <br/> - `cross-margin-transfer`: 现货-全仓杠杆划转 <br/> - `referral-fee`: 推荐返佣金 <br/> - `sub-transfer`: 子账户划转 <br/> - `spot-transfer`: 现货-现货划转 <br/> - `other` |

</Block>

<Block>

# 保证金余额频道

`spot.margin_balances`

**更新速度:** 实时

通知用户保证金余额更新。任何保证金资金的存入或借款都将生成新的通知。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.margin_balances",
    "event": "subscribe",  # "unsubscribe" for unsubscription
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.margin_balances",
		"event":   "subscribe",
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

不需要 payload

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1605248616,
  "channel": "spot.margin_balances",
  "event": "update",
  "result": [
    {
      "timestamp": "1605248812",
      "timestamp_ms": "1605248812123",
      "user": "1000001",
      "currency_pair": "BTC_USDT",
      "currency": "BTC",
      "change": "-0.002",
      "available": "999.999836",
      "freeze": "1",
      "borrowed": "0",
      "interest": "0"
    }
  ]
}
```

</Example>

通知结果格式：

| 字段              | 类型                   | 描述                           |
| ----------------- | ---------------------- | ------------------------------ |
| `result`          | `Array[MarginBalance]` | 保证金余额更新列表             |
| » `timestamp`     | String                 | 保证金更新时间戳戳，精确到秒   |
| » `timestamp_ms`  | String                 | 保证金更新时间戳戳，精确到毫秒 |
| » `user`          | String                 | 用户 ID                        |
| » `currency_pair` | String                 | 货币对                         |
| » `currency`      | String                 | 变化货币                       |
| » `change`        | String                 | 变更数量                       |
| » `available`     | String                 | 可用保证金余额                 |
| » `freeze`        | String                 | 锁定的金额，例如用于资金账簿   |
| » `borrowed`      | String                 | 借贷总额                       |
| » `interest`      | String                 | 借款未付利息总额               |

</Block>

<Block>

# 借贷余额频道

`spot.funding_balances`

**更新速度:** 实时

通知用户资金余额更新，包括新的借贷贷款的创建、取消或被他人借用。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.funding_balances",
    "event": "subscribe",  # "unsubscribe" for unsubscription
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.funding_balances",
		"event":   "subscribe",
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

不需要 payload

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1605248616,
  "channel": "spot.funding_balances",
  "event": "update",
  "result": [
    {
      "timestamp": "1605248616",
      "timestamp_ms": "1605248616123",
      "user": "1000001",
      "currency": "USDT",
      "change": "100",
      "freeze": "100",
      "lent": "0"
    }
  ]
}
```

</Example>

通知结果格式：

| 字段             | 类型                    | 描述                         |
| ---------------- | ----------------------- | ---------------------------- |
| `result`         | `Array[FundingBalance]` | 借贷余额更新列表             |
| » `timestamp`    | String                  | 更新时间戳，精确到秒         |
| » `timestamp_ms` | String                  | 更新时间戳，精确到毫秒       |
| » `user`         | String                  | 用户 ID                      |
| » `currency`     | String                  | 变更货币                     |
| » `change`       | String                  | 变更数量                     |
| » `freeze`       | String                  | 锁定的金额，例如用于资金账簿 |
| » `lent`         | String                  | 贷出款额                     |

</Block>

<Block>

# 全仓杠杆余额频道

`spot.cross_balances`

**更新速度:** 实时

通知用户全仓杠杆账户余额更新

注意，未完成的订单在部分或全部完成之前不会触发余额更新。

</Block>

<Block>

## 客户端订阅

<Example>

代码示例：

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.cross_balances",
    "event": "subscribe",  # "unsubscribe" for unsubscription
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
print(ws.recv())
```

</Example>

<Example>

代码示例：

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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.cross_balances",
		"event":   "subscribe",
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

不需要 payload

::: warning
需要认证
:::

</Block>

<Block>

## 服务端推送

<Example>

通知示例：

```json
{
  "time": 1605248616,
  "time_ms": 1605248616763,
  "channel": "spot.cross_balances",
  "event": "update",
  "result": [
    {
      "timestamp": "1605248616",
      "timestamp_ms": "1605248616123",
      "user": "1000001",
      "currency": "USDT",
      "change": "100",
      "total": "1032951.325075926",
      "available": "1022943.325075926",
      "freeze": "0",
      "freeze_change": "0",
      "change_type": "cross-margin-transfer"
    }
  ]
}
```

</Example>

通知结果格式：

| 字段             | 类型                 | 描述                                                                                                                                                                                                                                                                                            |
| ---------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `result`         | `Array[SpotBalance]` | 余额更新列表                                                                                                                                                                                                                                                                                    |
| » `timestamp`    | String               | 更新时间戳，精确到秒                                                                                                                                                                                                                                                                            |
| » `timestamp_ms` | String               | 更新时间戳，精确到毫秒                                                                                                                                                                                                                                                                          |
| » `user`         | String               | 用户 ID                                                                                                                                                                                                                                                                                         |
| » `currency`     | String               | 变更货币                                                                                                                                                                                                                                                                                        |
| » `change`       | String               | 变更数量                                                                                                                                                                                                                                                                                        |
| » `total`        | String               | 总现货余额                                                                                                                                                                                                                                                                                      |
| » `available`    | String               | 可用余额                                                                                                                                                                                                                                                                                        |
| »`freeze`        | String               | 冻结余额数量                                                                                                                                                                                                                                                                                    |
| »`freeze_change` | String               | 余额锁定的变动金额                                                                                                                                                                                                                                                                              |
| »`change_type`   | String               | 余额变动类型<br/> - `withdraw` <br/> - `deposit` <br/> - `trade-fee-deduct` <br/> - `order-create`: 订单创建 <br/> - `order-match`: 订单成交更新 <br/> - `order-update`: 取消订单或修改订单 <br/> - `margin-transfer` <br/> - `future-transfer` <br/> - `cross-margin-transfer` <br/> - `other` |

</Block>

<Block>

# 全仓保证金借贷频道 (废弃)

`spot.cross_loan`

**更新速度:** 实时

**此频道已被弃用，因为数据已过时**

通知用户交叉保证金借款和利息更新

任何交叉保证金借款或还款操作都会生成新的通知。

1. 如果发生贷款或还款操作，将同时通知此频道和`spot.cross_balances`（余额变动）频道，但`spot.cross_balances`
   频道通知只包含余额变动（没有贷款信息），而此频道通知将包含余额、已借金额和利息信息。
2. 此频道仅在发生贷款或还款时通知，如果只有交易发生，此频道不会通知。

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

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.cross_loan",
    "event": "subscribe",  # "unsubscribe" for unsubscription
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.cross_loan",
		"event":   "subscribe",
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

不需要 payload

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
  "time": 1658289372,
  "time_ms": 1658289372763,
  "channel": "spot.cross_loan",
  "event": "update",
  "result": {
    "timestamp": 1658289372338,
    "user": "1000001",
    "currency": "BTC",
    "change": "0.01",
    "total": "4.992341029566",
    "available": "0.078054772536",
    "borrowed": "0.01",
    "interest": "0.00001375"
  }
}
```

</Example>

推送格式:

| 字段         | 类型   | 描述                             |
| ------------ | ------ | -------------------------------- |
| `result`     | Object | 新的全仓保证金借款和利息更新消息 |
| »`timestamp` | int64  | 秒级时间戳                       |
| »`user`      | String | 用户 ID                          |
| »`currency`  | String | 变更的货币                       |
| »`change`    | String | 变更数量                         |
| »`total`     | String | 总现货余额                       |
| »`available` | String | 可用余额                         |
| »`borrowed`  | String | 已借金额                         |
| »`interest`  | String | 从借款产生的未付利息总额         |

</Block>

<Block>

# 自动订单频道

`spot.priceorders`

**更新速度:** 实时

通知用户自动订单的更新。

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

ws = create_connection("wss://api.gateio.ws/ws/v4/")
request = {
    "time": int(time.time()),
    "channel": "spot.priceorders",
    "event": "subscribe",  # "unsubscribe" for unsubscription
    "payload" : ["!all"],
}
# refer to Authentication section for gen_sign implementation
request['auth'] = gen_sign(request['channel'], request['event'], request['time'])
ws.send(json.dumps(request))
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	request := map[string]interface{}{
		"time":    timestamp,
		"channel": "spot.priceorders",
		"event":   "subscribe",
		"payload": []string{"!all"},
	}

	// refer to Authentication section for gen_sign implementation
	request["auth"] = genSign(request["channel"].(string), request["event"].(string), timestamp)

	msg, err := json.Marshal(request)
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

Payload 格式:

| 字段      | 类型            | 必选 | 描述                   |
| --------- | --------------- | ---- | ---------------------- |
| `payload` | `Array[String]` | 是   | List of currency pairs |

您可以多次订阅/取消订阅。除非明确取消订阅，否则之前订阅的货币对不会被覆盖。

如果你想订阅全部货币对订单更新，你可以使用`!all`来作为货币对列表参数。

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
  "time": 1691847986,
  "time_ms": 1691847986454,
  "channel": "spot.priceorders",
  "event": "update",
  "result": {
    "market": "METAN_USDT",
    "uid": "13679450",
    "id": "247480109",
    "currency_type": "METAN",
    "exchange_type": "USDT",
    "reason": "",
    "err_msg": "",
    "fired_order_id": 0,
    "instant_cancel": false,
    "trigger_price": "0.00302",
    "trigger_rule": "<=",
    "trigger_expiration": 900,
    "price": "0.00300",
    "amount": "26666.667",
    "source": "",
    "order_type": "limit",
    "side": "buy",
    "engine_type": "normal",
    "is_stop_order": false,
    "stop_trigger_price": "",
    "stop_trigger_rule": "",
    "stop_price": "",
    "ctime": "1691517983131",
    "ftime": "1691517983131"
  }
}
```

</Example>

推送格式:

| 字段                  | 类型   | 描述                                               |
| --------------------- | ------ | -------------------------------------------------- |
| `result`              | Object |                                                    |
| »`market`             | String | 市场名称                                           |
| »`uid`                | String | 用户 ID                                            |
| »`id`                 | String | 订单 id                                            |
| »`currency_type`      | String | 货币类型                                           |
| »`exchange_type`      | String | 交易货币类型                                       |
| »`reason`             | String | 订单结束的附加描述信息                             |
| »`err_msg`            | String | 错误信息                                           |
| »`fired_order_id`     | int    | 触发后委托单 ID                                    |
| »`instant_cancel`     | bool   | 是否即时取消                                       |
| »`trigger_price`      | String | 触发价格                                           |
| »`trigger_rule`       | String | 触发规则                                           |
| »`trigger_expiration` | int    | 触发最长等待触发时间，超时则取消该订单，单位是秒 s |
| »`price`              | String | 挂单价格                                           |
| »`amount`             | String | 挂单数量                                           |
| »`source`             | String | source                                             |
| »`order_type`         | String | 订单类型,目前默认为限价单                          |
| »`side`               | String | 买卖方向<br>\- buy: 买 <br/>\- sell: 卖            |
| »`engine_type`        | String | 引擎类型                                           |
| »`is_stop_order`      | bool   | 是否为止损单                                       |
| »`stop_trigger_price` | String | 触发价格                                           |
| »`stop_trigger_rule`  | String | 触发规则                                           |
| »`stop_price`         | String | 止损价格                                           |
| »`ctime`              | String | 创建时间                                           |
| »`ftime`              | String | 结束时间                                           |

</Block>

<Block>

# 现货账户交易

</Block>

<Block>

## Websocket API

WebSocket API 允许通过 WebSocket 连接下单、取消、修改和查询订单。

</Block>

<Block>

### Websocket API 客户端 API 请求

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


API_KEY = "YOUR_API_KEY"
SECRET = "YOUR_API_SECRET"
WS_URL = "wss://api.gateio.ws/ws/v4/"
CHANNEL_LOGIN = "spot.login"
CHANNEL_ORDER_PLACE = "spot.order_place"

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
        "text": "t-123456",
        "currency_pair": "ETH_BTC",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "iceberg": "0",
        "amount": "1",
        "price": "0.00032",
        "time_in_force": "gtc",
        "auto_borrow": False
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

if __name__ == "__main__":
    ws = websocket.WebSocketApp(
        WS_URL,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open
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
	channel := "spot.login"
	ts := time.Now().Unix()
	requestId := fmt.Sprintf("%d-%d", time.Now().UnixMilli(), 1)

	req := ApiRequest{
		Time:    ts,
		Channel: "spot.login",
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
	u := url.URL{Scheme: "ws", Host: "xx.xx.xxx.xx:xxx", Path: "xxx"}
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
	time.Sleep(2 * time.Second)

	//ws create an order
	orderParam := OrderParam{
		Text:         "t-123456",
		CurrencyPair: "ETH_BTC",
		Type:         "limit",
		Account:      "spot",
		Side:         "buy",
		Iceberg:      "0",
		Amount:       "1",
		Price:        "0.00032",
		TimeInForce:  "gtc",
		AutoBorrow:   false,
	}
	orderParamBytes, _ := json.Marshal(orderParam)

	//warn: if you want create batch_orders, the `RequestParam` : []byte([{orderParam},{orderParam},...])
	order_place := ApiRequest{
		Time:    ts,
		Channel: "spot.order_place",
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
	Text         string `json:"text,omitempty"`
	CurrencyPair string `json:"currency_pair,omitempty"`
	Type         string `json:"type,omitempty"`
	Account      string `json:"account,omitempty"`
	Side         string `json:"side,omitempty"`
	Iceberg      string `json:"iceberg,omitempty"`
	Amount       string `json:"amount,omitempty"`
	Price        string `json:"price,omitempty"`
	TimeInForce  string `json:"time_in_force,omitempty"`
	AutoBorrow   bool   `json:"auto_borrow,omitempty"`
	StpAct       string `json:"stp_act,omitempty"`
}

```

</Example>

客户端发起的`api`请求遵循通用的 JSON 格式，包含以下字段：

| 字段         | 类型    | 必选 | 描述                                                                       |
| ------------ | ------- | ---- | -------------------------------------------------------------------------- |
| `time`       | Integer | 是   | 请求时间（以秒为单位）。请求时间和服务器时间之间的差距不得超过 60 秒       |
| `id`         | Integer | 否   | 可选的请求 ID，服务器将回传该 ID，以帮助您识别服务器响应的是哪个请求       |
| `channel`    | String  | 是   | WebSocket 要订阅的频道                                                     |
| `event`      | String  | 是   | 固定为"api"                                                                |
| `payload`    | Object  | 是   | 可选的请求详细参数                                                         |
| »`req_id`    | String  | 是   | 由客户端提供的消息的唯一标识符。它将在响应消息中返回，用于识别相应的请求。 |
| »`req_param` | []Byte  | 是   | 请求的 API 参数                                                            |

请注意，`payload.req_param`的类型与频道相关。以`spot.order_place`为例，`payload.req_param`
与 apiv4 的 [/spot/orders](https://www.gate.com/docs/developers/apiv4/en/#create-an-order)
或 [/spot/batch_orders](https://www.gate.com/docs/developers/apiv4/en/#create-a-batch-of-orders) 相同。
您可以参考例子使用示例在 GT_USDT 上下一个限价订单。

</Block>

<Block>

### Websocket API 服务端推送

<Example>

服务端 ack 推送示例

```json
{
  "request_id": "request-2",
  "ack": true,
  "header": {
    "response_time": "1681985856667",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140033dc0c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gate_ratelimit_reset_timestamp": 1681985856667
  },
  "data": {
    "result": {
      "req_id": "request-2",
      "req_header": null,
      "req_param": {
        "text": "t-my-custom-id",
        "currency_pair": "BTC_USD",
        "trade_quote": "USDC",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "amount": "1",
        "price": "1"
      }
    }
  }
}
```

服务端 api 推送示例

```json
{
  "request_id": "request-2",
  "header": {
    "response_time": "1681986204784",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gate_ratelimit_reset_timestamp": 1681986204784
  },
  "data": {
    "result": {
      "id": "1700664330",
      "text": "t-my-custom-id",
      "amend_text": "-",
      "create_time": "1681986204",
      "update_time": "1681986204",
      "create_time_ms": 1681986204832,
      "update_time_ms": 1681986204832,
      "status": "open",
      "currency_pair": "GT_USDT",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "1",
      "price": "1",
      "time_in_force": "gtc",
      "iceberg": "0",
      "left": "1",
      "fill_price": "0",
      "filled_total": "0",
      "fee": "0",
      "fee_currency": "GT",
      "point_fee": "0",
      "gt_fee": "0",
      "gt_maker_fee": "0.0015",
      "gt_taker_fee": "0.0015",
      "gt_discount": true,
      "rebated_fee": "0",
      "rebated_fee_currency": "USDT",
      "stp_id": 1,
      "stp_act": "cn",
      "finish_as": "open"
    }
  }
}
```

</Example>

服务器响应包括对客户端请求的确认响应和 API 结果的回调消息更新。
服务器响应遵循通用的 JSON 格式，包含以下字段：

| 字段             | 类型   | 描述                                                                                                                                                               |
| ---------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                   |
| `header`         | Map    | 响应的元信息                                                                                                                                                       |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                       |
| »`channel`       | String | 请求的频道                                                                                                                                                         |
| »`event`         | String | 请求事件                                                                                                                                                           |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                  |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`x_gate_ratelimit_requests_remain`      | Integer | (仅涉及下单/改单)当前时间窗口剩余可用请求数(为0不展示)                       |
| »`x_gate_ratelimit_limit`                | Integer | (仅涉及下单/改单)当前频率限制上限(为0不展示)                     |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | (仅涉及下单/改单)已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒）     |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                                                                                                          |
| »`result`        | Object | 如果这是一个"ack"响应，那么"result"是请求的有效载荷（payload），否则"result"是 API 的响应。                                                                        |
| »`errs`          | Object | 这只在请求失败时可用                                                                                                                                               |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                           |
| »»`message`      | String | 错误消息详情                                                                                                                                                       |

</Block>

<Block>

### Error

<Example>

Error 推送示例

```json
{
  "request_id": "xxxx",
  "header": {
    "response_time": "1677816784084",
    "status": "401",
    "channel": "spot.login",
    "event": "api",
    "client_id": "::1-0x14002ba2300",
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
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x14002ba2300",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_limit": 10,
    "x_gate_ratelimit_reset_timestamp": 1677816785084
  },
  "data": {
    "errs": {
      "label": "TOO_MANY_REQUESTS",
      "message": "Request Rate limit Exceeded (211)"
    }
  }
}
```

</Example>

错误对象的格式如下：

<a id="schema_error"></a>

| 字段      | 类型   | 描述                     |
| --------- | ------ | ------------------------ |
| `label`   | String | 以字符串格式表示错误类型 |
| `message` | String | 错误信息详情             |

限频相关的错误码说明:

| 错误码      | 描述                     |
| --------- | ------------------------ |
| `100`   | 限流内部异常错误 |
| `211` | 现货限流    |
| `212` | 现货成交比率限流    |


</Block>

<Block>

## Login

::: warning
警告： 注意：您使用的 GateAPIv4 密钥对必须具有相应的权限（例如，订单下单频道必须具有现货写入权限），并且如果启用了密钥的 IP
白名单，则您的出站 IP 地址必须在白名单中。
:::

</Block>

<Block>

### Login 请求

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


API_KEY = "YOUR_API_KEY"
SECRET = "YOUR_API_SECRET"
WS_URL = "wss://api.gateio.ws/ws/v4/"
CHANNEL_LOGIN = "spot.login"

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

if __name__ == "__main__":
    ws = websocket.WebSocketApp(
        WS_URL,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        on_open=on_open
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
	"encoding/hex"
	"encoding/json"
	"fmt"
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
	channel := "spot.login"
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

	marshal, _ := json.Marshal(req)
	fmt.Println(string(marshal))

	// connect ws service
	u := url.URL{Scheme: "ws", Host: "xxxx", Path: "xxxx"}
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
  "channel": "spot.login",
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
Payload 格式:

| 字段         | 类型     | 必选 | 描述                                                                             |
| ------------ | -------- | ---- | -------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 请求 ID 将由服务器发送回，以帮助您识别服务器响应的是哪个请求，它与外部的`id`不同 |
| `api_key`    | `string` | 是   | apiv4 key                                                                        |
| `req_header` | `object` | 否   | apiv4 自定义 header                                                              |
| `signature`  | `string` | 是   | apiv4 签名                                                                       |
| `timestamp`  | `string` | 是   | Unix 时间戳（以秒为单位）                                                        |

WebSocket API 的操作身份验证使用与 Gate APIv4 API 相同的签名计算方法，即`HexEncode(HMAC_SHA512(secret, signature_string))`
，但有以下不同之处：

1. 签名字符串的连接方法：`"<event>\n<channel>\n<req_param>\n<timestamp>"`
   ，其中`<event>`、`<channel>`、`<req_param>`、`<timestamp>` 是对应的请求信息。
2. `login`频道中的`req_param`始终为空字符串。
3. 认证信息在请求体中以字段`payload`发送。

</Block>

<Block>

### Login 推送

<Example>

Login 推送示例

```json
{
  "request_id": "request-1",
  "header": {
    "response_time": "1681985856666",
    "status": "200",
    "channel": "spot.login",
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

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                                                             |
| ---------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                                                 |
| `header`         | Map    | 响应的元信息                                                                                                                                                                                     |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                                                     |
| »`channel`       | String | 请求的频道                                                                                                                                                                                       |
| »`event`         | String | 请求事件                                                                                                                                                                                         |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                                                |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                                                                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 如果这是"ack"响应，则"result"是请求的有效载荷，否则"result"是 API 的响应                                                                                                                         |
| »»`api_key`      | String | 登录成功的 API 密钥                                                                                                                                                                              |
| »»`uid`          | String | 登录用户 ID                                                                                                                                                                                      |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                                                           |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                                                         |
| »»`message`      | String | 错误信息详情                                                                                                                                                                                     |

</Block>

<Block>

## 下单

`spot.order_place`

您可以使用此频道和事件`api`下订单

**以下是 API 的功能:**

```json
POST /spot/orders
POST /spot/batch_orders
```

</Block>

<Block>

### 下单请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


placeParam = {"text":"t-my-custom-id","currency_pair":"BTC_USD","trade_quote":"USDC","type":"limit","account":"spot","side":"buy","amount":"1","price":"1"}
batchPlaceParam = [
                    {"text":"t-my-custom-id-1","currency_pair":"BTC_USD","trade_quote":"USDC","type":"limit","account":"spot","side":"buy","amount":"1","price":"1"},
                    {"text":"t-my-custom-id-2","currency_pair":"ETH_USD","trade_quote":"USDG","type":"limit","account":"spot","side":"buy","amount":"1","price":"1.1"}
                  ]


ws = create_connection("wss://api.gateio.ws/ws/v4/")
channel = "spot.order_place"

# refer to the Authentication section for a WebSocket API code example

# create a order
ws.send(json.dumps({
    "time":int(time.time()),
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"test_1",
        # create a order
        "req_param": placeParam
        # batch orders
        # "req_param": batchPlaceParam
    }
}))

for i in range(2):
    data = ws.recv()
    print("data: ", data)
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
	"net/url"
	"strconv"
	"time"
)

// example WebSocket create order in go
func main() {

	u := url.URL{Scheme: "ws", Host: "xxxx", Path: "xxx"}
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

	// warn: before order, you should login first, pls refer to the channel `spot.login`;
	// order_place
	orderParam := OrderParam{
		Text:         "t-123456",
		CurrencyPair: "BTC_USD",
		TradeQuote:   "USDC",
		Type:         "limit",
		Account:      "spot",
		Side:         "buy",
		Iceberg:      "0",
		Amount:       "1",
		Price:        "5.00032",
		TimeInForce:  "gtc",
		AutoBorrow:   false,
		StpAct:       "cn",
	}
	requestParam := interface{}(orderParam)
	// 批量下单时改为序列化 []OrderParam；每个订单分别携带自己的 trade_quote。
	// requestParam = []OrderParam{orderParam, {CurrencyPair: "ETH_USD", TradeQuote: "USDG", Type: "limit", Account: "spot", Side: "sell", Amount: "1", Price: "3000"}}
	paramBytes, _ := json.Marshal(requestParam)
	requestId := fmt.Sprintf("%d-%d", time.Now().UnixMilli(), 1)
	order_place := ApiRequest{
		Time:    time.Now().Unix(),
		Channel: "spot.order_place",
		Event:   "api",
		Payload: ApiPayload{
			RequestId:    requestId,
			RequestParam: []byte(paramBytes),
		},
	}
	orderPlaceReqByte, _ := json.Marshal(order_place)
	err = c.WriteMessage(websocket.TextMessage, orderPlaceReqByte)

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
	Text         string `json:"text,omitempty"`
	CurrencyPair string `json:"currency_pair,omitempty"`
	TradeQuote   string `json:"trade_quote,omitempty"`
	Type         string `json:"type,omitempty"`
	Account      string `json:"account,omitempty"`
	Side         string `json:"side,omitempty"`
	Iceberg      string `json:"iceberg,omitempty"`
	Amount       string `json:"amount,omitempty"`
	Price        string `json:"price,omitempty"`
	TimeInForce  string `json:"time_in_force,omitempty"`
	AutoBorrow   bool   `json:"auto_borrow,omitempty"`
	StpAct       string `json:"stp_act,omitempty"`
}
```

</Example>

<Example>

请求参数示例

下单

```json
{
  "time": 1681986203,
  "channel": "spot.order_place",
  "event": "api",
  "payload": {
    "req_id": "request-2",
    "req_param": {
      "text": "t-my-custom-id",
      "currency_pair": "BTC_USD",
      "trade_quote": "USDC",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "1",
      "price": "1"
    }
  }
}
```

批量下单

```json
{
  "time": 1681986203,
  "channel": "spot.order_place",
  "event": "api",
  "payload": {
    "req_id": "request-2",
    "req_param": [
      {
        "text": "t-my-custom-id-1",
        "currency_pair": "BTC_USD",
        "trade_quote": "USDC",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "amount": "1",
        "price": "1"
      },
      {
        "text": "t-my-custom-id-2",
        "currency_pair": "ETH_USD",
        "trade_quote": "USDG",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "amount": "2",
        "price": "1"
      }
    ]
  }
}
```

</Example>

Payload 格式:

| 字段         | 类型     | 必选 | 描述                                                                                                                                                                |
| ------------ | -------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`     | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的`id`不同                                                                                   |
| `req_param`  | `object` | 是   | API 订单模型的 JSON 字节数据，可以是包含 API 订单模型的数组；API 订单模型的详细信息请参考[API](https://www.gate.com/docs/developers/apiv4/zh_CN/#%E4%B8%8B%E5%8D%95) |
| `req_header` | `object` | 否   | apiv4 自定义 header                                                                                                                                                 |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `text`          | `string` | 否   | 订单自定义信息，用户可以用该字段设置自定义 ID，用户自定义字段必须满足以下条件：               |
| `currency_pair` | `string` | 是   | 交易货币对                                                                                    |
| `trade_quote`   | `string` | 否   | 订单实际使用的计价资产。统一市场要求时必填；普通市场不传或传空值。                                           |
| `type`          | `object` | 否   | 订单类型                                                                                      |
| `account`       | `string` | 否   | 账户类型，spot - 现货账户，margin - 杠杆账户，cross_margin - 全仓杠杆账户，unified - 统一账户 |
| `side`          | `string` | 是   | 买单或者卖单                                                                                  |
| `amount`        | `string` | 是   | 交易数量                                                                                      |
| `price`         | `string` | 否   | 交易价,`type`=`limit`时必填                                                                   |
| `time_in_force` | `object` | 否   | Time in force 策略。                                                                          |
| `iceberg`       | `string` | 否   | 冰山下单显示的数量，不指定或传 0 都默认为普通下单。目前不支持全部冰山。                       |
| `auto_borrow`   | `boolean` | 否   | 杠杆(包括逐仓全仓)交易时，如果账户余额不足，是否由系统自动借入不足部分                        |
| `auto_repay`    | `boolean` | 否   | 全仓杠杆下单是否开启自动还款，默认关闭。                                       |
| `stp_act`       | `string` | 否   | Self-Trading Prevention Action,用户可以用该字段设置自定义限制自成交策略。                     |
| `action_mode`   | `string` | 否   | 处理模式                     |
| `slippage`   | `string` | 否   | 滑点，默认范围为0.0001-0.05，0.05代表5%，代表市价单能接受的最大滑点                     |

`trade_quote` 在 WebSocket 层按原值透传。WebSocket 只校验 JSON 类型是否为字符串，不转换大小写、不做别名转换、不维护币种白名单。币种是否存在及市场是否支持，由下游交易服务按与 REST 相同的业务规则和错误语义校验。批量下单时，每个订单需分别设置 `trade_quote`。本期只对外提供计价侧选择，`trade_base`、`tqa` 和 `tba` 不是公开 WebSocket 参数。`spot.order_amend`、`spot.order_cancel` 和 `spot.order_cancel_ids` 不支持 `trade_quote`。

`req_header` 自定义 header 数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝               |

#### 详细描述

**text**: 订单自定义信息，用户可以用该字段设置自定义 ID，用户自定义字段必须满足以下条件：

- 必须以 t- 开头
- 不计算 t- ，长度不能超过 28 字节
- 输入内容只能包含数字、字母、下划线(\_)、中划线(-) 或者点(.)
- 不填，默认 `apiv4-ws`

**type**: 订单类型

- limit : 限价单
- market : 市价单

**account**: 账户类型，spot - 现货账户，margin - 杠杆账户，cross_margin - 全仓杠杆账户，unified - 统一账户
统一账户（旧）只能设置 `cross_margin`

**amount**: 交易数量
`type`为`limit`时，指交易货币，即需要交易的货币，如`BTC_USDT`中指`BTC`。
`type`为`market`时，根据买卖不同指代不同

- `side` : `buy` 指代计价货币，`BTC_USDT`中指`USDT`
- `side` : `sell` 指代交易货币，`BTC_USDT`中指`BTC`

**time_in_force**: Time in force 策略。

- gtc: GoodTillCancelled
- ioc: ImmediateOrCancelled，立即成交或者取消，只吃单不挂单
- poc: PendingOrCancelled，被动委托，只挂单不吃单
- fok: FillOrKill，全部成交或者全部取消
  `type`=`market`时仅支持`ioc`和`fok`

**auto_repay**: 全仓杠杆下单是否开启自动还款，默认关闭。需要注意的是:

- 此字段仅针对全仓杠杆有效。逐仓杠杆不支持订单级别的自动还款设置，只能通过 `POST /margin/auto_repay` 修改用户级别的设置
- `auto_borrow` 与 `auto_repay` 支持同时开启

**stp_act**: Self-Trading Prevention Action,用户可以用该字段设置自定义限制自成交策略。

- 用户在设置加入`STP用户组`后，可以通过传递 `stp_act` 来限制用户发生自成交的策略，没有传递 `stp_act` 默认按照 `cn` 的策略。
- 用户在没有设置加入`STP`用户组时，传递 `stp_act` 参数会报错。
- 用户没有使用 `stp_act` 发生成交的订单，`stp_act` 返回 `-`。

- cn: Cancel newest,取消新订单，保留老订单
- co: Cancel oldest,取消⽼订单，保留新订单
- cb: Cancel both,新旧订单都取消

**action_mode**: 处理模式:
下单时根据`action_mode`返回不同的字段, 该字段只在请求时有效，响应结果中不包含该字段。

- `ACK`: 异步模式，只返回订单关键字段
- `RESULT`: 无清算信息
- `FULL`: 完整模式（默认）

</Block>

<Block>

### Ack 推送

<Example>

单笔订单的 "ack" 推送示例

```json
{
  "request_id": "request-2",
  "ack": true,
  "header": {
    "response_time": "1681986203814",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "req_id": "request-2",
      "req_header": null,
      "req_param": {
        "text": "t-my-custom-id",
        "currency_pair": "GT_USDT",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "amount": "1",
        "price": "1"
      },
      "api_key": "",
      "timestamp": "",
      "signature": ""
    }
  }
}
```

**订单数组**的 "ack" 推送示例

```json
{
  "request_id": "xxxx",
  "ack": true,
  "header": {
    "response_time": "1677810708738",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140002f63c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "req_id": "xxxx",
      "req_header": null,
      "req_param": [
        {
          "text": "t-my-custom-id",
          "currency_pair": "GT_USDT",
          "type": "limit",
          "account": "spot",
          "side": "buy",
          "amount": "1",
          "price": "1"
        },
        {
          "text": "t-my-custom-id",
          "currency_pair": "GT_USDT",
          "type": "limit",
          "account": "spot",
          "side": "buy",
          "amount": "1",
          "price": "1"
        }
      ]
    }
  }
}
```

</Example>

</Block>

<Block>

### 下单推送

<Example>

下单 推送示例

单笔订单的推送示例

>action_mode: ACK

```json
{
  "request_id": "request-2",
  "header": {
    "response_time": "1681986204784",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": "12332324",
      "text": "t-123456",
      "amend_text": "test2"
    }
  }
}
```

>action_mode: RESULT

```json
{
  "request_id": "request-2",
  "header": {
    "response_time": "1681986204784",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": "12332324",
      "text": "t-123456",
      "create_time": "1548000000",
      "update_time": "1548000100",
      "create_time_ms": 1548000000123,
      "update_time_ms": 1548000100123,
      "currency_pair": "BTC_USD",
      "trade_quote": "USDC",
      "status": "cancelled",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "iceberg": "0",
      "amount": "1",
      "price": "5.00032",
      "time_in_force": "gtc",
      "auto_borrow": false,
      "left": "0.5",
      "filled_total": "2.50016",
      "avg_deal_price": "5.00032",
      "stp_act": "cn",
      "finish_as": "stp",
      "stp_id": 10240
    }
  }
}
```

>action_mode: FULL

```json
{
  "request_id": "request-2",
  "header": {
    "response_time": "1681986204784",
    "status": "200",
    "channel": "spot.order_place",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": "1852454420",
      "text": "t-abc123",
      "amend_text": "-",
      "create_time": "1710488334",
      "update_time": "1710488334",
      "create_time_ms": 1710488334073,
      "update_time_ms": 1710488334074,
      "status": "closed",
      "currency_pair": "BTC_USD",
      "trade_quote": "USDC",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "0.001",
      "price": "65000",
      "time_in_force": "gtc",
      "iceberg": "0",
      "left": "0",
      "filled_amount": "0.001",
      "fill_price": "63.4693",
      "filled_total": "63.4693",
      "avg_deal_price": "63469.3",
      "fee": "0.00000022",
      "fee_currency": "BTC",
      "point_fee": "0",
      "gt_fee": "0",
      "gt_maker_fee": "0",
      "gt_taker_fee": "0",
      "gt_discount": false,
      "rebated_fee": "0",
      "rebated_fee_currency": "USDC",
      "finish_as": "filled"
    }
  }
}
```

</Example>

下游返回非空值时，`trade_quote` 会出现在 RESULT 和 FULL 结果中；ACK 保持原有结果结构且不返回 `trade_quote`。请求直接校验失败时返回 API 错误且不产生订单 ID；订单被引擎接收后又因统一市场校验失败时，订单状态为 `cancelled`，且 `finish_as=unified_check_failed`。

已更新的订单列表。请注意，可能会在一条通知中更新多个货币对的订单

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                                                         |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                                             |
| `ack`            | Bool   | "ack"消息的返回表示 WebSocket 的确认消息(目前在下单接口中存在)。如果"ack"为 false(false 该字段不会出现在响应中)，这意味着该消息是一个响应消息，您可以通过检查"data.errs"来确定请求是否成功。 |
| `header`         | Map    | 响应的元信息                                                                                                                                                                                 |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                                                 |
| »`channel`       | String | 请求的频道                                                                                                                                                                                   |
| »`event`         | String | 请求事件                                                                                                                                                                                     |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                                            |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| »`x_gate_ratelimit_requests_remain`      | Integer | 当前时间窗口剩余可用请求数(为0不展示)                       |
| »`x_gate_ratelimit_limit`      | Integer | 当前频率限制上限(为0不展示)                     |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒）     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 如果这是"ack"响应，则"result"是请求的有效载荷，否则"result"是 API 的响应                                                                                                                     |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                                                       |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                                                     |
| »»`message`      | String | 错误信息详情                                                                                                                                                                                 |

</Block>

<Block>

## 取消订单

您可以使用此频道和事件`cancel`取消一个未成交的订单

**以下是 API 的功能:**

```json
DELETE /spot/orders/{
  order_id
}
```

</Block>

<Block>

### 取消订单请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


time = int(time.time())
cancelParam = {"order_id":"1694883366","currency_pair":"GT_USDT"}
channel = "spot.order_cancel"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"test_1",
        "req_param": cancelParam
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	cancelParam := map[string]interface{}{
		"order_id":    "1694883366",
		"currency_pair": "GT_USDT",
	}
	channel := "spot.order_cancel"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": cancelParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

取消订单请求示例

```json
{
  "time": 1681986206,
  "channel": "spot.order_cancel",
  "event": "api",
  "payload": {
    "req_id": "request-5",
    "req_param": {
      "order_id": "1700664330",
      "currency_pair": "GT_USDT"
    }
  }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                                            |
| ----------- | -------- | ---- | --------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。                             |
| `req_param` | `object` | 是   | 有关 API 取消订单的详细信息，请查看[API](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order)。 |
| `req_header` | `object` | 否   | apiv4 自定义 header   |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- | --- |
| `order_id`      | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即 `text` 字段）。                 |
| `currency_pair` | `string` | 是   | 交易货币对                                                                                    |     |
| `account`       | `string` | 否   | 账户类型，spot - 现货账户，margin - 杠杆账户，cross_margin - 全仓杠杆账户，unified - 统一账户 |

`req_header` 自定义 header 数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝               |

</Block>

<Block>

### 取消订单推送

<Example>

取消订单推送示例

```json
{
  "request_id": "request-5",
  "header": {
    "response_time": "1681986206282",
    "status": "200",
    "channel": "spot.order_cancel",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": {
      "id": "1700664330",
      "text": "t-my-custom-id",
      "amend_text": "-",
      "create_time": "1681986204",
      "update_time": "1681986206",
      "create_time_ms": 1681986204832,
      "update_time_ms": 1681986206330,
      "status": "cancelled",
      "currency_pair": "GT_USDT",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "1",
      "price": "2",
      "time_in_force": "gtc",
      "iceberg": "0",
      "left": "1",
      "fill_price": "0",
      "filled_total": "0",
      "fee": "0",
      "fee_currency": "GT",
      "point_fee": "0",
      "gt_fee": "0",
      "gt_maker_fee": "0.0015",
      "gt_taker_fee": "0.0015",
      "gt_discount": true,
      "rebated_fee": "0",
      "rebated_fee_currency": "USDT",
      "stp_id": 1,
      "stp_act": "cn",
      "finish_as": "cancelled"
    }
  }
}
```

</Example>

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                                                         |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                                             |
| `header`         | Map    | 响应的元信息                                                                                                                                                                                 |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                                                 |
| »`channel`       | String | 请求的频道                                                                                                                                                                                   |
| »`event`         | String | 请求事件                                                                                                                                                                                     |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                                            |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 单个订单取消的响应详细信息，请查看[API](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order)。                                                                               |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                                                       |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                                                     |
| »»`message`      | String | 错误信息详情                                                                                                                                                                                 |

</Block>

<Block>

## 取消所有 ID 列表内的订单

您可以使用此频道和事件`cancel_ids`取消所有未成交的订单。

**以下是 API 的功能:**

```json
POST /spot/cancel_batch_orders
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


time = int(time.time())
cancelWithIdsParam = [{"id":"1694883366","currency_pair":"GT_USDT"}]
channel = "spot.order_cancel_ids"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	cancelWithIdsParam := []map[string]interface{}{
		{
			"id":            "1694883366",
			"currency_pair": "GT_USDT",
		},
	}
	channel := "spot.order_cancel_ids"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": cancelWithIdsParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

客户端请求示例

```json
{
  "time": 1681986208,
  "channel": "spot.order_cancel_ids",
  "event": "api",
  "payload": {
    "req_id": "request-9",
    "req_param": [
      {
        "currency_pair": "GT_USDT",
        "id": "1700664343"
      }
    ]
  }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                                 |
| ----------- | -------- | ---- | ---------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。                  |
| `req_param` | `object` | 是   | 详情见 [api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-batch-of-orders-with-an-id-list) |
| `req_header` | `object` | 否   | apiv4 自定义 header   |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                       |
| --------------- | -------- | ---- | ------------------------------------------------------------------------------------------ |
| `currency_pair` | `string` | 是   | 交易货币对                                                                                 |
| `id`            | `string` | 是   | 订单 ID 或者用户自定义 ID 。如果使用自定义 ID，只能在订单创建后的 30 分钟内有效            |
| `account`       | `string` | 否   | 撤销的订单如果是全仓杠杆账户订单或是统一账户 apikey，该字段必须指定且设置为 `cross_margin` |

`req_header` 自定义 header 数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝               |

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
    "channel": "spot.order_cancel_ids",
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
        "currency_pair": "GT_USDT",
        "id": "1700664343",
        "succeeded": true
      }
    ]
  }
}
```

</Example>

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                                                         |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                                             |
| `header`         | Map    | 响应的元信息                                                                                                                                                                                 |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                                                 |
| »`channel`       | String | 请求的频道                                                                                                                                                                                   |
| »`event`         | String | 请求事件                                                                                                                                                                                     |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                                            |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 响应详见[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-batch-of-orders-with-an-id-list)                                                                                        |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                                                       |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                                                     |
| »»`message`      | String | 错误信息详情                                                                                                                                                                                 |

</Block>

<Block>

## 使用指定的货币对取消所有订单

您可以使用此频道和事件`cancel_cp`取消所有未成交的订单

**以下是 API 的功能:**

```json
DELETE /spot/orders
```

</Block>

<Block>

### 使用指定的货币对取消所有订单请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


time = int(time.time())
cancelParam = {"side":"buy","currency_pair":"BTC_USD","trade_quote":"USDC"}
channel = "spot.order_cancel_cp"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"test_1",
        "req_param": cancelParam
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	cancelParam := map[string]interface{}{
		"side":          "buy",
		"currency_pair": "BTC_USD",
		"trade_quote":   "USDC",
	}
	channel := "spot.order_cancel_cp"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": cancelParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

```json
{
  "time": 1681986207,
  "channel": "spot.order_cancel_cp",
  "event": "api",
  "payload": {
    "req_id": "request-7",
    "req_param": {
      "currency_pair": "BTC_USD",
      "side": "buy",
      "trade_quote": "USDC"
    }
  }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                                            |
| ----------- | -------- | ---- | --------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。                             |
| `req_param` | `object` | 是   | detail to[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-in-specified-currency-pair) |
| `req_header` | `object` | 否   | apiv4 自定义 header   |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                       |
| --------------- | -------- | ---- | ------------------------------------------------------------------------------------------ |
| `currency_pair` | `string` | 是   | 交易货币对                                                                                 |
| `side`          | `string` | 是   | 指定全部买单或全部卖单，不指定则两者都包括                                                 |
| `account`       | `string` | 否   | 指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 cross_margin 则查询全仓杠杆账户。 |
| `trade_quote`   | `string` | 否   | 仅撤销实际计价资产为该值的订单                                                               |

`trade_quote` 在 WebSocket 层按原值透传，由下游交易服务校验。它非空时是实际撤单过滤条件；如果下游不支持该过滤能力，请求将失败，不会退化为撤销名义市场下的全部订单。撤单结果保持原有结构，不回显 `trade_quote`。

`req_header` 自定义 header 数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝               |

</Block>

<Block>

### 使用指定的货币对取消所有订单推送

<Example>

取消订单推送示例

```json
{
  "request_id": "request-7",
  "header": {
    "response_time": "1681986207412",
    "status": "200",
    "channel": "spot.order_cancel_cp",
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
        "id": "1700664337",
        "text": "t-my-custom-id",
        "amend_text": "-",
        "create_time": "1681986206",
        "update_time": "1681986207",
        "create_time_ms": 1681986206384,
        "update_time_ms": 1681986207444,
        "status": "cancelled",
        "currency_pair": "GT_USDT",
        "type": "limit",
        "account": "spot",
        "side": "buy",
        "amount": "1",
        "price": "1",
        "time_in_force": "gtc",
        "iceberg": "0",
        "left": "1",
        "fill_price": "0",
        "filled_total": "0",
        "fee": "0",
        "fee_currency": "GT",
        "point_fee": "0",
        "gt_fee": "0",
        "gt_maker_fee": "0.0015",
        "gt_taker_fee": "0.0015",
        "gt_discount": true,
        "rebated_fee": "0",
        "rebated_fee_currency": "USDT",
        "stp_id": 1,
        "stp_act": "cn",
        "finish_as": "cancelled"
      }
    ]
  }
}
```

</Example>

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                                                         |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                                                             |
| `header`         | Map    | 响应的元信息                                                                                                                                                                                 |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                                                 |
| »`channel`       | String | 请求的频道                                                                                                                                                                                   |
| »`event`         | String | 请求事件                                                                                                                                                                                     |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                                                            |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 响应详情见[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-in-specified-currency-pair)                                                                             |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                                                       |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                                                     |
| »»`message`      | String | 错误信息详情                                                                                                                                                                                 |

</Block>

<Block>

## 修改订单

您可以使用此频道和事件`amend`来修改一个未成交的订单。

**以下是 API 的功能:**

```
PATCH /spot/orders/{order_id}
```

</Block>

<Block>

### 修改订单需求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


time = int(time.time())
amendParam = {"order_id":"1694883366","currency_pair":"GT_USDT","price":"2"}
channel = "spot.order_amend"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"test_1",
        "req_param": amendParam
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	amendParam := map[string]interface{}{
		"order_id":     "1694883366",
		"currency_pair": "GT_USDT",
		"price":        "2",
	}
	channel := "spot.order_amend"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": amendParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

客户端请求示例

```json
{
  "time": 1681986206,
  "channel": "spot.order_amend",
  "event": "api",
  "payload": {
    "req_id": "request-4",
    "req_param": {
      "order_id": "1700664330",
      "currency_pair": "GT_USDT",
      "price": "2"
    }
  }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                            |
| ----------- | -------- | ---- | ----------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。             |
| `req_param` | `object` | 是   | API 修改订单，详细信息请参考[api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order) |
| `req_header` | `object` | 否   | apiv4 自定义 header   |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                       |
| --------------- | -------- | ---- | ------------------------------------------------------------------------------------------ |
| `amount`        | `string` | 否   | 交易数量，`amount`和`price`必须指定其中一个                                                |
| `price`         | `string` | 否   | 交易价，`amount`和`price`必须指定其中一个                                                  |
| `amend_text`    | `string` | 否   | 用户可以备注这次修改的信息。                                                               |
| `order_id`      | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即 `text` 字段）。              |
| `currency_pair` | `string` | 是   | 交易对                                                                                     |
| `account`       | `string` | 否   | 指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 cross_margin 则查询全仓杠杆账户。 |

`req_header` 自定义 header 数据:

| 字段            | 类型     | 必选 | 描述                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | 否   | 指定过期的时间戳（毫秒）。如果 ws 收到请求的时间大于过期时间，请求将被拒绝               |

</Block>

<Block>

### 修改订单推送

<Example>

修改订单 推送示例

```json
{
  "request_id": "request-4",
  "header": {
    "response_time": "1681986206145",
    "status": "200",
    "channel": "spot.order_amend",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "x_gate_ratelimit_requests_remain": 9,
    "x_gate_ratelimit_limit": 10,
    "x_gat_ratelimit_reset_timestamp": 1736408263764
  },
  "data": {
    "result": {
      "id": "1700664330",
      "text": "t-my-custom-id",
      "amend_text": "-",
      "create_time": "1681986204",
      "update_time": "1681986206",
      "create_time_ms": 1681986204832,
      "update_time_ms": 1681986206176,
      "status": "open",
      "currency_pair": "GT_USDT",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "1",
      "price": "2",
      "time_in_force": "gtc",
      "iceberg": "0",
      "left": "1",
      "fill_price": "0",
      "filled_total": "0",
      "fee": "0",
      "fee_currency": "GT",
      "point_fee": "0",
      "gt_fee": "0",
      "gt_maker_fee": "0.0015",
      "gt_taker_fee": "0.0015",
      "gt_discount": true,
      "rebated_fee": "0",
      "rebated_fee_currency": "USDT",
      "stp_id": 1,
      "stp_act": "cn",
      "finish_as": "open"
    }
  }
}
```

</Example>

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                           |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                               |
| `header`         | Map    | 响应的元信息                                                                                                                                                   |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                                   |
| »`channel`       | String | 请求的频道                                                                                                                                                     |
| »`event`         | String | 请求事件                                                                                                                                                       |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                              |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| »`x_gate_ratelimit_requests_remain`      | Integer | 当前时间窗口剩余可用请求数(为0不展示)                       |
| »`x_gate_ratelimit_limit`      | Integer | 当前频率限制上限(为0不展示)                     |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | 已超过当前窗口频率限制，表示下个可用时间窗口的时间戳（毫秒），即什么时候可以恢复访问；未超过当前窗口频率限制，表示返回的是当前服务器时间（毫秒）          |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 响应详见[api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order)                                                                                    |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                         |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                                       |
| »»`message`      | String | 错误信息详情                                                                                                                                                   |

</Block>

<Block>

## 订单状态

您可以使用此频道和事件`status`来查询一个订单。

**以下是 API 的功能:**

```
GET /spot/orders/{order_id}
```

</Block>

<Block>

### 订单状态请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


time = int(time.time())
statusParam = {"order_id":"1694883366","currency_pair":"GT_USDT"}
channel = "spot.order_status"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"test_1",
        "req_param": statusParam
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	statusParam := map[string]interface{}{
		"order_id":     "1694883366",
		"currency_pair": "GT_USDT",
	}
	channel := "spot.order_status"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "test_1",
			"req_param": statusParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

客户端请求示例

```json
{
  "time": 1681986205,
  "channel": "spot.order_status",
  "event": "api",
  "payload": {
    "req_id": "request-3",
    "req_param": {
      "order_id": "1700664330",
      "currency_pair": "GT_USDT"
    }
  }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                |
| ----------- | -------- | ---- | ----------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。 |
| `req_param` | `object` | 是   | 详见[api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order)         |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                       |
| --------------- | -------- | ---- | ------------------------------------------------------------------------------------------ |
| `order_id`      | `string` | 是   | 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即 `text` 字段）。              |
| `currency_pair` | `string` | 否   | 交易对                                                                                     |
| `account`       | `string` | 否   | 指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 cross_margin 则查询全仓杠杆账户。 |

#### 详细描述

_order_id_: 成功创建订单时返回的订单 ID 或者用户创建时指定的自定义 ID（即 text 字段）。 基于自定义 ID 的操作只在挂单中可查，当结束订单（成交/取消）后，在结束之后 1 小时内可查，过期之后只能使用订单 ID

_account_: 指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 cross_margin 则查询全仓杠杆账户。 统一账户只能指定 cross_margin

</Block>

<Block>

### 订单状态推送

<Example>

订单状态 推送示例

```json
{
  "request_id": "request-3",
  "header": {
    "response_time": "1681986205829",
    "status": "200",
    "channel": "spot.order_status",
    "event": "api",
    "client_id": "::1-0x140001623c0",
    "x_in_time": 1681985856667508,
    "x_out_time": 1681985856667598,
    "conn_id": "5e74253e9c793974",
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d"
  },
  "data": {
    "result": {
      "id": "1700664330",
      "text": "t-my-custom-id",
      "amend_text": "-",
      "create_time": "1681986204",
      "update_time": "1681986204",
      "create_time_ms": 1681986204939,
      "update_time_ms": 1681986204939,
      "status": "open",
      "currency_pair": "GT_USDT",
      "type": "limit",
      "account": "spot",
      "side": "buy",
      "amount": "1",
      "price": "1",
      "time_in_force": "gtc",
      "iceberg": "0",
      "left": "1",
      "fill_price": "0",
      "filled_total": "0",
      "fee": "0",
      "fee_currency": "GT",
      "point_fee": "0",
      "gt_fee": "0",
      "gt_maker_fee": "0.0015",
      "gt_taker_fee": "0.0015",
      "gt_discount": true,
      "rebated_fee": "0",
      "rebated_fee_currency": "USDT",
      "stp_id": 1,
      "stp_act": "cn",
      "finish_as": "open"
    }
  }
}
```

</Example>

已更新的订单列表。请注意，可能会在一条通知中更新多个货币对的订单。

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                  |
| ---------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                      |
| `header`         | Map    | 响应的元信息                                                                                                                                          |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                          |
| »`channel`       | String | 请求的频道                                                                                                                                            |
| »`event`         | String | 请求事件                                                                                                                                              |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                     |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Object | 响应详见[api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order)                                                                       |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                              |
| »»`message`      | String | 错误信息详情                                                                                                                                          |

</Block>

<Block>

## 查询订单列表

`spot.order_list`

您可以使用此频道和事件`api`来查询订单。

**以下是 API 的功能:**

```
GET /spot/orders
```

</Block>

<Block>

### 查询订单请求

<Example>

代码示例：请求前要先登录

```python
#!/usr/bin/python

import time
import json
# pip install websocket_client
from websocket import create_connection


time = int(time.time())
statusParam = {"currency_pair": "BTC_USDT","status": "finished","limit": 3,"page": 1}
channel = "spot.order_list"

ws = create_connection("wss://api.gateio.ws/ws/v4/")

# refer to the Authentication section for a WebSocket API code example

ws.send(json.dumps({
    "time":time,
    "channel":channel,
    "event":"api",
    "payload":{
        "req_id":"1734081140-1",
        "req_param": statusParam
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
	url := "wss://api.gateio.ws/ws/v4/"
	conn, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	timestamp := time.Now().Unix()
	statusParam := map[string]interface{}{
		"currency_pair": "BTC_USDT",
		"status":        "finished",
		"limit":         3,
		"page":          1,
	}
	channel := "spot.order_list"

	request := map[string]interface{}{
		"time":    timestamp,
		"channel": channel,
		"event":   "api",
		"payload": map[string]interface{}{
			"req_id":    "1734081140-1",
			"req_param": statusParam,
		},
	}

	// refer to the Authentication section for a WebSocket API code example

	msg, err := json.Marshal(request)
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

客户端请求示例

```json
{
    "time": 1734400368,
    "channel": "spot.order_list",
    "event": "api",
    "payload": {
        "req_id": "1734081140-1",
        "req_param": {
            "currency_pair": "BTC_USDT",
            "status": "finished",
            "limit": 3,
            "page": 1
        }
    }
}
```

</Example>

Payload 格式:

| 字段        | 类型     | 必选 | 描述                                                                                |
| ----------- | -------- | ---- | ----------------------------------------------------------------------------------- |
| `req_id`    | `string` | 是   | 服务器将发送回的请求 ID，用于帮助您识别服务器响应的是哪个请求，它与外部的 id 不同。 |
| `req_param` | `object` | 是   | 详见[api](https://www.gate.com/docs/developers/apiv4/zh_CN/#%E6%9F%A5%E8%AF%A2%E8%AE%A2%E5%8D%95%E5%88%97%E8%A1%A8)         |

`req_param` API 订单模型的 JSON 字节数据:

| 字段            | 类型     | 必选 | 描述                                                                                       |
| --------------- | -------- | ---- | ------------------------------------------------------------------------------------------ |
|`currency_pair`|`string`|是|指定交易对查询。如果查询挂单的记录，该字段必选。如果查询已成交的记录，该字段可以不指定。|
|`status`|`string`|是|基于状态查询订单列表|
|`page`|`integer(int32)`|否|列表页数|
|`limit`|`integer`|否|列表返回的最大数量，如果 status 设置为 `open` ，`limit` 最大允许 100|
|`account`|`string`|否|指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 `cross_margin` 则查询全仓杠杆账户。  |
|`from`|`integer(int64)`|否|查询记录的起始时间|
|`to`|`integer(int64)`|否|查询记录的结束时间，不指定则默认为当前时间|
|`side`|`string`|否|指定全部买单或全部卖单，不指定则两者都包括|

#### 详细描述

**status**: 基于状态查询订单列表

`open` - 挂单中

`finished` - 已结束

**account**: 指定查询账户。不指定默认现货，保证金和逐仓杠杆账户。指定 `cross_margin` 则查询全仓杠杆账户。  
统一账户只能指定 `cross_margin`

</Block>

<Block>

### 订单列表推送

<Example>

订单列表 推送示例

```json
{
    "header": {
        "response_time": "1734486677912",
        "status": "200",
        "channel": "spot.order_list",
        "event": "api",
        "client_id": "35.79.119.197-0xc00a13a1a0",
        "x_in_time": 1734486677912508,
        "x_out_time": 1734486677912598,
        "conn_id": "5e74253e9c793974",
        "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
        "trace_id": "e410abb5f74b4afc519e67920548838d"
    },
    "data": {
        "result": [
            {
                "id": "20874890569",
                "text": "web",
                "amend_text": "-",
                "create_time": "1734081653",
                "update_time": "1734081653",
                "create_time_ms": 1734081653247,
                "update_time_ms": 1734081653249,
                "status": "closed",
                "currency_pair": "BTC_USDT",
                "type": "market",
                "account": "spot",
                "side": "buy",
                "amount": "70.0",
                "price": "0.0",
                "time_in_force": "ioc",
                "iceberg": "0.0",
                "left": "0.143511",
                "filled_amount": "0.00299",
                "fill_price": "69.856489",
                "filled_total": "69.856489",
                "avg_deal_price": "23363.3742475",
                "fee": "0.0000028704",
                "fee_currency": "BTC",
                "point_fee": "0.0",
                "gt_fee": "0.0",
                "gt_maker_fee": "0.0",
                "gt_taker_fee": "0.0",
                "rebated_fee": "0.0",
                "rebated_fee_currency": "USDT",
                "finish_as": "filled"
            },
            {
                "id": "20884808760",
                "text": "web",
                "amend_text": "-",
                "create_time": "1734081534",
                "update_time": "1734081534",
                "create_time_ms": 1734081534822,
                "update_time_ms": 1734081534824,
                "status": "closed",
                "currency_pair": "BTC_USDT",
                "type": "market",
                "account": "spot",
                "side": "buy",
                "amount": "100.0",
                "price": "0.0",
                "time_in_force": "ioc",
                "iceberg": "0.0",
                "left": "0.088092",
                "filled_amount": "0.00433",
                "fill_price": "99.911908",
                "filled_total": "99.911908",
                "avg_deal_price": "23074.34364897",
                "fee": "0.0000041568",
                "fee_currency": "BTC",
                "point_fee": "0.0",
                "gt_fee": "0.0",
                "gt_maker_fee": "0.0",
                "gt_taker_fee": "0.0",
                "rebated_fee": "0.0",
                "rebated_fee_currency": "USDT",
                "finish_as": "filled"
            },
            {
                "id": "20870148234",
                "text": "t-123456",
                "amend_text": "-",
                "create_time": "1733900250",
                "update_time": "1733900250",
                "create_time_ms": 1733900250750,
                "update_time_ms": 1733900250755,
                "status": "closed",
                "currency_pair": "BTC_USDT",
                "type": "market",
                "account": "spot",
                "side": "buy",
                "amount": "10.0",
                "price": "0.0",
                "time_in_force": "ioc",
                "iceberg": "0.0",
                "left": "0.144492",
                "filled_amount": "0.00013",
                "fill_price": "9.855508",
                "filled_total": "9.855508",
                "avg_deal_price": "75811.6",
                "fee": "0.0000001248",
                "fee_currency": "BTC",
                "point_fee": "0.0",
                "gt_fee": "0.0",
                "gt_maker_fee": "0.0",
                "gt_taker_fee": "0.0",
                "rebated_fee": "0.0",
                "rebated_fee_currency": "USDT",
                "finish_as": "filled"
            }
        ]
    },
    "request_id": "1734081140-1"
}
```

</Example>

推送格式:

| 字段             | 类型   | 描述                                                                                                                                                  |
| ---------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | 消息的唯一标识符                                                                                                                                      |
| `header`         | Map    | 响应的元信息                                                                                                                                          |
| »`response_time` | String | 响应发送时间（以毫秒为单位）                                                                                                                          |
| »`channel`       | String | 请求的频道                                                                                                                                            |
| »`event`         | String | 请求事件                                                                                                                                              |
| »`client_id`     | String | 唯一客户端标识 ID                                                                                                                                     |
| »`x_in_time`     | Integer| ws 接收请求的时间（以微秒为单位）                                                                                                                           |
| »`x_out_time`    | Integer| ws 返回响应的时间（以微秒为单位）                                                                                                                           |
| »`conn_id`       | String | 与客户端建立连接的链接Id（同一个连接的链接Id保持一致）                                     |
| »`conn_trace_id` | String | 与客户端建立连接的TraceId                                                                                                                                 |
| »`trace_id`      | String | 执行下单操作的TraceId                                                                                                                                     |
| `data`           | Object | 请求响应的数据                                                      |
| »`result`        | Array  | 响应详见[api](https://www.gate.com/docs/developers/apiv4/zh_CN/#order)                                                                       |
| »`errs`          | Object | 只有在请求失败时才可用                                                                                                                                |
| »»`label`        | String | 以字符串格式表示错误类型                                                                                                                              |
| »»`message`      | String | 错误信息详情                                                                                                                                          |

</Block>
