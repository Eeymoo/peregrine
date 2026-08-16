# material-runtime Specification (Delta)

## MODIFIED Requirements

### Requirement: 物料参数 schema 驱动 UI 控件生成

物料脚本通过 `fn schema()` 函数声明其参数元数据。前端配置界面 MUST 根据此 schema 自动生成参数控件，无需为每个物料手写控件代码。

schema 返回值的每个条目 MUST 包含字段：
- `key: String` — 参数名（与 `defaults()` 和 `build(params)` 的 key 对应）
- `label: String` — 参数标签的**源文案**（内置物料统一为 zh-CN）。对内置物料，该文案是 zh-CN 默认值与 locale 映射未命中时的回退兜底；前端实际展示 MUST 以 `builtin-material-i18n` 规范的 locale 映射覆盖结果为准。对用户物料，该文案原样展示。
- `widget: String` — 控件类型，取值：`"number"` / `"slider"` / `"color"` / `"select"` / `"toggle"` / `"image_path"` / `"text"`
- `min: Float`（可选，number/slider 时）
- `max: Float`（可选）
- `step: Float`（可选）
- `options: Array<Map>`（可选，select 时，每项 `{value, label}`；`label` 的定位同上——内置物料为 zh-CN 源文案与回退，展示以 locale 映射为准）
- `default: Any` — 默认值

同理，内置物料脚本顶部 `// Name:` 注释解析出的 `display_name` MUST 视为 zh-CN 源文案与回退兜底，前端展示 MUST 以 locale 映射覆盖结果为准。

#### Scenario: number 物料参数生成滑块控件

- **WHEN** 物料 schema 中声明 `{key: "size", widget: "slider", min: 1, max: 200, step: 1, label: "尺寸"}`
- **THEN** 前端配置面板 MUST 渲染一个范围 1-200、步长 1 的滑块控件
- **AND** 控件标签在 zh-CN 界面下显示"尺寸"，在其他语言界面下显示对应 locale 映射译文（内置物料）
- **AND** 用户调整滑块时，对应图层的 `params.size` 字段更新

#### Scenario: select 物料参数生成下拉框

- **WHEN** 物料 schema 中声明 `{key: "anchor", widget: "select", options: [{value: "top", label: "顶部"}, ...], default: "top"}`
- **THEN** 前端 MUST 渲染一个下拉选择框，列出所有 option 的 label（内置物料按 locale 映射覆盖后展示）
- **AND** 用户选择后，`params.anchor` 字段更新为对应的 value

#### Scenario: 未知 widget 类型回退

- **WHEN** schema 声明了一个不在支持列表内的 widget 类型（如 `"datetime"`）
- **THEN** 前端 MUST 回退为只读文本显示当前值，并记录警告
- **AND** 不影响其他参数控件的正常生成
