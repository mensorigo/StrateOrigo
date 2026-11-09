---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate 统一账户 WebSocket v1.0.0

Gate 提供简单而强大的 Websocket API，将 Gate 统一账户交易状态集成到您的业务或应用程序中。

我们在`Python`中有语言绑定，将来还会有更多！您可以在右侧的深色区域中查看代码示例，并且可以通过右上角的选项卡切换示例的编程语言

## 服务地址

 `wss://ws.gate.com/v4/ws/unified`

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
                    self._request("unified.ping", auth_required=False)
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
    ws.subscribe("unified.assets", [], False)


if __name__ == "__main__":
    logging.basicConfig(format="%(asctime)s - %(message)s", level=logging.DEBUG)
    app = GateWebSocketApp("wss://ws.gate.com/v4/ws/unified",
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
  u := url.URL{Scheme: "wss", Host: "ws.gateio.ws", Path: "/v4/ws/unified"}
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
  pingMsg := NewMsg("unified.ping", "", t, []string{})
  err = pingMsg.send(c)
  if err != nil {
    panic(err)
  }

  select {}
}
```

</Example>

2025-06-12

- `unified.assets` 和 `unified.asset_detail` 通道补充字段说明

2025-02-10

- `unified.assets` 通道移除字段 `c`(`credit_available_margin`). 此字段在通道中还有返回但是已经不再维护，仅供兼容使用，请以后不要再用。

2024-10-23

- 初始版本，支持 `unified.assets`,`unified.asset_detail`  通道

</Block>

<Block>

## Websocket API 概述

### 事件

每个通用 订阅频道都支持一些不同的事件消息，它们是：

1. **`subscribe`** (**推荐使用**)

   订阅，接受服务器的新数据通知。

2. **`unsubscribe`**

   如果取消订阅，服务器将不会发送新数据通知。

3. **`update`**

   服务器将向客户端发送新的订阅数据（增量数据）。

如果有新订阅的数据（所有数据）可用，服务器将向客户端发送通知。

### 请求

每个请求都遵循通用格式，其中包含`time`、`channel`、`event`和`payload`。

| 名称        | 类型      | 必选  | 描述                                                |
| --------- | ------- | --- | ------------------------------------------------- |
| `id`      | Integer | 否   | 可选的请求 ID，将由服务器发回，以帮助您识别服务器响应哪个请求                  |
| `time`    | Integer | 是   | 请求时间                                              |
| `channel` | String  | 是   | 请求 subscribe/unsubscribe 频道                       |
| `auth`    | String  | 否   | 请求身份验证信息，请参阅身份验证部分了解详细信息                          |
| `event`   | String  | 是   | 请求 `event` (subscribe/unsubscribe/update/all/api) |
| `payload` | Array   | 是   | 请求详细参数                                            |

### 响应

与请求类似，响应遵循以下通用格式，其中包含： `time`, `channel`, `event`
, `error` 和 `result`.

| 名称        | 类型      | 必选  | 描述                  |
| --------- | ------- | --- | ------------------- |
| `time`    | Integer | 是   | 响应时间                |
| `time_ms` | Integer | 是   | 毫秒响应时间              |
| `channel` | String  | 是   | 响应频道                |
| `event`   | String  | 是   | 响应频道事件 (update/all) |
| `error`   | Object  | 是   | 响应错误                |
| `result`  | Array   | 是   | 响应详细参数              |

### 错误

如果出现错误，您会收到`error`字段，其中包含错误代码和错误的类型。

| Code | Message                   |
| ---- | ------------------------- |
| `1`  | `invalid argument struct` |
| `2`  | `invalid argument`        |
| `3`  | `service error`           |

</Block>

<Block>

## 鉴权

如果频道是私有的，则请求体需要携带认证信息，
例如`unified.assets`

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
message = 'channel=%s&event=%s&time=%s' % ('unified.assets', 'subscribe', int(time.time()))
print(hmac.new(secret, message, hashlib.sha512).hexdigest())  ## Generating signature
```

</Example>

您可以登录账户获取永续合约账户的 api_key 和 secret。

| 名称       | 类型     | 描述             |
| -------- | ------ | -------------- |
| `method` | String | 验证方式:`api_key` |
| `KEY`    | String | api Key 的值     |
| `SIGN`   | String | 签名结果           |

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

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "unified.ping",
}))
print(ws.recv())
```

</Example>

<Example>

`unified.ping`操作返回 JSON 结构如下：

```json
{
    "time": 1701830644,
    "time_ms": 1701830644326,
    "channel": "unified.pong",
    "event": "",
    "result": null
}
```

</Example>

### 请求参数

- 频道

  `unified.ping`

</Block>

<Block>

# 资产概览通道

**推送资产概览信息，默认保留 2 位小数**

::: warning
需要认证
:::

</Block>

<Block>

## 资产概览订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 1700625194,
    "channel": "unified.assets",
    "event": "subscribe",
    "payload": [],
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

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1716796362,
  "time_ms": 1716796362915,
  "channel": "unified.assets",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**请求参数**

- channel

  `unified.assets`

- event

  `subscribe`

- params

  此通道不需要传递参数

  | 名称  | 类型  | 必选  | 描述  |
  | --- | --- | --- | --- |
  |     |     |     |     |

</Block>

<Block>

## 资产概览推送

<Example>

```json
{
        "time": 1700625194,
        "channel": "unified.assets",
        "event": "update",
        "result": {
                "u": 9008,
                "t": 1700625194,
                "r": "18.56",
                "R": "20.10",
                "b": "-1005719.51",
                "e": "-617985.29",
                "l": "1293939.74",
                "T": "675222.27",
                "a": "-1432719.62"
        }
}
```

</Example>

**推送参数**

- channel

  `unified.assets`

- event

  `update`

- params

  `推送结果参数含义请参考http接口.`

  | 名称       | 类型      | 描述      |
  | -------- | ------- | ------- |
  | `result` | objects | objects |

  | 名称  | 类型      | 字段全称 (非推送字段)                    | description  |
  | --- | ------- | ------------------------------- | ------------ |
  | `u` | integer | `user_id`                       | 用户 ID        |
  | `t` | integer | `refresh_time`                  | 数据刷新时间       |
  | `r` | string  | `total_initial_margin_rate`     | 总初始保证金率 (单位: %)      |
  | `R` | string  | `total_maintenance_margin_rate` | 总维持保证金率 (单位: %)    |
  | `b` | string  | `total_margin_balance`          | 总保证金余额       |
  | `e` | string  | `unified_account_total_equity`  | 统一账户总权益      |
  | `l` | string  | `unified_account_total_liab`    | 统一账户总借贷      |
  | `T` | string  | `unified_account_total`         | 统一账户总资产      |
  | `a` | string  | `total_available_margin`        | 可用的保证金额度     |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 123456,
    "channel": "unified.assets",
    "event": "unsubscribe",
    "payload": [],
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

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1716796362,
  "time_ms": 1716796362915,
  "channel": "unified.assets",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅资产概览更新通知**

**请求参数**

- channel

  `unified.assets`

- event

  `unsubscribe`

</Block>

<Block>

# 资产详情通道

**推送 货币资产 信息，货币资产包含 『现货资产、余币宝，理财，借贷』，其中 USDT还包含 合约和期权**

::: warning
需要认证
:::

</Block>

<Block>

## 资产详情订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 1716796362,
    "channel": "unified.asset_detail",
    "event": "subscribe",
    "payload": ["BTC","ETH"],
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

```python
# 订阅所有币种
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 1716796362,
    "channel": "unified.asset_detail",
    "event": "subscribe",
    "payload": ["!all"],
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

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1716796362,
  "time_ms": 1716796362915,
  "channel": "unified.asset_detail",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**请求参数**

- channel

  `unified.asset_detail`

- event

  `subscribe`

- params

  | 名称           | 类型        | 必选  | 描述   |
  | ------------ | --------- | --- | ---- |
  | `currencies` | string 数组 | 是   | 资产货币。您可以指定单个货币如 `["BTC","ETH"]` 或使用 `["!all"]` 订阅所有货币。注意：您不能在同一个订阅中混合使用单个货币和 `!all`。 |

</Block>

<Block>

## 资产详情推送

<Example>

```json
{
    "time": 1716796362,
    "time_ms": 1716796362915,
    "channel": "unified.asset_detail",
    "event": "update",
    "result":
    {
        "u": 11027732,
        "t": 1716796364,
        "dts":
        {
            "BTC":
            {
                "a": "1086390.949548",
                "f": "0.000000",
                "e": "1086390.949548",
                "tl": "0.00",
                "b": "1086390.949548"
            },
            "USDT":
            {
                "a": "8724.23263378",
                "f": "0.00",
                "e": "8724.23263378",
                "tl": "0.00",
                "b": "8724.23263378",
                "cb": "8724.23263378",
                "mb": "8724.23263378",
                "im": "0.00",
                "imr": "9999.99",
                "mm": "0.00",
                "mmr": "9999.99",
                "am": "8724.23263378",
                "iam": "8724.23263378"
            }
        }
    }
}
```

</Example>

**推送参数**

- channel

  `unified.asset_detail`

- event

  `update`

- params

  | 名称       | 类型      | 描述      |
  | -------- | ------- | ------- |
  | `result` | objects | objects |

  | 名称    | 类型      | 字段全称 (非推送字段)   | 描述       |
  | ----- | ------- | -------------- | -------- |
  | `u`   | integer | `user_id`      | 用户 id    |
  | `t`   | integer | `refresh_time` | 数据刷新时间   |
  | `dts` | map     | `details`      | 资产详情 map |
  | `>a`  | string  | `available`    | 可用额度     |
  | `>f`  | string  | `freeze`       | 被锁定的额度   |
  | `>e`  | string  | `equity`       | 权益       |
  | `>tl` | string  | `total_liab`   | 总借款      |
  | `>b`  | string  | `balance`      | 币种余额     |
  | 以下字段：单币种保证金模式字段 且 仅当币种是 USDT 时才有推送 |
  | `>cb`   | string | `cross_balance`          | 全仓余额        |
  | `>mb`   | string | `margin_balance`         | 全仓保证金余额   |
  | `>im`   | string | `initial_margin`         | 全仓总起始保证金 |
  | `>mm`   | string | `maintenance_margin`     | 全仓总维持保证金  |
  | `>imr`  | string | `initial_margin_rate`    | 全仓总起始保证金率 (单位: %) |
  | `>mmr`  | string | `maintenance_margin_rate`| 全仓总维持保证金率 (单位: %) |
  | `>am`   | string | `available_margin`       | 总可用保证金余额   |
  | `>iam`  | string | `iso_available_margin`   | 逐仓开仓可用      |

</Block>

<Block>

## 取消订阅

<Example>

```python
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 1716796362,
    "channel": "unified.asset_detail",
    "event": "unsubscribe",
    "payload": ["BTC", "ETH"],
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

```python
# 取消订阅所有币种
import json
from websocket import create_connection

ws = create_connection("wss://ws.gate.com/v4/ws/unified")
req = {
    "time": 1716796362,
    "channel": "unified.asset_detail",
    "event": "unsubscribe",
    "payload": ["!all"],
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

上面的命令返回 JSON 结构如下：

```json
{
  "time": 1716796362,
  "time_ms": 1716796362689,
  "channel": "unified.asset_detail",
  "event": "unsubscribe",
  "result": {
    "status": "success"
  }
}
```

</Example>

**取消订阅参数**

- channel

  `unified.asset_detail`

- event

  `unsubscribe`

</Block>
