// Name: 锚定环
// 静态锚点物料（id 保留 "teardrop"：PR 未合入前的历史名，行为已重设计）。
//
// 定位：填充式圆环锚点（rest-frame 参照物）。与 builtin.ring 的
// 描边环观感不同：镂空填充的「实心环」，视觉存在感更强，
// 适合需要醒目锚点的场景。
//
// 演进史（实机反馈驱动，三轮收敛）：
// 1. v4 加速度尖刺：不对称方向性形变，恰在玩家甩视角（最易晕的
//    时刻）模拟运动感、参与叠加 vection，与 rest-frame 语义相反
//    ——移除；
// 2. 呼吸 + 遇动定格（鼠标跟随）：静止呼吸、运动定格。实机 CPU 7%
//    （60fps 全屏光栅）→ 降频 + 量化后仍 3%——动态管线的固有成本
//    （唤醒节拍 + 每次求值重建 Rhai Engine + 指纹哈希）与呼吸的
//    观感价值不成正比 ——移除鼠标跟随，再移除呼吸；
// 3. 最终形态：纯静态填充环。is_dynamic=false → 求值一次、光栅
//    一次、零唤醒（纯事件驱动），稳态 CPU 与静态层同级（~0.1%）。
//
// 设计原则（全程不变）：
// - 永远是完美正圆，无不对称/方向性形变；
// - 中心凹注视的环孔区域像素级稳定；
// - 颜色继承图层基色（换色热键生效）。
//
// 环的实现：单条 Path 双圈绕行（外圈 + 内圈逆序 + Z），nonzero
// 填充规则下内圈环绕数相消（镂空）；采样点用「中点 Q 平滑」连接
// （端点取相邻采样点中点、控制点取采样点自身，C1 连续）。
//
// 输入：无（纯静态）。参数变更 / 屏幕尺寸变更时由外部事件触发
// 重新求值，静止配置下帧指纹恒定（稳态跳帧）。

fn defaults() {
    #{
        radius_pct: 0.05,
        ring_width: 10.0,
        samples: 24,
        thickness: 0.0,
    }
}

fn schema() {
    [
        #{key: "radius_pct", label: "大小", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{key: "ring_width", label: "环宽", widget: "slider", min: 2.0, max: 40.0, step: 1.0},
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.0, max: 12.0, step: 0.5},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let base = (screen.max_y - screen.min_y) * params.radius_pct;
    let samples = params.samples;
    let tau = 3.141592653589793 * 2.0;

    let outer = base;
    let width = params.ring_width;
    // 内径保护：极端参数（大环宽 × 小半径）下不允许内圈翻转或贴死圆心。
    let inner = if outer - width < outer * 0.25 {
        outer * 0.25
    } else {
        outer - width
    };

    // ===== 正圆环采样（中点 Q 平滑双圈 + nonzero 镂空）=====
    let outer_pts = [];
    let inner_pts = [];
    let i = 0;
    while i < samples {
        let theta = tau * i / samples;
        let ux = theta.cos();
        let uy = theta.sin();
        outer_pts.push(#{x: cx + ux * outer, y: cy + uy * outer});
        inner_pts.push(#{x: cx + ux * inner, y: cy + uy * inner});
        i += 1;
    }

    // 内圈逆序绕行（nonzero 镂空的关键）。
    let reversed_inner = [];
    let r = inner_pts.len() - 1;
    while r >= 0 {
        reversed_inner.push(inner_pts[r]);
        r -= 1;
    }

    // 单条 Path：外圈 + 内圈（M 开启新子路径）+ Z 闭合。
    let segments = build_ring(outer_pts);
    let inner_segs = build_ring(reversed_inner);
    segments.push(#{cmd: "M", x: inner_segs[0].x, y: inner_segs[0].y});
    let s = 1;
    while s < inner_segs.len() {
        segments.push(inner_segs[s]);
        s += 1;
    }
    segments.push(#{cmd: "Z"});

    // 颜色：不携带覆盖字段 = 继承图层基色（换色热键生效）。
    // fill 恒为 true（环靠镂空填充实现，false 会被转换层拒绝）。
    [
        #{
            type: "path",
            segments: segments,
            fill: true,
            thickness: params.thickness,
        },
    ]
}

// 两点中点（build_ring 的辅助：Rhai 不支持闭包赋值，用顶层具名函数）。
fn mid(a, b) {
    #{x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0}
}

// 构建一圈中点 Q 平滑闭合曲线段序列。
// 普通点：端点取相邻采样点中点、控制点取采样点自身（C1 连续）。
// 无角点逻辑——锚定环永不产生尖角（对称原则）。
fn build_ring(pts) {
    let n = pts.len();
    let first = mid(pts[n - 1], pts[0]);
    let segs = [#{cmd: "M", x: first.x, y: first.y}];
    let k = 0;
    while k < n {
        let cur = pts[k];
        let next = pts[(k + 1) % n];
        let next_mid = mid(cur, next);
        segs.push(#{
            cmd: "Q",
            x1: cur.x,
            y1: cur.y,
            x: next_mid.x,
            y: next_mid.y,
        });
        k += 1;
    }
    segs
}
