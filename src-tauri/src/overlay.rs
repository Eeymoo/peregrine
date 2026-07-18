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
    /// 切换 overlay 显示状态：活跃时停止，非活跃时用配置中的 target_window 启动。
    ToggleOverlay,
    /// 更新当前配置快照。
    UpdateConfig(ConfigSnapshot),
    /// 查询是否活跃（用于 heartbeat）。
    QueryActive,
    /// 标记需要重绘（follower 调整窗口位置后触发）。
    Invalidate,
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
    material_registry: Arc<peregrine_material::MaterialRegistry>,
    cmd_rx: mpsc::Receiver<OverlayCommand>,
) {
    #[cfg(not(windows))]
    {
        // 非 Windows 平台仅消费命令，避免在主线程外创建 winit EventLoop。
        let _ = material_registry;
        while let Ok(_cmd) = cmd_rx.recv() {}
        return;
    }
    #[cfg(windows)]
    run_overlay_loop_windows(config, material_registry, cmd_rx);
}

#[cfg(windows)]
fn run_overlay_loop_windows(
    config: Arc<Mutex<ConfigSnapshot>>,
    material_registry: Arc<peregrine_material::MaterialRegistry>,
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

    // 复制一份事件循环代理，用于 follower 线程在移动 overlay 后请求重绘。
    let redraw_proxy = event_loop.create_proxy();
    let mut app = OverlayApp::new(config, material_registry, redraw_proxy);
    event_loop
        .run_app(&mut app)
        .expect("run overlay event loop");
}

#[cfg(windows)]
struct OverlayApp {
    config: Arc<Mutex<ConfigSnapshot>>,
    material_registry: Arc<peregrine_material::MaterialRegistry>,
    window: Option<Arc<Window>>,
    renderer: Option<overlay_renderer::OverlayRenderer>,
    overlay_active: bool,
    target_title: String,
    follower_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// 事件循环代理：follower 线程移动 overlay 窗口后，
    /// 通过它把 `Invalidate` 命令转发回事件循环，触发重绘。
    redraw_proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    /// 上一帧渲染时间，用于限制 overlay 帧率避免空转占 CPU。
    last_render: Option<Instant>,
    /// 目标帧间隔（60 FPS ≈ 16.6 ms）。
    frame_interval: Duration,
    /// 静态准心脏标记：仅在配置变化/窗口尺寸变化时为 true，渲染后清除。
    needs_redraw: bool,
}

#[cfg(windows)]
impl OverlayApp {
    fn new(
        config: Arc<Mutex<ConfigSnapshot>>,
        material_registry: Arc<peregrine_material::MaterialRegistry>,
        redraw_proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    ) -> Self {
        Self {
            config,
            material_registry,
            window: None,
            renderer: None,
            overlay_active: false,
            target_title: String::new(),
            follower_stop: None,
            redraw_proxy,
            last_render: None,
            frame_interval: Duration::from_nanos(16_666_667),
            needs_redraw: false,
        }
    }

    fn handle_command(&mut self, event_loop: &ActiveEventLoop, cmd: OverlayCommand) {
        match cmd {
            OverlayCommand::Start(title) => self.create_overlay(event_loop, title),
            OverlayCommand::Stop => self.destroy_overlay(),
            OverlayCommand::ToggleOverlay => {
                if self.overlay_active {
                    self.destroy_overlay();
                } else {
                    // 从配置快照中读取目标窗口标题。
                    let title = {
                        let cfg = self.config.lock().expect("config lock");
                        cfg.active_profile()
                            .map(|p| p.target_window.clone())
                            .unwrap_or_default()
                    };
                    self.create_overlay(event_loop, title);
                }
            }
            OverlayCommand::UpdateConfig(snap) => {
                // 检测 fullscreen_overlay / live_drag_preview 是否变化，需要重启 follower。
                let old_fullscreen = {
                    let cfg = self.config.lock().expect("config lock");
                    cfg.settings.fullscreen_overlay
                };
                let old_live_drag = {
                    let cfg = self.config.lock().expect("config lock");
                    cfg.settings.live_drag_preview
                };

                let need_restart_follower = self.overlay_active
                    && (snap.settings.fullscreen_overlay != old_fullscreen
                        || snap.settings.live_drag_preview != old_live_drag);

                *self.config.lock().expect("config lock") = snap;

                // 配置变化，静态准心需要重绘。
                self.needs_redraw = true;

                if need_restart_follower {
                    let title = self.target_title.clone();
                    self.stop_follower();
                    self.start_follower(title);
                }
            }
            OverlayCommand::QueryActive => {}
            OverlayCommand::Invalidate => {
                // follower 调整了窗口位置，需要重绘一帧。
                // 直接调用 request_redraw，避免依赖 about_to_wait 的隐式行为
                // （与 create_overlay 中 needs_redraw + request_redraw 的写法保持一致）。
                self.needs_redraw = true;
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
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

        let fullscreen = {
            let cfg = self.config.lock().expect("config lock");
            cfg.settings.fullscreen_overlay
        };

        // 全屏模式允许空标题；窗口模式必须有目标窗口。
        if !fullscreen && title.is_empty() {
            tracing::warn!("cannot start overlay: no target window selected");
            return;
        }

        tracing::info!(fullscreen, title = %title, "creating overlay window");

        // 根据覆盖模式选择初始窗口尺寸。
        let (win_w, win_h) = if fullscreen {
            // 全屏模式：使用主屏幕逻辑尺寸，follower 会立即调整。
            (1920.0_f32, 1080.0_f32)
        } else {
            (800.0_f32, 600.0_f32)
        };

        let attributes = Window::default_attributes()
            .with_title("")
            .with_decorations(false)
            .with_transparent(true)
            .with_active(false)
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_inner_size(winit::dpi::LogicalSize::new(win_w, win_h));

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

            // 创建后立即将 overlay 定位到正确位置，避免首帧渲染在错误位置。
            if let Ok(overlay_hwnd) = platform::windows::hwnd_from_window(&window) {
                if fullscreen {
                    // 全屏模式：定位到 (0,0) 并使用主屏幕物理尺寸。
                    let (screen_w, screen_h) = unsafe {
                        (
                            windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
                            ),
                            windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                                windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
                            ),
                        )
                    };
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                            overlay_hwnd,
                            windows::Win32::Foundation::HWND(std::ptr::null_mut()),
                            0,
                            0,
                            screen_w,
                            screen_h,
                            windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                                | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                        );
                    }
                    tracing::debug!(screen_w, screen_h, "pre-positioned overlay to fullscreen");
                } else if !title.is_empty() {
                    // 窗口模式：将 overlay 对齐到目标窗口客户区。
                    if let Ok(target_hwnd) = platform::windows::find_target_window(&title) {
                        if let Ok(rect) = platform::windows::get_target_rect(target_hwnd) {
                            let width = rect.right - rect.left;
                            let height = rect.bottom - rect.top;
                            unsafe {
                                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                                    overlay_hwnd,
                                    windows::Win32::Foundation::HWND(std::ptr::null_mut()),
                                    rect.left,
                                    rect.top,
                                    width,
                                    height,
                                    windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                                        | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                                );
                            }
                            tracing::debug!(
                                left = rect.left,
                                top = rect.top,
                                width,
                                height,
                                "pre-positioned overlay to target window"
                            );
                        }
                    }
                }
            }
        }

        let renderer = overlay_renderer::OverlayRenderer::new(
            window.clone(),
            self.config.clone(),
            self.material_registry.clone(),
        );

        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.overlay_active = true;
        self.target_title = title.clone();
        self.needs_redraw = true;

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

            // 从配置读取覆盖模式和拖拽选项。
            let (fullscreen, live_drag) = {
                let cfg = self.config.lock().expect("config lock");
                (
                    cfg.settings.fullscreen_overlay,
                    cfg.settings.live_drag_preview,
                )
            };

            // 全屏模式不需要目标窗口；窗口模式必须找到目标窗口。
            let target_hwnd = if fullscreen {
                platform::windows::SendHwnd(windows::Win32::Foundation::HWND(std::ptr::null_mut()))
            } else {
                match platform::windows::find_target_window(&title) {
                    Ok(t) => platform::windows::SendHwnd(t),
                    Err(e) => {
                        tracing::error!("failed to find target window '{}': {}", title, e);
                        return;
                    }
                }
            };

            tracing::info!(title = %title, fullscreen, live_drag, "starting overlay follower");

            let (tx, rx) = tokio::sync::oneshot::channel();
            self.follower_stop = Some(tx);

            // follower 线程在每次移动/调整 overlay 后通过事件循环代理请求重绘。
            // 这保证了窗口仅移动（尺寸不变）时也能刷新画面，
            // 修复了「拖拽实时显示」开启后准心位置不跟随更新的问题。
            let redraw_proxy = self.redraw_proxy.clone();
            let on_moved = move || {
                let _ = redraw_proxy.send_event(UserEvent::Command(OverlayCommand::Invalidate));
            };

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
                rt.block_on(async move {
                    if let Err(e) = platform::windows::follow_target_window(
                        overlay_hwnd,
                        target_hwnd,
                        fullscreen,
                        live_drag,
                        rx,
                        on_moved,
                    )
                    .await
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
                    renderer.resize(_size);
                }
                // 窗口尺寸变化，需要重绘。
                self.needs_redraw = true;
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // DPI 缩放变化时，强制下一帧重绘。
                // follower 线程会检测到新的屏幕尺寸并调整窗口大小，
                // 随后的 Resized 事件会设置 needs_redraw，这里也额外标记确保及时。
                self.needs_redraw = true;
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
                // 静态准心渲染完毕后清除脏标记。
                self.needs_redraw = false;
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

        // 判断当前准心是否为动画样式（RandomOrb 需要持续重绘）。
        let is_animated = {
            let cfg = self.config.lock().expect("config lock");
            cfg.active_profile()
                .map(|p| p.crosshair.style == peregrine_config::CrosshairStyle::RandomOrb)
                .unwrap_or(false)
        };

        if is_animated {
            // RandomOrb 保持 60FPS 持续重绘。
            let now = Instant::now();
            if let Some(last) = self.last_render {
                let elapsed = now.saturating_duration_since(last);
                if elapsed < self.frame_interval {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(last + self.frame_interval));
                    return;
                }
            }
            window.request_redraw();
        } else if self.needs_redraw {
            // 静态准心仅在脏标记为 true 时重绘一帧。
            let now = Instant::now();
            if let Some(last) = self.last_render {
                let elapsed = now.saturating_duration_since(last);
                if elapsed < self.frame_interval {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(last + self.frame_interval));
                    return;
                }
            }
            window.request_redraw();
        } else {
            // 无需重绘，等待事件唤醒。
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
