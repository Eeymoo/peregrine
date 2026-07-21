// Name: 网格
// 对应旧 CrosshairStyle::Grid
// 全屏棋盘式格子，按单格宽度等分屏幕。

fn defaults() {
    #{
        grid_size: 120.0,
        thickness: 2.0,
        alignment: "center",
    }
}

fn schema() {
    [
        #{key: "grid_size", label: "单格宽度", widget: "slider", min: 10.0, max: 500.0, step: 1.0},
        #{key: "thickness", label: "线条粗细", widget: "slider", min: 0.5, max: 10.0, step: 0.5},
        #{
            key: "alignment",
            label: "对齐方式",
            widget: "select",
            options: [
                #{value: "center", label: "居中"},
                #{value: "edge", label: "贴边"},
            ],
            "default": "center",
        },
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let cell = if params.grid_size > 10.0 { params.grid_size } else { 10.0 };
    let thickness = params.thickness;
    let half_t = thickness / 2.0;

    let w = screen.max_x - screen.min_x;
    let h = screen.max_y - screen.min_y;

    let cols = to_int(w / cell + 0.999);
    let rows = to_int(h / cell + 0.999);
    let cols = if cols < 1 { 1 } else { cols };
    let rows = if rows < 1 { 1 } else { rows };

    let elements = [];

    if params.alignment == "edge" {
        let cell_w = w / cols;
        let cell_h = h / rows;
        // 竖线（含左右边缘）
        let i = 0;
        while i <= cols {
            let x = screen.min_x + cell_w * i;
            elements.push(#{
                type: "rect",
                x: x - half_t,
                y: screen.min_y,
                w: thickness,
                h: h,
            });
            i += 1;
        }
        // 横线（含上下边缘）
        let i = 0;
        while i <= rows {
            let y = screen.min_y + cell_h * i;
            elements.push(#{
                type: "rect",
                x: screen.min_x,
                y: y - half_t,
                w: w,
                h: thickness,
            });
            i += 1;
        }
    } else {
        // 居中：正方形格子居中
        let total_w = cell * cols;
        let total_h = cell * rows;
        let offset_x = (w - total_w) / 2.0;
        let offset_y = (h - total_h) / 2.0;

        // 竖线（不含边缘）
        let i = 1;
        while i < cols {
            let x = screen.min_x + offset_x + cell * i;
            elements.push(#{
                type: "rect",
                x: x - half_t,
                y: screen.min_y + offset_y,
                w: thickness,
                h: total_h,
            });
            i += 1;
        }
        // 横线（不含边缘）
        let i = 1;
        while i < rows {
            let y = screen.min_y + offset_y + cell * i;
            elements.push(#{
                type: "rect",
                x: screen.min_x + offset_x,
                y: y - half_t,
                w: total_w,
                h: thickness,
            });
            i += 1;
        }
    }

    elements
}

fn to_int(f) {
    // 把浮点截断为整数（Rhai 内置 int() 函数）。
    int(f)
}
