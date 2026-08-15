// Name: 路径图元演示
// 静态演示物料：同屏输出两组 Path 图元——
//   1) 星形（M/L 直线段 + Z 闭合，展示尖角与直线段语法）；
//   2) 贝塞尔花（C 三次曲线段花瓣，展示曲线段语法）。
// 纯静态（不调用任何动态输入 host function），静止配置下帧指纹不变，
// 参与稳态跳帧（ControlFlow::Wait 事件驱动渲染）。
//
// 颜色覆盖作为可选参数暴露以演示元素级覆盖机制；
// 缺省参数不携带颜色覆盖（继承图层色）。

fn defaults() {
    #{
        petals: 5,
        outer_pct: 0.06,
        ratio: 0.45,
        thickness: 2.0,
        use_override_colors: false,
    }
}

fn schema() {
    [
        #{key: "petals", label: "花瓣/角数", widget: "slider", min: 3, max: 12, step: 1},
        #{key: "outer_pct", label: "外径占比", widget: "slider", min: 0.02, max: 0.15, step: 0.005},
        #{key: "ratio", label: "内径/外径比", widget: "slider", min: 0.2, max: 0.9, step: 0.05},
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.5, max: 12.0, step: 0.5},
        #{key: "use_override_colors", label: "演示颜色覆盖（双色）", widget: "toggle"},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let screen_h = screen.max_y - screen.min_y;
    let screen_w = screen.max_x - screen.min_x;
    let outer = screen_h * params.outer_pct;
    let inner = outer * params.ratio;
    let petals = params.petals;
    let thickness = params.thickness;
    let tau = 3.141592653589793 * 2.0;

    // 左半：星形（M/L 直线段 + Z 闭合）。外径/内径顶点交替。
    let star_cx = cx - screen_w * 0.12;
    let star = [#{cmd: "M", x: star_cx, y: cy - outer}];
    let i = 1;
    while i < petals * 2 {
        let theta = -tau * i / (petals * 2) + tau * 0.25;
        let r = if i % 2 == 0 { outer } else { inner };
        star.push(#{cmd: "L", x: star_cx + theta.sin() * r, y: cy - theta.cos() * r});
        i += 1;
    }
    star.push(#{cmd: "Z"});

    // 右半：贝塞尔花（C 三次曲线段花瓣）。每片花瓣用两段 C 段
    // 从花心出发再回到花心，控制点外张形成饱满花瓣。
    let flower_cx = cx + screen_w * 0.12;
    let flower = [#{cmd: "M", x: flower_cx, y: cy}];
    let j = 0;
    while j < petals {
        // 花瓣中角与两侧控制角。
        let mid_a = tau * j / petals - tau * 0.25;
        let a0 = mid_a - tau / (petals * 2);
        let a1 = mid_a + tau / (petals * 2);
        let tip = flower_outer(outer, mid_a);
        let c0 = flower_ctrl(inner * 1.6, a0);
        let c1 = flower_ctrl(inner * 1.6, a1);
        flower.push(#{
            cmd: "C",
            x1: flower_cx + c0.x,
            y1: cy + c0.y,
            x2: flower_cx + c1.x,
            y2: cy + c1.y,
            x: flower_cx + tip.x,
            y: cy + tip.y,
        });
        j += 1;
    }
    flower.push(#{cmd: "Z"});

    // 颜色覆盖可选演示：开启时星形双色、花朵双色；缺省继承图层色。
    if params.use_override_colors {
        [
            #{
                type: "path",
                segments: star,
                fill: true,
                thickness: thickness,
                fill_color: [1.0, 0.8, 0.2, 0.2],
                stroke_color: [1.0, 0.8, 0.2, 0.9],
            },
            #{
                type: "path",
                segments: flower,
                fill: true,
                thickness: thickness,
                fill_color: [0.6, 0.3, 1.0, 0.2],
                stroke_color: [0.75, 0.5, 1.0, 0.9],
            },
        ]
    } else {
        [
            #{type: "path", segments: star, fill: true, thickness: thickness},
            #{type: "path", segments: flower, fill: true, thickness: thickness},
        ]
    }
}

// 花瓣尖端坐标（相对花心）。
fn flower_outer(outer, angle) {
    #{x: angle.sin() * outer, y: -angle.cos() * outer}
}

// 花瓣控制点坐标（相对花心）。
fn flower_ctrl(ctrl_r, angle) {
    #{x: angle.sin() * ctrl_r, y: -angle.cos() * ctrl_r}
}
