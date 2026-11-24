---
name: taste-color
description: "How colour works in a Taste design (a neutral interface, one accent, three grounds, the contrast numbers, never violet) and the compact table of Taste's palettes by trade, to pick from instead of inventing colours."
license: "MIT, Copyright (c) 2026 Leonxlnx"
source: "https://github.com/Leonxlnx/taste-skill (2.5.1)"
---
# Taste: colour and palettes

How colour works in a Taste design (a neutral interface, one accent, three grounds, the contrast numbers, never violet) and the compact table of Taste's palettes by trade, to pick from instead of inventing colours.

## Rules

- **The interface stays 90 to 95% neutral.** Colour comes from the photographs, the product, the work and real logos. One near-black ink (often warm, never #000000 unless the palette says so) for text, hairlines (ink at 12-15% alpha, e.g. `#2a262220`), button borders, interface glyphs and the mark.
- **One accent at most,** ideally from the client (product, shopfront paint, packaging): the primary button (the only filled accent), the focus ring, links, the one state mark in a UI piece, small marks and the one brand band; under 3% of a screen elsewhere. Its label 4.5:1 on it; it 3:1 on the ground. If the palette's accent is under 3:1 on the ground (marked * below), the primary button and links take the ink, and the accent appears only as a band or a small mark.
- **Highlight by contrast, never by colour:** no marker highlights or coloured bars behind words; colour inside a heading's word once a page at most, never on a place.
- **Three ground tokens** for every section and panel: `$color-background` (main), `$color-muted` (alt: panels, tiles, one band), and the custom pair `$color-inverse` / `$color-inverse-foreground` (dark band). One to three grounds down the page, same order at 390.
  - Main: white or a light tint (lightness 0.80-0.97, low saturation) from the client: a place's walls or paper, a product's backdrop, warm greys for a service, cool grey for software. It covers 55%+ of the page height (60% on software and product).
  - Surface: the same hue, a small step darker (`#ffffff` to `#f2f2ef`, `#f7f3ec` to `#ede6da`): panels, tiles, cards and at most one full band.
  - Inverse: near-black (`#111111` to `#1a1a1a`) or the deepest shade of the brand hue, never charcoal grey, never violet. One dark section at most (one screen), mid-page or later, never right under the hero; the footer may be dark.
- **Brand band** (product, place): one section in the brand's own colour may replace the dark band, once, in the second half. On a place it is a real colour (not neutral near-black), and every word on it passes 7:1: darken the colour until it does; the primary button on it is inverted.
- **Dark pages and dark heroes are tech only** (software, gaming, hardware).
- **Flat colour only** on product, service and place: no gradients, grain, glass, textures, patterns, tints or shadows on the interface (the canvas cannot draw most of these anyway). Tech may have up to two dealt grounds and one effect: draw their still as flat fills from the palette and annotate the intended ground for Koda (`taste-photos-motion`).
- **Words on a photograph** only in the hero and one closing band, on tech, product and service, on a placeholder whose note specifies a calm, dark (or light) empty area there at the phone crop too; never on a place; never a scrim box faked over a photo.
- **Light or dark theme** is chosen once, never per scroll; the dark grade is tech only. A light/dark theme pair in the `.pen` file is fine for app screens.
- **Never violet or lavender** in the design's own colours: accent, ground, band, tint or effect. A colour whose OKLCH hue falls between about 270 and 330 degrees turns to 255 (blue) or 345 (rose), whichever is nearer. A real third-party logo keeps its own colours.

## Contrast numbers

| What | Minimum |
|---|---|
| Text people read (sub-lines, card lines, list items, answers), 16px+ | 7:1 in `$color-foreground` |
| Muted text (labels, dates, meta, a heading's grey run-on) | 4.5:1 |
| Large text 24px+ in a muted half or inactive row | 3:1 |
| Every text on a place | 4.5:1; on the brand band 7:1 |
| Accent button label on the accent | 4.5:1 |
| Accent, focus ring, field border against the ground | 3:1 |

Never 14px paragraphs or 15px grey card text: card text is 16px in the ink (ink at about 84% is fine if it holds 7:1).

## Picking a palette

1. Find the trade (Indonesian mapping: kedai kopi = cafe; toko roti, kue = bakery; warung, rumah makan = resto; penginapan, homestay, villa = hotel; salon, spa, yoga, gym = wellness; bengkel, penjahit, mebel = workshop; toko, butik, florist = retail; klinik, dokter gigi = clinic; notaris, konsultan pajak = law; WO, fotografer pernikahan = wedding; aplikasi = saas; POS, payment = fintech).
2. Take the palettes listed for that trade, kind-filtered: a place uses only rows marked P (light, quiet grounds); dark rows are tech only.
3. Choose the first that does not repeat a palette you used in this conversation and does not clash with the client's logo or photos. With the client's own colours (logo, painted shopfront), build from those instead: ground from walls, paper or shopfront, ink near-black, accent darkened step by step until its label passes 4.5:1 and it passes 3:1 on the grounds.
4. Write it on plan line 2 with one sentence on why it fits the trade, from what the palette answers in it ("flour-terracotta: a flour-white ground and the terracotta of an oven's tiles"), never a fact the brief does not give.

Map the palette onto the shadcn tokens (light theme; Taste's "accent" is shadcn's **primary**, not `$color-accent`):

| Palette role | Token(s) |
|---|---|
| bg | `$color-background`, `$color-popover` |
| ink | `$color-foreground`, `$color-card-foreground`, `$color-popover-foreground`, `$color-secondary-foreground`, `$color-accent-foreground` |
| surface | `$color-muted`, `$color-secondary`, `$color-accent` (hover wash, active row), and `$color-card` (cards sit one step off the page; on a white page card = surface, never pure-white cards on a cream page) |
| muted | `$color-muted-foreground` |
| accent | `$color-primary` and `$color-ring`; if marked *, `$color-primary` = ink and the accent becomes the custom pair `$color-brand` / `$color-brand-foreground` for small marks and the band |
| accentInk | `$color-primary-foreground` (when primary = ink: the bg) |
| band | `$color-inverse`; `$color-inverse-foreground` = bg, or white or near-black, whichever passes 7:1 |
| ink at 12-15% alpha | `$color-border`; `$color-input` the same or a step stronger |
| (none in Taste) | `$color-destructive`: a red passing 4.5:1 on the background, never violet |

## The palettes

Light grounds (P = fits a place). * = accent under 3:1 on the ground: buttons in ink.

| id | P | trades | bg | surface | ink | muted | accent | accentInk | band |
|---|---|---|---|---|---|---|---|---|---|
| white-link-blue | P | saas,devtool,fintech,law,clinic,retail | #ffffff | #f4f4f4 | #111111 | #6b6b6b | #1a4be0 | #ffffff | #111111 |
| cool-white-slate | P | clinic,wellness,saas,fintech,law,hardware | #f7f9fa | #ecf0f2 | #1f3038 | #5d6b72 | #0d7480 | #ffffff | #1f3038 |
| rose-grey-green | P | cafe,bakery,studio,wellness,wedding,retail | #ebe5e3 | #e1dad7 | #2b2622 | #6a615d | #0f7d45 | #ffffff | #2b2622 |
| paper-grey-brass | P | agency,studio,resto,workshop,law | #e8e8e3 | #dcdcd6 | #181715 | #63625c | #bfb21a* | #181715 | #181715 |
| stone-blueprint | P | workshop,hardware,agency,studio,devtool | #dfdedb | #d3d2ce | #26251e | #5f5d55 | #2456d6 | #ffffff | #26251e |
| white-navy-tab | P | hotel,retail,product,clinic | #ffffff | #f4f4f2 | #1f2b4d | #6a7186 | #f7941d* | #1f2b4d | #05070a |
| three-stones | P | hotel,wellness,wedding,studio,law,clinic | #eeecea | #e4e1dd | #333231 | #6b6763 | #5c6b54 | #ffffff | #333231 |
| putty-black-teal | P | cafe,resto,workshop,studio,hardware,product | #dedad3 | #d3cec6 | #0a0a0a | #5c5852 | #256f82 | #ffffff | #0a0a0a |
| moss-sand | P | cafe,bakery,hotel,wellness,resto,product | #ddd6c6 | #d2cab8 | #2a2a26 | #59574e | #4f6238 | #ffffff | #2a2a26 |
| green-mat-chocolate | P | cafe,bakery,resto,hotel,retail | #e5e1d8 | #dad5ca | #2a1d17 | #6b5b51 | #456f3a | #ffffff | #2a1d17 |
| white-signal-red | P | resto,retail,agency,product,hardware | #ffffff | #f2f2f2 | #0a0a0a | #6b6b6b | #d92b21 | #ffffff | #000000 |
| white-mint-tiles | P | saas,ai,fintech,product | #ffffff | #f3f3f3 | #232323 | #6e6e6e | #a1ffcb* | #232323 | #232323 |
| cool-grey-cobalt | P | devtool,saas,ai,fintech,hardware | #eef0f3 | #e2e5ea | #0b0c0e | #5d6470 | #1f2de6 | #ffffff | #0b0c0e |
| offwhite-bottle-yellow | P | bakery,cafe,retail,product | #f7f7f7 | #ebebeb | #0d2a1c | #5f6b64 | #f6e016* | #0d2a1c | #0d2a1c |
| teal-on-paper | P | clinic,law,wellness,fintech,saas | #f7f9f8 | #ebf0ee | #06525c | #2f737d | #06525c | #f7f9f8 | #06525c |
| oxblood-grey | P | resto,law,agency,retail,studio | #e2e2e2 | #d6d6d6 | #7d0c14 | #7a4a4e | #7d0c14 | #e2e2e2 | #7d0c14 |
| olive-on-bone | P | resto,hotel,wellness,bakery,studio,law | #ecebe7 | #e1dfd9 | #38450f | #5d683a | #38450f | #ecebe7 | #38450f |
| card-pastels | P | agency,saas,product,studio | #f7f7f5 | #ebebe8 | #000000 | #666662 | #d0106f | #ffffff | #000000 |
| mint-pink-accounts | P | fintech,saas,bakery,retail | #f8f8f8 | #eeeeee | #0a0a0a | #6a6a6a | #1f6b3a | #ffffff | #0a0a0a |
| warm-paper-walnut | P | cafe,bakery,hotel,wedding,studio,retail | #f7f3ec | #ede6da | #2a2622 | #665e55 | #7a4a2c | #ffffff | #2a2622 |
| flour-terracotta | P | bakery,cafe,resto,retail,workshop | #f6f1e7 | #ebe3d4 | #2b241e | #6a5f54 | #a3472b | #ffffff | #2b241e |
| linen-harbour | P | hotel,wellness,wedding,cafe,clinic | #f3f1ec | #e6e2d9 | #1d2530 | #5a6270 | #2c5877 | #ffffff | #1d2530 |
| limewash-ochre | P | bakery,hotel,studio,workshop,resto | #f4efe5 | #e7dfcf | #29251e | #675e51 | #8c6418 | #ffffff | #29251e |
| clay-pine | P | wellness,hotel,cafe,workshop,studio | #efe9e0 | #e2d9cb | #22261f | #5f6158 | #2e5b44 | #ffffff | #22261f |
| chalk-tomato | P | resto,cafe,retail,bakery | #f8f5ef | #ede7dc | #1f1b18 | #645c54 | #b8381f | #ffffff | #1f1b18 |
| plaster-rose | P | wedding,retail,bakery,wellness,studio | #f6f0eb | #ebe0d9 | #2c2325 | #6a5d5e | #9a3c4c | #ffffff | #2c2325 |
| steel-orange | P | workshop,hardware,retail,product | #eeeeeb | #e1e1dd | #1c1d1f | #5c5e62 | #c2501b | #ffffff | #1c1d1f |
| blush-aubergine | | wedding,studio,agency,product | #edccbe | #e8c2b2 | #351a2b | #6c4857 | #351a2b | #edccbe | #351a2b |
| cobalt-on-sand | | bakery,cafe,studio,retail,agency,workshop | #e6dbc7 | #ddd0b9 | #062b8a | #3a5aa8 | #062b8a | #e6dbc7 | #062b8a |
| swap-pastels | | agency,studio,product,ai | #c3d7e5 | #d0e0eb | #141414 | #4a5068 | #0f5257 | #ffffff | #141414 |
| blush-sage-slate | | studio,wedding,agency,product | #a6b3bc | #b2bdc5 | #1c1c1c | #38424a | #2c3620 | #e3e2da | #2c3620 |
| sky-sand | | product,agency,studio | #cbe4f2 | #bcdcee | #0a0a0a | #44606e | #b0298f | #ffffff | #0a0a0a |
| studio-grey | | hardware,product | #a8a8aa | #b4b4b6 | #0e0e0e | #3a3a3c | #1f5c49 | #ffffff | #1b1b1c |

Dark grounds (tech only: software, gaming, hardware; band = the light field):

| id | trades | bg | surface | ink | muted | accent | accentInk | band |
|---|---|---|---|---|---|---|---|---|
| lunar-grey-bronze | hardware,ai,fintech,game | #111111 | #1c1c1c | #f2f2f2 | #8f8f8f | #a08672 | #111111 | #f2f2f2 |
| blue-black-cornflower | ai,devtool,saas,fintech,game | #0a0d14 | #131824 | #e9edf3 | #8891a1 | #5ab9fc | #0a0d14 | #e9edf3 |
| green-black-bone | ai,devtool,hardware,game | #161c1a | #1f2725 | #e3e2da | #8e948d | #9fc7b8 | #161c1a | #e3e2da |
| black-crimson | game,hardware | #0a0a0a | #151515 | #efefef | #8c8c8c | #b11226* | #ffffff | #efefef |
| forest-pale-lime | game,devtool,ai | #0f1f10 | #162a17 | #ffffff | #9aa39a | #c6f19d | #0f1f10 | #ffffff |
| charcoal-room-sage | hardware,ai,fintech | #1c1c1c | #262626 | #f1f1ef | #9a9b96 | #8fa68e | #1c1c1c | #c8c5ba |
| warm-black-gold | game,hardware,fintech | #0c0a08 | #181512 | #ffffff | #8f8a84 | #c9a45a | #0c0a08 | #ffffff |
| graphite-safety-orange | hardware,devtool,game | #18191b | #222427 | #eff0f1 | #9c9ea3 | #ff4a1f | #18191b | #eff0f1 |
| soot-pale-lime | devtool,ai,saas,game | #0f0f0f | #1a1a1a | #ffffff | #8f8f8f | #dafa9c | #0f0f0f | #ffffff |
| plum-black-pink | game,ai | #0d060b | #1b0e17 | #ffffff | #9a8f97 | #ff488b | #0d060b | #ffffff |
| candy-on-black | game,ai | #000000 | #0d0d0d | #ffffff | #999999 | #93f1f7 | #000000 | #9ae187 |
| walnut-tan | hardware,game | #2b2216 | #382d1e | #e4e2da | #a39a8b | #bda38b | #2b2216 | #bda38b |

Taste's library also holds loud poster palettes (saturated single-colour grounds, multi-colour "flat-loud" sets). Taste never deals them for a company site: do not use them.

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
