# overlay-dynamic-rendering Specification

## MODIFIED Requirements

### Requirement: 动态物料驱动的持续重绘

当 active profile 的任一**可见**图层引用的物料 `is_dynamic() == true`，且动态开关（编译期 AND 运行时）均开启时，overlay 事件循环 MUST 以 `settings.material.fps` 推导的帧间隔持续重绘；当所有可见图层均为静态物料或任一开关关闭时，overlay MUST 进入 `ControlFlow::Wait`，仅在配置变更、窗口事件或显式 Invalidate 时重绘。

帧间隔推导：`fps` 为 `Some(30/60/120)` 时用固定节拍；`None` 时跟随系统主屏刷新率（winit `refresh_rate_millihertz()`，异常值回退 60）。帧间隔 MUST 在 `UpdateConfig` 时热更新。

**变更说明**：

- 原「约 60FPS」硬编码改为配置推导节拍。
- 原「动态性判定结果 MUST 缓存（dynamic_dirty）」要求**废除**：判定改为每轮 `about_to_wait` 直接计算（一次配置锁 + 逐图层 registry 查找，微秒级），消除缓存失效点位。
- 原规格头部「软关闭期间不可达」注记废除（动态链路本 change 起默认活跃）。

#### Scenario: 含时钟物料的 profile 持续刷新

- **WHEN** active profile 包含引用 `builtin.time` 的可见图层，动态开关双开，overlay 正在运行
- **THEN** 时钟文本每秒自动更新，无需任何窗口交互，调度节拍为配置帧率

#### Scenario: 纯静态 profile 不空转

- **WHEN** active profile 所有可见图层均为静态物料（如 `builtin.cross`）
- **THEN** 事件循环进入 `ControlFlow::Wait`，无周期性重绘，CPU 占用与软关闭期间一致

#### Scenario: 切换 profile 后动态性即时生效

- **WHEN** overlay 运行中从纯静态 profile 切换到含动态物料的 profile（或反之）
- **THEN** 重绘调度行为在下一次 `about_to_wait` 即切换，无需重启 overlay

#### Scenario: 帧率档位切换热生效

- **WHEN** overlay 以 120FPS 节拍运行中，用户将帧率改为 30
- **THEN** 后续调度按 30FPS 节拍，无需重启 overlay

### Requirement: 物料热重载联动

物料 watcher 重建 registry 后 MUST 做两件事：经事件循环代理向 overlay 线程发送携带新 registry 句柄的 `OverlayCommand::RefreshMaterials`；向前端广播 `peregrine:materials-changed` 事件。overlay 侧处理 `RefreshMaterials` 时 MUST 替换自身持有的 registry 句柄并触发重绘；动态性判定因每轮直接计算，MUST 无需额外失效标记即反映新物料的 `is_dynamic`。

#### Scenario: 热重载改变动态性即时生效

- **WHEN** 用户物料目录中某物料被修改（静态改动态或反之），watcher 重建 registry
- **THEN** overlay 在下一次 `about_to_wait` 反映新的 `is_dynamic`，无需重启
- **AND** 前端物料列表经 `peregrine:materials-changed` 自动刷新

## ADDED Requirements

### Requirement: 内置时间物料使用上下文时间

`builtin.time`（时间显示物料）的 `build()` MUST 通过 `time_ms()`（`DynamicContext` 快照）获取时间，MUST NOT 使用 `now_ms()` 直读墙钟——保证 overlay（每帧轮询时刻）与预览（请求时刻）的时间来源一致受调度约束。`now_ms()` host function 保留注册以兼容既有用户脚本，但文档标注为不推荐。

#### Scenario: 时间物料随注入上下文求值

- **WHEN** 以固定 `DynamicContext { time_ms: T, .. }` 求值 `builtin.time`
- **THEN** 输出文本对应的时刻为 T，而非求值发生的真实墙钟时刻

### Requirement: 预览动态物料实时刷新

前端预览（`Preview.tsx`）在 profile 含 `is_dynamic` 物料且动态开关双开时，MUST 以约 1s 间隔定时重拉 `build_shapes_ipc` 刷新画布；条件不满足（纯静态 profile / 任一开关关闭）或组件卸载时 MUST 清除定时器，维持事件驱动刷新。

#### Scenario: 预览中时钟跳动

- **WHEN** 图层编辑器中 active profile 含 `builtin.time` 图层，动态开关双开
- **THEN** 预览画布中的时钟每秒刷新，与 overlay 显示的时刻一致（±1s）

#### Scenario: 静态 profile 预览无定时器

- **WHEN** active profile 全部图层为静态物料
- **THEN** 预览仅在配置变化时刷新，无周期性 IPC 请求
