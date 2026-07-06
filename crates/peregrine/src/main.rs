//! Peregrine 主程序入口。
//!
//! 双窗口架构：设置窗口（wgpu + egui）+ Overlay 窗口（softbuffer 像素缓冲区）。
//! 设置窗口用 GPU 渲染 egui 面板；Overlay 窗口用 CPU 像素缓冲区绘制准心，
//! 参考 simple-crosshair-overlay 的方案，透明天然可靠。
//! Overlay 窗口在点击「开始覆盖」时创建，跟随目标游戏窗口的位置与大小，
//! 目标窗口关闭时自动销毁。

// release 版使用 GUI 子系统，避免打包后的 exe 启动时弹出黑色控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icon;
mod overlay_renderer;
mod platform;
mod renderer;
mod settings_ui;
mod shapes;

use peregrine_config::{ConfigNotifier, ConfigStorage, ConfigWatcher};
use std::sync::{Arc, Mutex};
use tray_icon::TrayIcon;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// 自定义事件：把状态栏菜单点击和 Overlay 跟随结束转发到 winit 事件循环。
#[derive(Debug)]
#[allow(dead_code)]
enum UserEvent {
    /// 状态栏菜单项被点击。
    MenuEvent(MenuEvent),
    /// Overlay 跟随任务结束（目标窗口关闭或出错）。
    /// 仅在 Windows 平台构造。
    OverlayFollowerEnded,
}

/// 应用状态。
struct App {
    /// 设置窗口句柄。
    settings_window: Option<Arc<Window>>,
    /// 设置窗口的 wgpu + egui 渲染器。
    settings_renderer: Option<renderer::Renderer>,
    /// Overlay 窗口句柄（点击「开始覆盖」后创建）。
    overlay_window: Option<Arc<Window>>,
    /// Overlay 窗口的 softbuffer 像素渲染器。
    overlay_renderer: Option<overlay_renderer::OverlayRenderer>,
    /// Overlay 是否处于活动状态（用于 UI 按钮文字切换）。
    overlay_active: bool,
    /// 配置存储。
    storage: ConfigStorage,
    /// 配置广播器。
    notifier: ConfigNotifier,
    /// 当前配置快照，UI 与渲染器共享。
    ///
    /// 使用标准库互斥锁，避免在 tokio runtime 线程内调用 `blocking_lock` 导致 panic。
    config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
    /// UI 状态。
    settings_ui: settings_ui::SettingsUi,
    /// 监听停止信号，程序退出时关闭 watcher。
    watcher_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// 设置窗口是否已收起到状态栏（隐藏）。
    hidden: bool,
    /// 状态栏（托盘）图标句柄，需保持存活，否则图标会从状态栏消失。
    tray_icon: Option<TrayIcon>,
    /// "设置" 菜单项 id。
    menu_settings_id: Option<MenuId>,
    /// "退出" 菜单项 id。
    menu_quit_id: Option<MenuId>,
    /// Overlay 跟随任务的取消发送端。
    overlay_follower_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// 目标游戏窗口标题，用于查找并跟随。
    target_window_title: String,
    /// 事件循环代理，用于从后台任务向主线程发送自定义事件。
    event_proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
}

impl App {
    fn new(storage: ConfigStorage, notifier: ConfigNotifier) -> Self {
        let snapshot = notifier.subscribe().borrow().clone();
        let target_window_title = snapshot
            .active_profile()
            .map(|p| p.target_window.clone())
            .unwrap_or_default();

        Self {
            settings_window: None,
            settings_renderer: None,
            overlay_window: None,
            overlay_renderer: None,
            overlay_active: false,
            storage,
            notifier,
            config: Arc::new(Mutex::new(snapshot)),
            settings_ui: settings_ui::SettingsUi::new(),
            watcher_stop: None,
            hidden: false,
            tray_icon: None,
            menu_settings_id: None,
            menu_quit_id: None,
            overlay_follower_stop: None,
            target_window_title,
            event_proxy: None,
        }
    }

    /// 创建 Overlay 窗口并启动跟随任务。
    ///
    /// Overlay 窗口是无边框透明窗口，参考 simple-crosshair-overlay 的方案：
    /// - `with_transparent(true)` 启用 DWM 透明
    /// - `set_cursor_hittest(false)` 鼠标穿透
    /// - `WindowLevel::AlwaysOnTop` 置顶
    /// - softbuffer 像素缓冲区渲染准心
    #[allow(unused_variables)]
    fn create_overlay(&mut self, event_loop: &ActiveEventLoop) {
        if self.overlay_window.is_some() {
            return; // 已存在，不重复创建。
        }

        if self.target_window_title.is_empty() {
            tracing::warn!("cannot start overlay: no target window selected");
            return;
        }

        tracing::info!("creating overlay window");

        let attributes = Window::default_attributes()
            .with_title("")
            .with_decorations(false)
            .with_transparent(true)
            .with_active(false)
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        // Windows 平台：跳过任务栏 + 禁用拖放。
        #[cfg(windows)]
        let attributes = {
            use winit::platform::windows::WindowAttributesExtWindows;
            attributes.with_skip_taskbar(true).with_drag_and_drop(false)
        };

        let window = match event_loop.create_window(attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("failed to create overlay window: {}", e);
                return;
            }
        };

        // 鼠标穿透（必须在窗口可见后调用）。
        let _ = window.set_cursor_hittest(false);

        // Windows 平台：补充 WS_EX_NOACTIVATE 和 WS_EX_TOOLWINDOW。
        #[cfg(windows)]
        {
            if let Err(e) = platform::windows::setup_overlay_window(&window) {
                tracing::error!("setup overlay window failed: {}", e);
                // 窗口已创建但设置失败，让它自然 drop 销毁。
                return;
            }
        }

        // 初始化 Overlay 的 softbuffer 渲染器。
        let renderer = overlay_renderer::OverlayRenderer::new(window.clone(), self.config.clone());

        self.overlay_window = Some(window.clone());
        self.overlay_renderer = Some(renderer);
        self.overlay_active = true;

        // 启动跟随任务（仅 Windows 有实际跟随逻辑）。
        self.start_overlay_follower();
        window.request_redraw();
    }

    /// 销毁 Overlay 窗口，停止跟随任务。
    fn destroy_overlay(&mut self) {
        tracing::info!("destroying overlay window");
        self.stop_overlay_follower();
        // 先 drop renderer 再 drop window，确保 surface 正确释放。
        self.overlay_renderer = None;
        self.overlay_window = None;
        self.overlay_active = false;
    }

    /// 启动 Overlay 跟随任务。
    ///
    /// 在主线程获取 HWND（winit 限制），然后以 `SendHwnd` 传入异步任务。
    /// 目标窗口不存在时直接报错返回，不创建 Overlay。
    ///
    /// 非 Windows 平台为空实现，Overlay 不跟随目标窗口。
    #[allow(unused_variables)]
    fn start_overlay_follower(&mut self) {
        #[cfg(windows)]
        {
            if self.target_window_title.is_empty() {
                tracing::warn!("no target window selected, overlay will not follow");
                return;
            }
            let Some(overlay_window) = self.overlay_window.clone() else {
                return;
            };

            // 在主线程获取 HWND，避免 winit 跨线程访问报错。
            let overlay_hwnd = match platform::windows::hwnd_from_window(&overlay_window) {
                Ok(h) => platform::windows::SendHwnd(h),
                Err(e) => {
                    tracing::error!("failed to get overlay hwnd: {}", e);
                    return;
                }
            };

            // 同样在主线程查找目标窗口 HWND。
            let title = self.target_window_title.clone();
            let target_hwnd = match platform::windows::find_target_window(&title) {
                Ok(t) => platform::windows::SendHwnd(t),
                Err(e) => {
                    tracing::error!("failed to find target window '{}': {}", title, e);
                    return;
                }
            };

            tracing::info!(title = %title, "starting overlay follower");

            let (tx, rx) = tokio::sync::oneshot::channel();
            self.overlay_follower_stop = Some(tx);

            // 克隆 event proxy，跟随结束后通知主线程销毁 Overlay。
            let proxy = self.event_proxy.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    platform::windows::follow_target_window(overlay_hwnd, target_hwnd, rx).await
                {
                    tracing::info!("overlay follower ended: {}", e);
                }
                // 通知主线程 Overlay 跟随已结束。
                if let Some(proxy) = proxy {
                    let _ = proxy.send_event(UserEvent::OverlayFollowerEnded);
                }
            });
        }

        // 非 Windows 平台：Overlay 跟随暂不支持。
        #[cfg(not(windows))]
        {
            tracing::warn!("overlay window following is not supported on this platform");
        }
    }

    /// 停止 Overlay 跟随任务。
    fn stop_overlay_follower(&mut self) {
        if let Some(tx) = self.overlay_follower_stop.take() {
            let _ = tx.send(());
        }
    }

    /// 构建状态栏（托盘）图标及其菜单（设置 / 退出）。
    fn build_tray(&mut self) -> TrayIcon {
        let settings_item = MenuItem::new("设置", true, None);
        let quit_item = MenuItem::new("退出", true, None);
        self.menu_settings_id = Some(settings_item.id().clone());
        self.menu_quit_id = Some(quit_item.id().clone());

        let menu = Menu::new();
        menu.append(&settings_item)
            .expect("append settings menu item");
        menu.append(&quit_item).expect("append quit menu item");

        tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Peregrine")
            .with_icon(icon::tray_icon())
            .build()
            .expect("build tray icon")
    }

    /// 真正退出程序。
    fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(tx) = self.watcher_stop.take() {
            let _ = tx.send(());
        }
        self.destroy_overlay();
        self.tray_icon = None;
        event_loop.exit();
    }

    /// 从状态栏恢复设置窗口。
    fn show_settings(&mut self) {
        self.hidden = false;
        if let Some(window) = &self.settings_window {
            window.set_visible(true);
            window.focus_window();
            window.request_redraw();
        }
    }

    /// 收起到状态栏：隐藏设置窗口但保持程序在后台运行。
    fn hide_to_tray(&mut self) {
        self.hidden = true;
        if let Some(window) = &self.settings_window {
            window.set_visible(false);
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init && self.tray_icon.is_none() {
            self.tray_icon = Some(self.build_tray());
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 首次恢复时创建设置窗口。
        if self.settings_window.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Peregrine")
                .with_window_icon(Some(icon::window_icon()))
                .with_inner_size(winit::dpi::LogicalSize::new(960.0, 560.0));
            let window = Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("create settings window"),
            );
            let renderer =
                pollster::block_on(renderer::Renderer::new(window.clone(), self.config.clone()));
            self.settings_window = Some(window);
            self.settings_renderer = Some(renderer);

            // 启动配置热重载 watcher。
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.watcher_stop = Some(tx);
            let watcher_storage = self.storage.clone();
            let watcher_notifier = self.notifier.clone();
            let config_clone = self.config.clone();
            tokio::spawn(async move {
                let watcher = ConfigWatcher::new(watcher_storage, watcher_notifier.clone());
                let _handle = watcher.spawn(rx);
                let mut rx = watcher_notifier.subscribe();
                loop {
                    if rx.changed().await.is_err() {
                        break;
                    }
                    let snap = rx.borrow().clone();
                    *config_clone.lock().expect("config lock") = snap;
                }
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // 判断事件来自哪个窗口。
        let is_settings_window = self
            .settings_window
            .as_ref()
            .map(|w| w.id() == window_id)
            .unwrap_or(false);
        let is_overlay_window = self
            .overlay_window
            .as_ref()
            .map(|w| w.id() == window_id)
            .unwrap_or(false);

        // 设置窗口事件交给 egui。
        if is_settings_window {
            if let Some(renderer) = self.settings_renderer.as_mut() {
                renderer.handle_event(&event);
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                if is_settings_window {
                    // 关闭设置窗口不退出程序，收起到状态栏。
                    self.hide_to_tray();
                } else if is_overlay_window {
                    // Overlay 窗口不应该收到 CloseRequested（穿透窗口无关闭按钮），
                    // 但以防万一直接销毁。
                    self.destroy_overlay();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if is_settings_window {
                    tracing::debug!(?logical_key, "settings key pressed");
                    match logical_key {
                        // 设置窗口按 Esc 收起到状态栏。
                        Key::Named(NamedKey::Escape) => {
                            self.hide_to_tray();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if is_settings_window {
                    if let Some(renderer) = self.settings_renderer.as_mut() {
                        renderer.resize(size);
                    }
                } else if is_overlay_window {
                    if let Some(renderer) = self.overlay_renderer.as_mut() {
                        renderer.resize(size);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // 设置窗口重绘。
                if is_settings_window {
                    if self.hidden {
                        return;
                    }
                    if let Some(renderer) = self.settings_renderer.as_mut() {
                        let rt = tokio::runtime::Handle::current();
                        let config = self.config.lock().expect("config lock").clone();
                        // 同步 overlay_active 状态到 UI。
                        self.settings_ui.overlay_active = self.overlay_active;
                        let response = renderer.render_settings(&mut self.settings_ui, &config);
                        if response.changed {
                            let new_target = response
                                .config
                                .active_profile()
                                .map(|p| p.target_window.clone())
                                .unwrap_or_default();
                            let target_changed = new_target != self.target_window_title;
                            *self.config.lock().expect("config lock") =
                                response.config.clone();
                            self.target_window_title = new_target;
                            let storage = self.storage.clone();
                            let notifier = self.notifier.clone();
                            let new_config = response.config.clone();
                            let _ = rt.spawn(async move {
                                if let Err(e) = storage.save(&new_config).await {
                                    tracing::error!("failed to save config: {}", e);
                                    return;
                                }
                                if let Err(e) = notifier.update((*new_config).clone()) {
                                    tracing::error!("failed to notify config: {}", e);
                                }
                            });
                            // Overlay 运行中目标窗口变化：销毁旧 Overlay 并重建跟随新窗口。
                            if target_changed && self.overlay_active {
                                tracing::info!("target window changed, recreating overlay");
                                self.destroy_overlay();
                                self.create_overlay(event_loop);
                            }
                        }
                        // 处理「开始覆盖」按钮。
                        if response.start_overlay {
                            self.create_overlay(event_loop);
                        }
                        // 处理「停止覆盖」按钮。
                        if response.stop_overlay {
                            self.destroy_overlay();
                        }
                    }
                    if let Some(window) = &self.settings_window {
                        window.request_redraw();
                    }
                }
                // Overlay 窗口重绘（穿透窗口正常情况下收不到此事件，
                // overlay 渲染在 about_to_wait 中直接调用）。
                else if is_overlay_window {
                    if let Some(renderer) = self.overlay_renderer.as_mut() {
                        renderer.render_overlay();
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(menu_event) => {
                if self.menu_quit_id.as_ref() == Some(&menu_event.id) {
                    self.shutdown(event_loop);
                } else if self.menu_settings_id.as_ref() == Some(&menu_event.id) {
                    self.show_settings();
                }
            }
            UserEvent::OverlayFollowerEnded => {
                // 目标窗口关闭或跟随任务出错，销毁 Overlay。
                tracing::info!("overlay follower ended event received");
                self.destroy_overlay();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 设置窗口可见时持续重绘。
        if !self.hidden {
            if let Some(window) = &self.settings_window {
                window.request_redraw();
            }
        }
        // Overlay 窗口：透明穿透窗口收不到 RedrawRequested 事件，
        // 在此处直接渲染。
        if let Some(renderer) = self.overlay_renderer.as_mut() {
            renderer.render_overlay();
        }
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    let storage = ConfigStorage::with_default_path().expect("config storage path");
    let config = storage
        .load_or_create_default()
        .await
        .expect("load or create config");
    let notifier = ConfigNotifier::new(config);

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();
    // 菜单事件转发 proxy。
    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = App::new(storage, notifier);
    app.event_proxy = Some(proxy);
    event_loop.run_app(&mut app).expect("run event loop");
}

/// 初始化 tracing 日志：控制台 + 滚动文件。
///
/// 日志写入 `%APPDATA%/Peregrine/peregrine.log`。
fn init_logging() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

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
