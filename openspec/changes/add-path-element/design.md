# 设计：Path 路径图元

## Context

四层架构（元素 → 物料 → 图层 → 配置）中，Element 是物料脚本的输出单位，被 overlay 光栅化器（softbuffer CPU / resvg SVG 双后端）与前端 Canvas 预览共同消费。现有 9 种图元均为参数化固定形状；颜色与不透明度是图层属性（`Layer.style`），由 `build_layers_shapes` 展开为 `(Element, color, opacity)` 三元组后统一上色。

图元路由现状（overlay_renderer.rs）：`Image` 走 CPU blit；`Rect/Circle/CircleStroke/DashedCircle/Triangle` 走 CPU 直绘；`Polygon/Line/Text` 落入 `_ =>` 兜底交 SVG 后端（resvg）光栅化进同一 buffer。这一先例使新增复杂几何图元的成本极低。

动态输入管道已就绪：`mouse_pos()` host function（material.rs）、连续重绘（`MATERIAL_DYNAMIC_INPUT_ENABLED` 与门 + `settings.material.fps` 节拍）、帧指纹跳帧（`hash_element`）。物料脚本唯一缺的是能表达形变轮廓的图元。

## Goals / Non-Goals

**Goals:**

- 物料脚本可输出任意贝塞尔路径：开放/闭合、纯描边、纯填充、填充+描边双色。
- Path 在 overlay（SVG 后端兜底）与前端预览（Canvas Path2D）双端渲染一致。
- 几何变换精确：仿射变换直接作用于贝塞尔控制点（数学上精确，无近似误差）。
- 帧指纹覆盖 Path 全部字段，动态物料鼠标静止时回到稳态跳帧路径。
- 内置动态物料 `builtin.teardrop` 验证全链路。

**Non-Goals:**

- CPU softbuffer 直绘 Path（抗锯齿开关对 Path 无效，与 Text/Polygon/Line 同策略）。
- SVG `A`/`H`/`V`/`S`/`T` 命令、相对坐标、`fill-rule` 配置、端帽/连接样式配置。
- 颜色覆盖泛化到其他图元（仅 Path 携带 `Option` 颜色字段）。
- Rhai 侧路径构造辅助函数库（v1 直接输出 Map 字面量）。

## Decisions

### D1：段命令集 = M/L/Q/C/Z（绝对坐标）

**决策**：`PathSegment` 五种变体，serde `tag = "cmd"` + `rename_all = "snake_case"`（单字母命令名本身即合法 tag 值）。

**理由**：M/L/Q/C/Z 是表达任意平面曲线的最小完备集（二次曲线是三次的退化形式，保留 Q 是为了 Rhai 手感——水滴类形变常用单控制点采样）。有意识砍掉 `A`（弧）/`H`/`V`（平凡）/`S`/`T`（平滑简写，可由脚本算控制点）/相对坐标（脚本可自行累加），保持 Rust 解析器极小。

**替代方案**：直接收 SVG `d` 字符串——被否决：Rhai 中拼字符串易错、无法做 apply_transform（旋转需先解析）、类型安全为零。

### D2：颜色覆盖是 Path 专属的 `Option<[f32; 4]>`

**决策**：`stroke_color` / `fill_color` 均为 `Option<[f32;4]>`，`None` = 继承图层基色。最终颜色 = `(元素色 ?? 图层色) × 图层 opacity`——替换基色但保留图层不透明度乘法（Photoshop 组语义），图层整体调透明度仍对所有元素生效。

**理由**：现有链路（`build_layers_shapes` 三元组、`make_color`、`BuiltShape`）假设颜色活在图层上；Path 双色需求首次打破它。v1 用最小侵入（只加 Path 字段 + 渲染端判断），验证有价值后再考虑泛化。

**替代方案**：(a) 泛化到全部图元——波及 9 个变体 × 全部 match 点，v1 不值；(b) 双图层叠加（描边层 + 填充层）——动态物料每帧求值两次、参数两份、WYSIWYG 碎片化，被否决。

**已接受行为**（写入 spec）：quick_colors / 换色热键只改 `layer.style.color`，显式携带颜色覆盖的 Path 不跟随换色。显式覆盖 = 主动退出图层色。

### D3：绘制语义矩阵与校验位置

| `fill` | `thickness` | 效果 |
|--------|-------------|------|
| `false`（默认） | `> 0` | 纯描边，`stroke_color ?? 图层色` |
| `true` | `> 0` | 填充 + 描边（先 fill 后 stroke，描边压接缝，与 SVG 默认一致） |
| `true` | `0` | 纯填充，`fill_color ?? 图层色` |
| `false` | `0` | 不可见 → Rhai 转换层报 `ElementField` 错 |

**校验位置**：结构校验（首段必须 `M`、`fill=false && thickness=0` 拒绝、颜色分量 0..=1、thickness ≥ 0、segments 非空）全部在 `material.rs` 转换层完成，沿用 `font_weight` 先例（Element 是求值输出，不持久化到配置文件，`schema.rs` 不承担校验）。serde `#[serde(default)]` 保证缺省字段向后兼容。

### D4：变换精确、包围盒展平

**决策**：`apply_transform` 对 M/L/Q/C 的全部坐标点（锚点 + 控制点）直接施加仿射变换——仿射 × 贝塞尔在数学上精确（控制点变换 = 曲线变换），比 `Rect` 旋转拆三角形的近似更干净。`Z` 段无坐标，透传。

**包围盒**（`elements_center` 求图层内容中心用）：控制点壳会偏大（三次曲线最多 ~25%），采用自适应展平——de Casteljau 细分至中点偏离弦 < 0.5px（逻辑像素）后对折点取 min/max。同一曲线每帧得到同一 bbox（确定性阈值），动态物料变换轴心不抖动。描边模式 bbox 外扩 `thickness/2`（与 `CircleStroke` 的 `r + t/2` 一致）。

**替代方案**：直接用控制点壳——被否决（用户要求精度）；精确求导极值——三次贝塞尔解二次方程可行但代码量大，展平 0.5px 对轴心计算已足够且实现简单。

### D5：渲染路由——SVG 兜底 + Canvas Path2D

**决策**：overlay 端 Path 落入现有 `_ =>` 兜底（零路由改动）；`svg_renderer.rs::build_elements_svg` 新增 arm 生成 `<path d="M ... L ... Q ... C ... Z" fill="..." stroke="..." stroke-width="..."/>`，双色时按覆盖色分别取值（纯描边 `fill="none"`，纯填充 `stroke="none"`）。预览端 `Preview.tsx` 新增 case：`new Path2D()` + `moveTo/lineTo/quadraticCurveTo/bezierCurveTo/closePath`，按 fill/thickness 分色调用 `fill()` / `stroke()`。

**理由**：resvg 对 `<path>` 是原生支持；Canvas `Path2D` 与 SVG path 命令一一对应；两端实现都是几十行。round cap/join 统一（SVG `stroke-linecap="round" stroke-linejoin="round"` + Canvas `lineCap/lineJoin = "round"`），与现有 `Line` 图元的 round cap 行为一致。

### D6：帧指纹 arm

`hash_element` 新增 Path arm：判别值 + `fill` 位 + `thickness` 位模式 + 逐段（cmd 判别值 + 坐标位模式）+ 两个 `Option` 颜色逐分量位模式。与现有模式一致，鼠标静止 → 指纹不变 → 跳帧。

### D7：`builtin.teardrop` 演示物料（动态，加速度驱动重心偏移）

动态物料：以屏幕中心为圆心的**圆环**锚点，形变由鼠标**加速度**驱动——尖头朝加速度方向合成（cos³ 锥形前侧 + 尖角 L 直连段），背侧保持正圆弧不参与形变；静止或匀速移动（加速度 ≈ 0）时退化为正圆——这正是"3D 晃动多发生在视角急转"场景下锚点应有的行为。schema 声明 `is_dynamic`，预览 ~1s 轮询自动生效。

**实机反馈修正记录（v2 → v3，2026-08-15）**：
- 环的实现：单条 Path 双圈绕行（外圈顺时针采样 + 内圈逆序绕行 + Z），nonzero 填充规则下内圈以内环绕数相消（镂空）——与 `CircleStroke` 同视觉的圆环。采样点用「中点 Q 平滑」连接（端点取相邻采样点中点、控制点取采样点本身，C1 连续），缺省 24 采样无可见棱角。**不要用 M+L 直线段连采样点**（v1 实测呈十二边形）。
- **等宽形变（v3）**：径向偏移量只算一次（基于外径），内圈叠加同一绝对偏移——环宽在任意方向恒等于 `ring_width`，形变是「圆管弯折」而非「比例缩放」。v2 内外圈各自按比例缩放，尖端方向环越来越宽（视觉是铲子不是尖刺，实机反馈「不和谐」的根因之一）。
- **尖端连续（v3）**：尖角点按精确的加速度方向角插入采样序列（最近两采样点之间，叉积符号定侧），方向连续旋转时尖端平滑跟随。v2 尖角吸附到最近采样点，24 采样下以 15° 一档跳变（旋转时的量子化不和谐）。
- **形变幅度封顶（v3）**：strength 额外限制 `≤ 0.5 / tip_scale / 1.15`——缺省参数下尖端最大伸长 ≤ 半径 ~58%，轮廓仍读作「环」；v2 封顶 1.0 时尖端达半径 92%（巨型尖刺，实机观感突兀）。
- 前侧半径按 cos³ 锥形增长至尖端；尖端点偏移再乘 1.15 略过锥顶形成尖刺，用 L 直连角点（导数不连续 = 尖角）；背侧严格保持 base 半径不形变。**不要用线性 align 位移**（v1 实测呈「前拉后压」的椭圆畸变，底部不圆）。
- 触发阈值（`threshold` 参数，默认 150 px/s²）：幅度低于阈值的加速度（手抖/慢移的 EMA 残值）视为噪声，strength 精确取 0——配合调度层死区，静止时帧指纹稳定（跳帧）。且 strength=0 时所有采样点 tip=false（无角点，输出与零加速度逐段一致）。
- 颜色：**缺省不携带颜色覆盖 = 继承图层基色**（乘图层 opacity），quick_colors / 换色热键正常生效；「双色覆盖」作为 `use_override_colors` 可选参数暴露以演示元素级覆盖机制（v1 硬编码覆盖色导致用户换色无效，且 0.15 alpha 淡填充盖在预览棋盘格背景上显暗——均已修正）。

**为什么用加速度而非位置**：位置驱动的形变在鼠标停在任何非中心点时永久保持变形；加速度驱动只在"正在变速"时形变，静止/匀速回正——语义上更接近"运动冲击指示器"，也自然规避了"鼠标常驻角落 → 锚点永久歪着"的体验问题。

### D8：`builtin.path_showcase` 演示物料（静态，星形 + 贝塞尔花）

静态物料（`is_dynamic = false`）：同屏输出两组 Path——**星形**（`M`/`L` 直线段 + `Z` 闭合，展示尖角与直线段语法）与**贝塞尔花**（`C` 三次曲线段花瓣），可选第三组 `Q` 段示例。复用一套参数（花瓣数、外径/内径比、厚度），默认静止、事件驱动渲染（`ControlFlow::Wait`）。作为静态 Path 的参考实现：验证 WYSIWYG、图层变换（旋转/缩放对直线段与曲线段的精确作用）、以及**稳态跳帧**（静止输入下帧指纹不变，overlay 不重光栅化——teardrop 因每帧 mouse 轮询无法覆盖此场景）。参数缺省不使用颜色覆盖（继承图层色），`stroke_color`/`fill_color` 作为可选参数演示覆盖机制。

### D9：鼠标速度 / 加速度输入（差分 + EMA + 死区）

**架构原则不变**：`DynamicContext` 保持无状态快照，跨帧状态收敛在**平台轮询器**（`poll_dynamic_context`）。该函数每帧被调用（连续重绘路径），内部维护 `static` 上一采样（`frame_counter()` 的 `static AtomicU64` 先例），对 `GetCursorPos` 采样做差分：

```
每次 poll_dynamic_context():
  cur = GetCursorPos()
  dt  = 本帧与上一采样的时间差（Instant，非固定帧间隔假设）
  vel = (cur - prev) / dt                  # 逻辑像素/秒
  acc = (vel - prev_vel_ema) / dt          # 对平滑后速度差分
  vel_ema = α_v·vel + (1-α_v)·vel_ema      # 速度层 EMA，α_v ≈ 0.4
  acc_ema  = α_a·acc + (1-α_a)·acc_ema     # 加速度层 EMA，α_a ≈ 0.25（更平滑）
  if |vel_ema| < VEL_DEADZONE: vel_ema = 0    # 死区归零（关键！）
  if |acc_ema|  < ACC_DEADZONE:  acc_ema  = 0
  → 写入 DynamicContext { mouse_velocity, mouse_acceleration }
```

**双层 EMA 说明（v2 修正记录）**：加速度层 EMA 是形变平滑过渡的关键——差分会放大高频噪声，单层速度 EMA 不够（v1 实测 teardrop 形变逐帧跳变）；α_a = 0.25 让形变有 ~4 帧的缓入缓出。teardrop 物料侧另有 `threshold` 参数（默认 150 px/s²）做第二级死区：阈值以下 strength 精确取 0，杜绝"轻微移动就变形"。

**死区归零是硬性要求**：EMA 渐近衰减永不精确触零，若 `acc_ema` 一直微小非零，teardrop 每帧输出微小不同的段坐标 → 帧指纹永远变化 → 稳态跳帧失效 → 全帧率空转。低于阈值（约 5 px/s、50 px/s²，实施时标定）必须**直接置 0**，让指纹回到稳定值。

**确定性边界**：速度/加速度是物理量，同帧内多次调用 host function 返回同值（来自同一快照），但跨帧天然变化——与 `time_ms()` 同级的"显式动态输入"，帧指纹语义不受影响（静止输入 → 死区归零 → 指纹稳定）。预览端：`preview_snapshot` 补 0 速度/加速度（预览无法感知真实鼠标动力学）。

**Rhai API**：`mouse_velocity() -> Map {x: Float, y: Float}`（逻辑像素/秒）、`mouse_acceleration() -> Map {x, y}`（逻辑像素/秒²），与 `mouse_pos()` 同风格。非 Windows 平台 `poll_dynamic_context` 占位实现返回 0（先例：现有静态占位）。

## Risks / Trade-offs

- [Path 走 SVG 后端，抗锯齿开关对其无效] → 与 Text/Polygon/Line 现状一致，文档与 spec 明示；未来若需要可补 CPU 直绘。
- [元素级颜色引入两条上色路径，维护成本] → 颜色解析收敛在 Rhai 转换层单点，渲染端仅 Path arm 判断 `Option`；spec 锁定"仅 Path"防止半吊子泛化。
- [展平 bbox 有 0.5px 容差] → 仅影响变换轴心（图层居中旋转/缩放基准），不影响渲染几何；确定性阈值保证动态物料不抖。
- [Rhai Map 字面量写长路径较繁琐] → v1 接受（teardrop 示范采样循环构造数组的手法）；辅助函数库留作未来 change。
- [resvg 与 Canvas 2D 的贝塞尔光栅化实现不同，亚像素级可能有差异] → 双端几何定义完全一致（同一控制点），差异仅在抗锯齿采样，与 Text 图元现状相同，WYSIWYG 语义不受破坏。
- [速度/加速度的 EMA 死区阈值拍脑袋定不准] → 阈值作为常量集中在 `poll_dynamic_context` 附近，实施时用 teardrop 实测标定；调参不改架构。
- [差分对轮询间隔敏感（帧率档位 30/60/120 变化时 dt 波动）] → 用 `Instant` 实测 dt 而非假设固定帧间隔；EMA 平滑吸收残余噪声。
- [加速度驱动物料在预览端看不到效果] → `preview_snapshot` 速度/加速度为 0，teardrop 预览呈现正圆；属已接受行为，spec 明示。

## Migration Plan

Element 是求值输出，不持久化到配置文件——无配置迁移。旧物料不输出 `path` 类型则完全不受影响；`teardrop.rhai` 加入 `BUILTIN_MATERIALS` 后自动对所有用户可见（不影响现有图层，需手动添加图层才会使用）。回滚 = 移除枚举变体与物料条目，无状态残留。

## Open Questions

（无——探索阶段已收敛：D1–D7 均已拍板，见对话记录。）
