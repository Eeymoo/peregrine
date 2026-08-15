# material-settings Specification

## Purpose

定义设置窗口「物料」Tab 的行为：动态物料运行时开关与动画帧率选择器，及其与编译期总闸的双层开关语义、帧率档位语义与序列化兼容要求。

## Requirements

### Requirement: 物料设置 Tab

设置窗口 SHALL 提供「物料」Tab，承载动态物料运行时开关与动画帧率选择器。该 Tab 在物料运行时编译期开关（`MATERIAL_RUNTIME_ENABLED`）关闭时 MUST 整体隐藏。

#### Scenario: Tab 可见性与内容

- **WHEN** 用户打开设置窗口且物料运行时编译期开关开启
- **THEN** 「物料」Tab 可见，包含「动态物料」开关与「动画帧率」选择器
- **WHEN** 物料运行时编译期开关关闭（软关闭构建）
- **THEN** 「物料」Tab 不显示

### Requirement: 动态物料运行时开关

系统 SHALL 提供 `settings.material.dynamic_enabled` 运行时开关（默认 `true`），与编译期总闸 `MATERIAL_DYNAMIC_INPUT_ENABLED` 构成**与门**：动态链路（动态输入轮询、动态物料求值上下文、动态重绘调度、选择器动态物料可见性）仅在两者均为真时活跃。

开关状态变更 MUST 热生效（经配置保存 → overlay `UpdateConfig` 路径），不得要求重启应用或重建 overlay。

关闭开关时系统 MUST 表现为运行时软关闭：物料求值使用 `DynamicContext::static_context()`，overlay 对 layers 的动态性判定恒为 false，物料选择器隐藏 `is_dynamic = true` 的物料，预览停止定时刷新。

#### Scenario: 开关关闭后 overlay 冻结动态物料

- **WHEN** active profile 含引用 `builtin.time` 的可见图层，且用户将动态物料开关从开切为关
- **THEN** 配置保存后 overlay 停止周期重绘，时钟文本冻结在关闭前最后一帧
- **AND** 纯静态图层渲染不受影响

#### Scenario: 开关重新开启后恢复跳动

- **WHEN** 开关从关切回开（overlay 运行中）
- **THEN** 时钟在下一次调度节拍恢复跳动，无需重启 overlay

### Requirement: 动画帧率档位

系统 SHALL 提供 `settings.material.fps` 帧率设置（`Option<u32>`，仅接受 `30` / `60` / `120`）：

- `None`（默认）= 跟随系统主屏刷新率；探测使用 winit `refresh_rate_millihertz()`，探测失败或异常值（< 24 或 > 480 Hz）MUST 回退 60。
- `Some(fps)` = 固定节拍，帧间隔 = `1s / fps`。

FPS 语义为**动画最高帧率节拍（cap）**：纯静态 profile（无动态物料且非 RandomOrb）MUST 保持纯事件驱动（`ControlFlow::Wait`），不因 FPS 设置产生周期性唤醒。帧间隔 MUST 随配置变更热更新。

#### Scenario: 系统档跟随刷新率

- **WHEN** `fps` 缺省（跟随系统）且主屏刷新率为 144Hz
- **THEN** 含动态物料的 overlay 以约 144FPS 节拍调度持续重绘

#### Scenario: 非法值被校验拒绝

- **WHEN** 配置文件中 `fps` 为 `45`
- **THEN** `AppConfig::validate()` 失败，走损坏配置恢复流程（备份 `.bak` + 回退默认）

#### Scenario: 静态 profile 不空转

- **WHEN** `fps = 120` 且 active profile 全部可见图层为静态物料
- **THEN** overlay 事件循环保持 `ControlFlow::Wait`，无周期性重绘

### Requirement: 序列化兼容

`settings.material` 字段 MUST 以 `#[serde(default)]` 引入：旧配置文件（无该字段）加载后 `dynamic_enabled = true`、`fps = None`（跟随系统），不触发迁移或报错。

#### Scenario: 旧配置无感升级

- **WHEN** 加载不含 `material` 字段的既有 `config.json`
- **THEN** 配置加载成功，行为等同默认值，保存时写出完整 `material` 对象
