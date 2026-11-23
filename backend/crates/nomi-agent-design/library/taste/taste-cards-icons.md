# Taste: cards, icons, radius and grounds

How to draw cards and their picture panels, which radius ladder a trade gets, how real brand icons and interface glyphs are drawn, diagram cards, and how Taste's backgrounds become solid grounds on the canvas.

## The card

- **Barely there:** fill `$color-card` (one clear step off the page: white on a #f3f3f1 page; on a dark tech page #151517 on #070708), or no box at all when the photo or a flat fill is the card. Stroke 1px inside in `$color-border` (ink at 8-14%). No blurred shadow at rest; never glass, never a gradient.
- **Padding** 32 at 1440 and 1024, 28 below, 24 on a phone; never under 24, text always inside it. A filled split panel pads 48.
- **Text:** title one line at 20 (18 phone), weight 500; 8 under it at most two lines at 16/24 in `$color-foreground`, max 46 characters wide. All running text in a card is 16/24 in the ink, never small grey text. Muted small text only for labels ("/ bulan", a tag).
- **Panel:** 32 under the last line. Default: the panel runs off the card's bottom edge, side edges on the text column, top corners 12, the lower 60-65% of the card (264-340 tall, one height for the whole group, e.g. 336). Variants: inset (8px margin all round, radius = card radius - 8, never under 6), flush (to the card's sides and bottom), top (8px margin, the lines below; tall cards), device (a status row on the card, then the panel as the screen).
- **Two levels, never three:** card > panel (a picture) > UI piece. The panel reads as a picture, never a box with its own grid, dots or ruling; never a card inside a card for text.
- **UI on a panel** (tech, or an app the client really runs): the product's real subject at true size (an invoice, a booking calendar, a chat, an order list), real names and amounts, 13px text and 12px meta (never under 11), weights 400/500, fill white (dark: #1c1d20), 1px border at 12%, radius 12. It is cut by the panel's bottom or right edge by 24+ (clip the panel frame), or keeps 24+ of ground on that side: never a 1-23px sliver, never centred floating with even ground all round, never a cut through the point (the row being worked on, the button, the number stays visible). The accent appears once per piece (the active row, the marked date); chips and numbers in ink tints; never a row of green dots. Vary placements across a group (corner, rising from the bottom, right-anchored, a low stack).
- **On product and service pages** a panel holds the client's photo placeholder, real icons or a still diagram on a flat `$color-muted` or `$color-background` panel. **A place has no feature cards.**
- **Rows:** 6+6, 4+4+4, 3+3+3+3 on the grid, the first card on the left edge, the last on the right, one height per row, every row full. A linked card is designed to lift 2px on hover (annotate); a card without a link does nothing.
- **Never:** tilted cards, carousels (rows show whole cards), a fourth line of text, noun titles ("Ketenangan pikiran"), a "sample" note, three identical feature cards in every section.

## Radius by trade

One ladder a page, never mixed. Set the tokens to these values (`$radius-sm` stays 4 for checkboxes):

| Ladder | Use for | `$radius-lg` buttons, inputs | `$radius-xl` inner panels, UI pieces | `$radius-2xl` cards, frames, photos |
|---|---|---|---|---|
| sharp | fintech, developer tools, law | 4 | 6 | 8 |
| soft (default) | most software, product, service | 8 | 12 | 20 (18 on a phone) |
| round | consumer apps | 10 | 16 | 24 |
| place | cafe, shop, guesthouse, salon | 2 | 0 | 0 (square photos and cards) |

- Software windows: one radius 12-20 with a 1px line. Product and service: one radius for every photo. Place: photos square-cornered, no border, shadow or frame.
- Buttons: one look a page, dealt from the kind's list: solid, pill (radius 9999), tonal (`$color-secondary` fill), arrow (a primary with a drawn arrow, the secondary a text link), block (square, 48 tall). 44 tall (48 on a place and in the phone bar), 16px label, 24 side padding, at most two in a row, full width on a phone. Exactly one filled button marks the main action in any view. Never slanted, gradient, ghost-only or shadowed buttons, or a third style. One `button-primary` and one `button-secondary` reusable component, placed with `ref` everywhere.

## Icons

- **Interface glyphs** (search, arrow, check, menu, cart): one outline line set (Lucide's geometry), drawn as `path` with one stroke width (1.5-2) and one size per role (16, 20, 24), in the text colour. Never a second set, never emoji as icons. The canvas does not load icon fonts, so never use `icon_font`.
- **Real brand and app icons** (house rule 7): a page that names a tool shows that tool's real logo in its own full colours (WhatsApp, Instagram, Shopee, Tokopedia, GoPay, QRIS, Google Sheets, Canva). Never a drawn look-alike, a letter in a circle or a generic glyph where a real logo exists. Draw a brand mark as a `path` only when you have its exact official geometry; otherwise draw its tile (a light square, 24-56px, radius `$radius-md`) with the brand's name in text and a canvas note inside the tile, `Icon · WhatsApp logo, full colour`, for Koda to fetch.
- A logo sits on a light tile on every ground (the tile keeps its colours readable on dark). Black marks (GitHub, Apple, X) turn white on a dark ground. One treatment per set: never colour marks beside one-colour marks. A one-colour row only for customer logos, footer socials, or a brand without a colour mark.
- Name only tools the product really works with; with real logos, the footer adds "Nama produk dan logo adalah merek dagang pemiliknya".
- A violet third-party logo keeps its colours; the violet ban is for your own palette. Xbox and Nintendo: name in type, never a mark.
- Sizes 16, 20, 24, 28, 32, 40, 48, 56. Integration grids, file cards and "works with" lists use real icons at 20-40, never a grey box. Nothing animates.

## Diagram cards

- When the point is how things connect (the tools it reads from, one input split into parallel jobs, many sources into one, two systems in sync, steps in order); a UI piece when the point is what the user sees. Never a diagram of a screen.
- Count: software and tech 0-2 a page, one a section; product and service 0-1, held still; place none; gaming only for a game system (matchmaking, saves).
- Layouts to draw with frames, ellipses and `path` lines: hub (a core tile, up to 8 spokes to tool tiles), ring (logos on an orbit), tree, lanes (one input, a core, three parallel lanes), fan-in / bow-tie, sync (two systems, records crossing), timeline, grid of files into a pipeline.
- Lines are hairlines; the core is an ink tile with the product's own name (never "AI engine" or "Orchestration layer"); tool nodes are real logos; labels one to three words (a tool by name, a step by its result: "Balasan terkirim"); real-looking numbers in meta; up to 12 nodes in a card. No caption: the card title says what it shows. On tech, annotate the pulse ("light runs core to spokes, 3.2 s cycle, restarts on hover"); on product and service it is still, no pattern behind it.

## Grounds

- Every section and panel uses `$color-background`, `$color-muted` or `$color-inverse` (and on product or place one brand band), changing in whole sections, never inside one, never by a rule between sections, never an inset box standing in for a band.
- Product, service and place: flat colour only. No gradient, grain, paper, halo, texture, pattern, glass, contour or shadow on the interface.
- Tech pages may carry up to two dealt grounds (mist, halo, ruled, dots, arcs, grain, dusk, band, field, horizon, deep) and one effect. The canvas cannot draw them: fill the section with the solid colour that carries the ground (its main tone) and annotate the ground by name for Koda ("Ground: halo behind the heading, accent-tinted, never violet"). Grid marks or dots only on one dark technical band, never behind a window, list or text. Tech panels behind UI take a flat light tone clearly lighter than a dark card (sand #d9ccb7, sky #c6d3df, sage #cdd6c6).

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
