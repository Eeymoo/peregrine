# material-runtime Specification

## MODIFIED Requirements

### Requirement: 内置物料清单

内置物料清单 MUST 包含时间显示物料 `builtin.time`（动态物料，`is_dynamic = true`）：显示当前时间，参数含字号、位置、格式串（占位符 yyyy/MM/dd/HH/hh/mm/ss/a）、加粗开关。其 `build()` 实现遵循「上下文时间」要求（见 `overlay-dynamic-rendering` 规格之「内置时间物料使用上下文时间」）。

**变更说明**：`time.rhai` 由 `crates/material/examples/` 归位 `crates/material/builtin/`，`BUILTIN_MATERIALS` 嵌入清单加入 `"time"`；默认配置与迁移逻辑不引用 `builtin.time`（归位零迁移成本）。`examples/` 保留 `clock.rhai`（格式串示例）与 `simple_cross.rhai` / `key_indicator.rhai`（输入动态示例，不内置）。

#### Scenario: 开箱即得时钟物料

- **WHEN** 全新安装后打开图层编辑器的物料选择器（动态开关双开）
- **THEN** 列表包含 `builtin.time`（显示名「时间显示」），带动态徽章，可直接添加

#### Scenario: 求值走注册表

- **WHEN** `MaterialRegistry::load_builtin()` 后以 `DynamicContext::static_context()` 求值 `builtin.time`
- **THEN** 求值成功，输出 Text 图元序列；时间值为上下文时间（static 上下文下为 0 时刻对应值）
