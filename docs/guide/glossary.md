# 术语表

本文档统一 Peregrine 项目中的中英文术语，确保用户界面、文档、代码注释三侧用词一致。

## 核心概念

| 中文 | 英文 | 说明 |
|------|------|------|
| 视觉锚点 | Visual Anchor | 屏幕上用于缓解 3D 眩晕的固定参照物，如十字准星、中心圆环、边缘矩形等。 |
| 覆盖层 | Overlay | 透明、置顶、鼠标穿透的悬浮窗口，用于在游戏画面上方显示视觉锚点。 |
| 配置窗口 | Config Window | 主窗口，用于调整视觉锚点样式、颜色、透明度、目标窗口等参数。 |
| 设置窗口 | Settings Window | 系统设置窗口，目前包含语言、关于等系统级选项。 |
| 目标窗口 | Target Window | 覆盖层要跟随的游戏或应用窗口。 |
| 跟随 | Follow | 覆盖层根据目标窗口的位置和尺寸实时调整自己的位置与大小。 |
| 系统托盘 | System Tray | 屏幕右下角（Windows）或菜单栏（macOS）的托盘图标区域。 |
| 热重载 | Hot Reload | 配置文件被外部修改保存后，程序自动加载最新配置，无需重启。 |
| 配置档 | Profile | 一套完整的配置集合，可在不同场景（如不同游戏）间切换。 |
| 进程触发 | Process Trigger | 按进程名自动启用/禁用覆盖层的规则，目前为占位功能。 |

## 视觉锚点样式

样式在代码中以 `snake_case` 枚举值存储，UI 与文档使用下表中的中文/英文名称。

| 枚举值 | 中文 | 英文 |
|--------|------|------|
| `edge_rect` | 边缘矩形 | Edge Rectangle |
| `cross` | 十字准星 | Crosshair |
| `large_cross` | 大型十字准星 | Large Crosshair |
| `corner_dots4` | 四角标记 | 4 Corner Markers |
| `corner_dots6` | 六角标记 | 6 Corner Markers |
| `corner_dots8` | 八角标记 | 8 Corner Markers |
| `ring` | 中心圆环 | Center Ring |
| `custom_orb` | 自定义边缘标记 | Custom Edge Markers |
| `random_orb` | 随机边缘标记 | Random Edge Markers |
| `border_frame` | 边框 | Border Frame |
| `custom_image` | 自定义图片 | Custom Image |
| `edge_arrows` | 边缘箭头 | Edge Arrows |

## 通用参数

| 中文 | 英文 | 说明 |
|------|------|------|
| 尺寸 | Size | 视觉锚点的主尺寸，不同样式含义不同。 |
| 次尺寸 | Secondary Size | 部分样式（如边缘矩形）的次要尺寸。 |
| 线宽 / 厚度 | Thickness / Line Width | 线条或矩形条的粗细。 |
| 颜色 | Color | RGBA 颜色，各通道范围为 `[0.0, 1.0]`。 |
| 透明度 / 不透明度 | Opacity | `0.0` 完全透明，`1.0` 完全不透明。 |
| 间隙 | Gap | 十字准星等样式中心的留空距离。 |
| 偏移距离 | Offset | 元素距屏幕边缘或中心的距离。 |
| 半径 | Radius | 圆形或圆环的半径。 |
| 锚点 | Anchor | 边缘矩形等样式的贴边位置（上/下/左/右/居中）。 |
| 边距 | Margin | 边缘矩形与屏幕边缘的间距。 |
| 圆环样式 | Ring Style | 中心圆环的线型：实线、虚线、双线。 |
| 边框样式 | Border Frame Style | 边框的样式：完整四边、四边中缝缺口。 |


