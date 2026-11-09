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

# General

</Block>

<Block>

## Introduction

Gate provides a simple and robust Websocket API to integrate Gate trade status into your
business or application.

We have language bindings in `JavaScript` and `Python`, more in future! You can view code examples
in the dark area to the right, and you can switch the programming language of the examples with the
tabs in the top right.

</Block>

<Block>

## Current Version

Websocket API version is v3.3.9.

</Block>

<Block>

## Server URL

We provide two alternative server urls, you can choose one of them according to your condition.

::: tip
`wss://ws.gate.com/v3/`
:::

::: tip
`wss://ws.gateio.com/v3/`
:::

In code examples, we use `ws.gate.com/v3` to present.

</Block>

<Block>

## APIv4 Keys support

Spot websocket now supports APIv4 Keys. To use spot websocket with APIv4 keys, do the following
change based on v3:

- Change server url version from `/v3` to `/v4`
- Use APIv4 keys to generate signature
- Change the signature generation method to APIv4's,
  i.e., `HexEncode(HMAC_SHA512(secret, signature_string))` and make sure `nonce` is current unix
  time in milliseconds.

For example, if current unix time in milliseconds is `1583131539528`, your APIv4 key is `key`,
secret is
`secret`, the signature should be generated as (in Python
code) `hmac.new('secret', '1583131539528', hashlib.sha512).hexdigest()`, and the result is
`54613ee7f4236bf61bfaed71f10bc0cb8b24805c45822652f850812c9a43b2422cf84609197ccf5db7adaf5c6af5d143cf04646c2640ad89a7c89670b403b671`
. Send request body to `server.sign` like:

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

## API Overview

### Method

Each general api (such as ticker, depth, kline, etc.) supports 4 different method messages, they
are:

1. **`query`**

   Active inquiry information of Gate trade status. e.g. `ticker.query`

1. **`subscribe`** (**RECOMMENDED TO USE**)

   Subscribe to receive notification from server when new data is available. e.g. `ticker.subscribe`

1. **`unsubscribe`**

   Server will not send new data notification if unsubscribed. e.g. `ticker.unsubscribe`

1. **`update`** (**CLIENT SHOULD NEVER SEND**)

   If new subscribed data is available, server will send a notification to client.
   e.g. `ticker.update`

::: warning
User should NEVER send an `update` method message to server, because an `update` message is ALWAYS sent by server.
:::

### Request

Each request follows a common format, which contains `id`, `method` and `params`.

Server doesn't have strict restriction for request ID, but we strongly recommend you use different
id for different api in one websocket connection.

| parameter | type    | required | description           |
| --------- | ------- | -------- | --------------------- |
| `id`      | Integer | Yes      | the request ID        |
| `method`  | String  | Yes      | the method of request |
| `params`  | Array   | Yes      | detail parameters     |

### Response

Similar with request, response follows a common format compose of `id`, `error` and `result`.

| field    | type        | description                                                      |
| -------- | ----------- | ---------------------------------------------------------------- |
| `id`     | Integer     | corresponding to request ID                                      |
| `error`  | JSON object | null for success<br>object with `code` and `message` for failure |
| `result` | JSON object | result object, null for failure                                  |

### Notification

A notification message is sent for subscribed channels.

| parameter | type   | required | description                           |
| --------- | ------ | -------- | ------------------------------------- |
| `id`      | null   | Yes      | the request id, null for notification |
| `method`  | String | Yes      | method                                |
| `params`  | Array  | Yes      | detail parameters                     |

### Error

In case of error, you receive a message containing the proper error code and message within an error
object.

| Code | Message               |
| ---- | --------------------- |
| `1`  | `invalid argument`    |
| `2`  | `internal error`      |
| `3`  | `service unavailable` |
| `4`  | `method not found`    |
| `5`  | `service timeout`     |

### Authentication

User can connect public channels without any particular authentication.

For private channels, Gate provides a signature based authentication method.
See [Auth API](#authapi) for detail.

### Limitation

::: warning
User can only execute maximum to 50 requests per second for each connected channel.
:::

</Block>

<Block>

# System API

**Provides system status check, such as ping-pong and server time query.**

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

var socket = new WebSocket("wss://ws.gate.com/v3");

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

ws = create_connection("wss://ws.gate.com/v3/")
ws.send('{"id":12312, "method":"server.ping", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

The above command returns JSON structured like this:

```json
{
  "error": null,
  "result": "pong",
  "id": 1000
}
```

</Example>

**Check server connectivity.**

### Request

- method

  `server.ping`

### Response

- result

  | field  | type   | description       |
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

var socket = new WebSocket("wss://ws.gate.com/v3");

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

ws = create_connection("wss://ws.gate.com/v3/")
ws.send('{"id":12312, "method":"server.time", "params":[]}')
print(ws.recv())
```

</Example>

<Example>

The above command returns JSON structured like this:

```json
{
  "error": null,
  "result": 1523348055,
  "id": 12312
}
```

</Example>

**Acquire server time.**

### Request

- method

  `server.time`

### Response

- result

  | field       | type    | description |
  | ----------- | ------- | ----------- |
  | `timestamp` | Integer | timestamp   |

</Block>

<Block>

# Ticker API

**The ticker is a high level overview of the state of the market. It shows you the highest, lowest,
last trade price. It also includes information such as daily volume and how much the price has moved
over the last day.**

</Block>

<Block>

## Ticker query

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

var socket = new WebSocket("wss://ws.gate.com/v3");

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

The above command returns JSON structured like this:

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

**Query ticker of specified market, including price, deal volume etc. in certain period.**

### Request

- method

  `ticker.query`

- params

  | field    | type    | required | description                                                |
  | -------- | ------- | -------- | ---------------------------------------------------------- |
  | `market` | String  | Yes      | market name                                                |
  | `period` | Integer | Yes      | ticker period, unit is second<br/>e.g. 86400 equals to 24h |

### Response

- result

  | field         | type    | description |
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

## Ticker subscription

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

**Subscribe market ticker.**

### Request

- method

  `ticker.subscribe`

- params

  | parameter     | type   | required | description |
  | ------------- | ------ | -------- | ----------- |
  | `market list` | String | Yes      | market list |

</Block>

<Block>

## Ticker notification

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

**Notify subscribed market ticker.**

### Notify

- method

  `ticker.update`

- params

  | parameter     | type        | required | description                          |
  | ------------- | ----------- | -------- | ------------------------------------ |
  | `market`      | String      | Yes      | market name                          |
  | `ticker info` | JSON object | Yes      | ticker info, refer to query response |

</Block>

<Block>

## Cancel subscription

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

**Unsubscribe market ticker.**

### Request

- method

  `ticker.unsubscribe`

</Block>

<Block>

# Trade API

**This channel sends a trade message whenever a trade occurs at Gate. It includes details of the
trade, such as price, amount, time and type.**

</Block>

<Block>

## Trades query

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

The above command returns JSON structured like this:

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

**Query latest trades information, including time, price, amount, type and so on.**

### Request

- method

  `trades.query`

- params

  | parameter | type    | required | description  |
  | --------- | ------- | -------- | ------------ |
  | `market`  | String  | Yes      | market name  |
  | `limit`   | Integer | Yes      | amount limit |
  | `last_id` | Integer | Yes      | last id      |

### Response

- result

  | field    | type    | description |
  | -------- | ------- | ----------- |
  | `id`     | Integer | id          |
  | `time`   | Float   | time        |
  | `price`  | String  | price       |
  | `amount` | String  | amount      |
  | `type`   | String  | buy         |

</Block>

<Block>

## Trades subscription

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

**Subscribe trades update notification.**

### Request

- method

  `trades.subscribe`

- params

  | parameter     | type | required | description |
  | ------------- | ---- | -------- | ----------- |
  | `market list` | List | Yes      | market list |

</Block>

<Block>

## Trades notification

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

**Notify latest trades update.**

### Notify

- method

  `trades.update`

- params

  | parameter     | type   | required | description                                        |
  | ------------- | ------ | -------- | -------------------------------------------------- |
  | `market`      | String | Yes      | market name                                        |
  | `trades list` | List   | Yes      | list of trade info object, refer to query response |

</Block>

<Block>

## Cancel subscription

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

**Unsubscribe trades update notification.**

### Request

- method

  `trades.unsubscribe`

</Block>

<Block>

# Depth API

**The depth channel allow you to keep track of the state of the Gate order book depth. It is
provided on a price aggregated basis, with customizable precision.**

</Block>

<Block>

## Query depth

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

The above command returns JSON structured like this:

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

**Query specified market depth.**

### Request

- method

  `depth.query`

- params

  | parameter  | type    | required | description                         |
  | ---------- | ------- | -------- | ----------------------------------- |
  | `market`   | String  | Yes      | market name                         |
  | `limit`    | Integer | Yes      | limit                               |
  | `interval` | String  | Yes      | unit interval, e.g. "0.0001", "0.1" |

### Response

- result

  | field  | type | description |
  | ------ | ---- | ----------- |
  | `asks` | List | asks        |
  | `bids` | List | bids        |

  - asks

    | field    | type   | description |
    | -------- | ------ | ----------- |
    | `price`  | String | price       |
    | `amount` | String | amount      |

  - bids

    | field    | type   | description |
    | -------- | ------ | ----------- |
    | `price`  | String | price       |
    | `amount` | String | amount      |

</Block>

<Block>

## Depth subscription

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

The above command(Both single-market and multi-market mode) returns JSON structured like this:

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

**Subscribe depth.**

::: tip
Both single-market and multi-market depth subscription are supported. See Request for detail. But for multiple subscriptions in one websocket connection, only the last one takes effect.
:::

### Request

- method

  `depth.subscribe`

- params(Single-market mode)

  | parameter  | type    | required | description                                                                                              |
  | ---------- | ------- | -------- | -------------------------------------------------------------------------------------------------------- |
  | `market`   | String  | Yes      | market name                                                                                              |
  | `limit`    | Integer | Yes      | limit, legal limits: 1, 5, 10, 20, 30                                                                    |
  | `interval` | String  | Yes      | legal intervals: "0", "0.00000001", "0.0000001", "0.000001", "0.00001", "0.0001", "0.001", "0.01", "0.1" |

- params(Multi-market mode)

  A list of single-market mode params, e.g. `[["EOS_USDT", 2, "0"], ["BTC_USDT", 10, "0.01"]]`. See
  also examples at right.

</Block>

<Block>

## Depth notification

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

**Notify market depth update information**

### Notify

- method

  `depth.update`

- params

  | field    | type        | description                                               |
  | -------- | ----------- | --------------------------------------------------------- |
  | `clean`  | Boolean     | true: is complete result<br>false: is last updated result |
  | `depth`  | JSON object | depth json object, refer to query response                |
  | `market` | String      | market name                                               |

</Block>

<Block>

## Cancel subscription

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

### Request

- method

  `depth.unsubscribe`

</Block>

<Block>

# Kline API

**Provides a way to access charting candlestick info.**

</Block>

<Block>

## Kline query

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

The above command returns JSON structured like this:

> value from left to right: time, open, close, highest, lowest, volume, amount, market

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

**Query specified market kline information**

### Request

- method

  `kline.query`

- params

  | parameter  | type    | required | description             |
  | ---------- | ------- | -------- | ----------------------- |
  | `market`   | String  | Yes      | market name             |
  | `start`    | Integer | Yes      | start time, must be > 0 |
  | `end`      | Integer | Yes      | end time                |
  | `interval` | Integer | Yes      | interval                |

### Response

- result

  A list of kline information. Each kline data is a list:

  | field         | type    | description |
  | ------------- | ------- | ----------- |
  | `time`        | Integer | time        |
  | `open`        | String  | open        |
  | `close`       | String  | close       |
  | `highest`     | String  | highest     |
  | `lowest`      | String  | lowest      |
  | `volume`      | String  | volume      |
  | `amount`      | String  | amount      |
  | `market_name` | String  | market name |

</Block>

<Block>

## Kline subscription

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

**Subscribe specified market kline information.**

::: warning
Can only subscribe to one market at the same time, market list is not supported currently. For multiple subscriptions, only the last one takes effect.
:::

### Request

- method

  `kline.subscribe`

- params

  | field      | type    | description |
  | ---------- | ------- | ----------- |
  | `market`   | String  | market name |
  | `interval` | Integer | interval    |

</Block>

<Block>

## Kline notification

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

**Notify kline information of subscribed market.**

### Notify

- method

  `kline.update`

- params

  | field          | type    | description                                                                      |
  | -------------- | ------- | -------------------------------------------------------------------------------- |
  | `time`         | Integer | time                                                                             |
  | `open`         | String  | open                                                                             |
  | `close`        | String  | close                                                                            |
  | `highest`      | String  | highest                                                                          |
  | `lowest`       | String  | lowest                                                                           |
  | `volume`       | String  | volume                                                                           |
  | `amount`       | String  | amount                                                                           |
  | `market name`  | String  | market name                                                                      |
  | `window_close` | Boolean | `true` means window close. `true` may be missing, but does not affect data usage |

</Block>

<Block>

## Cancel subscription

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

**Unsubscribe specified market kline information.**

### Request

- method

  `kline.unsubscribe`

</Block>

<Block>

# Auth API

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

Gate provides a signature based authorization on private channels.

The algorithm can be simply described as below pseudo-code.

    `base64(hmac_sha512(secret_key, nonce))`

You can refer to code example at the right.

</Block>

<Block>

## Send authentication

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

**Signature based authorization.**

### Request

- method

  `server.sign`

- params

| parameter   | type    | required | description                                                       |
| ----------- | ------- | -------- | ----------------------------------------------------------------- |
| `apikey`    | String  | yes      | user apikey                                                       |
| `signature` | String  | yes      | user sign data                                                    |
| `nonce`     | Integer | yes      | timestamp, for milliseconds spent from Unix epoch to current time |

</Block>

<Block>

# Order API

::: warning
Authentication required before connection.
:::

</Block>

<Block>

## Order query

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

The above command returns JSON structured like this:

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

**Query user un-executed orders**

### Request

- method

  `order.query`

- params

  | parameter | type    | required | description |
  | --------- | ------- | -------- | ----------- |
  | `market`  | String  | yes      | market name |
  | `offset`  | Integer | yes      | offset      |
  | `limit`   | Integer | yes      | limit       |

### Response

- result

  | field     | type    | description         |
  | --------- | ------- | ------------------- |
  | `limit`   | Integer | limit               |
  | `offset`  | Integer | offset              |
  | `total`   | Integer | total               |
  | `records` | Object  | order record object |

  - record

    | field          | type    | description                     |
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

## Order subscription

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

**Subscribe user orders update**

### Request

- method

  `order.subscribe`

- params

  | parameter     | type   | required | description                        |
  | ------------- | ------ | -------- | ---------------------------------- |
  | `market list` | String | yes      | market list, null to subscribe all |

</Block>

<Block>

## Order notification

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

**Notify user orders information when an order is put, updated or finished.**

### Notify

- method

`order.update`

- params

  | parameter | type    | required | description                                      |
  | --------- | ------- | -------- | ------------------------------------------------ |
  | `event`   | Integer | yes      | event type,Integer, 1: PUT, 2: UPDATE, 3: FINISH |
  | `order`   | String  | yes      | order detail,Object                              |

  - order

    | field          | type    | description                     |
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

### Response

</Block>

<Block>

## Cancel subscription

<Example>

```python
from websocket import create_connection

ws = create_connection("wss://ws_server_host/v3/websocket/")
ws.send('{"id":12312, "method":"order.unsubscribe", "params":["EOS_USDT"]}')
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

**Unsubscribe user orders update notification, for all markets.**

### Request

- method

  `order.unsubscribe`

</Block>

<Block>

# Balance API

::: warning
Authentication required before connection.
:::

</Block>

<Block>

## Balance query

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

The above command returns JSON structured like this:

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

**Acquire user balance information of specified asset or assets.**

### Request

- method

  `balance.query`

- params

  | parameter    | type   | required | description                      |
  | ------------ | ------ | -------- | -------------------------------- |
  | `asset list` | String | yes      | asset list, null for inquire all |

### Response

- result

  | field         | type   | description                |
  | ------------- | ------ | -------------------------- |
  | `balance set` | Object | set of balance information |

</Block>

<Block>

## Balance subscription

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

**Subscribe for user balance update.**

### Request

- method

  `balance.subscribe`

- params

  | parameter    | type   | required | description                       |
  | ------------ | ------ | -------- | --------------------------------- |
  | `asset list` | String | yes      | asset list, null to subscribe all |

</Block>

<Block>

## Balance notification

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

**Notify user balance update.**

### Notify

- method

  `balance.update`

- params

  a list of balance information.

</Block>

<Block>

## Cancel subscription

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

**Unsubscribe user balance update.**

### Request

- method

  `balance.unsubscribe`

</Block>
