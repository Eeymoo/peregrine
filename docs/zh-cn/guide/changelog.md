# 更新日志

本页记录 Peregrine 全部版本发布。正式版在前，预览（alpha/beta）版见下方 **测试版发布** 章节。发布资产见 [GitHub Releases](https://github.com/Eeymoo/peregrine/releases)。

---

## [v0.2.1] — 2026-08-09

正式版本。引入四层自定义架构（元素 / 物料 / 图层 / 配置）与 Rhai 物料运行时，支持静态多图层渲染；新增多配置管理、匿名遥测与开发者模式；并合并 dev 分支功能（单例模式、Markdown 更新日志、镜像下载修复）。

### 新增

- **四层架构**：单一硬编码 `Crosshair` 配置被完全可组合的系统取代——元素（原子图元）、物料（Rhai 脚本）、图层（带变换的实例）、配置（多图层组合）。12 种旧准心样式全部迁移为内置 `.rhai` 物料。 @Eeymoo
- **图层编辑器**：三栏布局，实时预览 + 图层面板 + 由物料 `schema()` 驱动的动态参数控件；多图层覆盖层渲染与预览 WYSIWYG 一致。 @Eeymoo
- **多配置管理**：在配置窗口新建 / 重命名 / 复制 / 删除 / 切换 Profile；单/多图层模式跨重启记忆。 @Eeymoo
- **配置自动迁移**：旧 `crosshair` 配置在首次加载时自动迁移为 `layers` 格式，原文件备份为 `config.json.legacy.bak`。 @Eeymoo
- **匿名遥测（GlitchTip）**：首次启动授权弹窗、设置页开关、崩溃报告本地落盘并在授权后静默上传、按安装稳定的 `install_id`、严格脱敏（事件不含 IP / 用户名 / 机器名）。可通过 `PEREGRINE_DISABLE_TELEMETRY` 在编译期完全禁用。 @Eeymoo
- **开发者模式**：在「设置 → 关于」连点版本号 5 次解锁「开发」Tab（开启 DevTools、测试上报）；正式构建默认隐藏 DevTools。 @Eeymoo
- **单例模式**：重复启动应用时聚焦已有窗口，不再运行多个实例。 @Eeymoo
- **Markdown 更新日志**：更新检查面板以完整 Markdown 排版渲染发布说明。 @Eeymoo
- **i18n 与 UI 打磨**：设置与配置窗口完成中英双语审查；12 种样式统一字段控件（滑杆 / 数字 / 颜色 / 下拉 / 图片路径），双向同步。 @Eeymoo

### 修复

- 修复单图层编辑连续拖动时滞后一帧（useCallback 闭包陷阱）。 @Eeymoo
- 修复单图层预览落后一次修改：预览改为使用内存态配置渲染，不再等待防抖保存落盘。 @Eeymoo
- 修复单图层模式快捷颜色不立即生效、图层上移/下移方向相反、禁用态样式不一致、删除图层后立即新增偶发 `layer not found`。 @Eeymoo
- 修复启用中国大陆镜像时安装包下载链接未套用镜像前缀。 @Eeymoo

### 变更

- **动态物料输入已软禁用**（`MATERIAL_DYNAMIC_INPUT_ENABLED = false`）：静态多图层渲染完全启用，但时间 / 鼠标 / 键盘驱动的物料冻结渲染并从物料选择器隐藏，待后续重新启用。 @Eeymoo
- `custom_image` 物料因渲染问题暂时从选择器隐藏。 @Eeymoo

### 构建

- 遥测 DSN 由 CI 按通道注入（预发布进 TEST 项目、正式版进正式项目）；DSN 格式非法时构建直接失败。 @Eeymoo

### 下载

- Windows x86 / x86_64 / ARM64 NSIS 安装包（支持自动更新）见 Release Assets。
- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets。

---

## [v0.1.15] — 2026-07-18

正式版本。新增各准心样式独立的默认参数与一键恢复默认颜色；修复窗口模式切换与拖拽实时显示问题；文档重构为英文优先并完善简体中文对照版。

### 新增

- **各准心样式独立默认参数**：每种内置准心样式不再共用一套全局默认值，而是提供开箱即用的独立参数（尺寸、粗细、偏移、不透明度等），切换样式后不会出现准心看不见或无法使用的情况。 (#8) @Eeymoo
- **快捷颜色一键恢复默认**：快捷颜色预设标题旁新增「恢复默认」按钮，一键恢复 5 种默认颜色。 (#7) @Eeymoo

### 修复

- 修复覆盖层运行时切换窗口模式被错误阻止：覆盖层运行中，托盘菜单、后端命令及前端界面均已禁用窗口模式切换。 (#9) @Eeymoo
- 修复「拖拽实时显示」开启后拖拽过程中准心位置不实时更新：跟随线程在重定位覆盖层后立即请求重绘。 (#14) @Eeymoo

### 文档

- 文档站点重构为英文优先，并新增完整的简体中文对照版（含语言切换器及双语 README、HELP、贡献指南、更新日志）。 @Eeymoo

### 构建

- 新增 PR 快照构建工作流与 opencode 触发工作流以自动化 CI。 (#15) @Eeymoo

### 下载

- Windows x86 / x86_64 / ARM64 NSIS 安装包（支持自动更新）见 Release Assets。
- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets。

---

## [v0.1.9] — 2026-07-13

正式版本。新增 SVG 矢量渲染后端、网格准心样式、全局快捷键与快捷颜色预设，覆盖层抗锯齿默认开启。

### 新增

- **SVG 渲染后端**：覆盖层新增可选 SVG 渲染后端（基于 resvg + tiny-skia），在「设置 → 覆盖层 → 渲染后端」中切换。SVG 模式抗锯齿质量更高；CPU 模式（默认）零额外依赖、更轻量。两套方案并行，SVG 光栅化失败时自动回退到 CPU 渲染。 @Eeymoo
- **网格准心样式**：新增 `Grid` 准心样式，可调整网格行列数、线宽与颜色，为需要规则参照的用户提供更多选择。 @Eeymoo
- **全局快捷键体系**：支持为「开始/停止覆盖」等功能绑定全局热键，可在「设置 → 快捷键」中配置。 @Eeymoo
- **快捷颜色预设**：颜色选择器新增常用预设，一键切换准心颜色。 @Eeymoo
- **覆盖层抗锯齿**：CPU 渲染模式新增抗锯齿开关，默认开启，边缘更平滑；需要最低延迟时可关闭。 @Eeymoo
- **滚动条样式优化**：自定义滚动条样式，默认透明、悬停淡入，6px 宽圆角，与整体界面风格统一。 @Eeymoo

### 修复

- 移除拖拽实时显示在某些场景下被强制禁用的限制，交互更连贯。 @Eeymoo

### 下载

- Windows x86 / x86_64 / ARM64 NSIS 安装包（支持自动更新）见 Release Assets。
- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets。

---

## [v0.1.7] — 2026-07-12

正式版本。移除 Gitee 镜像改用 gh-proxy 加速代理；新增 GitHub Releases 自动更新、GPU 硬件加速开关与窗口模式优化。

### 新增

- **GitHub Releases 自动更新**：内置更新检查与下载安装，支持正式版（stable）与尝鲜版（prerelease）双通道。
- **中国大陆加速代理**：通过 gh-proxy 加速 GitHub 下载，简体中文用户默认开启；可在设置中选择加速站（v4 / v6 / cdn / 自定义）。
- **GPU 硬件加速开关**：设置中可开关 WebView2 GPU 硬件加速，关闭可降低约 60MB 内存占用。

### 修复

- 修复设置窗口最小化后闪退。

### 重构

- 将默认样式「卫生纸」重命名为「贴边矩形」并同步文档术语规范。

### 构建

- 修复 CI 构建失败（javascriptcoregtk 依赖缺失）。
- 文档部署仅在正式版 Release 时触发。

---

## [v0.1.5] — 2026-07-11

正式版本。新增 NSIS 安装程序与内置自动更新功能，支持正式版（stable）与尝鲜版（prerelease）双通道检测。

### 新增

- **NSIS 安装程序**：提供 `setup.exe` 安装包，安装后支持自动更新；便携 zip 仍保留。
- **内置自动更新**：设置页「检查更新」按钮，自动检测并下载安装新版本；下载进度条实时显示。
- **启动自动检测**：打开配置页面时延迟 3 秒自动检测新版本，发现后弹窗提示。
- **双通道更新**：正式版（stable）走 `releases/latest/download/stable.json`，尝鲜版（prerelease）走对应 tag 的 `prerelease.json`，用户可在设置中切换更新通道。
- **关于页面发行者信息**：关于对话框显示发行者（Eeymoo）、许可（MIT）、仓库链接与动态版本号。

### 修复

- 修复点击更新后弹窗未清除导致重复检测。
- 修复 `PreferencesPatch` 缺少 `update_channel` 字段导致 CI 编译失败。
- 修复 CI 签名缺失时未报错退出的问题。
- 移除设置页面冗余提示文案。

### 构建

- CI 启用 `createUpdaterArtifacts`，自动为 NSIS 安装包生成 `.sig` 签名文件。
- CI 清理调试日志，精简构建步骤。

### 下载

- Windows x86 / x86_64 / ARM64 NSIS 安装包（支持自动更新）见 Release Assets。
- Windows x86 / x86_64 / ARM64 便携 zip 见 Release Assets。

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

[v0.1.15]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.15
[v0.1.9]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.9
[v0.1.5]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.5
[v0.1.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.4
[v0.1.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3
[v0.1.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.2
[v0.1.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1
[v0.1.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0

---

# 测试版发布

## [v0.2.0-alpha.0] — 2026-07-18

**四层架构**首个预览版。这是把单一硬编码 `Crosshair` 配置替换为完全可组合系统的重大重构。

### 新增

- **四层架构**：Element（原子图元）→ Material（Rhai 脚本）→ Layer（带变换的实例）→ Profile（多图层组合）。
- **Rhai 物料运行时**（`crates/material`）：基于 `rhai` crate 的 CPU 安全嵌入式脚本。脚本需导出 `defaults()`、`schema()`、`is_dynamic()`、`build(params, screen)`。
- **物料动态输入**：物料脚本可调用 `time_ms()`、`mouse_pos()`、`key_down(code)`、`rand()`。Windows 实现通过 `GetCursorPos` / `GetAsyncKeyState` 的 `poll_dynamic_context`。
- **12 个内置物料**：所有旧 `CrosshairStyle` 变体已迁移到 `.rhai` 脚本（`cross`、`large_cross`、`edge_rect`、`corner_dots`、`ring`、`custom_orb`、`random_orb`、`border_frame`、`edge_arrows`、`grid`、`image`）。
- **图层组合**：可堆叠多个图层；每个图层有独立的物料、参数、颜色、不透明度、变换（位移/缩放/旋转）、可见性、锁定状态。
- **配置迁移**：首次加载时，含 `crosshair` 字段的旧 `config.json` 自动迁移到新 `layers` 格式。原文件备份为 `config.json.legacy.bak`。
- **Tauri IPC 命令**：`build_shapes_ipc`、`list_materials`、`add_layer`、`remove_layer`、`move_layer`、`duplicate_layer`、`update_layer`、`list_layers`。
- **前端图层编辑器**（`LayersEditor`）：三栏布局（实时预览 / 图层面板 / 物料 `schema()` 驱动的动态参数控件）。
- **匿名遥测（部分实现）**：接入 GlitchTip（Sentry 协议），含首次启动授权弹窗、设置开关、待上报存储与一次性授权上传；CI 按通道注入 DSN（TEST / 正式项目分离）。

### 变更

- `Profile` schema 双重支持旧格式 `crosshair: Option<Crosshair>` 与新格式 `layers: Vec<Layer>`。`load_or_create_default` 自动迁移旧配置。
- `Shape` 现为 `Element` 的类型别名（9 个变体：Rect、Circle、CircleStroke、DashedCircle、Triangle、Polygon、Line、Text、Image）。
- `Preview` 组件改为通过 IPC `build_shapes_ipc` 拉取图元列表，不再在 TypeScript 中计算几何（删除 `src/lib/shapes.ts`）。
- `OverlayRenderer` 采用双路径渲染：新格式（图层 + 物料求值）优先；旧 Crosshair 路径保留作为 fallback。

### 构建

- 新 workspace 成员 `crates/material`（依赖 `peregrine_config` + `rhai` 1.25 + `ahash` 0.8）。
- `SimpleRng` 移到 `peregrine_config::rng`，物料运行时与旧 shapes 跨 crate 共享。
- CI 扩展为对全部 3 个 crate（`config`、`material`、`peregrine`）执行 `cargo clippy` 和 `cargo test`。

### 已知限制

- `src-tauri`（Tauri 命令）在非 Windows 主机缺少 webkit2gtk 系统依赖无法编译；仅通过 Windows CI 验证。
- `ConfigApp.tsx` 中旧 Crosshair UI 默认保留；点击「切换到图层编辑器」进入新 UI。
- 旧版快捷颜色热键操作的是 `crosshair.color`；新图层版等价物尚未接入。

---

## [v0.1.15-alpha.0] — 2026-07-17

### 新增

- **快捷颜色重置**：在快捷颜色标题右侧新增「重置」按钮，一键恢复 5 个默认色值。[#3](https://github.com/Eeymoo/peregrine/issues/3)
- **各样式开箱即用默认参数**：内置准心样式现各自提供合理的默认参数（尺寸、厚度、偏移、透明度等），切换样式后即可直接看到效果，不再出现不可见或不可用的情况。前端切换样式时重置为对应样式默认值，保证预览与覆盖层 WYSIWYG。[#4](https://github.com/Eeymoo/peregrine/issues/4)

### 修复

- 修复「拖拽时实时显示」开启后拖拽过程中准心位置不实时更新的问题：follower 线程在移动 overlay 窗口后未通知渲染线程刷新，导致准心位置静止、仅松开鼠标后才跳转。现已在每次调整 overlay 位置后直接调用 `window.request_redraw()` 请求重绘。[#5](https://github.com/Eeymoo/peregrine/issues/5)
- 修复覆盖层活跃时切换窗口模式导致托盘勾选状态不同步的问题：Tauri v2 的 `CheckMenuItem` 在菜单事件触发前已自动切换勾选状态，拒绝切换时 checkbox 会与实际配置不一致。现已在 guard 阻断时回退勾选状态。覆盖层运行时切换窗口模式（全屏/窗口）现已在托盘菜单、后端 `update_preferences` 命令、前端（禁用复选框并提示）三处统一阻断。[#2](https://github.com/Eeymoo/peregrine/issues/2)

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
