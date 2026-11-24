---
name: huashu-slide-decks
description: "How you make an HTML slide deck the huashu way: HTML first with PDF and PPTX as exports, one file per slide, two showcase slides before the batch, slide grammar and rhythm, data pages, sizes, hard rules and the final checklist. Read it before any deck, pitch, lesson or report."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Slide decks

How you make an HTML slide deck the huashu way: HTML first with PDF and PPTX as exports, one file per slide, two showcase slides before the batch, slide grammar and rhythm, data pages, sizes, hard rules and the final checklist.

Decks are a new output beside the canvas, so huashu's logic leads here; Taste's copy and honesty rules still apply (`taste-copy`: real words in the person's language, nothing invented). Run the three-direction gate first (`huashu-directions`: two representative slides per direction).

## HTML is the source; PDF and PPTX are snapshots

Whatever the final format, the HTML deck comes first: it presents in a browser (full screen, arrow keys), every slide can be opened and checked alone, and it is the only upstream for the exports. Fixing the HTML after an export means exporting again, so never treat an export as the thing you edit.

The delivery chain, without asking each time: HTML deck, then **always a PDF** automatically, then an **editable PPTX only when asked**. A PDF keeps vector text (searchable, copyable) and the exact look; it is the right home for any visual-first deck, not a compromise.

**Ask about PPTX before writing the first slide.** It is the one decision that changes how you write: an editable PPTX needs slides written to four hard constraints from the first line, or a different export route. Ask once:

> Saya buat versi presentasi di browser dulu, lalu PDF otomatis. Perlu juga PowerPoint yang bisa diedit (misalnya rekan Anda mau ganti teks)? Kalau ya, saya susun dari awal supaya bisa diedit; beberapa efek visual (gradien, ilustrasi rumit) tidak ikut.

Routes and constraints are in `references/editable-pptx.md`. "HTML and editable PPTX" is not a conflict (a constraint-ready HTML deck still presents in a browser). "Editable PPTX and animation" is a real conflict: tell the person and let them choose; never quietly hand-build a PowerPoint file coordinate by coordinate.

## Structure: one file per slide

Default: **each slide its own HTML page at a fixed 1920 × 1080**, plus an index page that strings them together (keyboard paging, scaling with letterbox, a page counter, remembering the position, printing every slide). Why: one slide's styles can't break another; each slide opens alone for checking; slides can be built independently and merged with no conflicts. A single-file deck is only for five slides or fewer that share live state and don't need an overview wall.

- A shared tokens file holds only what truly crosses slides: colour, type scale and spacing variables, the canvas size, the identical header and footer chrome. Never a single slide's layout.
- Use fixed pixel sizes on the 1920 × 1080 canvas, never viewport units or percentages; the index page scales the whole slide.
- Label slides meaningfully ("04 Masalah") and number from 1, never 0: when the person says "slide 5" they mean the fifth.
- **No slide carries its own page number or progress marker**; only the index page does (two competing page counters, "02/03" and "6/16", happened).

## Two showcase slides before the batch

For any deck of five slides or more, never write from slide 1 to the end. Build the **two most different slide types** first (cover and a data page; cover and a quote page), get the grammar approved (header, type, colour, spacing, structure, language mix), then batch the rest on that grammar. A wrong direction then costs two slides, not thirteen.

| Deck type | Showcase pair |
|---|---|
| Brochure, product pitch | Cover + a values or emotion page |
| Launch | Cover + product feature page |
| Data report | Big chart page + conclusion page |
| Lesson, training | Chapter opener + a concept page |

## Slide design

**Write the system first** (in your notes): at most two grounds (say 90% light, 10% dark section dividers), display and body faces (`huashu-typography`), the rhythm (dividers as full colour with light type, content on light), images (full-bleed on hero slides, charts on data slides).

**Layouts:** title (plain ground, huge title, subtitle, author and date); section divider (colour ground, chapter number and title); content (title plus one to three points); data (title, one big chart or number, a short note); image (full-bleed photo, small caption); quote (space, huge quote, attribution); two-column (versus, before and after, problem and solution). **Four or five layouts per deck at most.**

**Each slide answers the five form questions** (`huashu-workflow` §3), above all its narrative role: cover, transition, data, quote, close. One role per slide; the layout follows the role.

**Rotate the visual hero.** Thirteen slides of "text plus one screenshot" is monotone. Rotate: cover type (huge title with a header strip); a single large portrait of one concept or person; a group of people or avatars; a timeline of cards; a network of connected nodes; before and after with a centre arrow; the product UI in a device frame; a big quote filling half the slide; real faces with quotes in a 2 × 2 or 1 × 4; a closing big-type slide with one link.

**Rhythm by intent:** mostly light with an occasional colour divider and an occasional dark stretch; some text-heavy, some image-heavy, some near-empty quotes; normal titles with an occasional giant one. Every slide looking the same is a PowerPoint template, not design.

**Density:** one core message, three or four supporting points and one visual hero per slide; more goes to a new slide. A viewer gives a slide about ten seconds; one memory beats four. In list or matrix slides don't draw all items the same size: the five you're discussing large, the rest small as background. On a big-number slide the number is the hero; keep its caption to three lines. Separate a quote from its attribution with space. Cut until the whitespace makes you slightly nervous.

**Data slides are not "three big numbers and a bar chart".** That is the template answer in every style. Ask what relationship the numbers express: proportion (objects scaled to each other), change (one narrative curve), composition (a whole taken apart), comparison (left against right), scale (one huge object beside a familiar one). Only real or verifiable numbers; mark any illustration "ilustrasi".

**Sizes at 1920 × 1080:** body 24px minimum (28 to 36 ideal); titles 60 to 120; section titles 80 to 160; hero type 180 to 240; labels, notes and chart ticks 16 minimum. A slide is read from ten metres. Contrast 4.5:1 for body.

## A publication grammar that worked (brochures, long reports)

Every slide: a thin top strip with the logo (22 to 28px) and the issue line, a rule; a short kicker; the headline (heavy serif, 80 to 140px by content amount, at most one key phrase in the brand colour); an optional subtitle in the second language; a divider; the content (60/40 columns, a 2 × 2, or a list); a footer with the section name. Body 17 to 21px at that size only in print-like brochures, leading 1.75 to 1.85; brand-colour bold on at most three key words per slide. Treat its kicker and highlight as optional: they turn into template chrome when every slide has them.

## Speaker notes

Only when asked. Then slides can carry fewer words and notes carry the full script: complete sentences as spoken, conversational, one entry per slide in order, roughly 200 to 400 words each, marking stress and pauses.

## Pitfalls that cost hours

- **Emoji vanish in export:** the headless browser has no colour emoji font; they render as empty boxes. Use plain symbols (✓ ✕ → · —) or words.
- **Fonts not loaded at capture:** webfonts need a few seconds before a screenshot or PDF, or the slide shows a fallback face. Wait, or host the fonts with the deck.
- **Overfull content slides:** a values slide with four blocks plus three creeds felt crushed; three blocks breathed.

## Before delivery

1. Open the index page: no broken images, fonts loaded.
2. Page through every slide: no blank slide, nothing misplaced or overflowing.
3. Print preview: exactly one page per slide, nothing cropped.
4. Reload on three random slides: the position is remembered.
5. Capture every slide and look at each one at full size (`huashu-polish-loop`).
6. Search for leftover placeholders and "TODO".

## The scripts (run later in Nomi's render worker)

- `export_deck_pdf.mjs`: prints each slide page of a multi-file deck to PDF in a real browser and merges them in filename order; vector text, zero visual loss, no changes to the HTML. (`export_deck_stage_pdf.mjs` is the variant for a single-file deck.)
- `export_deck_pptx.mjs`: exports a multi-file deck to an editable PPTX through `html2pptx.js`; only works on slides written to the four constraints.
- `html2pptx.js`: opens one slide in a real browser, measures every element's rendered position and style, and translates it into native PowerPoint text boxes, shapes, lines and pictures.
- `pptx_from_rendered.py`: the second PPTX route for decks already written visually or that must inherit a client template; reads rendered coordinates, needs no constraints, and checks afterwards for lost text.

The 20 slide styles (for the roulette and for ideas) are in `references/deck-styles.md`.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
