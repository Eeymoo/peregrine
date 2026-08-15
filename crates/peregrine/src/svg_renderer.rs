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
    // 解析 SVG：默认字体集（Segoe UI）或含非 ASCII 文本时的 CJK 全量兜底。
    let options = if svg_needs_cjk(svg) {
        &*FONT_OPTIONS_CJK
    } else {
        &*FONT_OPTIONS
    };
    let tree = match usvg::Tree::from_str(svg, options) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("SVG 解析失败: {}", e);
            return false;
        }
    };

    // 复用线程局部的 tiny-skia Pixmap：按需重建，同尺寸直接复用，
    // 消除每次渲染 8MB（1080p）的分配/释放波动。
    thread_local! {
        static PIXMAP: std::cell::RefCell<Option<tiny_skia::Pixmap>> =
            const { std::cell::RefCell::new(None) };
    }
    PIXMAP.with(|slot| {
        let mut slot = slot.borrow_mut();
        let need_new = slot
            .as_ref()
            .is_none_or(|pm| pm.width() != pixel_w || pm.height() != pixel_h);
        if need_new {
            match tiny_skia::Pixmap::new(pixel_w, pixel_h) {
                Some(pm) => *slot = Some(pm),
                None => {
                    tracing::warn!("pixmap 创建失败 ({}x{})", pixel_w, pixel_h);
                    return false;
                }
            }
        }
        let pixmap = slot.as_mut().expect("pixmap just ensured");
        pixmap.data_mut().fill(0);
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
    })
}

/// usvg 解析选项（默认字体集），按需懒加载。
///
/// 内存预算（<30MB）不允许驻留全量系统字体库（`load_system_fonts` 全量
/// 扫描并保留数百字体文件 ≈ 25MB+）。策略：
/// - **默认**只加载 Segoe UI 常规 + 粗体两个文件（各 <1MB，Windows 自带），
///   `font_family` 指向 Segoe UI——数字/拉丁/时钟场景全覆盖；
/// - **兜底**：文本图元含非 ASCII 字符（中文等 CJK，Segoe UI 无字形）时，
///   懒加载一次雅黑定向字体集（`FONT_OPTIONS_CJK`），豆腐块兜底。
///
/// 两条路径都进程内只加载一次，稳态零分配零重复扫描。
static FONT_OPTIONS: std::sync::LazyLock<usvg::Options> = std::sync::LazyLock::new(|| {
    let mut options = usvg::Options {
        font_family: "Segoe UI".to_string(),
        ..Default::default()
    };
    let mut db = fontdb::Database::new();
    // Segoe UI 家族（Windows 系统字体，始终存在；缺失时静默跳过，
    // 后续 resvg 回退 usvg 内置默认字体）。
    let fonts_dir = std::path::PathBuf::from(r"C:\Windows\Fonts");
    for font in ["segoeui.ttf", "segoeuib.ttf"] {
        let p = fonts_dir.join(font);
        if let Err(e) = db.load_font_file(&p) {
            tracing::debug!(path = %p.display(), error = %e, "optional font not loaded");
        }
    }
    *options.fontdb_mut() = db;
    options
});

/// CJK 兜底选项，仅在文本含非 ASCII 字符时懒加载一次。
///
/// 零遍历策略：定向 mmap 微软雅黑（`msyh.ttc`，Windows Vista+ 自带，
/// 含简中日韩字形），命中则零字体目录扫描；仅当文件缺失（精简系统 /
/// 非常规环境）才退回 `load_system_fonts()` 全量遍历兜底。
static FONT_OPTIONS_CJK: std::sync::LazyLock<usvg::Options> = std::sync::LazyLock::new(|| {
    let mut options = usvg::Options {
        font_family: "Microsoft YaHei".to_string(),
        ..Default::default()
    };
    let mut db = fontdb::Database::new();
    let fonts_dir = std::path::PathBuf::from(r"C:\Windows\Fonts");
    // msyh.ttc 常规 + msyhbd.ttc 粗体；任一命中即不遍历。
    let mut loaded = false;
    for font in ["msyh.ttc", "msyhbd.ttc"] {
        let p = fonts_dir.join(font);
        if db.load_font_file(&p).is_ok() {
            loaded = true;
        }
    }
    if !loaded {
        tracing::info!("msyh not found, fallback to full system font scan");
        db.load_system_fonts();
    }
    *options.fontdb_mut() = db;
    options
});

/// 判断 SVG 文本是否包含非 ASCII 字符（需要 CJK 字形兜底）。
///
/// 只扫 `<text>` 内容太琐碎，直接扫整个 SVG 字符串：坐标数字/标签全是
/// ASCII，命中非 ASCII 字节即视为含 CJK 文本，走全量字体库。
fn svg_needs_cjk(svg: &str) -> bool {
    svg.bytes().any(|b| b > 0x7F)
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
            Shape::Rect {
                x,
                y,
                w,
                h,
                corner_radius,
            } => {
                // 任务 9.4：SVG `<rect>` 支持 rx 属性渲染圆角。
                let rx_attr = match corner_radius {
                    Some(r) if *r > 0.0 => format!(r#" rx="{}""#, *r * scale),
                    _ => String::new(),
                };
                svg.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{w}" height="{h}"{rx} fill="{fill}" opacity="{op}"/>"#,
                    x = *x * scale,
                    y = *y * scale,
                    w = *w * scale,
                    h = *h * scale,
                    rx = rx_attr,
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
                font_weight,
            } => {
                // 字重缺省（None）时不输出 font-weight 属性，按常规 400 渲染。
                let weight_attr = match font_weight {
                    Some(w) => format!(r#" font-weight="{w}""#),
                    None => String::new(),
                };
                svg.push_str(&format!(
                    r#"<text x="{x}" y="{y}" font-size="{fs}"{weight_attr} fill="{fill}" opacity="{op}">{c}</text>"#,
                    x = *x * scale,
                    y = *y * scale,
                    fs = *font_size * scale,
                    weight_attr = weight_attr,
                    fill = fill,
                    op = alpha,
                    c = content.replace('<', "&lt;").replace('>', "&gt;"),
                ));
            }
            Shape::Image { x, y, w, h, path } => {
                // SVG 嵌入图片引用（实际渲染由上层单独处理）。
                let _ = (x, y, w, h, path);
            }
            Shape::Path {
                segments,
                fill: do_fill,
                thickness,
                stroke_color,
                fill_color,
            } => {
                // Path 图元：拼接 d 属性（坐标 × scale），覆盖色语义取值。
                // 纯描边 fill="none"；纯填充 stroke="none"；round cap/join 统一。
                let d = build_path_d(segments, scale);
                let (fill_attr, stroke_attr, sw_attr) = make_path_paint(
                    *do_fill,
                    *thickness,
                    stroke_color.as_ref(),
                    fill_color.as_ref(),
                    fill.as_str(),
                    *opacity,
                    scale,
                );
                svg.push_str(&format!(
                    r#"<path d="{d}"{f}{s}{sw} stroke-linecap="round" stroke-linejoin="round"/>"#,
                    d = d,
                    f = fill_attr,
                    s = stroke_attr,
                    sw = sw_attr,
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// 拼接 Path 段序列的 SVG `d` 属性（绝对坐标，全部乘 scale）。
fn build_path_d(segments: &[peregrine_config::PathSegment], scale: f32) -> String {
    use std::fmt::Write as _;
    let mut d = String::new();
    for seg in segments {
        match *seg {
            peregrine_config::PathSegment::M { x, y } => {
                let _ = write!(d, "M {} {} ", x * scale, y * scale);
            }
            peregrine_config::PathSegment::L { x, y } => {
                let _ = write!(d, "L {} {} ", x * scale, y * scale);
            }
            peregrine_config::PathSegment::Q { x1, y1, x, y } => {
                let _ = write!(
                    d,
                    "Q {} {} {} {} ",
                    x1 * scale,
                    y1 * scale,
                    x * scale,
                    y * scale
                );
            }
            peregrine_config::PathSegment::C {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let _ = write!(
                    d,
                    "C {} {} {} {} {} {} ",
                    x1 * scale,
                    y1 * scale,
                    x2 * scale,
                    y2 * scale,
                    x * scale,
                    y * scale
                );
            }
            peregrine_config::PathSegment::Z => d.push_str("Z "),
        }
    }
    d.trim_end().to_string()
}

/// 计算路径的 fill / stroke / stroke-width 属性串（元素级颜色覆盖语义）。
///
/// 最终颜色 = `(元素基色 ?? 图层基色) × 图层 opacity`——替换基色但保留
/// 图层不透明度乘法（design D2）。纯描边 `fill="none"`、纯填充 `stroke="none"`。
/// 覆盖色携带自身 alpha 分量，与图层 opacity 相乘后作为 rgba() 的 alpha。
fn make_path_paint(
    fill: bool,
    thickness: f32,
    stroke_color: Option<&[f32; 4]>,
    fill_color: Option<&[f32; 4]>,
    layer_fill: &str,
    layer_opacity: f32,
    scale: f32,
) -> (String, String, String) {
    let paint = |c: &[f32; 4]| -> String {
        let a = (c[3] * layer_opacity).clamp(0.0, 1.0);
        format!(
            "rgba({},{},{},{})",
            (c[0] * 255.0).round().clamp(0.0, 255.0) as u32,
            (c[1] * 255.0).round().clamp(0.0, 255.0) as u32,
            (c[2] * 255.0).round().clamp(0.0, 255.0) as u32,
            a
        )
    };

    let has_stroke = thickness > 0.0;
    let fill_attr = if fill {
        let color = fill_color
            .map(paint)
            .unwrap_or_else(|| layer_fill.to_string());
        format!(r#" fill="{}""#, color)
    } else {
        r#" fill="none""#.to_string()
    };
    let stroke_attr = if has_stroke {
        let color = stroke_color
            .map(paint)
            .unwrap_or_else(|| layer_fill.to_string());
        format!(r#" stroke="{}""#, color)
    } else {
        r#" stroke="none""#.to_string()
    };
    let sw_attr = if has_stroke {
        format!(r#" stroke-width="{}""#, thickness * scale)
    } else {
        String::new()
    };
    (fill_attr, stroke_attr, sw_attr)
}
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
            Shape::Rect {
                x,
                y,
                w,
                h,
                corner_radius,
            } => {
                // 任务 9.4：SVG `<rect>` 支持 rx 属性渲染圆角。
                let rx_attr = match corner_radius {
                    Some(r) if *r > 0.0 => format!(r#" rx="{}""#, *r * scale),
                    _ => String::new(),
                };
                svg.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{w}" height="{h}"{rx} fill="{fill}" opacity="{op}"/>"#,
                    x = *x * scale,
                    y = *y * scale,
                    w = *w * scale,
                    h = *h * scale,
                    rx = rx_attr,
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
                font_weight,
            } => {
                // 字重缺省（None）时不输出 font-weight 属性，按常规 400 渲染。
                let weight_attr = match font_weight {
                    Some(w) => format!(r#" font-weight="{w}""#),
                    None => String::new(),
                };
                svg.push_str(&format!(
                    r#"<text x="{x}" y="{y}" font-size="{fs}"{weight_attr} fill="{fill}" opacity="{op}">{c}</text>"#,
                    x = *x * scale,
                    y = *y * scale,
                    fs = *font_size * scale,
                    weight_attr = weight_attr,
                    fill = fill,
                    op = alpha,
                    c = content.replace('<', "&lt;").replace('>', "&gt;"),
                ));
            }
            Shape::Image { x, y, w, h, path } => {
                // SVG 嵌入图片引用（实际渲染由上层单独处理）。
                let _ = (x, y, w, h, path);
            }
            // 旧格式 build_shapes 不会生成 Path（物料求值仅在新格式路径），
            // 此 arm 仅满足穷尽性；若未来出现则按图层色渲染。
            Shape::Path {
                segments,
                fill: do_fill,
                thickness,
                stroke_color,
                fill_color,
            } => {
                let d = build_path_d(segments, scale);
                let (fill_attr, stroke_attr, sw_attr) = make_path_paint(
                    *do_fill,
                    *thickness,
                    stroke_color.as_ref(),
                    fill_color.as_ref(),
                    &fill,
                    opacity,
                    scale,
                );
                svg.push_str(&format!(
                    r#"<path d="{d}"{f}{s}{sw} stroke-linecap="round" stroke-linejoin="round"/>"#,
                    d = d,
                    f = fill_attr,
                    s = stroke_attr,
                    sw = sw_attr,
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use peregrine_config::{Element, PathSegment};

    fn test_rect() -> RectF {
        RectF {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1920.0,
            max_y: 1080.0,
        }
    }

    fn sample_segments() -> Vec<PathSegment> {
        vec![
            PathSegment::M { x: 10.0, y: 10.0 },
            PathSegment::Q {
                x1: 20.0,
                y1: 0.0,
                x: 30.0,
                y: 10.0,
            },
            PathSegment::Z,
        ]
    }

    /// 纯描边：fill="none"、stroke 取图层色、stroke-width 为 thickness × scale。
    #[test]
    fn path_stroke_only_svg() {
        let svg = build_elements_svg(
            &test_rect(),
            &[(
                Element::Path {
                    segments: sample_segments(),
                    fill: false,
                    thickness: 3.0,
                    stroke_color: None,
                    fill_color: None,
                },
                [0.2, 0.5, 1.0, 1.0],
                0.8,
            )],
            2.0,
        );
        assert!(
            svg.contains(r#"<path d="M 20 20 Q 40 0 60 20 Z""#),
            "svg: {}",
            svg
        );
        assert!(svg.contains(r#" fill="none""#), "svg: {}", svg);
        // 图层色 rgb(51,128,255)。
        assert!(svg.contains(r#" stroke="rgb(51,128,255)""#), "svg: {}", svg);
        assert!(svg.contains(r#" stroke-width="6""#), "svg: {}", svg);
        assert!(svg.contains(r#" stroke-linecap="round""#), "svg: {}", svg);
        assert!(svg.contains(r#" stroke-linejoin="round""#), "svg: {}", svg);
    }

    /// 双色覆盖：fill 与 stroke 分别取两个覆盖色 × 图层 opacity。
    #[test]
    fn path_dual_color_override_svg() {
        let svg = build_elements_svg(
            &test_rect(),
            &[(
                Element::Path {
                    segments: sample_segments(),
                    fill: true,
                    thickness: 2.0,
                    stroke_color: Some([1.0, 0.0, 0.0, 1.0]),
                    fill_color: Some([0.0, 0.0, 1.0, 0.5]),
                },
                [1.0, 1.0, 1.0, 1.0],
                0.5,
            )],
            1.0,
        );
        // 描边覆盖色红 × opacity 0.5 → rgba(255,0,0,0.5)。
        assert!(
            svg.contains(r#" stroke="rgba(255,0,0,0.5)""#),
            "svg: {}",
            svg
        );
        // 填充覆盖色蓝 alpha 0.5 × opacity 0.5 → rgba(0,0,255,0.25)。
        assert!(
            svg.contains(r#" fill="rgba(0,0,255,0.25)""#),
            "svg: {}",
            svg
        );
        assert!(svg.contains(r#" stroke-width="2""#), "svg: {}", svg);
    }

    /// 纯填充：stroke="none"、fill 取覆盖色（无覆盖时取图层色）。
    #[test]
    fn path_fill_only_svg() {
        let svg = build_elements_svg(
            &test_rect(),
            &[(
                Element::Path {
                    segments: sample_segments(),
                    fill: true,
                    thickness: 0.0,
                    stroke_color: None,
                    fill_color: None,
                },
                [0.0, 1.0, 0.0, 1.0],
                1.0,
            )],
            1.0,
        );
        assert!(svg.contains(r#" stroke="none""#), "svg: {}", svg);
        assert!(svg.contains(r#" fill="rgb(0,255,0)""#), "svg: {}", svg);
        // 纯填充不输出 stroke-width。
        assert!(!svg.contains("stroke-width"), "svg: {}", svg);
    }

    /// d 属性拼接：C 段与 L 段的命令格式。
    #[test]
    fn path_d_attribute_commands() {
        let d = build_path_d(
            &[
                PathSegment::M { x: 1.0, y: 2.0 },
                PathSegment::L { x: 3.0, y: 4.0 },
                PathSegment::C {
                    x1: 5.0,
                    y1: 6.0,
                    x2: 7.0,
                    y2: 8.0,
                    x: 9.0,
                    y: 10.0,
                },
                PathSegment::Z,
            ],
            1.0,
        );
        assert_eq!(d, "M 1 2 L 3 4 C 5 6 7 8 9 10 Z");
    }
}
