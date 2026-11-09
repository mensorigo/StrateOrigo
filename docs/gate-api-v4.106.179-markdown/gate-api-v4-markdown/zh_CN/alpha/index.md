---
title: Gate Alpha API
language_tabs:
  - python: Python
  - shell: Shell
toc_footers: []
includes: []
search: true
highlight_theme: darkula
headingLevel: 2

---

<Block>

# Gate Alpha API v1.1.1

<Example>

下方有示例代码，请求和返回示例。 点击上方的语言 tab 来切换各语言的代码示例。

</Example>

APIv4 提供 Alpha 服务的API, 接口默认限流200r/10s

</Block>

<Block>

## 访问链接

**REST API BaseURL:**

- 实盘交易: `https://api.gateio.ws/api/v4`
- 模拟交易：`https://api-testnet.gateapi.io/api/v4`

## SDK

可用 SDK:

- [Python](https://github.com/gate/gateapi-python)
- [Java](https://github.com/gate/gateapi-java)
- [PHP](https://github.com/gate/gateapi-php)
- [Go](https://github.com/gate/gateapi-go)
- [C#](https://github.com/gate/gateapi-csharp)
- [NodeJS](https://github.com/gate/gateapi-nodejs)
- [Javascript](https://github.com/gate/gateapi-js)

部分 SDK 除了各接口的示例文档以外，还额外提供了调用 SDK 的示例应用程序。
示例应用程序提供了一个相对更加丰富的 SDK 使用示例，可以完整构建运行，
具体使用方式参考各 SDK 的示例程序说明

- [Python](https://github.com/gate/gateapi-python/tree/master/example)
- [Java](https://github.com/gate/gateapi-java/tree/master/example)
- [C#](https://github.com/gate/gateapi-csharp/tree/master/example)
- [Go](https://github.com/gate/gateapi-go/tree/master/_example)

## 关于Alpha账户

Alpha账户是Gate.io推出的专业交易账户类型，提供专门的资产管理和交易功能。Alpha账户API提供了完整的账户查询、资产流水、询价下单等功能。

## 技术支持

使用过程中如有问题或者建议，您可以选择以下任意方式联系我们：

* 提交工单反馈
* 在线工单反馈
* 将您的联系方式与问题发送至 [mm@gate.com](mailto:mm@gate.com) ，我们将为您分配技术专员为您服务

如您遇到API错误，建议您整理以下内容，方便我们为您快速分析问题：

1. 问题描述
2. Gate UID
3. 请求的地址与参数
4. 错误代码
5. 返回结果

::: danger
即使是提交问题，也切勿将API key信息提交给客服及他人，否则会有严重的资产风险。如果已经不小心泄漏，请将已有API删除并重新生成。
:::

</Block>

<Block>

# 变更日志

**v1.1.1**

2026-03-12

- 更新 `/alpha/orders` 查询订单接口：`currency`、`side`、`status` 参数从必填改为选填

**v4.105.12**

2025-10-12

- 新增 `/alpha/tokens` 接口,支持查询token信息
- 支持按链(chain)查询token列表,包括solana、eth、bsc、base、world、sui、arbitrum、avalanche、polygon、linea、optimism、zksync、gatelayer等
- 支持按发射平台(launch_platform)查询token列表,包括meteora_dbc、fourmeme、moonshot、pump、raydium_launchlab、letsbonk、gatefun、virtuals等
- 支持按合约地址(address)查询token信息
- 默认限流: 200r/10s
- `/alpha/quote` 询价接口添加限流说明:限流单个用户10r/s
- `/alpha/orders` 下单接口添加限流说明:限流单个用户5r/s

**v4.100.3**

2025-07-05

- Alpha账户API初始发布
- 提供Alpha账户资产查询功能
- 支持Alpha账户资产流水查询
- 提供Alpha询价和下单功能
- 支持Alpha订单查询和管理
- 提供Alpha支持的币种和ticker信息查询

</Block>

<Block>

# 通用信息

## 数据中心

Gate 数据中心位于 AWS 日本东京 (ap-northeast-1) 地区。

## 接口概览

| 接口分类                  | 分类链接           | 概述                                 |
| ------------------------- | ------------------ | ------------------------------------ |
| host + `/api/v4/alpha/*`  | Alpha账户接口      | Alpha账户资产查询、流水、交易等功能  |

## Alpha账户特点

Alpha账户是Gate.io推出的专业交易账户类型，具有以下特点：

- 专业的资产管理功能
- 独立的账户体系
- 专门的询价和交易机制
- 完整的资产流水记录

## 使用说明

使用Alpha账户API前，请确保：

1. 您已开通Alpha账户
2. 已获得相应的API权限
3. 了解Alpha账户的交易规则和限制

## 接口分类

Alpha账户API主要包含以下功能模块：

- **账户查询**：查询Alpha账户的资产状况
- **资产流水**：查询账户资产变动记录
- **询价交易**：获取报价并进行交易
- **订单管理**：查询和管理交易订单
- **币种信息**：查询支持的币种和价格信息

</Block>

<Block>

# 认证

## 生成 API key

在调用私有API接口前，需要生成账户的API key来验证身份。
您可以在网页端登录成功后，在【账户管理】-> 【APIv4 Keys】中生成，
或点击 <a href="/myaccount/apiv4keys" target="_blank">这里</a> 生成 API keys

每个账户最多可以创建 20 个 API key，每个 Key 的权限配置都是相互独立的。
建议给每个 Key 设置能够标明用途的备注名

**`Key`** 访问密钥   
**`Secret Key`** 签名认证加密所使用的密钥

除此之外，还可以配置 IP 白名单，只允许服务端接收来自 IP 
白名单里的客户端请求。每个 Key 最多可配置 20 个 IP 地址，IP 地址按照 IPv4 配置，
不支持 IP 地址段。若不设置 IP 白名单，服务端不会验证客户端 IP 来源。

::: tip
注：如果发现 Key 的名字是 `spot` 或者 `futures` ，该 Key 很有可能是迁移之后系统的默认命名，
详情参考 “关于 APIv4 Key 升级” 一节。
:::

创建的 Key 还可以更新和删除，不过需要注意的是 Key 的更新和删除，最多需要 5 分钟才能生效。

另外模拟合约与实盘合约属于两套不同的环境，实盘合约的 API Key 不可用于模拟合约。
如果需要使用模拟合约做 API 接口联调测试，需要在个人账户 APIv4Keys 页面的模拟合约入口单独申请。
模拟合约与实盘合约的接口请求方式完全相同，区别只是在 API 的 Base URL 和使用的 API Key

## APIv4 权限

创建 Key 的时候，可以为该 Key 配置是否开启现货杠杆、合约、钱包或者提现的权限，
开启的权限可以配置读写或者只读。

| 产品      | 权限                            |
|---------|-------------------------------|
| `现货/杠杆` | `只读`查询订单 `读写`查询订单&下单          |
| `永续合约`  | `只读`查询订单 `读写`查询订单&下单          |
| `交割合约`  | `只读`查询订单 `读写`查询订单&下单          |
| `钱包`    | `只读`查询充提划转记录 `读写` 查询账户记录&资金划转 |
| `提现`    | `只读`查询提现记录 `读写` 查询提现记录&提现     |

所有请求方法为 `GET` 的都是读操作，其他的则是写请求。每个权限组可以设置为禁用、只读或读写。

值得注意的一点是，尽管提现操作组只有一个 API （即 `POST /withdrawals` ），考虑到一般使用情况，
还是将其从钱包操作里独立成一个权限组，而包括了提现记录的账户转出流水记录查询（即 
`GET /wallet/withdrawals` ）还是保留在钱包 API 权限组了。

## APIv4 验签请求接口发送要求

1. 在官网个人中心申请 APIv4 Key ，并确保该 Key 拥有对应操作的读写权限。
2. 在发送请求头部传入 `KEY` ，即 APIv4 密钥对的 Key
3. 在发送请求头部传入 `Timestamp` ，即请求发送的时间，格式是秒级精度的 Unix 时间戳。
   同时该时间不能与当前时间差距超过 60 秒。
4. 在发送请求头部传入 `SIGN` ，即将请求生成签名字符串并用 APIv4 Secret 加密后生成的签名。
   签名字符串生成方法参看下节，加密算法为 `HexEncode(HMAC_SHA512(secret, signature_string))` ，
   即通过 HMAC-SHA512 加密算法，将 APIv4 Secret 作为加密密钥，签名字符串作为加密消息，
   生成加密结果的 16 进制输出。
5. 确保发送请求的客户端 IP 地址在所使用的密钥的 IP 地址白名单里。

## APIv4 签名字符串生成方式

APIv4 中签名字符串按照如下方式拼接生成：

`
	Request Method + "\n"
	+ Request URL + "\n"
	+ Query String + "\n"
	+ HexEncode(SHA512(Request Payload)) + "\n"
	+ Timestamp
`

### Request Method

请求方法，全大写, 如 `POST`, `GET`

### Request URL

请求 URL，不包括服务地址和端口，如 `/api/v4/futures/orders`

### Query String

*没有*使用 URL 编码的请求参数，请求参数在参与计算签名时的顺序一定要保证和实际请求里的顺序一致。
如 `status=finished&limit=50` 。

如果没有请求参数，使用空字符串 ("")

### HexEncode(SHA512(Request Payload))

将请求体字符串使用 SHA512 哈希之后的结果。如果没有请求体，使用空字符串的哈希结果，即
`cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e`

### Timestamp

设置在请求头部 `Timestamp` 里的值

<Example>

示例

注：示例中所有的换行都是为了方便显示人为添加的，实际只有示例中的一个 `\n` 保留

假设使用的 Key 为 `key` ，Secret 为 `secret`

1. 查询所有合约订单

```http
	GET /api/v4/futures/orders?contract=BTC_USD&status=finished&limit=50 HTTP/1.1
```

签名字符串:

```
	GET\n
	/api/v4/futures/orders\n
	contract=BTC_USD&status=finished&limit=50\n
	cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\n
	1541993715
```

说明

 - `/api/v4/futures/orders`: 请求 URL
 -  `contract=BTC_USD&status=finished&limit=50`: 请求参数，与实际请求的顺序完全一致
 - 请求体为空，使用空字符串的哈希输出
 - `1541993715`: Unix 时间戳

签名结果

`55f84ea195d6fe57ce62464daaa7c3c02fa9d1dde954e4c898289c9a2407a3d6fb3faf24deff16790d726b66ac9f74526668b13bd01029199cc4fcc522418b8a`

2. 创建合约委托

```http
	POST /api/v4/futures/orders HTTP/1.1

	{"contract":"BTC_USD","type":"limit","size":100,"price":6800,"time_in_force":"gtc"}
```

签名字符串:

```
	POST\n
	/api/v4/futures/orders\n
	\n
	ad3c169203dc3026558f01b4df307641fa1fa361f086b2306658886d5708767b1854797c68d9e62fef2f991645aa82673622ebf417e091d0bd22bafe5d956cca\n
	1541993715
```

说明

- 请求参数为空，使用空字符串
- 使用 JSON 序列化之后的字符串的哈希输出

签名结果

`eae42da914a590ddf727473aff25fc87d50b64783941061f47a3fdb92742541fc4c2c14017581b4199a1418d54471c269c03a38d788d802e2c306c37636389f0`

```python
# coding: utf-8

# Python 示例验签代码

"""
本示例仅作为演示签名计算方式使用，推荐使用各语言的 SDK ，因为已经集成了验签规则
"""

# coding: utf-8
import time
import hashlib
import hmac
import requests
import json

def gen_sign(method, url, query_string=None, payload_string=None):
    key = ''        # api_key
    secret = ''     # api_secret

    t = time.time()
    m = hashlib.sha512()
    m.update((payload_string or "").encode('utf-8'))
    hashed_payload = m.hexdigest()
    s = '%s\n%s\n%s\n%s\n%s' % (method, url, query_string or "", hashed_payload, t)
    sign = hmac.new(secret.encode('utf-8'), s.encode('utf-8'), hashlib.sha512).hexdigest()
    return {'KEY': key, 'Timestamp': str(t), 'SIGN': sign}

if __name__ == "__main__":
    host = "https://api.gateio.ws"
    prefix = "/api/v4"
    common_headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

    url = '/futures/orders'
    body = {"contract": "BTC_USD", "size": 100, "price": "30", "tif": "gtc"}
    request_content = json.dumps(body)
    sign_headers = gen_sign('POST', prefix + url, "", request_content)
    sign_headers.update(common_headers)
    print('signature headers: %s' % sign_headers)
    res = requests.post(host + prefix + url, headers=sign_headers, data=request_content)
    print(res.status_code)
    print(res.content)
```

</Example>

</Block>

<Block>

## 异常处理

APIv4 对于所有的异常请求，会设置状态码为非 2xx ，同时返回一个 JSON 格式的返回体来描述具体的错误信息。

返回体的格式通常如下所示：

```json
{
  "label": "INVALID_PARAM_VALUE",
  "message": "Invalid parameter `text` with value: abc"
}
```

- `label` 用于标识某种错误的类型，格式 `string` ，它的值是一个固定的列表（见下方）。程序处理可以使用 `label` 字段的内容来设定和捕获异常
- `message` (或 `detail`) 表示详细的错误信息，方便 API 对接时，理解具体是因为什么样的参数设置导致请求出现了异常。
  该字段内容不建议用于异常的捕获或识别。

以 Python [requests](https://requests.readthedocs.io/zh_CN/latest/) 为例，异常的处理流程可以参考如下所示：

> 以下示例的异常捕获流程只涉及到业务相关的异常，网络连接超时等其他非业务相关的异常还需要自行处理。

```python
import requests

r = requests.get("https://api.gateio.ws/api/v4/futures/btc/contracts/BTC_USD")
try:
    r.raise_for_status()
except requests.HTTPError:
    # 捕获非 2xx 错误，尝试解析 body 里返回的错误消息，并根据不同 label 做不同的异常处理
    if r.json()['label'] == 'xxx':
        print(r.json())
```

以 [Python SDK](https://github.com/gate/gateapi-python) 为示例：

```python
import json
from gate_api import FuturesApi
from gate_api.rest import ApiException

api = FuturesApi()
try:
    api.get_futures_contract(settle='btc', contract="BTC_USD")
except ApiException as e:  # ApiException 封装了异常的各种信息，详情可参看类定义
    detail = json.loads(e.value.body)
    if detail['label'] == 'xxx':
        print(detail)
```

## 异常 `label` 列表

- 请求参数或格式问题

| `label`                | 含义                    |
|------------------------|-----------------------|
| INVALID_PARAM_VALUE    | 参数输入值无效               |
| INVALID_PROTOCOL       | 参数输入值无效               |
| INVALID_ARGUMENT       | 参数无效                  |
| INVALID_REQUEST_BODY   | 无效请求体                 |
| MISSING_REQUIRED_PARAM | 缺少必选参数                |
| BAD_REQUEST            | 无效请求                  |
| INVALID_CONTENT_TYPE   | 无效的 Content-Type 头部格式 |
| NOT_ACCEPTABLE         | Accept 头部无法满足         |
| METHOD_NOT_ALLOWED     | 请求方法不接受               |
| NOT_FOUND              | 资源 URL 不存在            |

- 认证相关

| `label`                        | 含义              |
|--------------------------------|-----------------|
| INVALID_CREDENTIALS            | 认证接口缺少用户认证信息    |
| INVALID_KEY                    | 无效的 API Key     |
| IP_FORBIDDEN                   | 请求 IP 不在白名单     |
| READ_ONLY                      | 请求账户只读，不可执行写操作  |
| INVALID_SIGNATURE              | 无效签名            |
| MISSING_REQUIRED_HEADER        | 缺少必要的认证头部       |
| REQUEST_EXPIRED                | 客户端时间与服务端时间相差过大 |
| ACCOUNT_LOCKED                 | 账户被锁定           |
| FORBIDDEN                      | 账户无权执行该操作       |
| API_WITHDRAW_DISABLED          | API提现操作临时禁用     |
| INVALID_WITHDRAW_ID            | 无效的提现ID         |
| INVALID_WITHDRAW_CANCEL_STATUS | 当前提现状态无法取消      |

- 钱包相关

| `label`                     | 含义                  |
|-----------------------------|---------------------|
| SUB_ACCOUNT_NOT_FOUND       | 子账户不存在              |
| SUB_ACCOUNT_LOCKED          | 子账号被冻结              |
| MARGIN_BALANCE_EXCEPTION    | 杠杆账户异常              |
| MARGIN_TRANSFER_FAILED      | 杠杆资金划转失败            |
| TOO_MUCH_FUTURES_AVAILABLE  | 合约账户总资产达到上限         |
| FUTURES_BALANCE_NOT_ENOUGH  | 合约账户余额不足            |
| ACCOUNT_EXCEPTION           | 账户异常                |
| SUB_ACCOUNT_TRANSFER_FAILED | 子账户资金划转失败           |
| ADDRESS_NOT_USED            | 指定的钱包地址未在网页执行过划转    |
| TOO_FAST                    | 提现频率过快              |
| WITHDRAWAL_OVER_LIMIT       | 超出提现额度              |
| DUPLICATE_REQUEST           | 重复请求                |
| ORDER_EXISTS                | 订单已存在               |
| INVALID_CLIENT_ORDER_ID     | 无效的client_order_id  |
| RISK_ERROR                  | 触发风控                |
| NEGATIVE_ASSETS             | 安全提现检查存在负资产    |
| RECEIVE_ERROR               | 接收失败                |
| DEDUCTION_ERROR             | 转账人扣款失败           |
| FINANCE_ERROR               | 财务账户扣款失败         |
| BALANCE_NOT_ENOUGH          | 余额不足         |

- 现货和杠杆相关

| `label`                   | 含义              |
|---------------------------|-----------------|
| INVALID_PRECISION         | 无效的精度           |
| INVALID_CURRENCY          | 无效的币种信息         |
| INVALID_CURRENCY_PAIR     | 无效的交易对          |
| POC_FILL_IMMEDIATELY      | 被动委托会立即成交       |
| ORDER_NOT_FOUND           | 订单不存在           |
| ORDER_CLOSED              | 订单已结束           |
| ORDER_CANCELLED           | 订单已撤销           |
| QUANTITY_NOT_ENOUGH       | 数量不足            |
| BALANCE_NOT_ENOUGH        | 余额不足            |
| MARGIN_NOT_SUPPORTED      | 该交易对不支持杠杆交易     |
| MARGIN_BALANCE_NOT_ENOUGH | 杠杆账户余额不足        |
| AMOUNT_TOO_LITTLE         | 数额小于最低值         |
| AMOUNT_TOO_MUCH           | 数额过大            |
| REPEATED_CREATION         | 重复创建            |
| LOAN_NOT_FOUND            | 借贷订单不存在         |
| LOAN_RECORD_NOT_FOUND     | 借贷记录不存在         |
| NO_MATCHED_LOAN           | 没有借贷记录能满足借入需求   |
| NOT_MERGEABLE             | 借贷单不可合并         |
| NO_CHANGE                 | 修改的参数与当前状态无区别   |
| REPAY_TOO_MUCH            | 还款数额超出借款数额      |
| TOO_MANY_CURRENCY_PAIRS   | 批量下单指定了过多交易对    |
| TOO_MANY_ORDERS           | 批量下单的单个交易对下单数过多 |
| MIXED_ACCOUNT_TYPE        | 批量下单中使用了多个账户类型  |
| AUTO_BORROW_TOO_MUCH      | 自动借入超出最多可借      |
| TRADE_RESTRICTED          | 高负债率导致交易操作被限制   |
| FOK_NOT_FILL              | FOK 订单无法全部成交    |
| INITIAL_MARGIN_TOO_LOW    | 用户总初始保证金率太低     |
| NO_MERGEABLE_ORDERS       | 找不到能够合并的借贷订单    |
| ORDER_BOOK_NOT_FOUND      | 市场深度不足          |
| FAILED_RETRIEVE_ASSETS    | 获取账户资产失败        |
| CANCEL_FAIL               | 订单撤销失败          |
| PRICE_THRESHOLD_EXCEEDED  | 触发限价保护          |

- 合约相关

| `label`                 | 含义                             |
|-------------------------|--------------------------------|
| USER_NOT_FOUND          | 用户无合约账户                        |
| CONTRACT_NO_COUNTER     | 没有匹配的对手单                       |
| CONTRACT_NOT_FOUND      | 合约未找到                          |
| NOT_FOUND               | 请求路径不存在                        |
| RISK_LIMIT_EXCEEDED     | 委托超出风险限额                       |
| INSUFFICIENT_AVAILABLE  | 余额不足                           |
| LIQUIDATE_IMMEDIATELY   | 操作可能导致爆仓                       |
| LEVERAGE_TOO_HIGH       | 杠杆倍数设置过高                       |
| LEVERAGE_TOO_LOW        | 杠杆倍数设置过低                       |
| ORDER_NOT_FOUND         | 委托不存在                          |
| ORDER_NOT_OWNED         | 委托不存在                          |
| ORDER_FINISHED          | 委托已结束                          |
| TOO_MANY_ORDERS         | 过多未完成的委托                       |
| POSITION_CROSS_MARGIN   | 全仓不支持更新保证金                     |
| POSITION_IN_LIQUIDATION | 仓位在强制平仓中                       |
| POSITION_IN_CLOSE       | 仓位正在平仓中                        |
| POSITION_EMPTY          | 仓位为空                           |
| REMOVE_TOO_MUCH         | 保证金超过可调范围                      |
| RISK_LIMIT_NOT_MULTIPLE | 风险限额未按照步长调整                    |
| RISK_LIMIT_TOO_HIGH     | 超出最大风险限额                       |
| RISK_LIMIT_TOO_lOW      | 风险限额设置过低                       |
| PRICE_TOO_DEVIATED      | 下单价与标记价格相差过大                   |
| SIZE_TOO_LARGE          | 下单数量超过上限                       |
| SIZE_TOO_SMALL          | 下单数量不足下限                       |
| PRICE_OVER_LIQUIDATION  | 增加仓位时价格不能超过平仓价                 |
| PRICE_OVER_BANKRUPT     | 减少仓位时价格不能超过破产价                 |
| ORDER_POC_IMMEDIATE     | 被动委托会立即成交                      |
| INCREASE_POSITION       | 只减仓委托会增加仓位                     |
| CONTRACT_IN_DELISTING   | 当前合约市场处于下线过渡期，只允许创建只减仓委托或者平仓委托 |
| POSITION_NOT_FOUND      | 仓位不存在                         |
| POSITION_DUAL_MODE      | 双向持仓模式不允许此操作           |
| ORDER_PENDING           | 有委托存在则不允许此操作           |
| POSITION_HOLDING        | 有持仓则不允许此操作               |
| REDUCE_EXCEEDED         | 双向持仓模式下，减仓单超过仓位大小 |
| NO_CHANGE               | 没有改变发生                       |
| AMEND_WITH_STOP         | 有止盈止损单时不能修改委托         |
| ORDER_FOK               | FOK 订单无法全部成交               |

- 抵押借币相关

| `label`                      | 含义               |
|------------------------------|------------------|
| COL_NOT_ENOUGH               | 质押物余额不足          |
| COL_TOO_MUCH                 | 超过质押币种质押额度       |
| INIT_LTV_TOO_HIGH            | 初始质押率过高          |
| REDEEMED_LTV_TOO_HIGH        | 提取后质押率过高         |
| BORROWABLE_NOT_ENOUGH        | 剩余可借余额不足         |
| ORDER_TOO_MANY_TOTAL         | 超过平台每天下单数量       |
| ORDER_TOO_MANY_DAILY         | 超过单一用户每天下单数量     |
| ORDER_TOO_MANY_USER          | 超过单一用户总下单数量      |
| ORDER_NOT_EXIST              | 订单不存在            |
| ORDER_FINISHED               | 订单已结束            |
| ORDER_NO_PAY                 | 订单待还金额为0         |
| ORDER_EXIST                  | 订单已存在            |
| ORDER_HISTORY_EXIST          | 订单历史记录已存在        |
| ORDER_REPAYING               | 订单已在还款中          |
| ORDER_LIQUIDATING            | 订单已在平仓中          |
| BORROW_TOO_LITTLE            | 小于币种最小可借         |
| BORROW_TOO_LARGE             | 借款金额超过最大剩余可借     |
| REPAY_AMOUNT_INVALID         | 还款金额无效           |
| REPAY_GREATER_THAN_AVAILABLE | 还款数额大于剩余可用       |
| POOL_BALANCE_NOT_ENOUGH      | 资金池余额不足          |
| CURRENCY_SETTLING            | 币种结算中，不允许还款      |
| RISK_REJECT                  | 风控检查中，请稍后再试      |
| LOAN_FAILED                  | 放款失败，可以再次发起借款    |

- 现货保证金相关

| `label`               | 含义        |
|-----------------------|-----------|
| USER_LIAB             | 用户存在负债  |
| USER_PENDING_ORDERS   | 用户存在挂单  |
| MODE_SET              | 保证金模式已设置 |

- 理财相关

| `label`               | 含义        |
|-----------------------|-----------|
| ERR_BALANCE_NOT_ENOUGH      | 余额不足  |
| ERR_PRODUCT_SELL_OUT        | 项目售罄  |
| ERR_PRODUCT_BUY             | 项目未开始 |
| ERR_CREATE_ORDER            | 下单失败 |
| ERR_QUOTA_LOWER_LIMIT       | 不满足最小下单金额 |
| ERR_QUOTA_SUPERIOR_LIMIT    | 已达最大下单额度 |
| ERR_ORDER_NUMBER_LIMIT      | 已达最大下单数量 |
| ERR_PRODUCT_CLOSE           | 项目已结束 |
| COPIES_NOT_ENOUGH           | 可申购仓位不足 |
| COPIES_TOO_SMALL            | 投资份额不足 |
| COPIES_TOO_BIG              | 投资份额超过最大购买限制 |
| TOTAL_AMOUNT_24             | 24小时内质押与赎回总量超限 |
| TOTAL_BUYCOUNT_24           | 24小时内质押与赎回次数超限 |
| REDEEM_24_LIMIT             | 质押后24小时内不能赎回 |

- 服务异常

| `label`      | 含义    |
|--------------|-------|
| INTERNAL     | 内部错误  |
| SERVER_ERROR | 内部错误  |
| INTERNAL_SERVER_ERROR | 操作失败，请稍后重试（等同于 SERVER_ERROR） |
| TOO_BUSY     | 服务当前忙 |

- 闪兑相关

| `label`                   | 含义          |
|---------------------------|-------------|
| INVALID_PARAM_VALUE       | 参数不合法       |
| INVALID_CURRENCY          | 不支持的币种      |
| INVALID_CURRENCY_PAIR     | 无效交易对       |
| PRICE_OBSOLETE            | 价格已失效/过期/作废 |
| ORDER_NOT_FOUND           | 订单不存在       |
| ORDER_BOOK_NOT_FOUND      | 订单簿不存在      |
| BALANCE_NOT_ENOUGH        | 余额不足，划转失败   |
| TOO_MANY_REQUESTS         | 请求过于频繁      |
| QUOTA_NOT_ENOUGH          | 额度不足        |
| SERVER_TIMEOUT            | 服务超时        |
| MISSING_REQUIRED_PARAM    | 缺失必要参数      |
| REQUEST_FORBIDDEN         | 资管账户访问限制    |
| CONVERT_PREVIEW_EXPIRED   | 预览缓存过期      |
| CONVERT_PREVIEW_NOT_MATCH | 预览数据不一致     |
| AMOUNT_TOO_LITTLE         | 金额太小        |
| AMOUNT_TOO_MUCH           | 金额太大        |

</Block>

<Block>

# Alpha

Alpha Account

</Block>

<Block>

##  Alpha 账户接口

<a id="opIdlistAlphaAccounts"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvYWNjb3VudHMiLCJxdWVyeVN0cmluZyI6W10sImhlYWRlcnMiOlt7Im5hbWUiOiJBY2NlcHQiLCJ2YWx1ZSI6ImFwcGxpY2F0aW9uL2pzb24ifV0sImhlYWRlcnNTaXplIjotMSwiY29va2llcyI6W119" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/accounts'
query_param = ''
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('GET', prefix + url, query_param)
headers.update(sign_headers)
r = requests.request('GET', host + prefix + url, headers=headers)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="GET"
url="/alpha/accounts"
query_param=""
body_param=''
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url"
curl -X $method $full_url \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`GET /alpha/accounts`

*Alpha 账户接口*

查询持仓资产

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "currency": "memeboxELON",
    "available": "1",
    "locked": "0",
    "token_address": "0x6952c5408b",
    "chain": "SOL"
  }
]
```

</Example>

<h3 id="listalphaaccounts-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|查询持仓成功|[Inline]|

<h3 id="listalphaaccounts-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» *None*|object|查询持仓返回|
|»» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|»» available|string|可用余额|
|»» locked|string|锁定余额|
|»» token_address|string|token 地址|
|»» chain|string|区块链名称|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  Alpha 账户资产流水接口

<a id="opIdlistAlphaAccountBook"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvYWNjb3VudF9ib29rP2Zyb209MCIsInF1ZXJ5U3RyaW5nIjpbXSwiaGVhZGVycyI6W3sibmFtZSI6IkFjY2VwdCIsInZhbHVlIjoiYXBwbGljYXRpb24vanNvbiJ9XSwiaGVhZGVyc1NpemUiOi0xLCJjb29raWVzIjpbXX0=" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/account_book'
query_param = 'from=0'
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('GET', prefix + url, query_param)
headers.update(sign_headers)
r = requests.request('GET', host + prefix + url + "?" + query_param, headers=headers)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="GET"
url="/alpha/account_book"
query_param="from=0"
body_param=''
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url?$query_param"
curl -X $method $full_url \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`GET /alpha/account_book`

*Alpha 账户资产流水接口*

查询资产流水

<h3 id="listalphaaccountbook-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|from|请求参数|integer(int64)|是|查询记录的起始时间|
|to|请求参数|integer(int64)|否|查询记录的结束时间，不指定则默认为当前时间|
|page|请求参数|integer(int32)|否|列表页数|
|limit|请求参数|integer(int32)|否|列表数量最大每页100|

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "id": "123456",
    "time": 1747827868,
    "currency": "memeboxELON",
    "change": "1.03",
    "balance": "4.59316525194"
  }
]
```

</Example>

<h3 id="listalphaaccountbook-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|查询流水成功|[Inline]|

<h3 id="listalphaaccountbook-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» *None*|object|查询资产流水返回|
|»» id|integer(int64)|订单id|
|»» time|integer(int64)|操作时间戳|
|»» currency|string(string)|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|»» change|string(string)|变更金额|
|»» balance|string(string)|变更后余额|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  Alpha 询价接口

<a id="opIdquoteAlphaOrder"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiUE9TVCIsInVybCI6Ii8vL2FscGhhL3F1b3RlIiwicXVlcnlTdHJpbmciOltdLCJoZWFkZXJzIjpbeyJuYW1lIjoiQ29udGVudC1UeXBlIiwidmFsdWUiOiJhcHBsaWNhdGlvbi9qc29uIn0seyJuYW1lIjoiQWNjZXB0IiwidmFsdWUiOiJhcHBsaWNhdGlvbi9qc29uIn1dLCJoZWFkZXJzU2l6ZSI6LTEsImNvb2tpZXMiOltdLCJib2R5U2l6ZSI6LTEsInBvc3REYXRhIjp7Im1pbWVUeXBlIjoiYXBwbGljYXRpb24vanNvbiIsInRleHQiOiJ7XCJjdXJyZW5jeVwiOlwibWVtZWJveEVMT05cIixcInNpZGVcIjpcImJ1eVwiLFwiYW1vdW50XCI6XCIzMjRcIixcImdhc19tb2RlXCI6XCJjdXN0b21cIixcInNsaXBwYWdlXCI6XCIxMFwifSJ9fQ==" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/quote'
query_param = ''
body='{"currency":"memeboxELON","side":"buy","amount":"324","gas_mode":"custom","slippage":"10"}'
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('POST', prefix + url, query_param, body)
headers.update(sign_headers)
r = requests.request('POST', host + prefix + url, headers=headers, data=body)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="POST"
url="/alpha/quote"
query_param=""
body_param='{"currency":"memeboxELON","side":"buy","amount":"324","gas_mode":"custom","slippage":"10"}'
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url"
curl -X $method $full_url -d "$body_param" -H "Content-Type: application/json" \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`POST /alpha/quote`

*Alpha 询价接口*

询价接口返回的quote_id存在一分钟有效期，需要在一分钟内进行下单，超时需要重新询价,限流单个用户10r/s.

<Example>

> 请求体示例

```json
{
  "currency": "memeboxELON",
  "side": "buy",
  "amount": "324",
  "gas_mode": "custom",
  "slippage": "10"
}
```

</Example>

<h3 id="quotealphaorder-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|body|body|object|是||
|» currency|body|string|是|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098，不是展示符号 USDS。先不传 currency 调用 GET /alpha/currencies，按 name、chain 和 address 确认币种，使用返回的 currency 字段；列表可通过 page 和 limit 分页。|
|» side|body|string|是|买单或者卖单<br>- buy<br>- sell|
|» amount|body|string|是|交易数量  <br>- `side` : `buy` 指代计价货币，指`USDT`<br>- `side` : `sell` 指代交易货币|
|» gas_mode|body|string|是|交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`|
|» slippage|body|string|否|滑点 10是浮动10%|

#### 详细描述

**» side**: 买单或者卖单<br>- buy<br>- sell

**» amount**: 交易数量  <br>- `side` : `buy` 指代计价货币，指`USDT`<br>- `side` : `sell` 指代交易货币

**» gas_mode**: 交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`

<Example>

> 返回示例

> 200 返回

```json
{
  "quote_id": "12345678",
  "min_amount": "0.1",
  "max_amount": "1000:0.0",
  "price": "11.666",
  "slippage": "11.666",
  "estimate_gas_fee_amount_usdt": "$0.04",
  "order_fee": "$0",
  "target_token_min_amount": "500.6",
  "target_token_max_amount": "666.6",
  "error_type": 0
}
```

</Example>

<h3 id="quotealphaorder-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|询价成功|Inline|

<h3 id="quotealphaorder-responseschema">返回格式</h3>

状态码 **200**

*询价返回*

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» quote_id|string|询价id 用于下单使用1分钟内有效|
|» min_amount|string|最小下单量|
|» max_amount|string|最大下单量|
|» price|string|币价格，U本位|
|» slippage|string|滑点|
|» estimate_gas_fee_amount_usdt|string|预估网络费 U本位|
|» order_fee|string|交易手续费|
|» target_token_min_amount|string|最小获得数量|
|» target_token_max_amount|string|最大获得数量|
|» error_type|integer(int32)|失败类型<br>- `0` : 成功<br>- `1` : 超过最大值<br>- `2` : 低于最小值|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  Alpha 下单接口

<a id="opIdplaceAlphaOrder"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiUE9TVCIsInVybCI6Ii8vL2FscGhhL29yZGVycyIsInF1ZXJ5U3RyaW5nIjpbXSwiaGVhZGVycyI6W3sibmFtZSI6IkNvbnRlbnQtVHlwZSIsInZhbHVlIjoiYXBwbGljYXRpb24vanNvbiJ9LHsibmFtZSI6IkFjY2VwdCIsInZhbHVlIjoiYXBwbGljYXRpb24vanNvbiJ9XSwiaGVhZGVyc1NpemUiOi0xLCJjb29raWVzIjpbXSwiYm9keVNpemUiOi0xLCJwb3N0RGF0YSI6eyJtaW1lVHlwZSI6ImFwcGxpY2F0aW9uL2pzb24iLCJ0ZXh0Ijoie1wiY3VycmVuY3lcIjpcIm1lbWVib3hFTE9OXCIsXCJzaWRlXCI6XCJidXlcIixcImFtb3VudFwiOlwiMzI0XCIsXCJnYXNfbW9kZVwiOlwiY3VzdG9tXCIsXCJzbGlwcGFnZVwiOlwiMTBcIixcInF1b3RlX2lkXCI6XCIxMjM0NTY3OFwifSJ9fQ==" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/orders'
query_param = ''
body='{"currency":"memeboxELON","side":"buy","amount":"324","gas_mode":"custom","slippage":"10","quote_id":"12345678"}'
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('POST', prefix + url, query_param, body)
headers.update(sign_headers)
r = requests.request('POST', host + prefix + url, headers=headers, data=body)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="POST"
url="/alpha/orders"
query_param=""
body_param='{"currency":"memeboxELON","side":"buy","amount":"324","gas_mode":"custom","slippage":"10","quote_id":"12345678"}'
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url"
curl -X $method $full_url -d "$body_param" -H "Content-Type: application/json" \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`POST /alpha/orders`

*Alpha 下单接口*

下单接口,限流单个用户5r/s.

<Example>

> 请求体示例

```json
{
  "currency": "memeboxELON",
  "side": "buy",
  "amount": "324",
  "gas_mode": "custom",
  "slippage": "10",
  "quote_id": "12345678"
}
```

</Example>

<h3 id="placealphaorder-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|body|body|object|是||
|» currency|body|string|是|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098，不是展示符号 USDS。先不传 currency 调用 GET /alpha/currencies，按 name、chain 和 address 确认币种，使用返回的 currency 字段；列表可通过 page 和 limit 分页。|
|» side|body|string|是|买单或者卖单<br>- buy<br>- sell|
|» amount|body|string|是|交易数量  <br>- `side` : `buy` 指代计价货币，指`USDT`<br>- `side` : `sell` 指代交易货币|
|» gas_mode|body|string|是|交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`|
|» slippage|body|string|否|滑点百分比，"10" 表示 10%。|
|» quote_id|body|string|是|询价返回的quote_id|

#### 详细描述

**» side**: 买单或者卖单<br>- buy<br>- sell

**» amount**: 交易数量  <br>- `side` : `buy` 指代计价货币，指`USDT`<br>- `side` : `sell` 指代交易货币

**» gas_mode**: 交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`

<Example>

> 返回示例

> 200 返回

```json
{
  "order_id": "12345678",
  "status": 1,
  "side": "buy",
  "gas_mode": "custom",
  "create_time": 1749468580,
  "amount": "324",
  "token_address": "string",
  "chain": "ETH"
}
```

</Example>

<h3 id="placealphaorder-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|下单成功|Inline|

<h3 id="placealphaorder-responseschema">返回格式</h3>

状态码 **200**

*下单返回*

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» order_id|string|订单id|
|» status|integer(int32)|订单状态<br>- `0` : 全部<br>- `1` : 处理中<br>- `2` : 成功<br>- `3` : 失败<br>- `4` : 已取消<br>- `5` : 买入下单后还未转账完成<br>- `6` : 取消订单未转账|
|» side|string|买单或者卖单<br>- buy<br>- sell|
|» gas_mode|string|交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`|
|» create_time|integer(int64)|创建时间 (时间戳)|
|» amount|string|交易数量  <br>- `side` : `buy` 指代计价货币，指`USDT`<br>- `side` : `sell` 指代交易货币|
|» token_address|string|币地址|
|» chain|string|链名|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  Alpha 查询订单列表接口

<a id="opIdlistAlphaOrder"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvb3JkZXJzIiwicXVlcnlTdHJpbmciOltdLCJoZWFkZXJzIjpbeyJuYW1lIjoiQWNjZXB0IiwidmFsdWUiOiJhcHBsaWNhdGlvbi9qc29uIn1dLCJoZWFkZXJzU2l6ZSI6LTEsImNvb2tpZXMiOltdfQ==" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/orders'
query_param = ''
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('GET', prefix + url, query_param)
headers.update(sign_headers)
r = requests.request('GET', host + prefix + url, headers=headers)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="GET"
url="/alpha/orders"
query_param=""
body_param=''
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url"
curl -X $method $full_url \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`GET /alpha/orders`

*Alpha 查询订单列表接口*

<h3 id="listalphaorder-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|currency|请求参数|string|否|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098，不是展示符号 USDS。先不传 currency 调用 GET /alpha/currencies，按 name、chain 和 address 确认币种，使用返回的 currency 字段；列表可通过 page 和 limit 分页。|
|side|请求参数|string|否|买单或者卖单<br>- buy<br>- sell<br>|
|status|请求参数|integer(int32)|否|订单状态<br>- `0` : 全部<br>- `1` : 处理中<br>- `2` : 成功<br>- `3` : 失败<br>- `4` : 已取消<br>- `5` : 买入下单后还未转账完成<br>- `6` : 取消订单未转账<br>|
|from|请求参数|integer(int64)|否|查询订单的起始时间|
|to|请求参数|integer(int64)|否|查询订单的结束时间，不指定则默认为当前时间|
|limit|请求参数|integer(int32)|否|列表返回的最大数量。默认为100，最小1，最大100。|
|page|请求参数|integer(int32)|否|列表页数|

#### 详细描述

**side**: 买单或者卖单<br>- buy<br>- sell<br>

**status**: 订单状态<br>- `0` : 全部<br>- `1` : 处理中<br>- `2` : 成功<br>- `3` : 失败<br>- `4` : 已取消<br>- `5` : 买入下单后还未转账完成<br>- `6` : 取消订单未转账<br>

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "order_id": "12345678",
    "tx_hash": "aaaaaaa",
    "side": "buy",
    "usdt_amount": "0.0000",
    "currency": "MEME",
    "currency_amount": "565455643.6400",
    "status": 1,
    "gas_mode": "1",
    "chain": "ETH",
    "gas_fee": "0.3",
    "transaction_fee": "0",
    "create_time": 1742972931,
    "failed_reason": ""
  }
]
```

</Example>

<h3 id="listalphaorder-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|列表查询成功|[Inline]|

<h3 id="listalphaorder-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|*None*|array|[下单返回]|
|» *None*|object|下单返回|
|»» order_id|string|订单id|
|»» tx_hash|string|交易哈希|
|»» side|string|买单或者卖单<br>- buy<br>- sell|
|»» usdt_amount|string|钱数|
|»» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|»» currency_amount|string|币的数量|
|»» status|integer(int32)|订单状态<br>- `0` : 全部<br>- `1` : 处理中<br>- `2` : 成功<br>- `3` : 失败<br>- `4` : 已取消<br>- `5` : 买入下单后还未转账完成<br>- `6` : 取消订单未转账|
|»» gas_mode|string|交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`|
|»» chain|string|链|
|»» gas_fee|string|矿工费 U本位|
|»» transaction_fee|string|交易手续费 U本位|
|»» failed_reason|string|失败原因（如果有的话）|
|»» create_time|integer(int64)|创建时间（时间戳）|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  Alpha 查询单个订单接口

<a id="opIdgetAlphaOrder"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvb3JkZXI/b3JkZXJfaWQ9ZmRhZjEyMzIxIiwicXVlcnlTdHJpbmciOltdLCJoZWFkZXJzIjpbeyJuYW1lIjoiQWNjZXB0IiwidmFsdWUiOiJhcHBsaWNhdGlvbi9qc29uIn1dLCJoZWFkZXJzU2l6ZSI6LTEsImNvb2tpZXMiOltdfQ==" />

```python
# coding: utf-8
import requests
import time
import hashlib
import hmac

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/order'
query_param = 'order_id=fdaf12321'
# `gen_sign` 的实现参考认证一章
sign_headers = gen_sign('GET', prefix + url, query_param)
headers.update(sign_headers)
r = requests.request('GET', host + prefix + url + "?" + query_param, headers=headers)
print(r.json())

```

```shell
key="YOUR_API_KEY"
secret="YOUR_API_SECRET"
host="//"
prefix="//"
method="GET"
url="/alpha/order"
query_param="order_id=fdaf12321"
body_param=''
timestamp=$(date +%s)
body_hash=$(printf "$body_param" | openssl sha512 | awk '{print $NF}')
sign_string="$method\n$prefix$url\n$query_param\n$body_hash\n$timestamp"
sign=$(printf "$sign_string" | openssl sha512 -hmac "$secret" | awk '{print $NF}')

full_url="$host$prefix$url?$query_param"
curl -X $method $full_url \
    -H "Timestamp: $timestamp" -H "KEY: $key" -H "SIGN: $sign"

```

</Example>

`GET /alpha/order`

*Alpha 查询单个订单接口*

<h3 id="getalphaorder-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|order_id|请求参数|string|是|订单id|

<Example>

> 返回示例

> 200 返回

```json
{
  "order_id": "12345678",
  "tx_hash": "aaaaaaa",
  "side": "buy",
  "usdt_amount": "0.0000",
  "currency": "MEME",
  "currency_amount": "565455643.6400",
  "status": 1,
  "gas_mode": "1",
  "chain": "ETH",
  "gas_fee": "0.3",
  "transaction_fee": "0",
  "create_time": 1742972931,
  "failed_reason": ""
}
```

</Example>

<h3 id="getalphaorder-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|订单查询成功|Inline|

<h3 id="getalphaorder-responseschema">返回格式</h3>

状态码 **200**

*下单返回*

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» order_id|string|订单id|
|» tx_hash|string|交易哈希|
|» side|string|买单或者卖单<br>- buy<br>- sell|
|» usdt_amount|string|钱数|
|» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|» currency_amount|string|币的数量|
|» status|integer(int32)|订单状态<br>- `0` : 全部<br>- `1` : 处理中<br>- `2` : 成功<br>- `3` : 失败<br>- `4` : 已取消<br>- `5` : 买入下单后还未转账完成<br>- `6` : 取消订单未转账|
|» gas_mode|string|交易模式 影响滑点选择<br>- `speed` : 智能模式<br>- `custom` : 自定义, 会使用 `slippage`|
|» chain|string|链|
|» gas_fee|string|矿工费 U本位|
|» transaction_fee|string|交易手续费 U本位|
|» failed_reason|string|失败原因（如果有的话）|
|» create_time|integer(int64)|创建时间（时间戳）|

</SchemaBlock>

::: warning
该请求需要 API key 和 secret 认证
:::

</Block>

<Block>

##  查询币种信息

<a id="opIdlistAlphaCurrencies"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvY3VycmVuY2llcyIsInF1ZXJ5U3RyaW5nIjpbXSwiaGVhZGVycyI6W3sibmFtZSI6IkFjY2VwdCIsInZhbHVlIjoiYXBwbGljYXRpb24vanNvbiJ9XSwiaGVhZGVyc1NpemUiOi0xLCJjb29raWVzIjpbXX0=" />

```python
# coding: utf-8
import requests

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/currencies'
query_param = ''
r = requests.request('GET', host + prefix + url, headers=headers)
print(r.json())

```

```shell

curl -X GET /alpha/currencies \
  -H 'Accept: application/json'

```

</Example>

`GET /alpha/currencies`

*查询币种信息*

传currency时，按currency查询返回指定币种信息,
不传currency时，分页返回币种列表

<h3 id="listalphacurrencies-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|currency|请求参数|string|否|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098，不是展示符号 USDS。先不传 currency 调用 GET /alpha/currencies，按 name、chain 和 address 确认币种，使用返回的 currency 字段；列表可通过 page 和 limit 分页。|
|limit|请求参数|integer(int32)|否|列表返回的最大数量|
|page|请求参数|integer(int32)|否|列表页数|

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "currency": "memeboxtrump",
    "name": "trump",
    "chain": "SOL",
    "address": "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN",
    "status": 1,
    "precision": 6,
    "amount_precision": 1
  }
]
```

</Example>

<h3 id="listalphacurrencies-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|查询成功|[Inline]|

<h3 id="listalphacurrencies-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» AlphaCurrency|object||
|»» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|»» name|string|币种展示名称或符号，例如 USDS；不能代替 currency 字段作为 Alpha 币种 ID。|
|»» chain|string|币对应的主链|
|»» address|string|合约地址|
|»» amount_precision|integer(int32)|数量精度|
|»» precision|integer(int32)|价格精度|
|»» status|integer(int32)|币种交易状态<br>- `1` : 正常交易<br>- `2` : 暂停交易<br>- `3` : 下架|

</SchemaBlock>

<aside class="success">
该请求不需要认证
</aside>

</Block>

<Block>

##  查询币种ticker

<a id="opIdlistAlphaTickers"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvdGlja2VycyIsInF1ZXJ5U3RyaW5nIjpbXSwiaGVhZGVycyI6W3sibmFtZSI6IkFjY2VwdCIsInZhbHVlIjoiYXBwbGljYXRpb24vanNvbiJ9XSwiaGVhZGVyc1NpemUiOi0xLCJjb29raWVzIjpbXX0=" />

```python
# coding: utf-8
import requests

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/tickers'
query_param = ''
r = requests.request('GET', host + prefix + url, headers=headers)
print(r.json())

```

```shell

curl -X GET /alpha/tickers \
  -H 'Accept: application/json'

```

</Example>

`GET /alpha/tickers`

*查询币种ticker*

传currency时，按currency查询返回指定ticker信息,不传currency时，分页返回ticker列表信息

<h3 id="listalphatickers-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|currency|请求参数|string|否|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098，不是展示符号 USDS。先不传 currency 调用 GET /alpha/currencies，按 name、chain 和 address 确认币种，使用返回的 currency 字段；列表可通过 page 和 limit 分页。|
|limit|请求参数|integer(int32)|否|列表返回的最大数量|
|page|请求参数|integer(int32)|否|列表页数|

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "currency": "memeboxtrump",
    "last": "11.38",
    "change": "-7.45",
    "volume": "3423412.221",
    "market_cap": "34234129.94"
  }
]
```

</Example>

<h3 id="listalphatickers-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|查询成功|[Inline]|

<h3 id="listalphatickers-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» AlphaTicker|object||
|»» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|»» last|string|最新成交价|
|»» change|string|最近24h涨跌百分比，跌用负数标识，如 -7.45|
|»» volume|string|最近24h成交额(USDT)|
|»» market_cap|string|币种当前市值|

</SchemaBlock>

<aside class="success">
该请求不需要认证
</aside>

</Block>

<Block>

##  查询token信息

<a id="opIdlistAlphaTokens"></a>

<Example sample-title="示例代码">

<OperationMeta har-b64="eyJodHRwVmVyc2lvbiI6IkhUVFAvMS4xIiwibWV0aG9kIjoiR0VUIiwidXJsIjoiLy8vYWxwaGEvdG9rZW5zIiwicXVlcnlTdHJpbmciOltdLCJoZWFkZXJzIjpbeyJuYW1lIjoiQWNjZXB0IiwidmFsdWUiOiJhcHBsaWNhdGlvbi9qc29uIn1dLCJoZWFkZXJzU2l6ZSI6LTEsImNvb2tpZXMiOltdfQ==" />

```python
# coding: utf-8
import requests

host = "//"
prefix = "//"
headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}

url = '/alpha/tokens'
query_param = ''
r = requests.request('GET', host + prefix + url, headers=headers)
print(r.json())

```

```shell

curl -X GET /alpha/tokens \
  -H 'Accept: application/json'

```

</Example>

`GET /alpha/tokens`

*查询token信息*

支持传chain、platform和address

<h3 id="listalphatokens-parameters">参数</h3>

|名称|位置|类型|必选|描述|
|---|---|---|---|---|
|chain|请求参数|string|否|根据链查询token列表<br><br>- solana<br>- eth<br>- bsc<br>- base<br>- world<br>- sui<br>- arbitrum<br>- avalanche<br>- polygon<br>- linea<br>- optimism<br>- zksync<br>- gatelayer<br>|
|launch_platform|请求参数|string|否|根据发射平台查询token列表<br><br>- meteora_dbc<br>- fourmeme<br>- moonshot<br>- pump<br>- raydium_launchlab<br>- letsbonk<br>- gatefun<br>- virtuals<br>|
|address|请求参数|string|否|根据合约地址查询token列表|
|page|请求参数|integer(int32)|否|列表页数|

#### 详细描述

**chain**: 根据链查询token列表<br><br>- solana<br>- eth<br>- bsc<br>- base<br>- world<br>- sui<br>- arbitrum<br>- avalanche<br>- polygon<br>- linea<br>- optimism<br>- zksync<br>- gatelayer<br>

**launch_platform**: 根据发射平台查询token列表<br><br>- meteora_dbc<br>- fourmeme<br>- moonshot<br>- pump<br>- raydium_launchlab<br>- letsbonk<br>- gatefun<br>- virtuals<br>

<Example>

> 返回示例

> 200 返回

```json
[
  {
    "currency": "memeboxtrump",
    "name": "trump",
    "chain": "SOL",
    "address": "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN",
    "status": 1,
    "precision": 6,
    "amount_precision": 1
  }
]
```

</Example>

<h3 id="listalphatokens-responses">返回</h3>

|状态码|含义|描述|格式|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|查询成功|[Inline]|

<h3 id="listalphatokens-responseschema">返回格式</h3>

状态码 **200**

<SchemaBlock :threshold="10">

|名称|类型|描述|
|---|---|---|---|---|
|» currency|string|Alpha 币种 ID，例如 USDS02b5a8dd2f59a098；用于 Alpha 接口的 currency 参数，不是展示符号 USDS。|
|» name|string|币种展示名称或符号，例如 USDS；不能代替 currency 字段作为 Alpha 币种 ID。|
|» chain|string|币对应的主链|
|» address|string|合约地址|
|» amount_precision|integer(int32)|数量精度|
|» precision|integer(int32)|价格精度|
|» status|integer(int32)|币种交易状态<br>- `1` : 正常交易<br>- `2` : 暂停交易<br>- `3` : 下架|

</SchemaBlock>

<aside class="success">
该请求不需要认证
</aside>

</Block>

