## MODIFIED Requirements

### Requirement: 图层样式统一控制颜色与不透明度

图层样式（`LayerStyle`）SHALL 统一控制该图层所有图元的颜色（RGBA，0.0..=1.0）与不透明度（0.0..=1.0）。`build_layers_shapes` 在求值每个图层时 MUST 把 `layer.style.color` 与 `layer.style.opacity` 附加到该图层产出的每个 `Element` 上，作为 `(element, color, opacity)` 三元组返回。

不透明度数值在 UI 展示时 MUST 以百分比形式呈现（0-1 存储值显示为 0%-100%），SliderField 控件通过 `format` 回调实现转换。

#### Scenario: 图层颜色与不透明度附加到每个图元

- **WHEN** 图层样式为 `{ color: [1.0, 0.0, 0.0, 1.0], opacity: 0.5 }`，物料 build 返回 `[Rect{...}, Circle{...}]`
- **THEN** `build_layers_shapes` 输出 `[(Rect, [1,0,0,1], 0.5), (Circle, [1,0,0,1], 0.5)]`

#### Scenario: 不透明度 UI 展示为百分比

- **WHEN** 用户在图层样式编辑器看到 opacity = 0.5
- **THEN** SliderField 的数值输入框显示 `50%`（而非 `0.5` 或 `0.5%`）
- **AND** 滑块拖动范围仍是 0-1，step=0.01

#### Scenario: 单图层模式 opacity 同样展示为百分比

- **WHEN** 用户在单图层模式（ConfigApp）看到 crosshair.opacity = 0.5
- **THEN** 右上角数值显示 `50%`（而非 `0.50`）

## ADDED Requirements

### Requirement: grid 物料的 grid_size 实际生效

`grid.rhai` 物料的 `build(params, screen)` 函数 MUST 使 `params.grid_size` 真实决定单格宽度。cols/rows 的计算 MUST 使用 `floor` 取整（能完整放下的格子数），而非 `ceil`（多算一格导致超屏）。

- edge 模式：实际单格宽度 MUST 等于用户设定的 `grid_size`
- center 模式：`total_w = cell * cols` MUST 小于等于屏幕宽度，不超出边界

#### Scenario: grid_size=200 在 1920 屏幕下渲染 9 列（非 10 列）

- **WHEN** 用户设定 grid_size=200，屏幕宽度为 1920
- **THEN** cols = floor(1920 / 200) = 9
- **AND** edge 模式实际 cell_w = 200（用户设定值）
- **AND** center 模式 total_w = 200 * 9 = 1800（不超出屏幕）

#### Scenario: grid_size=120 在 1920 屏幕下渲染 16 列

- **WHEN** 用户设定 grid_size=120，屏幕宽度为 1920
- **THEN** cols = floor(1920 / 120) = 16
- **AND** total_w = 120 * 16 = 1920（刚好填满）

### Requirement: 物料 dead parameter 消除

所有在 `defaults()` / `schema()` 中定义的参数 MUST 在 `build(params, screen)` 中被实际消费，或在 schema 中明确隐藏/禁用。不允许存在「UI 提供控件但 build 不读取」的 dead parameter。

具体处理：

- `border_frame.inset`：build MUST 根据 inset 值控制边框位于屏幕内侧（inset=true）或贴边（inset=false）
- `edge_rect.corner_radius`：build 输出的 Rect shape MUST 携带 `corner_radius` 字段；渲染器 MUST 支持圆角矩形渲染（SVG 后端输出 `<rect rx>`，CPU 后端降级为直角并 warn 日志）
- `random_orb.center_deviation`：build MUST 在生成随机球位置时根据该值规避屏幕中心区域（使用拒绝采样算法）
- `random_orb.mode`：schema 控件 MUST 标记为 disabled 并显示「coming soon」提示，不实际改变行为

#### Scenario: border_frame inset=false 时边框贴边渲染

- **WHEN** 用户设置 border_frame 的 `inset: false`
- **THEN** 边框矩形位于屏幕边缘（min_x, min_y 起算），而非向内偏移 offset

#### Scenario: edge_rect corner_radius 渲染为圆角矩形

- **WHEN** 用户设置 edge_rect 的 `corner_radius: 20`
- **THEN** build 输出的 Rect shape 包含 `corner_radius: 20` 字段
- **AND** SVG 渲染器输出 `<rect rx="20">`
- **AND** CPU 渲染器尝试绘制圆角（若不支持则降级为直角并 warn 日志）

#### Scenario: random_orb center_deviation 生效

- **WHEN** 用户设置 random_orb 的 `center_deviation: 0.2`
- **THEN** 生成的随机球位置距屏幕中心的距离 MUST 大于 `屏幕短边 * 0.2`
- **AND** 拒绝采样重试次数有上限（如 10 次），超过则使用最后生成的位置

#### Scenario: random_orb mode 控件显示 coming soon

- **WHEN** 用户在图层参数面板看到 random_orb 的 mode select 控件
- **THEN** 控件显示为 disabled 状态
- **AND** 旁边或 tooltip 显示「coming soon」或「功能开发中」提示

### Requirement: 物料 schema slider max 按分级表扩充

所有内置物料的 `schema()` 中 slider 控件的 `max` 值 MUST 按「距离/偏移/尺寸=1920、半径=500、线粗=50、间隙=200、缩放=50、字体=400」分级标准调整。数量类（count）与比例类（*_pct）的 max 保持不变。

具体调整清单：

| 物料 | 参数 | 新 max |
|---|---|---|
| cross | size | 1920 |
| cross | gap | 200 |
| cross | thickness | 50 |
| large_cross | thickness | 50 |
| corner_dots | offset | 1920 |
| corner_dots | thickness | 50 |
| corner_dots | radius | 500 |
| ring | thickness | 50 |
| custom_orb | radius | 500 |
| custom_orb | offset | 1920 |
| random_orb | offset | 1920 |
| random_orb | jitter | 1920 |
| random_orb | radius_min / radius_max | 500 |
| border_frame | thickness | 50 |
| border_frame | offset | 1920 |
| edge_rect | size / secondary_size / margin | 1920 |
| edge_rect | corner_radius | 500 |
| edge_arrows | size | 400 |
| edge_arrows | distance | 1920 |
| edge_arrows | width | 200 |
| edge_arrows | tail_top / tail_bottom / tail_left / tail_right | 1920 |
| grid | grid_size | 1920 |
| grid | thickness | 50 |
| image | scale | 50 |
| image | offset_x / offset_y | ±1920 |

#### Scenario: cross 物料 size 可拖到 1920

- **WHEN** 用户在 cross 物料的 size slider 上拖动
- **THEN** 拖动上限为 1920（而非旧的 200）

#### Scenario: 比例类参数不受影响

- **WHEN** 用户在 ring 物料的 ring_radius_pct slider 上拖动
- **THEN** max 仍为 0.08（不变）

#### Scenario: 数量类参数不受影响

- **WHEN** 用户在 custom_orb 物料的 top_count slider 上拖动
- **THEN** max 仍为 20（不变）

### Requirement: Rect 图元支持圆角

`Element::Rect` SHALL 支持可选的 `corner_radius: Option<f32>` 字段，控制矩形的圆角半径。未输出该字段或值为 None 时，渲染为直角矩形（向后兼容）。

#### Scenario: Rect 携带 corner_radius 渲染为圆角

- **WHEN** Rect shape 包含 `corner_radius: Some(15.0)`
- **THEN** SVG 后端输出 `<rect rx="15">`
- **AND** CPU 后端尝试绘制圆角（若不支持则降级为直角）

#### Scenario: Rect 无 corner_radius 渲染为直角（兼容）

- **WHEN** Rect shape 不包含 `corner_radius` 字段或为 None
- **THEN** 渲染为标准直角矩形（与旧行为一致）
