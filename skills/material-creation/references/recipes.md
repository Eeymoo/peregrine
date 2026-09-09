# 实战配方集

五个配方覆盖常见需求，全部取材自内置物料（`crates/material/builtin/`）的成熟实现。按需求选型后读对应配方，抄结构改参数。

## 目录

1. [配方一：静态组合图元（十字 + 环）](#配方一静态组合图元)
2. [配方二：呼吸动画（性能模型标杆）](#配方二呼吸动画)
3. [配方三：时钟文本](#配方三时钟文本)
4. [配方四：SVG 路径物料（任意形状 + 归一化）](#配方四svg-路径物料)
5. [配方五：按键指示器](#配方五按键指示器)
6. [通用设计原则](#通用设计原则)

## 配方一：静态组合图元

**适用**：十字、边框、网格等固定几何，多个专用图元拼装。参考 `builtin/cross.rhai`、`builtin/ring.rhai`。

结构要点：

- 从 `screen` 计算中心，**禁止硬编码分辨率**（用户屏幕可能是 2560×1080）。
- 尺寸参数用「屏幕短边百分比」（`radius_pct`）或绝对像素（`thickness`）二选一，前者适配不同分辨率。
- 静态物料不导出 `is_dynamic`（默认 false），结果永久缓存，零稳态开销。

```rhai
// Name: 十字圆环
fn defaults() {
    #{
        arm_length: 20.0,   // 绝对像素
        thickness: 3.0,
        gap: 4.0,
        ring_radius_pct: 0.03,  // 屏幕短边百分比
    }
}
fn schema() {
    [
        #{key: "arm_length", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
        #{key: "ring_radius_pct", label: "环半径", widget: "slider", min: 0.005, max: 0.2, step: 0.005},
    ]
}
fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let arm = params.arm_length;
    let t = params.thickness;
    let g = params.gap / 2.0;
    let r = (screen.max_y - screen.min_y) * params.ring_radius_pct;
    [
        // 四臂矩形
        #{type: "rect", x: cx - arm, y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx + g, y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx - t / 2.0, y: cy - arm, w: t, h: arm - g},
        #{type: "rect", x: cx - t / 2.0, y: cy + g, w: t, h: arm - g},
        // 中心描边环（颜色继承图层）
        #{type: "circle_stroke", cx: cx, cy: cy, radius: r, thickness: t},
    ]
}
```

## 配方二：呼吸动画

**适用**：任何 `time_ms()` 驱动的周期动画。参考 `builtin/teardrop.rhai`（呼吸环，动态物料性能标杆）。完整源码在仓库内，此处摘录核心骨架与设计决策。

```rhai
// Name: 呼吸圆点
fn defaults() {
    #{
        radius_pct: 0.05,
        breath_speed: "normal",   // select 档位优于自由数字
        breath_amp: 0.03,         // 内部参数不暴露 UI（3% 经验值，调大反成晃动干扰）
        samples: 24,
    }
}
fn schema() {
    [
        #{key: "radius_pct", label: "大小", widget: "slider", min: 0.01, max: 0.12, step: 0.005},
        #{
            key: "breath_speed", label: "呼吸速度", widget: "select",
            options: [
                #{value: "slow", label: "舒缓（8 次/分）"},
                #{value: "normal", label: "自然（14 次/分）"},
                #{value: "fast", label: "快节奏（20 次/分）"},
            ],
        },
    ]
}
fn is_dynamic() { true }

// 关键 1：唤醒节流。呼吸周期最短 3000ms，10Hz 采样相位精度无视觉损失。
fn refresh_interval_ms() { 100 }

// 档位 → 周期（ms），顶层具名函数（Rhai 无闭包赋值）。
fn breath_period_of(speed) {
    if speed == "slow" { 7500.0 } else if speed == "fast" { 3000.0 } else { 4300.0 }
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let base = (screen.max_y - screen.min_y) * params.radius_pct;
    let tau = 3.141592653589793 * 2.0;

    let period = breath_period_of(params.breath_speed);
    let phase = tau * ((time_ms() * 1.0 % period) / period);
    // 关键 2：0.5px 量化跳帧。同一量化档内帧指纹不变 → 光栅化整体跳过；
    // 档间跳变亚像素级不可见，平滑感不受影响。
    let radius_raw = base * (1.0 + params.breath_amp * phase.sin());
    let radius = (radius_raw * 2.0).floor() / 2.0;

    [
        #{type: "circle", cx: cx, cy: cy, radius: radius},
    ]
}
```

若需要**环形**（而非实心圆）：单条 path 双圈绕行——外圈采样 + 内圈**逆序**采样 + 各自 `Z`，nonzero 填充规则下内圈环绕数相消形成镂空；采样点用「中点 Q 平滑」（端点取相邻采样点中点、控制点取采样点自身）得到 C1 连续完美正圆。完整实现照抄 `builtin/teardrop.rhai` 的 `build_ring` / `mid` 函数。

演进史教训（teardrop 实机反馈，避免重蹈）：

- 加速度尖刺 → 移除（不对称方向性形变 = 反向 rest-frame）；
- 鼠标跟随 → 移除（60fps 全屏重光栅，CPU 7%）；
- 动画幅度 >5% → 用户感知为"晃动"而非"呼吸"，反而促成晕动。

## 配方三：时钟文本

**适用**：时间 / 状态文字显示。参考 `builtin/time.rhai`。

```rhai
// Name: 时间显示
fn defaults() {
    #{font_size: 24.0, x: 960.0, y: 540.0, format: "HH:mm:ss", bold: false}
}
fn schema() {
    [
        #{key: "font_size", label: "字体大小", widget: "slider", min: 8.0, max: 200.0, step: 1.0},
        #{key: "x", label: "X 坐标", widget: "slider", min: 0.0, max: 4000.0, step: 1.0},
        #{key: "y", label: "Y 坐标", widget: "slider", min: 0.0, max: 4000.0, step: 1.0},
        #{key: "format", label: "展示格式（如 HH:mm:ss）", widget: "text"},
        #{key: "bold", label: "加粗", widget: "toggle"},
    ]
}
fn is_dynamic() { true }
// 秒级显示 + 防跳秒余量：60FPS 下求值从 60 次/秒降到 2 次/秒。
fn refresh_interval_ms() { 500 }

fn build(params, screen) {
    // 时间来源用 time_ms()（上下文快照，预览/overlay 一致）。
    // 不要用 now_ms()（直读墙钟，两端漂移）。
    let content = format_time(time_ms(), params.format);
    // 加粗 → 700；否则 ()（= null → Rust 侧 None → 默认 400）。
    let font_weight = if params.bold { 700 } else { () };
    [
        #{type: "text", x: params.x, y: params.y, content: content,
          font_size: params.font_size, font_weight: font_weight},
    ]
}
```

格式占位符：`yyyy` `MM` `dd` `HH`（24h）`hh`（12h）`mm` `ss` `a`（AM/PM），支持中文字面量混排（`"yyyy年MM月dd日 HH时mm分"`）。

## 配方四：SVG 路径物料

**适用**：任意矢量形状（从 Figma / Illustrator / Inkscape 导出 path）。参考 `builtin/path_showcase.rhai`。

核心结构：

```rhai
// Name: 自定义路径
fn defaults() {
    #{
        // 默认形状：正圆（四个 90° C 段；控制点切向长度 r×0.5523 是圆弧标准贝塞尔近似）
        d: "M 50 5 C 74.85 5 95 25.15 95 50 C 95 74.85 74.85 95 50 95 C 25.15 95 5 74.85 5 50 C 5 25.15 25.15 5 50 5 Z",
        size_pct: 0.08,
        fill: true,
        thickness: 2.0,
    }
}
fn schema() { /* d: text；size_pct/fill/thickness 见模板 */ }
fn is_dynamic() { false }

fn build(params, screen) {
    let segs_in = parse_svg_path(params.d);
    if segs_in.len() == 0 {
        return build_fallback(screen, params);  // 解析失败（A/S/T/语法错）回退默认形状——永不渲染空输出
    }
    // 1) 扫描所有段坐标求包围盒（Q/C 控制点参与保守估计）；
    // 2) 等比缩放到目标大小：scale = target / max(bw, bh)，退化保护（零宽高用 1.0）；
    // 3) 包围盒中心平移到屏幕中心：x' = cx + (x - bcx) * scale；
    // 4) 重建段数组（每个坐标字段同样变换）。
    // ...完整实现见 builtin/path_showcase.rhai
}
```

设计决策：**包围盒归一化**让用户贴入任何坐标域（0..24 图标网格、0..512 设计稿）都正确铺满目标大小；**空回退**保证非法 `d` 不让图层消失（消失比回退更迷惑）。

## 配方五：按键指示器

**适用**：按键状态可视化（如显示 Shift 是否按下）。参考 `crates/material/examples/key_indicator.rhai`。

```rhai
// Name: 按键指示
fn defaults() {
    #{key: "shift", size: 20.0, x_pct: 0.05, y_pct: 0.9}
}
fn schema() {
    [
        #{key: "key", label: "按键（如 shift / ctrl / f1）", widget: "text"},
        #{key: "size", label: "大小", widget: "slider", min: 8.0, max: 80.0, step: 1.0},
    ]
}
fn is_dynamic() { true }

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) * params.x_pct;
    let cy = (screen.min_y + screen.max_y) * params.y_pct;
    let pressed = key_down(params.key);
    // 按下 = 实心圆，松开 = 描边环；尺寸微缩给出状态反馈。
    if pressed {
        [#{type: "circle", cx: cx, cy: cy, radius: params.size}]
    } else {
        [#{type: "circle_stroke", cx: cx, cy: cy, radius: params.size * 0.9, thickness: 2.0}]
    }
}
```

注意：按键物料输出变化频率高（键入时每帧都可能变），`refresh_interval_ms` 不要声明——输入事件需要即时反映，节流反而迟滞。

## 通用设计原则

所有配方共享的不变式（Peregrine 锚点用途决定，违反会被实机体验打回）：

1. **中心稳定**：中心凹注视区域像素级稳定；动态只在边缘 / 尺寸维度。
2. **无方向性形变**：对称缩放 ✅；拉伸 / 平移尖刺 ❌（反向 rest-frame）。
3. **动作克制**：呼吸级（±3%）缓慢节律 ✅；快速晃动 ❌（促成晕动）。
4. **颜色继承图层**：默认省略颜色字段，图层级换色热键统一调控；仅多色物料显式输出。
5. **尺寸走屏幕短边百分比**：适配任意分辨率；厚度类参数可用绝对像素。
6. **档位用 select 不用自由数字**：呼吸速度这类语义参数给预设枚举，防调出破坏体验的值。
