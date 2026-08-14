# material-dynamic-input Specification

## MODIFIED Requirements

### Requirement: 动态输入整体停用开关

系统 SHALL 支持双层停用动态输入：

1. **编译期总闸** `MATERIAL_DYNAMIC_INPUT_ENABLED = false`（Rust `crates/peregrine/src/lib.rs` + TS `src/lib/feature.ts` 成对）：整体停用动态输入与动态物料，所有门控点编译期折叠。
2. **运行时用户开关** `settings.material.dynamic_enabled = false`（默认 `true`）：用户侧软关闭，热生效。

两层开关构成与门：动态链路（`poll_dynamic_context` 调用、动态物料选择器可见性、预览动态刷新）仅在两者均为真时活跃。任一层关闭时：运行时 MUST NOT 轮询时间 / 鼠标 / 键盘；物料求值 MUST 使用 `DynamicContext::static_context()`；overlay 重绘 MUST 保持纯事件驱动。

**变更说明**：编译期开关当前值由 `false` 恢复为 `true`（本 change 起动态链路默认活跃）；运行时开关为新增层，用于用户在不重新构建的前提下冻结动态物料。

#### Scenario: 编译期开关独立回退

- **WHEN** 维护者将 `MATERIAL_DYNAMIC_INPUT_ENABLED` 改回 `false` 并重新构建
- **THEN** 动态链路整体停用，「物料」Tab 隐藏，行为回到 `material-static-rendering` 时代

#### Scenario: 运行时开关热生效

- **WHEN** `MATERIAL_DYNAMIC_INPUT_ENABLED = true` 且用户在「物料」Tab 关闭动态物料开关
- **THEN** 不重启的前提下，overlay 停止动态调度、求值改用 `static_context()`、选择器隐藏动态物料
- **AND** 重新开启后即时恢复

### Requirement: 物料选择器动态物料可见性

物料选择器对 `is_dynamic = true` 物料的展示 MUST 以「编译期开关 AND 运行时开关」为条件：两者均开启时展示并带「动态」徽章；任一关闭时隐藏。

#### Scenario: 双开时动态物料可选

- **WHEN** 编译期与运行时开关均开启
- **THEN** `builtin.time` 出现在物料选择器，带动态徽章，可添加为图层

#### Scenario: 运行时关闭时隐藏

- **WHEN** 运行时开关关闭（编译期开启）
- **THEN** 物料选择器不展示 `is_dynamic = true` 的物料，动态徽章不出现
