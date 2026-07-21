// Name: 自定义图片
// 对应旧 CrosshairStyle::CustomImage
// 加载 PNG 文件作为准心图案。
// 注意：本物料仅返回 Element::Image，实际 PNG 解码与 blit 由渲染器单独处理。

fn defaults() {
    #{
        path: "",
        scale: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        width: 64.0,
        height: 64.0,
    }
}

fn schema() {
    [
        #{key: "path", label: "图片路径", widget: "image_path"},
        #{key: "scale", label: "缩放", widget: "slider", min: 0.1, max: 10.0, step: 0.1},
        #{key: "offset_x", label: "水平偏移", widget: "slider", min: -500.0, max: 500.0, step: 1.0},
        #{key: "offset_y", label: "垂直偏移", widget: "slider", min: -500.0, max: 500.0, step: 1.0},
        #{key: "width", label: "原始宽度（显示用）", widget: "number", min: 1.0, max: 2000.0, step: 1.0},
        #{key: "height", label: "原始高度（显示用）", widget: "number", min: 1.0, max: 2000.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    if params.path == "" {
        return [];
    }

    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    let scaled_w = params.width * params.scale;
    let scaled_h = params.height * params.scale;

    let center_x = cx + params.offset_x;
    let center_y = cy + params.offset_y;

    [
        #{
            type: "image",
            path: params.path,
            x: center_x - scaled_w / 2.0,
            y: center_y - scaled_h / 2.0,
            w: scaled_w,
            h: scaled_h,
        },
    ]
}
