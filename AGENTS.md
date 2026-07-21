# AGENTS.md

This file is intended for AI coding agents working in this repository. It helps you quickly understand the Peregrine project and make changes safely, even without prior context. The content is verified against the actual codebase; if anything conflicts with the code, the code takes precedence.

## Project Overview

Peregrine is a desktop visual anchor (overlay) tool. **Its primary purpose is to reduce 3D motion sickness**: it draws semi-transparent visual anchors at the center or edges of the screen, giving players a fixed reference point in 3D games to alleviate dizziness.

- Language / ecosystem: **Rust**, Cargo **workspace** (`resolver = "3"`, `edition = "2024"`, `rust-version = 1.85`, MIT licensed).
- Graphics stack: **Tauri** (settings window Webview) + **React + Tailwind + shadcn/ui** (settings panel). The overlay still uses `winit` + `softbuffer`; the original `wgpu` + `egui` implementation has been removed.
- Async runtime: `tokio` (configuration read/write, file hot-reload, background follow task).
- Target platform: **Windows** (x86 / x86_64 / ARM64). Overlay transparency / click-through / window-following capabilities are intentionally Windows-only and are not planned for other platforms.
- Current status: **v0.1.0 stable released**. Windows transparent, always-on-top, click-through overlay; target window following; 12 crosshair styles; custom PNG decals; configuration hot-reload are all functional. "Process trigger" remains a configuration placeholder.

All code comments and documentation use **Simplified Chinese**. Please continue writing new comments, documentation, and commit message bodies in Chinese for consistency.

## Repository Structure

```
peregrine/
├── Cargo.toml            # workspace root: members, workspace.package, workspace.dependencies, build profiles
├── Cargo.lock
├── .gitignore            # ignores /target, *.log, .DS_Store, docs build artifacts, etc.
├── assets/               # application icon (icon.ico) and icon generation script (gen_icon.py)
├── docs/                 # VitePress documentation site (deployed to GitHub Pages)
│   ├── .vitepress/       # VitePress config (config.mts), theme, dist/ build output
│   ├── guide/            # user guide, introduction, quick start, features, configuration, development/build
│   ├── public/           # static assets (logo.svg, etc.)
│   ├── index.md          # documentation homepage
│   └── package.json      # vitepress + mermaid + llms plugins
├── .github/workflows/    # ci.yml (three-platform compile + lint), release.yml (tag-based release), pages.yml (docs deployment)
├── .agent/skills/        # AI agent skill definitions (release workflow specification)
├── src-tauri/            # peregrine-tauri: Tauri backend entry, tray, commands, overlay management
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── lib.rs             # Tauri startup entry: config init, tray, commands, watcher
│       ├── main.rs            # binary entrypoint, calls lib::run
│       └── overlay.rs         # runs winit event loop on a dedicated thread to manage overlay
├── package.json          # frontend npm dependencies (React / Vite / Tailwind / shadcn / Tauri JS API)
├── vite.config.ts
├── tailwind.config.ts
├── components.json
├── tsconfig.json
├── index.html
└── src/                  # frontend source
    ├── main.tsx
    ├── App.tsx              # settings panel main component
    ├── index.css
    ├── lib/
    │   ├── api.ts           # Tauri invoke wrapper
    │   └── shapes.ts        # frontend preview geometry calculations
    ├── types/config.ts      # TypeScript configuration types
    ├── components/
    │   └── Preview.tsx      # Canvas real-time preview
    │   └── ui/              # shadcn/ui base components
    └── ...
└── crates/
    ├── config/           # peregrine_config: pure logic crate (no UI / GPU / window code)
    │   └── src/
    │       ├── lib.rs        # module exports + unified error type ConfigError / Result
    │       ├── schema.rs     # configuration data structures AppConfig / Profile / Crosshair / Element / Layer / MaterialRef / Transform2D + validation + unit tests
    │       ├── storage.rs    # config file path management, atomic read/write, default config generation, legacy migration (includes inline dirs module)
    │       ├── migration.rs  # legacy Crosshair → Layer migration field mapping
    │       ├── rng.rs        # SimpleRng (cross-crate shared with material runtime)
    │       ├── notifier.rs   # config change broadcast based on tokio::sync::watch
    │       └── watcher.rs    # config file hot-reload based on notify crate (with debouncing)
    ├── material/         # peregrine_material: Rhai material runtime (CPU-safe embedded scripting)
    │   ├── Cargo.toml
    │   ├── builtin/          # 11 built-in .rhai material scripts (cross / ring / edge_arrows / grid / image / ...)
    │   │   ├── cross.rhai
    │   │   └── ...
    │   └── src/
    │       ├── lib.rs             # module exports + BUILTIN_MATERIALS const (include_str!)
    │       ├── material.rs        # Material struct: load() / evaluate() / Rhai Engine + host function registration
    │       ├── registry.rs        # MaterialRegistry: builtin + user material loading and lookup
    │       ├── context.rs         # DynamicContext: time_ms / mouse_pos / key_down / rng_seed
    │       └── error.rs           # MaterialError / MaterialResult
    └── peregrine/        # peregrine: shared library (reused by Tauri)
        ├── Cargo.toml        # provides lib only
        └── src/
            ├── lib.rs             # exports overlay_renderer / shapes / platform
            ├── overlay_renderer.rs # softbuffer (CPU pixel rasterization) **overlay** renderer, dual-path: legacy Crosshair fallback + new layers + material evaluation
            ├── shapes.rs           # dual entry: build_shapes (legacy) + build_layers_shapes (new); Shape is type alias for Element
            ├── svg_renderer.rs     # SVG backend (resvg + tiny-skia)
            └── platform/
                ├── mod.rs          # platform module entry + poll_dynamic_context(); compiled as a placeholder on non-Windows targets
                └── windows.rs      # Win32 API: transparency / always-on-top / click-through, target window lookup/following, GetCursorPos/GetAsyncKeyState for dynamic input
```

**Layering principle**: `peregrine_config` must not depend on any UI / GPU / window platform code (`winit` / `wgpu` / `egui`). Platform and rendering logic belong in the `peregrine` shared library and the `src-tauri` binary crate. Please preserve this boundary when making changes.

## Tech Stack and Key Dependencies

Dependency versions are declared centrally in the root `Cargo.toml` under `[workspace.dependencies]`; each crate references them with `{ workspace = true }`. Prefer adding new dependencies at the workspace level.

- `crates/config` (`peregrine_config`): `tokio` (features: sync/rt/rt-multi-thread/macros/time/fs), `serde` (derive), `serde_json`, `notify` 7.0, `thiserror` 2.0, `tracing`; dev dependency `tempfile`.
- `crates/material` (`peregrine_material`): `peregrine_config` (path dep), `rhai` 1.25 (features: `sync`), `ahash` 0.8, `serde`, `serde_json`, `tracing`, `thiserror`.
- `crates/peregrine` (shared library): `peregrine_config` (path dep), `peregrine_material` (path dep), `winit` 0.30, `softbuffer` 0.4 (overlay CPU rasterization), `png` 0.17 (custom PNG crosshair decoding), `serde` / `serde_json`, `tokio`, `tracing`, `thiserror` (platform layer `OverlayError`).
- `src-tauri` (`peregrine-tauri`, main entry): `peregrine` / `peregrine_config` (path deps), `tauri` 2.x (`tray-icon` feature), `tauri-plugin-dialog`, `tauri-build`, frontend artifacts (`dist/`).
- Frontend: `React` 18 + `Vite` 5 + `TypeScript` 5 + `Tailwind CSS` 3 + `shadcn/ui` + `@tauri-apps/api` / `@tauri-apps/cli` 2.x.
- `[target.'cfg(windows)'.dependencies]`: `windows` 0.58 (Win32 UI / Foundation / Gdi features).
- `[profile.dev]` sets `opt-level = 1` (speeds up debug runs of the graphics stack).
- `[profile.release]` enables `opt-level = "z"` + `lto` + `codegen-units = 1` + `strip` + `panic = "abort"` + `overflow-checks = false`, prioritizing binary size reduction and performance.

Note: `storage.rs` contains an **inline `dirs` module** that implements cross-platform configuration directories itself (Windows `%APPDATA%`, macOS `~/Library/Application Support`, Linux `$XDG_CONFIG_HOME` or `~/.config`). It is not the external `dirs` crate; do not add that dependency by mistake.

## Build, Run, and Test

Run these from the repository root (do not run inside `target/`):

```bash
# Install frontend dependencies (first time)
npm install

# Build the entire workspace
cargo build
cargo build --release

# Run the Tauri version
npx tauri dev

# Build Tauri release artifacts
npx tauri build

# Run all tests
cargo test

# Run tests with JUnit XML report (requires cargo-nextest)
cargo nextest run --workspace --profile ci

# Test only the config crate
cargo test -p peregrine_config

# Lint / format
cargo fmt
cargo clippy
```

- Currently, **all unit tests live in `crates/config`** (`schema.rs` / `storage.rs` / `notifier.rs` / `watcher.rs`), **`crates/material`** (`material.rs` / `context.rs`), and **`crates/peregrine`** (`shapes.rs`). The `src-tauri` binary crate has no tests yet.
- Tests involving tokio use `#[tokio::test]`; tests in `watcher.rs` require a multi-thread runtime and are annotated `#[tokio::test(flavor = "multi_thread")]`.
- `watcher` tests rely on real filesystem events and have a maximum 5-second timeout wait; they are integration-leaning and may occasionally be affected by the environment.

## Runtime Architecture (Tauri Version)

1. `src-tauri/src/lib.rs::run()`: initializes `tracing_subscriber` (console stderr + `%APPDATA%/Peregrine/peregrine.log` rolling file, default `info` level); locates the config file via `ConfigStorage::with_default_path()` and calls `load_or_create_default()`; constructs a `ConfigNotifier` from the config; starts `overlay::run_overlay_loop` on a dedicated thread to manage the overlay window; starts `ConfigWatcher` and syncs notifier changes to the shared snapshot and overlay thread; creates the Tauri app, configures the tray icon and commands, and runs the event loop.
2. **Settings window** (Tauri Webview): a normal bordered window (title "Peregrine 设置", logical size 960×560) hosting the React + Tailwind + shadcn/ui settings panel. Closing it **hides the window to the system tray** (`api.prevent_close()` + `window.hide()`).
3. **System tray** (Tauri tray): created at launch with a menu containing "Settings" and "Exit". Clicking "Settings" restores the window; clicking "Exit" terminates the app, and Tauri's `RunEvent::Exit` notifies the overlay thread to stop.
4. **Overlay window** (`src-tauri/src/overlay.rs`): runs a native `winit` event loop on a dedicated thread, creating a transparent, always-on-top, click-through window rendered by `OverlayRenderer` (softbuffer CPU rasterization). Created / destroyed via the Tauri commands `start_overlay` / `stop_overlay`. **A target window must be selected before creation**. On Windows, `platform::windows::setup_overlay_window` adds `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`.
5. **Target window following** (`platform::windows::follow_target_window`): on Windows, polls every 16 ms to align the overlay with the target window; hides the overlay when the target is minimized or not foreground; stops following when the target window is destroyed.
6. Configuration flow: frontend changes → Tauri command `save_config` → `ConfigStorage::save` (atomic write) + `ConfigNotifier::update` → `ConfigWatcher` detects the file change and broadcasts again → shared snapshot updated → overlay renderer reads it. The frontend gets the initial config via `get_config`.

### Configuration Model and Storage

- The configuration root is `AppConfig` (`active_profile` + multiple named `Profile`s). Each `Profile` contains `crosshair` (`Crosshair`), `trigger` (`TriggerRule`), `settings_hotkey`, and `target_window`.
- `Crosshair.style` (`CrosshairStyle`, 12 variants): edge-aligned rectangle `EdgeRect`, cross `Cross`, large cross `LargeCross`, corner dots `CornerDots4/6/8`, center ring `Ring`, custom orb `CustomOrb`, random orb `RandomOrb`, border frame `BorderFrame`, custom image `CustomImage`, and arrows `EdgeArrows`. Each style supports adjustable size, thickness, color, opacity, gap, edge position, margin, etc.; `CustomImage` additionally has path, scale, and offset.
- The configuration file is JSON; its path is determined by the OS standard directory:
  - Windows: `%APPDATA%/Peregrine/config.json`
  - macOS: `~/Library/Application Support/Peregrine/config.json`
  - Linux: `~/.config/Peregrine/config.json`
- Writes are atomic (temp file in the same directory + `rename`), and `AppConfig::validate()` is always called before writing to avoid persisting invalid config.
- `load_or_create_default`: if the file does not exist, a default config is generated; **if parsing or validation fails, no error is raised**. Instead, the corrupted file is backed up as `<name>.bak`, the app falls back to the default config, and the default is rewritten to ensure the program can always start.

## Code Style and Conventions

- Follow standard Rust style (default `cargo fmt` configuration). The repository does not customize rustfmt/clippy rules; in CI, `cargo clippy -p peregrine_config -- -D warnings` treats lint warnings in the config crate as errors.
- **All public items must have Chinese documentation comments** (`///`); module tops use `//!` to describe responsibilities. Please keep the same density of Chinese comments in new code.
- Error handling: library code uses the `thiserror`-defined `ConfigError` and `crate::Result<T>` uniformly; do not `panic`/`unwrap` in libraries (validation failures return `ConfigError::Validation`). The binary layer may use `expect`/`unwrap` for fatal initialization errors; the platform layer (`platform/windows.rs`) defines its own `OverlayError`.
- Logging: use `tracing` (`info!`/`warn!`/`error!`/`debug!`); do not add `println!`/`eprintln!`.
- Serialization compatibility: when adding fields to structures such as `Crosshair`, always add `#[serde(default)]` or `#[serde(default = "...")]` so old configuration files can still be deserialized (existing fields already use this pattern extensively; the `old_config_without_image_fields_loads` test specifically verifies this).
- Enum serialization uniformly uses `#[serde(rename_all = "snake_case")]`.
- When adding a new configurable item, you usually need to update five places in sync: `schema.rs` (field + default + validation), `shapes.rs` (geometry shape definitions, `build_shapes`), `src/App.tsx` (React settings panel controls), `src/lib/shapes.ts` (frontend preview geometry calculations), and `overlay_renderer.rs` (if a new primitive type is introduced, add rasterization). Both the preview (React Canvas) and the overlay (softbuffer) are based on the same geometry logic to ensure WYSIWYG.
- Concurrency: the configuration snapshot shared across tokio and winit threads uses **the standard library `std::sync::Mutex`** (comments explicitly state: avoids calling tokio `blocking_lock` inside the runtime thread, which would panic). Follow this convention; do not replace it with `tokio::sync::Mutex` casually.
- The configuration snapshot type is `ConfigSnapshot = Arc<AppConfig>`, shared via `Arc` to avoid deep copies.

## Testing Conventions

- Unit tests live in the same file as the code under test, inside `#[cfg(test)] mod tests`.
- `schema.rs` focuses on validation logic (size / opacity / range / enums), default values, and serde round-trip (including old-config compatibility); `storage.rs` uses `tempfile::tempdir()` for real file read/write tests (including corrupted-config recovery); `notifier.rs` verifies broadcast subscriptions and subscriber counts; `watcher.rs` verifies that external file changes are detected and broadcast.
- When modifying validation rules or defaults in `schema`, please update/add corresponding tests; after changing configuration structures, at least run `cargo test -p peregrine_config`.

## CI / CD and Release

Three workflows (`.github/workflows/`):

- **`ci.yml`**: triggered on push to main/master/dev or on pull requests. Runs 4 jobs in parallel:
  - `build` (Windows): `cargo test` (3 crates) + `npm ci && npm run build` + `cargo build --release` (x86_64-msvc)
  - `test-report` (Windows): uses `cargo-nextest` to run all workspace tests and generate **JUnit XML test reports** (published via `action-junit-report`, uploaded as artifacts, with summary in GitHub Step Summary)
  - `frontend-report` (Linux): TypeScript check + frontend build + build size report
  - `lint` (Linux): `cargo fmt --check` + `cargo clippy -- -D warnings` (3 crates)
  - `quality-gate`: aggregates all job results, fails if any check fails
  - CI does not package NSIS, avoiding Windows build failures due to missing signing keys.
- **`release.yml`**: triggered on pushes of `v*` tags. Builds Tauri release artifacts on Windows for i686 / x86_64 / aarch64 targets, including **NSIS installer (signed with Tauri updater) + portable zip + `latest.json` updater manifest**, then creates a GitHub Release with `softprops/action-gh-release`. CI reads `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from GitHub Secrets to sign the installer. Tags containing `-` (e.g., `v0.2.0-alpha.0`) are treated as prereleases; pure version tags (e.g., `v0.1.0`) are stable releases. The release body and updater `notes` are auto-generated by CI from commits between the current tag and the previous tag, grouped by conventional commit prefixes into "Added / Fixed / Changed / Build / Docs / Other"; if no commits are available, it falls back to the tag message or the most recent commit message.
- **`pages.yml`**: deploys documentation on a **stable Release** publish or manual trigger. Uses Node 22 to run `npm ci` + `npm run docs:build` under `docs/`, uploads the artifact, and deploys to GitHub Pages. Prereleases do not automatically trigger documentation deployment.

The release workflow specification is in `.agent/skills/release/SKILL.md`: follow SemVer (major/minor/patch + `-alpha.N`/`-beta.N` prerelease suffixes), **stable releases use odd version numbers** (0.1.1, 0.1.3, 0.1.5, 0.1.7, ...), **preview releases use even/prerelease version numbers** (0.1.8-alpha.0, ...). Release notes are grouped into "Added / Fixed / Changed / Build". Before pushing a tag, confirm the version number and tag message with the user. `CHANGELOG.md` records stable releases; `CHANGELOG_ALPHA.md` records preview releases.

### Branch Strategy

- **main**: stable branch, contains only stable-release code. Merged after a stable release.
- **dev**: development branch, contains features under test (such as auto-updater). After testing passes, merge into main to publish a stable release.

### Auto-Updater

The project integrates `tauri-plugin-updater` (Rust plugin + frontend `@tauri-apps/plugin-updater`):
- NSIS installer users can automatically download and install new versions via "Settings → Check for Updates"; portable zip does not support this.
- The signing key pair is stored locally in `.tauri/` (excluded by `.gitignore`); the public key is written to `plugins.updater.pubkey` in `tauri.conf.json`.
- CI reads the private key from GitHub Secrets for signing; the `latest.json` manifest is auto-generated and uploaded by CI.
- If the private key and password are lost, you will no longer be able to publish auto-updatable releases; keep them backed up safely.

## Documentation Site

`docs/` is a **VitePress**-based documentation site (`lang: en-US`, `base: /`, deployed to the custom domain `https://peregrine.eeymoo.com/`), including a mermaid diagram plugin and `vitepress-plugin-llms` (generates `llms.txt` / `llms-full.txt`). Local preview: `cd docs && npm ci && npm run docs:dev`. Content is in `docs/guide/` (user guide, introduction, quick start, features, configuration, development/build). The Simplified Chinese version is located under `docs/zh-cn/`.

## Security and Notes

- **Do not commit sensitive information**; configuration files live in the user directory and are not distributed with the repository.
- Config writes are atomic and validated; when modifying `storage.rs`, preserve the invariant of "validate before write, temp file + rename" to avoid corrupting user configs.
- `target/` is the Cargo build output directory and is ignored by `.gitignore`. All source changes, builds, and tests should target the repository root.

### Known Limitations (keep in mind when touching related code)

- Overlay transparency, always-on-top, and click-through behavior are implemented via the Win32 API in `platform/windows.rs`: transparency / click-through / always-on-top are set by winit window attributes (`with_transparent` + `set_cursor_hittest(false)` + `WindowLevel::AlwaysOnTop`); `setup_overlay_window` only adds `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW`. `overlay_renderer.rs` uses softbuffer CPU rasterization instead of a wgpu swapchain to avoid DirectComposition transparency pitfalls.
- **The overlay must have a target window selected before creation**: when `target_window` is empty, `create_overlay` returns immediately and does not create a fullscreen overlay.
- `TriggerRule` (process trigger) is already defined in the schema (`enabled` + `process_names`), but **the binary layer does not consume it** — there is currently no logic to automatically enable/hide the overlay based on foreground process names; it is purely a configuration placeholder.
- The two modes of `RandomOrb`, `LockOnStart` / `Reshuffle`, behave identically in the current rendering implementation — both regenerate every frame from a seed derived from configuration parameters (preview and overlay use the same `SimpleRng` implementation + seed for consistency). The `random_orb_x/y` persistent fields in the schema are defined but not yet consumed by the renderer; they will be wired up later.
- CI config tests and release compilation run on three platforms; the `windows` crate is declared under `[target.'cfg(windows)'.dependencies]` and is not pulled in on non-Windows targets.
