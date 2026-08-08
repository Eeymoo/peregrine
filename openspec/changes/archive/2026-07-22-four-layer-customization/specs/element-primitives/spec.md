## ADDED Requirements

### Requirement: 元素为不可再分的渲染原语

系统 SHALL 提供一组固定的基础图元（Element），每个图元描述屏幕上的一种几何形状或图像内容。图元是物料脚本的输出单位，也是光栅化器（overlay_renderer）和前端预览（Canvas 2D）的共同输入单位。

支持的图元类型 MUST 至少包含：
- `rect`：填充矩形（`x, y, w, h`）
- `circle`：填充圆（`cx, cy, radius`）
- `circle_stroke`：圆环描边（`cx, cy, radius, thickness`）
- `dashed_circle`：虚线圆环（`cx, cy, radius, thickness, dash_len, gap_len`）
- `triangle`：填充三角形（3 个顶点坐标）
- `polygon`：填充多边形（顶点数组，至少 3 个点）
- `line`：粗线段（`x1, y1, x2, y2, thickness`）
- `text`：文本（`x, y, content, font_size`）
- `image`：图片（`path, x, y, w, h`）

所有图元 MUST 可被 `serde` 序列化为 JSON，且字段使用 `snake_case`。

#### Scenario: 矩形图元序列化往返

- **WHEN** 构造 `Element::Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0 }` 并序列化为 JSON
- **THEN** JSON 字符串中包含 `"type": "rect"` 及 `x/y/w/h` 字段
- **AND** 反序列化后得到与原值完全相等的 `Element`

#### Scenario: 物料脚本返回图元列表

- **WHEN** 物料脚本的 `build(params, screen)` 函数被调用
- **THEN** 返回值 MUST 是一个 Element 的数组
- **AND** 每个数组成员都是上述支持的图元类型之一

#### Scenario: 未知图元类型被拒绝

- **WHEN** 物料脚本返回一个不在支持列表内的图元类型（如 `"ellipse"`）
- **THEN** 物料求值 SHALL 返回错误，错误信息 MUST 包含未知的类型名

### Requirement: 图元光栅化支持所有图元类型

`overlay_renderer`（softbuffer CPU 光栅化）和前端 Canvas 预览 MUST 支持渲染上述全部图元类型。对同一图元的渲染结果 MUST 像素级一致（抗锯齿策略、坐标变换规则相同）。

现有 `crates/peregrine/src/shapes.rs::Shape` 枚举的 5 种图元（`Rect`/`Circle`/`CircleStroke`/`DashedCircle`/`Triangle`）MUST 升格为新的 `Element` 类型，光栅化函数保持原算法以保证兼容。

#### Scenario: 旧图元光栅化算法保持一致

- **WHEN** 用旧 `build_shapes` 的输入参数调用新 `Element::Rect` / `Element::Circle` 等
- **THEN** 光栅化到像素缓冲区的输出 MUST 与重构前完全一致（逐像素相等）

#### Scenario: 文本图元光栅化

- **WHEN** 物料返回 `Element::Text { x, y, content: "Hello", font_size: 16.0 }`
- **THEN** 渲染时在 `(x, y)` 处以 16px 字号绘制 "Hello" 文字
- **AND** 文字颜色取自当前图层样式

#### Scenario: 多边形图元光栅化抗锯齿

- **WHEN** 物料返回多边形图元
- **THEN** 边缘 MUST 应用与 `Triangle` 图元相同的距离场抗锯齿算法

### Requirement: 图元使用逻辑坐标

所有图元坐标 MUST 使用逻辑像素（DPI 无关），物理像素转换由渲染器在光栅化时按窗口 `scale_factor` 完成。这一约束与现状一致，确保图元定义在不同 DPI 设备上行为一致。

#### Scenario: 高 DPI 设备上图元自动缩放

- **WHEN** 在 `scale_factor = 2.0` 的设备上渲染一个 `Element::Rect { x: 0, y: 0, w: 100, h: 100 }`
- **THEN** 实际光栅化覆盖 200×200 物理像素
