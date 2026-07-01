//! Peregrine 主程序入口。
//!
//! 当前实现是一个最小可运行的骨架：创建窗口、初始化 wgpu、在同一个 wgpu 实例中
//! 渲染 egui 设置面板。Settings Mode 通过热键切换，可修改辅助贴图类型/颜色/不透明度，
//! 并通过 `peregrine_config` 持久化与广播。

mod renderer;
mod settings_ui;

use peregrine_config::{ConfigNotifier, ConfigStorage, ConfigWatcher};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

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
}

impl App {
    fn new(storage: ConfigStorage, notifier: ConfigNotifier) -> Self {
        let snapshot = notifier.subscribe().borrow().clone();
        Self {
            mode: AppMode::Settings,
            window: None,
            renderer: None,
            storage,
            notifier,
            config: Arc::new(Mutex::new(snapshot)),
            settings_ui: settings_ui::SettingsUi::new(),
            watcher_stop: None,
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
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 首次恢复时创建窗口与渲染器。
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Peregrine")
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
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // 先把窗口事件传给 egui，让它处理鼠标/键盘输入。
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                // 默认启动即进入 Settings，按 Esc 可切回 Overlay 覆盖层。
                if self.mode == AppMode::Settings {
                    self.toggle_mode();
                } else {
                    if let Some(tx) = self.watcher_stop.take() {
                        let _ = tx.send(());
                    }
                    event_loop.exit();
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
                eprintln!("[key] pressed: {:?}, current mode: {:?}", logical_key, self.mode);
                // 按 Tab 进入/退出 Settings Mode（F10 在 macOS 上可能被系统占用，
                // 数字键 1 容易与输入混淆）。
                if matches!(logical_key, Key::Named(NamedKey::Tab)) {
                    eprintln!("[key] toggling mode");
                    self.toggle_mode();
                }
            }
            WindowEvent::RedrawRequested => {
                eprintln!("[redraw] mode: {:?}", self.mode);
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

    fn about_to_wait(&mut self,
        _event_loop: &ActiveEventLoop,
    ) {
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

    let event_loop = EventLoop::new().expect("create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(storage, notifier);
    event_loop.run_app(&mut app).expect("run event loop");
}
