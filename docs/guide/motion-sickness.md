# Relieving 3D Motion Sickness

This page explains what 3D motion sickness is, why it happens, and how to relieve it. The visual anchors provided by Peregrine are just one of many mitigation methods; this chapter also lists other proven methods to help you find the combination that works best for you.

## What Is 3D Motion Sickness

When playing first-person (FPS), third-person 3D games, or using VR devices, some players feel dizzy, nauseous, sweaty, or even vomit. Medically, this is called **motion sickness**; when triggered by virtual movement on a screen, it is also called **simulator sickness** or **cybersickness**[[1]](https://www.healthline.com/health/cybersickness).

In short, 3D motion sickness is the same type of reaction as carsickness, seasickness, and airsickness—only the trigger scenario is different.

## Why Motion Sickness Happens: Sensory Conflict Theory

The most widely accepted explanation is the **sensory conflict theory**[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044)[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/).

The human body maintains balance through the cooperation of three systems:

- **Visual system** (eyes) — tells me "where I am going and how fast"
- **Vestibular system** (inner ear) — tells me "how my head is moving and whether I am accelerating/rotating"
- **Proprioception** (muscles, joints) — tells me "what posture my body is in"

In the real world, signals from these three systems almost always match. But when playing 3D games, a contradiction occurs:

> **The eyes see** the character running, turning, and falling on screen;
> **The inner ear and body feel** firmly seated in a chair, with no movement at all.

The brain receives conflicting information and cannot determine "am I actually moving?", so it triggers an ancient defense response — **nausea and vomiting**. Academics speculate that the brain interprets this sensory conflict as a sign of "poisoning causing nerve signal confusion," and triggers vomiting to save itself[[2]](https://cdnsciencepub.com/doi/10.1139/y90-044).

This is also why some people are more prone to 3D motion sickness: individual sensitivity to sensory conflict varies, and women, migraine sufferers, and people with inner ear problems are usually more susceptible[[3]](https://pmc.ncbi.nlm.nih.gov/articles/PMC12034809/).

## Why Visual Anchors Work

A key idea for relieving sensory conflict is: **provide the brain with a stable, stationary reference** so it can confirm "I am actually not moving."

This is Peregrine's core principle — overlaying a **fixed visual anchor** on the screen (crosshair, center ring, edge markers, etc.). When the game screen shakes violently, this anchor stays completely still; the eyes treat it as "ground" or "horizon," reducing the sensory conflict.

This method is backed by academic evidence. Seok et al. from Korea published a study in *JMIR Serious Games* finding that enabling visual anchors (visual guides) in FPS games can **significantly reduce** VR sickness symptoms, especially when using a controller; anchors sized at about 30% of the screen and placed in the head-tracking direction worked best[[4]](https://games.jmir.org/2021/3/e18020/).

Peregrine turns this principle into a general tool: beyond crosshairs, you can choose edge rectangles, center rings, edge markers, borders, edge arrows, or even custom images to find the anchor form that suits you best.

## Methods to Relieve 3D Motion Sickness

Below are multiple validated or commonly recommended methods; using them in combination is suggested. Peregrine corresponds to the "visual anchor" item.

### 1. Use Visual Anchors (Peregrine's Core Feature)

Place a semi-transparent, fixed reference at the center or edge of the screen. This is the main method provided by Peregrine and currently the most convenient software-level mitigation[[4]](https://games.jmir.org/2021/3/e18020/).

**Peregrine usage suggestions:**

- Try `cross` (crosshair) or `ring` (center ring) first; these work best for most people.
- The anchor should not be too large or too bright; semi-transparent is enough to avoid blocking the game screen.
- If a center crosshair interferes with aiming, try `edge_rect` (edge rectangle) or `border_frame` (border frame) to place the anchor at the edges.

### 2. Adjust Field of View (FOV)

Many first-person games default to a narrow FOV (60°–70°), which makes the brain feel "everything is very close," worsening motion sickness. Increasing FOV (usually recommended to 90°–110°) gives the brain a more natural perspective and reduces discomfort[[5]](https://www.cise.ufl.edu/~eragan/papers/Benda_TVCG2023.pdf).

> Note: VR research shows the opposite conclusion — an overly wide FOV can **worsen** motion sickness, so developers often dynamically narrow the FOV during fast movement to relieve it[[6]](https://www.researchgate.net/publication/354730991_Tunnel_Vision_-_Dynamic_Peripheral_Vision_Blocking_Glasses_for_Reducing_Motion_Sickness_Symptoms). For ordinary desktop monitor games, increasing FOV is generally fine.

### 3. Increase Frame Rate and Reduce Latency

Low frame rates and screen tearing worsen sensory conflict. Try to:

- Keep the game frame rate stable at **60 FPS or higher** (a high-refresh-rate monitor is even better).
- Turn off or reduce motion blur, camera shake, chromatic aberration, and similar effects.
- Reduce input latency; turn off V-Sync if the frame rate is high enough[[7]](https://www.csit.carleton.ca/~rteather/pdfs/theses/thesis-yasinfarmani-final.pdf).

### 4. Adjust Posture and Viewing Distance

- **Sit farther away**: the closer you are to the screen, the more of your field of view the game occupies, and the easier it is to feel sick. Increasing the distance lets your peripheral vision see the stationary surroundings (desk, walls), providing extra stable references[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/).
- **Keep a light on**: do not play in complete darkness; ambient light helps the brain perceive real space.
- **Use a smaller screen**: some suggestions indicate that a smaller screen can reduce sensory conflict[[8]](https://www.hyperoptic.com/broadband/explained/lifestyle-and-entertainment/the-top-10-tips-for-avoiding-simulation-sickness-when-gaming/).

### 5. Take Breaks and Adapt Gradually

- Pause every **30–45 minutes**, look into the distance, or close your eyes for a few minutes.
- Motion sickness is **cumulative**; **stop immediately once you feel unwell** — pushing through will only make it worse.
- The human body has the ability to **habituate** to sensory conflict; repeated, short exposures will gradually reduce symptoms. Start with 10 minutes at a time and extend day by day.

### 6. Other Physical and Dietary Methods

These methods also apply to carsickness and come from authoritative health institutions such as the NHS and UC Davis Health[[9]](https://www.nhs.uk/conditions/motion-sickness/)[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05):

- **Ginger**: clinical evidence shows ginger can reduce nausea caused by motion sickness; drinking ginger tea or eating ginger slices before playing can help[[10]](https://health.ucdavis.edu/blog/cultivating-health/motion-sickness-how-you-can-prevent-symptoms-and-enjoy-travel/2024/05).
- **Light meals**: avoid greasy, spicy, or overly heavy food before playing, and do not play on an empty stomach.
- **Ventilation and cooling**: stuffiness worsens nausea; open a window or turn on air conditioning to stay cool.
- **Avoid looking down at your phone or reading while playing**: extra visual motion conflict will make things worse[[9]](https://www.nhs.uk/conditions/motion-sickness/).

### 7. Medication (Severe Cases)

If the above methods are not enough, consider over-the-counter motion sickness medicine (antihistamines such as dimenhydrinate/Dramamine). However, medication can cause drowsiness, and effects vary from person to person. It is recommended to **consult a doctor** before use and not to rely on medication to play through symptoms long-term[[9]](https://www.nhs.uk/conditions/motion-sickness/).

## Summary

| Method | Principle | Requires Tool |
|--------|-----------|---------------|
| Visual anchor | Provides a stationary reference to reduce sensory conflict | ✅ Peregrine |
| Increase FOV | Makes the picture closer to a natural perspective | In-game settings |
| Increase frame rate | Reduces screen tearing and latency | In-game settings / hardware |
| Sit farther away / keep lights on | Uses peripheral vision to provide a stable environmental reference | None |
| Take breaks and adapt gradually | Uses the body's ability to adapt | None |
| Ginger, light meals, ventilation | Reduces nausea response | None |
| Motion sickness medicine | Suppresses vestibular response | Consult a doctor |

**Recommended combination**: Peregrine visual anchor + increased FOV + stable 60 FPS + timely breaks can cover most mild to moderate 3D motion sickness scenarios. If symptoms are severe or persist, consult a doctor to rule out vestibular system problems.

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
