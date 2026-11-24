---
name: decks-storytelling
description: "How a deck tells a story: one idea per slide, skeletons and slide counts for pitch, sales, menu or catalogue, report and workshop decks, numbers and charts, and speaker notes. Read when planning what a deck says, before choosing layouts."
license: "MIT, Copyright (c) 2026 lewis <sudolewis@gmail.com>; pptx guidance Apache-2.0 (Anthropic)"
source: "https://github.com/lewislulu/html-ppt-skill @fd1629067909ff55b36b905476de4e5f26062a1f"
---
# How a deck tells a story

This file covers how a deck tells a story: one idea per slide, skeletons and slide counts for each kind of deck, numbers and charts, and speaker notes. Plan the story first, then pick layouts and the look with `decks-html-ppt`.

## Start from what the deck is for

Write these five lines before any slide:

1. **The purpose:** the decision or action the deck should produce. For example: a bank approves a loan, an office orders catering, a customer books a treatment, the team agrees next month's plan, trainees can do X by the end.
2. **The audience** and what they already know: an investor, a bank officer, an office manager, a regular customer, staff.
3. **How it's used:**
   - A **live** deck has a speaker, so slides stay light and the notes carry the talk.
   - A **read-alone** deck is a PDF sent on WhatsApp or email. It must carry the whole story itself, so allow a bit more text per slide.
   - A **carousel** is swiped on Instagram: one idea per card, readable on a phone.
4. **The length:** minutes for a talk, cards for a carousel.
5. **The fact sheet:** what the person actually gave you (prices, menu, photos, numbers, dates, names). **Nothing else is invented** (`taste-copy`): no made-up revenue, customers, testimonials, logos or phone numbers. When a slide needs a number you don't have, leave the slide out, or ask for that number in one short question.

## The rules of the story

- **One message per slide.** If a slide needs "dan juga", it is two slides.
- **The title is the message.** Write a claim a stranger understands in five seconds ("Pelanggan kantor pesan 3x seminggu"), never a topic word ("Pelanggan").
- **Pyramid order for decisions:** the main message first, then the reasons (one section each), then the ask and next steps. A bank officer or a busy buyer should know the ask by slide 3. An investor pitch may hold the ask for the end, once traction is shown.
- **Match structure to size.**
  - A 5-slide deck needs no agenda or summary.
  - A 15-slide deck needs section dividers and a recap before the ask.
  - Read-alone decks also need a one-slide summary near the front.
- **Build toward one ending:** a decision, an order, a booking, agreement, a celebration or an open discussion. The close names that one action.
- **Every slide shows something real:** the product, the place, the people, a number, a chart or a diagram. Two text-only slides are never neighbours.
- **Vary the shape.** Don't put every point on a title-and-bullets slide. Let the material choose the layout (photo, number, steps, comparison).

## Slide counts

| Talk length | Talking from notes (2–3 min a slide) | Brisk, visual (1.5–2 min a slide) |
|---|---|---|
| 5 min | 3–4 | 4–6 |
| 10 min | 4–6 | 6–8 |
| 20 min | 7–10 | 10–14 |
| 30 min | 8–12 | 15–20 |
| 45 min | 12–16 | 20–26 |
| 60 min | 16–22 | 25–35 |

Leave time for questions: take 5–10 minutes off a 30-minute slot before counting. A read-alone PDF runs 8–15 slides. A carousel runs 5–10 cards.

## Skeletons

Each line below is one slide, with its layout from `references/layouts.md`. Drop any slide whose material the brief doesn't have.

### Pitch deck (investor, bank, partner): 10–12 slides

1. Cover (cover with a product or place photo): the name and one line saying what it is.
2. The problem, told as a customer's real situation (image-text-split or big-quote).
3. The solution: what you sell and how (image-text-split or process-steps).
4. The product shown big (image-full-bleed or image-single for an app screen).
5. The market: who buys and how many, with a source (stat-highlight).
6. The business model: price, margin, how money comes in (table or three-column).
7. Traction: real sales, orders or users over time (chart-bar or kpi-grid).
8. Competition and why you win (comparison or table).
9. The team: real people with real photos (image-gallery).
10. Plan and use of funds (roadmap, or chart-bar for the split).
11. The ask: the amount, the terms, and what it buys (stat-highlight, then cta).
12. Thanks with a contact (thanks).

For a bank or KUR loan, swap slides 5 and 8 for cash flow and a repayment plan (table, then timeline), and bring the ask forward to slide 3.

### Sales deck (an offer to a business client): 7–9 slides

For example, catering for an office, laundry for a guesthouse, or uniforms for a school.

1. Cover naming the client and the offer.
2. Their need, in their words (bullets or big-quote).
3. What you deliver (image-text-split with real photos).
4. How it works week to week (process-steps or timeline).
5. Proof: real past clients and volumes, only if given (kpi-grid or image-gallery).
6. Packages and prices (table, with 2–3 columns and the recommended column emphasised).
7. Terms: minimum order, delivery area, payment (bullets).
8. Next step (cta: "Coba seminggu", "Jadwalkan tasting").

### Menu or catalogue deck (warung, cafe, salon, shop): 6–12 slides

Shown on a TV at the counter, sent as a PDF, or posted as a carousel.

1. Cover: the name, a real photo, a one-line promise ("Masakan rumahan Jawa sejak 2009"), and the street and district.
2. Best sellers (image-full-bleed for the hero dish, or image-gallery for 3–6 dishes, each labelled with its name and price).
3. Each category gets one slide (Makanan, Minuman, Paket): a table, or image-text-split with a short list. Prices are written "Rp 25.000" or "25rb", one style throughout.
4. Packages or promos (three-column or table). Show a promo period only if one was given.
5. For a salon: treatments and duration (table), and before and after (image-compare, real photos with permission).
6. How to order or book (process-steps: "Pilih menu → chat → bayar → ambil/antar").
7. Hours and place (cta, with the street and district only, and the real contact if given).

On a TV loop, keep a fixed slide order, no animations, and at most 20 words per slide beyond prices.

### Report (monthly or weekly, to the owner, partners or team): 6–10 slides

1. Cover with the period ("Laporan Oktober 2026").
2. Summary: 3 lines of what happened and what's needed (bullets).
3. Key numbers against last period or target (kpi-grid).
4. The trend that matters (chart-bar or chart-line), with the title stating the finding.
5. What went well and what didn't (pros-cons or comparison).
6. Shipped or done (todo-checklist).
7. Problems and decisions needed (bullets or table: issue, impact, proposal).
8. Next period's plan (roadmap or table: what, who, when).

### Workshop or training: 8–16 slides per hour

1. Cover, then the goal ("Setelah sesi ini, kamu bisa…") as 3 objectives (bullets).
2. Agenda (toc).
3. For each topic: a section divider, then the concept (two-column), a worked example (image-single or code on tech), and an exercise (process-steps or todo-checklist).
4. A self-check: 3 questions (bullets).
5. Summary: the 3 objectives revisited, then homework and resources (cta).

### Product launch: 8–10 slides

Cover, the moment (big-quote or stat-highlight), the product shown big (image-full-bleed), 3 key features (three-column or image-text-split, repeated), how it works (process-steps), price (table), availability (cta), thanks.

## Numbers

- **Use only real numbers,** each with a unit, a period and, where useful, a source: "Rp 18,5 juta per bulan (rata-rata Jan–Okt 2026)".
- **Indonesian format:** thousands with a dot, decimals with a comma (1.250 porsi, 4,8 dari 5). Write "Rp 25.000", or "25rb" in casual menus. "Juta" and "miliar" are written out on slides.
- **One hero number per slide** on a stat slide. Make it mean something against a comparison ("naik 32% dari Juli", "2x target"). A number alone says little.
- **Round for the room** (Rp 18,5 juta, not Rp 18.473.250). Exact figures belong in tables and read-alone appendices.
- A number never breaks from its unit across lines, and never outgrows the slide's title by mistake. The stat slide is the one place a figure is the biggest thing.

## Charts

Pick the chart by the question it answers:

| Question | Chart |
|---|---|
| How did it change over time? | Bars (≤ 12 periods) or a line (many periods, or 2–3 series) |
| What's biggest? | Horizontal bars sorted largest first |
| What share of the whole? | A doughnut only for 2–4 parts. Otherwise sorted bars |
| How do two options compare on several points? | A table or paired bars. Avoid radar charts |
| Is the plan on time? | A gantt or timeline |

- **The title states the finding** ("Penjualan GoFood menyalip makan di tempat sejak Juni"). The chart proves it.
- **One series in one colour,** with the bar or point that makes the point in the accent and the rest muted.
- **Label directly** (values on bars, names at line ends) instead of a legend box. Use quiet gridlines, or none.
- **Same label style on every chart in the deck:** colour, size and font. Change it only where one chart needs it (a highlighted bar, a label on a dark bar).
- **Keep charts editable.** Build them from shapes and real text, so the values survive as text in PPTX. A canvas chart exports as a picture. Always write the key value in the title or a takeaway line too.

## Speaker notes

Write notes on every slide of a deck that will be presented. They go in the slide's notes element. The presenter sees them in presenter view and they travel into the PPTX speaker notes. They never appear on the slide.

1. **Cue signals, not a script.**
   - Short paragraphs of 1–3 sentences.
   - Bold the keywords so one glance recovers the thread.
   - Give each transition sentence its own paragraph ("Nah, terus gimana caranya?").
   - End by leading into the next slide.
2. **60–150 words per slide.** Fewer and the speaker stalls. More and nobody can scan them while talking (2–3 minutes a slide).
3. **Spoken, not written,** in the person's language and register. Read it aloud: it should sound like talking.

| Written (avoid) | Spoken (use) |
|---|---|
| Oleh karena itu | Jadi |
| Namun demikian | Tapi |
| Pihak kami akan melakukan | Kami bakal / kita akan |
| Melakukan optimalisasi | Kita rapikan |
| Dengan demikian dapat disimpulkan | Intinya |
| Adapun hal tersebut | (drop it) |

Example notes for a report's trend slide:

> **Bulan ini omzet Rp 18,5 juta**, naik dari Rp 14 juta di Juli.
>
> Yang dorong naik itu **pesanan GoFood**, sekarang lebih dari separuh penjualan.
>
> Tapi ongkos kemasan juga ikut naik. Itu yang kita bahas di slide berikut.

## Before handoff

- Read only the titles in order. They should tell the whole story by themselves.
- Every number is from the fact sheet, with its unit and period.
- Every slide has one message and something real to see. No two text-only slides sit together.
- The close names one action, and the contact details are real or left out.
- Notes are on every slide of a presented deck: spoken, 60–150 words, keywords bold.
- A read-alone PDF makes sense without a speaker. A carousel card reads on a phone at arm's length.

Adapted from html-ppt-skill (MIT), and Anthropic's pptx skill (Apache-2.0). Changed: rewritten for Nomi's decks.
