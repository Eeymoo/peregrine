## 1. 创建共享字段组件（`src/components/fields/`）

- [x] 1.1 新建 `src/components/fields/SliderField.tsx`：两行布局——第一行 label + borderless `<input type="number">`（带可选 `unit` 后缀，聚焦时 `focus:border-b focus:outline-none`），第二行 Radix `<Slider>`；slider 和 input 共享 value/onChange 双向同步；input 解析失败 fallback 到 0
- [x] 1.2 新建 `src/components/fields/NumberField.tsx`：两行布局——label + 纯 `<input type="number">`，支持 min/max/step 属性
- [x] 1.3 新建 `src/components/fields/TextField.tsx`：两行布局——label + `<input type="text">`
- [x] 1.4 新建 `src/components/fields/ColorField.tsx`：混合布局——第一行 label + 可编辑 hex 输入框（解析失败保持当前值），第二行 `<input type="color">` picker + 可选 `quickColors` 快捷色块列表；接受 RGBA tuple 与 hex 互转的辅助函数（从 LayerEditors 抽离）
- [x] 1.5 新建 `src/components/fields/ToggleField.tsx`：同行布局——label 左对齐 + `@/components/ui/switch` 右对齐，同一行不产生第二行
- [x] 1.6 新建 `src/components/fields/SelectField.tsx`：两行布局——label + `@/components/ui/select`，options 通过 prop 传入（`{value, label}[]`）
- [x] 1.7 新建 `src/components/fields/ImagePathField.tsx`：两行布局——label + `<input type="text">` + "浏览"按钮，点击按钮 `invoke("pick_image_path")`
- [x] 1.8 验证：所有字段组件仅 import `@/components/ui/*` 与 React，无新增 npm 依赖

## 2. 迁移 `LayerPanel.tsx::renderWidget`（核心修复点）

- [x] 2.1 将 `case "slider":` 与 `case "number":` 分支分离，`"slider"` 分发到 `<SliderField>`（传入 label/value/min/max/step/onChange/disabled）
- [x] 2.2 `"number"` 分支分发到 `<NumberField>`
- [x] 2.3 `"text"` 分支分发到 `<TextField>`
- [x] 2.4 `"color"` 分支分发到 `<ColorField>`（`renderWidget` 内不传 quickColors，多图层侧快捷色由 LayerStyleEditor 管理）
- [x] 2.5 `"toggle"` 分支分发到 `<ToggleField>`
- [x] 2.6 `"select"` 分支分发到 `<SelectField>`（options 从 schema entry 转换为 `{value, label}[]`）
- [x] 2.7 `"image_path"` 分支分发到 `<ImagePathField>`
- [x] 2.8 保留 `default` 分支的未知 widget 提示
- [x] 2.9 删除 `renderWidget` 内联的 `rgbToHex` / `hexToRgba` 辅助函数（已移至 ColorField）

## 3. 迁移 `LayerEditors.tsx`

- [x] 3.1 opacity 内联 Slider + 百分比回显 → 替换为 `<SliderField unit="%">`（value 传原始 0..1 值，min=0 max=1 step=0.01）
- [x] 3.2 scale 内联 Slider + "x" 回显 → 替换为 `<SliderField unit="x">`
- [x] 3.3 rotation 内联 Slider + "°" 回显 → 替换为 `<SliderField unit="°">`
- [x] 3.4 `LayerStyleEditor` 的颜色部分（color picker + quickColors）→ 评估复用 `<ColorField quickColors={...}>`，保持现有行为不变
- [x] 3.5 删除 `LayerEditors.tsx` 底部的 `rgbaToHex` / `hexToRgba` 辅助函数（如已被 ColorField 取代）

## 4. 迁移 `StyleFields.tsx`

- [x] 4.1 删除文件底部私有 `SliderField` 函数（行 287-323）
- [x] 4.2 顶部新增 `import { SliderField } from "@/components/fields/SliderField"`
- [x] 4.3 将 anchor/RingStyle/BorderStyle/GridAlignment 的 `<Select>` 块替换为 `<SelectField>`（封装 label + Select 两行布局）
- [x] 4.4 验证所有 12 种 `*Fields` 组件（EdgeRectFields / CrossFields / ... / CustomImageFields）视觉符合两行布局规范
- [x] 4.5 `OrbPositionCheck` 保持独立不动（位掩码组合控件）

## 5. 验证与回归测试

- [x] 5.1 运行 `cargo build`（确认 Rust 侧未受影响，理论上无需改动）
- [x] 5.2 运行 `npm run build`（TypeScript 编译通过，无类型错误）
- [x] 5.3 运行 `npx tauri dev` 手动验证：多图层模式选中 `cross` 图层，size/thickness/gap 参数渲染为可拖拽 SliderField
- [x] 5.4 手动验证：多图层模式选中 `image` 图层（如可访问），width/height 渲染为 NumberField（无滑块）
- [x] 5.5 手动验证：单图层模式所有 12 种 style 字段视觉符合两行布局，slider 拖拽与数值输入双向同步
- [x] 5.6 手动验证：LayerEditors 的 opacity/scale/rotation 显示原始值 + 单位后缀（0.85% / 1.20x / 45°）
- [x] 5.7 手动验证：ColorField 的 hex 输入框输入非法值（如 `#zzz`）时保持当前值不变
- [x] 5.8 运行 ESLint（`npm run lint` 或类似命令，如存在）
