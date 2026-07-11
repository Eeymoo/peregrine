# Project Introduction

**Peregrine** is a desktop utility focused on relieving **3D motion sickness**. By providing a customizable **visual anchor** on your screen (such as a crosshair, border frame, edge arrows, etc.), it helps players maintain visual stability during rapid camera movement or complex scenes, easing nausea and dizziness caused by vestibular-visual conflict. This lets you comfortably play games like *Half-Life 2*, *Mirror's Edge*, *Dying Light*, and *No Man's Sky* that are prone to triggering motion sickness.

## What is 3D Motion Sickness?

When playing first-person or third-person 3D games, some players feel dizzy or nauseous because the image on screen frequently moves or the camera switches too quickly. This phenomenon is commonly called 3D motion sickness or motion sickness.

A common relief method is to place a **fixed visual anchor** at the center or edge of the screen. When the screen shakes violently, your eyes can quickly regain spatial orientation with the help of this anchor, reducing the sensation of sickness.

## What Peregrine Does

Peregrine draws a semi-transparent visual anchor at the center or edge of the screen, for example:

- Crosshair / Large crosshair
- Center ring
- Corner markers
- Edge rectangle (classic anchor)
- Custom PNG image

The anchor always stays on top of the screen and **does not block mouse or keyboard input**, so you barely notice it while gaming.

## How Peregrine Works

The core of Peregrine is the **overlay** — a special window that meets three conditions:

- **Transparent**: The window background is fully transparent; only the visual anchor / image is visible, so it does not obscure the game screen.
- **Always on top**: It stays above all other windows, even when the game is running fullscreen.
- **Click-through**: The window does not intercept any mouse clicks or keyboard input, so you can operate the game normally and barely feel its presence.

On Windows, these features are implemented via system APIs: transparency and click-through are handled by window attributes (`with_transparent` + disabled hit-testing + always-on-top level), and the program additionally sets "does not steal focus" and "not shown in the taskbar" attributes, ensuring the overlay floats quietly above the game.

For rendering, the overlay uses **CPU pixel rasterization** (softbuffer) instead of the GPU, to avoid some pitfalls of Windows transparent compositing. The program draws visual-anchor geometries (rectangles, circles, line segments, triangles, etc.) directly into a pixel buffer, then hands it to the system compositor. This approach is lightweight, stable, and independent of the game's rendering pipeline.

The config window is built with **Tauri** (Webview) carrying a **React + Tailwind CSS + shadcn/ui** interface, providing real-time preview — the anchor you see in the settings panel is exactly the same as what the overlay displays.

To learn more about why motion sickness happens and other relief methods besides visual anchors, see [Relieving 3D Motion Sickness](./motion-sickness.md).

## Dependencies

Peregrine is written in Rust. Below are the main dependencies and their purposes:

| Dependency | Purpose |
|------------|---------|
| [Tauri](https://tauri.app/) | Cross-platform desktop application framework; provides the Webview config window and system tray |
| [React](https://react.dev/) / [Tailwind CSS](https://tailwindcss.com/) / [shadcn/ui](https://ui.shadcn.com/) | Settings panel UI and components |
| [winit](https://github.com/rust-windowing/winit) | Cross-platform window creation and event loop |
| [softbuffer](https://github.com/rust-windowing/softbuffer) | CPU pixel-buffer rasterization for the overlay |
| [tokio](https://github.com/tokio-rs/tokio) | Async runtime, driving config read/write, hot reload, and window following |
| [png](https://github.com/image-rs/image-png) | Custom PNG image decoding |
| [notify](https://github.com/notify-rs/notify) | Config file hot reload (watching file changes) |
| [serde](https://github.com/serde-rs/serde) / serde_json | Config file serialization and deserialization |
| [tracing](https://github.com/tokio-rs/tracing) | Structured logging |
| [windows](https://github.com/microsoft/windows-rs) | Windows platform APIs (transparent / always-on-top / click-through / window following) |

All dependency versions are declared in the root `Cargo.toml` under `[workspace.dependencies]`. For the full dependency list, see [Cargo.lock](https://github.com/eeymoo/peregrine/blob/main/Cargo.lock).

## Open Source Notes

Peregrine is released under the [MIT](https://opensource.org/licenses/MIT) license, fully open source.

**This means you may:**

- ✅ Freely use, modify, and distribute Peregrine (including commercial use)
- ✅ Read and learn from the full source code
- ✅ Submit Issues and Pull Requests to help improve the project

### Contributing

Issues and Pull Requests are welcome. Please see [`CONTRIBUTING.md`](https://github.com/eeymoo/peregrine/blob/main/CONTRIBUTING.md) in the repository for contribution guidelines, and the [Development Build](./development.md) page for how to build and test locally.
