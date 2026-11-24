---
name: templates-posters
description: "Layouts for posters and image deliverables: promo posters, editorial posters, photo-led posters, device mockups, marketplace product image sets, sticker and mascot sheets. Sizes, structure, type and colour, and how to draw each as frames. Read before any poster or product image."
license: "Apache-2.0, Copyright nexu-io open-design contributors (open-design; html-anything poster and mockup templates, nexu-io/html-anything); MIT, MiniMax-AI/skills (gif-sticker-maker, idea only); hatch-pet (vendored from openai/skills, licence unconfirmed) and image-poster prompt order described in our own words only"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/image-poster; https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/magazine-poster; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (poster-hero, mockup-device-3d, gif-sticker-maker, hatch-pet, ecommerce-image-workflow); https://github.com/nexu-io/html-anything"
---
# Posters and image deliverables

How you lay out a single-image piece (a poster, a product image, a mockup, a sticker sheet) so it reads in two seconds, with the picture drawn as a placeholder and briefed precisely.

These are marketing pieces: they may follow their own logic rather than Taste's website rules. They still use tokens, the person's real offer, prices and dates, their language, and no invented reviews, follower counts or logos. You can't make the photo or illustration yourself: you draw its frame at its final size and write the note (see `../../images/images-visual-briefs/SKILL.md`). Social carousels and post cards are in `../templates-social-cards/SKILL.md`.

## Sizes

| Piece | Frame | Notes |
|---|---|---|
| Story / WhatsApp status / vertical poster | 1080x1920 | Keep text out of the top 220 and bottom 280 (app chrome) |
| Instagram feed poster | 1080x1350 | 4:5 |
| Printed poster, flyer (A4/A3/A2 ratio) | 1240x1754 | Add a 3 mm bleed note for the printer; keep text 60 px inside |
| Banner, screen, event backdrop | 1920x1080 | |
| Marketplace main image (Shopee, Tokopedia) | 1000x1000 | Square, product-first |
| Marketplace feature and lifestyle images | 1080x1350 | 4:5 |
| Sticker | 512x512 each | Drawn on a sheet frame |

## 1. Promo poster (vertical)

**For:** a promo, an event, a new menu, an opening, "Open PO", a Ramadan or 11.11 sale.

**Structure, top to bottom:**
1. **Picture zone, top ~35-45%:** the product or place as a photo placeholder, or one bold shape if there is no photo. Never a big emoji.
2. **Headline, the visual centre:** the offer in 4-7 words at 120-160 px, heavy weight ("Diskon 20% semua kue kering").
3. **One-line subhead:** who, what, until when ("Khusus pesanan sampai 30 Maret").
4. **Key points:** 3-5 short lines with small `path` icons (date, place, price, how to order).
5. **Footer strip:** brand mark left, how to order right (WhatsApp as the person gave it, a QR placeholder square with a note `QR · link pemesanan`).

**Type and colour:** one heavy display face for the headline, one sans for the rest. Bold colour is fine on a poster: one brand colour as the ground or a big block, text in white or ink, one contrast colour for the price or date only. Check contrast at a glance from 2 metres.

**Taste override:** the template's mesh gradients, violet-fuchsia blends and floating decorations become one flat brand colour and the real product photo. No fake "Best seller" badges unless true.

## 2. Editorial poster (type-led)

**For:** an opinion, a manifesto, a workshop recap, tips from a craftsperson ("6 cara merawat kain batik").

**Structure:**
1. A hairline top rule with a dateline: issue or brand left, date right, small mono.
2. A one-word mono eyebrow ("BARU TERBIT").
3. A 2-3 line headline in an oversized serif; at most one word struck through and one word in italic accent colour.
4. A 1-2 sentence italic deck at about 60% of the headline size.
5. A short accent bar (~80 px).
6. A body grid of six numbered cells (2x3): mono eyebrow `01 · RENDAM AIR DINGIN`, bold serif subhead, 2-3 sentences, one tinted callout line.
7. A footer band: handle, role, date, and a small "tip" plate.

**Type and colour:** heaviest serif for the headline, body serif for cells, mono for eyebrows and annotations. Cream paper (`$color-background` around #F3EEE2), ink, one accent. Paper grain is a build note, not drawn.

**Frame:** 1240x1754 for print or 1080x1350 for social. Each cell is a reusable component placed with `ref` (about 6 nodes each).

## 3. Photo-led poster

**For:** when the picture is the message: a dish, a room, a product hero.

The frame is mostly one placeholder at full bleed with a small type block in the calmest corner. Write the note in this order, because it is the order an image model or photographer needs:
1. **Subject and composition:** what is in frame, where, how big, the crop and the eye line.
2. **Light and mood:** daylight or studio, warm or cool, time of day.
3. **Palette and texture:** the brand's colours as anchors, or a three-word mood ("ochre, ink, linen").
4. **Camera or lens:** only for realism ("85mm, shallow depth of field").
5. **What to avoid:** warped text, extra fingers, fake logos.

Example note: `Photo · es kopi susu gula aren di meja kayu, gelas berembun, cahaya pagi dari kiri, latar hijau tanaman blur, ruang kosong di kanan atas untuk judul`.

## 4. Device mockup and product showcase

**For:** an app launch, a website reveal, "now on the App Store", a design portfolio.

**Structure (1920x1080):** a phone and optionally a laptop, centred or phone-left/laptop-right; a small caption row at the top (product name, version, date, muted); a product lockup bottom right (logo, one-line tagline, hairline).

**Drawing it:** draw devices flat and front-on. Phone: a rounded frame (radius ~56 at 390-wide screen scale, a 4 px solid bezel in a neutral like #A8A8AD); laptop: a screen frame plus a thin base rectangle, no keys. The screen holds real UI drawn at true size (status bar, title, content, tab bar) with the person's real data, or a `ref` to a screen you already drew. Background: one solid from a quiet set (charcoal, pearl, midnight blue, mocha), never a rainbow. 3D tilt, turntable spin, glass highlights and floor reflection are build notes (`Build note: phone rotated -12° on Y, slow turntable, soft floor reflection`).

## 5. Marketplace product image set

**For:** a shop selling on Shopee, Tokopedia, TikTok Shop or its own site.

| Slot | Frame | Goal |
|---|---|---|
| **Main** | 1000x1000 | The product alone, whole, centred, on white or a soft neutral, filling ~80% of the frame. No text |
| **Feature** | 1080x1350 | One selling point: a close-up detail plus room for one large callout ("Jahitan tangan"), never tiny text |
| **Lifestyle** | 1080x1350 | The product in real use (on a table, worn, in a kitchen), still faithful to the real thing |
| Optional: size chart, what's in the box, variants | 1080x1350 | A clear table or a row of variants |

**Rules:** the product must come from the person's real photos; you never design a product that doesn't exist. Before writing notes, write an identity lock from the person's reference: category and shape, colours and materials, label and pattern, scale, and what must not change. Repeat that lock in every slot's note. One product per set. Draw each slot as its own frame with the placeholder and, for the feature slot, the callout text as real text over a calm area.

## 6. Sticker sheets and mascots

**For:** a brand mascot, WhatsApp sticker pack, a character for a kids' class or a kopi brand.

- **Sheet:** one frame holding a grid of 512x512 cells (3x3 or 4x4), each a placeholder with a pose note (`Sticker · maskot kucing oren melambai, senyum lebar`) and the sticker's word if it has one ("Makasih!", big, heavy, outlined).
- **Character style that reads small:** chunky, compact silhouette (big head, small body), thick dark outline, a few flat colours, one clear expression or pose per cell. No soft gradients, painterly detail, floating sparkles or drop shadows: anything detached from the body breaks when cut out.
- **Consistency:** write a character lock once (shape, colours with hex, clothing, one signature detail) on a `Character` frame and repeat it in every note.
- **Animated stickers or pet sprites:** animation is a note per cell (`loop: tail wags, 1 s`). Turning a photo into a toy-style animated sticker is done by a generation tool you can't run; suggest it and give the lock.

## Before you hand over

- The headline or the picture is clearly the one centre; readable at thumbnail size.
- Every picture is a placeholder at its final ratio with a full note; no blank boxes.
- Real offer, real dates and prices; no invented badges, ratings or follower counts.
- Text clear of safe areas (story chrome, print trim).
- Effects, tilt and animation written as build notes.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus html-anything poster and device templates (nexu-io/html-anything, Apache-2.0) and the ideas of MiniMax-AI/skills gif-sticker-maker (MIT) and the vendored hatch-pet skill (described only). Changed: rewritten for the OpenPencil canvas.
