# Peregrine Help

This document is for **end users**. It explains what Peregrine is, how to use it, and how each visual anchor style works and can be adjusted. For development-related information, see [README](README.md).

---

## 1. What is Peregrine?

Peregrine is a desktop utility focused on relieving 3D motion sickness. It provides customizable visual anchors on your screen (such as crosshairs, borders, edge arrows, and more) to help players maintain visual stability during fast camera movement or complex scenes, reducing nausea and dizziness caused by vestibular-visual conflict. This lets you comfortably play games that often trigger motion sickness, such as *Half-Life 2*, *Mirror's Edge*, *Dying Light*, and *No Man's Sky*.

Many people feel dizzy or nauseous when playing first-person or third-person 3D games. One reason is that when the image shakes violently, the eyes cannot find a **fixed reference point** on the screen. Peregrine draws visual anchors (crosshairs, edge markers, borders, etc.) in the center or at the edges of the screen that always stay still, giving the eyes a stable focal point and reducing motion sickness.

> The default "Edge Rect" style is inspired by the trick many motion-sickness-prone players use: literally sticking a small piece of paper in the middle of the screen as a reference.

---

## 2. Quick Start

1. When you launch the app, the **settings window** opens. The settings panel is on the right, and the live preview is on the left.
2. Choose a style you like from the **Type** dropdown, then adjust the sliders below for size, color, and opacity. The preview on the left updates **instantly**.
3. Select the game window you want to overlay from the **Target Window** dropdown.
4. Click **Start Overlay**, and the visual anchor will appear on top of the game window.

### Shortcuts

| Key | Action |
| --- | --- |
| `Esc` | Hide the settings window to the system tray |

All your settings are **saved automatically**. The next time you open the app, your configuration will be restored—no need to save manually.

---

## 3. General Settings

No matter which style you choose, these options are always available:

- **Opacity**: 0 = completely invisible, 1 = fully opaque. For anti-motion-sickness use, set this **low** (for example, 0.3–0.6) so the anchor is visible without blocking the view.
- **Color**: Click the color swatch to choose freely. We recommend picking a color that **contrasts strongly** with the game image (bright green, magenta, etc.) so your eyes can "lock onto" it more easily.

---

## 4. Style Reference

Choose a style from the **Type** dropdown in the configuration panel. Each style has different adjustable options.

### Edge Rect (Default)

A semi-transparent rectangle in the center of the screen, which can snap to any of the four screen edges or stay centered. The simplest and most direct center reference.

- **Width / Height**: Rectangle size (10–400 / 10–300 pixels).
- **Corner Radius**: How rounded the rectangle's corners are.
- **Anchor / Margin**: Lets the rectangle snap to a screen edge instead of staying dead center.

### Cross

A small crosshair at the exact center of the screen, taking up minimal space.

- **Arm Length**: Length of each cross line.
- **Line Width**: Thickness of the lines.
- **Center Gap**: Size of the empty space in the very middle of the cross (creates a "hollow cross" when set).

### Large Cross

A horizontal and a vertical line that extend from the screen edges all the way to the center, dividing the screen into four quadrants. Provides a stronger sense of reference.

- **Line Width**: Thickness of the lines.

### 4 / 6 / 8 Corner Dots

Place several dots around the screen edges as anchors without blocking the central view. Great for scenes where you need to see the middle of the screen clearly.

- **4 Corner Dots**: One dot in each corner.
- **6 Corner Dots**: Four corners plus the middle of the top and bottom edges.
- **8 Corner Dots**: Four corners plus the middle of the top, bottom, left, and right edges.
- Adjustable **distance from edge**, **radius** (set to 0 to auto-calculate from line width), and **line width**.

### Ring

A circular ring centered on the screen, providing an annular anchor.

- **Radius Ratio to Screen Height**: Size of the ring, relative to screen height.
- **Line Width** and **Ring Style**: Solid / Dashed / Double.

### Custom Orb

Decide for yourself how many dots to place along the top, bottom, left, and right edges.

- Check to enable the **top / bottom / left / right** edges.
- Set the **dot count** (1–10), **radius**, and **distance from edge** for each edge independently.

### Random Orb

Randomly generates a batch of dots in the screen edge areas at startup. Positions and sizes may differ each time, suiting users who prefer irregular references.

- **Dots per edge**, **distance from edge**, **position jitter**, and **min / max radius** are all adjustable.
- Operating modes (must be changed in the configuration file):
  - `lock_on_start`: The first random layout is **locked in**; subsequent launches use the same positions.
  - `reshuffle`: **Re-randomized on every launch**.

### Border Frame

Draws a line around all four edges of the screen, like a film "safe area" frame, providing a boundary reference.

- **Bar height** and **distance from edge** are adjustable.
- **Style**: Full frame, or frame with gaps in the middle of each side.
- Check **20% gap in the middle of each side** to avoid covering HUD elements such as minimaps and health bars.

### Edge Arrows

Draws arrows pointing toward the center along all four screen edges. You can add tail lengths for a strong directional reference.

- **Arrow size**, **arrow width**, and **tail length** are adjustable.
- You can set a uniform tail length for all sides, or configure each side independently.

### Custom Image

Load any PNG image as a visual anchor. The app decodes it and draws it at the center of the screen.

- Supports **scale** and **horizontal / vertical offset** adjustments.

---

## 5. Profiles

Peregrine supports multiple **Profiles** so you can save a separate set of settings for each game or scenario. Switching the active Profile applies the corresponding style. For now, adding and switching Profiles is mainly done through the [configuration file](#7-configuration-file).

---

## 6. Tips for Reducing Motion Sickness

- **Start with "Edge Rect" or "Cross"**—they are the most intuitive.
- **Don't set opacity too high**; a solid anchor is more tiring to look at. Faintly visible is enough.
- **Pick a high-contrast color** that stands out against the game's main color palette.
- If you get very motion sick, try a "screen-filling" style such as **Large Cross** or **Border Frame**, which provides more reference information.
- After playing for a while, you can gradually **shrink or fade** the anchor so your eyes adapt.

---

## 7. Configuration File

All settings are saved as JSON in the system's standard directory:

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%/Peregrine/config.json` |
| macOS | `~/Library/Application Support/Peregrine/config.json` |
| Linux | `~/.config/Peregrine/config.json` |

- A default configuration is **generated automatically** on first launch, so manual editing is usually unnecessary.
- You can also edit this file directly with a text editor. The app **automatically reloads** the configuration after you save (hot-reload), so no restart is needed.
- Some advanced options (such as the Random Orb operating mode and Profile management) can currently only be changed in the configuration file.
- If the configuration gets corrupted, delete this file and restart the app to regenerate a default configuration.

---

## 8. Frequently Asked Questions

**Q: After switching to the overlay, the area behind the anchor is white / opaque, and I can't see the game.**
A: In the current version, the Windows overlay uses per-pixel alpha for true transparency, always-on-top behavior, and mouse pass-through. If this still happens, please make sure your graphics drivers are up to date and that DWM (Desktop Window Manager) is running normally.

**Q: I can't find the game window in the "Target Window" dropdown.**
A: Please confirm you are on Windows (target window following is only supported on Windows). The dropdown should list the titles of currently open windows; after selecting one, the overlay will automatically follow that window's position and size.

**Q: What do "Trigger Rules / Process Trigger" in the settings do?**
A: These are planned for "automatically show the anchor when a specific game process is detected." They are currently **placeholders** and not yet active.

**Q: After resizing the window, the anchor position is wrong.**
A: The overlay automatically adjusts to follow the target window's position and size changes. If you select "(None)" the overlay will not follow any window, and the anchor will remain fixed at the center of the screen.

**Q: I lost my settings / how do I restore defaults?**
A: Close the app, delete `config.json` at the path above, and restart. The default configuration will be restored.

---

> Note: v0.1.0 stable has been released. The transparent click-through overlay, target window following, multiple visual anchor styles, and custom PNG image support are available on Windows. Process trigger is a placeholder feature.
