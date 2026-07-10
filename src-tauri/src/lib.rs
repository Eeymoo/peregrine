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
    Emitter, Manager, State, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder},
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
}

/// 托盘菜单项句柄，用于运行时更新菜单文本。
pub struct TrayMenuState {
    pub config_item: MenuItem<tauri::Wry>,
    pub settings_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
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
    };

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            // 根据 locale 初始化托盘菜单文本。
            let locale = BackendLocale::from_str(&initial_locale);
            let config_label = tr(locale, "tray.config");
            let settings_label = tr(locale, "tray.settings");
            let quit_label = tr(locale, "tray.quit");

            let config_i = MenuItem::with_id(app, "config", &config_label, true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(app, "settings", &settings_label, true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", &quit_label, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&config_i, &settings_i, &quit_i])?;

            // 保存托盘菜单项句柄，供 update_locale 命令更新文本。
            app.manage(TrayMenuState {
                config_item: config_i.clone(),
                settings_item: settings_i.clone(),
                quit_item: quit_i.clone(),
            });

            let mut tray_builder = TrayIconBuilder::new().tooltip("Peregrine").menu(&menu);
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "config" => {
                        if let Some(window) = app.get_webview_window("config") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标恢复配置窗口。
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("config") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 启动时确保配置窗口可见并前置。
            let config_window = app.get_webview_window("config").unwrap();
            let _ = config_window.show();
            let _ = config_window.set_focus();

            // 关闭配置窗口时隐藏到托盘，而不是退出。
            let config_clone = config_window.clone();
            config_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = config_clone.hide();
                }
            });

            // 关闭设置窗口时同样隐藏到托盘。
            let settings_window = app.get_webview_window("settings").unwrap();
            let settings_clone = settings_window.clone();
            settings_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = settings_clone.hide();
                }
            });

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
        ])
        .on_window_event(|_app_handle, event| {
            // 窗口关闭时隐藏到托盘（双重保险）。
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("build tauri app")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // 退出时通知 overlay 线程停止。
                let state = _app_handle.state::<AppState>();
                let _ = state.overlay_cmd_tx.send(overlay::OverlayCommand::Stop);
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

/// 启动 overlay 跟随指定目标窗口。
#[tauri::command]
fn start_overlay(state: State<AppState>, target_window: String) -> Result<(), String> {
    if target_window.is_empty() {
        return Err(tr(current_locale(&state), "target_window_required").to_string());
    }
    state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::Start(target_window))
        .map_err(|e| e.to_string())
}

/// 停止 overlay。
#[tauri::command]
fn stop_overlay(state: State<AppState>) -> Result<(), String> {
    state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::Stop)
        .map_err(|e| e.to_string())
}

/// 询问 overlay 是否处于活动状态。
#[tauri::command]
fn get_overlay_active(state: State<AppState>) -> bool {
    state
        .overlay_cmd_tx
        .send(overlay::OverlayCommand::QueryActive)
        .is_ok()
}

/// 更新应用级偏好设置（locale / auto_switch_on_overlay）。
///
/// - 仅更新传入的字段，其余保持不变。
/// - 写入配置文件、更新内存快照、广播给 overlay。
/// - 如果 locale 发生变化，更新托盘菜单文本并广播 `peregrine:locale-changed` 事件给所有窗口。
#[tauri::command]
async fn update_preferences(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tray_state: State<'_, TrayMenuState>,
    preferences: PreferencesPatch,
) -> Result<(), String> {
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
        .send(overlay::OverlayCommand::UpdateConfig(snapshot));

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
        app.emit("peregrine:locale-changed", &saved)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct PreferencesPatch {
    locale: Option<String>,
    auto_switch_on_overlay: Option<String>,
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
#[tauri::command]
fn focus_target_window(target_window: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
        let hwnd = peregrine::platform::windows::find_target_window(&target_window)
            .map_err(|e| e.to_string())?;
        unsafe { SetForegroundWindow(hwnd) };
    }
    let _ = target_window;
    Ok(())
}
