---
name: craft-style-soft
description: "A style card for Soft Premium (warm cream or silver-white grounds, double-bezel cards, pill buttons with a nested arrow, huge whitespace, diffused shadows). Read it when you offer directions to a lifestyle, property, wellness or premium service brand."
license: "MIT, Copyright (c) 2026 Leonxlnx (Taste Skill family: soft-skill), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (high-end-visual-design, as vendored in open-design@5fdcc3d, 2026)"
---
# Style: Soft Premium

An airy, tactile direction that makes a small brand feel expensive: warm cream or silver-white grounds, big calm type, cards built like a glass plate in a tray (an outer shell around an inner core), pill buttons whose arrow sits in its own small circle, and shadows so soft they read as light rather than depth.

## The look

Everything breathes: sections twice as tall in padding as usual, few elements per screen, large headings. Important surfaces are machined: a pale outer shell with a hairline and a large radius holds an inner panel with its own ground and a slightly smaller, concentric radius. Buttons are fully round pills. Shadows are wide and faint. Motion is heavy and slow, like something with mass settling.

## Two moods (deal one by the trade)

| Mood | Grounds | Type | Suits |
|---|---|---|---|
| **Editorial luxury** | Cream #FAF7F1, espresso ink, muted sage accent | Serif display (Fraunces, EB Garamond, Newsreader) with a clean sans | Villas, homestays, boutique hotels, interior studios, florists, bridal, skincare |
| **Soft structural** | Silver-white #F4F5F3 or white, graphite ink, one cool accent | A bold-feeling grotesk at 400 (Bricolage Grotesque, Funnel Display, Hanken Grotesk) | Clinics, wellness, fitness studios, consumer apps, portfolios |

The source's third mood, "ethereal glass" (OLED black with purple and emerald glowing orbs and heavy blur), is not offered: Taste bans violet, glow, glass and dark pages outside tech, and the canvas cannot draw blur.

## Suits / avoid

- **Suits:** the trades above, and a software or app brand selling to consumers (soft structural).
- **Avoid for:** bengkel, hardware stores, street food, dense dashboards, anything urgent or price-led; the generous spacing wastes a busy buyer's time.

## Tokens (editorial luxury; soft structural swaps the values)

| Token | Light | Dark | Role |
|---|---|---|---|
| `$color-background` | #FAF7F1 | #1A1714 | Page |
| `$color-foreground` | #2A221C | #F3EEE6 | Ink |
| `$color-secondary` | #F0EBE2 | #24201B | Outer shell (bezel) |
| `$color-card` / `-foreground` | #FFFFFF / #2A221C | #2D2822 / #F3EEE6 | Inner core |
| `$color-muted-foreground` | #6E6258 | #BFB4A8 | Secondary text |
| `$color-primary` / `-foreground` | #2A221C / #FAF7F1 | #F3EEE6 / #1A1714 | Pill button |
| `$color-accent` / `-foreground` | #E4E9DF / #2F3A2A | #2E3529 / #DCE5D3 | Arrow circle, selected row |
| `$color-border` | #2A221C14 | #FFFFFF1A | Shell hairline |
| `$color-ring` | #5E7352 | #A9BF9C | Focus |

Radius: shell 32, core 26 (shell radius minus shell padding, so the curves stay concentric), photos 24, inputs 16, buttons pill. Taste lets the radius ladder follow the trade (`taste-cards-icons`): this style is the round end of it; never mix it with square elements.

## Components

- **Double-bezel card:** outer frame `$color-secondary`, 1px `$color-border`, radius 32, padding 6-8; inner frame `$color-card`, radius 26, padding 32-40. Inner highlight: an inner `effect` shadow #FFFFFF26, y 1, blur 1. The whole card counts as one box: never put a double-bezel card inside a section box, so Taste's "two boxes deep at most" holds.
- **Pill button with nested arrow:** 48 tall, pill, label 15/500, left padding 24, right padding 6; the arrow sits in a 36 circle (`$color-accent`, or 10% of the label colour on a dark pill), flush with the right inner edge. Hover note for Koda: the button presses to .98 and the inner circle moves 2px right and 1px up, scale 1.05.
- **Diffused shadow:** `{ "type": "shadow", "color": "#2A221C0F", "offset": { "x": 0, "y": 24 }, "blur": 48, "spread": -12 }`. Never a dark, tight shadow.
- **Nav:** the source floats a glass pill nav. Draw it as a solid `$color-card` pill with a hairline and the soft shadow, centred 24 below the top; on phones a full-width bar. Its menu opens as a full-screen sheet with links set at h2 size.

## Layout signatures

1. Section padding 160 at 1440 (112 at 768, 72 at 390). Fewer things per screen; each section still one or two screens tall (Taste house rule 3).
2. **Editorial split:** a large headline on the left half, a tall photo (4:5) or a row of rounded photos on the right, top-aligned.
3. **Asymmetric bento (software and product only):** one 8-column card beside two stacked 4-column cards, all one height, rows full. Phone: one column with 24 gaps.
4. **Taste wins over the source's "Z-axis cascade":** no overlapping, tilted or rotated cards. Use the split or the bento.
5. **Taste wins over eyebrow pills:** the source puts a tiny tracked-caps pill above every heading; Taste allows one or two eyebrow labels on a page, sentence case, 13/500.

## Motion notes (for Koda; durations from `motion-timing`)

- Curve: the drawer-like `cubic-bezier(0.32, 0.72, 0, 1)` for panels and the menu sheet; strong ease-out for hovers.
- Hero entrance as Taste sets it; the menu sheet's links rise 12px in a 50ms stagger after the sheet lands.
- Taste forbids the source's blur-and-fade-up on every section entering the viewport. Hover and press layer only.

## Copy voice

Quiet and sensory, with facts: "Kamar menghadap sawah, sarapan dari dapur kami sendiri" rather than "pengalaman menginap yang tak terlupakan".

## Tells

- Cards that look like a plate seated in a tray, with concentric corners.
- Pill buttons with the arrow in its own circle.
- Twice the usual whitespace and very soft, wide shadows.

Adapted from Taste Skill's high-end-visual-design (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
