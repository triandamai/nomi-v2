---
name: systems-index
description: "What the design-systems collection is (182 packages from open-design, VoltAgent and bergside), how you learn from it without copying a brand, and where each package lives; read it before seeding a direction from a system."
license: "Apache-2.0 (open-design, Copyright (c) 2026 nexu-io); MIT, Copyright (c) 2026 VoltAgent; MIT, Copyright (c) 2026 Bergside; MIT, Copyright (c) 2026 Tw93; MIT, Copyright (c) 2026 Tom-Opencart; MIT, Copyright (c) 2026 Cloudflare, Inc."
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/design-systems; https://github.com/VoltAgent/awesome-design-md (fcc7028, 28 Oct 2026); https://github.com/bergside/awesome-design-skills (f631a09, 28 Jun 2026)"
---
# Systems Index

What the design-systems collection is (182 packages from open-design, VoltAgent and bergside), how you learn from it without copying a brand, and where each package lives. Read it before seeding a direction from a system.

## What the collection is

A reference range of complete design systems, rewritten for the OpenPencil canvas. Each entry gives a system's logic (type scale, radius ladder, colour roles, component habits), its tokens mapped to your names (`$color-<shadcn role>`, `$radius-*`, `$font-*`), and a contrast check with the fixed value wherever a pair fails.

| Group | Count | What it is | Skill |
|---|---|---|---|
| Brand-inspired | 99 | Systems read from real products' public sites (payments, cafes, cars, AI tools). 85 come from open-design (70 via VoltAgent's DESIGN.md files, 15 written by open-design), 13 only from VoltAgent, and one is bergside's second reading of an existing brand. | `systems-brands` |
| Generic | 73 | Named styles (cafe, friendly, luxury, retro, brutalism…): 57 open-design copies, whose tokens.css is the truth, and 16 bergside-only skills. | `systems-generic` |
| Hand-written | 10 | Systems a person wrote: Neutral Modern, Warm Editorial, kami, Tom Modern, Atelier Zero, HUD, Mission Control, Trading Terminal, Urdu Modern, Cloudflare Kumo. | `systems-handwritten` |
| Schema | — | open-design's shared token contract, VoltAgent's DESIGN.md format, and how you write a DESIGN.md for Koda. | `systems-schema` |

## How you use it

Taste is your rulebook; these systems are a range to learn from. A system never overrides a Taste rule. Where a card and Taste disagree (an Inter face, a violet accent, a dark page for a bakery, uppercase tracked labels), Taste wins, and the card says so.

1. **Learn the logic.** Read a system for why it works: navy ink instead of black, the accent spent twice a screen, one radius ladder, a serif display over a quiet sans, a hairline instead of a shadow. That reasoning is what you carry.
2. **Seed a direction.** When you offer 2–4 directions (see `styles-index`), a system can seed one. Take its structure and character (ground, ink and accent roles, radius ladder, type scale ratio, density, component habits), then set the person's own colours from their shopfront, logo or product and pick a face from `taste-type`.
3. **Map, then check.** Put the direction into `$color-*` tokens using the card's mapping, then re-run the contrast numbers in `taste-color` with the person's colours. The cards' fixes show how: darken the accent until its label holds 4.5:1, push grey text to 4.5:1, ink buttons when the accent is under 3:1 on the ground.
4. **Name it plainly.** Call a direction by its character ("warm paper, serif, one walnut accent"), not by a source.

## Never

- Never reproduce a real brand's identity for another business: not its name, logo, mark, signature colour pairing, custom typeface or slogan. A cafe may share a structure with a coffee chain; it never shares the apron green.
- Never name a brand in the person's design, in copy, captions, frame names or the handoff. The person sees "warm cream and deep green", not a company.
- Never copy a system's photographs, illustrations or icons; describe your own placeholders (`pen-format`).
- Never carry over what the canvas can't draw (gradients, glass, blur, grain, glow) or what Taste forbids (violet, dark pages outside tech, tracked capitals, banned faces). The cards already flatten and swap these.

## Quick seeds by trade

Light, Taste-friendly systems first. These are starting points only; read the card before using one.

| Trade (Indonesian) | Seed from |
|---|---|
| Cafe, kedai kopi, bakery, toko roti | cream-walnut family (`cafe`), `warm-editorial`, `terracotta`, Starbucks card's structure, Mistral's amber grounds |
| Resto, warung | `retro` / sunny retro, `riso`, `publication` for a menu board |
| Hotel, homestay, villa | Airbnb card's photo-first listings, `premium`, `spacious`, Oura card's warm paper |
| Wellness, salon, spa, yoga | Oura card, `levels`, `sketch`, soft clay palette without effects |
| Workshop, bengkel, penjahit | `tom-modern`, BMW card's square corners, `stitch`, Cursor card's warm ink |
| Retail, toko, butik, florist | Pinterest card's image grid, DJI card's product tiles, `mono-ink` family plus one client accent |
| Clinic, dokter gigi | `default`, OpenAI card's serif on white, Webex card's teal |
| Law, notaris, konsultan pajak | `kami`, Oracle card's ink actions, IBM card's grid, `refined` |
| Wedding, photographer | `atelier-zero`, `elegant`, Pinterest card, ElevenLabs card's light serif |
| School, course, pesantren | Notion card, `urdu` for Arabic script, Duolingo card for kids |
| Agency, studio, freelancer | Figma card's black-and-white chrome, Vercel card, `tom-modern`, `brutalism` |
| App, SaaS, POS, fintech | Stripe card, `cloudflare-kumo`, `enterprise`, Linear card's luminance ladder for app screens |
| Game, hardware, AI (dark allowed) | blue-night family, `hud`, `mission-control`, Raycast and Spotify cards |

## References

| File | Holds |
|---|---|
| `references/index-brands.md` | The 99 brand-inspired packages in one table: character, best-fit trades, and the `systems-brands` reference holding each card |
| `references/index-styles.md` | The 73 generic packages and 10 hand-written systems in one table, with the reference holding each |

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (MIT, Copyright (c) 2026 VoltAgent), bergside/awesome-design-skills (MIT, Copyright (c) 2026 Bergside), tw93/kami (MIT), Tom-Opencart/tom-modern-html-style-rule (MIT), cloudflare/kumo (MIT). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
