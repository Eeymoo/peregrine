# Material Scripting

A **material** is Peregrine's unit of visual style: a pure mapping from `(params, screen rect) → list of elements`. Materials are written as small Rhai scripts and live either embedded in the binary (built-ins like `builtin.cross`) or as `.rhai` files in your user materials directory (`user.<name>`).

This guide walks through the **five-step authoring workflow** and documents the full API surface. Every snippet here is extracted from working examples that ship under `crates/material/examples/` — drop any of them into your materials folder and it will load.

> **Status note**: static materials render normally in current builds. Dynamic input (`time_ms()` / `mouse_pos()` / `key_down()` / `rand()`) is soft-disabled by default — the functions still resolve but return a frozen snapshot, so dynamic materials render as a single static frame. See the [Dynamic input API](#dynamic-input-api) section for details and how to re-enable.

## The five-step authoring workflow

```
1. Pick primitives      →  which Element types will you emit?
2. Lay them out         →  compute coordinates from screen + params
3. Extract parameters   →  promote magic numbers to named params with sensible defaults
4. Declare defaults/schema →  expose them to the UI so users can tweak
5. Verify               →  load in Material::load + evaluate; check the preview
```

Each material is one `.rhai` file exporting **three** required top-level functions and one optional flag function.

## The three required functions

### `fn defaults() -> Map`

Returns the default parameter map. Every parameter your `build` reads **must** have a default here. Keys are strings; values are numbers, strings, booleans, or nested maps.

```rhai
fn defaults() {
    #{
        size: 24.0,
        thickness: 2.0,
        gap: 4.0,
    }
}
```

These defaults are merged with the per-layer `params` (layer values win) before `build` is called, so `build` can always assume every key exists.

### `fn schema() -> Array`

Returns an array of parameter descriptors. The settings UI auto-generates a control for each entry. Every key in `defaults()` should appear in `schema()` so the user can edit it.

```rhai
fn schema() {
    [
        #{key: "size", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}
```

### `fn build(params, screen) -> Array`

The heart of the material. Receives the merged parameter map and the screen rectangle, returns an array of Element maps. This function must be **pure**: same inputs → same output (except for explicitly declared dynamic inputs, see below).

```rhai
fn build(params, screen) {
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;
    [
        #{type: "rect", x: cx - 10.0, y: cy - 1.0, w: 20.0, h: 2.0},
    ]
}
```

### Optional: `fn is_dynamic() -> bool`

Declare `true` if the material reads any [dynamic input](#dynamic-input-api) (`time_ms`, `mouse_pos`, `key_down`, `rand`). Defaults to `false` when absent.

- `false` (or missing) → the renderer caches the result **permanently** (same params → same output forever). Best for static anchors.
- `true` → the renderer re-evaluates on dynamic-context changes. Hidden from the material picker while dynamic input is soft-disabled.

## Parameter widget types

Each schema entry uses exactly one `widget`. The UI renders the matching control.

| `widget` | Fields | Value type | UI control |
|---|---|---|---|
| `number` | `min`, `max`, `step` | float | numeric input with spin buttons |
| `slider` | `min`, `max`, `step` | float | range slider with live value |
| `color` | _(none)_ | `[r, g, b, a]` 0..=1 array | color picker |
| `select` | `options: [{value, label}]`, optional `"default"` | string | dropdown |
| `toggle` | _(none)_ | bool | switch / checkbox |
| `image_path` | _(none)_ | string (filesystem path) | file picker (PNG) |
| `text` | _(none)_ | string | free-form text input |

> The `color` widget stores an RGBA array. Most layer-level styling (color / opacity) is applied at the [layer](./layers) level and overrides whatever the material emits; material-level colors are mostly relevant for multi-color materials.

## Element types

`build` returns an array of object maps. Each map has a `type` field and type-specific geometry fields. Coordinates are in **logical pixels** relative to the overlay window. The supported types:

| `type` | Fields | Notes |
|---|---|---|
| `rect` | `x`, `y`, `w`, `h` | axis-aligned rectangle (top-left origin) |
| `circle` | `cx`, `cy`, `radius` | filled circle |
| `circle_stroke` | `cx`, `cy`, `radius`, `thickness` | stroked ring |
| `dashed_circle` | `cx`, `cy`, `radius`, `thickness`, `dash_len`, `gap_len` | dashed ring |
| `triangle` | `x1`, `y1`, `x2`, `y2`, `x3`, `y3` | filled triangle by 3 vertices |
| `polygon` | `points: [[x,y], ...]` or `[{0:x,1:y}, ...]` or `[{x,y}, ...]` | filled polygon |
| `line` | `x1`, `y1`, `x2`, `y2`, `thickness` | line segment with thickness |
| `text` | `x`, `y`, `content`, `font_size`, optional `font_weight` | text; `font_weight` is 100..=900 in multiples of 100, omit or `()` for default |
| `image` | `path`, `x`, `y`, `w`, `h` | PNG file; renderer decodes separately |

Anything else returns `MaterialError::UnknownElementType` at evaluate time.

## The `screen` argument

`screen` is a map with `min_x`, `min_y`, `max_x`, `max_y` — the rectangular region the overlay covers (typically the target window's client area). Compute centers and edges from it rather than hardcoding 1920×1080:

```rhai
let cx = (screen.min_x + screen.max_x) / 2.0;
let cy = (screen.min_y + screen.max_y) / 2.0;
let radius = (screen.max_y - screen.min_y) * params.ring_radius_pct;
```

## Dynamic input API

These host functions are registered on the Rhai engine and let a material react to time / mouse / keyboard / randomness. **They only produce changing output when `is_dynamic()` returns `true`** and dynamic input is enabled in the build.

| Function | Returns | Description |
|---|---|---|
| `time_ms()` | `int` | Milliseconds since process start (monotonic). Cheap, use for animations. |
| `now_ms()` | `int` | Current Unix timestamp in ms (real clock). Use with `format_time`. |
| `format_time(ms, fmt)` | `string` | Format a ms timestamp; supports `yyyy` `MM` `dd` `HH` `hh` `mm` `ss` `a` placeholders. |
| `mouse_pos()` | `Map {x, y}` | Current mouse position in logical screen coords. |
| `key_down(code)` | `bool` | Whether the given key is currently pressed. Codes: `"shift"` `"ctrl"` `"a"`..`"z"` `"0"`..`"9"` `"f1"`..`"f12"` `"space"` etc. (case-insensitive) |
| `rand()` | `float` | Deterministic pseudo-random in `[0, 1)`; advances an internal counter per call. |
| `rand_range(min, max)` | `float` | Random float in `[min, max)`. |
| `rand_int(max)` | `int` | Random integer in `[0, max)`. |

### Determinism and caching

- **Static materials** (`is_dynamic() == false`) are evaluated **once** per parameter set and cached forever (the cache key ignores the dynamic context). Reading any dynamic input from a static material returns a frozen value — don't do it.
- **Dynamic materials** are re-evaluated when the dynamic context's `version` changes (per-frame when dynamic input is enabled). The RNG is seeded from `(material_id, params_hash, frame_count)` so two materials of the same kind with the same params produce the same random sequence within a frame, while successive `rand()` calls within one evaluation return different values.

### Current soft-disable

In shipped builds the dynamic-input path is disabled by default (`MATERIAL_DYNAMIC_INPUT_ENABLED = false` in `crates/peregrine/src/lib.rs` and `src/lib/feature.ts`). The host functions still resolve (so dynamic materials load and evaluate without errors), but the `DynamicContext` passed in is `static_context()` — `time_ms` returns 0, `mouse_pos` returns screen center, `key_down` returns false, and `version` stays 0 so the cache is permanent. Dynamic materials therefore render as a single static frame. To re-enable live updates, flip both `MATERIAL_DYNAMIC_INPUT_ENABLED` constants to `true`. See `AGENTS.md` (search the constant names) for the full gating.

## Sandbox limits

Materials run in a locked-down Rhai engine:

| Limit | Value | Effect |
|---|---|---|
| `max_operations` | 1,000,000 | Bounds total work per evaluation; a tight infinite loop hits this and aborts with `MaterialError::Evaluation`. |
| `max_call_levels` | 64 | Recursion depth; deep recursion aborts. |
| `max_expr_depths` | 128 / 128 | Nesting depth for expressions / statements. |
| File IO | none | No `import`, no `eval_file`, no filesystem access. |
| Network | none | No network primitives. |
| Host state | read-only | `rand_seed(s)` exists for API compatibility but cannot mutate host state — the seed is derived from `(material_id, params, frame)`. |

Common error shapes and their causes:

| Error | Likely cause | Fix |
|---|---|---|
| `MaterialError::MissingFunction { function }` | forgot to define `defaults` / `schema` / `build` | add the missing function |
| `MaterialError::Parse` | Rhai syntax error | check the script against Rhai syntax (note: `#{...}` for maps, `let x = ...;`, no `return` outside function body) |
| `MaterialError::InvalidReturnType: expected Array` | `build` returned a single map or number | wrap it: `[#{}]` not `#{}` |
| `MaterialError::ElementField: missing field 'x'` | element map missing a required geometry field | recheck the Element type table above |
| `MaterialError::UnknownElementType` | typo in the `type` string | use one of the listed type names |
| `MaterialError::Evaluation: ...` | runtime error (e.g. overflow operations, type mismatch) | simplify the script; print-debug with `print` (it goes to tracing) |

## A complete runnable example

This is the entire `simple_cross.rhai` from `crates/material/examples/`. Copy it to `<app_data_dir>/materials/simple_cross.rhai` and it will appear in the material picker as `user.simple_cross`.

```rhai
// Name: 简易十字
// 静态物料示例：四段矩形组成的十字准心。
// 参数：
//   arm_length — 单臂长度（像素）
//   thickness  — 矩形粗细
//   gap        — 中心透明间隙

fn defaults() {
    #{
        arm_length: 20.0,
        thickness: 3.0,
        gap: 4.0,
    }
}

fn schema() {
    [
        #{key: "arm_length", label: "臂长", widget: "slider", min: 1.0, max: 200.0, step: 1.0},
        #{key: "thickness", label: "粗细", widget: "slider", min: 0.5, max: 20.0, step: 0.5},
        #{key: "gap", label: "中心间隙", widget: "slider", min: 0.0, max: 40.0, step: 1.0},
    ]
}

fn is_dynamic() {
    false
}

fn build(params, screen) {
    let arm = params.arm_length;
    let t = params.thickness;
    let g = params.gap / 2.0;
    let cx = (screen.min_x + screen.max_x) / 2.0;
    let cy = (screen.min_y + screen.max_y) / 2.0;

    [
        #{type: "rect", x: cx - arm, y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx + g,     y: cy - t / 2.0, w: arm - g, h: t},
        #{type: "rect", x: cx - t / 2.0, y: cy - arm,     w: t, h: arm - g},
        #{type: "rect", x: cx - t / 2.0, y: cy + g,       w: t, h: arm - g},
    ]
}
```

## Loading your material

Materials are discovered from `<app_data_dir>/materials/`:

| Platform | Path |
|---|---|
| Windows | `%APPDATA%/Peregrine/materials/` |
| macOS | `~/Library/Application Support/Peregrine/materials/` |
| Linux | `~/.config/Peregrine/materials/` |

Drop a `.rhai` file there; the file name (without extension) becomes the material id suffix (`my_cross.rhai` → `user.my_cross`). The directory is scanned at startup and on manual reload; user materials override built-ins of the same name.

More example materials (static, time-dynamic, input-dynamic) live under [`crates/material/examples/`](https://github.com/eeymoo/peregrine/tree/main/crates/material/examples) — they double as smoke tests and are verified by `cargo test -p peregrine_material`.

## See also

- [Layers](./layers) — how materials are stacked, transformed, and styled at the layer level.
- [Configuration](./config) — the `Profile.layers` JSON shape.
- [`REPORT_CODES.md`](./report-codes) — telemetry code registry (materials don't emit telemetry themselves, but related code paths do).
