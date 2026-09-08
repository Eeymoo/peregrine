---
name: material-creation
description: 为 Peregrine 创建自定义物料（.rhai 脚本）。当用户想要新的准心/锚点视觉样式（十字、圆环、呼吸环、路径形状、时钟、响应鼠标/键盘的动态效果等）、提到"物料""material""自定义准心""锚点样式""写一个 .rhai"，或要求修改/调试现有用户物料时使用。即使用户只描述了想要的视觉效果而没提"物料"二字，也应触发本 skill。
---

## 目标读者

本 skill 面向 **AI 编码代理**：根据用户想要的视觉效果，产出一份可直接被 Peregrine 加载的 `.rhai` 物料脚本，写入用户物料目录并验证加载成功。

## 背景：物料是什么

物料（Material）是 Peregrine 的视觉样式单元——一个 Rhai 脚本定义的 `(参数, 屏幕区域) → 元素列表` 纯映射。overlay 与设置预览共用同一求值管线（WYSIWYG），所以只要脚本加载成功且求值正确，两端显示必然一致。

权威参考（按需读取，不必全部预读）：

| 资料 | 路径 | 何时读 |
|---|---|---|
| 完整脚本文档（契约 / widget / 图元 / API / 沙箱） | `docs/src/content/docs/zh-cn/guide/material-scripting.md` | 写脚本前通读一遍 |
| 内置物料实例（11 个，代码风格标杆） | `crates/material/builtin/*.rhai` | 想看同类物料的成熟写法 |
| 可运行示例（静态 / 时钟 / 按键指示） | `crates/material/examples/*.rhai` | 需要最小可用模板 |
| 求值与转换层实现 | `crates/material/src/material.rs` | 排查字段被拒 / 返回值报错 |

## 物料存放位置（默认）

用户物料放在**应用数据目录**的 `materials/` 子目录（与 `config.json` 同级）：

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%/Peregrine/materials/` |
| macOS | `~/Library/Application Support/Peregrine/materials/` |
| Linux | `~/.config/Peregrine/materials/` |

规则：

- 文件名（不含扩展名）即物料名：`my_cross.rhai` → `user.my_cross`。
- **目录受文件监视器监视，写入/修改后约 500ms 自动热加载，无需重启应用**。目录不存在时应用启动会自动创建；代理主动 `mkdir -p` 也无妨。
- 用户物料（`user.*`）与内置物料（`builtin.*`）命名空间独立、并列展示，互不覆盖。
- 若用户装了带「打开物料目录」按钮的版本（设置 → 物料页），可以提示用户从那里直接打开目录核对。

## 创作流程

### 1. 明确需求 → 选型

先问清（或从描述推断）三件事，再动笔：

1. **视觉形态**：什么形状？（十字 / 圆环 / 多边形 / 曲线 / 图片 / 文本…）→ 决定图元类型
2. **静态还是动态**：要不要随时间 / 鼠标 / 按键变化？→ 决定 `is_dynamic()` 与动态 API
3. **用户可调什么**：哪些数值要暴露成参数？→ 决定 `defaults()` / `schema()`

选型经验：

- 矩形 / 圆 / 三角 / 线 → 用对应专用图元（CPU 直连光栅，最便宜）。
- **曲线、花瓣、平滑环等任意矢量形状** → 用 `path` 图元（唯一支持贝塞尔；SVG path 的 `d` 字符串可用 `parse_svg_path(d)` 解析，`A/S/T` 命令不支持会返回空数组，脚本需回退）。
- 多色物料才在元素级输出颜色；单色一律省略颜色字段，继承图层基色 × 不透明度（换色热键才能生效）。

### 2. 写脚本（必需契约）

每个物料必须导出三个顶层函数；`// Name:` 首行注释设置显示名：

```rhai
// Name: 简易十字
fn defaults() {
    // build 里读的每个参数都必须有默认值
    #{ size: 24.0, thickness: 2.0, gap: 4.0 }
}

fn schema() {
    // 每个参数一条描述符，UI 自动生成控件
    [
        #{key: "size", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
    ]
}

fn build(params, screen) {
    // 纯函数：从 screen（min_x/min_y/max_x/max_y）计算布局，禁止硬编码分辨率
    let cx = (screen.min_x + screen.max_x) / 2.0;
    [
        #{type: "rect", x: cx - 10.0, y: cx, w: 20.0, h: 2.0},
    ]
}
```

动态物料额外导出：

```rhai
fn is_dynamic() { true }          // 读取了 time_ms/mouse_pos/key_down/rand 等
fn refresh_interval_ms() { 100 }  // 可选：声明最小刷新间隔（ms），调度器据此节流
```

**易错点**（都来自 Rhai 与转换层的真实约束）：

- map 字面量是 `#{...}` 不是 `{...}`；`build` 必须返回**数组** `[#{}]`，单个 map 会报 `InvalidReturnType`。
- 语义化标签用 snake_case（`ring_radius_pct`），显示名走 schema 的 `label`。
- `widget` 只能是 `number/slider/color/select/toggle/image_path/text` 之一；`select` 需要 `options: [{value, label}]`。
- 文本元素 `font_weight` 省略或 `()` = 默认；显式值必须是 100..=900 的百位整数倍。
- 动态物料求值每帧发生，**把计算量压到最低**；输出做量化（如半径取整到 0.5px）可让帧指纹一致、整体跳过光栅化（参考 `builtin/teardrop.rhai` 的呼吸环写法）。
- 沙箱限制：`max_operations = 1,000,000`、递归 64 层、无文件 IO / 网络 / import。死循环会以 `MaterialError::Evaluation` 中止。

### 3. 写入与验证

1. 把脚本写入用户物料目录（见上文路径表），文件名用 kebab/snake_case 英文（显示名靠 `// Name:`，不靠文件名）。
2. **验证加载**：查看日志文件 `<应用数据目录>/Peregrine/peregrine.log`（与 materials/ 同级）：
   - 成功：`loaded user material` (info, 含 `id=user.<name>`)；
   - 失败：`failed to load user material` (warn, 含具体 `MaterialError`)。
3. 按错误形态修脚本（对照 `material-scripting.md` 的「常见错误」表）：

| 错误 | 原因 |
|---|---|
| `MissingFunction` | 漏写 `defaults` / `schema` / `build` |
| `Parse` | Rhai 语法错误 |
| `InvalidReturnType: expected Array` | `build` 没返回数组 |
| `ElementField: missing field 'x'` | 图元 map 缺必需几何字段 |
| `UnknownElementType` | `type` 字符串拼错 |
| `Evaluation: ...` | 运行时错误（操作数超限 / 类型不匹配） |

4. 若在**仓库内**开发，还可以做离线验证：把脚本放进 `crates/material/examples/` 跑 `cargo test -p peregrine_material`（示例即烟雾测试），或直接读 `crates/material/examples/simple_cross.rhai` 对齐格式。
5. 最终请用户在 **设置 → 图层** 里添加该物料（`user.<name>`）并调参数确认视觉效果；预览与 overlay 是同一求值结果，预览正确即两端正确。

### 4. 交付说明

向用户报告：物料文件路径、物料 id（`user.<name>`）、暴露了哪些可调参数、是否动态（及刷新间隔）。提醒：后续修改同一文件会自动热加载；删除文件即卸载该物料。
