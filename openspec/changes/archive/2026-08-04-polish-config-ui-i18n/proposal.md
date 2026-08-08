## Why

设置面板存在两处影响使用的 UI 问题，且前端文案的国际化覆盖缺少系统性的审查手段：

1. 图层编辑器右侧的「变换」区块（位移 / 缩放 / 旋转）所依赖的物料运行时已软关闭，该功能当前不可用，继续展示会误导用户。
2. 单图层模式下，ProfileManager 进入新建 / 重命名编辑态时，「方案切换下拉框 + 输入框 + 确认 + 取消」同时渲染，在 320px 宽的右侧面板中溢出。
3. 前端 i18n 已落地（`src/i18n/` + `useI18n`），但缺少工具化审查：无法确认是否仍有硬编码文案未走 `t()`、locale key 是否双语对齐、代码中引用的 key（如 `common.add` 一类）是否都已创建对应文案。

## What Changes

- 暂时移除图层编辑器右侧的「变换」区块（隐藏 `LayerTransformEditor` 的挂载，保留组件与 i18n 文案，便于物料运行时恢复时重新启用）。
- 修复 ProfileManager 编辑态溢出：编辑时隐藏方案切换下拉框，用输入框替换其位置（编辑态只渲染输入框 + 确认 + 取消）。
- 新增一个 i18n 审查 skill（`.agent/skills/i18n-audit/`），支持审查：硬编码 UI 文案、`t()` 引用但 locale 缺失的 key、zh-CN / en 双语 key 一致性、locale 中存在但未被使用的冗余 key。
- 依据该 skill 的审查结果，补齐当前缺失的文案 / 修复已发现的未国际化位置。

## Capabilities

### New Capabilities

- `config-ui-polish`: 设置面板 UI 修正——暂时移除图层编辑器「变换」区块；ProfileManager 编辑态布局调整（编辑时移除切换下拉、替换为输入框）。
- `i18n-audit`: 前端国际化审查能力——提供可重复执行的 i18n 审查 skill，并依据审查结果补齐缺失文案、保证双语 key 对齐。

### Modified Capabilities

<!-- 无既有 spec 的需求变更 -->

## Impact

- 前端代码：
  - `src/components/LayersEditor.tsx`：移除「变换」区块的渲染（保留 `LayerTransformEditor` 组件源码与 `layers.transformSection` 等 i18n key）。
  - `src/components/ProfileManager.tsx`：编辑态布局调整。
  - `src/i18n/locales/zh-CN.json` / `src/i18n/locales/en.json`：补齐审查发现的缺失 key。
  - 审查发现的其他硬编码文案所在组件（如有）。
- 新增文件：`.agent/skills/i18n-audit/SKILL.md`（i18n 审查 skill 定义）。
- 不涉及 Rust 后端、配置 schema、CI 流程变更；无 BREAKING 变更。
- 约定保持：UI 组件中用户可见文案一律走 `t()`；注释继续用简体中文。
