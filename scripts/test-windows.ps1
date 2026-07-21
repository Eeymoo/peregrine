# Peregrine Windows 完整验证脚本
#
# 覆盖 OpenSpec 变更的 Windows 验证项：
#   - 全 workspace 测试 / lint / 格式检查
#   - 前端构建
#   - src-tauri release 构建（multi-profile-config 任务 27/28）
#   - 完整 Tauri 构建（可选，生成 NSIS 安装包）
#
# 用法（PowerShell）：
#   .\scripts\test-windows.ps1              # 自动化检查 + release 构建
#   .\scripts\test-windows.ps1 -Full        # 额外执行 npx tauri build（完整打包）
#   .\scripts\test-windows.ps1 -SkipBuild   # 仅自动化检查，不构建

param(
    [switch]$Full,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")

$pass = 0
$fail = 0
$failedSteps = @()

function Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Yellow }
function Ok($msg)   { Write-Host "✔ $msg" -ForegroundColor Green; $script:pass++ }
function Bad($msg)  { Write-Host "✘ $msg" -ForegroundColor Red; $script:fail++; $script:failedSteps += $msg }

Step "Rust 全 workspace 测试"
cargo test --workspace
if ($LASTEXITCODE -eq 0) { Ok "cargo test --workspace" } else { Bad "cargo test --workspace" }

Step "Clippy lint（-D warnings）"
cargo clippy --workspace -- -D warnings
if ($LASTEXITCODE -eq 0) { Ok "cargo clippy" } else { Bad "cargo clippy" }

Step "代码格式检查"
cargo fmt --all -- --check
if ($LASTEXITCODE -eq 0) { Ok "cargo fmt" } else { Bad "cargo fmt（运行 cargo fmt --all 修复）" }

Step "前端依赖安装（npm ci）"
npm ci
if ($LASTEXITCODE -eq 0) { Ok "npm ci" } else { Bad "npm ci" }

Step "TypeScript 类型检查"
npx tsc --noEmit
if ($LASTEXITCODE -eq 0) { Ok "tsc --noEmit" } else { Bad "tsc --noEmit" }

Step "前端构建（vite build）"
npm run build
if ($LASTEXITCODE -eq 0) { Ok "npm run build" } else { Bad "npm run build" }

if (-not $SkipBuild) {
    Step "src-tauri release 构建（对应 multi-profile-config 任务 27/28）"
    cargo build --manifest-path src-tauri/Cargo.toml --bins --release
    if ($LASTEXITCODE -eq 0) { Ok "src-tauri release 构建" } else { Bad "src-tauri release 构建" }
}

if ($Full) {
    Step "完整 Tauri 构建（npx tauri build，生成安装包）"
    npx tauri build
    if ($LASTEXITCODE -eq 0) { Ok "tauri build" } else { Bad "tauri build" }
}

Write-Host "`n=========================================="
if ($fail -eq 0) {
    Write-Host "全部通过：$pass 项" -ForegroundColor Green
    Write-Host "`n自动化验证已完成。请继续手动 UI 验证清单：docs/manual-test-checklist.md" -ForegroundColor Cyan
    exit 0
} else {
    Write-Host "失败 $fail 项 / 共 $($pass + $fail) 项：" -ForegroundColor Red
    foreach ($s in $failedSteps) { Write-Host "  - $s" -ForegroundColor Red }
    exit 1
}
