---
name: templates-web-pages
description: "Web page templates (landing, pricing, waitlist, studio or portfolio site, prototype layouts, low-fi wireframes, contact widget, variant panels): what each must answer, section order, and what Taste overrides. Read when a brief names one of these page types."
license: "Apache-2.0, Copyright nexu-io open-design contributors (open-design); MIT, Copyright (c) 2026 Aster Atelier template contributors (aster-atelier); MIT, tw93/kami (kami-landing); MIT, alchaincyf/huashu-design (tweaks idea); MIT, Copyright (c) 2026 Leonxlnx (web-prototype-taste-* distil Taste Skill)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (saas-landing, pricing-page, waitlist-page, aster-atelier, kami-landing, open-design-landing, web-prototype, web-prototype-taste-brutalist, web-prototype-taste-editorial, web-prototype-taste-soft, dating-web, contact-widget, wireframe-greybox, wireframe-sketch, webgl-experience, tweaks); https://github.com/tw93/kami; https://github.com/alchaincyf/huashu-design; https://github.com/Leonxlnx/taste-skill"
---
# Web page templates

What open-design's page templates teach about structure, and how you use them on the canvas with Taste as the final word on look.

A template here is a checklist of what the page must answer and in what order, never a look to copy. For any website or web app, Taste wins (`../../taste/taste-core/SKILL.md`): plain h1 naming the category, one section per screen, the real product or place shown big, 90-95% neutral interface, no invented numbers. For a named visual direction, see `../../styles/styles-index/SKILL.md`. Draw every page at 1440, 390 and 768.

## Landing page (product, service or software)

**Must answer, in order:** what it is and for whom (hero: h1, one-line subhead, the main action, an optional second action) · what you get (3-6 features, each a short title and 1-2 sentences, each with a real picture or UI fragment) · proof (only real: actual client logos, real quotes with names, or skip the section) · price (if the business has one) · a closing call to action · a plain footer (links, contact, address as street and district).

**Keep from the template:** the accent appears once in the hero, once in the closing band, and on links; nowhere else. Body copy is specific to the brief, never filler.

**Taste overrides:** no eyebrow + claim + three-card skeleton in every section; the features section follows the reader's questions and the material the brief has. A place (warung, salon, guesthouse) gets photographs, never UI fragments or icon tiles. The closing band is a ground change, not an accent flood.

## Pricing page

**Order:** a short hero (title + one line; a monthly/yearly switch only if both exist) · plan cards · a comparison table · FAQ (4-6 questions) · a closing line with one button.

- **Tier shape from the business:** 3 tiers is the common case (Basic / Pro / Bisnis), 2 for personal vs business, 4 only with a real enterprise tier. A salon or kursus usually has a price list, not tiers: use a list or table instead.
- **Plan card:** tier name, the price large in the display face (`Rp 149.000` with `/bulan` smaller), one line of who it's for, 4-6 features, one button. Mark the recommended tier with the accent border; only that card's button is primary.
- **Comparison table:** features grouped into 2-3 sections, tick / dash / value cells, header row pinned, every row a real feature.
- **Prices are the person's.** If unknown, show a dash and ask; never invent a plausible number.

Draw the plan card as a reusable component with Recommended and Default variants; the table as a reusable row.

## Waitlist or pre-launch page

**One job:** get an email or WhatsApp number. Exactly one primary action, visible without scrolling at 390x844.

- Hero with the product name, one sentence on what it will be and when, the form (labelled field + button), a quiet note on what happens next ("Kami kabari saat buka, tidak ada spam").
- Draw the form's states: default, focus, error ("Format email belum benar"), success (a calm confirmation in place of the form).
- **Never:** countdown timers, invented "10.000 orang sudah daftar", emoji decoration, more than two font families, a gradient over more than a sliver of the hero.

## Studio, portfolio and brand sites (editorial)

Several templates are chaptered editorial sites: a creative studio with large serif headings, small uppercase coordinates and fine rules (cobalt, ivory and bronze); a parchment "kami" brand page (eyebrow row, a serif hero 96-106 px, a manifesto paragraph with a left rule, a metrics row, numbered chapters, a footer with a giant kicker word); and a magazine-style product site with Roman-numeral sections and an oversized italic footer word.

**What you keep:** numbered chapters as the section rhythm, one serif display face at one weight, small mono or caps meta for coordinates and dates, a manifesto paragraph instead of feature tiles, a giant wordmark in the footer, contact and FAQ as plain disclosures.

**What Taste overrides:** paper texture, tilted cards, scroll-reveal on every section, stat rings, partner glyph rows and fake star counts go. Their artwork is never reused; the person's own work, photographed, carries the page. A portfolio or studio is a "service" kind in Taste: its photographs lead.

## Prototype layouts (any marketing page)

A small set of section shapes to compose from: centred hero · split hero (text + picture) · feature triplet · stat row · pull quote · split detail (2:1 or 1:2) · log list (changelog, blog index, news) · comparison table · closing CTA strip.

**Rhythm:** never two of the same shape in a row (two stat rows, two quotes, two triplets). A five-section landing: hero, features, stat or quote, split detail, closing. A docs or news index: centred hero, log list, closing.

## Three taste variants of a prototype

These are Taste Skill's own modes as page looks; offer them only where the trade fits.

- **Editorial minimal:** warm off-white (never pure white), off-black text, 1 px light borders, near-invisible shadows, serif display with tight tracking, grotesk body, mono meta, pastel chips only on tags. Fits services, studios, writers, consultants.
- **Soft premium:** silver-grey or cream ground, heavy grotesk display, large radii (28-40) on major surfaces, cards with an outer hairline shell and an inner core, pill buttons with the arrow in its own circle. Fits product and software. Keep one radius ladder.
- **Industrial brutalist:** newsprint ground, carbon ink, one hazard-red accent on rules and alerts only, heavy neo-grotesk display, mono caps micro type, square corners, a visible 1 px grid. Fits hardware, workshops, bold product launches; never a cafe.

## Consumer app shell as a web page

The dating-dashboard template is a web app home: left rail navigation, a strip of community signals, headline KPIs, a 30-day bar chart and a trend block, in restrained editorial type. Treat it as an app screen: see `../templates-app-dashboards/SKILL.md`.

## Low-fi wireframes (a first pass you can offer)

When the person wants to agree on structure before look, offer a wireframe pass first.

- **Greybox:** a pale neutral page, grey blocks, every image a rectangle with a diagonal cross (two thin `path` lines), text as solid grey bars of varying widths, sharp 1.5-2 px outlines, no colour, no real fonts. It should read as a blueprint at thumbnail size. Even in greybox, headings carry the real words.
- **Sketch:** graph-paper feel (a few faint hairlines, not a full grid), marker-weight strokes, variant tabs above the frame (`A · foto besar`, `B · daftar menu dulu`), sticky-note annotations as small yellow frames beside the screen, hatched chart placeholders.
- Name wireframe frames `Wireframe · Home · 390` so they are never mistaken for the final design; draw the final screens as separate frames.

## Contact widget

The template is a floating chat bubble with a panel (avatar, greeting, message field, social links, a book-a-meeting card, an offline form).

**Taste override:** on a Nomi website, no floating chat bubble or popup. Put contact in the page: a contact section or footer with the WhatsApp button (number exactly as the person gave it), opening hours, and the address as street and district. Draw the panel only if the person explicitly asks for an in-app support widget; then use one primary colour, neutral slate surfaces, 16 radius cards, a soft shadow, and only the channels they actually have.

## Real-time 3D or shader pages

Full-screen shader, particle or 3D scenes are allowed by Taste only for tech trades (software, AI, gaming, hardware). On the canvas, draw the composition with flat shapes in the scene's main colours and a build note describing the effect; never on a place or shop site.

## Showing alternatives (variant panels)

One template wraps a page in a live panel of controls (accent colour, type scale, density, theme). On the canvas you show the same choice as side-by-side frames: the hero at 1440 in two or three variants (`Hero · A warm`, `Hero · B ink`), each changing one thing, with a one-line note under each on what differs. Let the person pick; then carry the choice through every screen.

## Before you hand over

- Each page answers its questions in order with real copy and real prices.
- Taste checks pass: hero alone on the first screen, one section per screen, equal edges, no invented proof.
- Form states drawn; one primary action per screen.
- Wireframe and final frames are clearly named apart.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus aster-atelier (MIT, Copyright (c) 2026 Aster Atelier template contributors), tw93/kami (MIT), alchaincyf/huashu-design (MIT) and Taste Skill (MIT, Copyright (c) 2026 Leonxlnx). Changed: rewritten for the OpenPencil canvas.
