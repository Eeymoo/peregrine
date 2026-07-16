# Configuration Guide

The configuration file is in JSON format and is located at:

- Windows: `%APPDATA%\Peregrine\config.json`
- macOS: `~/Library/Application Support/Peregrine/config.json`
- Linux: `~/.config/Peregrine/config.json`

> Peregrine is a Windows-only tool. The config paths above are listed because the configuration crate uses OS-standard directories; the overlay and core user-facing features are implemented only on Windows.

A default configuration is generated automatically on first launch. You can edit this file directly. After saving, the program will hot-reload after about 300 ms of debouncing, with no restart required.

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
| `active_profile` | string | The name of the currently active profile. It must exist in `profiles`. |
| `profiles` | map | All profiles, keyed by name. Must contain at least one. |

### Profile

| Field | Type | Description |
|------|------|------|
| `crosshair` | Crosshair | Visual anchor configuration |
| `trigger` | TriggerRule | Process trigger rules (placeholder, not yet active) |
| `settings_hotkey` | string | Hotkey string to open the settings panel |
| `target_window` | string | Target window title (optional). An empty string means no specific window is followed, and the overlay stays fixed at the center of the screen |

### Crosshair

| Field | Type | Default | Description |
|------|------|--------|------|
| `style` | CrosshairStyle | `"edge_rect"` | Visual anchor style |
| `size` | number | `120.0` | Main size (px). Used as width for edge rectangles, arm length for crosshairs, and size for edge arrows |
| `secondary_size` | number | `80.0` | Secondary size (px). Used as height for edge rectangles |
| `thickness` | number | `2.0` | Line / bar thickness (px). Must be positive |
| `radius` | number | `0.0` | Radius of circular edge markers (px). When `<=0`, it is automatically calculated as `thickness * 3` |
| `offset` | number | `0.0` | Distance of the element from the outer screen edge (px). Must be non-negative |
| `color` | `[f32; 4]` | `[1.0, 1.0, 1.0, 1.0]` | RGBA color. Each channel is in the range `[0.0, 1.0]` |
| `opacity` | number | `0.6` | Opacity. `0.0` is fully transparent, `1.0` is fully opaque |
| `gap` | number | `4.0` | Center gap of the crosshair (px) |
| `corner_radius` | number | `4.0` | Corner radius of the edge rectangle (px) |
| `anchor` | Anchor | `"top"` | Anchor position of the edge rectangle |
| `margin` | number | `0.0` | Outer margin of the edge rectangle from the anchored edge (px) |
| `ring_radius_pct` | number | `0.05` | Center ring radius as a percentage of screen height. Range `[0.03, 0.08]` |
| `ring_style` | RingStyle | `"solid"` | Center ring line style |
| `orb_positions` | integer | `3` | Bitmask for custom edge marker / edge arrow positions: `TOP=1`, `BOTTOM=2`, `LEFT=4`, `RIGHT=8` |
| `random_mode` | RandomOrbMode | `"lock_on_start"` | Random edge marker working mode |
| `random_center_deviation` | number | `0.2` | Random edge marker offset range relative to the screen center. Range `[0.1, 0.3]` |
| `random_radius_min` | number | `4.0` | Minimum radius of random edge markers (px). Must be positive |
| `random_radius_max` | number | `12.0` | Maximum radius of random edge markers (px). Must be positive and no less than `random_radius_min` |
| `random_orb_x` | number | `0.0` | Locked relative center X offset in `LockOnStart` mode |
| `random_orb_y` | number | `0.0` | Locked relative center Y offset in `LockOnStart` mode |
| `border_frame_style` | BorderFrameStyle | `"solid"` | Border frame style |
| `border_gap` | boolean | `false` | Whether to leave a 20% gap in the middle of each border side |
| `border_inset` | boolean | `true` | Whether the border bars are placed inside the screen |
| `custom_orb_top_count` | integer | `3` | Number of custom edge markers on the top edge (1~10) |
| `custom_orb_bottom_count` | integer | `3` | Number of custom edge markers on the bottom edge (1~10) |
| `custom_orb_left_count` | integer | `3` | Number of custom edge markers on the left edge (reserved) |
| `custom_orb_right_count` | integer | `3` | Number of custom edge markers on the right edge (reserved) |
| `random_orb_count` | integer | `3` | Number of random edge markers per edge. Must be positive |
| `random_orb_offset` | number | `100.0` | Fixed offset of random edge markers from the screen edge (px) |
| `random_orb_jitter` | number | `40.0` | Random jitter range of random edge marker positions (px) |
| `image_path` | string | `""` | PNG file path for the custom image. An empty string means none is selected |
| `image_scale` | number | `1.0` | Custom image scale. Must be positive |
| `image_offset_x` | number | `0.0` | Horizontal offset of the custom image relative to the screen center (px) |
| `image_offset_y` | number | `0.0` | Vertical offset of the custom image relative to the screen center (px) |
| `arrow_distance` | number | `0.0` | Pixel distance of edge arrows from the screen edge (`0` means flush with the edge) |
| `arrow_width` | number | `0.0` | Tail width of edge arrows (px). `0` means equal to the arrow size |
| `arrow_tail_per_edge` | boolean | `false` | Whether to set the tail length separately for each edge |
| `arrow_tail_top` | number | `0.0` | Tail length on the top edge (px) |
| `arrow_tail_bottom` | number | `0.0` | Tail length on the bottom edge (px) |
| `arrow_tail_left` | number | `0.0` | Tail length on the left edge (px) |
| `arrow_tail_right` | number | `0.0` | Tail length on the right edge (px) |

### CrosshairStyle

Enum values (`snake_case`):

| Enum Value | Description |
|--------|------|
| `edge_rect` | Edge rectangle: a semi-transparent rectangle anchored to one of the four screen edges or centered |
| `cross` | Crosshair: cross lines at the center of the screen |
| `large_cross` | Large crosshair: horizontal and vertical lines extending from the screen edges to the center |
| `corner_dots4` | Four corner edge markers |
| `corner_dots6` | Four corners plus top and middle edge markers |
| `corner_dots8` | Four corners plus top, bottom, left, and right middle edge markers |
| `ring` | Center ring |
| `custom_orb` | Custom edge markers |
| `random_orb` | Random edge markers |
| `border_frame` | Border frame |
| `custom_image` | Custom PNG image |
| `edge_arrows` | Edge arrows |

> In older config files, `toilet_paper` is still recognized as `edge_rect` and will be written as `edge_rect` on save.

### Anchor

Anchor position enum (`snake_case`): `top`, `bottom`, `left`, `right`, `center`.

### RingStyle

Center ring line style enum (`snake_case`): `solid`, `dashed`, `double`.

### RandomOrbMode

Random edge marker mode enum (`snake_case`): `lock_on_start` (fixed after launch), `reshuffle` (re-randomized on every launch).

### BorderFrameStyle

Border frame style enum (`snake_case`): `solid` (full four sides), `gap` (gap in the middle of each side).

### TriggerRule

| Field | Type | Description |
|------|------|------|
| `enabled` | boolean | Whether the trigger is enabled (placeholder) |
| `process_names` | `string[]` | List of triggering process names. An empty array means no restriction (placeholder) |

## Hot Reload

After the config file is modified and saved in an external editor, `ConfigWatcher` detects the change after about 300 ms of debouncing and broadcasts the new config through `ConfigNotifier`. The renderer immediately uses the latest settings without requiring a restart.
