---
title: Gate for AI
pageClass: agent-page
---

<Block>

# Gate for AI Developer Guide

## Introduction

Gate for AI is a crypto-native infrastructure for AI Agents, connecting AI Agents to the Gate cryptocurrency trading ecosystem through standardized MCP (Model Context Protocol) and CLI tools.

It adopts a **four-layer architecture**:

- **Transport Layer**: Streamable HTTP + SSE, providing standardized communication protocol
- **Protocol Layer**: MCP service layer — multiple endpoints covering market data, trading, DEX, token info, and news
- **Capability Layer**: AI Skills, 40+ pre-built skills that encapsulate multi-step workflows for high-frequency trading scenarios
- **Integration Layer**: Adapts mainstream AI clients like Cursor, Claude, Codex through MCP; supports script automation and backend service integration through CLI

</Block>

<Block>

## Access Methods

Gate for AI provides multiple access methods:

| Access Method | Description | Authentication |
|--------------|-------------|----------------|
| MCP (Remote) | Cloud-hosted (api.gatemcp.ai), no deployment needed, just add Server URL to MCP-compatible client | OAuth |
| MCP (Local) | Run MCP Server locally, connect to local AI client | API Key |
| CLI | Command-line tool, suitable for local debugging and script automation | API Key |

**Skills** are capability modules built on top of the MCP protocol layer, encapsulating multiple atomic tool calls into workflows with business semantics for direct Agent orchestration.

</Block>

<Block>

## Core Capabilities

- **Market Data** — Real-time quotes, K-lines, depth for Spot / Futures / Options / Delivery
- **CEX Trading** — Spot, USDT Perpetual, Options, Copy Trading, Flash Swap, Alpha
- **DEX Swap** — Multi-chain EVM + Solana, cross-chain swaps
- **Wallet Management** — Asset queries, transfers, DApp interactions
- **News & Info** — Real-time news, announcements, social sentiment
- **Token Research** — Fundamentals, technical analysis, on-chain data, security audits

**GitHub Repositories**:
- MCP: [https://github.com/gate/gate-mcp](https://github.com/gate/gate-mcp)
- Skills: [https://github.com/gate/gate-skills](https://github.com/gate/gate-skills)

</Block>

<Block>

## Quick Start

### One-Click Installation

In any AI client that supports Skills, input one sentence to automatically configure all Gate MCPs and Skills:

```
Please auto-configure Gate Skills and MCPs: https://github.com/gate/gate-skills
```

- Auto-detect client type (Cursor / Claude Code / Codex)
- Auto-install all 41 Skills + all MCP endpoints
- No manual configuration file editing needed

![One-Click Installation Demo](/docs/developers/images/agent/quickstart-install.png)

### Manual MCP Client Configuration

**Step 1 — Connect**

No installation needed, just add the URL to your AI client configuration:

| Client | Configuration Method |
|--------|---------------------|
| Cursor | Settings → MCP → Add Remote MCP, URL: `https://api.gatemcp.ai/mcp` |
| Claude.ai | Settings → Integrations → Add MCP Server, URL: `https://api.gatemcp.ai/mcp` |
| Claude Code | `claude mcp add gate-mcp --transport http https://api.gatemcp.ai/mcp` |
| Codex | Add to `.codex/config.json`: `"gate-mcp": {"url": "https://api.gatemcp.ai/mcp"}` |
| ChatGPT | Settings → MCP → Add Server → Enter `https://api.gatemcp.ai/mcp` |

**Note**:
- If using **Remote MCP**, no dependencies or API Key needed for all above configurations.
- If using **Local MCP**, read-only interfaces (market data, news) require no authentication. For trading and account operations, obtain credentials from [Gate API Management](https://www.gate.io/myaccount/api_key_manage) and configure `GATE_API_KEY` / `GATE_API_SECRET` environment variables.

**Step 2 — Try It Out**

After connecting, query market data using natural language:

```
What's the BTC price now?
How's the ETH/USDT 4-hour K-line?
What's the SOL funding rate?
```

All 58 market tools on `/mcp` endpoint require no authentication.

**Step 3 — Enable Trading**

Switch endpoint from `/mcp` to `/mcp/exchange` to unlock 400+ trading tools:

```
https://api.gatemcp.ai/mcp/exchange
```

- For Remote MCP, OAuth2 triggers automatically on first trading tool call, browser pops up authorization page
- After authorization completes, auto-connects without manual API Key entry
- No need to create API Key or copy-paste

If you prefer traditional API Key approach, run locally:

```bash
npx -y gate-mcp
```

- Suitable for scenarios requiring full key control
- Supports offline / intranet environments
- Requires Node.js environment

</Block>

<Block>

## MCP Endpoints

**Base URL**: `https://api.gatemcp.ai`  
**Transport Protocol**: Streamable HTTP + SSE

| Endpoint | Tools | Authentication | Description |
|----------|-------|----------------|-------------|
| `/mcp` | 58 | None | All-category market data (Spot/Futures/Options/Delivery/Earn/Alpha) |
| `/mcp/exchange` | 400+ | OAuth2 | Full CEX trading (Spot/Futures/Options/Flash Swap/Copy Trading/Sub-accounts) |
| `/mcp/dex` | 33 | Google / Gate OAuth | DEX Swap, wallet management, DApp interactions |
| `/mcp/info` | 10 | None | Token info, technical analysis, on-chain data, security audits |
| `/mcp/news` | 3 | None | News search, exchange announcements, social sentiment |


### Market (No Authentication Required)

**Spot Market**

| Tool Name | Description |
|-----------|-------------|
| cex_spot_list_currencies | Get all currency information |
| cex_spot_get_currency | Get single currency details |
| cex_spot_list_currency_pairs | Get all spot trading pair information |
| cex_spot_get_currency_pair | Get single trading pair details |
| cex_spot_get_spot_tickers | Get all or single spot tickers |
| cex_spot_get_spot_order_book | Get spot depth / order book |
| cex_spot_get_spot_trades | Get latest trades |
| cex_spot_get_spot_candlesticks | Get spot K-line data |

**USDT Perpetual Futures Market**

| Tool Name | Description |
|-----------|-------------|
| cex_fx_list_fx_contracts | List all contracts |
| cex_fx_get_fx_contract | Get single contract details |
| cex_fx_get_fx_tickers | Get contract tickers |
| cex_fx_get_fx_order_book | Get contract order book depth |
| cex_fx_get_fx_trades | Get latest trades |
| cex_fx_get_fx_candlesticks | Get K-lines |
| cex_fx_get_fx_funding_rate | Get current funding rate |
| cex_fx_list_fx_batch_funding_rates | Batch get funding rates |
| cex_fx_get_fx_premium_index | Get premium index |
| cex_fx_list_fx_insurance_ledger | Insurance fund ledger |
| cex_fx_list_fx_liq_orders | Liquidation history |
| cex_fx_get_index_constituents | Index constituents |
| cex_fx_list_contract_stats | Contract statistics |
| cex_fx_list_fx_risk_limit_tiers | Risk limit tiers |

**Options Market**

| Tool Name | Description |
|-----------|-------------|
| cex_options_list_options_underlyings | Underlying asset list |
| cex_options_list_options_expirations | Expiration list |
| cex_options_list_options_contracts | All options contracts |
| cex_options_get_options_contract | Single contract details |
| cex_options_list_options_order_book | Depth / order book |
| cex_options_list_options_tickers | All options tickers |
| cex_options_list_options_underlying_tickers | Underlying tickers |
| cex_options_list_options_candlesticks | K-lines |
| cex_options_list_options_underlying_candlesticks | Underlying K-lines |

**Delivery Contracts**

| Tool Name | Description |
|-----------|-------------|
| cex_dc_list_dc_contracts | List all delivery contracts |
| cex_dc_get_dc_contract | Get single contract details |
| cex_dc_list_dc_tickers | Get tickers |
| cex_dc_list_dc_order_book | Get depth |
| cex_dc_list_dc_trades | Get trades |
| cex_dc_list_dc_candlesticks | Get K-lines |
| cex_dc_list_dc_insurance_ledger | Insurance fund ledger |
| cex_dc_list_dc_risk_limit_tiers | Risk limit tiers |

**Earn Products**

| Tool Name | Description |
|-----------|-------------|
| cex_earn_list_uni_currencies | Flexible earn product list |
| cex_earn_list_dual_investment_plans | Dual investment product list |
| cex_earn_list_structured_products | Structured product list |
| cex_earn_list_earn_fixed_term_products | Fixed-term earn product list |
| cex_earn_list_earn_fixed_term_products_by_asset | Fixed-term products by asset |

**Alpha Market**

| Tool Name | Description |
|-----------|-------------|
| cex_alpha_list_alpha_currencies | Alpha supported currencies |
| cex_alpha_list_alpha_tickers | Alpha token tickers |
| cex_alpha_list_alpha_tokens | Alpha token list |

### Spot Trading (OAuth2 Required)

| Tool Name | Description |
|-----------|-------------|
| cex_spot_get_spot_accounts | Query spot account balances |
| cex_spot_create_spot_order | Create spot order |
| cex_spot_cancel_spot_order | Cancel order |
| cex_spot_amend_spot_order | Amend order |
| cex_spot_create_spot_batch_orders | Batch create orders |
| cex_spot_get_spot_order | Query single order |
| cex_spot_list_spot_orders | Order list |
| cex_spot_list_spot_my_trades | Trade history |
| cex_spot_create_spot_price_triggered_order | Price-triggered order |
| cex_spot_list_all_open_orders | All open orders |

### Futures Trading (OAuth2 Required)

| Tool Name | Description |
|-----------|-------------|
| cex_fx_create_fx_order | Create futures order |
| cex_fx_cancel_fx_order | Cancel futures order |
| cex_fx_amend_fx_order | Amend futures order |
| cex_fx_create_fx_batch_orders | Batch create orders |
| cex_fx_cancel_fx_batch_orders | Batch cancel orders |
| cex_fx_amend_fx_batch_orders | Batch amend orders |
| cex_fx_get_fx_order | Query single order |
| cex_fx_list_fx_orders | Query order list |
| cex_fx_list_fx_my_trades | Query trade history |
| cex_fx_get_fx_position | Query one-way position |
| cex_fx_list_fx_positions | Query all positions |
| cex_fx_get_fx_dual_position | Query dual position |
| cex_fx_update_fx_position_leverage | Adjust leverage |
| cex_fx_update_fx_position_margin | Adjust margin |
| cex_fx_set_fx_dual | Set dual position mode |
| cex_fx_create_fx_price_triggered_order | Create conditional order |
| cex_fx_create_trail_order | Create trailing stop order |
| cex_fx_create_fx_bbo_order | Create BBO order |
| cex_fx_get_fx_accounts | Query futures account |

### Account (OAuth2 Required)

| Tool Name | Description |
|-----------|-------------|
| cex_wallet_get_total_balance | Total account assets |
| cex_wallet_create_transfer | Transfer between accounts |
| cex_wallet_list_deposits | Deposit history |
| cex_wallet_list_withdrawals | Withdrawal history |
| cex_wallet_get_deposit_address | Deposit address |
| cex_wallet_list_sa_balances | Sub-account balances |
| cex_unified_get_unified_accounts | Unified account info |
| cex_unified_get_unified_mode | Account mode |
| cex_unified_create_unified_loan | Borrow / loan |
| cex_sa_list_sas | Sub-account list |
| cex_sa_create_sa | Create sub-account |
| cex_sa_lock_sa / cex_sa_unlock_sa | Lock / unlock sub-account |
| cex_account_get_account_detail | Account details |

### DEX (Google / Gate OAuth Required)

| Tool Name | Description |
|-----------|-------------|
| dex_tx_swap | Execute DEX token swap |
| dex_tx_quote | Get swap quote |
| dex_token_list_swap_tokens | Supported swap tokens |
| dex_wallet_get_addresses | Wallet address list |
| dex_tx_transfer_preview + dex_tx_send_raw_transaction | On-chain transfer |
| dex_wallet_get_token_list | Query wallet balances |
| dex_token_get_coin_info | On-chain token info |
| dex_token_get_risk_info | Token security audit |
| dex_tx_list / dex_tx_history_list | Transaction history |
| dex_dapp_connect | Wallet connection |

### Info (No Authentication Required)

| Tool Name | Description |
|-----------|-------------|
| info_coin_get_coin_info | Get token basic info (intro/market cap/supply) |
| info_markettrend_get_technical_analysis | Get technical analysis indicators |
| info_markettrend_get_kline | Get specified token K-line data |
| info_markettrend_get_indicator_history | Get technical indicator history |
| info_marketsnapshot_get_market_snapshot | Get market snapshot |
| info_marketsnapshot_get_market_overview | Get market overview |
| info_coin_get_coin_rankings | Get token rankings |
| info_onchain_get_address_info | Query address info |
| info_onchain_get_token_onchain | Query token on-chain data |
| info_compliance_check_token_security | Token security check |

### News (No Authentication Required)

| Tool Name | Description |
|-----------|-------------|
| news_feed_search_news | Search cryptocurrency news |
| news_feed_get_exchange_announcements | Get exchange announcements (listing/delisting/maintenance) |
| news_feed_get_social_sentiment | Get social media sentiment analysis |

Note: All categories include but are not limited to the above tools; see [Gate MCP](https://github.com/gate/gate-mcp) for details.

</Block>

<Block>

## CLI Tool

### Installation

Download the binary file for your platform from [GitHub Releases](https://github.com/gate/gate-cli/releases), extract and move to system PATH. No additional dependencies required.

For macOS (Apple Silicon):

```bash
curl -Lo gate-cli.tar.gz https://github.com/gate/gate-cli/releases/download/v0.2.2/gate-cli_0.2.2_darwin_arm64.tar.gz
tar xf gate-cli.tar.gz
mv ./gate-cli /usr/local/bin/
```

For other platforms, please visit the [Releases page](https://github.com/gate/gate-cli/releases) to download the corresponding files.

### Common Command Examples

CEX market and trading commands are grouped under `gate-cli cex` (for example `gate-cli cex spot ...` and `gate-cli cex futures ...`). Run `gate-cli cex --help` for the full command tree.

**Market Queries (No Authentication)**:

```bash
# Get BTC/USDT real-time ticker
gate-cli cex spot market ticker --pair BTC_USDT

# Get ETH 4-hour K-lines (latest 100)
gate-cli cex spot market candlesticks --pair ETH_USDT --interval 4h --limit 100

# Get BTC perpetual funding rate (USDT settle is default; add --settle btc for inverse if needed)
gate-cli cex futures market funding-rate --contract BTC_USDT
```

**Trading Operations (Requires Authorization)**:

```bash
# Spot limit buy 0.01 BTC
gate-cli cex spot order buy --pair BTC_USDT --price 65000 --amount 0.01

# Query account balance
gate-cli cex spot account list

# Query futures positions
gate-cli cex futures position list --settle usdt
```

### Pipeline Support

Gate CLI supports machine-readable output via `--format json` for use with jq, python3, and other tools.

```bash
# Get BTC price and extract with jq
gate-cli cex spot market ticker --pair BTC_USDT --format json | jq '.last'

# Batch get multiple token quotes and process with Python
gate-cli cex spot market tickers --format json | python3 -c "
import sys, json
data = json.load(sys.stdin)
for t in data:
    if t['currency_pair'] in ['BTC_USDT','ETH_USDT','SOL_USDT']:
        print(f\"{t['currency_pair']}: {t['last']}\")
"

# Export K-line data to CSV
gate-cli cex spot market candlesticks --pair BTC_USDT --interval 1d --limit 30 --format json | jq -r '.[] | [.[0],.[1],.[2],.[3],.[4],.[5]] | @csv' > btc_daily.csv
```

</Block>

<Block>

## Skills

### One-Click Installation

In clients that support Skills, input:

```
Please auto-configure Gate Skills and MCPs: https://github.com/gate/gate-skills
```

- Auto-install all Skills
- Supports Cursor / Claude Code / Codex

### All Skills Overview

**Trading**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-exchange-spot | Spot trading: buy/sell, conditional orders, TP/SL, amend, cancel | OAuth2 |
| gate-exchange-futures | USDT perpetual: open/close, conditional orders, TP/SL | OAuth2 |
| gate-exchange-alpha | Alpha token discovery, market view, trading, account & order management | OAuth2 |
| gate-exchange-tradfi | TradFi traditional finance: stocks/forex/commodities trading, MT5 account | OAuth2 |
| gate-exchange-flashswap | Flash swap: one-to-one, one-to-many, many-to-one, pair support, quotes, order history | OAuth2 |
| gate-exchange-trading-copilot | End-to-end trading copilot: market judgment, risk control, execution, spot/futures full flow | OAuth2 |

**Account & Assets**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-exchange-assets | Asset & balance queries: total, spot, futures, margin, options, earn, alpha, tradfi | OAuth2 |
| gate-exchange-assets-manager | Multi-account asset overview, margin risk, SimpleEarn yield, affiliate commission, unified account lending & leverage config | OAuth2 |
| gate-exchange-unified | Unified account ops: asset equity, loan limits, borrow/repay, mode switching, leverage & collateral config | OAuth2 |
| gate-exchange-subaccount | Sub-account management: query, create, lock, unlock | OAuth2 |
| gate-exchange-transfer | Intra-UID transfers: spot, futures, margin account fund transfers | OAuth2 |

**Earn & Staking**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-exchange-simpleearn | Simple Earn flexible (Uni) and fixed: product list, holdings, subscribe, redeem, interest query | OAuth2 |
| gate-exchange-staking | On-chain staking: stake, redeem, mint, holdings, yield | OAuth2 |
| gate-exchange-launchpool | LaunchPool staking airdrop: project list, stake, redeem, stake records, airdrop history | OAuth2 |
| gate-exchange-dual | Dual investment: product query, settlement simulation, order, position management | OAuth2 |
| gate-exchange-collateralloan | Multi-currency collateral loan: query loans, fixed rate, repay, adjust collateral | OAuth2 |

**Marketing & Benefits**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-exchange-activitycenter | Platform activity center: activity recommendations, types, my activities | OAuth2 |
| gate-exchange-welfare | Newcomer welfare center: newcomer tasks, claim rewards | OAuth2 |
| gate-exchange-coupon | Coupon management: available coupons, details, usage rules, source tracking | OAuth2 |
| gate-exchange-affiliate | Affiliate program: commission query, volume, customer count, eligibility | OAuth2 |
| gate-exchange-vipfee | VIP level & fee query: VIP level, spot fees, futures fees | OAuth2 |
| gate-exchange-referral | Referral activity recommendations & rules: activities, rewards, commission, coupons, referral links | None |

**Tools & Utilities**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-exchange-marketanalysis | Market analysis: liquidity, depth, slippage simulation, buy/sell pressure, liquidation analysis, funding arbitrage, basis, manipulation risk, orderbook reading | OAuth2 |
| gate-exchange-kyc | KYC guidance: identity verification entry, why can't withdraw | None |
| gate-exchange-crossex | Cross-exchange ops: Gate/Binance/OKX/Bybit orders, transfers, swaps | OAuth2 |

**DEX & Wallet**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-dex-market | Read-only market data: token list, info, holders, rankings, new tokens, security audit, volume, K-lines, liquidity events | None |
| gate-dex-trade | Swap execution: quote, authorize, build, sign, submit, status query | Google/Gate OAuth |
| gate-dex-wallet | Wallet account management: Google/Gate OAuth, balance query, address retrieval, transfer, tx history, x402 payment, DApp interaction, CLI tools | Google/Gate OAuth |

**Market Info**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-info-research | Market research copilot (L2 composite skill): orchestrates 12 read-only MCP tools, outputs structured market briefs, single-token deep analysis, multi-token comparison, trend analysis, event attribution, risk checks | None |
| gate-info-coinanalysis | Single token comprehensive analysis: use only when user asks about one token with no other dimensions | None |
| gate-info-trendanalysis | Trend & technical analysis: use only when user asks about technical indicators/trends with no other dimensions | None |
| gate-info-riskcheck | Token & address risk assessment: use only when user asks about security with no other dimensions | None |
| gate-info-marketoverview | Market overview: use only when user asks about overall market without specific token analysis | None |
| gate-info-coincompare | Token comparison: comparative analysis of multiple tokens | None |
| gate-info-addresstracker | Address tracking & analysis: address info, transactions, fund flow tracking | None |
| gate-info-liveroomlocation | Livestream & replay list: find streams/replays by business type (tag), token, sorting | None |

**News**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-news-briefing | News brief: use only when user asks about recent news/headlines with no other analysis dimensions | None |
| gate-news-eventexplain | Event attribution & explanation: use only when user asks about price anomaly reasons with no other analysis dimensions | None |
| gate-news-listing | Exchange listing tracker: listing, delisting, maintenance announcements | None |

**Installers**

| Skill Name | Description | Auth |
|-----------|-------------|------|
| gate-mcp-cursor-installer | Cursor one-click install Gate MCP + Skills | None |
| gate-mcp-claude-installer | Claude Code one-click install | None |
| gate-mcp-codex-installer | Codex one-click install | None |
| gate-mcp-openclaw-installer | OpenClaw universal installer | None |

</Block>

<Block>

## Use Cases & Workflows

The following 8 complete use cases cover full scenarios from unauthenticated market queries to on-chain Swaps, each including user prompts, endpoints, tools/Skills, execution flow, and expected results.

### Use Case 1: Market Dashboard

| Item | Content |
|------|---------|
| **Authentication** | None |
| **User Prompt** | BTC ETH SOL real-time prices |
| **Endpoint** | `/mcp` |
| **Tools / Skills** | cex_spot_get_spot_tickers |

**Execution Flow:**

1️⃣ Agent parses user intent, extracts token list [BTC, ETH, SOL]  
2️⃣ Calls cex_spot_get_spot_tickers to get full spot market data  
3️⃣ Filters BTC_USDT, ETH_USDT, SOL_USDT trading pairs  
4️⃣ Extracts latest price, 24h change, 24h volume  
5️⃣ Formats and outputs market dashboard table

**Expected Result:**

```
Token  Price (USDT)   24h Change  24h Volume
BTC    67,234.50      +2.31%      12,345 BTC
ETH    3,456.78       +1.85%      89,012 ETH
SOL    178.92         +5.67%      2,345,678 SOL
```

![Use Case 1](/docs/developers/images/agent/usecase-1.png)

---

### Use Case 2: Technical Analysis Report

| Item | Content |
|------|---------|
| **Authentication** | None |
| **User Prompt** | ETH multi-timeframe technical analysis |
| **Endpoint** | `/mcp/info` |
| **Tools / Skills** | info_markettrend_get_technical_analysis<br>info_markettrend_get_kline<br>info_markettrend_get_indicator_history<br>info_marketsnapshot_get_market_snapshot |

**Execution Flow:**

1️⃣ Calls info_marketsnapshot_get_market_snapshot to get ETH real-time snapshot (price, market cap, volume)  
2️⃣ Calls info_markettrend_get_kline to get 1h / 4h / 1d K-lines  
3️⃣ Calls info_markettrend_get_indicator_history to get RSI, MACD, Bollinger Bands history  
4️⃣ Calls info_markettrend_get_technical_analysis to get comprehensive technical analysis rating  
5️⃣ Agent synthesizes multi-timeframe data, generates structured technical analysis report

**Expected Result:**

Complete technical analysis report including multi-timeframe trend judgment (1h bullish / 4h consolidation / 1d bullish), key support/resistance levels, RSI overbought/oversold status, MACD golden/death cross signals, and comprehensive recommendations.

![Use Case 2](/docs/developers/images/agent/usecase-2.png)

---

### Use Case 3: Spot Trading Loop

| Item | Content |
|------|---------|
| **Authentication** | OAuth2 |
| **User Prompt** | Analyze BTC, buy 500U if suitable |
| **Endpoint** | `/mcp` + `/mcp/exchange` |
| **Tools / Skills** | gate-exchange-trading-copilot Skill (orchestrates multiple tools) |

**Execution Flow:**

1️⃣ **Analysis Phase** — Calls info endpoint to get BTC technical analysis, market snapshot, news sentiment  
2️⃣ **Risk Control Phase** — Evaluates current volatility, funding rate, liquidation heatmap, judges if suitable for entry  
3️⃣ **Draft Phase** — If suitable, generates order draft: market buy 500 USDT worth of BTC  
4️⃣ **Confirmation Phase** — Displays order details to user (estimated quantity, fees, slippage), waits for confirmation  
5️⃣ **Execution Phase** — After user confirms, calls cex_spot_create_spot_order to execute market buy  
6️⃣ **Report Phase** — Queries fill results, outputs actual fill price, quantity, fees

**Expected Result:**

Agent outputs complete analysis→risk control→execution→report chain, user sees decision basis throughout, completes trade after final confirmation.

![Use Case 3](/docs/developers/images/agent/usecase-3.png)

---

### Use Case 4: Futures Opening

| Item | Content |
|------|---------|
| **Authentication** | OAuth2 |
| **User Prompt** | BTC perpetual long 1 contract 10x |
| **Endpoint** | `/mcp/exchange` |
| **Tools / Skills** | gate-exchange-futures Skill |

**Execution Flow:**

1️⃣ Parses intent: contract = BTC_USDT, direction = long (buy), quantity = 1 contract, leverage = 10x  
2️⃣ Calls cex_fx_get_fx_contract to confirm contract parameters (min order size, face value, maintenance margin rate)  
3️⃣ Calls cex_fx_update_fx_position_leverage to set leverage to 10x  
4️⃣ Calls cex_fx_get_fx_tickers to get current mark price, calculates required margin  
5️⃣ Displays order preview to user (estimated entry price, margin, liquidation price), waits for confirmation  
6️⃣ After user confirms, calls cex_fx_create_fx_order to execute long  
7️⃣ Queries position to confirm successful opening, outputs position details

**Expected Result:**

Successfully opened BTC_USDT perpetual long position 1 contract, 10x leverage, outputs entry price, used margin, estimated liquidation price.

![Use Case 4](/docs/developers/images/agent/usecase-4.png)

---

### Use Case 5: On-Chain Swap

| Item | Content |
|------|---------|
| **Authentication** | Google / Gate OAuth (DEX) |
| **User Prompt** | Swap 0.1 ETH to USDT |
| **Endpoint** | `/mcp/dex` |
| **Tools / Skills** | dex_tx_quote<br>dex_tx_swap |

**Execution Flow:**

1️⃣ Parses intent: sell 0.1 ETH, buy USDT, chain = Ethereum  
2️⃣ Calls dex_tx_quote to get quote (estimated USDT amount, slippage, Gas fee)  
3️⃣ Displays quote details to user: estimated xxx USDT, slippage 0.x%, Gas ≈ $x.xx  
4️⃣ After user confirms, calls dex_tx_swap to execute on-chain swap  
5️⃣ Returns transaction hash, waits for on-chain confirmation  
6️⃣ Outputs final result: actual USDT received, tx hash, blockchain explorer link

**Expected Result:**

On-chain Swap successful, 0.1 ETH → xxx USDT, outputs tx hash and confirmation status.

![Use Case 5](/docs/developers/images/agent/usecase-5.png)

---

### Use Case 6: Token Security Audit

| Item | Content |
|------|---------|
| **Authentication** | None |
| **User Prompt** | Is this contract safe 0x1234...abcd |
| **Endpoint** | `/mcp/info` + `/mcp/dex` |
| **Tools / Skills** | info_compliance_check_token_security<br>dex_token_get_risk_info |

**Execution Flow:**

1️⃣ Parses contract address 0x1234...abcd, identifies chain (auto-detect or user-specified)  
2️⃣ Calls info_compliance_check_token_security for CEX-level security check (blacklist, compliance marks)  
3️⃣ Calls dex_token_get_risk_info for on-chain security audit (honeypot detection, permission concentration, liquidity lock)  
4️⃣ Synthesizes results from both dimensions, generates security score and risk summary  
5️⃣ Outputs risk item list and operation suggestions

**Expected Result:**

```
Security Score: 72/100 (Medium Risk)

Risk Items:
  - Contract owner can pause trading
  - Liquidity not locked
  - Top 10 holder concentration > 60%

Safe Items:
  - Not a honeypot contract
  - Open-source verified
  - No mint backdoor
```

![Use Case 6](/docs/developers/images/agent/usecase-6.png)

---

### Use Case 7: News Event Attribution

| Item | Content |
|------|---------|
| **Authentication** | None |
| **User Prompt** | Why did SOL surge |
| **Endpoint** | `/mcp/news` + `/mcp/info` |
| **Tools / Skills** | gate-news-eventexplain Skill |

**Execution Flow:**

1️⃣ Calls info_marketsnapshot_get_market_snapshot to confirm SOL recent price movement data  
2️⃣ Calls news_feed_search_news to search SOL-related news from past 24-72h  
3️⃣ Calls news_feed_get_social_sentiment to get social media sentiment changes  
4️⃣ Calls info_onchain_get_token_onchain to check on-chain anomalies (large transfers, TVL changes)  
5️⃣ Agent synthesizes news, sentiment, on-chain data, attributes price movement causes  
6️⃣ Outputs event attribution report

**Expected Result:**

Structured event attribution report including: price movement data, trigger events (e.g. ecosystem bullish news/ETF progress/whale buying), market sentiment rating, follow-up focus points.

![Use Case 7](/docs/developers/images/agent/usecase-7.png)

---

### Use Case 8: Cross-Exchange Monitoring

| Item | Content |
|------|---------|
| **Authentication** | OAuth2 |
| **User Prompt** | Check Gate and Binance positions |
| **Endpoint** | `/mcp/exchange` |
| **Tools / Skills** | gate-exchange-crossex Skill |

**Execution Flow:**

1️⃣ Parses intent: cross-exchange position query, targets = Gate + Binance  
2️⃣ Queries Gate and Binance spot + futures positions through CrossEx module  
3️⃣ Unifies position data format (token, quantity, current value, P&L)  
4️⃣ Aggregates cross-exchange total assets  
5️⃣ Outputs cross-exchange position comparison dashboard

**Expected Result:**

```
📊 Cross-Exchange Position Summary

Exchange  Spot Assets (USDT)  Futures Assets (USDT)  Total
Gate      12,345.67           5,678.90                18,024.57
Binance   8,901.23            3,456.78                12,358.01
Total     21,246.90           9,135.68                30,382.58
```

![Use Case 8](/docs/developers/images/agent/usecase-8.png)

</Block>
<Block>

## Security

### Authentication Overview

| Endpoint | Auth Method | Key Storage | Trigger Method |
|----------|------------|-------------|----------------|
| `/mcp` | None | — | — |
| `/mcp/exchange` | OAuth2 | Gate server-side | Browser pops up authorization on first trading tool call |
| `/mcp/dex` | Google / Gate OAuth | Gate server-side | Triggered by dex_auth_google_login_start |
| `/mcp/info` | None | — | — |
| `/mcp/news` | None | — | — |


### OAuth2 Scopes

| Scope | Permission Scope | Example Tools |
|-------|------------------|---------------|
| market | Market data read | cex_spot_get_spot_tickers, cex_fx_get_fx_candlesticks |
| profile | User basic info | cex_unified_get_unified_accounts, VIP level query |
| trade | Spot & futures trading | cex_spot_create_spot_order, cex_fx_create_fx_order |
| wallet | Wallet & deposit/withdrawal | cex_wallet_create_transfer, cex_wallet_get_deposit_address |
| account | Account asset query | cex_wallet_get_total_balance, cex_spot_get_spot_accounts |

Users can select Scopes as needed during authorization, following minimum privilege principle.

### DEX Authentication Flow

```
Step 1  Call dex_auth_google_login_start → Returns Google OAuth authorization URL
Step 2  User completes Google login authorization in browser
Step 3  Call dex_auth_google_login_poll to poll authorization status → Returns session token on success
Step 4  All subsequent DEX tools automatically carry session token → No re-login needed (valid until token expires)
```
### Security Best Practices

- **Minimum Privilege** — Only grant OAuth2 Scopes required for current task, avoid over-authorization
- **Trading Confirmation Gate** — All fund-related operations (orders/transfers/swaps) require explicit user confirmation
- **Zero Local Key Storage** — In OAuth2 mode, API Keys always stored on Gate server, never locally
- **Session Management** — DEX session tokens have expiration time, auto-triggers re-authentication after expiry
- **Circuit Breaker** — Auto-pauses operations on consecutive auth failures or abnormal calls, prompts user to check authorization status

</Block>

<Block>

## FAQ

**Q: What can Gate MCP do?**  
A: 500+ tools covering full crypto scenarios including market queries, spot/futures/options trading, DEX Swap, wallet management, news & info, token research.

**Q: Need to install anything?**  
A: Gate MCP offers Remote (cloud-hosted) and Local deployment options. Both only require adding URL to client, zero installation, zero dependencies.

**Q: Need API Key?**  
A: Market queries require no authentication. For Remote MCP, trading features use one-click OAuth2 authorization via browser popup, no manual API Key creation/entry needed. For Local MCP, read-only scenarios don't need API key configuration; trading operations require API Key and API Secret.

**Q: Which AI clients are supported?**  
A: Cursor, Claude.ai, Claude Code, ChatGPT, Codex, Windsurf, any client supporting Remote MCP (Streamable HTTP).

**Q: Is it secure?**  
A: In OAuth2 mode, API Keys always stay on Gate servers, never stored locally. All trading operations require explicit user confirmation. OAuth2 authorization can be revoked anytime in Gate account settings.

**Q: Is it free?**  
A: MCP service itself is completely free. Trading fees follow Gate standard rates (same as web/app trading).

**Q: How to report issues?**  
A: GitHub Issues: [https://github.com/gate/gate-mcp/issues](https://github.com/gate/gate-mcp/issues), or submit issues in corresponding Skill repositories.

**Q: Are there rate limits?**  
A: Market endpoints follow Gate public API rate limits (typically 300 req/s). Trading endpoints rate-limited per user account, same as API trading. Agent auto-retries on 429 errors.

</Block>

<Block>

## Related Links

- Product Homepage: [https://www.gate.com/en/gate-for-ai](https://www.gate.com/en/gate-for-ai)
- Help Center — Gate for AI Guide: [https://www.gate.com/en/help/gateai/gateforai/50437](https://www.gate.com/en/help/gateai/gateforai/50437)
- GitHub — gate-mcp: [https://github.com/gate/gate-mcp](https://github.com/gate/gate-mcp)
- GitHub — gate-skills: [https://github.com/gate/gate-skills](https://github.com/gate/gate-skills)
- GitHub — gate-for-ai: [https://github.com/gate/gate-for-ai](https://github.com/gate/gate-for-ai)
- API Documentation: [https://www.gate.io/docs/developers/apiv4](https://www.gate.io/docs/developers/apiv4)
- ClawHub (Skill Marketplace): [https://clawhub.ai/u/gate-exchange](https://clawhub.ai/u/gate-exchange)
- npm Package: [https://www.npmjs.com/package/gate-mcp](https://www.npmjs.com/package/gate-mcp)

</Block>
