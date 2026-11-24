---
name: craft-style-minimalist
description: "A style card for Warm Editorial Minimalism (warm monochrome, serif headlines, 1px hairlines, pale pastel tags, flat surfaces). Read it when you offer directions to a calm service, studio, shop or small software brand."
license: "MIT, Copyright (c) 2026 Leonxlnx (Taste Skill family: minimalist-skill), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (minimalist-ui, as vendored in open-design@5fdcc3d, 2026)"
---
# Style: Warm Editorial Minimalism

A document-like, nearly colourless direction: warm off-white paper, off-black ink, an editorial serif for headlines, hairline structure, flat cards and four washed-out pastels used only as small tags, so the page feels calm, exact and expensive without decoration.

## The look

The page reads like a well-set workspace document. Hierarchy comes from type contrast (a large, tightly tracked serif headline over a quiet sans), generous space and 1px hairlines, never from shadows or colour. Colour is scarce: the ink, two greys, and a pale pastel chip where a status or category needs one. Cards are flat white on warm paper with a single hairline. The one dark element is the primary button.

## Suits / avoid

- **Suits:** consultants, notaries, accountants, tutors, architects and interior studios, bookshops, design-object shops, small software tools (notes, invoicing, booking), course pages.
- **Avoid for:** street food, nightlife, kids' brands, anything that must feel loud, festive or cheap-and-cheerful; a warung page in this style looks cold.
- **Taste kinds:** works on service, product and software. On a place, keep the warmth (paper and photos) and drop the pastel chips (a place carries no badges).

## Tokens

| Token | Light | Dark | Role |
|---|---|---|---|
| `$color-background` | #F7F6F3 | #161615 | Warm paper |
| `$color-foreground` | #1F1E1C | #EDEBE6 | Ink (never pure black) |
| `$color-card` / `-foreground` | #FFFFFF / #1F1E1C | #1F1E1C / #EDEBE6 | Flat cards |
| `$color-muted` | #EFEDE8 | #262523 | Soft bands, kbd keys |
| `$color-muted-foreground` | #6B6964 | #A6A39C | Secondary text (4.5:1 on paper) |
| `$color-primary` / `-foreground` | #1F1E1C / #FFFFFF | #EDEBE6 / #161615 | The one filled button |
| `$color-border`, `$color-input` | #E6E4DF | #FFFFFF1A | Every hairline |
| `$color-ring` | #1F6C9F | #7FB6DA | Focus |
| `$color-tag-red` / `-foreground` | #FDEBEC / #9F2F2D | #3A1F1F / #F2B8B6 | Status chip only |
| `$color-tag-blue` / `-foreground` | #E1F3FE / #1F6C9F | #16293A / #A9D4F2 | Status chip only |
| `$color-tag-green` / `-foreground` | #EDF3EC / #346538 | #1C2A1D / #B4D3B5 | Status chip only |
| `$color-tag-yellow` / `-foreground` | #FBF3DB / #7A5200 | #332A12 / #E8CF85 | Status chip only |

The yellow chip text is darkened from the source's #956400 so it passes 4.5:1. Never use a pastel as a section ground or a button.

Radius: `$radius-sm` 4, `$radius-md` 6 (buttons, inputs), `$radius-lg` 8, `$radius-xl` 12 (cards, photos). No pill on a card, a container or a primary button; pills only on tags.

## Type

- Headline serif from Taste's table: Instrument Serif or Newsreader (with Hanken Grotesk) on service and product; Source Serif 4 with Libre Franklin for law and finance. On software, Taste prefers a sans: use Geist or Instrument Sans for headlines and keep the serif only for a quote.
- Headlines 400, tracking about -1.5% (serif) to -3% (sans), line height 1.1. Body 16/26 at about 62 characters, in the ink, never pure black.
- Mono (Geist Mono) only on a software page, for keys, shortcuts and IDs.
- **Taste wins:** the source asks for tracked uppercase tags; Taste does not allow tracked capitals. Tags are sentence case, 13/500, no tracking.

## Layout signatures

1. **Macro whitespace:** section padding 128 at 1440 (96 at 768, 64 at 390); content column 720-880 wide for reading sections, the full 12-column grid for cards.
2. **Hero:** serif h1 on two lines at most, one quiet line under it, one filled ink button and one text link. A real photo or the real product to the right, top-aligned (Taste's split, never centred text over an image).
3. **Hairline cards:** white, 1px `$color-border`, radius 12, padding 32-40, no shadow. A card row is one height and fills the row.
4. **Ruled lists instead of boxes:** FAQ, price lists and services as rows divided by a bottom hairline, a plain `+` icon at the right. No accordion boxes.
5. **Keys and windows (software only):** a shortcut drawn as a small key (`$color-muted` fill, hairline, radius 4, mono 13); an app screenshot in a plain window frame with a white top bar and three small grey dots.
6. **Footer:** ruled, two or three columns of plain links, the legal line at 13.

## Components

- Button primary: 44 tall (48 on a place), ink fill, radius 6, label 15/500, no shadow. Hover note: background steps to #333333, press scale .98.
- Button secondary: a text link with an underline that draws on hover; the source's "no ghost button" rule matches Taste's one-pair rule.
- Tag: 24 tall pill, pastel fill, 13/500 sentence case, used for status (Lunas, Menunggu, Baru) in app screens and price tables, never as an eyebrow above a heading.
- Icons: one outline set at one stroke width (1.75 at 20px), drawn as `path`. The source bans Lucide-style thin icons; Taste only asks for one set and one width, so choose a slightly heavier set (Phosphor regular) and keep it consistent.

## What the canvas draws instead

| Source effect | On the canvas | Note for Koda |
|---|---|---|
| Warm grain over photos (4%) | Nothing: Taste forbids grain on photos | Do not add |
| Radial light spot or slow blob behind the hero | Flat paper | Do not add on product, service or place; on software one effect may be dealt (`taste-photos-motion`) |
| Navbar blur | Solid paper nav with a bottom hairline | Solid is fine |
| Ultra-diffuse card shadow (under 5%) | Optional `effect` shadow #0000000A, y 2, blur 8 | Only on hover lift |
| Scroll fade-ins and staggered reveals | Nothing | Taste forbids scroll fade-ins; the hero entrance and one moment carry motion |

## Copy voice

Plain, exact, unhurried. Facts where adjectives want to go: "Laporan pajak UMKM selesai dalam 5 hari kerja" rather than "solusi pajak modern". No "elevate", "seamless", "solusi terbaik".

## Tells

- Warm paper with off-black ink and almost no colour.
- A serif headline set tight, a calm sans for everything else.
- Hairlines do all the structural work; nothing floats on a shadow.
- Colour appears only as a small pastel chip carrying a real status.

Adapted from Taste Skill's minimalist-ui (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
