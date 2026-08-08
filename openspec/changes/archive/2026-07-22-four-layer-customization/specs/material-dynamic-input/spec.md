## ADDED Requirements

### Requirement: 物料脚本可读取实时时间

系统 SHALL 向 Rhai 脚本注入 `time_ms()` host function，返回自系统启动以来的毫秒数（`u64`）。物料脚本可据此实现时钟、闪烁、呼吸动画等动态效果。

时间来源 MUST 使用 `std::time::Instant::now()` 相对于某个固定起点的差值，保证单调递增。

#### Scenario: 物料实现闪烁效果

- **WHEN** 物料脚本调用 `time_ms()` 获取当前时间
- **AND** 基于时间计算某个图元的不透明度（如 `opacity = 0.5 + 0.5 * sin(time_ms() / 500.0)`）
- **THEN** 该图元在 overlay 实时渲染时每帧的 opacity 都不同
- **AND** 表现为视觉上的闪烁效果

#### Scenario: 预览时时间快照

- **WHEN** 前端通过 IPC 调用 `build_shapes` 预览一个依赖 `time_ms()` 的动态物料
- **THEN** 系统 SHALL 使用调用瞬间的真实时间求值
- **AND** 返回结果反映调用瞬间的时间状态（即一次"快照"）
- **AND** 前端预览界面 MUST 在物料标签上显示"动态物料 - 预览为快照"提示

### Requirement: 物料脚本可读取鼠标位置

系统 SHALL 注入 `mouse_pos()` host function，返回鼠标当前位置的 `Map`，含 `x: Float` 与 `y: Float` 字段（逻辑屏幕坐标）。

鼠标位置来源：在 overlay 活跃时由渲染线程通过 `WindowEvent::CursorMoved` 或 Win32 API（`GetCursorPos`）轮询获取，更新频率不低于 60Hz。

#### Scenario: 物料跟随鼠标

- **WHEN** 物料脚本调用 `mouse_pos()` 并据此调整图元坐标
- **THEN** overlay 渲染时图元位置随鼠标移动而实时变化

#### Scenario: 鼠标位置在预览中映射到 Canvas 内

- **WHEN** 前端预览一个依赖 `mouse_pos()` 的动态物料
- **THEN** 系统 SHALL 返回鼠标在预览 Canvas 内的相对坐标（按预览尺寸到全屏尺寸的比例缩放）
- **AND** 若鼠标不在预览 Canvas 内，返回 Canvas 中心坐标作为默认值

### Requirement: 物料脚本可读取键盘状态

系统 SHALL 注入 `key_down(key_code: String) -> Bool` host function，查询指定按键当前是否被按下。支持的按键代码 MUST 至少包含：
- 字母键：`"a"` 到 `"z"`
- 数字键：`"0"` 到 `"9"`
- 修饰键：`"shift"`、`"ctrl"`、`"alt"`、`"super"`
- 方向键：`"up"`、`"down"`、`"left"`、`"right"`
- 功能键：`"f1"` 到 `"f12"`
- 空格、回车、ESC、Tab

键盘状态来源：通过 `tauri-plugin-global-shortcut` 或 Win32 API（`GetAsyncKeyState`）维护当前按键状态表。

**重要安全约束**：键盘状态查询仅用于视觉效果（如显示按键提示），系统 MUST NOT 记录或持久化任何按键序列。日志中禁止打印具体按键值（只允许打印按键数量或聚合统计）。

#### Scenario: 物料响应按键显示提示

- **WHEN** 物料脚本调用 `key_down("shift")` 判断 Shift 是否按下
- **AND** Shift 按下时绘制额外的提示图元
- **THEN** overlay 渲染时用户按下 Shift 立即看到提示出现

#### Scenario: 键盘状态不写入日志

- **WHEN** 物料脚本调用 `key_down("a")`
- **THEN** tracing 日志中 MUST NOT 出现具体按键代码 `"a"`
- **AND** 仅允许记录"查询了 1 次键盘状态"这类聚合信息

### Requirement: 物料脚本可获取随机数

系统 SHALL 注入以下随机数 host function：
- `rand()`：返回 `[0.0, 1.0)` 区间的伪随机 `Float`
- `rand_range(min: Float, max: Float)`：返回 `[min, max)` 区间的伪随机 `Float`
- `rand_seed(seed: Int)`：设置 RNG 种子（影响后续 `rand()` 调用）

RNG 实现 MUST 与现有 `SimpleRng`（`crates/peregrine/src/shapes.rs:727-745`）使用相同的 LCG 算法和常数，保证 `builtin.random_orb` 物料与旧实现产生相同的随机球布局。

每个物料求值 SHALL 拥有独立的 RNG 状态（不跨物料共享），初始种子由 `(material_id, params_hash, dynamic_context_version)` 派生。

#### Scenario: 同一参数产生相同随机序列

- **WHEN** 同一物料在同一参数下多次求值
- **THEN** 每次求值产生的随机数序列 MUST 完全一致（确定性随机）

#### Scenario: 不同参数产生不同随机序列

- **WHEN** 物料在参数 `size: 24` 和 `size: 25` 下分别求值
- **THEN** 两次产生的随机数序列 MUST 不同

### Requirement: 动态输入可扩展

动态输入 host function 集合 MUST 设计为可扩展。新增输入源（如手柄状态、音频电平）时，只需在 `crates/material` 中新增 `Engine::register_fn` 注册，无需修改 schema 或渲染器。

`crates/material` MUST 导出 `DynamicContext` 结构体，聚合所有动态输入的当前快照。物料求值时传入此结构体，host function 内部读取其字段。

#### Scenario: 新增动态输入不影响既有物料

- **WHEN** 在未来版本中新增 `gamepad_button(button: String) -> Bool` host function
- **THEN** 现有的 12 个内置物料（不调用此函数）行为 MUST 保持不变
- **AND** 现有的用户物料脚本 MUST 无需修改即可继续工作

#### Scenario: 不可用的动态输入返回安全默认值

- **WHEN** 在非 Windows 平台上调用 `mouse_pos()`（鼠标位置 API 不可用）
- **THEN** 系统 SHALL 返回屏幕中心坐标作为默认值
- **AND** 求值不报错
- **AND** 日志记录"mouse_pos 在当前平台不可用，返回默认值"
