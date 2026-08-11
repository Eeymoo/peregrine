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
        #{key: "grid_size", label: "单格宽度", widget: "slider", min: 10.0, max: 1920.0, step: 1.0},
        #{key: "thickness", label: "线条粗细", widget: "slider", min: 0.5, max: 50.0, step: 0.5},
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

    // 任务 8.1：cols/rows 改用 floor（能完整放下的格子数），而非 ceil。
    // ceil 会多算一格导致最后一格超出屏幕。仅 edge 模式使用；
    // center 模式改为从屏幕中心向两侧对称扩展（half_cols/half_rows），允许超出边缘。
    let cols = to_int(w / cell);
    let rows = to_int(h / cell);
    let cols = if cols < 1 { 1 } else { cols };
    let rows = if rows < 1 { 1 } else { rows };

    let elements = [];

    if params.alignment == "edge" {
        // 任务 8.2：edge 模式 cell_w 直接用 `cell`（用户设定值），不再用 w/cols 重算。
        // 这样 grid_size 真实决定单格宽度。
        let cell_w = cell;
        let cell_h = cell;
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
        // 任务 8.3：center 模式 —— 网格以屏幕中心对称展开，线条允许延伸到屏幕外，
        // 由屏幕作为视窗裁切显示。这样无论 grid_size 能否整除屏幕宽高，
        // 视觉上都铺满屏幕（不会在边缘留下一圈空白）。
        //
        // 旧实现用 floor + total_w = cell * cols，当 cell 不整除 w 时两侧留白，
        // 视觉上"只在中间"。改为从中心向两侧扩展、允许超出边缘一圈。
        let cx = (screen.min_x + screen.max_x) / 2.0;
        let cy = (screen.min_y + screen.max_y) / 2.0;

        // 半边能放下的格子数 +1 圈（保证超出外层、铺满边缘）。
        let half_cols = to_int((w / 2.0) / cell) + 1;
        let half_rows = to_int((h / 2.0) / cell) + 1;

        // 竖线：从中心向两侧扩展，每条线高度铺满屏幕。
        let i = 0;
        while i <= half_cols {
            // 右侧
            let x_r = cx + cell * i;
            elements.push(#{
                type: "rect",
                x: x_r - half_t,
                y: screen.min_y,
                w: thickness,
                h: h,
            });
            // 左侧（跳过 i==0 避免中心线重复）
            if i > 0 {
                let x_l = cx - cell * i;
                elements.push(#{
                    type: "rect",
                    x: x_l - half_t,
                    y: screen.min_y,
                    w: thickness,
                    h: h,
                });
            }
            i += 1;
        }
        // 横线：同理
        let i = 0;
        while i <= half_rows {
            let y_b = cy + cell * i;
            elements.push(#{
                type: "rect",
                x: screen.min_x,
                y: y_b - half_t,
                w: w,
                h: thickness,
            });
            if i > 0 {
                let y_t = cy - cell * i;
                elements.push(#{
                    type: "rect",
                    x: screen.min_x,
                    y: y_t - half_t,
                    w: w,
                    h: thickness,
                });
            }
            i += 1;
        }
    }

    elements
}

fn to_int(f) {
    // 把浮点截断为整数（Rhai 内置 int() 函数）。
    int(f)
}
