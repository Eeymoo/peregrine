# Changelog

This page records every Peregrine release. Stable releases are listed first; preview (alpha/beta) releases follow in the **Preview Releases** section. Release assets are available on [GitHub Releases](https://github.com/Eeymoo/peregrine/releases).

---

## [v0.2.4] — 2026-08-13

Stable release. The UI goes international: Peregrine now ships in six languages with a data-driven i18n backend and a community translation pipeline (issue templates + an auto-PR workflow + a CI key-alignment gate). The OpenSpec spec-driven workflow is also now end-to-end integrated with GitHub (tracking issues / PRs / branching strategy).

### Added

- **6-language UI support**: Added Japanese (ja-JP), German (de-DE), French (fr-FR), and Russian (ru-RU) alongside Simplified Chinese and English. Non-Chinese translations are AI-generated and may be imperfect; native speakers can suggest improvements via the new translation-improvement issue template. The backend i18n was refactored from a hardcoded enum + match table to a data-driven model that embeds the same locale JSON as the frontend via `include_str!` — adding a new language is now a JSON-only change. `FALLBACK_LOCALE` switched from `zh-CN` to `en`. @Eeymoo
- **Issue templates**: Added a Bug Report template (default), a Translation Improvement template (feeds the auto-translate workflow), a Question template (catch-all since blank issues are disabled), and a config.yml gating the issue picker.
- **Translation automation**: A new `auto-translate` GitHub Actions job opens a PR editing the relevant locale JSON when an issue is labeled `translation`; an `i18n-check` CI job enforces JSON validity, single-locale key-set invariance, and 6-locale key-set alignment on PRs touching locale files.
- **OpenSpec ↔ GitHub integration**: The `/opsx:propose` / `/opsx:apply` / `/opsx:archive` workflow now creates a tracking issue, a dedicated feature branch, and a linked PR for each change; `.openspec.yaml` records the `issue` / `branch` / `pr` keys; archiving is gated on PR merge.

### Changed

- The `option.follow_system` label in the language dropdown now follows the active locale (previously always shown in Chinese). Language self-names (日本語 / Deutsch / …) are intentionally kept as fixed endonyms.

---

## [v0.2.2] — 2026-08-09

Multi-layer editing pipeline hardening release. Addresses nine issues surfaced by the v0.2.1 demo and real-world testing, taking the multi-layer path from "demoable" to "daily-usable".

### Added

- **`update_profile_field` backend command**: field-level patch updates to top-level profile fields (`target_window` / `settings_hotkey`) without touching `layers`, eliminating full-overwrite data loss. @Eeymoo
- **`BitmaskField` component**: edge bitmask for `custom_orb` / `edge_arrows` changed from raw numbers (0–15) to four checkboxes (up/down/left/right). @Eeymoo
- **`CornerDotsFields` count selector**: switch between 4/6/8 dots inline in the field panel in single-layer mode. @Eeymoo
- **i18n keys**: `fields.dotCount` + `cornerDotsCount.{4,6,8}` (zh-CN / en). @Eeymoo

### Fixed

- **Config loss root cause (#34 🔴 critical)**: removed full `saveConfig` overwrites on the multi-layer path; all profile field changes now go through the patch API. @Eeymoo
- **IPC error protocol (#27)**: error return type `Result<T, String>` → `Result<T, IpcError>` with structured `{code, message}`. @Eeymoo
- **Unified error feedback (#31)**: all layer operations now have try/catch + toast; no more silent error swallowing. @Eeymoo
- **Global dialog layering (#28)**: `AutoSwitchDialog` / `UpdateDialog` / `UpdateProgress` moved to the outermost layer of `ConfigApp`. @Eeymoo
- **Grid algorithm (#29)**: `grid.rhai` center mode now expands symmetrically from screen center and fills the screen (fixes "only in the middle"). @Eeymoo
- **Dead parameters (#30)**: `border_frame.inset` / `random_orb.center_deviation` implemented; `random_orb.mode` marked `coming_soon`. @Eeymoo
- **Opacity display (#32)**: single/multi-layer opacity unified to a 100% format and is keyboard-editable. @Eeymoo
- **Slider max limits (#33)**: all material schemas unified per the grading table (distance=1920, radius=500, thickness=50, gap=200). @Eeymoo
- **Locator-orb 6/8 rendering bug**: `select` `onChange` string-vs-number type mismatch always rendered 4 dots. @Eeymoo

---

## [v0.2.1] — 2026-08-09

Stable release. Introduces the four-layer customization architecture (Elements / Materials / Layers / Profiles) with a Rhai material runtime and static multi-layer rendering, multi-profile management, anonymous telemetry, and developer mode; also folds in dev-branch features (single instance, Markdown release notes, mirror download fix).

### Added

- **Four-layer architecture**: The single hardcoded `Crosshair` config is replaced by a fully composable system — Elements (atomic primitives), Materials (Rhai scripts), Layers (instances with transforms), and Profiles (multiple layers). All 12 legacy crosshair styles are migrated to built-in `.rhai` materials. @Eeymoo
- **Layer editor**: Three-column layout with live preview, layer panel, and dynamic parameter controls driven by material `schema()`; multi-layer overlay rendering is WYSIWYG with the preview. @Eeymoo
- **Multi-profile management**: Create / rename / duplicate / delete / switch profiles from the config window; the single/multi-layer mode is persisted across restarts. @Eeymoo
- **Automatic config migration**: Legacy `crosshair` configs are migrated to the `layers` format on first load, with the original file backed up as `config.json.legacy.bak`. @Eeymoo
- **Anonymous telemetry (GlitchTip)**: First-run consent dialog, settings opt-out toggle, crash reports stored locally and uploaded silently after consent, stable per-install `install_id`, and strict sanitization (no IP / username / machine name in events). Telemetry can be fully disabled at compile time via `PEREGRINE_DISABLE_TELEMETRY`. @Eeymoo
- **Developer mode**: Tap the version number 5 times in Settings → About to unlock a "Dev" tab (open DevTools, send test report); release builds hide DevTools until unlocked. @Eeymoo
- **Single instance mode**: Launching the app again focuses the existing window instead of starting a second instance. @Eeymoo
- **Markdown release notes**: The update panel renders release notes with full Markdown formatting. @Eeymoo
- **i18n & UI polish**: Full bilingual (en / zh-CN) settings and config UI audit; unified field widgets (slider / number / color / select / image path) across all 12 styles with two-way sync. @Eeymoo

### Fixed

- Fixed single-layer editing lagging one frame during continuous drags (useCallback closure trap). @Eeymoo
- Fixed single-layer preview lagging one modification behind: the preview now renders from the in-memory profile instead of waiting for the debounced save. @Eeymoo
- Fixed quick colors not applying immediately in single-layer mode, layer move up/down direction, disabled-state styling inconsistency, and a `layer not found` race when adding a layer right after deleting one. @Eeymoo
- Fixed installer download links not using the mainland China mirror prefix when the mirror is enabled. @Eeymoo

### Changed

- **Dynamic material input is soft-disabled** (`MATERIAL_DYNAMIC_INPUT_ENABLED = false`): static multi-layer rendering is fully enabled, but time / mouse / keyboard driven materials render frozen and are hidden from the material picker until re-enabled. @Eeymoo
- The `custom_image` material is temporarily hidden from the picker pending rendering fixes. @Eeymoo

### Build

- Telemetry DSN is injected by CI per channel (TEST project for prereleases, production project for stable releases); the build fails fast on malformed DSN. @Eeymoo

### Download

- Windows x86 / x86_64 / ARM64 NSIS installer (supports auto-update) available in Release Assets.
- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets.

---

## [v0.1.15] — 2026-07-18

Stable release. Added per-style crosshair defaults and one-click color reset; fixed window mode toggle and live drag preview issues; restructured documentation with full bilingual support.

### Added

- **Per-style default crosshair presets**: Each built-in crosshair style now provides out-of-the-box default parameters (size, thickness, offset, opacity, etc.) instead of sharing one global default, so switching styles no longer yields invisible or unusable crosshairs. (#8) @Eeymoo
- **Quick color reset**: Added a "Reset" button next to the quick color presets title that restores the 5 default colors in one click. (#7) @Eeymoo

### Fixed

- Fixed window mode toggle being blocked when the overlay is active: switching window mode (fullscreen/windowed) while the overlay is running is now properly disabled in the tray menu, the backend command, and the frontend. (#9) @Eeymoo
- Fixed "Live Drag Preview" not updating the crosshair position in real time during window dragging: the follower thread now requests a redraw immediately after repositioning the overlay. (#14) @Eeymoo

### Docs

- Restructured documentation site to English-first with full Simplified Chinese variants, including language switcher and bilingual README, HELP, contributing guide, and changelogs. @Eeymoo

### Build

- Added PR snapshot build workflow and opencode trigger workflow for automated CI. (#15) @Eeymoo

### Download

- Windows x86 / x86_64 / ARM64 NSIS installer (supports auto-update) available in Release Assets.
- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets.

---

## [v0.1.9] — 2026-07-13

Stable release. Added an SVG vector rendering backend, a Grid crosshair style, global hotkeys, and quick color presets; overlay anti-aliasing is now enabled by default.

### Added

- **SVG rendering backend**: The overlay now supports an optional SVG rendering backend (based on resvg + tiny-skia), switchable in "Settings → Overlay → Rendering Backend". SVG mode offers higher anti-aliasing quality; CPU mode (default) has zero extra dependencies and is more lightweight. Both backends run in parallel, and SVG rasterization automatically falls back to CPU rendering on failure. @Eeymoo
- **Grid crosshair style**: Added the `Grid` crosshair style, with adjustable rows, columns, line width, and color, giving users who need a regular reference more choices. @Eeymoo
- **Global hotkey system**: Supports binding global hotkeys for functions such as "Start/Stop Overlay", configurable in "Settings → Hotkeys". @Eeymoo
- **Quick color presets**: The color picker now includes common presets for one-click crosshair color switching. @Eeymoo
- **Overlay anti-aliasing**: CPU rendering mode adds an anti-aliasing toggle, enabled by default for smoother edges; can be disabled when minimum latency is required. @Eeymoo
- **Scrollbar styling improvements**: Custom scrollbar styling with default transparency and fade-in on hover, 6 px wide rounded corners, unified with the overall UI style. @Eeymoo

### Fixed

- Removed the restriction that caused drag-and-drop live preview to be forcibly disabled in some scenarios, making interactions more consistent. @Eeymoo

### Download

- Windows x86 / x86_64 / ARM64 NSIS installer (supports auto-update) available in Release Assets.
- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets.

---

## [v0.1.7] — 2026-07-12

Stable release. Removed the Gitee mirror in favor of the gh-proxy acceleration proxy; added GitHub Releases auto-updater, a GPU hardware acceleration toggle, and window mode improvements.

### Added

- **GitHub Releases auto-updater**: Built-in update check and download/install support for both stable and prerelease channels.
- **Mainland China acceleration proxy**: Accelerates GitHub downloads via gh-proxy, enabled by default for Simplified Chinese users; acceleration endpoints (v4 / v6 / cdn / custom) can be selected in settings.
- **GPU hardware acceleration toggle**: WebView2 GPU hardware acceleration can now be toggled in settings; disabling it reduces memory usage by approximately 60 MB.

### Fixed

- Fixed a crash after minimizing the settings window.

### Refactored

- Renamed the default style "Toilet Paper" to "Edge-Aligned Rectangle" and aligned documentation terminology accordingly.

### Build

- Fixed CI build failure (missing javascriptcoregtk dependency).
- Documentation deployment is now triggered only on stable releases.

---

## [v0.1.5] — 2026-07-11

Stable release. Added the NSIS installer and built-in auto-updater, supporting both stable and prerelease channel detection.

### Added

- **NSIS installer**: Provides a `setup.exe` installer that supports auto-update; the portable zip is still retained.
- **Built-in auto-updater**: A "Check for Updates" button in the settings page automatically detects, downloads, and installs new versions, with a real-time download progress bar.
- **Auto-check on launch**: Automatically checks for new versions 3 seconds after opening the settings page and shows a popup if an update is found.
- **Dual-channel updates**: Stable releases use `releases/latest/download/stable.json`, while prerelease versions use the corresponding tag's `prerelease.json`; users can switch update channels in settings.
- **About page publisher info**: The About dialog now shows the publisher (Eeymoo), license (MIT), repository link, and dynamic version number.

### Fixed

- Fixed repeated detection caused by an uncleared popup after clicking update.
- Fixed CI compilation failure due to the missing `update_channel` field in `PreferencesPatch`.
- Fixed CI not exiting with an error when signing was missing.
- Removed redundant hint text on the settings page.

### Build

- CI enables `createUpdaterArtifacts`, automatically generating `.sig` signature files for the NSIS installer.
- CI cleans up debug logs and streamlines build steps.

### Download

- Windows x86 / x86_64 / ARM64 NSIS installer (supports auto-update) available in Release Assets.
- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets.

---

## [v0.1.4] — 2026-07-11

Stable release. License changed to MIT fully open source; added full-screen / window overlay modes, a GPU acceleration toggle, and screen scaling adaptation; significantly optimized memory usage and CPU consumption.

### Added

- **Full-screen / Window overlay modes**: Full-screen mode (default) covers the entire screen directly without needing to select a target window; window mode covers only the target window area. Toggle via the checkbox in the configuration page or the tray menu, with both sides syncing automatically.
- **Live display during dragging**: When enabled in "Settings", the overlay follows in real time while the window is being dragged; when disabled (default), display resumes about 1200 ms after dragging stops, reducing CPU usage.
- **GPU hardware acceleration toggle**: GPU hardware acceleration can be enabled in "Settings" (default off); when off, pure CPU rendering is used to reduce GPU process memory usage. A restart confirmation dialog is shown when switching.
- **Automated versioning**: The version number is now read dynamically from the git tag, and CI automatically syncs it everywhere during packaging, eliminating manual maintenance.

### Fixed

- Fixed incorrect overlay positioning in full-screen mode: the overlay was not pre-positioned to the screen area on first creation.
- Fixed overlay not following screen resolution / DPI scaling changes: full-screen mode now continuously monitors screen size changes.
- Fixed incorrect overlay status display when opening the configuration page: `get_overlay_active` now reads the atomic state directly.
- Fixed left-side preview not refreshing after window resize: added a ResizeObserver so the preview redraws immediately during dragging or scaling.
- Fixed preview proportions not matching the actual overlay: the preview now builds crosshair shapes at the real resolution and scales them proportionally.
- Fixed ESC dialog behavior: ESC cancel now equals stopping the overlay; keeping the configuration window open does not stop the overlay.
- Fixed WebView2 process memory not being released after the window was closed: the window is now truly destroyed instead of hidden to the tray.
- Fixed tray "Exit" not working: the global `ExitRequested` prevention was intercepting active exit requests.
- Fixed documentation deployment CI failure: VitePress build was inheriting the root PostCSS config and failing to find the tailwindcss module.

### Improved

- **Static crosshairs no longer redraw continuously**: Introduced a dirty-flag mechanism so stationary crosshairs are no longer redrawn every frame, significantly reducing overlay CPU usage.
- **Config save debouncing**: Continuous operations such as dragging sliders now write only once 300 ms after the user stops, avoiding frequent file watcher triggers.
- **Settings window is no longer pre-created at startup**: created on demand to reduce startup memory.
- Release artifact zips now include README.md and LICENSE; exe filenames include the version number.
- Formatted all Rust code with `cargo fmt`.

### Changed

- **License changed to MIT**: From PolyForm Noncommercial 1.0.0 to MIT, fully open source and allowing commercial use.

### Download

- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets (contains `peregrine-v0.1.4.exe`, `README.md`, and `LICENSE`).

---

## [v0.1.3] — 2026-07-11

Stable release. Migrated to Tauri + React settings panel; added Simplified Chinese / English internationalization and auto-switching to the game window; release artifacts changed to portable zip; icon clarity greatly improved.

### Added

- New settings UI: rebuilt based on Tauri + React + shadcn/ui, with the configuration window and settings window separated.
- App internationalization: Supports Simplified Chinese and English, switchable in the settings page with window titles, tray menus, and error messages updating accordingly; supports "Follow system language".
- Documentation site English version: complete English usage instructions, configuration guide, and glossary.
- Auto-switch to game when starting overlay: Supports three preferences—"Ask every time", "Yes", and "No"; the Start Overlay button is disabled when no target window is selected.

### Fixed

- Fixed tray menu language not following system language: on Windows, system language detection now uses the Win32 API `GetUserDefaultLocaleName`.
- Fixed "Auto-hide and switch to game after starting overlay": replaced `SetForegroundWindow` with `AttachThreadInput` + `BringWindowToTop`.
- Fixed configuration window not syncing after changing preferences in the settings window: added a `peregrine:settings-changed` event broadcast.
- Fixed checkerboard background misalignment in the configuration preview: operator precedence was causing an incorrect grid pattern.
- Fixed CI `npm ci` failure due to inconsistent `picomatch` versions.

### Changed

- Release artifacts reverted from NSIS installer (`*-setup.exe`) to portable zip: download, extract, and run without installation.
- Removed the "Border: four-side center gap (20%)" option, which had no actual rendering effect.
- Temporarily hid the "Custom Image" crosshair style (known issues, to be fixed later).

### Improved

- Greatly improved icon clarity: the icon generation script now uses 8x supersampling anti-aliasing; the ICO contains 16/32/48/64/128/256 sizes; the tray and window title bar use a 1024×1024 high-resolution PNG source image, remaining sharp and clear at high DPI.

### Download

- Windows x86 / x86_64 / ARM64 portable zip available in Release Assets.

---

## [v0.1.2] — 2026-07-08

Stable release. Fixed wgpu crashes and icon display issues; optimized UI style naming.

### Fixed

- Fixed a crash when the settings window was minimized caused by wgpu viewport validation failure (`set_viewport` size was 0).
- Set a wgpu error handler so uncaught errors are downgraded to log records instead of panicking.
- Fixed incorrect taskbar and window title bar icons: the tray icon now loads from the exe's embedded resources.
- Restored the window title bar icon display and increased the pixel art size to 256×256.

### Changed

- The display name of the "Toilet Paper" style changed to "Rectangle".

### Documentation

- Added "Alleviating 3D Motion Sickness" and "Recommended Configurations" pages, expanding the project introduction.

### Download

- Windows x86 / x86_64 / ARM64 executables available in Release Assets.

---

## [v0.1.1] — 2026-07-07

Patch update after the first stable release. Fixed macOS startup crash; Windows artifacts now statically link the C runtime, enabling download-extract-run without installing the VC++ Redistributable.

### Fixed

- On macOS, wgpu surface did not support the `Inherit` alpha mode, causing a startup panic; now automatically selected based on capabilities.

### Build

- Enabled `+crt-static` static C runtime linking for all three Windows MSVC targets (x86 / x64 / ARM64), so the exe no longer depends on external DLLs such as `VCRUNTIME140.dll`.
- Release CI added a DLL dependency verification step to ensure artifacts have no dynamic VC runtime dependencies.

### Documentation

- Added VitePress documentation site and GitHub Pages automatic deployment.
- Improved README, HELP, and AGENTS documentation; added a Download Now button on the homepage.
- Added release process specifications and contribution guidelines.

### Download

- Windows x86 / x86_64 / ARM64 executables available in Release Assets.

---

## [v0.1.0] — 2026-07-07

First stable release. A desktop auxiliary sticker tool for alleviating 3D motion sickness, displaying semi-transparent visual anchors above the screen to help players get a fixed reference in 3D games.

### Added

- Windows transparent overlay window: an always-on-top, click-through overlay window that can float above games or applications.
- Target window following: select a target window from a dropdown list, and the overlay can follow its position and size.
- Multiple crosshair styles: cross, large cross, four / six / eight corner dots, center ring, custom orb, random orb, border frame, edge-aligned rectangle, etc.
- Custom PNG decal: supports loading PNG images as overlay content.
- Real-time settings panel: a standalone settings window for adjusting style, color, opacity, size, and other parameters with instant preview.
- Config file hot-reload: the configuration JSON file is automatically reloaded when edited externally.
- Multi-profile support: save independent configurations for different scenarios.
- Windows platform automated build and release: GitHub Actions automatically builds Windows x86 / x86_64 / ARM64 artifacts.

### Fixed

- Windows transparency completely ineffective: forced Bgra8Unorm to avoid sRGB gamma causing color-key mismatch.
- Color key eating black crosshairs, overlay switching flicker, and window title matching logic.
- HWND cross-thread retrieval failure, program crash when no window was selected, and window size restoration.
- Click-through window not receiving `RedrawRequested`, causing the overlay not to render.
- `SetWindowLongPtrW` / `GetWindowLongPtrW` type mismatch on 32-bit Windows.

### Changed

- Architecture refactor: dual-window architecture (standalone settings window + standalone overlay window).
- Overlay switched to per-pixel alpha transparency solution (softbuffer pixel buffer).
- Target window changed from an input box to a dropdown list.
- Preview area follows the target window's aspect ratio.
- License changed to PolyForm Noncommercial 1.0.0.
- Embedded Windows exe icon.

### Build

- Only builds and releases for Windows x86 / x86_64 / ARM64.

### Download

- Windows x86 / x86_64 / ARM64 executables available in Release Assets.

---

[v0.1.15]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.15
[v0.1.9]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.9
[v0.1.5]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.5
[v0.1.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.4
[v0.1.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3
[v0.1.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.2
[v0.1.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1
[v0.1.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0

---

# Preview Releases

## [v0.2.0-alpha.0] — 2026-07-18

This is the first preview release of the **four-layer customization architecture**, a major refactor that replaces the single hardcoded `Crosshair` config with a fully composable system.

### Added

- **Four-layer architecture**: Elements (atomic primitives), Materials (Rhai scripts), Layers (instances with transforms), Profiles (multiple layers).
- **Rhai material runtime** (`crates/material`): CPU-safe embedded scripting via `rhai` crate. Scripts export `defaults()`, `schema()`, `is_dynamic()`, and `build(params, screen)`.
- **Dynamic input for materials**: `time_ms()`, `mouse_pos()`, `key_down(code)`, `rand()` accessible from material scripts. Windows implementation uses `GetCursorPos` / `GetAsyncKeyState` via `poll_dynamic_context`.
- **12 built-in materials**: All legacy `CrosshairStyle` variants migrated to `.rhai` scripts (`cross`, `large_cross`, `edge_rect`, `corner_dots`, `ring`, `custom_orb`, `random_orb`, `border_frame`, `edge_arrows`, `grid`, `image`).
- **Layer composition**: Multiple layers can be stacked; each has its own material, params, color, opacity, transform (offset/scale/rotation), visibility, and lock state.
- **Config migration**: Legacy `config.json` with `crosshair` field is automatically migrated to the new `layers` format on first load. The original file is backed up as `config.json.legacy.bak`.
- **Tauri IPC commands**: `build_shapes_ipc`, `list_materials`, `add_layer`, `remove_layer`, `move_layer`, `duplicate_layer`, `update_layer`, `list_layers`.
- **Frontend layer editor** (`LayersEditor`): Three-column layout with live preview, layer panel, and dynamic parameter controls driven by material `schema()`.
- **Anonymous telemetry (partial)**: GlitchTip (Sentry protocol) integration with first-run consent dialog, settings toggle, pending-report storage with one-time authorized upload, and CI-injected DSN split by channel (TEST / production project).

### Changed

- `Profile` schema now dual-supports legacy `crosshair: Option<Crosshair>` (legacy) and `layers: Vec<Layer>` (new format). `load_or_create_default` auto-migrates legacy configs.
- `Shape` is now a type alias for `Element` (9 variants: Rect, Circle, CircleStroke, DashedCircle, Triangle, Polygon, Line, Text, Image).
- `Preview` component now fetches shapes via IPC `build_shapes_ipc` instead of computing geometry in TypeScript (`src/lib/shapes.ts` removed).
- `OverlayRenderer` uses a dual-path rendering strategy: new format (layers + material evaluation) takes precedence; legacy Crosshair path retained as fallback.

### Build

- New workspace member `crates/material` (depends on `peregrine_config` + `rhai` 1.25 + `ahash` 0.8).
- `SimpleRng` moved to `peregrine_config::rng` for cross-crate sharing between material runtime and legacy shapes.
- CI expanded to run `cargo clippy` and `cargo test` on all three crates (`config`, `material`, `peregrine`).

### Known Limitations

- `src-tauri` (Tauri commands) cannot be compiled on non-Windows hosts without webkit2gtk system deps; verified only via Windows CI.
- The legacy Crosshair UI in `ConfigApp.tsx` is retained as default; click "Switch to Layer Editor" to access the new UI.
- Old Quick Color hotkeys operate on `crosshair.color`; new layer-based equivalent is not yet wired up.

---

## [v0.1.15-alpha.0] — 2026-07-17

### Added

- **Quick color reset**: Added a "Reset" button next to the quick color presets title that restores the 5 default colors in one click. [#3](https://github.com/Eeymoo/peregrine/issues/3)
- **Per-style default crosshair presets**: Built-in crosshair styles now provide out-of-the-box default parameters (size, thickness, offset, opacity, etc.) instead of sharing one global default, so switching styles no longer yields invisible or unusable crosshairs. The frontend reverts to style-specific defaults when the user changes the style, keeping preview and overlay WYSIWYG. [#4](https://github.com/Eeymoo/peregrine/issues/4)

### Fixed

- Fixed "Live Drag Preview" not updating the crosshair position in real time during window dragging: the follower thread moved the overlay window but never notified the renderer to refresh, so the crosshair stayed frozen until the mouse was released. The follower now requests a redraw directly via `window.request_redraw()` whenever it repositions the overlay. [#5](https://github.com/Eeymoo/peregrine/issues/5)
- Fixed window mode toggle desync when the overlay is active: Tauri v2's `CheckMenuItem` auto-toggles the checkbox before the menu event fires, so rejecting the switch left the tray checkbox out of sync with the actual config. The tray checkbox is now reverted when the guard blocks. Switching window mode (fullscreen/windowed) while the overlay is running is now blocked in the tray menu, the backend `update_preferences` command, and the frontend (checkbox disabled with tooltip). [#2](https://github.com/Eeymoo/peregrine/issues/2)

## [v0.1.13-alpha.0] — 2026-07-13

v0.1.13 的预发布版本。

### 新增

- **单例模式**：重复启动应用时自动聚焦已有窗口，不再运行多个实例。 @Eeymoo
- **Markdown 更新日志**：更新检查面板使用 react-markdown 渲染发布说明，支持完整 Markdown 排版。 @Eeymoo

### 变更

- **前端组件拆分重构**：ConfigApp / SettingsApp 大幅拆分为独立 hooks 与子组件（`components/config`、`components/settings`、`hooks/`），提升可维护性。 @Eeymoo

### 修复

- **镜像下载修复**：启用中国大陆镜像时，安装包下载链接也套用镜像前缀，之前仅清单 URL 走镜像。 @Eeymoo

## [v0.1.9-alpha.0] — 2026-07-13

Preview release for v0.1.9. Changes have been merged into the v0.1.9 stable release.

### Added

- **SVG rendering backend**: Added an optional SVG rendering backend for the overlay (resvg + tiny-skia), switchable in "Settings → Overlay → Rendering Backend". SVG mode provides higher anti-aliasing quality; CPU mode (default) has zero extra dependencies and is more lightweight. Both backends run in parallel, and rendering automatically falls back to CPU if SVG rasterization fails. @Eeymoo
- **Grid crosshair style**: Added the `Grid` crosshair style, with adjustable row/column count, line width, and color. @Eeymoo
- **Global hotkey system**: Supports binding global hotkeys for functions such as "Start/Stop Overlay". @Eeymoo
- **Quick color presets**: Added commonly used color presets to the color picker. @Eeymoo
- **Overlay anti-aliasing**: Added an anti-aliasing toggle for CPU rendering mode, enabled by default. @Eeymoo
- **Scrollbar style optimization**: Added custom scrollbar styling that is transparent by default and fades in on hover, 6 px wide with rounded corners. @Eeymoo

### Fixed

- Removed the restriction that forced drag-to-move previews to be disabled in certain scenarios. @Eeymoo

---

## [v0.1.4-alpha.0] — 2026-07-11

### Improved

- Limited overlay rendering to 60 FPS: eliminated busy-loop rendering caused by duplicate `about_to_wait` and `RedrawRequested` events, significantly reducing CPU usage after starting the overlay.
- Destroyed WebView2 when the configuration/settings window is closed: no longer hidden to the tray and kept in memory; it is recreated when "Configuration" or "Settings" is clicked in the tray.
- No longer pre-creates the "Settings" window on startup: created on demand to further reduce startup memory usage.

### Fixed

- Fixed tray "Exit" not working: `RunEvent::ExitRequested` globally blocking exit would intercept `app.exit(0)`; changed to use a `quitting` flag to distinguish between active quit and window close.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.1.3-alpha.4] — 2026-07-11

### Changed

- Removed the "20% mid-edge notch" option for the "Border" style (`border_gap` field), as it had no actual rendering effect and was dead code.
- Temporarily hidden the "Custom Image" crosshair style (`custom_image`) due to known issues pending fixes.
- Disabled the "Start Overlay" button when no target window is selected, preventing accidental clicks.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.1.3-alpha.3] — 2026-07-11

### Changed

- Changed release artifacts from NSIS installer (`*-setup.exe`) back to portable zip archives: each architecture is packaged separately as `peregrine-windows-x86.zip` / `peregrine-windows-x64.zip` / `peregrine-windows-arm64.zip`, ready to run after extraction without installation.

### Fixed

- Fixed tray menu language not following system language: the `LANG` environment variable usually does not exist on Windows, so system language is now detected via the Win32 API `GetUserDefaultLocaleName`.
- Fixed "Auto-hide and switch to game after starting overlay" not working: `SetForegroundWindow` is restricted by foreground lock; switched to the reliable combination of `AttachThreadInput` + `BringWindowToTop`.
- Fixed configuration window not syncing after changing the "Auto-switch" preference in the settings window: added the `peregrine:settings-changed` event broadcast, keeping React state synchronized between both windows in real time.
- Fixed checkerboard background misalignment in the configuration preview: the `%` operator precedence is higher than `+`, causing the alternating tile pattern to be misaligned.

### Improved

- Significantly improved icon clarity: the generation script now uses 8x supersampling anti-aliasing; the ICO includes six sizes (16/32/48/64/128/256); the tray and window title bar use a 1024x1024 high-resolution PNG source, appearing crisp and sharp on high-DPI displays.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.1.3-alpha.2] — 2026-07-10

### Fixed

- Fixed TypeScript compilation failure caused by the `Locale` type including `"auto"` becoming incompatible with the `localeMap` index type, which broke the CI build.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.1.3-alpha.1] — 2026-07-10

### Added

- Added "Follow System" option to language settings; the app now automatically selects Simplified Chinese or English based on the system language by default.
- Added a "Switch to game when starting overlay" preference on the settings page: Ask every time / Yes / No.
- A confirmation dialog is shown the first time "Start Overlay" is clicked, with the option to remember the choice.

### Changed

- Language and auto-switch preferences are now persisted together in the `settings` section of `config.json`, removing the frontend's dependence on `localStorage` for more reliable cross-window synchronization.
- Tray menu text is now initialized at application startup according to the current language.

### Fixed

- Fixed installation failure during `npm ci` caused by the `picomatch` version not matching `package-lock.json`.
- Fixed alpha prerelease version numbers being unable to package MSI: release artifacts are now built with NSIS (`*-setup.exe`).
- Fixed panic caused by the overlay event loop being created on a non-main thread without `with_any_thread(true)`.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.1.3-alpha.0] — 2026-07-10

### Added

- Application internationalization: supports Simplified Chinese and English, switchable in "Settings → Language"; window titles, tray menus, and error messages switch accordingly.
- Added a complete English version of the documentation site.
- Added a "Glossary" page (Chinese/English) to enforce consistent core concepts and names for the 12 visual anchor styles.

### Fixed

- Fixed `RandomOrb` style RNG inconsistency between the frontend preview and the Rust overlay; unified to the same 64-bit LCG to ensure random edge marker positions are consistent.
- Cleaned up leftover egui / settings_ui era comments in `shapes.rs` / `overlay_renderer.rs`.

### Docs

- Unified Chinese/English terminology across `docs/`, `README.md`, and `HELP.md`: visual anchor, overlay, configuration window, EdgeRect, Cross, edge marker, Ring, etc.
- Updated build instructions to the Tauri workflow (`npm install` + `npx tauri dev/build`).
- Completed the English version of configuration instructions in `docs/en/guide/config.md`.

> Released by: Eeymoo (Peregrine maintainer)

---

## [v0.2.0-alpha.2] — 2026-07-08

### Fixed

- Fixed `Cross` crosshair shifting toward the top-left when adjusting gap: the left and top arms over-subtracted by half a gap, making the left/top gap twice the right/bottom gap. Corrected to expand symmetrically around the center with equal gaps on both sides.

---

## [v0.1.1-alpha.1] — 2026-07-07

### Fixed

- Fixed startup panic on macOS where the wgpu surface did not support the `Inherit` alpha mode; now automatically selected based on capabilities.

### Build

- Enabled `+crt-static` static C-runtime linking for all three Windows MSVC targets (x86/x64/ARM64), so the exe no longer depends on external DLLs such as `VCRUNTIME140.dll`.
- Added DLL dependency verification step to the Release CI to ensure artifacts do not have dynamic VC runtime dependencies.

### Docs

- Added a VitePress documentation site and automated GitHub Pages deployment.
- Fixed repository links and usage instructions; added a "Download Now" button to the homepage.
- Explicitly added the `search-insights` dependency to fix CI `npm ci`.

---

## [v0.2.0-alpha.0] — 2026-07-06

### Added

- PNG image support: custom PNGs can now be loaded as overlay decals.
- Unified geometry module shared between preview and overlay, reducing logic duplication.

### Changed

- Overlay rendering switched to a softbuffer pixel-buffer approach (inspired by simple-crosshair-overlay).
- Settings UI and overlay rendering now share geometric drawing logic.

---

## [v0.1.0-alpha.12] — 2026-07-02

- Refactored architecture to dual-window: settings window and independent overlay window separated.

## [v0.1.0-alpha.11] — 2026-07-02

- Removed all non-Windows platform code; project is now Windows-focused.

## [v0.1.0-alpha.10] — 2026-07-02

- Fixed transparency completely failing: forced Bgra8Unorm to avoid color-key mismatch caused by sRGB gamma.

## [v0.1.0-alpha.9] — 2026-07-02

- Fixed logs not being output by default: changed EnvFilter default level to info.

## [v0.1.0-alpha.8] — 2026-07-02

- Fixed HWND cross-thread retrieval failure.
- Added guard for no window selected.
- Fixed window size restoration and redundant cleanup.

## [v0.1.0-alpha.7] — 2026-07-02

- Fixed color key eating black crosshairs.
- Fixed overlay switching flicker.
- Fixed window title matching logic.

## [v0.1.0-alpha.6] — 2026-07-02

- Added "Start Overlay" button.
- Fixed transparent color key.
- Added window selection logging; cleaned up debug prints.

## [v0.1.0-alpha.5] — 2026-07-02

- Compilation optimizations.
- Embedded Windows exe icon.

## [v0.1.0-alpha.4] — 2026-07-02

- Windows overlay keeps Bgra8UnormSrgb to fix DWM transparent composition.

## [v0.1.0-alpha.3] — 2026-07-02

- Fixed Windows window selection: unified enumeration source and robust loop.

## [v0.1.0-alpha.2] — 2026-07-01

- Fixed Windows black window issue.
- Fixed Chinese characters appearing as boxes.
- Fixed window selection and transparent overlay.

## [v0.1.0-alpha.1] — 2026-07-01

- Release workflow now only builds and publishes Windows (x86_64).

## [v0.1.0-alpha.0] — 2026-07-01

- First alpha release.
- Added Windows overlay transparent always-on-top click-through window.
- Added target window following.
- Basic crosshair style support.

---

[v0.1.13-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.13-alpha.0
[v0.1.9-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.9-alpha.0
[v0.1.4-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.4-alpha.0
[v0.1.3-alpha.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.4
[v0.1.3-alpha.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.3
[v0.1.3-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.2
[v0.1.3-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.1
[v0.1.3-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.3-alpha.0
[v0.2.0-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.2.0-alpha.2
[v0.1.1-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.1-alpha.1
[v0.2.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.2.0-alpha.0
[v0.1.0-alpha.12]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.12
[v0.1.0-alpha.11]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.11
[v0.1.0-alpha.10]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.10
[v0.1.0-alpha.9]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.9
[v0.1.0-alpha.8]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.8
[v0.1.0-alpha.7]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.7
[v0.1.0-alpha.6]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.6
[v0.1.0-alpha.5]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.5
[v0.1.0-alpha.4]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.4
[v0.1.0-alpha.3]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.3
[v0.1.0-alpha.2]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.2
[v0.1.0-alpha.1]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.1
[v0.1.0-alpha.0]: https://github.com/Eeymoo/peregrine/releases/tag/v0.1.0-alpha.0
