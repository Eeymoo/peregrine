## MODIFIED Requirements

### Requirement: 字段组件遵循统一两行布局规范

所有字段组件（SliderField / NumberField / TextField / ColorField / ToggleField / SelectField / ImagePathField）SHALL 遵循统一的两行布局规范：第一行为 `<Label>` + 数值/状态展示（flex justify-between），第二行为可交互控件。字段组件 MUST 支持可选的 `disabled` 属性以响应图层锁定状态。

`SliderField` SHALL 新增可选的 `format?: (v: number) => string` 回调参数。当传入 `format` 时，数值输入框 MUST 显示 `format(value)` 的返回值（而非原始 value），且 `unit` 后缀 MUST 被隐藏（format 函数负责完整格式化）。未传入 `format` 时保持现有行为（显示原始 value + 可选 unit 后缀）。

#### Scenario: SliderField 传入 format 时显示格式化值

- **WHEN** 调用 SliderField 时传入 `format={(v) => Math.round(v * 100) + "%"}`，value=0.5
- **THEN** 数值输入框显示 `50%`
- **AND** 不显示 unit 后缀（format 已包含完整格式化）
- **AND** 滑块拖动范围仍是 min/max 原值

#### Scenario: SliderField 未传 format 时保持原有行为

- **WHEN** 调用 SliderField 时未传入 format，value=120，unit="px"
- **THEN** 数值输入框显示 `120`
- **AND** 右侧显示 unit 后缀 `px`

#### Scenario: SliderField format 用于角度显示

- **WHEN** 调用 SliderField 时传入 `format={(v) => v + "°"}`，value=45
- **THEN** 数值输入框显示 `45°`

#### Scenario: 字段组件支持 disabled 属性

- **WHEN** 字段组件接 `disabled={true}`
- **THEN** input/Slider 等控件呈现禁用样式（opacity-50 + cursor-not-allowed）
- **AND** 不响应点击/拖动事件
