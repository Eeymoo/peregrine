// Name: 路径
// 静态锚点物料：用户输入 SVG path 数据（d 字符串），自动解析为
// 路径图元渲染。适合从设计工具（Figma/Illustrator/Inkscape）复制
// 锚点形状——把导出的 path d 值贴进参数即可。
//
// 解析由 host function parse_svg_path() 提供（Rust 侧实现）：
// - 支持 M/L/Q/C/Z（绝对）与 m/l/q/c/z（相对，自动换算绝对坐标）；
// - H/V/h/v 展开为等价 L；
// - 隐式重复（"M 1 2 3 4" = M + L）与逗号/负号分隔符；
// - A/S/T 不支持（返回空数组，转换层对空 segments 报错可见）。
//
// 归一化：以 d 字符串自身的包围盒为基准缩放 + 居中——用户贴入的
// 任何坐标域（0..24 图标网格、0..512 设计稿）都能正确铺满目标
// 大小，无需手动换算。
//
// 内置默认形状：正圆（四个 90° C 段构成）——最中性的锚点，开箱即用。
// 纯静态（无动态输入），静止配置下帧指纹不变（稳态跳帧）。
//
// 颜色继承图层基色（换色热键生效）。

fn defaults() {
    #{
        // 默认形状：正圆（圆心 50,50 半径 45，四个 90° C 段，
        // 控制点切向长度 r×0.5523 为圆弧标准贝塞尔近似）。
        // 用户可贴入任意 SVG path d 值替换。
        d: "M 50 5 C 74.85 5 95 25.15 95 50 C 95 74.85 74.85 95 50 95 C 25.15 95 5 74.85 5 50 C 5 25.15 25.15 5 50 5 Z",
        // 目标大小（逻辑像素，短边基准）。
        size_pct: 0.08,
        // 填充（false = 仅描边）。
        fill: true,
        thickness: 2.0,
        // 线帽/转角圆润度由 SVG 后端默认处理。
    }
}

fn schema() {
    [
        #{
            key: "d",
            label: "路径数据（SVG path d，如从设计工具复制）",
            widget: "text",
        },
        #{key: "size_pct", label: "大小", widget: "slider", min: 0.01, max: 0.3, step: 0.005},
        #{key: "fill", label: "填充（关闭则仅描边）", widget: "toggle"},
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.5, max: 12.0, step: 0.5},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let segs_in = parse_svg_path(params.d);
    if segs_in.len() == 0 {
        // 解析失败（A/S/T 或语法错误）：回退默认形状，保证物料
        // 永不渲染空输出（图层突然消失比回退更迷惑）。
        return build_fallback(screen, params);
    }

    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let target = (screen.max_y - screen.min_y) * params.size_pct;

    // ===== 包围盒归一化 =====
    // 找 d 字符串所有坐标的 min/max（Q/C 控制点参与保守估计——
    // 控制点在包围盒外的曲线会略微溢出，锚点用途可接受）。
    let bx1 = 1e9;
    let by1 = 1e9;
    let bx2 = -1e9;
    let by2 = -1e9;
    let k = 0;
    while k < segs_in.len() {
        let s = segs_in[k];
        if s.cmd == "M" || s.cmd == "L" {
            bx1 = if s.x < bx1 { s.x } else { bx1 };
            by1 = if s.y < by1 { s.y } else { by1 };
            bx2 = if s.x > bx2 { s.x } else { bx2 };
            by2 = if s.y > by2 { s.y } else { by2 };
        } else if s.cmd == "Q" {
            bx1 = if s.x1 < bx1 { s.x1 } else { bx1 };
            by1 = if s.y1 < by1 { s.y1 } else { by1 };
            bx2 = if s.x1 > bx2 { s.x1 } else { bx2 };
            by2 = if s.y1 > by2 { s.y1 } else { by2 };
            bx1 = if s.x < bx1 { s.x } else { bx1 };
            by1 = if s.y < by1 { s.y } else { by1 };
            bx2 = if s.x > bx2 { s.x } else { bx2 };
            by2 = if s.y > by2 { s.y } else { by2 };
        } else if s.cmd == "C" {
            bx1 = if s.x1 < bx1 { s.x1 } else { bx1 };
            by1 = if s.y1 < by1 { s.y1 } else { by1 };
            bx2 = if s.x1 > bx2 { s.x1 } else { bx2 };
            by2 = if s.y1 > by2 { s.y1 } else { by2 };
            bx1 = if s.x2 < bx1 { s.x2 } else { bx1 };
            by1 = if s.y2 < by1 { s.y2 } else { by1 };
            bx2 = if s.x2 > bx2 { s.x2 } else { bx2 };
            by2 = if s.y2 > by2 { s.y2 } else { by2 };
            bx1 = if s.x < bx1 { s.x } else { bx1 };
            by1 = if s.y < by1 { s.y } else { by1 };
            bx2 = if s.x > bx2 { s.x } else { bx2 };
            by2 = if s.y > by2 { s.y } else { by2 };
        }
        k += 1;
    }

    // 退化保护：零宽/零高（单点或直线）时用 1 避免除零。
    let bw = if bx2 - bx1 < 0.001 { 1.0 } else { bx2 - bx1 };
    let bh = if by2 - by1 < 0.001 { 1.0 } else { by2 - by1 };
    // 等比缩放（取短边基准铺满 target），保持宽高比。
    let scale = target / (if bw > bh { bw } else { bh });
    // 包围盒中心映射到屏幕中心。
    let bcx = (bx1 + bx2) / 2.0;
    let bcy = (by1 + by2) / 2.0;

    // ===== 坐标变换 + 重建段数组 =====
    let segs_out = [];
    let j = 0;
    while j < segs_in.len() {
        let s = segs_in[j];
        if s.cmd == "M" {
            segs_out.push(#{
                cmd: "M",
                x: cx + (s.x - bcx) * scale,
                y: cy + (s.y - bcy) * scale,
            });
        } else if s.cmd == "L" {
            segs_out.push(#{
                cmd: "L",
                x: cx + (s.x - bcx) * scale,
                y: cy + (s.y - bcy) * scale,
            });
        } else if s.cmd == "Q" {
            segs_out.push(#{
                cmd: "Q",
                x1: cx + (s.x1 - bcx) * scale,
                y1: cy + (s.y1 - bcy) * scale,
                x: cx + (s.x - bcx) * scale,
                y: cy + (s.y - bcy) * scale,
            });
        } else if s.cmd == "C" {
            segs_out.push(#{
                cmd: "C",
                x1: cx + (s.x1 - bcx) * scale,
                y1: cy + (s.y1 - bcy) * scale,
                x2: cx + (s.x2 - bcx) * scale,
                y2: cy + (s.y2 - bcy) * scale,
                x: cx + (s.x - bcx) * scale,
                y: cy + (s.y - bcy) * scale,
            });
        } else {
            segs_out.push(#{cmd: "Z"});
        }
        j += 1;
    }

    [
        #{
            type: "path",
            segments: segs_out,
            fill: params.fill,
            thickness: params.thickness,
        },
    ]
}

// 解析失败回退：程序化正圆（与 defaults.d 相同形状——四个 90°
// C 段，切向控制点长度 k = r × 0.5523 为 90° 圆弧的标准贝塞尔近似）。
fn build_fallback(screen, params) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let r = (screen.max_y - screen.min_y) * params.size_pct / 2.0;
    let k = r * 0.5523;

    // 正圆：从顶部起四个 90° C 段（右上/右下/左下/左上象限）。
    let segs = [
        #{cmd: "M", x: cx, y: cy - r},
        #{cmd: "C", x1: cx + k, y1: cy - r, x2: cx + r, y2: cy - k, x: cx + r, y: cy},
        #{cmd: "C", x1: cx + r, y1: cy + k, x2: cx + k, y2: cy + r, x: cx, y: cy + r},
        #{cmd: "C", x1: cx - k, y1: cy + r, x2: cx - r, y2: cy + k, x: cx - r, y: cy},
        #{cmd: "C", x1: cx - r, y1: cy - k, x2: cx - k, y2: cy - r, x: cx, y: cy - r},
        #{cmd: "Z"},
    ];
    [
        #{
            type: "path",
            segments: segs,
            fill: params.fill,
            thickness: params.thickness,
        },
    ]
}
