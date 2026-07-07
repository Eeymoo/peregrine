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
- 卫生纸卷（经典锚点）
- 自定义 PNG 贴图

锚点始终位于屏幕最上层，并且**不会阻挡鼠标和键盘操作**，让你在游戏中几乎感觉不到它的存在。

## 技术栈

| 层级 | 技术 |
|------|------|
| 语言 / 构建 | Rust，Cargo Workspace |
| 窗口 / 事件循环 | winit |
| GPU 渲染 | wgpu + egui |
| 覆盖层渲染 | softbuffer（CPU 光栅化） |
| 异步运行时 | tokio |
| 系统托盘 | tray-icon |
| 目标平台 | Windows（x86 / x86_64 / ARM64） |

## 开源许可

Peregrine 使用 [PolyForm Noncommercial](https://polyformproject.org/licenses/noncommercial/1.0.0/) 许可发布，允许非商业用途自由使用与修改。

## 参与贡献

欢迎提交 Issue 与 Pull Request。请参阅仓库中的 `CONTRIBUTING.md` 了解贡献规范。
