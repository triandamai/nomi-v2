---
name: style-image-first-grid
description: "One big photograph is the stage: full-bleed image, a dark scrim for legibility, thin container rails and corner squares laid over it, and content anchored low in grid lanes so the place itself sells."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Image-First Grid

One big photograph is the stage: full-bleed image, a dark scrim for legibility, thin container rails and corner squares laid over it, and content anchored low in grid lanes so the place itself sells.

## The look

The first screen is almost all photo. Copy sits low-left in one grid lane; a quote, rating the person gave you, or "next available date" box floats in another lane. Vertical hairline rails and 6px squares mark the container edges over the image. Below the hero, the page turns to a quiet light neutral with serif headings, so the photo stays the event.

## Suits / avoid

- **Suits:** villas, homestays, guesthouses and kos with good photos, tour and dive operators, wedding venues, resorts, coffee farms, restaurants with a strong room, property listings.
- **Avoid for:** businesses without real photos yet (the design collapses into grey boxes), dense catalogues, dashboards, text-heavy services.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F6F5F2 | Light sections below the hero |
| `$color-foreground` | #1C201C | Text on light |
| `$color-card` / `-foreground` | #FFFFFF / #1C201C | Booking card, room cards |
| `$color-muted-foreground` | #767873 | Body, labels |
| `$color-primary` / `-foreground` | #1B241A / #F6F5F2 | Primary button on light |
| `$color-secondary` | #ECEAE5 | Secondary button |
| `$color-border` | #1C201C14 | Hairlines on light |
| `$color-ring` | #8A9A6B | Focus, one muted accent picked from the photo |
| `$color-scrim` | #0A0C0A99 | Scrim panel over the photo |
| `$color-scrim-strong` | #0A0C0ACC | Bottom band behind hero copy |
| `$color-rail` | #FFFFFF26 | Rails over the photo |
| `$color-placeholder` | #3F4A3C | Photo placeholder tone |

Radii: `$radius-lg` 12 · `$radius-xl` 24 · `$radius-2xl` 32; buttons 999. Fonts: `$font-heading` Playfair Display 400, `$font-body` Inter 300/400, custom `$font-mono` JetBrains Mono for 10–11px labels.

## Type

- Hero headline: serif 56–88px, 400, letter spacing −2%, white, max 2 lines.
- Section headings on light: serif 40px. Body 16px/26, 300–400.
- Labels: mono 10–11px uppercase, tracking 15%, white at 50% over the photo, `$color-muted-foreground` on light.

## Layout signatures

1. **Stage:** frame full width, height 100% of the first viewport (900 desktop, 760 phone). Children, bottom to top: photo placeholder fill_container with caption ("Foto: kolam villa saat matahari terbenam, Ubud"); scrim rectangles (a left panel 60% wide `$color-scrim`, a bottom band 45% tall `$color-scrim-strong`); rails; content.
2. **Rails:** two 1px vertical rectangles `$color-rail` at the container edges (max width 1200), 6px squares #FFFFFF66 where rails meet the nav line and the bottom band.
3. **Anchored copy:** bottom-left lane, padding 48: mono label "VILLA SAWAH · UBUD", serif headline, 2-line body, button row.
4. **Side box:** right lane, 320 wide, fill #0A0C0A66, stroke 1 #FFFFFF26, radius 24, padding 24: "Tanggal kosong berikutnya: 26 Jul 2026 · mulai Rp 1.450.000/malam".
5. **Light continuation:** rooms or packages as 2–3 large image cards, radius 32, with inner highlight (inner shadow y 2 blur 4 #FFFFFF) and drop shadow y 8 blur 20 #0000000A.

On phone: the photo takes 70% of the first screen, copy sits below the photo on `$color-primary` dark rather than over a busy area.

## Components

- Hero button: fill #0A0C0A66, stroke 1 #FFFFFF33, text white 14px, height 44, radius 999. Primary on light: `$color-primary`.
- Nav: transparent, white text 13px, logo left, 3 links, one "Cek tanggal" button.

## Copy voice

Sensory and exact: place, light, distance. "Sepuluh menit jalan kaki ke sawah Tegallalang." Prices per night with dates.

## Tells

- The photo fills the first screen; content only lives in its lower lanes.
- Hairline rails and tiny squares drawn over the photograph.
- A translucent dark side box with one useful fact.
- Serif headline on the image, mono micro-labels.
- Quiet off-white sections afterwards with big rounded image cards.

## Note for Koda

The source uses directional gradient washes and faint animated lines over the image. Canvas uses flat alpha scrim panels: Koda should turn them into left-to-right and bottom-up gradients and may add slow image drift. Lost on canvas: the smooth fade from image into scrim.

Adapted from mengto/skills: agent-skills/web-design/image-first-grid-layout (MIT, Copyright (c) 2026 Meng To).
