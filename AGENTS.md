# AGENTS.md

This file is intended for AI coding agents working in this repository. It helps you quickly understand the Peregrine project and make changes safely, even without prior context. The content is verified against the actual codebase; if anything conflicts with the code, the code takes precedence.

## Project Overview

Peregrine is a desktop visual anchor (overlay) tool. **Its primary purpose is to reduce 3D motion sickness**: it draws semi-transparent visual anchors at the center or edges of the screen, giving players a fixed reference point in 3D games to alleviate dizziness.

- Language / ecosystem: **Rust**, Cargo **workspace** (`resolver = "3"`, `edition = "2024"`, `rust-version = 1.85`, MIT licensed).
- Graphics stack: **Tauri** (settings window Webview) + **React + Tailwind + shadcn/ui** (settings panel). The overlay still uses `winit` + `softbuffer`; the original `wgpu` + `egui` implementation has been removed.
- Async runtime: `tokio` (configuration read/write, file hot-reload, background follow task).
- Target platform: **Windows** (x86 / x86_64 / ARM64). Overlay transparency / click-through / window-following capabilities are intentionally Windows-only and are not planned for other platforms.
- Current status: **v0.2.1 stable released**. Windows transparent, always-on-top, click-through overlay; target window following; 12 crosshair styles; custom PNG decals; multi-profile config; configuration hot-reload; in-app auto-updater (NSIS); GlitchTip telemetry; bilingual UI (zh-CN / en) are all functional. "Process trigger" remains a configuration placeholder.
- **Static layer rendering ENABLED; dynamic material input soft-disabled** (changes `disable-material-runtime`, `material-static-rendering`): static multi-layer rendering via layers + Rhai materials is active (`MATERIAL_RUNTIME_ENABLED = true`) — overlay and preview render `Profile.layers` through `build_layers_shapes`, and layer editing is WYSIWYG. Only the **dynamic** path is disabled (`MATERIAL_DYNAMIC_INPUT_ENABLED = false`, declared in both `crates/peregrine/src/lib.rs` and `src/lib/feature.ts`): no time/mouse/keyboard polling — materials evaluate with `DynamicContext::static_context()` (`version = 0`, so static-material eval caching is permanent and dynamic materials like `builtin.time` render frozen); dynamic redraw scheduling stays removed (event-driven only); the material picker hides `is_dynamic` materials. The settings UI allows free single-/multi-layer mode switching, and `layersMode` is persisted in `localStorage` ("however it was closed, that's how it reopens"). To re-enable dynamic materials, flip both `MATERIAL_DYNAMIC_INPUT_ENABLED` constants to `true`; to fully soft-disable the material runtime again, flip `MATERIAL_RUNTIME_ENABLED` to `false`. Gated sites are greppable via both constant names.

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
├── .github/workflows/    # ci.yml (three-platform compile + lint), release.yml (tag-based release), snapshot.yml (unsigned test builds), pages.yml (docs deployment)
├── .agents/skills/       # AI agent skill definitions (unified; opencode + Agent Skills spec)
├── src-tauri/            # peregrine-tauri: Tauri backend entry, tray, commands, overlay management
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── lib.rs             # Tauri startup entry: config init, tray, commands, watcher
│       ├── main.rs            # binary entrypoint, calls lib::run
│       ├── overlay.rs         # runs winit event loop on a dedicated thread to manage overlay
│       └── telemetry.rs       # GlitchTip/Sentry telemetry, safe_try! outlets, panic hook
├── package.json          # frontend npm dependencies (React / Vite / Tailwind / shadcn / Tauri JS API)
├── vite.config.ts
├── tailwind.config.ts
├── components.json
├── tsconfig.json
├── index.html
├── openspec/             # OpenSpec spec-driven development: changes/ (active + archive/), specs/ (capability specs), config.yaml
└── src/                  # frontend source
    ├── main.tsx
    ├── SettingsApp.tsx      # main settings window (tabs: general/overlay/hotkeys/update/about)
    ├── ConfigApp.tsx        # first-launch / error config window + telemetry consent dialog
    ├── index.css
    ├── i18n/                # locale JSON: locales/{zh-CN,en}.json + options.json
    ├── lib/
    │   ├── api.ts           # Tauri invoke wrapper
    │   ├── i18n.tsx         # in-house i18n (React context + Tauri event, NOT i18next)
    │   ├── shapes.ts        # frontend preview geometry calculations
    │   ├── feature.ts       # MATERIAL_DYNAMIC_INPUT_ENABLED frontend flag
    │   ├── telemetry.ts     # REPORT_CODES + frontend Sentry wrappers
    │   └── ...
    ├── hooks/               # React state hooks (useAppState / useConfigSave / useSettingsSync / ...)
    ├── types/config.ts      # TypeScript configuration types
    └── components/
        ├── Preview.tsx      # Canvas real-time preview
        ├── LayersEditor.tsx # layer editor (multi-layer mode)
        └── ui/              # shadcn/ui base components
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
- `crates/peregrine` (shared library): `peregrine_config` (path dep), `peregrine_material` (path dep), `winit` 0.30, `softbuffer` 0.4 (overlay CPU rasterization), `png` 0.17 (custom PNG crosshair decoding), `resvg` + `tiny-skia` (SVG backend), `serde` / `serde_json`, `tokio`, `tracing`, `thiserror` (platform layer `OverlayError`).
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
- **Frontend has no `lint`/`typecheck` npm scripts.** `npm run build` runs `tsc && vite build` (the `tsc` step is the typecheck). There is no ESLint/Prettier/i18next config — i18n is an in-house React-context implementation at `src/lib/i18n.tsx` (locale JSON lives in `src/i18n/locales/`). The `i18n-audit` skill checks hard-coded strings and bilingual key alignment.

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
- When adding a new configurable item, you usually need to update five places in sync: `schema.rs` (field + default + validation), `shapes.rs` (geometry shape definitions, `build_shapes`), `src/SettingsApp.tsx` + the relevant component under `src/components/settings/` (React settings panel controls), `src/lib/shapes.ts` (frontend preview geometry calculations), and `overlay_renderer.rs` (if a new primitive type is introduced, add rasterization). Both the preview (React Canvas) and the overlay (softbuffer) are based on the same geometry logic to ensure WYSIWYG.
- Concurrency: the configuration snapshot shared across tokio and winit threads uses **the standard library `std::sync::Mutex`** (comments explicitly state: avoids calling tokio `blocking_lock` inside the runtime thread, which would panic). Follow this convention; do not replace it with `tokio::sync::Mutex` casually.
- The configuration snapshot type is `ConfigSnapshot = Arc<AppConfig>`, shared via `Arc` to avoid deep copies.

## Testing Conventions

- Unit tests live in the same file as the code under test, inside `#[cfg(test)] mod tests`.
- `schema.rs` focuses on validation logic (size / opacity / range / enums), default values, and serde round-trip (including old-config compatibility); `storage.rs` uses `tempfile::tempdir()` for real file read/write tests (including corrupted-config recovery); `notifier.rs` verifies broadcast subscriptions and subscriber counts; `watcher.rs` verifies that external file changes are detected and broadcast.
- When modifying validation rules or defaults in `schema`, please update/add corresponding tests; after changing configuration structures, at least run `cargo test -p peregrine_config`.

## CI / CD and Release

Five workflows (`.github/workflows/`):

- **`ci.yml`**: triggered on push to main/master/dev or on pull requests. Runs 4 jobs in parallel:
  - `build` (Windows): `cargo test` (3 crates) + `npm ci && npm run build` + `cargo build --release` (x86_64-msvc)
  - `test-report` (Windows): uses `cargo-nextest` to run all workspace tests and generate **JUnit XML test reports** (published via `action-junit-report`, uploaded as artifacts, with summary in GitHub Step Summary)
  - `frontend-report` (Linux): TypeScript check + frontend build + build size report
  - `lint` (Linux): `cargo fmt --check` + `cargo clippy -- -D warnings` (3 crates)
  - `quality-gate`: aggregates all job results, fails if any check fails
  - CI does not package NSIS, avoiding Windows build failures due to missing signing keys.
- **`release.yml`**: triggered on pushes of `v*` tags. Builds Tauri release artifacts on Windows for i686 / x86_64 / aarch64 targets, including **NSIS installer (signed with Tauri updater) + portable zip + `latest.json` updater manifest**, then creates a GitHub Release with `softprops/action-gh-release`. CI reads `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from GitHub Secrets to sign the installer. Tags containing `-` (e.g., `v0.2.0-alpha.0`) are treated as prereleases; pure version tags (e.g., `v0.1.0`) are stable releases. The release body and updater `notes` are auto-generated by CI from commits between the current tag and the previous tag, grouped by conventional commit prefixes into "Added / Fixed / Changed / Build / Docs / Other"; if no commits are available, it falls back to the tag message or the most recent commit message.
- **`snapshot.yml`**: builds **unsigned test packages** (NSIS + portable zip) for Windows x86 / x64 / ARM64. Triggered on PR to main/master/dev or via `workflow_dispatch` (manual). **Version number is auto-generated** as `0.0.0-dev.<short_sha>` from the PR head / current commit SHA — agents do **not** need to bump `package.json` / `tauri.conf.json` / `Cargo.toml` version fields before a snapshot; CI overwrites them in-flight. Products are uploaded as **workflow artifacts** (not GitHub Releases), so they must be downloaded from the run detail page. No signing keys required — safe for PR contexts. Telemetry reports to the GlitchTip TEST project (`GLITCHTIP_DSN_TEST` mapped to `GLITCHTIP_DSN`/`VITE_GLITCHTIP_DSN`).
- **`pages.yml`**: deploys documentation on a **stable Release** publish or manual trigger. Uses Node 22 to run `npm ci` + `npm run docs:build` under `docs/`, uploads the artifact, and deploys to GitHub Pages. Prereleases do not automatically trigger documentation deployment.
- **`opencode.yml`**: triggers opencode cloud-agent runs on issue/PR comments (and auto-labels opened issues/PRs). Runs on `ubuntu-latest`; gated on `issue_comment` / `pull_request_review_comment` events for the opencode job. Not used for local agent sessions.

### Snapshot release workflow (test builds)

Use a snapshot to produce test packages without bumping versions or cutting a formal tag.

```bash
# 1. Commit and push your changes to any feature/dev branch first.
git push origin <branch>

# 2. Trigger the snapshot workflow on that branch.
gh workflow run snapshot.yml --ref <branch>

# 3. Capture the run ID from the most recent run.
gh run list --workflow=snapshot.yml --limit 1

# 4. Watch until complete (3 archs build in parallel, ~7 min).
gh run watch <run-id> --exit-status

# 5. Download artifacts from the run detail page (no GitHub Release is created).
#    Products: peregrine-0.0.0-dev.<sha>-windows-{x86,x64,arm64}-setup.exe + .zip
```

Key differences from `release.yml`: no version bump, no tag push, no signing, no GitHub Release, no auto-updater manifest. Only for testing.

The release workflow specification is in `.agents/skills/release/SKILL.md`: follow SemVer (major/minor/patch + `-alpha.N`/`-beta.N` prerelease suffixes), **stable releases use even version numbers** (0.2.2, 0.2.4, 0.2.6, ...), **preview releases use odd/prerelease version numbers** (0.2.3, 0.2.7-alpha.0, ...). Release notes are grouped into "Added / Fixed / Changed / Build". Before pushing a tag, confirm the version number and tag message with the user. The docs site changelog page (`docs/guide/changelog.md`) records stable releases and preview releases.

### Branch Strategy

- **main**: stable branch, contains only stable-release code. Merged after a stable release.
- **dev**: development branch, contains features under test (such as auto-updater). After testing passes, merge into main to publish a stable release.

### OpenSpec Workflow (spec-driven development)

This repo uses **OpenSpec** (`openspec/` directory + `.agents/workflows/opsx-*.md` + `.agents/skills/openspec-*`). Planning artifacts live under `openspec/changes/<name>/` (proposal / design / tasks / spec deltas); archived changes move to `openspec/changes/archive/`; canonical capability specs live in `openspec/specs/`. All artifacts are written in **Simplified Chinese** (enforced by `openspec/config.yaml`). Active changes are shipped `openspec` CLI commands (`openspec status --change "<name>" --json`, `openspec instructions apply --change "<name>" --json`, etc.).

**End-to-end OpenSpec lifecycle (propose → apply → archive) with GitHub integration.** Each change is tracked end-to-end by a GitHub issue + PR pair, recorded in the change's `.openspec.yaml`. The three slash workflows (`/opsx:propose` / `/opsx:apply` / `/opsx:archive`) enforce the rules below; each is mirrored in three locations — `.opencode/commands/opsx-*.md` (opencode slash command, `/opsx-*` syntax), `.agents/workflows/opsx-*.md` (`/opsx:*` syntax), and `.agents/skills/openspec-*/SKILL.md` (skill auto-trigger). Keep all three mirrors in sync when editing.

- **`.openspec.yaml` metadata keys** (written/maintained by the workflows):
  - `schema`, `created`, `status`, `note` — original OpenSpec fields.
  - `issue: <number>` — written by `/opsx:propose` (the tracking GitHub issue).
  - `branch: feature/<change-name>` — written by `/opsx:propose` (the intended working branch).
  - `pr: <number>` — written by `/opsx:apply` (the PR created after implementation).

- **`/opsx:propose` (a.k.a. `/opsx:new`) — creates the change + tracking issue**:
  1. Generates proposal / design / tasks / spec deltas as usual.
  2. Creates **one** GitHub tracking issue via `gh issue create --label feature,openspec`, body summarizing motivation/goals/non-goals/impact + the change path.
  3. Writes `issue:` and `branch:` into `.openspec.yaml`, and prepends a `> 跟踪 issue：#<n>` reference to `proposal.md`.
  4. If `gh` is unavailable, skips issue creation with a warning (does not block).
  - Issue template available at `.github/ISSUE_TEMPLATE/feature_proposal.yml` for manual entry.

- **`/opsx:apply` — branch FIRST, implement, open PR LAST**:
  1. **FIRST step (MANDATORY): create a dedicated working branch before any code change** — never implement on `main`/`master`. Check the current branch with `git branch --show-current`:
     - If on `main`/`master`: branch directly (`git checkout -b feature/<change-name>`) using the `branch:` recorded in `.openspec.yaml` (or `feature/<name>` if absent).
     - If **not** on `main`/`master`: **do not assume** — ask the user (via the `question` tool) whether to base the new branch off the **current branch** or off **`main`** (then `git checkout main && git pull --ff-only` before branching). Never just start committing to the user's current branch.
  2. Implement tasks until `tasks.md` is all `[x]`.
  3. **LAST step (FINAL, only when all tasks done): open the PR** — `gh pr create --base <default-branch> --head feature/<name> --title "feat(<name>): ..."` with body referencing the tracking issue (`Closes #<issue>`). The PR body auto-inherits `.github/PULL_REQUEST_TEMPLATE.md`. Write `pr: <number>` into `.openspec.yaml`, commit + push that metadata update on the same branch. Run local checks (`cargo fmt --check`, `cargo clippy`, `cargo test`, `npm run build`) before opening; do not open a PR on red. Open the PR exactly once per change.

- **`/opsx:archive` — PR-merge gate (HARD, NON-SKIPPABLE) + post-archive branch switch**:
  1. **PR-merge gate**: read `pr:` from `.openspec.yaml`; `gh pr view <n> --json state,mergedAt`. If `mergedAt: null` → **BLOCK, do not archive** (report `## Archive Blocked` and stop; the user must merge the PR first and re-run). Only changes without a `pr:` key (legacy / `gh` was unavailable during apply) may bypass after explicit user confirmation, with a warning in the summary.
  2. Run the usual artifact/task completion checks + delta-spec sync.
  3. Move the change to `archive/YYYY-MM-DD-<name>/`.
  4. **FINAL step**: `git checkout main` (fall back to `master`) and `git pull --ff-only`, so the next change starts from a clean default branch. Optionally offer to delete the merged feature branch after confirming with the user.

**Templates**: `.github/ISSUE_TEMPLATE/feature_proposal.yml` (OpenSpec proposal issue), `.github/PULL_REQUEST_TEMPLATE.md` (PR body with change-link / self-check sections). `.github/ISSUE_TEMPLATE/config.yml` keeps `blank_issues_enabled: false`.

### Auto-Updater

The project integrates `tauri-plugin-updater` (Rust plugin + frontend `@tauri-apps/plugin-updater`):
- NSIS installer users can automatically download and install new versions via "Settings → Check for Updates"; portable zip does not support this.
- The signing key pair is stored locally in `.tauri/` (excluded by `.gitignore`); the public key is written to `plugins.updater.pubkey` in `tauri.conf.json`.
- CI reads the private key from GitHub Secrets for signing; the `latest.json` manifest is auto-generated and uploaded by CI.
- If the private key and password are lost, you will no longer be able to publish auto-updatable releases; keep them backed up safely.

## Documentation Site

`docs/` is an **Astro 7 + Starlight 0.41** documentation site (bilingual: `root` = en, `zh-cn`; deployed to the custom domain `https://peregrine.aukcraft.org/`; `starlight-llms-txt` generates `llms.txt` / `llms-full.txt`). **Requires Node >= 22.12** (Astro 7). Local preview: `cd docs && npm ci && npm run dev`. Content lives in `docs/src/content/docs/` (16 guide pages + splash homepage per locale).

Design system (added by the docs-modern-redesign change; utility-class migration completed by docs-twcss-migration; aukcraft shell surface language applied by docs-aukcraft-shell):

- **Tailwind CSS v4** via `@tailwindcss/vite` + the official compat package `@astrojs/starlight-tailwind`. Entry: `src/styles/global.css` (official cascade-layer recipe, no full preflight). Brand tokens (accent = brand blue `#2563EB` scale; gray = aukcraft blue-tinted neutral ramp anchored at dark base `#0B0E11` / raised `#14181D` / light base `#F5F7F9`) are defined **only** in the `@theme` block there; Starlight consumes `--color-accent-*` / `--color-gray-*` natively. Never hand-write the same hex values elsewhere.
- **aukcraft shell surface language (docs-aukcraft-shell, supersedes the prior zinc / 0.75rem / shadow decisions)**: zero `box-shadow` everywhere (`--sl-shadow-*: none` in `global.css`); corner radius ≤ 4px site-wide (containers 4px, inline code/kbd 2px, no pill `rounded-full` buttons); layering is expressed only via surface-brightness difference + 1px hairline borders. Dual theme is blue-tinted near-black (`#0B0E11`) / near-white (`#F5F7F9`), never pure `#000`/`#fff`; raised surfaces in light may be pure white (`--sl-color-bg-nav`/`bg-sidebar` → `var(--color-white)`). Hairline tokens `--color-hairline-dark`/`-light` (8% alpha) are mapped to `--sl-color-hairline*`. Text colors are carried by the gray ramp (dark headings use `gray-50`, not pure white).
- **Landing signature components (landing pages only)**: `src/components/DotField.astro` (fixed dot-grid canvas, cursor-near dots fill brand blue, theme-aware hollow rings rebuilt on `data-theme` change via `MutationObserver`, static under `prefers-reduced-motion`, z-layer `--z-field`) and `src/components/FlightLine.astro` (SVG perimeter-trace stroke animation dropped inside `.flight` buttons). Primitive classes live in `global.css`: `.flight` / `.flight-line` / `.link-line` / `.micro` / `.serif-italic` (Newsreader italic) / `.serif-zh` (system CJK serif stack) / `.noise` (2.5% film grain). `DotField` + `.noise` are mounted **only** on the bilingual `index.mdx`; guide/download pages never carry them. Supporting tokens: `--z-field` / `--z-noise` / `--ease-lock` / `--motion-line-btn` / `--motion-line-link` / `--fl-duration`; `--flight-stroke` is theme-aware (dark `accent-400` for ≥4.5:1 contrast, light `accent-600`). CTA hierarchy: primary = `text-accent`, secondary = `text-ink`, repeats of the same destination downgrade to `.link-line`.
- **Custom-component styles are Tailwind utility-first (docs-twcss-migration, migration status: complete)**. All 6 custom components (`Header.astro` + `landing/*`) express styles as Tailwind v4 utilities; hand-written CSS is confined to the exemption list (design.md D5): (1) the verbatim-copied Starlight region of `Header.astro` (`.header` grid / `.title-wrapper` / `.right-group` / `.social-icons` — kept verbatim for the upgrade-diff workflow); (2) JS state hook classes (`DownloadTable` `is-active` / `is-hidden` — the *visual* state rides on `aria-pressed` attribute-selector utilities, ARIA is the single source of truth); (3) Starlight-internal / `:global` polish rules + aukcraft shell primitives that cannot be expressed as utilities (`starlight-polish.css`, the `.flight` / `.flight-line` / `.link-line` / `.noise` primitives and `[data-reveal]` block in `global.css`, plus the `.lh-title :global(em)` serif-italic rule in `LandingHero`).
- **Breakpoint alignment (D3)**: `@theme` declares `--breakpoint-md: 50rem` so Tailwind `md:` matches Starlight's `md` (50rem); `hidden md:flex` and `sl-hidden md:sl-flex` are interchangeable. `lg` (64rem) already matches.
- **Token mapping convention (D4)**: `--sl-color-*` references in components MUST go through utilities — compat-pack-covered colors use paired light/dark utilities (`--sl-color-gray-2` → `text-gray-700 dark:text-gray-300`, `--sl-color-text-accent` → `text-accent-600 dark:text-accent-400`, etc.); uncovered Starlight core vars use v4 arbitrary-value var shorthand (`border-(--sl-color-hairline)`). No new hand-written hex or duplicate token definitions.
- **Animation tokens (D6)**: `@theme` defines `--animate-rise` + `@keyframes rise`; components use `motion-safe:animate-rise` + arbitrary-value delays. `prefers-reduced-motion` collapse is carried by `motion-safe:` / `motion-reduce:` variants plus the reduced-motion blocks in `global.css` (flight trace / link-line / `[data-reveal]`); the only exemption CSS for motion that has no utility equivalent lives there.
- **Modernization boundary (D9)**: visual evolution stays within the existing design language — single accent (brand blue) locked, one radius system (≤4px), zero shadows, no em-dash additions, `MOTION_INTENSITY` capped at 4-5 with full reduced-motion collapse. Any new visual change must be explicitly enumerated and locked by verify assertions.
- **aukcraft family layout grammar (D10)**: landing sections open with the shared `SectionHeading.astro` (index + micro mono label + hairline rule); FeatureGrid uses the hairline-divider grid (`gap-px` + hairline background); label-type text is mono + uppercase + wide tracking; landing sections reveal on scroll via `Reveal.astro` (`IntersectionObserver` + `[data-reveal]`, 600ms rise, `html.js` gate, reduced-motion collapse). Product personality (brand blue + blue-tinted neutral dual theme, left-copy/right-shot hero, serif-italic keyword accents) stays independent from the org site; family consistency is expressed purely through visual language — no attribution copy or badges.
- **Docs-page polish**: `src/styles/starlight-polish.css` (typography rhythm, code blocks, asides, tables, links, sidebar, control overrides). Only CSS variables + `.sl-markdown-content` selectors + Starlight internal classes — this file is **entirely exempt** from the utility migration (D5-3); each retained rule documents its exemption reason in comments. Control overrides (sidebar pill, search box, theme-select) are L1 `customCss`-level (D7): each rule annotates the target Starlight version (0.41) + internal class name as an upgrade-check checklist — re-verify on Starlight upgrades. The **only** Starlight component overrides are `Hero` (`src/components/landing/LandingHero.astro`) and `Header` (`src/components/Header.astro`), both registered in `astro.config.mjs`. Do not add more overrides casually. The `Header` override implements the aukcraft flat header (header-aukcraft-nav): brand icon+text on the left; right side carries flat text nav links (Docs / Download / GitHub, no icons/arrows), an `EN / 中文` flat language toggle (no dropdown), a sun/moon icon theme toggle (writes `localStorage starlight-theme` + `<html data-theme>`, same protocol as Starlight ThemeProvider), and the Starlight `Search` component restyled icon-only via polish CSS (same hover transition as the nav links). The AukCraft org link lives in the landing `Footer.astro`, not the header. It copies the rest of the default Header structure verbatim — when upgrading Starlight, diff it against the new default `Header.astro` and re-sync. Note: the sidebar top-level `link` items (mobile mirror of the header nav) join Starlight's pagination sequence, so `guide/intro`, `guide/usage` and `download` (both locales) carry explicit `prev`/`next` frontmatter overrides to keep guide paging correct — keep them if the sidebar order changes.
- **Landing components**: `src/components/landing/` (LandingHero / FeatureGrid / HowItWorks / DownloadCta / DownloadTable / SectionHeading / Reveal), pure Astro + Tailwind utilities, no React islands; copy is passed via props from the bilingual `index.mdx` / `download.mdx` files. All icons come from `lucide-static` via `src/lib/icons.ts` (same path data as the main app's `lucide-react`); never hand-write SVG paths.
- **Download page**: `download.mdx` + `zh-cn/download.mdx` fetch the GitHub Releases API at build time (`src/lib/releases.ts`) and render version/asset links with zero hard-coded version numbers; on API failure the page degrades to a plain "view GitHub Releases" link. The zh-cn page passes `showProxy` to render the GitHub-proxy selector (multiple gh-proxy candidates). After each release, pages.yml rebuilds the site so the download page picks up new versions automatically.
- **Screenshot pipeline** (`docs/scripts/`): `mock-tauri.js` stubs the full Tauri IPC surface in a plain browser; fixtures in `docs/scripts/fixtures/` (checked in) were exported from the real Rust `build_layers_shapes` via a temporary example that was deleted after export — to regenerate, recreate a similar example calling `build_layers_shapes` + `MaterialRegistry::load_builtin` (see `openspec/changes/archive/*/design.md` D5 or the git history of this change). `capture-screenshots.mjs` renders the real settings UI (root `npm run dev`, ConfigApp window = the layers editor) in headless Chromium and writes `public/img/screenshots/settings-layers.png`. Note: the app UI is a fixed dark theme. Re-run with `npm run screenshots` (needs the root Vite dev server on :5199).
- **Acceptance checks**: `npm run verify` runs `verify-landing.mjs` + `verify-polish.mjs` (Playwright computed-style assertions over `dist/`; build first). Note for debugging: both scripts serve `dist/` with correct `text/css` MIME — a static server without the CSS MIME type makes the browser ignore the stylesheet and produces misleading "utilities not applied" results.

## Telemetry Module

Peregrine ships an **anonymous, opt-in** GlitchTip (Sentry-protocol) telemetry layer. Code is already stable and Windows-validated; user-facing privacy in `docs/guide/privacy.md` (zh-cn mirror), developer-facing Code registry in `docs/guide/report-codes.md` (docs site page, zh-cn mirror), dev guide in `docs/guide/development.md` (§ Telemetry Development).

**Module layout**:

- Rust backend: `src-tauri/src/telemetry.rs` — install_id lifecycle, pending storage, event anonymization, panic hook, `report_code` constants, startup / `safe_try!` error outlets. Dual-path implementation: normal impl behind `#[cfg(not(peregrine_disable_telemetry))]`, no-op stubs behind `#[cfg(peregrine_disable_telemetry)]`.
- Frontend: `src/lib/telemetry.ts` — `REPORT_CODES` constants, `initTelemetry`, `captureFrontendError`, Tauri command wrappers (`storePendingReport` / `authorizeUploadAll` / `testReport` / `restartApp`). `TELEMETRY_DSN_AVAILABLE` exported for UI gating.
- Build glue: `src-tauri/build.rs` — parses repo-root `.env.development`, propagates `GLITCHTIP_DSN_TEST` to the compile-time env, emits `peregrine_disable_telemetry` cfg when `PEREGRINE_DISABLE_TELEMETRY` is set.

**DSN injection**: DSN is injected at **build time**, never read from disk at runtime. Rust side uses `option_env!("GLITCHTIP_DSN_TEST")` (dev) / `option_env!("GLITCHTIP_DSN")` (release); frontend uses Vite `import.meta.env.VITE_GLITCHTIP_DSN_TEST` / `VITE_GLITCHTIP_DSN`. Missing/empty DSN → SDK does not initialize, **zero network traffic** (default for fresh clones).

**Privacy switch semantics**: `config.settings.telemetry_enabled` is `Option<bool>`:

- `None` (field absent, fresh install) → treated as **not authorized**, SDK stays uninitialized. The first-launch consent dialog (`ConfigApp.tsx`) is the only thing that flips it to `Some(true)` / `Some(false)`.
- `Some(false)` → SDK not initialized; `safe_try!` / panic hook fall back to local pending storage.
- `Some(true)` → SDK initialized when DSN is also available; on next launch, pending records are uploaded silently.

Changing the switch requires a restart (a "pending restart" badge is shown in the UI). The error-page "upload error report" button is a **one-time explicit authorization** that initializes the SDK just long enough to flush pending, then tears it down — it does **not** flip the persistent switch.

**`safe_try!` convention** (macro defined in `src-tauri/src/lib.rs`): wraps `Result`-returning calls on **key paths only** (file IO, render entry, window bridge, external call); Ok passes through, Err captures function name + caller file:line + sanitized message and reports via `telemetry::report_safe_try_error`, then returns the original Err so the caller can degrade. **Do not** wrap every method. The Code passed in **must** already be registered in `docs/guide/report-codes.md` (docs site) and the `report_code` module. Currently wired: `PGR-2101` (config IO) at 4 sites in `lib.rs`, `PGR-4101` (overlay render) at the render loop in `overlay.rs`.

**Code governance**: before adding any new report point, pick a free Code in the right number range, add it to `report_code` (Rust) or `REPORT_CODES` (frontend) **and** to `docs/guide/report-codes.md` in the same PR, then write the call site. Codes are stable once shipped — never renumber or reuse. See the docs site `report-codes` page for the full registry and the `docs/guide/development.md` Telemetry section for the dev workflow.

**Compile-time disable**: set `PEREGRINE_DISABLE_TELEMETRY=1` (any value) before building. `build.rs` emits the cfg, the entire `telemetry` module compiles to no-op stubs preserving API signatures but with no IO / no networking / no panic hook. Use this for builds that must contain no reporting code whatsoever.

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
