---
name: taste-type
description: "The type system for a Taste design (faces, weights, sizes at 1440 and 390, tracking, line lengths) and the compact table of Taste's Google Fonts pairings by kind and trade."
license: "MIT, Copyright (c) 2026 Leonxlnx"
source: "https://github.com/Leonxlnx/taste-skill (2.5.1)"
---
# Taste: type

The type system for a Taste design (faces, weights, sizes at 1440 and 390, tracking, line lengths) and the compact table of Taste's Google Fonts pairings by kind and trade.

## Rules

- **One family, or a display face plus a text face** with fixed jobs; never three. At most four font files (weights) in all.
- **Never** Inter, Roboto, Open Sans, Lato, Montserrat, Poppins, Space Grotesk, Plus Jakarta Sans or a system-UI look for the person's site.
- **Pick from the table** by kind (and on a place by the trade's group), the first that fits and that you did not use last in this conversation. A `word` or `editorial` hero needs a face with a weight or width axis (column "axis"). Write the pairing on plan line 2. Real Google Fonts names: Koda loads them; the Studio may show a fallback.
- **Weights:** headlines 400 on product, service and place; 500 on tech. Size and tight tracking carry a headline, never bold-and-big. Text 400, and 500 for labels, buttons, today's row and strong words. Never 700 or 800. No light weight under 20px.
- **Tracking:** headlines of 40px and up tightened by -0.5% to -4.5% (most pairings about -3% at h2, -4% at hero size; serifs about -1.5%). Body text at 0.
- **Sentence case everywhere.** Labels 13px at 500, never tracked capitals or small caps. Italic only for quotes, attributions and contact lines.
- **Lines at most 62 characters** (set the text frame's width: about 560px at 16px). A number never breaks from its unit. Text never shrinks to fit: cut words.
- **Nothing under 13px** on a page: notes, the legal line, image text. One size per kind of element across the page (every card title, every price, every address). A figure never outgrows the h2 or the h1, at 390 too.
- Each level at least 1.3x the next from 1024 wide (1.2x below). Display text of 40px+ outside a grid item counts as a heading: a statement set at h2 size in body text is a fault.
- The hero has one quieter level of small text at most (a grey fact line, or a pill above the h1).
- Every word is live text, never part of a picture.

## Sizes (px, size/line-height; desktop 1440 / phone 390)

| Token | Default kinds | Place |
|---|---|---|
| h1 (hero) | 72 / 38 (each hero composition's own size in `taste-layout`) | 48/53 / 32/37 |
| h2 | 48 / 30 | 32/40 / 24/30 |
| h3, quotes | 28 / 24 | names 20/28 / 18/26 |
| lead, card title | 20 / 18 | sub-line 20/30 / 18/28 |
| body | 16/26 (both) | 16/26 |
| small | 14 | 14/20 |
| label | 13/500 | 13/16 |
| stat (under an h2) | 38 / 28 | not used |

Body and reading text line height 1.5 to 1.8. Make these text styles (faces as `$font-heading` and `$font-body`) once in the design system frame and use only them.

## Pairings

hw = tech hardware, sw = software, game, prod = product, svc = service; the digit is how strongly Taste deals it to that kind (3 = first choice). Place group: a place uses only rows with a group. Trade groups for a place: serif for a bakery, restaurant, guesthouse, wedding; plain sans for a shop, workshop/bengkel, studio, clinic, law office, agency; soft sans for a cafe or wellness/salon/yoga. "=" means the text face is the display face.

| Display | Text | kinds | place | axis | suits |
|---|---|---|---|---|---|
| Geist | = | hw3 sw3 game1 prod1 svc1 | - | wght | developer tools, AI, infrastructure, fintech |
| Host Grotesk | = | hw2 sw3 svc2 prod1 | - | wght | software, studios, agencies wanting a quiet exact page |
| Instrument Sans | = | hw1 sw3 prod1 svc2 | - | wght | design and productivity software, modern agencies |
| Mona Sans | = | hw2 sw2 game2 prod2 | - | wdth | hardware, platforms, game studios |
| Hanken Grotesk | = | sw1 prod2 svc3 | - | wght | clinics, law, advice, agencies |
| Schibsted Grotesk | = | sw2 svc3 prod1 | - | wght | media, agencies, sturdy editorial services |
| Onest | = | sw2 svc2 prod1 | - | wght | even, open, readable software and services |
| Funnel Display | Funnel Sans | sw3 prod1 game1 svc1 | - | wght | consumer apps and products, friendly and bright |
| Manrope | = | hw2 sw2 prod1 svc1 | - | wght | fintech, data and AI products |
| IBM Plex Sans | = | hw3 sw2 svc1 | - | wght | developer tools, security, data infrastructure |
| Sora | Figtree | hw2 game3 sw1 | - | wght | games, fintech, hardware (a wide geometric face) |
| Archivo | = | prod3 hw1 game2 svc1 | - | wdth | hardware, vehicles, equipment, games |
| Anybody | Onest | game2 | - | wdth | game studios and esports |
| Bricolage Grotesque | Figtree | sw1 prod3 svc1 game1 | soft | wght | brands with character, studios, kitchens that sell online |
| Red Hat Display | Red Hat Text | svc3 sw1 prod1 | - | wght | trades, finance advice, health, schools |
| Gabarito | Figtree | sw2 svc1 game1 | soft | wght | consumer apps, casual games |
| Chivo | = | hw2 sw2 | - | wght | developer and data tools |
| Outfit | = | sw2 prod1 game1 svc1 | soft | wght | apps and consumer brands |
| Epilogue | = | hw2 sw2 game2 prod1 | - | wght | platforms, tools, game studios |
| Familjen Grotesk | = | hw1 sw2 game1 prod1 | - | wght | software with an editorial edge, creative tools |
| Urbanist | = | hw1 sw2 game1 prod1 | - | wght | apps, fintech, consumer software |
| Reddit Sans | = | hw2 sw2 game1 | - | wght | platforms, communities, AI tools |
| SUSE | = | hw2 sw1 game1 | - | wght | infrastructure, security, developer platforms |
| Albert Sans | = | svc2 prod2 sw1 | plain | wght | shops, studios, small services |
| Figtree | = | sw1 prod2 svc2 | plain | wght | friendly shops, studios, services |
| Work Sans | = | svc2 prod1 | plain | wght | bike shops, studios, trades |
| Rethink Sans | = | svc2 prod2 | plain | wght | florists, studios, small shops |
| Instrument Sans (static) | = | place only | plain | - | shops and studios selling design and objects |
| Instrument Serif | Instrument Sans | prod3 svc2 | - | - | refined, editorial brands, studios, firms |
| Fraunces | Figtree | prod3 svc1 | serif | wght | bakeries, roasters, food and homeware brands |
| Newsreader | Hanken Grotesk | svc3 prod1 | serif | wght | bookshops, guesthouses, advisers, schools |
| Source Serif 4 | Libre Franklin | svc3 | serif | wght | law, finance, health, schools |
| EB Garamond | Jost | prod2 svc1 | serif | wght | patisseries, bookshops, old guesthouses |
| Literata | Figtree | svc2 prod1 | serif | wght | bookshops, cafes with a reading corner |
| Lora | Nunito Sans | prod1 svc1 | serif | wght | roasters, bakeries, delis |
| Besley | Albert Sans | prod2 svc1 | serif | wght | cafes, bakeries, bars |
| Vollkorn | Work Sans | prod1 svc1 | serif | wght | bakeries, delis, bookshops |
| Zilla Slab | Rethink Sans | prod2 svc2 | serif | - | workshops, bengkel, outfitters, trades |
| Nunito | = | prod1 svc1 | soft | wght | cafes, bakeries, kids, wellbeing |

Display scale (outside a place): Instrument Serif and EB Garamond run small, so set their headline sizes about 1.12x; Newsreader 1.06x; Instrument Sans display 1.04x; Mona Sans 0.96x; Sora and Archivo 0.9x; Anybody 0.86x. On a place every face is set at scale 1. A mono face is used only when the design shows code or an app's technical data (Geist Mono, JetBrains Mono, DM Mono); never on service or place.

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
