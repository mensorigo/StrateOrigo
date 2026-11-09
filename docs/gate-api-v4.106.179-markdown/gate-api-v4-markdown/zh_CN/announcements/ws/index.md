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
Gate 提供一个简单的公告推送接口，当指定类型的公告发布时，会第一时间推送到所订阅的客户端。

## 访问链接

- `wss://api.gateio.ws/ws/v4/ann`

</Block>

<Block>

# Changelog

**v0.5.4**

2026-10-29

- 新增股票股息派发公告频道 `announcement.summary_dividenddistribution`
- 全量公告频道 `announcement.summary_all` 纳入 `dividenddistribution` 类型
- 现有分类频道协议保持不变；校验 `result.type` 的客户端需兼容新增类型

**v0.5.2**

2026-08-27

- 新增全量公告订阅频道 `announcement.summary_all`
- 新增 IPOs 公告订阅频道 `announcement.summary_ipos`

**v0.5.1**

2026-07-31

- 修复服务空闲后新公告无法继续实时推送的问题
- 确保订阅首条更新使用最新保存的公告

**v0.5.0**

2026-07-28

- 支持股票拆合股类型公告推送

**v0.2.0**

2024-08-05

- 支持引擎升级类型公告推送

**v0.1.0**

2024-05-20

- 支持精度类型公告推送

**v0.0.9**

2024-04-15

- 支持更名类型公告推送

**v0.0.8**

2024-04-01

- 支持充提币公告推送

**v0.0.6**

2023-07-26

- 初次上线，支持简体中文和英文公告推送

</Block>

<Block>

# 公告推送

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

	// 构建请求报文
	request := Request{
		Time:    time.Now().Unix(),
		Channel: "announcement.summary_listing",
		Event:   "subscribe",
		Payload: []string{"cn"},
	}

	// 发送请求报文
	if err := conn.WriteJSON(request); err != nil {
		log.Fatal("Failed to send WebSocket request:", err)
	}

	// 接收响应报文
	var response Response
	if err := conn.ReadJSON(&response); err != nil {
		log.Fatal("Failed to read WebSocket response:", err)
	}

	log.Printf("%#v\n", response)
}

```

</Example>

## 请求字段

| 字段名  | 类型    | 描述                                   |
| ------- | ------- | -------------------------------------- |
| time    | Integer | 请求时间戳                             |
| channel | String  | 订阅频道                               |
| event   | String  | 事件类型，`subscribe` 或 `unsubscribe` |
| payload | `Array<String>` | 公告语言：`cn`、`en` 或两者组合       |

### 订阅频道

- `announcement.ping`: 服务存活检查
- `announcement.summary_listing`: 订阅上新的公告
- `announcement.summary_delisting`: 订阅下架的公告
- `announcement.summary_fee`: 订阅费率相关公告
- `announcement.summary_etf`: 订阅 ETF 相关公告
- `announcement.summary_deposit_withdrawal`: 订阅充提币相关公告
- `announcement.summary_rename`: 订阅重命名相关公告
- `announcement.summary_precision`: 订阅精度相关公告
- `announcement.summary_engine_upgrade`: 订阅引擎升级相关公告
- `announcement.summary_stock_split`: 订阅股票拆合股相关公告
- `announcement.summary_ipos`: 订阅 IPOs 相关公告
- `announcement.summary_dividenddistribution`: 订阅股票股息派发相关公告
- `announcement.summary_all`: 订阅当前及后续新增的所有公告类型

### 订阅 IPOs 公告

使用 `announcement.summary_ipos` 可以接收 IPOs 相关公告，包括开放认购、分配或解锁、盘前交易开盘和规则变更等事件。payload 支持 `cn`、`en` 或两者组合。

该频道沿用现有分类公告频道的行为：服务端会针对每个已订阅语言，异步推送当前缓存的最新一条 IPOs 公告，并继续以 `update` 事件推送新的 IPOs 公告。客户端应独立处理每条 `update`，不应依赖缓存公告与订阅同时刻新发布公告的到达顺序。该频道的响应字段与其他分类频道完全一致，现有客户端无需迁移。

- 订阅请求

```json
{
  "time": 1787191200,
  "channel": "announcement.summary_ipos",
  "event": "subscribe",
  "payload": ["cn"]
}
```

- 订阅成功响应

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

- 最新快照及后续更新

```json
{
  "time": 1787191201,
  "time_ms": 1787191201123,
  "channel": "announcement.summary_ipos",
  "event": "update",
  "result": {
    "lang": "cn",
    "origin_url": "https://www.gate.com/article/123456",
    "title": "Gate Pre-IPOs 公告",
    "brief": "Pre-IPOs 公告摘要",
    "published_at": 1787191200
  }
}
```

### 订阅股票股息派发公告

使用公共频道 `announcement.summary_dividenddistribution` 无需鉴权，即可接收公告中心归类为 `dividenddistribution` 的公告，包括真股和 ETF 股息派发、股票永续合约分红结算，以及该分类下的预告或规则公告。服务端依据公告分类推送，不按标题关键词或股票标的过滤。

payload 支持 `["cn"]`、`["en"]` 或 `["cn", "en"]`，每种已订阅语言只接收对应语言的公告。该频道复用下方共享响应字段，不返回 `result.type`。

该频道沿用现有分类频道的行为：服务端会针对每个已订阅语言，异步推送当前缓存的最新一条该类型公告（如有），并继续以 `update` 事件推送新公告。没有缓存公告时不发送空快照，订阅也不会拉取全部历史。客户端应独立处理每条 `update`，不应依赖缓存公告与订阅同时刻新发布公告的到达顺序，也不应假定消息恰好送达一次。

如需停止接收指定语言的新公告，请发送 payload 包含对应语言的 `unsubscribe` 请求。其他已订阅语言及频道不受影响。订阅时已启动的缓存推送任务可能在退订后继续完成。

- 订阅请求（双语）

```json
{
  "time": 1792627200,
  "channel": "announcement.summary_dividenddistribution",
  "event": "subscribe",
  "payload": ["cn", "en"]
}
```

- 订阅成功响应

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

- 最新缓存公告或后续更新（每种语言单独一条消息）

```json
{
  "time": 1792627201,
  "time_ms": 1792627201123,
  "channel": "announcement.summary_dividenddistribution",
  "event": "update",
  "result": {
    "lang": "cn",
    "origin_url": "https://www.gate.io/article/123456",
    "title": "股票股息派发公告",
    "brief": "股票股息派发公告摘要",
    "published_at": 1792627200
  }
}
```

- 退订英文更新

```json
{
  "time": 1792627300,
  "channel": "announcement.summary_dividenddistribution",
  "event": "unsubscribe",
  "payload": ["en"]
}
```

### 订阅所有公告类型

当一个连接需要接收所有类型的公告时，可订阅 `announcement.summary_all`。payload 支持 `cn`、`en` 或两者组合；该频道不支持特殊 payload `!all`。

服务端返回订阅成功响应后，会针对每个已订阅语言，以 `update` 事件推送所有类型中最新的一条公告，之后继续通过同一频道推送新公告。全量频道的响应包含 `result.type`，当前可取 `listing`、`delisting`、`fee`、`etf`、`deposit_withdrawal`、`rename`、`precision`、`engine_upgrade`、`stock_split`、`ipos` 和 `dividenddistribution`；后续新增的公告类型会自动包含在全量频道中。

股票股息派发公告在该频道中使用 `result.type="dividenddistribution"`。同一连接也订阅 `announcement.summary_dividenddistribution` 时，可在两个频道分别收到同一公告，客户端应独立处理各频道消息。现有股票拆合股、IPOs 及其他分类频道的请求、响应格式和行为保持不变，客户端无需迁移协议；对 `result.type` 使用枚举白名单校验的客户端需接纳 `dividenddistribution`，并兼容后续新增类型。

如果一条公告同时属于多个类型，服务端会按类型分别推送一条 update；不同类型之间不保证推送顺序。现有分类频道及其响应字段保持不变。

如需停止接收指定语言的更新，请发送 payload 包含对应语言的 `unsubscribe` 请求。同一连接上订阅的其他语言不受影响。

- 订阅请求

```json
{
  "time": 1787104800,
  "channel": "announcement.summary_all",
  "event": "subscribe",
  "payload": ["cn"]
}
```

- 订阅成功响应

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

- 最新快照及后续更新

```json
{
  "time": 1787104801,
  "time_ms": 1787104801123,
  "channel": "announcement.summary_all",
  "event": "update",
  "result": {
    "lang": "cn",
    "origin_url": "https://www.gate.io/article/123456",
    "title": "公告标题示例",
    "brief": "公告摘要示例",
    "published_at": 1787104700,
    "type": "listing"
  }
}
```

- 退订中文更新

```json
{
  "time": 1787104900,
  "channel": "announcement.summary_all",
  "event": "unsubscribe",
  "payload": ["cn"]
}
```

## 响应字段

| 字段名       | 类型    | 描述                                         |
| ------------ | ------- | -------------------------------------------- |
| time         | Integer | 时间戳                                       |
| time_ms      | Integer | 时间戳（毫秒级）                             |
| channel      | String  | 通道名称                                     |
| event        | String  | 事件类型                                     |
| result       | Object  | 响应结果对象                                 |
| lang         | String  | 语言类型                                     |
| origin_url   | String  | 公告原始链接                                 |
| title        | String  | 标题                                         |
| brief        | String  | 简介                                         |
| published_at | Integer | 发布时间戳                                   |
| type         | String  | 公告类型，仅 `announcement.summary_all` 返回 |

<Example>

- 请求示例

```json
{
  "time": 1659956033,
  "channel": "announcement.summary_listing",
  "event": "subscribe",
  "payload": ["en"]
}
```

- 响应示例

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
