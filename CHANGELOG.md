# Changelog

[English](CHANGELOG.md) · [简体中文](CHANGELOG.zh-cn.md)

Only stable releases are recorded here. For beta / prerelease versions, see **[CHANGELOG_ALPHA.md](CHANGELOG_ALPHA.md)**.

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
