// Name: 水滴环
// 动态演示物料：以屏幕中心为圆心的圆环锚点，重心偏移由鼠标加速度驱动。
// 偏移方向 = 加速度方向（急转弯时水滴朝转弯方向甩出），
// 形变强度 ∝ 加速度幅度；静止/匀速移动（加速度 ≈ 0）时退化为正圆
// ——这正是「3D 晕眩多发生在视角急转」场景下锚点应有的行为。
//
// 为什么用加速度而非位置：位置驱动的形变在鼠标停在任何非中心点时永久保持
// 变形；加速度驱动只在「正在变速」时形变，静止/匀速回正（design D7）。
//
// 驱动量仅来自 mouse_acceleration()（DynamicContext 快照），
// 不使用墙钟直读、不依赖 mouse_pos() 位置量。
// 预览端速度/加速度恒为 0，预览呈正圆（spec 已接受行为）。

fn defaults() {
    #{
        radius_pct: 0.05,
        sensitivity: 900.0,
        samples: 12,
        thickness: 2.0,
    }
}

fn schema() {
    [
        #{key: "radius_pct", label: "半径占比", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{key: "sensitivity", label: "形变灵敏度", widget: "slider", min: 100.0, max: 3000.0, step: 50.0},
        #{key: "samples", label: "圆周采样数", widget: "slider", min: 6, max: 32, step: 1},
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.5, max: 12.0, step: 0.5},
    ]
}

fn is_dynamic() {
    true
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let radius = (screen.max_y - screen.min_y) * params.radius_pct;
    let samples = params.samples;
    let thickness = params.thickness;

    // 加速度驱动量：方向 = 加速度方向，幅度做灵敏度归一 + 上限截断。
    let acc = mouse_acceleration();
    let acc_mag = (acc.x * acc.x + acc.y * acc.y).sqrt();
    // 归一化形变系数 ∈ [0, 0.6]：大加速度封顶，避免轮廓自交。
    let strength = (acc_mag / params.sensitivity).min(0.6);
    // 加速度方向单位向量（零加速度时取 (0, 0)，形变自然退化为正圆）。
    let dir_x = if acc_mag > 0.0 { acc.x / acc_mag } else { 0.0 };
    let dir_y = if acc_mag > 0.0 { acc.y / acc_mag } else { 0.0 };

    // 圆周采样：对每个采样点按「加速度方向上的投影」做径向位移——
    // 朝加速度方向的点半径拉长（水滴尖端），反方向点半径压缩（尾部圆钝），
    // 侧向点保持，合成水滴形。采样循环构造段数组（teardrop 示范手法）。
    let tau = 3.141592653589793 * 2.0;
    let pts = [];
    let i = 0;
    while i < samples {
        let theta = tau * i / samples;
        let ux = theta.cos();
        let uy = theta.sin();
        // 位移量：与加速度方向的点积 ∈ [-1, 1]，前倾拉伸后压缩。
        let align = ux * dir_x + uy * dir_y;
        let r = radius * (1.0 + strength * align);
        pts.push(#{x: cx + ux * r, y: cy + uy * r});
        i += 1;
    }

    // 拼接闭合 Path：M + (samples-1) 个 L + Z。
    // 直线段连接足够密集的采样点即平滑轮廓；颜色覆盖展示元素级双色
    //（淡填充 + 实描边，覆盖色 = 主动退出图层色，换色热键不跟随）。
    let segments = [#{cmd: "M", x: pts[0].x, y: pts[0].y}];
    let j = 1;
    while j < samples {
        segments.push(#{cmd: "L", x: pts[j].x, y: pts[j].y});
        j += 1;
    }
    segments.push(#{cmd: "Z"});

    [
        #{
            type: "path",
            segments: segments,
            fill: true,
            thickness: thickness,
            fill_color: [0.2, 0.6, 1.0, 0.15],
            stroke_color: [0.4, 0.8, 1.0, 0.9],
        },
    ]
}
