# element-primitives Delta：新增 Path 路径图元

## ADDED Requirements

### Requirement: Path 图元定义贝塞尔路径

系统 SHALL 提供路径图元 `path`：由段命令序列描述的贝塞尔路径，支持描边与填充。`Element::Path` 的字段 MUST 为：

- `segments: Vec<PathSegment>`：段命令数组，非空，首段 MUST 为 `M`
- `fill: bool`：是否填充，serde 缺省 `false`
- `thickness: f32`：描边宽度（逻辑像素）；`0` 表示关闭描边
- `stroke_color: Option<[f32; 4]>`：描边基色覆盖；`None` 继承图层基色
- `fill_color: Option<[f32; 4]>`：填充基色覆盖；`None` 继承图层基色

`PathSegment` SHALL 为 serde `tag = "cmd"` 的枚举，MUST 支持五种绝对坐标命令：`M { x, y }`（移动，开启子路径）、`L { x, y }`（线段）、`Q { x1, y1, x, y }`（二次贝塞尔）、`C { x1, y1, x2, y2, x, y }`（三次贝塞尔）、`Z`（闭合当前子路径）。不实现 SVG 的 `A`/`H`/`V`/`S`/`T` 命令与相对坐标。

绘制语义 MUST 满足下表（`fill=false && thickness=0` 的组合不可见，MUST 在 Rhai 转换层被拒绝）：

| `fill` | `thickness` | 效果 |
|--------|-------------|------|
| `false`（默认） | `> 0` | 纯描边，`stroke_color ?? 图层基色` |
| `true` | `> 0` | 先填充后描边（描边压填充接缝） |
| `true` | `0` | 纯填充，`fill_color ?? 图层基色` |
| `false` | `0` | 拒绝（转换层 `ElementField` 错误） |

#### Scenario: Path 序列化往返

- **WHEN** 构造含 `M`/`C`/`Z` 段的 `Element::Path` 并序列化为 JSON
- **THEN** JSON 包含 `"type": "path"` 与 `segments` 数组，段对象含 `"cmd": "m"`/`"c"`/`"z"`（snake_case tag）
- **AND** 反序列化后得到与原值完全相等的 `Element`

#### Scenario: 缺省字段向后兼容

- **WHEN** 反序列化一个不携带 `fill` / `stroke_color` / `fill_color` 字段的 path JSON
- **THEN** 反序列化成功，`fill = false`，两个颜色为 `None`（等价于纯描边 + 图层色）

#### Scenario: 首段非 M 被拒绝

- **WHEN** 物料脚本返回 `segments` 首段为 `L` 的 path 元素
- **THEN** Rhai 转换层 SHALL 返回 `ElementField` 错误，错误信息 MUST 说明首段必须为 M

#### Scenario: 空段数组被拒绝

- **WHEN** 物料脚本返回 `segments` 为空数组的 path 元素
- **THEN** Rhai 转换层 SHALL 返回 `ElementField` 错误

#### Scenario: 不可见组合被拒绝

- **WHEN** 物料脚本返回 `fill: false` 且 `thickness: 0` 的 path 元素
- **THEN** Rhai 转换层 SHALL 返回 `ElementField` 错误

### Requirement: Path 支持元素级颜色覆盖

Path 图元的 `stroke_color` / `fill_color` SHALL 是元素级颜色覆盖机制：最终颜色 = `(元素基色 ?? 图层基色) × 图层 opacity`。元素基色替换图层基色，但图层不透明度乘法 MUST 仍然生效（图层整体调透明度对所有元素一致）。颜色覆盖机制 MUST 仅作用于 Path 图元，不泛化到其他图元。

显式携带颜色覆盖的 Path 图元 MUST NOT 跟随图层换色（quick_colors 循环与换色热键只修改 `layer.style.color`）——显式覆盖即主动退出图层色。

颜色各分量 MUST 在 `0..=1` 归一化区间，越界值在 Rhai 转换层被拒绝。

#### Scenario: 覆盖色与图层透明度复合

- **WHEN** 图层 `opacity = 0.5`，Path 携带 `stroke_color = [1.0, 0.0, 0.0, 1.0]`
- **THEN** 描边最终 alpha 为 `0.5`，色相为纯红

#### Scenario: 缺省颜色继承图层色

- **WHEN** Path 不携带颜色字段，图层基色为蓝色
- **THEN** 描边/填充使用蓝色基色 × 图层 opacity

#### Scenario: 换色热键不影响覆盖色

- **WHEN** 用户通过 quick_colors 循环切换图层颜色
- **THEN** 未携带颜色覆盖的图元（含 Path）跟随换色
- **AND** 携带 `stroke_color` / `fill_color` 覆盖的 Path 保持覆盖色不变

#### Scenario: 颜色分量越界被拒绝

- **WHEN** 物料脚本返回 `stroke_color = [1.5, 0.0, 0.0, 1.0]` 的 path 元素
- **THEN** Rhai 转换层 SHALL 返回 `ElementField` 错误

### Requirement: Path 几何变换精确作用于控制点

`apply_transform` 对 Path 的处理 MUST 将仿射变换（平移/缩放/旋转）直接施加于全部段坐标（锚点与控制点）——仿射 × 贝塞尔数学上精确，无近似误差。`Z` 段无坐标，MUST 原样保留。颜色字段 MUST 不参与几何变换，原样透传。

#### Scenario: 旋转闭合路径

- **WHEN** 对一个水滴形闭合 Path 施加 `rotation_deg = 90` 变换
- **THEN** 输出路径的全部坐标点绕图层内容中心旋转 90°，段结构与颜色字段不变

#### Scenario: 缩放描边宽度

- **WHEN** 对 `thickness = 2.0` 的 Path 施加 `scale = 2.0` 变换
- **THEN** 段坐标放大 2 倍；`thickness` 保持 `2.0`（描边宽度不随图层缩放，与现有图元行为一致）

### Requirement: Path 包围盒按自适应展平计算

`elements_center`（图层内容中心，变换轴心）计算 Path 的包围盒时 MUST 使用自适应展平：de Casteljau 细分至曲线中点偏离弦 < 0.5 逻辑像素后，对全部折点取 min/max。描边模式包围盒 MUST 外扩 `thickness / 2`。展平 MUST 确定性（同曲线同阈值同结果），保证动态物料变换轴心不抖动。

#### Scenario: 贝塞尔包围盒不大于控制点壳

- **WHEN** 一段凸三次贝塞尔曲线的包围盒按本要求计算
- **THEN** 展平包围盒小于等于控制点壳（凸包），且与真实曲线极值距离 < 0.5 逻辑像素

#### Scenario: 同曲线重复计算结果一致

- **WHEN** 同一条 Path 在连续两帧分别计算包围盒
- **THEN** 两次结果完全相等

### Requirement: Path 光栅化走 SVG 后端与 Canvas Path2D

Path 图元在 overlay 端 MUST 经 SVG 后端（resvg）光栅化：`build_elements_svg` 生成 `<path d="...">`，`d` 属性由段命令拼接（`M/L/Q/C/Z` 绝对坐标，坐标乘 scale factor）；纯描边时 `fill="none"`，纯填充时 `stroke="none"`；端帽与连接统一 `round`。Path MUST NOT 实现 CPU softbuffer 直绘（与 Text/Polygon/Line 同策略，抗锯齿开关对 Path 无效）。

前端 Canvas 预览 MUST 通过 `Path2D` API 渲染（`moveTo/lineTo/quadraticCurveTo/bezierCurveTo/closePath`），绘制顺序与颜色语义（先 fill 后 stroke、覆盖色 × 图层 opacity）与 overlay 端一致。

Path MUST 纳入帧指纹（`hash_element`）：段命令判别值 + 坐标位模式 + `fill` + `thickness` + 覆盖色分量全部参与哈希；静止输入下指纹不变，维持稳态跳帧。

#### Scenario: 描边路径生成 SVG

- **WHEN** `build_elements_svg` 处理一个纯描边 Path（`fill=false, thickness=3, stroke_color=None`）
- **THEN** 生成 `<path d="..." fill="none" stroke="<图层色>" stroke-width="<3×scale>" stroke-linecap="round" stroke-linejoin="round"/>`

#### Scenario: 双色路径分段上色

- **WHEN** Path 同时开启填充与描边，携带 `fill_color` 与 `stroke_color` 覆盖
- **THEN** SVG 元素的 `fill` 与 `stroke` 分别取两个覆盖色 × 图层 opacity

#### Scenario: 预览用 Path2D 渲染

- **WHEN** 预览组件收到 path 图元
- **THEN** 通过 `Path2D` 构造同构路径，按 fill/thickness 语义调用 `fill()` / `stroke()`

#### Scenario: 静止输入维持跳帧

- **WHEN** 动态 Path 物料两帧之间 `mouse_pos` 等输入完全不变
- **THEN** 帧指纹相等，overlay 跳过光栅化

## MODIFIED Requirements

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
- `path`：贝塞尔路径（`segments, fill, thickness, stroke_color, fill_color`），详见「Path 图元定义贝塞尔路径」要求

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
