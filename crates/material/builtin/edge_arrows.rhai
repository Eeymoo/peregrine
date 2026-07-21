// Name: 箭头
// 对应旧 CrosshairStyle::EdgeArrows
// 屏幕四边中点各一个指向中心的三角形 + 尾巴。
// 所有尺寸按 1080p 短边等比缩放，保证预览与实际 overlay 比例一致。

fn defaults() {
    #{
        size: 16.0,
        distance: 60.0,
        width: 0.0,
        tail_per_edge: false,
        tail_top: 0.0,
        tail_bottom: 0.0,
        tail_left: 0.0,
        tail_right: 0.0,
        positions_mask: 0,
    }
}

fn schema() {
    [
        #{key: "size", label: "箭头大小", widget: "slider", min: 4.0, max: 80.0, step: 1.0},
        #{key: "distance", label: "距边缘距离", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "width", label: "尾巴宽度（0=默认）", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
        #{key: "tail_per_edge", label: "每边单独尾巴长度", widget: "toggle", "default": false},
        #{key: "tail_top", label: "上尾巴长度", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "tail_bottom", label: "下尾巴长度", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "tail_left", label: "左尾巴长度", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "tail_right", label: "右尾巴长度", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "positions_mask", label: "位置掩码（0=全部）", widget: "number", min: 0, max: 15, step: 1},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    const BASE_SHORT = 1080.0;
    let screen_short = if (screen.max_x - screen.min_x) < (screen.max_y - screen.min_y) {
        screen.max_x - screen.min_x
    } else {
        screen.max_y - screen.min_y
    };
    let scale = screen_short / BASE_SHORT;

    let tri_size = if params.size > 4.0 { params.size } else { 4.0 } * scale;
    let tri_half = tri_size * 0.6;

    let default_tail = if params.distance > 0.0 { params.distance } else { 0.0 } * scale;

    let tail_top;
    let tail_bottom;
    let tail_left;
    let tail_right;
    if params.tail_per_edge {
        tail_top = (if params.tail_top > 0.0 { params.tail_top } else { 0.0 }) * scale;
        tail_bottom = (if params.tail_bottom > 0.0 { params.tail_bottom } else { 0.0 }) * scale;
        tail_left = (if params.tail_left > 0.0 { params.tail_left } else { 0.0 }) * scale;
        tail_right = (if params.tail_right > 0.0 { params.tail_right } else { 0.0 }) * scale;
    } else {
        tail_top = default_tail;
        tail_bottom = default_tail;
        tail_left = default_tail;
        tail_right = default_tail;
    }

    let tail_half = if params.width > 0.0 {
        let w = (params.width / 2.0) * scale;
        if w < tri_half { w } else { tri_half }
    } else {
        tri_half
    };

    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    let mask = params.positions_mask;
    let show_top = mask == 0 || (mask & 1) != 0;
    let show_bottom = mask == 0 || (mask & 2) != 0;
    let show_left = mask == 0 || (mask & 4) != 0;
    let show_right = mask == 0 || (mask & 8) != 0;

    let elements = [];

    if show_top {
        let base_y = screen.min_y + tail_top;
        let tip_y = base_y + tri_size;
        if tail_top > 0.0 {
            elements.push(#{
                type: "rect",
                x: cx - tail_half,
                y: screen.min_y,
                w: tail_half * 2.0,
                h: tail_top,
            });
        }
        elements.push(#{
            type: "triangle",
            x1: cx, y1: tip_y,
            x2: cx - tri_half, y2: base_y,
            x3: cx + tri_half, y3: base_y,
        });
    }

    if show_bottom {
        let base_y = screen.max_y - tail_bottom;
        let tip_y = base_y - tri_size;
        if tail_bottom > 0.0 {
            elements.push(#{
                type: "rect",
                x: cx - tail_half,
                y: base_y,
                w: tail_half * 2.0,
                h: tail_bottom,
            });
        }
        elements.push(#{
            type: "triangle",
            x1: cx, y1: tip_y,
            x2: cx - tri_half, y2: base_y,
            x3: cx + tri_half, y3: base_y,
        });
    }

    if show_left {
        let base_x = screen.min_x + tail_left;
        let tip_x = base_x + tri_size;
        if tail_left > 0.0 {
            elements.push(#{
                type: "rect",
                x: screen.min_x,
                y: cy - tail_half,
                w: tail_left,
                h: tail_half * 2.0,
            });
        }
        elements.push(#{
            type: "triangle",
            x1: tip_x, y1: cy,
            x2: base_x, y2: cy - tri_half,
            x3: base_x, y3: cy + tri_half,
        });
    }

    if show_right {
        let base_x = screen.max_x - tail_right;
        let tip_x = base_x - tri_size;
        if tail_right > 0.0 {
            elements.push(#{
                type: "rect",
                x: base_x,
                y: cy - tail_half,
                w: tail_right,
                h: tail_half * 2.0,
            });
        }
        elements.push(#{
            type: "triangle",
            x1: tip_x, y1: cy,
            x2: base_x, y2: cy - tri_half,
            x3: base_x, y3: cy + tri_half,
        });
    }

    elements
}
