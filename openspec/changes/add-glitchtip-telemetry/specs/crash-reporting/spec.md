# 能力规格：crash-reporting（崩溃与异常上报）

## ADDED Requirements

### Requirement: Rust panic 同步落盘（abort 兼容）

系统 SHALL 注册自定义 panic hook，在 panic 时将崩溃记录**同步**写入本地 pending 存储（与 install_id 同目录），随后保留默认 eprintln 输出。落盘内容 MUST 包含：时间戳、应用版本、install_id、panic 消息与堆栈摘要（脱敏后）。hook 内 MUST NOT 初始化 SDK、弹窗或发起网络请求；落盘 MUST 使用原子写（临时文件 + rename），保证 `panic = "abort"` 下进程终止前数据必达。

#### Scenario: panic 后记录已落盘

- **WHEN** 应用发生 Rust panic 并触发 abort
- **THEN** pending 存储中已存在该条记录且包含时间戳/version/install_id/panic 消息
- **AND** 崩溃全程无网络请求

#### Scenario: 落盘内容已脱敏

- **WHEN** panic 消息中包含用户目录绝对路径（如 `C:\Users\xxx\...`）
- **THEN** 落盘文件中的路径用户部分被替换为 `{user}` 占位符

### Requirement: 崩溃事件标记与分类

崩溃事件 MUST 使用 `Error` 或 `Fatal` 级别并进入 GlitchTip issue 列表，携带 tag `event_type=crash` 与 `priority`（p0=致命 / p1=高）。关键路径错误（safe_try! 上报）MUST 携带 tag `event_type=error`、`priority=p2`。

#### Scenario: 崩溃事件进入 issue 列表

- **WHEN** 一条崩溃记录被授权上传
- **THEN** 事件级别为 Error/Fatal 并出现在 GlitchTip issue 列表
- **AND** 事件携带 `event_type=crash` 与 `priority` tag

### Requirement: 上报 CODE 体系

所有上报事件（启动/崩溃/错误）MUST 携带 `code` tag，不同事件来源 MUST 使用不同的 Code。Code MUST 在仓库根 `REPORT_CODES.md` 中唯一登记，每条登记 MUST 包含：Code、含义、触发位置（模块/场景）、处理建议。Code 常量 MUST 集中定义于遥测模块，禁止散落硬编码；新增上报点 MUST 先在该文档登记 Code 后方可合入。

#### Scenario: 事件可按 code 过滤

- **WHEN** 任意事件（启动/崩溃/错误）被上报至 GlitchTip
- **THEN** 事件携带 `code` tag（格式 `PGR-<类别><序号>`）
- **AND** 在 GlitchTip 中可按该 tag 筛出同一来源的全部事件

#### Scenario: 文档条目与代码一致

- **WHEN** 查阅 `REPORT_CODES.md` 中任一 Code 条目
- **THEN** 代码中存在对应的集中定义常量与上报点
- **AND** 条目包含含义、触发位置与处理建议

#### Scenario: 未登记 Code 禁止合入

- **WHEN** 新增一个携带 `code` 的上报点
- **THEN** 该 Code 已在 `REPORT_CODES.md` 登记且未与既有码重复，否则不予合入

#### Scenario: 启动事件携带启动 Code

- **WHEN** 上报 Info 级启动统计事件
- **THEN** 该事件携带启动专属 Code（`PGR-0001`）

### Requirement: safe_try! 关键路径错误宏

系统 SHALL 提供 `safe_try!` 宏，用于包装返回 Result/Option 的关键路径调用，覆盖以下故障域：后端通用 IO（配置文件、贴图）、遮盖层（overlay 渲染、Win32 窗口设置与跟随）、Tauri command 操作域（配置/图层/覆盖层/更新/遥测操作）。失败时宏 MUST 自动携带函数名（`function_name!`）、文件与行号（`#[track_caller]` / `Location::caller()`）上报一条 `Error` 级事件，并原样返回 Err/None 供调用方降级处理。宏实现 MUST NOT 依赖 `catch_unwind`（release 的 abort 模式下失效），MUST 在 release 构建下正常工作。宏 MUST NOT 被用于包装所有方法，仅限关键路径；纯 getter 类方法可豁免。

#### Scenario: 关键路径失败自动上报位置信息

- **WHEN** 被 `safe_try!` 包装的调用返回 Err
- **THEN** 上报事件包含函数名、文件名、行号 tag 及错误信息
- **AND** 宏原样返回 Err，调用方继续降级处理

#### Scenario: release 构建下宏正常工作

- **WHEN** 应用以 release 模式（`panic = "abort"`）构建且关键路径调用失败
- **THEN** 错误事件仍被正常上报（不依赖 catch_unwind）

#### Scenario: 成功路径无额外开销上报

- **WHEN** 被 `safe_try!` 包装的调用返回 Ok
- **THEN** 不产生任何上报事件，直接返回 Ok 值

### Requirement: React 错误上报出口

系统 SHALL 复用已有 `ErrorBoundary.tsx`（componentDidCatch）与 `globalErrorToast.ts`（window.onerror / unhandledrejection）挂接 Sentry 上报出口，ErrorBoundary 上报 MUST 携带组件名 tag。前端 SDK 初始化 MUST 按 `import.meta.env.DEV` 选择 TEST/正式 DSN，`autoSessionTracking` 必须为 false，并配置 beforeSend 匿名化。可提供带 name 的 SafeBoundary 高阶组件包装关键组件（可选）。

#### Scenario: 组件渲染错误携带组件名上报

- **WHEN** 被 ErrorBoundary 包裹的组件抛出渲染错误
- **THEN** 错误事件被上报且 tag 包含组件名
- **AND** ErrorBoundary 原有的降级 UI 行为不变

#### Scenario: 全局未捕获错误上报

- **WHEN** 前端发生 window.onerror 或未处理的 Promise rejection
- **THEN** globalErrorToast 的上报出口将错误发送至 Sentry

#### Scenario: 前端无 DSN 或开关关闭时零请求

- **WHEN** 前端构建未注入 DSN 或 `telemetry_enabled` 为 false
- **THEN** 前端 SDK 不初始化，不产生遥测网络请求

### Requirement: 手动测试上报（开发者模式解锁后可见）

系统 SHALL 在开发者模式已解锁（设置窗口「关于」Tab 连点版本号 5 次解锁，或开发构建 `import.meta.env.DEV`）时于设置页「开发」Tab 提供「测试上报」按钮，触发一条 `Error` 级测试事件（进入 issue 列表），用于验证上报链路连通性。未解锁的正式构建 MUST NOT 向普通用户显示该按钮；构建未注入遥测 DSN 时即使已解锁也不显示。

#### Scenario: 开发者模式下测试按钮产生测试事件

- **WHEN** 用户在已解锁开发者模式的设置页「开发」Tab 点击「测试上报」按钮且遥测已启用

- **THEN** GlitchTip 收到一条 Error 级测试事件并进入 issue 列表

#### Scenario: 未解锁对普通用户隐藏测试按钮

- **WHEN** 普通用户使用未解锁的正式构建打开设置页

- **THEN** 设置页不显示「开发」Tab，也不显示「测试上报」按钮
