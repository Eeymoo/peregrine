# Introduction

**Peregrine** is a desktop tool focused on relieving 3D motion sickness. It provides customizable visual anchors on your screen (such as crosshairs, border frames, edge arrows, and more) to help players stay visually stable during fast camera movements or complex scenes, reducing nausea and dizziness caused by vestibular-visual conflict. This lets you comfortably play games like *Half-Life 2*, *Mirror's Edge*, *Dying Light*, and *No Man's Sky* that often trigger motion sickness.

## What Is 3D Motion Sickness?

When playing first-person or third-person 3D games, some players feel dizzy or nauseous due to frequent screen motion or rapid camera switching. This phenomenon is commonly called 3D motion sickness or motion sickness.

A common remedy is to place a **fixed visual anchor** at the center or edges of the screen. When the picture shakes violently, the eyes can use this anchor to quickly regain spatial orientation, thereby reducing motion sickness.

## What Peregrine Does

Peregrine draws a semi-transparent visual anchor at the center or edges of the screen, such as:

- Crosshair / Large Crosshair
- Center Ring
- Edge Markers
- Edge Rectangle (classic anchor)
- Custom PNG Decal

The anchor always stays on top of all other windows and **does not block mouse or keyboard input**, so you barely notice it while gaming.

## How Peregrine Works

The core of Peregrine is the "Overlay" — a special window that meets three conditions:

- **Transparent**: The window background is completely transparent; only the visual anchor / decal is visible, so it won't cover the game screen.
- **Always-on-top**: It always stays above all other windows, even when the game is fullscreen.
- **Click-through**: The window does not intercept any mouse clicks or keyboard input, so you can play normally and barely feel its presence.

On Windows, these features are implemented through system APIs: transparency and click-through are achieved with window attributes (`with_transparent` + mouse hit-testing disabled + always-on-top window level). The program additionally applies "do not activate" and "do not show in taskbar" attributes to ensure the overlay floats quietly above the game.

For rendering, the overlay uses **CPU pixel rasterization** (softbuffer) instead of the GPU, avoiding some pitfalls of Windows transparent composition. The program draws visual-anchor geometry (rectangles, circles, line segments, triangles, etc.) directly into a pixel buffer, which is then handed to the system for compositing on screen. This approach is lightweight, stable, and independent of the game's rendering pipeline.

The settings window uses **Tauri** (Webview) to host the interface built with **React + Tailwind CSS + shadcn/ui**, providing a real-time preview — the anchor you see in the settings panel is exactly the same as what the overlay displays.

To learn more about why motion sickness happens and other remedies besides visual anchors, see [Relieving Motion Sickness](./motion-sickness.md).

## Dependencies

Peregrine is written in Rust. The table below lists the main dependencies and their purposes:

| Dependency | Purpose |
|------|------|
| [Tauri](https://tauri.app/) | Cross-platform desktop app framework providing the Webview settings window and system tray |
| [React](https://react.dev/) / [Tailwind CSS](https://tailwindcss.com/) / [shadcn/ui](https://ui.shadcn.com/) | Settings panel UI and components |
| [winit](https://github.com/rust-windowing/winit) | Cross-platform window creation and event loop |
| [softbuffer](https://github.com/rust-windowing/softbuffer) | CPU pixel-buffer rasterization for the overlay |
| [tokio](https://github.com/tokio-rs/tokio) | Async runtime powering config read/write, hot reload, and window following |
| [png](https://github.com/image-rs/image-png) | Custom PNG decal decoding |
| [notify](https://github.com/notify-rs/notify) | Config file hot reload (watches file changes) |
| [serde](https://github.com/serde-rs/serde) / serde_json | Serialization and deserialization of the config file |
| [tracing](https://github.com/tokio-rs/tracing) | Structured logging |
| [windows](https://github.com/microsoft/windows-rs) | Windows platform APIs (transparency / always-on-top / click-through / window following) |

All dependency versions are declared centrally in the root `Cargo.toml` under `[workspace.dependencies]`. For the complete list, see [Cargo.lock](https://github.com/eeymoo/peregrine/blob/main/Cargo.lock).

## Open Source

Peregrine is released under the [MIT](https://opensource.org/licenses/MIT) license and is fully open source.

**This means you can:**

- ✅ Use, modify, and distribute it freely (including commercially)
- ✅ Read and study the full source code
- ✅ Submit Issues and Pull Requests to help improve it

### Contributing

Issues and Pull Requests are welcome. Please see [`CONTRIBUTING.md`](https://github.com/eeymoo/peregrine/blob/main/CONTRIBUTING.md) in the repository for contribution guidelines, and the [`Development & Build`](./development.md) page for how to build and test locally.
