#!/usr/bin/env bash
# Peregrine 本地自动化测试脚本（Linux / macOS）
#
# 覆盖 OpenSpec 变更的自动化验证项：
#   - Rust 单元测试（config / material / peregrine 三个 crate）
#   - Clippy lint（-D warnings）
#   - 代码格式检查
#   - TypeScript 类型检查
#   - 前端构建
#   - Windows MSVC 目标交叉编译检查（仅 check，验证 Windows 代码路径语法正确）
#
# 用法：
#   bash scripts/test.sh           # 全部测试
#   bash scripts/test.sh rust      # 仅 Rust 部分
#   bash scripts/test.sh frontend  # 仅前端部分

set -euo pipefail

# 确保 cargo 在 PATH 中（rustup 默认安装位置）
export PATH="$HOME/.cargo/bin:$PATH"

cd "$(dirname "$0")/.."

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass=0
fail=0
failed_steps=()

step() {
    echo -e "\n${YELLOW}==> $1${NC}"
}

ok() {
    echo -e "${GREEN}✔ $1${NC}"
    pass=$((pass + 1))
}

bad() {
    echo -e "${RED}✘ $1${NC}"
    fail=$((fail + 1))
    failed_steps+=("$1")
}

run_rust() {
    step "Rust 单元测试（config / material / peregrine）"
    if cargo test -p peregrine_config -p peregrine_material -p peregrine; then
        ok "cargo test"
    else
        bad "cargo test"
    fi

    step "Clippy lint（-D warnings）"
    if cargo clippy -p peregrine_config -p peregrine_material -p peregrine -- -D warnings; then
        ok "cargo clippy"
    else
        bad "cargo clippy"
    fi

    step "代码格式检查"
    if cargo fmt --all -- --check; then
        ok "cargo fmt"
    else
        bad "cargo fmt（运行 cargo fmt --all 修复）"
    fi

    step "Windows MSVC 交叉编译检查（cargo check）"
    if rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
        if cargo check --target x86_64-pc-windows-msvc \
            -p peregrine_config -p peregrine_material -p peregrine; then
            ok "Windows 目标编译检查"
        else
            bad "Windows 目标编译检查"
        fi
    else
        echo "跳过：未安装 x86_64-pc-windows-msvc target（rustup target add x86_64-pc-windows-msvc）"
    fi
}

run_frontend() {
    step "TypeScript 类型检查"
    if npx tsc --noEmit; then
        ok "tsc --noEmit"
    else
        bad "tsc --noEmit"
    fi

    step "前端构建（vite build）"
    if npm run build; then
        ok "npm run build"
    else
        bad "npm run build"
    fi
}

case "${1:-all}" in
    rust)
        run_rust
        ;;
    frontend)
        run_frontend
        ;;
    all)
        run_rust
        run_frontend
        ;;
    *)
        echo "用法: $0 [all|rust|frontend]"
        exit 1
        ;;
esac

echo ""
echo "=========================================="
if [ "$fail" -eq 0 ]; then
    echo -e "${GREEN}全部通过：$pass 项${NC}"
    exit 0
else
    echo -e "${RED}失败 $fail 项 / 共 $((pass + fail)) 项：${NC}"
    for s in "${failed_steps[@]}"; do
        echo -e "  ${RED}- $s${NC}"
    done
    exit 1
fi
