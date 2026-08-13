# settings-dev-mode 设计

## Context

- 设置窗口（`SettingsApp.tsx`）现有 5 个 Tab：通用 / 叠加层 / 快捷键 / 更新 / 关于；「关于」Tab（`AboutTab.tsx`）的版本号目前是不可交互的纯文本。
- 「测试上报」按钮现位于「通用」Tab（`GeneralTab.tsx:155`），门禁为 `import.meta.env.DEV && TELEMETRY_DSN_AVAILABLE`；后端 `test_report` command 已存在（`src-tauri/src/telemetry.rs`）。
- 配置窗口（`ConfigApp.tsx`）已有连点版本号 5 次解锁 DeveloperPanel 的彩蛋（localStorage key `peregrine:dev-tab`），面板含 DevTools 开关 / 开发者日志 / config.json 查看 / 重置配置——属错误功能，整体移除。
- Tauri 依赖已启用 `devtools` feature（根 `Cargo.toml`），Tauri 2.1+ 支持窗口创建时通过 `WebviewWindowBuilder::devtools(bool)` 禁用/启用 DevTools（右键「检查」、Ctrl+Shift+I、程序化 `open_devtools()` 同生同灭）。**该选项只在创建时生效，无运行时开关**。
- 配置窗口与设置窗口均由 Rust 侧创建（`create_config_window` / `create_settings_window`），且关闭即销毁、重开即重建（`show_or_recreate_window`）；创建时已有读配置决定 builder 选项的先例（`read_gpu_setting`）。

## Goals / Non-Goals

**Goals**

- 安卓式解锁：设置窗口「关于」Tab 连点版本号 5 次解锁开发者模式，持久化、重启保持。
- 解锁后出现「开发」Tab，仅含「开启 DevTools」「测试上报」两个按钮。
- 未解锁时两个 Webview 窗口的 DevTools 完全不可用；解锁后重开窗口可用。
- 移除配置窗口的 DeveloperPanel 及其彩蛋。
- 开发构建（`import.meta.env.DEV` / `cfg!(debug_assertions)`）下自动解锁，不影响日常调试。

**Non-Goals**

- 不做 DevTools 的运行时切换（无 API；用「重开窗口生效」规避）。
- 不新增后端 command（`test_report` 复用；`open_devtools` 走前端 `getCurrentWebviewWindow().openDevTools()`）。
- 不改动 `actionLog.ts` 及图层编辑处的 `logAction` 埋点调用（移除面板后成为无查看器的静默日志，无副作用）。
- 不改动叠加层（winit 原生窗口，无 Webview）。

## Decisions

### D1：解锁状态存配置文件，而非 localStorage

`AppSettings` 新增 `developer_mode: bool`（`#[serde(default)]`，默认 `false`）。

- **为什么**：DevTools 门在 Rust 侧窗口创建时判定，Rust 读不到 Webview 的 localStorage；配置文件是 Rust 创建窗口时唯一可靠的数据源。复用现有 `updatePreferences` → `save_config` 保存链路，零新通道。
- **备选**：localStorage + 前端重建窗口——设置窗口由 Rust 创建，前端无法可靠重建，否决。
- **兼容性**：`#[serde(default)]` 保证老配置反序列化不受影响（仓库既有惯例）。

### D2：DevTools 门 = `developer_mode || cfg!(debug_assertions)`，创建时设置在两个窗口上

`create_config_window()` / `create_settings_window()` 照搬 `read_gpu_setting` 模式读取快照配置，调用 `.devtools(...)`。

- **生效时序**：解锁写入配置后，当前已打开的窗口 DevTools 仍不可用（创建时定型）；窗口关闭即销毁、重开即重建，故「重开窗口生效」是自然结果，无需额外逻辑。解锁成功提示文案中带「重新打开窗口后 DevTools 可用」。
- **debug 构建恒 true**：`npx tauri dev` 日常调试不受门禁影响。

### D3：「开发」Tab 可见性 = `developer_mode || import.meta.env.DEV`（纯前端判定）

- 前端从 `get_config` 拿到的 `settings.developer_mode` 判定；`import.meta.env.DEV` 时恒显示（自动解锁）。
- TabsList 列数由 `grid-cols-5` 改为按 Tab 数动态（解锁后 6 列）。
- **与 D2 的关系**：D3 管 Tab 与按钮可见（即时生效），D2 管 WebView2 层 DevTools 能力（重开窗口生效）；解锁当次会话里点「开启 DevTools」可能失败属预期，提示文案覆盖。

### D4：「测试上报」搬家 + 门禁改写

从 `GeneralTab.tsx` 删除原块，移入新 `DevTab.tsx`；可见条件改为「开发 Tab 已可见 && `TELEMETRY_DSN_AVAILABLE`」。

- **为什么放 DevTab 而不是留在通用 Tab 加解锁条件**：用户明确要求「开发」Tab 只含这两个按钮，集中放置语义清晰。
- i18n 复用现有 `settings.telemetryTestReport*` 键，不新增。

### D5：i18n 新键统一 `settings.*` 前缀

新增：`sectionDev`（"开发"/"Dev"）、`devUnlockHint`、`devRemaining`、`devUnlocked`、`devOpenDevTools`、`devOpenDevToolsHint`、`devToolsDisabled`、`devOpenDevToolsFailed`。旧 `developer.*` 键块随 DeveloperPanel 整体删除。

- **为什么不复用 `developer.*`**：用户明确要求新键走 `settings.*` 前缀，与 `settings.sectionGeneral` 等现有键风格一致；Tab 名定为「开发」。

### D6：解锁交互参数复用现有彩蛋惯例

连点 5 次、间隔 < 1.5s 超时清零、第 3 击起显示剩余次数、解锁后短暂显示成功提示——与 `ConfigApp.tsx:88-109` 现有实现一致，降低认知成本。

### D7：`add-glitchtip-telemetry` 的 spec 就地更新

`crash-reporting` 能力尚属未归档变更，不进本变更的 delta specs；其「手动测试上报（仅开发者模式）」需求正文与两个 Scenario 直接改写为「开发者模式解锁后可见」语义，任务 10.11 同步勾选/改写。

## Risks / Trade-offs

- [解锁当次会话 DevTools 不可用，用户可能困惑] → 解锁提示与「开发」Tab 内 DevTools 按钮旁均注明「重新打开窗口后生效」。
- [`developer_mode` 进配置文件，用户手改 config.json 也能解锁] → 可接受：这正是「安卓式解锁」的语义，配置文件即用户主权数据；验证逻辑只需保证 bool 类型。
- [删除 DeveloperPanel 同时删除重置配置入口] → 属预期（错误功能移除）；`actionLog.ts` 静默保留，后续若要日志查看器再单独提案。
- [两个窗口 localStorage 中遗留旧 key `peregrine:dev-tab`] → 无害残留，不做迁移清理。

## Migration Plan

1. schema 加字段（serde default）→ 老配置无感。
2. 前端 + Rust 同步上线，无数据迁移。
3. 回滚：还原提交即可，配置里多出的 `developer_mode` 字段会被旧版本反序列化忽略（serde default 容忍未知字段容忍策略依赖现有配置——`AppSettings` 未加 `deny_unknown_fields`，安全）。

## Open Questions

- 无。
