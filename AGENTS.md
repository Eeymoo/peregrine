# AGENTS.md

本文件面向在此仓库中工作的 AI 编码代理，帮助你在不了解背景的情况下快速理解 Peregrine 项目并安全地做出修改。

## 项目概述

Peregrine 是一个桌面辅助贴图（准心 / 覆盖层）工具，**主要用途是防 3D 眩晕**：在屏幕中心或边缘绘制半透明的视觉锚点，为玩家在 3D 游戏中提供固定参照物。

- 语言 / 生态：**Rust**，Cargo **workspace**（`resolver = "3"`，`edition = "2024"`，`rust-version = 1.85`，PolyForm Noncommercial 许可）。
- 图形栈：`winit`（窗口 / 事件循环）+ `wgpu`（GPU 渲染）+ `egui` / `egui-wgpu` / `egui-winit`（即时模式 UI）。
- 异步运行时：`tokio`（配置读写、文件热重载、后台任务）。
- 目标平台：**Windows**（x86 / x86_64 / ARM64）。
- 当前状态：**v0.1.0 正式版已发布**。Windows 透明置顶穿透覆盖层、目标窗口跟随、多种准心样式、自定义 PNG 贴图等功能已可用。"进程触发" 仍为占位。

代码注释与文档一律使用**简体中文**。请沿用中文撰写新注释、文档与提交信息主体，保持一致。

## 仓库结构

```
peregrine/
├── Cargo.toml            # workspace 根：成员、workspace.package、workspace.dependencies
├── Cargo.lock
├── .gitignore            # 忽略 /target、*.log、.DS_Store
└── crates/
    ├── config/           # peregrine_config：纯逻辑库（无 UI / GPU / 窗口代码）
    │   └── src/
    │       ├── lib.rs        # 模块导出 + 统一错误类型 ConfigError / Result
    │       ├── schema.rs     # 配置数据结构 AppConfig / Profile / Crosshair 等 + 校验 + 单测
    │       ├── storage.rs    # 配置文件路径管理、原子读写、默认配置生成（含内联 dirs 模块）
    │       ├── notifier.rs   # 基于 tokio::sync::watch 的配置变更广播
    │       └── watcher.rs    # 基于 notify crate 的配置文件热重载（含去抖）
    └── peregrine/        # peregrine：二进制程序（GUI 主程序）
        └── src/
            ├── main.rs        # 入口、winit ApplicationHandler、模式切换、tokio 主循环
            ├── renderer.rs    # wgpu + egui 渲染器，Overlay 覆盖层绘制、字体加载
            └── settings_ui.rs # egui 设置面板与实时预览绘制
```

**分层原则**：`peregrine_config` 不得依赖任何 UI / GPU / 窗口平台代码（`winit` / `wgpu` / `egui`）；平台与渲染逻辑只放在 `peregrine` 二进制 crate。修改时请保持这一边界。

## 技术栈与关键依赖

依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 中声明，各 crate 用 `{ workspace = true }` 引用。新增依赖优先加到 workspace 层。

- `crates/config`：`tokio`（features: sync/rt/rt-multi-thread/macros/time/fs）、`serde`（derive）、`serde_json`、`notify` 7.0、`thiserror` 2.0、`tracing`；dev 依赖 `tempfile`。
- `crates/peregrine`：`peregrine_config`（path 依赖）、`winit` 0.30、`wgpu` 24.0、`egui`/`egui-wgpu`/`egui-winit` 0.31、`pollster` 0.4、`tokio`、`tracing`、`tracing-subscriber`。
- `[profile.dev]` 设置 `opt-level = 1`（加速图形栈的调试运行）。

注意：`storage.rs` 内有一个**内联的 `dirs` 模块**自行实现跨平台配置目录，并非外部 `dirs` crate，不要误加依赖。

## 构建、运行与测试

在仓库根目录执行（不要在 `target/` 内执行）：

```bash
# 构建整个 workspace
cargo build
cargo build --release

# 运行 GUI 主程序（workspace 有多个成员，需用 -p 指定二进制包）
cargo run -p peregrine

# 运行全部测试
cargo test

# 只测配置库（GUI crate 目前没有测试）
cargo test -p peregrine_config

# 代码检查 / 格式化（仓库未提供 rustfmt.toml / clippy.toml，使用默认配置）
cargo fmt
cargo clippy
```

- 目前**所有单元测试都在 `crates/config`** 中（`schema.rs` / `storage.rs` / `notifier.rs` / `watcher.rs`），`peregrine` 二进制 crate 暂无测试。
- 涉及 tokio 的测试使用 `#[tokio::test]`；`watcher.rs` 的测试需要多线程运行时，标注为 `#[tokio::test(flavor = "multi_thread")]`。
- `watcher` 测试依赖真实文件系统事件并有最长 5s 的超时等待，属于偏集成性质，偶发受环境影响。
- 仓库已配置 GitHub Actions CI（`.github/workflows/ci.yml`）：每次 push / PR 运行 config 测试并 release 编译二进制 crate；另有 lint job 检查 `cargo fmt` 与 `cargo clippy`。发布流程（`.github/workflows/release.yml`）在推送 `v*` 标签时触发，构建 Windows x86 / x86_64 / ARM64 产物并创建 GitHub Release。本地工具链已验证可用（rustc/cargo 1.96，edition 2024 需较新工具链）。

## 运行期架构

1. `main()`：初始化 `tracing_subscriber`；通过 `ConfigStorage::with_default_path()` 定位配置文件并 `load_or_create_default()`；用配置构造 `ConfigNotifier`；创建 winit `EventLoop`（`ControlFlow::Wait`）运行 `App`。
2. `App` 实现 `winit::ApplicationHandler`：
   - `resumed`：首次创建窗口（标题 "Peregrine"，逻辑尺寸 960×560）与 `Renderer`（`pollster::block_on` 同步初始化 wgpu）；启动 `ConfigWatcher` 热重载任务，并订阅广播把最新快照写回共享的 `Arc<Mutex<ConfigSnapshot>>`。
   - `window_event`：先把事件交给 egui（`renderer.handle_event`）；按 **Tab** 切换 `Overlay` / `Settings` 模式；`Esc` 在 Settings 下切回 Overlay，在 Overlay 下退出程序；`RedrawRequested` 时按模式渲染。
   - 启动默认进入 **Settings** 模式。
3. 渲染（`renderer.rs`）：
   - `render_overlay()`：清屏为透明，用 egui 生成覆盖层几何并走 wgpu 管线绘制当前 Profile 的辅助贴图。
   - `render_settings()`：绘制 egui 设置面板；若用户改动了配置（`SettingsResponse.changed`），更新共享快照并 `tokio::spawn` 异步 `storage.save` + `notifier.update` 持久化与广播。
4. 配置流动：`UI 改动 / 外部编辑文件` → `ConfigStorage`（原子写：临时文件 + rename）/ `ConfigWatcher`（notify + 300ms 去抖）→ `ConfigNotifier`（`watch` 广播）→ 各订阅者（渲染器读取的共享快照）。

### 配置模型与存储

- 配置根为 `AppConfig`（`active_profile` + 多个命名 `Profile`），每个 `Profile` 含 `crosshair`（`Crosshair`）、`trigger`（`TriggerRule`）、`settings_hotkey`、`target_window`。
- `Crosshair.style`（`CrosshairStyle`）支持：卫生纸 `ToiletPaper`、准星 `Cross`、大准星 `LargeCross`、定位球 `CornerDots4/6/8`、中心环 `Ring`、自定义定位球 `CustomOrb`、随机球 `RandomOrb`、边框 `BorderFrame`。
- 配置文件为 JSON，路径由 OS 标准目录决定：
  - Windows：`%APPDATA%/Peregrine/config.json`
- 写入为原子操作（同目录临时文件 + `rename`），写入前必先 `AppConfig::validate()` 校验，避免落盘非法配置。

## 代码风格与约定

- 遵循标准 Rust 风格（`cargo fmt` 默认配置）。仓库未定制 rustfmt/clippy 规则。
- **所有公开项都写中文文档注释**（`///`），模块顶部用 `//!` 说明职责。新增代码请保持同样密度的中文注释。
- 错误处理：库层统一用 `thiserror` 定义的 `ConfigError` 与 `crate::Result<T>`；不要在库中 `panic`/`unwrap`（校验失败返回 `ConfigError::Validation`）。二进制层可用 `expect`/`unwrap` 处理初始化期的致命错误。
- 日志：使用 `tracing`（`info!`/`warn!`/`error!`/`debug!`），不要新增 `println!`/`eprintln!`（现有代码里的 `eprintln!("[key]...")`、`[redraw]`、`[render_settings]` 等是**遗留的调试打印**，若你在附近改动，倾向于清理或替换为 `tracing`）。
- 序列化兼容：向 `Crosshair` 等结构新增字段时，务必加 `#[serde(default)]` 或 `#[serde(default = "...")]`，以保证旧配置文件仍可反序列化（现有字段大量使用此模式）。
- 枚举序列化统一 `#[serde(rename_all = "snake_case")]`。
- 新增可配置项时，通常需要同步改动三处：`schema.rs`（字段 + 默认值 + 校验）、`settings_ui.rs`（编辑控件 + 预览绘制 `draw_preview_shape`）、`renderer.rs`（覆盖层绘制 `draw_overlay_shape`）。预览与覆盖层应保持相同的几何逻辑。
- 并发：跨 tokio 与 winit 线程共享的配置快照使用**标准库 `std::sync::Mutex`**（注释明确：避免在 runtime 线程内调用 tokio `blocking_lock` 导致 panic）。沿用此约定，不要随意替换为 `tokio::sync::Mutex`。

## 测试约定

- 单元测试与被测代码同文件，置于 `#[cfg(test)] mod tests`。
- `schema.rs` 侧重校验逻辑（尺寸/透明度/范围）、默认值、以及 serde round-trip；`storage.rs` 用 `tempfile::tempdir()` 做真实文件读写；`notifier.rs` 验证广播订阅；`watcher.rs` 验证外部文件改动能被检测并广播。
- 修改 `schema` 的校验规则或默认值时，请同步更新/新增对应测试；改动配置结构后至少跑 `cargo test -p peregrine_config`。

## 安全与注意事项

- **不要提交敏感信息**；配置文件位于用户目录，不随仓库分发。
- 配置写入已做原子化与合法性校验；改动 `storage.rs` 时保持"先校验后写入、临时文件 + rename"的不变量，避免损坏用户配置。
- `renderer.rs::load_system_font` 会读取若干系统字体路径以显示中文；找不到时仅告警并回退默认字体。修改字体候选列表时保留失败回退逻辑。

### 已知局限（改动相关代码时留意）

- 覆盖层的透明、置顶、鼠标穿透等 overlay 特性通过 `platform/windows.rs` 的 Win32 API `WS_EX_LAYERED` + per-pixel alpha 透明实现。`TriggerRule`（进程触发）仍为**占位**，未接入平台 API。
- `RandomOrb` 的 `LockOnStart` / `Reshuffle` 两种模式在当前渲染实现中行为相同——均每帧从配置参数派生种子重新生成。schema 中的 `random_orb_x/y` 持久化字段已定义但渲染层尚未消费，待后续接入。
- `settings_ui.rs` 与 `overlay_renderer.rs` 中的预览绘制（`draw_preview_shape`）与覆盖层绘制（`draw_overlay_shape`）存在大量几何逻辑重复（包括 `SimpleRng`、虚线圆、边框绘制等辅助函数两套副本），后续应提取为公共模块。
- CI（`.github/workflows/ci.yml`）在 Windows 平台运行 config 测试与 release 编译；发布（`release.yml`）覆盖 Windows x86 / x86_64 / ARM64。`windows` crate 与 `embed-resource` 通过 `[target.'cfg(windows)'.dependencies]` 声明，非 Windows target 不拉取。

## 关于工作目录的提醒

本仓库的 `target/`（含 `target/release`）是 Cargo 构建产物目录，已被 `.gitignore` 忽略。**所有源码改动、构建与测试都应针对仓库根 `/Users/luanmeihua/Codes/peregrine` 进行**，不要在 `target/` 下创建或修改源文件。
