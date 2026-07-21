// Name: 定位球
// 对应旧 CrosshairStyle::CornerDots4/6/8（通过 count 参数区分）
// 屏幕四角各一个圆形，可选附带中点。

fn defaults() {
    #{
        count: 4,
        offset: 40.0,
        thickness: 3.0,
        radius: 0.0,
    }
}

fn schema() {
    [
        #{
            key: "count",
            label: "点数",
            widget: "select",
            options: [
                #{value: 4, label: "4 点（四角）"},
                #{value: 6, label: "6 点（四角+上下中）"},
                #{value: 8, label: "8 点（四角+四边中）"},
            ],
            "default": 4,
        },
        #{key: "offset", label: "距边缘距离", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "thickness", label: "粗细（决定默认半径）", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "radius", label: "半径（0 = 自动）", widget: "slider", min: 0.0, max: 60.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let configured_offset = if params.offset > 0.0 { params.offset } else { 100.0 };
    let offset = min3(configured_offset, (screen.max_x - screen.min_x) / 4.0, (screen.max_y - screen.min_y) / 4.0);

    let radius = if params.radius > 0.0 { params.radius } else { params.thickness * 3.0 };

    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    let elements = [];

    // 四角
    elements.push(#{
        type: "circle",
        cx: screen.min_x + offset,
        cy: screen.min_y + offset,
        radius: radius,
    });
    elements.push(#{
        type: "circle",
        cx: screen.max_x - offset,
        cy: screen.min_y + offset,
        radius: radius,
    });
    elements.push(#{
        type: "circle",
        cx: screen.min_x + offset,
        cy: screen.max_y - offset,
        radius: radius,
    });
    elements.push(#{
        type: "circle",
        cx: screen.max_x - offset,
        cy: screen.max_y - offset,
        radius: radius,
    });

    if params.count >= 6 {
        elements.push(#{
            type: "circle",
            cx: cx,
            cy: screen.min_y + offset,
            radius: radius,
        });
        elements.push(#{
            type: "circle",
            cx: cx,
            cy: screen.max_y - offset,
            radius: radius,
        });
    }

    if params.count >= 8 {
        elements.push(#{
            type: "circle",
            cx: screen.min_x + offset,
            cy: cy,
            radius: radius,
        });
        elements.push(#{
            type: "circle",
            cx: screen.max_x - offset,
            cy: cy,
            radius: radius,
        });
    }

    elements
}

fn min3(a, b, c) {
    if a < b {
        if a < c { a } else { c }
    } else {
        if b < c { b } else { c }
    }
}
