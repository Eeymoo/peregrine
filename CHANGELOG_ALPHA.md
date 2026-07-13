# 测试版更新日志

以下版本为正式版发布前的测试与开发迭代，仅供预览与反馈。

正式版日志见 **[CHANGELOG.md](CHANGELOG.md)**。

---

## [Unreleased]

## [v0.1.13-alpha.0] — 2026-07-13

v0.1.13 的预发布版本。

### 新增

- **单例模式**：重复启动应用时自动聚焦已有窗口，不再运行多个实例。 @Eeymoo
- **Markdown 更新日志**：更新检查面板使用 react-markdown 渲染发布说明，支持完整 Markdown 排版。 @Eeymoo

### 变更

- **前端组件拆分重构**：ConfigApp / SettingsApp 大幅拆分为独立 hooks 与子组件（`components/config`、`components/settings`、`hooks/`），提升可维护性。 @Eeymoo

### 修复

- **镜像下载修复**：启用中国大陆镜像时，安装包下载链接也套用镜像前缀，之前仅清单 URL 走镜像。 @Eeymoo

## [v0.1.9-alpha.0] — 2026-07-13

v0.1.9 的预发布版本。改动已合并至 v0.1.9 正式版。

### 新增

- **SVG 渲染后端**：覆盖层新增可选 SVG 渲染后端（resvg + tiny-skia），在「设置 → 覆盖层 → 渲染后端」中切换。SVG 模式抗锯齿质量更高；CPU 模式（默认）零额外依赖、更轻量。两套方案并行，SVG 光栅化失败时自动回退到 CPU。 @Eeymoo
- **网格准心样式**：新增 `Grid` 准心样式，可调整网格行列数、线宽与颜色。 @Eeymoo
- **全局快捷键体系**：支持为「开始/停止覆盖」等功能绑定全局热键。 @Eeymoo
- **快捷颜色预设**：颜色选择器新增常用预设。 @Eeymoo
- **覆盖层抗锯齿**：CPU 渲染模式新增抗锯齿开关，默认开启。 @Eeymoo
- **滚动条样式优化**：自定义滚动条样式，默认透明、悬停淡入，6px 宽圆角。 @Eeymoo

### 修复

- 移除拖拽实时显示在某些场景下被强制禁用的限制。 @Eeymoo

---

## [v0.1.4-alpha.0] — 2026-07-11

### 优化

- 限制 overlay 渲染帧率为 60 FPS：消除 `about_to_wait` 与 `RedrawRequested` 重复渲染导致的忙循环，显著降低「开始覆盖」后的 CPU 占用。
- 关闭配置/设置窗口时真正销毁 WebView2：不再隐藏到托盘占内存，托盘点「配置」「设置」时再重新创建。
- 启动时不预创建「设置」窗口：按需创建，进一步降低启动内存。

### 修复

- 修复托盘「退出」失效：`RunEvent::ExitRequested` 全局阻止退出会拦截 `app.exit(0)`，改为通过 `quitting` 标志区分主动退出与窗口关闭。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.1.3-alpha.4] — 2026-07-11

### 变更

- 移除「边框」样式的「四边中缝缺口（20%）」选项（`border_gap` 字段），该选项无实际渲染效果，属于死代码。
- 暂时隐藏「自定义图片」准心样式（`custom_image`），存在已知问题待修复。
- 未选择目标窗口时「开始覆盖」按钮禁用，防止用户误触。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.1.3-alpha.3] — 2026-07-11

### 变更

- 发布产物从 NSIS 安装程序（`*-setup.exe`）改回便携 zip 压缩包：每个架构单独打包为 `peregrine-windows-x86.zip` / `peregrine-windows-x64.zip` / `peregrine-windows-arm64.zip`，下载解压即可运行，无需安装。

### 修复

- 修复托盘菜单语言跟随系统语言失效：Windows 上 `LANG` 环境变量通常不存在，改用 Win32 API `GetUserDefaultLocaleName` 检测系统语言。
- 修复「开始覆盖后自动隐藏并切换到游戏」功能失效：`SetForegroundWindow` 受前台锁定限制，改用 `AttachThreadInput` + `BringWindowToTop` 组合可靠切换。
- 修复设置窗口修改「自动切换」偏好后配置窗口未同步：新增 `peregrine:settings-changed` 事件广播，两窗口 React state 实时同步。
- 修复配置预览棋盘格背景错乱：`%` 运算符优先级高于 `+` 导致格子交替模式错乱。

### 优化

- 图标清晰度大幅提升：生成脚本改用 8x 超采样抗锯齿，ICO 包含 16/32/48/64/128/256 六档，托盘与窗口标题栏使用 1024x1024 高分辨率 PNG 源图，高 DPI 下清晰锐利。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.1.3-alpha.2] — 2026-07-10

### 修复

- 修复 `Locale` 类型包含 `"auto"` 后与 `localeMap` 索引类型不匹配导致的 TypeScript 编译失败，CI 构建中断。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.1.3-alpha.1] — 2026-07-10

### 新增

- 语言设置新增「跟随系统」选项，默认根据系统语言自动选择简体中文或英文。
- 设置页新增「开始覆盖时自动切换到游戏」偏好：每次询问 / 是 / 否。
- 首次点击「开始覆盖」时弹出确认对话框，可选择是否记住该选择。

### 变更

- 语言与自动切换偏好统一持久化到 `config.json` 的 `settings` 中，移除前端的 `localStorage` 依赖，跨窗口同步更可靠。
- 托盘菜单文本在应用启动时即根据当前语言初始化。

### 修复

- 修复 `npm ci` 时 `picomatch` 版本与 `package-lock.json` 不一致导致的安装失败。
- 修复 alpha 预发布版本号无法打包 MSI 的问题：发布产物改用 NSIS（`*-setup.exe`）。
- 修复 overlay 事件循环在非主线程创建时缺少 `with_any_thread(true)` 导致的 panic。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.1.3-alpha.0] — 2026-07-10

### 新增

- 应用国际化：支持简体中文与英文，在「设置 → 语言」中切换，窗口标题、托盘菜单、错误提示同步切换。
- 文档站点增加完整英文版。
- 新增「术语表」页面（中英文），强制统一核心概念与 12 种视觉锚点样式名称。

### 修复

- 修复 `RandomOrb` 样式在前端预览与 Rust 覆盖层之间的 RNG 不一致，统一为相同 64-bit LCG，确保随机边缘标记位置一致。
- 清理 `shapes.rs` / `overlay_renderer.rs` 中残留的 egui / settings_ui 时代注释。

### 文档

- 统一 `docs/`、`README.md`、`HELP.md` 中的中英文术语：视觉锚点、覆盖层、配置窗口、边缘矩形、十字准星、边缘标记、中心圆环等。
- 更新构建说明为 Tauri 流程（`npm install` + `npx tauri dev/build`）。
- 补全 `docs/en/guide/config.md` 英文版配置说明。

> 更新者：Eeymoo（Peregrine 维护者）

---

## [v0.2.0-alpha.2] — 2026-07-08

### 修复

- 十字准星（Cross）调整间距时整体向左上偏移：左臂与顶臂多减了一个半间距，导致左侧/上方间距是右侧/下方的两倍。修正为以中心对称展开，间距两侧均等。

---

## [v0.1.1-alpha.1] — 2026-07-07

### 修复

- macOS 上 wgpu surface 不支持 `Inherit` alpha 模式导致启动 panic，改为按 capabilities 自动选择。

### 构建

- Windows MSVC 三个目标（x86/x64/ARM64）开启 `+crt-static` 静态链接 C 运行时，exe 不再依赖 `VCRUNTIME140.dll` 等外部 DLL。
- Release CI 增加 DLL 依赖验证步骤，确保产物不含 VC 运行时动态依赖。

### 文档

- 添加 VitePress 文档站点与 GitHub Pages 自动部署。
- 修正仓库链接与使用说明，首页增加立即下载按钮。
- 显式添加 search-insights 依赖以修复 CI `npm ci`。

---

## [v0.2.0-alpha.0] — 2026-07-06

### 新增

- PNG 图片支持：可加载自定义 PNG 作为覆盖层贴图。
- 预览与覆盖层统一几何模块，减少逻辑重复。

### 变更

- 覆盖层改用 softbuffer 像素缓冲区方案（参考 simple-crosshair-overlay）。
- 设置 UI 与覆盖层渲染共享几何绘制逻辑。

---

## [v0.1.0-alpha.12] — 2026-07-02

- 架构重构为双窗口：设置窗口与独立 Overlay 窗口分离。

## [v0.1.0-alpha.11] — 2026-07-02

- 移除所有非 Windows 平台代码，项目聚焦 Windows。

## [v0.1.0-alpha.10] — 2026-07-02

- 修复透明度彻底失效：强制 Bgra8Unorm 避免 sRGB gamma 导致颜色键不匹配。

## [v0.1.0-alpha.9] — 2026-07-02

- 修复日志默认不输出：EnvFilter 改为默认 info 级别。

## [v0.1.0-alpha.8] — 2026-07-02

- 修复 HWND 跨线程获取失败。
- 新增未选窗口防护。
- 修复窗口尺寸恢复与清理冗余。

## [v0.1.0-alpha.7] — 2026-07-02

- 修复颜色键吃黑色准心。
- 修复覆盖层切换闪烁。
- 修复窗口标题匹配逻辑。

## [v0.1.0-alpha.6] — 2026-07-02

- 新增"开始覆盖"按钮。
- 修复透明颜色键。
- 添加选择窗口日志，清理调试打印。

## [v0.1.0-alpha.5] — 2026-07-02

- 编译优化。
- 嵌入 Windows exe 图标。

## [v0.1.0-alpha.4] — 2026-07-02

- Windows 覆盖层保留 Bgra8UnormSrgb 以修复 DWM 透明合成。

## [v0.1.0-alpha.3] — 2026-07-02

- 修复 Windows 窗口选择：统一枚举源并健壮循环。

## [v0.1.0-alpha.2] — 2026-07-01

- 修复 Windows 黑窗口问题。
- 修复中文方框字体。
- 修复窗口选择与透明叠加。

## [v0.1.0-alpha.1] — 2026-07-01

- Release 工作流仅构建并发布 Windows (x86_64)。

## [v0.1.0-alpha.0] — 2026-07-01

- 首个测试版本。
- 新增 Windows Overlay 透明置顶穿透窗口。
- 新增目标窗口跟随功能。
- 基础准心样式支持。

---

[v0.1.13-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.13-alpha.0
[v0.1.9-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.9-alpha.0
[v0.1.4-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.4-alpha.0
[v0.1.3-alpha.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.4
[v0.1.3-alpha.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.3
[v0.1.3-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.2
[v0.1.3-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.1
[v0.1.3-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.0
[v0.2.0-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.2.0-alpha.2
[v0.1.1-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1-alpha.1
[v0.2.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.2.0-alpha.0
[v0.1.0-alpha.12]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.12
[v0.1.0-alpha.11]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.11
[v0.1.0-alpha.10]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.10
[v0.1.0-alpha.9]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.9
[v0.1.0-alpha.8]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.8
[v0.1.0-alpha.7]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.7
[v0.1.0-alpha.6]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.6
[v0.1.0-alpha.5]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.5
[v0.1.0-alpha.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.4
[v0.1.0-alpha.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.3
[v0.1.0-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.2
[v0.1.0-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.1
[v0.1.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.0
