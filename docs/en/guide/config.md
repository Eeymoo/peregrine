---
title: Configuration
---

# Configuration

The configuration file is in JSON format and located at:

- Windows: `%APPDATA%\Peregrine\config.json`
- macOS: `~/Library/Application Support/Peregrine/config.json`
- Linux: `~/.config/Peregrine/config.json`

A default configuration is generated automatically on first launch. You can edit this file directly; after saving, the program will hot reload within about 300ms, with no restart required.

## Configuration Structure

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

## Field Reference

### AppConfig

| Field | Type | Description |
|------|------|------|
| `active_profile` | string | The currently active Profile name; must exist in `profiles` |
| `profiles` | map | All Profiles, keyed by name; must contain at least one |

### Profile

| Field | Type | Description |
|------|------|------|
| `crosshair` | Crosshair | Visual anchor configuration |
| `trigger` | TriggerRule | Process trigger rules (placeholder, not yet effective) |
| `settings_hotkey` | string | Hotkey string to open the settings panel |
| `target_window` | string | Target window title (optional). Empty string means no specific window is followed, and the overlay is fixed at the screen center |

### Crosshair

| Field | Type | Default | Description |
|------|------|--------|------|
| `style` | CrosshairStyle | `"edge_rect"` | Visual anchor style |
| `size` | number | `120.0` | Primary size (px). Used as width for Edge Rectangle, arm length for Crosshair, and arrow size for Edge Arrows |
| `secondary_size` | number | `80.0` | Secondary size (px). Used as height for Edge Rectangle |
| `thickness` | number | `2.0` | Line/bar thickness (px), must be positive |
| `radius` | number | `0.0` | Circular marker radius (px). `<=0` automatically computes as `thickness * 3` |
| `offset` | number | `0.0` | Distance of the element from the screen edge (px), must be non-negative |
| `color` | `[f32; 4]` | `[1.0, 1.0, 1.0, 1.0]` | RGBA color, each channel in `[0.0, 1.0]` |
| `opacity` | number | `0.6` | Opacity, `0.0` fully transparent, `1.0` fully opaque |
| `gap` | number | `4.0` | Crosshair center gap (px) |
| `corner_radius` | number | `4.0` | Edge Rectangle corner radius (px) |
| `anchor` | Anchor | `"top"` | Anchor position for Edge Rectangle |
| `margin` | number | `0.0` | Margin between Edge Rectangle and the screen edge (px) |
| `ring_radius_pct` | number | `0.05` | Center ring radius as a percentage of screen height, range `[0.03, 0.08]` |
| `ring_style` | RingStyle | `"solid"` | Center ring line style |
| `orb_positions` | integer | `3` | Bitmask for Custom Edge Markers / Edge Arrows positions: `TOP=1`, `BOTTOM=2`, `LEFT=4`, `RIGHT=8` |
| `random_mode` | RandomOrbMode | `"lock_on_start"` | Random edge markers mode |
| `random_center_deviation` | number | `0.2` | Random edge markers relative center deviation range, `[0.1, 0.3]` |
| `random_radius_min` | number | `4.0` | Random edge markers minimum radius (px), must be positive |
| `random_radius_max` | number | `12.0` | Random edge markers maximum radius (px), must be positive and not less than `random_radius_min` |
| `random_orb_x` | number | `0.0` | Locked relative center X offset in `lock_on_start` mode |
| `random_orb_y` | number | `0.0` | Locked relative center Y offset in `lock_on_start` mode |
| `border_frame_style` | BorderFrameStyle | `"solid"` | Border frame style |
| `border_gap` | boolean | `false` | Whether to leave a 20% gap in the middle of each border side |
| `border_inset` | boolean | `true` | Whether the border bars are drawn inside the screen |
| `custom_orb_top_count` | integer | `3` | Custom edge markers count on the top edge (1~10) |
| `custom_orb_bottom_count` | integer | `3` | Custom edge markers count on the bottom edge (1~10) |
| `custom_orb_left_count` | integer | `3` | Custom edge markers count on the left edge (reserved) |
| `custom_orb_right_count` | integer | `3` | Custom edge markers count on the right edge (reserved) |
| `random_orb_count` | integer | `3` | Random edge markers count per edge, must be positive |
| `random_orb_offset` | number | `100.0` | Fixed distance of random edge markers from the screen edge (px) |
| `random_orb_jitter` | number | `40.0` | Random position jitter range of random edge markers (px) |
| `image_path` | string | `""` | PNG file path for Custom Image; empty means not selected |
| `image_scale` | number | `1.0` | Custom image scale, must be positive |
| `image_offset_x` | number | `0.0` | Custom image horizontal offset from screen center (px) |
| `image_offset_y` | number | `0.0` | Custom image vertical offset from screen center (px) |
| `arrow_distance` | number | `0.0` | Edge arrows distance from screen edge (`0` means flush) |
| `arrow_width` | number | `0.0` | Edge arrows tail width (px); `0` means equal to arrow size |
| `arrow_tail_per_edge` | boolean | `false` | Whether to set tail length per edge individually |
| `arrow_tail_top` | number | `0.0` | Top edge tail length (px) |
| `arrow_tail_bottom` | number | `0.0` | Bottom edge tail length (px) |
| `arrow_tail_left` | number | `0.0` | Left edge tail length (px) |
| `arrow_tail_right` | number | `0.0` | Right edge tail length (px) |

### CrosshairStyle

Enum values (`snake_case`):

| Enum Value | Description |
|--------|------|
| `edge_rect` | Edge Rectangle: a semi-transparent rectangle anchored to any screen edge or centered |
| `cross` | Crosshair: a cross at the screen center |
| `large_cross` | Large Crosshair: horizontal and vertical lines extending from screen edges to the center |
| `corner_dots4` | 4 Corner Markers |
| `corner_dots6` | 4 corners + top/bottom middle markers |
| `corner_dots8` | 4 corners + top/bottom/left/right middle markers |
| `ring` | Center Ring |
| `custom_orb` | Custom Edge Markers |
| `random_orb` | Random Edge Markers |
| `border_frame` | Border Frame |
| `custom_image` | Custom PNG Image |
| `edge_arrows` | Edge Arrows |

> Old config files using `toilet_paper` will still be recognized as `edge_rect`; after saving they will be written as `edge_rect`.

### Anchor

Anchor position enum (`snake_case`): `top`, `bottom`, `left`, `right`, `center`.

### RingStyle

Center ring line style enum (`snake_case`): `solid`, `dashed`, `double`.

### RandomOrbMode

Random edge markers mode enum (`snake_case`): `lock_on_start` (fixed after first launch), `reshuffle` (re-randomized on every launch).

### BorderFrameStyle

Border frame style enum (`snake_case`): `solid` (full four sides), `gap` (mid-edge gaps on each side).

### TriggerRule

| Field | Type | Description |
|------|------|------|
| `enabled` | boolean | Whether the trigger is enabled (placeholder) |
| `process_names` | `string[]` | List of triggering process names; empty means no restriction (placeholder) |

## Hot Reload

After the configuration file is modified and saved by an external editor, `ConfigWatcher` detects the change after about 300ms of debouncing and broadcasts the new configuration via `ConfigNotifier`. The renderer immediately uses the latest settings without requiring a restart.
