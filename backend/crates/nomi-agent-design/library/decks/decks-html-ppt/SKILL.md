---
name: decks-html-ppt
description: "How you build an HTML slide deck the html-ppt way: deck structure, the 36 page layouts, slide rhythm, type and density per slide, presenter mode and animation intent, and what breaks on PDF/PPTX export. Read before writing any deck."
license: "MIT, Copyright (c) 2026 lewis <sudolewis@gmail.com>; pptx guidance Apache-2.0 (Anthropic)"
source: "https://github.com/lewislulu/html-ppt-skill @fd1629067909ff55b36b905476de4e5f26062a1f"
---
# Building a deck the html-ppt way

This file covers how you build an HTML slide deck the html-ppt way. It walks through the deck structure, the 36 page layouts, slide rhythm, type and density per slide, presenter mode and animation intent, and what breaks when a deck is exported to PDF or PPTX.

You write the deck as HTML. Nomi serves the deck runtime once (navigation, presenter view, themes, animations). The person presents the deck in the Design Studio, and a render worker exports it to PDF and editable PPTX. Read `decks-storytelling` for what the deck says, and `decks-runtime-notes` for the exact class and attribute names. The catalogues are `references/layouts.md` (all 36 page layouts) and `references/themes.md` (all 36 themes).

Taste still rules: plain words, real photographs, no invented numbers, one accent, client colours first (`taste-core`, `taste-type`, `taste-color`). Where an html-ppt habit disagrees with Taste or with the export rules below, Taste and export win. For example, html-ppt uses tracked uppercase kickers, gradient text, violet gradients and Inter. You use none of them.

## Before you build: three decisions

1. **Content, audience, how it's shown.** What the deck is for, who watches, and how long. Then how it travels: live on a screen (a TV in the warung, a projector at a bank), sent as a PDF on WhatsApp, or edited by someone in PowerPoint or Google Slides. How it travels decides the export rules you follow.
2. **Look.** Choose one theme from `references/themes.md` as a mood, then express it in Nomi tokens (`--color-*`, `--font-heading`, `--font-body`, `--radius-*`, `--space-*`). Choose it by the kind (`taste-core`) and the client's own colours. Never pick it because it feels nice.
3. **Skeleton.** Start from the closest scenario skeleton in `decks-storytelling` (pitch, sales, menu or catalogue, report, workshop, launch). Then pick one layout per slide.

If the brief is thin, ask one question at most, normally about the audience and how the deck is shown. Otherwise propose a direction and build it.

## Deck structure

- **One file per deck** is the default: one `.deck` holding one `section.slide` per page, with a `data-title` on each slide. Navigation, the overview and presenter view only work inside one file. Use a single-slide file (one active slide in `body.single`) only for a standalone render.
- **Fixed canvas.** Slides are designed at **1920×1080** and the runtime scales them, never reflows them. A pixel therefore means the same thing on a laptop, in the presenter preview, in a thumbnail and in the export. All sizes below are px on that canvas. For an Instagram carousel, set the canvas to 1080×1350 (4:5) or 1080×1440 (3:4).
- **Skeleton:** cover, then an optional agenda, then a section divider, then 2–4 body slides, repeating for each section, then a close (the action) and a final slide. A short deck (under 8 slides) skips the agenda and dividers.
- **Chrome is declared once.** Footer, slide number and progress come from the runtime. Declare the logo once on `body` and turn it off on the cover and on full-bleed photo slides. Never paste a logo onto every slide.
- **Audience text only on a slide.** Anything addressed to the presenter ("di slide ini jelaskan…") goes in the slide's notes, never as small grey text on the slide.

## The 36 layouts at a glance

The brief mentioned 31 layouts. The repo at this commit has 36. Each one is in `references/layouts.md` with its purpose, structure and when to use it.

| Group | Layouts |
|---|---|
| Openers and transitions | cover, toc, section-divider |
| Text | bullets, two-column, three-column, big-quote |
| Numbers and data | stat-highlight, kpi-grid, table, chart-bar, chart-line, chart-pie, chart-radar |
| Code (tech decks only) | code, diff, terminal |
| Diagrams | flow-diagram, arch-diagram, process-steps, mindmap |
| Plans and comparisons | timeline, roadmap, gantt, comparison, pros-cons, todo-checklist |
| Pictures | image-single, image-full-bleed, image-text-split, image-gallery, image-compare, image-hero, image-grid |
| Closers | cta, thanks |

Picking a layout is covered at the end of `references/layouts.md`.

## Slide rhythm

- **Never use the same layout twice in a row.** Slides of the same type keep their title at the same position, size and alignment.
- **Alternate density.** Follow a dense slide (table, kpi-grid, chart) with a breathing slide (a stat, a quote, a full-bleed photo).
- **Use a section divider** before each major section in a deck of 10 or more slides. In shorter decks, drop the dividers.
- **Pace:** about 1.5–2 minutes per slide when talking freely, and 2–3 minutes when reading from notes.
- **Light and dark.** A deck for a place, product or service stays light throughout. A tech deck may run dark, or use a dark cover and dark close around light content.
- **Every slide carries something visible:** a photo, a chart, a number, a diagram or real UI. A title with bullets alone is the exception, at most once in a row.

## Type and density per slide (1920×1080)

| Element | Size / line height | Notes |
|---|---|---|
| Cover title | 104–128 / 1.05 | 1–2 lines, tightened about -3% |
| Section divider title | 96–112 / 1.05 | One short phrase |
| Slide title | 56–64 / 1.1 | At most 2 lines and about 45 characters. Write a claim, not a topic: "Omzet naik 3x sejak buka GoFood", not "Omzet" |
| Sub-heading, card title | 36–40 / 1.2 | |
| Body, list items | 28–32 / 1.4 | Left-aligned. Centre only titles and single statements |
| Small, caption, axis label | 22–24 | The floor. Nothing on a slide is under 22 |
| Label | 22, weight 500 | Sentence case, never tracked capitals |
| Stat figure | 160–240 / 1 | The number and its unit stay together |

On a 13.33-inch PowerPoint slide these sizes become 14–16 pt body and 10–12 pt captions, which is the PPTX floor. **Never shrink text to fit.** Cut words or split the slide instead.

- **Faces** come from the `taste-type` pairings (display face plus text face, never three families). The html-ppt themes' Inter, Space Grotesk and Archivo Black are mood references only. Weights follow Taste: size carries the headline, not weight 800.
- **Density caps:**

  | Content | Cap |
  |---|---|
  | Bullets | ≤ 5 items, ≤ 12 words each |
  | Cards | 3–4, each a title plus ≤ 20 words |
  | KPIs | 3–4 |
  | Table | ≤ 6 rows × 5 columns |
  | Chart | 1 per slide, ≤ 8 bars or 3 lines |
  | Words per slide, live | ≤ 40 |
  | Words per slide, read-alone PDF | ≤ 80 |

- **Space:** slide padding 72 top and bottom, 96 left and right (at least half an inch on any export). Keep text at least 72 from the bottom edge. Choose one gap between blocks (24, 32 or 48, from your `--space-*` scale) and keep it all through the deck.
- **One accent,** used for one thing per slide: the key number, the active step, the main button.

## Theme

A theme is a set of token values and nothing else. `references/themes.md` gives each theme's palette roles with hex values, its fonts, mood and best use, and how its roles map onto Nomi's tokens.

- One colour dominates (60–70% of the slide area), with 1–2 supporting tones and one sharp accent. Write the palette and the reason it fits the trade on line 2 of your plan.
- Text on an accent fill uses the accent's foreground token (`--color-primary-foreground`). Accents run from white to black across themes, so a fixed white or black is unreadable on some of them.
- The client's own colours (logo, shopfront paint, uniform) replace the theme's accent.

## Pictures

- Put every image in an image frame. A bare image ignores the slide height and pushes content off the page.
- **Cropping:** photos fill and crop (`cover`). Screenshots, menus, price lists, logos and diagrams letterbox (`contain`) and are never cropped.
- **Layout by picture count:** one photo carrying the slide uses image-full-bleed, with a dark scrim under the title. A photo plus an argument uses image-text-split. 3–6 photos use image-gallery. Before and after use image-compare.
- **Real photographs** of the client's food, room, team and work. A label that is content (a dish name and its price) is fine. A sentence describing the photo is not.
- Keep images next to the deck file and reference them relatively.

## Presenter mode and notes (intent)

The runtime gives the presenter:

- **Two-window presenting.** Press S to open a second window with four movable cards: the current slide, the next slide, the notes (large type) and a timer with slide count. Both windows stay in sync, so the audience screen goes fullscreen with F while the presenter keeps the notes.
- **Quick tools.** N opens a notes drawer. O opens an overview grid, useful for spotting clipped slides. T cycles through 3–5 themes you list, so the person can audition looks live.
- **Notes per slide** are cue signals, not a script (`decks-storytelling` gives the rules). The notes travel into the PPTX speaker notes.

When a deck will be presented with notes, write notes on every slide.

## Animation vocabulary (intent, no code)

Entry animations play each time a slide becomes active. **The runtime has no step-by-step reveals (fragments).** To reveal something in steps, split it into consecutive slides.

| Intent | Use | Avoid on Nomi decks |
|---|---|---|
| A title arrives | rise-in, fade-up; blur-in on a cover | glitch-in, typewriter |
| A list or grid builds | stagger-list (children one by one) | |
| A number lands | counter (counts up to the real value) | counter-explosion outside tech |
| Two sides compare | fade-right and fade-left on the two columns | card-flip-3d |
| A diagram draws | path-draw | morph-shape |
| A section turns | quiet fade only | cube-rotate-3d, page-turn-3d, perspective-zoom |
| A celebration close | confetti-burst, only when the deck celebrates (a launch, an award) | |
| Ambient effects | none off tech | neon-glow, gradient-flow, marquee, kenburns, every canvas FX |

- **One accent animation per slide,** plus at most one stagger. Everything else stays still.
- **Canvas FX** (particles, starfields, network graphs) are allowed only on tech-kind decks, at most one per deck (`taste-core`, rule 8). A warung, salon or bengkel deck gets photographs instead.
- **Motion never carries content.** Every slide must read completely as a still frame, because PDF and PPTX get the still frame. Reduced-motion settings switch animations off, and you never override that.

## What breaks on export (PDF and editable PPTX)

The PDF keeps almost everything visual but nothing moves. In the editable PPTX, each element becomes a PowerPoint text box, shape or picture, and anything PowerPoint can't express is dropped or flattened. **Write every deck editable-safe by default.** It costs the live deck nothing.

| In the HTML | PDF | Editable PPTX | Your rule |
|---|---|---|---|
| Gradient fill or gradient text | Kept | Blocked or dropped | Solid fills only. A gradient look goes in only as a picture |
| `background-image` on a box | Kept | Dropped | Every picture is a real image element |
| Text loose in a `div`, or inside a picture, canvas or SVG | Kept | Not editable, or lost | Every word sits in a heading, paragraph or list item |
| Background, border or shadow on a heading or paragraph | Kept | Error, or split apart | Put the fill or border on a wrapping box. The text element carries only text |
| Glass blur, blend modes, glow, text shadow | Mostly kept | Lost | Don't rely on them. The slide must work without them |
| Colours with alpha | Kept | Unreliable | Use solid lighter tints rather than transparency |
| Shadows | Kept | Only soft, downward ones survive | Use a faint downward shadow or none. Never an upward shadow |
| Web fonts | Embedded | Shown only if the viewer has them installed | Give a fallback stack ending in Arial or Calibri. Leave about 10% slack in text boxes |
| Animations, FX, video, iframes, live widgets | Final frame | Final frame or nothing | The still frame must be complete. A counter shows its real number |
| Canvas charts (Chart.js) | Picture | Picture | Draw simple charts from boxes, with labels and values as real text |
| Typed "•" bullets | Fine | Double bullets | Use real lists |
| Line breaks or indents between inline tags in the source | Fine | Become real line breaks | Keep one line's text runs on one source line |
| Text past the slide edge, or within 72 px of the bottom | Clipped | Flagged or cut | Cut words or split the slide |
| Header, footer, progress bar | Hidden in print | Hidden | Nothing important lives in the chrome. The logo prints on every page |
| Notes | Not printed | Speaker notes | Notes always go in the notes element |

These come from PowerPoint's own limits, as recorded in Anthropic's pptx guidance:

- No accent line under a title, and no decorative stripe along a slide or card edge (html-ppt's divider-accent and accent-top cards are out).
- Never centre body text.
- Chart labels share one colour, size and font across every chart.
- Never leave text overflowing its box.
- Never default to cream or beige when no ground has been chosen. Use white or the brand ground.

**Before handoff:** check every slide in the overview as a still frame. Nothing is under 22 px, every word is live text, no gradient or background-image is left, notes are on every presented slide, and titles of same-type slides line up.

Adapted from html-ppt-skill (MIT), and Anthropic's pptx skill (Apache-2.0). Changed: rewritten for Nomi's decks.
