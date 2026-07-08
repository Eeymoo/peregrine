# 配置说明

配置文件为 JSON 格式，位于：

- Windows：`%APPDATA%\Peregrine\config.json`
- macOS：`~/Library/Application Support/Peregrine/config.json`
- Linux：`~/.config/Peregrine/config.json`

首次运行会自动生成默认配置。你可以直接编辑该文件，保存后程序会在约 300ms 去抖后自动热重载，无需重启。

## 配置结构

```json
{
  "active_profile": "default",
  "profiles": {
    "default": {
      "crosshair": {
        "style": "edge_rect",
        "size": 120.0,
        "secondary_size": 80.0,
        "thickness": 2.0,
        "radius": 0.0,
        "offset": 0.0,
        "color": [1.0, 1.0, 1.0, 1.0],
        "opacity": 0.6,
        "gap": 4.0,
        "corner_radius": 4.0,
        "anchor": "top",
        "margin": 0.0,
        "ring_radius_pct": 0.05,
        "ring_style": "solid",
        "orb_positions": 3,
        "random_mode": "lock_on_start",
        "random_center_deviation": 0.2,
        "random_radius_min": 4.0,
        "random_radius_max": 12.0,
        "random_orb_x": 0.0,
        "random_orb_y": 0.0,
        "border_frame_style": "solid",
        "border_gap": false,
        "border_inset": true,
        "custom_orb_top_count": 3,
        "custom_orb_bottom_count": 3,
        "custom_orb_left_count": 3,
        "custom_orb_right_count": 3,
        "random_orb_count": 3,
        "random_orb_offset": 100.0,
        "random_orb_jitter": 40.0,
        "image_path": "",
        "image_scale": 1.0,
        "image_offset_x": 0.0,
        "image_offset_y": 0.0,
        "arrow_distance": 0.0,
        "arrow_width": 0.0,
        "arrow_tail_per_edge": false,
        "arrow_tail_top": 0.0,
        "arrow_tail_bottom": 0.0,
        "arrow_tail_left": 0.0,
        "arrow_tail_right": 0.0
      },
      "trigger": {
        "enabled": true,
        "process_names": []
      },
      "settings_hotkey": "F10",
      "target_window": ""
    }
  }
}
```

## 字段说明

### AppConfig

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_profile` | string | 当前激活的 Profile 名称，必须在 `profiles` 中存在 |
| `profiles` | map | 所有 Profile，键为名称，至少包含一个 |

### Profile

| 字段 | 类型 | 说明 |
|------|------|------|
| `crosshair` | Crosshair | 辅助贴图配置 |
| `trigger` | TriggerRule | 进程触发规则（占位，尚未生效） |
| `settings_hotkey` | string | 打开设置面板的热键字符串 |
| `target_window` | string | 目标窗口标题（可选）。空字符串表示不跟随特定窗口，覆盖层将固定显示在屏幕中心 |

### Crosshair

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `style` | CrosshairStyle | `"edge_rect"` | 准心样式 |
| `size` | number | `120.0` | 主尺寸（px）。贴边矩形用作宽度，准星用作臂长，箭头用作大小 |
| `secondary_size` | number | `80.0` | 次尺寸（px）。贴边矩形用作高度 |
| `thickness` | number | `2.0` | 线条/矩形条厚度（px），必须为正 |
| `radius` | number | `0.0` | 圆形定位球半径（px）。`<=0` 时按 `thickness * 3` 自动计算 |
| `offset` | number | `0.0` | 元素距屏幕外侧的距离（px），必须为非负数 |
| `color` | `[f32; 4]` | `[1.0, 1.0, 1.0, 1.0]` | RGBA 颜色，各通道范围 `[0.0, 1.0]` |
| `opacity` | number | `0.6` | 不透明度，`0.0` 完全透明，`1.0` 不透明 |
| `gap` | number | `4.0` | 准星中心间隙（px） |
| `corner_radius` | number | `4.0` | 贴边矩形圆角半径（px） |
| `anchor` | Anchor | `"top"` | 贴边矩形的贴靠位置 |
| `margin` | number | `0.0` | 贴边矩形与贴边外侧的边距（px） |
| `ring_radius_pct` | number | `0.05` | 中心环半径占屏幕高度的比例，范围 `[0.03, 0.08]` |
| `ring_style` | RingStyle | `"solid"` | 中心环线型 |
| `orb_positions` | integer | `3` | 自定义定位球/箭头的位置位掩码：`TOP=1`、`BOTTOM=2`、`LEFT=4`、`RIGHT=8` |
| `random_mode` | RandomOrbMode | `"lock_on_start"` | 随机球工作模式 |
| `random_center_deviation` | number | `0.2` | 随机球相对屏幕中心的偏移范围，范围 `[0.1, 0.3]` |
| `random_radius_min` | number | `4.0` | 随机球最小半径（px），必须为正 |
| `random_radius_max` | number | `12.0` | 随机球最大半径（px），必须为正且不小于 `random_radius_min` |
| `random_orb_x` | number | `0.0` | `LockOnStart` 模式下已锁定的相对中心 X 偏移 |
| `random_orb_y` | number | `0.0` | `LockOnStart` 模式下已锁定的相对中心 Y 偏移 |
| `border_frame_style` | BorderFrameStyle | `"solid"` | 边框样式 |
| `border_gap` | boolean | `false` | 边框四边中间是否留 20% 缺口 |
| `border_inset` | boolean | `true` | 边框矩形条是否位于屏幕内侧 |
| `custom_orb_top_count` | integer | `3` | 自定义定位球上边缘数量（1~10） |
| `custom_orb_bottom_count` | integer | `3` | 自定义定位球下边缘数量（1~10） |
| `custom_orb_left_count` | integer | `3` | 自定义定位球左边缘数量（预留） |
| `custom_orb_right_count` | integer | `3` | 自定义定位球右边缘数量（预留） |
| `random_orb_count` | integer | `3` | 随机球每边数量，必须为正 |
| `random_orb_offset` | number | `100.0` | 随机球距屏幕边缘的固定偏移（px） |
| `random_orb_jitter` | number | `40.0` | 随机球位置随机扰动范围（px） |
| `image_path` | string | `""` | 自定义图片的 PNG 文件路径，空字符串表示未选择 |
| `image_scale` | number | `1.0` | 自定义图片缩放比例，必须为正 |
| `image_offset_x` | number | `0.0` | 自定义图片相对屏幕中心的水平偏移（px） |
| `image_offset_y` | number | `0.0` | 自定义图片相对屏幕中心的垂直偏移（px） |
| `arrow_distance` | number | `0.0` | 箭头距屏幕边缘的像素距离（`0` 表示贴边） |
| `arrow_width` | number | `0.0` | 箭头尾巴宽度（px），`0` 表示等于箭头大小 |
| `arrow_tail_per_edge` | boolean | `false` | 是否为每边单独设置尾巴长度 |
| `arrow_tail_top` | number | `0.0` | 上边尾巴长度（px） |
| `arrow_tail_bottom` | number | `0.0` | 下边尾巴长度（px） |
| `arrow_tail_left` | number | `0.0` | 左边尾巴长度（px） |
| `arrow_tail_right` | number | `0.0` | 右边尾巴长度（px） |

### CrosshairStyle

枚举值（`snake_case`）：

| 枚举值 | 说明 |
|--------|------|
| `edge_rect` | 贴边矩形：可贴靠屏幕四边或居中的半透明矩形 |
| `cross` | 准星：屏幕中心十字线 |
| `large_cross` | 大准星：从屏幕边缘延伸到中心的横竖线 |
| `corner_dots4` | 四角定位点 |
| `corner_dots6` | 四角 + 上下中点定位点 |
| `corner_dots8` | 四角 + 上下左右中点定位点 |
| `ring` | 中心环 |
| `custom_orb` | 自定义定位球 |
| `random_orb` | 随机分布球 |
| `border_frame` | 屏幕边框 |
| `custom_image` | 自定义 PNG 图片 |
| `edge_arrows` | 屏幕四边箭头 |

> 旧配置文件中的 `toilet_paper` 仍会被识别为 `edge_rect`，保存后会写入 `edge_rect`。

### Anchor

贴边位置枚举（`snake_case`）：`top`、`bottom`、`left`、`right`、`center`。

### RingStyle

中心环线型枚举（`snake_case`）：`solid`（实线）、`dashed`（虚线）、`double`（双环）。

### RandomOrbMode

随机球模式枚举（`snake_case`）：`lock_on_start`（启动后固定）、`reshuffle`（每次启动重新随机）。

### BorderFrameStyle

边框样式枚举（`snake_case`）：`solid`（完整四边）、`gap`（四边中间留缺口）。

### TriggerRule

| 字段 | 类型 | 说明 |
|------|------|------|
| `enabled` | boolean | 是否启用触发器（占位） |
| `process_names` | `string[]` | 触发进程名列表，空数组表示不限制（占位） |

## 热重载

配置文件被外部编辑器修改并保存后，`ConfigWatcher` 会在约 300ms 去抖后检测变更，并通过 `ConfigNotifier` 广播新配置，渲染器立即使用最新设置，无需重启。
