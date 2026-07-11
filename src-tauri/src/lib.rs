//! Peregrine Tauri 后端。
//!
//! 负责：
//! - 初始化配置存储与热重载 notifier
//! - 提供前端调用的 Tauri commands
//! - 在独立线程运行 winit 事件循环管理 overlay 窗口
//! - 使用 Tauri tray 图标管理「配置」「设置」「退出」菜单
//! - 管理「配置窗口」（准心参数）与「设置窗口」（关于等）两个 Webview 窗口

use peregrine_config::{ConfigNotifier, ConfigSnapshot, ConfigStorage};
use std::sync::{Arc, Mutex, mpsc};
use tauri::{
    Emitter, Manager, State, WebviewUrl,
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder},
    webview::WebviewWindowBuilder,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod overlay;

/// 支持的后端 UI 语言。
#[derive(Clone, Copy, PartialEq, Eq)]
enum BackendLocale {
    ZhCN,
    En,
}

impl BackendLocale {
    fn detect() -> Self {
        // Windows 上 LANG 环境变量通常不存在，使用 Win32 API 获取系统 UI 语言。
        #[cfg(windows)]
        {
            use windows::Win32::Globalization::GetUserDefaultLocaleName;
            let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
            let ok = unsafe { GetUserDefaultLocaleName(&mut buf) };
            if ok > 0 {
                let locale = String::from_utf16_lossy(&buf[..ok as usize])
                    .trim_end_matches('\0')
                    .to_lowercase();
                if locale.starts_with("zh") {
                    return BackendLocale::ZhCN;
                }
            }
            return BackendLocale::En;
        }
        // 非 Windows 平台使用环境变量检测。
        #[cfg(not(windows))]
        {
            let locale = std::env::var("LANG")
                .or_else(|_| std::env::var("LC_ALL"))
                .unwrap_or_default()
                .to_lowercase();
            if locale.starts_with("zh") {
                BackendLocale::ZhCN
            } else {
                BackendLocale::En
            }
        }
    }

    fn from_str(s: &str) -> Self {
        if s.to_lowercase().starts_with("zh") {
            BackendLocale::ZhCN
        } else {
            BackendLocale::En
        }
    }
}

fn detect_locale() -> &'static str {
    match BackendLocale::detect() {
        BackendLocale::ZhCN => "zh-CN",
        BackendLocale::En => "en",
    }
}

fn tr(locale: BackendLocale, key: &str) -> String {
    match (locale, key) {
        (BackendLocale::ZhCN, "target_window_required") => "未选择目标窗口".to_string(),
        (BackendLocale::En, "target_window_required") => "No target window selected".to_string(),
        (BackendLocale::ZhCN, "png_filter") => "PNG 图片".to_string(),
        (BackendLocale::En, "png_filter") => "PNG images".to_string(),
        (BackendLocale::ZhCN, "tray.config") => "配置".to_string(),
        (BackendLocale::En, "tray.config") => "Config".to_string(),
        (BackendLocale::ZhCN, "tray.settings") => "设置".to_string(),
        (BackendLocale::En, "tray.settings") => "Settings".to_string(),
        (BackendLocale::ZhCN, "tray.quit") => "退出".to_string(),
        (BackendLocale::En, "tray.quit") => "Quit".to_string(),
        (BackendLocale::ZhCN, "tray.window_mode") => "窗口模式".to_string(),
        (BackendLocale::En, "tray.window_mode") => "Window Mode".to_string(),
        _ => key.to_string(),
    }
}

fn current_locale(state: &AppState) -> BackendLocale {
    let locale = state.locale.lock().map(|s| s.clone()).unwrap_or_default();
    let resolved = if locale == "zh-CN" || locale == "en" {
        locale.as_str()
    } else {
        detect_locale()
    };
    BackendLocale::from_str(resolved)
}

/// 全局应用状态，跨 commands 共享。
pub struct AppState {
    /// 配置存储。
    pub storage: ConfigStorage,
    /// 配置变更广播器。
    pub notifier: ConfigNotifier,
    /// 当前配置快照（共享给 overlay 渲染器）。
    pub config: Arc<Mutex<ConfigSnapshot>>,
    /// 向 overlay 管理线程发送命令。
    pub overlay_cmd_tx: mpsc::Sender<overlay::OverlayCommand>,
    /// 当前 UI 语言，用于后端错误提示国际化。
    pub locale: Mutex<String>,
    /// 标记是否由托盘「退出」主动触发，避免阻止真正的退出流程。
    pub quitting: std::sync::atomic::AtomicBool,
    /// overlay 是否活跃，供前端查询按钮状态。
    pub overlay_active: std::sync::atomic::AtomicBool,
}

/// 托盘菜单项句柄，用于运行时更新菜单文本与勾选状态。
pub struct TrayMenuState {
    pub config_item: MenuItem<tauri::Wry>,
    pub settings_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
    /// 「窗口模式」勾选项（勾选 = 窗口模式，取消 = 全屏模式）。
    pub window_mode_item: CheckMenuItem<tauri::Wry>,
}

/// 从配置快照读取 GPU 加速设置。
fn read_gpu_setting(app: &impl tauri::Manager<tauri::Wry>) -> bool {
    let state = app.state::<AppState>();
    state
        .config
        .lock()
        .ok()
        .map(|guard| guard.as_ref().settings.gpu_acceleration)
        .unwrap_or(false)
}

/// 创建配置窗口（config）。关闭时由 on_window_event 销毁 WebView2。
fn create_config_window(
    app: &impl tauri::Manager<tauri::Wry>,
) -> tauri::Result<tauri::WebviewWindow> {
    let win_icon =
        Image::from_bytes(include_bytes!("../icons/icon.png")).expect("failed to load window icon");
    let gpu_enabled = read_gpu_setting(app);
    let mut webview_builder =
        WebviewWindowBuilder::new(app, "config", WebviewUrl::App("index.html".into()));
    if !gpu_enabled {
        webview_builder = webview_builder.additional_browser_args("--disable-gpu");
    }
    let window = webview_builder
        .title("Peregrine 配置")
        .inner_size(1080.0, 720.0)
        .min_inner_size(900.0, 600.0)
        .resizable(true)
        .decorations(true)
        .center()
        .skip_taskbar(false)
        .build()?;
    let _ = window.set_icon(win_icon);
    Ok(window)
}

/// 创建设置窗口（settings）。关闭时由 on_window_event 销毁 WebView2。
fn create_settings_window(
    app: &impl tauri::Manager<tauri::Wry>,
) -> tauri::Result<tauri::WebviewWindow> {
    let win_icon =
        Image::from_bytes(include_bytes!("../icons/icon.png")).expect("failed to load window icon");
    let gpu_enabled = read_gpu_setting(app);
    let mut webview_builder =
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()));
    if !gpu_enabled {
        webview_builder = webview_builder.additional_browser_args("--disable-gpu");
    }
    let window = webview_builder
        .title("Peregrine 设置")
        .inner_size(600.0, 420.0)
        .min_inner_size(480.0, 360.0)
        .resizable(true)
        .decorations(true)
        .center()
        .skip_taskbar(false)
        .visible(false)
        .build()?;
    let _ = window.set_icon(win_icon);
    Ok(window)
}

/// 恢复或重新创建指定标签的窗口。
fn show_or_recreate_window<F>(app: &tauri::AppHandle, label: &str, create: F)
where
    F: FnOnce(&tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow>,
{
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    } else if let Ok(window) = create(app) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 启动 Tauri 应用。
pub fn run() {
    init_logging();

    let storage = ConfigStorage::with_default_path().expect("config storage path");
    let config = tauri::async_runtime::block_on(storage.load_or_create_default())
        .expect("load or create config");
    let notifier = ConfigNotifier::new(config);
    let snapshot = notifier.subscribe().borrow().clone();
    let shared_config = Arc::new(Mutex::new(snapshot.clone()));

    // 启动 overlay 管理线程（独立的 winit 事件循环）。
    let (overlay_cmd_tx, overlay_cmd_rx) = mpsc::channel();
    let overlay_config = shared_config.clone();
    std::thread::spawn(move || {
        overlay::run_overlay_loop(overlay_config, overlay_cmd_rx);
    });

    // 启动 watcher 任务，把 notifier 变更同步到共享快照。
    let watcher_storage = storage.clone();
    let watcher_notifier = notifier.clone();
    let watcher_config = shared_config.clone();
    let watcher_overlay_cmd_tx = overlay_cmd_tx.clone();
    tauri::async_runtime::spawn(async move {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let watcher =
            peregrine_config::ConfigWatcher::new(watcher_storage, watcher_notifier.clone());
        let _handle = watcher.spawn(rx);
        let mut sub = watcher_notifier.subscribe();
        loop {
            if sub.changed().await.is_err() {
                break;
            }
            let snap = sub.borrow().clone();
            *watcher_config.lock().expect("config lock") = snap.clone();
            let _ = watcher_overlay_cmd_tx.send(overlay::OverlayCommand::UpdateConfig(snap));
        }
        // 程序退出时发送停止信号给 watcher。
        let _ = tx.send(());
    });

    // 初始 locale：配置为 "auto" 或空时通过环境变量检测系统语言，否则直接使用保存值。
    let initial_locale = {
        let saved = snapshot.settings.locale.as_str();
        if saved == "zh-CN" || saved == "en" {
            saved.to_string()
        } else {
            detect_locale().to_string()
        }
    };

    let state = AppState {
        storage,
        notifier,
        config: shared_config,
        overlay_cmd_tx,
        locale: Mutex::new(initial_locale.clone()),
        quitting: std::sync::atomic::AtomicBool::new(false),
        overlay_active: std::sync::atomic::AtomicBool::new(false),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .on_window_event(|window, event| {
            // 关闭窗口时真正销毁 WebView2 渲染进程（~30-50MB/窗口），
            // 而非 hide 保留在内存中。下次打开由 show_or_recreate_window 重建。
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.destroy();
            }
        })
        .setup(move |app| {
            // 根据 locale 初始化托盘菜单文本。
            let locale = BackendLocale::from_str(&initial_locale);
            let config_label = tr(locale, "tray.config");
            let settings_label = tr(locale, "tray.settings");
            let quit_label = tr(locale, "tray.quit");
            let window_mode_label = tr(locale, "tray.window_mode");

            let config_i = MenuItem::with_id(app, "config", &config_label, true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(app, "settings", &settings_label, true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            // 窗口模式勾选：勾选 = 窗口模式，取消 = 全屏模式（默认取消）。
            let window_mode_i = CheckMenuItem::with_id(
                app,
                "window_mode",
                &window_mode_label,
                true,
                !snapshot.settings.fullscreen_overlay,
                None::<&str>,
            )?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", &quit_label, true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &config_i,
                    &settings_i,
                    &sep1,
                    &window_mode_i,
                    &sep2,
                    &quit_i,
                ],
            )?;

            // 保存托盘菜单项句柄，供 update_locale 命令更新文本。
            app.manage(TrayMenuState {
                config_item: config_i.clone(),
                settings_item: settings_i.clone(),
                quit_item: quit_i.clone(),
                window_mode_item: window_mode_i.clone(),
            });

            // 嵌入高分辨率 PNG（512x512）作为托盘图标源，Tauri 会按需缩放。
            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                .expect("failed to load embedded tray icon");

            let tray_builder = TrayIconBuilder::new()
                .tooltip("Peregrine")
                .menu(&menu)
                .icon(icon);
            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "config" => {
                        show_or_recreate_window(app, "config", create_config_window);
                    }
                    "settings" => {
                        show_or_recreate_window(app, "settings", create_settings_window);
                    }
                    "window_mode" => {
                        // 勾选 = 窗口模式（fullscreen_overlay=false），取消 = 全屏模式。
                        let tray_state = app.state::<TrayMenuState>();
                        let is_window_mode =
                            tray_state.window_mode_item.is_checked().unwrap_or(false);
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = update_preferences_inner(
                                app_clone,
                                PreferencesPatch {
                                    locale: None,
                                    auto_switch_on_overlay: None,
                                    fullscreen_overlay: Some(!is_window_mode),
                                    live_drag_preview: None,
                                    gpu_acceleration: None,
                                },
                            )
                            .await;
                        });
                    }
                    "quit" => {
                        // 标记主动退出，避免 ExitRequested 被阻止。
                        let state = app.state::<AppState>();
                        state
                            .quitting
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标恢复配置窗口（不存在则重新创建）。
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_or_recreate_window(app, "config", create_config_window);
                    }
                })
                .build(app)?;

            // 启动时只创建配置窗口并前置；设置窗口按需创建，不占用启动内存。
            let config_window = create_config_window(app)?;
            let _ = config_window.show();
            let _ = config_window.set_focus();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            list_window_titles,
            start_overlay,
            stop_overlay,
            pick_image_path,
            get_overlay_active,
            update_preferences,
            focus_target_window,
            get_app_version,
            relaunch_app,
        ])
        .build(tauri::generate_context!())
        .expect("build tauri app")
        .run(|_app_handle, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    // 窗口关闭时只销毁 WebView，不退出应用；
                    // 托盘点「退出」会设置 quitting 标志，此时允许退出。
                    let state = _app_handle.state::<AppState>();
                    if !state.quitting.load(std::sync::atomic::Ordering::SeqCst) {
                        api.prevent_exit();
                    }
                }
                tauri::RunEvent::Exit => {
                    // 退出时通知 overlay 线程停止。
                    let state = _app_handle.state::<AppState>();
                    let _ = state.overlay_cmd_tx.send(overlay::OverlayCommand::Stop);
                }
                _ => {}
            }
        });
}

/// 初始化 tracing 日志：控制台 + 滚动文件。
fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    let log_path = peregrine_config::ConfigStorage::with_default_path()
        .ok()
        .map(|s| s.path().with_file_name("peregrine.log"))
        .unwrap_or_else(|| std::path::PathBuf::from("peregrine.log"));

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file_appender = tracing_appender::rolling::daily(
        log_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
        log_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("peregrine.log")),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(_guard));

    let file_layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(file_layer)
        .with(filter)
        .init();
}

// ===== Tauri Commands =====

/// 获取当前配置快照。
#[tauri::command]
fn get_config(state: State<AppState>) -> Result<ConfigSnapshot, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// 保存配置并广播变更。
#[tauri::command]
async fn save_config(state: State<'_, AppState>, config: ConfigSnapshot) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;
    state
        .storage
        .save(&config)
        .await
        .map_err(|e| e.to_string())?;
    state
        .notifier
        .update((*config).clone())
        .map_err(|e| e.to_string())?;
    *state.config.lock().map_err(|e| e.to_string())? = config.clone();
    let _ = state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::UpdateConfig(config));
    Ok(())
}

/// 枚举当前可见的顶层窗口标题（仅 Windows）。
#[tauri::command]
fn list_window_titles() -> Vec<String> {
    #[cfg(windows)]
    {
        peregrine::platform::windows::list_window_titles()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// 启动 overlay。
///
/// - 全屏模式：不需要目标窗口，直接覆盖全屏。
/// - 窗口模式：需要选择目标窗口。
#[tauri::command]
fn start_overlay(state: State<AppState>, target_window: String) -> Result<(), String> {
    // 全屏模式不需要目标窗口。
    let is_fullscreen = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.settings.fullscreen_overlay
    };
    if !is_fullscreen && target_window.is_empty() {
        return Err(tr(current_locale(&state), "target_window_required").to_string());
    }
    state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::Start(target_window))
        .map_err(|e| e.to_string())?;
    state
        .overlay_active
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// 停止 overlay。
#[tauri::command]
fn stop_overlay(state: State<AppState>) -> Result<(), String> {
    state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::Stop)
        .map_err(|e| e.to_string())?;
    state
        .overlay_active
        .store(false, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

/// 询问 overlay 是否处于活动状态。
#[tauri::command]
fn get_overlay_active(state: State<AppState>) -> bool {
    state
        .overlay_active
        .load(std::sync::atomic::Ordering::SeqCst)
}

/// 获取应用版本号（从 Cargo.toml / tauri.conf.json 继承）。
#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 重启应用（GPU 加速等需要重新创建 WebView2 的设置变更后调用）。
#[tauri::command]
fn relaunch_app(app: tauri::AppHandle) {
    app.restart();
}

/// 更新应用级偏好设置（locale / auto_switch_on_overlay / fullscreen_overlay / live_drag_preview）。
///
/// - 仅更新传入的字段，其余保持不变。
/// - 写入配置文件、更新内存快照、广播给 overlay。
/// - locale 变化时更新托盘菜单文本并广播事件。
/// - fullscreen_overlay / live_drag_preview 变化时同步托盘勾选状态。
#[tauri::command]
async fn update_preferences(
    app: tauri::AppHandle,
    preferences: PreferencesPatch,
) -> Result<(), String> {
    update_preferences_inner(app, preferences).await
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct PreferencesPatch {
    locale: Option<String>,
    auto_switch_on_overlay: Option<String>,
    fullscreen_overlay: Option<bool>,
    live_drag_preview: Option<bool>,
    gpu_acceleration: Option<bool>,
    update_channel: Option<String>,
}

/// 更新偏好设置的共享逻辑，供 Tauri command 和托盘菜单事件复用。
async fn update_preferences_inner(
    app: tauri::AppHandle,
    preferences: PreferencesPatch,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let tray_state = app.state::<TrayMenuState>();

    let mut config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.as_ref().clone()
    };

    let old_locale = state.locale.lock().map(|s| s.clone()).unwrap_or_default();
    let locale_changed = preferences
        .locale
        .as_deref()
        .is_some_and(|l| l != old_locale);

    // 应用偏好设置变更。
    if let Some(locale) = &preferences.locale {
        config.settings.locale = locale.clone();
        if let Ok(mut guard) = state.locale.lock() {
            *guard = locale.clone();
        }
    }
    if let Some(auto_switch) = &preferences.auto_switch_on_overlay {
        config.settings.auto_switch_on_overlay = auto_switch.clone();
    }
    if let Some(fullscreen) = preferences.fullscreen_overlay {
        config.settings.fullscreen_overlay = fullscreen;
    }
    if let Some(live_drag) = preferences.live_drag_preview {
        config.settings.live_drag_preview = live_drag;
    }
    if let Some(gpu) = preferences.gpu_acceleration {
        config.settings.gpu_acceleration = gpu;
    }
    if let Some(channel) = &preferences.update_channel {
        config.settings.update_channel = channel.clone();
    }

    config.validate().map_err(|e| e.to_string())?;
    state
        .storage
        .save(&config)
        .await
        .map_err(|e| e.to_string())?;
    state
        .notifier
        .update(config.clone())
        .map_err(|e| e.to_string())?;
    let snapshot: ConfigSnapshot = Arc::new(config);
    *state.config.lock().map_err(|e| e.to_string())? = snapshot.clone();
    let _ = state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::UpdateConfig(snapshot.clone()));

    // locale 变化时更新托盘菜单并广播事件。
    if locale_changed {
        let saved = state.locale.lock().map(|s| s.clone()).unwrap_or_default();
        // "auto" 时根据系统语言解析为实际显示语言。
        let resolved = if saved == "zh-CN" || saved == "en" {
            saved.as_str()
        } else {
            detect_locale()
        };
        let bl = BackendLocale::from_str(resolved);
        tray_state
            .config_item
            .set_text(&tr(bl, "tray.config"))
            .map_err(|e| e.to_string())?;
        tray_state
            .settings_item
            .set_text(&tr(bl, "tray.settings"))
            .map_err(|e| e.to_string())?;
        tray_state
            .quit_item
            .set_text(&tr(bl, "tray.quit"))
            .map_err(|e| e.to_string())?;
        tray_state
            .window_mode_item
            .set_text(&tr(bl, "tray.window_mode"))
            .map_err(|e| e.to_string())?;
        app.emit("peregrine:locale-changed", &saved)
            .map_err(|e| e.to_string())?;
    }

    // fullscreen_overlay 变化时同步托盘「窗口模式」勾选状态。
    if let Some(fs) = preferences.fullscreen_overlay {
        let _ = tray_state.window_mode_item.set_checked(!fs);
    }

    // 广播 settings 变更，让所有窗口的 React state 同步更新。
    let settings_json = serde_json::json!({
        "auto_switch_on_overlay": snapshot.as_ref().settings.auto_switch_on_overlay,
        "locale": snapshot.as_ref().settings.locale,
        "fullscreen_overlay": snapshot.as_ref().settings.fullscreen_overlay,
        "live_drag_preview": snapshot.as_ref().settings.live_drag_preview,
        "gpu_acceleration": snapshot.as_ref().settings.gpu_acceleration,
        "update_channel": snapshot.as_ref().settings.update_channel,
    });
    app.emit("peregrine:settings-changed", &settings_json)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 弹出文件选择对话框，返回 PNG 路径。
#[tauri::command]
async fn pick_image_path(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let locale = current_locale(&state);
    let path = app
        .dialog()
        .file()
        .add_filter(tr(locale, "png_filter"), &["png"])
        .blocking_pick_file();
    Ok(path.map(|p| p.to_string()))
}

/// 将焦点切换到指定标题的目标窗口（游戏窗口）。
///
/// Windows 的 `SetForegroundWindow` 有前台锁定限制：只有当前前台窗口的线程
/// 才有权限切换前台。这里通过 `AttachThreadInput` 将当前线程的输入队列
/// 临时附加到目标窗口的线程，使其获得设置前台的权限，然后用
/// `BringWindowToTop` + `ShowWindow` 组合完成切换。
#[tauri::command]
fn focus_target_window(target_window: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows::Win32::UI::WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SW_RESTORE, SW_SHOW,
            ShowWindow,
        };

        let hwnd = peregrine::platform::windows::find_target_window(&target_window)
            .map_err(|e| e.to_string())?;

        unsafe {
            let foreground = GetForegroundWindow();
            let target_thread = GetWindowThreadProcessId(hwnd, None);
            let foreground_thread = GetWindowThreadProcessId(foreground, None);
            let current_thread = GetCurrentThreadId();

            // 如果目标窗口最小化了，先恢复。
            let _ = ShowWindow(hwnd, SW_RESTORE);

            if target_thread != foreground_thread {
                // 将当前线程和目标窗口线程的输入队列附加到前台线程，
                // 使前台权限传递到当前线程，从而可以设置前台窗口。
                let _ = AttachThreadInput(current_thread, foreground_thread, BOOL(1));
                let _ = AttachThreadInput(current_thread, target_thread, BOOL(1));

                let _ = BringWindowToTop(hwnd);
                let _ = ShowWindow(hwnd, SW_SHOW);

                let _ = AttachThreadInput(current_thread, foreground_thread, BOOL(0));
                let _ = AttachThreadInput(current_thread, target_thread, BOOL(0));
            } else {
                let _ = BringWindowToTop(hwnd);
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
        }
    }
    let _ = target_window;
    Ok(())
}
