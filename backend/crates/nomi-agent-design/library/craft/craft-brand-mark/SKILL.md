---
name: craft-brand-mark
description: "How to design a simple logo mark and a brand board when the person has none: brand strategy first, five ways to find a mark, drawing it as paths, and a nine-panel board on the canvas. Read it when the brief needs a logo, a mark or a brand kit."
license: "MIT, Copyright (c) 2026 Leonxlnx (Taste Skill family: brandkit), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (brandkit, as vendored in open-design@5fdcc3d, 2026)"
---
# Brand mark and brand board

How you make a small, ownable logo mark and a one-page brand board for a business that has none, drawn on the canvas as paths and frames, and grounded in what the business actually does.

When the person has a logo, use it: you never redraw, "improve" or replace it unasked, and you never draw a copy of a real brand's logo (Taste). This skill is for when there is no mark, or the person asks for one.

## 1. Strategy before shapes

Write five lines first:

| Line | Example (Kopi Lereng, a roaster on the slopes of Merapi) |
|---|---|
| What it sells and to whom | Single-origin beans to home brewers and cafes |
| The promise | Coffee from one slope, roasted the week you buy it |
| Personality (three words) | Patient, outdoorsy, exact |
| Core metaphor | The slope: contour lines, a path uphill |
| Avoid | Coffee bean icon, steaming cup, generic mountain triangle |

The mark must come from the metaphor, not from the category's clip-art (a cup for coffee, scissors for a barber, a wrench for a bengkel).

## 2. Five ways to find the mark (use one, combine two at most)

1. **Initial plus meaning:** the first letter cut, folded or built from the metaphor. A "K" whose diagonal is a contour line.
2. **The main action:** what the business does, abstracted. Laundry: a fold; tailor: a stitch line; tutor: a step up.
3. **Two ideas fused:** slope plus cup rim; a house plus a key for a kos.
4. **Negative space:** a hidden arrow, a cut-out initial, a path through the shape. Keep the cut crisp at 16px.
5. **Construction geometry:** circles, a grid, diagonal cuts, modular blocks; show the construction on one panel.

Draw three candidates small first (32px), pick the one that still reads at 16px, then refine.

## 3. Draw it on the canvas

- The mark is one or a few `path` nodes with solid fills, inside a frame named `Logo / Mark`; the wordmark is live text in the brand face beside it, in `Logo / Lockup`. Make both reusable.
- Build on a grid: a 24-unit square, geometry snapped to whole units; one stroke weight if it is a line mark.
- Variants as components: `Logo / Lockup / Horizontal`, `Logo / Lockup / Stacked`, `Logo / Mark / On dark`, `Logo / Mark / Favicon` (the mark alone in a 32 square, simplified if needed).
- Test it: at 16, 32 and 64; in one colour; on the ground and on the primary colour; beside the WhatsApp and Instagram icons where it will live as an avatar (a circle crop).

Mark quality bar: simple, memorable, tied to the metaphor, balanced, usable as icon, avatar, stamp and pattern. Avoid generic lightning bolts, random animals, fake luxury crests, sparkles, clip-art, and any shape close to a famous mark.

## 4. The brand board (a design-system frame)

One frame, 1440 wide, a 3 x 3 grid of panels with 24 gutters on a calm ground. Each panel has one job:

| # | Panel | What is in it |
|---|---|---|
| 1 | Cover | The lockup large, lots of space |
| 2 | Construction | The mark on its grid with the guide circles or cuts |
| 3 | Digital use | The mark as an app icon, a browser tab, a WhatsApp avatar crop |
| 4 | Essence | One short line in the display face |
| 5 | Colour | Swatches with names and hex: ground, ink, primary, one accent |
| 6 | Type | The pairing from `taste-type`: a large specimen and the text face |
| 7 | Physical use | Label, cup sleeve, tote, receipt, shop sign: drawn flat, front-on |
| 8 | Photo direction | One or two photo placeholders with notes on light and subject |
| 9 | Detail | Buttons, a tag, an icon row in the brand's components |

Give the panels rhythm: quiet (1, 4), functional (3, 9), emotional (8), technical (2, 5, 6). Not every panel equally loud. A 2 x 3 board (cover, digital, essence, colour, type, photo) is enough for a small business.

## 5. Colour and type discipline

- One dominant palette: ground, ink, one primary, at most one accent; the accent repeats across panels. Deal it from `taste-color` by the trade; the client's existing colours (a painted shopfront, their uniforms) win.
- Type from `taste-type`; the wordmark may be the display face, set tight, never stretched or outlined.

## 6. Taglines

Short, specific, in the person's language, true:
- Good: "Satu lereng, satu sangrai." "Cuci hari ini, rapi besok pagi." "Servis jujur, harga di depan."
- Bad: "Solusi kopi terbaik untuk Anda", "Elevate your coffee experience", anything with an exclamation mark.
Taste's device budget still applies: a two-fragment tagline counts as the page's one "X. Y." device.

## 7. References the person brings

Learn their rhythm, spacing, amount of text, accent logic and how the mark repeats. Never copy the logo, the name, the slogan, the exact composition or a unique asset.

## What the canvas cannot do

The source's boards use halftone, grain, scanlines, glows and photographic mockups. Draw flat: solid fills, a photo placeholder with a note for each photographic panel, applications drawn as simple front-on shapes. Tell Koda which brand files to export (SVG mark, favicon, social avatar) in the handoff.

Adapted from Taste Skill's brandkit (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
