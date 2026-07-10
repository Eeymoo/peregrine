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

use peregrine_config::{Crosshair, CrosshairStyle};
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
    /// PNG 图片缓存：路径 → 解码后的 RGBA 像素。
    image_cache: Option<CachedImage>,
}

/// 已解码的 PNG 图片，包含原始像素数据和尺寸。
struct CachedImage {
    /// 用于缓存匹配的路径。
    path: String,
    /// RGBA 像素数据（行优先，从上到下）。
    pixels: Vec<(u8, u8, u8, u8)>,
    /// 原始宽度（像素）。
    width: usize,
    /// 原始高度（像素）。
    height: usize,
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
            image_cache: None,
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

        tracing::debug!(
            width,
            height,
            scale = self.window.scale_factor(),
            "overlay render_overlay: window inner_size"
        );

        // 调整 softbuffer 缓冲区尺寸。
        if let Err(e) = self.surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        ) {
            tracing::error!("softbuffer resize failed: {}", e);
            return;
        }

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
        let rect = crate::shapes::RectF {
            min_x: 0.0,
            min_y: 0.0,
            max_x: logical_w,
            max_y: logical_h,
        };
        let scale = self.window.scale_factor() as f32;

        // CustomImage 需要访问 image_cache，单独处理。
        // 先加载图片（在获取 buffer 之前，避免与 surface 借用冲突）。
        let is_custom_image = crosshair.style == CrosshairStyle::CustomImage;
        if is_custom_image {
            self.ensure_image_loaded(&crosshair.image_path);
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

        // 诊断：检查 buffer 长度与预期是否一致。
        tracing::debug!(
            buf_len = buffer.len(),
            expected = (width as usize) * (height as usize),
            "overlay buffer size check"
        );

        if is_custom_image {
            // 绘制图片（只读 image_cache，不与 buffer 冲突）。
            if let Some(img) = &self.image_cache {
                let opacity = crosshair.opacity;
                let img_scale = crosshair.image_scale;
                let offset_x = crosshair.image_offset_x;
                let offset_y = crosshair.image_offset_y;
                draw_image(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &rect,
                    img,
                    img_scale,
                    offset_x,
                    offset_y,
                    opacity,
                );
            }
        } else {
            // 使用共享几何模块生成图元，确保预览与覆盖层完全一致。
            let color = make_color(&crosshair.color, crosshair.opacity);
            let shapes = crate::shapes::build_shapes(&rect, &crosshair);
            for shape in shapes {
                rasterize_shape(&mut buffer, width, height, scale, &shape, color);
            }
        }

        // 诊断：统计非透明像素数量。
        let non_transparent = buffer.iter().filter(|&&p| p != 0x00000000).count();
        tracing::debug!(
            non_transparent,
            total = buffer.len(),
            "overlay pixel stats after drawing"
        );

        if let Err(e) = buffer.present() {
            tracing::error!("softbuffer present failed: {}", e);
        }
    }

    /// 确保 image_cache 中缓存的是当前路径的图片。
    ///
    /// 如果路径为空或加载失败，清空缓存并记录警告。
    fn ensure_image_loaded(&mut self, path: &str) {
        // 路径未变且有缓存 → 无需重新加载。
        if let Some(cache) = &self.image_cache {
            if cache.path == path {
                return;
            }
        }

        if path.is_empty() {
            self.image_cache = None;
            return;
        }

        match load_png(path) {
            Ok((pixels, w, h)) => {
                tracing::info!(path, width = w, height = h, "loaded crosshair PNG");
                self.image_cache = Some(CachedImage {
                    path: path.to_string(),
                    pixels,
                    width: w,
                    height: h,
                });
            }
            Err(e) => {
                tracing::warn!(path, error = %e, "failed to load crosshair PNG");
                self.image_cache = None;
            }
        }
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

/// 将一条 [`Shape`]（共享几何图元）光栅化到 softbuffer 像素缓冲区。
///
/// 这是 overlay 侧的渲染器：与前端 `Preview` 组件的预览渲染一一对应。
/// 两者调用相同的 `build_shapes`，确保预览与实际效果完全一致。
fn rasterize_shape(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    shape: &crate::shapes::Shape,
    color: u32,
) {
    use crate::shapes::Shape;
    match shape {
        Shape::Rect { x, y, w, h } => {
            draw_rect(buffer, pixel_w, pixel_h, scale, *x, *y, *w, *h, color);
        }
        Shape::Circle { cx, cy, radius } => {
            draw_circle(buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, color);
        }
        Shape::CircleStroke {
            cx,
            cy,
            radius,
            thickness,
        } => {
            draw_circle_stroke(
                buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, color,
            );
        }
        Shape::DashedCircle {
            cx,
            cy,
            radius,
            thickness,
            dash_len,
            gap_len,
        } => {
            draw_dashed_circle(
                buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, *dash_len,
                *gap_len, color,
            );
        }
        Shape::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            draw_triangle(
                buffer, pixel_w, pixel_h, scale, *x1, *y1, *x2, *y2, *x3, *y3, color,
            );
        }
    }
}

// ===== 像素光栅化原语 =====

/// 绘制填充三角形（逻辑坐标，重心坐标法光栅化）。
fn draw_triangle(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: u32,
) {
    // 转换到物理像素坐标。
    let (px1, py1) = (x1 * scale, y1 * scale);
    let (px2, py2) = (x2 * scale, y2 * scale);
    let (px3, py3) = (x3 * scale, y3 * scale);

    // 包围盒。
    let min_x = px1.min(px2).min(px3).floor() as i32;
    let max_x = px1.max(px2).max(px3).ceil() as i32;
    let min_y = py1.min(py2).min(py3).floor() as i32;
    let max_y = py1.max(py2).max(py3).ceil() as i32;

    let x0 = min_x.max(0);
    let y0 = min_y.max(0);
    let x1_clip = max_x.min(pixel_w as i32);
    let y1_clip = max_y.min(pixel_h as i32);

    // 三角形面积（2 倍）。
    let area = (px2 - px1) * (py3 - py1) - (px3 - px1) * (py2 - py1);
    if area.abs() < 0.01 {
        return;
    }

    for py in y0..y1_clip {
        for px in x0..x1_clip {
            let pxc = px as f32 + 0.5;
            let pyc = py as f32 + 0.5;
            // 重心坐标判断点是否在三角形内。
            let w0 = (px2 - pxc) * (py3 - pyc) - (px3 - pxc) * (py2 - pyc);
            let w1 = (px3 - pxc) * (py1 - pyc) - (px1 - pxc) * (py3 - pyc);
            let w2 = (px1 - pxc) * (py2 - pyc) - (px2 - pxc) * (py1 - pyc);
            // 判断三个重心坐标同号。
            let inside =
                (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if inside {
                let idx = (py as u32) * pixel_w + (px as u32);
                if (idx as usize) < buffer.len() {
                    buffer[idx as usize] = color;
                }
            }
        }
    }
}

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

// ===== PNG 图片加载与绘制 =====

/// 加载 PNG 文件，解码为 RGBA 像素向量。
///
/// 返回 (pixels, width, height)，pixels 为行优先从上到下的 RGBA 元组。
#[allow(clippy::type_complexity)]
fn load_png(
    path: &str,
) -> Result<(Vec<(u8, u8, u8, u8)>, usize, usize), Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let mut reader = decoder.read_info()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf)?;
    let bytes = &buf[..frame.buffer_size()];

    let w = info.width as usize;
    let h = info.height as usize;

    // 根据 PNG 的颜色类型转换为统一的 RGBA 元组。
    let pixels: Vec<(u8, u8, u8, u8)> = match info.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|c| (c[0], c[1], c[2], c[3]))
            .collect(),
        png::ColorType::Rgb => {
            // RGB 无 alpha，默认不透明。
            bytes
                .chunks_exact(3)
                .map(|c| (c[0], c[1], c[2], 255))
                .collect()
        }
        png::ColorType::Grayscale => bytes.iter().map(|&v| (v, v, v, 255)).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|c| (c[0], c[0], c[0], c[1]))
            .collect(),
        png::ColorType::Indexed => {
            // 调色板模式：reader 已将输出转为 RGBA（png crate 的 output 转换），
            // 但如果 output 仍为 indexed，则按 RGB 处理。
            bytes
                .chunks_exact(3)
                .map(|c| (c[0], c[1], c[2], 255))
                .collect()
        }
    };

    Ok((pixels, w, h))
}

/// 将 PNG 图片按缩放比例绘制到 softbuffer 像素缓冲区。
///
/// - `img_scale`：图片缩放比例（1.0 = 原始大小，逻辑像素）。
/// - `offset_x`/`offset_y`：相对屏幕中心的偏移（逻辑像素）。
/// - `opacity`：全局不透明度（与图片 alpha 相乘）。
fn draw_image(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &crate::shapes::RectF,
    img: &CachedImage,
    img_scale: f32,
    offset_x: f32,
    offset_y: f32,
    opacity: f32,
) {
    // 缩放后的逻辑尺寸。
    let scaled_w = img.width as f32 * img_scale;
    let scaled_h = img.height as f32 * img_scale;

    // 图片中心在屏幕中心 + 偏移。
    let center_x = rect.center_x() + offset_x;
    let center_y = rect.center_y() + offset_y;

    // 图片左上角的物理像素坐标。
    let px_start_x = ((center_x - scaled_w / 2.0) * scale).round() as i32;
    let px_start_y = ((center_y - scaled_h / 2.0) * scale).round() as i32;
    // 图片覆盖的物理像素尺寸。
    let px_w = (scaled_w * scale).round() as usize;
    let px_h = (scaled_h * scale).round() as usize;

    for py in 0..px_h {
        let dst_y = px_start_y + py as i32;
        if dst_y < 0 || dst_y >= pixel_h as i32 {
            continue;
        }
        for px in 0..px_w {
            let dst_x = px_start_x + px as i32;
            if dst_x < 0 || dst_x >= pixel_w as i32 {
                continue;
            }
            // 将物理像素映射回原图坐标（最近邻采样）。
            let src_x = (px as f32 / px_w as f32 * img.width as f32) as usize;
            let src_y = (py as f32 / px_h as f32 * img.height as f32) as usize;
            let src_x = src_x.min(img.width - 1);
            let src_y = src_y.min(img.height - 1);
            let (r, g, b, a) = img.pixels[src_y * img.width + src_x];

            let final_alpha = (a as f32 / 255.0 * opacity).clamp(0.0, 1.0);
            if final_alpha < 0.01 {
                continue;
            }

            // 预乘 alpha 的 0xAARRGGBB 格式。
            let ai = (final_alpha * 255.0) as u32;
            let ri = (r as f32 * final_alpha) as u32;
            let gi = (g as f32 * final_alpha) as u32;
            let bi = (b as f32 * final_alpha) as u32;
            let pixel = (ai << 24) | (ri << 16) | (gi << 8) | bi;

            let idx = dst_y as usize * pixel_w as usize + dst_x as usize;
            if idx < buffer.len() {
                buffer[idx] = pixel;
            }
        }
    }
}
