# Changelog

This page records the major changes in each Peregrine version. Stable and preview versions are shown together in reverse chronological order.

For the full stable release notes, see [GitHub Releases](https://github.com/Eeymoo/peregrine/releases).

---

## v0.1.4 — 2026-07-11 (Stable)

License changed to MIT fully open source; added fullscreen/window overlay modes, GPU acceleration toggle, and screen scaling adaptation; greatly optimized memory usage and CPU consumption.

### Changed

- **License changed to MIT**: changed from PolyForm Noncommercial 1.0.0 to MIT, fully open source and commercial use allowed.

### Added

- **Fullscreen / Window overlay modes**: Fullscreen mode (default) covers the entire screen directly without selecting a target window; Window mode covers only the target window area. Toggle via the "Window mode" checkbox in the config page or the tray menu; both sides stay in sync.
- **Real-time display while dragging**: When enabled in Settings, the overlay follows in real time while dragging the window; when disabled (default), the overlay resumes about 1200 ms after dragging stops to reduce CPU usage.
- **GPU hardware acceleration toggle**: GPU hardware acceleration can be enabled in Settings (default off); a restart confirmation dialog appears after toggling.
- **Version number automation**: Version number is read dynamically from the git tag and automatically synced during CI packaging.

### Optimized

- **Preview ratio fix**: The preview area builds the visual anchor shapes at the real resolution and then scales proportionally, providing a true WYSIWYG experience.
- **Static visual anchors no longer redraw continuously**: A dirty-marking mechanism was introduced, significantly reducing overlay CPU usage.
- **Screen scaling adaptation**: Fullscreen overlay updates immediately when resolution / DPI changes.
- **Preview real-time refresh**: Added ResizeObserver so the preview redraws immediately when the window is dragged or resized.
- **Config save debouncing**: Continuous operations are written only once, 300 ms after stopping.
- Closing the window now truly destroys WebView2 instead of hiding it and consuming memory.
- Release zip contains `peregrine-v*.exe`, `README.md`, and `LICENSE`.

### Fixed

- Fixed overlay position errors in fullscreen mode.
- Fixed incorrect overlay status display when opening the config page.
- Fixed ESC dialog behavior.
- Fixed WebView2 process memory not being released after closing.
- Fixed tray "Exit" not working.
- Fixed documentation deployment CI failures.

---

## v0.1.3 — 2026-07-11 (Stable)

### Added

- App internationalization: supports Simplified Chinese and English; window titles, tray menus, and error messages switch together.
- Language setting added a "Follow system" option; defaults to automatically selecting based on system language.
- Settings page added a "Auto switch to game when starting overlay" preference: ask every time / yes / no.
- First click of "Start overlay" shows a confirmation dialog; you can choose whether to remember this choice.

### Optimized

- Icon clarity greatly improved: the generation script now uses 8x supersampling anti-aliasing, making icons sharp and clear on high-DPI displays.
- Release artifact changed to a portable zip package; download, extract, and use.

### Fixed

- Fixed tray menu language not following system language.
- Fixed "Auto-hide and switch to game after starting overlay" not working.
- Fixed config preview checkerboard background glitch.
- Fixed `RandomOrb` style frontend preview RNG not matching the overlay RNG.

---

## v0.1.0 — 2026-07-11 (Stable)

### Added

- Windows transparent always-on-top click-through overlay.
- Target window following.
- 12 visual anchor styles (edge rectangle, crosshair, large crosshair, 4/6/8 corner markers, center ring, custom edge markers, random edge markers, border frame, edge arrows, etc.).
- Custom PNG decal.
- Configuration hot reload.
- System tray integration.
