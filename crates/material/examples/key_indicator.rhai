// Name: 示例·按键指示器
// 创作流程：① 选图元 = circle + circle_stroke；② 定布局 = 屏幕中心圆 + 按下时外圈；
// ③ 抽参数 = radius / key_code / ring_size；④ 声明 defaults/schema + is_dynamic=true；
// ⑤ 验证（本文件由 cargo test 加载，动态输入软关闭下渲染为静态快照）。
//
// 输入动态物料示例：屏幕中心常驻小圆点，按下指定键时外圈亮起。
// 动态输入：key_down(code)。
// 注意：当前发布构建默认软关闭动态输入，key_down 始终返回 false，外圈永不显示；
//       在构建启用动态输入后会响应真实按键。
// 参数：
//   radius     — 中心点半径
//   key_code   — 监听的按键代码（如 "space" / "shift" / "a"）
//   ring_size  — 按下时外圈半径

fn defaults() {
    #{
        radius: 4.0,
        key_code: "space",
        ring_size: 24.0,
    }
}

fn schema() {
    [
        #{key: "radius", label: "中心点半径", widget: "slider", min: 1.0, max: 40.0, step: 0.5},
        #{key: "key_code", label: "按键代码", widget: "text"},
        #{key: "ring_size", label: "按下时外圈半径", widget: "slider", min: 4.0, max: 200.0, step: 1.0},
    ]
}

fn is_dynamic() {
    true
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    let elements = [
        #{type: "circle", cx: cx, cy: cy, radius: params.radius},
    ];

    if key_down(params.key_code) {
        elements.push(#{
            type: "circle_stroke",
            cx: cx,
            cy: cy,
            radius: params.ring_size,
            thickness: 2.0,
        });
    }

    elements
}
