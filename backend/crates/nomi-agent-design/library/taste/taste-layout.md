# Taste: layout, screens and heroes

How to lay out a page as screens on the canvas: frame sizes, the content edges and 12-column grid, spacing, section heights and rhythm, and the eleven hero compositions as drawable frames.

## Screens on the canvas

A site page is one tall frame per width, auto layout vertical, gap 0, each child a section frame.

| | desktop | tablet | phone |
|---|---|---|---|
| Screen (judge tile) | 1440 x 900 | 768 x 1024 | 390 x 844 |
| Nav bar | 72 tall | 72 | 56 |
| Content width | 1280 at x 80-1360 (place: 1200 at x 120-1320) | 672 (gutter 48) | 342 at x 24-366 |
| Section frame | 900 or 1800 tall | 1024 or 2048 | 844 or 1688 (grows only if content truly needs it) |
| Section padding | top 120, bottom 72, sides 80 (place 120) | top 120, bottom 72, sides 48 | top 96, bottom 48 (+64 with the phone action bar), sides 24 |
| Content window in one screen | 708 (y 120-828) | up to 832 | 464-700 |

- The fixed nav covers the top of each section, so the h2 sits at y 120 (96 on a phone): 48px under the bar. Draw the nav once above the hero; the hero frame is then 828 tall (900 - 72) with top padding 48 and bottom 64 (phone: 788 tall, top 40, bottom 24), so the first screen ends exactly at 900.
- **Nav:** opaque solid fill, 1px bottom stroke in ink at 12-15%, same on every screen; no blur, transparency, shadow or floating pill. Logo on the left content edge, 4-6 one-word nouns at 15/500, the one primary button ending on the right edge, the same 44px height as every button (48 on a place). 640-1023: the primary stays beside a 44x44 menu button. Phone: logo left, menu button right; the menu is a full-screen sheet of 56px rows.
- **Phone action bar** (required on a place, optional elsewhere): fixed at the foot, ground fill, 1px top hairline, one 342x48 main action or two 165x48 buttons. Never on the first screen; add 64 to every bottom padding when it exists.
- Nothing else is fixed: no popups, cookie walls, chat bubbles, back-to-top squares, badges.

## Spacing

- Scale: 4, 8, 12, 16, 24, 32, 48, 64, 96, 128 (tokens: 4 `$space-1`, 8 `$space-2`, 12 `$space-3`, 16 `$space-4`, 24 `$space-6`, 32 `$space-8`, 48 `$space-12`; 64, 96, 120 and 128 as numbers). Every gap and padding comes from it.
- Heading to content 48 (32 on a phone). Cards 24 apart (12 on a phone). Short blocks merged into a section sit 64-96 under its content.
- One height per card row and per tile kind, page-wide.

## One section per screen

Every tile (the screen at y = k x 900) shows exactly one section. Plan the page as a list of screens before drawing. Content height c is from the h2's top to the last content's bottom:

| c at 1440 | Screens |
|---|---|
| under 360 | merge into the section above, or give it the picture it lacks |
| 360-495 | one screen, but weak: add its picture |
| 495-708 | one screen |
| 708-1,320 | no height fits: cut to 708, split into two sections (each 495+ with its own h2), or use two groups |
| 1,320-1,608 | two screens |
| over 1,608 | split |

- **Two groups** in a two-screen section: row 1 the h2 and the first group (360-708), row 2 the rest (420-708) with no second h2, starting 120 under the second screen's top, so screen two reads like any section.
- **Top-aligned:** the one h2 is the first content, at the padding top. Never centre content vertically, never a second h2.
- **Fill** 55% of the content window or more. A quote, logo row, stat row or FAQ that alone holds under 495 (464 on a phone) merges into the section above as its last element, no heading or ground of its own. No strip sections.
- **Separation** only by a full-width ground change or space: no rules, shadows or inner hairlines between sections, no inset coloured box standing in for a ground. A list may close each row with a 1px rule inside the content edges.
- **The close and the footer are one section** (one screen): row 1 the close, row 2 the links and legal row.
- **Tablet 768:** splits stack (text first, then the picture in what is left). Stacked content over 832 is cut to one screen (lower picture height, drop a fourth card) or becomes two screens there, with no card row crossing the screen line. Write "2 at 768" on the plan line when it differs.
- **Phone:** one column; a section is at least its screens and grows with what it holds, never overlapping; photographs after their text; buttons full width.

## Edges and the grid

- **One content width** for nav, every section and footer, the same all the way down.
- **12 columns, 24 gutter** (12 on a phone). Column starts at 1440: 80, 189, 297, 406, 515, 623, 732, 841, 949, 1058, 1167, 1275; spans 3 = 302, 4 = 410, 5 = 519, 6 = 628, 7 = 737, 8 = 846. Place (1200): starts 120, 222, 324, 426, 528, 630, 732, 834, 936, 1038, 1140, 1242; spans 3 = 282, 4 = 384, 5 = 486, 6 = 588, 7 = 690. Phone: one 342 column or two 165 tiles 12 apart; nothing narrower than 165.
- Every block starts on the left edge or a column start and ends on a column end; the last on the right edge. Anything wider than half the content runs the full width. Card rows (6+6, 4+4+4, 3+3+3+3): first on the left edge, last on the right, one height, every row full.
- **Two text lines only:** the left edge and the page's split line (column 5, 6 or 7: x 623, 732 or 841; a place always 732). Pick it once and use it on every split. Headings, closes and pricing heads start on the left edge; no centred block wider than half the content (the `centred` hero is the one exception).
- **Splits:** text left, visual right, top-aligned, on every split; never mirrored or alternating. Phone: text then visual, both 342.
- **Pictures** start and end on column lines (628, 410, 302 wide; place 588, 384, 282), one ratio per row (4:5, 4:3, 3:2, 1:1; a place no 3:2). No photo taller than 708 outside the hero.
- **Bleed** only as a layer behind a whole section (90%+ of it) or a full-width band; never out to one side.
- Nothing crosses a section edge, is sliced, staggered, peeks behind another, or leaves dead space beside short text.

## The hero

- **Exactly the first screen:** nothing of the next section shows at any width. Content ends by y 836.
- **Content:** an optional notice line (one 14px line with a number, 16 above the h1), the h1 (top at y 120, at most three lines, never smaller than an h2; 28-60 on a phone), the sub-line, at most two buttons (a place: one), and the picture. One plain claim, never a slogan. No caption, no second float card.
- **The picture ends inside the hero:** a layer behind, beside in a split, or below the words only if the whole hero ends by 876 (900 - 24). Never a picture sliced by the fold, at 1440, 768 or 390. At 768 a stacked picture ends 24+ above the hero's foot, or clearly bleeds (a photo filling its frame).
- **Phone:** text first; a photo above it only if the whole hero still ends by 820.
- **Docked strip** (software and product only): one row under the hero content, at most 96 tall: real customer logos the brief names, or three facts. Service and place dock nothing.

### The eleven hero compositions

Choose by kind and the material the brief really has; never the same as your last design. A product page uses only `product`, `photo` or `film`. `word` and `statement` only without a strong photograph. A place always uses the place hero.

| id | at 1440 | at 390 | motion note |
|---|---|---|---|
| `split` | text on span-5 (x 80-599); the picture on span-6 from x 732 to the right edge, its top level with the h1; one solid float card on its lower left | text, then the picture (240+ tall) | tech: the float card's dot pulses |
| `photo` | a full-bleed photo placeholder as the hero's layer; words on its calm, dark side across six columns, h1 80px on two lines, four short facts at the foot | the photo cut to the top, the words under it on the ground | none |
| `product` | head row: h1 on cols 1-7, two lines; lead and buttons stacked on cols 9-12, their foot level with the h1's; then the product big: a photo edge to edge down to the fold, square corners, the product 80%+ of the content width | stacked; the photo the screen width | none |
| `editorial` | the headline is the picture: 156px on three lines across the content, one or two photos set into the line as pills | 56px on four lines | tech: pills open on load |
| `collage` | text span-5; a framed panel span-6 from x 732 with the product window cut by its bottom, one result card | the window alone | tech: one card loop |
| `word` | oversized h1 144px on two lines, its last word live and underlined; lead and buttons span-6, four facts span-5 from x 841 | 54px on three lines | tech: the live word changes twice then rests |
| `film` | a cinema frame on the content edges holding the poster still, h1 in white on its calm part, a play disc at its foot | the frame, then the words | plays on a click |
| `statement` | one calm sentence at 56px on two lines on the page ground (tech: may be dark), lead, buttons, four facts | 34px on three lines | none |
| `triptych` | h1 52px on span-3; a collage of three to five pieces on span-6 from x 297; one large button ending on the right edge | collage 342 wide, button full width | tech: sets change every 2.8 s |
| `centred` | a pill, the h1 72px centred with the mark inline, lead, two pill buttons, then a framed product window across the content, fading over its last 18% | the window without its list column | tech: the window's story |
| place hero | one full-bleed still photo under the nav, 480 tall (y 72-552), nothing on it; under it the h1 48/53 on cols 1-6 (x 120-708, y 592, max three lines); from x 732 the sub-line 20/30 (what, where, today's hours, max 80 characters) and one filled 48px button 24 below; ends by about y 760. A shop may use a 1200 x 504 letterbox inside the column instead | photo 390 wide, 292-488 tall (grows to fill what the words leave), then h1 32/37, sub-line 18/28, one 342x48 button; ends at 820 | enters once |

Every hero: one screen, on the edge lines, one h1, one motion of its own at most. It fails with a picture out to one viewport edge, words over the busy part of a picture, a second float card, a slogan, or a caption.

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
