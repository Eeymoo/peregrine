//! Overlay 窗口管理。
//!
//! 在独立线程中运行 winit 事件循环，负责创建/销毁透明穿透的 overlay 窗口，
//! 并启动目标窗口跟随任务。

#[cfg(windows)]
use peregrine::overlay_renderer;
#[cfg(windows)]
use peregrine::platform;
use peregrine_config::ConfigSnapshot;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
#[cfg(windows)]
use winit::application::ApplicationHandler;
#[cfg(windows)]
use winit::event::WindowEvent;
#[cfg(windows)]
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(windows)]
use winit::window::{Window, WindowId};

/// 发送给 overlay 管理线程的命令。
#[derive(Debug, Clone)]
pub enum OverlayCommand {
    /// 启动 overlay 并跟随指定标题的目标窗口。
    Start(String),
    /// 停止并销毁 overlay。
    Stop,
    /// 更新当前配置快照。
    UpdateConfig(ConfigSnapshot),
    /// 查询是否活跃（用于 heartbeat）。
    QueryActive,
}

/// 内部自定义事件：把外部命令转发进 winit 事件循环。
#[cfg(windows)]
enum UserEvent {
    Command(OverlayCommand),
}

/// 在独立线程中启动 overlay 事件循环。
pub fn run_overlay_loop(
    #[cfg(windows)] config: Arc<Mutex<ConfigSnapshot>>,
    #[cfg(not(windows))] _config: Arc<Mutex<ConfigSnapshot>>,
    cmd_rx: mpsc::Receiver<OverlayCommand>,
) {
    #[cfg(not(windows))]
    {
        // 非 Windows 平台仅消费命令，避免在主线程外创建 winit EventLoop。
        while let Ok(_cmd) = cmd_rx.recv() {}
        return;
    }
    #[cfg(windows)]
    run_overlay_loop_windows(config, cmd_rx);
}

#[cfg(windows)]
fn run_overlay_loop_windows(
    config: Arc<Mutex<ConfigSnapshot>>,
    cmd_rx: mpsc::Receiver<OverlayCommand>,
) {
    use winit::platform::windows::EventLoopBuilderExtWindows;

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_any_thread(true)
        .build()
        .expect("create overlay event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();
    // 把外部命令转发进事件循环。
    std::thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv() {
            if proxy.send_event(UserEvent::Command(cmd)).is_err() {
                break;
            }
        }
    });

    let mut app = OverlayApp::new(config);
    event_loop
        .run_app(&mut app)
        .expect("run overlay event loop");
}

#[cfg(windows)]
struct OverlayApp {
    config: Arc<Mutex<ConfigSnapshot>>,
    window: Option<Arc<Window>>,
    renderer: Option<overlay_renderer::OverlayRenderer>,
    overlay_active: bool,
    target_title: String,
    follower_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// 上一帧渲染时间，用于限制 overlay 帧率避免空转占 CPU。
    last_render: Option<Instant>,
    /// 目标帧间隔（60 FPS ≈ 16.6 ms）。
    frame_interval: Duration,
}

#[cfg(windows)]
impl OverlayApp {
    fn new(config: Arc<Mutex<ConfigSnapshot>>) -> Self {
        Self {
            config,
            window: None,
            renderer: None,
            overlay_active: false,
            target_title: String::new(),
            follower_stop: None,
            last_render: None,
            frame_interval: Duration::from_nanos(16_666_667),
        }
    }

    fn handle_command(&mut self, event_loop: &ActiveEventLoop, cmd: OverlayCommand) {
        match cmd {
            OverlayCommand::Start(title) => self.create_overlay(event_loop, title),
            OverlayCommand::Stop => self.destroy_overlay(),
            OverlayCommand::UpdateConfig(snap) => {
                *self.config.lock().expect("config lock") = snap;
            }
            OverlayCommand::QueryActive => {}
        }
    }

    fn create_overlay(&mut self, event_loop: &ActiveEventLoop, title: String) {
        if self.window.is_some() {
            // 已存在：如果目标相同则忽略，否则重建。
            if self.target_title == title {
                return;
            }
            self.destroy_overlay();
        }

        if title.is_empty() {
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

        let _ = window.set_cursor_hittest(false);

        #[cfg(windows)]
        {
            if let Err(e) = platform::windows::setup_overlay_window(&window) {
                tracing::error!("setup overlay window failed: {}", e);
                return;
            }
        }

        let renderer = overlay_renderer::OverlayRenderer::new(window.clone(), self.config.clone());

        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.overlay_active = true;
        self.target_title = title.clone();

        self.start_follower(title);
        window.request_redraw();
    }

    fn destroy_overlay(&mut self) {
        if self.window.is_none() {
            return;
        }
        tracing::info!("destroying overlay window");
        self.stop_follower();
        self.renderer = None;
        self.window = None;
        self.overlay_active = false;
        self.target_title.clear();
    }

    #[allow(unused_variables)]
    fn start_follower(&mut self, title: String) {
        #[cfg(windows)]
        {
            let Some(window) = self.window.clone() else {
                return;
            };
            let overlay_hwnd = match platform::windows::hwnd_from_window(&window) {
                Ok(h) => platform::windows::SendHwnd(h),
                Err(e) => {
                    tracing::error!("failed to get overlay hwnd: {}", e);
                    return;
                }
            };
            let target_hwnd = match platform::windows::find_target_window(&title) {
                Ok(t) => platform::windows::SendHwnd(t),
                Err(e) => {
                    tracing::error!("failed to find target window '{}': {}", title, e);
                    return;
                }
            };

            tracing::info!(title = %title, "starting overlay follower");

            let (tx, rx) = tokio::sync::oneshot::channel();
            self.follower_stop = Some(tx);

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
                rt.block_on(async move {
                    if let Err(e) =
                        platform::windows::follow_target_window(overlay_hwnd, target_hwnd, rx).await
                    {
                        tracing::info!("overlay follower ended: {}", e);
                    }
                });
            });
        }
        #[cfg(not(windows))]
        {
            tracing::warn!("overlay window following is not supported on this platform");
        }
    }

    fn stop_follower(&mut self) {
        if let Some(tx) = self.follower_stop.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(windows)]
impl ApplicationHandler<UserEvent> for OverlayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Command(cmd) => self.handle_command(event_loop, cmd),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.destroy_overlay();
            }
            WindowEvent::Resized(_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    // OverlayRenderer 的 resize 目前为空实现，大小变化在 render 中处理。
                    renderer.resize(_size);
                }
            }
            WindowEvent::RedrawRequested => {
                // 额外兜底：防止外部事件在短时间内触发多次重绘。
                let now = Instant::now();
                if let Some(last) = self.last_render {
                    if now.saturating_duration_since(last) < self.frame_interval {
                        return;
                    }
                }
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.render_overlay();
                    self.last_render = Some(now);
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(now + self.frame_interval));
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            // overlay 未运行时避免用旧的 WaitUntil 空转。
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };

        // 限制帧率：静态准心不需要无限制重绘，60 FPS 足够覆盖 RandomOrb 动画。
        let now = Instant::now();
        if let Some(last) = self.last_render {
            let elapsed = now.saturating_duration_since(last);
            if elapsed < self.frame_interval {
                event_loop.set_control_flow(ControlFlow::WaitUntil(last + self.frame_interval));
                return;
            }
        }

        window.request_redraw();
    }
}
