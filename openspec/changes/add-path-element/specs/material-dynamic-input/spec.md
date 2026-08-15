# material-dynamic-input Delta：鼠标速度与加速度输入

## ADDED Requirements

### Requirement: 物料脚本可读取鼠标速度与加速度

系统 SHALL 注入以下 host function：

- `mouse_velocity() -> Map { x: Float, y: Float }`：鼠标当前速度（逻辑像素/秒）
- `mouse_acceleration() -> Map { x: Float, y: Float }`：鼠标当前加速度（逻辑像素/秒²）

数值来源：平台轮询器（`poll_dynamic_context`）对鼠标位置做跨采样差分——`vel = (cur - prev) / dt`、`acc = (vel - prev_vel) / dt`，`dt` 取自 `Instant` 实测采样间隔（MUST NOT 假设固定帧间隔）。差分值 MUST 经 EMA 平滑（抑制轮询抖动），平滑系数为常量（实施时标定）。

跨帧状态（上一采样位置/速度/EMA 累积值）MUST 收敛在平台轮询器内部（函数级 `static`），`DynamicContext` 保持无状态快照语义不变——同一次求值内多次调用返回相同值。

#### Scenario: 物料消费速度输入

- **WHEN** 物料脚本调用 `mouse_velocity()` 并以返回值驱动图元参数
- **THEN** 返回值为含 `x` / `y` Float 字段的 Map（逻辑像素/秒），图元随鼠标移动速度变化

#### Scenario: 同一快照内重复调用返回同值

- **WHEN** 同一次物料求值内连续两次调用 `mouse_acceleration()`
- **THEN** 两次返回值完全相同（来自同一 `DynamicContext` 快照）

### Requirement: 速度与加速度死区归零保障稳态跳帧

EMA 平滑值在鼠标静止后渐近衰减、永不精确触零。为保障帧指纹稳态（动态物料静止输入下回到跳帧主路径），速度与加速度 MUST 设置死区：EMA 值幅度低于阈值时 MUST 直接置 `0.0`（而非保留微小残值）。阈值常量（量级：速度 ~5 逻辑像素/秒、加速度 ~50 逻辑像素/秒²，实施时标定）MUST 集中定义在轮询器附近。

#### Scenario: 鼠标静止后速度归零

- **WHEN** 鼠标完全静止若干采样周期后调用 `mouse_velocity()`
- **THEN** 返回值精确为 `{x: 0.0, y: 0.0}`（死区置零，非微小残值）

#### Scenario: 匀速移动时加速度为零

- **WHEN** 鼠标匀速直线移动时调用 `mouse_acceleration()`
- **THEN** 加速度分量在死区内，返回 `{x: 0.0, y: 0.0}`

#### Scenario: 急转时加速度非零

- **WHEN** 鼠标快速变向移动时调用 `mouse_acceleration()`
- **THEN** 返回值的幅度方向反映变向的加速度（非零）

#### Scenario: 静止输入维持跳帧

- **WHEN** 依赖速度/加速度的动态物料在鼠标静止时连续渲染两帧
- **THEN** 两次求值输出完全相同，帧指纹相等，overlay 跳过光栅化

### Requirement: 速度与加速度的预览与降级语义

前端预览的 `preview_snapshot` 上下文 MUST 以 `(0.0, 0.0)` 作为速度与加速度默认值（预览无法感知真实鼠标动力学），依赖加速度的物料在预览中呈现其静止形态。非 Windows 平台的 `poll_dynamic_context` 占位实现 MUST 返回 0 速度/加速度（不报错，与 `mouse_pos` 平台降级先例一致）。

#### Scenario: 预览中加速度物料呈静止形态

- **WHEN** 预览依赖 `mouse_acceleration()` 的物料
- **THEN** 求值上下文速度/加速度为 0，物料输出其静止形态（如正圆）

#### Scenario: 非 Windows 平台安全默认值

- **WHEN** 在非 Windows 平台调用 `mouse_velocity()`
- **THEN** 返回 `{x: 0.0, y: 0.0}`，求值不报错
