// Name: 边框
// 对应旧 CrosshairStyle::BorderFrame
// 围绕屏幕的矩形边框，可选实线或中间留空（避开小地图）。

fn defaults() {
    #{
        thickness: 6.0,
        offset: 24.0,
        frame_style: "solid",
        inset: false,
    }
}

fn schema() {
    [
        #{key: "thickness", label: "粗细", widget: "slider", min: 1.0, max: 30.0, step: 1.0},
        #{key: "offset", label: "距边缘距离", widget: "slider", min: 0.0, max: 200.0, step: 1.0},
        #{
            key: "frame_style",
            label: "样式",
            widget: "select",
            options: [
                #{value: "solid", label: "实线"},
                #{value: "gap", label: "中间留空"},
            ],
            "default": "solid",
        },
        #{key: "inset", label: "位于屏幕内侧", widget: "toggle", "default": false},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let thickness = params.thickness;
    let offset = params.offset;
    let top_y = screen.min_y + offset;
    let bottom_y = screen.max_y - offset;
    let left_x = screen.min_x + offset;
    let right_x = screen.max_x - offset;

    if params.frame_style == "gap" {
        gap_frame(screen, top_y, bottom_y, left_x, right_x, thickness)
    } else {
        solid_frame(screen, top_y, bottom_y, left_x, right_x, thickness)
    }
}

fn solid_frame(screen, top_y, bottom_y, left_x, right_x, thickness) {
    let half_t = thickness / 2.0;
    let w = screen.max_x - screen.min_x;
    let h = screen.max_y - screen.min_y;
    [
        #{type: "rect", x: screen.min_x, y: top_y - half_t, w: w, h: thickness},
        #{type: "rect", x: screen.min_x, y: bottom_y - half_t, w: w, h: thickness},
        #{type: "rect", x: left_x - half_t, y: screen.min_y, w: thickness, h: h},
        #{type: "rect", x: right_x - half_t, y: screen.min_y, w: thickness, h: h},
    ]
}

fn gap_frame(screen, top_y, bottom_y, left_x, right_x, thickness) {
    let half_t = thickness / 2.0;
    let half_gap_w = (screen.max_x - screen.min_x) * 0.2 / 2.0;
    let half_gap_h = (screen.max_y - screen.min_y) * 0.2 / 2.0;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    [
        // 上边（两段）
        #{type: "rect", x: screen.min_x, y: top_y - half_t, w: cx - half_gap_w - screen.min_x, h: thickness},
        #{type: "rect", x: cx + half_gap_w, y: top_y - half_t, w: screen.max_x - (cx + half_gap_w), h: thickness},
        // 下边（两段）
        #{type: "rect", x: screen.min_x, y: bottom_y - half_t, w: cx - half_gap_w - screen.min_x, h: thickness},
        #{type: "rect", x: cx + half_gap_w, y: bottom_y - half_t, w: screen.max_x - (cx + half_gap_w), h: thickness},
        // 左边（两段）
        #{type: "rect", x: left_x - half_t, y: screen.min_y, w: thickness, h: cy - half_gap_h - screen.min_y},
        #{type: "rect", x: left_x - half_t, y: cy + half_gap_h, w: thickness, h: screen.max_y - (cy + half_gap_h)},
        // 右边（两段）
        #{type: "rect", x: right_x - half_t, y: screen.min_y, w: thickness, h: cy - half_gap_h - screen.min_y},
        #{type: "rect", x: right_x - half_t, y: cy + half_gap_h, w: thickness, h: screen.max_y - (cy + half_gap_h)},
    ]
}
