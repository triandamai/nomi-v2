# The 36 page layouts

Every page type html-ppt ships, with its purpose, its structure (what's on the slide, top to bottom) and when you use it. Sizes and density caps are in the skill file.

The exact layout classes live in each layout's own CSS, not in the runtime, so you write that CSS in the deck (see `decks-runtime-notes`). **Export** flags what changes in the PDF or editable PPTX. "Tech" means tech-kind decks only (`taste-core`).

Every slide shares the same opening, with each part optional: a small label (sentence case, not tracked capitals), a title that makes a claim, and at most one lead line. Then comes the layout's body.

## Openers and transitions

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| cover | Opens the deck | Label (who and when), very large title, one lead line, a row of 2–3 fact pills (date, place, presenter). Footer with name and slide number | Always slide 1. On a place or product deck, pair it with a real photo (image-text-split or image-full-bleed as the cover) |
| toc | Shows the road | Title, then a 2×3 grid of numbered cards (number, short name, one line). The last card may span two columns | Decks of 10 or more slides with 3–5 sections. Skip it for short decks |
| section-divider | Turns a section | Centred: section number and name as a label, a big title (96–112), one lead line | Before each section in long decks. Skip html-ppt's accent rule under the title |

## Text

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| bullets | 3–5 points in order | Title, lead, then a single column of cards, each a short heading plus one line | The point really is a list ("3 alasan pelanggan balik lagi"). Never twice in a row |
| two-column | Concept beside example | Title, then two equal cards (left: the idea; right: the example or proof) | Explaining a rule with a case ("Paket hemat: apa isinya / contoh pesanan") |
| three-column | Three pillars | Title, then three equal cards, each with an icon, a short heading and one line | Three services, three values, three steps of equal weight. Use real icons, never emoji |
| big-quote | A voice | Centred serif quote (two lines at most), then the attribution | A real customer's words, with their permission. Never invented. A breathing slide after a dense one |

## Numbers and data

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| stat-highlight | One number lands | Centred: label, a huge figure with its unit (160–240), one line saying what it means, an optional source line | The single strongest real number ("1.200 porsi terjual per bulan"). Export: the counter must show the real value |
| kpi-grid | Status at a glance | Title, then 4 cards in a row, each with a label, figure and change (up or down, colour plus arrow) | Monthly or weekly reports, traction. 3–4 KPIs only |
| table | Exact comparison | Title, a card holding a table (header row, right-aligned numbers, one highlighted row) | Price lists, plan comparisons, schedules. ≤ 6 rows × 5 columns. Export: becomes a real table |
| chart-bar | Change or ranking | Title stating the finding, one large card with a bar chart | Sales per month, best sellers ranked. ≤ 8 bars, one series, the key bar in the accent. Export: draw with boxes and text, not a canvas |
| chart-line | Trend over time | Title, a card with 1–2 lines, the area under the main line tinted | Growth over 6–12 periods. Label line ends directly, no legend box |
| chart-pie | Share of a whole | Two columns: a doughnut on the left, a takeaways card on the right | Only with 2–4 parts. Otherwise use a sorted bar. Export: a picture unless built from shapes, so keep values as text in the takeaways |
| chart-radar | Two options on many axes | Title, a radar chart comparing 2 items on 6 axes | Rarely. Readers misjudge radar charts, so prefer a table or paired bars |

## Code and terminal (tech only)

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| code | Show real code | Title, a dark code card with syntax colouring | Developer audiences. ≤ 15 lines at 24 px or more. Export: a mono text box, colours may flatten |
| diff | Before and after in code | Title, a diff card (removed lines red, added green, context grey), one line under it | Showing a migration or fix |
| terminal | A command and its output | Title, a terminal window (title bar, prompt lines, output, cursor) | CLI tools and setup steps. Drop the traffic-light dots on export-heavy decks |

## Diagrams

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| flow-diagram | A pipeline | Title, 4–5 nodes in a row joined by arrows, one node highlighted | "Pesan di WhatsApp → bayar → dimasak → diantar". Keep arrows simple shapes |
| arch-diagram | Layers of a system | Title, 3 stacked tiers (name on the left, cells on the right), one tier highlighted | Tech: how a system is built. Off tech, use process-steps |
| process-steps | Numbered steps | Title, 4 step cards in a row (number, heading, one line, a small time or cost tag) | How to order, how a treatment works, onboarding. Off tech, the best diagram |
| mindmap | Ideas around a centre | Title, a root node with 5 smaller nodes on drawn curves | Workshops and brainstorm summaries. Export: curves become a picture, so keep node text as real text |

## Plans and comparisons

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| timeline | History or milestones | Title, a horizontal line with 5 dots, each with a year or date, a heading and one line | "2019 buka gerobak → 2026 tiga cabang". Real dates only |
| roadmap | What comes next | Title, 4 columns (Sekarang / Berikutnya / Nanti / Visi), each a list of tagged items, the current column emphasised | Pitch and plan decks |
| gantt | Parallel work over time | Title, a header of 12 weeks, 5 rows each with a label and a bar | Project or event plans. ≤ 6 rows |
| comparison | Before versus after in words | Title, two side cards (before muted, after positive) with a small "vs" mark between | Showing what changes for the customer. Its paired entry animations suit it |
| pros-cons | Honest trade-off | Title, two cards (pros, cons), each a short list | Choosing between options in a report or workshop |
| todo-checklist | What must be done | Title, 8 rows with a check state, task and tag. Done rows dimmed | Launch checklists, opening-day prep, workshop homework |

## Pictures

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| image-single | One picture, uncropped | Title, one contained frame (letterboxed), an optional short label | A screenshot, a menu board, a floor plan, a certificate |
| image-full-bleed | A photo carries the slide | The photo fills the slide, a dark scrim at the bottom, a label pill and title over it. No logo | The dish, the room, the team. The strongest slide in a place deck |
| image-text-split | A photo plus an argument | Two halves: the photo in a frame (with an optional corner tag) and the text column (title, 3 short points). It can flip sides | The most-used slide for products and services. Alternate sides through the deck |
| image-gallery | 3–6 photos | Title, a uniform grid of frames, each with a short content label | Menu highlights, salon results, portfolio. Mixed photo shapes are normalised by the frames |
| image-compare | Before and after | Title, two equal frames labelled Sebelum / Sesudah, a one-line conclusion under each | Renovations, hair results, redesigns. Real photos only |
| image-hero | Mood hero without a photo | Full-bleed slowly moving gradient with a title over it | Avoid. It is a gradient (lost in PPTX) and shows nothing real. Use image-full-bleed |
| image-grid | Bento of 7 cells | Title, an uneven 7-cell grid of placeholder tiles | Only when filled with real photos. Its placeholders are gradients |

## Closers

| Layout | Purpose | Structure | Use when |
|---|---|---|---|
| cta | The one action | Centred: label, big title naming the action, one line, one primary button plus one secondary | Every sales, menu and pitch deck. One action ("Pesan lewat WhatsApp", "Jadwalkan kunjungan"). No gradient headline |
| thanks | Ending | Centred very large word, presenter name and real contact line | After the CTA in live decks. Confetti only for celebrations. Never invent a phone number |

## Picking order

1. What is the one idea of this slide (`decks-storytelling`)?
2. What does the brief really have to show it: a photo, a number, a list, steps or a comparison?
3. Choose the layout by that material, not by variety for its own sake. Then check that it doesn't repeat the previous slide's layout.

Adapted from html-ppt-skill (MIT), and Anthropic's pptx skill (Apache-2.0). Changed: rewritten for Nomi's decks.
