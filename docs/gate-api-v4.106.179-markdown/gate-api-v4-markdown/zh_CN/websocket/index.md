---
title: WebSocketAPI Reference

language_tabs:
  - python: Python
  - javascript: JavaScript

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# 摘要

</Block>

<Block>

## 前言

Gate 提供了一个简单而强大的 Websocket API，可以将 Gate 的交易状态集成到您的业务或应用程序中。

我们提供了 JavaScript 和 Python 的语言绑定，未来还会有更多！您可以在右侧的暗色区域查看代码示例，并可以使用右上角的选项卡切换示例的编程语言。

</Block>

<Block>

## 当前版本

Websocket API 版本是 v3.3.9。

</Block>

<Block>

## 服务连接地址

我们提供两个备用服务器 URL，您可以根据您的情况选择其中一个。

::: tip
`wss://ws.gate.io/v3/`
:::

::: tip
`wss://ws.gateio.io/v3/`
:::

在代码示例中，我们使用 `ws.gate.io/v3` 进行展示。

</Block>

<Block>

## APIv4 密钥支持

现货 Websocket 支持 APIv4 密钥。要使用 APIv4 密钥进行现货 Websocket，请执行以下操作：

- 将服务器 URL 版本从 `/v3` 更改为 `/v4`
- 使用 APIv4 密钥生成签名
- 更改签名生成方法为 APIv4 的，即 `HexEncode(HMAC_SHA512(secret, signature_string))`,并确保 nonce 是当前的 Unix 时间（毫秒）

例如，如果当前的 Unix 时间（毫秒）为 `1583131539528`, 您的 APIv4 密钥为`key`,密钥为
`secret`, 则应该生成签名（在 Python 代码中）`hmac.new('secret', '1583131539528', hashlib.sha512).hexdigest()`, 结果为
`54613ee7f4236bf61bfaed71f10bc0cb8b24805c45822652f850812c9a43b2422cf84609197ccf5db7adaf5c6af5d143cf04646c2640ad89a7c89670b403b671`
. 将请求正文发送到 server.sign，如：

```json
{
  "id": 12312,
  "method": "server.sign",
  "params": [
    "key",
    "54613ee7f4236bf61bfaed71f10bc0cb8b24805c45822652f850812c9a43b2422cf84609197ccf5db7adaf5c6af5d143cf04646c2640ad89a7c89670b403b671",
    1583131539528
  ]
}
```

</Block>

<Block>

## API 概述

### Method

每个通用 API（例如 ticker、depth、kline 等）都支持 4 种不同的方法消息，它们是：

1. **`query`**

   主动查询 Gate 交易状态的信息。例如：ticker.query

2. **`subscribe`** (**推荐**)

   订阅以接收来自服务器的新数据通知。例如：ticker.subscribe

3. **`unsubscribe`**

   如果取消订阅，则服务器将不会发送新数据通知。例如：ticker.unsubscribe

4. **`update`** (**客户端不能主动发送**)

   如果有新的订阅数据可用，服务器将向客户端发送通知。例如：ticker.update

::: warning
用户绝不能向服务器发送 update 方法消息，因为 update 消息始终由服务器发送。
:::

### 请求参数

每个请求都遵循一个包含 id、method 和 params 的公共格式。

服务器没有对请求 ID 进行严格限制，但我们强烈建议您在一个 WebSocket 连接中为不同的 API 使用不同的 ID。

| 参数     | 类型    | 必填 | 描述                   |
| -------- | ------- | ---- | ---------------------- |
| `id`     | Integer | Yes  | the 请求参数 ID        |
| `method` | String  | Yes  | the method of 请求参数 |
| `params` | Array   | Yes  | detail 参数 s          |

### 响应

与请求类似，响应遵循由 id、error 和 result 组成的公共格式。

| 字段     | 类型        | 描述                                                             |
| -------- | ----------- | ---------------------------------------------------------------- |
| `id`     | Integer     | corresponding to 请求参数 ID                                     |
| `error`  | JSON object | null for success<br>object with `code` and `message` for failure |
| `result` | JSON object | result object, null for failure                                  |

### 通知

已订阅的通道会发送通知消息。

| 参数     | 类型   | 必填 | 描述                                   |
| -------- | ------ | ---- | -------------------------------------- |
| `id`     | null   | Yes  | the 请求参数 id, null for notification |
| `method` | String | Yes  | method                                 |
| `params` | Array  | Yes  | detail 参数 s                          |

### Error

如果出现错误，您将收到一个包含适当错误代码和错误消息的错误对象。

| Code | Message               |
| ---- | --------------------- |
| `1`  | `invalid argument`    |
| `2`  | `internal error`      |
| `3`  | `service unavailable` |
| `4`  | `method not found`    |
| `5`  | `service timeout`     |

### Authentication

用户可以在没有特定身份验证的情况下连接公共通道。

对于私人通道，gate 提供基于签名的身份验证方法。
有关详细信息，请参见 [Auth API](#authapi) .

### 限速

::: warning
用户每个连接的通道每秒最多只能执行 50 个请求。
:::

</Block>

<Block>

# System 频道

**提供系统状态检查，例如 ping-pong 和服务器时间查询.**

</Block>

<Block>

## Ping

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "server.ping");
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"server.ping", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": "pong",
  "id": 1000
}
```

</Example>

**检查服务器连接性.**

### 请求参数

- method

  `server.ping`

### 响应

- result

  | 字段   | 类型   | description       |
  | ------ | ------ | ----------------- |
  | `pong` | String | pong, ack of ping |

</Block>

<Block>

## Time

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "server.time");
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"server.time", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": 1523348055,
  "id": 12312
}
```

</Example>

**获取服务器时间.**

### 请求参数

- method

  `server.time`

### 响应

- result

  | 字段        | 类型    | 描述      |
  | ----------- | ------- | --------- |
  | `timestamp` | Integer | timestamp |

</Block>

<Block>

# Ticker 频道

**行情是市场状态的高级概述。它显示最高、最低和最新交易价格。它还包括每日成交量以及价格在过去一天内的波动情况等信息。**

</Block>

<Block>

## Ticker 查询

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "ticker.query", ["EOS_USDT", 86400]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"ticker.query", "params":["EOS_USDT", 86400]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "period": 86400,
    "open": "5.9606",
    "close": "5.9606",
    "high": "5.9606",
    "low": "5.9606",
    "last": "5.9606",
    "change": "0",
    "quoteVolume": "4",
    "baseVolume": "23.8424"
  },
  "id": 12312
}
```

</Example>

**查询指定市场的行情，包括某一时期内的价格、成交量等信息.**

### 请求参数

- method

  `ticker.query`

- params

  | 字段     | 类型    | 必填 | 描述                                                       |
  | -------- | ------- | ---- | ---------------------------------------------------------- |
  | `market` | String  | Yes  | market name                                                |
  | `period` | Integer | Yes  | ticker period, unit is second<br/>e.g. 86400 equals to 24h |

### 响应

- result

  | 字段          | 类型    | 描述        |
  | ------------- | ------- | ----------- |
  | `period`      | Integer | period      |
  | `open`        | String  | open        |
  | `close`       | String  | close       |
  | `high`        | String  | high        |
  | `low`         | String  | low         |
  | `last`        | String  | last        |
  | `change`      | String  | change      |
  | `quoteVolume` | String  | quoteVolume |
  | `baseVolume`  | String  | baseVolume  |

## Ticker 订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "ticker.subscribe", ["BOT_USDT"]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"ticker.subscribe", "params":["BOT_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**订阅市场 ticker 信息.**

### 请求参数

- method

  `ticker.subscribe`

- params

  | 参数          | 类型   | 必填 | 描述        |
  | ------------- | ------ | ---- | ----------- |
  | `market list` | String | Yes  | market list |

</Block>

<Block>

## Ticker 通知

<Example>

```json
{
  "method": "ticker.update",
  "params": [
    "BOT_USDT",
    {
      "period": 86400,
      "open": "0",
      "close": "0",
      "high": "0",
      "low": "0",
      "last": "0.2844",
      "change": "0",
      "quoteVolume": "0",
      "baseVolume": "0"
    }
  ],
  "id": null
}
```

</Example>

**通知已订阅的市场行情.**

### 通知

- method

  `ticker.update`

- params

  | 参数          | 类型        | 必填 | 描述                                 |
  | ------------- | ----------- | ---- | ------------------------------------ |
  | `market`      | String      | Yes  | market name                          |
  | `ticker info` | JSON object | Yes  | ticker info, refer to query response |

</Block>

<Block>

## 取消订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "ticker.unsubscribe", []);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"ticker.unsubscribe", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**取消 ticker 订阅.**

### 请求参数

- method

  `ticker.unsubscribe`

</Block>

<Block>

# 公共成交频道

**每当在 Gate 上进行交易时，此通道会发送交易消息。它包括交易的详细信息，例如价格、数量、时间和类型.**

</Block>

<Block>

## 公共成交 查询

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "trade.query", ["EOS_USDT", 2, 7177813]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12309, "method":"trades.query", "params":["EOS_USDT", 2, 7177813]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": [
    {
      "id": 7177814,
      "time": 1523887673.562782,
      "price": "6.05",
      "amount": "20",
      "type": "buy"
    },
    {
      "id": 7177813,
      "time": 1523887354.256974,
      "price": "6.05",
      "amount": "15",
      "type": "buy"
    }
  ],
  "id": 12309
}
```

</Example>

**查询最新的交易信息，包括时间、价格、数量、类型等.**

### 请求参数

- method

  `trades.query`

- params

  | 参数      | 类型    | 必填 | 描述         |
  | --------- | ------- | ---- | ------------ |
  | `market`  | String  | Yes  | market name  |
  | `limit`   | Integer | Yes  | amount limit |
  | `last_id` | Integer | Yes  | last id      |

### 响应

- result

  | 字段     | 类型    | 描述   |
  | -------- | ------- | ------ |
  | `id`     | Integer | id     |
  | `time`   | Float   | time   |
  | `price`  | String  | price  |
  | `amount` | String  | amount |
  | `type`   | String  | buy    |

</Block>

<Block>

## 公共成交 订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "trades.subscribe", ["ETH_USDT", "BTC_USDT"]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"trades.subscribe", "params":["ETH_USDT", "BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**订阅交易更新通知.**

### 请求参数

- method

  `trades.subscribe`

- params

  | 参数          | 类型 | 必填 | 描述        |
  | ------------- | ---- | ---- | ----------- |
  | `market list` | List | Yes  | market list |

</Block>

<Block>

## 公共成交 通知

<Example>

```json
{
  "method": "trades.update",
  "params": [
    "ETH_USDT",
    [
      {
        "id": 7172173,
        "time": 1523339279.761838,
        "price": "398.59",
        "amount": "0.027",
        "type": "buy"
      }
    ]
  ],
  "id": null
}
```

</Example>

**通知最新交易更新.**

### 通知

- method

  `trades.update`

- params

  | 参数          | 类型   | 必填 | 描述                                               |
  | ------------- | ------ | ---- | -------------------------------------------------- |
  | `market`      | String | Yes  | market name                                        |
  | `trades list` | List   | Yes  | list of trade info object, refer to query response |

</Block>

<Block>

## 取消订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "trades.unsubscribe", []);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"trades.unsubscribe", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**退订交易更新通知.**

### 请求参数

- method

  `trades.unsubscribe`

</Block>

<Block>

# 深度频道

**深度通道允许您跟踪 Gate 订单簿深度的状态。它以价格聚合为基础，具有可自定义的精度.**

</Block>

<Block>

## 深度查询

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "depth.query", ["EOS_USDT", 5, "0.0001"]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"depth.query", "params":["EOS_USDT", 5, "0.0001"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "asks": [
      ["15.72", "811.7089610788"],
      ["15.76", "18.45"]
    ],
    "bids": [
      ["15.71", "1544.941002"],
      ["15.7", "355.017"]
    ]
  },
  "id": 12312
}
```

</Example>

**查询指定市场深度.**

### 请求参数

- method

  `depth.query`

- params

  | 参数       | 类型    | 必填 | 描述                                |
  | ---------- | ------- | ---- | ----------------------------------- |
  | `market`   | String  | Yes  | market name                         |
  | `limit`    | Integer | Yes  | limit                               |
  | `interval` | String  | Yes  | unit interval, e.g. "0.0001", "0.1" |

### 响应

- result

  | 字段   | 类型 | 描述 |
  | ------ | ---- | ---- |
  | `asks` | List | asks |
  | `bids` | List | bids |

  - asks

    | 字段     | 类型   | 描述   |
    | -------- | ------ | ------ |
    | `price`  | String | price  |
    | `amount` | String | amount |

  - bids

    | 字段     | 类型   | 描述   |
    | -------- | ------ | ------ |
    | `price`  | String | price  |
    | `amount` | String | amount |

</Block>

<Block>

## 订阅深度

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

// single-market mode subscription
socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "depth.subscribe", ["ETH_USDT", 5, "0.0001"]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};

// ----------------------------------------------------------------------------------
// If you want to subscribe multi market depth, just replace socket.onopen as below.
// ----------------------------------------------------------------------------------
// multi-market mode subscription
socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "depth.subscribe", [
    ["BTC_USDT", 5, "0.01"],
    ["ETH_USDT", 5, "0"],
  ]);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
# single-market mode subscription
ws.send('{"id":12312, "method":"depth.subscribe", "params":["ETH_USDT", 5, "0.0001"]}')
print(ws.recv())

# ---------------------------------------------------------------------------------
# If you want to subscribe multi market depth, just replace ws.send as below.
# ---------------------------------------------------------------------------------
# multi-market mode subscription
ws.send(
    '{"id":12312, "method":"depth.subscribe", "params":[["BTC_USDT", 5, "0.01"], ["ETH_USDT", 5, "0"]]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示:

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**订阅深度.**

::: tip
支持单个市场和多个市场深度订阅。有关详细信息，请参见请求。但是对于在一个 WebSocket 连接中进行的多个订阅，只有最后一个订阅生效.
:::

### 请求参数

- method

  `depth.subscribe`

- params(Single-market mode)

  | 参数       | 类型    | 必填 | 描述                                                                                                     |
  | ---------- | ------- | ---- | -------------------------------------------------------------------------------------------------------- |
  | `market`   | String  | Yes  | market name                                                                                              |
  | `limit`    | Integer | Yes  | limit, legal limits: 1, 5, 10, 20, 30                                                                    |
  | `interval` | String  | Yes  | legal intervals: "0", "0.00000001", "0.0000001", "0.000001", "0.00001", "0.0001", "0.001", "0.01", "0.1" |

- params(Multi-market mode)

  A list of single-market mode params, e.g. `[["EOS_USDT", 2, "0"], ["BTC_USDT", 10, "0.01"]]`. See
  also examples at right.

</Block>

<Block>

## 深度通知

<Example>

```json
{
  "method": "depth.update",
  "params": [
    true,
    {
      "asks": [["8000.00", "9.6250"]],
      "bids": [["8000.00", "9.6250"]]
    },
    "EOS_USDT"
  ],
  "id": null
}
```

</Example>

**通知市场深度更新信息**

### 通知

- method

  `depth.update`

- params

  | 字段     | 类型        | 描述                                                      |
  | -------- | ----------- | --------------------------------------------------------- |
  | `clean`  | Boolean     | true: is complete result<br>false: is last updated result |
  | `depth`  | JSON object | depth json object, refer to query response                |
  | `market` | String      | market name                                               |

</Block>

<Block>

## 取消订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "depth.unsubscribe", []);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"depth.unsubscribe", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

The above command returns JSON structured like this:

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**Unsubscribe specified market depth.**

### 请求参数

- method

  `depth.unsubscribe`

</Block>

<Block>

# K 线频道

**提供一种访问图表蜡烛图信息的方式.**

</Block>

<Block>

## K 线查询

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "kline.query", ["BTC_USDT", 1, 1516951219, 1800]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"kline.query", "params":["BTC_USDT", 1, 1516951219, 1800]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

> 从左到右边的值依次为: time, open, close, highest, lowest, volume, amount, market

```json
{
  "error": null,
  "result": [
    [
      1492358400,
      "7000.00",
      "8000.0",
      "8100.00",
      "6800.00",
      "1000.00",
      "123456.00",
      "BTC_USDT"
    ]
  ],
  "id": 12312
}
```

</Example>

**查询指定市场行情信息**

### 请求参数

- method

  `kline.query`

- params

  | 参数       | 类型    | 必填 | 描述                    |
  | ---------- | ------- | ---- | ----------------------- |
  | `market`   | String  | Yes  | market name             |
  | `start`    | Integer | Yes  | start time, must be > 0 |
  | `end`      | Integer | Yes  | end time                |
  | `interval` | Integer | Yes  | interval                |

### 响应参数

- result

  一系列 k 线数据.每个 k 线格式如下:

  | 字段          | 类型    | 描述             |
  | ------------- | ------- | ---------------- |
  | `time`        | Integer | 开始时间戳       |
  | `open`        | String  | 开盘价           |
  | `close`       | String  | 收盘价           |
  | `highest`     | String  | 最高价           |
  | `lowest`      | String  | 最低价           |
  | `volume`      | String  | 成交量           |
  | `amount`      | String  | 交易金额         |
  | `market_name` | String  | 市场或交易对名称 |

</Block>

<Block>

## K 线订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "kline.subscribe", ["BTC_USDT", 1800]);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"kline.subscribe", "params":["BTC_USDT", 1800]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**Subscribe specified market kline information.**

::: warning
Can only subscribe to one market at the same time, market list is not supported currently. For multiple subscriptions, only the last one takes effect.
:::

### 请求参数

- method

  `kline.subscribe`

- params

  | 字段       | 类型    | 描述        |
  | ---------- | ------- | ----------- |
  | `market`   | String  | market name |
  | `interval` | Integer | interval    |

</Block>

<Block>

## K 线通知

<Example>

> value from left to right: time, open, close, highest, lowest, volume, amount, market

```json
{
  "method": "kline.update",
  "params": [
    [
      1492358400,
      "7000.00",
      "8000.0",
      "8100.00",
      "6800.00",
      "1000.00",
      "123456.00",
      "BTC_USDT",
      true
    ]
  ],
  "id": null
}
```

</Example>

**通知订阅市场的 K 线信息。**

### 通知

- method

  `kline.update`

- params

  | 字段           | 类型    | 描述                                                     |
  | -------------- | ------- | -------------------------------------------------------- |
  | `time`         | Integer | 开始时间戳                                               |
  | `open`         | String  | 开盘价                                                   |
  | `close`        | String  | 收盘价                                                   |
  | `highest`      | String  | 最高价                                                   |
  | `lowest`       | String  | 最低价                                                   |
  | `volume`       | String  | 交易量                                                   |
  | `amount`       | String  | 交易金额                                                 |
  | `market name`  | String  | 市场或交易对名称                                         |
  | `window_close` | Boolean | `true` 表示窗口关闭，可能会缺失 `true`，但不影响数据使用 |

</Block>

<Block>

## 取消订阅

<Example>

```javascript
function socket_send_cmd(socket, cmd, params) {
  if (!params) params = [];
  var msg = {
    id: client_id,
    method: cmd,
    params: params,
  };
  socket.send(JSON.stringify(msg));
}

var socket = new WebSocket("wss://ws.gate.io/v3");

socket.onopen = function () {
  console.log("Connected");
  socket_send_cmd(socket, "kline.unsubscribe", []);
};
socket.onmessage = function (e) {
  console.log("Server: " + e.data);
};
```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"kline.unsubscribe", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**退订 k 线通知.**

### 请求参数

- method

  `kline.unsubscribe`

</Block>

<Block>

# 私有频道

<a id="authapi"></a>

<Example>

```javascript

```

</Example>

<Example>

```python
import hmac
import base64
import hashlib


def get_sign(secret_key, message):
    h = hmac.new(secret_key, message, hashlib.sha512)
    return base64.b64encode(h.digest())
```

</Example>

Gate 在私人频道上提供基于签名的授权。算法可以简单地描述为下面的伪代码

    `base64(hmac_sha512(secret_key, nonce))`

您可以参考右侧的代码示例.

</Block>

<Block>

## 认证

<Example>

```python
import json
import time
from websocket import create_connection

api_key = 'your api key'
secret_key = 'your secret key'

ws = create_connection("wss://ws.gate.io/v3/")

nonce = int(time.time() * 1000)
signature = get_sign(secret_key, str(nonce))

ws.send(json.dumps({
    "id": 12312,
    "method": "server.sign",
    "params": [api_key, signature, nonce]
}))
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**认证签名.**

### 请求参数

- method

  `server.sign`

- params

| 参数        | 类型    | 必填 | 描述                                                              |
| ----------- | ------- | ---- | ----------------------------------------------------------------- |
| `apikey`    | String  | yes  | user apikey                                                       |
| `signature` | String  | yes  | user sign data                                                    |
| `nonce`     | Integer | yes  | timestamp, for milliseconds spent from Unix epoch to current time |

</Block>

<Block>

# 订单频道

::: warning
需要认证.
:::

</Block>

<Block>

## 订单查询

<Example>

```javascript

```

</Example>

<Example>

```python

from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"order.query", "params":["EOS_USDT", 0, 10]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "result": {
    "limit": 10,
    "offset": 0,
    "total": 1,
    "records": [
      {
        "id": 796563387,
        "market": "EOS_USDT",
        "user": 1336974,
        "ctime": 1527082744.51649,
        "mtime": 1527082744.51649,
        "price": "0.1",
        "amount": "100",
        "left": "100",
        "dealFee": "0",
        "orderType": 1,
        "type": 2,
        "filledAmount": "0",
        "filledTotal": "0"
      }
    ]
  },
  "error": null,
  "id": 12312
}
```

</Example>

**查询用户未执行的订单**

### 请求参数

- method

  `order.query`

- params

  | 参数     | 类型    | 必填 | 描述        |
  | -------- | ------- | ---- | ----------- |
  | `market` | String  | yes  | market name |
  | `offset` | Integer | yes  | offset      |
  | `limit`  | Integer | yes  | limit       |

### 响应

- result

  | 字段      | 类型    | 描述                |
  | --------- | ------- | ------------------- |
  | `limit`   | Integer | limit               |
  | `offset`  | Integer | offset              |
  | `total`   | Integer | total               |
  | `records` | Object  | order record object |

  - record

    | 字段           | 类型    | 描述                            |
    | -------------- | ------- | ------------------------------- |
    | `id`           | Integer | order id                        |
    | `market`       | String  | market                          |
    | `user`         | Integer | user id                         |
    | `ctime`        | Float   | create time                     |
    | `ftime`        | Float   | finish time                     |
    | `price`        | String  | price                           |
    | `amount`       | String  | amount                          |
    | `left`         | String  | left                            |
    | `dealFee`      | String  | deal fee                        |
    | `orderType`    | Integer | order type, 1: limit, 2: market |
    | `type`         | Integer | type, 1: sell, 2: buy           |
    | `filledAmount` | String  | filled amount                   |
    | `filledTotal`  | String  | filled total                    |

</Block>

<Block>

## 订单订阅

<Example>

```javascript

```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"order.subscribe", "params":["EOS_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**订阅用户订单更新**

### 请求参数

- method

  `order.subscribe`

- params

  | 参数          | 类型   | 必填 | 描述                               |
  | ------------- | ------ | ---- | ---------------------------------- |
  | `market list` | String | yes  | market list, null to subscribe all |

</Block>

<Block>

## 订单通知

<Example>

```json
{
  "method": "order.update",
  "params": [
    3,
    {
      "id": 34628963,
      "market": "EOS_USDT",
      "orderType": 1,
      "type": 2,
      "user": 602123,
      "ctime": 1523013969.6271579,
      "mtime": 1523013969.6271579,
      "price": "0.1",
      "amount": "1000",
      "left": "1000",
      "filledAmount": "0",
      "filledTotal": "0",
      "dealFee": "0"
    }
  ],
  "id": null
}
```

</Example>

**在下单、更新或完成订单时，通知用户订单信息.**

### 通知

- method

`order.update`

- params

  | 参数    | 类型    | 必填 | 描述                                             |
  | ------- | ------- | ---- | ------------------------------------------------ |
  | `event` | Integer | yes  | event type,Integer, 1: PUT, 2: UPDATE, 3: FINISH |
  | `order` | String  | yes  | order detail,Object                              |

  - order

    | 字段           | 类型    | 描述                            |
    | -------------- | ------- | ------------------------------- |
    | `id`           | Integer | order id                        |
    | `market`       | String  | market                          |
    | `user`         | Integer | user id                         |
    | `ctime`        | Float   | ctime                           |
    | `mtime`        | Float   | mtime                           |
    | `price`        | String  | price                           |
    | `amount`       | String  | amount                          |
    | `left`         | String  | left                            |
    | `dealFee`      | String  | deal fee                        |
    | `orderType`    | Integer | order type, 1: limit, 2: market |
    | `type`         | Integer | type, 1: sell, 2: buy           |
    | `filledAmount` | String  | filled amount                   |
    | `filledTotal`  | String  | filled total                    |

### 响应

</Block>

<Block>

## 取消订阅

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws_server_host/v3/websocket/")
ws.send('{"id":12312, "method":"order.unsubscribe", "params":["EOS_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**取消订阅用户订单更新通知，适用于所有市场.**

### 请求参数

- method

  `order.unsubscribe`

</Block>

<Block>

# 余额频道

::: warning
需要认证.
:::

</Block>

<Block>

## 余额查询

<Example>

```javascript

```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"balance.query", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "EOS": {
      "available": "13200.82187609",
      "freeze": "0"
    }
  },
  "id": 12312
}
```

</Example>

**获取指定资产或多个资产的用户余额信息.**

### 请求参数

- method

  `balance.query`

- params

  | 参数         | 类型   | 必填 | 描述                             |
  | ------------ | ------ | ---- | -------------------------------- |
  | `asset list` | String | yes  | asset list, null for inquire all |

### 响应

- result

  | 字段          | 类型   | 描述                       |
  | ------------- | ------ | -------------------------- |
  | `balance set` | Object | set of balance information |

</Block>

<Block>

## 余额订阅

<Example>

```javascript

```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"balance.subscribe", "params":["EOS"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**订阅用户余额更新.**

### 请求参数

- method

  `balance.subscribe`

- params

  | 参数         | 类型   | 必填 | 描述                              |
  | ------------ | ------ | ---- | --------------------------------- |
  | `asset list` | String | yes  | asset list, null to subscribe all |

</Block>

<Block>

## 余额通知

<Example>

```json
{
  "method": "balance.update",
  "params": [
    {
      "EOS": {
        "available": "96.765323611874",
        "freeze": "11"
      }
    }
  ],
  "id": null
}
```

</Example>

**通知用户余额更新.**

### 通知

- method

  `balance.update`

- params

  一系列余额信息更新列表.

</Block>

<Block>

## 取消订阅

<Example>

```javascript

```

</Example>

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws.gate.io/v3/")
ws.send('{"id":12312, "method":"balance.unsubscribe", "params":["EOS"]}')
print(ws.recv())
```

</Example>

<Example>

上述命令返回的 JSON 结构如下所示：

```json
{
  "error": null,
  "result": {
    "status": "success"
  },
  "id": 12312
}
```

</Example>

**取消余额更新.**

### 请求参数

- method

  `balance.unsubscribe`

</Block>
