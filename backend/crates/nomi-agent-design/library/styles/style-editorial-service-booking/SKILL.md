---
name: style-editorial-service-booking
description: "A quiet lookbook for appointment businesses: warm ivory paper, a large high-contrast serif, hairline rules instead of cards, documentary portrait crops, a calm treatment selector and a booking flow that keeps the same voice through every state."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Editorial Service Booking

A quiet lookbook for appointment businesses: warm ivory paper, a large high-contrast serif, hairline rules instead of cards, documentary portrait crops, a calm treatment selector and a booking flow that keeps the same voice through every state.

## The look

The page feels like turning through a printed salon or spa booklet. A split first screen pairs a huge serif statement with one portrait or place photo. Hairlines divide everything into a visible grid; there are no shadows and almost no radius. Navigation is tiny uppercase. Black is for decisive actions; one muted oxblood marks selection only.

## Suits / avoid

- **Suits:** salons, barbers, spas, nail and lash studios, massage, dental and skin clinics, physiotherapy, boutique hotels and homestays, wedding make-up artists.
- **Avoid for:** discount-driven shops, kids' businesses, anything needing playful colour, dense catalogues with hundreds of items.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F1EADF | Warm ivory paper |
| `$color-foreground` | #151513 | Text |
| `$color-card` / `-foreground` | #F1EADF / #151513 | Same as paper: no raised cards |
| `$color-primary` / `-foreground` | #151513 / #F1EADF | Decisive actions, selected time slot |
| `$color-secondary` | #E8DFD1 | Secondary button, form fields |
| `$color-muted-foreground` | #766F66 | Supporting copy |
| `$color-accent` / `-foreground` | #E8DFD1 / #151513 | Hovered row |
| `$color-border`, `$color-input` | #1515133D | Hairlines (ink at 24%) |
| `$color-ring`, `$color-signal` | #8F4438 | Muted oxblood: selection and status only |
| `$color-chapter` / `-foreground` | #151513 / #F1EADF | Dark "Favourites" chapter |
| `$color-placeholder` | #866E5C | Photo placeholder (also #C3A58C, #20201D for layered crops) |
| `$color-destructive` | #9E2A1F | Errors |

Radii: all `$radius-*` = 0; only circle buttons and slot pills are round. Fonts: `$font-heading` Cormorant Garamond, `$font-body` Work Sans.

## Type

- Display: **Cormorant Garamond** 400 (creator used Georgia; **Newsreader** is a sturdier alternative). Hero 72–118px desktop, 52px phone, line height 0.85, letter spacing −6%. Max 8 words per line group.
- Wordmark: the display serif in italic, 24px.
- Section headings: serif 30–34px, 400.
- Labels and nav: **Work Sans** 500, 10–11px uppercase, tracking 14%.
- Body: Work Sans 400, 15–16px/24, `$color-muted-foreground`, max 34 characters wide in the hero foot, 60 elsewhere.

## Shape, spacing, depth

- cornerRadius 0 everywhere, except circular buttons (52px ellipse) and text links.
- Stroke 1 `$color-border` for every division: nav bottom, hero split, selector columns, rows.
- No shadows. Ever.
- Outer padding 34 desktop, 20 phone. Rows 14px vertical padding.

## Layout signatures

1. **Nav bar:** horizontal, space-between, padding 22 34, bottom stroke. Left: 3 uppercase links gap 26. Centre: italic wordmark. Right: "Pesan jadwal" with a 1px underline.
2. **Split hero:** horizontal, two children 1 : 1.08. Left frame vertical space-between, padding 42 34 30: tiny label, huge serif statement, foot row (2-line `$color-muted-foreground` paragraph + 52px circle button with an arrow icon). Right: photo placeholder fill_container with a left stroke, caption bottom-right in 10px uppercase white ("Foto: Siti Rahayu di kursi dekat jendela").
3. **Treatment selector:** horizontal row of three columns (0.52 : 1 : 0.8) divided by vertical strokes. Column 1: serif heading "Perawatan". Column 2: list of rows, each top-stroked, space-between: name left, "45 mnt" right; selected row text `$color-signal` with a 6px `$color-signal` dot. Column 3: fixed-height stage with photo placeholder, duration, price "Rp 185.000", description, text link "Pesan perawatan ini".
4. **About:** two or three overlapping photo placeholders with captions, clear reading order.
5. **Favourites chapter:** full-width `$color-chapter` band with real popular services only.
6. **Location block:** address "Jl. Malioboro No. 52, Yogyakarta", hours, contact "halo@salonsiti.example", accessibility note — before the booking form.

## Booking flow frames

Draw: service → stylist → time → details → confirmation, with the chosen service, stylist and time always visible in a summary column. Also draw the stale-slot state ("Jam 14.00 baru saja terisi. Pilih jam lain.") and an error with retry. Never show a slot as booked before confirmation.

## Components

- Primary: fill `$color-primary`, text `$color-primary-foreground`, height 48, radius 0, label uppercase 11px tracking 14%.
- Circle action: 52px ellipse, stroke 1 `$color-foreground`, arrow icon 18px.
- Time slot: 44px tall, stroke `$color-border`; selected fill `$color-foreground` text `$color-background`; unavailable text `$color-muted-foreground` with strike-through.

## Copy voice

Short, confident, about craft and care. "Potongan yang dipikirkan, bukan terburu-buru." No beauty superlatives, no "terbaik se-Jogja".

## Tells

- A serif hero so large and tight (line height 0.85) it becomes the image.
- Hairlines dividing the page into a visible grid; zero cards, zero shadows.
- Tiny tracked uppercase nav with an underlined booking link.
- Treatment list as ruled rows with duration flush right.
- One muted oxblood used only for "selected".

## Note for Koda

Treatment changes swap the stage image in 160–220ms; section entrances 500–760ms. An optional slow text loop for one real positioning line, slower than 14s.

Adapted from mengto/skills: agent-skills/web-design/editorial-service-booking (MIT, Copyright (c) 2026 Meng To).
