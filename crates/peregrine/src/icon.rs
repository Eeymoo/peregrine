//! 占位图标：蓝色圆角背景 + 白色 "P"，四周留透明边距。
//!
//! 目前同时用于状态栏（托盘）图标与窗口图标，属于**临时占位素材**，
//! 后续会替换为正式设计的图标。图标像素在运行时按需生成，避免引入图片解码依赖。

/// "P" 字形位图（5 列 × 7 行），值为 1 的单元格绘制成白色。
const GLYPH_P: [[u8; 5]; 7] = [
    [1, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [1, 1, 1, 1, 0],
    [1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0],
];

/// 蓝色背景颜色（RGB）。
const BLUE: [u8; 3] = [37, 99, 235];

/// 生成占位图标的 RGBA 像素数据。
///
/// 返回 `(rgba, width, height)`，`rgba` 长度为 `size * size * 4`。
/// 布局：整幅图默认透明，中间绘制蓝色圆角背景块，其上叠加白色 "P"。
pub fn placeholder_rgba(size: u32) -> (Vec<u8>, u32, u32) {
    let size = size.max(16);
    let n = size as usize;
    let mut rgba = vec![0u8; n * n * 4];

    // 蓝色背景块：四周留出约 1/10 的透明边距。
    let margin = (size as f32 * 0.1).round();
    let sq_min = margin;
    let sq_size = (size as f32 - margin * 2.0).max(1.0);
    // 圆角半径，用于把背景块画成圆角矩形。
    let corner = sq_size * 0.22;

    // "P" 字形区域：在背景块内再留一层内边距。
    let pad = sq_size * 0.18;
    let glyph_min_x = sq_min + pad;
    let glyph_min_y = sq_min + pad;
    let glyph_w = sq_size - pad * 2.0;
    let glyph_h = sq_size - pad * 2.0;
    let cell_w = glyph_w / 5.0;
    let cell_h = glyph_h / 7.0;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y as usize) * n + x as usize) * 4;
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            // 仅在圆角矩形背景块内绘制，其余保持透明。
            if !inside_rounded_rect(fx, fy, sq_min, sq_min, sq_size, sq_size, corner) {
                continue;
            }

            // 默认蓝色背景。
            let mut color = [BLUE[0], BLUE[1], BLUE[2], 255u8];

            // 命中 "P" 字形单元格则改为白色。
            let gx = fx - glyph_min_x;
            let gy = fy - glyph_min_y;
            if gx >= 0.0 && gy >= 0.0 && gx < glyph_w && gy < glyph_h {
                let col = (gx / cell_w) as usize;
                let row = (gy / cell_h) as usize;
                if row < 7 && col < 5 && GLYPH_P[row][col] == 1 {
                    color = [255, 255, 255, 255];
                }
            }

            rgba[idx] = color[0];
            rgba[idx + 1] = color[1];
            rgba[idx + 2] = color[2];
            rgba[idx + 3] = color[3];
        }
    }

    (rgba, size, size)
}

/// 判断点 (px, py) 是否落在左上角为 (x, y)、尺寸 w×h、圆角半径 r 的圆角矩形内。
fn inside_rounded_rect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, r: f32) -> bool {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    if px < x || py < y || px > x + w || py > y + h {
        return false;
    }
    // 把点向内收缩到"直边核心矩形"，再用到该核心的距离判断是否在圆角内。
    let cx = px.clamp(x + r, x + w - r);
    let cy = py.clamp(y + r, y + h - r);
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}

/// 构造状态栏（托盘）图标（32×32）。
pub fn tray_icon() -> tray_icon::Icon {
    let (rgba, w, h) = placeholder_rgba(32);
    tray_icon::Icon::from_rgba(rgba, w, h).expect("build tray icon from rgba")
}

/// 构造窗口图标（64×64），作为窗口标题栏与任务栏图标生效。
pub fn window_icon() -> winit::window::Icon {
    let (rgba, w, h) = placeholder_rgba(64);
    winit::window::Icon::from_rgba(rgba, w, h).expect("build window icon from rgba")
}
