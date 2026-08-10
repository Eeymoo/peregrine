// Name: 时间显示
// 动态物料：显示当前时间，可自定义位置、字号、加粗与格式。
// 支持格式占位符：yyyy(年), MM(月), dd(日), HH(24小时), hh(12小时), mm(分), ss(秒), a(AM/PM)
// 示例：HH:mm:ss  → 14:30:25；yyyy-MM-dd HH:mm:ss → 2026-07-19 14:30:25

fn defaults() {
    #{
        font_size: 24.0,
        x: 960.0,
        y: 540.0,
        format: "HH:mm:ss",
        bold: false,
    }
}

fn schema() {
    [
        #{key: "font_size", label: "字体大小", widget: "slider", min: 8.0, max: 200.0, step: 1.0},
        #{key: "x", label: "X 坐标", widget: "number", min: 0.0, max: 4000.0, step: 1.0},
        #{key: "y", label: "Y 坐标", widget: "number", min: 0.0, max: 4000.0, step: 1.0},
        #{key: "format", label: "展示格式（如 HH:mm:ss / yyyy-MM-dd HH:mm:ss）", widget: "text"},
        #{key: "bold", label: "加粗", widget: "toggle"},
    ]
}

fn is_dynamic() {
    true
}

fn build(params, screen) {
    let font_size = params.font_size;
    let x = params.x;
    let y = params.y;
    let format = params.format;
    let bold = if params.contains("bold") { params.bold } else { false };

    let content = format_time(now_ms(), format);
    // 加粗时输出字重 700，否则用 ()（序列化为 null → Rust 侧 None，常规 400）。
    let font_weight = if bold { 700 } else { () };

    [
        #{type: "text", x: x, y: y, content: content, font_size: font_size, font_weight: font_weight},
    ]
}
