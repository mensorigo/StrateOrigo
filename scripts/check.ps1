# 一键检查（docs/04：每次改动后必须通过）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/check.ps1
#       -Pytest 时跳过 Python 检查；-SkipInstall 时假设 nextest/deny/maturin 已可用
#       （若不在 PATH，会尝试用 ~/.cargo/bin 和 uv tool 目录中的副本）。

param(
    [switch]$NoPython,
    [switch]$SkipInstall
)

$ErrorActionPreference = 'Stop'
if ($env:CARGO_HOME) { $cargoBin = Join-Path $env:CARGO_HOME 'bin' }
else { $cargoBin = Join-Path $HOME '.cargo\bin' }
$env:PATH = "$cargoBin;$env:PATH"

function Assert-Cmd {
    param([string]$Name, [scriptblock]$Version)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "找不到 $Name，请先安装（见 docs/02-tech-stack.md §1）"
    }
    & $Version | Out-Null
}

$failures = New-Object System.Collections.Generic.List[string]

function Invoke-Step {
    param([string]$Name, [scriptblock]$Action)
    Write-Host "`n==> $Name" -ForegroundColor Cyan
    try {
        & $Action
        if ($LASTEXITCODE -ne 0) { throw "exit $LASTEXITCODE" }
    }
    catch {
        Write-Host "    失败：$_" -ForegroundColor Red
        $failures.Add($Name) | Out-Null
    }
}

if (-not $SkipInstall) {
    Invoke-Step 'rustup 工具链 (1.99.1)' {
        Assert-Cmd 'rustup' { rustup --version }
        $v = (rustup run stable rustc --version)
        if ($v -notmatch '1\.99\.1') {
            Write-Host "实际：$v（要求 1.99.1，见 docs/02 §6）"
            exit 1
        }
    }
}

Invoke-Step 'cargo fmt --check' { Assert-Cmd 'cargo' { cargo --version }; cargo fmt --all --check }
Invoke-Step 'cargo clippy' { cargo clippy --workspace --all-targets -- -D warnings }
Invoke-Step 'cargo nextest' {
    if (Get-Command cargo-nextest -ErrorAction SilentlyContinue) { cargo nextest run }
    else { Write-Host 'cargo-nextest 未安装，退回 cargo test'; cargo test --workspace }
}
Invoke-Step '文件行数 ≤ 400 (docs/04 F1)' {
    $limit = 400
    $warn = 250
    $bad = @(); $over = @()
    $roots = @('crates', 'apps')
    foreach ($root in $roots) {
        if (-not (Test-Path $root)) { continue }
        Get-ChildItem $root -Recurse -Include *.rs | ForEach-Object {
            $n = (Get-Content $_.FullName | Measure-Object -Line).Lines
            if ($n -gt $limit) { $bad += "$($_.FullName): $n 行" }
            elseif ($n -gt $warn) { $over += "$($_.FullName): $n 行" }
        }
    }
    if ($over) { Write-Host "超过 $warn 行（建议拆分）：`n  $($over -join "`n  ")" -ForegroundColor Yellow }
    if ($bad) { Write-Host "超过 $limit 行（CI 失败，docs/04 F1）：`n  $($bad -join "`n  ")" -ForegroundColor Red; exit 1 }
}
Invoke-Step 'cargo deny check' {
    if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
        # 经 cmd 转发丢弃 stderr 警告（duplicate 等 warn 级不导致失败）：
        # PS 5.1 下原生 stderr 重定向（2>$null）+ ErrorActionPreference=Stop
        # 会把 warn 输出升级为异常，造成 exit 0 也报"失败"的误报。
        cmd /c "cargo deny check 2>nul"
    }
    else { Write-Host 'cargo-deny 未安装，跳过（安装：cargo install cargo-deny --locked）' }
}

if (-not $NoPython -and (Test-Path python/pyproject.toml)) {
    Invoke-Step 'uv sync' { Assert-Cmd 'uv' { uv --version }; uv sync --project python }
    Invoke-Step 'ruff' {
        uv run --project python ruff check python
        uv run --project python ruff format --check python
    }
    Invoke-Step 'pyright' { uv run --project python pyright }
    Invoke-Step 'pytest' { uv run --project python pytest python }
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "失败：$($failures -join ', ')" -ForegroundColor Red
    exit 1
}
Write-Host '全部检查通过' -ForegroundColor Green
exit 0
