//! wgpu + egui 渲染器。
//!
//! 负责：
//! - 创建 wgpu 实例、表面、交换链；
//! - 初始化 egui 渲染器；
//! - 在 Overlay 模式下绘制准心，在 Settings 模式下绘制 egui 面板。

use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use peregrine_config::{BorderFrameStyle, CrosshairStyle, OrbPosition, RingStyle};
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;
use winit::window::Window;

use egui::{Color32, Stroke};

/// 渲染器状态。
pub struct Renderer {
    /// wgpu 设备。
    device: wgpu::Device,
    /// wgpu 命令队列。
    queue: wgpu::Queue,
    /// 窗口表面。
    surface: wgpu::Surface<'static>,
    /// 表面配置，窗口大小变化时需要重新配置。
    #[allow(dead_code)]
    surface_config: wgpu::SurfaceConfiguration,
    /// 窗口句柄。
    window: Arc<Window>,
    /// egui 集成状态。
    egui_state: State,
    /// egui 渲染器。
    egui_renderer: egui_wgpu::Renderer,
    /// 当前配置快照，用于渲染准心。
    ///
    /// 使用标准库互斥锁，保证渲染线程内可安全读取而不阻塞 tokio runtime。
    config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
}

impl Renderer {
    /// 创建渲染器。
    pub async fn new(
        window: Arc<Window>,
        config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("peregrine-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("request device");

        let size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("default surface config");
        // 强制使用 Bgra8Unorm（非 sRGB），保证清屏像素值精确匹配
        // SetLayeredWindowAttributes 设置的颜色键（RGB(1,0,0)）。
        // sRGB 格式会做 gamma 校正，导致实际像素值与颜色键不匹配，
        // DWM 无法识别透明区域，窗口就会显示为不透明的普通窗口。
        surface_config.format = wgpu::TextureFormat::Bgra8Unorm;
        surface_config.alpha_mode = pick_alpha_mode(&surface, &adapter);
        surface.configure(&device, &surface_config);
        tracing::info!(
            "surface configured: format={:?}, alpha_mode={:?}, size={:?}",
            surface_config.format,
            surface_config.alpha_mode,
            size
        );

        let egui_context = egui::Context::default();
        // 使用浅色主题作为默认视觉风格。
        egui_context.set_visuals(egui::Visuals::light());
        // 加载系统中文字体，避免 egui 默认字体缺少中文字形而显示方块。
        // Windows 优先尝试微软雅黑（msyh），失败再回退其它候选字体。
        load_system_font(&egui_context);

        let egui_state = State::new(
            egui_context,
            egui::viewport::ViewportId::default(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_config.format, None, 1, true);

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            egui_state,
            egui_renderer,
            config,
        }
    }

    /// 处理窗口事件，主要用于 egui 输入。
    pub fn handle_event(&mut self, event: &WindowEvent) {
        let _ = self.egui_state.on_window_event(&self.window, event);
    }

    /// 在窗口大小变化时重新配置表面。
    ///
    /// Overlay 跟随目标窗口大小时会被调用，避免表面尺寸与窗口不一致。
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// 当 surface 获取纹理失败时，用当前窗口尺寸重新配置 surface。
    ///
    /// 常见于窗口刚创建、尺寸变为 0、或系统模式切换后，surface 进入过时状态。
    fn reconfigure_surface_if_needed(&mut self) {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// 渲染覆盖层（准心）。
    ///
    /// 清屏为颜色键色（RGB(1,0,0) 极深红），并根据当前 Profile 的 crosshair 配置
    /// 绘制辅助贴图。颜色键区域会被 DWM 透明化，只留下准心图形。
    pub fn render_overlay(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                // surface 暂时不可用（窗口大小为 0、刚创建、或模式切换），
                // 重新配置 surface 并跳过本帧，不 panic。
                tracing::debug!("overlay get_current_texture failed: {}", e);
                self.reconfigure_surface_if_needed();
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("overlay-encoder"),
            });

        let config = self.config.lock().expect("config lock");
        let crosshair = config
            .active_profile()
            .map(|p| p.crosshair.clone())
            .unwrap_or_else(peregrine_config::Crosshair::default_crosshair);

        let size = self.window.inner_size();
        let logical_size = [
            size.width as f32 / self.window.scale_factor() as f32,
            size.height as f32 / self.window.scale_factor() as f32,
        ];

        // 使用 egui 生成覆盖层几何体，然后走 wgpu 渲染管线。
        let egui_ctx = self.egui_state.egui_ctx().clone();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(logical_size[0], logical_size[1]),
            )),
            ..Default::default()
        };
        let full_output = egui_ctx.run(raw_input, |ctx| {
            egui::Area::new(egui::Id::new("overlay_area"))
                .default_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();
                    ui.set_min_size(screen_rect.size());
                    draw_overlay_shape(ui, screen_rect, &crosshair);
                });
        });
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let tris = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("overlay-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // Windows 颜色键透明：清屏色必须与 SetLayeredWindowAttributes
                            // 设定的色键一致（RGB(1,0,0) ≈ 极深红）。
                            // 使用非纯黑是为了避免用户把准心颜色设为黑色时也被透明。
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: (1.0 / 255.0),
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();
            self.egui_renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    /// 渲染 egui 设置面板。
    pub fn render_settings(
        &mut self,
        ui_state: &mut super::settings_ui::SettingsUi,
        config: &peregrine_config::ConfigSnapshot,
    ) -> super::settings_ui::SettingsResponse {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self
            .egui_state
            .egui_ctx()
            .run(raw_input, |ctx| ui_state.ui(ctx, config));
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                tracing::debug!("settings get_current_texture failed: {}", e);
                self.reconfigure_surface_if_needed();
                return ui_state.take_response();
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("settings-encoder"),
            });

        let size = self.window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let tris = self
            .egui_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("settings-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.95,
                                g: 0.95,
                                b: 0.95,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();
            self.egui_renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        ui_state.take_response()
    }
}

/// 在覆盖层区域绘制当前选中的辅助贴图。
///
/// 与设置面板预览使用同一套坐标逻辑，仅在坐标系上以物理像素为准。
fn draw_overlay_shape(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    crosshair: &peregrine_config::Crosshair,
) {
    use egui::{CornerRadius, Stroke};

    let _ = egui::CornerRadius::ZERO;
    let center = rect.center();
    let color = settings_ui_color(&crosshair.color, crosshair.opacity);

    match crosshair.style {
        CrosshairStyle::ToiletPaper => {
            let width = crosshair.size;
            let height = crosshair.secondary_size;
            let margin = crosshair.margin;
            let center = match crosshair.anchor {
                peregrine_config::Anchor::Top => {
                    egui::pos2(rect.center().x, rect.min.y + height / 2.0 + margin)
                }
                peregrine_config::Anchor::Bottom => {
                    egui::pos2(rect.center().x, rect.max.y - height / 2.0 - margin)
                }
                peregrine_config::Anchor::Left => {
                    egui::pos2(rect.min.x + width / 2.0 + margin, rect.center().y)
                }
                peregrine_config::Anchor::Right => {
                    egui::pos2(rect.max.x - width / 2.0 - margin, rect.center().y)
                }
                peregrine_config::Anchor::Center => rect.center(),
            };
            let shape_rect = egui::Rect::from_center_size(center, egui::vec2(width, height));
            ui.painter().rect_filled(
                shape_rect,
                CornerRadius::same(crosshair.corner_radius as u8),
                color,
            );
        }
        CrosshairStyle::Cross => {
            let arm = crosshair.size;
            let half_gap = crosshair.gap / 2.0;
            let thickness = crosshair.thickness;
            let horizontal_left = egui::Rect::from_center_size(
                egui::pos2(center.x - (arm + half_gap) / 2.0 - half_gap / 2.0, center.y),
                egui::vec2(arm - half_gap, thickness),
            );
            let horizontal_right = egui::Rect::from_center_size(
                egui::pos2(center.x + (arm + half_gap) / 2.0 + half_gap / 2.0, center.y),
                egui::vec2(arm - half_gap, thickness),
            );
            let vertical_top = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y - (arm + half_gap) / 2.0 - half_gap / 2.0),
                egui::vec2(thickness, arm - half_gap),
            );
            let vertical_bottom = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y + (arm + half_gap) / 2.0 + half_gap / 2.0),
                egui::vec2(thickness, arm - half_gap),
            );
            for rect in [
                horizontal_left,
                horizontal_right,
                vertical_top,
                vertical_bottom,
            ] {
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::ZERO, color);
            }
        }
        CrosshairStyle::LargeCross => {
            let thickness = crosshair.thickness;
            let horizontal = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, center.y - thickness / 2.0),
                egui::pos2(rect.max.x, center.y + thickness / 2.0),
            );
            let vertical = egui::Rect::from_min_max(
                egui::pos2(center.x - thickness / 2.0, rect.min.y),
                egui::pos2(center.x + thickness / 2.0, rect.max.y),
            );
            ui.painter()
                .rect_filled(horizontal, egui::CornerRadius::ZERO, color);
            ui.painter()
                .rect_filled(vertical, egui::CornerRadius::ZERO, color);
        }
        CrosshairStyle::CornerDots4 | CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8 => {
            let configured_offset = if crosshair.offset > 0.0 {
                crosshair.offset
            } else {
                crosshair.size
            };
            let offset = configured_offset
                .min(rect.width() / 4.0)
                .min(rect.height() / 4.0);
            let radius = if crosshair.radius > 0.0 {
                crosshair.radius
            } else {
                crosshair.thickness * 3.0
            };
            let corners = [
                egui::pos2(rect.min.x + offset, rect.min.y + offset),
                egui::pos2(rect.max.x - offset, rect.min.y + offset),
                egui::pos2(rect.min.x + offset, rect.max.y - offset),
                egui::pos2(rect.max.x - offset, rect.max.y - offset),
            ];
            for pos in corners {
                ui.painter().circle_filled(pos, radius, color);
            }
            if matches!(
                crosshair.style,
                CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8
            ) {
                ui.painter().circle_filled(
                    egui::pos2(center.x, rect.min.y + offset),
                    radius,
                    color,
                );
                ui.painter().circle_filled(
                    egui::pos2(center.x, rect.max.y - offset),
                    radius,
                    color,
                );
            }
            if matches!(crosshair.style, CrosshairStyle::CornerDots8) {
                ui.painter().circle_filled(
                    egui::pos2(rect.min.x + offset, center.y),
                    radius,
                    color,
                );
                ui.painter().circle_filled(
                    egui::pos2(rect.max.x - offset, center.y),
                    radius,
                    color,
                );
            }
        }
        CrosshairStyle::Ring => {
            let radius = rect.height() * crosshair.ring_radius_pct;
            let thickness = crosshair.thickness;
            match crosshair.ring_style {
                RingStyle::Solid => {
                    ui.painter()
                        .circle_stroke(center, radius, Stroke::new(thickness, color));
                }
                RingStyle::Dashed => {
                    draw_overlay_dashed_circle(ui, center, radius, thickness, color, 4.0, 4.0);
                }
                RingStyle::Double => {
                    ui.painter()
                        .circle_stroke(center, radius - 2.0, Stroke::new(1.0, color));
                    draw_overlay_dashed_circle(ui, center, radius + 2.0, 1.0, color, 4.0, 4.0);
                }
            }
        }
        CrosshairStyle::CustomOrb => {
            let radius = crosshair.radius.max(1.0);
            let offset = crosshair.offset;
            let draw_edge_orbs = |ui: &mut egui::Ui, count: u32, positions: Vec<egui::Pos2>| {
                if count == 0 || positions.is_empty() {
                    return;
                }
                if count == 1 {
                    ui.painter()
                        .circle_filled(positions[positions.len() / 2], radius, color);
                    return;
                }
                // 在两端各留 offset 边距后均匀分布。
                let step = (positions.len() - 1) as f32 / (count + 1) as f32;
                for i in 1..=count {
                    let idx_f = i as f32 * step;
                    let idx0 = idx_f.floor() as usize;
                    let idx1 = (idx0 + 1).min(positions.len() - 1);
                    let t = idx_f - idx0 as f32;
                    let p0 = positions[idx0];
                    let p1 = positions[idx1];
                    let pos = p0 + (p1 - p0) * t;
                    ui.painter().circle_filled(pos, radius, color);
                }
            };

            if crosshair.orb_positions.contains(OrbPosition::TOP) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let x = egui::lerp(rect.min.x..=rect.max.x, i as f32 / 16.0);
                        egui::pos2(x, rect.min.y + offset)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_top_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::BOTTOM) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let x = egui::lerp(rect.min.x..=rect.max.x, i as f32 / 16.0);
                        egui::pos2(x, rect.max.y - offset)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_bottom_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::LEFT) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let y = egui::lerp(rect.min.y..=rect.max.y, i as f32 / 16.0);
                        egui::pos2(rect.min.x + offset, y)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_left_count, positions);
            }
            if crosshair.orb_positions.contains(OrbPosition::RIGHT) {
                let positions: Vec<_> = (0..=16)
                    .map(|i| {
                        let y = egui::lerp(rect.min.y..=rect.max.y, i as f32 / 16.0);
                        egui::pos2(rect.max.x - offset, y)
                    })
                    .collect();
                draw_edge_orbs(ui, crosshair.custom_orb_right_count, positions);
            }
        }
        CrosshairStyle::RandomOrb => {
            // 使用配置值作为稳定种子，避免每帧闪烁。
            // seed 包含所有影响生成结果的参数（含 count、位置），
            // 任一参数变化都会产生不同的随机序列。
            let seed = ((crosshair.random_orb_offset * 1000.0) as u64)
                .wrapping_add((crosshair.random_orb_jitter * 100.0) as u64)
                .wrapping_add((crosshair.random_radius_min * 10.0) as u64)
                .wrapping_add((crosshair.random_radius_max * 10.0) as u64)
                .wrapping_add(crosshair.random_orb_count as u64);
            let mut rng = SimpleRng::new(seed);
            let count = crosshair.random_orb_count as usize;
            let offset = crosshair.random_orb_offset;
            let jitter = crosshair.random_orb_jitter;
            let min_r = crosshair.random_radius_min;
            let max_r = crosshair.random_radius_max;

            let radius_for =
                |rng: &mut SimpleRng| -> f32 { min_r + rng.next_f32() * (max_r - min_r) };
            let jitter_for = |rng: &mut SimpleRng| -> f32 { (rng.next_f32() - 0.5) * 2.0 * jitter };

            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let x = egui::lerp(rect.min.x..=rect.max.x, rng.next_f32());
                let pos = egui::pos2(
                    x + jitter_for(&mut rng),
                    rect.min.y + offset + jitter_for(&mut rng),
                );
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let x = egui::lerp(rect.min.x..=rect.max.x, rng.next_f32());
                let pos = egui::pos2(
                    x + jitter_for(&mut rng),
                    rect.max.y - offset + jitter_for(&mut rng),
                );
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let y = egui::lerp(rect.min.y..=rect.max.y, rng.next_f32());
                let pos = egui::pos2(
                    rect.min.x + offset + jitter_for(&mut rng),
                    y + jitter_for(&mut rng),
                );
                ui.painter().circle_filled(pos, radius, color);
            }
            for _ in 0..count {
                let radius = radius_for(&mut rng);
                let y = egui::lerp(rect.min.y..=rect.max.y, rng.next_f32());
                let pos = egui::pos2(
                    rect.max.x - offset + jitter_for(&mut rng),
                    y + jitter_for(&mut rng),
                );
                ui.painter().circle_filled(pos, radius, color);
            }
        }
        CrosshairStyle::BorderFrame => {
            let thickness = crosshair.thickness;
            let offset = crosshair.offset;

            let top_y = rect.min.y + offset;
            let bottom_y = rect.max.y - offset;
            let left_x = rect.min.x + offset;
            let right_x = rect.max.x - offset;

            match crosshair.border_frame_style {
                BorderFrameStyle::Solid => {
                    draw_overlay_solid_border_frame(
                        ui, rect, top_y, bottom_y, left_x, right_x, thickness, color,
                    );
                }
                BorderFrameStyle::Gap => {
                    draw_overlay_gap_border_frame(
                        ui, rect, top_y, bottom_y, left_x, right_x, thickness, color,
                    );
                }
            }
        }
    }
}

/// 把 [f32; 4] RGBA 与不透明度转换为 egui Color32。
fn settings_ui_color(color: &[f32; 4], opacity: f32) -> Color32 {
    let mut c = Color32::from_rgba_premultiplied(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    );
    c[3] = (c[3] as f32 * opacity.clamp(0.0, 1.0)) as u8;
    c
}

/// 覆盖层虚线圆。
fn draw_overlay_dashed_circle(
    ui: &mut egui::Ui,
    center: egui::Pos2,
    radius: f32,
    thickness: f32,
    color: Color32,
    dash_len: f32,
    gap_len: f32,
) {
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let unit = dash_len + gap_len;
    let segments = (circumference / unit).ceil() as usize;
    let step_angle = 2.0 * std::f32::consts::PI / segments as f32;

    for i in 0..segments {
        let start_angle = i as f32 * step_angle;
        let end_angle = start_angle + step_angle * (dash_len / unit);
        let start = egui::pos2(
            center.x + radius * start_angle.cos(),
            center.y + radius * start_angle.sin(),
        );
        let end = egui::pos2(
            center.x + radius * end_angle.cos(),
            center.y + radius * end_angle.sin(),
        );
        ui.painter()
            .line_segment([start, end], Stroke::new(thickness, color));
    }
}

/// 简单的线性同余 RNG，用于覆盖层生成稳定随机球位置。
#[derive(Debug, Clone, Copy)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }
}

/// 覆盖层完整边框（4 条矩形条）。
fn draw_overlay_solid_border_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: Color32,
) {
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, top_y - thickness / 2.0),
            egui::pos2(rect.max.x, top_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, bottom_y - thickness / 2.0),
            egui::pos2(rect.max.x, bottom_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x - thickness / 2.0, rect.min.y),
            egui::pos2(left_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x - thickness / 2.0, rect.min.y),
            egui::pos2(right_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
}

/// 覆盖层带缺口边框。
fn draw_overlay_gap_border_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: Color32,
) {
    let gap_pct = 0.2;
    let half_gap_w = rect.width() * gap_pct / 2.0;
    let half_gap_h = rect.height() * gap_pct / 2.0;

    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, top_y - thickness / 2.0),
            egui::pos2(rect.center().x - half_gap_w, top_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.center().x + half_gap_w, top_y - thickness / 2.0),
            egui::pos2(rect.max.x, top_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.min.x, bottom_y - thickness / 2.0),
            egui::pos2(rect.center().x - half_gap_w, bottom_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.center().x + half_gap_w, bottom_y - thickness / 2.0),
            egui::pos2(rect.max.x, bottom_y + thickness / 2.0),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x - thickness / 2.0, rect.min.y),
            egui::pos2(left_x + thickness / 2.0, rect.center().y - half_gap_h),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(left_x - thickness / 2.0, rect.center().y + half_gap_h),
            egui::pos2(left_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );

    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x - thickness / 2.0, rect.min.y),
            egui::pos2(right_x + thickness / 2.0, rect.center().y - half_gap_h),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(right_x - thickness / 2.0, rect.center().y + half_gap_h),
            egui::pos2(right_x + thickness / 2.0, rect.max.y),
        ),
        egui::CornerRadius::ZERO,
        color,
    );
}

/// 选择 surface 的 alpha 合成模式。
///
/// 当前 Windows 已改回颜色键透明，surface 保持 Opaque 即可。
/// 保留该函数以备后续需要重新启用逐像素 alpha。
fn pick_alpha_mode(
    _surface: &wgpu::Surface<'static>,
    _adapter: &wgpu::Adapter,
) -> wgpu::CompositeAlphaMode {
    wgpu::CompositeAlphaMode::Opaque
}

/// 尝试从 Windows 系统字体路径加载中文字体并注册到 egui。
///
/// 若所有候选字体均不可用，仅打印警告，仍使用 egui 默认字体（英文/ASCII 可正常显示）。
fn load_system_font(ctx: &egui::Context) {
    // 按平台给出中文字体候选列表。
    #[cfg(target_os = "windows")]
    const CANDIDATES: &[&str] = &[
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyh.ttf",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];

    #[cfg(target_os = "macos")]
    const CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ];

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/wqy-zenhei/wqy-zenhei.ttc",
    ];

    for path in CANDIDATES {
        match std::fs::read(path) {
            Ok(bytes) => {
                let mut fonts = egui::FontDefinitions::default();
                // 将中文字体作为 Proportional 和 Monospace 的 fallback，
                // 保留默认字体用于数字与 ASCII 字符。
                fonts.font_data.insert(
                    "system_cjk".to_string(),
                    egui::FontData::from_owned(bytes).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("system_cjk".to_string());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("system_cjk".to_string());
                ctx.set_fonts(fonts);
                tracing::info!("loaded system font: {}", path);
                return;
            }
            Err(e) => {
                tracing::debug!("failed to load font {}: {}", path, e);
            }
        }
    }

    tracing::warn!("no system CJK font found; Chinese text may render as boxes");
}
