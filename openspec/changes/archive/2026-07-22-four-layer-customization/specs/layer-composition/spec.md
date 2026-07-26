## ADDED Requirements

### Requirement: Profile 由有序图层列表组成

系统 SHALL 将 `Profile` 从"单个 `Crosshair`"重构为"`Vec<Layer>` 有序图层列表"。图层顺序决定渲染顺序：列表前面的图层在底层，后面的图层在顶层（first-below-last-above，符合 Photoshop / Figma 习惯）。

图层 MUST 至少包含以下字段：
- `id: String` — 图层内唯一标识（UUID v4 或简单序号）
- `name: String` — 用户可读的图层名（如"中心十字"、"边框"）
- `material: MaterialRef` — 引用的物料（内置或用户）
- `params: Map<String, Value>` — 该图层实例的具体参数（覆盖物料 `defaults()`）
- `style: LayerStyle` — 图层级样式（颜色、不透明度、混合模式）
- `transform: Transform2D` — 图层几何变换
- `visible: Bool` — 是否可见
- `locked: Bool` — 是否锁定（锁定后 UI 不可误改）

#### Scenario: 多图层叠加渲染

- **WHEN** 一个 Profile 包含两个图层：先 `grid` 后 `cross`
- **THEN** overlay 渲染时 SHALL 先绘制 `grid` 物料的所有图元
- **AND** 再在上方绘制 `cross` 物料的所有图元
- **AND** 两个图层的图元在重叠区域按透明度混合

#### Scenario: 图层可见性切换

- **WHEN** 图层的 `visible = false`
- **THEN** 该图层不参与渲染（其物料不调用 `build`，图元不输出）
- **AND** 渲染性能不消耗在该图层上

#### Scenario: 图层参数与物料默认值合并

- **WHEN** 图层的 `params` 仅指定 `{size: 50}`
- **AND** 物料 `defaults()` 返回 `{size: 24, thickness: 2, gap: 4}`
- **THEN** 实际求值时使用的参数为 `{size: 50, thickness: 2, gap: 4}`（用户值覆盖默认值）

#### Scenario: 图层 params 中未知 key 被忽略

- **WHEN** 图层的 `params` 含有物料 schema 未声明的 key（如 `{foobar: 123}`）
- **THEN** 该 key SHALL 被忽略（不传给 `build`，也不报错）
- **AND** 日志记录该 key 名（debug 级别）

### Requirement: 图层支持几何变换

每个图层 MUST 携带 `Transform2D`，包含：
- `offset_x: Float` — X 方向位移（逻辑像素，默认 0）
- `offset_y: Float` — Y 方向位移
- `scale: Float` — 均匀缩放因子（默认 1.0）
- `rotation_deg: Float` — 围绕屏幕中心的旋转角度（度，默认 0）

变换应用于图层内所有图元的坐标：先将图元坐标平移到屏幕中心，应用缩放和旋转，再平移回 `(屏幕中心 + offset)`。

变换后的最终坐标作为光栅化输入；变换 MUST 在求值阶段（生成 Element 列表后）应用，物料脚本本身不感知变换。

#### Scenario: 图层位移变换

- **WHEN** 一个 `cross` 图层的 `offset_x = 100`，`offset_y = 0`
- **THEN** 该图层所有图元的 X 坐标 SHALL 增加 100
- **AND** 物料脚本接收的 `screen` 参数保持不变（仍是原始屏幕区域）

#### Scenario: 图层旋转变换

- **WHEN** 一个图层的 `rotation_deg = 90`
- **THEN** 该图层所有图元 SHALL 围绕屏幕中心顺时针旋转 90 度
- **AND** 图元之间的相对位置关系保持不变

#### Scenario: 变换不修改物料输出

- **WHEN** 同一物料同一参数在 `transform = identity` 和 `transform = {scale: 2}` 下分别被求值
- **THEN** 物料脚本本身返回的 Element 列表 MUST 完全一致
- **AND** 差异仅体现在变换应用后的最终光栅化结果

### Requirement: 图层样式统一控制颜色与不透明度

每个图层 MUST 携带 `LayerStyle`，包含：
- `color: [f32; 4]` — RGBA 颜色（0.0..=1.0），覆盖图元的默认颜色
- `opacity: f32` — 图层整体不透明度（0.0..=1.0），与图元 alpha 相乘
- `blend_mode: BlendMode` — 混合模式（默认 `Normal`）

图层样式作用于该图层所有图元：每个 Element 的最终颜色 = `material 产生的颜色 × layer.color × layer.opacity`（按预乘 alpha 计算）。

未来版本可扩展 `BlendMode` 枚举支持 `Add` / `Multiply` / `Screen` 等高级混合。

#### Scenario: 图层颜色覆盖物料默认

- **WHEN** 一个图层 `style.color = [1, 0, 0, 1]`（红色）
- **AND** 物料脚本返回的图元不带颜色信息（图元只描述几何）
- **THEN** 渲染时该图层所有图元使用红色

#### Scenario: 图层不透明度相乘

- **WHEN** 一个图层 `style.opacity = 0.5`
- **THEN** 该图层所有图元的最终 alpha 值 SHALL 是物料输出 alpha × 0.5

### Requirement: 图层操作通过 Tauri commands 暴露

系统 MUST 提供以下 Tauri commands 支持前端管理图层：
- `add_layer(material_id, name)` — 在当前 Profile 末尾添加图层
- `remove_layer(layer_id)` — 删除指定图层
- `move_layer(layer_id, new_index)` — 调整图层顺序
- `duplicate_layer(layer_id)` — 复制图层（生成新 id）
- `update_layer(layer_id, patch)` — 批量更新图层字段（params / style / transform / visible / name 等）
- `list_layers()` — 返回当前 Profile 的所有图层

所有图层操作 MUST 即时持久化到配置文件，并广播 `peregrine:layers-changed` 事件让前端同步刷新。

#### Scenario: 添加图层

- **WHEN** 前端调用 `add_layer("builtin.cross", "中心十字")`
- **THEN** 当前 Profile 的 `layers` 末尾 SHALL 新增一个图层
- **AND** 图层 `params` 取物料的 `defaults()`
- **AND** 配置文件立即更新
- **AND** 前端通过 `peregrine:layers-changed` 事件收到新图层列表

#### Scenario: 移动图层顺序

- **WHEN** 当前图层顺序为 `[A, B, C]`
- **AND** 前端调用 `move_layer("A", 2)`
- **THEN** 新顺序为 `[B, C, A]`
- **AND** overlay 渲染顺序立即更新

### Requirement: 图层 UI 提供完整管理面板

前端 MUST 提供图层管理面板，至少包含：
- 图层列表（按渲染顺序显示，最顶层在最上方）
- 每个图层项：显示图层名、可见性切换、锁定状态、物料名、删除按钮
- 拖拽排序
- 选中图层后，下方显示该图层的参数控件（由物料 `schema()` 动态生成）
- 图层级样式编辑（颜色选择器、不透明度滑块）
- 图层变换编辑（位移、缩放、旋转）

#### Scenario: 图层列表显示渲染顺序

- **WHEN** 一个 Profile 有 3 个图层，从底到顶为 `grid`、`cross`、`border_frame`
- **THEN** 图层管理面板从上到下显示为 `border_frame`、`cross`、`grid`
- **AND** 每个图层项的物料名和可见性状态正确显示

#### Scenario: 选中图层显示对应参数控件

- **WHEN** 用户选中一个 `builtin.cross` 图层
- **THEN** 下方参数面板 SHALL 显示该物料的 schema 定义的所有参数控件
- **AND** 控件初始值取图层 `params`（已合并默认值）
- **AND** 用户调整控件后，图层 `params` 立即更新并通过 IPC 持久化
