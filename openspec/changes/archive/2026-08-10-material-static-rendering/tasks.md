# material-static-rendering 实施任务

> 来源：proposal.md + design.md（D1–D6）。验收标准见 specs/ 两个 delta 的场景。

## 1. 开关常量（D1）

- [x] 1.1 `crates/peregrine/src/lib.rs`：`MATERIAL_RUNTIME_ENABLED` 翻回 `true`；新增 `pub const MATERIAL_DYNAMIC_INPUT_ENABLED: bool = false;`，附中文注释（用途 / 与主开关关系 / 恢复方式）
- [x] 1.2 `src/lib/feature.ts`：同步翻转主常量并新增 `MATERIAL_DYNAMIC_INPUT_ENABLED = false`，附中文注释

## 2. 渲染恢复 + 静态上下文（D2）

- [x] 2.1 `crates/peregrine/src/overlay_renderer.rs`：两处动态上下文选择改为按 `MATERIAL_DYNAMIC_INPUT_ENABLED` 分支——启用走 `poll_dynamic_context`，停用走 `DynamicContext::static_context()`；更新门控注释
- [x] 2.2 `src-tauri/src/lib.rs` 预览 IPC（`build_shapes_ipc`）：确认主开关翻回后恢复 layers 求值分支；上下文同样按 `MATERIAL_DYNAMIC_INPUT_ENABLED` 选择（停用走 `static_context()`），保证预览与 overlay 一致
- [x] 2.3 确认 `overlay.rs` 无需改动（事件驱动重绘，不恢复动态唤醒）

## 3. UI 过滤动态物料（D3）

- [x] 3.1 `src/components/LayerPanel.tsx`：物料选择器在 `MATERIAL_DYNAMIC_INPUT_ENABLED = false` 时过滤 `is_dynamic = true` 的物料（当前内置仅 `builtin.time`）
- [x] 3.2 全局 grep 确认动态输入相关设置项无遗漏（动态徽章随过滤自然消失）

## 4. 前端语义复查（D5）

- [x] 4.1 `ConfigApp.tsx` `effectiveCompatible`：确认恢复真实 `isLegacyCompatible` 判定（不兼容 profile 禁用单图层编辑），更新门控注释
- [x] 4.2 `useConfigAppState.ts`：确认恢复「单图层 + 不兼容 → 强制切多图层并写回持久化值」，更新门控注释
- [x] 4.3 确认模式持久化（localStorage 恢复）与切换入口保留行为不变

## 5. 测试与质量

- [x] 5.1 `cargo test -p peregrine_config -p peregrine_material -p peregrine` 全部通过
- [x] 5.2 `cargo clippy`（3 crate）+ `cargo fmt --check` 通过
- [x] 5.3 `npx tsc --noEmit` + `npm run build` 通过
- [x] 5.4 全局 grep `MATERIAL_RUNTIME_ENABLED` / `MATERIAL_DYNAMIC_INPUT_ENABLED` 枚举门控点，确认注释统一
- [x] 5.5 更新 `AGENTS.md`：软关闭描述改为「仅动态物料停用（`MATERIAL_DYNAMIC_INPUT_ENABLED`）」，说明静态渲染已恢复

## 6. 实机验证（Windows，人工）

- [x] 6.1 多图层配置 overlay 按图层叠加渲染（验收场景：多图层配置按图层渲染）
- [x] 6.2 图层编辑保存后 overlay 与预览即时更新（WYSIWYG）
- [x] 6.3 12 种内置物料逐样式渲染目检（确认文本图元等无渲染退化）
- [x] 6.4 物料选择器不出现 `builtin.time`；CPU 无动态轮询空转
- [x] 6.5 纯旧 crosshair 配置渲染外观不变
