## Context

当前设置面板中，字段控件散落在三处实现，布局风格不统一：

- `StyleFields.tsx`（单图层）：内部私有 `SliderField`，"双行紧凑式"——label + 数值在同一行，slider 在第二行。
- `LayerEditors.tsx`（多图层样式/变换）：opacity/scale/rotation 内联实现，"三行舒展式"——label、slider、数值回显各占一行。
- `LayerPanel.tsx::renderWidget`（多图层物料参数）：根据 Rhai 物料 schema 的 `widget` 字段（slider/number/text/color/toggle/select/image_path）渲染控件；其中 `case "slider"` 和 `case "number"` 被合并到同一分支，两者都返回纯文本 `<input type="number">`——这是 slider 退化为纯文本的回归 bug 根因（见 `LayerPanel.tsx:368-390`）。

物料脚本（`crates/material/builtin/*.rhai`）中 90% 以上数值参数声明为 `widget: "slider"`，意味着多图层模式下绝大多数参数都丢失了拖拽能力。Rhai schema 的 `"slider"` 和 `"number"` 本意是两种不同控件（slider = 连续调节，number = 纯数值字段如图像宽高/位掩码），前端合并它们等于丢失了语义区分。

## Goals / Non-Goals

**Goals:**

- 修复 slider 退化回归：多图层 `widget: "slider"` 参数渲染为可拖拽 Radix `<Slider>`。
- 统一字段视觉规范：所有 widget 类型遵循"label 一行 + 控件一行"布局。
- 消除三处重复实现，收敛到 7 个对称命名的共享字段组件。
- 保持单图层与多图层视觉一致性，符合 WYSIWYG 原则。

**Non-Goals:**

- 不改变 Rhai 物料 schema 定义或 builtin 脚本内容（前端单方面修正渲染）。
- 不重构 `StyleFields` 的 `OrbPositionCheck`（位掩码组合控件，非单一 widget 类型，保持独立）。
- 不改变配置数据结构、IPC 接口或后端逻辑。
- 不新增 i18n 条目（复用现有 key）。

## Decisions

### Decision 1: 字段组件放在 `src/components/fields/` 子目录

**选择**：新建 `src/components/fields/` 目录，7 个组件一文件一组件对称命名。

**理由**：`src/components/ui/` 是 shadcn primitive 集合（radix 直接包装），字段组件是基于 primitive 的业务组合，语义层级不同。独立子目录命名清晰、便于批量 import。

**备选**：直接放 `src/components/` 根目录。否决理由：与 `Preview.tsx`、`LayerPanel.tsx` 等聚合组件混在一起，职责模糊。

### Decision 2: SliderField 采用"label + 可编辑数值 + slider"混合布局

**选择**：第一行 `label` 在左、`<input type="number">` 在右（borderless），第二行 Radix `<Slider>`。

```
┌─────────────────────────────────────────┐
│ 臂长                            [42]°    │  label + borderless input + 可选 unit
│ ●━━━━━━━━━━○──────────────────          │  Radix Slider
└─────────────────────────────────────────┘
```

**理由**：结合了单图层紧凑式（label + 数值同行省空间）与多图层舒展式（slider 独立一行）的优点。数值可点击精确输入，也可拖拽粗调，双输入互补。

**备选 A**：纯三行式（label/slider/数值回显各一行）。否决：数值不可编辑，多图层场景下精细调节不便。
**备选 B**：纯双行紧凑式（label + 数值同行，slider 第二行，数值只读）。否决：失去精确输入能力。

### Decision 3: 有单位的 slider 显示原始值 + 单位后缀，不做双向转换

**选择**：opacity 显示 `0.85%`（value 存 0.85）、scale 显示 `1.20x`、rotation 显示 `45°`。输入框里的值就是原始 value，用户输入什么就是什么。

**理由**：双向转换（如 opacity 输入 85 自动转 0.85）仅 opacity 一个场景需要，复杂度高收益低。原始值 + 后缀简单可靠，且与配置存储一致。

**备选**：带双向 toDisplay/fromDisplay 转换。否决：过度工程，仅 opacity 一个场景需要。

### Decision 4: ColorField 采用混合布局（label + hex 输入 + 第二行 picker + 快捷色）

**选择**：
```
┌─────────────────────────────────────────┐
│ 颜色                       [#ff0000]     │  label + 可编辑 hex
│ [■] ● ● ● ● ●                           │  color picker + 快捷颜色色块
└─────────────────────────────────────────┘
```

**理由**：`LayerEditors::LayerStyleEditor` 的颜色部分本质就是这个组件。抽离后 `LayerStyleEditor` 可直接复用 ColorField（带 `quickColors` prop），进一步消除重复。

### Decision 5: ToggleField 采用"label + switch 同行"例外布局

**选择**：label 在左，switch 在右，同一行。

**理由**：toggle 的语义是开关，第二行放一个孤零零的 switch 视觉上突兀且浪费空间。这是统一两行布局的合理例外。

### Decision 6: `renderWidget` 保持 switch 分发，case 分支收敛为一行调用

**选择**：保留 `renderWidget` 函数签名，每个 case 分发到对应字段组件，函数体从 ~60 行压缩到 ~15 行。

**理由**：`renderWidget` 是 schema widget 类型到 React 组件的映射中枢，保留它便于未来新增 widget 类型时单点扩展。

## Risks / Trade-offs

- **[Risk] 单图层视觉变化**：`StyleFields` 从"双行紧凑式（数值只读）"改为"两行混合式（数值可编辑）"，所有 12 种 style 字段视觉都会变。 → **Mitigation**：新布局是超集（多了精确输入能力），不丢失原信息；视觉差异可控，仍是两行。
- **[Risk] borderless input 可发现性**：用户可能没意识到数值可以点击编辑。 → **Mitigation**：聚焦时加 `focus:border-b focus:outline-none` 轻微高亮，hover 时 cursor 变 text。
- **[Risk] ColorField hex 输入解析失败**：用户输入非法 hex 字符串。 → **Mitigation**：onChange 解析失败时 fallback 到当前值（不更新），或用 `try/catch` 包裹 `hexToRgba`。
- **[Trade-off] 滑块拖拽 + 文本输入双路径**：同一 value 两个输入源，需要确保数值边界一致（input 输入超出 min/max 时 slider 显示何处）。 → **决策**：input 不强制 clamp（允许输入超出范围的中间态，如 min=0 max=1 时输入 5），但 onChange 回调由调用方决定是否接受；slider 本身 Radix 会 clamp。
