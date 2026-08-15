// Name: 呼吸环
// 动态锚点物料（id 保留 "teardrop"：PR 未合入前的历史名，行为已重设计）。
//
// 核心语义：半径以 ±3% 对称涨缩的填充环，周期对齐常见呼吸频率
// （默认 14 次/分，成人静息中段）。缓慢呼吸节律对缓解 cybersickness
// 有循证支持（呼吸 pacing），持续微动也防止锚点被视觉系统习惯化
// 而失效（rest-frame 持续再捕获）。
//
// 演进史（实机反馈驱动）：
// 1. v4 加速度尖刺 → 移除（不对称方向性形变，反向 rest-frame）；
// 2. 呼吸 + 鼠标遇动定格 → 移除鼠标跟随（CPU 7%：60fps 全屏光栅）；
// 3. 纯静态 → 按反馈恢复呼吸（用户明确需要），但保留全部性能手段。
//
// ===== 性能模型（10Hz 节流 + 量化跳帧）=====
// 呼吸是本物料唯一的动态输入，两条正交手段压稳态开销：
// 1. refresh_interval_ms() = 100：唤醒 60Hz → 10Hz，求值 + 指纹 +
//    光栅整条链路随唤醒降频；
// 2. 半径 0.5px 量化：同一量化档内 24 个采样点坐标完全相同 →
//    帧指纹不变 → 光栅化被整体跳过。呼吸一周期（4300ms）半径只跨
//    ~2 个量化档，实际光栅 <4 次/秒；档间跳变亚像素级不可见。
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
// 输入：time_ms()（呼吸相位）。预览端恒静止 → 预览只见呼吸。

fn defaults() {
    #{
        radius_pct: 0.05,
        ring_width: 10.0,
        // 呼吸速度预设 → 周期（ms）。
        breath_speed: "normal",
        samples: 24,
        // 呼吸幅度（内部参数，未暴露 UI——3% 经验值，暴露易调出
        //「晃动锚点」反而促成晕动症的反效果）。
        breath_amp: 0.03,
        thickness: 0.0,
    }
}

fn schema() {
    [
        #{key: "radius_pct", label: "大小", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{key: "ring_width", label: "环宽", widget: "slider", min: 2.0, max: 40.0, step: 1.0},
        #{
            key: "breath_speed",
            label: "呼吸速度",
            widget: "select",
            options: [
                #{value: "slow", label: "舒缓（8 次/分）"},
                #{value: "normal", label: "自然（14 次/分）"},
                #{value: "fast", label: "快节奏（20 次/分）"},
            ],
        },
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.0, max: 12.0, step: 0.5},
    ]
}

fn is_dynamic() {
    true
}

// 最小重绘节拍（ms）：唤醒从 fps 节拍（60Hz）降到 10Hz。
// 呼吸周期最短 3000ms、0.5px 量化档间跳变亚像素级不可见，
// 10Hz 采样相位精度已无视觉损失（Nyquist 余量充足）。
fn refresh_interval_ms() {
    100
}

// 呼吸速度预设 → 周期（ms），对齐常见人类呼吸频率：
// slow = 8 次/分（7.5s）——经典放松呼吸引导节律（慢呼吸放松）；
// normal = 14 次/分（~4.3s）——成人静息呼吸中段（12–20 次/分）；
// fast = 20 次/分（3s）——静息呼吸上限（再快是运动/紧张态，不安）。
fn breath_period_of(speed) {
    if speed == "slow" {
        7500.0
    } else if speed == "fast" {
        3000.0
    } else {
        4300.0
    }
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let base = (screen.max_y - screen.min_y) * params.radius_pct;
    let samples = params.samples;
    let tau = 3.141592653589793 * 2.0;

    // ===== 呼吸相位（time_ms 驱动，恒开启）=====
    let period = breath_period_of(params.breath_speed);
    let phase = tau * ((time_ms() * 1.0 % period) / period);
    // 0.5px 量化：同一量化档内帧指纹不变（跳过光栅化），档间跳变
    // 亚像素级不可见——呼吸平滑感不受影响。
    let outer_raw = base * (1.0 + params.breath_amp * phase.sin());
    let outer = (outer_raw * 2.0).floor() / 2.0;
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
// 无角点逻辑——呼吸环永不产生尖角（对称原则）。
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
