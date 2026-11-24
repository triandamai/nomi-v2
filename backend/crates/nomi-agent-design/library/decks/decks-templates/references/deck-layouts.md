# Deck slide layouts

The slide layouts you reuse across any deck look, each with what it is for, how it sits on the 1920x1080 grid and how to draw it as a frame.

Pick a layout by what the slide must say, not by variety. The same layout may repeat in different chapters. If a slide feels empty, the layout is wrong: pick another, don't add filler. Grid terms (margins 96, content stops at y 964, footer rail 988-1040, 12 columns) are from the main skill.

## Openers and dividers

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Cover** | Slide 1 | Kicker (mono, event or date), title 120-160 on two lines max, one-line subtitle, meta row at the bottom (presenter · place · date) | Hero light or brand colour. Title in the display face; meta row as a horizontal frame on the footer rail. 8-12 nodes |
| **Cover with picture** | Product, place, campaign | 7+5 split: copy left, a photo placeholder right, full height inside margins | Placeholder 4:5 or 3:4 with a caption note |
| **Chapter / act divider** | Start of each chapter | Big numeral (`01`, `II`) or a giant word, chapter title 88-120, one line of intro | Hero dark usually; a faint giant numeral behind can be a large text node in `$color-muted-foreground` at low opacity |
| **Question** | Act close, opening a debate | One question at 120, broken by meaning, nothing else | Hero slide, centred block |
| **Statement** | The one sentence to remember | Sentence at 88-120 across 8-9 columns, generous empty space, a hairline and a small note below | Light; left-aligned, top third |

## Numbers

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Big number** | One hard fact | A number at 160-220, a label under it, a source line | Hero dark or light; number in the display face, unit smaller |
| **KPI row** | 3-6 numbers side by side | Equal cells: label (mono) · number (64-88) · delta (mono, accent only if it matters) · one-line note | Reusable `KPI cell` component placed with `ref`; hairline dividers, no boxes or icons |
| **Hero metric + supporting** | One headline number with context | 7+5: big number left, 3 small metrics stacked right | Light |
| **KPI ledger** | Many numbers read top to bottom | Rows with hairlines: big number left, label middle, small icon or delta right | Light; good for 4-6 figures |
| **Bar ranking** | Ranking or share | Horizontal bars, length = value, value at the end, the one that matters in the accent | Rectangles with token fills; label column left |
| **Chart slide** | A trend or comparison | Title states the takeaway; chart takes 8-12 columns; note column 4 if needed | Bars as rectangles, a line as a `path`; axis labels small mono |

## Explaining

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Body slide** | Default content | Kicker · title (64-88) · lead paragraph (28-36, max 3 lines) | Light; 8 columns wide, right side empty |
| **Three points** | Three reasons, offers, pillars | Title, then 4+4+4 columns: number or small icon · short title · two-line description | Reusable column component; icons as `path`, one stroke width |
| **Six cells / four cards** | Capabilities, menu groups, features | 2x3 or 1x4 equal cells: number · title · one line | Cards only if the look uses them; otherwise hairline grid |
| **Pipeline / steps** | A process (order to delivery) | 3-5 numbered steps left to right: `№1` · step title · description | Horizontal frame of step components; build note if steps reveal one by one |
| **Timeline** | History, roadmap | A hairline axis with evenly spaced nodes; date above, step below. Vertical version: axis at left, year + KPI + text right | Ellipses for nodes, a rectangle 1-2 px tall for the axis |
| **Before / after** | Old way vs new way, problem vs solution | 6+6 split with a vertical hairline; left column muted (opacity ~0.55), right full strength | Two column frames; labels `Dulu` / `Sekarang` |
| **Four options matrix** | Strategy choices | 2x2 grid; axis labels; the recommended quadrant inverted (dark fill) | Four equal frames, one with the dark fill |
| **Loop or system** | A cycle or layered system | Numbered steps left; concentric rings or stacked bands right with labels | Ellipses with strokes only, labels as text |
| **Memo statement** | Pre-read, confidential note | One serif sentence 64-96 across the left half; mono meta bar at top; no boxes | Light; space does the work |

## Pictures and proof

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Quote + image** | Testimonial, story | Copy 7 columns (kicker, quote, name and role), photo 5 columns at 16:10 or 4:3 | Use only a real quote from the person; photo placeholder with caption |
| **Big quote** | The punchline, a founder line | Quote at 88-120 in the display face, attribution and date below | Hero slide, nothing else on it |
| **Image grid** | Several products, outlets, screenshots | 3x1 or 3x2 images at one fixed height, a caption line under each (slide captions are fine on decks) | Equal placeholder frames, one tone, each with its own note |
| **Lead image + side text** | Information-heavy story | 8+4: big image left, kicker/title/body/callout right (or the reverse with a 3:4 portrait) | One large placeholder |
| **Gallery plate** | Portfolio, craft, art | Black band top and bottom with three-column caps meta; the work fills the middle edge to edge | Two dark rectangles + one placeholder; no accent colour |
| **Device or screenshot** | App or website demo | A flat device or browser frame centred, 2-3 callouts with hairline leaders | Draw the frame flat; the screen is a real UI drawing or a placeholder |

## Tables and finance

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Data table** | Prices, comparisons, comps | Header row in mono caps, hairline rows, numbers right-aligned, total/median row bold, the subject row tinted | Rows as a reusable component; at most ~8 rows |
| **Sensitivity grid** | Two-variable what-if | 5x5 grid of values, the base case outlined | Small cells, tabular figures |
| **Valuation range ("football field")** | Ranges from several methods | Horizontal bars on one shared axis, a vertical tick for the current value | Rectangles on a common scale; label column left |
| **Use of funds / the ask** | Pitch close | 6+6: the amount and what it buys (bars or a short table) left, team or milestones right | Light; the amount in the display face |

## Closers

| Layout | For | Structure | Drawing it |
|---|---|---|---|
| **Closing / call to action** | Last content slide | One line of what to do next, 1-2 buttons or a contact line (WhatsApp link written as the person gave it, site on `.example` if stand-in) | Hero dark or light; buttons as kit `ref`s |
| **Closing manifesto** | Brand or culture decks | Accent block left with a short manifesto; three takeaways right | Two column frames |
| **End plate** | Final slide | One giant word ("Terima kasih", the brand name) and a small colophon | Hero slide, 3-5 nodes |

## Default sequences

- **6 slides:** Cover (hero light) · Body (light) · Big number (hero dark) · Three points (light) · Big quote (hero light) · Closing (hero dark).
- **10 slides:** Cover · Body · Big number · Three points · Pipeline (dark) · Big quote · Before/after · Big number 2 · KPI row or team · Closing.
- **Weekly update:** Cover · Statement + number · Shipped (list) · In flight · Blocked + ask · Chart slide · Asks · End plate.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus guizang-ppt as bundled in open-design (MIT, Copyright (c) 2026 op7418), html-anything deck templates (Apache-2.0) and the pitch-book workflow shape of anthropics/financial-services (Apache-2.0). Changed: rewritten for the OpenPencil canvas.
