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

use peregrine_config::{Crosshair, CrosshairStyle, RendererBackend};
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
    /// 物料注册表（用于图层求值）。
    material_registry: Arc<peregrine_material::MaterialRegistry>,
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
    pub fn new(
        window: Arc<Window>,
        config: Arc<Mutex<peregrine_config::ConfigSnapshot>>,
        material_registry: Arc<peregrine_material::MaterialRegistry>,
    ) -> Self {
        // softbuffer 要求 Context 和 Surface 共享同一个 window 引用。
        let context = softbuffer::Context::new(window.clone()).expect("create softbuffer context");
        let surface =
            softbuffer::Surface::new(&context, window.clone()).expect("create softbuffer surface");

        Self {
            window,
            context,
            surface,
            config,
            material_registry,
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
        let profile = config.active_profile();
        let antialiasing = config.settings.antialiasing;
        let renderer_backend = config.settings.renderer_backend;

        // 判断走新格式（layers）还是旧格式（crosshair）：
        // - 新格式：layers 非空 → 调用 build_layers_shapes
        // - 旧格式：crosshair = Some(...) → 调用旧 build_shapes
        let use_new_format = profile.map(|p| !p.layers.is_empty()).unwrap_or(false);

        // 旧格式路径：克隆 crosshair，供 build_shapes 使用。
        let legacy_crosshair = if !use_new_format {
            profile
                .and_then(|p| p.crosshair.clone())
                .unwrap_or_else(Crosshair::default_crosshair)
        } else {
            // 新格式不使用 crosshair，但保留默认值用于 is_custom_image 检查。
            Crosshair::default_crosshair()
        };
        let profile_clone = profile.cloned();
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
        // 新格式路径下，image 加载延迟到光栅化 Image 图元时处理。
        let is_custom_image =
            !use_new_format && legacy_crosshair.style == CrosshairStyle::CustomImage;
        if is_custom_image {
            self.ensure_image_loaded(&legacy_crosshair.image_path);
        }

        // 新格式路径：扫描 layers 中所有 Image 图元的路径，预加载到缓存。
        if use_new_format {
            if let Some(ref profile) = profile_clone {
                // 渲染时用真实动态输入（Win32 API 鼠标键盘 / 时间）。
                let ctx = crate::platform::poll_dynamic_context(logical_w, logical_h);
                let shapes = crate::shapes::build_layers_shapes(
                    &rect,
                    profile,
                    &self.material_registry,
                    &ctx,
                );
                // 收集需要预加载的 image path。
                let image_paths: Vec<String> = shapes
                    .iter()
                    .filter_map(|(e, _, _)| match e {
                        peregrine_config::Element::Image { path, .. } => Some(path.clone()),
                        _ => None,
                    })
                    .collect();
                for path in image_paths {
                    self.ensure_image_loaded(&path);
                }
            }
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

        if use_new_format {
            // ===== 新格式路径：遍历图层 =====
            let Some(ref profile) = profile_clone else {
                return;
            };
            // 渲染时使用真实动态输入。
            let ctx = crate::platform::poll_dynamic_context(logical_w, logical_h);
            let shapes =
                crate::shapes::build_layers_shapes(&rect, profile, &self.material_registry, &ctx);

            for (element, color, opacity) in shapes {
                let color_u32 = make_color(&color, opacity);
                match &element {
                    peregrine_config::Element::Image { x, y, w, h, .. } => {
                        if let Some(img) = &self.image_cache {
                            // 复用现有 draw_image 但参数从 Image 图元取。
                            // 注意：draw_image 期望中心点 + offset，这里改为左上角 + w/h。
                            // 简化实现：用 draw_image_at_left_top。
                            draw_image_at_left_top(
                                &mut buffer,
                                width,
                                height,
                                scale,
                                *x,
                                *y,
                                *w,
                                *h,
                                img,
                                opacity,
                            );
                        }
                    }
                    _ => {
                        // 其他图元：转 Shape 别名（Element）走 rasterize_shape。
                        rasterize_shape(
                            &mut buffer,
                            width,
                            height,
                            scale,
                            &element,
                            color_u32,
                            antialiasing,
                        );
                    }
                }
            }
        } else if is_custom_image {
            // 旧格式 CustomImage 路径（保留兼容）。
            if let Some(img) = &self.image_cache {
                let opacity = legacy_crosshair.opacity;
                let img_scale = legacy_crosshair.image_scale;
                let offset_x = legacy_crosshair.image_offset_x;
                let offset_y = legacy_crosshair.image_offset_y;
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
        } else if renderer_backend == RendererBackend::Svg {
            // SVG 后端：将图元转为 SVG 由 resvg/tiny-skia 光栅化。
            let ok = crate::svg_renderer::render_shapes_to_buffer(
                &mut buffer,
                width,
                height,
                scale,
                &rect,
                &legacy_crosshair,
            );
            if !ok {
                tracing::warn!("SVG 光栅化失败，回退到 CPU 渲染");
                let color = make_color(&legacy_crosshair.color, legacy_crosshair.opacity);
                let shapes = crate::shapes::build_shapes(&rect, &legacy_crosshair);
                for shape in shapes {
                    rasterize_shape(
                        &mut buffer,
                        width,
                        height,
                        scale,
                        &shape,
                        color,
                        antialiasing,
                    );
                }
            }
        } else {
            // CPU 后端：手写像素光栅化（旧格式路径，默认）。
            let color = make_color(&legacy_crosshair.color, legacy_crosshair.opacity);
            let shapes = crate::shapes::build_shapes(&rect, &legacy_crosshair);
            for shape in shapes {
                rasterize_shape(
                    &mut buffer,
                    width,
                    height,
                    scale,
                    &shape,
                    color,
                    antialiasing,
                );
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

/// 将 PNG 图片绘制到指定左上角坐标 + 宽高（用于新格式 Image 图元）。
///
/// 与 `draw_image`（基于中心点）不同，这个函数直接用左上角坐标，
/// 简化 Element::Image 的渲染。
fn draw_image_at_left_top(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    img: &CachedImage,
    opacity: f32,
) {
    let scaled_w = w;
    let scaled_h = h;
    let _ = scaled_w;
    let _ = scaled_h;

    let px_start_x = (x * scale).round() as i32;
    let px_start_y = (y * scale).round() as i32;
    let px_w = (w * scale).round() as usize;
    let px_h = (h * scale).round() as usize;

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
            let src_x = (px as f32 / px_w as f32 * img.width as f32) as usize;
            let src_y = (py as f32 / px_h as f32 * img.height as f32) as usize;
            let src_x = src_x.min(img.width - 1);
            let src_y = src_y.min(img.height - 1);
            let (r, g, b, a) = img.pixels[src_y * img.width + src_x];

            let final_alpha = (a as f32 / 255.0 * opacity).clamp(0.0, 1.0);
            if final_alpha < 0.01 {
                continue;
            }

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
    antialiasing: bool,
) {
    use crate::shapes::Shape;
    match shape {
        Shape::Rect { x, y, w, h } => {
            draw_rect(buffer, pixel_w, pixel_h, scale, *x, *y, *w, *h, color);
        }
        Shape::Circle { cx, cy, radius } => {
            if antialiasing {
                draw_circle(buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, color);
            } else {
                draw_circle_fast(buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, color);
            }
        }
        Shape::CircleStroke {
            cx,
            cy,
            radius,
            thickness,
        } => {
            if antialiasing {
                draw_circle_stroke(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, color,
                );
            } else {
                draw_circle_stroke_fast(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, color,
                );
            }
        }
        Shape::DashedCircle {
            cx,
            cy,
            radius,
            thickness,
            dash_len,
            gap_len,
        } => {
            if antialiasing {
                draw_dashed_circle(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, *dash_len,
                    *gap_len, color,
                );
            } else {
                draw_dashed_circle_fast(
                    buffer, pixel_w, pixel_h, scale, *cx, *cy, *radius, *thickness, *dash_len,
                    *gap_len, color,
                );
            }
        }
        Shape::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => {
            if antialiasing {
                draw_triangle(
                    buffer, pixel_w, pixel_h, scale, *x1, *y1, *x2, *y2, *x3, *y3, color,
                );
            } else {
                draw_triangle_fast(
                    buffer, pixel_w, pixel_h, scale, *x1, *y1, *x2, *y2, *x3, *y3, color,
                );
            }
        }
        // 新增图元类型的占位实现（Step 9 会完整实现光栅化）。
        Shape::Polygon { .. } | Shape::Line { .. } | Shape::Text { .. } => {
            tracing::debug!(
                "rasterize_shape: 该图元类型在旧 crosshair 路径下不渲染（Step 9 实现）"
            );
        }
        Shape::Image { x, y, w, h, path } => {
            // Step 9 完整实现；这里仅记录路径，实际 blit 由上层 CustomImage 分支处理。
            let _ = (x, y, w, h, path);
        }
    }
}

/// 把颜色分量按覆盖率混合写入像素缓冲区（前景预乘 alpha，背景透明）。
///
/// 由于覆盖层背景始终透明（0x00000000），混合结果为预乘值：
/// `out = fg_premul * coverage + bg * (1 - coverage)`。
/// 背景为 0 时简化为 `out = fg_premul * coverage`。
fn blend_pixel(buffer: &mut [u32], idx: usize, color: u32, coverage: f32) {
    if coverage <= 0.0 || idx >= buffer.len() {
        return;
    }
    let cov = coverage.min(1.0);
    // 颜色已经是预乘 alpha 格式，直接按覆盖率缩放各分量。
    let ai = ((color >> 24) & 0xFF) as f32 * cov;
    let ri = ((color >> 16) & 0xFF) as f32 * cov;
    let gi = ((color >> 8) & 0xFF) as f32 * cov;
    let bi = (color & 0xFF) as f32 * cov;
    buffer[idx] = ((ai as u32) << 24) | ((ri as u32) << 16) | ((gi as u32) << 8) | (bi as u32);
}

/// 绘制填充三角形（逻辑坐标，边距离抗锯齿）。
///
/// 使用三条边的有符号距离（重心坐标）判断像素在三角形内/外的程度：
/// 三条边线函数同号 → 内部；覆盖率取最接近 0 的边线值的平滑映射，
/// 在边缘 1 像素范围内平滑过渡。
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
    let x1_clip = (max_x + 1).min(pixel_w as i32);
    let y1_clip = (max_y + 1).min(pixel_h as i32);

    // 三角形面积（2 倍），用于确定绕序方向。
    let area = (px2 - px1) * (py3 - py1) - (px3 - px1) * (py2 - py1);
    if area.abs() < 0.01 {
        return;
    }
    // sign > 0 表示逆时针，sign < 0 表示顺时针。
    // 标准化：使所有内部像素的 w0/w1/w2 ≥ 0。
    let sign = if area > 0.0 { 1.0 } else { -1.0 };

    // 三条边的向量长度，用于将边线函数值归一化为像素距离。
    let len_e0 = ((px3 - px2).powi(2) + (py3 - py2).powi(2)).sqrt();
    let len_e1 = ((px1 - px3).powi(2) + (py1 - py3).powi(2)).sqrt();
    let len_e2 = ((px2 - px1).powi(2) + (py2 - py1).powi(2)).sqrt();

    for py in y0..y1_clip {
        for px in x0..x1_clip {
            let pxc = px as f32 + 0.5;
            let pyc = py as f32 + 0.5;
            // 三条边线函数（乘以 sign 使内部为正）。
            let w0 = sign * ((px2 - pxc) * (py3 - pyc) - (px3 - pxc) * (py2 - pyc));
            let w1 = sign * ((px3 - pxc) * (py1 - pyc) - (px1 - pxc) * (py3 - pyc));
            let w2 = sign * ((px1 - pxc) * (py2 - pyc) - (px2 - pxc) * (py1 - pyc));

            // 完全在外（任一边线为负且距离 > 1px）→ 跳过。
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                // 近似：如果最大负值对应边线距离 > 1 像素则跳过。
                let min_w = w0.min(w1).min(w2);
                let max_len = len_e0.max(len_e1).max(len_e2);
                if min_w < -max_len {
                    continue;
                }
            }

            // 将边线函数归一化为到边的距离（除以边长）。
            let d0 = w0 / len_e0;
            let d1 = w1 / len_e1;
            let d2 = w2 / len_e2;

            // 覆盖率 = 最小边距的平滑映射（在 0~1px 过渡）。
            let min_d = d0.min(d1).min(d2);
            let coverage = (min_d + 0.5).clamp(0.0, 1.0);

            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
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

/// 绘制填充圆（逻辑坐标，距离场抗锯齿）。
///
/// 使用像素中心到圆心的距离与半径的关系计算覆盖率：
/// `coverage = clamp(radius + 0.5 - dist, 0, 1)`，
/// 在边缘 1 像素范围内平滑过渡，消除锯齿。
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
    let pcx = cx * scale;
    let pcy = cy * scale;
    let pr = radius * scale;
    // 包围盒扩展 1px 以覆盖抗锯齿过渡区域。
    let x0 = ((pcx - pr - 1.0).floor() as i32).max(0);
    let y0 = ((pcy - pr - 1.0).floor() as i32).max(0);
    let x1 = ((pcx + pr + 1.0).ceil() as i32).min(pixel_w as i32);
    let y1 = ((pcy + pr + 1.0).ceil() as i32).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - pcx;
            let dy = py as f32 + 0.5 - pcy;
            let dist = (dx * dx + dy * dy).sqrt();
            let coverage = (pr + 0.5 - dist).clamp(0.0, 1.0);
            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
            }
        }
    }
}

/// 绘制圆环描边（逻辑坐标，距离场抗锯齿）。
///
/// 使用圆环的 SDF（有符号距离场）计算覆盖率：
/// `sdf = |dist - center_r| - half_thickness`
/// `coverage = clamp(0.5 - sdf, 0, 1)`，
/// 在内外边缘各 1 像素范围内平滑过渡。
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
    let pcx = cx * scale;
    let pcy = cy * scale;
    let center_r = radius * scale;
    let half_t = thickness * scale / 2.0;
    let outer_r = center_r + half_t;
    // 包围盒扩展 1px 以覆盖抗锯齿过渡区域。
    let x0 = ((pcx - outer_r - 1.0).floor() as i32).max(0);
    let y0 = ((pcy - outer_r - 1.0).floor() as i32).max(0);
    let x1 = ((pcx + outer_r + 1.0).ceil() as i32).min(pixel_w as i32);
    let y1 = ((pcy + outer_r + 1.0).ceil() as i32).min(pixel_h as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - pcx;
            let dy = py as f32 + 0.5 - pcy;
            let dist = (dx * dx + dy * dy).sqrt();
            // 圆环 SDF：到中心半径的距离减去半厚度。
            let sdf = (dist - center_r).abs() - half_t;
            let coverage = (0.5 - sdf).clamp(0.0, 1.0);
            if coverage > 0.0 {
                let idx = (py as u32) * pixel_w + (px as u32);
                blend_pixel(buffer, idx as usize, color, coverage);
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

// ===== 关闭抗锯齿时的快速路径（硬二值光栅化，无 sqrt） =====

/// 绘制填充圆（无抗锯齿）。
fn draw_circle_fast(
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

/// 绘制圆环描边（无抗锯齿）。
fn draw_circle_stroke_fast(
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

/// 绘制虚线圆（无抗锯齿）。
fn draw_dashed_circle_fast(
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
            draw_circle_fast(
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

/// 绘制填充三角形（无抗锯齿，重心坐标法）。
fn draw_triangle_fast(
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
    let (px1, py1) = (x1 * scale, y1 * scale);
    let (px2, py2) = (x2 * scale, y2 * scale);
    let (px3, py3) = (x3 * scale, y3 * scale);
    let min_x = px1.min(px2).min(px3).floor() as i32;
    let max_x = px1.max(px2).max(px3).ceil() as i32;
    let min_y = py1.min(py2).min(py3).floor() as i32;
    let max_y = py1.max(py2).max(py3).ceil() as i32;
    let x0 = min_x.max(0);
    let y0 = min_y.max(0);
    let x1_clip = max_x.min(pixel_w as i32);
    let y1_clip = max_y.min(pixel_h as i32);
    let area = (px2 - px1) * (py3 - py1) - (px3 - px1) * (py2 - py1);
    if area.abs() < 0.01 {
        return;
    }
    for py in y0..y1_clip {
        for px in x0..x1_clip {
            let pxc = px as f32 + 0.5;
            let pyc = py as f32 + 0.5;
            let w0 = (px2 - pxc) * (py3 - pyc) - (px3 - pxc) * (py2 - pyc);
            let w1 = (px3 - pxc) * (py1 - pyc) - (px1 - pxc) * (py3 - pyc);
            let w2 = (px1 - pxc) * (py2 - pyc) - (px2 - pxc) * (py1 - pyc);
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
