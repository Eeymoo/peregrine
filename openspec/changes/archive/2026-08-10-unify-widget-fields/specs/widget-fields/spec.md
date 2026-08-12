## ADDED Requirements

### Requirement: 字段组件遵循统一两行布局规范

所有共享字段组件 SHALL 采用"label 一行 + 控件一行"的两行布局，例外仅限 ToggleField（label + switch 同行）与 ColorField（混合布局：第一行 label + 可编辑 hex，第二行 picker + 快捷颜色色块）。

每个字段组件 MUST 接受 `label: string` 与 `disabled?: boolean` prop，label 渲染在第一行（ToggleField 例外，与控件同行右对齐）。控件 MUST 渲染在第二行（ColorField 的 picker + 快捷色块组合占据第二行；SliderField 的 Radix `<Slider>` 占据第二行）。

#### Scenario: SliderField 渲染为可拖拽滑块

- **WHEN** 渲染一个 `widget: "slider"` 类型的参数（如 `cross` 物料的 `size` 字段，min=1, max=200, step=1, value=42）
- **THEN** 第一行 SHALL 显示 label（如"臂长"）和可编辑的数值输入框（borderless，显示原始值 42）
- **AND** 第二行 SHALL 渲染 Radix `<Slider>`，滑块位置对应 value 在 min-max 区间的比例
- **AND** 拖拽滑块时数值输入框同步更新，输入框键入数值时滑块位置同步更新

#### Scenario: SliderField 带单位后缀显示

- **WHEN** 渲染 opacity slider，value=0.85，`unit="%"`
- **THEN** 数值输入框 SHALL 显示原始值 `0.85`，后跟 `%` 后缀
- **AND** 用户在输入框键入数字时，存储的 value 就是输入的原始数值，不做百分比双向转换

#### Scenario: NumberField 渲染为纯数值输入

- **WHEN** 渲染一个 `widget: "number"` 类型的参数（如图像物料的 `width` 字段）
- **THEN** 第一行 SHALL 显示 label
- **AND** 第二行 SHALL 渲染单个 `<input type="number">`，带 min/max/step 属性
- **AND** 不渲染滑块（与 SliderField 区分）

#### Scenario: TextField 渲染为纯文本输入

- **WHEN** 渲染一个 `widget: "text"` 类型的参数（如 time 物料的 `format` 字段）
- **THEN** 第一行 SHALL 显示 label
- **AND** 第二行 SHALL 渲染单个 `<input type="text">`

#### Scenario: ColorField 混合布局带可编辑 hex 与快捷色

- **WHEN** 渲染一个颜色字段，value=[1, 0, 0, 1]（红色），`quickColors` 包含若干预设色
- **THEN** 第一行 SHALL 显示 label 和可编辑的 hex 输入框（显示 `#ff0000`）
- **AND** 第二行 SHALL 渲染 `<input type="color">` picker 和快捷颜色色块列表
- **AND** 用户编辑 hex 输入框时，解析失败 SHALL 保持当前值不变（不更新 onChange）

#### Scenario: ToggleField 同行布局

- **WHEN** 渲染一个 `widget: "toggle"` 类型的参数（如 `tail_per_edge`）
- **THEN** label 和 Switch SHALL 在同一行，label 左对齐，switch 右对齐
- **AND** 不产生第二行（与其它字段的两行布局形成合理例外）

#### Scenario: SelectField 两行布局

- **WHEN** 渲染一个 `widget: "select"` 类型的参数（如 ring style）
- **THEN** 第一行 SHALL 显示 label
- **AND** 第二行 SHALL 渲染 Select 组件（下拉框），选项取自 schema entry 的 `options`

#### Scenario: ImagePathField 两行布局带浏览按钮

- **WHEN** 渲染一个 `widget: "image_path"` 类型的参数
- **THEN** 第一行 SHALL 显示 label
- **AND** 第二行 SHALL 渲染文本输入框和"浏览"按钮，点击按钮触发 `pick_image_path` IPC 调用

### Requirement: 字段组件位于共享目录统一导出

所有共享字段组件 SHALL 位于 `src/components/fields/` 目录，每个组件一个文件，文件名与组件名对称（`SliderField.tsx` → `SliderField` 组件）。组件 SHALL 通过 `@/components/fields/<Name>` 路径 import。

字段组件 MUST 基于 shadcn primitive（`@/components/ui/slider`、`@/components/ui/switch`、`@/components/ui/select`、`@/components/ui/label`、`@/components/ui/button`）构建，不得引入新的外部依赖。

#### Scenario: 字段组件复用现有 primitive

- **WHEN** 开发者查看任一字段组件源码
- **THEN** 该组件 import 的 primitive MUST 全部来自 `@/components/ui/*`
- **AND** 不得新增 npm 依赖到 `package.json`

#### Scenario: 三处调用点统一复用

- **WHEN** `StyleFields.tsx` 渲染 slider 类型字段
- **AND** `LayerEditors.tsx` 渲染 opacity/scale/rotation 字段
- **AND** `LayerPanel::renderWidget` 渲染 schema widget 字段
- **THEN** 三处 SHALL import 同一个 `SliderField`（或对应类型）组件
- **AND** 不存在重复的内联 slider/number/text/color/select 实现
