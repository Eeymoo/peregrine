// Name: 星花组合
// 静态锚点物料：左右对称的双锚点组合——
//   1) 星形（尖角锚点，直线段构成）；
//   2) 贝塞尔花（圆润锚点，曲线段构成）。
// 两个锚点分列屏幕中心两侧，为视野提供左右对称的参照标记，
// 适合开阔场景（驾驶/飞行/滑翔）中辅助方向感。
// 纯静态（不调用任何动态输入 host function），静止配置下帧指纹不变，
// 参与稳态跳帧（ControlFlow::Wait 事件驱动渲染）。
//
// 参数面板（玩家语言）：数量 / 大小 / 线宽；形状风格二选一。
// 颜色继承图层基色（换色热键生效）。

fn defaults() {
    #{
        petals: 5,
        outer_pct: 0.06,
        ratio: 0.45,
        thickness: 2.0,
        layout: "both",
    }
}

fn schema() {
    [
        #{
            key: "layout",
            label: "布局",
            widget: "select",
            options: [
                #{value: "both", label: "星 + 花（左右成对）"},
                #{value: "star", label: "仅星形（居中）"},
                #{value: "flower", label: "仅花朵（居中）"},
            ],
        },
        #{key: "petals", label: "角数 / 花瓣数", widget: "slider", min: 3, max: 12, step: 1},
        #{key: "outer_pct", label: "大小", widget: "slider", min: 0.02, max: 0.15, step: 0.005},
        #{key: "ratio", label: "内径比例", widget: "slider", min: 0.2, max: 0.9, step: 0.05},
        #{key: "thickness", label: "线宽", widget: "slider", min: 0.5, max: 12.0, step: 0.5},
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

    // 星形（M/L 直线段 + Z 闭合）：外径/内径顶点交替。
    // both 布局放左半，单独布局居中。
    let star_cx = if params.layout == "both" {
        cx - screen_w * 0.12
    } else {
        cx
    };
    let star = [#{cmd: "M", x: star_cx, y: cy - outer}];
    let i = 1;
    while i < petals * 2 {
        let theta = -tau * i / (petals * 2) + tau * 0.25;
        let r = if i % 2 == 0 { outer } else { inner };
        star.push(#{cmd: "L", x: star_cx + theta.sin() * r, y: cy - theta.cos() * r});
        i += 1;
    }
    star.push(#{cmd: "Z"});

    // 贝塞尔花（C 三次曲线段花瓣）。每片花瓣用一段 C 段从花心出发，
    // 控制点外张形成饱满花瓣，末段回到花心。
    // both 布局放右半，单独布局居中。
    let flower_cx = if params.layout == "both" {
        cx + screen_w * 0.12
    } else {
        cx
    };
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

    // 按布局输出：both = 星 + 花；star / flower = 单锚点居中。
    // 颜色继承图层基色（不携带覆盖字段，换色热键生效）。
    if params.layout == "star" {
        [#{type: "path", segments: star, fill: true, thickness: thickness}]
    } else if params.layout == "flower" {
        [#{type: "path", segments: flower, fill: true, thickness: thickness}]
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
