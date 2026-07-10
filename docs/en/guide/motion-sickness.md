# Relieving 3D Motion Sickness

This page explains what 3D motion sickness is, why it happens, and how to relieve it. Peregrine's visual anchor is only one of many mitigation techniques; this chapter also lists other proven methods to help you find the combination that works best for you.

## What Is 3D Motion Sickness

When playing first-person (FPS) or third-person 3D games, or using a VR headset, some players experience dizziness, nausea, cold sweats, or even vomiting. In medicine this is known as **motion sickness**; when triggered by virtual movement on a screen, it is also called **simulator sickness** or **cybersickness**[[1]](https://www.healthline.com/health/cybersickness).

In short, 3D motion sickness is the same reaction as carsickness, seasickness, or airsickness—the trigger is just different.

## Why Motion Sickness Happens: Sensory Conflict Theory

The most widely accepted explanation is the **sensory conflict theory**[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044)[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/).

The human sense of balance relies on the cooperation of three systems:

- **Visual system** (eyes) — tells me “where am I going and how fast”
- **Vestibular system** (inner ear) — tells me “how is my head moving, and whether I am accelerating or rotating”
- **Proprioception** (muscles, joints) — tells me “what posture my body is in”

In the real world, signals from these three systems almost always agree. But a contradiction appears when playing 3D games:

> **The eyes see** the character running, turning, and falling on screen;
> **while the inner ear and body feel** perfectly still, sitting in a chair.

The brain receives conflicting information and cannot determine “am I actually moving?”, so it triggers an ancient defense response—**nausea and vomiting**. Researchers speculate that the brain interprets this sensory conflict as a sign of “poisoning causing mixed-up neural signals” and induces vomiting to protect itself[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044).

This is also why some people are more prone to 3D motion sickness: individuals differ in sensitivity to sensory conflict. Women, migraine sufferers, and people with inner-ear problems are usually more susceptible[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/).

## Why Visual Anchors Work

A key idea for relieving sensory conflict is: **give the brain a stable, stationary reference object** so it can confirm “I am not actually moving”.

This is Peregrine's core principle—overlaying a **fixed visual anchor** on the screen (crosshair, ring, corner markers, and so on). When the game image shakes violently, the anchor stays perfectly still; the eyes treat it as the “ground” or “horizon”, reducing the sensory conflict.

This method is supported by academic evidence. A study by Seok et al. published in *JMIR Serious Games* found that enabling a crosshair (visual guide) in FPS games **significantly reduced** VR sickness symptoms, especially when using a gamepad; the best effect was achieved with a crosshair size of about 30% of the screen area, placed in the head-tracking direction[[4]](https://games.jmir.org/2021/3/e18020/).

Peregrine turns this principle into a general-purpose tool: beyond the crosshair, you can choose an edge rectangle, center ring, corner markers, border frame, edge arrows, or even a custom image to find the anchor shape that suits you best.

## Ways to Relieve 3D Motion Sickness

Below are several proven or commonly recommended mitigation methods; using them in combination is advised. Peregrine corresponds to the “visual anchor” item.

### 1. Use a Visual Anchor (Peregrine's Core Feature)

Place a semi-transparent, fixed reference object at the center or edge of the screen. This is the main method Peregrine provides and also the most convenient software-level mitigation currently available[[4]](https://games.jmir.org/2021/3/e18020/).

**Peregrine usage tips:**

- Try `Cross` (crosshair) or `Ring` (center ring) first; these work best for most people.
- Keep the anchor small and dim, semi-transparent, so it does not block the game view.
- If a center crosshair interferes with aiming, try `EdgeRect` (edge rectangle) or `BorderFrame` (border frame) to place the anchor at the edges.

### 2. Adjust Field of View (FOV)

Many first-person games default to a narrow FOV (60°–70°), which makes the brain feel “things are very close to me” and worsens motion sickness. **Increasing FOV** (usually recommended to 90°–110°) gives the brain a more natural perspective and reduces discomfort[[5]](https://www.cise.ufl.edu/~eragan/papers/Benda_TVCG2023.pdf).

> Note: research in VR reaches the opposite conclusion—a very wide FOV can **worsen** motion sickness, and developers often dynamically narrow FOV during fast movement to relieve it[[6]](https://www.researchgate.net/publication/354730991_Tunnel_Vision_-_Dynamic_Peripheral_Vision_Blocking_Glasses_for_Reducing_Motion_Sickness_Symptoms). For ordinary desktop monitors, simply increasing FOV is usually fine.

### 3. Increase Frame Rate and Reduce Latency

Low frame rates and screen tearing make sensory conflict worse. Aim for:

- Stable game frame rate **above 60 FPS** (high-refresh-rate monitors are even better).
- Turn off or reduce Motion Blur, Camera Shake, Chromatic Aberration, and similar effects.
- Reduce input latency; disable V-Sync if the frame rate is high enough[[7]](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf).

### 4. Adjust Posture and Viewing Distance

- **Sit farther away**: the closer you are to the screen, the more of your field of view the game occupies, and the easier it is to feel sick. Increasing distance lets your peripheral vision see the stationary environment around you (desk, walls), providing extra stable references[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/).
- **Keep a light on**: do not play in a completely dark room; ambient light helps the brain perceive real space.
- **Use a smaller screen**: some recommendations suggest that a smaller screen can reduce sensory conflict[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/).

### 5. Take Breaks and Adapt Gradually

- Pause every **30–45 minutes**, look into the distance or close your eyes for a few minutes.
- Motion sickness is cumulative; **stop as soon as you feel unwell**—pushing through only makes it worse.
- The human body can **habituate** to sensory conflict; repeated short exposure within a short period gradually reduces symptoms. Start with 10 minutes per session and extend day by day.

### 6. Other Physical and Dietary Methods

These methods also apply to carsickness and are recommended by authoritative health institutions such as the NHS and UC Davis Health[[9]](https://www.nhs.uk/conditions/motion-sickness/)[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05):

- **Ginger**: clinical evidence shows ginger can reduce nausea caused by motion sickness; drinking ginger tea or chewing ginger slices before playing may help[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05).
- **Eat light**: avoid greasy, spicy, or heavy meals before playing, and do not play on an empty stomach.
- **Ventilation and cooling**: stuffy heat worsens nausea; open a window or use air conditioning to stay cool.
- **Avoid looking down at your phone or reading while playing**: additional visual motion conflict makes things worse[[9]](https://www.nhs.uk/conditions/motion-sickness/).

### 7. Medication (Severe Cases)

If the methods above are not enough, over-the-counter motion-sickness medicine (antihistamines such as dimenhydrinate/dramamine) can be considered. However, these medications cause drowsiness and effects vary from person to person; **consult a doctor** before using them, and do not rely on medication to keep playing long term[[9]](https://www.nhs.uk/conditions/motion-sickness/).

## Summary

| Method | Principle | Tool Required |
|------|------|-------------|
| Visual anchor | Provides a stationary reference to reduce sensory conflict | ✅ Peregrine |
| Increase FOV | Makes the view closer to natural vision | In-game settings |
| Increase frame rate | Reduces screen tearing and latency | In-game settings / hardware |
| Sit farther away / keep a light on | Uses peripheral vision to provide a stable environmental reference | None |
| Take breaks and adapt gradually | Leverages the body's habituation ability | None |
| Ginger, light meals, ventilation | Reduces nausea response | None |
| Motion-sickness medicine | Suppresses vestibular response | Consult a doctor |

**Recommended combination**: Peregrine visual anchor + increased FOV + stable 60 FPS + timely breaks covers most mild to moderate 3D motion-sickness scenarios. If symptoms are severe or persistent, consult a doctor to rule out vestibular system problems.

## References

- [[1] Healthline — Cybersickness: What It Is, Symptoms, Causes, and Treatments](https://www.healthline.com/health/cybersickness)
- [[2] Reason, J. T., & Brand, J. J. — Motion sickness: a synthesis and evaluation of the sensory conflict theory (Canadian Science Publishing)](https://cdnsciencepub.com/doi/10.1139/y90-044)
- [[3] Karmali, F. et al. — Validating sensory conflict theory and mitigating motion sickness with galvanic vestibular stimulation (NIH PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/)
- [[4] Seok, K. H. et al. — Using Visual Guides to Reduce Virtual Reality Sickness in FPS Games (JMIR Serious Games)](https://games.jmir.org/2021/3/e18020/)
- [[5] Benda, P. et al. — Seated vs Standing VR: Reducing Simulator Sickness (IEEE TVCG 2023)](https://www.cise.ufl.edu/~eragan/papers/Benda_TVCG2023.pdf)
- [[6] Tunnel Vision — Dynamic Peripheral Vision Blocking Glasses for Reducing Motion Sickness Symptoms](https://www.researchgate.net/publication/354730991_Tunnel_Vision_-_Dynamic_Peripheral_Vision_Blocking_Glasses_for_Reducing_Motion_Sickness_Symptoms)
- [[7] Farmani, Y. — Mitigating Cybersickness in Virtual Reality: Challenges and Solutions (Carleton University Thesis)](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf)
- [[8] Hyperoptic — The top 10 tips for avoiding simulation sickness when gaming](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/)
- [[9] NHS — Motion sickness](https://www.nhs.uk/conditions/motion-sickness/)
- [[10] UC Davis Health — Motion sickness: How you can prevent symptoms and enjoy travel](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05)
