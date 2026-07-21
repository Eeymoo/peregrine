# Usage Guide

## Download

1. Open the [GitHub Releases](https://github.com/eeymoo/peregrine/releases) page.
2. Download the latest stable release archive for your system:
   - `peregrine-<version>-windows-x64.zip` (64-bit systems, most common)
   - `peregrine-<version>-windows-x86.zip` (32-bit systems)
   - `peregrine-<version>-windows-arm64.zip` (ARM64 devices)
3. Extract the archive to any folder.

## Launch

Double-click `peregrine.exe` after extraction. The app will open the **Settings window** by default.

## Basic Controls

| Key | Action |
|------|------|
| `Esc` | Hide the settings window to the system tray |

## Get Started in Three Steps

0. **Select a target window**: First, choose the game window you want to follow from the **Target Window** dropdown. If no window is selected, clicking **Start Overlay** will not take effect.
1. **Choose a style**: In the settings panel, find **Style** and select options like `Cross`, `Center Ring`, or `Edge Rect`.
2. **Adjust appearance**: Modify parameters such as **Size**, **Thickness**, **Color**, and **Opacity**. The preview on the left updates in real time.
3. **Enter the game**: Click **Start Overlay**, and the visual anchor will appear at the center of the screen without blocking mouse or keyboard input.

::: tip Fullscreen Game Tip
If the game is running in **Exclusive Fullscreen** mode, Peregrine's overlay may not appear above the game. We recommend setting the game display mode to **Borderless Fullscreen** (or fullscreen windowed), then clicking **Start Overlay**.
:::

## Custom PNG Decal

1. Prepare a PNG image with a transparent background.
2. In the settings panel, switch **Style** to `Custom Image`.
3. Paste the PNG file path into the file path text box (or use the **Browse...** button), and adjust the scale and offset.

## Follow a Game Window

Select a game window from the **Target Window** dropdown, and Peregrine will try to follow its movement. Select **(Not selected)** to keep the anchor fixed at the center of the screen.

## Exit the Program

Right-click the system tray icon and choose **Exit**, or end the process in Task Manager.

## Need Help?

- Check the [Configuration Guide](./config) to learn about the config file format.
- Report issues on [GitHub Issues](https://github.com/eeymoo/peregrine/issues).
