## Why

v0.2.1 稳定版发布后，多图层编辑链路暴露出多个严重缺陷：**用户演示过程中实际发生配置丢失**（多图层被前端旧快照覆盖回单图层）、AutoSwitchDialog 等全局对话框在多图层模式下根本不渲染、所有图层操作失败时用户无任何错误提示、部分物料参数是 dead UI、网格物料 grid_size 实际不生效、透明度显示错误、slider 上限在 4K 屏不够用。这些问题各自独立但都在多图层链路集中爆发，必须一次性系统修复以恢复可用性。

本次变更涵盖 #27~#35 共 9 个 issue，目的是把多图层链路从「能演示」提升到「可日常使用」。

## 目标

- **根治多图层配置丢失**：消除任何通过 `saveConfig` 全量覆盖后端的路径，所有 profile 字段变更走 patch API
- **统一错误反馈链路**：后端结构化错误 + 前端统一 toast 显示，用户能看到「为什么没保存」
- **修复多图层模式 UI 缺陷**：AutoSwitchDialog / UpdateDialog / UpdateProgress 移到全局挂载点
- **修复物料脚本逻辑 bug**：grid 的 ceil 取整、4 个 dead parameter
- **修复 UI 显示 bug**：透明度 0-1 应显示为 0-100%
- **扩充 slider 上限**：按 1920 / 500 / 50 分级，覆盖 4K 屏
- **README 嵌入演示视频**

## 非目标

- **不做 slider max 自适应屏幕宽度**（如 `screen_w * 0.5`）：涉及 schema 协议大改（`schema()` 签名、缓存策略、IPC 协议、TS 类型），复杂度中高，单独开 issue 跟进
- **不重构物料求值缓存**：当前每次 evaluate 都重新跑 Rhai，虽然注释里说"version=0 永久缓存"但实际无缓存逻辑，本次不处理
- **不实现 random_orb.mode 的 lock_on_start 行为**：AGENTS.md 已记录为待办，本次仅按 dead parameter 决策处理（实现 or 隐藏）
- **不做 SaveConfig 完整重构**：保留 `save_config` 命令用于单图层模式，仅消除多图层路径下的全量调用
- **不做错误码体系 (PGR-XXXX) 扩展**：复用现有 telemetry 上报，不新增 report code

## What Changes

### 配置丢失修复（issue #34，🔴 严重）
- **新增后端命令** `update_profile_field(profile_name, field, value)`：patch 式更新 profile 单字段（target_window / settings_hotkey 等），避免全量替换
- **新增后端命令** `update_target_window(profile_name, target_window)`：或合并入上一条
- `LayersEditor.updateTargetWindow` 改用 patch API，不再调 `saveConfig`
- `peregrine:layers-changed` 事件监听器除调 `refresh()` 外，同时调 `getConfig()` 同步整个 config 到 `setConfig`（兜底 race）

### 错误处理统一（issue #27 + #31）
- **后端**：所有图层/profile 命令的错误返回类型从 `Result<T, String>` 改为 `Result<T, serde_json::Value>`（结构化 `{code, message}`），或自定义 `AppError` 枚举 + `#[serde]`
- **前端**：`api.ts` 新增统一 `invoke` 包装，IPC reject 时 `throw new Error(message)` 并 toast 显示
- **前端**：`LayerPanel` / `LayerEditors` / `MaterialParamControls` 全部按 `ProfileManager` 模式加 try/catch + UI 错误提示
- **前端**：移除所有 `.catch(console.error)` 静默吞错点（保留 `.catch(() => {})` 用于真正可忽略的场景如 setTitle）

### 多图层全局对话框层（issue #28）
- `ConfigApp.tsx` 把 `<AutoSwitchDialog>` / `<UpdateDialog>` / `<UpdateProgress>` 三个组件从单图层模式 return 内移到 `layersMode return` 之前的「全局对话框层」，保证两种模式下都挂载

### 物料脚本修复（issue #29 + #30）
- **grid.rhai**：`ceil` 改 `floor`，用 `cell * cols` 计算 `total_w`，修复 grid_size 实际不生效 + 超屏
- **border_frame.rhai inset**：从 build 实现贴边/跨边渲染（推荐），或从 schema 移除该控件
- **edge_rect.rhai corner_radius**：shape 输出 `corner_radius` 字段 + 渲染器支持圆角矩形（推荐），或从 schema 移除
- **random_orb.rhai center_deviation**：在 build 中实现中心规避逻辑（推荐），或从 schema 移除
- **random_orb.rhai mode**：保留 schema 但 UI 加「coming soon」禁用标记（按 AGENTS.md 待办处理）

### UI 显示修复（issue #32）
- `SliderField` 新增 `format?: (v: number) => string` 可选参数
- `ConfigApp.tsx:257` 单图层 opacity 改为 `Math.round(ch.opacity * 100) + "%"`
- `LayerEditors.tsx:50` 多图层 opacity SliderField 传 `format={(v) => Math.round(v * 100) + "%"}`

### Slider max 扩充（issue #33）
- 按分级表统一调整所有内置物料的 schema `max`：
  - 距离/偏移/尺寸类（offset / margin / distance / tail / size / grid_size / offset_x/y）→ **1920**
  - 半径类（radius / radius_min/max）→ **500**
  - 粗细类（thickness）→ **50**
  - 间隙类（gap）→ **200**
  - 缩放类（scale）→ **50**
  - 字体类（font_size）→ **400**
  - 数量/比例类（count / *_pct）→ 不变

### README 视频（issue #35）
- `README.md` 和 `README.zh-cn.md` 在「Quick Start / 快速开始」之前嵌入 bilibili iframe，`<div max-width:100%>` 包裹

## Capabilities

### New Capabilities
- `profile-patch-api`: profile 字段级 patch 更新命令（`update_profile_field` 等），替代多图层路径下的全量 `saveConfig`
- `ipc-error-contract`: Tauri IPC 结构化错误协议（`{code, message}`），前端统一包装 + toast 显示

### Modified Capabilities
- `layer-composition`: 修复 `evaluate_layer` 后的 build 流程不丢图层；修复 grid 物料的 cols 取整；修复 4 个 dead parameter；调整 schema max 上限
- `profile-management`: `LayersEditor.updateTargetWindow` 改走 patch API；`peregrine:layers-changed` 事件同步整个 config
- `config-ui-polish`: AutoSwitchDialog / UpdateDialog / UpdateProgress 全局挂载；透明度显示改 0-100%；SliderField 新增 `format` 参数；图层操作统一错误提示
- `widget-fields`: SliderField 支持 `format` 回调；slider max 按分级表调整

## Impact

### 受影响代码
- **后端 Rust**：
  - `src-tauri/src/lib.rs`：新增 `update_profile_field` 命令；所有图层/profile 命令的错误类型改造；`persist_and_broadcast` / `save_config` 错误类型
  - `crates/config/src/schema.rs`：可能新增 profile 字段级 patch 方法
- **物料脚本**：`crates/material/builtin/*.rhai` 11 个文件（grid / border_frame / edge_rect / random_orb / cross / corner_dots / custom_orb / edge_arrows / image / large_cross / ring）
- **前端 TypeScript**：
  - `src/lib/api.ts`：新增 `updateProfileField` 包装、统一 `invoke` 错误包装
  - `src/types/config.ts`：可能新增错误类型定义
  - `src/hooks/useConfigSave.ts`、`src/hooks/useOverlayActions.ts`：移除 `.catch(console.error)` 静默
  - `src/components/ConfigApp.tsx`：全局对话框层重构、透明度显示修复
  - `src/components/LayersEditor.tsx`：`updateTargetWindow` 改走 patch API、layers-changed 同步整个 config、错误提示
  - `src/components/LayerPanel.tsx`、`src/components/LayerEditors.tsx`：所有图层操作加 try/catch + 错误提示
  - `src/components/fields/SliderField.tsx`：新增 `format` 参数
- **文档**：`README.md`、`README.zh-cn.md` 嵌入视频

### 不受影响
- overlay 渲染核心（`overlay_renderer.rs` 的 softbuffer 像素光栅化路径）
- 配置文件存储格式（JSON schema 不变）
- Win32 平台层（透明 / 置顶 / 点击穿透 / 窗口跟随）
- 遥测模块（不新增 report code）
- CI/CD 工作流

### 风险
- **错误类型改造是 BREAKING 变更**：所有图层/profile 命令的 IPC 协议都变，需要前后端同步落地，不能拆 PR 分批 merge
- **dead parameter 决策需谨慎**：实现 vs 隐藏各有取舍，建议优先实现（保留功能），实在复杂的（如 random_orb.mode）才隐藏
- **grid.rhai 算法变更可能影响现有用户配置**：已保存的 grid_size 值在新算法下渲染结果会变（更准确），但视觉上可能有差异
