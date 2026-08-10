# Layers

A **layer** is one material invocation plus its own parameters, transform, and style. A profile's visual anchor is the composition of all its layers stacked on top of each other. Layers are what you actually edit in the settings UI; materials are the reusable recipes they invoke.

This page covers the user-facing concepts. For authoring materials themselves, see [Material Scripting](./material-scripting); for the raw JSON shape, see [Configuration](./config).

## Why layers?

Before v0.2 each profile had exactly one crosshair style. Layers let you combine multiple materials into a single anchor: a center cross plus an edge frame; a ring plus corner dots; your custom PNG decal on top of a built-in grid. Each layer is independent and can be hidden, locked, reordered, or restyled without touching the others.

## Layer structure

Each layer has:

| Field | Type | Meaning |
|---|---|---|
| `id` | string | Layer-unique identifier (UUID or simple sequence). |
| `name` | string | Human-readable name shown in the UI. |
| `material` | `Builtin { id }` \| `User { name }` | Which material this layer invokes. |
| `params` | object | Per-layer parameter overrides merged on top of the material's `defaults()`. |
| `style` | `{ color, opacity, blend_mode }` | Layer-level color, opacity, and blend mode applied to the material's output. |
| `transform` | `{ offset_x, offset_y, scale, rotation_deg }` | Geometric transform applied after the material emits its elements. |
| `visible` | bool | Hidden layers are skipped at render time. |
| `locked` | bool | Locks the layer against accidental UI edits (no functional effect on rendering). |

### Stacking order

Layers render in **array order**: `layers[0]` is the **bottom**, `layers[N-1]` is the **top**. Later layers draw over earlier ones. Reordering in the UI rewrites the array.

### Transforms

`Transform2D` is applied to the material's output elements before styling:

- `offset_x` / `offset_y` — translation in logical pixels (default `0`).
- `scale` — uniform scale factor (default `1.0`).
- `rotation_deg` — rotation around the screen center in degrees (default `0`).

Transforms are handy for nudging a material to an off-center anchor, mirroring, or building symmetric patterns from a single material.

### Style

`LayerStyle` applies to everything the material emits:

- `color` — RGBA array in `[0, 1]` (default `[1, 1, 1, 1]`, white).
- `opacity` — layer opacity in `[0, 1]` (default `0.6`).
- `blend_mode` — currently only `normal` (`src over dst`); reserved for future blend modes (`add` / `multiply` / etc.).

> **Override semantics**: layer style overrides whatever color the material emits. If you need a multi-color anchor, use multiple layers (each with its own style) rather than trying to encode colors inside a single material.

### Visibility and locking

- **`visible: false`** skips the layer entirely at render time — its material is not even evaluated. Useful for keeping alternate designs in the same profile without deleting them.
- **`locked: true`** is a UI-only hint: the settings panel disables editing of that layer's fields so you don't accidentally nudge a carefully-tuned anchor. It has zero effect on rendering.

## Layers and Profiles

Each [`Profile`](./config) carries an ordered `layers` array. The active profile's layers are what the overlay renders. Switching profiles swaps the entire layer set.

The settings UI supports two modes:

- **Single-layer mode**: edit only `layers[0]`. This mirrors the pre-v0.2 single-crosshair experience and is the default view for migrated profiles.
- **Multi-layer mode**: full layer panel — add / remove / duplicate / reorder / hide / lock. The mode you close the panel in is remembered in `localStorage` and restored on next launch.

Both modes edit the same underlying `Profile.layers`; single-layer mode is just a focused view of `layers[0]`.

## Migrated profiles (single-layer shape)

When an older single-crosshair `config.json` is loaded, `migration.rs` converts the legacy `Crosshair.style` into exactly **one** layer referencing the corresponding built-in material:

```jsonc
{
  "layers": [
    {
      "id": "migrated_cross",
      "name": "准星",
      "material": { "kind": "builtin", "id": "builtin.cross" },
      "params": { "size": 24.0, "thickness": 2.0, "gap": 4.0 },
      "style": { "color": [1,1,1,1], "opacity": 0.6, "blend_mode": "normal" },
      "transform": { "offset_x": 0, "offset_y": 0, "scale": 1.0, "rotation_deg": 0 },
      "visible": true,
      "locked": false
    }
  ],
  // crosshair: null  ← 旧字段已清空，下次启动按新格式加载，不再重复迁移
}
```

Visually it is identical to the pre-v0.2 anchor. You can switch to multi-layer mode in the settings UI to add more layers on top.

## Building a composite anchor

A typical workflow for a multi-layer anchor:

1. Start with a base layer (e.g. `builtin.edge_rect` at the top).
2. **Add layer** → pick a material (e.g. `builtin.cross`) → adjust its `params`.
3. Style it: set `opacity` / `color` so it composites cleanly over the base.
4. Repeat — corner dots, a center ring, etc.
5. Reorder by dragging so the most important layer is on top.
6. Hide or lock anything you want to keep but not accidentally edit.

Each layer's parameters come from the material's `schema()`, so the UI shows material-specific controls (slider for arm length, dropdown for anchor, etc.). See [Material Scripting](./material-scripting) for what each material exposes.

## See also

- [Material Scripting](./material-scripting) — authoring custom materials.
- [Configuration](./config) — full JSON schema including `Profile.layers`.
- [Recommended Settings](./recommendations) — curated profiles that use multiple layers.
