# Glossary

This document unifies Chinese-English terminology in the Peregrine project to ensure consistency across the user interface, documentation, and code comments.

## Core Concepts

| Chinese | English | Description |
|---------|---------|-------------|
| 视觉锚点 | Visual Anchor | A fixed reference on the screen used to relieve 3D motion sickness, such as crosshairs, center rings, edge rectangles, etc. |
| 覆盖层 | Overlay | A transparent, always-on-top, click-through floating window used to display visual anchors over the game screen. |
| 配置窗口 | Config Window | The main window for adjusting visual anchor style, color, opacity, target window, and other parameters. |
| 设置窗口 | Settings Window | The system settings window, currently containing system-level options such as language and about. |
| 目标窗口 | Target Window | The game or application window that the overlay follows. |
| 跟随 | Follow | The overlay adjusts its position and size in real time based on the target window's position and size. |
| 系统托盘 | System Tray | The tray icon area in the bottom-right corner (Windows) or menu bar (macOS). |
| 热重载 | Hot Reload | When the configuration file is modified and saved externally, the program automatically loads the latest configuration without restarting. |
| 配置档 | Profile | A complete set of configurations that can be switched between different scenarios (such as different games). |
| 进程触发 | Process Trigger | Rules for automatically enabling/disabling the overlay based on process names; currently a placeholder feature. |

## Visual Anchor Styles

Styles are stored in code as `snake_case` enum values; the UI and documentation use the Chinese/English names in the table below.

| Enum Value | Chinese | English |
|------------|---------|---------|
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

## General Parameters

| Chinese | English | Description |
|---------|---------|-------------|
| 尺寸 | Size | The main size of the visual anchor; meaning differs by style. |
| 次尺寸 | Secondary Size | The secondary size of some styles, such as edge rectangles. |
| 线宽 / 厚度 | Thickness / Line Width | The thickness of lines or rectangular bars. |
| 颜色 | Color | RGBA color; each channel range is `[0.0, 1.0]`. |
| 透明度 / 不透明度 | Opacity | `0.0` is fully transparent; `1.0` is fully opaque. |
| 间隙 | Gap | The empty distance at the center of styles such as crosshairs. |
| 偏移距离 | Offset | The distance of an element from the screen edge or center. |
| 半径 | Radius | The radius of a circle or ring. |
| 锚点 | Anchor | The edge position of styles such as edge rectangles (top / bottom / left / right / center). |
| 边距 | Margin | The spacing between edge rectangles and the screen edge. |
| 圆环样式 | Ring Style | The line style of the center ring: solid, dashed, or double. |
| 边框样式 | Border Frame Style | The style of the border frame: full four sides or four sides with center gaps. |
