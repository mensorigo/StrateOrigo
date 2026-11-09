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
Gate provides a simple tradfi quote push interface, which will be pushed to the subscribed client as soon as the market changes.

## BaseURL

- `wss://fx-ws.gateio.ws/v4/ws/tradfi`

</Block>

<Block>

## changelog

**v0.1.0**

2026-01-14

- Initial launch: supports push of Gate's TradFi business-related data content

</Block>

<Block>

## Websocket API Overview

### Method

Each general api (such as ticker, order book etc.) supports 3 different event messages, they are:

1. **`subscribe`** (**RECOMMENDED TO USE**)

   Subscribe to receive notification from server when new data is available.

2. **`unsubscribe`**

   Server will not send new data notification if unsubscribed.

3. **`update`**

   If new subscribed data(incremental data) is available, server will send a notification to client.


### Request

Each request follows a common format, which contains `time`, `channel`, `event` and `payload`.

| Field     | type    | required | description                                                                                                |
|-----------| ------- |-----|--------------------------------------------------------------------------------------------------------------|
| `id`      | Integer | no  | Request ID, which is used to track requests and responses, and the corresponding responses carry the same ID |
| `time`    | Integer | no  | request time                                                                                                 |
| `channel` | String  | yes | request subscribe/unsubscribe channel                                                                                  |
| `auth`    | String  | no  | request auth info, see Authentication section for details                                                                                    |
| `event`   | String  | yes   | request event (subscribe/unsubscribe/update)                                                                      |
| `payload` | Any   | yes   | request detail parameters                                                                                                      |

### Response

Similar with request, response follows a common format composed of `time`, `channel`, `event`
, `error` and `result`.

| Field     | type    | required | description                                                  |
| --------- | ------- | ---- |------------------------------------------------------------|
| `time`    | Integer | Yes   | response time                                              |
| `channel` | String  | Yes   | response channel                                           |
| `event`   | String  | Yes   | response channel event (update/all)                        |
| `error`   | Object  | Yes   | response error                                             |
| `result`  | Any     | Yes   | New data notification from the server, or response to client requests. Null if`error` is not null. |

Note: If it is a server-initiated data update notification, then the type of 'result' is channel-based, and the 'result' type is different for different channels.

However, if it is a response to a client subscription request, then the 'result' is a fixed {"status": "success"}'.
To verify that the subscription request is successful, you only need to check that the 'error' field is empty, and you don't need to resolve the 'result' field.

For simplicity, the channel description below only gives the payload format for the corresponding channel.

### Error

If an error occurs, you receive an error field with the error code and the type of error.

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

## Authentication

::: warning
Note: The GateAPIv4 key pair you are using must have at least the option read permission enabled, and if the whitelist of the key is enabled, your outbound IP address must be in the IP whitelist of the key.
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

If the channel is private, for example, a client request needs to carry authentication information. For example: 'tradfi.orders' retrieves the channel where the user's order is updated.

Authentication is sent via the 'auth' field in the body of the request in the following format:

<a id="schema_auth"></a>

| Field    | Type   | Description                                                                       |
| -------- | ------ | --------------------------------------------------------------------------------- |
| `method` | String | Authentication method. Currently only one method `api_key` is accepted            |
| `KEY`    | String | Gate APIv4 user key string                                                        |
| `SIGN`   | String | Authentication signature generated using GateAPIv4 secret and request information |

WebSocket authentication uses the same signature calculation method with Gate APIv4 API, i.e.,
`HexEncode(HMAC_SHA512(secret, signature_string))`, but has the following differences:

1. Signature string concatenation method: `channel=<channel>&event=<event>&time=<time>`,
   where `<channel>`, `<event>`, `<time>` are corresponding request information
2. Authentication information are sent in request body in field `auth`.

You can log into the console to retrieve Gate APIv4 key and secret.

</Block>

<Block>

## websocket connection management

- Gate websockets use protocol-layer ping/pong messages. The server initiates a ping. If the client does not respond, the client will be disconnected.
  The server sends a ping message for the protocol every 20s, and if it does not receive a pong message for the protocol for 60s, it disconnects.
  [protocol layer ping/pong] (https://tools.ietf.org/html/rfc6455)
- Up to 30 connections per IP
- Up to 30 connection attempts per minute per IP
- Up to 10 requests per second per connection

</Block>

<Block>

## Ping/Pong

`tradfi.ping`

Ping/Pong
check the server client connection

If you want to proactively detect the connection status, you can send an app-layer ping message and receive a pong message.

<Example>

Code samples

```python
import time
# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/tradfi")
ws.send('{"time": %d, "channel": "tradfi.ping"}'% int(time.time()))
print(ws.recv())
```

Response example：

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

# tradfi ticker channel

`tradfi.tickers`

`tickers`It is a high-level overview of the market. It displays information such as the latest trading price, best ask price, best bid price, index price, and more.

**push type**: `incremental`

**update frequency**: `250ms`

</Block>

<Block>

## Client Subscription

<Example>

Code samples

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

Payload format:

| Field     | type    | required | description        |
|-------------|----------| ---- |--------------------|
| `payload`   | `object` | Yes   | request parameters |
| »»`markets` | `array`  | Yes   | List of symbols    |

You can subscribe/unsubscribe multiple times. Contract subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: tip
This channel does not require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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

| Field    | Type   | Description                                                      |
| ------------------ |----------------|--------------------------------------------------------------|
| `result`           | array          | Ticker array                                                 |
| »» `timestamp`          | string         | The timestamp generated by the quote (Unix time, in seconds) |
| »»` symbol`    | string         | symbol                                                       |
| »» `open_price`    | string         | opening price                                                |
| »» `last_price`   | string         | last price                                                   |
| »» `price_change_amount`     | string | price change last price opening price                                             |
| »» `price_change_rate`    | string         | price change unit percentage                                                |
| »» `high`     | string | the highest price of the day                                                        |
| »» `low`    | string         | the lowest price of the day                                                        |


</Block>

<Block>

# tradfi candlestick channel

`tradfi.candlesticks`

Provides a way to access charting candlestick info.

**push type**: `incremental`

**update frequency**: `250ms`

</Block>

<Block>

## Client Subscription

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

Payload format:

| Field           | Type            | Required | Description                                                   |
|--------------| --------------- |----------|---------------------------------------------------------------|
| `payload`    | `Array[String]` | yes      | Subscription parameters. From left to right, `interval`, `symbol` |
| » `interval` | String          | yes        | Candlestick data point interval                               |
| » `symbol`   | String          | yes        | symbol name                                                   |

#### Enumerated Values

| Field     | Value   |
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

To subscribe to multiple contracts or with different intervals, just send multiple subscribe request with different parameters.

::: tip
This channel does not require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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

Result format:

| Field    | Type    | Description                                                      |
|----------|---------|------------------------------------------------------------------|
| `result` | Array   | Array of Candlesticks                                            |
| `t`      | Integer | Unix timestamp in seconds                                        |
| `o`      | String  | Open price                                                       |
| `c`      | String  | Close price                                                      |
| `h`      | String  | Highest price                                                    |
| `l`      | String  | Lowest price                                                     |
| `v`      | Integer | Total volume                                                     |
| `a`      | String  | Amount                                                           |
| `n`      | String  | Name of the subscription, in the format of `<interval>_<symbol>` |
| `w`      | bool    | windows_close, whether the candlestick is over                                          |

</Block>

<Block>

# best bid and ask price

`tradfi.order_book`

**push type**: `continuous`

**update frequency**: `250ms`

</Block>

<Block>

## Client Subscription

<Example>

Code samples

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

Payload format:

| Field        | Type            | Required | Description |
|-----------|---------|----------|-------------|
| `payload`   | `array` | yes      | symbol list |


You can unsubscribe multiple times. Previously subscribed contracts are not overwritten unless you explicitly unsubscribe.

::: tip
This channel does not require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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

Result format:

| Field    | Type    | Description                                                  |
| -------- |---------|------------------------|
| `result` | Array   | result                     |
| »`time`     | Integer | Timestamp generated by the quote (Unix time, in seconds) |
| »`symbol`     | String  | symbol                 |
| »`bid`     | String  | best bid price                 |
| »`ask`    | String  | best selling price                |

</Block>

<Block>

# order channel

`tradfi.orders`

provides a way to receive updates on user orders

**push type**: `continuous`

**update frequency**: `real-time`

</Block>

<Block>

## Client Subscription

<Example>

Code samples

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

After the order subscription is successful, all symbol orders will be pushed

::: warning
This channel require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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

Result format:

| Field    | Type    | Description                                                                                             |
|---------------|-----------------|---------------------------------------------------------------------------------------------------------|
| `result`      | `Array[Object]` | updated order list                                                                                      |
| » order_id    | string          | Order unique ID                                                                                         |
| » gate_uid    | int64           | Gate user unique ID                                                                                     |
| » symbol      | string          | symbol (e.g. XAUUSD)                                                                                    |
| » side        | int             | trade direction 1 sell 2 buy                                                                              |
| » volume      | string          | order quantity                                                                                          |
| » fill_volume | string          | quantity sold                                                                                           |
| » price       | string          | order price                                                                                             |
| » price_tp    | string          | take profit price 0 means not set                                                                       |
| » price_sl    | string          | stop loss price 0 means not set                                                                         |
| » finished    | string          | Whether the order is completed (0 = not completed, 1 = completed)                                       |
| » finished_as | string          | Order completion method                                                                                 |
| » time_setup  | int64          | Order creation time (Unix time, seconds)                                                                |
| » timestamp   | int64          | Order status update timestamp (Unix time, milliseconds)                                                 |
| » order_opt_type | int          | Order Operation Type 1: Sell 2: Buy 3: Close Long 4: Close Short 5: Liquidates Long 6: Liquidates Short |

</Block>

<Block>

# user position channel

`tradfi.position`

provides a way to receive user positions

**push type**: `continuous`

**update frequency**: `real-time`

</Block>

<Block>

## Client Subscription

<Example>

Code samples

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

After the subscription is successful, all symbol positions will be pushed

::: warning
This channel require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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


Result format:

| Field    | Type    | Description                                           |
|---------------|-----------------|-------------------------------------------------------|
| `result`      | `Array[Object]` | updated position list                                 |
| »`position_id` | int64           | the unique id of the position                         |
| »`gate_uid` | int64           | gate user unique id                                   |
| »`side` | int             | Position direction (1 = short / sell, 2 = long / buy) |
| »`symbol` | string          | symbol (e.g. NZDSEK)                                  |
| »`volume` | string          | number of positions                                                  |
| »`price_open` | string          | opening price                                                  |
| »`price_tp` | string          | take profit price (0 means not set)                                        |
| »`price_sl` | string          | Stop Loss Price (0 means not set)                                         |
| »`time_create` | number          | Position creation time (Unix time, milliseconds)                                  |

</Block>

<Block>

# user balance channel

`tradfi.balance`

provides a way to receive user balance change.

**push type**: `continuous`

**update frequency**: `real-time`

</Block>

<Block>

## Client Subscription

<Example>

Code samples

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

The full balance will be pushed after the balance subscription is successful

::: warning
This channel require authentication
:::

</Block>

<Block>

## Server Notification

<Example>

Notification example

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

Result format:

| Field    | Type    | Description                                                                                    |
|---------------|-----------------|------------------------------------------------------------------------------------------------|
| `result`      | `Array[Object]` | updated balance list                                                                           |
| deal_id | string          | the unique id of the trade                                                                     |
| gate_uid | int64           | gate user unique id                                                                            |
| change | string          | Balance change amount (positive number indicates increase, negative number indicates decrease) |
| comment | string          | remarks on balance changes                                                                     |
| timestamp | int64          | Balance change timestamp (Unix time, milliseconds)                                                                            |

</Block>

