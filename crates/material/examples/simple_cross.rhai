// Name: 示例·简易十字
// 创作流程：① 选图元 = 4 段矩形；② 定布局 = 屏幕中心向外辐射；
// ③ 抽参数 = arm / thickness / gap；④ 声明 defaults/schema；⑤ 验证（本文件由 cargo test 加载）。
//
// 静态物料示例：由四段矩形组成的十字准心，可调臂长、粗细、中心间隙。
// 参数：
//   arm_length — 单臂长度（像素）
//   thickness  — 矩形粗细
//   gap        — 中心透明间隙

fn defaults() {
    #{
        arm_length: 20.0,
        thickness: 3.0,
        gap: 4.0,
    }
}

fn schema() {
    [
        #{key: "arm_length", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let arm = params.arm_length;
    let t = params.thickness;
    let g = params.gap / 2.0;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    [
        #{type: "rect", x: cx - arm, y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx + g,     y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx - t / 2.0, y: cy - arm,     w: t, h: arm - g},
        #{type: "rect", x: cx - t / 2.0, y: cy + g,       w: t, h: arm - g},
    ]
}
