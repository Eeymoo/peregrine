---
title: "Settings Guide"
description: Item-by-item walkthrough of every tab (General / Overlay / Material / Hotkeys / Update) in the Peregrine settings window.
---

The settings window is opened from the tray menu ("Settings") or the config panel. It contains 7 tabs: **General / Overlay / Material / Hotkeys / Update / About / Dev** (the "Dev" tab appears only after developer mode is unlocked). This page covers the first five; for the full field definitions see the [Configuration Guide](./config.md).

```text
Settings window
├── General   Language / GPU acceleration / Telemetry
├── Overlay   Start behavior / Drag preview / Anti-aliasing / Renderer backend / Quick colors
├── Material  Dynamic material switch / Animation FPS
├── Hotkeys   Action ↔ key bindings
├── Update    Update channel / Check for updates / Mirror
├── About     (not covered here)
└── Dev       Shown after developer-mode unlock (not covered here)
```

## General

![General settings](/img/screenshots/settings-general.png)

| Setting | Field | Default | Effective |
|---------|-------|---------|-----------|
| Language | `settings.locale` | `auto` | Instant |
| GPU acceleration | `settings.gpu_acceleration` | Off | Restart required |
| Telemetry | `settings.telemetry_enabled` | Not consented | Restart required |

- **Language**: `auto` follows the system language; you can also pin it to Simplified Chinese, English, etc. Switching away from Simplified Chinese automatically disables the mainland-China mirror.
- **GPU acceleration**: Controls WebView2 hardware acceleration for the settings window. Off by default, which drops the GPU process memory from ~80 MB to ~15 MB; enable it if the UI feels sluggish. After switching, the app asks whether to restart immediately.
- **Telemetry**: Consent switch for anonymous crash reports + launch statistics (see [Privacy & Telemetry](./privacy.md)). Changes take effect after a restart; choosing "restart later" shows a pending-restart badge. The whole item is hidden when the build has no DSN injected (e.g. self-compiled).

## Overlay

![Overlay settings](/img/screenshots/settings-overlay.png)

| Setting | Field | Default | Effective |
|---------|-------|---------|-----------|
| On overlay start | `settings.auto_switch_on_overlay` | `ask` | Instant |
| Live drag preview | `settings.live_drag_preview` | Off | Instant |
| Anti-aliasing | `settings.antialiasing` | On | Instant |
| Renderer backend | `settings.renderer_backend` | `cpu` | Instant |
| Quick colors | `settings.quick_colors` | white/green/blue/red/orange | Instant |

- **On overlay start**: Controls what happens to the config window when you click "Start overlay" — `Ask` confirms each time, `Auto switch` hides the config window and focuses the target window, `Keep shown` does nothing.
- **Live drag preview**: Window mode only (non-fullscreen overlay). When off, the crosshair hides while dragging and returns about 1.2 s after release, avoiding wobble while the window follows your hand; when on, it stays visible throughout.
- **Anti-aliasing**: Smooths the edges of circles, rings, triangles and other curves; disable it on low-performance machines to slightly reduce CPU cost.
- **Renderer backend**: How the two backends compare:

| | CPU (default) | SVG |
|---|---|---|
| Implementation | Hand-written CPU pixel rasterization | Primitives converted to SVG, rasterized by resvg / tiny-skia |
| AA quality | Moderate (with the anti-aliasing switch on) | Higher |
| Dependencies | None extra | Ships with the built-in SVG backend |
| Best for | Most cases | When curved-edge quality matters |

- **Quick colors**: 5 customizable color presets. Click a swatch to open a color picker; in the config window, clicking a swatch switches the crosshair color instantly — hotkeys can cycle or select them directly (see [Hotkeys](#hotkeys)). A reset button restores the defaults (white/green/blue/red/orange).

Layer management and per-layer parameters live in the config window's layer editor — see [Layers](./layers.md).

## Material

![Material settings](/img/screenshots/settings-material.png)

| Setting | Field | Default | Effective |
|---------|-------|---------|-----------|
| Dynamic materials | `settings.material.dynamic_enabled` | On | Instant (hot) |
| Animation FPS | `settings.material.fps` | Follow system | Instant (hot) |

### The two-layer AND gate

"Dynamic materials" are materials that change in real time based on runtime inputs (time, mouse position, key state, etc.), such as the `builtin.time` clock. Whether the dynamic pipeline is live is decided by **AND-ing two switches**:

```text
Compile-time switch MATERIAL_DYNAMIC_INPUT_ENABLED    Runtime user switch dynamic_enabled
                          │                                          │
                          └──────────────┐          ┌────────────────┘
                                         ▼          ▼
                                      ┌──────────┐
                                      │   AND    │────► dynamic pipeline live
                                      └──────────┘      (input polling / dynamic eval
                                                         context / continuous redraw /
                                                         dynamic materials in the picker)
```

- **Both on** (the default in official builds): dynamic materials redraw continuously at the FPS cadence.
- **Runtime switch off**: a user-side soft-off — dynamic materials are evaluated with a static context (frozen at one frame), effective immediately without a restart; input polling and periodic wake-ups stop.
- **Compile-time switch off** (special builds): the whole "Material" tab is hidden and the runtime switch has no consumer.

### Animation FPS semantics

| Option | Meaning |
|--------|---------|
| Follow system | Follows the primary monitor refresh rate (falls back to 60) |
| 30 / 60 / 120 | Fixed frame-rate cap; validation accepts only these three values |

FPS is an **upper-bound animation cadence (cap)**, not a forced frame rate: only profiles containing dynamic materials redraw periodically at the cadence; **purely static profiles stay event-driven and are unaffected**. FPS changes are hot-applied without a restart.

For writing Rhai material scripts see [Material Scripting](./material-scripting.md).

## Hotkeys

![Hotkeys settings](/img/screenshots/settings-hotkeys.png)

Bindable actions and their defaults:

| Action | Description | Default key |
|--------|-------------|-------------|
| `toggle_overlay` | Show / hide the overlay | `Ctrl+Alt+O` |
| `start_overlay` | Start the overlay | Unbound |
| `stop_overlay` | Stop the overlay | Unbound |
| `cycle_color_next` / `cycle_color_prev` | Cycle through quick color presets | Unbound |
| `set_color_1` ~ `set_color_5` | Select quick color 1–5 directly | Unbound |

**Recording interaction**: Click a row's key box to enter recording mode (highlighted border, "press a combo…" hint), then press a `Ctrl` / `Shift` / `Alt` / `Super` modifier plus a main key to bind (bare single keys cannot be bound, to avoid accidental triggers); press `Esc` to clear the row. A key cannot be bound twice — a new binding automatically removes the same key from any other action. Changes save immediately.

## Update

The "Update" tab offers a manual "Check for updates" button and two preferences:

- **Update channel** (`settings.update_channel`): `Stable` (releases only) or `Prerelease` (early pre-release builds).
- **Mirror acceleration** (`settings.cn_mirror`): Routes GitHub Release downloads through a gh-proxy mirror to improve speeds in mainland China; the mirror URL (`settings.mirror_url`) can be picked from presets or customized, defaulting to `https://v4.gh-proxy.org`.

In-app auto-update is available for the NSIS installer only; portable users should download new versions manually. See [Usage · Update](./usage.md).
