// Name: 水滴环
// 动态演示物料：以屏幕中心为圆心的圆环锚点，形变由鼠标加速度驱动——
// 加速度方向合成尖头（如水滴朝运动方向甩出的尖端），
// 背侧（反加速度方向）保持正圆弧不参与形变；
// 静止/匀速移动（加速度 ≈ 0）时退化为正圆。
//
// 为什么用加速度而非位置：位置驱动的形变在鼠标停在任何非中心点时永久保持
// 变形；加速度驱动只在「正在变速」时形变，静止/匀速回正（design D7）。
//
// 形变几何（v2，实机反馈修正）：
// - 前侧（与加速度同向的半圆）：采样点半径按 cos³ 锥形增长至尖端，
//   末端用一段 L 直线段直接连出尖角（Q 曲线在尖点处导数不连续，
//   恰好合成「尖头突出」的视觉）；尖头长度 ∝ 加速度幅度。
// - 背侧（反向半圆）：半径保持 base，完全不形变（底部圆的正圆弧）。
//
// 环的实现（真实圆环，非多边形填充）：
// - 单条 Path 双圈绕行：外圈 + 内圈（逆序绕行）+ Z 闭合，
//   nonzero 填充规则下内圈以内环绕数相消（镂空）。
// - 采样点用「中点 Q 平滑」连接：以相邻采样点的中点为 Q 段端点、
//   采样点自身为控制点，轮廓为 C1 连续曲线。
//
// 颜色语义（v2 修正）：缺省不携带颜色覆盖 = 继承图层基色（乘图层 opacity），
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

// 水滴半径场：前侧（与加速度方向同向）cos³ 锥形增长，背侧保持 base。
// ux/uy 为圆周方向单位向量，align = 与加速度方向的点积 ∈ [-1, 1]。
// strength = 0（含阈值以下）时所有点 tip=false（正圆，无角点）。
fn teardrop_radius(ux, uy, dir_x, dir_y, strength, base, tip_scale) {
    let align = ux * dir_x + uy * dir_y;
    if align <= 0.0 || strength <= 0.0 {
        // 背侧或无形变强度：正圆弧，不形变。
        #{r: base, tip: false}
    } else {
        // 前侧：cos³ 锥形（缓慢起步、快速收尖，水滴轮廓惯例）。
        let t = align * align * align;
        #{r: base * (1.0 + strength * tip_scale * t), tip: true}
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
    // 归一化形变系数 ∈ [0, 1]：大加速度封顶。
    let strength = (effective / params.sensitivity).min(1.0);
    // 加速度方向单位向量（零加速度时取 (0, 0)，形变自然退化为正圆）。
    let dir_x = if acc_mag > 0.0 { acc.x / acc_mag } else { 0.0 };
    let dir_y = if acc_mag > 0.0 { acc.y / acc_mag } else { 0.0 };

    // 圆周采样：单圈闭合序列，采样后按需把「正对加速度方向」的采样点
    // 提升为尖端角点。内圈共用同一形变场（同心形变，环宽大致均匀）。
    let outer_pts = [];
    let inner_pts = [];
    let tip_flag = [];
    let i = 0;
    while i < samples {
        let theta = tau * i / samples;
        let ux = theta.cos();
        let uy = theta.sin();
        let outer = teardrop_radius(ux, uy, dir_x, dir_y, strength, radius, tip_scale);
        let inner = teardrop_radius(ux, uy, dir_x, dir_y, strength, inner, tip_scale);
        outer_pts.push(#{x: cx + ux * outer.r, y: cy + uy * outer.r});
        inner_pts.push(#{x: cx + ux * inner.r, y: cy + uy * inner.r});
        tip_flag.push(outer.tip);
        i += 1;
    }

    // 尖头合成：找到 align 最大（正对加速度方向）的采样点 k*，
    // 其半径提升为尖点（base × (1 + strength × tip_scale × 1.15)，略过锥顶），
    // 标记为角点——构建 Q 序列时角点不取中点、直接 L 连接，
    // 曲线在角点两侧导数不连续 = 视觉尖角。
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
    if strength > 0.0 {
        let theta = tau * k_star / samples;
        let ux = theta.cos();
        let uy = theta.sin();
        let tip_r = radius * (1.0 + strength * tip_scale * 1.15);
        outer_pts[k_star] = #{x: cx + ux * tip_r, y: cy + uy * tip_r};
        let tip_ri = inner * (1.0 + strength * tip_scale * 1.15);
        inner_pts[k_star] = #{x: cx + ux * tip_ri, y: cy + uy * tip_ri};
        tip_flag[k_star] = true;
    }

    // 中点 Q 平滑闭合曲线 + 角点 L 直连。
    // 普通点：端点取相邻采样点中点、控制点取采样点（C1 连续）。
    // 角点：前一段直接 L 到采样点、后一段 L 离开（导数不连续 = 尖角）。
    let outer_segs = build_ring(outer_pts, tip_flag);

    // 内圈逆序绕行（nonzero 镂空的关键）；内圈不做尖角（环内缘平滑即可）。
    let inner_flat = [];
    let f = 0;
    while f < samples {
        inner_flat.push(false);
        f += 1;
    }
    let reversed_inner = [];
    let r = samples - 1;
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
