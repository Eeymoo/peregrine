# 使用说明

## 下载

1. 打开 [GitHub Releases](https://github.com/eeymoo/peregrine/releases) 页面。
2. 下载最新正式版（Release）对应你系统的压缩包：
   - `peregrine-<版本>-windows-x64.zip`（64 位系统，最常见）
   - `peregrine-<版本>-windows-x86.zip`（32 位系统）
   - `peregrine-<版本>-windows-arm64.zip`（ARM64 设备）
3. 解压到任意文件夹。

## 启动

双击解压后的 `peregrine.exe`，程序启动后默认进入**设置模式**。

## 基础操作

| 按键 | 作用 |
|------|------|
| `Tab` | 切换「设置模式」和「覆盖层模式」 |
| `Esc` | 设置模式下返回覆盖层；覆盖层模式下退出程序 |

## 三步上手

1. **选择样式**：在设置面板找到 **Crosshair → Style**，选择 `Cross`、`Ring`、`ToiletPaper` 等样式。
2. **调整外观**：修改 **Size**（尺寸）、**Thickness**（粗细）、**Color**（颜色）、**Opacity**（不透明度），右侧预览会实时更新。
3. **进入游戏**：按 `Tab` 切换到覆盖层模式，辅助贴图将显示在屏幕中心，且不会阻挡鼠标和键盘。

## 多配置方案

一个 **Profile** 是一套完整配置。你可以创建多个 Profile：

- 不同游戏使用不同准心
- 不同玩家共用电脑时各用各的配置

在设置面板切换 **Active Profile** 即可即时生效。

## 自定义 PNG 贴图

1. 准备一张带透明背景的 PNG 图片。
2. 在设置面板中将 **Style** 切换到支持自定义贴图的选项（如 `CustomOrb` 或相关样式）。
3. 选择图片路径并调整尺寸。

## 跟随游戏窗口

在 **Target Window** 中填写游戏窗口标题关键字（例如 `Cyberpunk 2077`），Peregrine 会尝试跟随该窗口移动。不填写则固定显示在屏幕中心。

## 退出程序

在覆盖层模式下按 `Esc`，或右键系统托盘图标选择退出。

## 遇到问题？

- 查看 [配置说明](./config) 了解配置文件格式。
- 到 [GitHub Issues](https://github.com/eeymoo/peregrine/issues) 反馈问题。
