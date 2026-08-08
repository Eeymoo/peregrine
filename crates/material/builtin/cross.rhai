// Name: 准星
// 对应旧 CrosshairStyle::Cross

fn defaults() {
    #{
        size: 24.0,
        thickness: 2.0,
        gap: 4.0,
    }
}

fn schema() {
    [
        #{key: "size", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let size = params.size;
    let thickness = params.thickness;
    let gap = params.gap;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let hg = gap / 2.0;

    [
        #{type: "rect", x: cx - size, y: cy - thickness / 2.0, w: size - hg, h: thickness},
        #{type: "rect", x: cx + hg, y: cy - thickness / 2.0, w: size - hg, h: thickness},
        #{type: "rect", x: cx - thickness / 2.0, y: cy - size, w: thickness, h: size - hg},
        #{type: "rect", x: cx - thickness / 2.0, y: cy + hg, w: thickness, h: size - hg},
    ]
}
