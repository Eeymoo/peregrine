# settings-dev-mode 提案

## Why

当前开发者功能存在两处错位：

1. 「测试上报」按钮门禁是编译期的 `import.meta.env.DEV`，release 构建（包括 dev 通道的 `0.0.0-dev.*` 安装包）里完全无法触发，验证上报链路必须重新跑 `npx tauri dev`，非常不便。
2. 配置窗口里的 DeveloperPanel（日志查看 / config.json 展示 / 重置配置）是早期跑偏的功能，与产品形态不符，属于应移除的错误功能。

同时，当前构建默认开启了 WebView2 的 DevTools（右键「检查」、Ctrl+Shift+I 对普通用户可用），应默认禁用、仅在用户主动解锁开发者模式后开放（安卓式「连点版本号」交互）。

## What Changes

- 设置窗口「关于」Tab 的版本号变为可点击：连续点击 5 次（间隔 < 1.5s，第 3 击起显示剩余次数）解锁开发者模式；解锁状态持久化到配置文件（`AppSettings.developer_mode`），重启后保持。
- 解锁后设置窗口出现第 6 个 Tab「开发」，仅包含两个按钮：**开启 DevTools**、**测试上报**。
- 「测试上报」按钮从「通用」Tab 搬到「开发」Tab，可见条件由 `import.meta.env.DEV` 改为「开发者模式已解锁」（`TELEMETRY_DSN_AVAILABLE` 门禁保留）。**BREAKING（行为）**：release 构建解锁后也可触发测试上报。
- DevTools 默认禁用：创建配置窗口与设置窗口时按 `developer_mode || cfg!(debug_assertions)` 设置 `.devtools(...)`；未解锁时右键「检查」与 Ctrl+Shift+I 不可用，解锁后重开窗口生效（窗口本身关闭即销毁、重开即重建）。
- 开发构建（`npx tauri dev`）下自动视为已解锁：「开发」Tab 直接显示，DevTools 恒可用。
- **移除**配置窗口的连点版本彩蛋与整个 DeveloperPanel（日志 / JSON / 重置配置），同步清理 `developer.*` i18n 键；`actionLog.ts` 及图层编辑处的 `logAction` 调用保留。
- 新增 i18n 键统一使用 `settings.*` 前缀（`sectionDev`、`devUnlockHint`、`devRemaining`、`devUnlocked`、`devOpenDevTools` 等）；测试上报复用现有 `settings.telemetryTestReport*` 键。

## Capabilities

### New Capabilities

- `developer-mode`: 设置窗口的安卓式开发者模式——连点版本号解锁、「开发」Tab（DevTools + 测试上报）、DevTools 默认禁用与解锁后启用、配置持久化、旧 DeveloperPanel 移除。

### Modified Capabilities

- 无（`crash-reporting` 属于未归档变更 `add-glitchtip-telemetry`，其「手动测试上报（仅开发者模式）」需求的可见性措辞将就地更新为「开发者模式解锁后可见」，见 Impact）。

## Impact

- **配置 schema**：`crates/config/src/schema.rs` 的 `AppSettings` 新增 `developer_mode: bool`（`#[serde(default)]`，老配置兼容）；`src/types/config.ts` 同步。
- **窗口创建**：`src-tauri/src/lib.rs` 的 `create_config_window()` / `create_settings_window()` 增加 `.devtools(...)` 门（照搬 `read_gpu_setting` 模式）。
- **前端**：`src/components/settings/AboutTab.tsx`（连点解锁）、`src/SettingsApp.tsx`（动态第 6 Tab）、新增 `src/components/settings/DevTab.tsx`、`src/components/settings/GeneralTab.tsx`（移除旧测试上报块）、`src/ConfigApp.tsx`（移除彩蛋与面板挂载）、删除 `src/components/DeveloperPanel.tsx`。
- **i18n**：`src/i18n/locales/zh-CN.json`、`en.json` 新增 `settings.dev*` 键、删除 `developer.*` 键块。
- **OpenSpec**：`openspec/changes/add-glitchtip-telemetry/specs/crash-reporting/spec.md` 的「手动测试上报（仅开发者模式）」需求措辞就地更新。
- **不改动**：`actionLog.ts` 及 `logAction` 调用点；叠加层（winit 原生窗口，无 Webview）。
