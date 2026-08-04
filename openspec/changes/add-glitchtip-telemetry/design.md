# 设计：GlitchTip 匿名崩溃上报 + 启动统计

## Context

Peregrine 是 Windows 桌面工具（Tauri 2 + Rust + React/TypeScript），release 构建使用 `panic = "abort"`（**禁止改为 unwind**，根 `Cargo.toml` `[profile.release]` 已固定）。已自建 GlitchTip 服务（glitchtip.onemue.cn）。现有可复用基础设施：

- 前端已有 `ErrorBoundary.tsx` 与 `globalErrorToast.ts`（onerror + unhandledrejection），只需挂上报出口，不做新捕获层
- 配置体系已有 AppSettings（config.json，validate + 原子写 + 热重载 + settings-changed 广播），遥测开关直接复用
- `storage.rs` 已有跨平台 dirs 模块（`%APPDATA%/Peregrine/`），install_id / pending 存储放同目录
- 已有 `reqwest` 依赖（备选双项目裸 POST 路径可用）
- 已有 tracing 日志体系，遥测模块的自身日志走 tracing，不用 println!

核心矛盾：release 的 `panic = "abort"` 使 `catch_unwind` 完全失效，且 sentry 默认的内存缓冲 + 定时 flush 在 abort 时来不及发送。因此崩溃链路必须走「落盘 → 下次启动上传」，这与 Sentry 官方 native 崩溃的标准做法一致，也天然兼容「开关关闭时零网络请求」的隐私要求。

## Goals / Non-Goals

**Goals:**

- Rust panic 崩溃记录 100% 落盘（abort 前必达），授权后可靠上传
- 关键路径错误（IO / 渲染 / 桥接 / 外部调用）经 `safe_try!` 上报，自动携带函数名/文件/行号，abort 构建下可用
- 匿名启动统计（install_id 去重估算用户量），Info 级不进 issue 列表
- 隐私合规：默认开启但明确告知、可一键关闭（零网络请求）、崩溃临时授权、全链路脱敏
- DSN 不进 git 历史/源码；未配置 DSN 时零副作用

**Non-Goals:**

- 不做实时崩溃发送（abort 下不可靠，统一走 pending 存储、授权或开关开启后上传）
- 不做 session tracking / release health（GlitchTip 不支持，`auto_session_tracking: false`）
- 不做性能监控（transactions / tracing 采样）、不做用户反馈表单
- 不改动 `panic = "abort"`；`safe_try!` 不做 panic 捕获，只处理显式 Result/Option
- 不采集任何可识别个人信息（IP / 用户名 / 机器名 / 设备 ID / 用户输入）
- 不为所有方法包 `safe_try!`，仅关键路径

## Decisions

### 决策 1：单 sentry 实例 + Level/tag 区分，而非双项目物理隔离

同一 sentry 客户端，用 Level 与 tag 分离三类事件：

| 事件类型 | Level | 进 issue 列表 | tag |
|---------|-------|:---:|-----|
| 崩溃/异常 | Error / Fatal | 是 | `event_type=crash`、`priority=p0/p1` |
| 关键路径错误（safe_try!） | Error / Warning | 是（按需） | `event_type=error`、`priority=p2` |
| 启动统计 | Info | 否 | `event_type=startup`、`priority=p3` |

**原理**：Sentry/GlitchTip 仅将 error 及以上级别聚合进 issue 列表，Info 只出现在事件流。**备选方案**（crashes/stats 双项目，`GLITCHTIP_DSN_CRASHES(_TEST)` + `GLITCHTIP_DSN_STATS(_TEST)`）保留为备注：崩溃走全局 sentry，stats 用已有 reqwest 裸 POST（约 10 行，无常驻内存）或独立 Hub + flush。默认采用单实例——配置面最小、无需管理两套 Hub 生命周期。

### 决策 2：错误走本地 pending 存储，授权后全量上传

- panic hook 内只做一件事：**同步原子写** pending 存储（tmp + rename），内容含时间戳、version、install_id、脱敏后的错误信息与堆栈摘要；hook 内不初始化 SDK、不弹窗、不网络请求（abort 前这些都不保证完成）
- **pending 存储模型**：多记录（每条错误一条记录），容量上限 5MB，超出删除最旧记录；**除此条件外不主动删除**（用户拒绝/忽略也不删）
- 遥测关闭时，safe_try! 关键路径错误与前端错误同样落盘 pending 存储（前端经 Tauri command 写入），不初始化 SDK
- **上传时机**：
  - 开关开：启动后**无感静默**全量上传 pending 历史并清除已上传记录，无任何弹窗或同意请求
  - 开关关：唯一授权入口为前端报错页面（ErrorBoundary 降级页）的「匿名上传错误报告」按钮 → 临时初始化 SDK → 上传当前错误 + 全部历史记录 → flush 后 drop SDK，**不继续上报、不修改开关状态**
- 除首次启动授权弹窗外，不再出现任何形式的遥测授权提示（移除原「下次启动询问弹窗」）
- **备选**：sentry 的 `flush(Some(timeout))` 实时发送 —— 否决，abort 模式下 flush 不保证执行完；启动询问弹窗 —— 否决，用户明确仅保留首次授权提示 + 报错页面按钮

### 决策 3：safe_try! 为错误上报宏，不依赖 catch_unwind

- 语义：包装返回 Result 的调用，失败时经 `sentry::with_scope` 附加 `function`（`function_name!()`）、`location`（`Location::caller()`，`#[track_caller]` 保证指向调用点）tag 上报 Error 级事件，原样返回 Err 供调用方降级
- Option 场景由调用方转 Result 后使用（保持宏单一职责）
- `function_name` crate 提供 `function_name!()` 宏；位置信息用标准库 `std::panic::Location::caller()`，零额外依赖
- **备选**：`catch_unwind` 捕获 panic —— 否决，release abort 下完全失效；`anyhow::Context` 链 —— 与现有 `thiserror` 错误体系不一致，仅作错误消息格式化参考

### 决策 4：install_id 独立文件，与 config.json 解耦

- 路径复用 `storage.rs` dirs 模块的应用数据目录，文件名 `install_id`，内容 UUID v4 纯文本
- 首次启动生成（原子写：tmp + rename）；损坏/为空自动重建
- 配置重置/导出/分享不带走；删除文件即「新安装」
- **备选**：存进 config.json —— 否决，配置分享会泄露、重置会丢历史连续性；用机器 GUID —— 否决，违反匿名原则

### 决策 5：DSN 构建时注入，双环境分离

- Rust：`option_env!("GLITCHTIP_DSN")`（正式）/ `option_env!("GLITCHTIP_DSN_TEST")`（开发），按 `cfg!(debug_assertions)` 选择；CI release 注入正式 DSN，本地 dev 用 `.env`
- 前端：`import.meta.env.DEV` 选择 `VITE_GLITCHTIP_DSN_TEST` / `VITE_GLITCHTIP_DSN`，Vite 按 mode 加载 `.env.development` / `.env.production`
- `.gitignore` 补 `.env.development` / `.env.production` / `.env.*.local`
- 语义澄清：DSN 是公开密钥（只能写事件），出现在构建产物中可接受；底线是**不进 git 历史/源码**
- `option_env!` 为 None 时 SDK 不初始化——本地无 DSN 开发者零影响

### 决策 6：开关复用 AppSettings，弹窗确认重启

- `telemetry_enabled: bool` 加入 AppSettings（`#[serde(default)]` 兼容旧配置；首次启动弹窗结果写入该字段）
- 开关修改后弹出确认对话框「修改将在重启后生效，是否立即重启？」，统一两个选项（**所有重启入口均采用此弹窗**）：
  - 「立即重启」：保存配置 → 立即重启应用（实现：Tauri 重启——`tauri-plugin-process` 的 relaunch，或后端 spawn 新进程后退出当前进程）
  - 「稍后重启」：保存配置，设置页保留「待重启生效」标记，继续以旧状态运行直至下次手动重启
- 不做运行时动态启停（SDK 全局单例，动态拆装有竞态且收益低）
- SDK 初始化集中在 `src-tauri/src/lib.rs::run()` 启动早期，telemetry 逻辑独立为 `src-tauri/src/telemetry.rs` 模块（install_id / pending 存储 / 脱敏 / safe_try! 宏 / 上报函数）
- 前端 beforeSend 与 Rust `before_send` 双侧脱敏：删 user/server_name/request；路径用户名正则替换为 `{user}`

### 决策 7：上报 CODE 体系

- 所有上报事件（启动/崩溃/错误）统一携带 `code` tag，格式 `PGR-<类别><序号>`（如 `PGR-2001`），号段划分：
  - `PGR-0xxx`：启动/生命周期事件——首个 Code `PGR-0001` = app_startup
  - `PGR-1xxx`：Rust panic / 崩溃（pending 记录上传时按 panic 位置粗分）
  - `PGR-2xxx`：safe_try! 关键路径错误——`21xx` 文件 IO、`22xx` 渲染、`23xx` 窗口/覆盖层桥接、`24xx` 外部调用
  - `PGR-3xxx`：前端 React 错误（ErrorBoundary / 全局 onerror / unhandledrejection）
- 文档：仓库根新增 `REPORT_CODES.md`，覆盖全部事件类型（不只是错误），每条登记 Code → 含义 → 触发位置（模块/场景）→ 处理建议；开发者向文档，随代码评审同步更新，docs 文档站做链接引用
- 治理：Code 常量集中定义于遥测模块（Rust 侧 `telemetry.rs` 内 `report_code` 模块，前端侧对应常量文件），禁止散落硬编码；新增上报点必须先登记 Code 再合入；GlitchTip 中按 `code` tag 过滤即得同一来源全量事件
- **备选**：仅错误事件带码（ERROR_CODE）—— 否决，启动统计同样需要按码聚合筛选，统一体系更简洁；纯自增数字码 —— 否决，号段自带类别语义，GlitchTip 列表一眼可辨

### 决策 8：编译期禁用与开发者模式

- **编译期禁用**：编译前设置环境变量 `PEREGRINE_DISABLE_TELEMETRY` → build.rs 检测并发出 cfg，telemetry 模块整体编译为 no-op 桩，二进制不含任何上报代码路径与网络请求；与运行时开关相互独立（编译期禁用优先于一切运行时配置）。前端侧通过不注入 DSN 达到同等效果
  - **备选**：cargo feature 开关 —— 否决，环境变量与既有 DSN 注入方式一致，CI 配置更统一
- **开发者模式**：「测试上报」按钮仅在开发构建（`import.meta.env.DEV`）下显示，正式构建对普通用户隐藏

## Risks / Trade-offs

- [panic hook 内文件 IO 本身失败（磁盘满/权限）] → 静默忽略（`.ok()`），崩溃落盘是 best-effort，绝不让 hook 二次 panic（hook 内 panic 会直接 abort 且无输出）
- [启动统计 Info 事件量大污染事件流、消耗 GlitchTip 配额] → 接受：桌面应用启动频次低；后续可在 GlitchTip 侧配置入站过滤规则
- [`option_env!` 在增量编译下环境变量变化不触发重编译] → CI release 为干净构建无此问题；本地 dev 文档注明改 DSN 后需 `cargo clean -p peregrine-tauri` 或 touch 源文件
- [遥测关闭且 WebView 本身故障时，报错页面无法展示，错误只能累积无法临时授权上传] → 接受：记录保留在本地 pending 存储（5MB 轮转），待前端恢复或用户开启开关后无感上传
- [双端（Rust/前端）各自初始化 SDK 可能重复上报同一前端错误] → 职责划分：webview 内错误只由前端 `@sentry/react` 上报，Rust 侧只处理 native panic 与 safe_try!；两端 release 标记一致便于关联
- [ GlitchTip 版本差异导致部分 sentry 协议字段不支持 ] → 只使用基础 envelope（message/exception + tags + level），`auto_session_tracking: false`，不启用新特性

## Migration Plan

1. 先在 dev 分支实现并自测（开发 DSN 指向 TEST 项目）
2. CI release.yml 增加 `GLITCHTIP_DSN` 环境变量注入（GitHub Secrets）
3. 按版本约定发预览版（偶数/prerelease 号，如 0.1.8-alpha.0）验证上报链路，再合并 main 发稳定版
4. 回滚：SDK 初始化为可选路径，移除依赖或关闭默认开关即可完全退回现状；无数据迁移负担（install_id / pending 存储删除即清理）

## Open Questions

- 首次启动授权弹窗的实现形态：复用设置窗口内嵌对话框（tauri-plugin-dialog 已有）还是独立窗口？——倾向复用 dialog 插件，实现时确认
- `priority` 的 p0/p1 细分规则（哪些崩溃算 p0）——实现时按 panic 是否发生在主事件循环粗略划分即可，后续按实际数据调整
