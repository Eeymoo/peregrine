# Features

## Overlay Mode

In overlay mode, Peregrine draws a visual anchor on top of the screen. Window characteristics include:

- **Transparent**: The background is fully transparent, showing only the visual anchor or image.
- **Always on top**: Always stays above other windows.
- **Click-through**: Does not intercept mouse clicks or keyboard input, so gameplay is unaffected.
- **Window follow**: Optionally follows a specified game window as it moves.

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

## Process Trigger (Placeholder)

`TriggerRule` (auto-enable by process) is already defined in the config model but has not yet been integrated with the platform API. It will be implemented in a future version.
