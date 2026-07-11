# Changelog

This page documents the major changes in each version of Peregrine. Stable and pre-release versions are listed together in reverse chronological order.

For full stable release notes, see [GitHub Releases](https://github.com/Eeymoo/peregrine/releases).

---

## v0.1.4-alpha.0 — 2026-07-11

### New

- **Fullscreen / Window overlay mode**: In fullscreen mode, the overlay covers the entire screen without needing to select a target window. In window mode, the overlay only covers the target window area. Toggle via the "Window Mode" checkbox in the config page or the tray menu — both stay in sync.
- **Live drag preview setting**: In Settings, you can enable "Live Drag Preview". When enabled, the overlay follows the window in real time during dragging. When disabled (default), the overlay reappears ~1200ms after dragging stops, reducing CPU usage.
- **GPU hardware acceleration toggle**: In Settings, you can enable GPU hardware acceleration (disabled by default). When off, pure CPU rendering is used to reduce GPU process memory usage.
- **Automated version numbering**: The version number is now read dynamically from the git tag. CI builds sync it automatically — no more manual maintenance.

### Optimizations

- **Preview scale fix**: The preview area now builds crosshair shapes at real resolution then scales down proportionally. What you see is what you get — no more distorted proportions from canvas size mismatch.
- **Static crosshairs no longer redraw continuously**: A dirty-flag mechanism skips per-frame redraws for stationary crosshairs, significantly reducing overlay CPU usage.
- Capped overlay render rate to 60 FPS, eliminating busy-loop redraws.
- Closing config/settings windows now truly destroys WebView2 instead of hiding to tray, freeing memory.
- Settings window is no longer pre-created at startup — created on demand to reduce startup memory.

### Fixes

- Fixed incorrect overlay position in fullscreen mode: initial placement was missing.
- Fixed overlay status showing incorrectly when opening the config page: `get_overlay_active` now reads the atomic state directly.
- Fixed ESC dialog behavior: ESC cancel now equals stop overlay + close dialog; keeping the config window no longer stops the overlay.
- Fixed tray "Exit" not working: a `quitting` flag now distinguishes explicit exit from window close.

---

## v0.1.3 — 2026-07-11 (Stable)

### New

- App internationalization: Simplified Chinese and English support. Window titles, tray menus, and error messages switch together.
- Language setting adds "Follow System" option, auto-detecting system language by default.
- Settings page adds "Auto-switch to game on overlay start" preference: Ask / Yes / No.
- Confirmation dialog on first "Start Overlay" click, with option to remember the choice.

### Optimizations

- Icon clarity greatly improved: 8x supersampled anti-aliasing, sharp under high DPI.
- Release artifacts changed to portable zip packages — download, extract, and run.

### Fixes

- Fixed tray menu language not following system language.
- Fixed "auto-hide and switch to game on overlay start" not working.
- Fixed preview checkerboard background pattern error.
- Fixed RNG mismatch between `RandomOrb` preview and overlay.

---

## v0.1.0 — 2026-07-11 (Stable)

### New

- Windows transparent always-on-top click-through overlay.
- Target window following.
- 12 crosshair styles (edge rect, cross, large cross, 4/6/8 corner markers, center ring, custom markers, random markers, border frame, edge arrows, etc.).
- Custom PNG image support.
- Config hot reload.
- System tray integration.
