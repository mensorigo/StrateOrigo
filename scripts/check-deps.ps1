# 依赖分层机检（docs/04 §3.5 A1/A7）。
#
# 层级表**不在本脚本里**：唯一事实来源是 docs/04 §3.5 的 markdown 表格，
# 本脚本按行解析它，因此「改文档」与「改检查」不会漂移（A7 的立项理由）。
#
# 检查项
#   1.  每个 workspace 成员都必须登记层级（新 crate 忘了登记 ⇒ 失败）
#   2.  依赖只能由高编号层指向低编号层；同层边必须在例外表里（A1）
#   2b. 例外表的边若已不存在 ⇒ 过期，失败；但**规划**边（两端 crate 未建）为休眠
#   3.  不得出现环（A2；cargo 也会报，这里是显式断言）
#   4.  依赖图里不得出现 GUI crate（A5）
#   4b. so-engine 不得依赖 tokio/网络库（A3；so-engine 未建时跳过）
#
# 用法：powershell -ExecutionPolicy Bypass -File scripts/check-deps.ps1 [-Verbose]

param([switch]$Verbose)

$ErrorActionPreference = 'Stop'
# 固定子进程输出的解码为 UTF-8：cargo metadata 的 description 含中文，
# 若跟随调用方的控制台代码页（CI 里常是 GBK），JSON 会被解成非法字节而解析失败。
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$repoRoot = Split-Path -Parent $PSScriptRoot
$docPath = Join-Path $repoRoot 'docs\04-dev-constraints.md'

if (-not (Test-Path $docPath)) { throw "找不到 docs/04-dev-constraints.md" }

$lines = Get-Content $docPath -Encoding UTF8

# ---- 解析层级表 ----
# 行形如：  | `层2` | `so-book`、`so-journal` | 已建 |
#           |        | `so-strategy`、`so-signal` | 规划 |   ← 第一格留空 = 继承上一行层号
# 续行是必需的：已建与规划分开写状态，但两者都算登记（规划项建成前不在 cargo metadata 里，
# 所以"未登记即失败"的检查 1 对它们不生效——检查的是"代码里有的必须在表里"）。
# 同时接受 `LN` 写法，方便将来换标签。
$layers = @{}          # crate 名 → 层号
$inSection = $false
$curLayer = -1
foreach ($line in $lines) {
    if ($line -match '^###\s+3\.5') { $inSection = $true; $curLayer = -1; continue }
    if ($inSection -and $line -match '^###\s+3\.6') { break }
    if (-not $inSection) { continue }

    if ($line -match '^\|\s*`(?:层|L)(\d)`\s*\|') {
        $curLayer = [int]$Matches[1]
    }
    elseif ($line -match '^\|\s*\|') {
        if ($curLayer -lt 0) { continue }     # 层级表之前的表格，忽略
    }
    else {
        continue
    }

    # 第二格里的所有 `xx` 名字
    $cells = $line -split '\|'
    if ($cells.Count -lt 3) { continue }
    foreach ($m in [regex]::Matches($cells[2], '`([A-Za-z0-9_\-/*]+)`')) {
        $name = $m.Groups[1].Value
        # 登记名可以是 so-*、apps*、so-adapters/*（通配）、st-*（crates/strategies/ 下的策略 crate）。
        # 其余（状态列里的词等）忽略。
        if ($name -notlike 'so-*' -and $name -notlike 'apps*' -and $name -notlike 'st-*') { continue }
        if ($layers.ContainsKey($name)) {
            throw "docs/04 §3.5：crate 重复登记层级：$name"
        }
        $layers[$name] = $curLayer
    }
}
if ($layers.Count -eq 0) { throw "docs/04 §3.5 没解析出任何层级登记（表格格式变了？）" }

# ---- 解析同层例外表 ----
# 行形如：  | `so-backtest → so-engine` | 层3 → 层3 | 规划 | 理由与消除计划 |
# 两端名支持 `so-adapters/*` 这类通配。「现状」列为 `规划` 的行，若两端 crate 尚未建立
# ⇒ **休眠**（不判过期）；两端一建立，检查 2 立刻开始机检，无需回头补登记。
$exceptionRows = New-Object System.Collections.Generic.List[object]
$inException = $false
foreach ($line in $lines) {
    if ($line -match '同层依赖例外表') { $inException = $true; continue }
    if ($inException -and $line -match '^##\s') { break }
    if (-not $inException -or $line -notmatch '^\|') { continue }
    $cells = $line -split '\|'
    if ($cells.Count -lt 4) { continue }
    $m = [regex]::Match($cells[1], '`([A-Za-z0-9_\-/*]+)\s*→\s*([A-Za-z0-9_\-/*]+)`')
    if (-not $m.Success) { continue }
    $exceptionRows.Add([pscustomobject]@{
            Label   = "$($m.Groups[1].Value) → $($m.Groups[2].Value)"
            From    = $m.Groups[1].Value
            To      = $m.Groups[2].Value
            Planned = ($cells[3] -match '规划')
        }) | Out-Null
}

# ---- 读依赖图 ----
$raw = (& cargo metadata --format-version 1 --no-deps 2>$null) | Out-String
if ($LASTEXITCODE -ne 0) { throw "cargo metadata 失败" }
try { $meta = $raw | ConvertFrom-Json }
catch { throw "解析 cargo metadata 失败（输出编码不是 UTF-8？）：$($_.Exception.Message)" }
$wsMembers = @{}
foreach ($id in $meta.workspace_members) { $wsMembers[$id] = $true }

$wsNames = New-Object System.Collections.Generic.List[string]
foreach ($p in $meta.packages) {
    if (-not $wsMembers.ContainsKey($p.id)) { continue }
    $wsNames.Add($p.name) | Out-Null
}

$failures = New-Object System.Collections.Generic.List[string]
$dormant = New-Object System.Collections.Generic.List[string]
$edges = New-Object System.Collections.Generic.List[object]

foreach ($p in $meta.packages) {
    if (-not $wsMembers.ContainsKey($p.id)) { continue }
    foreach ($d in $p.dependencies) {
        if (-not $d.name.StartsWith('so-')) { continue }
        $edges.Add([pscustomobject]@{ From = $p.name; To = $d.name; Kind = $(if ($d.kind) { $d.kind } else { 'normal' }) }) | Out-Null
    }
}

# ---- 名字模式 → 正则（末尾 `/*` 视为前缀通配）----
function Get-NameRegex {
    param([string]$Pattern)
    if ($Pattern.EndsWith('/*')) {
        return '^' + [regex]::Escape($Pattern.Substring(0, $Pattern.Length - 1))
    }
    return '^' + [regex]::Escape($Pattern) + '$'
}

# ---- 解析 crate 名 → 层号（支持 `so-adapters/*` 前缀通配）----
function Get-Layer {
    param([string]$Name)
    if ($layers.ContainsKey($Name)) { return $layers[$Name] }
    foreach ($k in $layers.Keys) {
        if ($k.EndsWith('/*')) {
            $prefix = $k.Substring(0, $k.Length - 1)
            if ($Name.StartsWith($prefix)) { return $layers[$k] }
        }
    }
    return -1
}

$exceptionMatchers = New-Object System.Collections.Generic.List[object]
foreach ($row in $exceptionRows) {
    $exceptionMatchers.Add([pscustomobject]@{
            Row    = $row
            FromRe = Get-NameRegex $row.From
            ToRe   = Get-NameRegex $row.To
        }) | Out-Null
}

function Test-Exception {
    param([string]$From, [string]$To)
    foreach ($x in $exceptionMatchers) {
        if ($From -match $x.FromRe -and $To -match $x.ToRe) { return $true }
    }
    return $false
}

# 检查 1：未登记的 crate
foreach ($name in ($wsNames | Sort-Object -Unique)) {
    if ((Get-Layer $name) -eq -1) {
        $failures.Add("未登记层级：$name（docs/04 §3.5 层级表）") | Out-Null
    }
}

# 检查 2：依赖方向
foreach ($e in $edges) {
    $lf = Get-Layer $e.From
    $lt = Get-Layer $e.To
    if ($lf -eq -1 -or $lt -eq -1) { continue }   # 已由检查 1 报出
    $key = "$($e.From) → $($e.To)"
    if ($lf -eq $lt) {
        if (-not (Test-Exception $e.From $e.To)) {
            $failures.Add("同层依赖未登记例外：$key（L$lf → L$lt，$($e.Kind)）") | Out-Null
        }
    }
    elseif ($lf -lt $lt) {
        $failures.Add("依赖方向错误：$key（L$lf → L$lt，低层依赖高层）") | Out-Null
    }
    if ($Verbose) { Write-Host ("  {0,-28} L{1} → L{2}" -f $key, $lf, $lt) }
}

# 检查 2b：例外表与依赖图对齐（防止例外表变成永久豁免，也防止它写成空头支票）
foreach ($x in $exceptionMatchers) {
    $row = $x.Row
    $fromExists = @($wsNames | Where-Object { $_ -match $x.FromRe }).Count -gt 0
    $toExists = @($wsNames | Where-Object { $_ -match $x.ToRe }).Count -gt 0
    if (-not $fromExists -or -not $toExists) {
        if ($row.Planned) {
            # 规划边：两端未齐，休眠。建成后本行自动转入正式机检。
            $dormant.Add($row.Label) | Out-Null
        }
        else {
            $failures.Add("例外表两端有 crate 不存在：$($row.Label)（既非规划项，又对不上依赖图）") | Out-Null
        }
        continue
    }
    $still = @($edges | Where-Object { $_.From -match $x.FromRe -and $_.To -match $x.ToRe }).Count -gt 0
    if (-not $still) {
        $failures.Add("例外表已过期：$($row.Label) 已不存在，请删除该行（docs/04 §3.5）") | Out-Null
    }
}

# 检查 3：环
$graph = @{}
foreach ($e in $edges) {
    if (-not $graph.ContainsKey($e.From)) { $graph[$e.From] = New-Object System.Collections.Generic.List[string] }
    $graph[$e.From].Add($e.To) | Out-Null
}
$state = @{}   # 0=未访问 1=在栈 2=已完成
function Test-Cycle {
    param([string]$Node, [string]$Path)
    $state[$Node] = 1
    if ($graph.ContainsKey($Node)) {
        foreach ($next in $graph[$Node]) {
            if ($state[$next] -eq 1) {
                $failures.Add("存在循环依赖：$Path → $next") | Out-Null
                return
            }
            if (-not $state.ContainsKey($next) -or $state[$next] -eq 0) {
                Test-Cycle $next "$Path → $next"
            }
        }
    }
    $state[$Node] = 2
}
foreach ($n in $graph.Keys) { if (-not $state.ContainsKey($n) -or $state[$n] -eq 0) { Test-Cycle $n $n } }

# 检查 4：GUI 依赖（A5：服务端完全无头）。deny.toml 的 [bans] 是第二道闸。
$guiBanned = @('gpui', 'gpui-component', 'egui', 'eframe', 'iced', 'slint', 'tauri', 'winit', 'druid', 'qt_core')
foreach ($p in $meta.packages) {
    if (-not $wsMembers.ContainsKey($p.id)) { continue }
    foreach ($d in $p.dependencies) {
        if ($guiBanned -contains $d.name) {
            $failures.Add("GUI 依赖（A5 服务端完全无头）：$($p.name) → $($d.name)") | Out-Null
        }
    }
}

# 检查 4b：A3 —— so-engine 不得依赖异步运行时/网络库（内核必须能在无网络的确定性环境跑）。
# 未建立时跳过：这条检查是给未来的 crate 立的护栏，今天不应因"还没写"而失败。
$netBanned = @(
    'tokio', 'tokio-util', 'hyper', 'reqwest', 'tokio-tungstenite', 'tungstenite',
    'tokio-rustls', 'rustls', 'native-tls', 'openssl', 'ureq', 'isahc', 'awc', 'async-std', 'smol'
)
$engine = @($meta.packages | Where-Object { $wsMembers.ContainsKey($_.id) -and $_.name -eq 'so-engine' })
if ($engine.Count -gt 0) {
    foreach ($d in $engine[0].dependencies) {
        if ($netBanned -contains $d.name) {
            $failures.Add("A3 违规：so-engine → $($d.name)（内核不得依赖网络/异步运行时）") | Out-Null
        }
    }
}
elseif ($Verbose) {
    Write-Host "  so-engine 尚未建立，跳过检查 4b（A3 网络依赖）" -ForegroundColor DarkGray
}

# ---- 输出 ----
if ($Verbose) {
    Write-Host "解析到 $($layers.Count) 条层级登记、$($edges.Count) 条 workspace 内依赖边" -ForegroundColor DarkGray
    foreach ($k in ($layers.Keys | Sort-Object { $layers[$_] }, { $_ })) {
        Write-Host ("  L{0}  {1}" -f $layers[$k], $k) -ForegroundColor DarkGray
    }
    foreach ($d in $dormant) {
        Write-Host "  休眠例外（两端未建）：$d" -ForegroundColor DarkGray
    }
}

if ($failures.Count -gt 0) {
    Write-Host "依赖分层检查失败（docs/04 §3.5）：" -ForegroundColor Red
    foreach ($f in $failures) { Write-Host "  VIOLATION: $f" -ForegroundColor Red }
    exit 1
}
$tail = if ($dormant.Count -gt 0) { "，$($dormant.Count) 条休眠例外（规划）" } else { "" }
Write-Host "依赖分层检查通过（$($edges.Count) 条依赖边，$($layers.Count) 个登记项$tail）" -ForegroundColor Green
exit 0
