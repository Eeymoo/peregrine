// Name: 水滴环
// 动态演示物料：以屏幕中心为圆心的圆环锚点，形变由鼠标加速度驱动——
// 加速度方向合成尖头（如水滴朝运动方向甩出的尖端），
// 背侧（反加速度方向）保持正圆弧不参与形变；
// 静止/匀速移动（加速度 ≈ 0）时退化为正圆。
//
// 为什么用加速度而非位置：位置驱动的形变在鼠标停在任何非中心点时永久保持
// 变形；加速度驱动只在「正在变速」时形变，静止/匀速回正（design D7）。
//
// 形变几何（v4，实机反馈修正）：
// - 制动门控：加速度与速度反向（点积 < 0）= 刹车 → 不形变。
//   松手急停会产生与运动方向相反的减速度脉冲，若不过滤，
//   「向左移动的最后一下」会朝右甩尖刺（假动作，实机反馈）。
//   加速与转弯（点积 ≥ 0）才是水滴的正信号。
// - 等宽形变：径向偏移量只算一次（基于外径），内圈叠加同一绝对偏移——
//   环宽在任意方向恒等于 ring_width，形变是「圆管弯折」而非「比例缩放」
//   （v2 内外圈各自按比例缩放，尖端方向环变宽，视觉是铲子不是尖刺）。
// - 尖端连续：尖角点按精确的加速度方向角插入采样序列（在最近两个采样点
//   之间），方向连续旋转时尖端平滑跟随（v2 尖角吸附到最近采样点，
//   24 采样下以 15° 一档跳变——旋转时「不和谐」的根因）。
// - 前侧（与加速度同向）：偏移按 cos³ 锥形增长至尖端；尖端点再乘 1.15
//   略过锥顶形成尖刺，用 L 直连角点（导数不连续 = 尖角）。
// - 背侧（反向半圆）：偏移恒为 0，保持正圆弧。
//
// 环的实现（真实圆环，非多边形填充）：
// - 单条 Path 双圈绕行：外圈 + 内圈（逆序绕行）+ Z 闭合，
//   nonzero 填充规则下内圈以内环绕数相消（镂空）。
// - 采样点用「中点 Q 平滑」连接：以相邻采样点的中点为 Q 段端点、
//   采样点自身为控制点，轮廓为 C1 连续曲线。
//
// 颜色语义：缺省不携带颜色覆盖 = 继承图层基色（乘图层 opacity），
// quick_colors / 换色热键正常生效；「双色覆盖」作为可选参数暴露
// 以演示元素级覆盖机制（显式覆盖 = 主动退出图层色）。
//
// 驱动量仅来自 mouse_acceleration()（DynamicContext 快照），
// 不使用墙钟直读、不依赖 mouse_pos() 位置量。
// 预览端速度/加速度恒为 0，预览呈正圆（spec 已接受行为）。

fn defaults() {
    #{
        radius_pct: 0.05,
        ring_width: 10.0,
        sensitivity: 900.0,
        threshold: 150.0,
        samples: 24,
        tip_scale: 0.8,
        thickness: 0.0,
        use_override_colors: false,
    }
}

fn schema() {
    [
        #{key: "radius_pct", label: "半径占比", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{key: "ring_width", label: "环宽（像素）", widget: "slider", min: 2.0, max: 40.0, step: 1.0},
        #{key: "sensitivity", label: "形变灵敏度", widget: "slider", min: 100.0, max: 3000.0, step: 50.0},
        #{key: "threshold", label: "触发阈值（px/s²）", widget: "slider", min: 0.0, max: 1000.0, step: 10.0},
        #{key: "samples", label: "圆周采样数", widget: "slider", min: 8, max: 48, step: 1},
        #{key: "tip_scale", label: "尖头长度系数", widget: "slider", min: 0.2, max: 2.0, step: 0.1},
        #{key: "thickness", label: "描边粗细（0 = 无描边）", widget: "slider", min: 0.0, max: 12.0, step: 0.5},
        #{key: "use_override_colors", label: "演示颜色覆盖（退出图层色）", widget: "toggle"},
    ]
}

fn is_dynamic() {
    true
}

// 等宽径向偏移量：前侧（与加速度方向同向）cos³ 锥形增长，背侧恒为 0。
// 偏移量基于外径 outer_r 计算，内圈叠加同一绝对值 → 环宽处处恒定。
// strength = 0（含阈值以下）时偏移恒 0（正圆）。
fn teardrop_offset(ux, uy, dir_x, dir_y, strength, outer_r, tip_scale) {
    let align = ux * dir_x + uy * dir_y;
    if align <= 0.0 || strength <= 0.0 {
        0.0
    } else {
        // cos³ 锥形（缓慢起步、快速收尖，水滴轮廓惯例）。
        outer_r * strength * tip_scale * align * align * align
    }
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let radius = (screen.max_y - screen.min_y) * params.radius_pct;
    let inner = radius - params.ring_width;
    let samples = params.samples;
    let tip_scale = params.tip_scale;
    let tau = 3.141592653589793 * 2.0;

    // 加速度驱动量：方向 = 加速度方向。
    // 触发阈值（threshold）：幅度低于阈值视为噪声（手抖/慢移），
    // 精确取 0 = 正圆——配合调度层死区，静止时帧指纹稳定（跳帧）。
    // 阈值以上部分做灵敏度归一 + 上限截断。
    let acc = mouse_acceleration();
    let acc_mag = (acc.x * acc.x + acc.y * acc.y).sqrt();
    let effective = if acc_mag < params.threshold {
        0.0
    } else {
        acc_mag - params.threshold
    };
    // 制动门控（v4）：加速度与速度反向（dot < 0）= 减速/刹车——
    // 松手急停必然产生与运动方向相反的加速度脉冲，若不加过滤，
    // 「向左移动最后一下」会朝右甩一下尖刺（实机反馈的假动作）。
    // 水滴的语义是「运动冲击指示」：加速与转弯才是正信号，制动丢弃。
    // 速度死区归零后速度精确为 (0,0)，点积为 0 → 自然放行（由阈值接管）。
    let vel = mouse_velocity();
    let braking = acc.x * vel.x + acc.y * vel.y < 0.0;
    // 归一化形变系数 ∈ [0, 1]：大加速度封顶。
    // 注意 strength × tip_scale × 1.15 是尖端相对半径的最大伸长率，
    // 缺省 0.8 × 1.15 ≈ 0.92 过长（实机观感「不和谐」）——
    // 整体封顶到 0.5：最大形变时尖端伸长不超过半径的 ~58%，轮廓仍属「环」。
    let strength = if braking {
        0.0
    } else {
        ((effective / params.sensitivity).min(1.0))
            .min(0.5 / params.tip_scale.max(0.1) / 1.15)
    };
    // 加速度方向单位向量（零加速度时取 (0, 0)，形变自然退化为正圆）。
    let dir_x = if acc_mag > 0.0 { acc.x / acc_mag } else { 0.0 };
    let dir_y = if acc_mag > 0.0 { acc.y / acc_mag } else { 0.0 };

    // 圆周采样：外圈/内圈叠加同一偏移（等宽形变）。
    let outer_pts = [];
    let inner_pts = [];
    let i = 0;
    while i < samples {
        let theta = tau * i / samples;
        let ux = theta.cos();
        let uy = theta.sin();
        let off = teardrop_offset(ux, uy, dir_x, dir_y, strength, radius, tip_scale);
        outer_pts.push(#{x: cx + ux * (radius + off), y: cy + uy * (radius + off)});
        inner_pts.push(#{x: cx + ux * (inner + off), y: cy + uy * (inner + off)});
        i += 1;
    }

    // 尖端插入（strength > 0 时）：按精确加速度方向角在最近两个采样点之间
    // 插入尖角点（外圈角点 + 内圈同偏移点），方向连续旋转时尖端平滑跟随
    // （无 15° 量化跳变）。尖端偏移在 cos³ 锥顶（=strength·tip_scale）上
    // 再乘 1.15 略过锥顶，配合 L 直连形成尖刺。
    let corners = [];
    let c = 0;
    while c < samples {
        corners.push(false);
        c += 1;
    }
    if strength > 0.0 {
        // 找与加速度方向点积最大的采样点 k*。
        let k_star = 0;
        let best = -2.0;
        let k = 0;
        while k < samples {
            let theta = tau * k / samples;
            let align = theta.cos() * dir_x + theta.sin() * dir_y;
            if align > best {
                best = align;
                k_star = k;
            }
            k += 1;
        }
        // 加速度方向在 k* 的哪一侧：叉积符号决定插入位置。
        let theta = tau * k_star / samples;
        let cross = theta.cos() * dir_y - theta.sin() * dir_x;
        let insert_at = if cross > 0.0 { k_star + 1 } else { k_star };

        // 尖角点：方向 = (dir_x, dir_y)（精确连续），偏移乘 1.15 过锥顶。
        let tip_off = radius * strength * tip_scale * 1.15;
        let outer_tip = #{
            x: cx + dir_x * (radius + tip_off),
            y: cy + dir_y * (radius + tip_off),
        };
        let inner_tip = #{
            x: cx + dir_x * (inner + tip_off),
            y: cy + dir_y * (inner + tip_off),
        };
        outer_pts.insert(insert_at, outer_tip);
        inner_pts.insert(insert_at, inner_tip);
        corners.insert(insert_at, true);
    }

    // 中点 Q 平滑闭合曲线 + 角点 L 直连。
    // 普通点：端点取相邻采样点中点、控制点取采样点（C1 连续）。
    // 角点：直接 L 到采样点、L 离开（导数不连续 = 尖角）。
    let outer_segs = build_ring(outer_pts, corners);

    // 内圈逆序绕行（nonzero 镂空的关键）；内圈不做尖角（环内缘平滑即可）。
    let inner_flat = [];
    let f = 0;
    while f < inner_pts.len() {
        inner_flat.push(false);
        f += 1;
    }
    let reversed_inner = [];
    let r = inner_pts.len() - 1;
    while r >= 0 {
        reversed_inner.push(inner_pts[r]);
        r -= 1;
    }
    let inner_segs = build_ring(reversed_inner, inner_flat);

    // 单条 Path：外圈 + 内圈（M 开启新子路径）+ Z 闭合。
    let segments = outer_segs;
    segments.push(#{cmd: "M", x: inner_segs[0].x, y: inner_segs[0].y});
    let s = 1;
    while s < inner_segs.len() {
        segments.push(inner_segs[s]);
        s += 1;
    }
    segments.push(#{cmd: "Z"});

    // 颜色：缺省不携带覆盖字段 = 继承图层基色（换色热键生效）；
    // use_override_colors 开启时演示元素级双色覆盖（退出图层色）。
    // fill 恒为 true（环靠镂空填充实现，false 会被转换层拒绝）。
    if params.use_override_colors {
        [
            #{
                type: "path",
                segments: segments,
                fill: true,
                thickness: params.thickness,
                stroke_color: [0.4, 0.8, 1.0, 0.9],
                fill_color: [0.2, 0.6, 1.0, 0.4],
            },
        ]
    } else {
        [
            #{
                type: "path",
                segments: segments,
                fill: true,
                thickness: params.thickness,
            },
        ]
    }
}

// 两点中点（build_ring 的辅助：Rhai 不支持闭包赋值，用顶层具名函数）。
fn mid(a, b) {
    #{x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0}
}

// 构建一圈平滑闭合曲线段序列（中点 Q + 角点 L）。
fn build_ring(pts, corners) {
    let n = pts.len();
    let first = mid(pts[n - 1], pts[0]);
    let segs = [#{cmd: "M", x: first.x, y: first.y}];
    let k = 0;
    while k < n {
        let cur = pts[k];
        let next = pts[(k + 1) % n];
        let next_mid = mid(cur, next);
        if corners[k] {
            // 角点：直线连入连出（与前后中点各一段 L），合成尖角。
            segs.push(#{cmd: "L", x: cur.x, y: cur.y});
            segs.push(#{cmd: "L", x: next_mid.x, y: next_mid.y});
        } else {
            // 平滑点：Q 以 cur 为控制点弯到下一段中点。
            segs.push(#{
                cmd: "Q",
                x1: cur.x,
                y1: cur.y,
                x: next_mid.x,
                y: next_mid.y,
            });
        }
        k += 1;
    }
    segs
}
