# Recommended Configurations

This page first introduces common knowledge about 3D motion sickness, then provides Peregrine reference configurations for different types of games. For a complete explanation of the principles and general mitigation methods, please read [Relieving 3D Motion Sickness](./motion-sickness.md) first.

> ⚠️ **Disclaimer**: All recommended configurations below are **reference opinions only**, not authoritative prescriptions. Everyone's sensitivity to sensory conflict is different, and the best configuration varies from person to person. Use the recommended values as a starting point and fine-tune based on how you feel.

## Common Knowledge About 3D Motion Sickness

### Who Is More Prone to Motion Sickness

Susceptibility to 3D motion sickness varies from person to person. The following groups are usually more likely to experience symptoms[[1]](https://www.healthline.com/health/cybersickness)[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/):

- **Women**: statistically, women are more likely to experience motion sickness than men.
- **Migraine sufferers**: the visual-vestibular system is more sensitive in migraine patients.
- **People with inner ear / vestibular problems**: the vestibular system is already imbalanced, making it easier to be disturbed by virtual movement.
- **People with a history of carsickness, seasickness, or airsickness**: those who get motion sick in real life usually also get sick more easily in 3D games.
- **People who rarely play 3D games**: lack of adaptation makes symptoms more obvious on first exposure.
- **Children and teenagers**: the vestibular system is not yet fully developed, making them more sensitive to sensory conflict.

### Which Game Factors Worsen Motion Sickness

Many game settings affect the severity of motion sickness. The following are the most common "aggravating factors"[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html)[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/)[[5]](https://www.polygon.com/gaming/502730/indiana-jones-great-circle-motion-sick-settings-fix-head-bob/):

| Factor | Description |
|--------|-------------|
| **Narrow FOV** | A field of view below 90° creates a "telescope effect," making the screen feel like it is moving through a tunnel; the brain cannot perceive surrounding space. |
| **Low frame rate / frame drops** | Below 60 FPS or unstable frame rates cause screen tearing and stuttering, worsening sensory conflict. |
| **Motion Blur** | Simulates the blur of high-speed movement, but can make the brain misjudge the direction of movement. |
| **Head Bob / Camera Bob** | Simulates the up-and-down bumps of walking; for many people this is the strongest motion sickness trigger. |
| **Camera Shake** | Screen shake from explosions, firing, heavy hits, etc. |
| **Chromatic Aberration / Depth of Field / Vignette** | Post-processing effects deform or blur the edges of the screen, interfering with spatial perception. |
| **Fast turning / high sensitivity** | Excessively high mouse sensitivity causes rapid screen rotation and is one of the most common FPS motion sickness triggers. |
| **Vertical movement** | Climbing, flying, falling, and other large vertical view changes are more likely to cause sickness than horizontal movement. |
| **Lack of clear references in-game** | Fog, darkness, and vast empty scenes lack visual references, making it harder for the brain to orient itself. |

### Common Symptoms

Symptoms vary from person to person and usually worsen gradually in the following order[[1]](https://www.healthline.com/health/cybersickness):

1. **Mild**: slight dizziness, difficulty concentrating, slight sweating.
2. **Moderate**: obvious dizziness, headache, increased sweating, increased saliva, stomach discomfort.
3. **Severe**: nausea, urge to vomit, pale complexion, cold sweat, unable to continue playing.

### Motion Sickness Is Cumulative

3D motion sickness **does not disappear immediately after stopping the game**; symptoms usually last from 10 minutes to several hours. And multiple exposures on the same day can stack — if you already felt sick in the morning, you will feel sick more quickly when playing again in the evening. Therefore:

- **Stop immediately once you feel unwell**, do not push through.
- Do not play 3D games again until symptoms subside.
- Playing continuously over several days will gradually reduce symptoms (habituation effect), but symptoms may return after a break.

## Scenario-Based Reference Configurations

### Scenario 1: FPS (First-Person Shooter)

**Representative games**: CS2, Apex Legends, Call of Duty, Overwatch

**Motion sickness characteristics**: FPS is the genre most likely to cause motion sickness. First-person perspective + fast horizontal turning + firing camera shake combine to create very strong sensory conflict[[3]](https://www.alibaba.com/product-insights/why-do-i-get-motion-sickness-playing-fps-games-potential-fixes.html).

**Recommended configuration**:

| Parameter | Recommended Value | Description |
|-----------|-------------------|-------------|
| Style (`style`) | `cross` | Crosshair provides a precise center reference without blocking vision |
| Main size (`size`) | `8`–`12` | Small and precise, avoids blocking the screen |
| Thickness (`thickness`) | `2` | Thin lines are visible without being distracting |
| Gap (`gap`) | `4`–`6` | Leaves the center empty so it does not interfere with aiming |
| Opacity (`opacity`) | `0.5`–`0.6` | Semi-transparent, visible but not glaring |
| Color (`color`) | Green `[0.2, 1.0, 0.2, 1.0]` or cyan `[0.0, 1.0, 1.0, 1.0]` | Cool colors are highly visible in most scenes and do not blend into the sky like white |

**Principle**: The core source of FPS motion sickness is fast horizontal turning. The crosshair marks the exact center of the screen; when turning, the eyes can lock onto the crosshair, letting the brain confirm "I am actually not turning." The small size is to avoid blocking enemies — FPS requires constant attention to screen details.

**Extra suggestions**: Be sure to increase the in-game FOV (recommended 90°–100°) and lower mouse sensitivity to reduce fast turning.

---

### Scenario 2: Third-Person Shooter (TPS)

**Representative games**: PUBG, The Division, Gears of War

**Motion sickness characteristics**: Third-person perspective is **relatively less likely** to cause sickness than first-person because the camera is farther from the character, the field of view is wider, and camera movement is more stable[[4]](https://access-ability.uk/2022/04/25/gaming-with-motion-sickness/). However, games like PUBG have a lot of **camera shake while running** and **ADS (aiming down sights) view zoom**, which can still cause discomfort[[6]](https://forums.ea.com/discussions/battlefield-6-general-discussion-en/head-bobbing-and-camera-shake-causing-nausea/12760421).

**Recommended configuration**:

| Parameter | Recommended Value | Description |
|-----------|-------------------|-------------|
| Style (`style`) | `cross` or `ring` | Crosshair or center ring both work |
| Main size (`size`) | `10`–`15` (cross) / `ring_radius_pct: 0.04` | Slightly larger; third-person FOV is wider, so the center anchor can be bigger |
| Thickness (`thickness`) | `2`–`3` | Slightly thicker for increased visibility |
| Gap (`gap`) | `4`–`8` | |
| Opacity (`opacity`) | `0.4`–`0.5` | TPS is less sickness-inducing than FPS, so the anchor can be more subtle |
| Color (`color`) | White `[1.0, 1.0, 1.0, 1.0]` or light green `[0.3, 1.0, 0.3, 1.0]` | |

**Principle**: TPS motion sickness mainly comes from **running shake** and **view zoom**. The anchor's role is to provide a stable "horizon" reference when the camera shakes. The center ring (`ring`) works especially well in TPS because it does not rely on precise aiming — it only needs to provide a stationary center.

**Extra suggestions**: In game settings, turn off or reduce "camera shake," "head bob," and "ADS animation" options.

---

### Scenario 3: Parkour / High-Speed Movement

**Representative games**: Dying Light, Mirror's Edge, Titanfall 2

**Motion sickness characteristics**: Parkour games combine **high-speed movement + large amounts of vertical movement (jumping, sliding, climbing) + first-person perspective**, making them the most challenging genre. The developers of Dying Light even built a "camera stability" option into the game to relieve motion sickness[[7]](https://steelseries.com/blog/dying-light-the-beast-tips). The developers of Mirror's Edge used a minimalist white city design to reduce visual clutter[[8]](https://www.notebookcheck.net/Devs-Mirror-s-Edge-s-clean-white-city-a-practical-fix-not-a-pure-artistic-vision.1208014.0.html).

**Recommended configuration**:

| Parameter | Recommended Value | Description |
|-----------|-------------------|-------------|
| Style (`style`) | `ring` or `edge_rect` | Center ring is good for jump reference; edge rectangle provides extra edge reference |
| Main size (`size`) | `ring_radius_pct: 0.05` / `edge_rect`: `size: 100, secondary_size: 60` | |
| Thickness (`thickness`) | `2`–`3` | |
| Opacity (`opacity`) | `0.5`–`0.7` | Fast movement means the screen changes quickly, so the anchor needs to be more obvious |
| Anchor position (`anchor`) | `top` (if using `edge_rect`) | Top anchor simulates a "ceiling reference," helping quickly perceive height changes when jumping |
| Color (`color`) | High contrast, such as bright green `[0.0, 1.0, 0.3, 1.0]` or orange `[1.0, 0.6, 0.0, 1.0]` | Parkour scenes are visually complex, so a vivid color is needed |

**Principle**: Parkour game motion sickness comes from **multi-directional intense movement** — horizontal running, vertical jumping, and camera bumps happening at the same time. The center ring provides a reference for "my position relative to the ground" even in the most chaotic moments. The edge rectangle simulates a fixed frame, letting the brain perceive the picture boundary during intense shaking.

**Extra suggestions**: Be sure to turn off motion blur and head bob; prioritize upgrading skills or gear that improve "stability / shock absorption" (Dying Light's Explorer outfit has this effect)[[7]](https://steelseries.com/blog/dying-light-the-beast-tips).

---

### Scenario 4: Horizontal Movement Mainly

**Representative games**: 2D side-scrolling action, platformers, side-scrolling fighters

**Motion sickness characteristics**: Pure 2D side-scrolling games rarely cause motion sickness because the screen only moves horizontally with no depth deception. However, if you are playing a **3D-rendered side-scrolling game** (such as *Trine* or some scenes in *Ori*), or a 2.5D platformer, you may still feel mild discomfort.

**Recommended configuration**:

| Parameter | Recommended Value | Description |
|-----------|-------------------|-------------|
| Style (`style`) | `edge_rect` (anchor: `left` or `right`) | Side anchor that does not interfere with center controls |
| Main size (`size`) | `60`–`80` | |
| Thickness (`thickness`) | `2` | |
| Opacity (`opacity`) | `0.3`–`0.4` | Side-scrolling motion sickness is already mild, so the anchor can be very subtle |
| Anchor position (`anchor`) | `left` or `right` | Place on the opposite side of the movement direction to provide a "fixed background" reference |

**Principle**: During horizontal movement the whole screen pans left or right, and the brain most easily loses its sense of "where am I." A side anchor acts like a "wall"; when the screen scrolls to the right, the anchor stays still, letting the brain confirm that the screen is moving but I am not.

---

### Scenario 5: Vertical Movement Mainly

**Representative games**: climbing games, flight simulators, vertical scrollers, games with lots of falling mechanics

**Motion sickness characteristics**: **Vertical movement is more likely to cause motion sickness than horizontal movement** because the inner ear is more sensitive to upward/downward acceleration (gravity direction)[[2]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/). Large amounts of ascending, falling, and diving scenes strongly stimulate the vestibular system's sense of conflict.

**Recommended configuration**:

| Parameter | Recommended Value | Description |
|-----------|-------------------|-------------|
| Style (`style`) | `large_cross` or `border_frame` | Large crosshair provides complete horizontal and vertical reference lines; border frame provides an overall frame |
| Main size (`size`) | `large_cross`: screen radius | Large crosshair extends from the center to the edges, providing complete horizontal and vertical references |
| Thickness (`thickness`) | `1`–`2` | Thin lines avoid blocking too much of the screen |
| Opacity (`opacity`) | `0.3`–`0.5` | Vertical scenes can cause heavy sickness, but the anchor should also avoid being too eye-catching |
| Color (`color`) | White `[1.0, 1.0, 1.0, 1.0]` or light blue `[0.3, 0.5, 1.0, 1.0]` | |
| Border gap (`border_gap`) | `true` (if using `border_frame`) | Avoids blocking the top status bar and bottom mini-map |

**Principle**: During vertical movement the whole screen moves up or down, and the brain has the hardest time judging "am I rising or falling." The vertical line of a large crosshair is like a **plumb line**; no matter how the screen shakes, this line stays perpendicular to the ground, helping the brain confirm the true direction of gravity. The border frame acts like a fixed window framing the entire picture.

**Extra suggestions**: If the game has falling mechanics (such as jumping in *Sekiro* or falling in *Elden Ring*), try to control falling speed and avoid long free falls.

## General Tuning Advice

### Anchor Size

- **Smaller is better**, as long as it is visible. An anchor that is too large can itself become a new source of visual distraction.
- FPS / competitive games should prioritize small sizes (8–15 px); casual / exploration games can be slightly larger.
- If you cannot see it clearly, try **increasing opacity** rather than increasing size.

### Opacity

- Start at `0.4` and adjust until it is "noticeable but not distracting."
- Usually `0.3`–`0.6` is the comfortable range for most people.
- For games with generally dark screens (such as horror games), opacity can be slightly increased.

### Color

- Choose a color with **high contrast** against the game's main color palette.
  - Games with a lot of red / orange (war, fire scenes) → use cyan or green anchors.
  - Games with green forests / grasslands → use white or red anchors.
  - Games with sky / ocean blue tones → use orange or yellow anchors.
- White is the most universal choice, but it can blend into bright scenes (snow, sky).
- You can also use a custom PNG image (`CustomImage` style) for more refined visual effects.

### Position

- **Center** (`Cross` / `Ring`): the most common choice, suitable for most scenarios.
- **Edges** (`EdgeRect` / `BorderFrame`): when you do not want anything in the center of the screen, edge anchors provide a framing feel without interfering with controls.
- **Edge markers** (`CornerDots`): the least conspicuous option, placing small dots only in the four corners to provide minimal reference.

### Rule of Thumb

1. **Start from the default configuration**, play for 5–10 minutes, and see if it helps.
2. If you still feel sick, **try changing the style first** (cross → ring → edge_rect → border_frame) to find the anchor type that suits you best.
3. After finding the right style, **fine-tune size and opacity**.
4. **Change only one parameter at a time**, so you can tell which change is having an effect.
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
