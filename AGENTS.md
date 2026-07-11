# AGENTS.md

本文件面向在此仓库中工作的 AI 编码代理，帮助你在不了解背景的情况下快速理解 Peregrine 项目并安全地做出修改。内容基于实际代码核对，如与代码不一致以代码为准。

## 项目概述

Peregrine 是一个桌面视觉锚点（覆盖层）工具，**主要用途是防 3D 眩晕**：在屏幕中心或边缘绘制半透明的视觉锚点，为玩家在 3D 游戏中提供固定参照物，缓解眩晕感。

- 语言 / 生态：**Rust**，Cargo **workspace**（`resolver = "3"`，`edition = "2024"`，`rust-version = 1.85`，MIT 许可）。
- 图形栈：**Tauri**（设置窗口 Webview）+ **React + Tailwind + shadcn/ui**（设置面板），覆盖层仍用 `winit` + `softbuffer`；原 `wgpu` + `egui` 实现已移除。
- 异步运行时：`tokio`（配置读写、文件热重载、后台跟随任务）。
- 目标平台：**Windows**（x86 / x86_64 / ARM64）。CI 同时在 macOS / Linux 做编译验证，但 overlay 的透明 / 穿透 / 跟随能力仅 Windows 实现。
- 当前状态：**v0.1.0 正式版已发布**。Windows 透明置顶穿透覆盖层、目标窗口跟随、12 种准心样式、自定义 PNG 贴图、配置热重载等功能已可用。"进程触发" 仍为配置占位。

代码注释与文档一律使用**简体中文**。请沿用中文撰写新注释、文档与提交信息主体，保持一致。

## 仓库结构

```
peregrine/
├── Cargo.toml            # workspace 根：成员、workspace.package、workspace.dependencies、编译 profile
├── Cargo.lock
├── .gitignore            # 忽略 /target、*.log、.DS_Store、docs 构建产物等
├── assets/               # 应用图标（icon.ico）与图标生成脚本（gen_icon.py）
├── docs/                 # VitePress 文档站点（部署到 GitHub Pages）
│   ├── .vitepress/       # VitePress 配置（config.mts）、主题、构建产物 dist/
│   ├── guide/            # 使用说明、介绍、快速开始、功能、配置、开发构建
│   ├── public/           # 静态资源（logo.svg 等）
│   ├── index.md          # 文档首页
│   └── package.json      # vitepress + mermaid + llms 插件
├── .github/workflows/    # ci.yml（三平台编译 + lint）、release.yml（打 tag 发布）、pages.yml（文档部署）
├── .agent/skills/        # AI agent 技能定义（release 发布流程规范）
├── src-tauri/            # peregrine-tauri：Tauri 后端入口、tray、commands、overlay 管理
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── lib.rs             # Tauri 启动入口：配置初始化、tray、commands、watcher
│       ├── main.rs            # 二进制入口，调用 lib::run
│       └── overlay.rs         # 独立线程运行 winit 事件循环管理 overlay
├── package.json          # 前端 npm 依赖（React / Vite / Tailwind / shadcn / Tauri JS API）
├── vite.config.ts
├── tailwind.config.ts
├── components.json
├── tsconfig.json
├── index.html
└── src/                  # 前端源码
    ├── main.tsx
    ├── App.tsx              # 设置面板主组件
    ├── index.css
    ├── lib/
    │   ├── api.ts           # Tauri invoke 封装
    │   └── shapes.ts        # 前端预览几何计算
    ├── types/config.ts      # TypeScript 配置类型
    ├── components/
    │   └── Preview.tsx      # Canvas 实时预览
    │   └── ui/              # shadcn/ui 基础组件
    └── ...
└── crates/
    ├── config/           # peregrine_config：纯逻辑库（无 UI / GPU / 窗口代码）
    │   └── src/
    │       ├── lib.rs        # 模块导出 + 统一错误类型 ConfigError / Result
    │       ├── schema.rs     # 配置数据结构 AppConfig / Profile / Crosshair 等 + 校验 + 单测
    │       ├── storage.rs    # 配置文件路径管理、原子读写、默认配置生成（含内联 dirs 模块）
    │       ├── notifier.rs   # 基于 tokio::sync::watch 的配置变更广播
    │       └── watcher.rs    # 基于 notify crate 的配置文件热重载（含去抖）
    └── peregrine/        # peregrine：共享库（供 Tauri 复用）
        ├── Cargo.toml        # 仅提供 lib
        └── src/
            ├── lib.rs             # 导出 overlay_renderer / shapes / platform
            ├── overlay_renderer.rs # softbuffer（CPU 像素光栅化）**覆盖层**渲染器，透明置顶穿透窗口
            ├── shapes.rs           # 准心几何共享模块，预览与覆盖层共用同一套公式（所见即所得）
            └── platform/
                ├── mod.rs          # 平台模块入口，非 Windows 编译为占位
                └── windows.rs      # Win32 API：透明/置顶/穿透、目标窗口查找与跟随
```

**分层原则**：`peregrine_config` 不得依赖任何 UI / GPU / 窗口平台代码（`winit` / `wgpu` / `egui`）；平台与渲染逻辑放在 `peregrine` 共享库与 `src-tauri` 二进制 crate。修改时请保持这一边界。

## 技术栈与关键依赖

依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 中声明，各 crate 用 `{ workspace = true }` 引用。新增依赖优先加到 workspace 层。

- `crates/config`（`peregrine_config`）：`tokio`（features: sync/rt/rt-multi-thread/macros/time/fs）、`serde`（derive）、`serde_json`、`notify` 7.0、`thiserror` 2.0、`tracing`；dev 依赖 `tempfile`。
- `crates/peregrine`（共享库）：`peregrine_config`（path 依赖）、`winit` 0.30、`softbuffer` 0.4（覆盖层 CPU 光栅化）、`png` 0.17（自定义 PNG 准心解码）、`tokio`、`tracing`、`thiserror`（平台层 `OverlayError`）。
- `src-tauri`（`peregrine-tauri`，主入口）：`peregrine` / `peregrine_config`（path 依赖）、`tauri` 2.x（`tray-icon` feature）、`tauri-plugin-dialog`、`tauri-build`、前端产物（`dist/`）。
- 前端：`React` 18 + `Vite` 5 + `TypeScript` 5 + `Tailwind CSS` 3 + `shadcn/ui` + `@tauri-apps/api` / `@tauri-apps/cli` 2.x。
- `[target.'cfg(windows)'.dependencies]`：`windows` 0.58（Win32 UI / Foundation / Gdi features）。
- `[profile.dev]` 设置 `opt-level = 1`（加速图形栈的调试运行）。
- `[profile.release]` 启用 `opt-level = "z"` + `lto` + `codegen-units = 1` + `strip` + `panic = "abort"` + `overflow-checks = false`，优先级为减小体积与提升性能。

注意：`storage.rs` 内有一个**内联的 `dirs` 模块**自行实现跨平台配置目录（Windows `%APPDATA%`、macOS `~/Library/Application Support`、Linux `$XDG_CONFIG_HOME` 或 `~/.config`），并非外部 `dirs` crate，不要误加依赖。

## 构建、运行与测试

在仓库根目录执行（不要在 `target/` 内执行）：

```bash
# 安装前端依赖（首次）
npm install

# 构建整个 workspace
cargo build
cargo build --release

# 运行 Tauri 版本
npx tauri dev

# 构建 Tauri release 产物
npx tauri build

# 运行全部测试
cargo test

# 只测配置库
cargo test -p peregrine_config

# 代码检查 / 格式化
cargo fmt
cargo clippy
```

- 目前**所有单元测试都在 `crates/config`** 中（`schema.rs` / `storage.rs` / `notifier.rs` / `watcher.rs`），`peregrine` 共享库与 `src-tauri` 暂无测试。
- 涉及 tokio 的测试使用 `#[tokio::test]`；`watcher.rs` 的测试需要多线程运行时，标注为 `#[tokio::test(flavor = "multi_thread")]`。
- `watcher` 测试依赖真实文件系统事件并有最长 5s 的超时等待，属于偏集成性质，偶发受环境影响。

## 运行期架构（Tauri 版本）

1. `src-tauri/src/lib.rs::run()`：初始化 `tracing_subscriber`（控制台 stderr + `%APPDATA%/Peregrine/peregrine.log` 滚动文件，默认 `info` 级别）；通过 `ConfigStorage::with_default_path()` 定位配置文件并 `load_or_create_default()`；用配置构造 `ConfigNotifier`；在独立线程启动 `overlay::run_overlay_loop` 管理覆盖层窗口；启动 `ConfigWatcher` 并把 notifier 变更同步到共享快照与 overlay 线程；创建 Tauri app，配置 tray 图标与 commands，运行事件循环。
2. **设置窗口**（Tauri Webview）：普通带边框窗口（标题 "Peregrine 设置"，逻辑尺寸 960×560），承载 React + Tailwind + shadcn/ui 设置面板。关闭时**收起到状态栏**（`api.prevent_close()` + `window.hide()`）。
3. **系统托盘**（Tauri tray）：启动即建立，菜单含「设置」「退出」。点击「设置」恢复窗口；点击「退出」结束应用，Tauri `RunEvent::Exit` 中通知 overlay 线程停止。
4. **覆盖层窗口**（`src-tauri/src/overlay.rs`）：在独立线程中运行原生 `winit` 事件循环，创建透明、置顶、鼠标穿透窗口，由 `OverlayRenderer`（softbuffer CPU 光栅化）渲染准心。通过 Tauri command `start_overlay` / `stop_overlay` 创建/销毁。**创建前必须已选择目标窗口**。Windows 下调用 `platform::windows::setup_overlay_window` 补充 `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`。
5. **目标窗口跟随**（`platform::windows::follow_target_window`）：Windows 下以 16ms 周期轮询，对齐覆盖层到目标窗口；目标窗口最小化/非前台时隐藏 overlay；目标窗口被销毁时结束跟随。
6. 配置流动：前端改动 → Tauri command `save_config` → `ConfigStorage::save`（原子写）+ `ConfigNotifier::update` → `ConfigWatcher` 检测到文件变化后再次广播 → 共享快照更新 → overlay 渲染器读取。前端通过 `get_config` 获取初始配置。

### 配置模型与存储

- 配置根为 `AppConfig`（`active_profile` + 多个命名 `Profile`），每个 `Profile` 含 `crosshair`（`Crosshair`）、`trigger`（`TriggerRule`）、`settings_hotkey`、`target_window`。
- `Crosshair.style`（`CrosshairStyle`，12 种）：贴边矩形 `EdgeRect`、准星 `Cross`、大准星 `LargeCross`、定位球 `CornerDots4/6/8`、中心环 `Ring`、自定义定位球 `CustomOrb`、随机球 `RandomOrb`、边框 `BorderFrame`、自定义图片 `CustomImage`、箭头 `EdgeArrows`。各样式可调尺寸、厚度、颜色、透明度、间隙、贴边位置、边距等参数；`CustomImage` 额外有路径、缩放、偏移。
- 配置文件为 JSON，路径由 OS 标准目录决定：
  - Windows：`%APPDATA%/Peregrine/config.json`
  - macOS：`~/Library/Application Support/Peregrine/config.json`
  - Linux：`~/.config/Peregrine/config.json`
- 写入为原子操作（同目录临时文件 + `rename`），写入前必先 `AppConfig::validate()` 校验，避免落盘非法配置。
- `load_or_create_default`：文件不存在则生成默认配置；**解析或校验失败时不报错**，而是把损坏文件备份为 `<name>.bak` 后回退到默认配置并重新写入，保证程序始终能启动。

## 代码风格与约定

- 遵循标准 Rust 风格（`cargo fmt` 默认配置）。仓库未定制 rustfmt/clippy 规则；CI 中 `cargo clippy -p peregrine_config -- -D warnings` 把 config crate 的 lint 警告视为错误。
- **所有公开项都写中文文档注释**（`///`），模块顶部用 `//!` 说明职责。新增代码请保持同样密度的中文注释。
- 错误处理：库层统一用 `thiserror` 定义的 `ConfigError` 与 `crate::Result<T>`；不要在库中 `panic`/`unwrap`（校验失败返回 `ConfigError::Validation`）。二进制层可用 `expect`/`unwrap` 处理初始化期的致命错误；平台层（`platform/windows.rs`）定义自己的 `OverlayError`。
- 日志：使用 `tracing`（`info!`/`warn!`/`error!`/`debug!`），不要新增 `println!`/`eprintln!`。
- 序列化兼容：向 `Crosshair` 等结构新增字段时，务必加 `#[serde(default)]` 或 `#[serde(default = "...")]`，以保证旧配置文件仍可反序列化（现有字段大量使用此模式，`old_config_without_image_fields_loads` 测试专门验证这一点）。
- 枚举序列化统一 `#[serde(rename_all = "snake_case")]`。
- 新增可配置项时，通常需要同步改动五处：`schema.rs`（字段 + 默认值 + 校验）、`shapes.rs`（几何形状定义，`build_shapes`）、`src/App.tsx`（React 设置面板控件）、`src/lib/shapes.ts`（前端预览几何计算）、`overlay_renderer.rs`（如引入新图元类型，补光栅化）。预览（React Canvas）与覆盖层（softbuffer）都基于同一套几何逻辑，保证所见即所得。
- 并发：跨 tokio 与 winit 线程共享的配置快照使用**标准库 `std::sync::Mutex`**（注释明确：避免在 runtime 线程内调用 tokio `blocking_lock` 导致 panic）。沿用此约定，不要随意替换为 `tokio::sync::Mutex`。
- 配置快照类型为 `ConfigSnapshot = Arc<AppConfig>`，通过 `Arc` 共享避免深拷贝。

## 测试约定

- 单元测试与被测代码同文件，置于 `#[cfg(test)] mod tests`。
- `schema.rs` 侧重校验逻辑（尺寸/透明度/范围/枚举）、默认值、以及 serde round-trip（含旧配置兼容性）；`storage.rs` 用 `tempfile::tempdir()` 做真实文件读写（含损坏配置恢复）；`notifier.rs` 验证广播订阅与订阅者计数；`watcher.rs` 验证外部文件改动能被检测并广播。
- 修改 `schema` 的校验规则或默认值时，请同步更新/新增对应测试；改动配置结构后至少跑 `cargo test -p peregrine_config`。

## CI / CD 与发布

三个 workflow（`.github/workflows/`）：

- **`ci.yml`**：push 到 main/master 或提交 PR 时触发。`build` job 在 **Windows（x86_64-msvc）/ macOS（aarch64 + x86_64 交叉编译）/ Linux（x86_64-gnu）** 三平台矩阵运行 `cargo test -p peregrine_config --locked` + `npm ci && npx tauri build --target <target>`（Tauri 入口）；Linux job 需先安装 GUI 依赖（xcb/x11/wayland/gtk 等）。`lint` job 仅在 Linux 跑 `cargo fmt --all -- --check` 与 `cargo clippy -p peregrine_config -- -D warnings`。
- **`release.yml`**：推送 `v*` 标签时触发。在 Windows 构建 i686 / x86_64 / aarch64 三个 target 的 Tauri release 产物，包含 **NSIS 安装包（带 Tauri updater 签名）+ 便携 zip + `latest.json` 更新清单**，用 `softprops/action-gh-release` 创建 GitHub Release。CI 从 GitHub Secrets 读取 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 对安装包签名。标签带 `-`（如 `v0.2.0-alpha.0`）判定为预发布，纯版本号（如 `v0.1.0`）为正式版。Release body 与 updater `notes` 由 CI 根据当前 tag 与上一个 tag 之间的 commit 自动生成，按 conventional commit 前缀分为「新增 / 修复 / 变更 / 构建 / 文档 / 其他」；无可用提交时回退到 tag 消息或最近 commit message。
- **`pages.yml`**：push 到 main、发布 Release 或手动触发时部署文档。用 Node 22 在 `docs/` 下 `npm ci` + `npm run docs:build` 构建 VitePress 站点，上传产物并部署到 GitHub Pages。

发布流程规范见 `.agent/skills/release/SKILL.md`：遵循 SemVer（major/minor/patch + `-alpha.N`/`-beta.N` 预发布后缀），Release Notes 按「新增 / 修复 / 变更 / 构建」分类，打 tag 推送前需向用户确认版本号与 tag 消息。`CHANGELOG.md` 记录正式版，`CHANGELOG_ALPHA.md` 记录测试版。

### 分支策略

- **main**：稳定分支，仅包含正式版代码。正式版发布后合并。
- **dev**：开发分支，包含正在测试的功能（如自动更新等）。测试通过后合并到 main 发布正式版。

### 自动更新

项目集成了 `tauri-plugin-updater`（Rust 插件 + 前端 `@tauri-apps/plugin-updater`）：
- NSIS 安装包用户可通过「设置 → 检查更新」自动下载安装新版本，便携 zip 不支持。
- 签名密钥对存在本地 `.tauri/`（已被 `.gitignore` 排除），公钥写入 `tauri.conf.json` 的 `plugins.updater.pubkey`。
- CI 从 GitHub Secrets 读取私钥签名；`latest.json` 清单由 CI 自动生成并上传。
- 私钥与密码丢失后将无法发布可自动更新的版本，需妥善备份。

## 文档站点

`docs/` 是基于 **VitePress** 的文档站点（`lang: zh-CN`，`base: /`，部署到自定义域名 `https://peregrine.eeymoo.com/`），含 mermaid 图表插件与 `vitepress-plugin-llms`（生成 `llms.txt` / `llms-full.txt`）。本地预览：`cd docs && npm ci && npm run docs:dev`。内容在 `docs/guide/`（使用说明、项目介绍、快速开始、功能特性、配置说明、开发构建）。

## 安全与注意事项

- **不要提交敏感信息**；配置文件位于用户目录，不随仓库分发。
- 配置写入已做原子化与合法性校验；改动 `storage.rs` 时保持"先校验后写入、临时文件 + rename"的不变量，避免损坏用户配置。
- `target/` 是 Cargo 构建产物目录，已被 `.gitignore` 忽略。所有源码改动、构建与测试都应针对仓库根进行。

### 已知局限（改动相关代码时留意）

- 覆盖层的透明、置顶、鼠标穿透等 overlay 特性通过 `platform/windows.rs` 的 Win32 API 实现：透明/穿透/置顶由 winit 窗口属性（`with_transparent` + `set_cursor_hittest(false)` + `WindowLevel::AlwaysOnTop`）设置，`setup_overlay_window` 仅补充 `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`。`overlay_renderer.rs` 走 softbuffers CPU 光栅化而非 wgpu swapchain，以避开 DirectComposition 的透明坑。
- **覆盖层必须选择目标窗口才能创建**：`target_window` 为空时 `create_overlay` 直接返回，不会创建全屏 overlay。
- `TriggerRule`（进程触发）在 schema 中已定义（`enabled` + `process_names`），但**二进制层未消费**——当前没有任何按前台进程名自动启用/隐藏覆盖层的逻辑，属于纯配置占位。
- `RandomOrb` 的 `LockOnStart` / `Reshuffle` 两种模式在当前渲染实现中行为相同——均每帧从配置参数派生种子重新生成（预览与 overlay 用相同的 `SimpleRng` 实现 + 种子，保证一致）。schema 中的 `random_orb_x/y` 持久化字段已定义但渲染层尚未消费，待后续接入。
- CI 的 config 测试与 release 编译在三平台运行；`windows` crate 通过 `[target.'cfg(windows)'.dependencies]` 声明，非 Windows target 不拉取。
