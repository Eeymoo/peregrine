# Peregrine

[English](README.md) · [简体中文](README.zh-cn.md)

[![Download](https://img.shields.io/github/v/release/Eeymoo/peregrine?style=for-the-badge&label=Download)](https://github.com/Eeymoo/peregrine/releases)

Peregrine is a desktop utility focused on relieving 3D motion sickness. It provides customizable visual anchors on your screen—such as crosshairs, borders, and edge arrows—to help players maintain visual stability during fast camera movement or complex scenes, reducing nausea and dizziness caused by vestibular-visual conflict. This lets you comfortably play games that often trigger motion sickness, such as *Half-Life 2*, *Mirror's Edge*, *Dying Light*, and *No Man's Sky*.

> **Current status: stable release.** The transparent, always-on-top, click-through overlay on Windows, target window following, multiple visual anchor styles, custom PNG image support, and configuration hot-reload are all available.
>
> For end-user instructions, see **[Help (HELP.md)](HELP.md)**.
> For contribution guidelines, see **[CONTRIBUTING.md](CONTRIBUTING.md)**.

## Quick Start

### Download and Run (Recommended)

1. Go to the **[Releases page](https://github.com/Eeymoo/peregrine/releases)** and download the latest version.
2. Choose the package for your system:
   - `peregrine-v*-windows-x64.zip` — 64-bit systems
   - `peregrine-v*-windows-x86.zip` — 32-bit systems
   - `peregrine-v*-windows-arm64.zip` — ARM devices
3. Extract the archive and run `peregrine.exe`. The app opens the **settings window** by default.
4. Select a target window, adjust the style, then click **Start Overlay** to display the visual anchor on top of your game.

> A default configuration is generated automatically on first launch, and all settings are saved automatically. Closing the settings window minimizes the app to the system tray.

### Build from Source

You need the Rust toolchain (≥ 1.85) and Node.js (≥ 20). Run the following from the repository root:

```bash
# Install frontend dependencies
npm install

# Build the workspace
cargo build
cargo build --release

# Run the Tauri development version
npx tauri dev

# Build Tauri release artifacts
npx tauri build

# Run tests
cargo test                      # all
cargo test -p peregrine_config  # config crate only

# Format / lint
cargo fmt
cargo clippy -p peregrine_config -- -D warnings
```

## Features

- **Transparent overlay window on Windows**: an always-on-top, click-through overlay that floats above games or apps.
- **Target window following**: select a target window from the dropdown and the overlay follows its position and size.
- Multiple visual anchor styles: Edge Rect, Cross, Large Cross, corner dots (4/6/8), Center Ring, Custom Orb, Random Orb, Border Frame, and Edge Arrows.
- **Custom PNG image**: load any PNG image as the overlay content.
- Each style supports adjustable size, thickness, color, opacity, gap, edge position, and more.
- Multiple profiles: save independent settings for different scenarios.
- Real-time preview: see changes instantly as you adjust parameters in the settings panel.
- Persistent configuration + hot-reload: changes made externally take effect automatically.
- **Tauri + React settings UI**: the settings panel is built on Webview, making it easy to extend and theme.
- **GitHub Actions automated builds**: Windows x86 / x86_64 / ARM64 builds and releases.

## Tech Stack

- **Language / ecosystem**: Rust, Cargo workspace (`edition = "2024"`, `rust-version = 1.85`, MIT licensed).
- **Settings UI**: [Tauri](https://tauri.app/) 2.x + [React](https://react.dev/) 18 + [Tailwind CSS](https://tailwindcss.com/) 3 + [shadcn/ui](https://ui.shadcn.com/).
- **Overlay**: [winit](https://github.com/rust-windowing/winit) (window / event loop) + [softbuffer](https://github.com/rust-windowing/softbuffer) (CPU pixel rasterization).
- **Async runtime**: [tokio](https://github.com/tokio-rs/tokio) (configuration read/write, file hot-reload, background follow task).
- **Target platform**: **Windows** (x86 / x86_64 / ARM64).

### System Tray

After launch the app lives in the system tray:

| Menu | Behavior |
| --- | --- |
| Settings | Show / focus the settings window |
| Preferences | Show system-level preferences such as About |
| Exit | Quit the application completely |

Left-clicking the tray icon also restores the settings window.

## Project Structure

```
peregrine/
├── Cargo.toml            # workspace root: members, workspace.package, workspace.dependencies
├── Cargo.lock
├── package.json          # frontend npm dependencies
├── src/                  # frontend source (React + Tailwind + shadcn/ui)
│   ├── ConfigApp.tsx     # settings window main component
│   ├── SettingsApp.tsx   # preferences window main component
│   ├── lib/
│   │   ├── api.ts        # Tauri invoke wrapper
│   │   ├── shapes.ts     # frontend preview geometry calculations
│   │   └── i18n.tsx      # internationalization
│   ├── components/
│   │   ├── Preview.tsx   # Canvas real-time preview
│   │   ├── StyleFields.tsx # style parameter form
│   │   └── ui/           # shadcn/ui base components
│   └── i18n/             # multilingual JSON
├── src-tauri/            # peregrine-tauri: Tauri backend entry
│   └── src/
│       ├── lib.rs        # Tauri startup entry, config init, tray, commands
│       ├── main.rs       # binary entrypoint
│       └── overlay.rs    # runs winit event loop on a dedicated thread
├── crates/
│   ├── config/           # peregrine_config: pure logic crate (no UI / GPU / window code)
│   │   └── src/
│   │       ├── lib.rs        # module exports + unified error type
│   │       ├── schema.rs     # configuration data structures + validation + unit tests
│   │       ├── storage.rs    # config file path management, atomic read/write, default config
│   │       ├── notifier.rs   # config change broadcast based on tokio::sync::watch
│   │       └── watcher.rs    # config file hot-reload based on notify crate (with debouncing)
│   └── peregrine/        # peregrine: shared library reused by Tauri
│       └── src/
│           ├── lib.rs             # exports overlay_renderer / shapes / platform
│           ├── overlay_renderer.rs # softbuffer overlay renderer
│           ├── shapes.rs           # shared crosshair geometry module
│           └── platform/
│               ├── mod.rs          # platform module entry
│               └── windows.rs      # Win32 API: transparency / always-on-top / click-through, target window lookup and following
└── docs/                 # VitePress documentation site
```

**Layering principle**: `peregrine_config` must not depend on any UI / GPU / window platform code (`winit` / `wgpu` / `egui`). Platform and rendering logic belong in the `peregrine` shared library and the `src-tauri` binary crate.

## Configuration

The configuration file is JSON. Its path depends on the OS standard directories:

| Platform | Path |
| --- | --- |
| Windows | `%APPDATA%/Peregrine/config.json` |
| macOS | `~/Library/Application Support/Peregrine/config.json` |
| Linux | `~/.config/Peregrine/config.json` |

> Peregrine is a Windows-only tool. The config paths above are listed because the configuration crate uses OS-standard directories; the overlay and core user-facing features are implemented only on Windows.

- The configuration root is `AppConfig` (`active_profile` + multiple named `Profile`s). A default configuration is generated automatically on first launch.
- Writes are atomic (temp file in the same directory + `rename`), and validation is always performed before writing to avoid persisting invalid config.
- Hot-reload is supported: after the configuration file is edited externally, `ConfigWatcher` (notify + 300 ms debounce) detects the change and broadcasts it to the renderer.

Configuration flow:

```
UI change / external file edit
   → ConfigStorage (atomic write) / ConfigWatcher (notify + debounce)
   → ConfigNotifier (watch broadcast)
   → subscribers (shared snapshot read by the renderer)
```

## License

MIT License.
