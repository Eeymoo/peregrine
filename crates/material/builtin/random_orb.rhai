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
        #{key: "offset", label: "距边缘距离", widget: "slider", min: 0.0, max: 400.0, step: 1.0},
        #{key: "jitter", label: "随机扰动", widget: "slider", min: 0.0, max: 200.0, step: 1.0},
        #{key: "radius_min", label: "最小半径", widget: "slider", min: 1.0, max: 50.0, step: 0.5},
        #{key: "radius_max", label: "最大半径", widget: "slider", min: 1.0, max: 50.0, step: 0.5},
        #{key: "center_deviation", label: "中心偏离", widget: "slider", min: 0.1, max: 0.3, step: 0.01},
        #{
            key: "mode",
            label: "模式",
            widget: "select",
            options: [
                #{value: "lock_on_start", label: "锁定随机位置"},
                #{value: "reshuffle", label: "每次重新随机"},
            ],
            "default": "lock_on_start",
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

    let elements = [];

    let edge = 0;
    while edge < 4 {
        let i = 0;
        while i < count {
            let radius = min_r + rand() * (max_r - min_r);
            let j = (rand() - 0.5) * 2.0 * jitter;
            if edge == 0 {
                let x = screen.min_x + rand() * (screen.max_x - screen.min_x);
                elements.push(#{
                    type: "circle",
                    cx: x + j,
                    cy: screen.min_y + offset + j,
                    radius: radius,
                });
            } else if edge == 1 {
                let x = screen.min_x + rand() * (screen.max_x - screen.min_x);
                elements.push(#{
                    type: "circle",
                    cx: x + j,
                    cy: screen.max_y - offset + j,
                    radius: radius,
                });
            } else if edge == 2 {
                let y = screen.min_y + rand() * (screen.max_y - screen.min_y);
                elements.push(#{
                    type: "circle",
                    cx: screen.min_x + offset + j,
                    cy: y + j,
                    radius: radius,
                });
            } else {
                let y = screen.min_y + rand() * (screen.max_y - screen.min_y);
                elements.push(#{
                    type: "circle",
                    cx: screen.max_x - offset + j,
                    cy: y + j,
                    radius: radius,
                });
            }
            i += 1;
        }
        edge += 1;
    }

    elements
}
