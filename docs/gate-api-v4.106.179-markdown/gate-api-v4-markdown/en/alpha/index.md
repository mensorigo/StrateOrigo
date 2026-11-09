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

Scroll down for code samples, example requests and responses. Select a language for code samples from the tabs above or the mobile navigation menu.

</Example>

APIv4 provides Alpha service APIs with default rate limit of 200 requests per 10 seconds

</Block>

<Block>

## Access URL

**REST API BaseURL:**

- Live trading: `https://api.gateio.ws/api/v4`
- TestNet trading: `https://api-testnet.gateapi.io/api/v4`

## SDK

Available SDK:

- [Python](https://github.com/gate/gateapi-python)
- [Java](https://github.com/gate/gateapi-java)
- [PHP](https://github.com/gate/gateapi-php)
- [Go](https://github.com/gate/gateapi-go)
- [C#](https://github.com/gate/gateapi-csharp)
- [NodeJS](https://github.com/gate/gateapi-nodejs)
- [Javascript](https://github.com/gate/gateapi-js)

Besides API examples, some SDK provides an additional demo application.
The demo application is a relatively complete example demonstrating how to use the SDK.
It can be built and run separately. Refer to corresponding repository for details.

- [Python](https://github.com/gate/gateapi-python/tree/master/example)
- [Java](https://github.com/gate/gateapi-java/tree/master/example)
- [C#](https://github.com/gate/gateapi-csharp/tree/master/example)
- [Go](https://github.com/gate/gateapi-go/tree/master/_example)

## About Alpha Account

Alpha Account is a professional trading account type launched by Gate.io, providing specialized asset management and trading functions. The Alpha Account API provides complete account query, asset transaction history, quotation and order placement functions.

## Technical Support

If you have any questions or suggestions during the use, you can contact us in any of the following ways:

* Submit Work Order Feedback
* Online Work Order Feedback
* Send your contact information and questions to [mm@gate.com](mailto:mm@gate.com) We will assign
  technical specialists to serve you.

If you encounter API errors, it is recommended that you sort out the following content, so that we can quickly analyze the problem for you:

1. Problem Description
2. Gate UID
3. Request URI and parameters
4. Error Code
5. Responses

::: danger
Even if you submit a problem, you should not submit the API key information to customer service or others, otherwise there will be serious asset risk. If it has been accidentally leaked, please delete the existing API and rebuild it.
:::

</Block>

<Block>

# Changelog

**v4.105.13**

2026-03-12

- Update `/alpha/orders` list orders endpoint: change `currency`, `side`, `status` parameters from required to optional

**v4.105.12**

2025-10-12

- Added `/alpha/tokens` endpoint to query token information
- Support querying token list by chain, including solana, eth, bsc, base, world, sui, arbitrum, avalanche, polygon, linea, optimism, zksync, gatelayer, etc.
- Support querying token list by launch platform, including meteora_dbc, fourmeme, moonshot, pump, raydium_launchlab, letsbonk, gatefun, virtuals, etc.
- Support querying token information by contract address
- Added default rate limit description in API: 200r/10s by default
- Added rate limit description for `/alpha/quote` endpoint: 10r/s per user
- Added rate limit description for `/alpha/orders` endpoint: 5r/s per user

**v4.100.3**

2025-07-05

- Initial release of Alpha Account API
- Provide Alpha account asset query functionality
- Support Alpha account asset transaction history query
- Provide Alpha quotation and order placement functionality
- Support Alpha order query and management
- Provide Alpha supported currency and ticker information query

</Block>

<Block>

# General

## Data Center

Gate data center is located in AWS Japan's ap-northeast-1 region.

## API Overview

| API Classification           | Category Links     | Overview                                            |
| ---------------------------- | ------------------ | --------------------------------------------------- |
| host + `/api/v4/alpha/*`     | Alpha Account API  | Alpha account asset query, transaction and trading |

## Alpha Account Features

Alpha Account is a professional trading account type launched by Gate.io with the following features:

- Professional asset management functionality
- Independent account system
- Specialized quotation and trading mechanism
- Complete asset transaction records

## Usage Instructions

Before using the Alpha Account API, please ensure:

1. You have opened an Alpha account
2. You have obtained the corresponding API permissions
3. You understand the trading rules and restrictions of Alpha accounts

## API Categories

Alpha Account API mainly includes the following functional modules:

- **Account Query**: Query the asset status of Alpha accounts
- **Asset Transactions**: Query account asset change records
- **Quotation Trading**: Get quotes and conduct transactions
- **Order Management**: Query and manage trading orders
- **Currency Information**: Query supported currencies and price information

</Block>

<Block>

# Authentication

## Generate API key

Before calling the private API interface, the API key of the account needs to be generated to verify
the identity. You can log in on the website and generate it in
[account management] - > [APIv4 keys], or click <a href="/myaccount/apiv4keys" target="_blank">
here</a>
to generate API keys.

Each account can create 20 API keys, and the permission configuration of each key is independent of
each other. It is recommended to set a note name for each key to indicate its purpose.

**`Key`** Access Key
**`Secret Key`** The key used for signature authentication encryption

Besides, you can attach an IP whitelist, which requires the server only accept requests from
specified IPs. Each key can have at most 20 IPs formatted in IPv4(not supporting IP range though).
If IP whitelist is not set, the server will skip client IP validation.

Each user can create at most 5 keys with separate permissions. It is recommended to set a name for
key denoting how the key will be used.

::: tip
Note: If the key is named with `spot` or `futures`, then it could be the default name after
APIv4 migration. For details refer to *About APIv4 key improvement* section
:::

Created key can also be updated or deleted, but any modification(s) can take up to 5 minutes to take
effect.

Please note that futures TestNet trading is a separate environment from futures real trading. Real
trading API keys cannot be used in TestNet. If you want to test futures API with TestNet, you need
to log into the console to generate TestNet API keys(in _"Futures TestNet APIKeys"_ tab on _"
APIv4Keys"_ page). Making futures requests are identical between real and TestNet trading, with the
only exceptions are different base URLs and different API keys.

## APIv4 Permissions

When creating a Key, you can configure whether to enable spot, margin, contract, wallet, or
withdrawal permissions for the Key, and whether to enable read-write or read-only permissions.

| Products             | Permissions                                                                                               |
|----------------------|-----------------------------------------------------------------------------------------------------------|
| `spot/margin`        | `Read-only` query orders `Read-write` query orders & place orders                                         |
| `perpetual contract` | `Read-only` query orders `Read-write` query orders & place orders                                         |
| `delivery contract`  | `Read-only` query orders `Read-write` query orders & place orders                                         |
| `wallet`             | `Read-only` Query for withdrawal transfer records `Read-write` Query for account records & fund transfers |
| `withdrawal `        | `Read-only` Query cash withdrawal records `Read-write` Query cash withdrawal records & withdrawals        |

All `GET` operations are read requests, while others are write requests. Each permission group can
be set to disabled, read-only or read-write.

Please note that even though withdrawal API has only one operation(i.e.
`POST /withdrawals`), for general concern, it is still separated from wallet API into a standalone
permission group, while withdrawal history retrieving API stays inside wallet operations(
i.e., `GET /wallet/withdrawals`).

## APIv4 signed request requirements

1. Generate APIv4 Key pairs in web console, and make sure it has the right permissions.
2. Set request header `KEY` to the key.
3. Set request header `Timestamp` to current time formatted in Unix time in seconds. Pay
   attention that the gap between its value and current time cannot exceed 60 seconds.
4. Set request header `SIGN` to encrypted request signature. Refer to next section for how signature
   string is generated. Signature generation method is
   `HexEncode(HMAC_SHA512(secret, signature_string))`, i.e., the hexadecimal digest output of
   HMAC-SHA512 with APIv4 secret as secret and signature string as message,
5. Make sure request client's IP is in your APIv4 Key's IP whitelist.

## API Signature string generation

In APIv4, signature string is concatenated as the following way:

`
	Request Method + "\n"
	+ Request URL + "\n"
	+ Query String + "\n"
	+ HexEncode(SHA512(Request Payload)) + "\n"
	+ Timestamp
`

### Request Method

Request method in UPPERCASE, e.g. `POST`, `GET`

### Request URL

Request url. Protocol, host and port are not included, e.g. `/api/v4/futures/orders`

### Query String

Request query string without URL encode. query parameters order should be the
same as how they are concatenated in the request URL, e.g. `status=finished&limit=50`. Use empty string("") if no query parameters.

### HexEncode(SHA512(Request Payload))

Hash the request body with SHA512 and output its Hex encoded form. If no request body, use empty string's hashed result, i.e.
`cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e`

### Timestamp

`Timestamp` request header value.

<Example>

Examples

Note: all example signature string are broken into multiple lines for displaying purpose only.
Only the `\n` character in signature string is reserved in reality.

Suppose the key we used is `key`, while the secret is `secret`.

1. List all orders

```http
	GET /api/v4/futures/orders?contract=BTC_USD&status=finished&limit=50 HTTP/1.1
```

Signature string：

```
	GET\n
	/api/v4/futures/orders\n
	contract=BTC_USD&status=finished&limit=50\n
	cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\n
	1541993715
```

Explanation：

- `/api/v4/futures/orders`: request url
-  `contract=BTC_USD&status=finished&limit=50`: keep the query string as it is in the request url
- request body use empty string's hashed result
- `1541993715`: Unix timestamp in seconds

Signature generated

`55f84ea195d6fe57ce62464daaa7c3c02fa9d1dde954e4c898289c9a2407a3d6fb3faf24deff16790d726b66ac9f74526668b13bd01029199cc4fcc522418b8a`

2. Create an order

```http
	POST /api/v4/futures/orders HTTP/1.1

	{"contract":"BTC_USD","type":"limit","size":100,"price":6800,"time_in_force":"gtc"}
```

Signature string：

```
	POST\n
	/api/v4/futures/orders\n
	\n
	ad3c169203dc3026558f01b4df307641fa1fa361f086b2306658886d5708767b1854797c68d9e62fef2f991645aa82673622ebf417e091d0bd22bafe5d956cca\n
	1541993715
```

Explanation：

- request query string is empty, use plain empty string
- use the hashed result of the json-string-formatted request body

Signature generated

`eae42da914a590ddf727473aff25fc87d50b64783941061f47a3fdb92742541fc4c2c14017581b4199a1418d54471c269c03a38d788d802e2c306c37636389f0`

```python

# example authentication implementation in Python

"""
Python SDK is recommended as it has already implemented the authentication process for every API:
"""

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

## Error Handling

For all abnormal requests, APIv4 will return non-2xx status code, with a response body in JSON
format to
explain the error.

The error response body follows a format like:

```json
{
  "label": "INVALID_PARAM_VALUE",
  "message": "Invalid parameter `text` with value: abc"
}
```

- `label`: denotes error type in `string` format. Its value are chosen from a certain list(see
  below).
  Programs can use `label` to identify and catch a specific error.
- `message`(or `detail`): detailed error message. A longer explanation showing why the error is
  generated
  or how to avoid it. Its purpose is helping to better understand the API. Error handling
  mechanism with this field is highly **NOT** recommended.

Take Python [requests](https://requests.readthedocs.io/en/latest/) for example,
error handling can be written like:

> Following examples only deal with business-related errors. Network timeout or other
> common errors need to be handled separately:

```python
import requests

r = requests.get("https://api.gateio.ws/api/v4/futures/btc/contracts/BTC_USD")
try:
    r.raise_for_status()
except requests.HTTPError:
    # catch 2xx errors, parse error message in body, and do something based on `label`
    if r.json()['label'] == 'xxx':
        print(r.json())
```

or with [Python SDK](https://github.com/gate/gateapi-python):

```python
import json
from gate_api import FuturesApi
from gate_api.rest import ApiException

api = FuturesApi()
try:
    api.get_futures_contract(settle='btc', contract="BTC_USD")
except ApiException as e:  # ApiException wraps whole error information, see implementation for details
    detail = json.loads(e.value.body)
    if detail['label'] == 'xxx':
        print(detail)
```

## `label` list

- Request parameter or format related

| `label`                | Meaning                       |
|------------------------|-------------------------------|
| INVALID_PARAM_VALUE    | Invalid parameter value       |
| INVALID_PROTOCOL       | Invalid parameter value       |
| INVALID_ARGUMENT       | Invalid argument              |
| INVALID_REQUEST_BODY   | Invalid request body          |
| MISSING_REQUIRED_PARAM | Missing required parameter    |
| BAD_REQUEST            | Invalid request               |
| INVALID_CONTENT_TYPE   | Invalid `Content-Type` header |
| NOT_ACCEPTABLE         | Invalid `Accept-` Header      |
| METHOD_NOT_ALLOWED     | Request method is not allowed |
| NOT_FOUND              | Request URL not exists        |

- Authentication related

| `label`                 | Meaning                                         |
|-------------------------|-------------------------------------------------|
| INVALID_CREDENTIALS     | Invalid credentials provided                    |
| INVALID_KEY             | Invalid API Key                                 |
| IP_FORBIDDEN            | Request IP not in whitelist                     |
| READ_ONLY               | API key is read-only                            |
| INVALID_SIGNATURE       | Invalid signature                               |
| MISSING_REQUIRED_HEADER | Missing required authentication header          |
| REQUEST_EXPIRED         | Request `Timestamp` is far from the server time |
| ACCOUNT_LOCKED          | Account is locked                               |
| FORBIDDEN               | Account has no permission to request operation  |

- Wallet related

| `label`                        | Meaning                                               |
|--------------------------------|-------------------------------------------------------|
| SUB_ACCOUNT_NOT_FOUND          | Sub account not found                                 |
| SUB_ACCOUNT_LOCKED             | Sub account is locked                                 |
| MARGIN_BALANCE_EXCEPTION       | Abnormal margin account                               |
| MARGIN_TRANSFER_FAILED         | Failed to transfer with margin account                |
| TOO_MUCH_FUTURES_AVAILABLE     | Futures balance exceeds max allowed                   |
| FUTURES_BALANCE_NOT_ENOUGH     | Futures balance not enough                            |
| ACCOUNT_EXCEPTION              | Abnormal account                                      |
| SUB_ACCOUNT_TRANSFER_FAILED    | Failed to transfer with sub account                   |
| ADDRESS_NOT_USED               | Address never being used in web console               |
| TOO_FAST                       | Withdrawing request exceeds frequency limit           |
| WITHDRAWAL_OVER_LIMIT          | Withdrawal limit exceeded                             |
| API_WITHDRAW_DISABLED          | API withdrawal operation is disabled temporarily      |
| INVALID_WITHDRAW_ID            | Invalid withdraw ID                                   |
| INVALID_WITHDRAW_CANCEL_STATUS | Cancelling withdrawal not allowed with current status |
| DUPLICATE_REQUEST              | Duplicate request                                     |
| ORDER_EXISTS                   | Order already exists, do not resubmit                 |
| INVALID_CLIENT_ORDER_ID        | The client_order_id is invalid                        |

- Spot and margin trading related

| `label`                   | Meaning                                                       |
|---------------------------|---------------------------------------------------------------|
| INVALID_PRECISION         | Invalid precision                                             |
| INVALID_CURRENCY          | Invalid currency                                              |
| INVALID_CURRENCY_PAIR     | Invalid currency pair                                         |
| POC_FILL_IMMEDIATELY      | Order would match and take immediately so it's cancelled      |
| ORDER_NOT_FOUND           | Order not found                                               |
| ORDER_CLOSED              | Order already closed                                          |
| ORDER_CANCELLED           | Order already cancelled                                       |
| QUANTITY_NOT_ENOUGH       | Amount is not enough                                          |
| BALANCE_NOT_ENOUGH        | Balance is not enough                                         |
| MARGIN_NOT_SUPPORTED      | Request currency pair doesn't provide margin trading          |
| MARGIN_BALANCE_NOT_ENOUGH | Margin balance is not enough                                  |
| AMOUNT_TOO_LITTLE         | Amount does not reach minimum required                        |
| AMOUNT_TOO_MUCH           | Amount exceeds maximum allowed                                |
| REPEATED_CREATION         | Repeated creation                                             |
| LOAN_NOT_FOUND            | Margin loan is not found                                      |
| LOAN_RECORD_NOT_FOUND     | Margin loan record is not found                               |
| NO_MATCHED_LOAN           | No loan can match request borrow requirement                  |
| NOT_MERGEABLE             | Request loans cannot be merged                                |
| NO_CHANGE                 | No change is made                                             |
| REPAY_TOO_MUCH            | Repay more than required                                      |
| TOO_MANY_CURRENCY_PAIRS   | Too many currency pairs in batch orders creation              |
| TOO_MANY_ORDERS           | Too many orders in one currency pair in batch orders creation |
| MIXED_ACCOUNT_TYPE        | More than one account type is used in batch orders creation   |
| AUTO_BORROW_TOO_MUCH      | Auto borrow exceeds maximum allowed                           |
| TRADE_RESTRICTED          | Trading is restricted due to high debt ratio                  |
| FOK_NOT_FILL              | FOK order cannot be filled completely                         |
| INITIAL_MARGIN_TOO_LOW    | User's total initial margin rate is too low                   |
| NO_MERGEABLE_ORDERS       | Orders can be merged not found                                |
| ORDER_BOOK_NOT_FOUND      | Insufficient liquidity                                        |
| FAILED_RETRIEVE_ASSETS    | Failed to retrieve account assets                             |
| CANCEL_FAIL               | Order cancel failed                                           |

- Futures related

| `label`                 | Meaning                                                                 |
|-------------------------|-------------------------------------------------------------------------|
| USER_NOT_FOUND          | User has no futures account                                             |
| CONTRACT_NO_COUNTER     | No counter order found                                                  |
| CONTRACT_NOT_FOUND      | Contract not found                                                      |
| RISK_LIMIT_EXCEEDED     | Risk limit exceeded                                                     |
| INSUFFICIENT_AVAILABLE  | Balance is not enough                                                   |
| LIQUIDATE_IMMEDIATELY   | Operation may cause liquidation                                         |
| LEVERAGE_TOO_HIGH       | leverage too high                                                       |
| LEVERAGE_TOO_LOW        | leverage too low                                                        |
| ORDER_NOT_FOUND         | Order not found                                                         |
| ORDER_NOT_OWNED         | Order not owned                                                         |
| ORDER_FINISHED          | Order already finished                                                  |
| TOO_MANY_ORDERS         | Too many open orders                                                    |
| POSITION_CROSS_MARGIN   | margin updating is not allowed in cross margin                          |
| POSITION_IN_LIQUIDATION | Position is being liquidated                                            |
| POSITION_IN_CLOSE       | Position is closing                                                     |
| POSITION_EMPTY          | Position is empty                                                       |
| REMOVE_TOO_MUCH         | Changed margin exceeds allowed                                          |
| RISK_LIMIT_NOT_MULTIPLE | Risk limit is not a multiple of step                                    |
| RISK_LIMIT_TOO_HIGH     | Risk limit too high                                                     |
| RISK_LIMIT_TOO_lOW      | Risk limit too low                                                      |
| PRICE_TOO_DEVIATED      | Order price deviates too much from mark price                           |
| SIZE_TOO_LARGE          | Order size exceeds maximum                                              |
| SIZE_TOO_SMALL          | Order size does not reach minimum                                       |
| PRICE_OVER_LIQUIDATION  | Price to increase position can not exceeds liquidation price            |
| PRICE_OVER_BANKRUPT     | Price to decrease position cannot exceeds bankrupting price             |
| ORDER_POC_IMMEDIATE     | POC order will be finished immediately                                  |
| INCREASE_POSITION       | POC order will increase position                                        |
| CONTRACT_IN_DELISTING   | Contract is delisting, only reduce-only order or close order is allowed |
| POSITION_NOT_FOUND      | Position not found                                                      |
| POSITION_DUAL_MODE      | Operation forbidden in dual-mode                                        |
| ORDER_PENDING           | Operation forbidden with pending orders                                 |
| POSITION_HOLDING        | Operation forbidden with holding position                               |
| REDUCE_EXCEEDED         | Reduce order would exceed position in dual-mode                         |
| NO_CHANGE               | No change is made                                                       |
| AMEND_WITH_STOP         | Amend forbidden with stop order                                         |
| ORDER_FOK               | Killed for FOK                                                          |

- Collateral Loan related

| `label`                      | Meaning                                       |
|------------------------------|-----------------------------------------------|
| COL_NOT_ENOUGH               | Collateral balance not enough                 |
| COL_TOO_MUCH                 | Exceed collateral currency quota              |
| INIT_LTV_TOO_HIGH            | Init ltv too high                             |
| REDEEMED_LTV_TOO_HIGH        | Ltv too high after redeem                     |
| BORROWABLE_NOT_ENOUGH        | Left borrowable not enough                    |
| ORDER_TOO_MANY_TOTAL         | Exceed platform order count one day           |
| ORDER_TOO_MANY_DAILY         | Exceed single user order count one day        |
| ORDER_TOO_MANY_USER          | Exceed single user order count total          |
| ORDER_NOT_EXIST              | Order id not exist                            |
| ORDER_FINISHED               | Order id finished                             |
| ORDER_NO_PAY                 | Order unpaid amount is zero                   |
| ORDER_EXIST                  | Order exist                                   |
| ORDER_HISTORY_EXIST          | Order history exist                           |
| ORDER_REPAYING               | Order is repaying                             |
| ORDER_LIQUIDATING            | Order is liquidating                          |
| BORROW_TOO_LITTLE            | Less than currency min borrow amount          |
| BORROW_TOO_LARGE             | Greater than total max borrow amount quantity |
| REPAY_AMOUNT_INVALID         | Repay request amount invalid                  |
| REPAY_GREATER_THAN_AVAILABLE | Repay greater than available                  |
| POOL_BALANCE_NOT_ENOUGH      | Pool balance not enough                       |
| CURRENCY_SETTLING            | Currency settlement in progress               |
| RISK_REJECT                  | Risk reject, please try again later           |
| LOAN_FAILED                  | Loan failed, you can borrow again             |

- Portfolio related

| `label`                | Meaning                                |
|------------------------|----------------------------------------|
| USER_LIAB              | User has liab                          |
| USER_PENDING_ORDERS    | User has pending orders                |
| MODE_SET               | already set portfolio_margin mode      |

- Earn related

| `label`               | Meaning     |
|-----------------------|-----------|
| ERR_BALANCE_NOT_ENOUGH      | balance not enough |
| ERR_PRODUCT_SELL_OUT        | Target quota reached  |
| ERR_PRODUCT_BUY             | The project is not yet open for purchase |
| ERR_CREATE_ORDER            | Put order fail |
| ERR_QUOTA_LOWER_LIMIT       | Not meeting the minimum order amount |
| ERR_QUOTA_SUPERIOR_LIMIT    | The maximum order limit has been reached |
| ERR_ORDER_NUMBER_LIMIT      | The maximum order quantity has been reached |
| ERR_PRODUCT_CLOSE           | Project closed |
| COPIES_NOT_ENOUGH           | Not enough shares available to subscribe |
| COPIES_TOO_SMALL            | Investment share is too small |
| COPIES_TOO_BIG              | The number of investment shares exceeds the upper limit |
| TOTAL_AMOUNT_24             | The total amount of pledge and redemption within 24 hours exceeds the limit |
| TOTAL_BUYCOUNT_24           | Pledge and redemption times exceeding the limit within 24 hours |
| REDEEM_24_LIMIT             | Redemption are allowed 24 hours after the last staking |

- Server errors

| `label`      | Meaning                          |
|--------------|----------------------------------|
| INTERNAL     | Internal server error            |
| SERVER_ERROR | Internal server error            |
| INTERNAL_SERVER_ERROR | Operation failed, please try again later. (same as SERVER_ERROR) |
| TOO_BUSY     | Server is too busy at the moment |

- Flash Convert Related

| `label`                   | Meaning                                                                        |
|---------------------------|--------------------------------------------------------------------------------|
| INVALID_PARAM_VALUE       | Invalid request parameter                                                      |
| INVALID_CURRENCY          | Invalid currency                                                               |
| INVALID_CURRENCY_PAIR     | Invalid currency pair                                                          |
| PRICE_OBSOLETE            | The price was obsoleted                                                        |
| ORDER_NOT_FOUND           | Order not found                                                                |
| ORDER_BOOK_NOT_FOUND      | Order book not found                                                           |
| BALANCE_NOT_ENOUGH        | Not enough balance/balance transfer fail                                       |
| TOO_MANY_REQUESTS         | Request rate exceeds limits                                                    |
| QUOTA_NOT_ENOUGH          | Quota not enough,please reduce the amount or try again later                   |
| SERVER_TIMEOUT            | Service timeout                                                                |
| MISSING_REQUIRED_PARAM    | Missing required parameter                                                     |
| REQUEST_FORBIDDEN         | Asset management product is under liquidation; only USDT purchases are allowed |
| CONVERT_PREVIEW_EXPIRED   | The result of preview is expired                                               |
| CONVERT_PREVIEW_NOT_MATCH | The result of preview is not match                                             |
| AMOUNT_TOO_LITTLE         | Under minimum transaction amount                                               |
| AMOUNT_TOO_MUCH           | Over maximum transaction amount                                                |

</Block>

<Block>

# Alpha

Alpha Account

</Block>

<Block>

##  Alpha Account API

<a id="opIdlistAlphaAccounts"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Account API*

Query position assets

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphaaccounts-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Positions queried successfully|[Inline]|

<h3 id="listalphaaccounts-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» *None*|object|Query Position Return|
|»» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|»» available|string|Available Balance|
|»» locked|string|Locked balance|
|»» token_address|string|Token address|
|»» chain|string|Blockchain name|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Alpha Account Transaction History API

<a id="opIdlistAlphaAccountBook"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Account Transaction History API*

Query asset transactions

<h3 id="listalphaaccountbook-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|from|query|integer(int64)|true|Start timestamp for the query|
|to|query|integer(int64)|false|End timestamp for the query, defaults to current time if not specified|
|page|query|integer(int32)|false|Page number|
|limit|query|integer(int32)|false|Maximum 100 items per page|

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphaaccountbook-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Transaction history retrieved successfully|[Inline]|

<h3 id="listalphaaccountbook-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» *None*|object|Query Asset Flow Return|
|»» id|integer(int64)|Order ID|
|»» time|integer(int64)|Operation timestamp|
|»» currency|string(string)|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|»» change|string(string)|Change amount|
|»» balance|string(string)|Balance after change|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Alpha Quote API

<a id="opIdquoteAlphaOrder"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Quote API*

The quote_id returned by the price inquiry interface is valid for one minute; an order must be placed within this minute. If it times out, a new price inquiry is required. Rate-limited at 10 requests per second per user.

<Example>

> Body parameter

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

<h3 id="quotealphaorder-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|object|true|none|
|» currency|body|string|true|Alpha currency ID, for example USDS02b5a8dd2f59a098, not the display symbol USDS. First call GET /alpha/currencies without currency, match the token by name, chain and address, and use the returned currency field. Paginate with page and limit.|
|» side|body|string|true|Buy or sell orders<br>- buy<br>- sell|
|» amount|body|string|true|Trade Quantity<br>- `side` : `buy` refers to the quote currency, i.e., `USDT`<br>- `side` : `sell` refers to the base currency|
|» gas_mode|body|string|true|Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter|
|» slippage|body|string|false|Slippage tolerance (10 means 10% tolerance)|

#### Detailed descriptions

**» side**: Buy or sell orders<br>- buy<br>- sell

**» amount**: Trade Quantity<br>- `side` : `buy` refers to the quote currency, i.e., `USDT`<br>- `side` : `sell` refers to the base currency

**» gas_mode**: Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter

<Example>

> Example responses

> 200 Response

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

<h3 id="quotealphaorder-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Quote retrieved successfully|Inline|

<h3 id="quotealphaorder-responseschema">Response Schema</h3>

Status Code **200**

*Quote Response*

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» quote_id|string|Quote ID for order placement, valid for 1 minute|
|» min_amount|string|Minimum order size|
|» max_amount|string|Maximum order size|
|» price|string|Token Price (USDT-based)|
|» slippage|string|Slippage|
|» estimate_gas_fee_amount_usdt|string|Estimated Gas Fee (USDT-based)|
|» order_fee|string|Trading fee|
|» target_token_min_amount|string|Minimum received amount|
|» target_token_max_amount|string|Maximum received amount|
|» error_type|integer(int32)|Failure Type<br>- `0` : Success<br>- `1` : Exceeds maximum value<br>- `2` : Below minimum value|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Alpha Order API

<a id="opIdplaceAlphaOrder"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Order API*

Order placement interface, rate-limited at 5 requests per second per user.

<Example>

> Body parameter

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

<h3 id="placealphaorder-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|object|true|none|
|» currency|body|string|true|Alpha currency ID, for example USDS02b5a8dd2f59a098, not the display symbol USDS. First call GET /alpha/currencies without currency, match the token by name, chain and address, and use the returned currency field. Paginate with page and limit.|
|» side|body|string|true|Buy or sell orders<br>- buy<br>- sell|
|» amount|body|string|true|Trade Quantity<br>- `side` : `buy` refers to the quote currency, i.e., `USDT`<br>- `side` : `sell` refers to the base currency|
|» gas_mode|body|string|true|Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter|
|» slippage|body|string|false|Slippage percentage: "10" means 10%.|
|» quote_id|body|string|true|Quote ID returned from quotation API|

#### Detailed descriptions

**» side**: Buy or sell orders<br>- buy<br>- sell

**» amount**: Trade Quantity<br>- `side` : `buy` refers to the quote currency, i.e., `USDT`<br>- `side` : `sell` refers to the base currency

**» gas_mode**: Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter

<Example>

> Example responses

> 200 Response

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

<h3 id="placealphaorder-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Order placed successfully|Inline|

<h3 id="placealphaorder-responseschema">Response Schema</h3>

Status Code **200**

*Order response*

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» order_id|string|Order ID|
|» status|integer(int32)|Order Status<br>- `0` : All<br>- `1` : Processing<br>- `2` : Successful<br>- `3` : Failed<br>- `4` : Cancelled<br>- `5` : Buy order placed but transfer not completed<br>- `6` : Order cancelled but transfer not completed|
|» side|string|Buy or sell orders<br>- buy<br>- sell|
|» gas_mode|string|Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter|
|» create_time|integer(int64)|Creation timestamp|
|» amount|string|Trade Quantity<br>- `side` : `buy` refers to the quote currency, i.e., `USDT`<br>- `side` : `sell` refers to the base currency|
|» token_address|string|Token contract address|
|» chain|string|Blockchain name|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Alpha Order List API

<a id="opIdlistAlphaOrder"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Order List API*

<h3 id="listalphaorder-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|currency|query|string|false|Alpha currency ID, for example USDS02b5a8dd2f59a098, not the display symbol USDS. First call GET /alpha/currencies without currency, match the token by name, chain and address, and use the returned currency field. Paginate with page and limit.|
|side|query|string|false|Buy or sell orders<br>- buy<br>- sell|
|status|query|integer(int32)|false|Order Status<br>- `0` : All<br>- `1` : Processing<br>- `2` : Successful<br>- `3` : Failed<br>- `4` : Cancelled<br>- `5` : Buy order placed but transfer not completed<br>- `6` : Order cancelled but transfer not completed|
|from|query|integer(int64)|false|Start time for order query|
|to|query|integer(int64)|false|End time for order query, defaults to current time if not specified|
|limit|query|integer(int32)|false|Maximum number of items returned. Default: 100, minimum: 1, maximum: 100|
|page|query|integer(int32)|false|Page number|

#### Detailed descriptions

**side**: Buy or sell orders<br>- buy<br>- sell

**status**: Order Status<br>- `0` : All<br>- `1` : Processing<br>- `2` : Successful<br>- `3` : Failed<br>- `4` : Cancelled<br>- `5` : Buy order placed but transfer not completed<br>- `6` : Order cancelled but transfer not completed

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphaorder-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|List retrieved successfully|[Inline]|

<h3 id="listalphaorder-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|*None*|array|[Order response]|
|» *None*|object|Order response|
|»» order_id|string|Order ID|
|»» tx_hash|string|Transaction Hash|
|»» side|string|Buy or sell orders<br>- buy<br>- sell|
|»» usdt_amount|string|Amount (USDT)|
|»» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|»» currency_amount|string|Token amount|
|»» status|integer(int32)|Order Status<br>- `0` : All<br>- `1` : Processing<br>- `2` : Successful<br>- `3` : Failed<br>- `4` : Cancelled<br>- `5` : Buy order placed but transfer not completed<br>- `6` : Order cancelled but transfer not completed|
|»» gas_mode|string|Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter|
|»» chain|string|Blockchain|
|»» gas_fee|string|Gas Fee (USDT-based)|
|»» transaction_fee|string|Trading Fee (USDT-based)|
|»» failed_reason|string|Failure reason (if applicable)|
|»» create_time|integer(int64)|Creation timestamp|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Alpha Single Order Query API

<a id="opIdgetAlphaOrder"></a>

<Example sample-title="Code samples">

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
# for `gen_sign` implementation, refer to section `Authentication` above
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

*Alpha Single Order Query API*

<h3 id="getalphaorder-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|order_id|query|string|true|Order ID|

<Example>

> Example responses

> 200 Response

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

<h3 id="getalphaorder-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Order queried successfully|Inline|

<h3 id="getalphaorder-responseschema">Response Schema</h3>

Status Code **200**

*Order response*

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» order_id|string|Order ID|
|» tx_hash|string|Transaction Hash|
|» side|string|Buy or sell orders<br>- buy<br>- sell|
|» usdt_amount|string|Amount (USDT)|
|» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|» currency_amount|string|Token amount|
|» status|integer(int32)|Order Status<br>- `0` : All<br>- `1` : Processing<br>- `2` : Successful<br>- `3` : Failed<br>- `4` : Cancelled<br>- `5` : Buy order placed but transfer not completed<br>- `6` : Order cancelled but transfer not completed|
|» gas_mode|string|Trading mode affects slippage selection<br>- `speed` : Smart mode<br>- `custom` : Custom mode, uses `slippage` parameter|
|» chain|string|Blockchain|
|» gas_fee|string|Gas Fee (USDT-based)|
|» transaction_fee|string|Trading Fee (USDT-based)|
|» failed_reason|string|Failure reason (if applicable)|
|» create_time|integer(int64)|Creation timestamp|

</SchemaBlock>

::: warning
To perform this operation, you must be authenticated by API key and secret
:::

</Block>

<Block>

##  Query currency information

<a id="opIdlistAlphaCurrencies"></a>

<Example sample-title="Code samples">

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

*Query currency information*

When currency is provided, query and return specified currency information; when currency is not provided, return paginated currency list

<h3 id="listalphacurrencies-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|currency|query|string|false|Alpha currency ID, for example USDS02b5a8dd2f59a098, not the display symbol USDS. First call GET /alpha/currencies without currency, match the token by name, chain and address, and use the returned currency field. Paginate with page and limit.|
|limit|query|integer(int32)|false|Maximum number of records returned in a single list|
|page|query|integer(int32)|false|Page number|

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphacurrencies-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Query successful|[Inline]|

<h3 id="listalphacurrencies-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» AlphaCurrency|object|none|
|»» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|»» name|string|Display name or symbol, for example USDS; it cannot replace the currency field as the Alpha currency ID.|
|»» chain|string|The main chain corresponding to the coin|
|»» address|string|Contract Address|
|»» amount_precision|integer(int32)|Quantity precision|
|»» precision|integer(int32)|Price precision|
|»» status|integer(int32)|Currency Trading Status<br>- `1` : Normal trading<br>- `2` : Suspended trading<br>- `3` : Delisted|

</SchemaBlock>

<aside class="success">
This operation does not require authentication
</aside>

</Block>

<Block>

##  Query currency ticker

<a id="opIdlistAlphaTickers"></a>

<Example sample-title="Code samples">

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

*Query currency ticker*

When currency is provided, returns ticker information for the specified currency. When currency is not provided, returns paginated ticker list

<h3 id="listalphatickers-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|currency|query|string|false|Alpha currency ID, for example USDS02b5a8dd2f59a098, not the display symbol USDS. First call GET /alpha/currencies without currency, match the token by name, chain and address, and use the returned currency field. Paginate with page and limit.|
|limit|query|integer(int32)|false|Maximum number of records returned in a single list|
|page|query|integer(int32)|false|Page number|

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphatickers-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Query successful|[Inline]|

<h3 id="listalphatickers-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» AlphaTicker|object|none|
|»» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|»» last|string|Last trading price|
|»» change|string|24h price change percentage (negative for decrease, e.g., -7.45)|
|»» volume|string|24h Trading Volume (USDT)|
|»» market_cap|string|Current Token Market Cap|

</SchemaBlock>

<aside class="success">
This operation does not require authentication
</aside>

</Block>

<Block>

##  Query Token Information

<a id="opIdlistAlphaTokens"></a>

<Example sample-title="Code samples">

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

*Query Token Information*

Supports passing chain, platform, and address

<h3 id="listalphatokens-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|chain|query|string|false|Query Token List by Chain<br><br>- solana<br>- eth<br>- bsc<br>- base<br>- world<br>- sui<br>- arbitrum<br>- avalanche<br>- polygon<br>- linea<br>- optimism<br>- zksync<br>- gatelayer|
|launch_platform|query|string|false|Query Token List by Launch Platform<br><br>- meteora_dbc<br>- fourmeme<br>- moonshot<br>- pump<br>- raydium_launchlab<br>- letsbonk<br>- gatefun<br>- virtuals|
|address|query|string|false|Query Token List by Contract Address|
|page|query|integer(int32)|false|Page number|

#### Detailed descriptions

**chain**: Query Token List by Chain<br><br>- solana<br>- eth<br>- bsc<br>- base<br>- world<br>- sui<br>- arbitrum<br>- avalanche<br>- polygon<br>- linea<br>- optimism<br>- zksync<br>- gatelayer

**launch_platform**: Query Token List by Launch Platform<br><br>- meteora_dbc<br>- fourmeme<br>- moonshot<br>- pump<br>- raydium_launchlab<br>- letsbonk<br>- gatefun<br>- virtuals

<Example>

> Example responses

> 200 Response

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

<h3 id="listalphatokens-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|Query successful|[Inline]|

<h3 id="listalphatokens-responseschema">Response Schema</h3>

Status Code **200**

<SchemaBlock :threshold="10">

|Name|Type|Description|
|---|---|---|---|---|
|» currency|string|Alpha currency ID, for example USDS02b5a8dd2f59a098; use it as the currency parameter in Alpha APIs, not the display symbol USDS.|
|» name|string|Display name or symbol, for example USDS; it cannot replace the currency field as the Alpha currency ID.|
|» chain|string|The main chain corresponding to the coin|
|» address|string|Contract Address|
|» amount_precision|integer(int32)|Quantity precision|
|» precision|integer(int32)|Price precision|
|» status|integer(int32)|Currency Trading Status<br>- `1` : Normal trading<br>- `2` : Suspended trading<br>- `3` : Delisted|

</SchemaBlock>

<aside class="success">
This operation does not require authentication
</aside>

</Block>

