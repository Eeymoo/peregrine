# Usage

## Download

1. Open the [GitHub Releases](https://github.com/eeymoo/peregrine/releases) page.
2. Download the latest release archive for your system:
   - `peregrine-<version>-windows-x64.zip` (64-bit systems, most common)
   - `peregrine-<version>-windows-x86.zip` (32-bit systems)
   - `peregrine-<version>-windows-arm64.zip` (ARM64 devices)
3. Extract it to any folder.

## Launch

Double-click the extracted `peregrine.exe`. The application opens the **config window** by default.

## Basic Operations

| Key | Action |
|------|------|
| `Esc` | Hide the config window to the system tray |

## Getting Started in Three Steps

0. **Select target window**: First, choose the game window you want to follow from the **Target Window** dropdown. If none is selected, clicking Start Overlay will have no effect.
1. **Select style**: In the settings panel, find **Style** and choose styles such as `Crosshair`, `Center Ring`, or `Edge Rectangle`.
2. **Adjust appearance**: Modify **Size**, **Line Width**, **Color**, **Opacity**, and other parameters. The preview on the left updates in real time.
3. **Enter the game**: Click **Start Overlay**, and the overlay will appear at the center of the screen without blocking mouse or keyboard input.

## Custom PNG Image

1. Prepare a PNG image with a transparent background.
2. In the settings panel, switch **Style** to `Custom Image`.
3. Paste the PNG file path into the file path text box (or use the **Browse...** button), and adjust the scale and offset.

## Follow Game Window

Select a game window from the **Target Window** dropdown, and Peregrine will try to follow that window as it moves. Select **(Not selected)** to keep the overlay fixed at the center of the screen.

## Exit the Application

Right-click the system tray icon and select **Exit**, or end the process in Task Manager.

## Having Issues?

- See [Configuration](./config.md) for the configuration file format.
- Report issues on [GitHub Issues](https://github.com/eeymoo/peregrine/issues).
