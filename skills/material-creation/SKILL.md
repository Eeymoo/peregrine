---
name: material-creation
description: 为 Peregrine 创建自定义物料（.rhai 脚本）。当用户想要新的准心/锚点视觉样式（十字、圆环、呼吸环、路径形状、时钟、响应鼠标/键盘的动态效果等）、提到"物料""material""自定义准心""锚点样式""写一个 .rhai"，或要求修改/调试现有用户物料时使用。即使用户只描述了想要的视觉效果而没提"物料"二字，也应触发本 skill。
---

## 目标读者

本 skill 面向 **AI 编码代理**：根据用户想要的视觉效果，产出一份可直接被 Peregrine 加载的 `.rhai` 物料脚本，写入用户物料目录并验证加载成功。

## 背景：物料是什么

物料（Material）是 Peregrine 的视觉样式单元——一个 Rhai 脚本定义的 `(参数, 屏幕区域) → 元素列表` 纯映射。overlay 与设置预览共用同一求值管线（WYSIWYG），所以只要脚本加载成功且求值正确，两端显示必然一致。

## 参考资料（按阶段读取，不要全部预读）

| 资料 | 路径（相对本 skill 目录） | 何时读 |
|---|---|---|
| 完整 API 手册 | `references/api.md` | **写脚本前必读**：契约函数语义、widget 全表、10 种图元字段、动态 API、沙箱限制、错误对照 |
| 实战配方集 | `references/recipes.md` | 按需求选型后读对应配方：静态组合、呼吸动画、时钟、SVG 路径归一化、按键指示，含性能模型 |
| 起步模板 | `assets/template.rhai` | 复制为起点，替换 `// TODO` 标记处 |
| 内置物料源码 | 仓库内 `crates/material/builtin/*.rhai`（14 个） | 想看同类物料的成熟完整写法 |
| 求值与转换层实现 | 仓库内 `crates/material/src/material.rs` | 排查字段被拒 / 返回值报错的最后手段 |

不在仓库内运行时（远程安装场景）：`references/` 与 `assets/` 随 skill 一起分发，直接使用；内置源码与 `material.rs` 不可用，以 `references/api.md` 为准。

## 物料存放位置（默认）

用户物料放在**应用数据目录**的 `materials/` 子目录（与 `config.json` 同级）：

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

## 创作流程

### 1. 明确需求 → 选型

先问清（或从描述推断）三件事，再动笔：

1. **视觉形态**：什么形状？→ 决定图元类型（对照 `references/api.md` 图元全表）
2. **静态还是动态**：要不要随时间 / 鼠标 / 按键变化？→ 决定 `is_dynamic()` 与动态 API
3. **用户可调什么**：哪些数值要暴露成参数？→ 决定 `defaults()` / `schema()`

选型速查：

- 矩形 / 圆 / 三角 / 线 → 专用图元（CPU 直连光栅，最便宜）。
- **曲线、花瓣、平滑环等任意矢量形状** → `path` 图元（唯一支持贝塞尔；SVG `d` 字符串用 `parse_svg_path(d)` 解析，`A/S/T` 不支持返回空数组需回退）。
- 环形镂空 → 单条 path 双圈绕行（外圈 + 内圈逆序 + Z，nonzero 填充镂空），见 `references/recipes.md` 配方二。
- 多色物料才在元素级输出颜色；单色一律**省略颜色字段**，继承图层基色 × 不透明度（换色热键才能生效）。

### 2. 写脚本

从 `assets/template.rhai` 起步，或参照配方。写之前**必须**对照 `references/api.md` 核对：

- 三个必需契约函数（`defaults` / `schema` / `build`）+ 可选 `is_dynamic` / `refresh_interval_ms`
- 图元字段名与类型（拼错 = `UnknownElementType` / `ElementField`）
- 动态物料的性能纪律：`refresh_interval_ms()` 节流 + 输出量化跳帧（见配方二的性能模型）

### 3. 写入与验证

1. 把脚本写入用户物料目录（见上文路径表）。
2. **验证加载**：查日志 `<应用数据目录>/Peregrine/peregrine.log`（与 materials/ 同级）：
   - 成功：`loaded user material`（info，含 `id=user.<name>`）
   - 失败：`failed to load user material`（warn，含具体 `MaterialError`）
3. 按错误形态修脚本（对照 `references/api.md` 错误对照表）。
4. 若在**仓库内**开发：把脚本放进 `crates/material/examples/` 跑 `cargo test -p peregrine_material`（示例即烟雾测试）。
5. 请用户在 **设置 → 图层** 添加该物料（`user.<name>`）调参确认；预览与 overlay 同一求值结果，预览正确即两端正确。

### 4. 交付说明

向用户报告：物料文件路径、物料 id（`user.<name>`）、暴露了哪些可调参数、是否动态（及刷新间隔）。提醒：后续修改同一文件自动热加载；删除文件即卸载。

## 锚点设计原则（Peregrine 特有，写物料时自检）

Peregrine 的目的是**缓解 3D 晕动症**，物料是视觉锚点而非装饰，历史实机反馈沉淀出这些不变式（违反会被用户实际体验打回）：

- **中心稳定**：中心凹注视区域（环孔 / 十字交点附近）像素级稳定，动态只发生在边缘。
- **无方向性形变**：对称缩放可以，拉伸 / 加速度尖刺不行（反向 rest-frame 效应）。
- **动作克制**：呼吸级别（±3%）的缓慢节律有循证支持；快速晃动反而促成晕动。
- **颜色继承图层**：默认不携带颜色字段，让用户用图层级换色热键统一调控。
