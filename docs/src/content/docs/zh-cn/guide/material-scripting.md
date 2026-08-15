---
title: "物料脚本创作"
---

**物料**（Material）是 Peregrine 的视觉样式单元：从 `(参数, 屏幕区域) → 元素列表` 的纯映射。物料以 Rhai 脚本编写，既可以是二进制内嵌的内置物料（如 `builtin.cross`），也可以是用户物料目录下的 `.rhai` 文件（`user.<名称>`）。

本指南按 **五步创作流程** 组织，并完整记录 API。文中代码片段取自 `crates/material/examples/` 下可直接运行并通过测试的示例，复制到用户物料目录即可加载。

> **状态说明**：当前构建中静态与动态物料均已启用。动态输入（`time_ms()` / `mouse_pos()` / `mouse_velocity()` / `mouse_acceleration()` / `key_down()` / `rand()`）默认开启，由设置 → 物料页的运行时开关门控。详见下文 [动态输入 API](#动态输入-api)。

## 五步创作流程

```
1. 选图元       →  要输出哪些 Element 类型？
2. 定布局       →  从屏幕区域 + 参数计算坐标
3. 抽参数       →  把魔法数字提升为命名参数，给合理默认值
4. 声明 defaults/schema →  暴露给 UI，让用户可调
5. 验证         →  用 Material::load 加载求值；在预览里检查
```

每个物料是一个 `.rhai` 文件，导出 **三个** 必需顶层函数与一个可选的标记函数。

## 三个必需函数

### `fn defaults() -> Map`

返回默认参数 map。`build` 中读取的每个参数 **必须** 在这里有默认值。键是字符串，值是数字 / 字符串 / 布尔 / 嵌套 map。

```rhai
fn defaults() {
    #{
        size: 24.0,
        thickness: 2.0,
        gap: 4.0,
    }
}
```

调用 `build` 前，这些默认值会与图层级 `params` 合并（图层值优先），所以 `build` 可以假设每个键都已存在。

### `fn schema() -> Array`

返回参数描述符数组。设置 UI 会按每条描述自动生成控件。`defaults()` 中的每个键都应出现在 `schema()` 里，以便用户编辑。

```rhai
fn schema() {
    [
        #{key: "size", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}
```

### `fn build(params, screen) -> Array`

物料的核心。接收合并后的参数 map 与屏幕矩形，返回 Element map 数组。该函数必须是 **纯函数**：相同输入 → 相同输出（除显式声明的动态输入外）。

```rhai
fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    [
        #{type: "rect", x: cx - 10.0, y: cy - 1.0, w: 20.0, h: 2.0},
    ]
}
```

### 可选：`fn is_dynamic() -> bool`

若物料读取任意 [动态输入](#动态输入-api)（`time_ms` / `mouse_pos` / `key_down` / `rand`），声明 `true`。缺失时默认 `false`。

- `false`（或缺失）→ 渲染器对结果 **永久缓存**（相同参数永远输出相同结果）。适合静态锚点。
- `true` → 渲染器在动态上下文变化时重新求值。设置 → 物料页关闭动态输入时，物料选择器会隐藏动态物料。

## 参数 widget 类型

每条 schema 条目使用恰好一种 `widget`，UI 按类型渲染对应控件。

| `widget` | 字段 | 值类型 | UI 控件 |
|---|---|---|---|
| `number` | `min`, `max`, `step` | float | 带微调按钮的数字输入 |
| `slider` | `min`, `max`, `step` | float | 实时显示数值的范围滑块 |
| `color` | _(无)_ | `[r, g, b, a]` 0..=1 数组 | 颜色选择器 |
| `select` | `options: [{value, label}]`，可选 `"default"` | string | 下拉框 |
| `toggle` | _(无)_ | bool | 开关 / 复选框 |
| `image_path` | _(无)_ | string（文件路径） | 文件选择器（PNG） |
| `text` | _(无)_ | string | 自由文本输入 |

> `color` widget 存储 RGBA 数组。多数颜色 / 不透明度由 [图层](./layers) 级样式统一应用并覆盖物料输出；物料级颜色主要用于多色物料。

## 图元类型

`build` 返回对象 map 数组。每个 map 含 `type` 字段与按类型不同的几何字段。坐标单位是 **逻辑像素**，原点为 overlay 窗口左上角。支持类型：

| `type` | 字段 | 说明 |
|---|---|---|
| `rect` | `x`, `y`, `w`, `h` | 轴对齐矩形（左上原点） |
| `circle` | `cx`, `cy`, `radius` | 实心圆 |
| `circle_stroke` | `cx`, `cy`, `radius`, `thickness` | 描边环 |
| `dashed_circle` | `cx`, `cy`, `radius`, `thickness`, `dash_len`, `gap_len` | 虚线环 |
| `triangle` | `x1`, `y1`, `x2`, `y2`, `x3`, `y3` | 三顶点实心三角形 |
| `polygon` | `points: [[x,y], ...]` 或 `[{0:x,1:y}, ...]` 或 `[{x,y}, ...]` | 实心多边形 |
| `line` | `x1`, `y1`, `x2`, `y2`, `thickness` | 带粗细的线段 |
| `text` | `x`, `y`, `content`, `font_size`，可选 `font_weight` | 文本；`font_weight` 取 100..=900 的百位整数倍，省略或 `()` 表示默认 |
| `image` | `path`, `x`, `y`, `w`, `h` | PNG 文件；解码由渲染器单独处理 |
| `path` | `segments`, `fill`, `thickness`, 可选 `stroke_color`, `fill_color` | 矢量路径；见下文 [路径图元](#路径图元) |

其他值会在求值时返回 `MaterialError::UnknownElementType`。

## 路径图元

`path` 图元以段列表输出任意矢量几何——唯一支持曲线的图元类型，经 SVG 后端渲染。

```rhai
fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let r = params.radius;
    [
        #{
            type: "path",
            segments: [
                #{cmd: "M", x: cx, y: cy - r},
                #{cmd: "Q", x1: cx + r, y1: cy - r, x: cx, y: cy + r},
                #{cmd: "Z"},
            ],
            fill: true,
            thickness: 0.0,
        },
    ]
}
```

段命令（`cmd`，snake_case）：

| `cmd` | 字段 | 含义 |
|---|---|---|
| `M` | `x`, `y` | 移动到；开启新子路径 |
| `L` | `x`, `y` | 直线到 |
| `Q` | `x1`, `y1`, `x`, `y` | 二次贝塞尔（控制点 + 终点） |
| `C` | `x1`, `y1`, `x2`, `y2`, `x`, `y` | 三次贝塞尔（两个控制点 + 终点） |
| `Z` | _(无)_ | 闭合子路径 |

字段语义：

- 当前转换层要求 `fill: true`（圆环依赖反向内圈子路径的镂空填充实现）。
- `thickness` 为描边宽度（0 = 无描边）。
- 省略 `stroke_color` / `fill_color`（各为 0..=1 的 `[r, g, b, a]`）时，元素继承 **图层基色 × 图层不透明度**——推荐缺省，换色热键保持生效；显式颜色覆盖图层色。

平滑圆环、花瓣及任何曲线锚点形状用 `path`。纯矩形/圆形请用专用图元（CPU 光栅直连，无 SVG 往返，更便宜）。

## `screen` 参数

`screen` 是包含 `min_x` / `min_y` / `max_x` / `max_y` 的 map，表示 overlay 覆盖的矩形区域（通常是目标窗口客户区）。应从中计算中心 / 边缘，而不是硬编码 1920×1080：

```rhai
let cx = (screen.min_x + screen.max_x) / 2.0;
let cy = (screen.min_y + screen.max_y) / 2.0;
let radius = (screen.max_y - screen.min_y) * params.ring_radius_pct;
```

## 动态输入 API

下列 host function 注册在 Rhai 引擎上，让物料响应时间 / 鼠标 / 键盘 / 随机数。**仅当 `is_dynamic()` 返回 `true` 且构建启用了动态输入时，输出才会真正变化。**

| 函数 | 返回 | 描述 |
|---|---|---|
| `time_ms()` | `int` | 自进程启动以来的毫秒数（单调）。开销低，适合做动画。 |
| `now_ms()` | `int` | 当前 Unix 时间戳（毫秒，真实时钟）。配合 `format_time` 使用。 |
| `format_time(ms, fmt)` | `string` | 把毫秒时间戳格式化为本地时间字符串，支持 `yyyy` `MM` `dd` `HH` `hh` `mm` `ss` `a` 占位符。 |
| `mouse_pos()` | `Map {x, y}` | 当前鼠标位置（逻辑屏幕坐标）。 |
| `mouse_velocity()` | `Map {x, y}` | 鼠标速度（逻辑像素/秒）。平台层差分采样 + EMA 平滑 + 死区归零（静止时精确为 0，帧稳定）。 |
| `mouse_acceleration()` | `Map {x, y}` | 鼠标加速度（逻辑像素/秒²）。同上管线；静止或匀速时精确为 0。 |
| `key_down(code)` | `bool` | 指定按键是否按下。键码：`"shift"` `"ctrl"` `"a"`..`"z"` `"0"`..`"9"` `"f1"`..`"f12"` `"space"` 等（大小写不敏感） |
| `rand()` | `float` | 确定性伪随机数 `[0, 1)`；内部计数器在每次调用时前进。 |
| `rand_range(min, max)` | `float` | `[min, max)` 内的随机浮点。 |
| `rand_int(max)` | `int` | `[0, max)` 内的随机整数。 |

### 确定性与缓存

- **静态物料**（`is_dynamic() == false`）按参数集 **求值一次**，永久缓存（缓存键忽略动态上下文）。静态物料读取动态输入会得到冻结值，请不要这样做。
- **动态物料** 在动态上下文 `version` 变化时重新求值（动态输入启用时每帧一次）。RNG 种子派生自 `(material_id, params_hash, frame_count)`，因此同一参数的两个同类物料在一帧内产生相同随机序列，而一次求值内多次 `rand()` 返回不同值。

### 刷新间隔节流

可选的第四个导出函数让动态物料保持低开销：

```rhai
fn refresh_interval_ms() {
    100  // 最小唤醒间隔（ms）
}
```

调度器把唤醒节流到 `max(配置帧率, 可见动态物料声明的最短间隔)`——内置时钟声明 500ms，唤醒从 60Hz 降到 2Hz；锚定环声明 100ms。外部事件（配置变更、窗口移动）触发的重绘不受限、即时生效。未声明该函数的物料按配置帧率运行。

第二个开销杠杆是 **输出量化**：两次求值若产生字节级相同的元素，帧指纹一致，光栅化被整体跳过。内置锚定环把呼吸半径量化到 0.5px，大多数唤醒完全不产生光栅化。

## 沙箱限制

物料运行在严格沙箱化的 Rhai 引擎内：

| 限制 | 值 | 影响 |
|---|---|---|
| `max_operations` | 1,000,000 | 限制单次求值总工作量；紧凑死循环会触顶并以 `MaterialError::Evaluation` 中止 |
| `max_call_levels` | 64 | 递归深度；过深递归会中止 |
| `max_expr_depths` | 128 / 128 | 表达式 / 语句嵌套深度 |
| 文件 IO | 无 | 无 `import`、无 `eval_file`、无文件系统访问 |
| 网络 | 无 | 无网络原语 |
| 宿主状态 | 只读 | `rand_seed(s)` 存在仅为 API 兼容，不能修改宿主状态 —— 种子由 `(material_id, params, frame)` 派生 |

常见错误形态与原因：

| 错误 | 可能原因 | 修复 |
|---|---|---|
| `MaterialError::MissingFunction { function }` | 漏写 `defaults` / `schema` / `build` | 补齐缺失函数 |
| `MaterialError::Parse` | Rhai 语法错误 | 对照 Rhai 语法检查（注意：map 用 `#{...}`，赋值用 `let x = ...;`，函数体外不能 `return`） |
| `MaterialError::InvalidReturnType: expected Array` | `build` 返回了单个 map 或数字 | 用数组包裹：`[#{}]` 而非 `#{}` |
| `MaterialError::ElementField: missing field 'x'` | 图元 map 缺少必需几何字段 | 对照上文图元类型表 |
| `MaterialError::UnknownElementType` | `type` 字符串拼写错误 | 使用列表中的类型名 |
| `MaterialError::Evaluation: ...` | 运行时错误（操作数超限 / 类型不匹配等） | 简化脚本；用 `print` 调试（输出到 tracing） |

## 完整可运行示例

下面是 `crates/material/examples/simple_cross.rhai` 的全文。复制到 `<应用数据目录>/materials/simple_cross.rhai`，物料选择器中会出现 `user.simple_cross`。

```rhai
// Name: 简易十字
// 静态物料示例：四段矩形组成的十字准心。
// 参数：
//   arm_length — 单臂长度（像素）
//   thickness  — 矩形粗细
//   gap        — 中心透明间隙

fn defaults() {
    #{
        arm_length: 20.0,
        thickness: 3.0,
        gap: 4.0,
    }
}

fn schema() {
    [
        #{key: "arm_length", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let arm = params.arm_length;
    let t = params.thickness;
    let g = params.gap / 2.0;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    [
        #{type: "rect", x: cx - arm, y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx + g,     y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx - t / 2.0, y: cy - arm,     w: t, h: arm - g},
        #{type: "rect", x: cx - t / 2.0, y: cy + g,       w: t, h: arm - g},
    ]
}
```

## 加载物料

物料从 `<应用数据目录>/materials/` 自动发现：

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%/Peregrine/materials/` |
| macOS | `~/Library/Application Support/Peregrine/materials/` |
| Linux | `~/.config/Peregrine/materials/` |

把 `.rhai` 文件放入该目录，文件名（不含扩展名）即物料 id 后缀（`my_cross.rhai` → `user.my_cross`）。目录在启动时扫描，也会响应手动重载；同名用户物料覆盖内置物料。

更多示例物料（静态 / 时间动态 / 输入动态）位于 [`crates/material/examples/`](https://github.com/eeymoo/peregrine/tree/main/crates/material/examples)，同时作为烟雾测试，由 `cargo test -p peregrine_material` 验证。

## 相关文档

- [图层管理](./layers) —— 物料如何在图层级别堆叠、变换、应用样式
- [配置说明](./config) —— `Profile.layers` 的 JSON 结构
- [`REPORT_CODES.md`](./report-codes) —— 遥测 Code 登记表（物料本身不发遥测，但相关代码路径会发）
