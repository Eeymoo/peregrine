# AGENTS.md

本文件面向在此仓库中工作的 AI 编码代理，帮助你在不了解背景的情况下快速理解 Peregrine 项目并安全地做出修改。内容基于实际代码核对，如与代码不一致以代码为准。

## 项目概述

Peregrine 是一个桌面辅助贴图（准心 / 覆盖层）工具，**主要用途是防 3D 眩晕**：在屏幕中心或边缘绘制半透明的视觉锚点，为玩家在 3D 游戏中提供固定参照物，缓解眩晕感。

- 语言 / 生态：**Rust**，Cargo **workspace**（`resolver = "3"`，`edition = "2024"`，`rust-version = 1.85`，PolyForm Noncommercial 许可）。
- 图形栈：`winit`（窗口 / 事件循环）+ `wgpu`（设置窗口 GPU 渲染）+ `egui` / `egui-wgpu` / `egui-winit`（即时模式 UI）+ `softbuffer`（覆盖层 CPU 像素光栅化）。
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
└── crates/
    ├── config/           # peregrine_config：纯逻辑库（无 UI / GPU / 窗口代码）
    │   └── src/
    │       ├── lib.rs        # 模块导出 + 统一错误类型 ConfigError / Result
    │       ├── schema.rs     # 配置数据结构 AppConfig / Profile / Crosshair 等 + 校验 + 单测
    │       ├── storage.rs    # 配置文件路径管理、原子读写、默认配置生成（含内联 dirs 模块）
    │       ├── notifier.rs   # 基于 tokio::sync::watch 的配置变更广播
    │       └── watcher.rs    # 基于 notify crate 的配置文件热重载（含去抖）
    └── peregrine/        # peregrine：二进制程序（GUI 主程序）
        ├── build.rs          # Windows target 嵌入 exe 图标资源（embed-resource）
        ├── assets.rc         # 资源描述：1 ICON "../../assets/icon.ico"
        └── src/
            ├── main.rs             # 入口、winit ApplicationHandler、双窗口管理、托盘、tokio 主循环、日志初始化
            ├── renderer.rs         # wgpu + egui 渲染器，**设置窗口**渲染、字体加载（含旧 wgpu overlay 分支，现 dead_code）
            ├── overlay_renderer.rs # softbuffer（CPU 像素光栅化）**覆盖层**渲染器，透明置顶穿透窗口
            ├── shapes.rs           # 准心几何共享模块，预览与覆盖层共用同一套公式（所见即所得）
            ├── settings_ui.rs      # egui 设置面板与实时预览绘制
            ├── icon.rs             # 占位图标（运行时生成位图），同时用于托盘与窗口图标
            └── platform/
                ├── mod.rs          # 平台模块入口，非 Windows 编译为占位
                └── windows.rs      # Win32 API：透明/置顶/穿透、目标窗口查找与跟随
```

**分层原则**：`peregrine_config` 不得依赖任何 UI / GPU / 窗口平台代码（`winit` / `wgpu` / `egui`）；平台与渲染逻辑只放在 `peregrine` 二进制 crate。修改时请保持这一边界。

## 技术栈与关键依赖

依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 中声明，各 crate 用 `{ workspace = true }` 引用。新增依赖优先加到 workspace 层。

- `crates/config`（`peregrine_config`）：`tokio`（features: sync/rt/rt-multi-thread/macros/time/fs）、`serde`（derive）、`serde_json`、`notify` 7.0、`thiserror` 2.0、`tracing`；dev 依赖 `tempfile`。
- `crates/peregrine`（二进制）：`peregrine_config`（path 依赖）、`winit` 0.30、`wgpu` 24.0、`egui`/`egui-wgpu`/`egui-winit` 0.31、`pollster` 0.4、`softbuffer` 0.4（覆盖层 CPU 光栅化）、`tray-icon` 0.24（系统托盘）、`png` 0.17（自定义 PNG 准心解码）、`tokio`、`tracing`、`tracing-subscriber`（env-filter + fmt）、`tracing-appender`（滚动文件日志）、`thiserror`（平台层 `OverlayError`）。
- `[target.'cfg(windows)'.dependencies]`：`windows` 0.58（Win32 UI / Foundation / Gdi features）。
- `[target.'cfg(windows)'.build-dependencies]`：`embed-resource` 3.0（嵌入 exe 图标）。
- `[profile.dev]` 设置 `opt-level = 1`（加速图形栈的调试运行）。
- `[profile.release]` 启用 `opt-level = "z"` + `lto` + `codegen-units = 1` + `strip` + `panic = "abort"` + `overflow-checks = false`，优先级为减小体积与提升性能。

注意：`storage.rs` 内有一个**内联的 `dirs` 模块**自行实现跨平台配置目录（Windows `%APPDATA%`、macOS `~/Library/Application Support`、Linux `$XDG_CONFIG_HOME` 或 `~/.config`），并非外部 `dirs` crate，不要误加依赖。

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

## 运行期架构

1. `main()`：初始化 `tracing_subscriber`（控制台 stderr + `%APPDATA%/Peregrine/peregrine.log` 滚动文件，默认 `info` 级别，可用 `RUST_LOG` 覆盖）；通过 `ConfigStorage::with_default_path()` 定位配置文件并 `load_or_create_default()`；用配置构造 `ConfigNotifier`；创建 winit `EventLoop<UserEvent>`（`ControlFlow::Wait`），把托盘菜单事件与 overlay 跟随结束事件转发进事件循环；运行 `App`。`main` 用 `#[tokio::main]`，事件循环与 tokio runtime 在同一线程。
2. **双窗口 + 托盘**架构（`main.rs`）：
   - **设置窗口**（`settings_window`）：普通 winit 窗口（标题 "Peregrine"，逻辑尺寸 960×560），承载 egui 设置面板，由 `Renderer`（wgpu）渲染。启动时默认创建并显示；关闭（`CloseRequested`）/按 `Esc` 时**收起到状态栏**（`hide_to_tray`，隐藏而非退出）。
   - **覆盖层窗口**（`overlay_window`）：透明、置顶、鼠标穿透的窗口，由 `OverlayRenderer`（softbuffer CPU 光栅化）渲染准心。通过设置面板的「开始覆盖」按钮创建、「停止覆盖」或目标窗口关闭时销毁。**创建前必须已选择目标窗口**（`target_window_title` 为空时 `create_overlay` 直接 warn 返回，不创建）。创建时调用 `platform::windows::setup_overlay_window` 补充 `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`（透明/穿透/置顶已由 winit 窗口属性完成）。
   - **系统托盘**（`tray_icon`）：启动即建立（`new_events` 的 `Init` 时机），菜单含「设置」「退出」两项。点击「设置」恢复设置窗口（`show_settings`）；点击「退出」走 `shutdown` 销毁 overlay、停止 watcher、退出事件循环。
3. **目标窗口跟随**（`platform::windows::follow_target_window`）：以 16ms 周期轮询，把覆盖层窗口对齐到目标窗口的位置与尺寸（典型场景：跟随游戏窗口）。**关键行为**：目标窗口最小化、或不是前台窗口（`GetForegroundWindow != target`）时隐藏 overlay；目标窗口被销毁（`IsWindow` 为 false）时结束跟随并通过 `EventLoopProxy` 发送 `OverlayFollowerEnded`，主线程收到后销毁 overlay。HWND 通过 `SendHwnd`（`unsafe impl Send`）跨线程传入 tokio 任务。
4. 渲染：
   - 覆盖层（`overlay_renderer.rs::render_overlay`）：用 `softbuffer` 取像素缓冲，按当前 `Crosshair` 调用 `shapes.rs` 的几何公式得到形状列表后 CPU 光栅化（矩形/圆/圆环/线段/三角形，预乘 alpha），透明区域填 `0x00000000`。`CustomImage` 样式单独走 PNG 解码 + blit 路径（带路径缓存）。**穿透窗口收不到 `RedrawRequested`**，因此 overlay 在 `about_to_wait` 中每帧直接渲染。窗口大小变化时在 `render` 内按当前窗口尺寸自动重建缓冲（`resize` 为空实现）。
   - 设置窗口（`renderer.rs::render_settings`）：绘制 egui 设置面板，含实时预览（同样调用 `shapes.rs`，保证所见即所得）；若用户改动了配置（`SettingsResponse.changed`），更新共享快照并 `tokio::spawn` 异步 `storage.save` + `notifier.update` 持久化与广播。`Renderer::resize` 会重配 `surface_config` 并 `surface.configure`，另有 `reconfigure_surface_if_needed` 在 `get_current_texture` 失败时兜底。`renderer.rs` 中还保留一个旧的 wgpu overlay 渲染分支（`render_overlay`，`#[allow(dead_code)]`），当前不使用。
5. 配置流动：`UI 改动 / 外部编辑文件` → `ConfigStorage`（原子写：临时文件 + rename）/ `ConfigWatcher`（notify + 300ms 去抖）→ `ConfigNotifier`（`watch` 广播）→ 各订阅者（渲染器读取的共享 `Arc<Mutex<ConfigSnapshot>>`）。watcher 在 `resumed` 时 spawn，同时 spawn 一个把 notifier 变更回写到共享快照的循环。

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
- 新增可配置项时，通常需要同步改动四处：`schema.rs`（字段 + 默认值 + 校验）、`shapes.rs`（几何形状定义，`build_shapes`）、`settings_ui.rs`（编辑控件 + 预览绘制）、`overlay_renderer.rs`（如引入新图元类型，补光栅化）。预览（egui）与覆盖层（softbuffer）都从 `shapes.rs::build_shapes` 取同一组 `Shape`，保证所见即所得。`CustomImage` 与 `EdgeArrows` 是例外：前者由各渲染器单独 blit，后者只在 `shapes.rs` 生成（设置面板预览与 overlay 都走 `build_shapes`，`renderer.rs` 旧分支里对应分支为 dead_code）。
- 并发：跨 tokio 与 winit 线程共享的配置快照使用**标准库 `std::sync::Mutex`**（注释明确：避免在 runtime 线程内调用 tokio `blocking_lock` 导致 panic）。沿用此约定，不要随意替换为 `tokio::sync::Mutex`。
- 配置快照类型为 `ConfigSnapshot = Arc<AppConfig>`，通过 `Arc` 共享避免深拷贝。

## 测试约定

- 单元测试与被测代码同文件，置于 `#[cfg(test)] mod tests`。
- `schema.rs` 侧重校验逻辑（尺寸/透明度/范围/枚举）、默认值、以及 serde round-trip（含旧配置兼容性）；`storage.rs` 用 `tempfile::tempdir()` 做真实文件读写（含损坏配置恢复）；`notifier.rs` 验证广播订阅与订阅者计数；`watcher.rs` 验证外部文件改动能被检测并广播。
- 修改 `schema` 的校验规则或默认值时，请同步更新/新增对应测试；改动配置结构后至少跑 `cargo test -p peregrine_config`。

## CI / CD 与发布

三个 workflow（`.github/workflows/`）：

- **`ci.yml`**：push 到 main/master 或提交 PR 时触发。`build` job 在 **Windows（x86_64-msvc）/ macOS（aarch64 + x86_64 交叉编译）/ Linux（x86_64-gnu）** 三平台矩阵运行 `cargo test -p peregrine_config --locked` + `cargo build --release --locked -p peregrine --target <target>`；Linux job 需先安装 GUI 依赖（xcb/x11/wayland/gtk 等）。`lint` job 仅在 Linux 跑 `cargo fmt --all -- --check` 与 `cargo clippy -p peregrine_config -- -D warnings`。
- **`release.yml`**：推送 `v*` 标签时触发。在 Windows 构建 i686 / x86_64 / aarch64 三个 target 的 release 产物，打包为 zip（含 README、LICENSE），用 `softprops/action-gh-release` 创建 GitHub Release。标签带 `-`（如 `v0.2.0-alpha.0`）判定为预发布，纯版本号（如 `v0.1.0`）为正式版。Release body 取 annotated tag 消息，回退到 commit message。
- **`pages.yml`**：push 到 main、发布 Release 或手动触发时部署文档。用 Node 22 在 `docs/` 下 `npm ci` + `npm run docs:build` 构建 VitePress 站点，上传产物并部署到 GitHub Pages。

发布流程规范见 `.agent/skills/release/SKILL.md`：遵循 SemVer（major/minor/patch + `-alpha.N`/`-beta.N` 预发布后缀），Release Notes 按「新增 / 修复 / 变更 / 构建」分类，打 tag 推送前需向用户确认版本号与 tag 消息。`CHANGELOG.md` 记录正式版，`CHANGELOG_ALPHA.md` 记录测试版。

## 文档站点

`docs/` 是基于 **VitePress** 的文档站点（`lang: zh-CN`，`base: /peregrine/`），含 mermaid 图表插件与 `vitepress-plugin-llms`（生成 `llms.txt` / `llms-full.txt`）。本地预览：`cd docs && npm ci && npm run docs:dev`。内容在 `docs/guide/`（使用说明、项目介绍、快速开始、功能特性、配置说明、开发构建）。

## 安全与注意事项

- **不要提交敏感信息**；配置文件位于用户目录，不随仓库分发。
- 配置写入已做原子化与合法性校验；改动 `storage.rs` 时保持"先校验后写入、临时文件 + rename"的不变量，避免损坏用户配置。
- `renderer.rs::load_system_font` 会读取若干系统字体路径（Windows 微软雅黑 / macOS PingFang / Linux NotoCJK）以显示中文；找不到时仅告警并回退默认字体。修改字体候选列表时保留失败回退逻辑。
- `target/` 是 Cargo 构建产物目录，已被 `.gitignore` 忽略。所有源码改动、构建与测试都应针对仓库根进行。

### 已知局限（改动相关代码时留意）

- 覆盖层的透明、置顶、鼠标穿透等 overlay 特性通过 `platform/windows.rs` 的 Win32 API 实现：透明/穿透/置顶由 winit 窗口属性（`with_transparent` + `set_cursor_hittest(false)` + `WindowLevel::AlwaysOnTop`）设置，`setup_overlay_window` 仅补充 `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`。`overlay_renderer.rs` 走 softbuffers CPU 光栅化而非 wgpu swapchain，以避开 DirectComposition 的透明坑。
- **覆盖层必须选择目标窗口才能创建**：`target_window` 为空时 `create_overlay` 直接返回，不会创建全屏 overlay。
- `TriggerRule`（进程触发）在 schema 中已定义（`enabled` + `process_names`），但**二进制层未消费**——当前没有任何按前台进程名自动启用/隐藏覆盖层的逻辑，属于纯配置占位。
- `RandomOrb` 的 `LockOnStart` / `Reshuffle` 两种模式在当前渲染实现中行为相同——均每帧从配置参数派生种子重新生成（预览与 overlay 用相同的 `SimpleRng` 实现 + 种子，保证一致）。schema 中的 `random_orb_x/y` 持久化字段已定义但渲染层尚未消费，待后续接入。
- `settings_ui.rs::pick_png_file` 当前是占位实现（始终返回 `None`），「浏览…」按钮暂不可用，用户需手动粘贴 PNG 路径。
- CI 的 config 测试与 release 编译在三平台运行；`windows` crate 与 `embed-resource` 通过 `[target.'cfg(windows)'.dependencies]` 声明，非 Windows target 不拉取。
