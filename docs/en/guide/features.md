# Features

## Overlay Mode

In overlay mode, Peregrine draws an assistive image on top of the screen. Window characteristics include:

- **Transparent**: The background is fully transparent, showing only the visual anchor or image.
- **Always on top**: Always stays above other windows.
- **Click-through**: Does not intercept mouse clicks or keyboard input, so gameplay is unaffected.
- **Window follow**: Optionally follows a specified game window as it moves.

## Crosshair Styles

`CrosshairStyle` currently supports the following styles:

| Style | Description |
|-------|-------------|
| `EdgeRect` | Edge rectangle, classic visual anchor |
| `Cross` | Crosshair |
| `LargeCross` | Large crosshair |
| `CornerDots4` | Corner markers (4) |
| `CornerDots6` | Corner markers (6) |
| `CornerDots8` | Corner markers (8) |
| `Ring` | Center ring |
| `CustomOrb` | Custom edge markers |
| `RandomOrb` | Random edge markers |
| `BorderFrame` | Border frame |
| `CustomImage` | Custom image |
| `EdgeArrows` | Edge arrows |

## Custom PNG Image

You can load any PNG image as an assistive overlay; the program decodes it and draws it at the center of the screen.

## Settings and Preview

The settings panel is built with Tauri + React and provides:

- Real-time parameter adjustment
- Instant preview
- Auto-save and config validation

## Process Trigger (Placeholder)

`TriggerRule` (auto-enable by process) is already defined in the config model but has not yet been integrated with the platform API. It will be implemented in a future version.
