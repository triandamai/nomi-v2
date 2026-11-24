---
name: ux-accessibility
description: "The visual accessibility floor for a design: contrast per text size and UI part and how to check your $color-* pairs, focus as a drawn variant, touch targets, real labels, never colour alone, and the motion notes Koda needs. Read when building the design system frame and before saving."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (craft/accessibility-baseline.md, craft/animation-discipline.md)"
---
# Accessibility you can see on the canvas

Aim at WCAG 2.2 AA everywhere; Taste's numbers are stricter in places and then Taste wins. Code-side wiring is Koda's; this is what the design must already get right.

## Contrast floors

| What | WCAG AA | Your bar (Taste) |
|---|---|---|
| Text under 24px regular or under 18.5px bold | 4.5:1 | Reading text 7:1 in `$color-foreground`; muted text 4.5:1 |
| Large text: 24px+ regular, or 18.5px+ bold | 3:1 | 3:1 only for large muted text |
| Text on the primary (button labels) | 4.5:1 | 4.5:1 |
| UI parts that carry meaning: field borders, checkbox and radio outlines, icons without a label, chart lines and bars | 3:1 against what touches them | Same; `$color-input`, `$color-ring` and the accent 3:1 on the ground |
| Focus indicator against the unfocused look | 3:1 | Same |
| Disabled controls, pure decoration, logos | exempt | Keep disabled text readable anyway |

18px regular is not large text: it needs 4.5:1. Thresholds count as they are: 4.49:1 fails, never round up.

## Checking your token pairs

Check every pair in light and in dark, and write the ratio in the caption on the design system frame ("primary-foreground on primary 5.3:1"):
- `$color-foreground` on `$color-background`, `$color-card`, `$color-muted`, `$color-popover`.
- `$color-muted-foreground` on the same four grounds (the muted band is where it most often fails).
- `$color-primary-foreground` on `$color-primary`; `$color-secondary-foreground` on `$color-secondary`; `$color-accent-foreground` on `$color-accent`.
- `$color-primary` on `$color-background` (3:1 for the ring and filled controls; 4.5:1 if links use it as text).
- `$color-destructive` as text on the grounds (4.5:1) and white on `$color-destructive` for the destructive button.
- `$color-input` and `$color-ring` on the grounds fields sit on (3:1). A hairline `$color-border` around a card is decoration; around a field it is the field's edge and needs 3:1.
- Every chart colour 3:1 on the ground and against its neighbour.

How: contrast = (lighter luminance + 0.05) / (darker luminance + 0.05), where luminance weights the linearised red, green and blue at 0.2126, 0.7152 and 0.0722. A colour with alpha is measured as it shows on its ground. When a pair fails, darken the colour (or lighten it in dark), or change the label colour; never accept a brand hex that fails behind 14px text.

## Focus is a drawn variant

- Every interactive component gets a `/ Focus` frame in its state matrix: a 2px `$color-ring` stroke drawn outside the control with 2px clearance (`taste-app-screens`), at full strength so it passes 3:1 on the ground and against the unfocused look. A soft low-alpha ring alone often fails: measure it.
- The ring is never removed and never colour-only on a same-colour fill: on a filled primary button the outside ring carries it.
- On a dark band use a ring that passes there too; add `$color-ring` a dark-theme value.
- Draw one focused "Langsung ke isi" (skip to content) link at the top-left of one page frame, so Koda builds it.
- Heading levels by job: one h1 per screen, then h2, h3 in order; name text styles by level so the size you see and the level Koda writes match.

## Touch targets

- The legal floor is 24×24; your rule is 44 tall for every link and control outside running text (48 on a place page), 8+ apart.
- An icon button is a 44×44 frame holding a 20px icon; draw the frame, not just the glyph.
- Links inside a sentence are exempt from size but are underlined.

## Labels and names

- Every field has a visible label above it. The placeholder is never the label: it disappears when they type.
- Mark the odd ones out in words: "(opsional)" on the few optional fields, or "(wajib)" if most are optional.
- Icon-only buttons get their spoken name in the handoff ("icon button cart-remove: Hapus dari keranjang").
- In the handoff, give each content photo its alt text and mark decorative ones as decorative. These are handoff notes, never captions on the page (Taste rule 1).

## Never colour alone

- Errors: red border plus an icon plus words.
- Status chips carry a word ("Lunas", "Menunggu"), not only a colour dot.
- The selected tab or nav item: a 2px bar or weight change as well as colour.
- Charts: direct labels or different marks, not just hues; links in running text underlined.

## Motion notes for Koda

Every motion note says what happens under reduced motion (`motion-timing`):
- Movement (slide, scale, rotate, parallax) is replaced by a short crossfade or nothing; the state change still shows without it.
- Nothing flashes more than three times a second.
- Anything moving longer than 5 seconds needs a pause; loops stop when content arrives; spinners stop at 60 s.
- No autoplay, no auto-advancing slides (Taste bans carousels anyway).

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): craft/accessibility-baseline.md, craft/animation-discipline.md. Changed: rewritten for the OpenPencil canvas.
