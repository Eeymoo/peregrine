---
title: "图层管理"
---

**图层**（Layer）是一次物料调用加上自己的参数、变换与样式。一个 Profile 的视觉锚点是其全部图层按顺序堆叠的合成结果。你在设置 UI 里实际编辑的就是图层；物料是图层调用的可复用配方。

本文档面向使用者讲解图层概念。要编写物料本身，请看 [物料脚本创作](./material-scripting)；原始 JSON 结构见 [配置说明](./config)。

## 为什么需要图层？

v0.2 之前，每个 Profile 只能有单一准心样式。图层让你把多种物料组合成一个锚点：中心十字加边框；圆环加角点；自定义 PNG 贴图叠加在内置网格上。每个图层独立，可单独隐藏、锁定、重排、改样式而不影响其他图层。

## 图层结构

每个图层包含：

| 字段 | 类型 | 含义 |
|---|---|---|
| `id` | string | 图层内唯一标识（UUID 或简单序号字符串） |
| `name` | string | UI 显示的图层名 |
| `material` | `Builtin { id }` \| `User { name }` | 本图层调用的物料 |
| `params` | object | 图层级参数覆盖，叠加在物料 `defaults()` 之上 |
| `style` | `{ color, opacity, blend_mode }` | 应用到物料输出的图层级颜色 / 不透明度 / 混合模式 |
| `transform` | `{ offset_x, offset_y, scale, rotation_deg }` | 物料输出后应用的几何变换 |
| `visible` | bool | 隐藏的图层不参与渲染 |
| `locked` | bool | 锁定图层以防 UI 误改（对渲染无影响） |

### 叠加顺序

图层按 **数组顺序** 渲染：`layers[0]` 在 **最底层**，`layers[N-1]` 在 **最顶层**。后渲染的覆盖先渲染的。UI 中重排会改写数组。

### 变换

`Transform2D` 在物料输出图元后、应用样式前应用：

- `offset_x` / `offset_y` —— 位移（逻辑像素，默认 `0`）。
- `scale` —— 均匀缩放因子（默认 `1.0`）。
- `rotation_deg` —— 围绕屏幕中心的旋转角度（度，默认 `0`）。

变换适合把物料推到偏心位置、镜像，或用一个物料构造对称图案。

### 样式

`LayerStyle` 应用到物料输出的所有图元：

- `color` —— `[0, 1]` 区间的 RGBA 数组（默认 `[1, 1, 1, 1]`，白色）。
- `opacity` —— 图层不透明度 `[0, 1]`（默认 `0.6`）。
- `blend_mode` —— 当前仅支持 `normal`（`src over dst`）；为未来混合模式（`add` / `multiply` 等）预留。

> **覆盖语义**：图层样式会覆盖物料输出的任何颜色。需要多色锚点时，请用多个图层（各自样式）而不是在单个物料里编码颜色。

### 可见性与锁定

- **`visible: false`** 渲染时直接跳过该图层 —— 物料甚至不会求值。适合在同一 Profile 中保留备选设计而不删除。
- **`locked: true`** 仅是 UI 提示：设置面板会禁用该图层字段编辑，避免误动精心调好的锚点。对渲染零影响。

## 图层与 Profile

每个 [`Profile`](./config) 持有有序的 `layers` 数组。当前激活 Profile 的图层就是 overlay 渲染的内容。切换 Profile 即整体替换图层集合。

设置 UI 支持两种模式：

- **单图层模式**：只编辑 `layers[0]`。等价于 v0.2 之前的单准心体验，是迁移后 Profile 的默认视图。
- **多图层模式**：完整图层面板 —— 添加 / 删除 / 复制 / 重排 / 隐藏 / 锁定。关闭面板时的模式会记到 `localStorage`，下次启动恢复。

两种模式编辑的是同一份 `Profile.layers`；单图层模式只是聚焦展示 `layers[0]`。

## 迁移后的 Profile（单图层形态）

加载旧版单准心 `config.json` 时，`migration.rs` 会把旧 `Crosshair.style` 转换成恰好 **一个** 图层，引用对应的内置物料：

```jsonc
{
  "layers": [
    {
      "id": "migrated_cross",
      "name": "准星",
      "material": { "kind": "builtin", "id": "builtin.cross" },
      "params": { "size": 24.0, "thickness": 2.0, "gap": 4.0 },
      "style": { "color": [1,1,1,1], "opacity": 0.6, "blend_mode": "normal" },
      "transform": { "offset_x": 0, "offset_y": 0, "scale": 1.0, "rotation_deg": 0 },
      "visible": true,
      "locked": false
    }
  ],
  // crosshair: null  ← 旧字段已清空，下次启动按新格式加载，不再重复迁移
}
```

视觉效果与 v0.2 之前完全一致。在设置 UI 切到多图层模式即可叠加更多图层。

## 组合多图层锚点

搭建多图层锚点的典型流程：

1. 以基础图层起步（例如 `builtin.edge_rect` 放在顶部）。
2. **添加图层** → 选择物料（例如 `builtin.cross`）→ 调整 `params`。
3. 应用样式：设置 `opacity` / `color`，让它与底层干净合成。
4. 重复添加角点、中心圆环等。
5. 拖动重排，把最重要的图层放最上面。
6. 隐藏或锁定想保留但不想误改的图层。

每个图层的参数来自物料的 `schema()`，所以 UI 会显示物料专属控件（臂长滑块、位置下拉框等）。每种物料暴露哪些参数，见 [物料脚本创作](./material-scripting)。

## 相关文档

- [物料脚本创作](./material-scripting) —— 编写自定义物料
- [设置详解](./settings) —— 设置窗口各选项（渲染后端 / 抗锯齿 / 动态物料等）
- [配置说明](./config) —— 完整 JSON schema（含 `Profile.layers`）
- [推荐配置](./recommendations) —— 使用多图层的精选 Profile
