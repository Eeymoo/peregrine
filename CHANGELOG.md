# 更新日志

仅记录正式版本发布。测试版 / 预览版本见 **[CHANGELOG_ALPHA.md](CHANGELOG_ALPHA.md)**。

---

## [v0.1.4] — 2026-07-11

正式版本。协议变更为 MIT 完全开源；新增全屏/窗口覆盖模式、GPU 加速开关、屏幕缩放自适应；大幅优化内存占用与 CPU 消耗。

### 新增

- **全屏 / 窗口覆盖模式**：全屏模式（默认）直接覆盖整个屏幕，无需选择目标窗口；窗口模式仅覆盖目标窗口区域。配置页面勾选「窗口模式」或托盘菜单切换，两侧自动同步。
- **拖拽时实时显示设置**：在「设置」中开启后窗口拖拽时覆盖层实时跟随；关闭时（默认）停止拖拽约 1200ms 后恢复显示，降低 CPU 占用。
- **GPU 硬件加速开关**：在「设置」中可开启 GPU 硬件加速（默认关闭），关闭时使用纯 CPU 渲染以减少 GPU 进程内存占用；切换后弹出重启确认对话框。
- **版本号自动化**：版本号从 git tag 动态读取，CI 打包时自动同步到各处，不再手动维护。

### 修复

- 修复全屏模式下覆盖层位置错误：首次创建时未预定位到屏幕区域。
- 修复屏幕分辨率/DPI 缩放变化后覆盖层不跟随更新：全屏模式现在持续检测屏幕尺寸变化。
- 修复打开配置页面时覆盖状态显示错误：`get_overlay_active` 改为直接读取原子状态。
- 修复左侧预览在窗口尺寸变化后不刷新：加入 ResizeObserver，拖拽/缩放时立即重绘。
- 修复预览比例与实际覆盖层不一致：预览以真实分辨率构建准心形状再等比缩放。
- 修复 ESC 对话框行为：ESC 取消等同于停止覆盖；保持配置窗口不会停止覆盖。
- 修复 WebView2 进程在窗口关闭后未释放内存：改为真正销毁而非隐藏到托盘。
- 修复托盘「退出」失效：`ExitRequested` 全局阻止退出会拦截主动退出。
- 修复文档部署 CI 失败：VitePress 构建时继承根目录 PostCSS 配置导致找不到 tailwindcss 模块。

### 优化

- **静态准心不再持续重绘**：引入脏标记机制，静止不动的准心不再每帧重绘，显著降低覆盖层 CPU 占用。
- **配置保存防抖**：拖滑块等连续操作时只在停止后 300ms 写入一次，避免频繁触发文件 watcher。
- **启动时不预创建设置窗口**：按需创建，降低启动内存。
- 发布产物 zip 内增加 README.md 与 LICENSE，exe 文件名包含版本号。
- `cargo fmt` 格式化全部 Rust 代码。

### 变更

- **协议变更为 MIT**：从 PolyForm Noncommercial 1.0.0 改为 MIT，完全开源，允许商业使用。

### 下载

- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets（内含 `peregrine-v0.1.4.exe`、`README.md`、`LICENSE`）。

---

## [v0.1.3] — 2026-07-11

正式版本。迁移到 Tauri + React 设置面板，新增中英文国际化与自动切换游戏窗口，发布产物改为便携 zip，图标清晰度大幅提升。

### 新增

- 全新设置界面：基于 Tauri + React + shadcn/ui 重新构建，配置窗口与设置窗口分离。
- 应用国际化：支持简体中文与英文，设置页一键切换，窗口标题、托盘菜单、错误提示同步切换；支持「跟随系统语言」。
- 文档站点英文版：完整英文使用说明、配置说明与术语表。
- 开始覆盖时自动切换到游戏：支持「每次询问 / 是 / 否」三种偏好，未选目标窗口时禁用开始覆盖按钮。

### 修复

- 修复托盘菜单语言跟随系统语言失效：Windows 上改用 Win32 API `GetUserDefaultLocaleName` 检测系统语言。
- 修复「开始覆盖后自动隐藏并切换到游戏」失效：用 `AttachThreadInput` + `BringWindowToTop` 替代 `SetForegroundWindow`。
- 修复设置窗口修改偏好设置后配置窗口未同步：新增 `peregrine:settings-changed` 事件广播。
- 修复配置预览区棋盘格背景错乱：运算符优先级导致格子模式错误。
- 修复 CI 中 `npm ci` 因 `picomatch` 版本不一致而失败的问题。

### 变更

- 发布产物从 NSIS 安装程序（`*-setup.exe`）改回便携 zip 压缩包：下载解压即可运行，无需安装。
- 移除无实际渲染效果的「边框：四边中缝缺口（20%）」选项。
- 暂时隐藏「自定义图片」准心样式（存在已知问题，待后续修复）。

### 优化

- 图标清晰度大幅提升：图标生成脚本改用 8x 超采样抗锯齿，ICO 包含 16/32/48/64/128/256 六档；托盘与窗口标题栏使用 1024×1024 高分辨率 PNG 源图，高 DPI 下清晰锐利。

### 下载

- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets。

---

## [v0.1.2] — 2026-07-08

正式版本。修复 wgpu 崩溃与图标显示问题，优化 UI 样式命名。

### 修复

- 修复设置窗口最小化时 wgpu 视口校验失败导致程序崩溃（`set_viewport` 尺寸为 0）。
- 设置 wgpu 错误处理器，将未捕获错误降级为日志记录而非直接 panic。
- 修复任务栏与窗口标题栏图标不正确：托盘图标改为从 exe 嵌入资源加载。
- 恢复窗口标题栏图标显示，提升像素图尺寸至 256×256。

### 变更

- 「卫生纸」样式显示名改为「矩形」。

### 文档

- 新增「缓解晕 3D」与「推荐配置」页面，扩充项目介绍。

### 下载

- Windows x86 / x86_64 / ARM64 可执行文件见 Release Assets。

---

## [v0.1.1] — 2026-07-07

首个正式版本后的补丁更新。修复 macOS 启动崩溃，Windows 产物改为静态链接 C 运行时，实现下载解压即可运行，无需额外安装 VC++ Redistributable。

### 修复

- macOS 上 wgpu surface 不支持 `Inherit` alpha 模式导致启动 panic，改为按 capabilities 自动选择。

### 构建

- Windows MSVC 三个目标（x86/x64/ARM64）开启 `+crt-static` 静态链接 C 运行时，exe 不再依赖 `VCRUNTIME140.dll` 等外部 DLL。
- Release CI 增加 DLL 依赖验证步骤，确保产物不含 VC 运行时动态依赖。

### 文档

- 新增 VitePress 文档站点与 GitHub Pages 自动部署。
- 完善 README、HELP 与 AGENTS 文档，首页增加立即下载按钮。
- 新增发布流程规范与贡献指南。

### 下载

- Windows x86 / x86_64 / ARM64 可执行文件见 Release Assets。

---

## [v0.1.0] — 2026-07-07

首个正式版本。一个用于缓解 3D 眩晕的桌面辅助贴图工具，在屏幕上方显示半透明视觉锚点，帮助玩家在 3D 游戏中获得固定参照。

### 新增

- Windows 透明覆盖层窗口：置顶、鼠标穿透的 Overlay 窗口，可悬浮于游戏或应用上方。
- 目标窗口跟随：通过下拉列表选择目标窗口，覆盖层可跟随其位置与尺寸。
- 多种准心样式：十字、大十字、四角/六角/八角定位点、中心环、自定义球、随机球、边框框、贴边矩形等。
- 自定义 PNG 贴图：支持加载 PNG 图片作为覆盖层内容。
- 实时设置面板：独立设置窗口，实时调整样式、颜色、透明度、尺寸等参数并即时预览。
- 配置文件热重载：配置 JSON 文件被外部编辑后自动重载生效。
- 多 Profile 支持：为不同场景保存独立配置。
- Windows 平台自动构建与发布：GitHub Actions 自动构建 Windows x86 / x86_64 / ARM64 产物。

### 修复

- Windows 透明度彻底失效：强制 Bgra8Unorm 避免 sRGB gamma 导致颜色键不匹配。
- 颜色键吃黑色准心、覆盖层切换闪烁、窗口标题匹配逻辑。
- HWND 跨线程获取失败、未选窗口时程序崩溃、窗口尺寸恢复。
- 穿透窗口收不到 RedrawRequested 导致 overlay 不渲染。
- 32 位 Windows 下 `SetWindowLongPtrW` / `GetWindowLongPtrW` 类型不匹配。

### 变更

- 架构重构：双窗口架构（独立设置窗口 + 独立 Overlay 窗口）。
- 覆盖层改用 per-pixel alpha 透明方案（softbuffer 像素缓冲区）。
- 目标窗口从输入框改为下拉列表。
- 预览区跟随目标窗口宽高比。
- 协议改为 PolyForm Noncommercial 1.0.0。
- 嵌入 Windows exe 图标。

### 构建

- 仅构建并发布 Windows x86 / x86_64 / ARM64 三个平台。

### 下载

- Windows x86 / x86_64 / ARM64 可执行文件见 Release Assets。

---

[v0.1.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.4
[v0.1.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3
[v0.1.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.2
[v0.1.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1
[v0.1.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0
