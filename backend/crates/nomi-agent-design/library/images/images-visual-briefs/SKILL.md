---
name: images-visual-briefs
description: "How to write the note inside a photo, illustration or video placeholder, and the fuller brief a person can paste into an image or video tool: the caption formula, the brief fields, sets, video shot lists and honesty rules."
license: "Apache-2.0, Copyright open-design contributors (nexu-io/open-design); MIT, Copyright (c) 2026 Leonxlnx (Taste Skill: imagegen-frontend-web, imagegen-frontend-mobile); prompt patterns studied, not copied, from CC-BY-4.0 YouMind-OpenLab/awesome-gpt-image-2 and YouMind-OpenLab/awesome-seedance-2-prompts (their individual authors) and Apache-2.0 heygen-com/hyperframes"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/prompt-templates; https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (imagegen-frontend-web, imagegen-frontend-mobile, ecommerce-image-workflow); https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (image-poster, video-shortform, open-design-landing/assets/imagegen-prompts.md); https://github.com/Leonxlnx/taste-skill; https://github.com/YouMind-OpenLab/awesome-gpt-image-2; https://github.com/YouMind-OpenLab/awesome-seedance-2-prompts; https://github.com/heygen-com/hyperframes"
---
# Visual briefs: placeholder notes and paste-ready briefs

How to write the note inside every photo, illustration or video placeholder, and, when the person asks, the longer brief they can give a photographer or paste into an image or video tool.

## What you can and can't do

You draw placeholders; you never make the picture. You can't call an image, video, audio or screenshot tool, and you never say a picture "is being generated". Your note is the hand-off: the person shoots it, picks a licensed photo, or pastes your brief into a tool themselves (which tools exist: `images-media-tools`). Taste's photo rules (`taste-photos-motion`) decide what a photo shows, how many go where and which ratios; this file only decides how you write it down. Where they disagree, Taste wins.

## The canvas note (every placeholder)

Draw the frame at its final size and ratio, fill it with the photo's real dominant tone, and put one note inside (layer `photo-note`, 13-14px, top-left inset 16). Write it in the person's language; keep the English prefix so Koda can find it.

**Formula:** `<Kind> · <subject doing what>, <where>, <framing>, <light>[, <calm area for text>]`

- **Kind:** `Photo`, `Illustration` or `Video` (video adds length, loop or not, sound or not).
- **Subject doing what:** one concrete thing, with a verb or state: "tangan menuang kopi susu gula aren", not "kopi".
- **Where:** the real place or surface: meja kayu warung, teras homestay, kain batik.
- **Framing:** shot size and angle: top-down, close-up samping, setinggi mata, dari pintu.
- **Light:** source and time: cahaya jendela pagi, matahari sore, lampu toko hangat.
- **Calm area:** only when page words sit on it: "langit tenang di setengah kiri".

Keep it to one or two lines (about 60-110 characters). Examples:

- `Photo · nasi goreng di daun pisang, meja kayu, top-down, cahaya pagi hangat`
- `Photo · penjahit merapikan kerah kebaya, close-up tangan, cahaya jendela samping`
- `Photo · kamar homestay di Ubud dari pintu, sawah di jendela, cahaya pagi, sisi kiri tenang`
- `Illustration · peta garis jalan dari Pasar Ubud ke homestay, 3 penanda, warna palet`
- `Video · 8 dtk, loop, tanpa suara: sambal diulek di cobek, close-up atas, cahaya dapur`

Never write mood words alone ("aesthetic", "premium vibe"), camera brands, or a style name instead of a subject.

## Sets: one look across every picture

A page, a feed or an app is one set. Before writing the notes, fix a short **look line** for the whole set and keep it in the handoff: light (daylight, one direction), grade (natural colour, one warmth), surfaces (kayu, anyaman, beton) and the palette tones it should sit with. Every note then only adds subject and framing.

- Repeated modules keep one ratio and one crop rule (all menu items 1:1 top-down; all rooms 4:3 from the door).
- Each picture has a job on the page (show the product whole, show it in use, show the place, show the people at work); no decorative filler.
- Match the region and people: Indonesian faces, homes, streets, food and light, not a default Western café.
- Never the same picture twice, and never a crop of one passed as another (Taste).

## The paste-ready brief (when the person asks)

When the person wants to shoot it themselves, brief a photographer, or use a tool, write a fuller brief in the chat or handoff, one per slot. Use these fields, in this order; leave out what doesn't apply.

| Field | What to write |
|---|---|
| Slot | Where it sits and why: "hero halaman Menu, layar pertama, di belakang judul". |
| Output | Ratio and pixel size from your frame (1440×900 is 16:10), orientation, how many variants. |
| Subject | The thing itself, exact: product name, colour, size, label; the person's role (pemilik, barista). |
| Setting | The real place and surface, time of day, season or weather if it shows. |
| Action / moment | What is happening, caught mid-action rather than posed. |
| Composition | Shot size, angle, where the subject sits, what's cropped, where the calm area for text is. |
| Lens / look | Only for realistic photos: natural perspective (around 35-50mm), portrait (around 85mm), close detail (macro). |
| Light | Source, direction, warmth: "jendela di kiri, pagi, lembut". |
| Palette | The design's palette hexes the picture must sit with; one grade for the set. |
| Mood | Two or three plain words: tenang, hangat, sibuk pagi. |
| Text in image | Default none. Page words are set by you on the canvas, never baked into the picture. |
| Consistency | The set's look line; reference photos to follow and what must not change. |
| Avoid | The negatives below plus anything slot-specific. |
| Honesty | Real, stand-in or illustration; consent needed (see below). |

Write the brief as plain sentences under those labels, subject first, concrete nouns over adjectives. Keep the swappable parts (product name, location, colour) easy to spot so the person can change them.

### Reference photos and identity

When the person has photos of the real product, the brief tells the tool to use them as reference and lists what must stay exact: shape, colour, material, label and logo placement, proportions. "Keep the bottle, the red cap and the label exactly as in the reference; do not redesign or add features." Never ask a tool to invent a product the person sells.

For a product that sells, the set has three slots: **main** (the product whole, centred, plain light ground, 1:1), **feature** (one real selling point, close-up, room left for a label you add later, 4:5) and **in use** (a real setting, product still the focus, 4:5).

### What every brief avoids

- Taste's never-list: mood stock, latte art from above, the croissant-and-coffee still life, a made bed, people at screens, stock handshakes, another business's sign, a landmark passed off as the client's place, composite-looking scenes.
- No filters, duotone, halftone, grain, film look, lens flare, light leaks, vignettes or a gradient box on a photo: daylight and one natural grade.
- No words, logos, watermarks, badges or labels drawn into the picture; leave calm space instead. Tools misspell, and Taste puts no words on a place's photo.
- No warped hands, extra fingers, melted text or plastic skin: name these as negatives for people shots.
- No invented claims in the picture: no halal mark, BPOM number, award or "best seller" sticker unless the person holds it, and then you set it on the canvas, not in the photo.

## Honest content

- **Real customers and staff are photographed, never generated.** A testimonial, team or owner photo is that real person, with their consent. If there is none, keep the placeholder and say so (`Photo · Bu Rina, pelanggan sejak 2019, foto asli dari beliau`); never pair a made-up face with a real name or quote, and never invent the quote.
- **What is sold is shown as it is.** A menu item, a kebaya for sale, a homestay room with a price is pictured from the real thing. A tool may clean up or extend the person's own photo; it never replaces it with an invented one.
- **Stand-ins are marked.** When the person has no photo yet and a licensed stock or generated picture stands in (the region's landscape, a generic texture), the note ends with `(stand-in, ganti foto asli)` and the handoff lists it under "replace before launch". Never present a stand-in as the client's place, product or people.
- **Illustrations are honest by nature** (a route map, a spot drawing) but don't draw a real person, a real logo or a real building as if photographed.
- In the handoff, list every slot with its note as the shot list (Taste's default list when the person has none) and mark each as real, stand-in or illustration.

## Video briefs

Taste forbids autoplay video and scroll effects on place, product, shop and service sites. Write video briefs for marketing pieces (Reels, TikTok, Stories, a YouTube intro) or for a click-to-play video the person will provide. Draw the placeholder at the video's ratio (9:16 for Reels, TikTok and Stories; 1:1 or 4:5 for feed; 16:9 for YouTube and the web) showing its strongest still, with the `Video ·` note.

- **One shot, one idea** works best in today's tools: plan 5-10 second shots and join them, rather than one long take.
- **Each shot gets:** subject, camera (still, slow push-in, pan, follow), action and pace, light, and what moves versus what stays.
- **A short film is a shot list with times:** a header with look, place and the main person or product, then one line per shot with its start and end second, ending on a hold of the product or the name.

| Time | Shot |
|---|---|
| 0-3 dtk | Close-up: es batu jatuh ke gelas, kamera diam, cahaya jendela samping |
| 3-7 dtk | Medium: barista menuang gula aren lalu susu, kamera pelan mendekat |
| 7-10 dtk | Gelas di meja kayu, logo kedai di cup sudah asli (bukan dibuat alat), tahan 2 dtk |

- **Sound:** say "tanpa suara" or describe it (suara es, musik akustik pelan); a voiceover script is written final, word for word, with names spelled as spoken.
- **Avoid in video:** faces changing between shots, jitter, morphing hands, invented text on screen. Titles and prices are added in editing, from your canvas.
- **Motion-graphic videos** (title cards, a price reveal, an app tour) are designed on your canvas as frames with timings in a `motion-note`, not described as camera shots; Koda or an editor animates them.

## Social and print pieces

For a feed post, story or poster, the picture is one layer: you draw the layout, type and colour on the canvas, and the photo slot's note asks for the picture alone with its calm area placed where your words sit. State the platform ratio in the note when it isn't obvious.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus Taste Skill imagegen-frontend-web and imagegen-frontend-mobile (MIT, Copyright (c) 2026 Leonxlnx); prompt patterns studied from YouMind-OpenLab/awesome-gpt-image-2 and YouMind-OpenLab/awesome-seedance-2-prompts (CC-BY-4.0, their individual authors) and heygen-com/hyperframes (Apache-2.0). Changed: rewritten for the OpenPencil canvas.
