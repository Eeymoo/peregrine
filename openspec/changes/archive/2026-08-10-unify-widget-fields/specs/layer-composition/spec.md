## MODIFIED Requirements

### Requirement: 图层 UI 提供完整管理面板

前端 MUST 提供图层管理面板，至少包含：
- 图层列表（按渲染顺序显示，最顶层在最上方）
- 每个图层项：显示图层名、可见性切换、锁定状态、物料名、删除按钮
- 拖拽排序
- 选中图层后，下方显示该图层的参数控件（由物料 `schema()` 动态生成）
- 图层级样式编辑（颜色选择器、不透明度滑块）
- 图层变换编辑（位移、缩放、旋转）

参数控件 MUST 按 Rhai 物料 schema 声明的 `widget` 类型精确渲染：`widget: "slider"` SHALL 渲染为可拖拽 Radix `<Slider>` 组件（不得退化为纯文本 `<input type="number">`）；`widget: "number"` SHALL 渲染为纯数值输入框；二者 MUST 在前端区分对待，不得合并到同一渲染分支。所有参数控件 SHALL 复用 `widget-fields` 规范定义的共享字段组件，遵循统一两行布局规范。

#### Scenario: 图层列表显示渲染顺序

- **WHEN** 一个 Profile 有 3 个图层，从底到顶为 `grid`、`cross`、`border_frame`
- **THEN** 图层管理面板从上到下显示为 `border_frame`、`cross`、`grid`
- **AND** 每个图层项的物料名和可见性状态正确显示

#### Scenario: 选中图层显示对应参数控件

- **WHEN** 用户选中一个 `builtin.cross` 图层
- **THEN** 下方参数面板 SHALL 显示该物料的 schema 定义的所有参数控件
- **AND** 控件初始值取图层 `params`（已合并默认值）
- **AND** 用户调整控件后，图层 `params` 立即更新并通过 IPC 持久化

#### Scenario: slider 类型参数渲染为可拖拽滑块

- **WHEN** 用户选中一个图层，其物料 schema 声明某参数 `widget: "slider"`（如 `cross` 物料的 `size` 字段）
- **THEN** 该参数 SHALL 渲染为 SliderField 组件（Radix `<Slider>` + 可编辑数值输入框）
- **AND** 不得渲染为纯文本 `<input type="number">`
- **AND** 用户可通过拖拽滑块或键入数值两种方式调整参数

#### Scenario: number 类型参数渲染为纯数值输入

- **WHEN** 用户选中一个图层，其物料 schema 声明某参数 `widget: "number"`（如图像物料的 `width` 字段、time 物料的坐标字段）
- **THEN** 该参数 SHALL 渲染为 NumberField 组件（纯 `<input type="number">`，无滑块）
- **AND** 不得与 slider 类型合并到同一渲染分支
