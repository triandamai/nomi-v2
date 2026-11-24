---
name: starter-systems
description: "Ten contrast-checked starter token sets for app UIs (dashboards, admin, booking and shop apps), with the open-design semantic token set mapped to your $color-*, $space-*, $radius-* and $font-* names. Read when an app needs a system and no taste-color palette or client colour fits."
license: "Apache-2.0, Copyright 2026 Open Design contributors; style families from bergside/awesome-design-skills, MIT, Copyright (c) 2026 Bergside"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (design-systems/_schema/, design-systems/<package>/tokens.css); https://github.com/bergside/awesome-design-skills"
---
# Starter systems

Order of choice: the client's own colours (`brand-extract`), then Taste's palettes (`taste-color`) for anything that is the person's site. These sets are starting points for app UIs, and every Taste rule still holds: 90–95% neutral, one accent, never violet, Taste's contrast numbers and fonts.

## The semantic set, in your names

| open-design token | Your token | Note |
|---|---|---|
| bg | `$color-background`, `$color-popover` | |
| surface | `$color-card` | On a site's cream page Taste wants cards one step darker: use the muted value there |
| surface-warm | `$color-muted`, `$color-secondary`, `$color-accent` | Bands, tinted secondary, hover wash |
| fg | `$color-foreground` and every `-foreground` partner on light grounds | |
| fg-2, meta | (none) | Second text tier: use `$color-foreground` at a smaller size, or `$color-muted-foreground` |
| muted | `$color-muted-foreground` | Must pass 4.5:1 on background, card and muted |
| border, border-soft | `$color-border` | One hairline token |
| (none) | `$color-input` | Field edge, 3:1 on background and card; upstream borders are about 1.3:1, so it gets its own value |
| accent | `$color-primary`, `$color-ring` | |
| accent-on | `$color-primary-foreground` | |
| accent-hover, accent-active | Button `Hover` and `Active` frames | The primary about 8% and 14% darker |
| success, warn, danger | `$color-success`, `$color-warning`, `$color-destructive` | Each with a `-foreground` |
| font-display, font-body, font-mono | `$font-heading`, `$font-body`, `$font-mono` | Mono only for an app's technical data |
| text-xs … text-4xl | Type styles by job (caption … display) | `taste-type` sizes win |
| space-1 … space-12 | `$space-1` … `$space-12` | Same 4px steps |
| radius-sm (buttons, inputs) | `$radius-lg` | |
| radius-md (cards, dialogs) | `$radius-xl`, `$radius-2xl` | |
| radius-lg (featured blocks) | `$radius-2xl` | |
| radius-pill | 9999 | |
| elev-flat, elev-ring, elev-raised | flat, hairline, soft | Deep is for dialogs only (`components-kit`) |
| focus-ring | the `Focus` state | `ux-accessibility` |
| motion-fast 150, motion-base 200 | Motion notes | `motion-timing` |
| container, gutters, section-y | Taste's edges and paddings | x 80 / 24, y 120 / 96 win |

## The sets (light theme)

Values are from each package's `tokens.css`, not its DESIGN.md, which contradicts it (the cafe DESIGN.md says primary #5D4432 and Poppins; its tokens say #9B5B32 and a serif). **Fixed** marks a value darkened to pass; the upstream value follows in brackets. Build dark values per `components-kit`.

| Set (packages) | Fits | background / card / muted | foreground / muted-fg | border / input | primary / on | destructive | radius sm/md/lg |
|---|---|---|---|---|---|---|---|
| krem-tembaga (cafe, elegant, refined, warm-editorial, artistic, storytelling) | Cafe, bakery, catering, warung order apps | #FBF6EE / #FFFDF8 / #F1E3CF | #201914 / #70645A fixed (#7A6D63) | #DED2C3 / #968C80 | #9B5B32 / #FFFFFF | #B33A3A | 10/16/24 |
| kertas-perunggu (paper, vintage, spacious) | Bookshop, batik, crafts, heritage guesthouse | #FAF3DF / #FFFAF0 / #EFE0BD | #211B12 / #6D6354 fixed (#7C7160) | #DDD0B4 / #968B76 | #8F5D24 / #FFFFFF | #A64036 | 10/16/24 |
| cerah-jingga (friendly, colorful, vibrant, doodle, creative, energetic) | Kids' classes, snacks, laundry, toy shop | #FFF8D7 / #FFFFFF / #FFEF9F | #1D1836 / #716789 fixed (#796F91) | #EADFBA / #968D84 | #C04F00 fixed (#FF6B00: 2.9:1 under white) / #FFFFFF | #C03C41 fixed (#E5484D) | 10/16/24 |
| retro-karat (retro, dithered, neobrutalism) | Barbershop, roaster, bengkel, record shop | #FFF4CF / #FFFAF0 / #FFDCA8 | #2A1810 / #7E5D4A fixed (#8A6652) | #D9AA7A / #AD8660 | #CE4A1E fixed (#D24B1F: 4.4:1) / #FFFFFF | #B4392E | 4/8/12 |
| tanah-liat (claymorphism, neumorphism, skeumorphism) | Salon, spa, pottery, wellness | #F7EEE6 / #FFF8F1 / #EAD6C7 | #2B211C / #6B5D54 fixed (#8A7A70: 3.6:1) | #DAC8B9 / #96877C | #A96442 fixed (#B46A46: 4.1:1) / #FFFFFF | #9E4141 fixed (#B84C4C) | 14/22/34 |
| gading-karamel (premium) | Boutique, jewellery, wedding, guesthouse | #FAF8F4 / #FFFFFF / #F0E7D8 | #1C1B19 / #6E675E fixed (#746D63) | #DED6C9 / #948F86 | #A06A3B / #FFFFFF | #AF4646 fixed (#B84A4A) | 10/16/28 |
| tinta-bersih (clean, minimal, bold, mono) | Photographer, architect, fashion label, studio | #FFFFFF / #F7F7F7 / #EEEEEE | #111111 / #6C6C6C fixed (#707070) | #D9D9D9 / #8F8F8F | #111111 (ink) / #FFFFFF | #C53030 | 4/8/12 |
| biru-jernih (professional, corporate, simple, flat, bento, perspective, contemporary) | Clinic, accountant, tutor, school admin | #F5F8FF / #FFFFFF / #EAF1FF | #101828 / #646E83 fixed (#667085) | #D7E0EF / #87909F | #2563EB / #FFFFFF | #C63838 fixed (#EF4444: 3.5:1) | 10/16/24 |
| hijau-kebun (levels) | Vegetable farm, healthy catering, plant shop | #FBF7EF / #FFFFFF / #EEF7ED | #1F2A24 / #697368 fixed (#788276: 3.7:1) | #DBE3D7 / #889288 | #2C8642 fixed (#2F8F46: 4.1:1) / #FFFFFF | #D82525 fixed (#DC2626) | 12/18/28 |
| biru-abu-app (dashboard) | Admin dashboards, POS back office, booking desk | #F4F7FB / #FFFFFF / #EEF6FF | #111827 / #627188 fixed (#64748B) | #D8E2EE / #868F9C | #0B7DB1 fixed (#0EA5E9: 2.6:1 on the ground) / #FFFFFF | #CB3A3A fixed (#EF4444) | 8/12/18 |

Keep the upstream accent where it was the brand's look (#FF6B00, #0EA5E9) as a custom `$color-brand` pair for marks and one band, never for text or labels.

## Checked contrast (worst case over background, card and muted)

| Set | fg | muted-fg | on/primary | primary on grounds | destructive | white on destructive | input on bg/card |
|---|---|---|---|---|---|---|---|
| krem-tembaga | 13.7 | 4.54 | 5.34 | 4.22 | 4.64 | 5.86 | 3.0+ |
| kertas-perunggu | 13.1 | 4.51 | 5.59 | 4.27 | 4.71 | 6.16 | 3.0+ |
| cerah-jingga | 14.6 | 4.52 | 4.82 | 4.15 | 4.57 | 5.30 | 3.0+ |
| retro-karat | 13.0 | 4.52 | 4.54 | 3.47 | 4.50 | 5.89 | 3.0+ |
| tanah-liat | 11.2 | 4.50 | 4.58 | 3.26 | 4.55 | 6.39 | 3.0+ |
| gading-karamel | 14.0 | 4.55 | 4.55 | 3.71 | 4.52 | 5.54 | 3.0+ |
| tinta-bersih | 16.3 | 4.53 | 18.9 | 16.3 | 4.71 | 5.47 | 3.0+ |
| biru-jernih | 15.7 | 4.52 | 5.17 | 4.56 | 4.60 | 5.22 | 3.0+ |
| hijau-kebun | 13.5 | 4.51 | 4.57 | 4.17 | 4.55 | 4.99 | 3.0+ |
| biru-abu-app | 16.3 | 4.55 | 4.58 | 4.20 | 4.58 | 4.99 | 3.0 |

All foregrounds pass Taste's 7:1 for reading text. Several pass by a hair (4.50–4.55): don't lighten them; for a place site, where Taste wants 4.5:1 on every text and 7:1 on a brand band, measure again.

## Fonts

Upstream sets use Inter, Georgia and system stacks, which Taste bans for a person's site. Pair from `taste-type` instead: krem-tembaga Fraunces + Figtree; kertas-perunggu Newsreader + Hanken Grotesk; cerah-jingga Nunito; retro-karat Zilla Slab + Rethink Sans; tanah-liat Bricolage Grotesque + Figtree; gading-karamel Instrument Serif + Instrument Sans (EB Garamond + Jost on a place); tinta-bersih Albert Sans; biru-jernih Source Serif 4 + Libre Franklin (Hanken Grotesk alone in an app); hijau-kebun Rethink Sans; biru-abu-app Onest. All are Google Fonts.

## Radius

Upstream ladders are rounder than `components-kit`'s default; the trade's ladder from `taste-cards-icons` wins. tanah-liat's 34 is for featured blocks only; never mix ladders.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): design-systems/_schema/AGENTS.md, design-systems/_schema/defaults.css, packages/contracts/src/design-systems/token-schema.ts, and tokens.css of cafe, paper, friendly, retro, claymorphism, premium, clean, professional, levels, dashboard. Style families from bergside/awesome-design-skills (MIT, © 2026 Bergside). Changed: rewritten for the OpenPencil canvas; contrast fixes and font pairings are ours.
