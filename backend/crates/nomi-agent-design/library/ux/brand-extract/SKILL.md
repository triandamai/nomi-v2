---
name: brand-extract
description: "How to take a person's existing brand from what they show you (logo, shopfront photo, Instagram screenshots, their site's text): measure instead of guessing, rank colours by how often they appear, map them to $color-* tokens, pick the logo source and fonts, and stop and ask at walls."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (skills/brand-extract/SKILL.md)"
---
# Brand extract: measure, don't guess

Left alone, a designer drifts to the mean: Inter, an indigo accent, a purple gradient. That fits nobody. Every brand value you use traces to something you saw, something the person gave you, or an assumption you label. Taste agrees: the client's own colours (logo, shopfront paint) replace the dealt palette (`taste-color`).

## What you can measure, and from what

- **`read_web_page` returns text only.** From a site you get the name, tagline, offer, prices, hours, street and district, and the voice. You do not get its colours, fonts, logo or photos.
- **Images you can see.** Colours, logo, type and photo style come from pictures: a logo file, a screenshot of their site or Instagram grid, a shopfront, packaging, a printed menu, a uniform.
- So when the person says "pakai brand kami" and only gives a link, read it for words, then ask once: "Boleh kirim logo dan 2–3 screenshot Instagram atau foto toko? Warnanya saya ambil dari situ."

## Colours: rank, then map

1. **Collect brand surfaces only:** logo, signage, shop paint, packaging, Instagram post templates, the site's backgrounds and buttons in a screenshot. Ignore what is inside photos (food, skin, sky, plants).
2. **List the distinct colours and rank them by how much area they cover across all pictures.** The most frequent near-white or cream is usually the background; the darkest frequent colour the ink; the most frequent mid-saturated colour the accent; a second chromatic colour the secondary.
3. **Read each hex from the flattest source:** a logo file or screenshot beats a photo. From a photo, sample an evenly lit flat patch, away from highlights and shadows, and label the value "observed, approximate".
4. **Map the seven roles to tokens:**

| Measured role | Token |
|---|---|
| background (most frequent light) | `$color-background`, `$color-popover` |
| surface (cards, panels) | `$color-card`, `$color-muted` |
| foreground (darkest frequent) | `$color-foreground` and the `-foreground` partners |
| muted (secondary text grey) | `$color-muted-foreground` |
| border (hairlines) | `$color-border`; `$color-input` a step stronger |
| accent (most frequent chromatic) | `$color-primary`, `$color-ring` |
| accent-secondary | `$color-secondary` pair, or the custom `$color-brand` pair |

5. **A role with nothing measured** is derived from a measured colour (a lighter step of the background for the card, ink at 12–15% alpha for the border) and labelled "derived from <colour>".
6. **Check contrast** (`ux-accessibility`). A brand accent that fails under a white label stays exactly as measured in `$color-brand` for the logo, marks and one band, and `$color-primary` becomes the nearest darker shade that passes (or the ink, per `taste-color`'s * rule). Tell the person in one line: "Oranye logo dipakai untuk tanda; tombolnya sedikit lebih gelap supaya terbaca."

## Logo

Priority, best first: a vector file they send (SVG, PDF, AI) > a transparent PNG > their app icon or profile picture > a clean crop from a screenshot > a photo of the sign. Never leave the brand without a mark when they have one.
- The canvas holds no images: draw a tile at the logo's real size and ratio with a note ("Logo · file from the person, kopinako-logo.png, use the green version on cream"). Koda adds the file itself.
- Never redraw their logo as a `path` look-alike. If they have no logo, set the name in `$font-heading` as a wordmark and call it a stand-in in the handoff.

## Fonts

- Name the type class you see in the logo, sign or posts (geometric sans, rounded sans, slab, high-contrast serif, script).
- If the face is identifiable and free, use it; if it is proprietary or unknown, choose the closest pairing from `taste-type` and write "stand-in for the logo's rounded sans". Never present a guessed font name as fact.
- A face Taste bans for the site (Poppins, Montserrat…) stays in the logo file only; the site gets the nearest allowed face, and you say why.

## Voice and photos

- Quote 3–5 real lines from their captions or site to ground the voice: how they greet ("Kak"), their language mix, sentence length. Taste's copy rules still apply (no exclamation marks, no slop words).
- Describe their photo style for your photo notes: subject, framing, light ("top-down, warm daylight, wooden table").
- Invent nothing beyond what their words and pictures support: no founding year, no awards, no customer counts.

## Walls: stop and ask

If `read_web_page` returns a login page, "Just a moment…", "Verify you are human" or an empty shell, or Instagram asks for a login: stop. Never try to get around it. Ask the person for screenshots. If they say to carry on without, use only what they gave and label the rest "inferred".

## The brand card

Write it in your plan and pin it with `post_crew_note`, so the directions, the preview and Koda use the same values:

| Item | Value | Label |
|---|---|---|
| Background | #F7F1E6 | observed, screenshot of Instagram grid |
| Primary | #2F6B3A (sign green) | observed, approximate, photo of shopfront |
| Brand mark | #3E8A4C | provided, logo SVG |
| Fonts | Bricolage Grotesque + Figtree | inferred, stand-in for the logo's rounded sans |
| Logo | kopinako-logo.svg | provided |
| Voice | "Kak, pesan lewat WA ya. Kopi susu gula aren Rp 18.000." | observed, Instagram caption |

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): skills/brand-extract/SKILL.md. Changed: rewritten for the OpenPencil canvas, and for images and page text instead of a driven browser.
