---
title: Gate for AI
pageClass: agent-page
---

<Block>

# Gate for AI 开发者指南

## 简介

Gate for AI 是面向 AI Agent 的加密货币原生基础设施，通过标准化的 MCP（Model Context Protocol）协议与 CLI 工具，将 AI Agent 接入 Gate 加密货币交易生态。

整体采用**四层架构**：

- **传输层**：Streamable HTTP + SSE，提供标准化通信协议
- **协议层**：MCP 服务层 — 多端点，涵盖行情、交易、DEX、币种信息、新闻
- **能力层**：AI Skills，40+ 预置技能，封装高频交易场景的多步骤工作流
- **集成层**：通过 MCP 适配 Cursor、Claude、Codex 等主流 AI 客户端；通过 CLI 支持脚本自动化与后端服务接入

</Block>

<Block>

## 接入方式

Gate for AI 提供多种接入方式：

| 接入方式 | 说明 | 鉴权 |
|---------|---------|------|
| MCP（Remote） | 云端托管（api.gatemcp.ai），无需自行部署，在 MCP 兼容客户端中添加 Server URL 即可连接 | OAuth |
| MCP（Local） | 本地运行 MCP Server，连接至本地 AI 客户端 | API Key |
| CLI | 命令行工具，适合本地调试与脚本自动化 | API Key |

**Skills** 是构建于 MCP 协议层之上的能力模块，将多个原子工具调用封装为具备业务语义的工作流，供 Agent 直接编排调用。

</Block>

<Block>

## 核心能力

- **行情数据** — 现货 / 合约 / 期权 / 交割的实时行情、K 线、深度
- **CEX 交易** — 现货、USDT 永续、期权、跟单、闪兑、Alpha
- **DEX Swap** — EVM 多链 + Solana，跨链兑换
- **钱包管理** — 资产查询、转账、DApp 交互
- **新闻资讯** — 实时新闻、公告、社交情绪
- **币种研究** — 基本面、技术分析、链上数据、安全审计

**GitHub 仓库**：
- MCP：[https://github.com/gate/gate-mcp](https://github.com/gate/gate-mcp)
- Skills：[https://github.com/gate/gate-skills](https://github.com/gate/gate-skills)

</Block>

<Block>

## 快速开始

### 一键安装

在任何支持 Skills 的 AI 客户端中，输入一句话即可自动配置全部 Gate MCP 和 Skills：

```
帮我自动配置 Gate Skills 和 MCPs: https://github.com/gate/gate-skills
```

- 自动识别客户端类型（Cursor / Claude Code / Codex）
- 自动安装全部 41 个 Skills + 所有 MCP 端点
- 无需手动编辑配置文件

![一键安装效果图](/docs/developers/images/agent/quickstart-install.png)

### MCP 客户端手动配置

**Step 1 — 连接**

无需安装，只需将 URL 添加到你的 AI 客户端配置中：

| 客户端 | 配置方式 |
|--------|---------|
| Cursor | Settings → MCP → 添加 Remote MCP，URL 填 `https://api.gatemcp.ai/mcp` |
| Claude.ai | Settings → Integrations → 添加 MCP Server，URL 填 `https://api.gatemcp.ai/mcp` |
| Claude Code | `claude mcp add gate-mcp --transport http https://api.gatemcp.ai/mcp` |
| Codex | `.codex/config.json` 中添加 `"gate-mcp": {"url": "https://api.gatemcp.ai/mcp"}` |
| ChatGPT | Settings → MCP → Add Server → 输入 `https://api.gatemcp.ai/mcp` |

**注意**：
- 如果安装的是 **Remote MCP**，以上所有配置都不需要安装任何依赖，不需要 API Key，可直接使用。
- 如果安装的是 **Local MCP**，行情查询、资讯等只读接口无需任何鉴权。涉及交易执行、账户操作等写权限接口，需提前通过 [Gate API 管理页面](https://www.gate.io/myaccount/api_key_manage) 获取密钥，并配置 `GATE_API_KEY` / `GATE_API_SECRET` 环境变量。

**Step 2 — 试一试**

连接后，直接用自然语言查询行情数据：

```
BTC 现在多少钱？
ETH/USDT 的 4 小时 K 线怎么样？
SOL 的资金费率是多少？
```

`/mcp` 端点的 58 个行情工具全部免认证。

**Step 3 — 开启交易**

将端点从 `/mcp` 切换为 `/mcp/exchange` 即可解锁 400+ 交易工具：

```
https://api.gatemcp.ai/mcp/exchange
```

- Remote MCP 首次调用交易工具时，OAuth2 自动触发，浏览器弹出授权页面
- 授权完成后自动连接，无需手动填写 API Key
- 无需创建 API Key、无需复制粘贴

如果你更倾向于使用传统 API Key，可以本地运行：

```bash
npx -y gate-mcp
```

- 适合需要完全自主控制密钥的场景
- 支持离线 / 内网环境
- 需要 Node.js 环境

</Block>

<Block>

## MCP 端点

**Base URL**: `https://api.gatemcp.ai`  
**传输协议**: Streamable HTTP + SSE

| 端点 | 工具数 | 认证方式 | 说明 |
|------|---------|---------|------|
| `/mcp` | 58 | 无需认证 | 全品类行情数据（现货/合约/期权/交割/理财/Alpha） |
| `/mcp/exchange` | 400+ | OAuth2 | CEX 全功能交易（现货/合约/期权/闪兑/跟单/子账户等） |
| `/mcp/dex` | 33 | Google / Gate OAuth | DEX Swap、钱包管理、DApp 交互 |
| `/mcp/info` | 10 | 无需认证 | 币种信息、技术分析、链上数据、安全审计 |
| `/mcp/news` | 3 | 无需认证 | 新闻搜索、交易所公告、社交情绪 |


### Market（无需认证）

**现货行情**

| 工具名 | 说明 |
|--------|------|
| cex_spot_list_currencies | 获取所有币种信息 |
| cex_spot_get_currency | 获取单个币种详情 |
| cex_spot_list_currency_pairs | 获取所有现货交易对信息 |
| cex_spot_get_currency_pair | 获取单个交易对详情 |
| cex_spot_get_spot_tickers | 获取所有/单个现货行情 |
| cex_spot_get_spot_order_book | 获取现货深度/订单簿 |
| cex_spot_get_spot_trades | 获取最新成交记录 |
| cex_spot_get_spot_candlesticks | 获取现货 K 线数据 |

**永续合约行情**

| 工具名 | 说明 |
|--------|------|
| cex_fx_list_fx_contracts | 获取所有合约列表 |
| cex_fx_get_fx_contract | 获取单个合约详情 |
| cex_fx_get_fx_tickers | 获取合约行情 |
| cex_fx_get_fx_order_book | 获取合约深度 |
| cex_fx_get_fx_trades | 获取最新成交 |
| cex_fx_get_fx_candlesticks | 获取 K 线 |
| cex_fx_get_fx_funding_rate | 获取当前资金费率 |
| cex_fx_list_fx_batch_funding_rates | 批量获取资金费率 |
| cex_fx_get_fx_premium_index | 获取溢价指数 |
| cex_fx_list_fx_insurance_ledger | 保险基金记录 |
| cex_fx_list_fx_liq_orders | 强平历史 |
| cex_fx_get_index_constituents | 指数成分 |
| cex_fx_list_contract_stats | 合约统计数据 |
| cex_fx_list_fx_risk_limit_tiers | 风险限额档位 |

**期权行情**

| 工具名 | 说明 |
|--------|------|
| cex_options_list_options_underlyings | 标的资产列表 |
| cex_options_list_options_expirations | 到期日列表 |
| cex_options_list_options_contracts | 所有期权合约 |
| cex_options_get_options_contract | 单个合约详情 |
| cex_options_list_options_order_book | 深度/订单簿 |
| cex_options_list_options_tickers | 所有期权行情 |
| cex_options_list_options_underlying_tickers | 标的行情 |
| cex_options_list_options_candlesticks | K 线 |
| cex_options_list_options_underlying_candlesticks | 标的 K 线 |

**交割合约行情**

| 工具名 | 说明 |
|--------|------|
| cex_dc_list_dc_contracts | 获取所有交割合约 |
| cex_dc_get_dc_contract | 获取单个合约详情 |
| cex_dc_list_dc_tickers | 获取行情 |
| cex_dc_list_dc_order_book | 获取深度 |
| cex_dc_list_dc_trades | 获取成交记录 |
| cex_dc_list_dc_candlesticks | 获取 K 线 |
| cex_dc_list_dc_insurance_ledger | 保险基金记录 |
| cex_dc_list_dc_risk_limit_tiers | 风险限额档位 |

**理财产品**

| 工具名 | 说明 |
|--------|------|
| cex_earn_list_uni_currencies | 获取活期理财产品列表 |
| cex_earn_list_dual_investment_plans | 获取双币投资产品列表 |
| cex_earn_list_structured_products | 获取结构化产品列表 |
| cex_earn_list_earn_fixed_term_products | 获取定期理财产品列表 |
| cex_earn_list_earn_fixed_term_products_by_asset | 按币种获取定期产品 |

**Alpha 市场**

| 工具名 | 说明 |
|--------|------|
| cex_alpha_list_alpha_currencies | 获取 Alpha 支持币种 |
| cex_alpha_list_alpha_tickers | 获取 Alpha 代币行情 |
| cex_alpha_list_alpha_tokens | 获取 Alpha 代币列表 |

### Spot Trading（需要 OAuth2 认证）

| 工具名 | 说明 |
|--------|------|
| cex_spot_get_spot_accounts | 查询现货账户余额 |
| cex_spot_create_spot_order | 创建现货订单 |
| cex_spot_cancel_spot_order | 取消订单 |
| cex_spot_amend_spot_order | 修改订单 |
| cex_spot_create_spot_batch_orders | 批量创建订单 |
| cex_spot_get_spot_order | 查询单个订单 |
| cex_spot_list_spot_orders | 订单列表 |
| cex_spot_list_spot_my_trades | 成交记录 |
| cex_spot_create_spot_price_triggered_order | 条件单 |
| cex_spot_list_all_open_orders | 所有挂单 |

### Futures Trading（需要 OAuth2 认证）

| 工具名 | 说明 |
|--------|------|
| cex_fx_create_fx_order | 创建合约订单 |
| cex_fx_cancel_fx_order | 取消合约订单 |
| cex_fx_amend_fx_order | 修改合约订单 |
| cex_fx_create_fx_batch_orders | 批量创建订单 |
| cex_fx_cancel_fx_batch_orders | 批量取消订单 |
| cex_fx_amend_fx_batch_orders | 批量修改订单 |
| cex_fx_get_fx_order | 查询单个订单 |
| cex_fx_list_fx_orders | 查询订单列表 |
| cex_fx_list_fx_my_trades | 查询成交记录 |
| cex_fx_get_fx_position | 查询单向持仓 |
| cex_fx_list_fx_positions | 查询所有持仓 |
| cex_fx_get_fx_dual_position | 查询双向持仓 |
| cex_fx_update_fx_position_leverage | 调整杠杆 |
| cex_fx_update_fx_position_margin | 调整保证金 |
| cex_fx_set_fx_dual | 设置双向持仓模式 |
| cex_fx_create_fx_price_triggered_order | 创建条件单 |
| cex_fx_create_trail_order | 创建追踪止损单 |
| cex_fx_create_fx_bbo_order | 创建 BBO 订单 |
| cex_fx_get_fx_accounts | 查询合约账户 |

### Account（需要 OAuth2 认证）

| 工具名 | 说明 |
|--------|------|
| cex_wallet_get_total_balance | 账户总资产 |
| cex_wallet_create_transfer | 账户间划转 |
| cex_wallet_list_deposits | 充值记录 |
| cex_wallet_list_withdrawals | 提现记录 |
| cex_wallet_get_deposit_address | 充值地址 |
| cex_wallet_list_sa_balances | 子账户余额 |
| cex_unified_get_unified_accounts | 统一账户信息 |
| cex_unified_get_unified_mode | 账户模式 |
| cex_unified_create_unified_loan | 借款 |
| cex_sa_list_sas | 子账户列表 |
| cex_sa_create_sa | 创建子账户 |
| cex_sa_lock_sa / cex_sa_unlock_sa | 锁定/解锁子账户 |
| cex_account_get_account_detail | 账户详情 |

### DEX（需要 Google / Gate OAuth 认证）

| 工具名 | 说明 |
|--------|------|
| dex_tx_swap | 执行 DEX 代币兑换 |
| dex_tx_quote | 获取兑换报价 |
| dex_token_list_swap_tokens | 获取支持的代币列表 |
| dex_wallet_get_addresses | 获取钱包地址列表 |
| dex_tx_transfer_preview + dex_tx_send_raw_transaction | 链上转账 |
| dex_wallet_get_token_list | 查询钱包余额 |
| dex_token_get_coin_info | 获取链上代币信息 |
| dex_token_get_risk_info | 代币安全审计 |
| dex_tx_list / dex_tx_history_list | 查询交易历史 |
| dex_dapp_connect | 钱包连接 |

### Info（无需认证）

| 工具名 | 说明 |
|--------|------|
| info_coin_get_coin_info | 获取币种基本信息（简介/市值/供应量） |
| info_markettrend_get_technical_analysis | 获取技术分析指标汇总 |
| info_markettrend_get_kline | 获取指定币种 K 线数据 |
| info_markettrend_get_indicator_history | 获取技术指标历史数据 |
| info_marketsnapshot_get_market_snapshot | 获取市场快照 |
| info_marketsnapshot_get_market_overview | 获取市场总览 |
| info_coin_get_coin_rankings | 获取币种排行榜 |
| info_onchain_get_address_info | 查询地址信息 |
| info_onchain_get_token_onchain | 查询代币链上数据 |
| info_compliance_check_token_security | 代币安全检测 |

### News（无需认证）

| 工具名 | 说明 |
|--------|------|
| news_feed_search_news | 搜索加密货币新闻 |
| news_feed_get_exchange_announcements | 获取交易所公告（上币/下架/维护） |
| news_feed_get_social_sentiment | 获取社交媒体情绪分析 |

注意：所有类别包括但不局限于以上 Tools，详见 [Gate MCP](https://github.com/gate/gate-mcp)。

</Block>

<Block>

## CLI 工具

### 安装

从 [GitHub Releases](https://github.com/gate/gate-cli/releases) 下载对应平台的二进制文件，解压后移至系统 PATH 即可使用，无需额外依赖。

以 macOS（Apple Silicon）为例：

```bash
curl -Lo gate-cli.tar.gz https://github.com/gate/gate-cli/releases/download/v0.2.2/gate-cli_0.2.2_darwin_arm64.tar.gz
tar xf gate-cli.tar.gz
mv ./gate-cli /usr/local/bin/
```

其他平台请前往 [Releases 页面](https://github.com/gate/gate-cli/releases) 下载对应文件。

### 常用命令示例

现货、合约等 CEX 相关子命令已统一挂在 `gate-cli cex` 下（例如 `gate-cli cex spot ...`、`gate-cli cex futures ...`）。可用 `gate-cli cex -h` 查看完整子命令树。

**行情查询（无需认证）**：

```bash
# 获取 BTC/USDT 实时行情
gate-cli cex spot market ticker --pair BTC_USDT

# 获取 ETH 4 小时 K 线（最近 100 根）
gate-cli cex spot market candlesticks --pair ETH_USDT --interval 4h --limit 100

# 获取 BTC 永续合约资金费率（默认 USDT 结算；反向合约可显式加 --settle btc）
gate-cli cex futures market funding-rate --contract BTC_USDT
```

**交易操作（需要授权）**：

```bash
# 现货限价买入 0.01 BTC
gate-cli cex spot order buy --pair BTC_USDT --price 65000 --amount 0.01

# 查询账户余额
gate-cli cex spot account list

# 查询合约持仓
gate-cli cex futures position list --settle usdt
```

### 管道支持

Gate CLI 支持通过 `--format json` 输出机器可读结果，可与 jq、python3 等工具组合使用。

```bash
# 获取 BTC 价格并用 jq 提取
gate-cli cex spot market ticker --pair BTC_USDT --format json | jq '.last'

# 批量获取多币种行情并用 Python 处理
gate-cli cex spot market tickers --format json | python3 -c "
import sys, json
data = json.load(sys.stdin)
for t in data:
    if t['currency_pair'] in ['BTC_USDT','ETH_USDT','SOL_USDT']:
        print(f\"{t['currency_pair']}: {t['last']}\")
"

# 导出 K 线数据为 CSV
gate-cli cex spot market candlesticks --pair BTC_USDT --interval 1d --limit 30 --format json | jq -r '.[] | [.[0],.[1],.[2],.[3],.[4],.[5]] | @csv' > btc_daily.csv
```

</Block>

<Block>

## Skills

### 一键安装

在支持 Skills 的客户端中输入：

```
帮我自动配置 Gate Skills 和 MCPs: https://github.com/gate/gate-skills
```

- 自动安装全部 Skills
- 支持 Cursor / Claude Code / Codex

### 全部 Skills 一览

**交易**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-exchange-spot | 现货交易：买卖、条件单、止盈止损、改单、撤单 | OAuth2 |
| gate-exchange-futures | USDT 永续合约：开平仓、条件单、止盈止损 | OAuth2 |
| gate-exchange-alpha | Alpha 代币发现、市场查看、交易、账户与订单管理 | OAuth2 |
| gate-exchange-tradfi | TradFi 传统金融产品：股票/外汇/大宗商品交易、MT5 账户 | OAuth2 |
| gate-exchange-flashswap | 闪兑：一对一、一对多、多对一，支持查询支持币对、报价预览、订单历史 | OAuth2 |
| gate-exchange-trading-copilot | 端到端交易副驾驶：市场判断、风险控制、执行、现货/合约交易全流程 | OAuth2 |

**账户资产**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-exchange-assets | 资产与余额查询：总资产、现货、合约、杠杆、期权、理财、Alpha、TradFi | OAuth2 |
| gate-exchange-assets-manager | 多账户资产总览、保证金风险、SimpleEarn 收益、联盟佣金、统一账户借贷与杠杆配置 | OAuth2 |
| gate-exchange-unified | 统一账户操作：资产权益、借贷额度、借还款、模式切换、杠杆与抵押配置 | OAuth2 |
| gate-exchange-subaccount | 子账户管理：查询、创建、锁定、解锁 | OAuth2 |
| gate-exchange-transfer | 同 UID 内部转账：现货、合约、杠杆账户间资金划转 | OAuth2 |

**理财质押**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-exchange-simpleearn | Simple Earn 活期（Uni）和定期理财：产品列表、持仓、申购、赎回、利息查询 | OAuth2 |
| gate-exchange-staking | 链上质押（Staking）理财：质押、赎回、Mint、持仓、收益 | OAuth2 |
| gate-exchange-launchpool | LaunchPool 质押空投：项目列表、质押、赎回、质押记录、空投历史 | OAuth2 |
| gate-exchange-dual | 双币投资：产品查询、结算模拟、下单、持仓管理 | OAuth2 |
| gate-exchange-collateralloan | 多币种抵押借贷：查询贷款、固定利率、还款、增减抵押 | OAuth2 |

**营销福利**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-exchange-activitycenter | 平台活动中心：活动推荐、活动类型、我的活动 | OAuth2 |
| gate-exchange-welfare | 新手福利中心：新手任务、领取奖励 | OAuth2 |
| gate-exchange-coupon | 券码管理：查询可用券、券详情、使用规则、来源追溯 | OAuth2 |
| gate-exchange-affiliate | 联盟计划：佣金查询、交易量、客户数、申请资格 | OAuth2 |
| gate-exchange-vipfee | VIP 等级与费率查询：VIP 等级、现货费率、合约费率 | OAuth2 |
| gate-exchange-referral | 邀请好友活动推荐与规则解读：推荐活动、邀请奖励、佣金、券码、返佣链接 | 无 |

**工具辅助**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-exchange-marketanalysis | 市场分析：流动性、深度、滑点模拟、买卖压力、清算分析、资金费率套利、基差、操纵风险、订单簿解读 | OAuth2 |
| gate-exchange-kyc | KYC 引导：身份验证入口、为什么不能提现 | 无 |
| gate-exchange-crossex | 跨交易所操作：Gate/Binance/OKX/Bybit 订单、转账、兑换 | OAuth2 |

**DEX 钱包**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-dex-market | 只读市场数据查询：代币列表、基本信息、持有人、排行榜、新币发现、安全审计、交易量、K 线、流动性事件 | 无 |
| gate-dex-trade | Swap 交易执行：报价、授权、构建、签名、提交、状态查询 | Google/Gate OAuth |
| gate-dex-wallet | 钱包账户管理：Google/Gate OAuth 认证、余额查询、地址获取、转账、交易历史、x402 支付、DApp 交互、CLI 工具 | Google/Gate OAuth |

**市场信息**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-info-research | 市场研究副驾驶（L2 复合技能）：编排 12 个只读 MCP 工具，产出结构化市场简报、单币深度分析、多币对比、趋势分析、事件归因、风险检查 | 无 |
| gate-info-coinanalysis | 单币综合分析：仅当用户只询问一个币且无其他维度时使用 | 无 |
| gate-info-trendanalysis | 趋势与技术分析：仅当用户只询问技术指标/趋势且无其他维度时使用 | 无 |
| gate-info-riskcheck | 代币与地址风险评估：仅当用户只询问安全性且无其他维度时使用 | 无 |
| gate-info-marketoverview | 市场概览：仅当用户只询问整体市场状况且无具体币分析时使用 | 无 |
| gate-info-coincompare | 币种对比：多个币的对比分析 | 无 |
| gate-info-addresstracker | 地址追踪与分析：地址信息、地址交易、资金流追踪 | 无 |
| gate-info-liveroomlocation | 直播与回放列表：按业务类型（tag）、币种、排序查找直播间/回放 | 无 |

**资讯**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-news-briefing | 新闻简报：仅当用户只询问最近新闻/头条且无其他分析维度时使用 | 无 |
| gate-news-eventexplain | 事件归因与解释：仅当用户只询问价格异动原因且无其他分析维度时使用 | 无 |
| gate-news-listing | 交易所上币追踪：上币、下架、维护公告 | 无 |

**安装器**

| Skill 名称 | 说明 | 认证 |
|----------|---------|------|
| gate-mcp-cursor-installer | Cursor 一键安装 Gate MCP + Skills | 无需认证 |
| gate-mcp-claude-installer | Claude Code 一键安装 | 无需认证 |
| gate-mcp-codex-installer | Codex 一键安装 | 无需认证 |
| gate-mcp-openclaw-installer | OpenClaw 通用安装器 | 无需认证 |

</Block>

<Block>

## UseCase 与工作流

以下 8 个完整用例覆盖从免认证行情查询到链上 Swap 的全场景，每个用例均包含用户提示词、调用端点、工具/Skill、执行流程与预期结果。

### 用例 1：行情监控仪表盘

| 项目 | 内容 |
|------|------|
| **认证要求** | 无需认证 |
| **用户提示词** | BTC ETH SOL 实时价格 |
| **端点** | `/mcp` |
| **工具 / Skill** | cex_spot_get_spot_tickers |

**执行流程：**

1️⃣ Agent 解析用户意图，提取币种列表 [BTC, ETH, SOL]  
2️⃣ 调用 cex_spot_get_spot_tickers 获取全量现货行情  
3️⃣ 过滤出 BTC_USDT、ETH_USDT、SOL_USDT 三个交易对  
4️⃣ 提取最新价、24h 涨跌幅、24h 成交量  
5️⃣ 格式化输出行情仪表盘表格

**预期结果：**

```
币种  最新价 (USDT)   24h 涨跌  24h 成交量
BTC   67,234.50      +2.31%    12,345 BTC
ETH   3,456.78       +1.85%    89,012 ETH
SOL   178.92         +5.67%    2,345,678 SOL
```

![用例1效果图](/docs/developers/images/agent/usecase-1.png)

---

### 用例 2：技术分析研报

| 项目 | 内容 |
|------|------|
| **认证要求** | 无需认证 |
| **用户提示词** | ETH 多周期技术分析 |
| **端点** | `/mcp/info` |
| **工具 / Skill** | info_markettrend_get_technical_analysis<br>info_markettrend_get_kline<br>info_markettrend_get_indicator_history<br>info_marketsnapshot_get_market_snapshot |

**执行流程：**

1️⃣ 调用 info_marketsnapshot_get_market_snapshot 获取 ETH 实时快照（价格、市值、成交量）  
2️⃣ 调用 info_markettrend_get_kline 分别获取 1h / 4h / 1d 三个周期 K 线  
3️⃣ 调用 info_markettrend_get_indicator_history 获取 RSI、MACD、布林带历史数据  
4️⃣ 调用 info_markettrend_get_technical_analysis 获取综合技术分析评级  
5️⃣ Agent 综合多周期数据，生成结构化技术分析报告

**预期结果：**

包含多周期趋势判断（1h 看涨 / 4h 震荡 / 1d 看涨）、关键支撑/阻力位、RSI 超买超卖状态、MACD 金叉死叉信号、综合建议的完整技术分析研报。

![用例2效果图](/docs/developers/images/agent/usecase-2.png)

---

### 用例 3：现货交易闭环

| 项目 | 内容 |
|------|------|
| **认证要求** | OAuth2 |
| **用户提示词** | 分析 BTC，如果适合就买 500U |
| **端点** | `/mcp` + `/mcp/exchange` |
| **工具 / Skill** | gate-exchange-trading-copilot Skill（编排多工具） |

**执行流程：**

1️⃣ **分析阶段** — 调用 info 端点获取 BTC 技术分析、市场快照、新闻情绪  
2️⃣ **风控阶段** — 评估当前波动率、资金费率、爆仓热力图，判断是否适合入场  
3️⃣ **草案阶段** — 若判断适合，生成订单草案：市价买入 500 USDT 等值 BTC  
4️⃣ **确认阶段** — 向用户展示订单详情（预估数量、手续费、滑点），等待确认  
5️⃣ **执行阶段** — 用户确认后调用 cex_spot_create_spot_order 执行市价买入  
6️⃣ **回报阶段** — 查询成交结果，输出实际成交价格、数量、手续费

**预期结果：**

Agent 输出完整的分析→风控→执行→回报链路，用户全程可见决策依据，最终确认后完成交易。

![用例3效果图](/docs/developers/images/agent/usecase-3.png)

---

### 用例 4：合约开仓

| 项目 | 内容 |
|------|------|
| **认证要求** | OAuth2 |
| **用户提示词** | BTC 永续做多 1 张 10x |
| **端点** | `/mcp/exchange` |
| **工具 / Skill** | gate-exchange-futures Skill |

**执行流程：**

1️⃣ 解析意图：合约 = BTC_USDT，方向 = 做多（buy），数量 = 1 张，杠杆 = 10x  
2️⃣ 调用 cex_fx_get_fx_contract 确认合约参数（最小下单量、面值、维持保证金率）  
3️⃣ 调用 cex_fx_update_fx_position_leverage 设置杠杆倍数为 10x  
4️⃣ 调用 cex_fx_get_fx_tickers 获取当前标记价格，计算所需保证金  
5️⃣ 向用户展示订单预览（预估开仓价、保证金、强平价），等待确认  
6️⃣ 用户确认后调用 cex_fx_create_fx_order 执行开多  
7️⃣ 查询持仓确认开仓成功，输出持仓详情

**预期结果：**

成功开仓 BTC_USDT 永续多单 1 张，杠杆 10x，输出开仓均价、已用保证金、预估强平价。

![用例4效果图](/docs/developers/images/agent/usecase-4.png)

---

### 用例 5：链上 Swap

| 项目 | 内容 |
|------|------|
| **认证要求** | Google / Gate OAuth（DEX） |
| **用户提示词** | 0.1 ETH 换 USDT |
| **端点** | `/mcp/dex` |
| **工具 / Skill** | dex_tx_quote<br>dex_tx_swap |

**执行流程：**

1️⃣ 解析意图：卖出 0.1 ETH，买入 USDT，链 = Ethereum  
2️⃣ 调用 dex_tx_quote 获取报价（预估收到的 USDT 数量、滑点、Gas 费）  
3️⃣ 向用户展示报价详情：预计收到 xxx USDT，滑点 0.x%，Gas ≈ $x.xx  
4️⃣ 用户确认后调用 dex_tx_swap 执行链上交换  
5️⃣ 返回交易哈希，等待链上确认  
6️⃣ 输出最终结果：实际收到 USDT 数量、交易哈希、区块链浏览器链接

**预期结果：**

链上 Swap 成功，0.1 ETH → xxx USDT，输出 tx hash 与确认状态。

![用例5效果图](/docs/developers/images/agent/usecase-5.png)

---

### 用例 6：代币安全审计

| 项目 | 内容 |
|------|------|
| **认证要求** | 无需认证 |
| **用户提示词** | 这个合约安全吗 0x1234...abcd |
| **端点** | `/mcp/info` + `/mcp/dex` |
| **工具 / Skill** | info_compliance_check_token_security<br>dex_token_get_risk_info |

**执行流程：**

1️⃣ 解析合约地址 0x1234...abcd，识别所在链（自动检测或用户指定）  
2️⃣ 调用 info_compliance_check_token_security 执行 CEX 维度安全检测（黑名单、合规标记）  
3️⃣ 调用 dex_token_get_risk_info 执行链上维度安全审计（蜜罐检测、权限集中度、流动性锁定）  
4️⃣ 综合两维度结果，生成安全评分与风险摘要  
5️⃣ 输出风险项列表与操作建议

**预期结果：**

```
安全评分：72/100（中等风险）

风险项：
  - 合约所有者可暂停交易
  - 流动性未锁定
  - 持币前10地址集中度 > 60%

安全项：
  - 非蜜罐合约
  - 已开源验证
  - 无铸造后门
```

![用例6效果图](/docs/developers/images/agent/usecase-6.png)

---

### 用例 7：新闻事件归因

| 项目 | 内容 |
|------|------|
| **认证要求** | 无需认证 |
| **用户提示词** | SOL 为什么大涨 |
| **端点** | `/mcp/news` + `/mcp/info` |
| **工具 / Skill** | gate-news-eventexplain Skill |

**执行流程：**

1️⃣ 调用 info_marketsnapshot_get_market_snapshot 确认 SOL 近期涨幅数据  
2️⃣ 调用 news_feed_search_news 搜索近 24-72h 内 SOL 相关新闻  
3️⃣ 调用 news_feed_get_social_sentiment 获取社交媒体情绪变化  
4️⃣ 调用 info_onchain_get_token_onchain 检查链上异动（大额转账、TVL 变化）  
5️⃣ Agent 综合新闻、情绪、链上数据，归因价格异动原因  
6️⃣ 输出事件归因报告

**预期结果：**

结构化的事件归因报告，包含：涨幅数据、触发事件（如生态利好/ETF 进展/大户买入）、市场情绪评级、后续关注点。

![用例7效果图](/docs/developers/images/agent/usecase-7.png)

---

### 用例 8：跨交易所监控

| 项目 | 内容 |
|------|------|
| **认证要求** | OAuth2 |
| **用户提示词** | 查看 Gate 和 Binance 持仓 |
| **端点** | `/mcp/exchange` |
| **工具 / Skill** | gate-exchange-crossex Skill |

**执行流程：**

1️⃣ 解析意图：跨交易所持仓查询，目标 = Gate + Binance  
2️⃣ 通过 CrossEx 模块分别查询 Gate 和 Binance 的现货 + 合约持仓  
3️⃣ 统一格式化持仓数据（币种、数量、当前价值、盈亏）  
4️⃣ 汇总计算跨交易所总资产  
5️⃣ 输出跨交易所持仓对比仪表盘

**预期结果：**

```
📊 跨交易所持仓汇总

交易所    现货资产 (USDT)  合约资产 (USDT)  合计
Gate      12,345.67        5,678.90         18,024.57
Binance   8,901.23         3,456.78         12,358.01
合计      21,246.90        9,135.68         30,382.58
```

![用例8效果图](/docs/developers/images/agent/usecase-8.png)

</Block>
<Block>

## 安全

### 认证体系总览

| 端点 | 认证方式 | 密钥存储位置 | 触发方式 |
|------|---------|------------|---------|
| `/mcp` | 无需认证 | — | — |
| `/mcp/exchange` | OAuth2 | Gate 服务端 | 首次调用交易工具时浏览器弹出授权 |
| `/mcp/dex` | Google / Gate OAuth | Gate 服务端 | dex_auth_google_login_start 触发 |
| `/mcp/info` | 无需认证 | — | — |
| `/mcp/news` | 无需认证 | — | — |


### OAuth2 Scopes

| Scope | 权限范围 | 涉及工具举例 |
|-------|---------|------------|
| market | 行情数据读取 | cex_spot_get_spot_tickers、cex_fx_get_fx_candlesticks |
| profile | 用户基本信息 | cex_unified_get_unified_accounts、VIP 等级查询 |
| trade | 现货与合约交易 | cex_spot_create_spot_order、cex_fx_create_fx_order |
| wallet | 钱包与充提 | cex_wallet_create_transfer、cex_wallet_get_deposit_address |
| account | 账户资产查询 | cex_wallet_get_total_balance、cex_spot_get_spot_accounts |

用户授权时可按需勾选 Scope，最小权限原则。

### DEX 认证流程

```
Step 1  调用 dex_auth_google_login_start → 返回 Google OAuth 授权 URL
Step 2  用户在浏览器中完成 Google 登录授权
Step 3  调用 dex_auth_google_login_poll 轮询授权状态 → 授权成功后返回 session token
Step 4  后续所有 DEX 工具自动携带 session token → 无需再次登录（token 过期前有效）
```
### 安全最佳实践

- **最小权限** — 仅授予当前任务所需的 OAuth2 Scope，避免过度授权
- **交易确认门控** — 所有涉及资金变动的操作（下单/转账/Swap）必须经用户明确确认后执行
- **密钥零本地存储** — OAuth2 模式下，API Key 始终保存在 Gate 服务端，本地不存储任何密钥
- **Session 管理** — DEX session token 设有过期时间，过期后自动触发重新认证
- **异常熔断** — 连续认证失败或异常调用时，自动暂停操作并提示用户检查授权状态

</Block>

<Block>

## 常见问题

**Q: Gate MCP 能做什么？**  
A: 行情查询、现货/合约/期权交易、DEX Swap、钱包管理、新闻资讯、币种研究等 500+ 工具覆盖加密货币全场景。

**Q: 需要安装什么吗？**  
A: Gate MCP 分远程托管（Remote MCP）和本地部署（Local MCP），均只需在客户端添加 URL 即可使用，零安装、零依赖。

**Q: 需要 API Key 吗？**  
A: 行情查询等无需任何认证。如安装了 Remote MCP，交易等功能需要使用 OAuth2 一键授权，浏览器弹窗完成，无需手动创建或填写 API Key。如安装了 Local MCP，在非认证场景下可以不用配置 API 密钥，在需要认证操作时需要提供 API Key 和 API Secret。

**Q: 支持哪些 AI 客户端？**  
A: Cursor、Claude.ai、Claude Code、ChatGPT、Codex、Windsurf、任何支持 Remote MCP (Streamable HTTP) 的客户端。

**Q: 资金安全吗？**  
A: OAuth2 模式下 API Key 始终在 Gate 服务端，本地不存储密钥。所有交易操作需用户明确确认。OAuth2 授权可随时在 Gate 账户设置中撤销。

**Q: 免费吗？**  
A: MCP 服务本身完全免费，交易产生的手续费按 Gate 标准费率收取（与网页/App 交易一致）。

**Q: 如何反馈问题？**  
A: GitHub Issues：[https://github.com/gate/gate-mcp/issues](https://github.com/gate/gate-mcp/issues)，也可在对应 Skill 仓库提交 Issue。

**Q: 有速率限制吗？**  
A: 行情端点遵循 Gate 公开 API 速率限制（通常 300 次/秒）。交易端点按用户账户维度限速，与 API 交易一致。遇到 429 错误时 Agent 会自动重试。

</Block>

<Block>

## 相关链接

- 产品主页：[https://www.gate.com/zh/gate-for-ai](https://www.gate.com/zh/gate-for-ai)
- 帮助中心 — Gate for AI 使用说明：[https://www.gate.com/zh/help/gateai/gateforai/50437](https://www.gate.com/zh/help/gateai/gateforai/50437)
- GitHub — gate-mcp：[https://github.com/gate/gate-mcp](https://github.com/gate/gate-mcp)
- GitHub — gate-skills：[https://github.com/gate/gate-skills](https://github.com/gate/gate-skills)
- GitHub — gate-for-ai：[https://github.com/gate/gate-for-ai](https://github.com/gate/gate-for-ai)
- API 文档：[https://www.gate.io/docs/developers/apiv4](https://www.gate.io/docs/developers/apiv4)
- ClawHub（技能市场）：[https://clawhub.ai/u/gate-exchange](https://clawhub.ai/u/gate-exchange)
- npm 包：[https://www.npmjs.com/package/gate-mcp](https://www.npmjs.com/package/gate-mcp)

</Block>
