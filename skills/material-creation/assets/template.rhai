// Name: TODO显示名
// TODO一句话描述物料（静态 / 动态、核心视觉语义）。
//
// 起步模板：复制本文件到 <应用数据目录>/materials/<snake_case名>.rhai，
// 替换所有 TODO 标记。写完对照 references/api.md 核对字段名。

fn defaults() {
    // build 里读的每个参数都必须在这里有默认值。
    #{
        size_pct: 0.05,       // TODO 尺寸类参数建议走屏幕短边百分比
        thickness: 2.0,
        // TODO 更多参数
    }
}

fn schema() {
    // defaults 的每个键都应出现；widget 只能是
    // number / slider / color / select / toggle / image_path / text。
    [
        #{key: "size_pct", label: "TODO大小", widget: "slider", min: 0.005, max: 0.2, step: 0.005},
        #{key: "thickness", label: "TODO粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        // TODO 更多控件
        // select 档位示例：
        // #{key: "mode", label: "模式", widget: "select", options: [
        //     #{value: "a", label: "甲"}, #{value: "b", label: "乙"},
        // ]},
    ]
}

// 读取了 time_ms / mouse_pos / key_down / rand 等动态输入则改为 true 并删除本注释。
// 静态物料（false / 缺省）结果永久缓存，读动态输入只会得到冻结值。
fn is_dynamic() {
    false
}

// 仅动态物料需要：声明"输出最快多久变一次"（ms），调度器据此节流唤醒。
// 时钟 500 / 呼吸 100；输入响应类（按键指示）不要声明。
// fn refresh_interval_ms() { 100 }

fn build(params, screen) {
    // 从 screen 计算布局，禁止硬编码分辨率。
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    let r = (screen.max_y - screen.min_y) * params.size_pct;

    // 必须返回数组（[#{}] 而非 #{}）；颜色字段省略 = 继承图层基色（推荐）。
    [
        #{type: "circle_stroke", cx: cx, cy: cy, radius: r, thickness: params.thickness},
    ]
}
