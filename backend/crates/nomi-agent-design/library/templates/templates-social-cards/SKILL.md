---
name: templates-social-cards
description: "Templates for social carousels and single social cards (quote cards, mock posts, now-playing cards, paid ads, app-store screenshots): sizes, slide logic, type and colour, and how to draw each as artboards. Read before any Instagram, X, WhatsApp or ad piece."
license: "Apache-2.0, Copyright nexu-io open-design contributors; Apache-2.0, nexu-io/html-anything contributors; MIT, Copyright (c) 2026 lewis (lewislulu/html-ppt-skill); MIT, coreyhaines31/marketingskills (ad-creative, idea only); screenshots-marketing described from open-design's own catalogue entry only (upstream Shpigford/screenshots, licence unconfirmed, nothing copied)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/social-carousel; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/card-twitter; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/card-xiaohongshu; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/social-x-post-card; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/social-reddit-card; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/social-spotify-card; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/ad-creative; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills/screenshots-marketing; https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/html-ppt-xhs-pastel-card; https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/html-ppt-xhs-white-editorial; upstreams https://github.com/nexu-io/html-anything, https://github.com/lewislulu/html-ppt-skill, https://github.com/coreyhaines31/marketingskills"
---
# Social Cards and Carousels

How you design swipe carousels and single social cards for a small business: the sizes, the job of each slide, the type and colour logic of each template family, and how to draw it on the canvas within the element budget.

## Ground rules for every social piece

These pieces follow their own template logic, not Taste's one-section-per-screen website rules. What still holds: tokens for every colour, space and radius; real copy in the person's language (usually Indonesian); a fact sheet first, nothing invented.

- **No invented proof.** No follower counts, likes, views, reviews, ratings, "terjual 10rb+", client logos or testimonials the person didn't give you. If a template has a metrics row, draw the icons without numbers or ask for the real ones.
- **Handles.** A mock post shows the person's own handle (`@kopi.senja.jogja`) or an obvious stand-in (`@namatokokamu`). Never a real stranger's or a celebrity's.
- **Platform chrome is drawn generically.** Recreate the layout (avatar, name, actions row) with your own `path` icons; don't draw a platform's logo as if the post were official. A small platform glyph is fine on a quote card that says "ikuti kami di …".
- **Phone-read type.** These are read on a phone at about a third of their pixel size. At 1080 wide: body never under 36 px, labels 28 px, headlines 72–140 px. One idea per card.
- **Photos** are placeholder frames at their final size with a caption inside: `Photo · es kopi susu gula aren di meja kayu, cahaya pagi dari kiri`. Gradients, grain, blobs and textures from the templates become one solid token colour; note the intent for Koda if it matters.
- **No motion.** A source "loop" or animated bar becomes a still plus a note (`Note · versi video: bar progres bergerak`).

## Sizes

| Piece | Artboard |
|---|---|
| Instagram feed post or carousel slide (default) | 1080 x 1350 (4:5) |
| Square post | 1080 x 1080 |
| Story, reel cover, WhatsApp status | 1080 x 1920 (keep text 250 px clear of top and bottom) |
| X or LinkedIn landscape card, quote card | 1200 x 675 |
| Now-playing strip | 1200 x 400 |
| App store screenshot (phone) | 1290 x 2796 iOS, 1080 x 1920 Android |

## Drawing on the canvas

- Each slide or card is its own top-level artboard: `name` like `Carousel · 03 Cara pesan`, its fixed `width` and `height`, its own `fill` and `theme`, placed in a row 120 apart (x = 0, 1200, 2400 … for 1080-wide slides).
- Repeated parts are reusable frames placed with `ref`: the handle and logo strip, the page counter, page dots, the "geser →" hint, the CTA pill. Keep them on a small `Kit · Social` artboard so one edit updates every slide.
- **Budget.** At most 300 elements in the preview. An 8-slide carousel at 25–30 nodes a slide is 200–240; the kit artboard takes the rest. Use 5–7 slides when content allows. A single card: 15–40 nodes.

## 1. Carousel (series of slides)

**For:** tips, a menu launch, how-to-order steps, a before/after story, a promo explained (Promo Ramadan, open PO batik, diskon 11.11).

**Structure:**
- **Cover:** one huge headline (2–3 lines), one subline, and a hook tag in a pill ("Simpan dulu", "Baru!", "PO dibuka"). It must work alone in the feed.
- **Body slides:** one point each: a big number or step ("02"), a short heading, two to four short lines or one example, optionally one photo placeholder. Never two points on one slide.
- **Close:** a summary line and one action: "Pesan lewat WhatsApp di bio", "Klik link di bio untuk PO", "Simpan & bagikan ke temanmu". No invented phone number; point to the bio or a stand-in.
- **On every slide:** the handle strip (logo mark + handle) and a counter (`03 / 07`) or page dots, same spot every time; a "geser →" hint on the cover only.
- A strong series lets the slide headlines read as one sentence when stacked ("Pilih rasa." → "Pilih ukuran." → "Kami antar."). Use it when the content allows; don't force it.

**Type and colour:** one display face for headlines, one sans for body, counters in mono or the body face's tabular figures. Three looks the sources offer:

| Look | Ground | Signature |
|---|---|---|
| Cinematic | dark token, full-bleed photo placeholder per slide | white headline bottom-left with one italic accent word, small mono counter top-left; each slide a distinct dominant hue |
| Pastel card | warm cream page; each slide a different soft solid tint (peach, mint, sky, lilac, lemon) | serif italic accent words, italic numerals 01–04, big rounded 28–32 radius |
| White editorial | white ground | very large headline with slight negative tracking, one solid-dark "focus" box holding the key line, small pill tags with a dot |

Pastel and editorial are adapted from lewis's html-ppt-skill decks; the pastel look also suits the many-card knowledge posts common on Chinese platforms.

**Frames:** 1080 x 1350, `layout: "vertical"`, padding 80/64, `justifyContent: "space-between"` so the handle strip sits top and the counter bottom. Headline text `textGrowth: "fixed-width"`, width `fill_container`.

## 2. Quote or insight card

**For:** one sentence worth sharing: an owner's principle, a customer's real words (with permission), a single real figure ("Sejak 2009 di Jl. Kaliurang").

**Structure:** a small type tag top-left (`Tips`, `Data`, `Kata pelanggan`); the quote centred, 2–3 lines, semibold, large; below it attribution: an ellipse avatar placeholder, name and handle; the brand mark small bottom-right.

**Type and colour:** dark or light ground chosen by the mood of the words. The source's grid or dot texture becomes plain ground, or at most three hairline rules. Quote at about 64 px on 1200 x 675, 80 px on 1080 x 1350.

**Budget:** about 12–18 nodes.

## 3. Mock platform posts

Use only when the person asks for "a post that looks like X/Twitter" (for a reel overlay or a story share) and the words are theirs.

- **Text post (X style):** rounded card (radius 16, hairline border, padding 16) centred on a flat ground. Top row: 48 px avatar, display name, handle, optional verified tick only if the account really is verified, time in small grey. Body 20–22 px with links and hashtags in the accent colour. Optional quoted card or one 16:9 photo placeholder. Actions row of four icons (reply, repost, like, share). Dark (near-black ground) suits video overlays; light suits feeds. Draw on 1200 x 675 or 1080 x 1080.
- **Forum post (Reddit style):** a narrow vote rail on the left (up arrow, count, down arrow), then community name, poster and age; a 22–28 px title; body or one image placeholder; an action row (comments, share). Rare for Indonesian businesses; keep it for community stories.
- **Now-playing card (music style):** for a musician, podcaster or a cafe playlist. Square cover placeholder left (or a drawn monogram); right: a small "SEDANG DIPUTAR" label in the accent colour, title 24–28 px bold, artist below at reduced opacity, a 4 px progress bar (two rectangles: track and fill), times at both ends, then previous / play / next icons. 1200 x 675 with the card floating, or a 1200 x 400 strip.

Metrics, counts and timestamps are the person's real ones or left out. Budget: 20–35 nodes each.

## 4. Paid social ad

**For:** Instagram/Facebook or TikTok ads, marketplace ads. From the ad-creative idea: an ad is a set of words as much as a picture.

**Structure of the visual:** one message, one product or place photo placeholder taking half or more of the frame, the offer in large type ("Diskon 30% · 11–13 Nov"), and one CTA pill ("Pesan sekarang"). Logo small. Keep text short: the platform crops and the primary text carries the rest.

**Deliver variants, not one:** draw 3 artboards on different angles (the price, the problem it solves, the occasion like Lebaran hampers) in the same layout, so only the words and photo change. Reuse one frame with `ref` and override text with `descendants`.

**Write the copy beside it** in the handoff, not on canvas: for each variant a headline (≤ 40 characters), primary text (≤ 125 characters before the "more" cut) and a short description. Claims only from the fact sheet.

**Sizes:** 1080 x 1350 feed, 1080 x 1080 square, 1080 x 1920 story. Budget: 15–25 nodes each.

## 5. App-store and marketing screenshots

**For:** an app's store listing, a landing-page hero shot, a "what's new" visual.

**Structure:** a slide of the person's real app: a short benefit headline on top (6 words or fewer: "Catat pesanan dalam 10 detik" only if that is true), then a phone body frame holding the real screen, optionally cropped off the bottom edge. A set is 4–6 slides, each a different screen, one headline style, one ground colour (or alternating two tokens).

**Frames:** don't redraw the app: place the screen's components with `ref` from the app design, inside a phone frame (radius 56–64, dark stroke as the bezel, a small notch pill). This keeps each slide around 20–30 nodes. Status bar shows a neutral time and full signal, no notifications.

## A note on Chinese platforms

Xiaohongshu is the Instagram-carousel equivalent: 3:4 cards (1080 x 1440), cover with a giant title and hook tag, one point per card, a summary-and-follow close, author watermark bottom-right. Treat it as a carousel at 3:4.

## Before you save

- Each card or slide readable alone at phone size; one idea each.
- Every colour, space and radius a token; no gradients, no textures.
- Handle strip, counter and dots are `ref`s in the same place on every slide.
- No invented numbers, reviews, followers, verified ticks or logos; handles real or clearly stand-in.
- Element count under 300 across the preview.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus nexu-io/html-anything (Apache-2.0), lewislulu/html-ppt-skill (MIT, (c) 2026 lewis) and coreyhaines31/marketingskills (MIT). Changed: rewritten for the OpenPencil canvas.
