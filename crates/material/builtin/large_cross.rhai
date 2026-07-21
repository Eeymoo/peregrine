// Name: 大准星
// 对应旧 CrosshairStyle::LargeCross
// 从屏幕边缘延伸到中心的水平线与垂直线。

fn defaults() {
    #{
        thickness: 2.0,
    }
}

fn schema() {
    [
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let thickness = params.thickness;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    [
        #{type: "rect", x: screen.min_x, y: cy - thickness / 2.0, w: screen.max_x - screen.min_x, h: thickness},
        #{type: "rect", x: cx - thickness / 2.0, y: screen.min_y, w: thickness, h: screen.max_y - screen.min_y},
    ]
}
