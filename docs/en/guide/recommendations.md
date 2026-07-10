# Recommended Configurations

This page first covers common knowledge about 3D motion sickness, then provides Peregrine reference configurations for different game types. For a complete explanation of the motion-sickness mechanism and general mitigation methods, please read [Relieving 3D Motion Sickness](./motion-sickness.md) first.

> ⚠️ **Disclaimer**: All recommended configurations below are **for reference only** and not authoritative prescriptions. Everyone's sensitivity to sensory conflict varies, and the best configuration is individual. Use the recommended values as a starting point and fine-tune based on how you feel.

## Common Knowledge About 3D Motion Sickness

### Who Is More Likely to Get Motion Sickness

Susceptibility to 3D motion sickness varies from person to person. The following groups are usually more prone to symptoms[[1]](https://www.healthline.com/health/cybersickness)[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/):

- **Women**: Statistically, women are more likely to experience motion sickness than men.
- **Migraine sufferers**: The visual-vestibular system is more sensitive in people with migraines.
- **People with inner-ear / vestibular issues**: The vestibular system is already imbalanced, making it more easily disrupted by virtual motion.
- **People with a history of car, sea, or air sickness**: Those who get motion sick in real life usually get motion sick more easily in 3D games.
- **People who rarely play 3D games**: Lack of adaptation means symptoms are more obvious on first contact.
- **Children and adolescents**: The vestibular system is not fully developed, making them more sensitive to sensory conflict.

### Which Game Factors Worsen Motion Sickness

Many game settings affect the severity of motion sickness. The following are the most common aggravating factors[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)[[5]](https://www.polygon.com/gaming/502730/indiana-jones-great-circle-motion-sick-settings-fix-head-bob/):

| Factor | Explanation |
|------|------|
| **Narrow FOV** | A field of view below 90° creates a "telescope effect"; the screen feels like it's moving inside a tunnel, and the brain cannot perceive the surrounding space. |
| **Low / unstable frame rate** | Frame rates below 60 FPS or unstable frame rates cause tearing and stuttering, aggravating sensory conflict. |
| **Motion blur** | Simulates blurring during high-speed motion but causes the brain to misjudge direction. |
| **Head bob / camera bob** | Simulates the up-and-down sway of walking and is a strong motion-sickness trigger for many people. |
| **Camera shake** | Screen shake during explosions, gunfire, heavy hits, and similar scenes. |
| **Chromatic aberration / depth of field / vignette** | Post-processing effects deform or blur the edges of the screen, interfering with spatial perception. |
| **Fast turning / high sensitivity** | Excessively high mouse sensitivity causes rapid screen rotation and is one of the most common FPS motion-sickness triggers. |
| **Vertical movement** | Climbing, flying, and falling involve large amounts of up-and-down camera movement and are more likely to cause motion sickness than horizontal movement. |
| **No clear reference objects in-game** | Fog, darkness, and vast open scenes lack visual references, making it harder for the brain to orient itself. |

### Common Symptoms

Symptoms vary by individual and usually worsen in the following order[[1]](https://www.healthline.com/health/cybersickness):

1. **Mild**: Slight dizziness, difficulty concentrating, mild sweating.
2. **Moderate**: Noticeable dizziness, headache, increased sweating, increased saliva, stomach discomfort.
3. **Severe**: Nausea, urge to vomit, pale complexion, cold sweat, inability to continue playing.

### Motion Sickness Accumulates

3D motion sickness **does not disappear immediately after stopping the game**; symptoms usually last from 10 minutes to several hours. Multiple exposures on the same day can also stack—if you already felt sick in the morning, symptoms will appear faster when you play again in the evening. Therefore:

- **Stop immediately once you feel discomfort**; do not push through it.
- Do not play 3D games again until symptoms subside.
- Playing on consecutive days can gradually reduce symptoms (adaptation effect), but symptoms may return after a break.

## Scenario-Based Reference Configurations

### Scenario 1: FPS (First-Person Shooter)

**Representative games**: CS2, Apex Legends, Call of Duty, Overwatch

**Motion-sickness characteristics**: FPS games are the most likely type to cause motion sickness. First-person perspective + fast horizontal turning + camera shake when firing combine to create very strong sensory conflict[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html).

**Recommended configuration**:

| Parameter | Recommended value | Explanation |
|------|--------|------|
| Style (style) | `cross` | Crosshair provides a precise central reference without blocking the view |
| Main size (size) | `8`–`12` | Small and precise, avoiding screen obstruction |
| Thickness (thickness) | `2` | Thin lines are visible without being distracting |
| Gap (gap) | `4`–`6` | Leaves the center empty so aiming is not interfered with |
| Opacity (opacity) | `0.5`–`0.6` | Semi-transparent so it is visible but not glaring |
| Color (color) | Green `[0.2, 1.0, 0.2, 1.0]` or cyan `[0.0, 1.0, 1.0, 1.0]` | Cool colors are easy to distinguish in most scenes and don't blend into the sky like white |

**Principle**: The core motion sickness in FPS comes from fast horizontal turning. The crosshair marks the exact center of the screen; when turning, the eyes can lock onto the crosshair, letting the brain confirm "I'm not actually turning". The small size avoids blocking enemies—FPS games require constant attention to screen details.

**Additional advice**: Be sure to increase the in-game FOV (recommended 90°–100°) and lower mouse sensitivity to reduce fast turning.

---

### Scenario 2: Third-Person Shooter (TPS)

**Representative games**: PUBG, The Division, Gears of War

**Motion-sickness characteristics**: Third-person perspective is **relatively less likely** to cause motion sickness than first-person because the camera is farther from the character, the field of view is wider, and camera movement is more stable[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/). However, games like PUBG have a lot of **camera shake while running** and **perspective zoom when aiming down sights (ADS)**, which can still cause discomfort[[6]](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/head-bobbing-and-camera-shake-causing-nausea/12760421).

**Recommended configuration**:

| Parameter | Recommended value | Explanation |
|------|--------|------|
| Style (style) | `cross` or `ring` | Either crosshair or center ring works |
| Main size (size) | `10`–`15` (cross) / `ring_radius_pct: 0.04` | Slightly larger; the wider TPS field of view can accommodate a larger central anchor |
| Thickness (thickness) | `2`–`3` | Slightly thicker to increase visibility |
| Gap (gap) | `4`–`8` | |
| Opacity (opacity) | `0.4`–`0.5` | Third-person is less likely to cause motion sickness than FPS, so the anchor can be more subtle |
| Color (color) | White `[1.0, 1.0, 1.0, 1.0]` or light green `[0.3, 1.0, 0.3, 1.0]` | |

**Principle**: Motion sickness in third-person mainly comes from **running shake** and **perspective zoom**. The anchor provides a stable "horizon" reference when the camera shakes. The center ring (`ring`) works especially well in TPS because it doesn't rely on precise aiming and only needs to provide a stationary center point.

**Additional advice**: In game settings, turn off or reduce "camera shake", "head bob", and "ADS animation" options.

---

### Scenario 3: Parkour / High-Speed Movement

**Representative games**: Dying Light, Mirror's Edge, Titanfall 2

**Motion-sickness characteristics**: Parkour games combine **high-speed movement + lots of vertical motion (jumping, sliding, climbing) + first-person perspective**, making them the most challenging type. The developers of Dying Light even built a "camera stability" option into the game to relieve motion sickness[[7]](https://steelseries.com/blog/dying-light-the-beast-tips). The developers of Mirror's Edge used a minimalist white city design to reduce visual clutter[[8]](https://www.notebookcheck.net/Devs-Mirror-s-Edge-s-clean-white-city-a-practical-fix-not-a-pure-artistic-vision.1208014.0.html).

**Recommended configuration**:

| Parameter | Recommended value | Explanation |
|------|--------|------|
| Style (style) | `ring` or `edge_rect` | Center ring is suitable for jumping reference; edge rectangle provides additional edge reference |
| Main size (size) | `ring_radius_pct: 0.05` / `edge_rect`: `size: 100, secondary_size: 60` | |
| Thickness (thickness) | `2`–`3` | |
| Opacity (opacity) | `0.5`–`0.7` | Fast screen changes require a more obvious anchor |
| Anchor position (anchor) | `top` (if using edge_rect) | Top anchor simulates a "ceiling reference" and helps quickly perceive height changes when jumping |
| Color (color) | High-contrast colors such as bright green `[0.0, 1.0, 0.3, 1.0]` or orange `[1.0, 0.6, 0.0, 1.0]` | Complex parkour scenes need eye-catching colors |

**Principle**: Motion sickness in parkour games comes from **multi-directional intense motion**—horizontal running + vertical jumping + camera bobbing happening at the same time. The center ring provides a reference for "my position relative to the ground" even in the most chaotic moments. The edge rectangle simulates a fixed frame, letting the brain perceive the screen boundary during violent shaking.

**Additional advice**: Be sure to turn off motion blur and head bob; prioritize skills or equipment that improve "stability / damping" (the Explorer outfit in Dying Light has this effect)[[7]](https://steelseries.com/blog/dying-light-the-beast-tips).

---

### Scenario 4: Lateral Movement Dominant

**Representative games**: 2D side-scrolling action, platformers, side-scrolling fighters

**Motion-sickness characteristics**: Pure 2D side-scrolling games rarely cause motion sickness because the screen only moves horizontally and does not trick depth perception. However, if you are playing a **3D-rendered side-scrolling game** (such as Trine or some scenes in Ori) or a 2.5D platformer, you may still experience mild discomfort.

**Recommended configuration**:

| Parameter | Recommended value | Explanation |
|------|--------|------|
| Style (style) | `edge_rect` (anchor: `left` or `right`) | Side anchor that does not interfere with center-screen actions |
| Main size (size) | `60`–`80` | |
| Thickness (thickness) | `2` | |
| Opacity (opacity) | `0.3`–`0.4` | Side-scrolling motion sickness is already mild, so the anchor can be very subtle |
| Anchor position (anchor) | `left` or `right` | Place on the side opposite to the movement direction to provide a "fixed background" reference |

**Principle**: During lateral movement, the whole screen shifts left or right, and the brain most easily loses the sense of "where am I". A side anchor acts like a "wall"; when the screen scrolls to the right and the anchor stays still, the brain can confirm that the screen is moving while I am not.

---

### Scenario 5: Vertical Movement Dominant

**Representative games**: Climbing games, flight simulators, vertical scrollers, games with lots of falling mechanics

**Motion-sickness characteristics**: **Vertical movement is more likely to cause motion sickness than horizontal movement** because the inner ear is more sensitive to up-and-down acceleration (gravity direction)[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/). Scenes with a lot of ascending, falling, and diving strongly stimulate the vestibular system's sense of conflict.

**Recommended configuration**:

| Parameter | Recommended value | Explanation |
|------|--------|------|
| Style (style) | `large_cross` or `border_frame` | Large crosshair provides complete horizontal and vertical reference lines; border frame provides an overall frame |
| Main size (size) | `large_cross`: screen radius | Large crosshair extends from the center to the edge, providing complete horizontal and vertical reference |
| Thickness (thickness) | `1`–`2` | Thin lines avoid blocking too much of the screen |
| Opacity (opacity) | `0.3`–`0.5` | Vertical scenes can cause severe motion sickness, but the anchor should also avoid being too distracting |
| Color (color) | White `[1.0, 1.0, 1.0, 1.0]` or light blue `[0.3, 0.5, 1.0, 1.0]` | |
| Border gap (border_gap) | `true` (if using border_frame) | Avoid blocking the top status bar and bottom minimap |

**Principle**: During vertical movement, the whole screen moves up or down, and the brain has the hardest time judging "am I rising or falling". The vertical line of the large crosshair acts like a **plumb line**; no matter how the screen shakes, this line always stays perpendicular to the ground, helping the brain confirm the true direction of gravity. The border frame acts like a fixed window that frames the entire screen.

**Additional advice**: If the game has falling mechanics (such as jumping in Sekiro or falling in Elden Ring), try to control descent speed and avoid long periods of free fall.

## General Tuning Advice

### Anchor Size

- **Smaller is better**, as long as it remains visible. An anchor that is too large can itself become a new source of visual distraction.
- For FPS / competitive games, prefer small sizes (8–15px); casual / exploration games can be slightly larger.
- If you can't see it clearly, try **increasing opacity** first rather than increasing size.

### Opacity

- Start at `0.4` and gradually adjust to a level that is "noticeable but not distracting".
- Usually `0.3`–`0.6` is the comfortable range for most people.
- For games with generally dark visuals (such as horror games), opacity can be increased slightly.

### Color

- Choose a color with **high contrast** against the game's main color palette.
  - Games with a lot of red / orange (such as war or fire scenes) → use cyan or green anchors.
  - Green forests / grasslands → use white or red anchors.
  - Sky / ocean blue tones → use orange or yellow anchors.
- White is the most universal choice but can blend into bright scenes (snow, sky).
- You can also use a custom PNG image (`CustomImage` style) for finer visual effects.

### Position

- **Center** (`Cross` / `Ring`): The most common choice, suitable for most scenarios.
- **Edge** (`EdgeRect` / `BorderFrame`): When you don't want anything in the dead center of the screen, edge anchors provide a frame feel without interfering with actions.
- **Corner markers** (`CornerDots`): The least conspicuous option; only small dots at the four corners provide a minimal reference.

### Rules of Thumb

1. **Start with the default configuration**, play for 5–10 minutes, and see if it helps.
2. If you still feel motion sick, **try changing the style first** (cross → ring → edge_rect → border_frame) to find the anchor type that suits you best.
3. After finding the right style, **fine-tune size and opacity**.
4. **Change only one parameter at a time** so you can tell which change is doing the work.
5. Record your most comfortable configuration combination and save it as a Profile for easy recall later.

## References

- [[1] Healthline — Cybersickness: What It Is, Symptoms, Causes, and Treatments](https://www.healthline.com/health/cybersickness)
- [[2] Karmali, F. et al. — Validating sensory conflict theory and mitigating motion sickness (NIH PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)
- [[3] Alibaba Product Insights — Why Do I Get Motion Sickness Playing FPS Games: Potential Fixes](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)
- [[4] Access Ability — Gaming with Motion Sickness](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)
- [[5] Polygon — How to fix Indiana Jones and the Great Circle motion sickness problems](https://www.polygon.com/gaming/502730/indiana-jones-great-circle-motion-sick-settings-fix-head-bob/)
- [[6] EA Forums — Head bobbing and camera shake causing nausea](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/head-bobbing-and-camera-shake-causing-nausea/12760421)
- [[7] SteelSeries — How to Survive the Night in Dying Light: The Beast](https://steelseries.com/blog/dying-light-the-beast-tips)
- [[8] Notebookcheck — Devs: Mirror's Edge's clean white city a practical fix](https://www.notebookcheck.net/Devs-Mirror-s-Edge-s-clean-white-city-a-practical-fix-not-a-pure-artistic-vision.1208014.0.html)
