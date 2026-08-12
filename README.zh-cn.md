# Peregrine

[English](README.md) · [简体中文](README.zh-cn.md)

[![下载](https://img.shields.io/github/v/release/Eeymoo/peregrine?style=for-the-badge&label=%E4%B8%8B%E8%BD%BD)](https://github.com/Eeymoo/peregrine/releases)

Peregrine 是一款专注于缓解 3D 眩晕（Motion Sickness）的桌面工具。它通过在屏幕上提供可定制的视觉锚点（如十字准星、边框、边缘箭头等），帮助玩家在快速转动视角或复杂场景中保持视觉稳定，减轻前庭-视觉冲突带来的恶心、头晕症状。让你能够正常游玩《半条命 2》《镜之边缘》《消逝的光芒》《无人深空》等容易引发眩晕的游戏。

> **当前状态：正式版已发布。** Windows 透明置顶穿透覆盖层、目标窗口跟随、多种视觉锚点样式、自定义 PNG 贴图、配置热重载等功能已可用。
>
> 面向使用者的操作说明请看 **[使用帮助](https://peregrine.aukcraft.org/zh-cn/guide/help.html)**。
> 贡献指南请看 **[贡献指南](https://peregrine.aukcraft.org/zh-cn/guide/contributing.html)**。
>
> **翻译说明：** 界面支持 6 门语言（简体中文、English、日本語、Deutsch、Français、Русский）。非中文翻译由 AI 初版生成，可能不够地道。欢迎母语者通过[翻译改进 issue 模板](https://github.com/Eeymoo/peregrine/issues/new?template=translation-improvement.yml)提交修正建议。

## 快速开始

### 下载与启动（推荐）

1. 前往 **[Releases 页面](https://github.com/Eeymoo/peregrine/releases)** 下载最新版本。
2. 根据系统选择对应包：
   - `peregrine-v*-windows-x64.zip` — 64 位系统
   - `peregrine-v*-windows-x86.zip` — 32 位系统
   - `peregrine-v*-windows-arm64.zip` — ARM 设备
3. 解压后直接运行 `peregrine.exe`，程序默认打开**配置窗口**。
4. 选择目标窗口、调整样式后，点击「开始覆盖」即可在游戏上方显示视觉锚点。

> 首次运行会自动生成默认配置，所有设置自动保存。关闭配置窗口后会收起到系统托盘。

### 从源码构建

需要 Rust 工具链（≥ 1.85）与 Node.js（≥ 20）。在仓库根目录执行：

```bash
# 安装前端依赖
npm install

# 构建 workspace
cargo build
cargo build --release

# 运行 Tauri 开发版本
npx tauri dev

# 构建 Tauri release 产物
npx tauri build

# 测试
cargo test                      # 全部
cargo test -p peregrine_config  # 只测配置库

# 格式化 / 检查
cargo fmt
cargo clippy -p peregrine_config -- -D warnings
```

## 特性

Peregrine 的视觉样式系统基于 **四层架构**：

1. **元素（Elements）** —— 渲染器知道如何绘制的基础几何（矩形、圆、环、三角形、多边形、线、文本、图片）。
2. **物料（Materials）** —— 把 `(参数, 屏幕区域) → 元素列表` 映射起来的 Rhai 脚本。既有内置物料（准星、贴边矩形、定位球、圆环、自定义定位球、随机球、边框、箭头、网格、图片），也支持放入用户物料目录的自编写 `.rhai` 文件。
3. **图层（Layers）** —— 物料实例 + 自带参数 / 变换 / 样式，按数组顺序堆叠合成完整锚点。任意叠加。
4. **配置（Configuration）** —— 由图层集合 + 触发规则 / 热键 / 目标窗口构成的 Profile，原子写入 + 热重载。

内置物料之外，你还可以用 Rhai 编写自己的锚点样式（见 [物料脚本创作](https://github.com/eeymoo/peregrine/blob/main/docs/zh-cn/guide/material-scripting.md)），无需改动 Rust 代码。

亮点：

- **Windows 透明覆盖层窗口**：置顶、鼠标穿透的 Overlay 窗口，可悬浮于游戏或应用上方。
- **目标窗口跟随**：通过下拉列表选择目标窗口，覆盖层可跟随其位置与尺寸。
- **多图层锚点**：可叠加准星、边框、圆环、角点、箭头、图片等任意组合；每图层独立重排、隐藏、锁定。
- **用户可编程物料**：编写 `.rhai` 物料脚本，UI 自动生成控件（number / slider / color / select / toggle / image_path / text）。启用动态物料后还可使用动态输入 API（`time_ms` / `mouse_pos` / `key_down` / `rand`）。
- **自定义 PNG 贴图**：支持加载 PNG 图片作为覆盖层内容。
- 每种物料通过 `schema()` 暴露可调参数（尺寸、厚度、颜色、不透明度、间隙、边缘位置等）。
- 多 Profile 配置，可为不同场景保存独立图层集合。
- 实时预览：在配置面板中调整参数即时看到效果。
- 配置持久化 + 热重载：外部编辑配置文件后自动生效。
- **匿名、可选的遥测**，支持编译期彻底禁用 —— 详见 [隐私与遥测](https://github.com/eeymoo/peregrine/blob/main/docs/zh-cn/guide/privacy.md)。
- **Tauri + React 配置界面**：设置面板基于 Webview，易于扩展与主题化。
- **GitHub Actions 自动构建**：Windows x86 / x86_64 / ARM64 构建与发布。

## 技术栈

- **语言 / 生态**：Rust，Cargo workspace（`edition = "2024"`，`rust-version = 1.85`，MIT 许可）。
- **配置界面**：[Tauri](https://tauri.app/) 2.x + [React](https://react.dev/) 18 + [Tailwind CSS](https://tailwindcss.com/) 3 + [shadcn/ui](https://ui.shadcn.com/)。
- **覆盖层**：[winit](https://github.com/rust-windowing/winit)（窗口 / 事件循环）+ [softbuffer](https://github.com/rust-windowing/softbuffer)（CPU 像素光栅化）。
- **异步运行时**：[tokio](https://github.com/tokio-rs/tokio)（配置读写、文件热重载、后台跟随任务）。
- **目标平台**：**Windows**（x86 / x86_64 / ARM64）。

### 系统托盘

启动后程序常驻系统托盘：

| 菜单 | 行为 |
| --- | --- |
| 配置 | 显示/聚焦配置窗口 |
| 设置 | 显示关于等系统设置窗口 |
| 退出 | 完全退出程序 |

左键点击托盘图标也可恢复配置窗口。

## 项目结构

```
peregrine/
├── Cargo.toml            # workspace 根：成员、workspace.package、workspace.dependencies
├── Cargo.lock
├── package.json          # 前端 npm 依赖
├── src/                  # 前端源码（React + Tailwind + shadcn/ui）
│   ├── ConfigApp.tsx     # 配置窗口主组件
│   ├── SettingsApp.tsx   # 设置窗口主组件
│   ├── lib/
│   │   ├── api.ts        # Tauri invoke 封装
│   │   ├── shapes.ts     # 前端预览几何计算
│   │   └── i18n.tsx      # 国际化
│   ├── components/
│   │   ├── Preview.tsx   # Canvas 实时预览
│   │   ├── StyleFields.tsx # 样式参数表单
│   │   └── ui/           # shadcn/ui 基础组件
│   └── i18n/             # 多语言 JSON
├── src-tauri/            # peregrine-tauri：Tauri 后端入口
│   └── src/
│       ├── lib.rs        # Tauri 启动入口、配置初始化、tray、commands
│       ├── main.rs       # 二进制入口
│       └── overlay.rs    # 独立线程运行 winit 事件循环管理 overlay
├── crates/
│   ├── config/           # peregrine_config：纯逻辑库（无 UI / GPU / 窗口代码）
│   │   └── src/
│   │       ├── lib.rs        # 模块导出 + 统一错误类型
│   │       ├── schema.rs     # 配置数据结构 + 校验 + 单测
│   │       ├── storage.rs    # 配置文件路径管理、原子读写、默认配置生成
│   │       ├── notifier.rs   # 基于 tokio::sync::watch 的配置变更广播
│   │       └── watcher.rs    # 基于 notify crate 的配置文件热重载（含去抖）
│   └── peregrine/        # peregrine：共享库（供 Tauri 复用）
│       └── src/
│           ├── lib.rs             # 导出 overlay_renderer / shapes / platform
│           ├── overlay_renderer.rs # softbuffer 覆盖层渲染器
│           ├── shapes.rs           # 准心几何共享模块
│           └── platform/
│               ├── mod.rs          # 平台模块入口
│               └── windows.rs      # Win32 API：透明/置顶/穿透、目标窗口查找与跟随
└── docs/                 # VitePress 文档站点
```

**分层原则**：`peregrine_config` 不得依赖任何 UI / GPU / 窗口平台代码（`winit` / `wgpu` / `egui`）；平台与渲染逻辑放在 `peregrine` 共享库与 `src-tauri` 二进制 crate。

## 配置

配置文件为 JSON，路径由操作系统标准目录决定：

| 平台 | 路径 |
| --- | --- |
| Windows | `%APPDATA%/Peregrine/config.json` |
| macOS | `~/Library/Application Support/Peregrine/config.json` |
| Linux | `~/.config/Peregrine/config.json` |

> Peregrine 是仅面向 Windows 的工具。上表列出 macOS / Linux 路径是因为配置库按操作系统标准目录实现；覆盖层及核心功能仅在 Windows 上实现。

- 配置根为 `AppConfig`（`active_profile` + 多个命名 `Profile`）。首次运行会自动生成默认配置。
- 写入为原子操作（同目录临时文件 + `rename`），写入前必先校验，避免落盘非法配置。
- 支持热重载：外部编辑配置文件后，`ConfigWatcher`（notify + 300ms 去抖）检测到改动并广播给渲染器。

配置流动链路：

```
UI 改动 / 外部编辑文件
   → ConfigStorage（原子写）/ ConfigWatcher（notify + 去抖）
   → ConfigNotifier（watch 广播）
   → 订阅者（渲染器读取的共享快照）
```

## 许可

MIT License。
