# 能力规格：telemetry-consent（隐私授权与开关体系）

## ADDED Requirements

### Requirement: 首次启动授权弹窗

系统 SHALL 在首次启动（配置中无遥测授权记录）时弹出授权对话框，文案明确说明「是否允许匿名上报崩溃信息与使用情况？不收集任何个人数据」，默认勾选允许。文案用词 SHALL 使用「使用情况」（而非「使用统计」）以预留未来功能使用分析（功能埋点）的叙事余地——当前 change 范围仍只覆盖崩溃上报与启动统计，不实现功能使用分析埋点。用户取消时，系统 MUST NOT 初始化遥测 SDK，且 MUST NOT 产生任何遥测网络请求。用户的选择 MUST 持久化到 AppSettings/config.json 体系（`telemetry_enabled` 字段，`#[serde(default)]` 兼容旧配置），且 MUST NOT 再次弹窗。

#### Scenario: 首次启动默认允许

- **WHEN** 用户首次启动应用且配置中无 `telemetry_enabled` 记录
- **THEN** 系统弹出授权对话框且默认为勾选状态
- **AND** 用户确认后 `telemetry_enabled` 写入 true 并初始化 SDK

#### Scenario: 首次启动拒绝

- **WHEN** 用户在首次启动授权对话框中取消勾选/拒绝
- **THEN** `telemetry_enabled` 写入 false，SDK 不初始化
- **AND** 应用本次运行期间零遥测网络请求，功能完全正常

#### Scenario: 非首次启动不重复弹窗

- **WHEN** 配置中已存在 `telemetry_enabled` 字段（true 或 false）
- **THEN** 系统不再弹出授权对话框，直接按配置值决定是否初始化 SDK

### Requirement: 设置页遥测开关

系统 SHALL 在设置页提供 `telemetry_enabled` 开关，复用 AppSettings/config.json 的 validate + 原子写 + 热重载体系。开关修改后 MUST 弹出确认对话框「修改将在重启后生效，是否立即重启？」，提供「立即重启」与「稍后重启」两个选项：选「立即重启」则保存配置并立即重启应用；选「稍后重启」则保存配置并在设置页保留「待重启生效」标记，待用户下次手动重启后生效。运行中切换 MUST NOT 动态启停 SDK。

#### Scenario: 关闭开关后重启停止上报

- **WHEN** 用户在设置页关闭遥测开关并重启应用
- **THEN** SDK 不初始化，应用零遥测网络请求正常运行

#### Scenario: 开启开关后重启恢复上报

- **WHEN** 用户在设置页开启遥测开关并重启应用
- **THEN** SDK 正常初始化，崩溃/错误/启动统计恢复上报

#### Scenario: 弹窗选择立即重启

- **WHEN** 用户修改遥测开关后在确认对话框中选择「立即重启」
- **THEN** 配置保存后应用立即重启，新开关状态即时生效

#### Scenario: 弹窗选择稍后重启

- **WHEN** 用户修改遥测开关后在确认对话框中选择「稍后重启」
- **THEN** 配置已保存，设置页保留「待重启生效」标记
- **AND** 应用继续以旧开关状态运行直至下次重启

### Requirement: 本地历史错误存储与临时授权

当遥测开关关闭时，系统 MUST 仍将错误（Rust panic、safe_try! 关键路径错误、前端错误）落盘到本地 pending 存储（不初始化 SDK、零网络请求）。pending 存储 MUST 容量上限 5MB，超出时删除最旧记录；除此条件外 MUST NOT 主动删除任何记录。临时授权的唯一入口为前端报错页面（ErrorBoundary 降级页）：遥测未开启时该页面 SHALL 显示「匿名上传错误报告」按钮；用户点击后系统初始化 SDK，上传当前错误与全部历史 pending 记录，上传完成后 MUST 关闭 SDK 且不继续上报（一次性显式授权，不修改开关状态）。遥测开启时，系统 SHALL 在启动后无感静默上传全部 pending 历史记录并清除已上传记录，MUST NOT 再次弹窗或要求同意。除首次启动授权弹窗外，系统 MUST NOT 以任何形式再次弹出遥测授权提示。

#### Scenario: 开关关闭时错误落盘不上报

- **WHEN** `telemetry_enabled` 为 false 且发生错误（Rust panic / 关键路径错误 / 前端错误）
- **THEN** 错误记录落盘到本地 pending 存储
- **AND** 全程无 SDK 初始化、无网络请求

#### Scenario: 报错页面临时授权上传全部历史

- **WHEN** 遥测未开启且前端报错页面出现，用户点击「匿名上传错误报告」按钮
- **THEN** 系统初始化 SDK，上传当前错误与全部历史 pending 记录
- **AND** 上传完成后清除已上传记录

#### Scenario: 授权完成后不继续上报

- **WHEN** 临时授权上传完成
- **THEN** 系统关闭 SDK，后续错误仍仅落盘不上报
- **AND** `telemetry_enabled` 开关状态保持不变

#### Scenario: pending 存储超限轮转

- **WHEN** pending 存储总量超过 5MB
- **THEN** 系统删除最旧的记录直至回到上限以内

#### Scenario: 非超限不主动删除

- **WHEN** pending 存储未超过 5MB 且用户未授权上传
- **THEN** 系统不主动删除任何 pending 记录

#### Scenario: 开关开启时无感静默上传历史

- **WHEN** `telemetry_enabled` 为 true 且启动后检测到 pending 历史记录
- **THEN** 系统静默上传全部历史记录并清除，无任何弹窗或同意请求

#### Scenario: 不再出现额外授权提示

- **WHEN** 用户已在首次启动授权弹窗中做出选择（允许或拒绝）
- **THEN** 系统不再以任何形式弹出遥测授权提示

### Requirement: 上报数据匿名化

所有上报事件 MUST 经过脱敏处理：删除 user 字段、server_name、request 头信息；事件中出现的绝对路径 MUST 将用户目录部分替换为占位符（如 `C:\Users\xxx` → `{user}`）。系统 MUST NOT 采集 IP、用户名、机器名、设备 ID、用户输入内容。install_id 为本地随机 UUID，不代表任何真实身份。

#### Scenario: 事件发送前脱敏

- **WHEN** 任意事件（崩溃/错误/启动统计）即将发送
- **THEN** beforeSend 钩子移除 user/server_name/request 信息
- **AND** 事件文本中的用户目录路径被替换为 `{user}` 占位符

### Requirement: 编译期禁用上报

系统 SHALL 支持通过编译前设置环境变量 `PEREGRINE_DISABLE_TELEMETRY` 构建完全无上报功能的程序：遥测模块编译为空操作，二进制不包含任何上报代码路径与网络请求，与遥测相关的 UI（开关、上报按钮）不可用或隐藏。该机制与运行时开关相互独立。

#### Scenario: 编译期禁用构建零上报功能

- **WHEN** 以 `PEREGRINE_DISABLE_TELEMETRY` 环境变量编译并运行程序
- **THEN** 二进制不含任何上报代码路径，全程零遥测网络请求
- **AND** 无论配置中 `telemetry_enabled` 为何值，均不发生任何上报

### Requirement: 未配置 DSN 时零副作用

当构建产物中未注入 DSN（`option_env!` 为空）时，系统 MUST NOT 初始化 SDK，MUST NOT 产生任何遥测网络请求，应用行为与未集成遥测完全一致。

#### Scenario: 无 DSN 构建正常运行

- **WHEN** 应用以未注入 DSN 的方式构建并启动
- **THEN** SDK 不初始化，无遥测网络请求，全部功能正常

### Requirement: DSN 不进入源码与 git 历史

DSN MUST 通过构建时环境变量（Rust：`option_env!("GLITCHTIP_DSN")` / `GLITCHTIP_DSN_TEST`；前端：`VITE_GLITCHTIP_DSN` / `VITE_GLITCHTIP_DSN_TEST`）注入，MUST NOT 硬编码于源码。`.gitignore` MUST 忽略 `.env.development`、`.env.production`、`.env.*.local`。开发构建 MUST 使用 TEST 项目 DSN，正式发布构建 MUST 使用正式项目 DSN。

#### Scenario: 开发构建上报至测试项目

- **WHEN** 通过 `tauri dev` 启动开发构建并触发上报
- **THEN** 事件发送至 GlitchTip 测试项目

#### Scenario: 正式构建上报至正式项目

- **WHEN** 通过 CI `tauri build` 产出正式发布包并触发上报
- **THEN** 事件发送至 GlitchTip 正式项目

#### Scenario: 敏感文件被 git 忽略

- **WHEN** 本地存在 `.env.development` / `.env.production` 文件
- **THEN** `git status` 不显示这些文件，不会进入提交
