# material-dynamic-input Delta Spec

> 来源：`material-static-rendering` proposal/design；动态输入整体停用为独立开关控制的能力。

## ADDED Requirements

### Requirement: 动态输入可整体停用

系统 SHALL 支持通过 `MATERIAL_DYNAMIC_INPUT_ENABLED = false` 整体停用动态输入：运行时 MUST NOT 轮询时间 / 鼠标 / 键盘（`poll_dynamic_context` 无活跃调用点）；物料求值 MUST 使用 `DynamicContext::static_context()`（`version = 0`，动态输入均为默认值）；overlay 重绘 MUST 保持纯事件驱动，不得为动态物料定期唤醒。静态物料求值缓存 MUST 因 `version = 0` 永久命中，不随帧重复求值。

#### Scenario: 无动态轮询

- **WHEN** `MATERIAL_DYNAMIC_INPUT_ENABLED = false` 且 overlay 运行中
- **THEN** 运行时不调用 `poll_dynamic_context`，CPU 行为与纯静态渲染一致

#### Scenario: 动态物料冻结渲染不崩溃

- **WHEN** 某图层引用 `is_dynamic = true` 的物料（如时钟）且动态输入停用
- **THEN** 该图层按 `static_context()` 的默认值冻结渲染（如时钟显示固定时间），不崩溃、不报错、不自动刷新

#### Scenario: 静态物料缓存命中

- **WHEN** 配置未变化且连续渲染多帧
- **THEN** 静态物料求值结果命中缓存（`version = 0`），不重复执行 Rhai 求值

### Requirement: 动态物料在设置 UI 不可选

当 `MATERIAL_DYNAMIC_INPUT_ENABLED = false` 时，物料选择器 MUST NOT 展示 `is_dynamic = true` 的物料；动态输入相关的设置项 MUST 隐藏。开关翻回 `true` 时上述物料与设置项 MUST 恢复可见。

#### Scenario: 动态物料从选择器隐藏

- **WHEN** 动态输入停用，用户在图层编辑器打开物料选择器
- **THEN** 列表中不出现 `is_dynamic = true` 的物料（如 `builtin.time`）

#### Scenario: 开关翻回后恢复可选

- **WHEN** `MATERIAL_DYNAMIC_INPUT_ENABLED` 翻回 `true` 并重新构建
- **THEN** 物料选择器重新展示动态物料，动态输入恢复轮询
