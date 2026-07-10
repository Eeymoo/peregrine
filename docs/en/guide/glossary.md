# Glossary

This page standardizes the Chinese and English terminology used across Peregrine's UI, documentation, and code comments.

## Core Concepts

| English | Chinese | Description |
|---------|---------|-------------|
| Visual Anchor | 视觉锚点 | A fixed reference object on the screen used to reduce 3D motion sickness, such as a crosshair, center ring, or edge rectangle. |
| Overlay | 覆盖层 | A transparent, always-on-top, click-through window that displays the visual anchor above a game or application. |
| Config Window | 配置窗口 | The main window for adjusting visual anchor style, color, opacity, target window, and other parameters. |
| Settings Window | 设置窗口 | The system settings window, currently containing language and about options. |
| Target Window | 目标窗口 | The game or application window that the overlay follows. |
| Follow | 跟随 | The overlay updates its position and size in real time based on the target window. |
| System Tray | 系统托盘 | The tray icon area at the bottom-right (Windows) or menu bar (macOS). |
| Hot Reload | 热重载 | The program automatically loads the latest configuration after the config file is modified externally, without restarting. |
| Profile | 配置档 | A complete set of configurations that can be switched between different scenarios (e.g., different games). |
| Process Trigger | 进程触发 | Rules to automatically enable/disable the overlay based on process names; currently a placeholder feature. |

## Visual Anchor Styles

Styles are stored as `snake_case` enum values in code; the UI and documentation use the Chinese/English names below.

| Enum Value | English | Chinese |
|------------|---------|---------|
| `edge_rect` | Edge Rectangle | 边缘矩形 |
| `cross` | Crosshair | 十字准星 |
| `large_cross` | Large Crosshair | 大型十字准星 |
| `corner_dots4` | 4 Corner Markers | 四角标记 |
| `corner_dots6` | 6 Corner Markers | 六角标记 |
| `corner_dots8` | 8 Corner Markers | 八角标记 |
| `ring` | Center Ring | 中心圆环 |
| `custom_orb` | Custom Edge Markers | 自定义边缘标记 |
| `random_orb` | Random Edge Markers | 随机边缘标记 |
| `border_frame` | Border Frame | 边框 |
| `custom_image` | Custom Image | 自定义图片 |
| `edge_arrows` | Edge Arrows | 边缘箭头 |

## Common Parameters

| English | Chinese | Description |
|---------|---------|-------------|
| Size | 尺寸 | The primary size of the visual anchor; meaning varies by style. |
| Secondary Size | 次尺寸 | The secondary size for some styles (e.g., Edge Rectangle). |
| Thickness / Line Width | 线宽 / 厚度 | The thickness of lines or bars. |
| Color | 颜色 | RGBA color with each channel in `[0.0, 1.0]`. |
| Opacity | 透明度 / 不透明度 | `0.0` fully transparent, `1.0` fully opaque. |
| Gap | 间隙 | The empty space at the center of styles such as Crosshair. |
| Offset | 偏移距离 | The distance of an element from the screen edge or center. |
| Radius | 半径 | The radius of circles or rings. |
| Anchor | 锚点 | The edge attachment position for styles such as Edge Rectangle (top/bottom/left/right/center). |
| Margin | 边距 | The spacing between Edge Rectangle and the screen edge. |
| Ring Style | 圆环样式 | Center ring line style: solid, dashed, or double. |
| Border Frame Style | 边框样式 | Border frame style: solid four sides or mid-edge gaps. |

## Deprecated Terms

The following terms are no longer used and should be replaced with the standardized terms above:

- ~~corner dots~~ → **Corner Markers**
- ~~center ring~~ is already correct; do not use "ring" alone in user-facing text.
- ~~crosshair style~~ (when referring to all styles) → **visual anchor style**
- ~~settings mode~~ → **Config Window**
- ~~auxiliary overlay~~ → **visual anchor** (general) or the specific style name
