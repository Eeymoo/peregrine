# 上报 Code 登记表（Telemetry Report Codes）

Peregrine 的匿名遥测（GlitchTip / Sentry 协议）通过 **Code** 常量标识每一类上报点。本表集中登记所有已定义 Code，与下列常量定义逐条对齐：

- Rust 侧：[`src-tauri/src/telemetry.rs::report_code`](src-tauri/src/telemetry.rs)（`pub mod report_code`）
- 前端侧：[`src/lib/telemetry.ts::REPORT_CODES`](src/lib/telemetry.ts)

> **治理约定**：新增上报点必须 **先在本表与对应常量模块登记 Code**，而后方可编写 `safe_try!` / `capture_message` / `captureFrontendError` 调用。未登记的硬编码 Code 不得在 PR 中合入。Code 号段一经分配不再变更。

---

## 号段约定

| 号段 | 域 | 用途 |
|---|---|---|
| `PGR-0xxx` | 生命周期 | 启动 / 测试事件（Info 级居多） |
| `PGR-1xxx` | 崩溃 | Rust panic / abort |
| `PGR-2xxx` | 后端通用缺陷 | 21xx IO / 24xx 外部调用 / 25xx 桥接 |
| `PGR-3xxx` | 前端缺陷 | React 渲染 / 全局错误 / Promise |
| `PGR-4xxx` | 遮盖层（overlay） | 41xx 渲染 / 42xx Win32 窗口 |
| `PGR-5xxx` | 方法（Tauri command） | 51xx 操作域各类命令 |

**优先级**（`priority` tag）：

- `p1`：崩溃（`PGR-1xxx`）
- `p2`：关键路径错误（`PGR-2xxx` / `PGR-3xxx` / `PGR-4xxx` / `PGR-0901`）
- `p3`：启动统计（`PGR-0001`，Info 级，不进 issue 列表）

---

## PGR-0xxx 生命周期

| Code | 常量 | 语义 | 触发点 | 接线状态 |
|---|---|---|---|---|
| `PGR-0001` | `report_code::APP_STARTUP` | 启动统计事件（Info 级，携带 install_id / version / os / arch） | `src-tauri/src/lib.rs::run()` 启动序列末尾调用 `telemetry::report_startup()` | ✅ 已接线（SDK 激活时上报） |
| `PGR-0901` | `report_code::TEST_REPORT` | 开发者模式「测试上报」按钮触发的 Error 级测试事件 | `src-tauri/src/telemetry.rs::test_report()`（由 `test_report` Tauri command 调用） | ✅ 已接线 |

---

## PGR-1xxx Rust panic / 崩溃

| Code | 常量 | 语义 | 触发点 | 接线状态 |
|---|---|---|---|---|
| `PGR-1001` | `report_code::RUST_PANIC` | Rust panic 崩溃记录（`panic = "abort"` 下同步落盘 pending，授权后上传） | `src-tauri/src/telemetry.rs::register_panic_hook()` 自定义 panic hook | ✅ 已接线（开关关闭时 panic 也落盘） |

---

## PGR-2xxx 后端 Rust 通用缺陷

| Code | 常量 | 语义 | 触发点 | 接线状态 |
|---|---|---|---|---|
| `PGR-2101` | `report_code::CONFIG_IO` | 配置文件读写错误（`ConfigStorage::save` 失败） | `src-tauri/src/lib.rs` 多处 `safe_try!(...save..., CONFIG_IO)`（`save_config` / `update_preferences` / profile 操作 / `save_and_broadcast`） | ✅ 已接线（0.2.0） |
| `PGR-2102` | `report_code::IMAGE_IO` | 贴图 / 图片文件读写错误（PNG 解码归后端） | 暂无（PNG 解码失败目前仅日志告警） | 🔵 预留声明（0.2.1 接线） |
| `PGR-2401` | `report_code::EXTERNAL_CALL` | 外部调用错误（HTTP / 外部进程等，TriggerRule 未消费） | 暂无 | 🔵 预留声明 |
| `PGR-2501` | `report_code::TAURI_BRIDGE` | Tauri 桥接层失败（前端 3xxx 已覆盖，首版不接线） | 暂无 | 🔵 预留声明 |

---

## PGR-3xxx 前端 React 错误

> 常量定义于 `src/lib/telemetry.ts::REPORT_CODES`。

| Code | 常量 | 语义 | 触发点 | 接线状态 |
|---|---|---|---|---|
| `PGR-3001` | `REPORT_CODES.REACT_BOUNDARY` | React ErrorBoundary 捕获的组件渲染错误 | `src/components/ErrorBoundary.tsx::componentDidCatch` → `captureFrontendError` | ✅ 已接线 |
| `PGR-3002` | `REPORT_CODES.GLOBAL_ONERROR` | `window.onerror` 全局未捕获错误 | `src/lib/globalErrorToast.ts::installGlobalErrorHandler` | ✅ 已接线 |
| `PGR-3003` | `REPORT_CODES.UNHANDLED_REJECTION` | `unhandledrejection` 未处理的 Promise rejection | `src/lib/globalErrorToast.ts::installGlobalErrorHandler` | ✅ 已接线 |

---

## PGR-4xxx 遮盖层缺陷（overlay 域）

| Code | 常量 | 语义 | 触发点 | 接线状态 |
|---|---|---|---|---|
| `PGR-4101` | `report_code::OVERLAY_RENDER` | overlay 主渲染入口错误（`render_overlay` 失败，当前仅在 `surface.resize()` 失败时返回 Err） | `src-tauri/src/overlay.rs::render` 循环内 `safe_try!(...render_overlay()..., OVERLAY_RENDER)` | ✅ 已接线（0.2.0） |
| `PGR-4102` | `report_code::OVERLAY_RESIZE` | overlay resize 错误（softbuffer resize 当前合并在 `render_overlay` 内） | 暂无独立触发点（合入 PGR-4101） | 🔵 预留声明 |
| `PGR-4201` | `report_code::WIN32_SETUP` | Win32 透明 / 置顶 / 点击穿透设置错误 | 暂无（`setup_overlay_window` 当前无显式失败路径） | 🔵 预留声明（0.2.1 接线） |
| `PGR-4202` | `report_code::WIN32_FOLLOW` | Win32 目标窗口跟随错误 | 暂无（跟随循环内失败仅日志告警） | 🔵 预留声明（0.2.1 接线） |

---

## PGR-5xxx 方法缺陷（Tauri command 操作域）

> 本号段全部为 **预留声明**（0.2.1 起按需接线）。当前各 Tauri command 的错误路径由 `PGR-2101`（配置 IO）/ 前端 `PGR-3xxx`（桥接失败）覆盖。`PGR-5106`（元信息 getter）因纯 getter 失败意义低，**豁免接线**。

| Code | 常量 | 语义 | 接线状态 |
|---|---|---|---|
| `PGR-5101` | `report_code::CONFIG_OP` | 配置类操作（save_config / update_preferences / 各 profile 操作等） | 🔵 预留声明（0.2.1 接线） |
| `PGR-5102` | `report_code::LAYER_OP` | 图层类操作（add/remove/move/duplicate/update_layer / list_layers） | 🔵 预留声明 |
| `PGR-5103` | `report_code::OVERLAY_OP` | 覆盖层类操作（start_overlay / stop_overlay / focus_target_window） | 🔵 预留声明 |
| `PGR-5104` | `report_code::UPDATE_OP` | 更新类操作（check_update / download_install_update / relaunch_app / restart_app） | 🔵 预留声明 |
| `PGR-5105` | `report_code::TELEMETRY_OP` | 遥测类操作（store_pending_report / list_pending_reports / authorize_upload_all / test_report） | 🔵 预留声明 |
| `PGR-5106` | `report_code::META_OP` | 元信息 getter 类操作（get_app_version / list_window_titles / list_materials 等） | ⚪ 豁免接线（纯 getter，失败意义低） |

---

## 图例

- ✅ 已接线：在代码中存在 `safe_try!` / `captureFrontendError` / `capture_message` 触发点，SDK 激活时实时上报、SDK 未激活时落盘 pending。
- 🔵 预留声明：常量已登记但代码触发点尚未接入；按计划在后续版本接线。
- ⚪ 豁免接线：常量已登记但评估为低价值 / 由其他 Code 覆盖，不安排接入触发点。
