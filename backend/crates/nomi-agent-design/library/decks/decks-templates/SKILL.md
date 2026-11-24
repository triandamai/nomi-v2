---
name: decks-templates
description: "How you design a presentation deck: the story arc per occasion, slide grid and type scale at 1920x1080, light/dark rhythm, data slides, drawing slides as canvas frames, and what survives PDF/PPTX export. Read before any deck, pitch, company profile or report slides."
license: "Apache-2.0, Copyright nexu-io open-design contributors (open-design; html-anything deck templates); MIT, Copyright (c) 2026 op7418 (guizang-ppt, open-design's bundled copy only); MIT, Copyright (c) 2026 Zara Zhang (beautiful-html-templates looks); MIT, Copyright (c) 2026 lewis (html-ppt-skill looks); MIT, tw93/kami; MIT, Copyright (c) 2026 Leonxlnx (taste-skill looks); Apache-2.0, anthropics/financial-services (pitch-book workflow shape)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (simple-deck, kami-deck, replit-deck, open-design-landing-deck, ib-pitch-book, weekly-update, guizang-ppt, html-ppt, html-ppt-*, html-ppt-zhangzara-*); https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (deck-guizang-editorial, deck-open-slide-canvas, deck-swiss-international, ppt-keynote, slides, frontend-slides, pptx, pptx-generator, pptx-html-fidelity-audit, nanobanana-ppt, digits-fintech-swiss-template, editorial-burgundy-principles-template, swiss-creative-mode-template, html-ppt-retro-quarterly-review); https://github.com/zarazhangrui/beautiful-html-templates; https://github.com/anthropics/financial-services"
---
# Designing decks

How you plan, lay out and draw a slide deck so it reads at a glance on a projector or a phone, and still holds together when it is built as HTML and exported to PDF or PPTX.

A deck is a marketing piece: it may follow its own look (pick one from [references/deck-looks.md](references/deck-looks.md)), not Taste's website rules. It still uses tokens, the person's real words and numbers, and their language. Slide layouts are in [references/deck-layouts.md](references/deck-layouts.md).

## 1. Before any slide: purpose, audience, length

Write three lines first: who sits in the room (a bank officer, a class, the team, buyers at a bazaar), the one thing they must do or believe at the end (approve the loan, sign up, decide), and how long the talk is.

| Talk | Slides | Notes |
|---|---|---|
| Lightning talk, product overview (5-10 min) | 6 | One idea per slide, almost no body text |
| Pitch to an investor, bank or partner (15 min) | 8-10 | Problem, proof, money, ask |
| Operating review, laporan bulanan (20-30 min) | 12-18 | Numbers carry it; chapters every 4-5 slides |
| Class or workshop module | 7-12 | Objectives up front, a self-check near the end |
| Weekly team update | 7-8 | Same order every week so people can skim |

Then write every slide title as a full sentence before drawing ("Penjualan online naik dua kali lipat sejak Juni", not "Penjualan"). If the titles read as a story on their own, the deck works. Never draw with placeholder text.

## 2. Story arcs

Pick the arc for the occasion and adapt; one slide per line.

- **Pitch (8-10):** cover · the problem in the customer's words · one big number that proves it · what you do (three points) · how it works (steps) · proof (customer quote or traction, only real) · before/after · the money (use of funds, unit numbers) · the ask.
- **Company profile (8-12):** cover · who we are in one sentence · what we make or serve (photo-led) · how we work · numbers that are true (years, outlets, clients served) · selected work or menu · people · where to find us · contact.
- **Operating review / laporan:** cover · the headline (one sentence + one number) · KPI row · what moved and why · what stalled · chapter · deep dives (chart slides) · risks · decisions needed · next period.
- **Weekly update:** cover (team, week, author) · headline + the number that matters · shipped · in flight (owners) · blocked + the ask · metrics (1-2 charts) · next week's asks · thanks.
- **Class module:** cover · learning objectives · chapter per objective with explain + example + practice · recap · self-check questions · next step.
- **Brand or manifesto (short, 3-6):** a numeric or typographic headline · the lockup · principle cards · closing line.
- **Investor-grade pitch book (finance):** cover with confidentiality line · contents · market context (KPI strip + one chart) · comparables table · precedent deals · valuation range chart · cash-flow assumptions + sensitivity grid · four options matrix with the recommended one inverted · recommendation and timeline. Every figure gets a source and an as-of date; it is discussion material, not advice; unknown numbers are a dash, never invented.

## 3. Rhythm: light, dark and hero slides

Every slide is one of four kinds: **light** (the paper ground), **dark** (inverted), **hero light** and **hero dark** (more air, one huge element: cover, chapter, big number, big quote, closing).

- Never three slides of the same kind in a row.
- Decks of 8+ slides have at least one hero dark and one hero light.
- A dark or hero slide every 3-4 slides gives the eye a breath and makes the next slide land.
- The cover is usually hero light (or the brand colour); the close is hero dark or hero light.
- Write the rhythm as a list next to your titles before drawing ("01 hero light Cover, 02 light Problem, 03 hero dark Big number…"). Show it to the person: changing a list is cheap.

## 4. The slide grid at 1920x1080

- **Margins:** 96 at the sides is the default; 128 or 160 for airy, editorial decks. Top 80. Never closer to the edge than 64.
- **Rails:** a thin chrome strip at the top (deck title or brand mark left, section right) and a footer rail at y 988-1040 (folio `03 / 12` right, date or source left). Content never crosses y 964: that gap is what keeps the PPTX export from colliding with the footer.
- **Columns:** a 12-column grid inside the margins (gutter 32). Common splits: 12, 6+6, 7+5, 8+4, 4+4+4, 3x4.
- **Type scale (px):** 18 · 22 · 28 · 36 · 48 · 64 · 88 · 120 · 160 · 220. Body 28-36 (never below 22 on a slide), slide titles 64-88, hero statements 120-160, a single giant number 160-220. Kickers and meta in mono, 18-22, uppercase, tracked +1 to +2.
- **One visual centre per slide:** one sentence, one number, or one picture. If two things are equal, use an equal 3-column grid; never two competing paragraphs.
- **Body text:** at most ~40 words a slide (speaker notes carry the rest). Three bullets, not seven.

## 5. Type and colour logic

- **Three jobs, three faces at most:** a display face for titles, numbers and quotes (often a serif); a sans for body; a mono for kickers, meta, page numbers and data labels. Never mix two display faces in a deck.
- **One accent per deck, used at most twice per slide** (a key number, a marker, one underline). An accent is type colour or a thin rule; filling big boxes with it reads as cheap.
- **One look per deck.** Never switch palettes mid-deck; chapters change by light/dark, not by new colours.
- **The person's brand colour replaces the look's accent** (logo colour, shopfront paint). Keep the look's paper and ink, and check the accent against both for contrast.
- **Numbers:** tabular figures, units smaller than the digits (`64 hari`, `Rp 1,2 M`), deltas in mono (`▲ 38% YoY`).
- No emoji as decoration, no drawn stand-ins for real logos, no stock-icon clutter on metric cards.

## 6. Data, chart and table slides

- One chart per slide, with the takeaway as the title ("Pesanan GoFood menyumbang 41% omzet Oktober"). Highlight the one series that matters in the accent; the rest stay muted ink.
- Bars for comparison, a line for time, a single big number when there is only one figure. No pies with more than three slices, no 3D.
- Tables: hairline rows, numbers right-aligned in tabular figures, the total or median row in bold, the row being discussed tinted. At most ~8 rows a slide; split the rest.
- Every figure has a source line in the footer area (`Sumber: laporan kasir Moka, Okt 2026`). An unknown number is shown as `—`, never guessed.

## 7. Drawing the deck on the canvas

- **One top-level frame per slide**, 1920x1080, named in order: `01 Cover`, `02 Masalah`, `03 Angka besar`. Lay them in rows of four, 120 apart, so the deck reads like a contact sheet. Give each its own `fill` and theme.
- **Masters as components:** draw the chrome strip, the footer rail (folio, source) and the kicker style once as reusable frames, and place them on every slide with `ref`, overriding only the text (`descendants` for the folio `05 / 12`).
- **Element budget:** the whole preview holds 300 elements. A 10-slide deck has about 25 nodes a slide, masters included. Big type and empty space are cheap; dense grids are expensive. Over 12 slides, split slides into extra `.pen` files and keep the element count in mind, or draw the key slides and list the rest as titles on a `Deck outline` frame.
- **Tokens:** paper is `$color-background`, ink `$color-foreground`, secondary text `$color-muted-foreground`, hairlines `$color-border`, the accent `$color-primary`, dark slides use a dark theme on the frame (or `$color-foreground` as fill with `$color-background` text). Chart series use `$color-chart-1…5`.
- **Pictures:** a photo or screenshot is a placeholder frame at the slide's real ratio (16:10 or 4:3 for an image beside text, 16:9 for a full-bleed visual, 1:1 or 3:2 for small insets, one fixed height across an image grid), toned from the palette, with a note: `Photo · etalase toko roti dari luar, pagi, pintu terbuka`.
- **No gradients, glows or textures:** where a look relies on them (sun-glow, scanlines, paper grain, blurred blobs, gradient text), use its solid base colour and add a one-line note on the frame for the build (`Build note: soft radial glow behind title, sun yellow`).
- **Motion is a note,** never drawn: slide transitions (a fade), step-by-step reveals on a pipeline slide.
- **Speaker notes:** if the person wants them, put a text block under each slide frame, outside it, named `Notes · 03`, 2-4 sentences in their language.

## 8. Built as HTML, exported to PDF or PPTX

Design so the export can't break:

- **Fixed canvas:** every slide fits 1920x1080 with nothing overflowing; a slide that needs scrolling is two slides.
- **Footer rail discipline:** content stops at y 964. Most broken exports are text blocks that slide into the footer or off the bottom.
- **Hero slides are centred by measuring the block**, not pushed down from the top margin.
- **What survives PPTX:** solid fills, text boxes, lines, simple shapes and placed images. Italic emphasis, letter-spacing on kickers, mono vs serif assignments and coloured chrome rows are the first things to fall back to defaults: keep emphasis meaningful even without italics, and don't force italics on scripts without an italic tradition.
- **Fonts:** name the real font you chose; give a common fallback in the handoff (Inter, Georgia) for PPTX opened without it.
- **PDF:** each slide is one page; nothing hides behind navigation dots or hover.

## 9. Before you hand over

- Titles alone tell the story; every slide has one visual centre.
- Rhythm list checked: no three of a kind in a row, hero slides placed.
- One look, one accent, three faces at most, body never below 22 px.
- Every number is the person's or sourced; unknowns are dashes.
- Masters placed with `ref`; folios correct; element count under budget.
- Content clear of the footer rail on every slide.

Tools that turn a brief into a finished .pptx or image-heavy deck exist (see `../../images/images-media-tools/SKILL.md`); you can't run them, but you can say what the deck needs.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus guizang-ppt as bundled in open-design (MIT, Copyright (c) 2026 op7418), html-anything deck templates (Apache-2.0), tw93/kami (MIT), Zara Zhang's beautiful-html-templates (MIT), lewis's html-ppt-skill (MIT) and the pitch-book workflow shape of anthropics/financial-services (Apache-2.0). Changed: rewritten for the OpenPencil canvas.
