// Name: 自定义定位球
// 对应旧 CrosshairStyle::CustomOrb
// 在屏幕上/下/左/右边缘均匀分布的圆形锚点。
// 通过 orb_positions 位掩码控制（位 1=top, 2=bottom, 4=left, 8=right）。

fn defaults() {
    #{
        radius: 6.0,
        offset: 30.0,
        orb_positions: 3,
        top_count: 3,
        bottom_count: 3,
        left_count: 3,
        right_count: 3,
    }
}

fn schema() {
    [
        #{key: "radius", label: "半径", widget: "slider", min: 1.0, max: 30.0, step: 0.5},
        #{key: "offset", label: "距边缘距离", widget: "slider", min: 0.0, max: 200.0, step: 1.0},
        #{key: "orb_positions", label: "边缘位掩码", widget: "number", min: 0, max: 15, step: 1},
        #{key: "top_count", label: "上边球数", widget: "slider", min: 0, max: 20, step: 1},
        #{key: "bottom_count", label: "下边球数", widget: "slider", min: 0, max: 20, step: 1},
        #{key: "left_count", label: "左边球数", widget: "slider", min: 0, max: 20, step: 1},
        #{key: "right_count", label: "右边球数", widget: "slider", min: 0, max: 20, step: 1},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let radius = if params.radius >= 1.0 { params.radius } else { 1.0 };
    let offset = params.offset;
    let pos = params.orb_positions;
    let elements = [];

    if (pos & 1) != 0 {
        elements += edge_orbs(screen.min_x, screen.min_y + offset, screen.max_x, screen.min_y + offset, params.top_count, radius);
    }
    if (pos & 2) != 0 {
        elements += edge_orbs(screen.min_x, screen.max_y - offset, screen.max_x, screen.max_y - offset, params.bottom_count, radius);
    }
    if (pos & 4) != 0 {
        elements += edge_orbs(screen.min_x + offset, screen.min_y, screen.min_x + offset, screen.max_y, params.left_count, radius);
    }
    if (pos & 8) != 0 {
        elements += edge_orbs(screen.max_x - offset, screen.min_y, screen.max_x - offset, screen.max_y, params.right_count, radius);
    }

    elements
}

fn edge_orbs(x0, y0, x1, y1, count, radius) {
    if count == 0 { return []; }
    if count == 1 {
        return [#{
            type: "circle",
            cx: (x0 + x1) / 2.0,
            cy: (y0 + y1) / 2.0,
            radius: radius,
        }];
    }
    let step = 1.0 / (count + 1);
    let out = [];
    let i = 1;
    while i <= count {
        let t = i * step;
        out.push(#{
            type: "circle",
            cx: x0 + (x1 - x0) * t,
            cy: y0 + (y1 - y0) * t,
            radius: radius,
        });
        i += 1;
    }
    out
}
