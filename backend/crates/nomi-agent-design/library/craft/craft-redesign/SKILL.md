---
name: craft-redesign
description: "How to redesign a person's existing website or app: scan what exists, audit it against a fixed checklist, sort every finding into keep, fix or replace, and change things in a safe order. Read it whenever the person already has a site, a shop page or an app."
license: "MIT, Copyright (c) 2026 Leonxlnx (Taste Skill family: redesign-skill), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (redesign-skill, as vendored in open-design@5fdcc3d, 2026)"
---
# Redesign an existing site

How you turn someone's current website or app into a better one without throwing away what already works: scan it, audit it, sort each finding into keep, fix or replace, then redraw in a fixed order.

A redesign is not a fresh brief. The person's customers already know the name, the colours, the menu and where the WhatsApp button is. Your job is to keep what earns trust and remove what costs it.

## 1. Scan: what exists

Work from what the person gives you: a link they describe, screenshots, photos of a printed menu, their Instagram grid, an old canvas. Read it into frames the way `craft-screenshot-to-design` describes, then write a short inventory:

| Line | Example (Bengkel Jaya Motor) |
|---|---|
| Kind and main action | Place; "Booking servis" via WhatsApp |
| Pages and sections | Home: hero, layanan, harga, galeri, testimoni, kontak |
| Brand assets | Logo (red and black), shopfront photo, 14 workshop photos |
| Real content | Price list for 9 services, opening hours, address on Jl. Kaliurang |
| What customers use | WhatsApp button (the person says most bookings come from it) |
| Constraints | Keep the logo and red; keep the URL structure |

Ask the person one question if it changes the work: what do customers do most on the current site, and what do they complain about?

## 2. Diagnose: the audit

Go through every group. Write each finding as what you see, why it hurts, and the fix.

**Type**
- Default or overused faces (Inter, Roboto, Poppins, Montserrat): swap for a pairing from `taste-type`.
- Headlines without presence, or bold-and-big doing the work: size and tight tracking instead.
- Body lines wider than about 62 characters; text in all caps; orphaned single words on a heading's last line.
- Only 400 and 700 in use: Taste uses 400 and 500, never 700.
- Prices and numbers in proportional figures in a table: tabular figures.

**Colour and surfaces**
- More than one accent; oversaturated accent; purple-blue "AI" gradient.
- Warm and cool greys mixed; pure #000000 grounds.
- One random dark section in a light page (or the reverse): Taste separates sections by ground change within one palette; a sudden black band reads as pasted in.
- Shadows lit from different directions; generic black shadows on coloured grounds.

**Layout**
- Everything centred; three equal feature cards as the default section.
- No container width, so text runs edge to edge on wide screens.
- Cards in a row at different heights with buttons at random heights; pricing features starting at different heights.
- Equal radius on everything regardless of size; mismatched paddings.
- Blocks off the content edges (Taste house rule 4).

**Interaction and states**
- No hover, press or focus state; dead links to `#`; no current-page mark in the nav.
- No loading, empty or error states in the app parts; errors as pop-up alerts.

**Content**
- Placeholder names, lorem, round fake numbers, identical dates, the same avatar for several people.
- Slop words ("solusi terbaik", "terpercaya", "elevate", "seamless"), exclamation marks, "Oops!" errors, passive voice.
- Invented reviews or customer counts: remove unless the person confirms them.

**Components and icons**
- Generic card look (border + shadow + white) on every block; pill "Baru" badges; accordions for two questions; sun-moon toggles; four-column footer link farms; carousels.
- Mixed icon sets or stroke widths; rocket for "launch", shield for "aman".

**What is usually missing**
- Legal links (Kebijakan privasi), a way back from every page, a 404 page, form validation, a favicon, a share image.

## 3. Sort: keep, fix, replace

Put every finding in one of three columns and show the person this table before you draw:

| Keep (customers rely on it) | Fix (right idea, weak execution) | Replace (works against them) |
|---|---|---|
| Logo, brand red, shopfront photo, WhatsApp as main action, price list content, page order customers know | Type pairing, spacing scale, card heights, button states, nav current page | Auto-sliding carousel, invented testimonials, stock mechanic photo, three icon cards with "Kualitas terbaik" |

Rules for sorting:
- **The brand stays** unless the person asks to change it: their logo, their colours (Taste: the client's own colours replace the dealt palette), their name. Improve how they are used, not what they are.
- **Real content stays**: prices, hours, menu, services. Rewrite the words around them, not the facts.
- **Structure customers know stays** unless it breaks a Taste rule: if people find the price list second on the page, keep it second.
- **Anything invented goes**: reviews, numbers, logos of customers the person never named.

## 4. Fix in this order

Each step is visible on its own, so the person can stop after any of them:

1. **Type:** the face swap and the scale (biggest change for the least risk).
2. **Colour cleanup:** one accent, one grey family, tokens for everything.
3. **Hover, press and focus states.**
4. **Layout and spacing:** the grid, edges, container width, equal card rows.
5. **Replace generic components** with what the content needs (ruled price list for icon cards, photo for stock).
6. **Loading, empty and error states.**
7. **Final type and spacing polish** (`craft-polish`).

## 5. Rules

- **Never redesign from scratch** when the person asked for an improvement; show the same page, better. If you believe a full redesign is right, say why in one line and offer it as a second direction.
- **Keep ids for elements that survive**, so the person's feedback on the old version still finds them.
- **Show before and after** for the hero and one inner section at 1440 and 390 when you present.
- **Small, reviewable rounds:** one or two of the fix steps per round, each round checked with `taste-check`.

## Taste wins over the source

The source suggests grain overlays, mesh gradients, faint background photos behind sections, overlapping layers, broken grids, parallax stacks, smooth-scroll inertia, scroll-driven reveals, glassmorphism and spotlight borders as "upgrades". None of those are allowed on product, service or place pages, and on tech pages only as Taste's one dealt effect or moment. Its "vary the radius" means use the ladder by size, never random radii.

Adapted from Taste Skill's redesign-skill (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
