## ADDED Requirements

### Requirement: 上报 Code 集中登记文档

仓库根 SHALL 提供开发者向的 `REPORT_CODES.md`，集中登记所有遥测上报 Code（`PGR-0xxx` 启动 / `PGR-1xxx` panic / `PGR-2xxx` 后端 IO / `PGR-3xxx` 前端 / `PGR-4xxx` 遮盖层 / `PGR-5xxx` 操作域）。文档 MUST 与遥测模块常量定义（Rust `telemetry.rs::report_code` / 前端 `src/lib/telemetry.ts::REPORT_CODES`）对齐，每条 Code 列出语义、触发点、接线状态（已接线 / 预留 / 豁免）。新增上报点 MUST 先在本文档与常量模块登记后方可合入。

#### Scenario: 文档与代码常量对齐

- **WHEN** 开发者查阅 `REPORT_CODES.md`
- **THEN** 文档中登记的每个 Code MUST 在 `report_code` 模块或 `REPORT_CODES` 常量中存在对应定义
- **AND** 已接线的 Code MUST 标注其触发点（函数名 / 文件 / 行号域）

#### Scenario: 新增上报点先登记

- **WHEN** 开发者准备新增一个上报点
- **THEN** MUST 先在 `REPORT_CODES.md` 与对应常量模块登记 Code 常量
- **AND** 而后方可编写 `safe_try!` / `capture_message` 调用

### Requirement: 用户向隐私说明文档

文档站 SHALL 提供面向用户的隐私说明（`docs/guide/privacy.md` 及中文镜像 `docs/zh-cn/guide/privacy.md`），覆盖：数据匿名保证（不采集 IP / 用户名 / 机器名 / 路径用户名 / 设备 ID）、上报三类事件（启动统计 / 崩溃 / 关键路径错误）的说明、开关位置与临时授权入口行为、`PEREGRINE_DISABLE_TELEMETRY` 编译期禁用选项（面向自构建用户）。该文档 MUST 在 VitePress 侧边栏注册。

#### Scenario: 用户查阅隐私说明

- **WHEN** 用户打开文档站并从侧边栏进入隐私说明
- **THEN** 文档 MUST 明确列出采集的数据维度（install_id / version / os / arch / 崩溃栈 / 错误函数位置）与不采集的维度（IP / 用户名等）
- **AND** MUST 指明设置页开关位置与「修改后重启生效」语义

#### Scenario: 隐私文档无死链

- **WHEN** 执行 `npm run docs:build`
- **THEN** 构建成功且隐私文档条目可达、无死链警告

### Requirement: 开发文档遥测章节

`docs/guide/development.md` SHALL 补充遥测开发章节，覆盖：DSN 环境变量配置（`.env.development` / `.env.production` / `GLITCHTIP_DSN` / `GLITCHTIP_DSN_TEST`）、本地调试（无 DSN 时 SDK 不初始化、`PEREGRINE_DISABLE_TELEMETRY` 编译期禁用）、`safe_try!` 使用约定（仅关键路径，禁止滥用）、Code 登记治理流程。

#### Scenario: 开发者按文档配置本地 DSN

- **WHEN** 开发者按 `development.md` 遥测章节创建 `.env.development` 并填入测试 DSN
- **THEN** `npx tauri dev` 启动后前端 Sentry 与 Rust sentry 初始化均生效
- **AND** 测试事件上报至测试项目

### Requirement: AGENTS.md 遥测模块章节

`AGENTS.md` SHALL 新增「遥测模块」章节，覆盖：模块定位（`src-tauri/src/telemetry.rs` + 前端 `src/lib/telemetry.ts`）、DSN 注入方式（`option_env!` / Vite `.env.*`）、隐私开关语义（`telemetry_enabled` 首次缺省 = 未授权，首次启动弹窗为唯一授权提示）、`safe_try!` 约定（仅关键路径）、Code 治理约定、编译期禁用方式（`PEREGRINE_DISABLE_TELEMETRY`）。

#### Scenario: AI 代理依据 AGENTS.md 理解遥测约定

- **WHEN** AI 代理阅读 `AGENTS.md` 遥测章节
- **THEN** MUST 能定位遥测模块文件、DSN 注入点、开关字段
- **AND** MUST 明确新增上报点需先登记 Code 常量与 `REPORT_CODES.md`
