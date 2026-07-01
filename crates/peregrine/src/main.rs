//! Peregrine 主程序入口。
//!
//! 当前实现是一个最小可运行的骨架：创建窗口、初始化 wgpu、在同一个 wgpu 实例中
//! 渲染 egui 设置面板。Settings Mode 通过热键切换，可修改辅助贴图类型/颜色/不透明度，
//! 并通过 `peregrine_config` 持久化与广播。

mod icon;
mod platform;
mod renderer;
mod settings_ui;

use peregrine_config::{ConfigNotifier, ConfigStorage, ConfigWatcher};
use std::sync::{Arc, Mutex};
use tray_icon::TrayIcon;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// 自定义事件：把状态栏菜单点击转发到 winit 事件循环，使其被唤醒。
#[derive(Debug)]
enum UserEvent {
    /// 状态栏菜单项被点击。
    MenuEvent(MenuEvent),
}

/// 程序运行模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    /// 正常显示准心覆盖层。
    Overlay,
    /// 显示 egui 设置界面。
    Settings,
}

/// 应用状态。
struct App {
    /// 当前显示模式。
    mode: AppMode,
    /// 窗口句柄，创建后填充。
    window: Option<Arc<Window>>,
    /// wgpu + egui 渲染器。
    renderer: Option<renderer::Renderer>,
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
    /// 窗口是否已收起到状态栏（隐藏）。
    hidden: bool,
    /// 状态栏（托盘）图标句柄，需保持存活，否则图标会从状态栏消失。
    tray_icon: Option<TrayIcon>,
    /// "设置" 菜单项 id。
    menu_settings_id: Option<MenuId>,
    /// "退出" 菜单项 id。
    menu_quit_id: Option<MenuId>,
    /// Windows 平台：Overlay 跟随任务的取消发送端。
    #[cfg(target_os = "windows")]
    overlay_follower_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// Windows 平台：目标游戏窗口标题，用于查找并跟随。
    #[cfg(target_os = "windows")]
    target_window_title: String,
}

impl App {
    fn new(storage: ConfigStorage, notifier: ConfigNotifier) -> Self {
        let snapshot = notifier.subscribe().borrow().clone();
        #[cfg(target_os = "windows")]
        let target_window_title = snapshot
            .active_profile()
            .map(|p| p.target_window.clone())
            .unwrap_or_default();

        Self {
            mode: AppMode::Settings,
            window: None,
            renderer: None,
            storage,
            notifier,
            config: Arc::new(Mutex::new(snapshot)),
            settings_ui: settings_ui::SettingsUi::new(),
            watcher_stop: None,
            hidden: false,
            tray_icon: None,
            menu_settings_id: None,
            menu_quit_id: None,
            #[cfg(target_os = "windows")]
            overlay_follower_stop: None,
            #[cfg(target_os = "windows")]
            target_window_title,
        }
    }

    /// 模式切换：Overlay <-> Settings。
    ///
    /// 从 Settings 切回 Overlay 时，强制把当前配置快照持久化一次，避免未触发保存的
    /// 修改丢失（例如只改了颜色但未动其他控件）。
    fn toggle_mode(&mut self) {
        if self.mode == AppMode::Settings {
            // 切出设置前保存当前配置快照。
            if let Ok(config) = self.config.lock() {
                let config = config.clone();
                let storage = self.storage.clone();
                let notifier = self.notifier.clone();
                tokio::spawn(async move {
                    if let Err(e) = storage.save(&config).await {
                        tracing::error!("failed to save config on mode exit: {}", e);
                        return;
                    }
                    if let Err(e) = notifier.update((*config).clone()) {
                        tracing::error!("failed to notify config on mode exit: {}", e);
                    }
                });
            }
        }
        self.mode = match self.mode {
            AppMode::Overlay => AppMode::Settings,
            AppMode::Settings => AppMode::Overlay,
        };

        #[cfg(target_os = "windows")]
        {
            if let Some(window) = self.window.clone() {
                match self.mode {
                    AppMode::Overlay => {
                        if let Err(e) = platform::windows::setup_overlay_window(&window) {
                            tracing::error!("setup overlay window failed: {}", e);
                        } else {
                            self.start_overlay_follower();
                        }
                    }
                    AppMode::Settings => {
                        self.stop_overlay_follower();
                        if let Err(e) = platform::windows::restore_normal_window(&window) {
                            tracing::error!("restore normal window failed: {}", e);
                        }
                    }
                }
                window.request_redraw();
            }
        }
    }

    /// 把 UI 中的修改写回磁盘并广播。
    ///
    /// 当前由渲染循环内联调用，保留此辅助方法供后续重构使用。
    #[allow(dead_code)]
    async fn save_config(&self,
        config: peregrine_config::ConfigSnapshot,
    ) -> peregrine_config::Result<()> {
        self.storage.save(&config).await?;
        self.notifier.update((*config).clone())?;
        Ok(())
    }

    /// 构建状态栏（托盘）图标及其菜单（设置 / 退出）。
    ///
    /// 记录两个菜单项的 id，供 `user_event` 分辨点击的是哪一项。
    fn build_tray(&mut self) -> TrayIcon {
        let settings_item = MenuItem::new("设置", true, None);
        let quit_item = MenuItem::new("退出", true, None);
        self.menu_settings_id = Some(settings_item.id().clone());
        self.menu_quit_id = Some(quit_item.id().clone());

        let menu = Menu::new();
        menu.append(&settings_item).expect("append settings menu item");
        menu.append(&quit_item).expect("append quit menu item");

        tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Peregrine")
            .with_icon(icon::tray_icon())
            .build()
            .expect("build tray icon")
    }

    /// 真正退出程序：停止 watcher、移除状态栏图标并退出事件循环。
    fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(tx) = self.watcher_stop.take() {
            let _ = tx.send(());
        }
        #[cfg(target_os = "windows")]
        self.stop_overlay_follower();
        // 主动释放托盘图标，确保其从状态栏移除。
        self.tray_icon = None;
        event_loop.exit();
    }

    /// 从状态栏恢复：显示窗口并切到设置模式。
    fn show_settings(&mut self) {
        self.mode = AppMode::Settings;
        self.hidden = false;
        #[cfg(target_os = "windows")]
        self.stop_overlay_follower();
        if let Some(window) = &self.window {
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = platform::windows::restore_normal_window(window) {
                    tracing::error!("restore normal window failed: {}", e);
                }
            }
            window.set_visible(true);
            window.focus_window();
            window.request_redraw();
        }
    }

    /// 收起到状态栏：隐藏窗口但保持程序在后台运行。
    fn hide_to_tray(&mut self) {
        self.hidden = true;
        #[cfg(target_os = "windows")]
        self.stop_overlay_follower();
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
    }
}

#[cfg(target_os = "windows")]
impl App {
    /// 启动 Overlay 跟随任务。
    ///
    /// 若已配置目标窗口标题，则查找该窗口并在后台任务中以 16ms 周期同步 Overlay 位置。
    fn start_overlay_follower(&mut self) {
        if self.target_window_title.is_empty() {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        let title = self.target_window_title.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.overlay_follower_stop = Some(tx);

        tokio::spawn(async move {
            let overlay = match platform::windows::hwnd_from_window(&window) {
                Ok(h) => platform::windows::SendHwnd(h),
                Err(e) => {
                    tracing::error!("failed to get overlay hwnd: {}", e);
                    return;
                }
            };
            let target = match platform::windows::find_target_window(&title) {
                Ok(t) => platform::windows::SendHwnd(t),
                Err(e) => {
                    tracing::error!("failed to find target window '{}': {}", title, e);
                    return;
                }
            };
            if let Err(e) = platform::windows::follow_target_window(overlay, target, rx).await {
                tracing::debug!("overlay follower ended: {}", e);
            }
        });
    }

    /// 停止 Overlay 跟随任务。
    fn stop_overlay_follower(&mut self) {
        if let Some(tx) = self.overlay_follower_stop.take() {
            let _ = tx.send(());
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // 状态栏图标必须在事件循环启动后（StartCause::Init）、于主线程创建，
        // 这是 tray-icon 在 winit（尤其 macOS）下要求的最早时机。
        if cause == StartCause::Init && self.tray_icon.is_none() {
            self.tray_icon = Some(self.build_tray());
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 首次恢复时创建窗口与渲染器。
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Peregrine")
                            .with_window_icon(Some(icon::window_icon()))
                            .with_inner_size(winit::dpi::LogicalSize::new(960.0, 560.0)),
                    )
                    .expect("create window"),
            );
            let renderer = pollster::block_on(renderer::Renderer::new(
                window.clone(),
                self.config.clone(),
            ));
            self.window = Some(window);
            self.renderer = Some(renderer);

            // 启动配置热重载 watcher。
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.watcher_stop = Some(tx);
            let watcher_storage = self.storage.clone();
            let watcher_notifier = self.notifier.clone();
            let config_clone = self.config.clone();
            tokio::spawn(async move {
                let watcher = ConfigWatcher::new(watcher_storage, watcher_notifier.clone());
                let _handle = watcher.spawn(rx);
                // 同时监听广播并更新本地快照。
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
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // 先把窗口事件传给 egui，让它处理鼠标/键盘输入。
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                // 关闭窗口不退出程序，收起到状态栏后台运行；真正退出请用状态栏菜单。
                self.hide_to_tray();
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
                tracing::debug!(?logical_key, mode = ?self.mode, "key pressed");
                match logical_key {
                    // 设置模式下按 Esc 切回覆盖层；覆盖层下按 Esc 收起到状态栏。
                    Key::Named(NamedKey::Escape) => {
                        if self.mode == AppMode::Settings {
                            self.toggle_mode();
                        } else {
                            self.hide_to_tray();
                        }
                    }
                    // 按 Tab 在 覆盖层 / 设置 之间切换（F10 在 macOS 上可能被系统占用，
                    // 数字键 1 容易与输入混淆）。
                    Key::Named(NamedKey::Tab) => {
                        self.toggle_mode();
                    }
                    _ => {}
                }
            }
            WindowEvent::Resized(size) => {
                // Overlay 跟随目标窗口大小时会触发此事件，同步到 wgpu 表面。
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                // 已收起到状态栏时不渲染，避免向隐藏窗口提交绘制。
                if self.hidden {
                    return;
                }
                // 根据模式绘制覆盖层或设置界面。
                if let Some(renderer) = self.renderer.as_mut() {
                    let rt = tokio::runtime::Handle::current();
                    match self.mode {
                        AppMode::Overlay => {
                            renderer.render_overlay();
                        }
                        AppMode::Settings => {
                            let config = self.config.lock().expect("config lock").clone();
                            let response = renderer.render_settings(
                                &mut self.settings_ui,
                                &config,
                            );
                            if response.changed {
                                *self.config.lock().expect("config lock") = response.config.clone();
                                #[cfg(target_os = "windows")]
                                {
                                    self.target_window_title = response
                                        .config
                                        .active_profile()
                                        .map(|p| p.target_window.clone())
                                        .unwrap_or_default();
                                }
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
                            }
                        }
                    }
                    // 请求持续重绘，保证 UI 响应。
                    if let Some(window) = &self.window {
                        window.request_redraw();
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
        }
    }

    fn about_to_wait(&mut self,
        _event_loop: &ActiveEventLoop,
    ) {
        // 收起到状态栏时进入空闲，等待状态栏菜单事件唤醒，避免持续重绘空转。
        if self.hidden {
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

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

    // 将状态栏菜单事件转发到事件循环，使其在每次点击时被唤醒并派发到 user_event。
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = App::new(storage, notifier);
    event_loop.run_app(&mut app).expect("run event loop");
}
