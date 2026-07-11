# Getting Started

## Download and Install

Visit the [GitHub Releases](https://github.com/eeymoo/peregrine/releases) page and download the executable for your platform.

> Currently, Peregrine primarily supports Windows (x86 / x86_64 / ARM64).

## Launch the Program

After downloading, double-click `peregrine.exe` to run it. Once launched, the program displays the **config window**, where you can configure the visual anchor style and parameters.

## Common Operations

| Operation | Function |
|-----------|----------|
| Press `Esc` in the config window | Hide to system tray |
| Click **Start Overlay** | Create a transparent always-on-top overlay |
| Click **Stop Overlay** | Close the overlay |
| Right-click tray icon **Config** | Show the config window again |
| Right-click tray icon **Exit** | Exit the program |

## Step 1: Choose a Visual Anchor Style

1. In the settings panel, click the **Style** dropdown and choose a style you like, such as `Crosshair`, `Center Ring`, or `Edge Rectangle`.
2. Adjust parameters such as size, opacity, and color; the preview on the left updates in real time.
3. Click **Start Overlay**, and the visual anchor will appear at the center of the screen.

## Configuration File

The configuration file is located at:

- Windows: `%APPDATA%\Peregrine\config.json`

You can edit this file directly; the program will automatically hot reload after you save.
