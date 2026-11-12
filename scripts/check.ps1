# 一键检查（docs/04：每次改动后必须通过）。
# 用法：powershell -ExecutionPolicy Bypass -File scripts/check.ps1
#       -Pytest 时跳过 Python 检查；-SkipInstall 时假设 nextest/deny/maturin 已可用
#       （若不在 PATH，会尝试用 ~/.cargo/bin 和 uv tool 目录中的副本）。

param(
    [switch]$NoPython,
    [switch]$SkipInstall,
    # 自动安装缺失的工具（rustup 组件、cargo-nextest、cargo-deny、Python 检查器）。
    # 只装 docs/02 §1 工具链清单内的项；不升级已有版本。
    [switch]$Install
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

# 缺工具时的提示/自动安装。$Install 打开时真正执行安装；否则只打印命令。
function Ensure-Tool {
    param([string]$Name, [scriptblock]$InstallAction, [string]$InstallHint)
    if (Get-Command $Name -ErrorAction SilentlyContinue) { return $true }
    if ($Install) {
        Write-Host "    安装 $Name ..." -ForegroundColor Yellow
        try {
            & $InstallAction
            if ($LASTEXITCODE -ne 0) { throw "exit $LASTEXITCODE" }
            if (Get-Command $Name -ErrorAction SilentlyContinue) {
                Write-Host "    已安装 $Name" -ForegroundColor Green
                return $true
            }
            # rustup 组件等装完不在当前 PATH 的情况：补探 PATH 后再试
            $env:PATH = "$cargoBin;$env:PATH"
            if (Get-Command $Name -ErrorAction SilentlyContinue) { return $true }
            throw "安装后仍找不到 $Name"
        }
        catch {
            Write-Host "    安装失败：$_" -ForegroundColor Red
            Write-Host "    手动安装：$InstallHint" -ForegroundColor Yellow
            return $false
        }
    }
    else {
        Write-Host "    缺 $Name（$InstallHint）" -ForegroundColor Yellow
        Write-Host "    或运行 scripts/check.ps1 -Install 自动安装" -ForegroundColor Yellow
        return $false
    }
}

function Invoke-Step {
    param([string]$Name, [scriptblock]$Action)
    Write-Host "`n==> $Name" -ForegroundColor Cyan
    try {
        & $Action
        # 纯 cmdlet 步骤不产生 LASTEXITCODE（保持 null），不能视为失败
        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) { throw "exit $LASTEXITCODE" }
    }
    catch {
        Write-Host "    失败：$_" -ForegroundColor Red
        $failures.Add($Name) | Out-Null
    }
}

# ---- 工具链完备性检查（docs/02 §1；缺失项 -Install 自动安装）----

Invoke-Step '工具链检查 (docs/02 §1)' {
    $missing = 0
    # Rust 核心：rustup / cargo / rustfmt / clippy（版本在后续步骤验证）
    foreach ($t in @('rustup', 'cargo', 'rustfmt', 'cargo-clippy')) {
        if (-not (Ensure-Tool $t {
                if ($t -eq 'rustup') {
                    throw "rustup 需手动安装（https://rustup.rs 或 winget install Rustlang.Rustup）"
                }
                rustup component add $(if ($t -eq 'rustfmt') { 'rustfmt' } else { 'clippy' })
            } "rustup component add $($t -replace 'cargo-', '')") ) { $missing++ }
    }
    # cargo-nextest（nextest 缺失时 nextest 步骤会退回 cargo test，这里给统一入口）
    Ensure-Tool 'cargo-nextest' { cargo install cargo-nextest --locked } `
        'cargo install cargo-nextest --locked' | Out-Null
    # cargo-deny（许可证/漏洞/重复依赖，docs/02 §6）
    Ensure-Tool 'cargo-deny' { cargo install cargo-deny --locked } `
        'cargo install cargo-deny --locked' | Out-Null
    # Python 侧：uv（包管理，3.12 由 .python-version + uv 自动提供）
    Ensure-Tool 'uv' {
        throw "uv 需手动安装（powershell -c 'irm https://astral.sh/uv/install.ps1 | iex'）"
    } "powershell -c 'irm https://astral.sh/uv/install.ps1 | iex'" | Out-Null
    # maturin（PyO3 构建；uv 环境里 dev 组自带，这里查全局 PATH）
    # protoc（M3 gRPC 阶段才需要；先探测提示不阻塞）
    if (-not (Get-Command protoc -ErrorAction SilentlyContinue)) {
        Write-Host "    提示：protoc 未装（M3 gRPC 才需要；届时可改用 protox 免装）" -ForegroundColor DarkGray
    }
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Host "    缺 git（winget install Git.Git）" -ForegroundColor Yellow
        $missing++
    }
    if ($missing -gt 0) { throw "$missing 个必需工具缺失" }
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
    $have = Ensure-Tool 'cargo-nextest' { cargo install cargo-nextest --locked } `
        'cargo install cargo-nextest --locked'
    if ($have) { cargo nextest run }
    else { Write-Host 'cargo-nextest 未安装，退回 cargo test'; cargo test --workspace }
}
Invoke-Step '依赖分层 (docs/04 A1/A7)' {
    foreach ($name in (Get-ChildItem scripts -Filter *.ps1).Name) {
        # 脚本自身必须是 UTF-8 带 BOM：Windows PowerShell 5.1 无 BOM 时按 ANSI 读，
        # 含中文的 .ps1 会解析失败或乱码（曾发生过）。
        $bytes = [System.IO.File]::ReadAllBytes((Join-Path 'scripts' $name))
        if ($bytes.Length -ge 3 -and -not ($bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)) {
            throw "scripts/$name 缺少 UTF-8 BOM（PowerShell 5.1 会按 ANSI 读取）"
        }
    }
    powershell -ExecutionPolicy Bypass -File scripts/check-deps.ps1
}

Invoke-Step '文件行数 ≤ 400 (docs/04 F1)' {
    $limit = 400
    $warn = 250
    $bad = @(); $over = @()
    # F1 管的是"源文件"，Rust 与 Python 都算（此前只扫 *.rs，Python 侧形同豁免）。
    # 排除构建产物与第三方依赖目录，否则 .venv / target 里的文件会被算成我们的。
    $roots = @('crates', 'apps', 'python')
    $skipDir = '\\(target|\.venv|venv|__pycache__|node_modules|\.git)\\'
    foreach ($root in $roots) {
        if (-not (Test-Path $root)) { continue }
        Get-ChildItem $root -Recurse -Include *.rs, *.py -File |
            Where-Object { $_.FullName -notmatch $skipDir } |
            ForEach-Object {
                $n = (Get-Content $_.FullName | Measure-Object -Line).Lines
                if ($n -gt $limit) { $bad += "$($_.FullName): $n 行" }
                elseif ($n -gt $warn) { $over += "$($_.FullName): $n 行" }
            }
    }
    if ($over) { Write-Host "超过 $warn 行（建议拆分）：`n  $($over -join "`n  ")" -ForegroundColor Yellow }
    if ($bad) { Write-Host "超过 $limit 行（CI 失败，docs/04 F1）：`n  $($bad -join "`n  ")" -ForegroundColor Red; exit 1 }
}
Invoke-Step 'cargo deny check' {
    $have = Ensure-Tool 'cargo-deny' { cargo install cargo-deny --locked } `
        'cargo install cargo-deny --locked'
    if ($have) {
        # 经 cmd 转发丢弃 stderr 警告（duplicate 等 warn 级不导致失败）：
        # PS 5.1 下原生 stderr 重定向（2>$null）+ ErrorActionPreference=Stop
        # 会把 warn 输出升级为异常，造成 exit 0 也报"失败"的误报。
        cmd /c "cargo deny check 2>nul"
    }
    else { $failures.Add('cargo deny check') | Out-Null; Write-Host 'cargo-deny 缺失，本轮跳过' }
}

if (-not $NoPython -and (Test-Path python/pyproject.toml)) {
    Invoke-Step 'uv 环境与 Python 3.12' {
        $have = Ensure-Tool 'uv' {
            throw "uv 需手动安装（powershell -c 'irm https://astral.sh/uv/install.ps1 | iex'）"
        } "powershell -c 'irm https://astral.sh/uv/install.ps1 | iex'"
        if (-not $have) { throw 'uv 缺失' }
        uv sync --project python
    }
    Invoke-Step 'ruff' {
        $have = Ensure-Tool 'ruff' { uv tool install ruff } 'uv tool install ruff'
        if ($have) {
            uv run --project python ruff check python
            uv run --project python ruff format --check python
        }
        else { throw 'ruff 缺失' }
    }
    Invoke-Step 'pyright' {
        $have = Ensure-Tool 'pyright' { uv tool install pyright } 'uv tool install pyright'
        if ($have) { uv run --project python pyright }
        else { throw 'pyright 缺失' }
    }
    Invoke-Step 'pytest' { uv run --project python pytest python }
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "失败：$($failures -join ', ')" -ForegroundColor Red
    exit 1
}
Write-Host '全部检查通过' -ForegroundColor Green
exit 0
