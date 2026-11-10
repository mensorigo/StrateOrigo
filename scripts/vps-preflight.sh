#!/usr/bin/env bash
# VPS 运行前环境检查与配置（docs/01 §11 部署拓扑；在 VPS 上执行）。
#
# 用法：
#   bash scripts/vps-preflight.sh            # 只检查（缺什么报什么，exit 1）
#   bash scripts/vps-preflight.sh --fix      # 检查 + 自动安装/修复缺失项
#
# 检查项（M1 录制验收的最小运行条件）：
#   1. 操作系统 / 架构（Ubuntu LTS / x86_64 或 aarch64）
#   2. 时钟同步：chronyd 运行、NTP 已同步、时钟偏移量
#   3. 时区无关性（录制统一 UTC，但时区不影响 Unix 时间戳——仅提示）
#   4. 磁盘空间（data/ 所在分区 ≥ 5 GB 空闲）
#   5. 内存与 swap（编译需要，1 GB 内存机器必须有 swap）
#   6. 工具链：git、curl、build-essential、pkg-config
#   7. rustup + rustc 1.99.1（rust-toolchain.toml 驱动）
#   8. 出站连通性：Gate WS（fx-ws.gateio.ws:443）与 REST（api.gateio.ws）
#
# 只读检查不改动系统；--fix 只安装缺失的 apt 包与 rustup，不动任何
# 已有配置（不代用户改 NTP 服务选择——那是运维决策，缺服务时只报错）。

set -u

FIX=0
[ "${1:-}" = "--fix" ] && FIX=1

PASS=0; FAIL=0; FIXED=0
ok()   { echo "  ✅ $1"; PASS=$((PASS+1)); }
bad()  { echo "  ❌ $1"; FAIL=$((FAIL+1)); }
fix()  { echo "  🔧 已修复：$1"; FIXED=$((FIXED+1)); }
warn() { echo "  ⚠️  $1"; }

have() { command -v "$1" >/dev/null 2>&1; }

apt_install() {
    if [ "$FIX" -eq 1 ]; then
        sudo apt-get update -qq && sudo apt-get install -y -qq "$@" >/dev/null \
            && fix "apt install $*" || bad "apt install $* 失败（手动处理）"
    fi
}

echo "=== VPS 运行前检查（docs/01 §11）==="

# ---- 1. OS / 架构 ----
echo "[1/8] 操作系统与架构"
if [ -r /etc/os-release ]; then
    . /etc/os-release
    ok "$PRETTY_NAME ($(uname -m))"
else
    bad "无法读取 /etc/os-release（非标准 Linux？）"
fi

# ---- 2. 时钟同步（M1 延迟测量的地基）----
echo "[2/8] 时钟同步"
CHRONY_RUNNING=0
if have chronyc; then
    if chronyc tracking >/dev/null 2>&1; then
        CHRONY_RUNNING=1
        # system time 偏移：chronyc tracking 的 "System time" 行，如
        # "System time     : 0.000012 seconds fast of NTP time"
        OFFSET=$(chronyc tracking | awk -F: '/System time/ {print $2}' | awk '{print $1}')
        LEAP=$(chronyc tracking | awk '/Leap status/ {print $4}')
        if [ "$LEAP" != "Normal" ]; then
            bad "chrony Leap status = $LEAP（非 Normal，时间不可信）"
        else
            ok "chronyd 运行中，Leap=$LEAP"
        fi
        # 偏移阈值：ts_recv - ts_exch 的测量精度受此直接限制。
        # 100ms 延迟测量，1ms 时钟偏移即可接受；超过 10ms 数据可疑。
        if have awk; then
            OVER=$(echo "$OFFSET" | awk '{v=($1<0?-$1:$1); print (v>0.010)?1:0}')
            if [ "$OVER" = "1" ]; then
                bad "时钟偏移 ${OFFSET}s 超过 10ms——延迟测量不可信，检查 chrony 源"
            else
                ok "时钟偏移 ${OFFSET}s（≤10ms）"
            fi
        fi
    else
        bad "chronyc 存在但 tracking 失败（chronyd 未运行？）"
        [ "$FIX" -eq 1 ] && sudo systemctl enable --now chrony >/dev/null 2>&1 \
            && fix "启动 chrony" || true
    fi
else
    # systemd-timesyncd 也能同步时间，只是 chrony 更精；允许，但查同步状态
    if timedatectl show 2>/dev/null | grep -q "NTPSynchronized=yes"; then
        warn "未装 chrony，但 systemd-timesyncd 已同步（可接受；建议换 chrony 提精度）"
        OFFSET=$(timedatectl show -p NTPMessage --value 2>/dev/null | grep -oE 'offset [0-9.]+[mu]?s' || echo "未知")
        ok "NTP 已同步（$OFFSET）"
    else
        bad "无 chrony 且 NTP 未同步——ts_recv 时间戳不可信（docs/01 §11 要求 chrony）"
        [ "$FIX" -eq 1 ] && apt_install chrony
    fi
fi

# ---- 3. 时区提示 ----
echo "[3/8] 时区"
TZ_NOW=$(timedatectl show -p Timezone --value 2>/dev/null || echo "unknown")
warn "时区 $TZ_NOW（录制内部全用 UTC，不影响数据；仅影响人类读日志的直觉）"

# ---- 4. 磁盘空间 ----
echo "[4/8] 磁盘空间"
# 项目目录（脚本所在 repo）或当前目录
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd || pwd)"
FREE_GB=$(df -BG --output=avail "$ROOT_DIR" 2>/dev/null | tail -1 | tr -dc '0-9')
if [ "${FREE_GB:-0}" -ge 5 ]; then
    ok "data/ 分区空闲 ${FREE_GB} GB（≥5）"
else
    bad "data/ 分区空闲仅 ${FREE_GB} GB——10 品种实测约 0.6 GB/天（压缩后），"
    bad "  20 品种 1 天约 1.5–2 GB，另需编译空间 ~2 GB，合计建议 ≥5 GB"
fi

# ---- 5. 内存与 swap ----
echo "[5/8] 内存与 swap"
MEM_MB=$(awk '/MemTotal/ {print int($2/1024)}' /proc/meminfo)
SWAP_MB=$(awk '/SwapTotal/ {print int($2/1024)}' /proc/meminfo)
ok "内存 ${MEM_MB} MB，swap ${SWAP_MB} MB"
if [ "$MEM_MB" -lt 1800 ] && [ "$SWAP_MB" -lt 2000 ]; then
    bad "内存 <1.8GB 且 swap <2GB——cargo 链接会 OOM（编译期需要，运行期录制器仅 ~30MB）"
    if [ "$FIX" -eq 1 ]; then
        if sudo fallocate -l 2G /swapfile 2>/dev/null \
           && sudo chmod 600 /swapfile \
           && sudo mkswap /swapfile >/dev/null 2>&1 \
           && sudo swapon /swapfile 2>/dev/null; then
            fix "创建并启用 /swapfile (2G)（重启失效；长期用请写 /etc/fstab）"
        else
            bad "swap 创建失败（可能已存在或权限不足）"
        fi
    fi
fi

# ---- 6. 基础工具 ----
echo "[6/8] 基础工具链"
# 检查单个命令 → 对应的安装包（gcc/make 都在 build-essential 里）
for t in git curl pkg-config make gcc; do
    if have "$t"; then ok "$t"; else
        bad "缺 $t"
        case "$t" in
            git)  apt_install git ;;
            curl) apt_install curl ;;
            *)    apt_install build-essential pkg-config ;;
        esac
    fi
done

# ---- 7. Rust 工具链 ----
echo "[7/8] Rust 工具链"
if have cargo; then
    # 在项目目录下 rust-toolchain.toml 自动选择 1.99.1
    RV=$(cd "$ROOT_DIR" 2>/dev/null && rustc --version 2>/dev/null || rustc --version 2>/dev/null)
    case "$RV" in
        *1.99.1*) ok "rustc 1.99.1（rust-toolchain.toml 生效）" ;;
        "")       bad "cargo 存在但 rustc 不可用" ;;
        *)        bad "rustc 版本 $RV ≠ 1.99.1（进入项目目录后 rustup 会自动装对版本，或 rustup toolchain install）" ;;
    esac
else
    bad "未安装 rustup/cargo"
    if [ "$FIX" -eq 1 ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable >/dev/null 2>&1 \
            && fix "安装 rustup（需重新登录 shell 或 source \$HOME/.cargo/env）" \
            || bad "rustup 安装失败"
    fi
fi

# ---- 8. 出站连通性（Gate）----
echo "[8/8] Gate 出站连通性"
# TCP 443 握手即可，不依赖 curl 的 DNS/证书行为差异
check_tcp() {
    local host=$1
    if exec 3<>/dev/tcp/"$host"/443 2>/dev/null; then
        exec 3<&- 3>&-
        ok "$host:443 可达"
    else
        bad "$host:443 不可达——录制无法工作（防火墙/出站封锁）"
    fi
}
check_tcp "fx-ws.gateio.ws"
check_tcp "api.gateio.ws"
# REST 延迟采样（3 次，看抖动；选址报告的另一输入）
if have curl; then
    echo "  --- REST 延迟采样（GET /futures/usdt/contracts?contract=BTC_USDT）---"
    for i in 1 2 3; do
        T=$(curl -s -o /dev/null -m 15 -w '%{time_total}' \
            -H "X-Gate-Size-Decimal: 1" \
            "https://api.gateio.ws/api/v4/futures/usdt/contracts?contract=BTC_USDT")
        if [ -n "$T" ] && [ "$T" != "0.000000" ]; then
            echo "    采样 $i: ${T}s"
        else
            echo "    采样 $i: 失败/超时"
            bad "REST 请求失败（i=$i）——跨境限速或墙的证据，记录进选址报告"
        fi
        sleep 1
    done
fi

echo ""
echo "=== 结果：$PASS 通过，$FAIL 失败，$FIXED 已修复 ==="
if [ "$FAIL" -gt 0 ]; then
    echo "存在失败项：先处理 ❌ 再部署录制器（--fix 可自动装缺的包/swap/rustup）。"
    exit 1
fi
echo "环境就绪。下一步：cargo build --release --bin so-cli"
