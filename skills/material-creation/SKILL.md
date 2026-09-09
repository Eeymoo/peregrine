---
name: material-creation
description: 为 Peregrine 创建自定义物料（.rhai 脚本）。当用户想要新的准心/锚点视觉样式（十字、圆环、呼吸环、路径形状、时钟、响应鼠标/键盘的动态效果等）、提到"物料""material""自定义准心""锚点样式""写一个 .rhai"，或要求修改/调试现有用户物料时使用。即使用户只描述了想要的视觉效果而没提"物料"二字，也应触发本 skill。
---

## 目标读者与产出

本 skill 面向 **AI 编码代理**：根据用户想要的视觉效果，产出一份可直接被 Peregrine 加载的 `.rhai` 物料脚本，写入用户物料目录并验证加载成功。本文件是**完整自足的手册**；`references/recipes.md` 提供五个完整实战配方，`assets/template.rhai` 是起步模板。

## 背景：物料是什么

物料（Material）是 Peregrine 的视觉样式单元——一个 Rhai 脚本定义的 `(参数, 屏幕区域) → 元素列表` 纯映射。overlay 与设置预览共用同一求值管线（WYSIWYG），所以只要脚本加载成功且求值正确，两端显示必然一致。

---

# 第一部分：创作流程

## 1. 明确需求 → 选型

先问清（或从描述推断）三件事，再动笔：

1. **视觉形态**：什么形状？→ 决定图元类型（对照下文图元全表）
2. **静态还是动态**：要不要随时间 / 鼠标 / 按键变化？→ 决定 `is_dynamic()` 与动态 API
3. **用户可调什么**：哪些数值要暴露成参数？→ 决定 `defaults()` / `schema()`

选型速查：

- 矩形 / 圆 / 三角 / 线 → 专用图元（CPU 直连光栅，最便宜）。
- **曲线、花瓣、平滑环等任意矢量形状** → `path` 图元（唯一支持贝塞尔；SVG `d` 字符串用 `parse_svg_path(d)` 解析，`A/S/T` 不支持返回空数组需回退）。
- 环形镂空 → 单条 path 双圈绕行（外圈 + 内圈逆序 + Z，nonzero 填充镂空），见 `references/recipes.md` 配方二。
- 多色物料才在元素级输出颜色；单色一律**省略颜色字段**，继承图层基色 × 不透明度（换色热键才能生效）。

## 2. 写脚本

从 `assets/template.rhai` 起步，或参照 `references/recipes.md` 的配方。写之前对照下文第二部分核对契约、字段名、类型与性能纪律。

## 3. 写入与验证

物料存放位置（**应用数据目录**的 `materials/` 子目录，与 `config.json` 同级）：

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%/Peregrine/materials/` |
| macOS | `~/Library/Application Support/Peregrine/materials/` |
| Linux | `~/.config/Peregrine/materials/` |

规则：

- 文件名（不含扩展名）即物料名：`my_cross.rhai` → `user.my_cross`。文件名用 kebab/snake_case 英文；显示名靠首行 `// Name: xxx` 注释。
- **目录受文件监视器监视，写入/修改后约 500ms 自动热加载，无需重启应用**。目录不存在时先 `mkdir -p`。
- 用户物料（`user.*`）与内置物料（`builtin.*`）命名空间独立、并列展示，互不覆盖。
- 若用户装了带「打开物料目录」按钮的版本（设置 → 物料页），提示用户从那里直接打开目录核对。

**验证加载**：查日志 `<应用数据目录>/Peregrine/peregrine.log`（与 materials/ 同级）：

- 成功：`loaded user material`（info，含 `id=user.<name>`）
- 失败：`failed to load user material`（warn，含具体 `MaterialError`）

按错误形态修脚本（对照下文错误对照表）。若在**仓库内**开发：把脚本放进 `crates/material/examples/` 跑 `cargo test -p peregrine_material`（示例即烟雾测试）。最后请用户在 **设置 → 图层** 添加该物料（`user.<name>`）调参确认；预览与 overlay 同一求值结果，预览正确即两端正确。

## 4. 交付说明

向用户报告：物料文件路径、物料 id（`user.<name>`）、暴露了哪些可调参数、是否动态（及刷新间隔）。提醒：后续修改同一文件自动热加载；删除文件即卸载。

---

# 第二部分：完整 API 手册

与 `crates/material/src/material.rs`（求值/转换层实现）核对一致。

## 契约函数

| 函数 | 必需 | 签名 | 语义 |
|---|---|---|---|
| `defaults` | ✅ | `fn defaults() -> Map` | 默认参数 map。`build` 读取的每个参数**必须**有默认值。调用 `build` 前与图层级 `params` 合并（图层值优先），`build` 可假设每个键都存在 |
| `schema` | ✅ | `fn schema() -> Array` | 参数描述符数组，每条一个 `widget` 声明，UI 自动生成控件。`defaults` 的每个键都应出现 |
| `build` | ✅ | `fn build(params, screen) -> Array` | 核心：接收合并后参数 map + 屏幕矩形，返回 Element map **数组**。必须是纯函数 |
| `is_dynamic` | ⬜ | `fn is_dynamic() -> bool` | 读取了任何动态输入则 `true`。缺失默认 `false`（结果永久缓存） |
| `refresh_interval_ms` | ⬜ | `fn refresh_interval_ms() -> Int` | 动态物料声明最小刷新间隔（ms），调度器据此节流。缺失 = 0（按配置帧率每帧求值） |
| （首行注释） | ⬜ | `// Name: xxx` | 显示名；缺失取 id 末段（`user.my_cross` → `my_cross`） |

额外顶层具名函数（如 `mid(a, b)`）允许且常用于辅助计算——Rhai 不支持闭包赋值，复用逻辑写成顶层函数。

## Rhai 语言要点

物料跑在沙箱化 Rhai 上，与 JS/Python 的关键差异：

- map 字面量 `#{...}`（不是 `{...}`）；数组 `[...]`；访问 `m.key` / `m["key"]`。
- 赋值 `let x = ...;`；**没有闭包/lambda 赋值**，复用逻辑用顶层 `fn`。
- 循环用 `while i < n { ... i += 1; }`；`for x in arr { }` 可用。
- 条件表达式：`if a { x } else { y }` 是表达式可直接赋值。
- 字符串比较用 `==`；浮点取整 `.floor()` / `.ceil()`；`%` 取模对 float 有效（`time_ms() * 1.0 % period`）。
- `()` 是 unit（序列化为 null）；`params.contains("key")` 判断键是否存在（旧配置兼容读取时有用）。
- 函数体外不能 `return`；`build` 内可以用 `return` 提前退出。

## 参数 widget 全表

schema 每条 `#{key, label, widget, ...}`，`widget` 恰好一种：

| `widget` | 额外字段 | 值类型 | UI 控件 |
|---|---|---|---|
| `number` | `min`, `max`, `step` | float | 带微调按钮的数字输入 |
| `slider` | `min`, `max`, `step` | float | 实时显示数值的范围滑块 |
| `color` | _(无)_ | `[r, g, b, a]` 0..=1 数组 | 颜色选择器 |
| `select` | `options: [{value, label}]` | string | 下拉框（预设枚举，推荐代替自由数字表达"档位"） |
| `toggle` | _(无)_ | bool | 开关 / 复选框 |
| `image_path` | _(无)_ | string（文件路径） | 文件选择器（PNG） |
| `text` | _(无)_ | string | 自由文本输入 |

语义化参数键用 snake_case（`ring_radius_pct`），显示名走 `label`（可中文）。

## 图元类型全表

`build` 返回对象 map 数组，每个 map 含 `type` + 几何字段。坐标单位**逻辑像素**，原点为 overlay 窗口左上角。颜色字段省略 = 继承图层基色 × 图层不透明度（推荐缺省）。

| `type` | 字段 | 说明 |
|---|---|---|
| `rect` | `x`, `y`, `w`, `h`，可选 `corner_radius` | 轴对齐矩形（左上原点） |
| `circle` | `cx`, `cy`, `radius` | 实心圆 |
| `circle_stroke` | `cx`, `cy`, `radius`, `thickness` | 描边环 |
| `dashed_circle` | `cx`, `cy`, `radius`, `thickness`, `dash_len`, `gap_len` | 虚线环 |
| `triangle` | `x1`, `y1`, `x2`, `y2`, `x3`, `y3` | 三顶点实心三角形 |
| `polygon` | `points: [[x,y],...]` 或 `[{x,y},...]` 或 `[{0:x,1:y},...]` | 实心多边形 |
| `line` | `x1`, `y1`, `x2`, `y2`, `thickness` | 带粗细线段 |
| `text` | `x`, `y`, `content`, `font_size`，可选 `font_weight` | 文本；`font_weight` 取 100..=900 百位整数倍，省略或 `()` = 默认 400 |
| `image` | `path`, `x`, `y`, `w`, `h` | PNG 文件；解码由渲染器单独处理 |
| `path` | `segments`, `fill`, `thickness`，可选 `stroke_color`, `fill_color` | 矢量路径，唯一支持曲线，见下节 |

坐标字段接受 float 或 int（转换层都认）。其他 `type` 值 → `UnknownElementType`。

## path 图元详解

唯一支持贝塞尔曲线的图元，经 SVG 后端（resvg）渲染。段命令（`cmd`）：

| `cmd` | 字段 | 含义 |
|---|---|---|
| `M` | `x`, `y` | 移动到；开启新子路径（环形镂空 = 外圈后再次 M 开内圈） |
| `L` | `x`, `y` | 直线到 |
| `Q` | `x1`, `y1`, `x`, `y` | 二次贝塞尔（控制点 + 终点） |
| `C` | `x1`, `y1`, `x2`, `y2`, `x`, `y` | 三次贝塞尔（两控制点 + 终点） |
| `Z` | _(无)_ | 闭合子路径 |

结构约束（转换层硬校验）：

- `segments` 非空且**首段必须 `M`**。
- 当前转换层要求 `fill: true`（环形镂空依赖 nonzero 填充规则的反向内圈子路径）。
- `fill=false && thickness=0` 的不可见组合被拒绝；`thickness >= 0`。
- `stroke_color` / `fill_color` 各为 0..=1 的 `[r, g, b, a]` 四元数组；**省略则继承图层基色**（推荐——换色热键保持生效）。

平滑圆环构造法（见 recipes.md 配方二）：采样点用「中点 Q 平滑」——端点取相邻采样点中点、控制点取采样点自身，C1 连续无尖角。正圆也可用四个 90° C 段精确近似（控制点切向长度 = r × 0.5523）。

## 动态输入 API

host function 注册在 Rhai 引擎上。**仅当 `is_dynamic() == true` 且动态链路开启时输出才会变化**：

| 函数 | 返回 | 描述 |
|---|---|---|
| `time_ms()` | `int` | 自进程启动的毫秒数（**单调，来自 DynamicContext 快照**）。动画一律用它 |
| `now_ms()` | `int` | 当前 Unix 时间戳（ms，直读墙钟）。**不推荐**：绕过上下文快照导致预览/overlay 漂移；仅为兼容保留 |
| `format_time(ms, fmt)` | `string` | 毫秒时间戳 → 本地时间字符串；占位符 `yyyy` `MM` `dd` `HH` `hh` `mm` `ss` `a` |
| `mouse_pos()` | `Map {x, y}` | 当前鼠标位置（逻辑屏幕坐标） |
| `mouse_velocity()` | `Map {x, y}` | 鼠标速度（逻辑像素/秒）；差分采样 + EMA 平滑 + 死区归零（静止精确 0） |
| `mouse_acceleration()` | `Map {x, y}` | 鼠标加速度（逻辑像素/秒²）；静止/匀速精确 0 |
| `key_down(code)` | `bool` | 按键是否按下。键码：`"shift"` `"ctrl"` `"a"`..`"z"` `"0"`..`"9"` `"f1"`..`"f12"` `"space"` 等（大小写不敏感） |
| `rand()` | `float` | 确定性伪随机 `[0, 1)`；同一求值内多次调用递进 |
| `rand_range(min, max)` | `float` | `[min, max)` 随机浮点 |
| `rand_int(max)` | `int` | `[0, max)` 随机整数 |
| `rand_seed(s)` | unit | **无效**，仅为 API 兼容保留（宿主状态只读） |
| `parse_svg_path(d)` | `Array` | SVG path `d` 字符串 → 段数组（绝对坐标，段格式与 path 图元一致）；支持 `M/L/Q/C/Z` + 小写相对命令、`H/V` 展开、隐式重复；**`A/S/T` 不支持返回空数组**，脚本必须判空回退 |

## 确定性与缓存

- **静态物料**（`is_dynamic` false / 缺失）按参数集**求值一次永久缓存**（缓存键忽略动态上下文）。静态物料读动态输入得到冻结值——不要这样做。
- **动态物料**在动态上下文 `version` 变化时重新求值（动态链路开启时每帧一次）。RNG 种子派生自 `(material_id, params_hash, frame_count)`：同参数同类物料一帧内随机序列相同；一次求值内多次 `rand()` 不同。
- 预览端用静态快照求值——动态物料在预览里表现为某一时刻的静止帧（时钟/呼吸环预览不跳动是正常的）。

## 性能模型

动态物料每帧求值（最坏 120Hz），两条正交的降开销手段（内置物料都在用）：

1. **`refresh_interval_ms()` 节流**：声明"输出最快多久变一次"，调度器把唤醒节流到 `max(配置帧率, 最短声明间隔)`。时钟声明 500ms（60Hz → 2Hz）；呼吸环声明 100ms。外部事件（配置变更/窗口移动）触发的重绘不受限。
2. **输出量化跳帧**：两次求值若产生字节级相同的元素，帧指纹一致，**光栅化整体跳过**。呼吸环把半径量化到 0.5px（`(x * 2.0).floor() / 2.0`），一个呼吸周期只跨 ~2 个量化档，实际光栅 <4 次/秒。

辅助纪律：`build` 内不做无谓的数组重建 / 大循环；采样点数（如圆环 24 个）取视觉足够的最小值。

## 沙箱限制

| 限制 | 值 | 影响 |
|---|---|---|
| `max_operations` | 1,000,000 | 单次求值总工作量上限；紧凑死循环触顶，以 `Evaluation` 错误中止 |
| `max_call_levels` | 64 | 递归深度上限 |
| `max_expr_depths` | 128 / 128 | 表达式 / 语句嵌套深度 |
| 文件 IO | 无 | 无 `import`、无 `eval_file`、无文件系统访问 |
| 网络 | 无 | 无网络原语 |
| 宿主状态 | 只读 | 不能修改宿主；RNG 种子由 `(material_id, params, frame)` 派生 |

## 错误对照表

| 错误 | 可能原因 | 修复 |
|---|---|---|
| `MissingFunction { function }` | 漏写 `defaults` / `schema` / `build` | 补齐缺失函数 |
| `Parse` | Rhai 语法错误 | 对照语言要点（map 用 `#{}`；赋值 `let`；函数体外无 `return`） |
| `InvalidReturnType: expected Array` | `build` 返回单个 map / 数字 | 用数组包裹：`[#{}]` 而非 `#{}` |
| `ElementField: missing field 'x'` | 图元 map 缺必需几何字段 | 对照图元全表 |
| `ElementField: must be number/string/bool` | 字段类型不对 | 坐标给数字；`type`/`content` 给字符串 |
| `UnknownElementType` | `type` 字符串拼错 | 用全表中的类型名（snake_case） |
| path 相关 `ElementField` | 首段非 M / segments 空 / fill=false 且 thickness=0 / 颜色数组长度 ≠ 4 或分量越界 | 对照 path 结构约束 |
| `Evaluation: ...` | 运行时错误（操作数超限 / 类型不匹配 / 除零） | 简化脚本；`print` 调试（输出到 tracing 日志） |

---

# 第三部分：锚点设计原则

Peregrine 的目的是**缓解 3D 晕动症**，物料是视觉锚点而非装饰，历史实机反馈沉淀出这些不变式（违反会被用户实际体验打回，源自 `builtin/teardrop.rhai` 的演进史）：

1. **中心稳定**：中心凹注视区域（环孔 / 十字交点附近）像素级稳定，动态只发生在边缘 / 尺寸维度。
2. **无方向性形变**：对称缩放 ✅；拉伸 / 平移 / 加速度尖刺 ❌（反向 rest-frame 效应——teardrop v4 的加速度尖刺因此被移除）。
3. **动作克制**：呼吸级别（±3%）的缓慢节律有循证支持（呼吸 pacing 缓解 cybersickness）；快速晃动反而促成晕动。幅度 >5% 用户感知为"晃动"而非"呼吸"。
4. **颜色继承图层**：默认省略颜色字段，让用户用图层级换色热键统一调控；仅多色物料显式输出颜色。
5. **尺寸走屏幕短边百分比**：适配任意分辨率；厚度类参数可用绝对像素。
6. **档位用 select 不用自由数字**：呼吸速度这类语义参数给预设枚举，防调出破坏体验的值。
7. **不做鼠标全屏跟随**：teardrop 曾试过鼠标跟随，60fps 全屏重光栅 CPU 7%，被移除；鼠标响应限定在局部小图元。

---

# 附：参考资料

| 资料 | 路径（相对本 skill 目录） | 何时用 |
|---|---|---|
| 实战配方集 | `references/recipes.md` | 选型后读对应配方：静态组合、呼吸动画（性能标杆全文）、时钟、SVG 路径归一化、按键指示器 |
| 起步模板 | `assets/template.rhai` | 复制为起点，替换 `// TODO` 标记处 |
| 内置物料源码 | 仓库内 `crates/material/builtin/*.rhai`（14 个） | 想看同类物料的成熟完整写法 |
| 求值与转换层实现 | 仓库内 `crates/material/src/material.rs` | 排查字段被拒 / 返回值报错的最后手段 |
| 人类向文档 | https://peregrine.aukcraft.org/zh-cn/guide/material-scripting/ | 转给人类用户阅读 |

不在仓库内运行时（远程安装场景）：`references/` 与 `assets/` 随 skill 一起分发，直接使用；内置源码与 `material.rs` 不可用，以本手册为准。
