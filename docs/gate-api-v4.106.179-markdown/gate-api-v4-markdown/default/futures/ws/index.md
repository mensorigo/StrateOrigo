---
title: Futures WebSocket

language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>

# Gate Futures WebSocket v4.0.0

Gate provides a simple and robust WebSocket API to integrate trading data from Gate's BTC-, USDT-, and
USD1-settled futures contracts into your business or application.

We have language bindings in `Python` and `Golang` , more in the future! You can view code
examples in the dark area to the right, and you can switch the programming language of the examples
with the tabs in the top right.

## Server URL

We provide BTC-, USDT-, and USD1-settled futures contract trade server URLs. You can choose one according to your
condition.

### BTC Contract

Base URLs:

- Real Trading: `wss://fx-ws.gateio.ws/v4/ws/btc`
- TestNet Trading: `wss://fx-ws-testnet.gateio.ws/v4/ws/btc`

### USDT Contract

Base URLs:

- Real Trading: `wss://fx-ws.gateio.ws/v4/ws/usdt`
- TestNet: `wss://ws-testnet.gate.com/v4/ws/futures/usdt`

### USD1 Contract

Base URLs:

- Real Trading: `wss://fx-ws.gateio.ws/v4/ws/usd1`
- Contract example: `BTC_USD1`

::: warning
If you use old server urls(`wss://fx-ws.gateio.ws/v4/ws`
or `wss://fx-ws-testnet.gateio.ws/v4/ws`), we will use BTC contract for you.
:::

</Block>

<Block>

## Changelog

<Example>

Demo WebSocket application

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

Demo WebSocket application

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
	u := url.URL{Scheme: "wss", Host: "fx-ws.gateio.ws", Path: "/v4/ws/usdt"}
	websocket.DefaultDialer.TLSClientConfig = &tls.Config{RootCAs: nil, InsecureSkipVerify: true}
	c, _, err := websocket.DefaultDialer.Dial(u.String(),  http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

- Clarified the purpose of `futures.adl_warning`, the meanings of its three market-level risk states, and their relationship to individual positions and actual ADL events.

2026-10-16

- Added `futures.orders_v2` for live trading and TestNet, available only to UTA3.0 users of the USDT-settled futures API WebSocket. The JSON channel pushes order creation, fill, amendment, and cancellation updates with Order Management System (OMS) and Matching Engine (ME) millisecond timestamps, without clearing information.

2026-08-25

- Added the public `futures.adl_warning` channel for market-level ADL risk warnings

2026-08-19

- Added USD1-settled futures WebSocket support at `wss://fx-ws.gateio.ws/v4/ws/usd1`; use the lowercase `usd1` value for `settle`

2025-01-07

- Added field `market_order_slip_ratio` to `futures.orders` (preset slippage ratio for market orders)
- `futures.order_place` `futures.order_batch_place` added field `market_order_slip_ratio` (allows maximum slippage for custom market orders)

2025-12-09

- The number of contracts supports decimals, and all pushed contracts, trading volume, etc. are changed to strings (possibly decimals).
- How to connect to websocket decimal support:

- When requesting a link to a websocket again, add the header of `"X-Gate-Size-Decimal": "1"`, and the number of shares, trading volume, etc. will be a string (possibly a decimal).
    - If you do not add the header of `"X-Gate-Size-Decimal": "1"` when linking to a websocket, the push will keep the original field type (int).
    - `Please switch to string push support as soon as possible, and the subsequent shaping push will be offline (the offline time will be notified separately).`
    - If there is a decimal size, if the user is still using the int push, then the pushed size will be rounded down, for example, the size of `1.1`, `1.5`, and `1.7` will be `1`.
- The following push channels and corresponding fields will be affected.

| channel                   | field                                             |
  |---------------------------| ------------------------------------------------ |
| futures.trades            | size                                             |
| futures.tickers           | total\_size                                      |
| futures.book_ticker       | A B                                              |
| futures.order_book_update | a.s、b.s                                         |
| futures.order_book        | a.s、b.s                                         |
| futures.obu               | size                                    |
| futures.candlesticks      | v                                                |
| futures.liquidates        | left、size、 order_size                          |
| futures.public_liquidates | size                                             |
| futures.contract_stats    | long\_liq\_size、short\_liq\_size、open\_interest |
| futures.orders            | iceberg、left、size                              |
| futures.usertrades        | size                                             |
| futures.auto_deleverages  | position\_size、trade\_size                      |
| futures.positions         | size                                             |
| futures.autoorders        | position\_size、 trade\_size                     |

- The following API request channels will be affected, mainly involving fields such as the number of contracts and trading volume.

| channel                      | field        |
| ------------------------- |-----------|
|futures.order_place           | size、left |
| futures.order_batch_place          | size      |
| futures.order_cancel       | size、left     |
| futures.order_cancel_cp | size、left   |
| futures.order_amend       | size、left     |
| futures.order_list | size、left   |
| futures.order_status | size、left   |


2025-09-25

- Add `Positions ADL API` documentation

2025-05-22

- Added documentation for the Order Book V2 API
- Added a new field `full` to `futures.order_book_update`

2025-04-25

- Futures Account Trade has added a new `futures.order_cancel_ids` channel.
- `futures.order_book` and `futures.order_book_update` have added a new depth level field `l`.

2025-04-18

- Add additional code examples to the documentation.

<!-- 2025-04-14

- Futures Account Trade has added a new `futures.order_cancel_ids` channel.

2025-04-10

- Added description of `timeUnit=microsecond` in the WebSocket API overview. -->

2025-03-24

- Fixed the incorrect descriptions in some documents of the order_book channel.
- Fixed the incorrect descriptions in some documents of the orders channel.

2025-03-21

- The documentation for channel `futures.orders` has been updated, with new explanations added for fields such as `update_id`, `update_time`, `biz_info`,	`stop_profit_price`, `stop_loss_price`.

2025-03-12

- Add new field `contract` in channel `contract_stats`
- Updated the Futures Account Trade module and added the `x-gate-exptime` field
- Fixed some descriptive errors in the Futures Account Trade documentation

2025-02-19

- Added channel `futures.public_liquidates` to push liquidate orders snapshot

2025-02-10

- Updated the Futures Account Trade module and added `x_in_time`, `x_out_time`, `conn_trace_id`, `trace_id` fields
- `futures.order_place`, `futures.order_batch_place`, `futures.order_cancel`, `futures.order_cancel_cp` and `futures.order_amend` added `x_gate_ratelimit_requests_remain`, `x_gate_ratelimit_limit` and `x_gat_ratelimit_reset_timestamp` fields

2024-11-18

- remove `10` level and `1000ms` interval in channel `futures.order_book_update`

2023-09-21

- Add new field `is_internal` in channel `futures.trades`

2023-08-18

- Add WebSocket API
- WebSocket API allows placing, canceling, amending, querying orders through a WebSocket connection.

2023-07-07

- Add new interval `20ms` in channel `futures.order_book_update`, please note that the interval of `20ms` is only supported for `20` levels.

2023-06-20

- Add `update_id` of `futures.positions`

2022-12-22

- Add new field `auto_size` in `futures.autoorders` initial struct, field detail to http api

2022-11-22

- Add new field `time_ms` in common msg response for time of message created

2022-08-11

- Add new field `text` in user trades notification
- Add new fields `low_24h` and `high_24h` in tickers notification

2022-04-15

- Add new field `currency` in balance notification

2021-03-31

- Add milliseconds field `t` in `futures.book_ticker` and `futures.order_book` response

2021-03-10

- Add new order book channel `futures.book_ticker` to push best ask/bid in realtime
- Add new order book channel `futures.order_book_update` to push order book change with user
  specified update frequency
- Add local order book maintenance documentation

2021-03-01

- Add new millisecond-precision timestamp ended with `_ms` in server notification.
- Add new field `id` in order book `all` notification.

2020-8-08

- Add a complete code demo(golang, python)

2020-8-07

- Add auto orders subscription

2020-7-07

- Add order book annotation

2020-4-30

- Add positions subscription

2019-11-06

- Add USDT futures contract
- Add volume_24h_base field, volume_24h_settle field, volume_24h_quote field for tickers
- Remove old server urls(`wss://fx-ws.gateio.ws/v4/ws` or `wss://fx-ws-testnet.gateio.ws/v4/ws`)

::: tip
If you use old server urls(`wss://fx-ws.gateio.ws/v4/ws` or `wss://fx-ws-testnet.gateio.ws/v4/ws`), we
will use BTC contract for you.
:::

2019-10-22

- Add application layer ping/pong message

2019-04-30

- Add index and mark candlestick subscription
- Add funding_rate_indicative field for tickers
- Add is_reduce_only and status field for orders

2019-02-13

- Change WebSocket base url
- Add volume_24h_usd field and volume_24h_btc field for tickers

2019-01-11

- Add position_closes and balances subscription
- Del finish_time field for auto_deleverages and liquidates
- Add time field for auto_deleverages and liquidates

</Block>

<Block>

## API Overview

### Method

Each general api (such as ticker, order book etc.) supports some different event messages, they are:

1. **`subscribe`** (**RECOMMENDED TO USE**)

   Subscribe to receive notification from server when new data is available.

2. **`unsubscribe`**

   Server will not send new data notification if unsubscribed.

3. **`update`**

   If new subscribed data(incremental data) is available, server will send a notification to client.

4. **`all`**

   If new subscribed data(all data) is available, server will send a notification to client.

### Request

Each request follows a common format, which contains `time`, `channel`, `event` and `payload`.

| parameter | type    | required | description                                                                                                         |
| --------- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------- |
| `id`      | Integer | No       | Optional request id which will be sent back by the server to help you identify which request the server responds to |
| `time`    | Integer | Yes      | Request time                                                                                                        |
| `channel` | String  | Yes      | Request subscribe/unsubscribe channel                                                                               |
| `auth`    | String  | No       | Request auth info, see Authentication section for details                                                           |
| `event`   | String  | Yes      | Request event (subscribe/unsubscribe/update/all/api)                                                                |
| `payload` | Array   | Yes      | Request detail parameters                                                                                           |

### Response

Similar with request, response follows a common format composed of `time`, `channel`, `event`
, `error` and `result`.

| field     | type    | required | description                         |
| --------- | ------- | -------- | ----------------------------------- |
| `time`    | Integer | Yes      | Response time                       |
| `time_ms` | Integer | Yes      | Response time of millisecond        |
| `channel` | String  | Yes      | Response channel                    |
| `event`   | String  | Yes      | Response channel event (update/all) |
| `error`   | Object  | Yes      | Response error                      |
| `result`  | Any     | Yes      | New data notification from the server, or response to client requests. Null if`error` is not null.   |

Note: type of `result` is channel specific if it's server-initiated data update notification, but
response to client subscription request always set the `result` to `{"status": "success"}`. To
verify if subscription request is successful or not, you only need to check if `error` field is
null. Parsing `result` field is not necessary.

<!-- To obtain a microsecond-level response time, add the parameter `timeUnit=microsecond` to the URL. The response will include an additional `time_us` field, which provides a timestamp with microsecond precision.
This feature is not supported in channels under the `Futures Account Trade` module, as a `x_out_time` field with microsecond precision is already included by default.

URL example: `wss://fx-ws.gateio.ws/v4/ws/usdt?timeUnit=microsecond`. -->

Channel specific description below will only give the server-initiated data update notification
format for simplicity.

### Error

In case of error, you receive a message containing the proper error code and message within an error
object.

| Code | Message                   |
| ---- | ------------------------- |
| `1`  | `invalid argument struct` |
| `2`  | `invalid argument`        |
| `3`  | `service error`           |
| `4`  | `authentication fail`     |

</Block>

<Block>

## Authentication

Request body needs to carry authentication information if channels are private,
e.g. `futures.usertrades`

WebSocket authentication uses the same signature calculation method with HTTP API, but has the
following differences:

1. Signature string concatenation method: `channel=<channel>&event=<event>&time=<time>`,
   where `<channel>`, `<event>`, `<time>` are corresponding request information
2. Authentication information are sent in request body in field `auth`.

<Example>

Code example

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

Code example

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

You can log into the console to retrieve futures API key and secret.

| field    | type   | description             |
| -------- | ------ | ----------------------- |
| `method` | String | Allowed value:`api_key` |
| `KEY`    | String | User key string         |
| `SIGN`   | String | User sign string        |

</Block>

<Block>

# System API

**Provides system status check, such as ping-pong.**

</Block>

<Block>

## Ping and Pong

**Check Server/Client connectivity.**

**Gate futures contract use the protocol layer ping/pong message.The server will initiate a ping
message actively. If the client does not reply, the client will be disconnected.**

[websocket rfc](https://tools.ietf.org/html/rfc6455)

**if you want to actively detect the connection status, you can send application layer ping message
and receive pong message.**

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.ping"}')
print(ws.recv())
```

</Example>

<Example>

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
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

The above command returns JSON structured like this:

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

### Request

- channel

  `futures.ping`

</Block>

<Block>

## Service upgrade notification

When the service is about to shut down for an upgrade, it will push a system notification to connected clients. Clients should reconnect as soon as possible after receiving it.

**Channel:** `futures.system`

**Event:** `update`

Clients can identify a service upgrade notification when `channel` is `futures.system`, `event` is `update`, and `result.type` is `upgrade`.

**`result` format (SystemNotifyDTO):**

| Field  | Type   | Description                        |
| ------ | ------ | ---------------------------------- |
| `type` | String | Notification type, e.g. `upgrade` |
| `msg`  | String | Message text                       |
| `data` | Object | Optional, extra data               |

**Example (service upgrade):**

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

# Tickers API

**The ticker is a high level overview of the state of the contract. It shows you the highest,
lowest, last trade price. It also includes information such as daily volume and how much the price
has moved over the last day.**

</Block>

<Block>

## Tickers subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.tickers","event": "subscribe", "payload" : ["BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
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

The above command returns JSON structured like this:

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

**Subscribe futures contract ticker.**

### Request

- channel

  `futures.tickers`

- event

  `subscribe`

- params

  | parameter | type  | required | description   |
    | --------- | ----- | -------- | ------------- |
  | `payload` | Array | Yes      | Contract list |

</Block>

<Block>

## Tickers notification

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
      "volume_24h_base": "5526",
      "low_24h": "99.2",
      "high_24h": "132.5"
    }
  ]
}
```

</Example>

**Notify subscribed contract ticker.**

### Notify

- channel

  `futures.tickers`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field                     | type   | description                                                                                                            |
    | ------------------------- | ------ | ---------------------------------------------------------------------------------------------------------------------- |
  | `contract`                | String | Futures contract name                                                                                                  |
  | `last`                    | String | Last price                                                                                                             |
  | `change_percentage`       | String | Change percentage                                                                                                      |
  | `funding_rate`            | String | Funding rate                                                                                                           |
  | `funding_rate_indicative` | String | Indicative Funding rate in next period. (deprecated. use funding_rate)                                                                                |
  | `mark_price`              | String | Recent mark price                                                                                                      |
  | `index_price`             | String | Index price                                                                                                            |
  | `total_size`              | String | Total size                                                                                                             |
  | `volume_24h`              | String | Volume 24h                                                                                                             |
  | `quanto_base_rate`        | String | Exchange rate of base currency and settlement currency in Quanto contract. Does not exists in contracts of other types |
  | `volume_24h_btc`          | String | Trade volumes in recent 24h in BTC(deprecated, use`volume_24h_base`, `volume_24h_quote`, `volume_24h_settle` instead)  |
  | `volume_24h_usd`          | String | Trade volumes in recent 24h in USD(deprecated, use`volume_24h_base`, `volume_24h_quote`, `volume_24h_settle` instead)  |
  | `volume_24h_quote`        | String | Trade volume in recent 24h, in quote currency                                                                          |
  | `volume_24h_settle`       | String | Trade volume in recent 24h, in settle currency                                                                         |
  | `volume_24h_base`         | String | Trade volume in recent 24h, in base currency                                                                           |
  | `low_24h`                 | String | Lowest trading price in recent 24h                                                                                     |
  | `high_24h`                | String | Highest trading price in recent 24h                                                                                    |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
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

Code example

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe contract ticker.**

### Request

- channel

  `futures.tickers`

- event

  `unsubscribe`

</Block>

<Block>

# Trades API

**This channel sends a trade message whenever a trade occurs at Gate. It includes details of the
trade, such as price, amount, time and type.**

</Block>

<Block>

## Trades subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.trades","event": "subscribe", "payload" : ["BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe trades update notification.**

### Request

- channel

  `futures.trades`

- event

  `subscribe`

- params

  | parameter | type  | required | description   |
    | --------- | ----- | -------- | ------------- |
  | `payload` | Array | Yes      | Contract list |

</Block>

<Block>

## Trades notification

<Example>

Positive size means taker is buyer，negative seller

```json
{
  "channel": "futures.trades",
  "event": "update",
  "time": 1541503698,
  "time_ms": 1541503698123,
  "result": [
    {
      "size": "-108",
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

**Notify latest trades update.**

### Notify

- channel

  `futures.trades`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field            | type       | description                                                                                                                                                                                                                                                                                                                    |
    | ---------------- |------------| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
  | `contract`       | string     | Futures contract name                                                                                                                                                                                                                                                                                                          |
  | `size`           | string/int | Trades size                                                                                                                                                                                                                                                                                                                    |
  | `id`             | int        | Trades id                                                                                                                                                                                                                                                                                                                      |
  | `create_time`    | int        | Trades msg create time                                                                                                                                                                                                                                                                                                         |
  | `create_time_ms` | int        | Trades msg create time in milliseconds                                                                                                                                                                                                                                                                                         |
  | `price`          | string     | Trades price                                                                                                                                                                                                                                                                                                                   |
  | `is_internal`    | bool       | Whether internal trade. Internal trade refers to the takeover of liquidation orders by the insurance fund and ADL users. Since it is not a normal matching on the market depth, the transaction price may deviate, and it will not be recorded in the K-line. If it is not an internal trade, this field will not be returned. |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send(
  '{"time" : 123456, "channel" : "futures.trades", "event": "subscribe", "payload" : ["BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.trades","event":"subscribe","payload":["BTC_USD"]}`

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

The above command returns JSON structured like this:

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

**Unsubscribe trades update notification.**

### Request

- channel

  `futures.trades`

- event

  `unsubscribe`

</Block>

<Block>

# Order Book API

**The order_book channel allow you to keep track of the state of the Gate order book depth. It is
provided on a price aggregated basis, with customizable precision.**

There are three different order book channels for subscription:

- `futures.order_book`

  Legacy channel, which uses `all` to push full limited-level order book, and `update` to send every
  order book change event.

- `futures.book_ticker`

  Push best bid and ask in real-time.

- `futures.order_book_update`

  Push order book change with user-specified update frequency.

::: warning
Receiving order book update through `futures.order_book` is not recommended
to use. `futures.order_book_update` can provide more timely update with less traffic
:::

How to maintain local order book:

1. Subscribe `futures.order_book_update` with specified level and update frequency, e.g.
   `["BTC_USDT", "100ms", "100"]` pushes the first 100 levels' update in BTC_USDT order book every 100ms
2. Cache WebSocket notifications. Every notification use `U` and `u` to tell the first and last
   update ID since last notification.
3. Retrieve base order book using REST API, and make sure the order book ID is recorded(referred
   as `baseID` below)
   e.g. `https://api.gateio.ws/api/v4/futures/usdt/order_book?contract=BTC_USDT&limit=10&with_id=true`
   retrieves the 10-level base order book of BTC_USDT
4. Iterate the cached WebSocket notifications, and find the first one which contains the baseID,
   i.e. `U <= baseId+1` and `u >= baseId+1`, then start consuming from it. Note that sizes in
   notifications are all absolute values. Use them to replace original sizes in corresponding price.
   If size equals to 0, delete the price from the order book.
5. Dump all notifications which satisfy `u < baseID+1`. If `baseID+1 < first notification U`, it
   means current base order book falls behind notifications. Start from step 3 to retrieve newer
   base order book.
6. If any subsequent notification which satisfy `U > baseID+1` is found, it means some updates are
   lost. Reconstruct local order book from step 3.

Note:

* Even if the `a`, `b`, or `asks`, `bids` fields in the WebSocket push message are empty, users must still update the local order book's `id` and `timestamp` to ensure consistency with the server's state.

* The subscribed `level` (depth) in WebSocket should match the `limit` parameter in the REST snapshot. A mismatch may result in incremental updates failing to fully cover the snapshot levels, leading to misaligned or incomplete local order books.

</Block>

<Block>

## Legacy order book subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.order_book","event": "subscribe", "payload" : ["BTC_USD", "20", "0"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book","event":"subscribe","payload":["BTC_USD","20","0"]}`

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

The above command returns JSON structured like this:

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

**Subscribe order_book.**

### Request

- channel

  `futures.order_book`

- event

  `subscribe`

- params

  | parameter  | type   | required | description                                |
    | ---------- | ------ | -------- | ------------------------------------------ |
  | `contract` | String | Yes      | Contract name                              |
  | `limit`    | String | Yes      | Depth level: 100, 50, 20, 10, 5, 1 |
  | `interval` | String | Yes      | Legal intervals: "0"                       |

</Block>

<Block>

## Legacy order book notification

<Example>

```json
{
  "channel": "futures.order_book",
  "event": "all",
  "time": 1541500161,
  "time_ms": 1541500161123,
  "result": {
    "t": 1541500161123,
    "contract": "BTC_USD",
    "id": 93973511,
    "asks": [
      {
        "p": "97.1",
        "s": "2245"
      },
      {
        "p": "97.1",
        "s": "2245"
      }
    ],
    "bids": [
      {
        "p": "97.1",
        "s": "2245"
      },
      {
        "p": "97.1",
        "s": "2245"
      }
    ],
    "l": "20"
  }
}
```

</Example>

**Notify contract order book update information**

### Notify

- channel

  `futures.order_book`

- event

  `all`

- params

  | field       | type    | description                                     |
    | ----------- | ------- | ----------------------------------------------- |
  | `result`    | object  | Order book info                                 |
  | »`t`        | Integer | Order book generation timestamp in milliseconds |
  | »`contract` | String  | Contract name                                   |
  | »`id`       | Integer | OrderBook id                                    |
  | »`asks`     | Array   | OrderBook ask list                              |
  | »»`p`       | String  | Ask price                                       |
  | »»`s`       | String  | Ask size                                        |
  | »`bids`     | Array   | OrderBook bid List                              |
  | »»`p`       | String  | Bid price                                       |
  | »»`s`       | String  | Bid size                                        |
  | »`l`       | String  | Depth level (For example, 100 represents a depth update of 100 levels)                    |

</Block>

<Block>

## Legacy order book unsubscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.order_book","event": "unsubscribe", "payload" : ["BTC_USD", "20", "0"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.order_book","event":"unsubscribe","payload":["BTC_USD","20","0"]}`

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

The above command returns JSON structured like this:

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

**Unsubscribe specified contract order book.**

### Request

- channel

  `futures.order_book`

- event

  `unsubscribe`

</Block>

<Block>

## Best ask/bid subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.book_ticker","event": "subscribe", "payload" : ["BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe book_ticker.**

### Request

- channel

  `futures.book_ticker`

- event

  `subscribe`

- params

  `payload` is an array contains contracts interested.

</Block>

<Block>

## Best ask/bid notification

If `a` is empty string, it means empty asks; if `b` is empty string, it means empty bids.

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
    "s": "BTC_USD",
    "b": "54696.6",
    "B": "37000",
    "a": "54696.7",
    "A": "47061"
  }
}
```

</Example>

**Notify contract order book best bid and ask**

### Notify

- channel

  `futures.book_ticker`

- event

  `update`

- params

  | field    | type           | description                                     |
    | -------- |----------------| ----------------------------------------------- |
  | `result` | object         | Best bid and ask                                |
  | »`t`     | Integer        | Book ticker generated timestamp in milliseconds |
  | »`u`     | String         | Order book update ID                            |
  | »`s`     | String         | Contract name                                   |
  | »`b`     | String         | Best bid price. If no bids, it's empty string   |
  | »`B`     | String/Integer | Best bid size. If no bids, it will be 0         |
  | »`a`     | String         | Best ask price. If no asks, it's empty string   |
  | »`A`     | String/Integer | Best ask size. If no asks, it will be 0         |

</Block>

<Block>

## Best ask/bid unsubscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.book_ticker","event": "unsubscribe", "payload" : ["BTC_USDT"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe specified contract order book.**

### Request

- channel

  `futures.book_ticker`

- event

  `unsubscribe`

</Block>

<Block>

## Order book update subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.order_book_update","event": "subscribe", "payload" : ["BTC_USDT", "100ms", "100"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe order_book_update.**

### Request

- channel

  `futures.order_book_update`

- event

  `subscribe`

- params

  | parameter   | type   | required | description                                                                                                                         |
    | ----------- | ------ | -------- | ----------------------------------------------------------------------------------------------------------------------------------- |
  | `contract`  | String | Yes      | Contract name                                                                                                                       |
  | `frequency` | String | Yes      | Update frequency, `20ms` or `100ms`                                                                                      |
  | `level`     | String | No       | Optional depth level interested. Only updates within are notified. Allowed values: "100", "50", "20"; `20ms` only Allowed "20" |

</Block>

<Block>

## Order book update notification

<Example>

```json
{
  "time": 1615366381,
  "time_ms": 1615366381123,
  "channel": "futures.order_book_update",
  "event": "update",
  "result": {
    "t": 1615366381417,
    "s": "BTC_USD",
    "U": 2517661101,
    "u": 2517661113,
    "b": [
      {
        "p": "54672.1",
        "s": "0"
      },
      {
        "p": "54664.5",
        "s": "58794"
      }
    ],
    "a": [
      {
        "p": "54743.6",
        "s": "0"
      },
      {
        "p": "54742",
        "s": "95"
      }
    ],
    "l": "100"
  }
}
```

</Example>

**Notify contract order book update**

### Notify

- channel

  `futures.order_book_update`

- event

  `update`

- params

  | field    | type           | description                                                               |
    | -------- |----------------| ------------------------------------------------------------------------- |
  | `result` | Object         | Changed asks and bids since last update                                   |
  | »`t`     | Integer        | Order book generation timestamp in milliseconds                           |
  | »`full`   | Boolean        | `true` indicates a full depth snapshot (e.g., if the subscribed `level` is 100, the pushed data will contain 100 levels of depth). Upon receiving this, the user should replace the local order book with the provided data.`false` indicates incremental depth updates. When the value is `false`, this field is omitted from the message |
  | »`s`     | String         | Contract name                                                             |
  | »`U`     | Integer        | First order book update ID since last update                              |
  | »`u`     | Integer        | Last order book update ID since last update                               |
  | »`b`     | Array          | Changed bids                                                              |
  | »»`p`    | String         | Changed price                                                             |
  | »»`s`    | String/Integer | Absolute size value after change. If 0, remove this price from order book |
  | »`a`     | Array          | Changed asks                                                              |
  | »»`p`    | String         | Changed price                                                             |
  | »»`s`    | String/Integer | Absolute size value after change. If 0, remove this price from order book |
  | »`l`       | String         | Depth level (For example, 100 represents a depth update of 100 levels)                    |

</Block>

<Block>

## Order book update unsubscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws.gateio.ws/v4/ws/usdt")
ws.send(
  '{"time" : 123456, "channel" : "futures.order_book_update", "event": "unsubscribe", "payload" : ["BTC_USDT", "100ms"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe specified contract order book.**

### Request

- channel

  `futures.order_book_update`

- event

  `unsubscribe`

</Block>

<Block>

# Order Book V2 API

**Provide a faster method for retrieving depth information**

</Block>

<Block>

## Maintain local depth

Note:

1. Full Depth Snapshot Push (`full=true`):When the channel pushes full depth data, the local depth should be completely replaced with the received data. Additionally, the `depth ID` should be updated to the value of the `u` field in the message. Note that the server may send full depth snapshots multiple times.
    1. The first message pushed after subscribing to this channel is a full depth snapshot.
2. Incremental Depth Push (`full=false`):Incremental messages do not include the full field. Instead, each message contains the fields `U` (starting depth ID) and `u` (ending depth ID).
    1. If `U` equals the local `depth ID` + `1`, it indicates a continuous depth update:
        1. Replace the local depth ID with the value of `u` from the message.
        2. If `a` (asks) and `b` (bids) in the update are not empty, update the corresponding bid and ask depth levels based on price.(Each `level[0]` represents the price, and `level[1]` represents the quantity.)If `level[1]` is `"0"`, the corresponding price level should be removed from the order book.
    2. If `U` ≠ local `depth ID` + `1`, the depth data is not continuous. In this case, you must unsubscribe from the market and resubscribe to retrieve the initial depth snapshot.
3. Subscription Limitations:For the same contract’s depth stream, a single connection is allowed to subscribe only once. Duplicate subscription attempts will result in an error.
   Example of a failed attempt:

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

## Order book V2 subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/usdt")
ws.send('{"time" : 123456, "channel" : "futures.obu",
        "event": "subscribe", "payload" : ["ob.BTC_USDT.400"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

### Request

- channel

  `futures.obu`
- event

  `subscribe`
- params

  `payload`It is a list containing stream names. The format is: ob.{symbol}.{level}; for example, ob.BTC_USDT.400, ob.BTC_USDT.50

  The `level` enum values are: 400, 50; Update frequency: 400-level updates every 100 ms; 50-level updates every 20 ms.

</Block>

<Block>

## Order book V2 update notification

<Example>

Example of full depth push:

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

Example of incremental push:

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

**Notify contract order book v2 update**

### Notify

- channel

  `futures.obu`
- event

  `update`
- params


| field                | type                    | description                                                  |
| ------------------- | ----------------------- | ----------------------------------------------------- |
| `result`            | Object                  | Ask and bid price changes since the previous update                      |
| »`t`               | Integer                 | Order book generation timestamp (in milliseconds)                      |
| »`full`             | Boolean                 | `true` indicates a full depth snapshot; `false` indicates incremental depth updates. When the value is `false`, this field is omitted from the message  |
| »`s`               | String                  | Name of the depth stream                                         |
| »`U`               | Integer                 | Starting order book update ID of this update                           |
| »`u`               | Integer                 | Ending order book update ID of this update                           |
| »`b`               | `Array[OrderBookArray]` | Bids updates since the last update                            |
| »» OrderBookArray | `Array[String]`         | An array pair [Price, Amount]; if Amount = 0, the corresponding entry should be removed from the local depth. |
| »`a`               | `Array[OrderBookArray]` | Asks updates since the last update                            |
| »» OrderBookArray | `Array[String]`         | An array pair [Price, Amount]; if Amount = 0, the corresponding entry should be removed from the local depth. |

</Block>

<Block>

## Order book V2 update unsubscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send(
  '{"time" : 123456, "channel" : "futures.obu", "event": "unsubscribe", "payload" : ["ob.BTC_USDT.400"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe specified contract order book v2**

### Request

- channel

  `futures.obu`
- event

  `unsubscribe`

</Block>

<Block>

# Candlesticks API

**Provides a way to access charting candlestick info.**

</Block>

<Block>

## Candlesticks subscription

<strong><em>If prefix `contract` with `mark_`, the contract's mark price candlesticks will be subscribed; if
prefix with `index_`, index price candlesticks will be subscribed.</em></strong>

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.candlesticks","event": "subscribe", "payload" : ["1m", "BTC_USD"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.candlesticks","event":"subscribe","payload":["1m", "BTC_USD"]}`

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

The above command returns JSON structured like this:

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

### Request

- channel

  `futures.candlesticks`

- event

  `subscribe`

- params

  | field      | type   | description                                                              |
    | ---------- | ------ | ------------------------------------------------------------------------ |
  | `interval` | String | Interval : "10s", "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "7d" |
  | `contract` | String | Futures contract name                                                    |

</Block>

<Block>

## Candlesticks notification

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
      "v": "27525555",
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USD",
      "a": "314732.87412",
      "w": false
    },
    {
      "t": 1545129300,
      "v": "27525555",
      "c": "95.4",
      "h": "96.9",
      "l": "89.5",
      "o": "94.3",
      "n": "1m_BTC_USD",
      "a": "314732.87412",
      "w": true
    }
  ]
}
```

</Example>

**Notify kline information of subscribed contract.**

### Notify

- channel

  `futures.candlesticks`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field | type           | description           |
    | ----- |----------------| --------------------- |
  | `t`   | Integer        | Time                  |
  | `o`   | String         | Open                  |
  | `c`   | String         | Close                 |
  | `h`   | String         | Highest               |
  | `l`   | String         | Lowest                |
  | `v`   | String/Integer | Volume                |
  | `n`   | String         | Futures contract name |
  | `a`   | String         | Amount                |
  | `w`   | Boolean        | `true` means window close. `true` may be missing, but does not affect data usage                |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send(
  '{"time" : 123456, "channel" : "futures.candlesticks", "event": "unsubscribe", "payload" : ["1m", "BTC_USD"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	msg := `{"time":123456,"channel":"futures.candlesticks","event":"unsubscribe","payload":["1m", "BTC_USD"]}`

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

The above command returns JSON structured like this:

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

**Unsubscribe specified contract kline information.**

### Request

- channel

  `futures.candlesticks`

- event

  `unsubscribe`

</Block>

<Block>

# public liquidates order API

**Provide a way to receive Gate liquidate order information, and each contract can push up to one liquidate order data every 1 second**

</Block>

<Block>

## public liquidate order subscription

If you want to subscribe to liquidates order push in all contracts, use `!all` in the subscription request list

<Example>

Code example

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": 123456,
    "channel": "futures.public_liquidates",
    "event": "subscribe",
    "payload": ["BTC_USD","ETH_USD"],
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.public_liquidates",
		"event":   "subscribe",
		"payload": []string{"BTC_USD", "ETH_USD"},
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

The above command returns JSON structured like this:

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

**public liquidates order notify**

### request parameters

- channel

  `futures.public_liquidates`

- event
  `subscribe`

- params

  | name       | type   | required | describe     |
      | ---------- | ------ | ---- | -------- |
  | `contract` | String | yes   | contract list |

</Block>

<Block>

## public liquidate order notification

<Example>

```json
{
    "channel": "futures.public_liquidates",
    "event": "update",
    "time": 1541505434,
    "time_ms": 1541505434123,
    "result": [
        {
        "price": 215.1,
        "size": "-124",
        "time_ms": 1541486601123,
        "contract": "BTC_USD",
        }
    ]
}
```

</Example>

**Push public liquidate order updates**

### Notify params

- channel

  `futures.public_liquidates`

- event

  `update`

- params

  | name     | type  | describe             |
      | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | parameter  | type           | description           |
      | ------------- |----------------| -------------------- |
  | `price` | Float          | order price             |
  | `size`        | string/Integer | liquidate order quantity         |
  | `time_ms`     | Integer        | time_in_milliseconds |
  | `contract`    | string         | contract             |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": 123456,
    "channel": "futures.public_liquidates",
    "event": "unsubscribe",
    "payload": ["BTC_USD"],
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"
	"encoding/json"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    123456,
		"channel": "futures.public_liquidates",
		"event":   "unsubscribe",
		"payload": []string{"BTC_USD"},
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

The above command returns the JSON structure as follows：

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

**public liquidate order Unsubscribe**

### params

- channel

  `futures.public_liquidates`

- event

  `unsubscribe`

</Block>

<Block>

# Contract Stats API

**The contract_stats channel allows you to obtain contract statistics.**

</Block>

<Block>

## Contract Stats subscription

<Example>

Code example

```python
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
ws.send('{"time" : 123456, "channel" : "futures.contract_stats","event": "subscribe", "payload" : ["BTC_USD","1m"]}')
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	message := `{
		"time": 123456,
		"channel": "futures.contract_stats",
		"event": "subscribe",
		"payload": ["BTC_USD", "1m"]
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

The above command returns JSON structured like this:

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

**Subscribe futures contract stats.**

### Request

- channel

  `futures.contract_stats`

- event

  `subscribe`

- params

  | parameter  | type   | required | description   |
    | ---------  | -----  | -------- | ------------- |
  | `contract` | String | Yes      | Futures contract name |
  | `interval` | String | Yes      | Interval : "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "3d", "7d" |

</Block>

<Block>

## Contract Stats notification

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
      "contract": "BTC_USDT",
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

**Notify subscribed contract stats.**

### Notify

- channel

  `futures.contract_stats`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field                     | type           | description                                                                                                            |
    | ------------------------- |----------------| ---------------------------------------------------------------------------------------------------------------------- |
  | `time`                | Integer        | Stat timestamp                  |
  | `contract`                | String         | Contract Name                  |
  | `mark_price`          | Float          | Current mark price        |
  | `lsr_taker`           | Float          | Long/short account number ratio     |
  | `lsr_account`         | Float          | Long/short taker size ratio                          |
  | `long_liq_size`       | String/Integer | Long liquidation size         |
  | `long_liq_amount`     | Float          | Long liquidation amount(base currency)               |
  | `long_liq_usd`        | Float          | Long liquidation volume(quote currency)                  |
  | `short_liq_size`      | String/Integer | Short liquidation size                     |
  | `short_liq_amount`    | Float          | Short liquidation amount(base currency) |
  | `short_liq_usd`       | Float          | Short liquidation volume(quote currency) |
  | `open_interest`       | String/Integer | Open interest size|
  | `open_interest_usd`   | Float          | Open interest volume(quote currency)  |
  | `top_lsr_account`     | Float          | Top trader long/short account ratio    |
  | `top_lsr_size`        | Float          | Top trader long/short position ratio   |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": 123456,
    "channel": "futures.contract_stats",
    "event": "unsubscribe",
    "payload": ["BTC_USD","1m"]
}
ws.send(json.dumps(req))
print(ws.recv())
```

</Example>

<Example>

Code example

```go
package main

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := `{
		"time": 123456,
		"channel": "futures.contract_stats",
		"event": "unsubscribe",
		"payload": ["BTC_USD", "1m"]
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

The above command returns JSON structured like this:

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

**Unsubscribe contract stats.**

### Request

- channel

  `futures.contract_stats`

- event

  `unsubscribe`

- params

  | parameter  | type   | required | description   |
    | ---------  | -----  | -------- | ------------- |
  | `contract` | String | Yes      | Futures contract name |
  | `interval` | String | Yes      | Interval : "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "7d" |

**Note: `contract` is `unsub_all`, which means cancel all**

</Block>

<Block>

# ADL Risk Warning API

**The public `futures.adl_warning` channel provides market-level ADL risk warning updates.**

This channel does not require authentication and supports JSON connections only.

Market-level auto-deleveraging (ADL) risk warnings help users understand changes in the risk of ADL occurring in a contract market. The platform determines the risk state based on the insurance fund's positions in that market and the corresponding market limits. Only the resulting state is provided; the underlying thresholds and calculation formulas are not disclosed.

Users receive notifications only after establishing a WebSocket connection and subscribing to the relevant channel or contracts. Users who have not subscribed or have disconnected do not receive notifications.

### Status description

| Status | Description |
| ------ | ----------- |
| `normal` | The probability of ADL occurring in this market is low. However, sudden, significant market volatility may generate a large volume of liquidation orders and trigger ADL. |
| `warning` | The probability of ADL occurring in this market is high. The insurance fund has limited remaining capacity to absorb liquidation orders, and new liquidations in this market are likely to be handled through ADL. |
| `adl_risk` | The market is in a state where the insurance fund can no longer absorb liquidation orders. Any new liquidations must therefore be handled through ADL. This status does not, by itself, indicate that ADL has already occurred. |

### Additional notes

1. `normal`, `warning`, and `adl_risk` are market-level risk states. They do not determine whether an individual user's position will or will not be subject to ADL.
2. `adl_risk` does not mean that ADL has already occurred, nor does it mean that all users will be affected.
3. These states are provided solely to help users understand changes in market risk and do not constitute trading advice.
4. Users are solely responsible for the trading outcomes of any actions taken based on these states, including reducing or increasing positions, canceling orders, or adjusting trading strategies.
5. To query historical liquidation orders, refer to [Query Liquidation Order History](https://www.gate.com/docs/developers/apiv4/en/futures/#query-liquidation-order-history).

</Block>

<Block>

## ADL risk warning subscription

<Example>

Subscribe to one or more contracts:

```json
{
  "time": 1781234567,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["BTC_USD"]
}
```

Subscribe to all contracts for the settlement currency of the connection:

```json
{
  "time": 1781234567,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["!all"]
}
```

If all matching contracts are currently `normal`, the server returns only the subscription response:

```json
{
  "time": 1781234567,
  "time_ms": 1781234567123,
  "channel": "futures.adl_warning",
  "event": "subscribe",
  "payload": ["BTC_USD"],
  "result": {
    "status": "success"
  }
}
```

</Example>

### Request

- channel

  `futures.adl_warning`
- event

  `subscribe`
- params

  | parameter  | type   | required | description                                                                                                      |
  | ---------- | ------ | -------- | ---------------------------------------------------------------------------------------------------------------- |
  | `contract` | String | Yes      | Contract name. Multiple contracts can be provided, or use `!all` as the only value to subscribe to all contracts |

The payload must not be empty. `!all` cannot be combined with a contract name, and every specified contract must exist for the settlement currency of the connection. Repeated contract names are ignored.

If a matching contract is already in `warning` or `adl_risk`, its initial `event=all` snapshot is sent before the subscription response.

</Block>

<Block>

## ADL risk warning notification

<Example>

Initial risk snapshot after subscription:

```json
{
  "time": 1781234567,
  "time_ms": 1781234567123,
  "channel": "futures.adl_warning",
  "event": "all",
  "result": {
    "contract": "BTC_USD",
    "settle": "btc",
    "state": "warning",
    "update_time": 1781234567000
  }
}
```

Risk update:

```json
{
  "time": 1781234568,
  "time_ms": 1781234568123,
  "channel": "futures.adl_warning",
  "event": "update",
  "result": {
    "contract": "BTC_USD",
    "settle": "btc",
    "state": "adl_risk",
    "update_time": 1781234567000
  }
}
```

Recovery update:

```json
{
  "time": 1781234578,
  "time_ms": 1781234578123,
  "channel": "futures.adl_warning",
  "event": "update",
  "result": {
    "contract": "BTC_USD",
    "settle": "btc",
    "state": "normal",
    "update_time": 1781234577000
  }
}
```

</Example>

### Notify

- channel

  `futures.adl_warning`
- event

  `all` for the initial risk snapshot, or `update` for subsequent notifications
- params

  | field         | type    | description                                                                                      |
  | ------------- | ------- | ------------------------------------------------------------------------------------------------ |
  | `contract`    | String  | Contract name                                                                                    |
  | `settle`      | String  | Settlement currency in lowercase, such as `btc`, `usdt`, or `usd1`                              |
  | `state`       | String  | Market-level ADL risk state: `normal`, `warning`, or `adl_risk`. See [Status description](#status-description)                                               |
  | `update_time` | Integer | Time when the risk state was calculated, in Unix milliseconds                                    |

`warning` and `adl_risk` states are pushed once per second while they remain active. A transition back to `normal` is pushed once and is not repeated. Contracts that are `normal` when first subscribed do not produce an initial snapshot.

</Block>

<Block>

## Cancel ADL risk warning subscription

<Example>

```json
{
  "time": 1781234579,
  "channel": "futures.adl_warning",
  "event": "unsubscribe",
  "payload": ["BTC_USD"]
}
```

The server returns:

```json
{
  "time": 1781234579,
  "time_ms": 1781234579123,
  "channel": "futures.adl_warning",
  "event": "unsubscribe",
  "payload": ["BTC_USD"],
  "result": {
    "status": "success"
  }
}
```

</Example>

### Request

- channel

  `futures.adl_warning`
- event

  `unsubscribe`
- params

  Use the same payload rules as subscription. Use `["!all"]` to cancel an all-contract subscription.

</Block>

<Block>

# Orders API

**Provides a way to receive user closed orders.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Orders subscription

If you want to subscribe to order updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.orders",
  "event": "subscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

```go
package main

import (
	"fmt"
	"log"
  "time"

	"github.com/gorilla/websocket"
)

func main() {
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.orders",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe user orders update**

### Request

- channel

  `futures.orders`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Orders notification

<Example>

```json
{
  "channel": "futures.orders",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "contract": "BTC_USD",
      "create_time": 1628736847,
      "create_time_ms": 1628736847325,
      "fill_price": 40000.4,
      "finish_as": "filled",
      "finish_time": 1628736848,
      "finish_time_ms": 1628736848321,
      "iceberg": "0",
      "id": 4872460,
      "is_close": false,
      "is_liq": false,
      "is_reduce_only": false,
      "left": "0",
      "mkfr": -0.00025,
      "price": 40000.4,
      "refr": 0,
      "refu": 0,
      "size": "1",
      "status": "finished",
      "text": "-",
      "tif": "gtc",
      "tkfr": 0.0005,
      "user": "110xxxxx",
      "update_id": 1,
      "update_time": 1541505434123,
      "stop_loss_price": "",
      "stop_profit_price": "",
      "market_order_slip_ratio": "0.03"
    }
  ]
}
```

</Example>

**Notify user orders information when an order is put, updated or finished.**

### Notify

- channel

  `futures.orders`

- event

  `update`

- params

  `For the meaning of parameters, please refer to http interface.`

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field            | type           | description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
    | ---------------- |----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
  | `create_time`    | Integer        | Creation time of order(deprecated)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
  | `create_time_ms` | Integer        | Creation timestamp in milliseconds of order                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
  | `fill_price`     | Float          | Fill price of the order                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
  | `finish_as`      | String         | How the order was completed.<br/>- `filled`: Fully Filled<br/>- `cancelled`: Manually Cancelled<br/>- `liquidated`: Cancelled Due to Liquidation<br/>- `ioc`: IOC (Immediate or Cancel), executed immediately<br/>- `auto_deleveraging`: Completed via ADL (Automatic Deleveraging)<br/>- `reduce_only`: Cancelled when attempting to increase position under reduce-only rule<br/>- `position_close`: Cancelled due to position closing<br/>- `stp`: Cancelled due to self-trade prevention (STP)<br/>- `_new`: Newly Created<br/> - `_update`: Filled, Partially Filled, or Order Updated<br/>- `reduce_out`: Reduce-only: Excluded illiquid pending orders                   |
  | `iceberg`        | String/Integer | Display size for iceberg order. 0 for non-iceberg. Note that you will have to pay the taker fee for the hidden size                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
  | `id`             | Integer        | Futures order ID                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
  | `is_close`       | Bool           | Is the order to close position                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
  | `is_liq`         | Bool           | Is the order for liquidation                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
  | `left`           | String/Integer | Size left to be traded                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
  | `mkfr`           | Float          | Maker fee                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
  | `is_reduce_only` | Bool           | Is the order reduce-only                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
  | `status`         | String         | Order status<br />- open: waiting to be traded<br />- finished: finished                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
  | `tkfr`           | Float          | Taker fee                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
  | `price`          | Float          | Order price. 0 for market order with tif set as ioc                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
  | `refu`           | Integer        | Reference user ID                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
  | `refr`           | Float          |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
  | `size`           | String/Integer | Order size. Specify positive number to make a bid, and negative number to ask                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
  | `text`           | String         | User defined information.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
  | `tif`            | String         | Time in force <br/>- gtc: GoodTillCancelled <br/>- ioc: ImmediateOrCancelled, taker only<br/>- poc: PendingOrCancelled, makes a post-only order that always enjoys a maker fee<br/>- fok: FillOrKill, fill either completely or none<br/>Only ioc and fok are supported when type=market                                                                                                                                                                                                                                                                                                                                                                                        |
  | `finish_time`    | Integer        | Order finished unix timestamp in seconds. For unfinished orders, this field returns a value of 0.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
  | `finish_time_ms` | Integer        | Order finished unix timestamp in milliseconds. For unfinished orders, this field returns a value of 0.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
  | `user`           | String         | User ID                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
  | `contract`       | String         | Futures contract                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
  | `stp_id`         | String         | Orders between users in the same stp_id group are not allowed to be self-traded<br />1. If the stp_id of two orders being matched is non-zero and equal, they will not be executed. Instead, the corresponding strategy will be executed based on the stp_act of the taker.<br />2. stp_id returns 0 by default for orders that have not been set for STP group                                                                                                                                                                                                                                                                                                                 |
  | `stp_act`        | String         | Self-Trading Prevention Action. Users can use this field to set self-trade prevetion strategies<br />1. After users join the STP Group, he can pass stp_act to limit the user's self-trade prevetion strategy. If stp_act is not passed, the default is cn strategy。<br />2. When the user does not join the STP group, an error will be returned when passing the stp_act parameter。<br />3. If the user did not use 'stp_act' when placing the order, 'stp_act' will return '-'<br />- cn: Cancel newest, Cancel new orders and keep old ones<br />- co: Cancel oldest, Cancel old orders and keep new ones<br />- cb: Cancel both, Both old and new orders will be cancelled |
  | `amend_text`     | String         | The custom data that the user remarked when amending the order                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
  | `update_id`        | Integer        | Update ID                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
  | `update_time`     | Integer        | Update Time (Milliseconds Timestamp)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
  | `biz_info`     | String         | Users can annotate this modification with information.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
  | `stop_profit_price`     | String         | Stop Profit Price                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
  | `stop_loss_price`     | String         | Stop Loss Price                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
  | `market_order_slip_ratio`   | String         | The preset maximum slippage ratio (not representing actual slippage) for this market order is 0.03 representing the maximum slippage of 3%                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.orders",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.orders",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe user orders update notification, for all contract.**

### Request

- channel

  `futures.orders`

- event

  `unsubscribe`

</Block>

<Block>

# Orders V2 API

**Supported environments: USDT futures live trading and TestNet.**

- Real Trading: `wss://fx-ws.gateio.ws/v4/ws/usdt`
- TestNet: `wss://ws-testnet.gate.com/v4/ws/futures/usdt`

`futures.orders_v2` provides real-time order creation, fill, amendment, and cancellation updates from the matching stage, before clearing. It is available only for UTA3.0 users on the USDT-settled futures API WebSocket endpoint, using JSON encoding. PB and SBE subscriptions are not supported.

Authentication is required. The authenticated user determines which orders are pushed. The service checks that user's UTA3.0 eligibility; a user ID supplied in `payload` cannot select another account.

This channel excludes liquidation orders and clearing information. It retains `mkfr` as a fee rate and omits `fee`, `point_fee`, `role`, `refu`, `refr`, `tkfr`, and `update_time`. Use `update_time_ms` for the ME timestamp.

`futures.orders` and `futures.orders_v2` can be subscribed independently on the same connection. Do not infer event ordering from which channel's message arrives first. After migrating an account to UTA3.0, subscribe again to use this channel.

</Block>

<Block>

## Orders V2 subscription

Use `channel: "futures.orders_v2"` and `event: "subscribe"`. Generate the signature using this channel name, event, and request time as described in Authentication.

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

Payload format:

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `payload` | Array[String] | Yes | Contract list, optionally preceded by the user ID. Examples: `["BTC_USDT"]`, `["20011", "BTC_USDT", "ETH_USDT"]`, or `["!all"]`. |

A contract entry may also contain comma-separated names, such as `"BTC_USDT,ETH_USDT"`. Use `!all` to subscribe to all contracts. Repeated subscriptions add contracts; duplicate contracts and overlap with `!all` do not cause duplicate pushes. The entire payload is validated before subscriptions change; an invalid contract rejects the whole request.

A successful subscription returns an acknowledgement only, with no historical order snapshot. The first message after subscribing may be an update to an existing order. Events missed during a disconnected period are not replayed.

Subscription errors:

| Code | Description |
| --- | --- |
| `2` | Invalid user ID or contract, non-UTA3.0 account, or unsupported PB/SBE encoding. |
| `3` | The service cannot query the account eligibility. |
| `4` | Authentication is required or has failed. |

</Block>

<Block>

## Orders V2 notification

All order events use `event: "update"`. `result` is an array of order objects.

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

`size`, `left`, `iceberg`, `price`, and `fill_price` are always decimal strings, with fractional quantities preserved. This is the same with or without the `X-Gate-Size-Decimal: 1` header.

`create_time_ms` comes from OMS. For an order creation event, `update_time_ms` is when the order reaches ME; for later fills, amendments, or cancellations, it is the ME update time. “Creation event” does not mean the first message on a newly subscribed connection. A later event's `update_time_ms - create_time_ms` includes the order's lifetime and does not measure that amendment's or cancellation's OMS-to-ME latency.

The outer `time` and `time_ms` are WS envelope timestamps in seconds and milliseconds. `finish_time` and `finish_time_ms` follow the upstream event log timestamp. These fields must not substitute for the ME `update_time_ms`. Messages without valid positive OMS/ME millisecond timestamps are not pushed; the service does not fill them using local time.

Order fields:

| Field | Type | Description |
| --- | --- | --- |
| `contract` | String | Futures contract name. |
| `create_time` | Integer | OMS order creation timestamp in seconds. |
| `create_time_ms` | Integer | OMS order creation timestamp in milliseconds. |
| `fill_price` | String | Order fill price, as a decimal string. |
| `finish_as` | String | `_new` for creation and `_update` for updates while the order is open. For a finished order, the completion reason, such as `filled`, `cancelled`, `ioc`, `poc`, `fok`, or `stp`; other reasons follow `futures.orders`. |
| `finish_time` | Integer | Upstream order event log timestamp in seconds; this is not a clearing completion timestamp. |
| `finish_time_ms` | Integer | Upstream order event log timestamp in milliseconds; this is not the ME update timestamp. |
| `iceberg` | String | Displayed iceberg order size, as a decimal string. `"0"` for a regular order. |
| `id` | Integer | Order ID. Use `id_string` if your client cannot preserve large integers. |
| `id_string` | String | Order ID as a string. |
| `is_close` | Boolean | Whether the order closes a position. |
| `is_liq` | Boolean | Always `false` on this channel; liquidation orders are excluded. |
| `is_reduce_only` | Boolean | Whether the order is reduce-only. |
| `is_voucher` | Boolean | Whether this is a voucher order. |
| `left` | String | Remaining order size, as a decimal string. |
| `mkfr` | Float | Maker fee rate; this is not a settled fee amount. |
| `price` | String | Order price, as a decimal string. `"0"` for a market order. |
| `size` | String | Order size, as a decimal string; positive for buy and negative for sell. |
| `status` | String | Order status: `open` (waiting to trade) or `finished`. |
| `text` | String | User-defined order information. |
| `tif` | String | Time in force: `gtc` (good till cancelled), `ioc` (immediate or cancel), `poc` (post only), or `fok` (fill or kill). |
| `user` | String | User ID. |
| `update_id` | Integer | Order update ID. |
| `update_time_ms` | Integer | ME timestamp in milliseconds. For creation, when the order reaches ME; for a fill, amendment, or cancellation, when ME updates the order. |
| `biz_info` | String | Business information attached to the order update. |
| `stp_act` | String | Self-trade prevention action: `cn` (cancel newest), `co` (cancel oldest), `cb` (cancel both), or `-` if unset. |
| `stp_id` | String | Self-trade prevention group ID; `"0"` if unset. |
| `amend_text` | String | User-defined information attached to an amendment. |
| `bbo` | String | BBO order option; `-` if unset. |
| `pos_margin_mode` | String | Position margin mode: `cross` or `isolated`. |
| `leverage` | String | Leverage, or an empty string if not supplied. |
| `stop_loss_price` | String | Stop-loss price, or an empty string if unset. |
| `stop_profit_price` | String | Take-profit price, or an empty string if unset. |
| `market_order_slip_ratio` | String | Preset maximum slippage ratio for a market order. `"0.03"` means 3%; this is not actual slippage. |

</Block>

<Block>

## Orders V2 cancellation of subscription

Use `event: "unsubscribe"` with the same payload rules. `payload: ["!all"]` clears all subscriptions to `futures.orders_v2` on this connection. Removing subscriptions does not require a new UTA3.0 eligibility query.

A contract currently subscribed on this connection can still be unsubscribed after delisting; if a request includes an unknown contract that is not subscribed on this connection, the entire request is rejected and existing subscriptions are kept.

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

# User trades API

**Provides a way to receive user trades.**

::: warning
Authentication required.
:::

</Block>

<Block>

## User trades subscription

If you want to subscribe to user trade updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.usertrades",
  "event": "subscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.usertrades",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user trades update.**

### Request

- channel

  `futures.usertrades`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## User trades notification

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
      "contract": "BTC_USD",
      "order_id": "4872460",
      "size": "1",
      "price": "40000.4",
      "role": "maker",
      "text": "api",
      "fee": 0.0009290592,
      "point_fee": 0
    }
  ]
}
```

</Example>

**Notify user trades update.**

### Notify

- channel

  `futures.usertrades`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field            | type           | description                 |
    | ---------------- |----------------| --------------------------- |
  | `contract`       | String         | Futures contract name       |
  | `create_time`    | Integer        | Create time                 |
  | `create_time_ms` | Integer        | Create time in milliseconds |
  | `id`             | String         | Trades id                   |
  | `order_id`       | String         | Order Id                    |
  | `price`          | String         | Price                       |
  | `size`           | String/Integer | Trades size                 |
  | `role`           | String         | User role (maker/taker)     |
  | `text`           | String         | Order Custom Information: Users can use this field to set a custom ID. User-defined fields must meet the following conditions:<br><br>1. Must start with `t-`<br>2. Excluding `t-`, the length cannot exceed 28 bytes<br>3. Input content can only contain numbers, letters, underscores (_), hyphens (-), or dots (.)<br><br>Besides user-defined information, the following are internal reserved fields indicating order sources:<br><br>- web: Web<br>- api: API Call<br>- app: Mobile App<br>- auto_deleveraging: Auto deleveraging<br>- liquidation: Legacy classic mode position forced liquidation<br>- liq-xxx: a. New classic mode position forced liquidation, including isolated margin, one-way cross margin, two-way cross margin unhedged position forced liquidation. b. Unified account single-currency margin mode isolated margin forced liquidation<br>- hedge-liq-xxx: New classic mode two-way cross margin hedging part forced liquidation, which means closing both long and short positions simultaneously<br>- pm_liquidate: Unified account cross-currency margin mode forced liquidation<br>- comb_margin_liquidate: Unified account portfolio margin mode forced liquidation<br>- scm_liquidate: Unified account single-currency margin mode position forced liquidation<br>- insurance: Insurance<br>- clear: Contract delisting and clearance    |
  | `fee`            | Float          | Fee deducted                |
  | `point_fee`      | Float          | Points used to deduct fee   |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.usertrades",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.usertrades",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe user trades update.**

### Request

- channel

  `futures.usertrades`

- event

  `unsubscribe`

</Block>

<Block>

# Liquidates API

**Provides a way to receive user liquidates info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Liquidates subscription

If you want to subscribe to liquidate updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.liquidates",
  "event": "subscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.liquidates",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user liquidates update.**

### Request

- channel

  `futures.liquidates`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Liquidates notification

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
      "size": "-124",
      "time": 1541486601,
      "time_ms": 1541486601123,
      "contract": "BTC_USD",
      "user": "1040xxxx"
    }
  ]
}
```

</Example>

**Notify liquidates update.**

### Notify

- channel

  `futures.liquidates`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field         | type           | description             |
    | ------------- |----------------| ----------------------- |
  | `entry_price` | Float          | Average entry price     |
  | `fill_price`  | Float          | Average execution price |
  | `leverage`    | Float          | Leverage                |
  | `liq_price`   | Float          | Liquidates price        |
  | `margin`      | Float          | Margin                  |
  | `mark_price`  | Float          | Mark price              |
  | `order_id`    | Integer        | Order id                |
  | `order_price` | Float          | Commission price        |
  | `left`        | String/Integer | Order unfilled quantity |
  | `size`        | String/Integer | Position original size  |
  | `time`        | Integer        | Time                    |
  | `time_ms`     | Integer        | Time in milliseconds    |
  | `user`        | String         | User id                 |
  | `contract`    | String         | Futures contract name   |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.liquidates",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.liquidates",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe liquidates update.**

### Request

- channel

  `futures.liquidates`

- event

  `unsubscribe`

</Block>

<Block>

# Auto_deleverages API

**Provides a way to receive user auto deleverages info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Auto_deleverages subscription

If you want to subscribe to auto deleverage updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.auto_deleverages",
  "event": "subscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.auto_deleverages",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user auto_deleverages update.**

### Request

- channel

  `futures.auto_deleverages`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Auto_deleverages notification

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
      "contract": "BTC_USD",
      "user": "1040"
    }
  ]
}
```

</Example>

**Notify auto_deleverages update.**

### Notify

- channel

  `futures.auto_deleverages`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field           | type           | description           |
    | --------------- |----------------| --------------------- |
  | `entry_price`   | Float          | Entry price           |
  | `fill_price`    | Float          | Execution price       |
  | `position_size` | String/Integer | Position size         |
  | `trade_size`    | String/Integer | Trade size            |
  | `time`          | Integer        | Time                  |
  | `time_ms`       | Integer        | Time in milliseconds  |
  | `user`          | String         | User id               |
  | `contract`      | String         | Futures contract name |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.auto_deleverages",
  "event": "unsubscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.auto_deleverages",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe auto_deleverages update.**

### Request

- channel

  `futures.auto_deleverages`

- event

  `unsubscribe`

</Block>

<Block>

# Position_closes API

**Provides a way to receive user position closes info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Position_closes subscription

If you want to subscribe to position close updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
  "time": int(time.time()),
  "channel": "futures.position_closes",
  "event": "subscribe",
  "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_closes",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user position_closes update.**

### Request

- channel

  `futures.position_closes`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Position_closes notification

<Example>

```json
{
  "channel": "futures.position_closes",
  "event": "update",
  "time": 1541505434,
  "time_ms": 1541505434123,
  "result": [
    {
      "contract": "BTC_USD",
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

**Notify position_closes update.**

### Notify

- channel

  `futures.position_closes`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field      | type    | description           |
    | ---------- | ------- | --------------------- |
  | `contract` | String  | Futures contract name |
  | `pnl`      | Number  | Profit & loss         |
  | `side`     | String  | Side (long or short)  |
  | `text`     | String  | Messages              |
  | `time`     | Integer | Time                  |
  | `time_ms`  | Integer | Time in milliseconds  |
  | `user`     | String  | User id               |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.position_closes",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.position_closes",
		"event":   "unsubscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe position_closes update.**

### Request

- channel

  `futures.position_closes`

- event

  `unsubscribe`

</Block>

<Block>

# Balances API

**Provides a way to receive user balances info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Balances subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe for user balances update.**

### Request

- channel

  `futures.balances`

- event

  `subscribe`

- params

  | parameter  | type   | required | description |
    | ---------- | ------ | -------- | ----------- |
  | `user id ` | String | yes      | User id     |

</Block>

<Block>

## balances notification

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
      "text": "BTC_USD:3914424",
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

**Notify balances update.**

### Notify

- channel

  `futures.balances`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field      | type    | description           |
    | ---------- | ------- | --------------------- |
  | `balance`  | Number  | Balance after changed |
  | `change`   | Number  | Change                |
  | `text`     | String  | Messages              |
  | `time`     | Integer | Time                  |
  | `time_ms`  | Integer | Time in milliseconds  |
  | `type`     | String  | Type                  |
  | `user`     | String  | User id               |
  | `currency` | String  | Transfer unit         |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

# Reduce_risk_limits API

**Provides a way to receive user reduce risk limits info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Reduce_risk_limits subscription

If you want to subscribe to reduce risk limit updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.reduce_risk_limits",
    "event": "subscribe",
    "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.reduce_risk_limits",
		"event":   "subscribe",
		"payload": []interface{}{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user reduce_risk_limits update.**

### Request

- channel

  `futures.reduce_risk_limits`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id               |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Reduce_risk_limits notification

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

**Notify reduce risk limits update.**

### Notify

- channel

  `futures.reduce_risk_limits`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field              | type    | description          |
    | ------------------ | ------- | -------------------- |
  | `cancel_orders`    | Integer | Cancel orders        |
  | `contract`         | String  | Contract name        |
  | `leverage_max`     | Integer | Leverage max         |
  | `liq_price`        | Float   | Liquidates price     |
  | `maintenance_rate` | Float   | Maintenance rate     |
  | `risk_limit`       | Integer | Risk limit           |
  | `time`             | Integer | Time                 |
  | `time_ms`          | Integer | Time in milliseconds |
  | `user`             | String  | User id              |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.reduce_risk_limits",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("Dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.reduce_risk_limits",
		"event":   "unsubscribe",
		"payload": []string{"20011", "BTC_USD"},
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

**Unsubscribe reduce risk limits update.**

### Request

- channel

  `futures.reduce_risk_limits`

- event

  `unsubscribe`

</Block>

<Block>

# Positions API

**Provides a way to receive user positions info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Positions subscription

If you want to subscribe to position updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.positions",
    "event": "subscribe",
    "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal("Dial error:", err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.positions",
		"event":   "subscribe",
		"payload": []string{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Subscribe for user positions update.**

### Request

- channel

  `futures.positions`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `user id ` | String | yes      | User id (This field is deprecated and is only used as a placeholder)              |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Positions notification

<Example>

```json
{
  "time": 1588212926,
  "time_ms": 1588212926123,
  "channel": "futures.positions",
  "event": "update",
  "result": [
    {
      "contract": "BTC_USD",
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
      "update_id": 170919
    }
  ]
}
```

</Example>

**Notify positions update.**

### Notify

- channel

  `futures.positions`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field              | type           | description                           |
    | ------------------ |----------------| ------------------------------------- |
  | `contract`         | String         | Futures contract name                 |
  | `cross_leverage_limit`         | Float          | Cross margin leverage                                |
  | `entry_price`      | Float          | Entry price                           |
  | `history_pnl`      | Float          | History realized PNL                  |
  | `history_point`    | Float          | History realized POINT PNL            |
  | `last_close_pnl`   | Float          | PNL of last position close            |
  | `leverage`         | Integer        | Position leverage. 0 means cross margin; positive number means isolated margin    |
  | `leverage_max`     | Integer        | Maximum leverage under current risk limit                     |
  | `liq_price`        | Float          | Liquidation price                     |
  | `maintenance_rate` | Float          | Maintenance rate under current risk limit                                      |
  | `margin`           | Float          | Position margin                       |
  | `realised_pnl`     | Float          | Realized PNL                          |
  | `realised_point`   | Float          | Realized POINT PNL                    |
  | `risk_limit`       | Integer        | Position risk limit                   |
  | `size`             | String/Integer | Contract size                         |
  | `time`             | Integer        | Update unix timestamp                 |
  | `time_ms`          | Integer        | Update unix timestamp in milliseconds |
  | `user`             | String         | User id                               |
  | `update_id`        | Integer        | Message sequence number               |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.positions",
    "event": "unsubscribe",
    "payload": ["20011", "BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	req := map[string]interface{}{
		"time":    time.Now().Unix(),
		"channel": "futures.positions",
		"event":   "unsubscribe",
		"payload": []string{"20011", "BTC_USD"},
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

The above command returns JSON structured like this:

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

**Unsubscribe positions update.**

### Request

- channel

  `futures.positions`

- event

  `unsubscribe`

</Block>

<Block>

# Positions ADL API

**The channel for pushing users' ADL ranking information**

::: warning
Authentication required.
:::

</Block>

<Block>

## Positions adl subscription

If you want to subscribe to adl updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://ws-testnet.gate.com/v4/ws/futures/usdt")
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

Code example

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
	url := "wss://ws-testnet.gate.com/v4/ws/futures/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe for user positions adl update.**

### Request

- channel

  `futures.position_adl_rank`

- event

  `subscribe`

- params

| parameter       | type   | required | description     |
| ---------- | ------ | ---- | -------- |
| `contract` | String | yes   | Futures contract name |

</Block>

<Block>

## Positions adl notification

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

**Notify positions adl update.**

### Notify

- channel

  `futures.position_adl_rank`

- event

  `update`

- params

| field     | type  | description             |
| -------- | ----- | ---------------- |
| `result` | Array | Array of objects |

| field               | type    | description                                    |
| ------------------ | ------- | --------------------------------------- |
| `contract`         | String  | Futures contract name                                |
| `mode`             | String  | Position mode                                |
| `rank_division`    | Integer | Ranking tier (1～6, with 1 being the highest priority for ADL, and 5 being the lowest priority for ADL. 6: in liquidation, not included in ADL ranking)                        |
| `time_ms`          | Integer | The millisecond timestamp of the message push    |
| `user_id`          | Integer | User Id                       |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://ws-testnet.gate.com/v4/ws/futures/usdt")
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

Code example

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
	url := "wss://ws-testnet.gate.com/v4/ws/futures/usdt"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe positions adl update**

### Request

- channel

  `futures.position_adl_rank`

- event

  `unsubscribe`

</Block>

<Block>

# Auto orders API

**Provides a way to receive user auto orders info.**

::: warning
Authentication required.
:::

</Block>

<Block>

## Auto orders subscription

If you want to subscribe to auto order updates in all contracts, use `!all` in contract list.

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Subscribe for user auto orders update.**

### Request

- channel

  `futures.autoorders`

- event

  `subscribe`

- params

  | parameter  | type   | required | description           |
    | ---------- | ------ | -------- | --------------------- |
  | `contract` | String | yes      | Futures contract name |

</Block>

<Block>

## Auto orders notification

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
      "me_order_id": "213867453823"
    }
  ]
}
```

</Example>

**Notify auto orders update.**

### Notify

- channel

  `futures.autoorders`

- event

  `update`

- params

  | field    | type  | description      |
    | -------- | ----- | ---------------- |
  | `result` | Array | Array of objects |

  | field           | type    | description                                            |
    | --------------- | ------- | ------------------------------------------------------ |
  | `user`          | Number  | User id                                                |
  | `trigger`       | Object  |                                                        |
  | `initial`       | Object  |                                                        |
  | `id`            | Number  | Auto order id                                          |
  | `trade_id`      | Number  | Trade id                                               |
  | `status`        | String  | Order status                                           |
  | `reason`        | String  | Change reason                                          |
  | `create_time`   | Number  | Create time                                            |
  | `name`          | String  | Name                                                   |
  | `is_stop_order` | boolean | Is stop                                                |
  | `stop_trigger`  | Object  |                                                        |
  | `order_type`    | String  | Take-profit/stop-loss types, detail to http api        |
  | `me_order_id`   | Number  | Corresponding order ID of order take-profit/stop-loss. |

</Block>

<Block>

## Cancel subscription

<Example>

Code example

```python
import json, time
from websocket import create_connection

ws = create_connection("wss://fx-ws-testnet.gateio.ws/v4/ws/btc")
req = {
    "time": int(time.time()),
    "channel": "futures.autoorders",
    "event": "unsubscribe",
    "payload": ["BTC_USD"],
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

Code example

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
	url := "wss://fx-ws-testnet.gateio.ws/v4/ws/btc"
	conn, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

The above command returns JSON structured like this:

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

**Unsubscribe auto orders update.**

### Request

- channel

  `futures.autoorders`

- event

  `unsubscribe`

</Block>

<Block>

# Futures Account Trade

</Block>

<Block>

## Websocket API

WebSocket API allows placing , canceling , amending , querying orders through a WebSocket connection.

</Block>

<Block>

### Websocket API Client Api Request

<Example>

Code example

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

Code example

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
		Contract: "BTC_USDT",
		Size:     6024,
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

Client request example

```json
{
  "time": 1680772890,
  "channel": "futures.order_place",
  "event": "api",
  "payload": {
    "req_id": "xxxx",
    "req_param": {
      "contract": "BTC_USDT",
      "size": 10,
      "price": "80048.240000",
      "tif": "gtc",
      "text": "t-my-custom-id"
    }
  }
}
```

</Example>

`api` requests initiated from the client follow a common JSON format, which
contains the following fields:

| Field        | Type    | Required | Description                                                                                                                                                                  |
| ------------ | ------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `time`       | Integer | Yes      | Request time in seconds. Gap between request time and server time must not exceed 60 seconds                                                                                 |
| `id`         | Integer | No       | Optional request id which will be sent back by the server to help you identify which<br />request the server responds to                                                     |
| `channel`    | String  | Yes      | WebSocket channel to subscribe to.                                                                                                                                           |
| `event`      | String  | Yes      | Fixed as api                                                                                                                                                                 |
| `payload`    | Object  | Yes      | Optional request detail parameters                                                                                                                                           |
| »`req_id`    | String  | Yes      | Unique identifier of the message Provided by client.It will be returned in response message for<br />identifying the corresponding request.                                  |
| »`timestamp` | String  | Yes      | Signature time in seconds                                                                                                                                                    |
| »`api_key`   | String  | Yes      | Gate APIv4 user key string                                                                                                                                                   |
| »`signature` | String  | Yes      | Authentication signature generated using GateAPIv4 secret and request information,<br />See[Websocket API Authentication](#Websocket API Authentication) section for details |
| »`req_param` | []Byte  | Yes      | Request api param                                                                                                                                                            |

Note that the type of `payload.req_param` is channel specific, Take `futures.order_place` for example, `payload.req_param` same as
apiv4 [/futures/{settle}/orders](https://www.gate.com/docs/developers/apiv4/en/#create-a-futures-order).
You can place a limit order for BTC_USDT with example.

</Block>

<Block>

### Websocket API Server Response

<Example>

Server ack response example

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

Server api response example

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

Server response includes ack response to client requests and api result callback message updates.
Server responses follow a common JSON format, which contains the following fields:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                           |
| ---------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                      |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                    |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                            |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                       |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                         |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                      |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`x_gate_ratelimit_requests_remain`      | Integer | (For order placement/modification/cancel only) Remaining available requests in the current time window (hidden if 0)       |
| »`x_gate_ratelimit_limit`                | Integer | (For order placement/modification/cancel only) Current rate limit cap (hidden if 0)                    |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | (For order placement/modification/cancel only) If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)     |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | If this is ack response, result is the payload of the request, otherwise result is the response of the api                                                                                                                                                                                                            |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                           |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                   |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                                |

</Block>

<Block>

### Error

<Example>

Error Response Notification example

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

Error Response Notification example（rate limit）

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

Error object has the following format:

<a id="schema_error"></a>

| Field     | Type   | Description                         |
| --------- | ------ | ----------------------------------- |
| `label`   | String | Denotes error type in string format |
| `message` | String | Detailed error message              |

Explanation of rate limit-related error codes:

| Code      | Description                     |
| --------- | ------------------------ |
| `100`   | Internal rate limiting exception error |
| `311` | Futures rate limit    |
| `312` | Futures rate limiting based on fill rate    |

</Block>

<Block>

## Login

::: warning
Note: the GateAPIv4 key pair you used MUST have future Corresponding permissions(eg: order-place channel must have future write permissions),
and your outbound IP address must be in the key's IP whitelist if its whitelist is enabled.
:::

</Block>

<Block>

### Login Request

Client Api Request

<Example>

Code example

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

Code example

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

Request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                         |
| ----------- | -------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br /> it's different from outside's`id` |
| `api_key`   | `string` | Yes      | Apiv4 key                                                                                                                                           |
| `headers`   | `object` | Yes      | Apiv4 custom header                                                                                                                                 |
| `signature` | `string` | Yes      | Apiv4 signature                                                                                                                                     |
| `timestamp` | `string` | Yes      | Unix timestamp in seconds                                                                                                                           |

WebSocket api operation authentication uses the same signature calculation method with Gate APIv4 API, i.e.,
`HexEncode(HMAC_SHA512(secret, signature_string))`, but has the following differences:

1. Signature string concatenation method: `"<event>\n<channel>\n<req_param>\n<timestamp>"`,
   where `<event>`, `<channel>`,`<req_param>`, `<timestamp>` are corresponding request information
2. `req_param` in `login` channel always empty string
3. Authentication information are sent in request body in field `payload`.

</Block>

<Block>

### Login Notification

<Example>

Login Response example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                          |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                     |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                   |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                           |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                      |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                        |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                     |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| `data`           | Object | Response data of the request             |
| »`result`        | Object | If this is ack response, result is the payload of the request, otherwise result is the response of the api                                                                                                                                                                                                           |
| »»`api_key`      | String | Login success apikey                                                                                                                                                                                                                                                                                                 |
| »»`uid`          | String | Login user id                                                                                                                                                                                                                                                                                                        |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                          |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                  |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                               |

## Order Place

`futures.order_place`

You can place orders with this channel and event `api`.

**function as api below:**

```json
POST /futures/{settle}/orders
```

</Block>

<Block>

### Order Place Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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

func main() {

	// warn: you should login first before order

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

Request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                                  |
| ----------- | -------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id`           |
| `req_param` | `object` | Yes      | Api order model's json byte data, use api order model; api order model detail to [api](https://www.gate.com/docs/developers/apiv4/en/#create-a-futures-order) |
| `req_header`   | `object` | No      | Apiv4 custom header                                                                                                                                          |

`req_param` JSON byte data of the API order model:

| Field          | Type     | Required | Description                                                                                            |
| ------------- | -------- | ---- | ----------------------------------------------------------------------------------------------- |
| `contract`    | `string` | true   | contract                                                                                            |
| `size`        | `int64`  | true   | Order size. Specify a positive number for bidding, a negative number for inquiry                                                    |
| `iceberg`     | `string`  | false   |Iceberg order display size. 0 means non-iceberg. Note that you will incur fees for the hidden size                        |
| `price`       | `string` | false   | Order price. 0 means market order, with `tif` set to `ioc`                                                    |
| `close`       | `bool`   | false   | Set to `true` for closing the position, with `size` set to 0                                                                |
| `reduce_only` | `bool`   | false   | Set to `true` to only reduce the order                                                                        |
| `tif`         | `string` | false   | Validity time                                                                                        |
| `text`        | `string` | false   | User-defined information. If not empty, it must follow these rules：                                                |
| `auto_size`   | `string` | false   | Set the side to close a dual-mode position. `close_long` closes the long side, while `close_short` closes the short side. Note that `size` also needs to be set to 0 |
| `stp_act`     | `string` | false   | Self-trade prevention actions. Users can set self-trade prevention strategies through this field                                        |
| `market_order_slip_ratio`   | String         | The preset maximum slippage ratio (not representing actual slippage) for this market order is 0.03 representing the maximum slippage of 3%                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

#### Detail

**tif**: Validity time

- gtc: Good Till Canceled
- ioc: Immediate Or Cancelled, accepted only
- poc: Pending Or Cancelled, for posted orders only, always incurs maker fees
- fok: Fill Or Kill, must be completely filled or not filled at all

**text**: User-defined information. If not empty, it must follow these rules：

- Must start with "t-"
- Excluding "t-", the length cannot exceed 28 bytes
- The input can only contain numbers, letters, underscores (_), hyphens (-), or dots (.)
- If left empty, the default is apiv4-ws, from WebSocket
- web: From the web
- api: From the API
- app: From the mobile app
- auto_deleveraging: From ADL
- liquidation: From liquidation
- insurance: From insurance

**stp_act**：Self-trade prevention actions. Users can set self-trade prevention strategies through this field

After the user joins the `STP Group`, they can use `stp_act` to limit their self-trade prevention strategy. If `stp_act` is not provided, the default strategy is `cn`.
When the user is not part of the `STP group`, passing the parameter will return an error with `stp_act`.
If the user does not use `stp_act` when placing an order, `stp_act` will return `-`.

- cn: Cancel the latest order, cancel the new order and keep the old one
- co: Cancel the oldest order, cancel the old order and keep the new one
- cb: Cancel both, cancel both the new and old orders

</Block>

<Block>

### Ack Notification

<Example>

Ack Notification example for an order

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
    "conn_trace_id": "1bde5aaa0acf2f5f48edfd4392e1fa68",
    "trace_id": "e410abb5f74b4afc519e67920548838d",
    "conn_id": "5e74253e9c793974",
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

### Order Place Notification

<Example>

Response Notification example

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

Return of order information for placing an order

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                          |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                     |
| `ack`            | Bool   | The return of the "ack" message represents the WebSocket confirmation message(Currently exists in the order-place api).<br />If "ack" is false(this field will not be returned), it means that the message is a response message, and you can determine if the request was successful <br />by checking "data.errs". |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                   |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                           |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                      |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                        |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                     |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| »`x_gate_ratelimit_requests_remain`      | Integer | Remaining available requests in the current time window (hidden if 0)       |
| »`x_gate_ratelimit_limit`                | Integer | Current rate limit cap (hidden if 0)                    |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)     |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | If this is ack response, result is the payload of the request, otherwise result is the response of the api                                                                                                                                                                                                           |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                          |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                  |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                               |

</Block>

<Block>

## Order Batch Place

`futures.order_batch_place`

You can batch place orders with this channel and event `api`.

**function as api below:**

```json
POST /futures/{settle}/batch_orders
```

</Block>

<Block>

### Order Batch Place Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                                                      |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id`                               |
| `req_param` | `object` | Yes      | Api order model's json byte data, an array with api order model; api order model detail to [api](https://www.gate.com/docs/developers/apiv4/en/#create-a-batch-of-futures-orders) |
| `req_header`   | `object` | No      | Apiv4 custom header                     |

`req_param` The JSON byte data for the API order model can refer to a single order placement, or an array of multiple single orders. For details, refer to the order placement documentation.

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

</Block>

<Block>

### Order Batch Place Ack Notification

<Example>

Ack Notification example

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

### Order Batch Place Notification

<Example>

Response Notification example

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

Return of order information for batch placing order

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                           |
| ---------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                      |
| `ack`            | Bool   | The return of the "ack" message represents the WebSocket confirmation message(Currently exists in the order-place api).<br /> If "ack" is false(this field will not be returned), it means that the message is a response message, and you can determine if the request was successful <br />by checking "data.errs". |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                    |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                            |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                       |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                         |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                      |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| »`x_gate_ratelimit_requests_remain`      | Integer | Remaining available requests in the current time window (hidden if 0)       |
| »`x_gate_ratelimit_limit`                | Integer | Current rate limit cap (hidden if 0)                    |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)     |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | If this is ack response, result is the payload of the request, otherwise result is the response of the api                                                                                                                                                                                                            |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                           |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                   |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                                |

</Block>

<Block>

## Order Cancel

`futures.order_cancel`

You can cancel an open order with this channel and event `api`.

**function as api below:**

```json
DELETE /futures/{settle}/orders/{order_id}
```

</Block>

<Block>

### Order Cancel Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Order cancel request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                        |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `object` | Yes      | Api cancel order, detail to [api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order-2)                                           |
| `req_header` | `object` | No      | apiv4 custom header  |

`req_param` API order model JSON data:

| Field       | Type     | Required | Description                                                                          |
| ----------- | -------- | -------- | ------------------------------------------------------------------------------------ |
| `order_id`  | `string` | Yes      | Order ID returned when the order was successfully created, or user specified custom ID (i.e. `text` field). |

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

</Block>

<Block>

### Order Status Notification

<Example>

Order status notification example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                          |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                     |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                   |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                           |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                      |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                        |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                     |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| »`x_gate_ratelimit_requests_remain`      | Integer | Remaining available requests in the current time window (hidden if 0)       |
| »`x_gate_ratelimit_limit`                | Integer | Current rate limit cap (hidden if 0)                    |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)     |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | Single order cancel response, detail to [api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-single-order-2)                                                                                                                                                                                                 |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                          |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                  |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                               |

</Block>

<Block>

## Order Cancel All With Id List

You can cancel all open orders with this channel `futures.order_cancel_ids`.

Multiple distinct order IDs can be specified. Each request can cancel a maximum of 20 records.

**function as api below:**

```json
POST /futures/{settle}/batch_cancel_orders
```

</Block>

<Block>

### Order Cancel All With Id List Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Client request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                        |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `array` | Yes      | Order ID List                                             |
| `req_header` | `object` | No      | apiv4 custom header  |

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

</Block>

<Block>

### Order Cancel All With Id List Notification

<Example>

Order cancel notification example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                     |
| ---------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                |
| `header`         | Map    | response meta info                                                                                                                                                                                                                                                                                              |
| »`response_time` | String | response send time in mill                                                                                                                                                                                                                                                                                      |
| »`channel`       | String | request channel                                                                                                                                                                                                                                                                                                 |
| »`event`         | String | request event                                                                                                                                                                                                                                                                                                   |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | response detail to[api](https://www.gate.com/docs/developers/apiv4/en/#cancel-a-batch-of-orders-with-an-id-list-2)                                                                                                                                                                                                 |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                     |
| »»`label`        | String | denotes error type in string format                                                                                                                                                                                                                                                                             |
| »»`message`      | String | detailed error message                                                                                                                                                                                                                                                                                          |

</Block>

<Block>

## Cancel matched open orders

`futures.order_cancel_cp`

You can cancel all matched open orders through this channel.

**This channel has the same function as the following APIV4:**

```json
DELETE /futures/{settle}/orders
```

</Block>

<Block>

### Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Client request example

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

`payload` parameters:

| Field       | Type     | Required | Description                                                                                                                                    |
| ----------- | -------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `object` | Yes      | Details refer to [api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-matched)                                         |
| `req_header` | `object` | No      | APIV4 custom request header                                                                                                                   |

`req_param` API order model JSON byte data:

| Field       | Type     | Required | Description                                                                                    |
| ----------- | -------- | -------- | ---------------------------------------------------------------------------------------------- |
| `contract`  | `string` | Yes      | Contract                                                                                        |
| `side`     | `string` | No       | All bids or asks. If not specified, both are included.                                        |

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

</Block>

<Block>

### Response result

<Example>

Order cancel response example

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

Response parameters:

| Field                                 | Type    | Description                                                                                                                                                                                                                                                                    |
| ------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`                         | String  | Corresponding request ID                                                                                                                                                                                                                                                      |
| `header`                             | Map     | Response meta information                                                                                                                                                                                                                                                     |
| »`response_time`                     | String  | Response send time (in milliseconds)                                                                                                                                                                                                                                           |
| »`channel`                           | String  | Request channel                                                                                                                                                                                                                                                                |
| »`event`                             | String  | Request `event`                                                                                                                                                                                                                                                                |
| »`client_id`                         | String  | Unique client ID                                                                                                                                                                                                                                                               |
| »`x_in_time`                         | Integer | Time when ws receives the request (in microseconds)                                                                                                                                                                                                                            |
| »`x_out_time`                        | Integer | Time when ws returns the response (in microseconds)                                                                                                                                                                                                                           |
| »`conn_id`                           | String  | Connection ID established with the client (remains consistent for the same connection)                                                                                                                                                                                         |
| »`conn_trace_id`                     | String  | TraceId to establish connection with client                                                                                                                                                                                                                                    |
| »`trace_id`                          | String  | TraceId for executing order operation                                                                                                                                                                                                                                          |
| »`x_gate_ratelimit_requests_remain`  | Integer | Remaining available requests in the current time window (hidden if 0)                                                                                                                                                                                                          |
| »`x_gate_ratelimit_limit`           | Integer | Current rate limit cap (hidden if 0)                                                                                                                                                                                                                                          |
| »`x_gat_ratelimit_reset_timestamp`   | Integer | If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)                |
| `data`                               | Object  | Response data of the request                                                                                                                                                                                                                                                    |
| »`result`                            | Object  | Details refer to [api](https://www.gate.com/docs/developers/apiv4/en/#cancel-all-open-orders-matched)                                                                                                                                                                         |
| »`errs`                              | Object  | Only available when the request fails                                                                                                                                                                                                                                           |
| »»`label`                            | String  | Error type                                                                                                                                                                                                                                                                      |
| »»`message`                          | String  | Detailed error message                                                                                                                                                                                                                                                          |

</Block>

<Block>

## Order Amend

`futures.order_amend`

You can amend an open order with this channel and event `api`.

**function as api below:**

```
PUT /futures/{settle}/orders/{order_id}
```

</Block>

<Block>

### Order Amend Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Client request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                        |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `object` | Yes      | Api amend order, detail to [api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order-2)                                                   |
| `req_header` | `object` | No      | apiv4 custom header  |

`req_header` Custom header data:

| Field            | Type     | Required | Description                                                                                          |
| --------------- | -------- | ---- | --------------------------------------------------------------------------------------------- |
| `x-gate-exptime`          | `string` | false   | Specifies the expiration timestamp (in milliseconds). If the request time received by the WebSocket is later than the expiration time, the request will be rejected              |

#### Detail descriptions

**size**: New order size, including filled part.

- If new size is less than or equal to filled size, the order will be cancelled.
- Order side must be identical to the original one.
- Close order size cannot be changed.
- For reduce only orders, increasing size may leads to other reduce only orders being cancelled.
- If price is not changed, decreasing size will not change its precedence in order book, while increasing will move it to the last at current price.

</Block>

<Block>

### Order Amend Notification

<Example>

Order amend notification example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                           |
| ---------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                      |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                    |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                            |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                       |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                         |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                      |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| »`x_gate_ratelimit_requests_remain`      | Integer | Remaining available requests in the current time window (hidden if 0)       |
| »`x_gate_ratelimit_limit`                | Integer | Current rate limit cap (hidden if 0)                    |
| »`x_gat_ratelimit_reset_timestamp`      | Integer | If the current rate limit has been exceeded, this indicates the timestamp (in milliseconds) of the next available time window when access can be resumed. If the limit has not been exceeded, the response returns the current server time (in milliseconds)     |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| `data`           | Object | Response data of the request                         |
| »`result`        | Object | Response detail to [api](https://www.gate.com/docs/developers/apiv4/en/#amend-an-order-2)                                                                                                                                                                                                                              |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                           |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                   |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                                |

</Block>

<Block>

## Order List

`futures.order_list`

You can list futures orders with this channel and event `api`.

**function as api below:**

```
GET /futures/{settle}/orders
```

</Block>

<Block>

### Order List Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Client request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                        |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `object` | Yes      | Api list order, detail to [api](https://www.gate.com/docs/developers/apiv4/en/#list-futures-orders)                                                 |

</Block>

<Block>

### Order List Notification

<Example>

Order list notification example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                          |
| ---------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                     |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                   |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                           |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                      |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                        |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                     |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| `data`           | Object | Signature time in seconds                                                                                                                                                                                                                                                                                            |
| »`result`        | Object | Response detail to [api](https://www.gate.com/docs/developers/apiv4/en/#list-futures-orders)                                                                                                                                                                                                                          |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                          |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                  |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                               |

</Block>

<Block>

## Order Status

`futures.order_status`

You can query an order with this channel and event `api`.

**function as api below:**

```
GET /futures/{settle}/orders/{order_id}
```

</Block>

<Block>

### Order Status Request

<Example>

Code example: Login is required before making requests

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

Code example: Login is required before making requests

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
	ws, _, err := websocket.DefaultDialer.Dial((url, http.Header{"X-Gate-Size-Decimal": []string{"1"}})
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

Client request example

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

Payload format:

| Field       | Type     | Required | Description                                                                                                                                        |
| ----------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `req_id`    | `string` | Yes      | Request id which will be sent back by the server to help you identify which request the server responds to,<br />it's different from outside's`id` |
| `req_param` | `object` | Yes      | Detail to [api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order-2)                                                                |

`req_param` API order model JSON data:

| Field       | Type     | Required | Description                                                                          |
| ----------- | -------- | -------- | ------------------------------------------------------------------------------------ |
| `order_id`  | `string` | Yes      | Order ID returned when the order was successfully created, or user specified custom ID (i.e. `text` field). |

</Block>

<Block>

### Order Status Notification

<Example>

Order status notification example

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

Result format:

| Field            | Type   | Description                                                                                                                                                                                                                                                                                                         |
| ---------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `request_id`     | String | Unique identifier of the message                                                                                                                                                                                                                                                                                    |
| `header`         | Map    | Response meta info                                                                                                                                                                                                                                                                                                  |
| »`response_time` | String | Response send time in mill                                                                                                                                                                                                                                                                                          |
| »`channel`       | String | Request channel                                                                                                                                                                                                                                                                                                     |
| »`event`         | String | Request event                                                                                                                                                                                                                                                                                                       |
| »`client_id`     | String | Unique client id                                                                                                                                                                                                                                                                                                    |
| »`x_in_time`     | Integer | time to receive the request (in microseconds)                       |
| »`x_out_time`    | Integer | time to return response (in microseconds)                          |
| »`conn_id`       | String | Connection ID established with the client (remains consistent for the same connection)                                    |
| »`conn_trace_id` | String | TraceId to establish connection with client                          |
| »`trace_id`      | String | TraceId for executing order operation                               |
| `data`           | Object | Signature time in seconds                                                                                                                                                                                                                                                                                           |
| »`result`        | Object | Response detail to [api](https://www.gate.com/docs/developers/apiv4/en/#get-a-single-order-2)                                                                                                                                                                                                                        |
| »`errs`          | Object | It is only available when the request fails                                                                                                                                                                                                                                                                         |
| »»`label`        | String | Denotes error type in string format                                                                                                                                                                                                                                                                                 |
| »»`message`      | String | Detailed error message                                                                                                                                                                                                                                                                                              |

</Block>
