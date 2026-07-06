//! softbuffer 版遮盖层渲染器。
//!
//! 用 CPU 像素缓冲区（softbuffer）替代 wgpu swapchain，
//! 参考 simple-crosshair-overlay 的方案：
//! - `with_transparent(true)` 让 winit 启用 DWM 透明
//! - 像素格式 `0xAARRGGBB`，透明区域填 `0x00000000`
//! - Windows 上需要预乘 alpha
//!
//! 优点：不涉及 swapchain/DirectComposition，透明天然可靠。
//! 缺点：需要自己实现像素光栅化（矩形、圆、线段）。

// 像素光栅化原语参数较多（坐标、尺寸、颜色等），允许超过 clippy 默认限制。
#![allow(clippy::too_many_arguments)]

use peregrine_config::{
    Anchor, BorderFrameStyle, Crosshair, CrosshairStyle, OrbPosition, RingStyle,
};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use winit::window::Window;

/// softbuffer 版遮盖层渲染器。
pub struct OverlayRenderer {
    /// winit 窗口（`Rc` 包裹以满足 softbuffer 生命周期要求）。
    window: Arc<Window>,
    /// softbuffer 上下文。
    #[allow(dead_code)]
    context: softbuffer::Context<Arc<Window>>,
    /// softbuffer 表面。
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// 当前配置快照。
    config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
}

impl OverlayRenderer {
    /// 创建渲染器。
    pub fn new(window: Arc<Window>, config: Arc<Mutex<peregrine_config::ConfigSnapshot>>) -> Self {
        // softbuffer 要求 Context 和 Surface 共享同一个 window 引用。
        let context = softbuffer::Context::new(window.clone()).expect("create softbuffer context");
        let surface =
            softbuffer::Surface::new(&context, window.clone()).expect("create softbuffer surface");

        Self {
            window,
            context,
            surface,
            config,
        }
    }

    /// 窗口大小变化时调用（softbuffer 在 render 时自动 resize，此处空实现）。
    pub fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {
        // softbuffer 的 resize 在 render_overlay 中按当前窗口尺寸自动处理。
    }

    /// 渲染一帧遮盖层。
    pub fn render_overlay(&mut self) {
        let size = self.window.inner_size();
        let width = size.width;
        let height = size.height;
        if width == 0 || height == 0 {
            return;
        }

        // 调整 softbuffer 缓冲区尺寸。
        if let Err(e) = self.surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        ) {
            tracing::error!("softbuffer resize failed: {}", e);
            return;
        }

        let mut buffer = match self.surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("softbuffer buffer_mut failed: {}", e);
                return;
            }
        };

        // 清屏为完全透明。
        buffer.fill(0x00000000);

        // 读取当前准心配置。
        let config = self.config.lock().expect("config lock");
        let crosshair = config
            .active_profile()
            .map(|p| p.crosshair.clone())
            .unwrap_or_else(Crosshair::default_crosshair);
        drop(config);

        // 在像素缓冲区上绘制准心。
        let logical_w = width as f32 / self.window.scale_factor() as f32;
        let logical_h = height as f32 / self.window.scale_factor() as f32;
        let rect = RectF {
            min_x: 0.0,
            min_y: 0.0,
            max_x: logical_w,
            max_y: logical_h,
        };
        let scale = self.window.scale_factor() as f32;
        draw_crosshair(&mut buffer, width, height, scale, rect, &crosshair);

        if let Err(e) = buffer.present() {
            tracing::error!("softbuffer present failed: {}", e);
        }
    }
}

/// 逻辑矩形区域（浮点坐标）。
struct RectF {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl RectF {
    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }
    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
    fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) / 2.0
    }
    fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) / 2.0
    }
}

/// 把 [f32;4] RGBA + opacity 转换为预乘 alpha 的 0xAARRGGBB u32。
fn make_color(color: &[f32; 4], opacity: f32) -> u32 {
    let a = (color[3] * opacity).clamp(0.0, 1.0);
    let r = color[0] * a;
    let g = color[1] * a;
    let b = color[2] * a;
    let ai = (a * 255.0) as u32;
    let ri = (r * 255.0) as u32;
    let gi = (g * 255.0) as u32;
    let bi = (b * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}

/// 在像素缓冲区上绘制准心。
///
/// `buffer` 是 softbuffer 的 `&mut [u32]`，像素格式 `0xAARRGGBB`。
/// `scale` 是 DPI 缩放因子（逻辑坐标 → 物理像素）。
fn draw_crosshair(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: RectF,
    crosshair: &Crosshair,
) {
    let color = make_color(&crosshair.color, crosshair.opacity);
    let cx = rect.center_x();
    let cy = rect.center_y();

    match crosshair.style {
        CrosshairStyle::Cross => {
            let arm = crosshair.size;
            let half_gap = crosshair.gap / 2.0;
            let thickness = crosshair.thickness;
            // 四条臂。
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                cx - arm - half_gap,
                cy - thickness / 2.0,
                arm - half_gap,
                thickness,
                color,
            );
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                cx + half_gap,
                cy - thickness / 2.0,
                arm - half_gap,
                thickness,
                color,
            );
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                cx - thickness / 2.0,
                cy - arm - half_gap,
                thickness,
                arm - half_gap,
                color,
            );
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                cx - thickness / 2.0,
                cy + half_gap,
                thickness,
                arm - half_gap,
                color,
            );
        }
        CrosshairStyle::LargeCross => {
            let thickness = crosshair.thickness;
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                rect.min_x,
                cy - thickness / 2.0,
                rect.width(),
                thickness,
                color,
            );
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                cx - thickness / 2.0,
                rect.min_y,
                thickness,
                rect.height(),
                color,
            );
        }
        CrosshairStyle::ToiletPaper => {
            let w = crosshair.size;
            let h = crosshair.secondary_size;
            let margin = crosshair.margin;
            let (px, py) = match crosshair.anchor {
                Anchor::Top => (cx, rect.min_y + h / 2.0 + margin),
                Anchor::Bottom => (cx, rect.max_y - h / 2.0 - margin),
                Anchor::Left => (rect.min_x + w / 2.0 + margin, cy),
                Anchor::Right => (rect.max_x - w / 2.0 - margin, cy),
                Anchor::Center => (cx, cy),
            };
            draw_rect(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                px - w / 2.0,
                py - h / 2.0,
                w,
                h,
                color,
            );
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
                (rect.min_x + offset, rect.min_y + offset),
                (rect.max_x - offset, rect.min_y + offset),
                (rect.min_x + offset, rect.max_y - offset),
                (rect.max_x - offset, rect.max_y - offset),
            ];
            for (x, y) in corners {
                draw_circle(buffer, pixel_w, pixel_h, scale, x, y, radius, color);
            }
            if matches!(
                crosshair.style,
                CrosshairStyle::CornerDots6 | CrosshairStyle::CornerDots8
            ) {
                draw_circle(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    cx,
                    rect.min_y + offset,
                    radius,
                    color,
                );
                draw_circle(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    cx,
                    rect.max_y - offset,
                    radius,
                    color,
                );
            }
            if matches!(crosshair.style, CrosshairStyle::CornerDots8) {
                draw_circle(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.min_x + offset,
                    cy,
                    radius,
                    color,
                );
                draw_circle(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.max_x - offset,
                    cy,
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
                    draw_circle_stroke(
                        buffer, pixel_w, pixel_h, scale, cx, cy, radius, thickness, color,
                    );
                }
                RingStyle::Dashed => {
                    draw_dashed_circle(
                        buffer, pixel_w, pixel_h, scale, cx, cy, radius, thickness, 4.0, 4.0, color,
                    );
                }
                RingStyle::Double => {
                    draw_circle_stroke(
                        buffer,
                        pixel_w,
                        pixel_h,
                        scale,
                        cx,
                        cy,
                        radius - 2.0,
                        1.0,
                        color,
                    );
                    draw_dashed_circle(
                        buffer,
                        pixel_w,
                        pixel_h,
                        scale,
                        cx,
                        cy,
                        radius + 2.0,
                        1.0,
                        4.0,
                        4.0,
                        color,
                    );
                }
            }
        }
        CrosshairStyle::CustomOrb => {
            let radius = crosshair.radius.max(1.0);
            let offset = crosshair.offset;
            if crosshair.orb_positions.contains(OrbPosition::TOP) {
                draw_edge_orbs(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.min_x,
                    rect.min_y + offset,
                    rect.max_x,
                    rect.min_y + offset,
                    crosshair.custom_orb_top_count,
                    radius,
                    color,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::BOTTOM) {
                draw_edge_orbs(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.min_x,
                    rect.max_y - offset,
                    rect.max_x,
                    rect.max_y - offset,
                    crosshair.custom_orb_bottom_count,
                    radius,
                    color,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::LEFT) {
                draw_edge_orbs(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.min_x + offset,
                    rect.min_y,
                    rect.min_x + offset,
                    rect.max_y,
                    crosshair.custom_orb_left_count,
                    radius,
                    color,
                );
            }
            if crosshair.orb_positions.contains(OrbPosition::RIGHT) {
                draw_edge_orbs(
                    buffer,
                    pixel_w,
                    pixel_h,
                    scale,
                    rect.max_x - offset,
                    rect.min_y,
                    rect.max_x - offset,
                    rect.max_y,
                    crosshair.custom_orb_right_count,
                    radius,
                    color,
                );
            }
        }
        CrosshairStyle::RandomOrb => {
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

            for edge in 0..4 {
                for _ in 0..count {
                    let radius = min_r + rng.next_f32() * (max_r - min_r);
                    let j = (rng.next_f32() - 0.5) * 2.0 * jitter;
                    let (x, y) = match edge {
                        0 => {
                            let x = rect.min_x + rng.next_f32() * rect.width();
                            (x + j, rect.min_y + offset + j)
                        }
                        1 => {
                            let x = rect.min_x + rng.next_f32() * rect.width();
                            (x + j, rect.max_y - offset + j)
                        }
                        2 => {
                            let y = rect.min_y + rng.next_f32() * rect.height();
                            (rect.min_x + offset + j, y + j)
                        }
                        _ => {
                            let y = rect.min_y + rng.next_f32() * rect.height();
                            (rect.max_x - offset + j, y + j)
                        }
                    };
                    draw_circle(buffer, pixel_w, pixel_h, scale, x, y, radius, color);
                }
            }
        }
        CrosshairStyle::BorderFrame => {
            let thickness = crosshair.thickness;
            let offset = crosshair.offset;
            let top_y = rect.min_y + offset;
            let bottom_y = rect.max_y - offset;
            let left_x = rect.min_x + offset;
            let right_x = rect.max_x - offset;
            match crosshair.border_frame_style {
                BorderFrameStyle::Solid => {
                    draw_solid_frame(
                        buffer, pixel_w, pixel_h, scale, &rect, top_y, bottom_y, left_x, right_x,
                        thickness, color,
                    );
                }
                BorderFrameStyle::Gap => {
                    draw_gap_frame(
                        buffer, pixel_w, pixel_h, scale, &rect, top_y, bottom_y, left_x, right_x,
                        thickness, color,
                    );
                }
            }
        }
    }
}

// ===== 像素光栅化原语 =====

/// 绘制填充矩形（逻辑坐标）。
fn draw_rect(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let x0 = (x * scale).round() as i32;
    let y0 = (y * scale).round() as i32;
    let x1 = ((x + w) * scale).round() as i32;
    let y1 = ((y + h) * scale).round() as i32;
    let x0 = x0.max(0);
    let y0 = y0.max(0);
    let x1 = x1.min(pixel_w as i32);
    let y1 = y1.min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let idx = (py as u32) * pixel_w + (px as u32);
            if (idx as usize) < buffer.len() {
                buffer[idx as usize] = color;
            }
        }
    }
}

/// 绘制填充圆（逻辑坐标）。
fn draw_circle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: u32,
) {
    let pcx = (cx * scale).round() as i32;
    let pcy = (cy * scale).round() as i32;
    let pr = (radius * scale).round() as i32;
    let pr_sq = (pr * pr) as f32;
    let x0 = (pcx - pr).max(0);
    let y0 = (pcy - pr).max(0);
    let x1 = (pcx + pr).min(pixel_w as i32);
    let y1 = (pcy + pr).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px - pcx;
            let dy = py - pcy;
            if (dx * dx + dy * dy) as f32 <= pr_sq {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

/// 绘制圆环描边（逻辑坐标）。
fn draw_circle_stroke(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: u32,
) {
    let pcx = (cx * scale).round() as i32;
    let pcy = (cy * scale).round() as i32;
    let outer_r = ((radius + thickness / 2.0) * scale).round() as i32;
    let inner_r = ((radius - thickness / 2.0) * scale).round().max(0.0) as i32;
    let outer_sq = (outer_r * outer_r) as f32;
    let inner_sq = (inner_r * inner_r) as f32;
    let x0 = (pcx - outer_r).max(0);
    let y0 = (pcy - outer_r).max(0);
    let x1 = (pcx + outer_r).min(pixel_w as i32);
    let y1 = (pcy + outer_r).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px - pcx;
            let dy = py - pcy;
            let d_sq = (dx * dx + dy * dy) as f32;
            if d_sq <= outer_sq && d_sq >= inner_sq {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

/// 绘制虚线圆（逻辑坐标）。
fn draw_dashed_circle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    dash_len: f32,
    gap_len: f32,
    color: u32,
) {
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let unit = dash_len + gap_len;
    let segments = (circumference / unit).ceil() as usize;
    let step_angle = 2.0 * std::f32::consts::PI / segments as f32;
    let dash_angle = step_angle * (dash_len / unit);
    for i in 0..segments {
        let a0 = i as f32 * step_angle;
        let a1 = a0 + dash_angle;
        // 在 [a0, a1] 范围内逐角度采样绘制。
        let steps = ((a1 - a0) * radius).ceil() as usize + 1;
        for s in 0..steps {
            let t = if steps > 0 {
                s as f32 / steps as f32
            } else {
                0.0
            };
            let a = a0 + (a1 - a0) * t;
            let x = cx + radius * a.cos();
            let y = cy + radius * a.sin();
            draw_circle(
                buffer,
                pixel_w,
                pixel_h,
                scale,
                x,
                y,
                thickness / 2.0,
                color,
            );
        }
    }
}

/// 在一条边上均匀分布绘制圆点。
fn draw_edge_orbs(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    count: u32,
    radius: f32,
    color: u32,
) {
    if count == 0 {
        return;
    }
    if count == 1 {
        let mx = (x0 + x1) / 2.0;
        let my = (y0 + y1) / 2.0;
        draw_circle(buffer, pixel_w, pixel_h, scale, mx, my, radius, color);
        return;
    }
    let step = 1.0 / (count + 1) as f32;
    for i in 1..=count {
        let t = i as f32 * step;
        let x = x0 + (x1 - x0) * t;
        let y = y0 + (y1 - y0) * t;
        draw_circle(buffer, pixel_w, pixel_h, scale, x, y, radius, color);
    }
}

/// 绘制完整边框（4 条矩形条）。
fn draw_solid_frame(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &RectF,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: u32,
) {
    let half_t = thickness / 2.0;
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        rect.min_x,
        top_y - half_t,
        rect.width(),
        thickness,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        rect.min_x,
        bottom_y - half_t,
        rect.width(),
        thickness,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        left_x - half_t,
        rect.min_y,
        thickness,
        rect.height(),
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        right_x - half_t,
        rect.min_y,
        thickness,
        rect.height(),
        color,
    );
}

/// 绘制带缺口的边框。
fn draw_gap_frame(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &RectF,
    top_y: f32,
    bottom_y: f32,
    left_x: f32,
    right_x: f32,
    thickness: f32,
    color: u32,
) {
    let half_t = thickness / 2.0;
    let half_gap_w = rect.width() * 0.2 / 2.0;
    let half_gap_h = rect.height() * 0.2 / 2.0;
    let cx = rect.center_x();
    let cy = rect.center_y();

    // 上边（两段）。
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        rect.min_x,
        top_y - half_t,
        cx - half_gap_w - rect.min_x,
        thickness,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        cx + half_gap_w,
        top_y - half_t,
        rect.max_x - (cx + half_gap_w),
        thickness,
        color,
    );
    // 下边。
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        rect.min_x,
        bottom_y - half_t,
        cx - half_gap_w - rect.min_x,
        thickness,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        cx + half_gap_w,
        bottom_y - half_t,
        rect.max_x - (cx + half_gap_w),
        thickness,
        color,
    );
    // 左边。
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        left_x - half_t,
        rect.min_y,
        thickness,
        cy - half_gap_h - rect.min_y,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        left_x - half_t,
        cy + half_gap_h,
        thickness,
        rect.max_y - (cy + half_gap_h),
        color,
    );
    // 右边。
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        right_x - half_t,
        rect.min_y,
        thickness,
        cy - half_gap_h - rect.min_y,
        color,
    );
    draw_rect(
        buffer,
        pixel_w,
        pixel_h,
        scale,
        right_x - half_t,
        cy + half_gap_h,
        thickness,
        rect.max_y - (cy + half_gap_h),
        color,
    );
}

/// 简单的线性同余 RNG。
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
