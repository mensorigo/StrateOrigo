---
title: CrossEx WebSocket
language_tabs:
  - python: Python
  - shell: Shell

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# CrossEx WebSocket v1.0.0

CrossEx provides a simple and powerful Websocket API to integrate trading status into your business or application.

## Server URL

- Production:
  - Public Market: wss://api.gateio.ws/ws/crossex/public 
  - Private Push: wss://api.gateio.ws/ws/crossex
- Heartbeat: Clients must send a **protocol-level** Ping frame to the WebSocket server every 30 seconds to maintain the connection. If the server does not receive this frame for more than 30 seconds, it will automatically disconnect.
- Connection limit: Maximum 20 connections per uid+channel

</Block>

<Block>

## Changelog
**Oct 12 2026**

1. CrossEx adds market data, order, and security position updates for stocks and ETFs in the US, Hong Kong, South Korean, and Japanese markets
2. Added the security position subscription channel (`security_position`)

**Sep 15 2026**

1. CrossEx adds support for LIGHTER exchange

**Aug 27 2026**
1. Error responses now include the `label` field, consistent with REST API responses

**Aug 11 2026**
1. Added Hyperliquid isolated margin support to CrossEx, including changing the margin mode and adding or reducing isolated margin
2. CrossEx Convert now supports USDC on Hyperliquid and can also be used to repay liabilities

**Jul 15 2026**
1. Added the batch order cancellation API channel (`batch_cancel_orders`)

**Jul 9 2026**
1. Added the ADL rank subscription channel (`adl_rank`), providing both the native exchange ADL rank and the CrossEx ADL rank

**Jul 2 2026**
1. CrossEx adds support for DERIBIT exchange

**Jun 9 2026**
1. CrossEx adds support for HYPERLIQUID exchange

**May 19 2026**
1. CrossEx adds support for KRAKEN exchange

**Mar 24 2026**
1. Added Ticker subscription channel (ticker) for spot/futures ticker updates from major exchanges
2. Added Public Trade subscription channel (trade) for tick-by-tick trade updates from major exchanges
3. Added Kline subscription channel (kline_x) supporting multiple intervals: 1m/5m/15m/30m/1h/4h/1d
4. Added Futures Funding Rate subscription channel (funding_rate) for futures funding rate updates
5. Added Futures Open Interest subscription channel (open_interest) for futures open interest updates (excluding BINANCE)

**Mar 12 2026**
1. CrossEx adds support for BYBIT exchange

**Feb 27 2026**
1. Added public market data push, including last trade price, index price, mark price
2. Added incremental order book depth and full order book depth subscriptions

**Jan 21 2026**
1. Added channel for complete position closing
2. Added connection limit description
3. Added rate limit descriptions for all APIs

**Jan 19 2026**
1. Added margin position subscription/unsubscription
2. Added margin interest subscription/unsubscription
3. Added modify spot margin leverage

</Block>

<Block>

## Websocket API Overview

### Events

Each common channel (e.g., order, asset, etc.) supports 2 different event messages:

1. **`subscribe`** (**Recommended**)

   Subscribe to receive new data pushes from the server.

2. **`unsubscribe`**

   If unsubscribed, the server will not send new data pushes.

### Request Parameters

Each request follows a common format containing `time`, `channel`, `event` and `payload`.

| Name      | Type    | Required | Description                                             |
| --------- | ------- | -------- | ------------------------------------------------ |
| `time`    | Integer | Yes      | Request time                                         |
| `channel` | String  | Yes      | Request subscribe/unsubscribe channel                  |
| `auth`    | Object  | No       | Request authentication information, see Authentication section for details |
| `event`   | String  | Yes      | Request event (subscribe/unsubscribe/api/login)    |
| `payload` | Array/Object   | Yes      | Request detailed parameters                                     |

### Push Parameters

Similar to requests, push parameters follow the following common format containing: `time`, `channel`, `event`, `error` and `result`.

| Name      | Type    | Required | Description                      |
| --------- | ------- | -------- | ------------------------- |
| `time`    | Integer | Yes      | Push time                  |
| `channel` | String  | Yes      | Push channel                  |
| `event`   | String  | Yes      | Push channel event (subscribe/update) |
| `error`   | Object  | No       | Push error                  |
| `result`  | Any     | Yes      | Returns new data notification from the server or response to client request. If there is an error, `error` is not empty, otherwise this field is empty. |

Note: If it is a data update notification initiated by the server, then the type of `result` is based on the channel, and the `result` type varies for different channels.

But if it is a response to a client subscription request, then `result` is fixed as `{"code": "100000", "message": "success"}`.
To verify whether the subscription request is successful, you only need to check whether the `error` field is empty, and you don't need to parse the `result` field.

For simplicity, the channel descriptions below only give the payload format for the corresponding channel.

### Errors

If an error occurs, you will receive an error field containing the error code and error type.

| Code | Message                   |
| ---- | ------------------------- |
| `100001`  | `Parameter missing or invalid` |
| `100002`  | `Not logged in`        |
| `100003`  | `Connection limit reached`           |
| `100004`  | `Timeout`     |
| `100005`  | `Operation failed`     |
| `100006`  | `Login failed`     |
| `100007`  | `Network error, please try again later`     |

</Block>

<Block>

## Authentication

::: warning
Note: The GateAPIv4 key pair you use must have at least CrossEx read permissions enabled. If IP whitelisting is enabled for the key, your outbound IP address must be in the key's IP whitelist.
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
    return {'method': 'api_key', 'api_key': api_key, 'sign': sign}


request = {
    'time': int(time.time()),
    'event': 'login',
    'payload': {
        'method': 'api_key',
        'api_key': 'YOUR_API_KEY',
        'sign': gen_sign('', 'login', int(time.time()))['sign']
    }
}
print(json.dumps(request))
```

</Example>

If the channel is private, for example, the client request needs to carry authentication information. For example: the `order` channel to retrieve user order updates.

Authentication is sent through the `payload` field in the request body, formatted as follows:

<a id="schema_auth"></a>

| Name     | Type   | Description                |
| -------- | ------ | ------------------- |
| `method` | String | Authentication method: `api_key` |
| `api_key`    | String | apiKey value         |
| `sign`   | String | Signature result            |

WebSocket authentication uses the same signature calculation method as Gate APIv4 API, i.e. `HexEncode(HMAC_SHA512(secret, signature_string))`, but with the following differences:

1. Signature string concatenation method: `channel=<channel>&event=<event>&time=<time>`, where `<channel>`, `<event>`, `<time>` correspond to the request information
2. Authentication information is sent in the `payload` field of the request body.

You can log in to your account to obtain the api_key and secret for your CrossEx account.

</Block>

<Block>

# Authentication

**» event**: login

**» method**: api_key

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event login|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» method|body|string| Yes |Authentication method|
|»» api_key|body|string| Yes |api key|
|»» sign|body|string| Yes |Signature|

<Example>

Request example

```json
{
	"time": 1754272114,
	"event": "login",
	"request_id": "request-1",
	"payload": {
		"method": "api_key",
		"api_key": "0d5245a5d8be8ac0e96ed8aaa47386cf",
		"sign": "568e694ebf998fc0e9a26ab26c8cb962d8649fd61fbfc61f068251ae8169f00066b3205bb2491c1625e111f18c4bf1e6bcf8d3fd519eaea8b7335edea960eee2"
	}
}
```

</Example>

## Response Data Structure

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» conn_id|string|true|none|Connection ID|
|» result|object|true|none|Login result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "event": "login",
  "request_id": "request-1",
  "payload": {
    "conn_id": "644ff57f-add4-407a-97aa-b654c4ede839"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168928
}
```

</Example>

</Block>

<Block>

# Last Price Subscription

`last_price`

The last price channel provides real-time last trade price updates from major exchanges for spot/margin and futures trading pairs.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs, subscribe all not supported|

### Details

**» event**: subscribe、unsubscribe

**» channel**: last_price

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "subscribe",
   "channel": "last_price",
   "payload": [
      "BINANCE_SPOT_BTC_USDT",
      "OKX_FUTURE_BTC_USDT",
      "GATE_FUTURE_ETH_USDT"
   ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
   "time": 1771988975,
   "channel": "last_price",
   "event": "update",
   "result": {
      "s": "BINANCE_SPOT_BTC_USDT",
      "lp": "65873.49000000"
   },
   "time_ms": 1771988975033
}
```

</Example>

Push parameters:

| Name       |Type|Required|Constraint| Description | Note |
|----------|---|---|---|--|----|
| » channel|string|true|none|Channel name|none|
| » event  |string|true|none|Event|none|
| » result |object|true|none|Result|none|
| »» s     |string|true|none|Trading pair|Symbol|
| »» lp    |string|true|none|Last price|LastPrice|

</Block>

<Block>

# Index Price Subscription

`index_price`

The index price channel provides real-time index price updates from major exchanges for spot margin trading pairs.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs, subscribe all not supported|

### Details

**» event**: subscribe、unsubscribe

**» channel**: index_price

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "subscribe",
   "channel": "index_price",
   "payload": [
      "BINANCE_MARGIN_BTC_USDT",
      "OKX_MARGIN_BTC_USDT",
      "GATE_MARGIN_ETH_USDT"
   ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
   "time": 1771990104,
   "channel": "index_price",
   "event": "update",
   "result": {
      "s": "BINANCE_MARGIN_BTC_USDT",
      "ip": "65590.40739130"
   },
   "time_ms": 1771990104116
}
```

</Example>

Push parameters:

| Name        |Type|Required|Constraint| Description | Note |
|-----------|---|---|---|--|----|
| » channel |string|true|none|Channel name|none|
| » event   |string|true|none|Event|none|
| » result  |object|true|none|Result|none|
| »» s      |string|true|none|Trading pair|Symbol|
| »» ip     |string|true|none|Index price|IndexPrice|

</Block>

<Block>

# Mark Price Subscription

`mark_price`

The mark price channel provides real-time mark price updates from major exchanges for spot margin trading pairs.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs, subscribe all not supported|

### Details

**» event**: subscribe、unsubscribe

**» channel**: mark_price

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "subscribe",
   "channel": "mark_price",
   "payload": [
      "BINANCE_FUTURE_BTC_USDT",
      "OKX_FUTURE_BTC_USDT",
      "GATE_FUTURE_ETH_USDT"
   ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
   "time": 1771990349,
   "channel": "mark_price",
   "event": "update",
   "result": {
      "s": "BINANCE_FUTURE_BTC_USDT",
      "mp": "65566.30000000"
   },
   "time_ms": 1771990349113
}
```

</Example>

Push parameters:

| Name        |Type|Required|Constraint| Description | Note |
|-----------|---|---|---|--|----|
| » channel |string|true|none|Channel name|none|
| » event   |string|true|none|Event|none|
| » result  |object|true|none|Result|none|
| »» s      |string|true|none|Trading pair|Symbol|
| »» mp     |string|true|none|Mark price|MarkPrice|

</Block>

<Block>

# Full Limited-Level Order Book Subscription

`order_book_x`

Where `x` indicates the order book snapshot depth level. The depth levels supported by each exchange are as follows:

- BINANCE: [5, 10, 20]
- OKX: [1, 5]
- GATE:
  - SPOT: [5, 10, 20, 30, 50, 100]
  - FUTURE: [1, 5, 10, 20, 50, 100]
- BYBIT: [1]
- KRAKEN: `order_book_x` snapshot depth levels are not supported, please use `order_book_update`
- HYPERLIQUID: [1, 5, 10, 20, 30, 50, 100, 400]
- DERIBIT: [1, 5, 10, 20]
- LIGHTER: [1, 5, 10, 20, 30, 50, 100, 400]

The full limited-level order book channel provides full limited-level order book depth push from major exchanges for spot/futures trading pairs. Stocks and ETFs are also supported. Stock and ETF order book prices use the native currency of the listing market (USD/HKD/KRW/JPY).

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs, subscribe all not supported|

### Details

**» event**: subscribe、unsubscribe

**» channel**: order_book_x

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "subscribe",
   "channel": "order_book_5",
   "payload": [
      "BINANCE_SPOT_BTC_USDT"
   ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
   "time": 1771990690,
   "channel": "order_book_5",
   "event": "update",
   "result": {
      "ts": 1771990690844,
      "s": "BINANCE_SPOT_BTC_USDT",
      "a": [
         [
            "65485.33000000",
            "2.25708000"
         ],
         [
            "65485.34000000",
            "0.00025000"
         ],
         [
            "65485.38000000",
            "0.15432000"
         ],
         [
            "65485.39000000",
            "0.50423000"
         ],
         [
            "65486.26000000",
            "0.00208000"
         ]
      ],
      "b": [
         [
            "65485.32000000",
            "0.04107000"
         ],
         [
            "65485.31000000",
            "0.00097000"
         ],
         [
            "65485.20000000",
            "0.00008000"
         ],
         [
            "65485.10000000",
            "0.00018000"
         ],
         [
            "65484.99000000",
            "0.00045000"
         ]
      ]
   },
   "time_ms": 1771990690844
}
```

</Example>

Push parameters:

| Name        | Type     |Required|Constraint| Description    | Note          |
|-----------|--------|---|---|-------|-------------|
| » channel | string |true|none| Channel name   | none        |
| » event   | string |true|none| Event    | none        |
| » result  | object |true|none| Result    | none        |
| »» ts     | long   |true|none| Millisecond timestamp | Timestamp      |
| »» s      | string |true|none| Trading pair   | Symbol      |
| »» a      | array  |true|none| Ask depth  | [Price,Qty] |
| »» b      | array  |true|none| Bid depth  | [Price,Qty] |

</Block>

<Block>

# Incremental Order Book Subscription

`order_book_update`

The incremental order book channel provides real-time order book depth updates from major exchanges for spot/futures trading pairs.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs, subscribe all not supported|

### Details

**» event**: subscribe、unsubscribe

**» channel**: order_book_update

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "subscribe",
   "channel": "order_book_update",
   "payload": [
      "BINANCE_FUTURE_BTC_USDT"
   ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
   "time": 1771991193,
   "channel": "order_book_update",
   "event": "update",
   "result": {
      "snapshot": false,
      "ts": 1771991193314,
      "s": "BINANCE_FUTURE_BTC_USDT",
      "U": 88686516370,
      "u": 88686516769,
      "a": [
         [
            "65475.36000000",
            "0.00000000"
         ],
         [
            "65475.37000000",
            "0.00000000"
         ]
      ],
      "b": [
         [
            "65479.75000000",
            "0.96961000"
         ],
         [
            "65478.71000000",
            "0.03395000"
         ]
      ]
   },
   "time_ms": 1771991193344
}
```

</Example>

Push parameters:

| Name          | Type     |Required|Constraint| Description                   | Note          |
|-------------|--------|---|---|----------------------|-------------|
| » channel   | string |true|none| Channel name                  | none        |
| » event     | string |true|none| Event                   | none        |
| » result    | object |true|none| Result                   | none        |
| »» snapshot | boolean |true|none| Whether it is a snapshot (yes for full override, otherwise incremental update) | Snapshot    |
| »» ts       | long   |true|none| Millisecond timestamp                | Timestamp      |
| »» s        | string |true|none| Trading pair                  | Symbol      |
| »» U        | string |true|none| Start sequence number                | StartIndex  |
| »» u        | string |true|none| End sequence number                | EndIndex    |
| »» a        | array  |true|none| Ask depth                 | [Price,Qty] |
| »» b        | array  |true|none| Bid depth                 | [Price,Qty] |

</Block>

<Block>

# Ticker Subscription

`ticker`

The ticker channel provides ticker updates for spot/futures markets from major exchanges and supports stocks and ETFs. Stock and ETF prices use the native currency of the listing market (USD/HKD/KRW/JPY).

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required| Description               |
|---|---|---|---|------------------|
|» time|body|integer| Yes | Timestamp (seconds)           |
|» event|body|string| Yes | Event               |
|» channel|body|string| Yes | Channel name              |
|» payload|body|[string]| Yes | Subscribe/Unsubscribe trading pairs, subscription to all is not supported yet |

### Details

**» event**: subscribe, unsubscribe

**» channel**: ticker

<Example>

Request example

```json
{
  "event": "subscribe",
  "channel": "ticker",
  "payload": [
    "BINANCE_FUTURE_BTC_USDT"
  ]
}
```

</Example>

Format:

| Name      | Type            | Required | Description     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | Yes   | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
  "time": 1770709567,
  "channel": "ticker",
  "event": "update",
  "result": {
    "s": "BINANCE_FUTURE_BTC_USDT",
    "lp": "67280.50",
    "bp": "67280.20",
    "bs": "35",
    "ap": "67280.70",
    "as": "28",
    "o": "69100.00",
    "h": "69100.00",
    "l": "66000.00",
    "v": "825432",
    "q": "5523487123.55",
    "ts": 1710001234567
  },
  "time_ms": 1770709567032
}
```

</Example>

Push parameters:

| Name        | Type     | Required    |Constraint| Description            | Remarks             |
|-----------|--------|-------|---|---------------|----------------|
| » channel | string | true  |none| Channel name           | none           |
| » event   | string | true  |none| Event            | none           |
| » result  | object | true  |none| Result            | none           |
| »» s      | string | true  |none| Trading pair           | Symbol         |
| »» lp     | string | true  |none| Latest trade price         | LastPrice      |
| »» bp     | string | true  |none| Best bid price          | BidPrice       |
| »» bs     | string | true  |none| Best bid size          | BidSize        |
| »» ap     | string  | true  |none| Best ask price          | AskPrice       |
| »» as     | string  | true  |none| Best ask size          | AskSize        |
| »» o      | string | true  |none| 24h open price        | Open24h        |
| »» h      | string | true  |none| 24h high price        | High24h        |
| »» l      | string | true  |none| 24h low price        | Low24h         |
| »» v      | string | true  |none| 24h volume (base)  | Volume24h      |
| »» q      | string | false |none| 24h volume (quote) | QuoteVolume24h |
| »» ts     | long   | true  |none| Exchange timestamp (milliseconds)      | Timestamp      |

</Block>

<Block>

# Public Trade Subscription

`trade`

The trade channel provides tick-by-tick public trade updates for spot/futures markets from major exchanges

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required| Description               |
|---|---|---|---|------------------|
|» time|body|integer| Yes | Timestamp (seconds)           |
|» event|body|string| Yes | Event               |
|» channel|body|string| Yes | Channel name              |
|» payload|body|[string]| Yes | Subscribe/Unsubscribe trading pairs, subscription to all is not supported yet |

### Details

**» event**: subscribe, unsubscribe

**» channel**: trade

<Example>

Request example

```json
{
  "event": "subscribe",
  "channel": "trade",
  "payload": [
    "BINANCE_FUTURE_BTC_USDT"
  ]
}
```

</Example>

Format:

| Name      | Type            | Required | Description     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | Yes   | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
  "time": 1770709567,
  "channel": "trade",
  "event": "update",
  "result": {
    "s": "BTCUSDT",
    "i": "18473628192",
    "p": "67250.12",
    "q": "0.015",
    "S": "BUY",
    "ts": 1710001234567,
    "m": false
  },
  "time_ms": 1770709567032
}
```

</Example>

Push parameters:

| Name        | Type     | Required    |Constraint| Description            | Remarks        |
|-----------|--------|-------|---|---------------|-----------|
| » channel | string | true  |none| Channel name           | none      |
| » event   | string | true  |none| Event            | none      |
| » result  | object | true  |none| Result            | none      |
| »» s      | string | true  |none| Trading pair           | Symbol    |
| »» i      | string | true  |none| Trade ID (exchange raw format) | TradeId   |
| »» p      | string | true  |none| Trade price           | Price     |
| »» q      | string | true  |none| Trade quantity (base)     | Quantity  |
| »» S      | string  | true  |none| Side            | BUY/SELL  |
| »» ts     | long   | true  |none| Exchange timestamp (milliseconds)      | Timestamp |

</Block>

<Block>

# Kline Subscription

`kline_x`
- kline_1m,kline_5m,kline_15m,kline_30m,kline_1h,kline_4h,kline_1d;

The kline channel provides kline updates for spot/futures markets from major exchanges and supports stocks and ETFs. Stock and ETF candlestick prices use the native currency of the listing market (USD/HKD/KRW/JPY).

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required| Description               |
|---|---|---|---|------------------|
|» time|body|integer| Yes | Timestamp (seconds)           |
|» event|body|string| Yes | Event               |
|» channel|body|string| Yes | Channel name              |
|» payload|body|[string]| Yes | Subscribe/Unsubscribe trading pairs, subscription to all is not supported yet |

### Details

**» event**: subscribe, unsubscribe

**» channel**: kline

<Example>

Request example

```json
{
  "channel": "kline_1m",
  "event": "subscribe",
  "payload": ["BYBIT_FUTURE_BTC_USDT", "BINANCE_FUTURE_ETH_USDT"]
}
```

</Example>

Format:

| Name      | Type            | Required | Description     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | Yes   | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
  "channel": "kline_1m",
  "event": "update",
  "result": {
    "s": "BYBIT_FUTURE_BTC_USDT",
    "o": "95000",
    "h": "95100",
    "l": "94900",
    "c": "95050",
    "v": "1234.5",
    "t": 1710000000000,
    "T": 1710000059999,
    "x": true
  }
}
```

</Example>

Push parameters:

| Name        | Type      | Required    |Constraint| Description  | Remarks     |
|-----------|---------|-------|---|-----|--------|
| » channel | string  | true  |none| Channel name | none   |
| » event   | string  | true  |none| Event  | none   |
| » result  | object  | true  |none| Result  | none   |
| »» s      | string  | true  |none| Trading pair | Symbol |
| »» o      | string  | true  |none| Open price | OpenPrice |
| »» h      | string  | true  |none| High price | HighPrice |
| »» l      | string  | true  |none| Low price | LowPrice |
| »» c      | string  | true  |none| Close price | ClosePrice |
| »» v      | string  | true  |none| Volume | Volume |
| »» t      | long    | true  |none| Kline start time (milliseconds) | StartTime |
| »» T      | long    | true  |none| Kline end time (milliseconds) | EndTime |
| »» x      | boolean | true  |none| Is kline closed (0: in progress, 1: closed)| Confirm|

</Block>

<Block>

# Futures Funding Rate Subscription

`funding_rate`

The funding_rate channel provides funding rate updates for futures markets from major exchanges

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required| Description               |
|---|---|---|---|------------------|
|» time|body|integer| Yes | Timestamp (seconds)           |
|» event|body|string| Yes | Event               |
|» channel|body|string| Yes | Channel name              |
|» payload|body|[string]| Yes | Subscribe/Unsubscribe trading pairs, subscription to all is not supported yet |

### Details

**» event**: subscribe, unsubscribe

**» channel**: funding_rate

<Example>

Request example

```json
{
  "event": "subscribe",
  "channel": "funding_rate",
  "payload": [
    "BINANCE_FUTURE_BTC_USDT"
  ]
}
```

</Example>

Format:

| Name      | Type            | Required | Description     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | Yes   | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
  "time": 1773382659,
  "channel": "funding_rate",
  "event": "update",
  "result": {
    "s": "BINANCE_FUTURE_BTC_USDT",
    "r": "0.00002534",
    "T": 1773388800000
  },
  "time_ms": 1773382659999
}
```

</Example>

Push parameters:

| Name        | Type     | Required    |Constraint| Description        | Remarks              |
|-----------|--------|-------|---|-----------|-----------------|
| » channel | string | true  |none| Channel name       | none            |
| » event   | string | true  |none| Event        | none            |
| » result  | object | true  |none| Result        | none            |
| »» s      | string | true  |none| Trading pair       | Symbol          |
| »» r      | string | true  |none| Current funding rate    | FundingRate     |
| »» T      | long   | true  |none| Next funding time (milliseconds) | NextFundingTime |

</Block>

<Block>

# Futures Open Interest Subscription

`open_interest`

The open_interest channel provides open interest updates for futures markets from major exchanges (excluding BINANCE)

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters


|Name|Location|Type|Required| Description               |
|---|---|---|---|------------------|
|» time|body|integer| Yes | Timestamp (seconds)           |
|» event|body|string| Yes | Event               |
|» channel|body|string| Yes | Channel name              |
|» payload|body|[string]| Yes | Subscribe/Unsubscribe trading pairs, subscription to all is not supported yet |

### Details

**» event**: subscribe, unsubscribe

**» channel**: open_interest

<Example>

Request example

```json
{
  "event": "subscribe",
  "channel": "open_interest",
  "payload": [
    "OKX_FUTURE_BTC_USDT"
  ]
}
```

</Example>

Format:

| Name      | Type            | Required | Description     |
| --------- | --------------- | ---- | -------- |
| `payload` | `Array[String]` | Yes   | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

::: warning
No authentication required
:::

</Block>

<Block>

## Server Push

<Example>

Push example

```json
{
  "time": 1773384822,
  "channel": "open_interest",
  "event": "update",
  "result": {
    "s": "OKX_FUTURE_BTC_USDT",
    "oi": "2929823.88000001813",
    "oiV": "29298.2388000001813"
  },
  "time_ms": 1773384822164
}
```

</Example>

Push parameters:

| Name       | Type     | Required   |Constraint| Description        | Remarks                |
|----------|--------|------|---|-----------|-------------------|
| » channel | string | true |none| Channel name       | none              |
| » event  | string | true |none| Event        | none              |
| » result | object | true |none| Result        | none              |
| »» s     | string | true |none| Trading pair       | Symbol            |
| »» oi    | string | true |none| Open interest       | OpenInterest      |
| »» oiV   | string | false |none| Open interest value | OpenInterestValue |

</Block>

<Block>

# Order Subscription/Unsubscription

`order`

The order channel provides a way to receive user order updates, including stock and ETF orders. For stocks and ETFs, `exchange_type` is `US/HK/KR/JP`, `business_type` is `STOCK/ETF`, and prices and executed amounts use the native currency of the listing market.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name |
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe trading pairs !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: order

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "order",
  "request_id": "request-1",
  "payload": [
    "GATE_FUTURE_ETH_USDT"
  ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

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
  "channel": "order",
  "event": "subscribe",
  "payload": {
    "attribute": "COMMON",
    "business_type": "FUTURE",
    "create_time": "1756434169096",
    "exchange_type": "OKX",
    "executed_amount": "0",
    "executed_avg_price": "0",
    "executed_qty": "0",
    "fee": "0",
    "fee_coin": "",
    "last_executed_amount": "0",
    "last_executed_price": "0",
    "last_executed_qty": "0",
    "leverage": "3",
    "order_id": "2072652940337152",
    "position_side": "NONE",
    "price": "0.8499",
    "qty": "10",
    "quote_qty": "0",
    "reason": "{\"label\":\"TRADE_INSUFFICIENT_AVAILABLE_MARGIN_ERROR\",\"message\":\"Insufficient availableMargin\"}",
    "reduce_only": "false",
    "side": "BUY",
    "state": "FAIL",
    "symbol": "OKX_FUTURE_ADA_USDT",
    "text": "2072652940337152",
    "time_in_force": "GTC",
    "type": "MARKET",
    "update_time": "1756434169098",
    "user_id": "2124485957"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168968
}
```

</Example>

Push parameters:

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|true|none|User ID|none|
|»» order_id|string|true|none|Order ID|Order ID|
|»» text|string|true|none|Customer order ID|Customer-defined order ID|
|»» state|string|true|none|Order state|Order State: <br />NEW: The order is legal and waiting to be sent to the exchange<br />OPEN: The order has been placed on the orderbook of the exchange<br />PARTIALLY_FILLED: The order has been partially completed<br />FILLED: The order has been fully executed<br />FAIL: The order verification in CrossEx did not pass. Please check the order reason<br />REJECT：The order was rejected by the exchange. Please check the order reason|
|»» symbol|string|true|none|Trading pair|Trading pair unique identifier ,example:<br />BINANCE_SPOT_BTC_USDT, BINANCE_FUTURE_BTC_USDT|
|»» side|string|true|none|Direction|Side(BUY,SELL)|
|»» type|string|true|none|Type|Type(LIMIT, MARKET)|
|»» attribute|string|true|none|Attribute|Attribute(COMMON regular order / LIQ liquidation takeover order / REDUCE liquidation position-reduction order / ADL auto-deleveraging order / SETTLEMENT delisting settlement order)|
|»» exchange_type|string|true|none|Exchange type|Exchange type(BINANCE,OKX,GATE,BYBIT,KRAKEN,HYPERLIQUID,DERIBIT,LIGHTER)|
|»» business_type|string|true|none|Business type|Business type(SPOT,FUTURE)|
|»» qty|string|true|none|Base currency quantity|Order base quantity|
|»» quote_qty|string|true|none|Quote currency quantity|Order quote quantity|
|»» price|string|true|none|Price|Order price|
|»» time_in_force|string|true|none|Time in force strategy|Timeinforce (default GTC, enums:GTC,IOC,FOK,POC,RPI)|
|»» executed_qty|string|true|none|Executed quantity|Executed quantity|
|»» executed_amount|string|true|none|Executed amount|Executed quote quantity|
|»» executed_avg_price|string|true|none|Executed average price|Average transaction price|
|»» fee_coin|string|true|none|Fee coin|Transaction fee coin|
|»» fee|string|true|none|Fee|Transaction fee amount|
|»» reduce_only|string|true|none|Reduce only|Reduce position orders only, "true" or "false"|
|»» leverage|string|true|none|Leverage|Order leverage|
|»» reason|string|true|none|Reason|Fail message|
|»» last_executed_qty|string|true|none|Last executed quantity|Last transaction quantity|
|»» last_executed_price|string|true|none|Last executed price|Last transaction price|
|»» last_executed_amount|string|true|none|Last executed amount|Last transaction amount|
|»» position_side|string|true|none|Position side|Position side(NONE/LONG/SHORT)|
|»» create_time|string|true|none|Create time|Create time|
|»» update_time|string|true|none|Update time|Update time|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

</Block>

<Block>

# Asset Subscription/Unsubscription

`asset`

The asset channel provides a way to receive user asset updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe coin set !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: asset

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "asset",
  "request_id": "request-1",
  "payload": [
    "USDT"
  ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Coin list |

You can subscribe/unsubscribe multiple times. Coins subscribed earlier will not be overridden unless explicitly unsubscribed to.

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
  "channel": "asset",
  "event": "subscribe",
  "payload": {
    "user_id": "2124492766",
    "coin": "USDT",
    "exchange_type": "CROSSEX",
    "price": "1",
    "balance": "9967.013209",
    "upnl": "0",
    "equity": "9967.013209",
    "futures_initial_margin": "1.236",
    "futures_maintenance_margin": "0",
    "borrowing_initial_margin": "0",
    "borrowing_maintenance_margin": "0",
    "available_balance": "9940.013209",
    "liability": "0"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756465636,
  "time_ms": 1756465636463
}
```

</Example>

Push parameters:

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» coin|string|false|none|Coin|none|
|»» exchange_type|string|false|none|Exchange type|none|
|»» price|string|false|none|Coin price|none|
|»» balance|string|false|none|Balance|none|
|»» upnl|string|false|none|Unrealized P&L|none|
|»» equity|string|false|none|Equity|none|
|»» futures_initial_margin|string|false|none|Futures initial margin|none|
|»» futures_maintenance_margin|string|false|none|Futures maintenance margin|none|
|»» borrowing_initial_margin|string|false|none|Borrowing initial margin for the coin|none|
|»» borrowing_maintenance_margin|string|false|none|Borrowing maintenance margin for the coin|none|
|»» available_balance|string|false|none|Available balance|none|
|»» liability|string|false|none|Liability||
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|»» message|string|true|none|Response message||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

</Block>

<Block>

# Trade Subscription/Unsubscription

`usertrades`

The trade channel provides a way to receive user trade updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name |
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes |Subscribe/Unsubscribe coin set !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: usertrades

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "usertrades",
  "request_id": "request-1",
  "payload": [
    "GATE_FUTURE_ETH_USDT"
  ]
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name <br />usertrades|
|» event|string|true|none|Event<br />subscribe<br />unsubscribe|
|» payload|[string]|true|none|Subscribe/Unsubscribe trading pair set|
|» result|object|true|none|Subscription/Unsubscription result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "usertrades",
  "event": "subscribe",
  "payload": [
    "OKX_FUTURE_ADA_USDT"
  ],
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168928
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

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
  "channel": "usertrades",
  "event": "subscribe",
  "payload": {
    "user_id": "2124492766",
    "transaction_id": "2072784922594048",
    "order_id": "2072784922592768",
    "text": "2072784922592768",
    "symbol": "GATE_SPOT_ADA_USDT",
    "exchange_type": "GATE",
    "business_type": "SPOT",
    "side": "BUY",
    "qty": "13.36",
    "price": "0.8228",
    "fee": "0.004008000000000000",
    "fee_coin": "ADA",
    "fee_rate": "0.0003",
    "match_role": "TAKER",
    "rpnl": "0",
    "position_mode": "SINGLE",
    "position_side": "",
    "create_time": "1756465636528"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168984
}
```

</Example>

Push parameters:

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» transaction_id|string|false|none|Trade record ID|none|
|»» order_id|string|false|none|Order ID|none|
|»» text|string|false|none|User order ID|none|
|»» symbol|string|false|none|Trading pair|none|
|»» exchange_type|string|false|none|Exchange type|none|
|»» business_type|string|false|none|Business type|none|
|»» side|string|false|none|Buy/Sell direction|none|
|»» qty|string|false|none|Trade quantity|none|
|»» price|string|false|none|Trade price|none|
|»» fee|string|false|none|Fee|none|
|»» fee_coin|string|false|none|Fee coin|none|
|»» fee_rate|string|false|none|Fee rate|none|
|»» match_role|string|false|none|Trade role|none|
|»» rpnl|string|false|none|Realized P&L|none|
|»» position_mode|string|false|none|Position mode|none|
|»» position_side|string|false|none|Position side|none|
|»» create_time|string|false|none|Create time|none|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

</Block>

<Block>

# Position Subscription/Unsubscription

`position`

The position channel provides a way to receive user position updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes | !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: position

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "position",
  "request_id": "request-1",
  "payload": [
    "GATE_FUTURE_ETH_USDT"
  ]
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />position|
|» event|string|true|none|Event<br />subscribe<br />unsubscribe|
|» payload|[string]|true|none|Subscribe/Unsubscribe trading pair set|
|» result|object|true|none|Subscription/Unsubscription result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "position",
  "event": "subscribe",
  "payload": [
    "OKX_FUTURE_ADA_USDT"
  ],
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168929
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list, `!all` means subscribe to all trading pairs |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed to.

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
	"channel": "position",
	"event": "subscribe",
	"payload": {
		"create_time": "1756434169096",
		"entry_price": "0",
		"fee": "0",
		"funding_fee": "0",
		"funding_time": "0",
		"initial_margin": "0",
		"isolated_margin": "0",
		"margin_mode": "CROSS",
		"leverage": "3",
		"maintenance_margin": "0",
		"liq_price": "0",
		"mark_price": "0.8499",
		"max_leverage": "20",
		"position_id": "20087051449819136",
		"position_qty": "0",
		"position_side": "NONE",
		"position_value": "0",
		"risk_limit": "1",
		"symbol": "OKX_FUTURE_ADA_USDT",
		"update_time": "1756434169098",
		"upnl": "0",
		"upnl_rate": "0",
		"user_id": "2124485957",
		"closed_pnl": "12"
	},
	"result": {
		"code": "100000",
		"message": "success"
	},
	"time": 1756434168,
	"time_ms": 1756434168984
}
```

</Example>

Push parameters:

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» position_id|string|false|none|Position ID|none|
|»» symbol|string|false|none|Trading pair|none|
|»» position_side|string|false|none|Position side|none|
|»» initial_margin|string|false|none|Initial margin|none|
|»» isolated_margin|string|false|none|Isolated margin. Returns `0` in cross margin mode; applicable only in isolated margin mode|none|
|»» margin_mode|string|false|none|Margin mode (`CROSS` or `ISOLATED`)|none|
|»» maintenance_margin|string|false|none|Maintenance margin|none|
|»» position_qty|string|false|none|Position quantity|none|
|»» position_value|string|false|none|Position value|none|
|»» upnl|string|false|none|Unrealized P&L|none|
|»» upnl_rate|string|false|none|Unrealized P&L rate|none|
|»» entry_price|string|false|none|Position entry average price|none|
|»» liq_price|string|false|none|Liquidation price. Returns `0` in cross margin mode; applicable only in isolated margin mode. In isolated margin mode, `0` indicates that the position has no liquidation price|none|
|»» mark_price|string|false|none|Mark price|none|
|»» leverage|string|false|none|Position leverage|none|
|»» max_leverage|string|false|none|Maximum leverage|none|
|»» risk_limit|string|false|none|Risk limit|none|
|»» fee|string|false|none|Position fee|none|
|»» funding_fee|string|false|none|Position funding fee|none|
|»» funding_time|string|false|none|Position funding fee collection time|none|
|»» create_time|string|false|none|Position create time|none|
|»» update_time|string|false|none|Position update time|none|
|»» closed_pnl|string|false|none|Realized P&L|none|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

</Block>

<Block>

# Security Position Subscription/Unsubscription

`security_position`

The security position channel provides stock and ETF position updates. Prices, position values, and unrealized PnL are denominated in the native currency of the listing market (USD/HKD/KRW/JPY).

**Push type**: `continuous`

**Update frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

| Name           | Location   | Type       | Required | Description                            |
|--------------|------|----------|----|-------------------------------|
| » time       | body | integer  | Yes  | Timestamp (seconds)                        |
| » event      | body | string   | Yes  | subscribe, unsubscribe          |
| » channel    | body | string   | Yes  | security_position              |
| » request_id | body | string   | No  | Request ID echoed by the server to help identify the response         |
| » payload    | body | [string] | Yes  | List of stock or ETF trading pairs; `!all` means all pairs       |

<Example>

Request example

```json
{
  "time": 1791777600,
  "event": "subscribe",
  "channel": "security_position",
  "request_id": "request-1",
  "payload": [
    "HK_STOCK_00700_USDT"
  ]
}
```

</Example>

You can subscribe or unsubscribe multiple times. Previously subscribed trading pairs will not be overwritten unless explicitly unsubscribed.

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
  "channel": "security_position",
  "event": "subscribe",
  "payload": {
    "user_id": "59138103",
    "symbol": "HK_STOCK_00700_USDT",
    "price": "424.8",
    "entry_price": "428",
    "diluted_cost_price": "428",
    "upnl": "-0.032",
    "position_qty": "0.01",
    "position_value": "4.248"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1791777600,
  "time_ms": 1791777600000
}
```

</Example>

Push parameters:

| Name                       | Type      | Required    | Description                         |
|--------------------------|---------|-------|----------------------------|
| » channel                | string  | true  | Channel name                        |
| » event                  | string  | true  | Event                         |
| » payload                | object  | true  | Security position data                    |
| »» user_id               | string  | false | User ID                       |
| »» symbol                | string  | false | Stock or ETF trading pair                  |
| »» price                 | string  | false | Latest price in the native currency of the listing market        |
| »» entry_price           | string  | false | Position cost price in the native currency of the listing market       |
| »» diluted_cost_price    | string  | false | Diluted cost price in the native currency of the listing market       |
| »» upnl                  | string  | false | Unrealized PnL in the native currency of the listing market        |
| »» position_qty          | string  | false | Position quantity                       |
| »» position_value        | string  | false | Position value in the native currency of the listing market        |
| » result                 | object  | true  | Subscription response                       |
| »» code                  | string  | true  | Return code                        |
| » time                   | integer | true  | Timestamp (seconds)                    |
| » time_ms                | integer | true  | Timestamp (milliseconds)                   |

</Block>

<Block>

# Margin Position Subscription/Unsubscription

`margin_position`

The margin position channel provides a way to receive user margin position updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes | !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: margin_position

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "margin_position",
  "request_id": "request-1",
  "payload": [
    "GATE_MARGIN_ETH_USDT"
  ]
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />position|
|» event|string|true|none|Event<br />subscribe<br />unsubscribe|
|» payload|[string]|true|none|Subscribe/Unsubscribe trading pair set|
|» result|object|true|none|Subscription/Unsubscription result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "margin_position",
  "event": "subscribe",
  "payload": [
    "GATE_MARGIN_ETH_USDT"
  ],
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168929
}
```

</Example>

## Push Message Response

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» position_id|string|false|none|Position ID|none|
|»» symbol|string|false|none|Trading pair EXCHANGE_MARGIN_BASE_COUNTER|none|
|»» position_side|string|false|none|Position side|none|
|»» initial_margin|string|false|none|Position initial margin|none|
|»» maintenance_margin|string|false|none|Position maintenance margin|none|
|»» asset_qty|string|false|none|Position asset quantity|none|
|»» asset_coin|string|false|none|Position asset coin|none|
|»» position_value|string|false|none|Position value|none|
|»» liability|string|false|none|Liability quantity|none|
|»» liability_coin|string|false|none|Liability coin|none|
|»» interest|string|false|none|Deducted interest|none|
|»» max_position_qty|string|false|none|Maximum position quantity|none|
|»» entry_price|string|false|none|Position cost price (entry average price)|none|
|»» index_price|string|false|none|Index price|none|
|»» upnl|string|false|none|Unrealized P&L|none|
|»» upnl_rate|string|false|none|Unrealized P&L rate|none|
|»» leverage|string|false|none|Entry leverage|none|
|»» max_leverage|string|false|none|Maximum leverage|none|
|»» create_time|string|false|none|Create time|none|
|»» update_time|string|false|none|Update time|none|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

<Example>

Push message response example

```json
{
  "channel": "margin_position",
  "event": "subscribe",
  "payload":{
        "user_id": "20251115",
        "position_id": "20116122196200192",
        "symbol": "OKX_MARGIN_DOGE_USDT",
        "position_side": "LONG",
        "initial_margin": "3.339831063595188875",
        "maintenance_margin": "0.20743774877",
        "asset_qty": "61.72130605",
        "asset_coin": "DOGE",
        "position_value": "9.9969999409185",
        "liability": "10.00003424116",
        "liability_coin": "USDT",
        "interest": "0.0000343",
        "max_position_qty": "61.72130605",
        "entry_price": "0.162018605585874506",
        "index_price": "0.16197",
        "upnl": "-0.0030343002415",
        "upnl_rate": "-0.000909",
        "leverage": "3",
        "max_leverage": "8",
        "create_time": "1763365175843",
        "update_time": "1763365689548"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168984
}
```

</Example>

</Block>

<Block>

# Margin Interest Subscription/Unsubscription

`margin_interest`

The margin interest channel provides a way to receive user margin interest updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes | !all means all|

### Details

**» event**: subscribe、unsubscribe

**» channel**: margin_interest

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "margin_interest",
  "request_id": "request-1",
  "payload": [
    "GATE_MARGIN_ETH_USDT"
  ]
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />position|
|» event|string|true|none|Event<br />subscribe<br />unsubscribe|
|» payload|[string]|true|none|Subscribe/Unsubscribe trading pair set|
|» result|object|true|none|Subscription/Unsubscription result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "margin_interest",
  "event": "subscribe",
  "payload": [
    "GATE_MARGIN_ETH_USDT"
  ],
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168929
}
```

</Example>

## Push Message Response

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» symbol|string|false|none|Trading pair|none|
|»» interest_id|string|false|none|Interest deduction ID|none|
|»» liability_id|string|false|none|Liability source ID, can be order ID/position ID|none|
|»» liability|string|false|none|Liability|none|
|»» liability_coin|string|false|none|Coin|none|
|»» interest|string|false|none|Interest|none|
|»» interest_rate|string|false|none|Interest rate|none|
|»» interest_type|string|false|none|Interest deduction type|none|
|»» create_time|string|false|none|Position create time|none|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

### Details
**» interest_type**: Interest deduction type
PERIODIC_POSITION hourly position interest, PERIODIC_OPEN_ORDER hourly order interest, IMMEDIATE_OPEN_ORDER order interest

<Example>

Push message response example

```json
{
  "channel": "margin_interest",
  "event": "subscribe",
  "payload": {
    "user_id": "20251115",
    "symbol": "OKX_MARGIN_DOGE_USDT",
    "interest_id": "2101724189861376",
    "liability_id": "2101724189861120",
    "liability": "10",
    "liability_coin": "USDT",
    "interest": "0.0000343",
    "interest_rate": "0.00000343",
    "interest_type": "IMMEDIATE_OPEN_ORDER",
    "create_time": "1763365295219"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168984
}
```

</Example>

</Block>

<Block>

# ADL Rank Subscription/Unsubscription

`adl_rank`

The ADL rank channel provides a way to receive user ADL rank updates.

**Push Type**: `continuous`

**Update Frequency**: `real-time`

</Block>

<Block>

## Client Subscription

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|[string]| Yes | !all means all|

### Details

**» event**: subscribe, unsubscribe

**» channel**: adl_rank

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "subscribe",
  "channel": "adl_rank",
  "request_id": "request-1",
  "payload": [
    "BINANCE_FUTURE_ADA_USDT"
  ]
}
```

</Example>

Payload format:

| Name      | Type            | Required | Description   |
| --------- | --------------- | -------- | ------------- |
| `payload` | `Array[String]` | Yes      | Trading pair list, `!all` means subscribe to all trading pairs |

You can subscribe/unsubscribe multiple times. Trading pairs subscribed earlier will not be overridden unless explicitly unsubscribed from.

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
  "channel": "adl_rank",
  "event": "subscribe",
  "payload": {
    "user_id": "20251115",
    "symbol": "BINANCE_FUTURE_ADA_USDT",
    "crossex_adl_rank": "1",
    "exchange_adl_rank": "1"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168984
}
```

</Example>

Push parameters:

|Name|Type|Required|Constraint|Definition|Description|
|---|---|---|---|---|---|
|» channel|string|true|none|Channel name|none|
|» event|string|true|none|Event|none|
|» payload|object|true|none|Data set|none|
|»» user_id|string|false|none|User ID|none|
|»» symbol|string|false|none|Trading pair|none|
|»» crossex_adl_rank|string|false|none|CrossEx ADL rank|1-5; a larger value indicates higher ADL priority|
|»» exchange_adl_rank|string|false|none|Native exchange ADL rank|Binance: 0-4, larger value indicates higher priority; OKX: 0-5, larger value indicates higher priority; Gate: 1-5, smaller value indicates higher priority; Kraken: 20, 40, 80, 100, smaller value indicates higher priority|
|» result|object|true|none|Update time||
|»» code|string|true|none|Return code||
|» time|integer|true|none|Timestamp (seconds)||
|» time_ms|integer|true|none|Timestamp (milliseconds)||

</Block>

<Block>

# Place Order

`place_order`

Provides a way to place orders via WebSocket.

Rate Limit: 100 requests per 10 seconds (shared with REST order placement). Stocks and ETFs also share a per-user order placement rate limit of 1 request per second.

Stocks and ETFs support only cross-exchange account mode and GTC, require `qty`, and do not support `quote_qty`. Order prices use the native currency of the listing market.

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |none|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
| »» header | body | object | No | Request header information |
| »»» X-Gate-Channel-Id | body | string | No | Broker rebate channel ID |
|»» text|body|string¦null| No |User-defined ID|
|»» symbol|body|string| Yes |Unique identifier: Exchange_Business_Base_Counter|
|»» side|body|string| Yes |Trading direction|
|»» qty|body|string¦null| No |Base currency quantity|
|»» price|body|string¦null| No |Price|
|»» type|body|string¦null| No |Order type (default LIMIT, supported types: LIMIT, MARKET)|
|»» time_in_force|body|string¦null| No |Execution method|
|»» quote_qty|body|string¦null| No |Quote currency quantity|
|»» reduce_only|body|boolean¦null| No |Reduce only|
|»» position_side|body|string¦null| No |Position side|

### Details
**» event**: api

**» channel**: place_order

**»» text**: User-defined ID
Customer-defined order ID, only supports letters (a-z), numbers (0-9), symbols (-, _)

**»» symbol**: Unique identifier: Exchange_Business_Base_Counter
Examples:
- If you want to place a SPOT order for ADA/USDT on BINANCE exchange, you can use this unique identifier: `BINANCE_SPOT_ADA_USDT`;  
- If you want to place a USDT-M perpetual FUTURE order for ADA/USDT on OKX exchange, you can use this unique identifier: `OKX_FUTURE_ADA_USDT`;  
- If you want to place a spot MARGIN order for ADA/USDT on GATE exchange, you can use this unique identifier: `GATE_MARGIN_ADA_USDT`;
- If you want to place a SPOT order for ADA/USDT on BYBIT exchange, you can use this unique identifier: `BYBIT_SPOT_ADA_USDT`;
- If you want to place a FUTURE order for ADA/USD on KRAKEN exchange, you can use this unique identifier: `KRAKEN_FUTURE_ADA_USD`;
- If you want to place a FUTURE order for ADA/USDC on HYPERLIQUID exchange, you can use this unique identifier: `HYPERLIQUID_FUTURE_ADA_USDC`;
- If you want to place a FUTURE order for ADA/USDC on DERIBIT exchange, you can use this unique identifier: `DERIBIT_FUTURE_ADA_USDC`;
- If you want to place a FUTURE order for ADA/USDC on LIGHTER exchange, you can use this unique identifier: `LIGHTER_FUTURE_ADA_USDC`;
- To trade US stock AAPL, use the unique identifier: `US_STOCK_AAPL_USDT`;
- To trade Hong Kong stock 00700, use the unique identifier: `HK_STOCK_00700_USDT`;
- To trade US ETF SPY, use the unique identifier: `US_ETF_SPY_USDT`;
- Supported order types include SPOT orders, USDT-M perpetual FUTURE orders, spot MARGIN orders, stock orders, and ETF orders. Stocks and ETFs support cross-exchange account mode only

**»» side**:
Buy BUY
Sell SELL

**»» qty**: Base currency quantity
Order quantity (mandatory, except for spot market buy; qty is required for stocks and ETFs)

**»» price**: Price
Limit order price (required for limit orders). Stocks and ETFs use the native currency of the listing market (USD/HKD/KRW/JPY)

**»» time_in_force**: Execution method
Default GTC, supported types: GTC, IOC, FOK, POC, RPI. Stocks and ETFs support only GTC

**»» quote_qty**: Quote currency quantity
Order quote quantity, only used for spot market buy orders; not supported for stocks and ETFs

**»» reduce_only**: Reduce only
true、false

**»» position_side**: Position side
Not passed for single position. LONG, SHORT
Enum values: LONG  SHORT  NONE

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "place_order",
  "request_id": "request-1",
  "payload": {
    "header":{
      "X-Gate-Channel-Id":"broker-api"
    },
    "text": "test1",
    "symbol": "GATE_FUTURE_ETH_USDT",
    "side": "BUY",
    "qty": "0.01",
    "price": "2800",
    "type": "LIMIT"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />place_order|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|Place order request parameters|
|»» order_id|string|true|none|Order ID|
|»» text|string|true|none|Client ID|
|» result|object|true|none|Place order result|
|»» code|string|true|none|none|
|»» message|string|true|none|none|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "place_order",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "order_id": "2072652940337152",
    "text": "2072652940337152"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Cancel Order

`cancel_order`

Provides a way to cancel orders via WebSocket.

Rate Limit: 100 requests per 10 seconds (shared with REST order placement)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|string| Yes |Order ID (supports order ID or text)|

### Details

**» event**: api

**» channel**: cancel_order

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "cancel_order",
  "request_id": "request-1",
  "payload": "2065058175870464"
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />cancel_order|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» result|object|true|none|Cancel order result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "cancel_order",
  "event": "api",
  "request_id": "request-1",
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Batch Cancel Orders

`batch_cancel_orders`

Provides a way to cancel multiple orders via WebSocket. Each order can be canceled using either an order ID or a user-defined ID. If both are provided, the order ID takes precedence.

Rate Limit: 100 requests per 10 seconds (shared with the REST order cancellation rate limit)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|array| Yes |List of orders to cancel|
|»» order_id|body|string| No |Order ID; either this field or text is required|
|»» text|body|string| No |User-defined ID specified when the order was created; either this field or order_id is required|

### Details

**» event**: api

**» channel**: batch_cancel_orders

<Example>

Request example

```json
{
   "time": 1754272114,
   "event": "api",
   "channel": "batch_cancel_orders",
   "request_id": "request-1",
   "payload": [
      {
         "order_id": "2065058175870464"
      },
      {
         "text": "crossex-test-1"
      }
   ]
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />batch_cancel_orders|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|array|true|none|Batch order cancellation results|
|»» order_id|string|true|none|Order ID|
|»» text|string|true|none|User-defined ID specified when the order was created|
|»» accepted|string|true|none|Whether the request was accepted, as the string true or false|
|»» label|string|true|none|Error label if the request was not accepted; empty on success|
|»» message|string|true|none|Error message if the request was not accepted; empty on success|
|» result|object|true|none|API call result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
   "channel": "batch_cancel_orders",
   "event": "api",
   "request_id": "request-1",
   "payload": [
      {
         "order_id": "2065058175870464",
         "text": "crossex-test-1",
         "accepted": "true",
         "label": "",
         "message": ""
      },
      {
         "order_id": "999999",
         "text": "",
         "accepted": "false",
         "label": "TRADE_ORDER_NOT_FOUND_ERROR",
         "message": "The order was not found"
      }
   ],
   "result": {
      "code": "100000",
      "message": "success"
   },
   "time": 1756434168,
   "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Update Order

`update_order`

Provides a way to update orders via WebSocket.

Rate Limit: 100 requests per 10 seconds (shared with REST order placement)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» order_id|body|string| Yes | Order ID|Supports order ID or text|
|»» qty|body|string| Yes | Quantity|none|
|»» price|body|string| Yes |none|
|»» symbol|body|string| No |none|

### Details

**» event**: api

**» channel**: update_order

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "update_order",
  "request_id": "request-1",
  "payload": {
    "order_id": "2065057827622656",
    "qty": "0.01",
    "symbol": "GATE_SPOT_BTC_USDT",
    "price": "2801"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />update_order|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» order_id|string|true|none|Order ID|
|»» text|string|true|none|Client ID|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "update_order",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "order_id": "2072652940337152",
    "text": "2072652940337152"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Set Leverage

`set_leverage`

Provides a way to modify contract leverage via WebSocket.

Rate Limit: 100 requests per 10 seconds (shared with REST order placement)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» symbol|body|string| Yes |Trading pair|
|»» leverage|body|string| Yes |Leverage|

## Details

**» event**: api

**» channel**: set_leverage

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "set_leverage",
  "request_id": "request-1",
  "payload": {
    "symbol": "GATE_FUTURE_ETH_USDT",
    "leverage": "5"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />set_leverage|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» symbol|string|true|none|Trading pair|
|»» leverage|string|true|none|Leverage|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "set_leverage",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "symbol": "GATE_FUTURE_ETH_USDT",
    "leverage": "10"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Set Margin Leverage

`set_margin_leverage`

Provides a way to modify spot margin leverage via WebSocket.

Rate Limit: 100 requests per 10 seconds (shared with REST order placement)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» symbol|body|string| Yes |Trading pair|
|»» leverage|body|string| Yes |Leverage|

## Details

**» event**: api

**» channel**: set_margin_leverage

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "set_margin_leverage",
  "request_id": "request-1",
  "payload": {
    "symbol": "OKX_MARGIN_ADA_USDT",
    "leverage": "5"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />set_leverage|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» symbol|string|true|none|Trading pair|
|»» leverage|string|true|none|Leverage|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "set_margin_leverage",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "symbol": "OKX_MARGIN_ADA_USDT",
    "leverage": "5"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Update Accounts

`update_accounts`

Provides a way to change account contract position mode and account mode via WebSocket.

Rate Limit: 100 requests per 60 seconds

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» position_mode|body|string| No |Position mode|
|»» account_mode|body|string| No |Account mode|
|»» exchange_type|body|string| No |Exchange, default is cross-exchange|

### Details

**» event**: api

**» channel**: update_accounts

**»» position_mode**: SINGLE/DUAL

**»» account_mode**: CROSS_EXCHANGE/ISOLATED_EXCHANGE

**»» exchange_type**: BINANCE/OKX/GATE/BYBIT/KRAKEN/HYPERLIQUID/DERIBIT/LIGHTER/CROSSEX, when account mode is ISOLATED_EXCHANGE, exchange must be specified when modifying contract position mode

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "update_accounts",
  "request_id": "request-1",
  "payload": {
    "account_mode":"ISOLATED_EXCHANGE",
    "position_mode": "DUAL",
    "exchange_type": "BINANCE"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />update_accounts|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» position_mode|string|false|none|Position mode|
|»» account_mode|string|false|none|Account mode|
|»» exchange_type|string|false|none|Exchange|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "update_accounts",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "account_mode":"ISOLATED_EXCHANGE",
    "position_mode": "DUAL",
    "exchange_type": "BINANCE"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Close Position

`close_position`

Provides a way to close positions completely via WebSocket.

Rate Limit: 100 requests per day

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» symbol|body|string| Yes |Trading pair|
|»» position_side|body|string| No |Position side|

## Details

**» event**: api

**» channel**: close_position

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "close_position",
  "request_id": "request-1",
  "payload": {
    "symbol": "GATE_MARGIN_ETH_USDT",
    "position_side": "LONG"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />close_position|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» order_id|string|true|none|Order ID|
|»» text|string|true|none|Client ID|

<Example>

Response example

```json
{
  "channel": "close_position",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "order_id": "2072652940337152",
    "text": "2072652940337152"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Change Futures Position Margin Mode

`set_margin_mode`

Provides a way to change the margin mode for a futures position via WebSocket.

This operation is supported only for Hyperliquid futures trading pairs. The margin mode cannot be changed while there are open orders or open positions.

Rate Limit: 100 requests per 10 seconds (shared with the corresponding REST API rate limit)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» symbol|body|string| Yes |Hyperliquid futures trading pair|
|»» margin_mode|body|string| Yes |Margin mode (`CROSS` or `ISOLATED`)|

## Details

**» event**: api

**» channel**: set_margin_mode

<Example>

Request example

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "set_margin_mode",
  "request_id": "request-1",
  "payload": {
    "symbol": "HYPERLIQUID_FUTURE_CXMT_USDC",
    "margin_mode": "ISOLATED"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />set_margin_mode|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» symbol|string|true|none|Futures trading pair|
|»» margin_mode|string|true|none|Margin mode (`CROSS` or `ISOLATED`)|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "set_margin_mode",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "symbol": "HYPERLIQUID_FUTURE_CXMT_USDC",
    "margin_mode": "ISOLATED"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Add or Reduce Isolated Margin

`update_isolated_margin`

Provides a way to add or reduce isolated margin via WebSocket.

This operation is supported only for Hyperliquid futures positions in isolated margin mode. A positive amount adds margin, while a negative amount reduces margin.

Rate Limit: 100 requests per 10 seconds (shared with the corresponding REST API rate limit)

</Block>

<Block>

## Client Request

## Request Parameters

|Name|Location|Type|Required|Description|
|---|---|---|---|---|
|» time|body|integer| Yes |Timestamp (seconds)|
|» event|body|string| Yes |Event|
|» channel|body|string| Yes |Channel name|
|» request_id|body|string| No |Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|body|object| Yes |none|
|»» symbol|body|string| Yes |Hyperliquid futures trading pair|
|»» margin|body|string| Yes |Margin adjustment amount. A positive value adds margin, while a negative value reduces margin. Values with more than two decimal places are truncated to two decimal places|
|»» position_side|body|string| No |Position side (`NONE`, `LONG`, or `SHORT`). In one-way mode, it defaults to `NONE` when omitted|

## Details

**» event**: api

**» channel**: update_isolated_margin

<Example>

Request example for reducing isolated margin in one-way mode

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "update_isolated_margin",
  "request_id": "request-1",
  "payload": {
    "symbol": "HYPERLIQUID_FUTURE_CXMT_USDC",
    "margin": "-30",
    "position_side": "NONE"
  }
}
```

Request example for adding isolated margin to a long position in hedge mode

```json
{
  "time": 1754272114,
  "event": "api",
  "channel": "update_isolated_margin",
  "request_id": "request-2",
  "payload": {
    "symbol": "HYPERLIQUID_FUTURE_CXMT_USDC",
    "margin": "30",
    "position_side": "LONG"
  }
}
```

</Example>

## Response Result

|Name|Type|Required|Constraint|Description|
|---|---|---|---|---|
|» channel|string|true|none|Channel name<br />update_isolated_margin|
|» event|string|true|none|Event<br />api|
|» request_id|string|false|none|Request id which will be sent back by the server to help you identify which request the server responds to|
|» payload|object|true|none|none|
|»» symbol|string|true|none|Futures trading pair|
|»» margin|string|true|none|Isolated margin added or reduced by this request|
|»» position_side|string|true|none|Position side|
|» result|object|true|none|Update result|
|»» code|string|true|none|Response code|
|»» message|string|true|none|Response message|
|» time|integer|true|none|Timestamp (seconds)|
|» time_ms|integer|true|none|Timestamp (milliseconds)|

<Example>

Response example

```json
{
  "channel": "update_isolated_margin",
  "event": "api",
  "request_id": "request-1",
  "payload": {
    "symbol": "HYPERLIQUID_FUTURE_CXMT_USDC",
    "margin": "-30",
    "position_side": "NONE"
  },
  "result": {
    "code": "100000",
    "message": "success"
  },
  "time": 1756434168,
  "time_ms": 1756434168948
}
```

</Example>

</Block>

<Block>

# Response Codes

## `code` Description

|Response Code|Description|
|---|---|
|100000|Success|
|100001|Parameter missing or invalid|
|100002|Not logged in|
|100003|Connection limit reached|
|100004|Timeout|
|100005|Operation failed|
|100006|Login failed|
|100007|Network error, please try again later|

</Block>

<Block>

## `label` Description

For invalid requests, the response `result` includes a `label`. Its value has the same meaning as the corresponding REST API error label. See the REST API documentation for details.

<Example>

Response example with `label`

```json
{
  "time": 1787560434,
  "channel": "not_exist",
  "event": "api",
  "result": {
    "code": "100001",
    "label": "COMMON_PARAM_BIND_ERROR",
    "message": "Invalid Parameter, please check api doc"
  },
  "time_ms": 1787560400000,
  "request_id": "xxx"
}
```

</Example>

</Block>
