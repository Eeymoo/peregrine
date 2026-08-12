## Why

多图层模式（`LayerPanel::renderWidget`）中所有数值参数都退化为纯文本 `<input type="number">`，原本应该是滑块的参数（占 Rhai 物料 90% 以上）全部丢失了拖拽交互能力，与单图层模式（`StyleFields`）的 SliderField 体验严重不一致。同时，各 widget 类型在前端散落三处重复实现（`StyleFields` / `LayerEditors` / `LayerPanel::renderWidget`），布局风格各异，缺少统一的视觉规范。

## What Changes

- **修复回归 bug**：`LayerPanel::renderWidget` 的 `case "slider"` 分支不再与 `case "number"` 合并，改用真正的 Radix `<Slider>` 渲染，恢复拖拽交互。
- **统一字段视觉规范**：所有 widget 类型采用统一的"label 一行 + 控件一行"两行布局（ToggleField 同行例外、ColorField 混合布局例外）。
- **抽离共享字段组件**：新建 `src/components/fields/` 目录，包含 7 个对称命名的字段组件，消除三处重复实现。
- **消除重复**：`StyleFields` 的私有 `SliderField`、`LayerEditors` 的 opacity/scale/rotation 内联滑块、`LayerPanel::renderWidget` 的 switch + 内联控件全部收敛到共享组件。

## Capabilities

### New Capabilities
- `widget-fields`: 共享字段组件库，为设置面板中所有 widget 类型（slider/number/text/color/toggle/select/image_path）提供统一的视觉规范与交互行为。

### Modified Capabilities
- `layer-composition`: 多图层右侧参数面板中，物料参数控件（`MaterialParamControls`）按 widget 类型正确渲染——slider 类型渲染为可拖拽 SliderField，不再退化为纯文本输入。

## Impact

- **新增文件**（7 个）：`src/components/fields/{SliderField,NumberField,TextField,ColorField,ToggleField,SelectField,ImagePathField}.tsx`
- **修改文件**（3 个）：
  - `src/components/StyleFields.tsx`：删除私有 `SliderField`，改 import 共享组件；anchor/ring style/border style/grid alignment 的 select 改用 `SelectField`。
  - `src/components/LayerEditors.tsx`：opacity/scale/rotation 内联滑块改用 `SliderField`（带 `unit` 后缀）。
  - `src/components/LayerPanel.tsx`：`renderWidget` 重构为纯分发器，所有 case 分发到新字段组件。
- **保留不动**：`StyleFields` 的 `OrbPositionCheck`（位掩码组合控件，非单一 widget 类型）。
- **依赖**：复用现有 `@/components/ui/slider`、`@/components/ui/switch`、`@/components/ui/select`、`@/components/ui/label`、`@/components/ui/button` 等 shadcn primitive，无新增依赖。
- **i18n**：复用现有 i18n key，不新增翻译条目。
