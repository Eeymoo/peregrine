# material-runtime Delta Spec

> 来源：`material-static-rendering` proposal/design；修订 `disable-material-runtime` 的全量软关闭为「静态恢复 + 仅动态停用」。

## ADDED Requirements

### Requirement: 物料运行时静态渲染路径恢复

系统 SHALL 恢复 layers 多图层为 overlay 与预览的活跃渲染路径：当 profile 含 layers 时，overlay 与预览 IPC MUST 通过 `build_layers_shapes` 对图层物料求值并渲染其结果，不再回退旧版 `Crosshair` 路径（无 layers 的纯旧配置仍走旧路径，行为不变）。图层编辑 MUST 在保存后反映到 overlay 与预览（WYSIWYG）。

#### Scenario: 多图层配置按图层渲染

- **WHEN** 激活 profile 含两个 layers（如十字 + 圆环），overlay 运行中
- **THEN** overlay 渲染两个图层物料的叠加结果，而非旧版默认准星

#### Scenario: 图层编辑即时生效

- **WHEN** 用户在图层编辑器中修改某图层的颜色并保存
- **THEN** overlay 与预览在配置广播后按新颜色渲染，无需重启应用

#### Scenario: 纯旧 crosshair 配置行为不变

- **WHEN** 激活 profile 无 layers（纯旧 `crosshair` 配置）
- **THEN** overlay 与预览仍走旧版 `Crosshair` 渲染路径，外观与之前一致

### Requirement: 静态与动态物料分开关控制

系统 SHALL 提供两个独立的编译期开关（Rust 与 TS 成对）：`MATERIAL_RUNTIME_ENABLED` 控制物料静态渲染路径，`MATERIAL_DYNAMIC_INPUT_ENABLED` 控制动态输入与动态物料。两个开关 MUST 可独立翻转而互不影响；所有门控点 MUST 带统一注释标记且可 grep 检索。

#### Scenario: 动态关闭时静态渲染正常

- **WHEN** `MATERIAL_RUNTIME_ENABLED = true` 且 `MATERIAL_DYNAMIC_INPUT_ENABLED = false`
- **THEN** 静态物料正常渲染，动态输入不轮询，动态物料按固定快照冻结渲染

#### Scenario: 全量回滚

- **WHEN** `MATERIAL_RUNTIME_ENABLED` 翻回 `false`
- **THEN** 渲染整体回退旧版 `Crosshair` 路径，行为与全量软关闭一致
