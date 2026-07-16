# Alpha Changelog

[English](CHANGELOG_ALPHA.md) · [简体中文](CHANGELOG_ALPHA.zh-cn.md)

The following versions are preview and development iterations before the stable release. They are intended for testing and feedback only.

For the stable release changelog, see **[CHANGELOG.md](CHANGELOG.md)**.

---

## [Unreleased]

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
