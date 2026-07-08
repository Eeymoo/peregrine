# 更新日志

仅记录正式版本发布。测试版 / 预览版本见 **[CHANGELOG_ALPHA.md](CHANGELOG_ALPHA.md)**。

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
- 多种准心样式：十字、大十字、四角/六角/八角定位点、中心环、自定义球、随机球、边框框、卫生纸样式等。
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

[v0.1.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.2
[v0.1.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1
[v0.1.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0
