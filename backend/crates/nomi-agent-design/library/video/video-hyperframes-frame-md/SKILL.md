---
name: video-hyperframes-frame-md
description: "How to translate a brand's design tokens into frame.md, the design.md for the camera: what stays strict, how to rescale type, borders and spacing for a frame, safe areas for TVs and phones, and type sizes per format. Read it before the first scene of any branded video."
license: "Apache-2.0, Copyright 2026 HeyGen, Inc."
source: "https://github.com/heygen-com/hyperframes @574b5e3d3bbf526809d268a6728f9c5ce52e18bb"
---
# frame.md: the brand, rewritten for the camera

A design system is written for a web page: 16 px body text, 1 px borders, soft 6% shadows, a page that scrolls. None of that survives a 1080×1920 frame seen once on a phone. `frame.md` is the translation layer. It keeps the same tokens and rules, rewritten so you can compose a video without guessing at scale or reaching for web chrome. Its principle is **atoms are sacred, composition is free, numbers come from the script**.

Write `frame.md` at the root of the video version, next to `index.html`, before the first scene. When the brand already has one, read it first. The frontmatter is normative, and the prose states intent.

## What stays and what changes

| Strict: copy exactly | Free: rescale for the frame |
|---|---|
| Hex values of every colour token | Type sizes (to frame-relative units) |
| Font families and the weight relationships | Spacing and padding |
| The Do's and Don'ts (never white on the accent, no shadows…) | Border weight: a 1 px web border vanishes; use 3–5 px at 1080 |
| Signature components and their single piece of chrome | Shadow strength: a 6% web shadow vanishes; use a clear contact shadow or none |
| The accent's ration (where it may appear) | Radii (scale with the element) |

Never round or invent a hex value. Where a token is missing, take it from the client's real material (logo, shopfront paint, packaging) and say so.

## From Nomi tokens to frame.md

| Nomi token | frame.md frontmatter | In the composition |
|---|---|---|
| `$color-background`, `$color-foreground`, `$color-primary`, `$color-primary-foreground`, `$color-muted-foreground`, `$color-card`, `$color-border` | `colors: { background: "#F7F4EE", … }` | A `color` variable per token, as `var(--color-primary)` |
| `$font-heading`, `$font-body` | `typography:` entries with `fontFamily`, a size in `cqw`, `weight`, `lineHeight`, `tracking` | A `string` variable, plus `@font-face` if the family isn't bundled |
| `$space-1` … `$space-12` | `spacing:` in `cqw` (`pad-x`, `pad-y`, `gap-grid`, `card-pad`) | Constants in `:root`, or variables |
| `$radius-sm` … `$radius-2xl` | per component, in `cqw` | Constants |

## Frame-relative units: the container law

Size the frame ground with `container-type: size`, and author display sizes in `cqw`/`cqh` resolved against it, **never `vw`**. A `vw` size inflates whenever the preview isn't full-screen; `cqw` resolves against the frame at any size, which keeps the Studio preview and the render the same.

- Convert with px ÷ frame width × 100 = cqw. At 1920 wide, 120 px is 6.25cqw. At 1080 wide, 120 px is 11.1cqw.
- Re-step sizes per aspect ratio, because a `cqw` that works at 16:9 is too small at 9:16.

## Type sizes

| Role | 16:9 (1920×1080) | 9:16 (1080×1920) | 1:1 (1080×1080) |
|---|---|---|---|
| Hero words (1–3 words) | 6.5–9cqw (125–175 px) | 11–15cqw (120–160 px) | 10–13cqw |
| Title (4–6 words) | 4–9% of frame height (45–95 px) | about ×1.35 the landscape size (60–130 px) | 6–9cqw |
| Body or support line | 2.5–4% of height (28–42 px) | 36–48 px | 32–44 px |
| Floor for any word meant to be read | 28 px (2.6% of the short side) | 28 px, and use 36+ for social | 28 px |
| Captions | base 78 px, min 42 | base 78 px, min 42, maxWidth 900 | base 70 px |
| Price on a menu TV | 5–7% of height, tabular figures | | |

- Size the headline to its line length: ≤3 words gets hero size, 4–6 words title size, 7+ words a smaller step. Cap a text block at about 78% of the width.
- When words don't fit, cut words. Never go below the floor. A 9:16 claim holds about 5 English or 4–5 Indonesian words a line.
- A deck slide's reading text is at least 40 px at 1920×1080.
- Use `font-variant-numeric: tabular-nums` for prices and counts.

## Safe areas

| Where it plays | Keep clear |
|---|---|
| Any video | Content inside the **90% action-safe** box (5% margin each side). Words inside the **80% title-safe** box (10% each side). |
| TV in a cafe or shop (16:9) | Title-safe is strict: TVs overscan and crop edges. Prices and names go no closer than 192 px to the sides and 108 px to the top and bottom. |
| Reels, TikTok, Shorts (1080×1920) | Words and the product stay from **288 to 1248 px down** and **120 to 888 px across**. Below 840 px down, keep the right edge at 780 px, where TikTok's action buttons sit. Outside this band, put only texture or background. |
| Instagram Reels ads | The bottom 35% is covered by the caption and the buttons |
| Feed 1:1 or 4:5 | 6% margins. The profile UI sits outside the frame. |

Burned-in captions sit at the bottom of the clear band (bottom edge at y 1248 on 9:16).

## What a frame.md contains

The frontmatter (YAML) holds:

```yaml
version: alpha
name: Toko Roti Senja — Frame
unit: the frame — 1080×1920 primary; 1920×1080 and 1:1 documented
principle: atoms are sacred · composition is free · numbers come from the script
colors:   { background: "#F7F4EE", foreground: "#2B2118", primary: "#C2552D", muted-foreground: "#7A6A5C" }
typography:
  hero-title: { fontFamily: "Fraunces", cqw: 13, weight: 700, lineHeight: 0.95, tracking: "-1px" }
  title:      { fontFamily: "Fraunces", cqw: 8,  weight: 700, lineHeight: 1.05 }
  body:       { fontFamily: "Inter",    cqw: 3.6, weight: 500, lineHeight: 1.3 }
  price:      { fontFamily: "Inter",    cqw: 7,  weight: 800, note: "tabular-nums" }
spacing: { pad-x: "8cqw", pad-y: "6cqw", gap-grid: "3cqw" }
components:
  price-tag: { backgroundColor: "{colors.primary}", textColor: "{colors.background}", rounded: "2cqw",
               description: "The one place the accent fills a shape." }
```

The body then has these sections, in order:

1. **Overview:** the look in a paragraph, and its key characteristics at frame scale.
2. **Frame craft bar:** the eye tests every frame must pass. *Squint*: one element dominates at 3–6× its nearest neighbour. *Silence*: frames read 40–55% empty. *Restraint*: the accent appears either as an accent or as one full region, never both at full strength. *Reference*: what it aims at, and what failure looks like.
3. **The frame:** primary size, the other ratios, safe area, and the container law.
4. **Colours:** which surfaces combine, what colour headlines take on each ground, and the only gradient (if any).
5. **Typography:** the reading ramp and the display ramp, the legibility floor, and fit-to-measure.
6. **Depth and surface; shapes:** what carries depth (borders, regions, photographs), and the radius rules.
7. **Components:** each one with its single piece of chrome.
8. **Frame treatments:** one recipe per plate: ground · container · what it composes · focal · chrome · accent · silence · fixed vs free · density. Typical plates are a cover, a feature (photo plus price), a list, a quote or testimonial, and a closing plate.
9. **Composition rules:** Do and Don't.
10. **Aspect-ratio behaviour:** a table of how each treatment recomposes at 16:9, 9:16 and 1:1.
11. **Approved entities:** the real logos and names allowed; everything else is a placeholder.
12. **Numerals and claims:** never invent figures. Use slots (`{harga}`, `— figure —`) until the brief supplies a value.
13. **Pre-render self-audit:** squint, silence, surfaces, type floor, depth, anchor (no three frames in a row share one anchor), fabrication.
14. **Known gaps:** motion is out of scope (it lives in the spec and the timeline), and so are fonts that need files.

## Video facts to bake in

- A full-screen linear gradient on a dark ground bands under H.264. Use a solid colour, or a solid with a local radial glow.
- An accent at 5% opacity disappears in compression. Use 15–25% for atmosphere and full saturation for the focal element.
- Thin coloured text and hairlines blur under 4:2:0 chroma. Keep coloured text heavy, or ask the worker for `--chroma-subsampling 444`.
- Photographs carry the look for places and products (Taste). frame.md sets their ground, crop and the type over them, not filters.

## Presets

HyperFrames ships 13 frame presets. They are deck looks, used when the person names one or for a deck, and never as the fallback for a film without a brand. They are: biennale-yellow, blockframe, blue-professional, bold-poster, broadside, capsule, cartesian, cobalt-grid, code-editorial, coral, creative-mode, daisy-days and editorial-forest. Take only their structure as a model. A Nomi client's own tokens always win.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
