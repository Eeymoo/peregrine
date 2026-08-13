---
name: aukcraft-site-design
description: Reproduce the aukcraft.org page shell and design system (dark editorial style, interactive dot-field dynamic background, Flight Line motion language, bilingual routes) on a new Astro + Tailwind static site. Use whenever creating a new aukcraft project site or sub-site (e.g. peregrine.aukcraft.org style pages), when the user asks to copy/reuse the aukcraft website's overall look, dynamic background, header, footer, or motion style, or when a new page must visually match aukcraft.org. Provides drop-in asset files plus the hard design rules that keep the result consistent.
---

# aukcraft Site Design

The complete, production-verified "shell" of aukcraft.org, packaged as drop-in files in `assets/`. Applying this skill gives a new site the same tokens, dynamic background, header, footer, and interaction primitives as aukcraft.org — the visible format stays identical even though the content differs.

Content sections (Philosophy, Projects, Workflow, etc.) are intentionally NOT included: they are site-specific copy. This skill covers the reusable chassis + the rules that keep new sections consistent with it.

## Hard design rules (brand design system v2)

These rules are why the site feels coherent. Enforce all of them in anything you add:

- **No shadows, ever.** Layering is done via surface brightness (`base #0B0E11` below `raised #14181D`) plus 1px hairlines (`rgba(255,255,255,0.08)`).
- **Corner radius ≤ 4px** everywhere (Tailwind default radius is set to 4px; Flight Line rings use `rx: 3`).
- **Auk Teal `#14B8A6` is rationed**: links, CTAs, Flight Line, focus rings, cursor-fills in canvases. Never decoration, never backgrounds.
- **Single dark theme.** No light sections, no pure `#000` or `#fff`.
- **Motion honors `prefers-reduced-motion`**: traces/reveals downgrade to instant color change or static. No auto-playing or looping animation (the one exception is HeroCanvas's slow idle wave, which is the brand signature).
- **Serif-italic accents are English-keywords-only** (Newsreader italic, e.g. "Craft the *architecture*."). Chinese accents use `.serif-zh` (Noto Serif SC). Never sprinkle serif randomly.
- **CTA hierarchy**: a `.flight` button with `text-teal` is the primary action; `text-ink` is secondary. One flight-level CTA per intent per page; repeats of the same destination downgrade to `.link-line`.
- **Layout families must not repeat** across sections of one page (card grid, hairline row list, terminal block, prose + links are distinct families). Audit before adding a section.
- **Section rhythm**: `mx-auto max-w-5xl px-6`, section padding `py-40 md:py-56`. Every section opens with `SectionHeading` (`NN ─ LABEL` micro-mono + hairline rule).
- **Emoji extremely restrained**; open-source voice only, no company-ized language.

## File map (`assets/`)

| Asset | Copy to (target project) | Provides |
|---|---|---|
| `global.css` | `src/styles/global.css` | All design tokens (colors, `--ease-lock`, 4 motion durations, z-index scale) + primitives: `.flight` / `.flight-line` (Flight Line), `.link-line`, `[data-reveal]` + stagger, `.glass` / `.glass-deep`, `.noise`, `.skip-link`, `.micro`, `.serif-zh`, `.puffin-mark` colors, scroll-snap, focus-visible, reduced-motion fallbacks |
| `tailwind.config.mjs` | project root | Token mapping into Tailwind: 6 colors, font stacks, radius 4px, `max-w-prose 65ch`, `ease-lock` |
| `Layout.astro` | `src/layouts/Layout.astro` | Page shell: font loading, SEO/canonical/hreflang/og driven by a `lang` prop, `skip-link`, `<DotField />`, noise overlay, scroll-reveal IntersectionObserver, `html.js` gating |
| `components/DotField.astro` | `src/components/` | Site-wide dynamic background: fixed canvas dot grid (30px gap, hollow rings at 5% ink); dots near the cursor fill teal with eased distance falloff; sleeps when the trail settles; static under reduced-motion |
| `components/HeroCanvas.astro` | `src/components/` | Hero backdrop: a wordmark (default `AUKCRAFT`) sampled into a dot matrix; hollow rings, teal fill near cursor, slow idle wave sweep; re-samples on font load and resize |
| `components/FlightLine.astro` | `src/components/` | SVG perimeter-trace element; drop inside any `.flight` element, optional `duration` prop |
| `components/SectionHeading.astro` | `src/components/` | `01 ─ LABEL` section opener with hairline rule |
| `components/Hero.astro` | `src/components/` | Reference implementation of the header (wordmark + language switch + GitHub link, single line) and hero (serif-accent headline, ≤20-word body, dual CTA with correct hierarchy) |
| `components/Footer.astro` | `src/components/` | Footer: wordmark, section anchor nav, contact emails with role notes, hairline-separated copyright row |
| `components/PuffinMark.astro` | `src/components/` | aukcraft puffin mascot (inline SVG, dark-theme `.pm-*` classes). Keep for aukcraft-family sites; replace for anything else |

## Applying to a new project

1. Scaffold Astro + Tailwind v3 (`@astrojs/tailwind`), `output: 'static'`.
2. Install fonts: `@fontsource-variable/inter`, `@fontsource-variable/newsreader`, `@fontsource/jetbrains-mono`. (Noto Sans SC / Noto Serif SC load via Google Fonts `<link>` in Layout — keep for CJK support.)
3. Copy every file per the file map above.
4. Build pages as `Layout` wrapping `<main id="main">` with one component per section; pass `lang` (`'en' | 'zh'`) down. Bilingual = separate routes (`/` and `/zh/`), never inline mixing.
5. Compose sections with the primitives instead of inventing new ones: `.flight` + `<FlightLine />` for buttons, `.link-line` for inline links, `data-reveal` on blocks that should fade-rise in, `glass` / `glass-deep` surfaces over the DotField, `.micro` for small uppercase mono labels.
6. Customize the three identity points only: the wordmark string in `HeroCanvas.astro` (`const text = 'AUKCRAFT'`), title/description/canonical URLs in `Layout.astro`, and the mascot in `PuffinMark.astro` if the site is not aukcraft-branded.
7. Key technical terms (crate, CI, PR, SDD, TDD) stay in English even in Chinese copy.

## Pre-ship checklist

- [ ] Teal audit: only links / CTAs / motion lines / focus / canvas cursor-fills
- [ ] Radius audit: nothing above 4px; zero shadows
- [ ] Single dark theme; no pure black/white
- [ ] Reduced-motion: traces, reveals, canvases all degrade to static/color
- [ ] Every section opens with SectionHeading; rhythm `max-w-5xl px-6 py-40 md:py-56`
- [ ] No repeated layout family across sections; CTA hierarchy (teal primary / ink secondary) holds
- [ ] Header renders on one line; footer matches the reference structure
- [ ] `npm run build` passes; bilingual routes both render if applicable
