---
title: "隐私与遥测"
---

Peregrine 内置 **匿名、可选** 的遥测功能，用于诊断崩溃、确定修复优先级。本文完整说明：采集什么、不采集什么、如何关闭，以及自构建用户如何在编译期彻底禁用。

> 一句话：**绝不采集 IP / 用户名 / 机器名 / 设备 ID / 含用户名的路径**。遥测只在「启动统计、崩溃、关键错误」三类场景触发，不会上报配置内容、游戏标题或浏览行为。

## 采集的数据维度

当遥测开关已开启 **且** 当前二进制在构建期注入了 DSN（官方构建满足）时，以下事件会发送到自托管的 GlitchTip 实例：

| 事件 | 级别 | 触发时机 | 携带数据 |
|---|---|---|---|
| **应用启动** | Info | 每次进程启动一次 | `code=PGR-0001`、`install_id`、`version`、`os`、`arch` |
| **Rust panic / 崩溃** | Error | 任意 `panic!`（自定义 panic hook 捕获，先落盘本地再上传） | `code=PGR-1001`、`install_id`、`version`、脱敏后的堆栈与消息 |
| **关键路径错误** | Error | `safe_try!` 宏捕获的 IO / 渲染 / 桥接失败 | `code=PGR-2xxx/4xxx`、函数名、文件:行号、脱敏消息 |
| **前端错误** | Error | React `ErrorBoundary` / `window.onerror` / `unhandledrejection` | `code=PGR-3xxx`、脱敏后的消息与堆栈 |

每条事件固定的 tag 集合：

- `install_id` —— 首次启动生成的随机 UUID v4，存放在 `<应用数据目录>/install_id`，**不与任何真实身份关联**，删除该文件即可重置。
- `version` —— 应用版本号（如 `0.2.0`）。
- `os` / `arch` —— `std::env::consts::OS` / `ARCH`（如 `windows` / `x86_64`）。
- `code` —— 来自 [上报 Code 登记表](./report-codes) 的稳定标识符，表示事件类别。
- `event_type` —— `startup` / `crash` / `error`。
- `priority` —— `p1`（崩溃）/ `p2`（错误）/ `p3`（启动）。

崩溃报告额外包含 Rust / JS 堆栈与 panic 消息（经脱敏，见下文）。

## 不采集的内容

下列数据 **绝不** 上报：

- ❌ **IP 地址** —— 事件离机前删除 `user` / `server_name` / `request` 字段（sentry `before_send` 钩子）。
- ❌ **用户名** —— Windows 路径如 `C:\Users\<name>`、macOS `/Users/<name>`、Linux `/home/<name>` 中的用户名会被替换为 `C:\Users\{user}` / `/Users/{user}` / `/home/{user}`（覆盖消息、堆栈、`abs_path`）。
- ❌ **机器名 / 主机名 / 设备 ID**（无 `server_name`、无 SMB/NetBIOS 名）。
- ❌ **配置内容** —— 不上报准心样式、图层参数、Profile 名、目标窗口标题、热键绑定。
- ❌ **游戏标题 / 窗口标题**（`target_window` 是用户设置，**不** 上送）。
- ❌ **截图、剪贴板、文件内容**。
- ❌ **浏览历史、输入事件、鼠标坐标**（物料脚本有动态输入 API，但目前处于软关闭状态，且绝不外发）。

## 脱敏规则

Rust 与前端两侧的 Sentry 客户端应用相同的 `anonymize_event` / `beforeSend` 钩子：

1. 删除 `event.user`（丢掉 IP、可能存在的用户名）。
2. 删除 `event.server_name`（丢掉主机名）。
3. 删除 `event.request`（丢掉 URL / headers）。
4. 在 `message`、`exception.value`、每个堆栈帧的 `filename` / `abs_path` 中做「路径用户名替换」。

替换模式覆盖 Windows（`C:\Users\x`）、macOS（`/Users/x`）、Linux（`/home/x`）三种主目录形式。

## 授权与开关

遥测采用 **首次启动询问** 模式：

- 首次启动配置窗口会弹出一次性对话框，询问是否启用匿名上报。你的选择（`true` / `false`）持久化到 `config.settings.telemetry_enabled`。
- 若字段缺失（`null`），SDK **不初始化** —— 沉默视为 **未授权**，而非默认允许。
- 开关位于 **设置 → 匿名崩溃上报与使用情况**。修改后需重启生效，UI 会显示「待重启生效」徽标提示。
- 报错页面还提供 **一次性「匿名上传错误报告」** 按钮：点击后 SDK 临时初始化，仅够上传本地 pending 历史，随后立即关闭，**不** 修改持久开关状态。

## 本地 pending 存储

遥测关闭（或未注入 DSN）时，崩溃与关键路径错误仍会落盘到本地，便于事后查看或手动上传：

- 路径：`<应用数据目录>/pending/*.json`（每条一文件，原子写）。
- 上限：总量 5MB，超限按最旧优先删除。
- 字段：`ts`、`version`、`install_id`、`code`、脱敏后的 `message` —— 仅此而已。

报错页面会显示 pending 条数，可选择上传（或直接删除这些文件）。

## 编译期禁用（`PEREGRINE_DISABLE_TELEMETRY`）

希望二进制内零遥测代码的自构建用户，可在编译期整体禁用：

```bash
# 构建前设置环境变量（任意值均可）
PEREGRINE_DISABLE_TELEMETRY=1 cargo build --release
# 或 Tauri 构建：
PEREGRINE_DISABLE_TELEMETRY=1 npx tauri build
```

设置后，`src-tauri/build.rs` 发出 `peregrine_disable_telemetry` cfg，整个 `telemetry` 模块编译为 no-op 桩：所有公开 API 保留签名但内部无 IO、无网络、无 panic hook。最终二进制不包含任何上报代码路径。

## 数据去向

- GlitchTip 实例为 **自托管**（Sentry 协议兼容）。
- 官方 release 构建上报到 **正式项目**；官方开发 / snapshot 构建上报到 **测试项目**。未注入 DSN 的自构建产物 **零网络请求**。

## 小结

| 问题 | 回答 |
|---|---|
| 能看到我的身份吗？ | 不能。IP / 用户名 / 机器名 / 设备 ID 全部剔除或根本不采集。 |
| 能看到我的配置 / 游戏吗？ | 不能。只采集崩溃 / 错误诊断与粗粒度环境 tag。 |
| 默认开启吗？ | 否。首次启动询问；未授权视为关闭。 |
| 可以关闭吗？ | 可以，设置 → 匿名崩溃上报，随时修改（需重启）。 |
| 可以无遥测构建吗？ | 可以，`PEREGRINE_DISABLE_TELEMETRY=1` 在编译期剔除全部上报代码。 |

面向开发者的 Code 登记表见仓库根目录的 [`REPORT_CODES.md`](./report-codes)。
