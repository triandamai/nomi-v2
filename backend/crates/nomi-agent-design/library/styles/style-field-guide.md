# Style: Field Guide

A botanical field-guide layout: pale sage paper and deep forest chapters, one light geometric sans at large sizes, continuous vertical guide lines, aligned chapter labels and specimen captions, and a short diagonal registration tick anchored to a real figure.

## The look

Light, airy and natural, but measured like a scientific plate. Display type is big and thin. Vertical guides run the full height just outside the content. Each figure sits in a faint registration box with a diagonal tick across it and a caption like a specimen label. A dark forest-green spread breaks the page once, with a full-width landscape.

## Suits / avoid

- **Suits:** plant nurseries, farms and kebun, eco-tourism and trekking guides, coffee and cacao origin stories, honey and herbal producers, outdoor schools, environmental groups, science tutors.
- **Avoid for:** fashion, nightlife, finance, anything urban and loud.

## Tokens

| Token | Light | Forest chapter | Role |
|---|---|---|---|
| `$color-background` | #F1F0E6 | #18271F | Page / dark spread |
| `$color-foreground` | #26352A | #EEEDDC | Text |
| `$color-card` / `-foreground` | #E9EBD6 / #26352A | #243128 / #EEF0D9 | Pricing and info cards |
| `$color-muted` | #E6E6D7 | #1F3026 | Figure backgrounds |
| `$color-muted-foreground` | #5E6B58 | #B9C2A8 | Captions |
| `$color-primary` / `-foreground` | #2F3C2F / #EEF0D9 | #EEF0D9 / #18271F | Pill button |
| `$color-border` | #78867044 | #D5DDC43B | Bars, caption rules |
| `$color-guide` | #26352A21 | #EEEDDC21 | Vertical guides (13%) |
| `$color-signal` | #94A371 | #AEBD71 | Olive dot, brand mark |
| `$color-ring` | #AEBD71 | #AEBD71 | Focus |

Radii: `$radius-sm` 3 (figures) · `$radius-lg` 12 (cards) · pill buttons 30. Fonts: `$font-heading` and `$font-body` Lexend (330 display, 350 body, 500–600 labels).

## Type

- Display: 52–116px (up to 260px for a single wordmark), weight 330, line height 0.99, letter spacing −7%.
- Section 38–84px, same weight. Body 16–22px/1.55, measure 39 characters in the hero, 60 elsewhere.
- Labels 11px uppercase, tracking 11–14%, 600, preceded by a 9px `$color-signal` dot.

## Layout signatures

1. **Shell:** max width 1560, padding 24–90. Two vertical guide rectangles (1px `$color-guide`) full height, 13–69px in from the edges.
2. **Bar:** height 85, bottom stroke `$color-border`: wordmark with a small asterisk mark in `$color-signal` left, mono-like caps meta right ("STASIUN LAPANGAN · 1.240 MDPL").
3. **Panel:** two columns 0.9 : 1.1, gap 4% of width: copy left (label, display headline, short paragraph, pill button); figure right, height 340–740.
4. **Figure:** placeholder `$color-muted`, radius 3; inner registration box inset 6% with stroke 1 #7F8C6924; one diagonal line (1px #63745626, rotated −16°) crossing it; caption bottom-right 11px tracking 8% ("Gbr. 3 — Pakis haji, Cycas rumphii").
5. **Caption bar:** top stroke, 17px padding, label left, folio right.
6. **Forest spread:** full-width `$color-background` dark chapter with a landscape placeholder and 3 pricing cards ("Tur pagi · Rp 249.000 / orang").

Phone: figure follows its text; interior rules collapse before type shrinks.

## Components

- Pill button: radius 30, min height 44, fill `$color-primary`, padding 7 8 7 17, label 14px plus a 32px round icon well on the right (fill `$color-card`).
- Inner highlight on pills: inner shadow y 1 #F4F5CF24; drop shadow y 6 blur 18 #131B1229.

## Copy voice

Observational, calm, specific to place and species. "Kebun di lereng Merbabu, dipanen setiap Selasa."

## Tells

- Thin, very large geometric sans with tight tracking.
- Full-height vertical guides just outside the content.
- Figures in registration boxes with one diagonal tick and specimen captions.
- Labels led by a small olive dot.
- One deep forest-green spread breaking the pale page.

## Note for Koda

The source fills figures with radial gradients and live canvases and darkens the landscape with gradient overlays; canvas uses flat tones and a flat alpha band. Koda may restore the soft radial figure backgrounds.

Adapted from mengto/skills: agent-skills/web-design/sylva-field-guide-layout (MIT, Copyright (c) 2026 Meng To).
