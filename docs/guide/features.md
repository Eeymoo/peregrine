# Features

## Overlay Mode

In overlay mode, Peregrine draws a visual anchor on top of the screen. The overlay window has the following characteristics:

- **Transparent**: The background is fully transparent, showing only the visual anchor or decal.
- **Always on top**: It stays above other windows at all times.
- **Click-through**: It does not intercept mouse clicks or keyboard input, so your gameplay is unaffected.
- **Window following**: In window mode, you can choose to follow a specific game window.

## Overlay Modes

Peregrine supports two overlay modes, switchable from the settings page or the tray menu. Both sides stay in sync:

- **Fullscreen Mode** (default): The overlay covers the entire screen, no target window is required. This is suitable for fullscreen gaming across multiple monitors.
- **Window Mode**: The overlay covers only the target window area. After enabling **Window Mode**, you need to select a target window. The overlay will follow the target window's position and size.

::: tip Real-Time Display While Dragging
You can enable "Real-time display while dragging" in **Settings**. When enabled, the overlay follows window movement in real time during dragging. When disabled (default), the overlay resumes about 1200 ms after dragging stops, reducing CPU usage.
:::

## Visual Anchor Styles

`CrosshairStyle` currently supports the following styles:

| Style | Description |
|------|------|
| `edge_rect` | Edge rectangle, classic visual anchor |
| `cross` | Crosshair |
| `large_cross` | Large crosshair |
| `corner_dots4` | Corner dots (4) |
| `corner_dots6` | Corner dots (6) |
| `corner_dots8` | Corner dots (8) |
| `ring` | Center ring |
| `custom_orb` | Custom edge markers |
| `random_orb` | Random edge markers |
| `border_frame` | Border frame |
| `custom_image` | Custom image |
| `edge_arrows` | Edge arrows |

## Custom PNG Decals

You can load any PNG image as a visual anchor. The program decodes it and draws it at the center of the screen.

## Settings & Preview

The settings panel is built with Tauri + React, providing:

- Real-time parameter adjustment
- Instant preview
- Auto-save and config validation

### Settings

The following settings are stored in the standalone **Settings** window (not profile-specific) and do not change when switching profiles:

- **Real-time display while dragging**: When enabled, the overlay follows window movement in real time while dragging. When disabled (default), it resumes after a delay when dragging stops, reducing CPU usage.
- **GPU hardware acceleration**: When enabled, WebView2 GPU rendering is used (disabled by default). When disabled, pure CPU rendering is used to reduce GPU process memory usage.
- **Auto-switch to game when starting overlay**: Ask every time / Yes / No.
- **Language**: Simplified Chinese / English / Follow system.

## Process Trigger (Placeholder)

`TriggerRule` (auto-enable by process) is already defined in the configuration model, but it has not yet been connected to the platform API. It will be implemented in a future version.
