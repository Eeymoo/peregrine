# material-static-rendering 提案

## Why

`disable-material-runtime` 将**整个物料运行时**软关闭：overlay 忽略 `Profile.layers`、图层编辑不影响渲染与预览、UI 回退旧版准星路径。实机走查后确认该范围**过大**：

- 当年反复出问题的只是**动态链路**（动态刷新调度、时钟等动态文本渲染），静态多图层渲染本身稳定，属于被误伤。
- 物料 / 多图层功能**从未随正式版发布**，无存量用户配置需要兼容，恢复静态渲染没有迁移负担。
- 用户明确决策：**应该多图层渲染可用，只是动态物料相关内容不可设置**（2026-08-03 实机走查结论）。

因此将软关闭范围从「整个物料运行时」收窄为「仅动态物料能力」。

## What Changes

- **静态渲染恢复**：overlay 与预览恢复 layers 多图层渲染（`MATERIAL_RUNTIME_ENABLED` 翻回 `true`），图层编辑重新影响画面（WYSIWYG）。
- **新增动态独立开关**：`MATERIAL_DYNAMIC_INPUT_ENABLED = false`（Rust `crates/peregrine/src/lib.rs` + TS `src/lib/feature.ts` 成对），单独控制动态输入与动态物料，与静态渲染开关联耦。
- **动态输入保持关闭**：不轮询时间 / 鼠标 / 键盘；`DynamicContext` 使用固定快照（`time_ms` 冻结、鼠标居中、按键恒 false、种子固定），保证静态渲染确定性与预览/overlay 一致。
- **动态重绘调度保持移除**：overlay 事件循环维持纯事件驱动（配置变更 / 窗口跟随 / RedrawRequested），不为动态物料定期唤醒。
- **UI 过滤动态物料**：物料选择器隐藏 `is_dynamic = true` 的物料（当前内置物料中仅 `time.rhai`）；动态徽章等相关设置项随开关隐藏。存量引用动态物料的图层按固定快照静态渲染（冻结），不崩溃不迁移。
- **兼容判定恢复原语义**：随主开关翻回，`effectiveCompatible` 恢复真实判定——不兼容 profile 禁用单图层编辑并引导至多图层模式（与 `multi-profile-config` 原始设计一致）。
- **规格关系**：本 change 修订 `disable-material-runtime` 的 D1–D3（软关闭范围收窄）；其 D4 已被 2026-08-03 二次修订（入口保留、切换自由）取代，本 change 维持该状态。

## Capabilities

### New Capabilities

（无新能力；本变更是对既有能力的范围修订。）

### Modified Capabilities

- `material-runtime`：静态物料求值恢复为活跃渲染路径；新增「静态/动态分开关」行为。
- `material-dynamic-input`：动态输入整体停用（独立开关门控轮询与 UI 可见性），动态物料渲染冻结为固定快照。

## Impact

- **代码**：
  - `crates/peregrine/src/lib.rs`（新增 `MATERIAL_DYNAMIC_INPUT_ENABLED` 常量）
  - `crates/peregrine/src/overlay_renderer.rs`（`use_new_format` 恢复；动态上下文轮询改固定快照门控）
  - `src-tauri/src/lib.rs`（预览 IPC 恢复 layers 求值 + 固定快照）
  - `src-tauri/src/overlay.rs`（保持事件驱动，原则上不动）
  - `src/lib/feature.ts`（新增同名 TS 常量）
  - `src/components/LayerPanel.tsx`（物料选择器过滤 `is_dynamic`）
  - `src/ConfigApp.tsx` / `src/hooks/useConfigAppState.ts`（随主开关翻回复查门控注释与原语义恢复）
- **文档**：`AGENTS.md` 软关闭描述更新为「仅动态物料停用」。
- **依赖**：无变更。
- **用户配置**：无迁移——功能未发布，无存量 layers 配置；既有纯 `crosshair` 配置行为不变。
- **挂起变更**：`overlay-dynamic-text-fixes` / `material-e2e-validation` / `material-docs-examples` 维持挂起，待将来 `MATERIAL_DYNAMIC_INPUT_ENABLED` 翻回时继续。
