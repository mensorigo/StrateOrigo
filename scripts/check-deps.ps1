# 依赖分层机检（docs/04 §3.5 A1/A7）。
#
# 层级表**不在本脚本里**：唯一事实来源是 docs/04 §3.5 的 markdown 表格，
# 本脚本按行解析它，因此「改文档」与「改检查」不会漂移（A7 的立项理由）。
#
# 检查项
#   1. 每个 workspace 成员都必须登记层级（新 crate 忘了登记 ⇒ 失败）
#   2. 依赖只能由高编号层指向低编号层；同层边必须在例外表里（A1）
#   3. 不得出现环（A2；cargo 也会报，这里是显式断言）
#   4. 依赖图里不得出现 GUI crate（A5）
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
# 行形如：  | `层2` | `so-book`、`so-journal`、`（拟建）`so-portfolio`` |
# 同时接受 `LN` 写法，方便将来换标签。
$layers = @{}          # crate 名 → 层号
$inSection = $false
foreach ($line in $lines) {
    if ($line -match '^###\s+3\.5') { $inSection = $true; continue }
    if ($inSection -and $line -match '^###\s+3\.6') { break }
    if (-not $inSection) { continue }
    if ($line -notmatch '^\|\s*`(?:层|L)(\d)`\s*\|') { continue }
    $layer = [int]$Matches[1]
    # 第二格里的所有 `xx` 名字
    $cells = $line -split '\|'
    if ($cells.Count -lt 3) { continue }
    foreach ($m in [regex]::Matches($cells[2], '`([A-Za-z0-9_\-/*]+)`')) {
        $name = $m.Groups[1].Value
        if ($name -notlike 'so-*' -and $name -notlike 'apps*') { continue }
        if ($layers.ContainsKey($name)) {
            throw "docs/04 §3.5：crate 重复登记层级：$name"
        }
        $layers[$name] = $layer
    }
}
if ($layers.Count -eq 0) { throw "docs/04 §3.5 没解析出任何层级登记（表格格式变了？）" }

# ---- 解析同层例外表 ----
# 行形如：  | `so-data → so-recorder` | 层3 → 层3 | 现状 | 消除计划 |
$exceptions = @{}
$inException = $false
foreach ($line in $lines) {
    if ($line -match '同层依赖例外表') { $inException = $true; continue }
    if ($inException -and $line -match '^\|\s*`([a-z0-9\-]+)\s*→\s*([a-z0-9\-]+)`') {
        $exceptions["$($Matches[1]) → $($Matches[2])"] = $true
    }
    if ($inException -and $line -match '^##\s') { break }
}

# ---- 读依赖图 ----
$raw = (& cargo metadata --format-version 1 --no-deps 2>$null) | Out-String
if ($LASTEXITCODE -ne 0) { throw "cargo metadata 失败" }
try { $meta = $raw | ConvertFrom-Json }
catch { throw "解析 cargo metadata 失败（输出编码不是 UTF-8？）：$($_.Exception.Message)" }
$wsMembers = @{}
foreach ($id in $meta.workspace_members) { $wsMembers[$id] = $true }

$pkgName = @{}
foreach ($p in $meta.packages) { $pkgName[$p.id] = $p.name }

$failures = New-Object System.Collections.Generic.List[string]
$edges = New-Object System.Collections.Generic.List[object]

foreach ($p in $meta.packages) {
    if (-not $wsMembers.ContainsKey($p.id)) { continue }
    foreach ($d in $p.dependencies) {
        if (-not $d.name.StartsWith('so-')) { continue }
        $edges.Add([pscustomobject]@{ From = $p.name; To = $d.name; Kind = $(if ($d.kind) { $d.kind } else { 'normal' }) }) | Out-Null
    }
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

# 检查 1：未登记的 crate
foreach ($name in ($pkgName.Values | Sort-Object -Unique)) {
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
        if (-not $exceptions.ContainsKey($key)) {
            $failures.Add("同层依赖未登记例外：$key（L$lf → L$lt，$($e.Kind)）") | Out-Null
        }
    }
    elseif ($lf -lt $lt) {
        $failures.Add("依赖方向错误：$key（L$lf → L$lt，低层依赖高层）") | Out-Null
    }
    if ($Verbose) { Write-Host ("  {0,-28} L{1} → L{2}" -f $key, $lf, $lt) }
}

# 检查 2b：例外表里的边若已不存在，说明该行该删了（防止例外表变成永久豁免）
foreach ($k in $exceptions.Keys) {
    $parts = $k -split ' → '
    $still = $edges | Where-Object { $_.From -eq $parts[0] -and $_.To -eq $parts[1] }
    if (-not $still) {
        $failures.Add("例外表已过期：$k 不再存在，请删除该行（docs/04 §3.5）") | Out-Null
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

# ---- 输出 ----
if ($Verbose) {
    Write-Host "解析到 $($layers.Count) 条层级登记、$($edges.Count) 条 workspace 内依赖边" -ForegroundColor DarkGray
    foreach ($k in ($layers.Keys | Sort-Object { $layers[$_] }, { $_ })) {
        Write-Host ("  L{0}  {1}" -f $layers[$k], $k) -ForegroundColor DarkGray
    }
}

if ($failures.Count -gt 0) {
    Write-Host "依赖分层检查失败（docs/04 §3.5）：" -ForegroundColor Red
    foreach ($f in $failures) { Write-Host "  VIOLATION: $f" -ForegroundColor Red }
    exit 1
}
Write-Host "依赖分层检查通过（$($edges.Count) 条依赖边，$($layers.Count) 个登记项）" -ForegroundColor Green
exit 0
