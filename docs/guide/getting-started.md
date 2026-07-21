# Quick Start

## Download & Install

Visit the [GitHub Releases](https://github.com/eeymoo/peregrine/releases) page and download the executable for your platform.

> Currently, Windows is the primary supported platform (x86 / x86_64 / ARM64).

## Launch the App

After downloading, double-click `peregrine.exe` to run it. When the app starts, the **settings window** will appear; configure the visual anchor style and parameters there.

## Common Actions

| Action | Effect |
|------|------|
| Press `Esc` in the settings window | Hide to system tray |
| Click **Start Overlay** | Create the transparent always-on-top overlay |
| Click **Stop Overlay** | Close the overlay |
| Right-click tray icon → **Settings** | Show the settings window again |
| Right-click tray icon → **Exit** | Quit the app |

## Step 1: Choose a Visual Anchor Style

1. In the settings panel, click the **Style** dropdown and choose a style you like, such as `Crosshair`, `Center Ring`, or `Edge Rectangle`.
2. Adjust parameters like size, opacity, and color; the preview on the left updates in real time.
3. Click **Start Overlay**, and the visual anchor will appear at the center of the screen.

## Config File

The config file is located at:

- Windows: `%APPDATA%\Peregrine\config.json`

You can edit this file directly; the app will automatically hot-reload after you save.
