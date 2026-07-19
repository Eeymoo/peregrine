//! SVG 渲染后端。
//!
//! 将 [`shapes::Shape`] 图元序列转换为 SVG 字符串，
//! 再由 resvg/usvg + tiny-skia 光栅化为物理像素缓冲区。
//!
//! 与 CPU 手写光栅化（`overlay_renderer` 内的 `draw_*` 函数）并行存在，
//! 通过 `settings.renderer_backend` 配置切换。
//!
//! 像素格式：softbuffer 要求 `0xAARRGGBB`（预乘 alpha）。
//! tiny-skia 输出为非预乘 `0xAARRGGBB`，本模块负责转换。

use peregrine_config::Crosshair;

use crate::shapes::{self, RectF, Shape};

/// 用 SVG 光栅化后端渲染准心到像素缓冲区。
///
/// - `buffer`：softbuffer 像素缓冲区（`0xAARRGGBB`，预乘 alpha）。
/// - `pixel_w` / `pixel_h`：物理像素宽高。
/// - `scale`：DPI 缩放因子（`window.scale_factor()`）。
/// - `rect`：逻辑坐标绘制区域（通常为全屏）。
/// - `crosshair`：准心配置。
///
/// 返回 true 表示渲染成功；false 表示光栅化失败（调用方可回退到 CPU 路径）。
pub fn render_shapes_to_buffer(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &RectF,
    crosshair: &Crosshair,
) -> bool {
    // 构造 SVG 字符串（逻辑坐标 + scale 变换）。
    let svg = build_svg(rect, crosshair, scale);
    if svg.is_empty() {
        return true; // 无图元需要绘制（如 CustomImage 不走此路径）。
    }

    render_svg_to_buffer(buffer, pixel_w, pixel_h, &svg)
}

/// 把 SVG 字符串光栅化到像素缓冲区。
fn render_svg_to_buffer(buffer: &mut [u32], pixel_w: u32, pixel_h: u32, svg: &str) -> bool {
    // 解析 SVG 为 usvg Tree。
    let tree = match usvg::Tree::from_str(svg, &usvg::Options::default()) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("SVG 解析失败: {}", e);
            return false;
        }
    };

    // 用 tiny-skia 光栅化。
    let mut pixmap = match tiny_skia::Pixmap::new(pixel_w, pixel_h) {
        Some(pm) => pm,
        None => {
            tracing::warn!("pixmap 创建失败 ({}x{})", pixel_w, pixel_h);
            return false;
        }
    };
    // tiny-skia 的 transform 是逻辑→物理的缩放。
    // 但 SVG 中已经乘了 scale（见 build_svg），所以这里用 identity。
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    // 将 tiny-skia 的非预乘 RGBA 像素转为 softbuffer 的预乘 0xAARRGGBB。
    // 只写入 alpha > 0 的像素，保留 buffer 中已有内容（用于与 CPU 光栅化结果叠加）。
    let data = pixmap.data();
    let len = data.len() / 4;
    let buf_len = buffer.len().min(len);
    for i in 0..buf_len {
        let r = data[i * 4] as f32 / 255.0;
        let g = data[i * 4 + 1] as f32 / 255.0;
        let b = data[i * 4 + 2] as f32 / 255.0;
        let a = data[i * 4 + 3] as f32 / 255.0;
        // 预乘 alpha。
        let pr = (r * a * 255.0).round() as u32;
        let pg = (g * a * 255.0).round() as u32;
        let pb = (b * a * 255.0).round() as u32;
        let pa = (a * 255.0).round() as u32;
        if pa > 0 {
            buffer[i] = (pa << 24) | (pr << 16) | (pg << 8) | pb;
        }
    }

    true
}

/// 把多图层图元列表（Element + color + opacity）光栅化到像素缓冲区。
///
/// 用于 overlay 新格式路径中渲染 `Element::Text` 等 CPU 路径暂未实现的图元。
pub fn render_elements_to_buffer(
    buffer: &mut [u32],
    pixel_w: u32,
    pixel_h: u32,
    scale: f32,
    rect: &RectF,
    elements: &[(peregrine_config::Element, [f32; 4], f32)],
) -> bool {
    let svg = build_elements_svg(rect, elements, scale);
    if svg.is_empty() {
        return true;
    }
    render_svg_to_buffer(buffer, pixel_w, pixel_h, &svg)
}

/// 把 Element 列表转成 SVG 字符串。
fn build_elements_svg(
    rect: &RectF,
    elements: &[(peregrine_config::Element, [f32; 4], f32)],
    scale: f32,
) -> String {
    if elements.is_empty() {
        return String::new();
    }

    let pw = rect.width() * scale;
    let ph = rect.height() * scale;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = pw,
        h = ph,
    );

    for (shape, color, opacity) in elements {
        let [cr, cg, cb, ca] = *color;
        let alpha = (ca * *opacity).clamp(0.0, 1.0);
        let r = (cr * 255.0).round().clamp(0.0, 255.0) as u32;
        let g = (cg * 255.0).round().clamp(0.0, 255.0) as u32;
        let b = (cb * 255.0).round().clamp(0.0, 255.0) as u32;
        let fill = format!("rgb({},{},{})", r, g, b);
        let stroke = &fill;

        match shape {
            Shape::Rect { x, y, w, h } => {
                svg.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" opacity="{op}"/>"#,
                    x = *x * scale,
                    y = *y * scale,
                    w = *w * scale,
                    h = *h * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Circle { cx, cy, radius } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::CircleStroke {
                cx,
                cy,
                radius,
                thickness,
            } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{stroke}" stroke-width="{sw}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    op = alpha,
                ));
            }
            Shape::DashedCircle {
                cx,
                cy,
                radius,
                thickness,
                dash_len,
                gap_len,
            } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{stroke}" stroke-width="{sw}" stroke-dasharray="{dl},{gl}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    dl = *dash_len * scale,
                    gl = *gap_len * scale,
                    op = alpha,
                ));
            }
            Shape::Triangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                svg.push_str(&format!(
                    r#"<polygon points="{x1},{y1} {x2},{y2} {x3},{y3}" fill="{fill}" opacity="{op}"/>"#,
                    x1 = *x1 * scale,
                    y1 = *y1 * scale,
                    x2 = *x2 * scale,
                    y2 = *y2 * scale,
                    x3 = *x3 * scale,
                    y3 = *y3 * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Polygon { points } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|p| format!("{},{}", p[0] * scale, p[1] * scale))
                    .collect();
                svg.push_str(&format!(
                    r#"<polygon points="{pts}" fill="{fill}" opacity="{op}"/>"#,
                    pts = pts.join(" "),
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Line {
                x1,
                y1,
                x2,
                y2,
                thickness,
            } => {
                svg.push_str(&format!(
                    r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{sw}" stroke-linecap="round" opacity="{op}"/>"#,
                    x1 = *x1 * scale,
                    y1 = *y1 * scale,
                    x2 = *x2 * scale,
                    y2 = *y2 * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    op = alpha,
                ));
            }
            Shape::Text {
                x,
                y,
                content,
                font_size,
            } => {
                svg.push_str(&format!(
                    r#"<text x="{x}" y="{y}" font-size="{fs}" fill="{fill}" opacity="{op}">{c}</text>"#,
                    x = *x * scale,
                    y = *y * scale,
                    fs = *font_size * scale,
                    fill = fill,
                    op = alpha,
                    c = content.replace('<', "&lt;").replace('>', "&gt;"),
                ));
            }
            Shape::Image { x, y, w, h, path } => {
                // SVG 嵌入图片引用（实际渲染由上层单独处理）。
                let _ = (x, y, w, h, path);
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// 将 [`Shape`] 序列 + 准心配置转换为 SVG 字符串。
///
/// 所有坐标已经乘以 `scale`（物理像素），SVG viewBox 为物理像素尺寸。
/// 颜色使用准心的 `color` + `opacity`。
fn build_svg(rect: &RectF, crosshair: &Crosshair, scale: f32) -> String {
    let shapes = shapes::build_shapes(rect, crosshair);
    if shapes.is_empty() {
        return String::new();
    }

    let pw = rect.width() * scale;
    let ph = rect.height() * scale;

    // 颜色：SVG 用非预乘 sRGB，格式 #RRGGBB，opacity 属性控制透明度。
    let [cr, cg, cb, ca] = crosshair.color;
    let opacity = crosshair.opacity;
    let alpha = (ca * opacity).clamp(0.0, 1.0);
    let r = (cr * 255.0).round().clamp(0.0, 255.0) as u32;
    let g = (cg * 255.0).round().clamp(0.0, 255.0) as u32;
    let b = (cb * 255.0).round().clamp(0.0, 255.0) as u32;
    let fill = format!("rgb({},{},{})", r, g, b);
    let stroke = &fill;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = pw,
        h = ph,
    );

    for shape in &shapes {
        match shape {
            Shape::Rect { x, y, w, h } => {
                svg.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" opacity="{op}"/>"#,
                    x = *x * scale,
                    y = *y * scale,
                    w = *w * scale,
                    h = *h * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Circle { cx, cy, radius } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::CircleStroke {
                cx,
                cy,
                radius,
                thickness,
            } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{stroke}" stroke-width="{sw}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    op = alpha,
                ));
            }
            Shape::DashedCircle {
                cx,
                cy,
                radius,
                thickness,
                dash_len,
                gap_len,
            } => {
                svg.push_str(&format!(
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{stroke}" stroke-width="{sw}" stroke-dasharray="{dl},{gl}" opacity="{op}"/>"#,
                    cx = *cx * scale,
                    cy = *cy * scale,
                    r = *radius * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    dl = *dash_len * scale,
                    gl = *gap_len * scale,
                    op = alpha,
                ));
            }
            Shape::Triangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                svg.push_str(&format!(
                    r#"<polygon points="{x1},{y1} {x2},{y2} {x3},{y3}" fill="{fill}" opacity="{op}"/>"#,
                    x1 = *x1 * scale,
                    y1 = *y1 * scale,
                    x2 = *x2 * scale,
                    y2 = *y2 * scale,
                    x3 = *x3 * scale,
                    y3 = *y3 * scale,
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Polygon { points } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|p| format!("{},{}", p[0] * scale, p[1] * scale))
                    .collect();
                svg.push_str(&format!(
                    r#"<polygon points="{pts}" fill="{fill}" opacity="{op}"/>"#,
                    pts = pts.join(" "),
                    fill = fill,
                    op = alpha,
                ));
            }
            Shape::Line {
                x1,
                y1,
                x2,
                y2,
                thickness,
            } => {
                svg.push_str(&format!(
                    r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{sw}" stroke-linecap="round" opacity="{op}"/>"#,
                    x1 = *x1 * scale,
                    y1 = *y1 * scale,
                    x2 = *x2 * scale,
                    y2 = *y2 * scale,
                    stroke = stroke,
                    sw = *thickness * scale,
                    op = alpha,
                ));
            }
            Shape::Text {
                x,
                y,
                content,
                font_size,
            } => {
                svg.push_str(&format!(
                    r#"<text x="{x}" y="{y}" font-size="{fs}" fill="{fill}" opacity="{op}">{c}</text>"#,
                    x = *x * scale,
                    y = *y * scale,
                    fs = *font_size * scale,
                    fill = fill,
                    op = alpha,
                    c = content.replace('<', "&lt;").replace('>', "&gt;"),
                ));
            }
            Shape::Image { x, y, w, h, path } => {
                // SVG 嵌入图片引用（实际渲染由上层单独处理）。
                let _ = (x, y, w, h, path);
            }
        }
    }

    svg.push_str("</svg>");
    svg
}
