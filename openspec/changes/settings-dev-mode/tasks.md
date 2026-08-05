# settings-dev-mode 任务

## 1. 配置 schema

- [x] 1.1 `crates/config/src/schema.rs`：`AppSettings` 新增 `developer_mode: bool`，`#[serde(default)]`（默认 false）+ 中文文档注释
- [x] 1.2 `schema.rs` 测试：老配置（无 `developer_mode` 字段）反序列化为 false；serde round-trip
- [x] 1.3 `src/types/config.ts`：`AppSettings` 增加 `developer_mode?: boolean`
- [x] 1.4 运行 `cargo test -p peregrine_config` 通过

## 2. Rust 窗口 DevTools 门

- [x] 2.1 `src-tauri/src/lib.rs`：新增读配置快照取 `developer_mode` 的辅助函数（照搬 `read_gpu_setting` 模式）
- [x] 2.2 `create_config_window()` 与 `create_settings_window()`：`webview_builder.devtools(developer_mode || cfg!(debug_assertions))`

## 3. i18n

- [x] 3.1 `zh-CN.json` / `en.json`：新增 `settings.sectionDev`（"开发"/"Dev"）、`devUnlockHint`、`devRemaining`、`devUnlocked`（含「重新打开窗口后 DevTools 可用」）、`devOpenDevTools`、`devOpenDevToolsHint`、`devToolsDisabled`、`devOpenDevToolsFailed`
- [x] 3.2 删除两语言文件中的整个 `developer.*` 键块

## 4. 设置窗口解锁交互 + 「开发」Tab

- [x] 4.1 `src/components/settings/AboutTab.tsx`：版本号可点击，连点 5 次（间隔 < 1.5s 超时清零，第 3 击起剩余次数提示），解锁后 `updatePreferences({ developer_mode: true })` + 成功提示
- [x] 4.2 `src/SettingsApp.tsx`：解锁状态（`config.settings.developer_mode || import.meta.env.DEV`）控制第 6 个 Tab「开发」；TabsList 列数动态化（5/6）
- [x] 4.3 新增 `src/components/settings/DevTab.tsx`：仅「开启 DevTools」（照搬 DeveloperPanel 的 `getCurrentWebviewWindow().openDevTools()` 逻辑与失败提示）与「测试上报」两个区块
- [x] 4.4 `src/components/settings/GeneralTab.tsx`：删除 `import.meta.env.DEV && TELEMETRY_DSN_AVAILABLE` 门禁的旧测试上报块（按钮与 `testReportState` 逻辑迁入 DevTab，`TELEMETRY_DSN_AVAILABLE` 门禁保留）

## 5. 移除配置窗口 DeveloperPanel

- [x] 5.1 `src/ConfigApp.tsx`：移除连点版本逻辑、`devTabUnlocked` 状态、DeveloperPanel 挂载与导入；底部版本号恢复为纯文本
- [x] 5.2 删除 `src/components/DeveloperPanel.tsx`
- [x] 5.3 确认 `actionLog.ts` 及 `logAction` 调用点（`LayerPanel.tsx` / `LayerEditors.tsx`）保持不动、无悬空引用

## 6. OpenSpec 同步

- [x] 6.1 `openspec/changes/add-glitchtip-telemetry/specs/crash-reporting/spec.md`：「手动测试上报（仅开发者模式）」需求正文与两个 Scenario 改写为「开发者模式解锁后可见」语义
- [x] 6.2 `openspec/changes/add-glitchtip-telemetry/tasks.md` 相关任务描述同步

## 7. 验证

- [x] 7.1 `npm run build`（含 TypeScript 检查）通过
- [x] 7.2 `cargo build` 通过；`cargo fmt --check` 与 `cargo clippy -- -D warnings` 通过
- [ ] 7.3 `npx tauri dev` 手测：「开发」Tab 直接可见；DevTools 可用；测试上报可触发
- [ ] 7.4 release 构建手测：默认右键无「检查」→ 关于 Tab 连点 5 次解锁 → 出现「开发」Tab → 重开窗口后右键「检查」与「开启 DevTools」可用 → 测试上报在 GlitchTip 产生 issue
- [ ] 7.5 配置窗口底部版本号点击无任何反应；无 DeveloperPanel 残留
