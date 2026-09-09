---
title: "配置说明"
---

配置文件为 JSON 格式，位于：

- Windows：`%APPDATA%\Peregrine\config.json`
- macOS：`~/Library/Application Support/Peregrine/config.json`
- Linux：`~/.config/Peregrine/config.json`

> Peregrine 是仅面向 Windows 的工具。上表列出 macOS / Linux 路径是因为配置库按操作系统标准目录实现；覆盖层及核心功能仅在 Windows 上实现。

首次运行会自动生成默认配置。你可以直接编辑该文件，保存后程序会在约 300ms 去抖后自动热重载，无需重启。

> 本文以 `crates/config/src/schema.rs` 为唯一事实源编写；若文档与代码冲突，以代码为准。

## 配置结构（新格式）

正文示例与程序生成的默认配置一致（`settings` 全局块 + `layers` 多图层列表）：

```json
{
  "active_profile": "default",
  "profiles": {
    "default": {
      "layers": [
        {
          "id": "default",
          "name": "贴边矩形",
          "material": { "kind": "builtin", "id": "builtin.edge_rect" },
          "params": {},
          "style": {
            "color": [1.0, 1.0, 1.0, 1.0],
            "opacity": 0.6,
            "blend_mode": "normal"
          },
          "transform": {
            "offset_x": 0.0,
            "offset_y": 0.0,
            "scale": 1.0,
            "rotation_deg": 0.0
          },
          "visible": true,
          "locked": false
        }
      ],
      "trigger": { "enabled": true, "process_names": [] },
      "settings_hotkey": "F10",
      "target_window": ""
    }
  },
  "settings": {
    "auto_switch_on_overlay": "ask",
    "locale": "auto",
    "fullscreen_overlay": true,
    "live_drag_preview": false,
    "gpu_acceleration": false,
    "update_channel": "stable",
    "cn_mirror": false,
    "mirror_url": "https://v4.gh-proxy.org",
    "antialiasing": true,
    "renderer_backend": "cpu",
    "quick_colors": [
      [1.0, 1.0, 1.0, 1.0],
      [0.0, 1.0, 0.0, 1.0],
      [0.2, 0.5, 1.0, 1.0],
      [1.0, 0.0, 0.0, 1.0],
      [1.0, 0.5, 0.0, 1.0]
    ],
    "hotkey_bindings": [["toggle_overlay", "Ctrl+Alt+O"]],
    "material": { "dynamic_enabled": true }
  }
}
```

各字段的详细说明见下文表格；图层与物料的进阶用法参见[图层](./layers.md)与[物料脚本](./material-scripting.md)，各设置项在设置窗口中的位置参见[设置详解](./settings.md)。

## 字段说明

### AppConfig

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_profile` | string | 当前激活的 Profile 名称，必须在 `profiles` 中存在 |
| `profiles` | map | 所有 Profile，键为名称，至少包含一个 |
| `settings` | AppSettings | 应用级全局设置（不随 Profile 切换），字段可缺省 |

### AppSettings

全局偏好，所有字段均有默认值（旧配置文件缺省该块也能正常加载）。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `auto_switch_on_overlay` | string | `"ask"` | 开始覆盖时是否自动隐藏配置窗口并切换到目标窗口：`"yes"` 自动隐藏并切换、`"no"` 保持显示、`"ask"` 每次询问 |
| `locale` | string | `"auto"` | UI 语言：`"zh-CN"` / `"en"`，`"auto"` 跟随系统语言 |
| `fullscreen_overlay` | boolean | `true` | 覆盖模式：`true` 全屏覆盖，`false` 仅跟随目标窗口区域 |
| `live_drag_preview` | boolean | `false` | 拖拽窗口时是否实时显示准心（仅窗口模式生效；关闭时拖拽期间隐藏，停止拖拽 1200ms 后恢复） |
| `gpu_acceleration` | boolean | `false` | 是否启用 WebView2 GPU 硬件加速（关闭可显著降低内存占用） |
| `update_channel` | string | `"stable"` | 自动更新通道：`"stable"` 正式版 / `"prerelease"` 尝鲜版 |
| `cn_mirror` | boolean | `false` | 是否使用中国大陆加速镜像（gh-proxy）访问 GitHub Release；简体中文用户首次启动时自动设为 `true` |
| `mirror_url` | string | `"https://v4.gh-proxy.org"` | 加速镜像站地址，可在设置中自定义 |
| `antialiasing` | boolean | `true` | 覆盖层抗锯齿：开启后曲线边缘更平滑，关闭可略微降低 CPU 开销 |
| `renderer_backend` | RendererBackend | `"cpu"` | 覆盖层渲染后端：`"cpu"` 手写 CPU 像素光栅化（默认，零额外依赖）/ `"svg"` 转 SVG 由 resvg/tiny-skia 光栅化（抗锯齿质量更高） |
| `quick_colors` | `[f32; 4] × 5` | 白绿蓝红橙 | 快捷颜色预设（5 色），配置页点击色块一键切换准心颜色 |
| `hotkey_bindings` | `(HotkeyAction, string)[]` | `[["toggle_overlay", "Ctrl+Alt+O"]]` | 快捷键绑定（动作 → 键位） |
| `telemetry_enabled` | boolean? | 缺省 | 遥测授权：字段缺失 = 尚未授权；`true` 允许匿名上报；`false` 拒绝（零网络请求）。序列化时 `None` 不写出 |
| `developer_mode` | boolean | `false` | 开发者模式解锁标志（设置窗口「开发」Tab 显示开关） |
| `material` | MaterialSettings | 见下表 | 物料运行时设置（「物料」Tab） |

### MaterialSettings

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `dynamic_enabled` | boolean | `true` | 动态物料总开关（运行时层）。与编译期总闸构成与门，详见[设置详解 · 物料](./settings.md#物料)。关闭后热生效、无需重启 |
| `fps` | integer? | 缺省 | 动画帧率档位（上限节拍）：缺省 = 跟随主屏刷新率（回退 60）；固定值仅接受 `30` / `60` / `120`。纯静态 profile 不受影响 |

### Profile

| 字段 | 类型 | 说明 |
|------|------|------|
| `layers` | Layer[] | 图层列表，按顺序绘制（前底层、后顶层）。新格式配置的主体 |
| `crosshair` | Crosshair? | 旧格式字段，新格式中为 `null` / 缺失。见[遗留格式](#遗留格式legacy) |
| `trigger` | TriggerRule | 进程触发规则（占位，尚未生效） |
| `settings_hotkey` | string | 打开设置面板的热键字符串 |
| `target_window` | string | 目标窗口标题（可选）。空字符串表示不跟随特定窗口 |

`layers` 为空且 `crosshair` 缺失也是合法配置（语义为「当前不显示任何锚点」）。

### Layer

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `id` | string | — | 图层内唯一标识（UUID 或简单序号字符串），不可为空 |
| `name` | string | — | 用户可读的图层名，不可为空 |
| `material` | MaterialRef | — | 图层引用的物料 |
| `params` | object | `{}` | 该图层实例的具体参数（JSON 对象，覆盖物料 `defaults()`） |
| `style` | LayerStyle | 白色 / 0.6 | 图层级样式（颜色 / 不透明度 / 混合模式） |
| `transform` | Transform2D | 恒等 | 几何变换（平移 / 缩放 / 旋转） |
| `visible` | boolean | `true` | 是否可见（`false` 时不参与渲染） |
| `locked` | boolean | `false` | 是否锁定（锁定后 UI 不可误改） |

### MaterialRef

物料来源，`kind` 标签区分两种形态：

| 形态 | 字段 | 说明 |
|------|------|------|
| `{ "kind": "builtin", "id": "builtin.cross" }` | `id` | 内置物料（二进制内嵌 `.rhai`），如 `builtin.cross` |
| `{ "kind": "user", "name": "user.my_dot" }` | `name` | 用户物料，位于 `%APPDATA%/Peregrine/materials/<name>.rhai` |

### LayerStyle

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `color` | `[f32; 4]` | `[1.0, 1.0, 1.0, 1.0]` | RGBA 颜色，各通道范围 `[0.0, 1.0]` |
| `opacity` | number | `0.6` | 图层整体不透明度 `[0.0, 1.0]` |
| `blend_mode` | BlendMode | `"normal"` | 混合模式，当前仅 `normal`（普通透明度混合，预留扩展） |

### Transform2D

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `offset_x` / `offset_y` | number | `0.0` | 平移（逻辑像素） |
| `scale` | number | `1.0` | 均匀缩放因子，必须为正 |
| `rotation_deg` | number | `0.0` | 围绕屏幕中心的旋转角度（度） |

### HotkeyAction

快捷键动作枚举（`snake_case`）：`toggle_overlay`（切换覆盖层）、`start_overlay` / `stop_overlay`（开 / 关覆盖层）、`cycle_color_next` / `cycle_color_prev`（切换颜色预设）、`set_color_1` ~ `set_color_5`（设置颜色 1~5）。

### TriggerRule

| 字段 | 类型 | 说明 |
|------|------|------|
| `enabled` | boolean | 是否启用触发器（占位） |
| `process_names` | `string[]` | 触发进程名列表，空数组表示不限制（占位） |

## 遗留格式（Legacy）

:::note[自动迁移]
旧格式（`crosshair` 单字段）配置文件**加载时即自动迁移**为 `layers`（单个等价图层）；下次保存配置时 `crosshair` 字段即消失、只保留 `layers`。**无需手工转换**。
:::

迁移映射（旧 `Crosshair` 字段 → 新图层 `params`）详见[图层](./layers.md)。以下字段表仅供查阅旧文件或历史备份使用。

### Crosshair（旧）

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `style` | CrosshairStyle | `"edge_rect"` | 视觉锚点样式 |
| `size` | number | `120.0` | 主尺寸（px）。边缘矩形用作宽度，十字准星用作臂长，边缘箭头用作大小 |
| `secondary_size` | number | `80.0` | 次尺寸（px）。边缘矩形用作高度 |
| `thickness` | number | `2.0` | 线条/矩形条厚度（px），必须为正 |
| `radius` | number | `0.0` | 圆形边缘标记半径（px）。`<=0` 时按 `thickness * 3` 自动计算 |
| `offset` | number | `0.0` | 元素距屏幕外侧的距离（px），必须为非负数 |
| `color` | `[f32; 4]` | `[1.0, 1.0, 1.0, 1.0]` | RGBA 颜色，各通道范围 `[0.0, 1.0]` |
| `opacity` | number | `0.6` | 不透明度，`0.0` 完全透明，`1.0` 不透明 |
| `gap` | number | `4.0` | 十字准星中心间隙（px） |
| `corner_radius` | number | `4.0` | 边缘矩形圆角半径（px） |
| `anchor` | Anchor | `"top"` | 边缘矩形的锚点位置 |
| `margin` | number | `0.0` | 边缘矩形与锚边外侧的边距（px） |
| `ring_radius_pct` | number | `0.05` | 中心圆环半径占屏幕高度的比例，范围 `[0.03, 0.08]` |
| `ring_style` | RingStyle | `"solid"` | 中心圆环线型 |
| `orb_positions` | integer | `3` | 自定义边缘标记/边缘箭头的位置位掩码：`TOP=1`、`BOTTOM=2`、`LEFT=4`、`RIGHT=8` |
| `random_mode` | RandomOrbMode | `"lock_on_start"` | 随机边缘标记工作模式 |
| `random_center_deviation` | number | `0.2` | 随机边缘标记相对屏幕中心的偏移范围，范围 `[0.1, 0.3]` |
| `random_radius_min` | number | `4.0` | 随机边缘标记最小半径（px），必须为正 |
| `random_radius_max` | number | `12.0` | 随机边缘标记最大半径（px），必须为正且不小于 `random_radius_min` |
| `random_orb_x` | number | `0.0` | `LockOnStart` 模式下已锁定的相对中心 X 偏移 |
| `random_orb_y` | number | `0.0` | `LockOnStart` 模式下已锁定的相对中心 Y 偏移 |
| `border_frame_style` | BorderFrameStyle | `"solid"` | 边框样式 |
| `border_gap` | boolean | `false` | 边框四边中间是否留 20% 缺口 |
| `border_inset` | boolean | `true` | 边框矩形条是否位于屏幕内侧 |
| `custom_orb_top_count` | integer | `3` | 自定义边缘标记上边缘数量（1~10） |
| `custom_orb_bottom_count` | integer | `3` | 自定义边缘标记下边缘数量（1~10） |
| `custom_orb_left_count` | integer | `3` | 自定义边缘标记左边缘数量（预留） |
| `custom_orb_right_count` | integer | `3` | 自定义边缘标记右边缘数量（预留） |
| `random_orb_count` | integer | `3` | 随机边缘标记每边数量，必须为正 |
| `random_orb_offset` | number | `100.0` | 随机边缘标记距屏幕边缘的固定偏移（px） |
| `random_orb_jitter` | number | `40.0` | 随机边缘标记位置随机扰动范围（px） |
| `image_path` | string | `""` | 自定义图片的 PNG 文件路径，空字符串表示未选择 |
| `image_scale` | number | `1.0` | 自定义图片缩放比例，必须为正 |
| `image_offset_x` | number | `0.0` | 自定义图片相对屏幕中心的水平偏移（px） |
| `image_offset_y` | number | `0.0` | 自定义图片相对屏幕中心的垂直偏移（px） |
| `arrow_distance` | number | `0.0` | 边缘箭头距屏幕边缘的像素距离（`0` 表示贴边） |
| `arrow_width` | number | `0.0` | 边缘箭头尾宽（px），`0` 表示等于箭头大小 |
| `arrow_tail_per_edge` | boolean | `false` | 是否为每边单独设置尾部长度 |
| `arrow_tail_top` | number | `0.0` | 上边缘尾部长度（px） |
| `arrow_tail_bottom` | number | `0.0` | 下边缘尾部长度（px） |
| `arrow_tail_left` | number | `0.0` | 左边缘尾部长度（px） |
| `arrow_tail_right` | number | `0.0` | 右边缘尾部长度（px） |

### CrosshairStyle（旧）

枚举值（`snake_case`）：

| 枚举值 | 说明 |
|--------|------|
| `edge_rect` | 边缘矩形：可锚定屏幕四边或居中的半透明矩形 |
| `cross` | 十字准星：屏幕中心十字线 |
| `large_cross` | 大型十字准星：从屏幕边缘延伸到中心的横竖线 |
| `corner_dots4` | 四角边缘标记 |
| `corner_dots6` | 四角 + 上下中点边缘标记 |
| `corner_dots8` | 四角 + 上下左右中点边缘标记 |
| `ring` | 中心圆环 |
| `custom_orb` | 自定义边缘标记 |
| `random_orb` | 随机边缘标记 |
| `border_frame` | 边框 |
| `custom_image` | 自定义 PNG 图片 |
| `edge_arrows` | 边缘箭头 |
| `grid` | 网格：全屏棋盘式格子 |

> 旧配置文件中的 `toilet_paper` 仍会被识别为 `edge_rect`，保存后会写入 `edge_rect`。

### 其他旧枚举

- **Anchor**（锚点，`snake_case`）：`top`、`bottom`、`left`、`right`、`center`。
- **RingStyle**（圆环线型）：`solid`（实线）、`dashed`（虚线）、`double`（双线）。
- **RandomOrbMode**（随机球模式）：`lock_on_start`（启动后固定）、`reshuffle`（每次启动重新随机）。
- **BorderFrameStyle**（边框样式）：`solid`（完整四边）、`gap`（四边中间留缺口）。

## 热重载

配置文件被外部编辑器修改并保存后，`ConfigWatcher` 会在约 300ms 去抖后检测变更，并通过 `ConfigNotifier` 广播新配置，渲染器立即使用最新设置，无需重启。
