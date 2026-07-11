# Changelog

This page documents the major changes in each version of Peregrine. Stable and pre-release versions are listed together in reverse chronological order.

For full stable release notes, see [GitHub Releases](https://github.com/Eeymoo/peregrine/releases).

---

## v0.1.4 — 2026-07-11 (Stable)

License changed to MIT — fully open source. New fullscreen/window overlay modes, GPU acceleration toggle, screen scaling adaptation, and major memory/CPU optimizations.

### Changed

- **License changed to MIT**: Switched from PolyForm Noncommercial 1.0.0 to MIT — fully open source, commercial use allowed.

### New

- **Fullscreen / Window overlay mode**: Fullscreen mode (default) covers the entire screen without needing a target window. Window mode covers only the target window area. Toggle via config page or tray menu — both stay in sync.
- **Live drag preview setting**: Enable in Settings for real-time overlay following during window drag. When off (default), overlay reappears ~1200ms after dragging stops.
- **GPU hardware acceleration toggle**: Enable in Settings (off by default). A restart confirmation dialog appears when toggled.
- **Automated version numbering**: Version read dynamically from git tag, synced automatically during CI builds.

### Optimizations

- **Preview scale fix**: Preview builds crosshair shapes at real resolution then scales proportionally — WYSIWYG.
- **Static crosshairs no longer redraw continuously**: Dirty-flag mechanism significantly reduces overlay CPU usage.
- **Screen scaling adaptation**: Fullscreen overlay follows resolution/DPI changes instantly.
- **Real-time preview refresh**: ResizeObserver triggers immediate redraw on window resize.
- **Debounced config saving**: Continuous operations write only once 300ms after the last change.
- Closing windows now truly destroys WebView2 instead of hiding to tray.
- Release zip now includes `peregrine-v*.exe`, `README.md`, and `LICENSE`.

### Fixes

- Fixed incorrect overlay position in fullscreen mode.
- Fixed overlay status showing incorrectly when opening config page.
- Fixed ESC dialog behavior.
- Fixed WebView2 process memory not released after window close.
- Fixed tray "Exit" not working.
- Fixed documentation deployment CI failure.

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
