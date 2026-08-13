# developer-mode Specification

## Purpose

定义 Peregrine 的开发者模式能力：通过「关于」Tab 版本号连点彩蛋解锁隐藏的「开发」Tab（仅含 DevTools 与测试上报入口），并控制 WebView DevTools 的启停。解锁状态持久化到配置文件，重启后保持。

## Requirements

### Requirement: 开发者模式解锁（连点版本号）

系统 SHALL 在设置窗口「关于」Tab 的版本号上提供连续点击解锁交互：连续点击 5 次（相邻点击间隔 < 1.5 秒，超时计数清零）后解锁开发者模式；自第 3 次点击起 SHALL 显示剩余次数提示。解锁状态 MUST 持久化到配置文件（`AppSettings.developer_mode = true`），应用重启后保持。解锁成功后 SHALL 显示临时成功提示，并注明 DevTools 需重新打开窗口后可用。

#### Scenario: 连点 5 次解锁

- **WHEN** 用户在设置窗口「关于」Tab 连续点击版本号 5 次且每次间隔小于 1.5 秒
- **THEN** 开发者模式解锁，`developer_mode` 写入配置文件，并显示解锁成功提示

#### Scenario: 点击间隔超时计数清零

- **WHEN** 用户点击版本号后超过 1.5 秒未继续点击且未满 5 次
- **THEN** 点击计数清零，不解锁

#### Scenario: 解锁状态重启后保持

- **WHEN** 用户此前已解锁开发者模式并重启应用
- **THEN** 设置窗口仍显示「开发」Tab，无需重新解锁

#### Scenario: 开发构建自动解锁

- **WHEN** 应用以开发构建（`import.meta.env.DEV`）运行
- **THEN** 无论 `developer_mode` 取值，「开发」Tab 均直接显示

### Requirement: 「开发」Tab（仅 DevTools 与测试上报）

系统 SHALL 在开发者模式已解锁（或开发构建）时于设置窗口显示第 6 个 Tab「开发」。该 Tab MUST 仅包含两个功能入口：「开启 DevTools」按钮与「测试上报」按钮；MUST NOT 包含日志查看、配置 JSON 展示、重置配置等其他功能。

#### Scenario: 解锁后出现「开发」Tab

- **WHEN** 开发者模式已解锁且用户打开设置窗口
- **THEN** Tab 栏显示「开发」Tab，内含「开启 DevTools」与「测试上报」两个按钮

#### Scenario: 未解锁时无「开发」Tab

- **WHEN** 开发者模式未解锁且非开发构建
- **THEN** 设置窗口仅显示原有的 5 个 Tab

### Requirement: DevTools 默认禁用、解锁后启用

系统 SHALL 在创建配置窗口与设置窗口时按 `developer_mode || cfg!(debug_assertions)` 判定是否启用 WebView DevTools。未解锁的正式构建中，右键「检查」、Ctrl+Shift+I 与程序化 `open_devtools()` MUST 全部不可用；解锁后重新打开窗口 MUST 可用。开发构建 MUST 恒启用 DevTools。

#### Scenario: 未解锁时右键无「检查」

- **WHEN** 普通用户使用未解锁的正式构建，在配置或设置窗口网页区域点击右键
- **THEN** 右键菜单不出现「检查」，Ctrl+Shift+I 无法打开 DevTools

#### Scenario: 解锁后重开窗口 DevTools 可用

- **WHEN** 用户解锁开发者模式后关闭并重新打开设置窗口
- **THEN** 右键「检查」、Ctrl+Shift+I 与「开启 DevTools」按钮均可用

#### Scenario: 解锁当次会话的提示

- **WHEN** 用户刚完成解锁且当前窗口尚未重建
- **THEN** UI 提示 DevTools 需重新打开窗口后生效

### Requirement: 测试上报按钮迁移与可见性

系统 SHALL 将「测试上报」按钮从「通用」Tab 移至「开发」Tab，其可见条件为「『开发』Tab 可见 && 构建注入了遥测 DSN」。点击后 SHALL 触发一条 `Error` 级测试事件（进入 GlitchTip issue 列表）。正式构建在解锁开发者模式后 MUST 同样显示该按钮。

#### Scenario: 解锁后正式构建可测试上报

- **WHEN** 用户在已解锁的正式构建中打开「开发」Tab，点击「测试上报」且遥测已启用
- **THEN** GlitchTip 收到一条 Error 级测试事件并进入 issue 列表

#### Scenario: 无 DSN 构建不显示按钮

- **WHEN** 构建未注入遥测 DSN，即使开发者模式已解锁
- **THEN** 「开发」Tab 不显示「测试上报」按钮

#### Scenario: 「通用」Tab 不再含测试上报

- **WHEN** 用户打开设置窗口「通用」Tab
- **THEN** 该 Tab 不出现「测试上报」按钮

### Requirement: 移除配置窗口 DeveloperPanel

系统 MUST 移除配置窗口的连点版本号彩蛋与整个 DeveloperPanel（开发者日志、config.json 查看、重置配置、DevTools 开关）。图层编辑等处的 `logAction` 埋点调用 MAY 保留（成为无查看器的静默日志）。

#### Scenario: 配置窗口版本号不可点击

- **WHEN** 用户点击配置窗口底部的版本号文本
- **THEN** 无任何解锁交互，不显示开发者面板

#### Scenario: 开发者面板代码移除

- **WHEN** 构建应用
- **THEN** `DeveloperPanel` 组件及其专属 i18n 键（`developer.*`）不再包含于产物中
