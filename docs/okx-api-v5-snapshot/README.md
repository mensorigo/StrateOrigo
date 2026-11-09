# OKX API v5 本地快照

- 来源：`https://www.okx.com/docs-v5/en/`（官方英文版，Firecrawl 全页抓取，SPA 完整渲染）
- 抓取日期：2026-11-09
- 文件：`okx-api-v5-en.md`（约 185 万字符，含全部 REST / WS / SBE 接口、错误码附录）
- SHA256：`1b7913c96b32ee2338e66d2b0650747c61efd197afca9ec779a254a1893a9266`
- 中文版（`/docs-v5/zh/`）同一时间抓取时正文中文出现编码乱码（抓取链路问题），故以英文版为准；如需中文，以本快照对照官网在线版。

## 用途

按 `CLAUDE.md` 规则：写 `so-adapter-okx` 之前先把当前官方文档整理到 `docs/venues/okx.md`（工程速查），
本目录是它的可追溯依据（等价于 Gate 的 `docs/gate-api-v4.106.179-markdown/`）。

## 快照结构速查（字符偏移，便于定位）

| 主题 | 位置（约） |
|---|---|
| REST 认证与签名 | 2,985 |
| WS 通用（连接/订阅/心跳/升级通知） | 4,859 |
| 限频总则、子账户限频、成交率限频 | 23,464 |
| Market Data REST（tickers/深度/K线/逐笔） | 931,412 |
| WS 行情频道（tickers/K线/trades/深度） | 962,790 – 995,824 |
| Public Data REST（instruments/资金费/标记价/涨跌幅） | 123,2079 |
| WS 公共数据频道（instruments/OI/资金费/标记价/强平/ADL） | 1,329,397 – 1,397,986 |
| Trade REST（下单/撤单/改单/Cancel All After） | 325,425 – 484,951 |
| WS 交易（下单/撤单/改单/订单与成交频道） | 484,951 – 559,221 |
| SBE 行情 | 999,544 |
| REST 错误码附录（50000–59999） | 1,735,183 |
| WS 错误码附录（60000–64010 + close frame） | 1,846,287 |
