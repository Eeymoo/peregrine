# Peregrine

Peregrine 是一个桌面辅助贴图(准心 / 覆盖层)工具,**主要用途是防 3D 眩晕**:在屏幕中心或边缘绘制半透明的视觉锚点,为玩家在 3D 游戏中提供固定参照物,缓解眩晕感。

> **当前状态:可运行的骨架。** UI、配置、渲染管线已打通,但覆盖层尚未实现真正的透明置顶穿透窗口,"跟随窗口""进程触发"等功能仍为占位。详见文末 [已知局限](#已知局限)。

## 特性

- 多种辅助贴图样式:卫生纸(矩形贴边)、准星、大准星、定位球(4/6/8 点)、中心环、自定义定位球、随机球、边框。
- 每种样式可调尺寸、厚度、颜色、不透明度、间隙、贴边位置等参数。
- 多 Profile 配置,可为不同场景保存独立设置。
- 实时预览:在设置面板中调整参数即时看到效果。
- 配置持久化 + 热重载:外部编辑配置文件后自动生效。
- 内置 egui 设置界面,通过热键在覆盖层 / 设置两种模式间切换。

## 技术栈

- **语言 / 生态**:Rust,Cargo workspace(`edition = "2024"`,`rust-version = 1.85`,MIT 许可)。
- **图形栈**:[`winit`](https://github.com/rust-windowing/winit)(窗口 / 事件循环) + [`wgpu`](https://github.com/gfx-rs/wgpu)(GPU 渲染) + [`egui`](https://github.com/emilk/egui) / `egui-wgpu` / `egui-winit`(即时模式 UI)。
- **异步运行时**:[`tokio`](https://github.com/tokio-rs/tokio)(配置读写、文件热重载、后台任务)。
- **目标平台**:以 **macOS** 为主要开发 / 运行平台(字体加载、配置目录等有 macOS 专门处理),同时对 Windows / Linux 做了条件编译兼容。

## 构建与运行

需要较新的 Rust 工具链(edition 2024,rustc/cargo ≥ 1.85)。在仓库根目录执行:

```bash
# 构建
cargo build
cargo build --release

# 运行 GUI 主程序(workspace 有多个成员,需用 -p 指定)
cargo run -p peregrine

# 测试
cargo test                      # 全部
cargo test -p peregrine_config  # 只测配置库

# 格式化 / 检查
cargo fmt
cargo clippy
```

### 快捷键

| 按键 | 行为 |
| --- | --- |
| `Tab` | 在覆盖层 / 设置两种模式间切换 |
| `Esc` | 在设置模式下切回覆盖层;在覆盖层模式下退出程序 |

程序启动后默认进入**设置**模式,窗口逻辑尺寸 960×560。

## 项目结构

```
peregrine/
├── Cargo.toml            # workspace 根:成员、workspace.package、workspace.dependencies
├── Cargo.lock
└── crates/
    ├── config/           # peregrine_config:纯逻辑库(无 UI / GPU / 窗口代码)
    │   └── src/
    │       ├── lib.rs        # 模块导出 + 统一错误类型
    │       ├── schema.rs     # 配置数据结构 + 校验 + 单测
    │       ├── storage.rs    # 配置文件路径管理、原子读写、默认配置生成
    │       ├── notifier.rs   # 基于 tokio::sync::watch 的配置变更广播
    │       └── watcher.rs    # 基于 notify crate 的配置文件热重载(含去抖)
    └── peregrine/        # peregrine:二进制程序(GUI 主程序)
        └── src/
            ├── main.rs        # 入口、winit ApplicationHandler、模式切换、tokio 主循环
            ├── renderer.rs    # wgpu + egui 渲染器、覆盖层绘制、字体加载
            └── settings_ui.rs # egui 设置面板与实时预览绘制
```

**分层原则**:`peregrine_config` 不得依赖任何 UI / GPU / 窗口平台代码(`winit` / `wgpu` / `egui`);平台与渲染逻辑只放在 `peregrine` 二进制 crate。

## 配置

配置文件为 JSON,路径由操作系统标准目录决定:

| 平台 | 路径 |
| --- | --- |
| macOS | `~/Library/Application Support/Peregrine/config.json` |
| Windows | `%APPDATA%/Peregrine/config.json` |
| Linux | `~/.config/Peregrine/config.json` |

- 配置根为 `AppConfig`(`active_profile` + 多个命名 `Profile`)。首次运行会自动生成默认配置。
- 写入为原子操作(同目录临时文件 + `rename`),写入前必先校验,避免落盘非法配置。
- 支持热重载:外部编辑配置文件后,`ConfigWatcher`(notify + 300ms 去抖)检测到改动并广播给渲染器。

配置流动链路:

```
UI 改动 / 外部编辑文件
   → ConfigStorage(原子写)/ ConfigWatcher(notify + 去抖)
   → ConfigNotifier(watch 广播)
   → 订阅者(渲染器读取的共享快照)
```

## 已知局限

当前处于骨架阶段,以下功能尚未完成:

- 覆盖层窗口目前是普通窗口,**尚未实现透明、置顶、鼠标穿透**等真正的 overlay 特性。
- `Profile.target_window`("选择窗口"按钮)与 `TriggerRule`(进程触发)均为**占位**,未接入平台 API。
- `Renderer` 未处理窗口大小变化(无 resize 重配逻辑)。

## 许可

MIT License。
