# Features

## Overlay Mode

In overlay mode, Peregrine draws a visual anchor on top of the screen. Window characteristics include:

- **Transparent**: The background is fully transparent, showing only the visual anchor or image.
- **Always on top**: Always stays above other windows.
- **Click-through**: Does not intercept mouse clicks or keyboard input, so gameplay is unaffected.
- **Window follow**: In window mode, optionally follows a specified game window as it moves.

## Overlay Mode

Peregrine supports two overlay modes, switchable from the config page or tray menu — both stay in sync:

- **Fullscreen mode** (default): The overlay covers the entire screen without needing to select a target window. Ideal for multi-monitor fullscreen gaming.
- **Window mode**: The overlay only covers the target window area. After enabling "Window Mode", select a target window and the overlay will follow its position and size.

::: tip Live Drag Preview
In Settings, you can enable "Live Drag Preview". When enabled, the overlay follows the window in real time during dragging. When disabled (default), the overlay reappears ~1200ms after dragging stops, reducing CPU usage.
:::

## Visual Anchor Styles

`CrosshairStyle` currently supports the following styles:

| Style | Description |
|-------|-------------|
| `edge_rect` | Edge Rectangle, classic visual anchor |
| `cross` | Crosshair |
| `large_cross` | Large Crosshair |
| `corner_dots4` | 4 Corner Markers |
| `corner_dots6` | 6 Corner Markers |
| `corner_dots8` | 8 Corner Markers |
| `ring` | Center Ring |
| `custom_orb` | Custom Edge Markers |
| `random_orb` | Random Edge Markers |
| `border_frame` | Border Frame |
| `custom_image` | Custom Image |
| `edge_arrows` | Edge Arrows |

## Custom PNG Image

You can load any PNG image as a visual anchor; the program decodes it and draws it at the center of the screen.

## Settings and Preview

The settings panel is built with Tauri + React and provides:

- Real-time parameter adjustment
- Instant preview
- Auto-save and config validation

### Settings

The following settings are located in the separate Settings window (not profile-specific):

- **Live Drag Preview**: When enabled, the overlay follows the window in real time during dragging. Disabled by default — the overlay reappears after dragging stops, reducing CPU usage.
- **GPU Hardware Acceleration**: When enabled, activates WebView2 GPU rendering (disabled by default). When off, pure CPU rendering is used to reduce GPU process memory usage.
- **Auto-switch to game on overlay start**: Ask / Yes / No.
- **Language**: Simplified Chinese / English / Follow System.

## Process Trigger (Placeholder)

`TriggerRule` (auto-enable by process) is already defined in the config model but has not yet been integrated with the platform API. It will be implemented in a future version.
