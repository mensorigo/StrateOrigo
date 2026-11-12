# Raw Journal 同步：VPS → 本地，逐文件 sha256 校验（docs/06 §3.2 A9、M1 交付物）。
#
# 用法（本地 PowerShell）：
#   powershell -ExecutionPolicy Bypass -File scripts/sync-journal.ps1 -Host <vps> [-User root] `
#       [-RemoteDir /opt/strateorigo/data/raw] [-LocalDir data/raw]
#
# 流程：
#   1. scp 递归拉取 RemoteDir 下全部 .journal.zst / .sha256 到临时目录（不直接写目标目录）
#   2. 逐文件按 .sha256 边车校验（sha256sum -c 等价逻辑，PowerShell 原生实现）
#   3. 校验通过的移动到 LocalDir（保留目录结构）；任何文件校验失败则整体退出 1，
#      临时目录保留供排查
#
# 设计要点：
#   - 先落临时目录再校验后移动——传输中断不会污染本地数据
#     （本地 data/raw 也是"原始数据不可变"的一部分，docs/06 §4 原则）
#   - 边车缺失的文件视为不完整（录制器按小时滚动收尾时才写边车；
#     当前小时仍在写入的文件没有边车，属于预期，跳过并提示）
#   - 已存在于本地且校验一致的文件跳过（幂等重跑）

param(
    [Parameter(Mandatory = $true)][string]$Host_,
    [string]$User = 'root',
    [string]$RemoteDir = '/opt/strateorigo/data/raw',
    [string]$LocalDir = 'data/raw'
)

# PowerShell 不允许 $Host（内置变量），参数名用 Host_，这里转回可读名。
$vpsHost = $Host_
$ErrorActionPreference = 'Stop'

function Assert-Tool {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "找不到 $Name（同步需要 OpenSSH 客户端）"
    }
}

Assert-Tool 'scp'
Assert-Tool 'ssh'

# $HostUser 形如 root@1.2.3.4
$dest = "${User}@${vpsHost}"
$stage = Join-Path $env:TEMP ("so-journal-sync-" + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force -Path $stage | Out-Null

Write-Host "1/3 拉取 $dest`:$RemoteDir → $stage"
# 尾随斜杠：拉取目录**内容**而非目录本身
scp -q -r "${dest}:$($RemoteDir.TrimEnd('/'))/" $stage
if ($LASTEXITCODE -ne 0) { throw "scp 失败（exit $LASTEXITCODE）" }

Write-Host "2/3 校验 sha256 边车"
# 录制器边车格式："<hex>  <path>`n"（hash.rs finish_current）
$failed = New-Object System.Collections.Generic.List[string]
$staged = New-Object System.Collections.Generic.List[string]
$missingSidecar = 0

$zstFiles = Get-ChildItem $stage -Recurse -Filter '*.journal.zst'
foreach ($f in $zstFiles) {
    $sidecar = "$($f.FullName).sha256"
    if (-not (Test-Path $sidecar)) {
        $missingSidecar++
        continue # 当前小时文件仍在写入，无边车属预期
    }
    $expected = (Get-Content $sidecar -TotalCount 1) -split '\s+' | Select-Object -First 1
    $actual = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $expected.ToLower()) {
        $failed.Add($f.FullName)
        Write-Host "    校验失败：$($f.FullName)`n      期望 $expected`n      实际 $actual" -ForegroundColor Red
    }
    else {
        $staged.Add($f.FullName)
    }
}
if ($missingSidecar -gt 0) {
    Write-Host "    跳过 $missingSidecar 个无边车文件（当前小时仍在写入，下次同步）" -ForegroundColor Yellow
}
if ($failed.Count -gt 0) {
    throw "校验失败 $($failed.Count) 个文件，临时目录保留：$stage"
}

Write-Host "3/3 落地到 $LocalDir（$($staged.Count) 个文件）"
foreach ($f in $staged) {
    # stage/<venue>/<date>/<HH>.<conn>.journal.zst → LocalDir/<venue>/<date>/...
    $rel = $f.Substring($stage.Length + 1)
    $target = Join-Path $LocalDir $rel
    $targetDir = Split-Path $target -Parent
    if (-not (Test-Path $targetDir)) { New-Item -ItemType Directory -Force -Path $targetDir | Out-Null }
    if (Test-Path $target) {
        # 幂等：已有文件校验一致则跳过
        $existing = (Get-FileHash $target -Algorithm SHA256).Hash.ToLower()
        $stagedHash = (Get-FileHash $f -Algorithm SHA256).Hash.ToLower()
        if ($existing -eq $stagedHash) { continue }
        throw "本地已存在不同内容的文件：$target（人工确认后再处理）"
    }
    Move-Item $f $target
    Move-Item "$f.sha256" "$target.sha256"
}

Remove-Item -Recurse -Force $stage
Write-Host "同步完成：$($staged.Count) 个文件已校验落地" -ForegroundColor Green
