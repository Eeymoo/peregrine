//! 遥测模块：GlitchTip 匿名崩溃上报 + 启动统计。
//!
//! 职责：
//! - install_id 生命周期管理（独立文件、原子写、损坏重建）
//! - 本地 pending 错误存储（多记录、原子写、5MB 轮转）
//! - 事件脱敏（删 user/server_name/request，路径用户名替换为 `{user}`）
//! - panic hook 同步落盘（兼容 release 的 `panic = "abort"`）
//! - 上报 Code 常量集中定义（`report_code` 子模块）
//! - 启动统计事件与 safe_try! 关键路径错误上报出口
//!
//! 编译期禁用：构建前设置环境变量 `PEREGRINE_DISABLE_TELEMETRY` 后，
//! build.rs 发出 `peregrine_disable_telemetry` cfg，本模块全部公开 API
//! 保留签名但编译为空实现（no-op），二进制不含任何上报代码路径与网络请求。

/// pending 存储容量上限（5MB），超出时删除最旧记录。
#[cfg(not(peregrine_disable_telemetry))]
const PENDING_MAX_BYTES: u64 = 5 * 1024 * 1024;

/// 上报 Code 常量（集中定义，禁止散落硬编码）。
///
/// 号段约定（与设计决策 7 一一对应，新增上报点必须先登记）：
/// - `PGR-0xxx`：启动/生命周期事件
/// - `PGR-1xxx`：Rust panic / 崩溃
/// - `PGR-2xxx`：后端 Rust 通用缺陷（21xx IO / 24xx 外部调用 / 25xx 桥接）
/// - `PGR-3xxx`：前端 React 错误（常量定义于前端 `src/lib/telemetry.ts`）
/// - `PGR-4xxx`：遮盖层缺陷（41xx 渲染 / 42xx Win32 窗口）
/// - `PGR-5xxx`：方法缺陷（Tauri command 操作域，51xx 各类操作）
pub mod report_code {
    // ===== 0xxx 生命周期 =====
    /// 启动统计事件（Info 级，不进 issue 列表）。
    pub const APP_STARTUP: &str = "PGR-0001";
    /// 开发者模式「测试上报」按钮触发的测试事件。
    pub const TEST_REPORT: &str = "PGR-0901";

    // ===== 1xxx Rust panic / 崩溃 =====
    /// Rust panic 崩溃记录（panic hook 落盘 / 授权后上传）。
    pub const RUST_PANIC: &str = "PGR-1001";

    // ===== 2xxx 后端 Rust 通用缺陷 =====
    /// 配置文件读写错误（0.2.0 接线）。
    pub const CONFIG_IO: &str = "PGR-2101";
    /// 贴图 / 图片文件读写错误（PNG 解码归后端，预留声明，0.2.1 接线）。
    pub const IMAGE_IO: &str = "PGR-2102";
    /// 外部调用错误（HTTP / 外部进程等，预留声明，TriggerRule 未消费）。
    pub const EXTERNAL_CALL: &str = "PGR-2401";
    /// Tauri 桥接层失败（预留声明，桥接层失败由前端 3xxx 捕获，首版不接线）。
    pub const TAURI_BRIDGE: &str = "PGR-2501";

    // ===== 4xxx 遮盖层缺陷（overlay 域） =====
    /// overlay 主渲染入口错误（0.2.0 接线）。
    pub const OVERLAY_RENDER: &str = "PGR-4101";
    /// overlay resize 错误（预留声明，softbuffer resize 当前在 render 内）。
    pub const OVERLAY_RESIZE: &str = "PGR-4102";
    /// Win32 透明/置顶/点击穿透设置错误（预留声明，0.2.1 接线）。
    pub const WIN32_SETUP: &str = "PGR-4201";
    /// Win32 目标窗口跟随错误（预留声明，0.2.1 接线）。
    pub const WIN32_FOLLOW: &str = "PGR-4202";

    // ===== 5xxx 方法缺陷（Tauri command 操作域，全部预留声明，0.2.1 接线） =====
    /// 配置类操作（save_config / update_preferences / 各 profile 操作等）。
    pub const CONFIG_OP: &str = "PGR-5101";
    /// 图层类操作（add/remove/move/duplicate/update_layer / list_layers）。
    pub const LAYER_OP: &str = "PGR-5102";
    /// 覆盖层类操作（start_overlay / stop_overlay / focus_target_window）。
    pub const OVERLAY_OP: &str = "PGR-5103";
    /// 更新类操作（check_update / download_install_update / relaunch_app / restart_app）。
    pub const UPDATE_OP: &str = "PGR-5104";
    /// 遥测类操作（store_pending_report / list_pending_reports /
    /// authorize_upload_all / test_report）。
    pub const TELEMETRY_OP: &str = "PGR-5105";
    /// 元信息 getter 类操作（get_app_version / list_window_titles / list_materials 等，
    /// 纯 getter 失败意义低，豁免接线）。
    pub const META_OP: &str = "PGR-5106";
}

/// 本地 pending 错误记录（每条错误一条记录，JSON 序列化落盘）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingRecord {
    /// 记录时间戳（Unix 毫秒）。
    pub ts: u64,
    /// 应用版本号。
    pub version: String,
    /// 匿名安装 ID（本地随机 UUID，不关联真实身份）。
    pub install_id: String,
    /// 上报 Code（见 `report_code`）。
    pub code: String,
    /// 脱敏后的错误信息。
    pub message: String,
}

// ============================================================================
// 实现选择：正常实现 / 编译期禁用 no-op 桩
// ============================================================================

#[cfg(not(peregrine_disable_telemetry))]
mod imp {
    use super::{PENDING_MAX_BYTES, PendingRecord, report_code};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// 遥测运行时上下文（应用数据目录 + install_id），由 `init` 写入。
    struct TelemetryContext {
        app_data_dir: PathBuf,
        install_id: String,
    }

    static CONTEXT: Mutex<Option<TelemetryContext>> = Mutex::new(None);
    /// SDK 是否已初始化（开启开关 + DSN 齐备）。
    static SDK_ACTIVE: AtomicBool = AtomicBool::new(false);
    /// 常驻 SDK guard，保持进程生命周期内有效。
    static SDK_GUARD: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);

    /// 按构建 profile 选择 DSN：dev 用 TEST 项目，release 用正式项目。
    ///
    /// `option_env!` 在编译期读取环境变量；未注入或为空时返回 None，
    /// SDK 不初始化、零网络请求。
    fn dsn() -> Option<&'static str> {
        let raw = if cfg!(debug_assertions) {
            option_env!("GLITCHTIP_DSN_TEST")
        } else {
            option_env!("GLITCHTIP_DSN")
        };
        raw.filter(|s| !s.is_empty())
    }

    /// 构建 profile 对应的环境名。
    fn environment() -> &'static str {
        if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        }
    }

    /// 原子写文件（同目录临时文件 + rename），保证 panic/abort 下不写出半文件。
    fn atomic_write(path: &Path, content: &[u8]) -> std::io::Result<()> {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// 读取或生成 install_id（UUID v4）。
    ///
    /// - 文件不存在：生成并原子写入。
    /// - 文件为空或内容损坏（非合法 UUID）：重新生成并原子覆盖。
    /// - 与 config.json 完全解耦，重置/导出/分享配置不影响本文件。
    pub fn get_install_id(app_data_dir: &Path) -> String {
        let path = app_data_dir.join("install_id");
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim();
            if uuid::Uuid::parse_str(trimmed).is_ok() {
                return trimmed.to_string();
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        // 落盘失败仅记录日志（best-effort），不影响启动。
        if let Err(e) =
            std::fs::create_dir_all(app_data_dir).and_then(|_| atomic_write(&path, id.as_bytes()))
        {
            tracing::warn!(error = %e, "failed to persist install_id");
        }
        id
    }

    /// pending 存储目录。
    fn pending_dir(app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("pending")
    }

    /// 当前时间戳（Unix 毫秒，失败时为 0）。
    fn now_millis() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// 写入一条 pending 记录（原子写），随后执行 5MB 容量轮转（删最旧）。
    ///
    /// 除容量超限外不主动删除任何记录。
    pub fn write_pending(app_data_dir: &Path, record: &PendingRecord) {
        let dir = pending_dir(app_data_dir);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!(error = %e, "failed to create pending dir");
            return;
        }
        let file = dir.join(format!(
            "{}-{}.json",
            record.ts,
            uuid::Uuid::new_v4().simple()
        ));
        match serde_json::to_vec_pretty(record) {
            Ok(json) => {
                if let Err(e) = atomic_write(&file, &json) {
                    tracing::warn!(error = %e, "failed to write pending record");
                    return;
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to serialize pending record");
                return;
            }
        }
        enforce_pending_cap(&dir);
    }

    /// 容量轮转：pending 总量超过 5MB 时按文件名时间戳从最旧开始删除。
    fn enforce_pending_cap(dir: &Path) {
        let mut files: Vec<(PathBuf, u64)> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| {
                        let meta_len = e.metadata().map(|m| m.len()).unwrap_or(0);
                        (e.path(), meta_len)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut total: u64 = files.iter().map(|(_, len)| len).sum();
        if total <= PENDING_MAX_BYTES {
            return;
        }
        // 文件名以时间戳开头，字典序即时间序。
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, len) in files {
            if total <= PENDING_MAX_BYTES {
                break;
            }
            if std::fs::remove_file(&path).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }

    /// 列出全部 pending 记录（按时间升序），返回 (记录 id, 记录内容)。
    pub fn list_pending(app_data_dir: &Path) -> Vec<(String, PendingRecord)> {
        let dir = pending_dir(app_data_dir);
        let mut out: Vec<(String, PendingRecord)> = Vec::new();
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return out;
        };
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(record) = serde_json::from_str::<PendingRecord>(&content) else {
                continue;
            };
            let id = entry.file_name().to_string_lossy().to_string();
            out.push((id, record));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// 按记录 id 清除已上传的 pending 记录。
    pub fn clear_uploaded(app_data_dir: &Path, ids: &[String]) {
        let dir = pending_dir(app_data_dir);
        for id in ids {
            // 防路径穿越：仅允许纯文件名。
            if id.contains('/') || id.contains('\\') || id.contains("..") {
                continue;
            }
            let _ = std::fs::remove_file(dir.join(id));
        }
    }

    /// pending 记录条数（报错页面提示用）。
    pub fn pending_count(app_data_dir: &Path) -> usize {
        list_pending(app_data_dir).len()
    }

    /// 脱敏文本：把绝对路径中的用户名部分替换为 `{user}`。
    ///
    /// 覆盖三种常见形式：
    /// - `C:\Users\xxx`（任意盘符，反斜杠）
    /// - `/Users/xxx`（macOS）
    /// - `/home/xxx`（Linux）
    pub fn anonymize_text(input: &str) -> String {
        const PREFIXES: &[&str] = &["\\Users\\", "/Users/", "/home/"];
        let bytes = input.as_bytes();
        let mut out = String::with_capacity(input.len());
        let mut i = 0;
        while i < bytes.len() {
            let rest = &input[i..];
            let mut matched = false;
            for prefix in PREFIXES {
                if rest.starts_with(prefix) {
                    out.push_str(prefix);
                    out.push_str("{user}");
                    // 跳过用户名：直到路径分隔符或空白/引号。
                    let name_start = i + prefix.len();
                    let mut j = name_start;
                    while j < bytes.len() {
                        let c = bytes[j] as char;
                        if matches!(c, '\\' | '/' | '"' | '\'' | ' ' | '\n' | '\t' | '\r') {
                            break;
                        }
                        j += 1;
                    }
                    i = j;
                    matched = true;
                    break;
                }
            }
            if !matched {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
        out
    }

    /// sentry before_send 脱敏钩子：删 user / server_name / request，
    /// 并对消息与异常文本做路径用户名替换。
    pub fn anonymize_event(
        mut event: sentry::protocol::Event<'static>,
    ) -> Option<sentry::protocol::Event<'static>> {
        event.user = None;
        event.server_name = None;
        event.request = None;
        if let Some(message) = event.message.take() {
            event.message = Some(anonymize_text(&message));
        }
        for exception in event.exception.values.iter_mut() {
            if let Some(value) = exception.value.take() {
                exception.value = Some(anonymize_text(&value));
            }
            if let Some(stacktrace) = exception.stacktrace.as_mut() {
                for frame in stacktrace.frames.iter_mut() {
                    if let Some(filename) = frame.filename.take() {
                        frame.filename = Some(anonymize_text(&filename).into());
                    }
                    if let Some(abs) = frame.abs_path.take() {
                        frame.abs_path = Some(anonymize_text(&abs).into());
                    }
                }
            }
        }
        Some(event)
    }

    /// 初始化遥测：解析 install_id、初始化 SDK（开关开 + DSN 齐备时）、
    /// 并在 SDK 初始化**之后**注册自定义 panic hook。
    ///
    /// hook 无论开关都注册，保证开关关闭时 panic 也能落盘 pending。
    /// 顺序至关重要：sentry::init 会注册它自己的 panic integration hook，
    /// 若先注册自定义 hook 会被其覆盖，导致 `panic = "abort"` 下 pending
    /// 落盘失效；因此必须最后注册，让我们的 hook 成为最终生效者。
    ///
    /// 返回 SDK 是否已激活。
    pub fn init(app_data_dir: PathBuf, telemetry_enabled: bool) -> bool {
        let install_id = get_install_id(&app_data_dir);
        if let Ok(mut ctx) = CONTEXT.lock() {
            *ctx = Some(TelemetryContext {
                app_data_dir,
                install_id,
            });
        }

        let Some(dsn) = dsn() else {
            tracing::info!("telemetry DSN not injected; SDK disabled");
            register_panic_hook();
            return false;
        };
        if !telemetry_enabled {
            tracing::info!("telemetry disabled by user; SDK not initialized");
            register_panic_hook();
            return false;
        }

        let guard = sentry::init(sentry::ClientOptions {
            dsn: dsn.parse().ok(),
            release: sentry::release_name!(),
            environment: Some(environment().into()),
            auto_session_tracking: false,
            before_send: Some(std::sync::Arc::new(anonymize_event)),
            ..Default::default()
        });
        SDK_ACTIVE.store(true, Ordering::SeqCst);
        if let Ok(mut g) = SDK_GUARD.lock() {
            *g = Some(guard);
        }
        // 必须在 sentry::init 之后注册，避免被 SDK 的 panic integration 覆盖。
        register_panic_hook();
        tracing::info!(environment = environment(), "telemetry SDK initialized");
        true
    }

    /// SDK 是否处于激活状态。
    pub fn sdk_active() -> bool {
        SDK_ACTIVE.load(Ordering::SeqCst)
    }

    /// 注册 panic hook：格式化 panic 信息 → 脱敏 → 同步原子写入 pending 存储。
    ///
    /// hook 内不初始化 SDK / 不弹窗 / 不网络请求；IO 失败静默忽略（防二次
    /// panic 导致无输出 abort）；保留默认 eprintln 输出。
    pub fn register_panic_hook() {
        std::panic::set_hook(Box::new(|info| {
            let payload = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic payload".to_string());
            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "unknown location".to_string());

            // 落盘 pending（best-effort，任何失败都静默忽略）。
            if let Ok(ctx) = CONTEXT.lock()
                && let Some(ctx) = ctx.as_ref()
            {
                let record = PendingRecord {
                    ts: now_millis(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    install_id: ctx.install_id.clone(),
                    code: report_code::RUST_PANIC.to_string(),
                    message: anonymize_text(&format!("panicked at {location}: {payload}")),
                };
                // write_pending 内部已处理全部 IO 错误（best-effort 不落 panic），
                // hook 内绝不允许二次 panic（会直接 abort 且无输出）。
                write_pending(&ctx.app_data_dir, &record);
            }

            // 保留默认 eprintln 输出。
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("<unnamed>");
            eprintln!("thread '{name}' panicked at {location}:\n{payload}");
        }));
    }

    /// 上报启动统计事件（Info 级，不进 issue 列表）。
    ///
    /// 携带 tag：code=PGR-0001 / event_type=startup / priority=p3 /
    /// install_id / version / os / arch。SDK 未激活时不产生任何网络请求。
    pub fn report_startup() {
        if !sdk_active() {
            return;
        }
        let install_id = CONTEXT
            .lock()
            .ok()
            .and_then(|ctx| ctx.as_ref().map(|c| c.install_id.clone()))
            .unwrap_or_default();
        sentry::with_scope(
            |scope| {
                scope.set_tag("code", report_code::APP_STARTUP);
                scope.set_tag("event_type", "startup");
                scope.set_tag("priority", "p3");
                scope.set_tag("install_id", install_id);
                scope.set_tag("version", env!("CARGO_PKG_VERSION"));
                scope.set_tag("os", std::env::consts::OS);
                scope.set_tag("arch", std::env::consts::ARCH);
            },
            || sentry::capture_message("app_startup", sentry::Level::Info),
        );
    }

    /// safe_try! 宏的错误上报出口（宏内调用，勿直接使用）。
    ///
    /// SDK 激活时上报 Error 级事件（tag：event_type=error / priority=p2 /
    /// code / function / location）；SDK 未激活（开关关闭/无 DSN）时
    /// 落盘 pending 存储，不产生任何网络请求。
    #[doc(hidden)]
    pub fn report_safe_try_error(code: &str, function: &str, location: String, message: &str) {
        let message = anonymize_text(message);
        if sdk_active() {
            let location = location.clone();
            sentry::with_scope(
                |scope| {
                    scope.set_tag("code", code.to_string());
                    scope.set_tag("event_type", "error");
                    scope.set_tag("priority", "p2");
                    scope.set_tag("function", function.to_string());
                    scope.set_tag("location", location);
                },
                || sentry::capture_message(&message, sentry::Level::Error),
            );
        } else if let Ok(ctx) = CONTEXT.lock()
            && let Some(ctx) = ctx.as_ref()
        {
            let record = PendingRecord {
                ts: now_millis(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                install_id: ctx.install_id.clone(),
                code: code.to_string(),
                message: format!("[{function} @ {location}] {message}"),
            };
            // 落盘为 best-effort，内部错误全部静默。
            write_pending(&ctx.app_data_dir, &record);
        }
    }

    /// 根据 Code 前缀推断事件分类 tag。
    fn event_class(code: &str) -> (&'static str, &'static str) {
        if code.starts_with("PGR-1") {
            ("crash", "p1")
        } else if code.starts_with("PGR-0") {
            ("startup", "p3")
        } else {
            ("error", "p2")
        }
    }

    /// 上传全部 pending 记录并清除已上传部分，返回上传条数。
    fn upload_all_pending() -> usize {
        let Some((dir,)) = CONTEXT
            .lock()
            .ok()
            .and_then(|ctx| ctx.as_ref().map(|c| (c.app_data_dir.clone(),)))
        else {
            return 0;
        };
        let records = list_pending(&dir);
        if records.is_empty() {
            return 0;
        }
        let mut uploaded_ids = Vec::with_capacity(records.len());
        for (id, record) in &records {
            let (event_type, priority) = event_class(&record.code);
            sentry::with_scope(
                |scope| {
                    scope.set_tag("code", record.code.clone());
                    scope.set_tag("event_type", event_type);
                    scope.set_tag("priority", priority);
                    scope.set_tag("install_id", record.install_id.clone());
                    scope.set_tag("version", record.version.clone());
                },
                || sentry::capture_message(&record.message, sentry::Level::Error),
            );
            uploaded_ids.push(id.clone());
        }
        clear_uploaded(&dir, &uploaded_ids);
        uploaded_ids.len()
    }

    /// 开关开启时启动后无感静默上传全部 pending 历史（无弹窗/同意请求）。
    ///
    /// SDK 未激活时不做任何网络动作（仅保留累积）。
    pub fn upload_pending_silently() {
        if !sdk_active() {
            return;
        }
        let count = upload_all_pending();
        if count > 0 {
            tracing::info!(count, "uploaded pending telemetry records");
        }
    }

    /// 报错页面「匿名上传错误报告」一次性显式授权：
    ///
    /// 把当前错误写入 pending → 临时初始化 SDK（若未激活）→ 上传全部历史
    /// → flush → 关闭 SDK（不继续上报、不修改开关状态）。
    pub fn authorize_upload_all(code: &str, message: &str) -> Result<u32, String> {
        // 当前错误也落盘，与历史一起走统一上传路径。
        {
            let ctx = CONTEXT.lock().map_err(|e| e.to_string())?;
            let ctx = ctx.as_ref().ok_or("telemetry context not initialized")?;
            let record = PendingRecord {
                ts: now_millis(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                install_id: ctx.install_id.clone(),
                code: code.to_string(),
                message: anonymize_text(message),
            };
            write_pending(&ctx.app_data_dir, &record);
        }

        let was_active = sdk_active();
        let temp_guard = if was_active {
            None
        } else {
            let Some(dsn) = dsn() else {
                return Err("telemetry DSN not injected; cannot upload".to_string());
            };
            let guard = sentry::init(sentry::ClientOptions {
                dsn: dsn.parse().ok(),
                release: sentry::release_name!(),
                environment: Some(environment().into()),
                auto_session_tracking: false,
                before_send: Some(std::sync::Arc::new(anonymize_event)),
                ..Default::default()
            });
            SDK_ACTIVE.store(true, Ordering::SeqCst);
            Some(guard)
        };

        let count = upload_all_pending() as u32;

        if let Some(guard) = temp_guard {
            // 一次性授权：flush 后关闭 SDK，不继续上报。
            guard.flush(Some(std::time::Duration::from_secs(5)));
            drop(guard);
            SDK_ACTIVE.store(false, Ordering::SeqCst);
        }
        Ok(count)
    }

    /// 开发者模式「测试上报」：发送一条 Error 级测试事件（进 issue 列表）。
    #[function_name::named]
    pub fn test_report() -> Result<(), String> {
        if !sdk_active() {
            return Err("telemetry SDK not active".to_string());
        }
        sentry::with_scope(
            |scope| {
                scope.set_tag("code", report_code::TEST_REPORT);
                scope.set_tag("event_type", "error");
                scope.set_tag("priority", "p2");
                // function_name!() 由 #[function_name::named] 生成，
                // 展开为当前函数名字符串字面量。
                scope.set_tag("function", function_name!());
            },
            || sentry::capture_message("peregrine telemetry test report", sentry::Level::Error),
        );
        // capture_message 仅把事件入队，由后台传输线程异步发送；
        // 这里显式 flush 5 秒，确保事件在返回前真正发出。否则用户点击后
        // 立即关闭窗口会让传输队列被销毁，表现为「已发送 ✓」但 GlitchTip 收不到。
        if let Ok(guard) = SDK_GUARD.lock()
            && let Some(g) = guard.as_ref()
        {
            g.flush(Some(std::time::Duration::from_secs(5)));
        }
        Ok(())
    }

    /// 前端错误落盘（遥测关闭时经 Tauri command 写入 pending）。
    pub fn store_pending_report(code: &str, message: &str) -> Result<(), String> {
        let ctx = CONTEXT.lock().map_err(|e| e.to_string())?;
        let ctx = ctx.as_ref().ok_or("telemetry context not initialized")?;
        let record = PendingRecord {
            ts: now_millis(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            install_id: ctx.install_id.clone(),
            code: code.to_string(),
            message: anonymize_text(message),
        };
        write_pending(&ctx.app_data_dir, &record);
        Ok(())
    }

    /// 查询 pending 记录条数（Tauri command 出口）。
    pub fn list_pending_reports() -> usize {
        CONTEXT
            .lock()
            .ok()
            .and_then(|ctx| ctx.as_ref().map(|c| pending_count(&c.app_data_dir)))
            .unwrap_or(0)
    }

    /// 测试辅助：设置遥测上下文（不初始化 SDK）。
    #[cfg(test)]
    pub fn set_test_context(app_data_dir: PathBuf, install_id: &str) {
        let mut ctx = CONTEXT.lock().expect("context lock");
        *ctx = Some(TelemetryContext {
            app_data_dir,
            install_id: install_id.to_string(),
        });
    }
}

/// 编译期禁用（`PEREGRINE_DISABLE_TELEMETRY`）时的 no-op 桩：
/// 全部公开 API 保留签名、内部空实现，二进制不含任何上报代码路径。
#[cfg(peregrine_disable_telemetry)]
mod imp {
    use super::PendingRecord;
    use std::path::{Path, PathBuf};

    /// no-op：返回空 install_id，不做任何 IO。
    pub fn get_install_id(_app_data_dir: &Path) -> String {
        String::new()
    }

    /// no-op：不写入任何记录。
    pub fn write_pending(_app_data_dir: &Path, _record: &PendingRecord) {}

    /// no-op：恒返回空列表。
    pub fn list_pending(_app_data_dir: &Path) -> Vec<(String, PendingRecord)> {
        Vec::new()
    }

    /// no-op：不删除任何记录。
    pub fn clear_uploaded(_app_data_dir: &Path, _ids: &[String]) {}

    /// no-op：文本原样返回（无上报路径，无需脱敏）。
    pub fn anonymize_text(input: &str) -> String {
        input.to_string()
    }

    /// no-op：不初始化 SDK、不注册 hook，恒返回 false。
    pub fn init(_app_data_dir: PathBuf, _telemetry_enabled: bool) -> bool {
        false
    }

    /// no-op：恒返回 false。
    pub fn sdk_active() -> bool {
        false
    }

    /// no-op：不注册 panic hook。
    pub fn register_panic_hook() {}

    /// no-op：不产生启动事件。
    pub fn report_startup() {}

    /// no-op：safe_try! 宏的错误出口，直接丢弃。
    #[doc(hidden)]
    pub fn report_safe_try_error(_code: &str, _function: &str, _location: String, _message: &str) {}

    /// no-op：不上传历史。
    pub fn upload_pending_silently() {}

    /// no-op：恒返回错误。
    pub fn authorize_upload_all(_code: &str, _message: &str) -> Result<u32, String> {
        Err("telemetry disabled at compile time".to_string())
    }

    /// no-op：恒返回错误。
    pub fn test_report() -> Result<(), String> {
        Err("telemetry disabled at compile time".to_string())
    }

    /// no-op：恒成功但不写入。
    pub fn store_pending_report(_code: &str, _message: &str) -> Result<(), String> {
        Ok(())
    }

    /// no-op：恒返回 0。
    pub fn list_pending_reports() -> usize {
        0
    }
}

pub use imp::*;

// ============================================================================
// 单元测试（仅正常实现路径；桩路径所有函数均为空实现）
// ============================================================================

#[cfg(all(test, not(peregrine_disable_telemetry)))]
mod tests {
    use super::*;

    /// 首次启动生成 install_id 并落盘。
    #[test]
    fn install_id_generated_on_first_run() {
        let dir = tempfile::tempdir().unwrap();
        let id = get_install_id(dir.path());
        assert!(uuid::Uuid::parse_str(&id).is_ok());
        let on_disk = std::fs::read_to_string(dir.path().join("install_id")).unwrap();
        assert_eq!(on_disk, id);
    }

    /// 同一安装多次读取 install_id 不变。
    #[test]
    fn install_id_stable_across_reads() {
        let dir = tempfile::tempdir().unwrap();
        let first = get_install_id(dir.path());
        let second = get_install_id(dir.path());
        assert_eq!(first, second);
    }

    /// install_id 文件损坏（空 / 非法内容）时自动重建。
    #[test]
    fn install_id_rebuilt_when_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("install_id");
        std::fs::write(&path, "not-a-uuid").unwrap();
        let id = get_install_id(dir.path());
        assert!(uuid::Uuid::parse_str(&id).is_ok());

        std::fs::write(&path, "").unwrap();
        let id2 = get_install_id(dir.path());
        assert!(uuid::Uuid::parse_str(&id2).is_ok());
    }

    /// 删除/重置 config.json 不影响 install_id（两者解耦）。
    #[test]
    fn install_id_unaffected_by_config_reset() {
        let dir = tempfile::tempdir().unwrap();
        let id = get_install_id(dir.path());
        // 模拟 config.json 被删除/重建。
        std::fs::write(dir.path().join("config.json"), b"{}").unwrap();
        std::fs::remove_file(dir.path().join("config.json")).unwrap();
        let id2 = get_install_id(dir.path());
        assert_eq!(id, id2);
    }

    /// pending 记录写入 / 列出 / 清除闭环。
    #[test]
    fn pending_write_list_clear() {
        let dir = tempfile::tempdir().unwrap();
        let record = PendingRecord {
            ts: 1000,
            version: "0.0.0-test".to_string(),
            install_id: "test-id".to_string(),
            code: report_code::RUST_PANIC.to_string(),
            message: "boom".to_string(),
        };
        write_pending(dir.path(), &record);
        let records = list_pending(dir.path());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1.message, "boom");

        clear_uploaded(dir.path(), &[records[0].0.clone()]);
        assert!(list_pending(dir.path()).is_empty());
    }

    /// 非超限不主动删除记录。
    #[test]
    fn pending_not_deleted_under_cap() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..5 {
            write_pending(
                dir.path(),
                &PendingRecord {
                    ts: i,
                    version: "0.0.0-test".to_string(),
                    install_id: "test-id".to_string(),
                    code: report_code::CONFIG_IO.to_string(),
                    message: format!("record {i}"),
                },
            );
        }
        assert_eq!(list_pending(dir.path()).len(), 5);
    }

    /// 超过 5MB 时删除最旧记录直至回到上限以内。
    #[test]
    fn pending_rotates_oldest_over_cap() {
        let dir = tempfile::tempdir().unwrap();
        // 单条约 1MB，写 6 条必然超限。
        let big = "x".repeat(1024 * 1024);
        for i in 0..6u64 {
            write_pending(
                dir.path(),
                &PendingRecord {
                    ts: i,
                    version: "0.0.0-test".to_string(),
                    install_id: "test-id".to_string(),
                    code: report_code::RUST_PANIC.to_string(),
                    message: big.clone(),
                },
            );
        }
        let records = list_pending(dir.path());
        assert!(records.len() < 6);
        // 最旧的 ts=0 应已被删除。
        assert!(records.iter().all(|(_, r)| r.ts > 0));
    }

    /// 路径用户名替换：Windows / macOS / Linux 三种形式。
    #[test]
    fn anonymize_text_replaces_user_paths() {
        assert_eq!(
            anonymize_text(r"failed at C:\Users\alice\AppData\x"),
            r"failed at C:\Users\{user}\AppData\x"
        );
        assert_eq!(
            anonymize_text("open /Users/bob/Documents/f"),
            "open /Users/{user}/Documents/f"
        );
        assert_eq!(
            anonymize_text("read /home/carol/.config/x"),
            "read /home/{user}/.config/x"
        );
        // 无路径时原样返回。
        assert_eq!(anonymize_text("plain message"), "plain message");
    }

    /// safe_try!：Ok 直通，不产生上报。
    #[test]
    fn safe_try_ok_passthrough() {
        let result: Result<i32, String> = Ok(42);
        let out = crate::safe_try!(result, report_code::CONFIG_IO);
        assert_eq!(out.unwrap(), 42);
    }

    /// safe_try!：Err 原样返回；SDK 未激活时落盘 pending。
    #[test]
    fn safe_try_err_reported_to_pending() {
        let dir = tempfile::tempdir().unwrap();
        set_test_context(dir.path().to_path_buf(), "test-install-id");
        let result: Result<i32, String> = Err("disk full".to_string());
        let out = crate::safe_try!(result, report_code::CONFIG_IO);
        assert!(out.is_err());
        let records = list_pending(dir.path());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1.code, report_code::CONFIG_IO);
        assert!(records[0].1.message.contains("disk full"));
    }
}
