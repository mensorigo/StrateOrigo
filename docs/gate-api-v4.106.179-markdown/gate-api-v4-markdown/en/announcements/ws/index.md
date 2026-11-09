---
language_tabs:
  - python: Python
  - go: Golang

toc_footers:
  - <a href='https://gate.com/'>Sign Up for Gate</a>

search: true
---

<Block>
# Gate Announcement WebSocket v1.0.0
Gate provides a simple announcement push websocket api. When a specified type of announcement is published, it will be pushed to the subscribed client as soon as possible.

## BaseURL

- `wss://api.gateio.ws/ws/v4/ann`

</Block>

<Block>

# Changelog

**v0.5.4**

2026-10-29

- add `announcement.summary_dividenddistribution` for stock dividend distribution announcements
- include `dividenddistribution` in `announcement.summary_all`
- keep existing type-specific channel protocols unchanged; clients that validate `result.type` must accept the new type

**v0.5.2**

2026-08-27

- add `announcement.summary_all` for subscribing to all announcement types
- add `announcement.summary_ipos` for subscribing to IPOs announcements

**v0.5.1**

2026-07-31

- improve real-time announcement delivery reliability after idle periods
- ensure the initial subscription update uses the latest saved announcement

**v0.5.0**

2026-07-28

- support stock split / reverse split announcements

**v0.2.0**

2024-08-05

- support engine upgrade announcements

**v0.1.0**

2024-05-20

- support precision announcements

**v0.0.9**

2024-04-15

- Support rename announcements

**v0.0.8**

2024-04-01

- Support deposit/withdrawal announcements

**v0.0.6**

2023-07-26

- Release for the first time, supporting Simplified Chinese and English announcement push

</Block>

<Block>

# Announcement Push API

<Example>

```python
import time
import json

# pip install websocket_client
from websocket import create_connection

ws = create_connection("wss://api.gateio.ws/ws/v4/ann")
ws.send(json.dumps({
    "time": int(time.time()),
    "channel": "announcement.summary_listing",
    "event": "subscribe",
    "payload": ["en"]
}))
print(ws.recv())
```

</Example>

<Example>

```go
package main

import (
	"log"
	"time"

	"github.com/gorilla/websocket"
)

type Request struct {
	Time    int64    `json:"time"`
	Channel string   `json:"channel"`
	Event   string   `json:"event"`
	Payload []string `json:"payload"`
}

type Response struct {
	Time    int64  `json:"time"`
	TimeMs  int64  `json:"time_ms"`
	Channel string `json:"channel"`
	Event   string `json:"event"`
	Result  struct {
		Status string `json:"status"`
	} `json:"result"`
}

func main() {
	conn, _, err := websocket.DefaultDialer.Dial("wss://api.gateio.ws/ws/v4/ann", nil)
	if err != nil {
		log.Fatal("Failed to connect to WebSocket server:", err)
	}
	defer conn.Close()

	// build the request body
	request := Request{
		Time:    time.Now().Unix(),
		Channel: "announcement.summary_listing",
		Event:   "subscribe",
		Payload: []string{"cn"},
	}

	// send request
	if err := conn.WriteJSON(request); err != nil {
		log.Fatal("Failed to send WebSocket request:", err)
	}

	// receive response
	var response Response
	if err := conn.ReadJSON(&response); err != nil {
		log.Fatal("Failed to read WebSocket response:", err)
	}

	log.Printf("%#v\n", response)
}

```

</Example>

## Request Fields

| Field   | Type    | Description                                    |
| ------- | ------- | ---------------------------------------------- |
| time    | Integer | Request timestamp                              |
| channel | String  | Announcement channel name                      |
| event   | String  | Event type，`subscribe` or `unsubscribe`       |
| payload | `Array<String>` | Announcement languages: `cn`, `en`, or both |

### Channel Type

- `announcement.ping`: service liveness check
- `announcement.summary_listing`: subscribe listing announcement
- `announcement.summary_delisting`: subscribe delisting announcement
- `announcement.summary_fee`: subscribe fee announcement
- `announcement.summary_etf`: subscribe etf announcement
- `announcement.summary_deposit_withdrawal`: subscribe deposit/withdrawal announcement
- `announcement.summary_rename`: subscribe rename announcement
- `announcement.summary_precision`: subscribe precision announcement
- `announcement.summary_engine_upgrade`: subscribe engine upgrade announcement
- `announcement.summary_stock_split`: subscribe stock split / reverse split announcement
- `announcement.summary_ipos`: subscribe IPOs announcement
- `announcement.summary_dividenddistribution`: subscribe stock dividend distribution announcement
- `announcement.summary_all`: subscribe all current and future announcement types

### Subscribe to IPO announcements

Use `announcement.summary_ipos` to receive IPOs announcements, including subscription openings, allocation or unlock updates, pre-market openings, and rule changes. The payload accepts `cn`, `en`, or both.

This channel follows the existing type-specific announcement channel behavior: for each subscribed language, the server asynchronously sends the currently cached latest IPOs announcement and continues pushing new IPOs announcements as `update` events. Clients should process each `update` independently and must not rely on the arrival order of the cached announcement and an announcement published at the same time. Its response fields are identical to the other type-specific channels, so no client migration is required.

- Subscribe request

```json
{
  "time": 1787191200,
  "channel": "announcement.summary_ipos",
  "event": "subscribe",
  "payload": ["en"]
}
```

- Subscription success

```json
{
  "time": 1787191200,
  "time_ms": 1787191200123,
  "channel": "announcement.summary_ipos",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

- Latest snapshot and subsequent update

```json
{
  "time": 1787191201,
  "time_ms": 1787191201123,
  "channel": "announcement.summary_ipos",
  "event": "update",
  "result": {
    "lang": "en",
    "origin_url": "https://www.gate.com/article/123456",
    "title": "Gate Pre-IPOs announcement",
    "brief": "Pre-IPOs announcement summary",
    "published_at": 1787191200
  }
}
```

### Subscribe to stock dividend distribution announcements

Use the public channel `announcement.summary_dividenddistribution` without authentication to receive announcements classified as `dividenddistribution` by the announcement center. This includes dividends for stocks and ETFs, dividend settlement for stock perpetual contracts, and advance notices or rule announcements in that category. The server uses the announcement category and does not filter by title keywords or stock symbol.

The payload accepts `["cn"]`, `["en"]`, or `["cn", "en"]`. Each subscribed language receives announcements in that language. The channel uses the shared response fields below and does not return `result.type`.

This channel follows the existing type-specific channel behavior: for each subscribed language, the server asynchronously sends the currently cached latest announcement of this type, if available, and continues pushing new announcements as `update` events. No empty snapshot is sent when the cache has no announcement, and the subscription does not retrieve the full history. Clients should process each `update` independently and must not rely on the arrival order of the cached announcement and an announcement published at the same time, or assume exactly-once delivery.

To stop new updates for selected languages, send an `unsubscribe` request with those languages in the payload. Other subscribed languages and channels remain active. A cached update task already started by the subscription may still finish after an unsubscribe.

- Subscribe request (both languages)

```json
{
  "time": 1792627200,
  "channel": "announcement.summary_dividenddistribution",
  "event": "subscribe",
  "payload": ["cn", "en"]
}
```

- Subscription success

```json
{
  "time": 1792627200,
  "time_ms": 1792627200123,
  "channel": "announcement.summary_dividenddistribution",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

- Latest cached announcement or subsequent update (one message per language)

```json
{
  "time": 1792627201,
  "time_ms": 1792627201123,
  "channel": "announcement.summary_dividenddistribution",
  "event": "update",
  "result": {
    "lang": "en",
    "origin_url": "https://www.gate.io/article/123456",
    "title": "Stock dividend distribution announcement",
    "brief": "Stock dividend distribution announcement summary",
    "published_at": 1792627200
  }
}
```

- Unsubscribe from English updates

```json
{
  "time": 1792627300,
  "channel": "announcement.summary_dividenddistribution",
  "event": "unsubscribe",
  "payload": ["en"]
}
```

### Subscribe to all announcement types

Use `announcement.summary_all` when one connection needs every announcement type. The payload accepts `cn`, `en`, or both. The special payload `!all` is not supported by this channel.

After the subscription success response, the server sends the latest announcement across all types for each subscribed language as an `update` event. New announcements are then pushed on the same channel. The aggregate response includes `result.type`, whose current values are `listing`, `delisting`, `fee`, `etf`, `deposit_withdrawal`, `rename`, `precision`, `engine_upgrade`, `stock_split`, `ipos`, and `dividenddistribution`. Newly added announcement types are included automatically.

Stock dividend distribution announcements use `result.type="dividenddistribution"` on this channel. The same announcement can also be received on `announcement.summary_dividenddistribution` when both channels are subscribed; process each channel independently. Existing stock split, IPOs, and other type-specific channels retain their request and response formats and behavior. No protocol migration is required; clients that validate `result.type` against an allowlist must accept `dividenddistribution` and allow for future types.

If one announcement belongs to multiple types, the server sends one update for each type. Update ordering across different types is not guaranteed. Existing type-specific channels and their response fields are unchanged.

To stop updates for selected languages, send an `unsubscribe` request with those languages in the payload. Other languages subscribed on the same connection remain active.

- Subscribe request

```json
{
  "time": 1787104800,
  "channel": "announcement.summary_all",
  "event": "subscribe",
  "payload": ["en"]
}
```

- Subscription success

```json
{
  "time": 1787104800,
  "time_ms": 1787104800123,
  "channel": "announcement.summary_all",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

- Latest snapshot and subsequent update

```json
{
  "time": 1787104801,
  "time_ms": 1787104801123,
  "channel": "announcement.summary_all",
  "event": "update",
  "result": {
    "lang": "en",
    "origin_url": "https://www.gate.io/article/123456",
    "title": "Example announcement",
    "brief": "Example announcement summary",
    "published_at": 1787104700,
    "type": "listing"
  }
}
```

- Unsubscribe from English updates

```json
{
  "time": 1787104900,
  "channel": "announcement.summary_all",
  "event": "unsubscribe",
  "payload": ["en"]
}
```

## Response Fields

| Field        | Type    | Description                                                    |
| ------------ | ------- | -------------------------------------------------------------- |
| time         | Integer | Timestamp                                                      |
| time_ms      | Integer | Timestamp (millisecond)                                        |
| channel      | String  | Channel name                                                   |
| event        | String  | Event type                                                     |
| result       | Object  | Announcement result                                            |
| lang         | string  | Language of announcement                                       |
| origin_url   | string  | Announcement origin url                                        |
| title        | string  | Announcement title                                             |
| brief        | string  | Announcement brief                                             |
| published_at | Integer | Announcement published timestamp                               |
| type         | string  | Announcement type; only returned by `announcement.summary_all` |

<Example>

- Request Example

```json
{
  "time": 1659956033,
  "channel": "announcement.summary_listing",
  "event": "subscribe",
  "payload": ["en"]
}
```

- Response Example

```json
{
  "time": 1690365913,
  "time_ms": 1690365913056,
  "channel": "announcement.summary_listing",
  "event": "update",
  "result": {
    "lang": "en",
    "origin_url": "https://www.gate.com/article/228103",
    "title": "Gate will commence Worldcoin(WLD) Trading at 7:30am UTC 24th July 2023 test",
    "brief": "Gate is going to Gate will commence Worldcoin(WLD) Trading at 7:30am UTC 24th July 2023\r\n\r\nToken symbol：WLD\r\nWebsite：https://worldcoin.org/\r\nToken Contract : Erc20: 0x163f8c2467924be0ae7b5347228cabf260318753\r\nOptimism: 0xdc6ff44d5d932cbd77b52e5612ba0529dc6226f1\r\n\r\nDeposit Worldcoin (WLD) at https://www.gate.com/myaccount/deposit/WLD\r\nTrade Worldcoin (WLD) in USDT market via: https://www.gate.com/trade/WLD_USDT\r\n\r\n\r\nGateway to Crypto\r\nTrade over 1,400 cryptocurrencies safely, quickly and easily on Gate.io\r\n\r\nSign up to enter the crypto gateway and get 40% commission from referrals\r\n\r\nDownload iOS/Android App right now.\r\n\r\nReach out to us!\r\nTwitter: https://twitter.com/gate_io\r\nGateVIP Twitter:https://twitter.com/Gateio_Inst\r\nTelegram: https://t.me/gate_zh\r\nAPI Telegram: https://t.me/gateioapi\r\nInstagram: https://www.instagram.com/gateioglobal\r\nMedium: https://gateio.medium.com/\r\nPodcast: https://gateio.buzzsprout.com/?ch=buzzs\r\n\r\nGate.io Team\r\nJuly 24th, 2023",
    "published_at": 1690365913
  }
}
```

</Example>

</Block>
