---
name: style-portfolio-chapters
description: "A work-first portfolio told as magazine chapters: near-black frame, one muted colour field per project, a billboard grotesk wordmark, square media, hairlines and almost no shadow, ending in one bright contact chapter."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Portfolio Chapters

A work-first portfolio told as magazine chapters: near-black frame, one muted colour field per project, a billboard grotesk wordmark, square media, hairlines and almost no shadow, ending in one bright contact chapter.

## The look

The page opens on one big image with the name set huge across it. Each project is a new chapter: its own muted colour (pink, lavender, lime), an oversized title, role and year in tiny caps, square media, and a visible action. The studio's story comes only after the work. Services appear as giant type. Contact switches to one unmistakable colour.

## Suits / avoid

- **Suits:** photographers, wedding videographers, graphic and interior designers, make-up artists, illustrators, architects, small creative studios, freelancers with 3–8 strong projects.
- **Avoid for:** anyone without strong visual work, shops, clinics, booking-heavy services.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #10100F | Frame, hero, dark chapters |
| `$color-foreground` | #F3F0E8 | Text on black |
| `$color-card` / `-foreground` | #F3F0E8 / #10100F | Warm white chapter |
| `$color-muted-foreground` | #A8A49A | Metadata |
| `$color-primary` / `-foreground` | #F3F0E8 / #10100F | Pill buttons on black |
| `$color-border` | #FFFFFF3D | Hairlines on black |
| `$color-chapter-1` | #ADA4CF | Lavender project field |
| `$color-chapter-2` | #F4A9C7 | Pink project field |
| `$color-chapter-3` | #B9CF79 | Lime project field |
| `$color-signal` | #D46C47 | Contact chapter (one bold colour) |
| `$color-ring` | #F3F0E8 | Focus |

Text on a chapter field is `$color-background` #10100F. Radii: `$radius-sm` 2 for media, 999 for nav pills only. Fonts: `$font-heading` Archivo 800–900, `$font-body` Inter 400.

## Type

- Wordmark: 88–220px, 900, letter spacing −9%, line height 0.85.
- Project title: 70–146px, 800, letter spacing −6%.
- Services: 56–96px links, wrapping on purpose.
- Labels: 10px uppercase, tracking 13%. Body 15–18px/26.

## Layout signatures

1. **Header:** pill nav at both outer edges (radius 999, stroke 1 `$color-border`, 12px text), wordmark centred.
2. **Hero:** photo placeholder about 90% of the first viewport ("Foto: kampanye kebaya, studio Yogyakarta, cahaya keras"); wordmark over it; one small positioning line bottom-right. The first project starts just below the fold.
3. **Project chapter:** full-width frame filled with `$color-chapter-n`, 12-column logic, edge padding 48 (24 phone). Left 7 columns: square media placeholder. Right 5: title, then a metadata row "Klien · Peran · 2026", then "Lihat proyek" with an arrow, visible without hover.
4. **Studio:** an asymmetrical portrait placeholder plus 3 short paragraphs, never a team-card grid.
5. **Services:** stacked giant type ("Foto produk", "Video pernikahan", "Art direction") divided by hairlines.
6. **Contact:** `$color-signal` chapter, one line, one button: "Ceritakan proyekmu → halo@rumahrana.example".

## Components

- Nav pill: height 36, padding 0 16. Project action: text 14px plus a 32px circle with an arrow.
- No cards, no shadows. Media radius 2.

## Copy voice

Few words, confident, specific to the work. "Kampanye Lebaran untuk Batik Larasati, 2026." No "passionate creative".

## Tells

- Each project owns a muted colour field; black frames between them.
- Billboard-scale grotesk with very tight tracking.
- Role and year in tiny caps next to giant titles.
- Services set as huge type, not icon cards.
- One loud colour, used only for contact.

## Note for Koda

Project media reveal with a 500–760ms mask; on hover or focus scale media to 1.02 only. No scroll hijacking.

Adapted from mengto/skills: agent-skills/web-design/editorial-portfolio-chapters (MIT, Copyright (c) 2026 Meng To).
