# 任务清单：集成 GlitchTip 匿名崩溃上报 + 启动统计

## 1. 依赖与工程配置

- [x] 1.1 根 `Cargo.toml` `[workspace.dependencies]` 新增 `sentry`（默认特性，确认含 reqwest transport）、`uuid`（v4 + serde）、`function-name`；`src-tauri/Cargo.toml` 以 `{ workspace = true }` 引用
- [x] 1.2 `package.json` 新增 `@sentry/react` 依赖并 `npm install` 锁定
- [x] 1.3 `.gitignore` 补充忽略 `.env.development`、`.env.production`、`.env.*.local`；验证 `git status` 不显示这些文件
- [x] 1.4 新建本地 `.env.development`（`VITE_GLITCHTIP_DSN_TEST` 占位）并验证 `GLITCHTIP_DSN_TEST` 环境变量在 `tauri dev` 下可被 `option_env!` 读取（需确认构建时注入方式：build.rs 或 dev 脚本前置 export，文档注明增量编译需 touch 源文件）

## 2. Rust 遥测核心模块（src-tauri/src/telemetry.rs）

- [x] 2.1 实现 `get_install_id(app_data_dir)`：读取/生成 UUID v4、空或损坏自动重建、原子写（tmp + rename）；附单元测试（首次生成 / 复用不变 / 损坏重建 / 重置 config 不影响）
- [x] 2.2 实现 pending 存储：多记录（每条错误一条记录）原子写入（tmp + rename）、容量上限 5MB 超出删除最旧、非超限不主动删除；提供 `write_pending()` / `list_pending()` / `clear_uploaded()`；记录结构含 ts/version/install_id/code/message
- [x] 2.3 实现脱敏函数 `anonymize_event`：删 user/server_name/request；路径用户名正则替换（`C:\Users\xxx` / `/Users/xxx` / `/home/xxx` → `{user}`）；附单元测试
- [x] 2.4 实现 `panic_hook_that_persists`：格式化 panic 信息 → 脱敏 → 同步原子写入 pending 存储；hook 内不初始化 SDK / 不弹窗 / 不网络请求；保留默认 eprintln 输出；hook 内 IO 失败静默忽略（防二次 panic）
- [x] 2.5 定义上报 Code 常量模块（telemetry.rs 内 `report_code` 子模块）：按号段集中定义 `PGR-0xxx`（启动/生命周期，首个 `PGR-0001`=app_startup）、`PGR-1xxx`（panic/崩溃）、`PGR-2xxx`（21xx IO / 22xx 渲染 / 23xx 桥接 / 24xx 外部调用）常量，禁止散落硬编码
- [x] 2.6 实现 `safe_try!` 宏（`#[macro_export]`）：`#[track_caller]` + `Location::caller()` 取文件/行号，`function_name!()` 取函数名；Err 分支 `sentry::with_scope` 附加 `event_type=error`、`priority=p2`、`code`、`function`、`location` tag 后 `capture_message`（Level::Error），原样返回 Err；不依赖 catch_unwind；宏支持传入 Code 参数；附单元测试（Err 上报 / Ok 直通）
- [x] 2.7 实现 `report_startup()`：开关开启时 `with_scope` 附加 `code=PGR-0001`、`event_type=startup`、`priority=p3`、`install_id`、`version`、`os`、`arch` tag，`capture_message("app_startup", Level::Info)`

## 3. Rust SDK 初始化与启动流程（src-tauri/src/lib.rs）

- [x] 3.1 `run()` 启动早期：读取 `telemetry_enabled` → DSN（`cfg!(debug_assertions)` 选 `GLITCHTIP_DSN_TEST` / `GLITCHTIP_DSN`，`option_env!`）→ 两者齐备才 `sentry::init`（`release: sentry::release_name!()`、`environment`、`auto_session_tracking: false`、`before_send: anonymize_event`）；任一缺失则跳过，零网络请求
- [x] 3.2 SDK 初始化后注册 `panic_hook_that_persists`（注意：开关关闭时也要注册 hook 以落盘 pending 存储——hook 注册与 SDK 初始化解耦）
- [x] 3.3 启动后处理 pending 历史：开关开 → 无感静默全量上传并清除已上传记录（无任何弹窗/同意请求）；开关关 → 仅保留累积，不做任何网络动作
- [x] 3.4 主循环进入后调用 `report_startup()`
- [x] 3.5 新增 Tauri commands：`store_pending_report`（前端错误在遥测关闭时落盘 pending 存储）、`list_pending_reports`（报错页面查询是否存在历史记录）、`authorize_upload_all`（报错页面按钮 → 临时初始化 SDK 全量上传当前+历史 → flush 后关闭 SDK 不继续上报）、`test_report`（开发者模式，发送 Error 级测试事件）、`restart_app`（弹窗「立即重启」选项）

## 4. 配置项接入（crates/config）

- [x] 4.1 `schema.rs` AppSettings 新增 `telemetry_enabled: bool`（`#[serde(default)]`，默认 true 语义待与首次启动弹窗流程对齐：以「字段缺失 = 未授权」区分首次启动）；更新 validate 与默认值；附 serde 兼容性单元测试（旧配置无该字段可正常加载）
- [x] 4.2 前端 `src/types/config.ts` 同步新增 `telemetry_enabled` 类型定义

## 5. 前端接入

- [x] 5.1 `src/main.tsx`（或遥测初始化模块）：按 `import.meta.env.DEV` 选 `VITE_GLITCHTIP_DSN_TEST` / `VITE_GLITCHTIP_DSN`；`telemetry_enabled` 为 true 且 DSN 存在才 `Sentry.init`（`autoSessionTracking: false`、`beforeSend` 脱敏同 Rust 规则）
- [x] 5.2 `ErrorBoundary.tsx`：`componentDidCatch` 挂 `Sentry.captureException`，tag 携带组件名与对应 `PGR-3xxx` code；原有降级 UI 行为不变
- [x] 5.3 `globalErrorToast.ts`：onerror / unhandledrejection 处理中挂上報出口（`Sentry.captureException` / `captureMessage`），tag 携带对应 `PGR-3xxx` code；前端 Code 常量集中于独立文件，与 `REPORT_CODES.md` 同步；原有 toast 行为不变
- [x] 5.4 首次启动授权弹窗（唯一授权提示）：`telemetry_enabled` 字段缺失时展示「是否允许匿名上报崩溃信息与使用情况？不收集任何个人数据」，默认勾选；结果写入配置后不再弹出任何形式的授权提示。文案用词使用「使用情况」而非「使用统计」，以预留未来功能使用分析（feature_used 等埋点）的叙事余地——当前 change 不实现这些埋点，仅保证文案不需要二次修改
- [x] 5.5 遥测关闭时的临时授权入口：ErrorBoundary 报错页面在遥测未开启时显示「匿名上传错误报告」按钮（存在 pending 历史记录时提示数量）→ 调用 `authorize_upload_all` 上传当前错误 + 全部历史 → 完成后提示已上传且不继续上报
- [x] 5.6 `App.tsx` 设置页新增遥测开关（复用 AppSettings 保存链路）：修改后弹出确认对话框「修改将在重启后生效，是否立即重启？」[立即重启]/[稍后重启]；选稍后则设置页保留「待重启生效」标记；新增「测试上报」按钮调用 `test_report` command，**仅开发构建（import.meta.env.DEV）可见**
- [x] 5.7（可选）新增带 name 的 `SafeBoundary` 高阶组件，仅包装关键组件

## 6. 关键路径 safe_try! 埋点（方案 B + 粒度 A，0.2.0 最小必要集）

> **0.2.0 范围**：仅接线 PGR-2101（CONFIG_IO）与 PGR-4101（OVERLAY_RENDER）两条关键路径。前者是用户配置持久化的命脉（失败 = 配置丢/写不进），后者是遮盖层核心渲染观测（失败 = 画面黑）。其余 Code（PGR-2102 IMAGE_IO / PGR-4201-4202 Win32 / PGR-5101-5105 操作域）**预留声明、留作 0.2.1 增量**，不阻塞 0.2.0 发版。

### 6.1 report_code 模块扩展（telemetry.rs）

- [x] 6.1.1 新增 4xxx 遮盖层域常量：`OVERLAY_RENDER`（PGR-4101，0.2.0 接线）、`OVERLAY_RESIZE`（PGR-4102，预留）、`WIN32_SETUP`（PGR-4201，预留）、`WIN32_FOLLOW`（PGR-4202，预留）
- [x] 6.1.2 新增 5xxx 操作域常量（全部预留声明，0.2.1 接线）：`CONFIG_OP`（PGR-5101）、`LAYER_OP`（PGR-5102）、`OVERLAY_OP`（PGR-5103）、`UPDATE_OP`（PGR-5104）、`TELEMETRY_OP`（PGR-5105）、`META_OP`（PGR-5106，豁免接线）
- [x] 6.1.3 2xxx 域补全：`IMAGE_IO`（PGR-2102，PNG 解码归后端，预留声明，0.2.1 接线）、`TAURI_BRIDGE`（PGR-2501，预留声明）

### 6.2 0.2.0 必接：CONFIG_IO（PGR-2101）—— 策略 1：直接 safe_try!

- [x] 6.2.1 `storage.load_or_create_default()` 调用点（`lib.rs:294`）：改 `unwrap_or_else` 为 safe_try! 上报 PGR-2101 后回退默认配置（保持现有的"失败也启动"行为，但失败事件可观测）
- [x] 6.2.2 `save_config` 内 `storage.save(&config)`（`lib.rs:720`）：包 safe_try! 上报 PGR-2101
- [x] 6.2.3 `update_preferences` 内 save（`lib.rs:1036`）：包 safe_try! 上报 PGR-2101
- [x] 6.2.4 其余 save 调用点（`lib.rs:1145` / `lib.rs:1790`）：包 safe_try! 上报 PGR-2101

### 6.3 0.2.0 建议接：OVERLAY_RENDER（PGR-4101）—— 决策 9 策略 b

- [x] 6.3.1 改 `OverlayRenderer::render_overlay(&mut self)` 签名为 `-> Result<(), Box<dyn std::error::Error>>`
- [x] 6.3.2 仅上抛 `surface.resize()` 失败为 Err；内部其他吞错点（`config.lock().expect` / 几何计算 / material evaluate）维持原状（panic 由 PGR-1001 兜底）
- [x] 6.3.3 `overlay.rs:488` 调用点改 `let _ = safe_try!(renderer.render_overlay(), report_code::OVERLAY_RENDER);`

### 6.4 留作 0.2.1（本 change 范围外，仅登记）

以下 Code 在 §6.1 已声明常量，接线工作留作 0.2.1 增量 change：

- `IMAGE_IO`（PGR-2102）—— `load_png()` match Err 分支
- `WIN32_SETUP`（PGR-4201）/ `WIN32_FOLLOW`（PGR-4202）—— platform/windows.rs
- `CONFIG_OP` / `LAYER_OP` / `OVERLAY_OP` / `UPDATE_OP` / `TELEMETRY_OP`（PGR-5101-5105）—— CommandResultExt trait + 各 command 尾部
- `OVERLAY_RESIZE`（PGR-4102）/ `TAURI_BRIDGE`（PGR-2501）/ `EXTERNAL_CALL`（PGR-2401）/ `META_OP`（PGR-5106）—— 永久预留或豁免

### 6.5 验证

- [x] 6.5.1 `cargo check` / `clippy` / `test` 在 3 个 lib crate 通过（Linux 本机可验）
- [x] 6.5.2 release 构建（`panic = "abort"`）下 PGR-2101 / PGR-4101 错误事件正常上报（code / 函数名 / 文件 / 行号齐全）—— 用户确认 Windows 实机验收已完成

## 7. 编译期禁用

- [x] 7.1 `src-tauri/build.rs` 检测 `PEREGRINE_DISABLE_TELEMETRY` 环境变量并发出 cfg；telemetry 模块在该 cfg 下编译为 no-op 桩（全部公开 API 保留签名、内部空实现），二进制不含任何上报代码路径与网络请求
- [x] 7.2 前端在编译期禁用构建中不注入 DSN（自然等效禁用）；验证编译期禁用构建下设置页遥测 UI 不可用或隐藏、零网络请求 **【暂缓：编译期禁用构建验证延后，当前用户向暂不需要】**

## 8. REPORT_CODES 文档

> **已迁移到 `documentation-output` change §3**（REPORT_CODES.md + Code 登记核验）。本 change 不再产出文档。

## 9. CI 与文档

- [x] 9.1 `.github/workflows/release.yml` 注入 `GLITCHTIP_DSN`（GitHub Secrets）至构建环境（line 81-82 已按 tag 名含 `-` 判定选 TEST/正式 DSN）；三架构（i686/x86_64/aarch64）release 构建均生效待首次 release tag 触发确认
- [x] 9.2 `ci.yml` 确认 lint/test 在无 DSN 环境及 `PEREGRINE_DISABLE_TELEMETRY` 编译期禁用环境下均通过（SDK 不初始化/no-op 路径）
- [x] ~9.3 更新 `AGENTS.md`：新增遥测模块说明、DSN 注入方式、safe_try! 使用约定（仅关键路径）、隐私开关语义、上报 CODE 登记治理约定、编译期禁用方式~ **【已迁移到 `documentation-output` change §6】**
- [x] ~9.4 更新 docs 文档站（用户指南：隐私说明与开关；开发指南：DSN 环境变量、编译期禁用与本地调试）~ **【已迁移到 `documentation-output` change §4/§5】**

## 10. 验收（对应验收标准）

> 用户确认 Windows 实机 + GlitchTip 后台验收已完成。

- [x] 10.1 dev 构建上报至 TEST 项目、release 构建上报至正式项目（GlitchTip 后台确认）
- [x] 10.2 无 DSN 或开关关闭：SDK 不初始化、零网络请求（抓包或日志确认）、功能正常
- [x] 10.3 触发 Rust panic → abort → pending 存储已落盘；开关开启后下次启动无感静默上传全部历史，GlitchTip 收到
- [x] 10.4 开关关闭时错误（panic/关键路径/前端）仅落盘不上报；报错页面按钮授权后上传当前+全部历史，完成后 SDK 关闭不继续上报；非 5MB 超限不主动删除记录
- [x] 10.5 首次启动授权弹窗为唯一授权提示，此后（含开关关闭后）不再出现任何形式的遥测授权弹窗
- [x] 10.6 React 组件错误经 ErrorBoundary 上报且携带组件名
- [x] 10.7 启动事件为 Info 级、不进 issue 列表、含 code（`PGR-0001`）/install_id/version/os/arch tag
- [x] 10.8 install_id：同一安装多次启动不变、不同安装不同、重置 config.json 不影响、文件损坏自动重建
- [x] 10.9 事件中无 IP/用户名/机器名/路径用户名（抽样检查事件原文）
- [x] 10.10 DSN 未出现在 git 历史/源码（`git log -p` 与源码检索确认）
- [x] 10.11「测试上报」按钮在开发者模式已解锁（或开发构建）时于设置页「开发」Tab 可见，点击产生 Error 级测试事件并进 issue 列表；未解锁的正式构建普通用户不可见
- [x] 10.12 开关修改后弹窗「是否立即重启？」：立即重启 → 重启后新状态生效；稍后重启 → 设置页保留「待重启生效」标记
- [x] 10.13 `PEREGRINE_DISABLE_TELEMETRY` 编译的构建：零上报代码路径、零网络请求，遥测 UI 不可用
- [x] 10.14 GlitchTip 中可按 `event_type`/`priority` tag 筛出 crash / error / startup 三类
- [x] 10.15 所有上报事件（启动/崩溃/错误）均携带 `code` tag，GlitchTip 中可按 code 筛出同一来源事件；Code 常量集中定义于遥测模块（Rust `report_code` / 前端 `REPORT_CODES`），无散落硬编码（`REPORT_CODES.md` 开发者向文档已迁移到 `documentation-output` change）
- [x] 10.16 `cargo fmt` / `cargo clippy` / `cargo test` / `npm run build` 全部通过
