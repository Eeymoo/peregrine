// Name: 贴边矩形
// 对应旧 CrosshairStyle::EdgeRect
// 屏幕顶/底/左/右/中心位置的半透明矩形。

fn defaults() {
    #{
        size: 180.0,
        secondary_size: 24.0,
        anchor: "top",
        margin: 16.0,
        corner_radius: 12.0,
    }
}

fn schema() {
    [
        #{key: "size", label: "宽度", widget: "slider", min: 10.0, max: 600.0, step: 1.0},
        #{key: "secondary_size", label: "高度", widget: "slider", min: 4.0, max: 200.0, step: 1.0},
        #{
            key: "anchor",
            label: "位置",
            widget: "select",
            options: [
                #{value: "top", label: "顶部"},
                #{value: "bottom", label: "底部"},
                #{value: "left", label: "左侧"},
                #{value: "right", label: "右侧"},
                #{value: "center", label: "中心"},
            ],
            "default": "top",
        },
        #{key: "margin", label: "边距", widget: "slider", min: 0.0, max: 200.0, step: 1.0},
        #{key: "corner_radius", label: "圆角半径", widget: "slider", min: 0.0, max: 60.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let w = params.size;
    let h = params.secondary_size;
    let margin = params.margin;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    let px;
    let py;
    if params.anchor == "bottom" {
        px = cx;
        py = screen.max_y - h / 2.0 - margin;
    } else if params.anchor == "left" {
        px = screen.min_x + w / 2.0 + margin;
        py = cy;
    } else if params.anchor == "right" {
        px = screen.max_x - w / 2.0 - margin;
        py = cy;
    } else if params.anchor == "center" {
        px = cx;
        py = cy;
    } else {
        // top（默认）
        px = cx;
        py = screen.min_y + h / 2.0 + margin;
    }

    [
        #{type: "rect", x: px - w / 2.0, y: py - h / 2.0, w: w, h: h},
    ]
}
