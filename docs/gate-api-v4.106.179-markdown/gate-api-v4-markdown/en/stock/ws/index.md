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

Gate provides a WebSocket push interface for TradFi spot markets (US / HK / KR stocks). When market data changes, updates are pushed to subscribed clients in real time.

## BaseURL

- `wss://fx-ws.gateio.ws/v4/ws/stock`

## Supported Markets

The service supports **US stocks** (`exchange=us`), **HK stocks** (`exchange=hk`), and **KR stocks** (`exchange=kr`). Subscription parameters are unchanged; use the `exchange` field in push messages to distinguish markets.

</Block>

<Block>

## Changelog

**v1.0.0**

2026-06-30

- Initial public release of Stock WebSocket documentation
- Supported channels: `stock.tickers`, `stock.orderbooks`, `stock.kline`, `stock.orders`, `stock.position`
- Multi-market support (us / hk / kr) with pre-market / after-hours ticker data

</Block>

<Block>

## Websocket API Overview

### Method

Each general channel (such as ticker, order book, etc.) supports 3 different event messages:

1. **`subscribe`** (**RECOMMENDED TO USE**)

   Subscribe to receive notification from server when new data is available.

2. **`unsubscribe`**

   Server will not send new data notification if unsubscribed.

3. **`update`**

   If new subscribed data (incremental data) is available, server will send a notification to client.

### Request

Each request follows a common format, which contains `id`, `time`, `channel`, `event` and `payload`.

| Field     | type    | required | description                                                                                                |
|-----------| ------- |-----|--------------------------------------------------------------------------------------------------------------|
| `id`      | Integer | no  | Request ID, which is used to track requests and responses, and the corresponding responses carry the same ID |
| `time`    | Integer | no  | request time                                                                                                 |
| `channel` | String  | yes | request subscribe/unsubscribe channel                                                                                  |
| `auth`    | Object  | no  | request auth info, see Authentication section for details                                                                                    |
| `event`   | String  | yes   | request event (subscribe/unsubscribe/update)                                                                      |
| `payload` | Any   | yes   | request detail parameters                                                                                                      |

### Response

Similar with request, response follows a common format composed of `time`, `channel`, `event`, `error` and `result`.

| Field     | type    | required | description                                                  |
| --------- | ------- | ---- |------------------------------------------------------------|
| `time`    | Integer | Yes   | response time                                              |
| `channel` | String  | Yes   | response channel                                           |
| `event`   | String  | Yes   | response channel event (update/all)                        |
| `error`   | Object  | No   | response error                                             |
| `result`  | Any     | Yes   | New data notification from the server, or response to client requests. Null if `error` is not null. |
| `conn_id` | String  | No   | Current connection ID                                      |

Note: If it is a server-initiated data update notification, then the type of `result` is channel-based, and the `result` type is different for different channels.

However, if it is a response to a client subscription request, then `result` is fixed as `{"status": "success"}`.
To verify that the subscription request is successful, you only need to check that the `error` field is empty, and you don't need to resolve the `result` field.

For simplicity, the channel descriptions below only give the payload format for the corresponding channel.

### Error

If an error occurs, you receive an error field with the error code and the type of error.

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

## Authentication

::: warning
Note: The GateAPIv4 key pair you use must have at least read permissions enabled. If IP whitelisting is enabled for the key, your outbound IP address must be in the key's IP whitelist.
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

If the channel is private, for example `stock.orders` or `stock.position`, the client request needs to carry authentication information.

Authentication is sent through the `auth` field in the request body, formatted as follows:

<a id="schema_auth"></a>

| Name     | Type   | Description                |
| -------- | ------ | ------------------- |
| `method` | String | Authentication method: `api_key` |
| `KEY`    | String | apiKey value (`api_key` method)         |
| `SIGN`   | String | Signature result            |

WebSocket authentication uses the same signature calculation method as Gate APIv4 API, i.e. `HexEncode(HMAC_SHA512(secret, signature_string))`, but with the following differences:

1. Signature string concatenation method: `channel=<channel>&event=<event>&time=<time>`, where `<channel>`, `<event>`, `<time>` correspond to the request information
2. Authentication information is sent in the `auth` field of the request body.

You can log in to Gate console to obtain the api_key and secret for Gate APIv4.

</Block>

<Block>

## Websocket Connection Management

- Gate websocket uses protocol-level ping/pong messages. The server initiates ping. If the client does not reply, the connection will be closed.
  The server sends a protocol ping every 20s. If no protocol pong is received within 60s, the connection is disconnected.
  [protocol layer ping/pong](https://tools.ietf.org/html/rfc6455)
- App clients are recommended to send both Header `version` and `X-Gate-Device-Type` on connect (see server implementation notes)
- After disconnect, you must re-subscribe to all channels; `conn_id` will change

</Block>

<Block>

## Ping/Pong

`stock.ping`

Ping/Pong checks server/client connectivity.

For application-layer keepalive, send a `subscribe` request on channel `stock.ping`. On success, the response `channel` is rewritten to `stock.pong`.

<Example>

Example:

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

Response JSON structure:

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

# Ticker Channel

`stock.tickers`

The `stock.tickers` channel pushes spot ticker snapshots and trade-driven incremental updates for US / HK / KR stocks.

**Push Type**: `incremental`

**Update Frequency**: `200ms` (configurable)

</Block>

<Block>

## Client Subscription

<Example>

Example:

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

Format:

| Name          | Type       | Required | Description   |
|-------------|----------| ---- |------|
| `payload`   | `object` | Yes   | Request parameters |
| »»`markets` | `array`  | Yes   | Symbol list |

To unsubscribe all ticker subscriptions on this connection, include `unsub_all` in `markets`.

::: tip
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

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

| Name               | Type             | Description                     |
| ------------------ |----------------|------------------------|
| `result`           | array          | Ticker array           |
| »» `symbol`    | string         | Symbol code                  |
| »» `symbol_name`    | string         | Display name                 |
| »» `last_price`   | string         | Latest price (regular session)                 |
| »» `today_open_price`    | string         | Today's open                 |
| »» `last_today_close_price`     | string | Previous close                 |
| »» `prev_close_price`    | string         | Previous trading day regular-session close; `"0"` if unavailable                 |
| »» `highest_price` / `lowest_price`     | string | Today's high / low                 |
| »» `high_24h` / `low_24h`     | string | All-session high / low (including pre/post market)                 |
| »» `change` / `price_change_rate`     | string | Change rate, multiplied by 100 (e.g. `-11.93` means -11.93%)                 |
| »» `change_utc0` / `change_utc8`     | string | Change rate in UTC0 / UTC+8, same as `change`                 |
| »» `change_amount`     | string | Price change amount                 |
| »» `status`    | string         | Trading switch: `open` / `closed`                 |
| »» `status_code`    | int         | Market session: pre `1`, post `2`, regular `3`, closed `4`; internal matching `gt_lp` is `5` (US) or `6` (HK/KR). Use this field for session logic, not `status` alone                 |
| »» `open_cal`    | int         | Whether to calculate regular-session P&L: `1` when `status_code=3`, otherwise `0`                 |
| »» `market_cal`    | int         | Whether to calculate pre/post/regular P&L: `1` when `status_code` is `1`, `2`, or `3`, otherwise `0`                 |
| »» `pre_market_last_price` / `pre_market_change_amount` / `pre_market_change`    | string         | Pre-market price and change fields when available                 |
| »» `after_market_last_price` / `after_market_change_amount` / `after_market_change`    | string         | Post-market price and change fields when available                 |
| »» `price_limit_status`    | string         | Limit status: `limit_up`, `limit_down`, or `""` if not at limit                 |
| »» `total_market_cap` / `price_earn_ratio`    | string         | Total market cap / P/E ratio                 |
| »» `fx_rate`    | string         | FX rate; `"1"` for US stocks, USD rate for HK/KR (6 decimal places)                 |
| »» `fx_usdt`    | string         | Price in USDT                 |
| »» `exchange`    | string         | Market: `us` / `hk` / `kr`                 |
| »» `quote_currency`    | string         | Quote currency, e.g. `USD`, `HKD`, `KRW`                 |
| »» `quote_currency_precision`    | int         | Quote currency precision                 |
| »» `precision`    | int         | Display decimal precision                 |


</Block>

<Block>

# Candlestick Channel

`stock.kline`

Provides K-line (candlestick) updates.

**Push Type**: `incremental`

**Update Frequency**: `200ms` (configurable)

</Block>

<Block>

## Client Subscription

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

Data format:

| Name           | Type            | Required | Description                         |
|--------------| --------------- | ---- |----------------------------|
| `payload`    | `Array[String]` | Yes   | `[interval, symbol, include_extended_hours]` |
| » `interval` | String          | Yes   | Candle interval                   |
| » `symbol`   | String          | Yes   | Symbol                     |
| » 3rd param   | String          | No   | `"true"` / `"false"`, default `"false"`; US stocks only                     |

#### Enumerated Values

| Property     | Value   |
| -------- |-----|
| interval | 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M, 30d, etc. |

::: tip
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

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

Data structure:

| Name       | Type      | Description                              |
|----------|---------|---------------------------------|
| `result` | Object   | Candlestick data                         |
| `t`      | Integer | Candle start time                 |
| `o` / `c` / `h` / `l`      | String  | Open / close / high / low                             |
| `v`      | String | Volume                            |
| `a`      | String  | Turnover                              |
| `n`      | String  | Subscription name: `<interval>_<symbol>` |
| `w`      | bool    | Whether the window is closed             |
| `m`      | String    | Optional. `earlyHours`, `afterHours`, `open`, `closed`             |
| `exchange`      | String    | Market: `us` / `hk` / `kr`             |

</Block>

<Block>

# Order Book Channel

`stock.orderbooks`

Pushes order book depth; supports merged external and internal ME quotes.

**Push Type**: `continuous`

**Update Frequency**: `200ms` (configurable)

</Block>

<Block>

## Client Subscription

<Example>

Code example

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

Data structure:

| Name        | Type      | Required | Description                         |
|-----------|---------| ---- | ---------------------------- |
| `payload`   | `array` | Yes   | Symbol list |


::: tip
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

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

Push parameters:

| Name     | Type      | Description                     |
| -------- |---------|------------------------|
| `result` | Object   | Order book object                     |
| »`t` / `ut`     | Integer | Timestamp (ms) |
| »`symbol`     | String  | Symbol                 |
| »`bids` / `asks`     | Array  | Bid / ask price arrays (bids desc, asks asc)                 |
| »`mode`    | int  | Quote mode                |
| »`exchange`    | String  | Market: `us` / `hk` / `kr`                |

</Block>

<Block>

# Order Channel

`stock.orders`

Provides user order update pushes.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

<Example>

Code example

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

Use `["!all"]` in `payload` to subscribe to all orders.

::: warning
Authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

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

Push parameters:

| Name               | Type            | Description                                         |
|------------------|---------------|--------------------------------------------|
| `result`         | Object | Order object                                    |
| » `id`       | string        | Order ID                                    |
| » `symbol`         | string        | Symbol                             |
| » `side`           | int           | Order side (integer enum)                         |
| » `type`             | int    | Order type (integer enum)                                               |
| » `price` / `qty`          | string        | Order price / quantity                                       |
| » `filled_qty` / `filled_value`    | string        | Filled quantity / filled value                                      |
| » `status`           | int    | **OrderStatus** (integer enum)          |
| » `finish_as`        | int    | **OrderFinishAs** (integer enum)                       |
| » `exchange`         | string           | Market: `us` / `hk` / `kr`              |
| » `precision` / `volume_precision`        | int           | Price / quantity decimal places (server injected)                        |

**OrderStatus values**:

| Value  | Description    |
| --- | ----- |
| `1` | New   |
| `2` | Partially filled  |
| `3` | Filled  |
| `4` | Canceled  |
| `5` | New order rejected |
| `6` | Cancel request rejected |

</Block>

<Block>

# Position Channel

`stock.position`

Provides user position update pushes.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

<Example>

Code example

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
Authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

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


Push parameters:

| Name            | Type              | Description                            |
|---------------|-----------------|-------------------------------|
| `result`      | Object | Position object                       |
| »`user_id` | uint32           | User ID                       |
| »`symbol` | string          | Symbol                |
| »`type` | int             | Event type: `1` sell, `2` buy |
| »`change` / `qty` | string          | Change quantity / total position                          |
| »`sell_qty` / `buy_qty` etc. | string          | Today's sell / buy quantity and value                  |
| »`yesterday_qty`   | string | Yesterday's closing quantity                  |
| »`avg_entry_price` | string | Average entry price            |
| »`exchange`         | string | Market: `us` / `hk` / `kr` |
| »`quote_currency`   | string | Quote currency |
| »`fx_rate`          | string | FX rate; `"1"` for US stocks; HKD/KRW to USD rate (6 decimals) for HK/KR |
| »`precision` / `volume_precision`        | int    | Price / quantity decimal places |

</Block>
