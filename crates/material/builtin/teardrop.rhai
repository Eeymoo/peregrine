// Name: 锚定环
// 动态锚点物料（id 保留 "teardrop"：PR 未合入前的历史名，行为已重设计）。
//
// 核心语义：「呼吸 → 遇动定格」（下锚隐喻）。
// - 静止时缓慢呼吸：半径以 breath_amp（±3%）对称涨缩，~0.1Hz
//   （10 秒/周期 ≈ 6 次呼吸/分钟）。缓慢呼吸节律对缓解 cybersickness
//   有循证支持（呼吸 pacing），持续微动也防止锚点被视觉系统
//   习惯化而失效（rest-frame 持续再捕获）。
// - 检测到鼠标运动（速度或加速度超阈值）立即定格：呼吸熄灭、
//   半径锁回基准值、环宽随运动量加宽变实——「世界乱了，锚钉死」。
//
// 为什么放弃 v4 的加速度尖刺（实机反馈不符合预期的根因）：
// 锚点对晕 3D 人群的全部价值 = 镜头运动时提供稳定参照系（rest frame）。
// 尖刺是不对称的方向性运动信号——恰在玩家甩视角（最易晕的时刻）
// 模拟运动感、参与叠加 vection，与锚点存在的意义竞争；且形变时
// 环上没有任何像素保持稳定，眼睛无处可锁。
// 重设计遵循的原则：
// - 永不产生不对称/方向性形变，任何时刻都是完美正圆；
// - 只做对称调制：整体涨缩（呼吸）+ 环宽增实（遇动）；
// - 中心凹注视的环孔区域像素级稳定；
// - 「世界静止时它是活的，世界运动时它死死钉住」。
//
// 运动量 motion ∈ [0,1] = max(归一化速度, 归一化加速度)，各自
// 「阈值起步 + 满量程封顶」（阈值滤手抖，封顶防过度响应）：
// - breath = 1 - motion：呼吸幅度随运动量线性熄灭，motion=1 时
//   精确为 0（半径锁定基准值，帧指纹稳定，持续运动中可回跳帧）；
// - 环宽 = ring_width × (1 + solid_boost × motion)：遇动变实变宽。
// 速度通道让匀速平移镜头（加速度≈0）也能触发定格；制动门控不再
// 需要——没有方向性形变就没有「假动作」可言。
//
// 环的实现（与 v4 相同）：单条 Path 双圈绕行（外圈 + 内圈逆序 + Z），
// nonzero 填充规则下内圈环绕数相消（镂空）；采样点用「中点 Q 平滑」
// 连接（端点取相邻采样点中点、控制点取采样点自身，C1 连续）。
//
// 参数面板（玩家语言）：反应灵敏度 / 呼吸速度 / 遇动增强均为
// select 预设（低/中/高），内部映射到运动检测阈值组与呼吸周期——
// 避免向玩家暴露 px/s² 等工程单位；预设数值经实机标定。
// breath_amp 保持内部参数不暴露（3% 经验值，暴露易调出反效果）。
//
// 输入：mouse_velocity() / mouse_acceleration()（运动检测，平台层
// 已做 EMA 平滑 + 死区归零）+ time_ms()（呼吸相位）。
// 预览端速度/加速度恒为 0 → 预览只见呼吸（spec 已接受行为）。
// breathing=false 关闭呼吸后，静止输出与时间无关（回到稳态跳帧）。

fn defaults() {
    #{
        radius_pct: 0.05,
        ring_width: 10.0,
        // 反应灵敏度预设 → 运动检测双通道（阈值起步 + 满量程封顶）。
        // low：大动作才响应；medium：默认；high：轻微移动即响应。
        sensitivity_preset: "medium",
        // 呼吸速度预设 → 周期（ms）。
        breath_speed: "normal",
        // 遇动增强预设 → solid_boost 系数（0 = 形状完全不变）。
        solid_boost: 1.0,
        samples: 24,
        // 呼吸幅度（内部参数，未暴露 UI——3% 经验值，暴露后易调出
        //「晃动锚点」反而促成晕动症的反效果）。
        breath_amp: 0.03,
        thickness: 0.0,
        breathing: true,
    }
}

fn schema() {
    [
        // ===== 常用（开箱即用：默认即最佳，玩家只动这三样）=====
        #{key: "radius_pct", label: "大小", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{key: "ring_width", label: "环宽", widget: "slider", min: 2.0, max: 40.0, step: 1.0},
        #{
            key: "sensitivity_preset",
            label: "反应灵敏度",
            widget: "select",
            options: [
                #{value: "low", label: "低（大幅移动才定格）"},
                #{value: "medium", label: "中（推荐）"},
                #{value: "high", label: "高（轻微移动即定格）"},
            ],
        },
        #{key: "breathing", label: "静止时呼吸", widget: "toggle"},
        // ===== 进阶（有明确需求的用户才动）=====
        #{key: "thickness", label: "描边粗细", widget: "slider", min: 0.0, max: 12.0, step: 0.5},
        #{
            key: "breath_speed",
            label: "呼吸速度",
            widget: "select",
            options: [
                #{value: "slow", label: "缓慢"},
                #{value: "normal", label: "正常"},
                #{value: "fast", label: "轻快"},
            ],
        },
        #{
            key: "solid_boost",
            label: "遇动增强",
            widget: "select",
            options: [
                #{value: 0.0, label: "关闭（形状不变）"},
                #{value: 0.5, label: "轻微"},
                #{value: 1.0, label: "明显（推荐）"},
                #{value: 2.0, label: "强烈"},
            ],
        },
    ]
}

fn is_dynamic() {
    true
}

// 灵敏度预设 →（加速度阈值, 加速度满量程, 速度阈值, 速度满量程）。
// 阈值起步滤手抖，满量程封顶防过度响应；数值经实机标定：
// low = 大幅甩动才触发；medium = 常规玩法动作触发；high = 轻微移动即触发。
fn motion_tuning(preset) {
    if preset == "low" {
        #{acc_th: 400.0, acc_full: 2500.0, vel_th: 150.0, vel_full: 3000.0}
    } else if preset == "high" {
        #{acc_th: 60.0, acc_full: 400.0, vel_th: 12.0, vel_full: 400.0}
    } else {
        #{acc_th: 150.0, acc_full: 900.0, vel_th: 40.0, vel_full: 1200.0}
    }
}

// 呼吸速度预设 → 周期（ms）。slow ≈ 0.07Hz（接近放松呼吸节律），
// normal ≈ 0.1Hz（6 次/分钟），fast ≈ 0.16Hz（轻快但不安）。
fn breath_period_of(speed) {
    if speed == "slow" {
        14000.0
    } else if speed == "fast" {
        6000.0
    } else {
        10000.0
    }
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let base = (screen.max_y - screen.min_y) * params.radius_pct;
    let samples = params.samples;
    let tau = 3.141592653589793 * 2.0;

    // ===== 运动量归一（速度 + 加速度双通道，取大者）=====
    // 各通道：阈值起步（滤手抖/EMA 残值）→ 满量程封顶（防过度响应）。
    // 平台层死区已保证静止时输入精确为 0（阈值以下亦然），
    // 因此静止时 motion 精确为 0，呼吸全幅。
    let tuning = motion_tuning(params.sensitivity_preset);
    let vel = mouse_velocity();
    let vel_mag = (vel.x * vel.x + vel.y * vel.y).sqrt();
    let acc = mouse_acceleration();
    let acc_mag = (acc.x * acc.x + acc.y * acc.y).sqrt();
    let m_acc = ((acc_mag - tuning.acc_th) / tuning.acc_full).min(1.0).max(0.0);
    let m_vel = ((vel_mag - tuning.vel_th) / tuning.vel_full).min(1.0).max(0.0);
    let motion = if m_acc > m_vel { m_acc } else { m_vel };

    // ===== 呼吸相位（time_ms 驱动，运动时熄灭）=====
    // amp = breath_amp × (1 - motion)：motion=1 时精确为 0 →
    // 半径锁定基准值，帧指纹稳定（持续运动中回跳帧）。
    // breathing=false 时相位恒 0（输出与时间无关，稳态跳帧）。
    let period = breath_period_of(params.breath_speed);
    let phase = if params.breathing {
        tau * ((time_ms() * 1.0 % period) / period)
    } else {
        0.0
    };
    let amp = params.breath_amp * (1.0 - motion);
    let outer = base * (1.0 + amp * phase.sin());

    // ===== 遇动增实（对称环宽调制，无方向性）=====
    let width = params.ring_width * (1.0 + params.solid_boost * motion);
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
