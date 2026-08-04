# 提案：集成 GlitchTip 匿名崩溃上报 + 启动统计

## Why

Peregrine 目前已发布 v0.1.0 稳定版并分发给真实用户，但缺乏任何线上可观测手段：用户遇到崩溃/异常时开发者无从得知，也无法估算真实安装量来判断投入方向。已自建的 GlitchTip 服务（glitchtip.onemue.cn）具备接收能力，本次接入客户端 SDK，以**匿名、可关闭、透明可审计**的方式实现崩溃上报与启动统计。

## What Changes

- **接入 sentry SDK（Rust `sentry` crate + 前端 `@sentry/react`）**，指向自建 GlitchTip，单实例单 DSN，用 Level + tag（`event_type` / `priority`）区分崩溃、关键路径错误、启动统计三类事件
- **Rust panic 适配 `panic = "abort"`**：注册 panic hook 将崩溃信息**同步落盘**到本地 pending 存储（不依赖网络 flush、不依赖 catch_unwind），经用户授权或开关开启后上传
- **新增 `safe_try!` 宏**（错误上报语义，非 panic 捕获）：包装返回 Result/Option 的关键路径（IO / 渲染 / 窗口桥接 / 外部调用），失败时自动携带函数名 / 文件 / 行号上报，不依赖 `catch_unwind`（release 的 abort 模式下失效）
- **匿名启动统计**：启动后上报一条 `Info` 级事件（不进 issue 列表），携带 `install_id` / `version` / `os` / `arch` tag，用于按安装去重估算用户量
- **install_id 独立文件**：`%APPDATA%/Peregrine/install_id`（UUID v4，原子写），与 config.json 解耦，配置重置/导出/分享不带走
- **隐私开关系统**：首次启动弹窗（默认勾选、取消则零网络请求，此后不再出现任何授权提示）；设置页 `telemetry_enabled` 开关（复用 AppSettings/config.json 体系，修改后弹窗「是否立即重启？」——立即重启/稍后重启）；本地历史错误存储（pending 存储，5MB 轮转，非超限不主动删除）；临时授权（遥测关闭时错误仅落盘，前端报错页面提供「匿名上传错误报告」按钮，一次性上传当前错误 + 全部历史记录后关闭 SDK 不继续上报；开关开启时启动后无感静默上传历史）
- **编译期禁用**：支持 `PEREGRINE_DISABLE_TELEMETRY` 环境变量构建完全无上报功能的二进制（遥测模块编译为 no-op，与运行时开关独立）
- **前端上报出口**：复用已有 `ErrorBoundary.tsx` 与 `globalErrorToast.ts`，挂载 Sentry 上报，不做新的捕获层
- **上报 CODE 体系**：所有上报事件（启动/崩溃/错误）统一携带 `code` tag，不同事件来源使用不同 Code；生成专属 `REPORT_CODES.md` 文档（Code → 含义 → 触发位置 → 处理建议），按码针对性排查处理；Code 常量集中于遥测模块，新增上报点必须先登记；启动事件为首条 Code（`PGR-0001` = app_startup）
- **DSN 注入与保密**：Rust 侧 `option_env!` 构建时注入，前端 Vite `.env.*` 按 mode 加载；补全 `.gitignore` 忽略 `.env.development` / `.env.production` / `.env.*.local`；DSN 不进 git 历史/源码；未配置 DSN 时 SDK 不初始化、零网络请求
- **设置页新增「测试上报」按钮**（仅开发者模式/开发构建可见，普通用户隐藏）：产生一条 Error 级测试事件便于验证链路

## Capabilities

### New Capabilities

- `telemetry-consent`: 隐私授权与开关体系——首次启动弹窗（唯一授权提示）、设置页开关（弹窗确认立即/稍后重启）、本地历史错误存储（5MB 轮转）与报错页面临时授权、编译期禁用、匿名化保证（beforeSend 脱敏）
- `crash-reporting`: 崩溃与异常上报——Rust panic hook 落盘 + 授权全量上传、`safe_try!` 关键路径错误上报（abort 兼容）、React ErrorBoundary / 全局错误挂接 Sentry、上报 CODE 标记与专属文档、开发者模式测试上报
- `startup-metrics`: 匿名启动统计——install_id 生命周期管理（独立文件、原子写、损坏重建）、Info 级启动事件（install_id/version/os/arch tag + 启动 Code `PGR-0001`）

### Modified Capabilities

（无——现有 spec 不涉及遥测相关需求变更）

## Impact

- **Rust 依赖**：新增 `sentry`、`uuid`、`function-name`（workspace 级声明）；复用已有 `reqwest`（备选双项目裸 POST 路径）
- **前端依赖**：新增 `@sentry/react`
- **受影响代码**：
  - `src-tauri/src/lib.rs`（SDK 初始化、panic hook 注册、pending 历史无感上传、启动统计上报）
  - `src-tauri` 新增遥测模块（install_id 管理、pending 存储读写与 5MB 轮转、脱敏函数、`safe_try!` 宏）
  - `crates/config/src/schema.rs`（AppSettings 增加 `telemetry_enabled` 字段，`#[serde(default)]` 兼容旧配置）
  - `src/App.tsx`（隐私开关 UI、测试上报按钮、首次启动弹窗）
  - `src/components/ErrorBoundary.tsx`、`src/lib/globalErrorToast.ts`（挂接上报出口）
  - `src/main.tsx`（前端 Sentry.init）
  - `.gitignore`（补全 `.env.*` 忽略规则）
  - CI release 流程（注入 `GLITCHTIP_DSN` 环境变量）
- **硬性约束**：根 `Cargo.toml` 的 `panic = "abort"` **保持不变**；所有上报匿名（不采集 IP / 用户名 / 机器名 / 路径用户名 / 设备 ID）
- **行为兼容**：未配置 DSN 或开关关闭时应用行为与现状完全一致（零网络请求）
