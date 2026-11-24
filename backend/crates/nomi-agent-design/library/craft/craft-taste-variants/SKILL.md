---
name: craft-taste-variants
description: "What the older and sibling Taste skills (Taste v1, GPT-Taste, Full-Output, Stitch) add on top of Taste 2.5.1: density dials for app screens, state coverage, hero and grid checks, complete hand-offs, and writing the system in words. Read it when designing app screens or a handoff."
license: "MIT, Copyright (c) 2026 Leonxlnx (taste-skill-v1, gpt-tasteskill, output-skill, stitch-skill), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (variants as vendored in open-design@5fdcc3d, 2026)"
---
# Taste: what the variants add

The parts of Taste v1, GPT-Taste, Full-Output Enforcement and Stitch Design Taste that 2.5.1 does not already say, kept where they help you on the canvas and marked where 2.5.1 overrules them.

You already work from Taste 2.5.1 (`taste-core` and the other `taste-*` skills). Most of the variants' rules are earlier drafts of the same ideas; this file keeps only the additions. Where a variant contradicts 2.5.1, 2.5.1 wins, and the list at the end says what not to take.

## 1. Three dials for app screens (from Taste v1 and Stitch)

Taste 2.5.1 sets the look of a company site by its kind. Inside an app (a dashboard, a POS, an admin, a booking back-office) you also need a density decision. Set three dials on plan line 2 for every app screen set, 1 to 10:

| Dial | 1-3 | 4-7 | 8-10 |
|---|---|---|---|
| **Density** | Gallery: large gaps, few items | Daily app: normal spacing | Cockpit: tight rows, no card boxes, 1px dividers only |
| **Variance** | Predictable symmetric grid | Offset: mixed ratios, left headers over centred data | Asymmetric: fractional columns, large empty zones |
| **Motion** | Hover and press only | Fluid state transitions, short staggers | Choreographed transitions (rare in an app) |

Defaults for Nomi's apps: density 6, variance 3, motion 3. A kasir screen or stock table is density 8; an onboarding is density 3.

Rules that follow from the dials:
- **Density 8 and up:** no cards around metrics; group with top hairlines or space; every number in tabular figures (a mono face only if the brand already uses one, per `taste-type`); rows 40-44 tall, still 44 tap targets on phones.
- **Variance 4 and up:** no centred page header over a centred grid; put the title left and let the data take the width.
- **Motion:** never above 3 for anything used many times a day (see `craft-motion-design`).

## 2. Every state, drawn (from Taste v1 and Stitch)

Draw the full cycle of every screen that loads or accepts input, beside the main frame, with the same element ids:
- **Loading:** skeleton blocks at the real sizes of the content they stand in for, in `$color-muted`, radius from the ladder. No centred spinner on a page.
- **Empty:** a composed state that says what will appear and the one action that fills it: "Belum ada pesanan hari ini" with "Bagikan link toko".
- **Error:** inline, next to the cause, in `$color-destructive` with the fix: "Nomor WhatsApp harus diawali 08 atau +62".
- **Pressed:** the press state of the primary button (1px down, scale .98) as a variant in the design system.
- **Forms:** label above the field, helper text under it when needed, error text under it in place of the helper, 8 between label and field.

## 3. Checks before you call a hero done (from GPT-Taste)

- **The h1 fits in two lines at 1440 and three at 390.** If it wraps more, widen its text frame or cut words; never shrink below the hero size Taste sets.
- **Nothing in the hero but the h1, one line, the action pair and the picture:** no floating stamp, no badge on the headline, no stats strip inside the hero.
- **Button labels read at a glance:** dark fill with light label, light fill with dark label, at least 4.5:1.
- **No meta labels:** never "Section 01", "Bagian 03", "Pertanyaan 05", "Tentang kami" as an eyebrow. A heading with a noun does the job.

## 4. Grids with no holes (from GPT-Taste and Taste v1)

- A bento or card grid has no empty cell and no missing corner: write the column spans of each row and check each row sums to 12 before drawing.
- Three to five strong cards beat eight weak ones.
- Pin each card's button to its bottom edge so buttons in a row line up; start feature lists at the same height in every pricing column.

## 5. Hand off complete work (from Full-Output Enforcement)

The source bans truncated code; on the canvas the same failure is a half-finished design. Before saying a version is ready:
- **Count the deliverables** the person asked for (screens, widths, states, components) and check each exists on the canvas: every section at 1440, 768 and 390; every state from section 2; every component used in a screen present in the design system frame.
- **No stand-ins:** no "lorem", "contoh", "[nama produk]", "dan seterusnya", or one card drawn with a note "the rest look the same". Draw them all with real content.
- **If the preview would pass a limit** (300 elements, file size in `pen-format`), split it into more `.pen` files or more rounds, and tell the person plainly which screens come in the next save. Never compress the remaining screens to fit.

## 6. Write the system in words (from Stitch)

Stitch writes a `DESIGN.md` so another agent can rebuild the look. Your handoff to Koda (`components-handoff`) carries the same thing, in this order:

1. **Atmosphere:** one or two sentences with the three dials: "Tenang dan rapi seperti meja resepsionis klinik; density 4, variance 3, motion 2."
2. **Colours:** each as descriptive name, hex and job: "Tinta arang (#1F1E1C), teks utama dan tombol utama."
3. **Type:** each face with its job and weights; the banned faces.
4. **Components:** shape, fill, shadow and interaction in words: "Tombol: tinggi 44, sudut 6, ditekan turun 1px."
5. **Layout:** grid, edges, section rhythm, how it stacks on phones.
6. **Motion:** the notes from `craft-motion-notes`.
7. **Never:** the bans for this design, so the next round doesn't drift.

Use plain words with the value in brackets ("sudut agak bulat (8)"), name colours by purpose, and keep the same term for the same thing throughout.

## Not taken (Taste 2.5.1 overrules)

| Variant rule | Why it is dropped |
|---|---|
| Fonts Satoshi, Cabinet Grotesk, Outfit-only stacks; "Plus Jakarta Sans is fine" | Taste's pairing table decides faces; Plus Jakarta Sans is banned |
| Perpetual loops in every card, typewriter search bars, auto-sorting lists | Card loops only on software and tech, annotated, one group per page |
| Magnetic buttons, gooey menus, dock magnification, custom cursors | Not in Taste's hover layer |
| Pinned scroll galleries, image scale-and-fade on scroll, card stacking, scrubbed word reveals on every section | Taste allows one or two dealt moments, never scroll animation beyond them |
| Grainy mesh gradients, radial blurs, CSS filters on photos | Flat grounds; photos unfiltered |
| Overlap and negative-margin layering, tilted cards, masonry | Nothing staggered, tilted or overlapping |
| Inline photos inside headline words | Taste: every word live text, pictures in their own frames |
| Carousels and marquees of partner logos | No carousels; no invented logos |
| A seeded "random" pick of hero and components | Taste deals from lists by kind and from what you used last |
| Picsum or Unsplash placeholder images | Photo placeholders with a note |

Adapted from Taste Skill v1, GPT-Taste, Full-Output Enforcement and Stitch Design Taste (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
