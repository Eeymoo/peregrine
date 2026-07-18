// Name: 中心环
// 对应旧 CrosshairStyle::Ring
// 以屏幕中心为圆心的圆环，提供同心圆锚点。
// 支持三种线型：solid（实线）/ dashed（虚线）/ double（双环）。

fn defaults() {
    #{
        thickness: 2.0,
        ring_radius_pct: 0.06,
        ring_style: "solid",
    }
}

fn schema() {
    [
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "ring_radius_pct", label: "半径占比", widget: "slider", min: 0.03, max: 0.08, step: 0.005},
        #{
            key: "ring_style",
            label: "线型",
            widget: "select",
            options: [
                #{value: "solid", label: "实线"},
                #{value: "dashed", label: "虚线"},
                #{value: "double", label: "双环"},
            ],
            "default": "solid",
        },
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let radius = (screen.max_y - screen.min_y) * params.ring_radius_pct;
    let thickness = params.thickness;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    if params.ring_style == "dashed" {
        [
            #{
                type: "dashed_circle",
                cx: cx,
                cy: cy,
                radius: radius,
                thickness: thickness,
                dash_len: 4.0,
                gap_len: 4.0,
            },
        ]
    } else if params.ring_style == "double" {
        [
            #{
                type: "circle_stroke",
                cx: cx,
                cy: cy,
                radius: radius - 2.0,
                thickness: 1.0,
            },
            #{
                type: "dashed_circle",
                cx: cx,
                cy: cy,
                radius: radius + 2.0,
                thickness: 1.0,
                dash_len: 4.0,
                gap_len: 4.0,
            },
        ]
    } else {
        // solid
        [
            #{
                type: "circle_stroke",
                cx: cx,
                cy: cy,
                radius: radius,
                thickness: thickness,
            },
        ]
    }
}
