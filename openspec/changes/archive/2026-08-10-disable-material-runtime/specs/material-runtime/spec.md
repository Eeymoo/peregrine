# material-runtime 增量规格

## ADDED Requirements

### Requirement: 物料运行时在应用层软关闭

物料运行时（`MaterialRegistry`、`Material::evaluate`、内置物料脚本）在主程序渲染链路中 SHALL NOT 被实例化或求值。overlay 渲染器 MUST 通过集中的编译期常量 `MATERIAL_RUNTIME_ENABLED`（当前值为 `false`）门控所有新格式（layers/物料）分支；常量折叠后物料求值代码 MUST NOT 进入运行时执行路径。`peregrine_material` crate 代码、内置物料脚本与单元测试 MUST 保留在仓库中并继续参与编译与测试，保证随时可通过翻转常量恢复。

#### Scenario: 渲染器不触达物料求值

- **WHEN** overlay 渲染器执行任意一次帧渲染
- **THEN** 渲染路径不调用 `build_layers_shapes` 与任何 `Material::evaluate`，仅走旧版 `build_shapes`（Crosshair）路径

#### Scenario: 预览 IPC 不触达物料求值

- **WHEN** 设置面板的预览组件调用 `build_shapes_ipc` 计算图元
- **THEN** 该 IPC MUST 通过 `MATERIAL_RUNTIME_ENABLED` 门控：软禁用时走旧版 `build_shapes_from_crosshair`（复用 `build_shapes` 几何 + crosshair 颜色/不透明度），不调用 `build_layers_shapes`、不构造 `DynamicContext`、不执行任何 `Material::evaluate`

#### Scenario: 物料代码保持可编译可测试

- **WHEN** 在 workspace 根目录执行 `cargo build` 与 `cargo test -p peregrine_material`
- **THEN** 编译成功且物料 crate 全部测试通过

#### Scenario: 单点恢复

- **WHEN** 维护者将 `MATERIAL_RUNTIME_ENABLED` 常量改为 `true` 并重新编译
- **THEN** 物料渲染链路恢复生效，无需修改其他任何代码或配置
