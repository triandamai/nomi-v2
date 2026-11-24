---
name: huashu-typography
description: "Huashu's type reasoning: the order of decisions, modular scales, line length and leading, ten open pairings with why they work, the overused-font list and type anti-patterns, adapted for Indonesian text. Read it when setting type for a deck, video or image; for sites taste-type decides."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Typography: a reasoning system, not a font list

Huashu's type reasoning: the order of decisions, modular scales, line length and leading, ten open pairings with why they work, the overused-font list and type anti-patterns, adapted for Indonesian text.

For sites and apps, `taste-type` (its pairings by kind and trade, its sizes at 1440 and 390) wins. Use this file for decks, video, images and social carousels, and to reason about any pairing. If the person has their own fonts, lift those first; everything here applies only when they don't.

## 0. Decision order

Derive, don't jump to "a nice font":

1. **Content type:** long reading, data-dense, marketing big type, interface. Sets the scale ratio and body size.
2. **Language mix:** Indonesian only, Indonesian with English terms, or English. Sets fallback and leading.
3. **Temperature** (quiet, neutral, bold, as in `huashu-design-styles`): sets where the pairing's contrast comes from.
4. **Only then the font name.**

Choosing the name first means the content has no effect on the type, which is exactly why everything looks alike.

## 1. Modular scale

Sizes come from the body size times a fixed ratio. The ratio sets the drama:

| Ratio | Name | Character | Use |
|---|---|---|---|
| 1.2 | Minor third | Gentle, many quiet levels | Dashboards, docs, dense UI |
| 1.25 | Major third | Safe, general | Most pages and landing pages |
| 1.333 | Perfect fourth | Headings jump out | Editorial, marketing, reports |
| 1.5 | Perfect fifth | Dramatic, few levels | Posters, slides, one-sentence heroes |

Set body at 16 to 18px (web) and step up and down. More than five levels is loss of control: cut. Display type (3 to 8 times body) leaves the scale: it is a layout element sized by how much of the frame it fills, not a text level. For slides at 1920 × 1080: body 24 minimum (28 to 36 ideal), titles 60 to 120, hero type 180 to 240, labels 16 minimum.

## 2. Line length and leading

| Text | Comfortable | Best |
|---|---|---|
| Latin body (Indonesian, English) | 45 to 75 characters | about 66 |
| Captions, side notes | Shorter | |

Indonesian runs long: affixed words ("pengirimannya", "keberlanjutan") make ragged right edges and bad breaks in narrow columns. Give narrow columns a little more width or a smaller size rather than hyphenating headlines, and check that no single short word sits alone on a headline's last line.

Leading grows with line length (a longer line needs a wider rail for the eye's return):

| Context | Leading |
|---|---|
| Display, one or two lines | 0.95 to 1.1 |
| Headings | 1.1 to 1.3 |
| Short-line body | 1.4 to 1.5 |
| Long-line body | about 1.6 |

## 3. Ten open pairings

Three sources of contrast; know which you use: **form** (serif display with sans body; the x-heights must match or the body seems to jump in size), **same family** (a superfamily; zero risk, a bit flat), **era** (a classical form with a modern one; needs 200+ years between them, 50 looks like a mistake).

| # | Display + body | Logic | Temperature |
|---|---|---|---|
| 1 | Newsreader + Geist | Form: screen-tuned transitional serif with a tall x-height; **the proper replacement for Fraunces** | Quiet |
| 2 | Source Serif 4 + Source Sans 3 | Same family, rhythm aligned; reports never fail | Quiet |
| 3 | EB Garamond + IBM Plex Sans | Era: 16th-century old style with a 2017 grotesque; Garamond's low x-height needs about 8% more size in mixed lines | Quiet, literary |
| 4 | Lora + Hanken Grotesk | Form: brushy mid-contrast serif; Hanken is an open cousin of Söhne | Neutral |
| 5 | Instrument Serif + Geist | Form: one weight only, display only; **becoming overused by AI tools**, avoid when you want to look distinct | Neutral |
| 6 | Schibsted Grotesk + Source Serif 4 | Reversed: grotesque display, serif body, newspaper feel; **replaces Space Grotesk** | Neutral |
| 7 | Bricolage Grotesque + Newsreader | Ink traps show only large; rough display, gentle body | Bold |
| 8 | Archivo (Expanded or Black) + Inter | Poster structure: wide heavy display, Inter as the 14 to 16px body workhorse (its right job) | Bold |
| 9 | Cormorant Garamond + Work Sans | High-contrast luxury; Cormorant needs 40px or more or its hairlines break | Bold |
| 10 | Geist Mono or JetBrains Mono + Geist | Mono as the star: technical; mono for labels, numbers and code only, never paragraphs (lines grow about 30%) | Neutral, technical |

**Overused (swap for the replacement):**

| Overused | Why | Use instead |
|---|---|---|
| Fraunces as display | The default "tasteful" pick of every AI design tool 2023 to 2025 | Newsreader, Libre Caslon Text |
| Inter as display | Built for small UI text; flat and expressionless large | Archivo, Anton, Schibsted Grotesk |
| Space Grotesk | The lazy "tech" answer | Schibsted Grotesk, Familjen Grotesk |
| Playfair Display | The lazy "elegant" answer; wedding-invitation feel | Cormorant (more extreme), DM Serif Display (sturdier) |

High-contrast serif on warm cream is a look family, not a bad font: use it, just not in all three directions or every project.

## 4. Type as voice

Excellent apps and films never use the system default everywhere. Decide who is speaking first: a serif for the days of the week, lowercase rounded heavy type with a full stop, one italic serif word inside lowercase sans, mono all-caps legends. Then pair.

For motion: brand and narration in a serif (taste, publication), UI, data and code in sans and mono (function). One face for everything is wrong in a film. A variable font's weight animating from thin to bold reads more cinematic than scaling, because it feels like filling rather than pushing in.

## 5. Anti-patterns

| Anti-pattern | Why |
|---|---|
| Inter for both display and body | The number-one fingerprint of AI pages |
| Leaving body to the system default | Different face on every device: no design at all |
| Synthetic italic or bold | The renderer distorts the glyphs; use a real weight or none |
| Loose tracking on big headlines | Large type needs tightening; adding space looks like a placeholder |
| No maximum line length | On a wide screen a 120-character line loses the reader; worse than a wrong font |
| More than six size levels | Hierarchy inflates and nobody knows what matters |
| Only 400 and 700 | Hierarchy carried by size alone; 300 to 900 are free expression |
| Proportional figures in tables | Columns wobble; use tabular figures for prices (Rp 18.000) and data |
| A display face for paragraphs | Personality becomes reading friction after a few lines |

Tracking: body 0; headlines of 40px and more tightened (see `taste-type` for the numbers); all-caps Latin labels are the only place for wide positive tracking (0.08 to 0.15em), and they are template chrome on most pages anyway.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
