# 项目介绍

**Peregrine** 是一款桌面辅助贴图（准心 / 覆盖层）工具，核心用途是帮助玩家缓解 **3D 眩晕**。

## 什么是 3D 眩晕？

在玩第一人称或第三人称 3D 游戏时，部分玩家会因为画面频繁移动、视角切换过快而感到头晕、恶心。这种现象通常被称为 3D 眩晕或晕动症（Motion Sickness）。

一个常见缓解方法是：在屏幕中心或边缘放置一个**固定的视觉锚点**。当画面剧烈晃动时，眼睛可以借助这个锚点快速恢复空间感，从而减轻眩晕。

## Peregrine 的作用

Peregrine 在屏幕中心绘制一个半透明的视觉锚点，例如：

- 准星 / 大准星
- 中心环
- 定位球
- 贴边矩形（经典锚点）
- 自定义 PNG 贴图

锚点始终位于屏幕最上层，并且**不会阻挡鼠标和键盘操作**，让你在游戏中几乎感觉不到它的存在。

## Peregrine 的工作原理

Peregrine 的核心是「覆盖层」（Overlay）——一个特殊的窗口，它满足三个条件：

- **透明**：窗口背景完全透明，只有准心 / 贴图部分可见，不会遮挡游戏画面。
- **置顶**：始终位于所有其他窗口之上，即使游戏全屏也能显示。
- **鼠标穿透**：窗口不拦截任何鼠标点击和键盘输入，你可以正常操作游戏，几乎感觉不到它的存在。

在 Windows 上，这些特性通过系统 API 实现：透明与穿透由窗口属性（`with_transparent` + 鼠标命中测试关闭 + 置顶层级）完成，程序额外补上「不抢焦点」和「不在任务栏显示」的属性，确保覆盖层安静地浮在游戏之上。

渲染上，覆盖层走 **CPU 像素光栅化**（softbuffer）而非 GPU，以避开 Windows 透明合成的一些坑。程序直接在像素缓冲里绘制准心几何形状（矩形、圆、线段、三角形等），再交给系统合成到屏幕上。这种方式轻量、稳定，不依赖游戏的渲染管线。

设置窗口则用 **egui**（即时模式 GUI）+ **wgpu**（GPU 渲染）绘制，提供实时预览——你在设置面板里看到的准心，和覆盖层实际显示的完全一致。

想了解为什么会眩晕、以及除视觉锚点外的其他缓解方法，请参阅 [缓解晕 3D](./motion-sickness.md)。

## 依赖说明

Peregrine 使用 Rust 编写，以下是主要依赖及其用途：

| 依赖 | 用途 |
|------|------|
| [winit](https://github.com/rust-windowing/winit) | 跨平台窗口创建与事件循环 |
| [wgpu](https://github.com/gfx-rs/wgpu) | 设置窗口的 GPU 渲染（基于 WebGPU） |
| [egui](https://github.com/emilk/egui) / egui-wgpu / egui-winit | 即时模式 GUI，用于设置面板与实时预览 |
| [softbuffer](https://github.com/rust-windowing/softbuffer) | 覆盖层的 CPU 像素缓冲光栅化 |
| [tokio](https://github.com/tokio-rs/tokio) | 异步运行时，驱动配置读写、热重载、窗口跟随 |
| [tray-icon](https://github.com/tauri-apps/tray-icon) | 系统托盘图标与菜单 |
| [png](https://github.com/image-rs/image-png) | 自定义 PNG 准心贴图解码 |
| [notify](https://github.com/notify-rs/notify) | 配置文件热重载（监听文件变动） |
| [serde](https://github.com/serde-rs/serde) / serde_json | 配置文件的序列化与反序列化 |
| [tracing](https://github.com/tokio-rs/tracing) | 结构化日志 |
| [windows](https://github.com/microsoft/windows-rs) | Windows 平台 API（透明 / 置顶 / 穿透 / 窗口跟随） |

所有依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]` 中声明。完整依赖列表请参阅 [Cargo.lock](https://github.com/eeymoo/peregrine/blob/main/Cargo.lock)。

## 开源说明

Peregrine 使用 [PolyForm Noncommercial 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) 许可发布。

**这意味着你可以：**

- ✅ 个人非商业用途自由使用、修改、分发
- ✅ 阅读和学习全部源码
- ✅ 提交 Issue 和 Pull Request 参与改进

**但你不能：**

- ❌ 将 Peregrine 或其衍生作品用于商业目的（销售、捆绑销售、商业服务、内部商业工具等）

如果你对商业用途有需求，欢迎通过 [GitHub Issues](https://github.com/eeymoo/peregrine/issues) 联系作者协商商业授权。

### 参与贡献

欢迎提交 Issue 与 Pull Request。请参阅仓库中的 [`CONTRIBUTING.md`](https://github.com/eeymoo/peregrine/blob/main/CONTRIBUTING.md) 了解贡献规范，以及 [`开发构建`](./development.md) 页面了解如何本地构建与测试。
