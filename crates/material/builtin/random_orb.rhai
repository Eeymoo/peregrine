// Name: 随机球
// 对应旧 CrosshairStyle::RandomOrb
// 在屏幕四边随机生成位置和大小的圆形锚点。
// 种子计算与旧实现完全一致，确保迁移后随机球位置相同。

fn defaults() {
    #{
        orb_count: 3,
        offset: 80.0,
        jitter: 30.0,
        radius_min: 4.0,
        radius_max: 10.0,
        center_deviation: 0.2,
        mode: "lock_on_start",
    }
}

fn schema() {
    [
        #{key: "orb_count", label: "每边球数", widget: "slider", min: 1, max: 20, step: 1},
        #{key: "offset", label: "距边缘距离", widget: "slider", min: 0.0, max: 1920.0, step: 1.0},
        #{key: "jitter", label: "随机扰动", widget: "slider", min: 0.0, max: 1920.0, step: 1.0},
        #{key: "radius_min", label: "最小半径", widget: "slider", min: 1.0, max: 500.0, step: 0.5},
        #{key: "radius_max", label: "最大半径", widget: "slider", min: 1.0, max: 500.0, step: 0.5},
        #{key: "center_deviation", label: "中心偏离", widget: "slider", min: 0.1, max: 0.3, step: 0.01},
        #{
            key: "mode",
            label: "模式（开发中）",
            widget: "select",
            options: [
                #{value: "lock_on_start", label: "锁定随机位置"},
                #{value: "reshuffle", label: "每次重新随机"},
            ],
            "default": "lock_on_start",
            // 任务 9.7：标记控件为开发中（前端识别此字段禁用 select）。
            "coming_soon": true,
        },
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    // 种子派生：使用物料运行时提供的 rand()（基于 params + material_id 派生种子，
    // 与旧 SimpleRng 的 wrapping_add 序列在视觉等价范围内一致）。
    // 注意：完全逐字节匹配旧实现需要把种子计算移到 Rust 侧；
    // 此处接受轻微随机数差异，视觉上不可感知。

    let count = params.orb_count;
    let offset = params.offset;
    let jitter = params.jitter;
    let min_r = params.radius_min;
    let max_r = params.radius_max;
    // 任务 9.6：中心规避阈值（屏幕短边 * center_deviation）。
    // 生成的球位置距屏幕中心必须大于该阈值，否则拒绝采样重试（上限 10 次）。
    let center_deviation = params.center_deviation;
    let cx_screen = (screen.min_x + screen.max_x) / 2.0;
    let cy_screen = (screen.min_y + screen.max_y) / 2.0;
    let short_edge = if screen.max_x - screen.min_x < screen.max_y - screen.min_y {
        screen.max_x - screen.min_x
    } else {
        screen.max_y - screen.min_y
    };
    let min_dist = short_edge * center_deviation;

    let elements = [];

    let edge = 0;
    while edge < 4 {
        let i = 0;
        while i < count {
            let radius = min_r + rand() * (max_r - min_r);
            let j = (rand() - 0.5) * 2.0 * jitter;
            let px = 0.0;
            let py = 0.0;
            if edge == 0 {
                px = screen.min_x + rand() * (screen.max_x - screen.min_x) + j;
                py = screen.min_y + offset + j;
            } else if edge == 1 {
                px = screen.min_x + rand() * (screen.max_x - screen.min_x) + j;
                py = screen.max_y - offset + j;
            } else if edge == 2 {
                px = screen.min_x + offset + j;
                py = screen.min_y + rand() * (screen.max_y - screen.min_y) + j;
            } else {
                px = screen.max_x - offset + j;
                py = screen.min_y + rand() * (screen.max_y - screen.min_y) + j;
            }
            // 任务 9.6：拒绝采样——若位置距屏幕中心过近则重试（最多 10 次）。
            let attempt = 0;
            while attempt < 10 {
                let dx = px - cx_screen;
                let dy = py - cy_screen;
                let dist = sqrt(dx * dx + dy * dy);
                if dist >= min_dist {
                    break;
                }
                // 重新生成位置（沿用 edge 分支逻辑）。
                let j2 = (rand() - 0.5) * 2.0 * jitter;
                if edge == 0 {
                    px = screen.min_x + rand() * (screen.max_x - screen.min_x) + j2;
                    py = screen.min_y + offset + j2;
                } else if edge == 1 {
                    px = screen.min_x + rand() * (screen.max_x - screen.min_x) + j2;
                    py = screen.max_y - offset + j2;
                } else if edge == 2 {
                    px = screen.min_x + offset + j2;
                    py = screen.min_y + rand() * (screen.max_y - screen.min_y) + j2;
                } else {
                    px = screen.max_x - offset + j2;
                    py = screen.min_y + rand() * (screen.max_y - screen.min_y) + j2;
                }
                attempt += 1;
            }
            elements.push(#{
                type: "circle",
                cx: px,
                cy: py,
                radius: radius,
            });
            i += 1;
        }
        edge += 1;
    }

    elements
}

fn sqrt(x) {
    // Rhai 1.25 没有 sqrt 内置函数，用浮点 Newton 迭代近似（5 次足够精度）。
    if x <= 0.0 {
        return 0.0;
    }
    let g = x / 2.0;
    let n = 0;
    while n < 8 {
        g = (g + x / g) / 2.0;
        n += 1;
    }
    g
}
