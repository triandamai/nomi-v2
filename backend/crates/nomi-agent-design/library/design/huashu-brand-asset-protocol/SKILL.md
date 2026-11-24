---
name: huashu-brand-asset-protocol
description: "The five-step core asset protocol for any design involving a specific brand or recognisable product: ask, search official sources, collect, verify, freeze a brand spec, with the 5-10-2-8 quality bar and fallbacks. Read it before designing for or showing any named brand."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Core asset protocol

The five-step core asset protocol for any design that involves a specific brand or a recognisable product: ask, search official sources, collect, verify, freeze a brand spec, with the 5-10-2-8 quality bar and what to do when assets are missing.

Following it or not decides whether a design scores 40 or 90. It extends Taste rule 7 (real full-colour icons for real products, never a drawn look-alike). If `ux/brand-extract` is present, use it for pulling tokens out of a site; this file is about whether you have the **assets** at all.

## When it applies

Two triggers; the second is the one people forget:

1. You design **for** a brand: the person's own business, a client, a named company.
2. The design **shows** any recognisable product or brand: a comparison, a "bayar pakai" row (GoPay, OVO, DANA, QRIS), a "pesan lewat" row (GoFood, GrabFood, ShopeeFood), a marketplace link (Tokopedia, Shopee), a list of tools. **As many products as appear, that many official logos.** Not "use one if handy".

Before running it, the facts must be confirmed (`huashu-workflow` §0): the brand exists, the product is released, the version is right.

## Assets beat specs

A brand is recognised through its assets, in this order:

| Asset | Recognition | Required |
|---|---|---|
| Logo | Highest: a logo identifies at a glance | **Every brand** |
| Product photos or renders | Very high: for a physical product, the product is the hero | **Every physical product** (food, batik, skincare, hardware, packaging) |
| UI screenshots | Very high: for a digital product, the interface is the hero | **Every app, site or SaaS** |
| Colour values | Medium: alone they often collide with other brands | Supporting |
| Fonts | Low without the above | Supporting |
| Character keywords | Low: for your own self-check | Supporting |

Breaking the protocol looks like: extracting only colours and fonts without the logo, product or UI; drawing a silhouette of the product instead of the product ("it becomes a generic tech animation that could be any brand"); missing an asset and neither telling the person nor generating from a reference. Stop and ask rather than fill with generic material.

## The five steps

**1. Ask, with a checklist.** "Ada panduan merek?" is too vague; the person won't know what to send. Ask once, by priority:

> Untuk usaha Anda, mana yang sudah ada? Kirim yang ada, sisanya saya cari:
> 1. Logo (file asli, atau foto yang jelas)
> 2. Foto produk (untuk produk fisik)
> 3. Tangkapan layar aplikasi atau toko online (untuk produk digital)
> 4. Warna merek (kode warna, atau benda yang warnanya Anda anggap "warna kami")
> 5. Huruf yang biasa dipakai
> 6. Contoh materi lama: spanduk, menu, kemasan, Instagram

**2. Search official sources, by asset.** Logo: the brand's press or brand page, the header of its site, its official social avatars; for well-known digital products, open icon collections that carry official marks. Product photos: the official product page, press kit, frames from the official launch video, the official marketplace listing. UI: app store screenshots (check they are real UI, not mockups), the product site, official demo videos, the person's own screenshots. Colours: the site's styles, the brand guide, the logo file. For a small business with no web presence the sources are the person: photos of the shopfront, packaging, menu board, Instagram feed.

**3. Collect, with the 5-10-2-8 bar.** The logo is exempt: if it exists, use it, even a mediocre one; a 6/10 logo beats no logo tenfold. Every other asset (product photos, UI, reference images) passes **five rounds of searching, ten candidates, two chosen, each scoring 8/10 or more**. Fewer is better than padding: a 7/10 asset is a deduction, worse than an honest placeholder. Score each on: resolution (2000px or more; 3000 for print or big screens); rights clarity (official over public domain over free stock; suspected stolen scores zero); fit with the brand's character keywords; consistency of light, composition and style with the other chosen asset; ability to carry a narrative role on its own, not decoration. The hero asset should be 9 or 10.

**4. Verify and extract.**

| Asset | Check |
|---|---|
| Logo | The file opens and is real (not an empty page saved as an image); a version for light and for dark grounds; clean background |
| Product | At least one image 2000px or larger, clean background, several angles (hero, detail, in use) |
| UI | Real resolution, the current version, no stranger's personal data visible |
| Colours | Count the most frequent hex values in the real assets, ignoring black, white and grey |

Two traps. **Demo-brand pollution:** a tool's screenshot often shows a customer's colours (a POS demo full of a café's red); that red is not the tool's. When two strong colours appear, find out which belongs to whom. **Brand faces:** a brand's marketing site and its product UI often use different palettes; both are real. Pick the face that fits the deliverable.

**5. Freeze a brand spec.** Write it in your notes (or a canvas side note) so page three doesn't drift to "close but not quite" the brand colour:

- Collected on, sources, completeness (full / partial / inferred)
- Logo: main version, version for dark ground, where it may appear, forbidden changes (no stretching, recolouring, outlines)
- Product photos: hero, details, in use, how they'll be used
- UI screenshots: home, key feature screens
- Palette: primary, ground, ink, accent, each with its source; colours the brand never uses
- Type: display, body, mono if data appears
- Signature details: what gets the 120% treatment
- No-go zone: what must never happen
- Character: three to five adjectives

After the spec exists, every frame uses its values and its assets by reference, never a hand-drawn copy. Adding a new colour means editing the spec first: brand consistency by structure, not by memory.

## On Nomi's canvas

- The canvas holds no images. A logo you have as a vector becomes a `path` node with its real geometry (`pen-format`, "Icons and marks"); never redraw a logo by eye. A logo you only have as a photo becomes a placeholder frame at its real size with the note `Logo · file asli dari pemilik`, and you tell the person you need the file.
- Product photos and UI screenshots become sized placeholders with a caption naming the exact asset (`Foto · Batik Larasati motif parang, flat lay, dari katalog pemilik`) so Koda places the real file. In decks and videos the real files are used directly.
- Brand colours become `$color-*` tokens on the design system frame; the brand spec is their source.

## When the protocol fails

| Missing | Do |
|---|---|
| Logo entirely | **Stop and ask.** The logo is the root of recognition |
| Product photos (physical) | Generate from an official reference image as the base (`huashu-image-briefs`), or ask the person, or as a last resort an honest placeholder clearly labelled "foto produk menyusul" |
| UI screenshots (digital) | Ask for screenshots from the person's own account, or frames from an official demo video; never a mockup generator |
| Colours entirely | Run the three directions with colours sampled from the subject and its real photos, marked as assumptions |

Never fill a missing asset silently with a silhouette or a generic gradient. That is the protocol's worst failure.

## Real failures it prevents

- An animation for a chat AI guessed "orange" from memory; the brand is blue. Redone.
- A design tool's spec took the bubble-tea red shown in its demo screenshot as the tool's own colour.
- A camera launch animation used only extracted colours and a drawn silhouette instead of the logo and real product shots. Result: a generic black-and-orange tech animation with no brand in it. The author's words: "Otherwise, what are we even expressing?"
- A deck comparing five coding tools fetched five brand colours and zero logos, because the comparison wasn't recognised as trigger 2.
- Colours extracted but not written down; by page three a "close enough" hex crept in.

The protocol costs about 30 minutes (logo 5, three to five product or UI images 10, colours 5, the spec 10). Skipping it costs one to two hours of rework, or a redo.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
