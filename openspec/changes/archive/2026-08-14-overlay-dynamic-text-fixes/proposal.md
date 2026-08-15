# Overlay 动态物料刷新修复与文本加粗

> **取代说明**：本 change 已由 `restore-dynamic-material`（2026-08-14）收编并取代——与本 change 范围重叠的未竟事项已并入该 change 的 What Changes，其余宣告放弃。随该 change 归档，详见 `openspec/changes/restore-dynamic-material/proposal.md` 头部取代关系说明。

## Why

Windows 实机走查（`docs/manual-test-checklist.md` B4 区）发现两个 overlay 渲染问题：

1. **动态物料不自动刷新**：时钟物料不跳动，鼠标跟随 / 键盘响应物料无反应，只有拖拽窗口触发重绘后才更新一帧。根因已定位：`src-tauri/src/overlay.rs::about_to_wait` 的新格式（layers）分支写死返回 `None`，动态物料被当成静态物料，`ControlFlow::Wait` 永久挂起。
2. **文本字重过细**：`Element::Text` 没有字重字段，SVG 后端未输出 `font-weight`，时钟等文本物料在 overlay 上过于纤细，游戏画面背景上可读性差。

## What Changes

### 修复：动态物料持续重绘

- `overlay.rs::about_to_wait` 的新格式分支：遍历 active profile 的 layers，通过 `MaterialRegistry` 查询任一图层物料 `is_dynamic == true` 即按 60FPS 持续重绘（与旧格式 RandomOrb 路径行为一致）。
- 动态性判定结果做帧级缓存（仅在 layers 变更或物料热重载后重新查询），避免每帧遍历开销。
- 修复后时钟物料每秒更新、鼠标跟随物料延迟 < 50ms、键盘响应物料即时反馈（对齐 `material-e2e-validation` §6 验收标准）。

### 新增：文本字重支持

- `crates/config/src/schema.rs`：`Element::Text` 新增 `font_weight: Option<u16>` 字段（`#[serde(default)]`，取值 100–900，`None` 表示常规 400），旧配置序列化兼容。
- `crates/peregrine/src/svg_renderer.rs`：`<text>` 元素输出 `font-weight` 属性。
- `crates/material/builtin/time.rhai`：schema 新增 `bold` 参数（widget: `toggle`，label「加粗」，默认 `false`），build 时将 `font_weight` 写入 Text 图元。
- `src/components/Preview.tsx`：Canvas 预览按 `font_weight` 设置 `bold` 字体串，保持 WYSIWYG。
- `src/types/config.ts`：`Element` text 变体类型同步新增 `fontWeight` 字段。

## Capabilities

### New Capabilities

- `overlay-dynamic-rendering`：overlay 对动态物料的持续重绘行为（is_dynamic 判定、帧调度、动态性缓存失效条件）。

### Modified Capabilities

- 无（文本字重作为 `overlay-dynamic-rendering` spec 的一部分一并描述；`openspec/specs/` 下现有的 `dev-merge-integration` 不涉及本变更的需求级修改）。

## Impact

- `src-tauri/src/overlay.rs`（about_to_wait 动态判定 + 缓存失效）
- `crates/config/src/schema.rs`（`Element::Text` 新字段 + 校验 + 测试）
- `crates/peregrine/src/svg_renderer.rs`（font-weight 输出）
- `crates/material/builtin/time.rhai`（bold 参数）
- `src/types/config.ts`、`src/components/Preview.tsx`（前端类型与预览）
- 验收：`docs/manual-test-checklist.md` B4 区全部条目 + 字重目测项
