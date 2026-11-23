---
name: style-agency-grid-minimal
description: "Architectural minimalism on a strict grid: huge tight headlines spanning most columns, tiny uppercase metadata in narrow side columns, large panoramic image blocks, service rows as clean listings, and a white content block that overlaps the hero."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Agency Grid Minimal

Architectural minimalism on a strict grid: huge tight headlines spanning most columns, tiny uppercase metadata in narrow side columns, large panoramic image blocks, service rows as clean listings, and a white content block that overlaps the hero.

## The look

Light neutral page, lots of open space, every element placed on a visible column logic. Scale contrast does the work: headlines at 9rem beside 10px labels. Images are big and calm (facades, interiors, materials), framed only by spacing. A lime accent appears as a tiny dot or underline, nowhere else.

## Suits / avoid

- **Suits:** architects, interior designers, contractors and renovation firms, furniture makers, property developers, landscape designers, strategy consultancies.
- **Avoid for:** shops with many products, food, booking flows, anyone without strong images; also not for dashboards.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F2F1EE | Page |
| `$color-foreground` | #111111 | Text |
| `$color-card` / `-foreground` | #FFFFFF / #111111 | Overlapping content block |
| `$color-muted` | #E8E6E1 | Image placeholder base, hover row |
| `$color-muted-foreground` | #6B6B6B | Body, labels |
| `$color-primary` / `-foreground` | #111111 / #F2F1EE | Primary button |
| `$color-secondary` | #F2F1EE | Secondary button fill |
| `$color-border` | #1111111A | Separators |
| `$color-ring` | #111111 | Focus |
| `$color-signal` | #CCFF00 | Tiny dot, active underline only |
| `$color-placeholder` | #B9B4AA | Photo placeholder tone |

Radii: `$radius-sm` 1 (buttons) · `$radius-2xl` 32 (top corners of the overlapping block). Fonts: `$font-heading` and `$font-body` Inter Tight (500 headings, 400 body), or Geist.

## Type

- Hero: 72–144px, 500, letter spacing −4%, line height 0.95, deliberate line breaks (3 lines max).
- Section titles 48–72px. Service row titles 28–32px.
- Labels 10–11px uppercase, tracking 18%. Body 15px/24, max 40 characters wide in side columns.

## Layout signatures

1. **Hero grid:** 12 columns, margins 48 (20 phone). Headline spans columns 1–10. A narrow lane (columns 10–12) holds a 3-line description and an index "01 — Rumah tinggal". Tiny meta row at top: "STUDIO · BANDUNG · EST. 2014".
2. **Panoramic image:** full container width, 21:9, placeholder `$color-placeholder` with caption ("Foto: fasad rumah beton dan kayu, Dago, pagi").
3. **Overlapping block:** `$color-card` frame with top radius 32 sitting 64px up over the image bottom, padding 64.
4. **Service rows:** each row horizontal, top hairline, padding 24 0: index (10px label, columns 1–2), title (columns 3–8), one-line description (columns 9–12). Hover state: background `$color-muted`.
5. **Project index:** list of name / location / year in three columns, no thumbnails until opened.

Phone: headline drops to 48px, side lane moves under the headline, service rows stack index above title.

## Components

- Button: height 40, radius 1, stroke 1 `$color-foreground` at 40%, label 11px uppercase tracking 14%; primary filled.
- Active marker: 6px `$color-signal` square beside the current nav item.

## Copy voice

Spare and spatial. "Rumah yang menua dengan baik." Projects named by place and year. No "solusi desain inovatif".

## Tells

- Enormous tight headlines with tiny caps metadata beside them.
- Content placed in visible column lanes, much of the grid left empty.
- One panoramic architectural image framed only by space.
- A white block with rounded top corners overlapping the image.
- Service rows as ruled listings with indices, no cards or icons.

## Note for Koda

Optional: masked headline reveal and slow image settle; the source mentions a faint wireframe object in the background, which is not drawn on canvas and is not required.

Adapted from mengto/skills: agent-skills/web-design/agency-grid-layout-minimal (MIT, Copyright (c) 2026 Meng To).
