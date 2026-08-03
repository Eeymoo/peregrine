# material-static-rendering 设计

## Context

`disable-material-runtime` 用单一开关 `MATERIAL_RUNTIME_ENABLED = false` 关闭了整个物料运行时：渲染回退旧 `Crosshair` 路径、动态输入停采、动态重绘调度移除。实机走查确认关闭范围过大——出 bug 的只是动态链路，静态多图层渲染稳定且从未正式发布（无存量配置）。本设计将软关闭范围收窄为「仅动态物料能力」，恢复静态多图层渲染。

## Goals / Non-Goals

- Goals：overlay 与预览恢复 layers 静态渲染（WYSIWYG）；动态输入/动态物料独立开关关闭；UI 不可设置动态物料。
- Non-Goals：不恢复动态重绘调度（overlay.rs 保持事件驱动）；不处理存量动态物料配置的迁移（无存量）；不改动挂起的三个物料 change。

## Decisions

### D1：双开关拆分——静态渲染与动态输入各自独立门控

- `MATERIAL_RUNTIME_ENABLED`（已有，Rust + TS 成对）翻回 `true`：恢复物料运行时静态渲染路径。
- 新增 `MATERIAL_DYNAMIC_INPUT_ENABLED = false`（Rust `crates/peregrine/src/lib.rs` + TS `src/lib/feature.ts` 成对，中文注释说明用途与恢复方式）：单独门控动态输入轮询、动态物料 UI 可见性。
- 理由：两个维度生命周期不同——静态渲染现在恢复；动态链路要等 `overlay-dynamic-text-fixes` 等挂起 change 收尾后才适合翻回。

### D2：动态输入关闭时使用 `DynamicContext::static_context()`

`overlay_renderer.rs` 两处（约 162、213 行）与预览 IPC 的上下文选择改为：

```rust
let ctx = if crate::MATERIAL_DYNAMIC_INPUT_ENABLED {
    crate::platform::poll_dynamic_context(logical_w, logical_h)
} else {
    peregrine_material::DynamicContext::static_context()
};
```

- `static_context()` 的 `version = 0`：静态物料求值结果永久缓存命中，避免每帧重复求值。
- 动态物料（如时钟）渲染冻结在默认值，不崩溃；存量引用无需迁移（功能未发布，无存量）。
- 预览与 overlay 使用同一策略，保证 WYSIWYG。

### D3：UI 过滤动态物料

- `LayerPanel.tsx` 物料选择器：`MATERIAL_DYNAMIC_INPUT_ENABLED` 为 false 时过滤掉 `is_dynamic = true` 的物料（当前内置仅 `time.rhai`），动态徽章不再出现（无动态物料可选）。
- 动态输入相关设置项（如有）随同一开关隐藏。
- 理由：用户决策「动态物料相关内容不可设置」；过滤而非禁用展示，避免用户选了之后发现不动的困惑。

### D4：重绘调度保持事件驱动，不恢复动态唤醒

`src-tauri/src/overlay.rs` 原则上不改动：重绘仍由配置热重载、窗口跟随、`RedrawRequested` 触发。动态重绘调度（60FPS 唤醒）属于动态链路，待 `MATERIAL_DYNAMIC_INPUT_ENABLED` 翻回时随 `overlay-dynamic-text-fixes` 一并恢复。

### D5：前端语义随主开关翻回，逐项复查

`MATERIAL_RUNTIME_ENABLED` 翻回 `true` 后，以下既有门控自动恢复原语义，需逐项复查注释：

- `ConfigApp.tsx` `effectiveCompatible`：恢复真实 `isLegacyCompatible` 判定——不兼容 profile 禁用单图层编辑（避免改坏多图层配置），提示区提供切换入口。
- `useConfigAppState.ts` 加载逻辑：恢复「恢复为单图层 + active profile 不兼容 → 强制切多图层并写回持久化值」。
- 模式持久化（localStorage 恢复）与切换入口保留的决策不变。

### D6：与 disable-material-runtime 的规格关系

本 change 修订其 D1–D3（渲染器软关闭 / 动态调度移除 / 配置降级的范围收窄）；其 D4 已被 2026-08-03 二次修订（入口保留、切换自由）取代并维持。`AGENTS.md` 的软关闭描述同步更新为「仅动态物料停用」。

## Risks / Trade-offs

- [当年文本图元渲染缺陷随静态渲染回归] → `overlay-dynamic-text-fixes` 的 `font_weight` 修复已在代码中；tasks 中安排 12 种内置物料的逐样式渲染验证。
- [`static_context` 下动态物料冻结显示（如时钟 00:00）被误认为 bug] → UI 已过滤动态物料不可选；无存量配置引用；提示文案在 `profile.layersDisabled` 基础上按需要更新。
- [缓存键与静态上下文耦合出错导致样式不更新] → 配置变更使缓存失效的既有逻辑不变；tasks 安排编辑-保存-渲染一致性验证。

## Migration Plan

无迁移：物料 / 多图层功能未随正式版发布，无存量 layers 配置。纯 `crosshair` 配置行为不变（`use_new_format` 对无 layers 配置恒为 false）。

回滚方式：`MATERIAL_RUNTIME_ENABLED` 翻回 `false` 即恢复全量软关闭。
