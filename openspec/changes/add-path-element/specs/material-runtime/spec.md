# material-runtime Delta：path 类型解析与 builtin.teardrop

## ADDED Requirements

### Requirement: Rhai 转换层解析 path 元素

Rhai → Element 转换层（`material.rs` 的 type_name match）SHALL 支持 `"path"` 类型：物料脚本返回的 path Map MUST 包含 `segments`（数组）与 `thickness`（数值），可选 `fill`（bool）、`stroke_color` / `fill_color`（`[r, g, b, a]` 数组）。段对象为 Map 风格（`#{cmd: "M", x: .., y: ..}`），坐标字段接受 Rhai float 或 int（与 polygon points 的解析先例一致）。

转换层 MUST 校验：`segments` 非空且首段为 `M`；`fill=false && thickness=0` 拒绝；`thickness >= 0`；颜色分量为 `0..=1` 的数值且数组长度为 4。校验失败 SHALL 返回 `ElementField` 错误（携带物料 id 与具体原因），错误信息 MUST 使用英文技术描述（与现有 ElementField 错误一致）。

#### Scenario: 解析完整 path 元素

- **WHEN** 物料脚本返回 `#{type: "path", segments: [#{cmd: "M", x: 100, y: 100}, #{cmd: "L", x: 200, y: 100}, #{cmd: "Z"}], thickness: 2.0, fill: true, fill_color: [0.3, 0.5, 1.0, 0.2]}`
- **THEN** 转换层输出 `Element::Path`，段结构、`fill=true`、`fill_color=Some([0.3, 0.5, 1.0, 0.2])`、`stroke_color=None` 正确

#### Scenario: 段命令为未知 cmd 被拒绝

- **WHEN** 段对象的 `cmd` 不是 `M/L/Q/C/Z` 之一（如 `"a"`）
- **THEN** 转换层返回 `ElementField` 错误，错误信息包含未知命令名

#### Scenario: Q 段缺少控制点被拒绝

- **WHEN** 段 `#{cmd: "Q", x: 10, y: 10}` 缺少 `x1/y1`
- **THEN** 转换层返回 `ElementField` 错误，错误信息指明缺失字段

### Requirement: 内置演示物料 builtin.teardrop

内置物料清单 MUST 包含 `builtin.teardrop`（动态物料，`is_dynamic = true`）：以屏幕中心为基准的圆环锚点，重心偏移由鼠标**加速度**驱动——偏移方向 = 加速度方向，形变强度与加速度幅度成正比；输出闭合 Path 图元（淡填充 + 实描边，展示元素级双色）。其 `build()` MUST 仅通过 `mouse_acceleration()`（动态输入快照）获取驱动量，MUST NOT 使用墙钟直读、MUST NOT 依赖 `mouse_pos()` 位置量（位置驱动会在鼠标停驻非中心点时永久形变）。默认配置与迁移逻辑 MUST NOT 引用 `builtin.teardrop`（零迁移成本）。

`builtin.teardrop` 作为 Path 图元表达能力的参考实现（活文档）：示范采样循环构造段数组、径向位移合成、双色输出的标准手法，以及加速度输入的标准消费手法。

#### Scenario: 开箱即得水滴物料

- **WHEN** 全新安装后打开图层编辑器的物料选择器（动态开关双开）
- **THEN** 列表包含 `builtin.teardrop`（带动态徽章），可直接添加

#### Scenario: 求值输出闭合 Path

- **WHEN** `MaterialRegistry::load_builtin()` 后以任意 `DynamicContext` 求值 `builtin.teardrop`
- **THEN** 求值成功，输出为闭合 Path 图元（首段 M、末段 Z），携带 `fill=true` 与双色覆盖

#### Scenario: 加速度驱动重心偏移

- **WHEN** 分别以 `mouse_acceleration = (0, 0)` 与 `mouse_acceleration = (-500, 0)` 两个上下文求值同一参数
- **THEN** 零加速度下输出正圆（各采样点半径一致）
- **AND** 非零加速度下轮廓向加速度方向重心偏移，段数与语义结构一致

#### Scenario: 形变强度与加速度幅度成正比

- **WHEN** 以幅度小 / 大的两个加速度上下文求值同一参数
- **THEN** 大加速度下水滴尖端偏离正圆的幅度 MUST 大于小加速度下的偏离幅度

#### Scenario: 静态上下文冻结快照

- **WHEN** 以 `DynamicContext::static_context()`（加速度为默认值 0）求值 `builtin.teardrop`
- **THEN** 求值成功，输出正圆形态（运行时软关闭的语义基础）

### Requirement: 内置演示物料 builtin.path_showcase

内置物料清单 MUST 包含 `builtin.path_showcase`（静态物料，`is_dynamic = false`）：同屏输出星形（`M`/`L` 直线段 + `Z` 闭合）与贝塞尔花（`C` 三次曲线段）两组 Path 图元，参数 MUST 暴露花瓣/角数与外径/内径比。物料 MUST 为纯静态（不调用任何动态输入 host function），静止配置下参与稳态跳帧（帧指纹不变）。缺省参数下 MUST NOT 携带颜色覆盖（继承图层色）；颜色覆盖作为可选参数暴露以演示覆盖机制。默认配置与迁移逻辑 MUST NOT 引用 `builtin.path_showcase`。

#### Scenario: 开箱即得静态演示物料

- **WHEN** 全新安装后打开图层编辑器的物料选择器
- **THEN** 列表包含 `builtin.path_showcase`（无动态徽章），可直接添加

#### Scenario: 求值输出星形与贝塞尔花

- **WHEN** 以任意 `DynamicContext` 求值 `builtin.path_showcase`
- **THEN** 求值成功，输出至少两个 Path 图元：一个仅含 `M`/`L`/`Z` 段（星形），一个含 `C` 段（贝塞尔花）

#### Scenario: 静态物料稳态跳帧

- **WHEN** 图层引用 `builtin.path_showcase` 且参数不变，连续两帧渲染
- **THEN** 帧指纹相等，overlay 跳过光栅化（事件驱动渲染，`ControlFlow::Wait`）

#### Scenario: 求值不依赖动态输入

- **WHEN** 以两个仅 `mouse_pos`/`time_ms` 不同的上下文求值同一参数
- **THEN** 两次输出完全相同
