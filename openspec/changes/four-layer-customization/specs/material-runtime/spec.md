## ADDED Requirements

### Requirement: 物料是 Rhai 脚本定义的可复用组件

系统 SHALL 引入 Rhai 嵌入式脚本引擎作为物料（Material）运行时。物料是一份 `.rhai` 脚本文件，定义"参数 + 屏幕区域 → Element 列表"的纯映射。物料 MUST 可被引用、缓存、求值，但不能直接修改全局状态。

#### Scenario: 物料脚本结构

- **WHEN** 加载一份合法的 `.rhai` 物料脚本
- **THEN** 该脚本 MUST 导出以下顶层函数：
  - `fn build(params, screen) -> Array` — 根据参数和屏幕区域返回 Element 数组
  - `fn defaults() -> Map` — 返回该物料的默认参数
  - `fn schema() -> Array` — 返回参数元数据（label、min、max、step、widget）
- **AND** 若缺失任一函数，加载 SHALL 失败并返回明确的错误信息

#### Scenario: 物料求值输出 Element 列表

- **WHEN** 调用 `MaterialRuntime::evaluate(material_id, params, screen)`
- **THEN** 系统 SHALL 执行物料脚本的 `build` 函数
- **AND** 返回值 MUST 是 `Vec<Element>`
- **AND** 若脚本抛出异常，求值 SHALL 返回错误，错误信息 MUST 包含物料 id 与异常消息

### Requirement: 物料按 id 引用，内置物料嵌入二进制

每个物料 MUST 有唯一的 id。内置物料通过 `include_str!` 嵌入二进制，id 形如 `builtin.<name>`（如 `builtin.cross`、`builtin.ring`）。用户物料存放在 `%APPDATA%/Peregrine/materials/<name>.rhai`，id 形如 `user.<name>`。

启动时系统 MUST 扫描用户物料目录并合并到物料注册表。当用户物料与内置物料同名时，用户物料 MUST 优先（覆盖内置）。

#### Scenario: 内置物料可被引用

- **WHEN** 图层的 `material` 字段为 `MaterialRef::Builtin { id: "builtin.cross" }`
- **THEN** 系统 SHALL 从二进制内嵌的物料库中加载 `cross.rhai` 并求值

#### Scenario: 用户物料优先级高于内置

- **WHEN** 用户在 `%APPDATA%/Peregrine/materials/cross.rhai` 放置了一份自定义 cross 物料
- **AND** 图层引用 `builtin.cross`
- **THEN** 系统 SHALL 实际加载用户物料版本的 `cross.rhai` 进行求值
- **AND** 日志中 MUST 记录"用户物料 builtin.cross 已被 user.cross 覆盖"

#### Scenario: 用户物料脚本加载失败不崩溃

- **WHEN** 用户物料脚本有语法错误或运行时异常
- **THEN** 系统 SHALL 记录警告日志并跳过该物料
- **AND** 引用该物料的图层在求值时返回空 Element 列表（不绘制任何内容）
- **AND** 不影响其他物料和图层的正常工作

### Requirement: 物料参数 schema 驱动 UI 控件生成

物料脚本通过 `fn schema()` 函数声明其参数元数据。前端配置界面 MUST 根据此 schema 自动生成参数控件，无需为每个物料手写控件代码。

schema 返回值的每个条目 MUST 包含字段：
- `key: String` — 参数名（与 `defaults()` 和 `build(params)` 的 key 对应）
- `label: String` — 显示给用户的本地化标签（中文）
- `widget: String` — 控件类型，取值：`"number"` / `"slider"` / `"color"` / `"select"` / `"toggle"` / `"image_path"` / `"text"`
- `min: Float`（可选，number/slider 时）
- `max: Float`（可选）
- `step: Float`（可选）
- `options: Array<Map>`（可选，select 时，每项 `{value, label}`）
- `default: Any` — 默认值

#### Scenario: number 物料参数生成滑块控件

- **WHEN** 物料 schema 中声明 `{key: "size", widget: "slider", min: 1, max: 200, step: 1, label: "尺寸"}`
- **THEN** 前端配置面板 MUST 渲染一个范围 1-200、步长 1 的滑块控件
- **AND** 控件标签显示"尺寸"
- **AND** 用户调整滑块时，对应图层的 `params.size` 字段更新

#### Scenario: select 物料参数生成下拉框

- **WHEN** 物料 schema 中声明 `{key: "anchor", widget: "select", options: [{value: "top", label: "顶部"}, ...], default: "top"}`
- **THEN** 前端 MUST 渲染一个下拉选择框，列出所有 option 的 label
- **AND** 用户选择后，`params.anchor` 字段更新为对应的 value

#### Scenario: 未知 widget 类型回退

- **WHEN** schema 声明了一个不在支持列表内的 widget 类型（如 `"datetime"`）
- **THEN** 前端 MUST 回退为只读文本显示当前值，并记录警告
- **AND** 不影响其他参数控件的正常生成

### Requirement: 物料求值结果缓存

系统 MUST 缓存物料求值结果以避免每帧重复执行 Rhai 脚本。缓存 key 为 `(material_id, params_hash, screen_hash, dynamic_context_version)`。

`dynamic_context_version` 在动态输入（时间/鼠标/键盘）发生变化时 MUST 递增，使缓存失效；当物料脚本不依赖动态输入时（通过脚本元数据声明），缓存 key 中可省略 `dynamic_context_version`。

#### Scenario: 静态物料缓存命中

- **WHEN** 同一图层在同一参数和屏幕尺寸下被多次求值
- **AND** 物料脚本声明不依赖动态输入
- **THEN** 系统 SHALL 只执行一次 Rhai 脚本
- **AND** 后续求值直接返回缓存结果

#### Scenario: 参数变化使缓存失效

- **WHEN** 用户调整图层的 `size` 参数
- **THEN** 该图层的缓存 MUST 失效
- **AND** 下一次求值 SHALL 执行 Rhai 脚本重新生成 Element 列表

### Requirement: Rhai 脚本沙箱限制

物料脚本运行在沙箱中。系统 MUST 对 Rhai `Engine` 施加以下限制：
- `set_max_operations(N)`：限制单次求值总操作数，防止死循环
- `set_max_call_levels(M)`：限制递归深度
- `disable_module_resolution()`：禁止脚本 `import` 外部模块（用户物料之间不互相依赖）
- 禁止访问文件系统、网络、进程、环境变量

内置的 host function（`time_ms`、`mouse_pos` 等）MUST 通过 `Engine::register_fn` 显式注入，而非通过全局变量暴露。

#### Scenario: 死循环物料被终止

- **WHEN** 物料脚本包含 `loop {}` 无限循环
- **THEN** 求值 SHALL 在达到 `max_operations` 限制时返回错误
- **AND** overlay 渲染不被阻塞超过 50ms

#### Scenario: 脚本无法访问文件系统

- **WHEN** 物料脚本尝试调用任何文件 IO 操作
- **THEN** 求值 SHALL 返回"未定义函数"错误
- **AND** 实际文件系统状态不变

### Requirement: 12 种现有 CrosshairStyle 翻译为内置物料

系统 MUST 提供 12 份内置 Rhai 物料脚本，对应现有 `CrosshairStyle` 的全部变体：

| 内置物料 id | 对应旧 style |
|---|---|
| `builtin.edge_rect` | `EdgeRect` |
| `builtin.cross` | `Cross` |
| `builtin.large_cross` | `LargeCross` |
| `builtin.corner_dots` | `CornerDots4/6/8`（通过 `count` 参数区分） |
| `builtin.ring` | `Ring` |
| `builtin.custom_orb` | `CustomOrb` |
| `builtin.random_orb` | `RandomOrb` |
| `builtin.border_frame` | `BorderFrame` |
| `builtin.edge_arrows` | `EdgeArrows` |
| `builtin.grid` | `Grid` |
| `builtin.image` | `CustomImage` |

每份物料的 `build(params, screen)` 输出 MUST 与现有 `crates/peregrine/src/shapes.rs::build_shapes` 对应分支的输出**逐元素、逐字段相等**（在参数等价映射下）。

#### Scenario: builtin.cross 输出与旧 build_shapes 一致

- **WHEN** 给 `builtin.cross` 物料传入 `params = {size: 24, thickness: 2, gap: 4}` 和屏幕区域 `(0, 0, 1920, 1080)`
- **AND** 用相同参数调用旧 `build_shapes`
- **THEN** 两者返回的 Element 列表 MUST 逐元素相等（顺序、类型、坐标都一致）

#### Scenario: RandomOrb 物料保持种子一致性

- **WHEN** `builtin.random_orb` 物料被调用时
- **THEN** 其内部使用的随机数生成器 MUST 与旧 `SimpleRng`（`shapes.rs:727-745`）使用相同的 LCG 常数和种子计算方式
- **AND** 同一参数生成的随机球位置与旧实现完全一致

#### Scenario: CustomImage 物料仍由渲染器单独处理

- **WHEN** 图层引用 `builtin.image` 物料且参数含 `image_path`
- **THEN** 物料脚本 SHALL 返回单个 `Element::Image` 图元
- **AND** 渲染器 MUST 复用现有 PNG 加载/缓存逻辑（`overlay_renderer.rs::ensure_image_loaded`）
