// Name: 示例·时钟
// 创作流程：① 选图元 = text；② 定布局 = 屏幕中心；③ 抽参数 = font_size / format；
// ④ 声明 defaults/schema + is_dynamic=true；⑤ 验证（本文件由 cargo test 加载，动态输入软关闭下渲染为静态快照）。
//
// 时间动态物料示例：屏幕中心显示当前时间，每秒刷新。
// 动态输入：time_ms() / now_ms() / format_time()。
// 注意：当前发布构建默认软关闭动态输入，本物料会渲染为启动瞬间的静态时间；
//       在构建启用动态输入后会实时刷新。
// 参数：
//   font_size — 字体像素大小
//   format    — 时间格式（占位符 yyyy MM dd HH hh mm ss a）

fn defaults() {
    #{
        font_size: 32.0,
        format: "HH:mm:ss",
    }
}

fn schema() {
    [
        #{key: "font_size", label: "字体大小", widget: "slider", min: 8.0, max: 200.0, step: 1.0},
        #{key: "format", label: "时间格式（如 HH:mm:ss）", widget: "text"},
    ]
}

fn is_dynamic() {
    true
}

fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let content = format_time(now_ms(), params.format);

    [
        #{type: "text", x: cx, y: cy, content: content, font_size: params.font_size},
    ]
}
