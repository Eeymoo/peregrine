# 提案：新增 Path 路径图元

> 状态：active
> 跟踪 issue：#73（https://github.com/Eeymoo/peregrine/issues/73）

## Why

现有 9 种图元（rect / circle / line / text 等）都是"参数化固定形状"，物料脚本无法表达任意曲线轮廓——例如"鼠标向左移动时圆环渐变为偏心水滴"这类随动态输入形变的锚点。`CircleStroke` 只能渲染均匀厚度的正圆，无法表达曲率变化的闭合轮廓。新增 Path 图元（M/L/Q/C/Z 段 + 描边/填充双色）可从元素层面解除这一限制，让物料脚本获得真正的形状表达自由；动态输入管道（`mouse_pos()` host function、连续重绘、帧指纹跳帧）已全部就绪，只缺图元本身。

## What Changes

- 新增 `Element::Path`：贝塞尔路径图元，`segments: Vec<PathSegment>`（`M`/`L`/`Q`/`C`/`Z` 五种段命令），支持描边与填充。
- 新增元素级颜色覆盖：`stroke_color` / `fill_color` 为 `Option<[f32; 4]>`，`None` 时继承图层颜色（乘图层 opacity）；这是首个打破"颜色只活在图层上"假设的图元，范围锁死在 Path。
- Rhai 转换层（`material.rs`）新增 `"path"` 类型解析：segments 数组（Map 风格，与 polygon points 一致）+ 可选颜色数组字段。
- overlay 渲染走现有 SVG 后端兜底分支（resvg 原生 `<path d>`）；前端预览走 Canvas `Path2D`；CPU softbuffer 直绘不实现（与 Text/Polygon/Line 同策略）。
- 几何变换（`apply_transform`）直接作用于控制点（仿射 × 贝塞尔数学上精确）；图层内容中心（`elements_center`）用自适应展平（de Casteljau，偏离 < 0.5px）计算精确包围盒。
- `DynamicContext` 新增鼠标速度/加速度输入：`mouse_velocity()` / `mouse_acceleration()` host function；平台轮询器内部差分采样 + EMA 平滑 + 死区归零（鼠标静止后有限帧内精确归零，保住稳态跳帧）。
- 新增内置动态演示物料 `builtin.teardrop`：圆环重心偏移由鼠标**加速度**驱动（偏移方向 = 加速度方向，形变强度 ∝ 加速度幅度；静止/匀速移动退化为正圆），验证加速度输入 → Path → 双端渲染全链路。
- 新增内置静态演示物料 `builtin.path_showcase`：星形（M/L 直线段 + 尖角连接）与贝塞尔花（C 曲线段）复合锚点，验证静态 Path 的 WYSIWYG、图层变换与稳态跳帧（teardrop 是动态的，覆盖不了静止帧场景）。
- 帧指纹（`hash_element`）新增 Path arm：鼠标静止时指纹不变，回到稳态跳帧主路径。

## 目标

- 物料脚本可输出任意贝塞尔路径（开放/闭合、描边/填充/双色组合）。
- Path 在 overlay 与预览双端渲染一致（WYSIWYG）。
- 动态物料可用 `mouse_pos()` 等输入驱动 Path 每帧形变。
- 提供 `builtin.teardrop` 作为表达能力的活文档。

## 非目标

- 不实现 SVG `A`（弧）/ `H`/`V` / `S`/`T`（平滑简写）/ 相对坐标命令——脚本采样成 `C` 段即可表达。
- 不在 CPU softbuffer 后端直绘 Path（走 SVG 兜底，与 Text/Polygon/Line 一致）。
- 不将颜色覆盖机制泛化到其他 8 种图元（留待未来 change）。
- 不实现 SVG `fill-rule`（非零/奇偶）配置——统一使用非零规则（nonzero winding）。
- 不做路径布尔运算、描边端帽/连接样式（butt cap / miter join）配置，统一 round cap/join。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `element-primitives`：图元类型集合从 9 种扩展到 10 种，新增 `path` 图元的序列化格式、校验规则、光栅化行为（SVG 后端兜底 + Canvas Path2D）、几何变换语义（控制点精确仿射 + 展平包围盒）与首个元素级颜色覆盖机制。
- `material-runtime`：Rhai → Element 转换层新增 `"path"` 类型解析（segments 数组 + 可选颜色字段）；内置物料集合从 11 份扩展到 13 份（新增动态 `builtin.teardrop` 与静态 `builtin.path_showcase`）。
- `material-dynamic-input`：动态输入集合新增鼠标速度与加速度（`mouse_velocity()` / `mouse_acceleration()`），由平台轮询器差分采样 + EMA 平滑 + 死区归零提供。

## Impact

- `crates/config/src/schema.rs`：`Element` 枚举新增 `Path` 变体 + `PathSegment` 枚举定义。
- `crates/material/src/material.rs`：转换层 `"path"` arm + segments/颜色解析 + 单测。
- `crates/peregrine/src/platform/{mod,windows}.rs`：`poll_dynamic_context` 差分采样鼠标速度/加速度（EMA + 死区）。
- `crates/material/src/context.rs`：`DynamicContext` 新增 `mouse_velocity` / `mouse_acceleration` 字段。
- `crates/material/src/material.rs`：注册 `mouse_velocity()` / `mouse_acceleration()` host function。
- `crates/material/builtin/teardrop.rhai`：新增内置物料（动态，加速度驱动重心偏移）。
- `crates/material/builtin/path_showcase.rhai`：新增内置物料（静态，星形 + 贝塞尔花）。
- `crates/material/src/lib.rs`：`BUILTIN_MATERIALS` 常量追加（teardrop + path_showcase）。
- `crates/peregrine/src/shapes.rs`：`apply_transform` / `elements_center` 新增 Path arm + 展平工具函数 + 单测。
- `crates/peregrine/src/overlay_renderer.rs`：`hash_element` 新增 arm；图元路由分发无需改动（落入现有 SVG 兜底）。
- `crates/peregrine/src/svg_renderer.rs`：`build_elements_svg` 新增 `<path d>` 生成（含双色处理）。
- `src/types/config.ts`：`Element` union 新增 path 分支 + `PathSegment` 类型。
- `src/components/Preview.tsx`：`drawElement` 新增 path case（Path2D + 分色 fill/stroke）。
- 已接受行为（写入 spec）：quick_colors / 换色热键只修改图层色，显式携带颜色覆盖的 Path 图元不跟随换色。
- 兼容性：Element 为求值输出不持久化到配置文件，无旧配置迁移负担；未使用 Path 的物料与图层完全不受影响。
