# Editable PPTX: two routes, never mixed

## Pick the route

| Situation | Route |
|---|---|
| The HTML isn't written yet | **Route A:** write every slide to the four constraints, export with `export_deck_pptx.mjs` (through `html2pptx.js`). Cleanest structure for later editing |
| The HTML is already written and visual-first (flex layouts, centred blocks, bare text, background images, drawn charts) | **Route B:** `pptx_from_rendered.py` reads the rendered coordinates; no rewrite |
| The client says "harus pakai template kami" | **Route B only**: Route A builds a file from nothing and can't inherit a master |

Never mix routes in one project: the two coordinate systems fight. Never downgrade a finished design to satisfy Route A; that is why Route B exists. Visual-first HTML pushed through Route A passed under 30% of slides in practice.

## Route A: the canvas size

PowerPoint measures in inches. Set each slide's page to **960pt × 540pt** (13.333 × 7.5 inches, PowerPoint's standard wide layout); the converter checks the page matches the layout within 0.1 inch. 720 × 405pt only for an old widescreen template. 1920 × 1080px makes a 20-inch slide where type looks oddly small when projected. Size is physical, not resolution: a bigger page doesn't make sharper text. (Route B follows the client template's size instead.)

## Route A: the four hard constraints

They mirror the PowerPoint file format itself (text lives in text frames; a shape and a text frame are different objects; arbitrary gradients aren't supported; pictures must be real image files). Don't expect a smarter tool; write the HTML to fit.

1. **No bare text inside a box element.** Every piece of text sits in a paragraph, a heading (levels 1 to 6) or a list. Text directly in a generic box is an error, and the error names the first 50 characters. Inline spans only restyle part of a paragraph (bold, colour, underline); they never become their own text box, and they take no margins.
2. **No CSS gradients**, and no background images at all on boxes: a gradient is a kind of background image. Use solid colours; to suggest a gradient, place a few solid blocks side by side.
3. **Backgrounds, borders and shadows only on box elements**, never on paragraphs, headings or lists. Put the coloured panel on a wrapping box and the text inside it.
4. **Pictures as real image elements**, never as a box's background image. To layer text over a picture, position the image and the text as two separate elements.

Also: no decorative text inserted by pseudo-elements (it can't be extracted); no web components or complex drawn vectors; keep large text at least half an inch above the bottom edge; content may not run past 540pt.

**Merging text boxes:** by default every paragraph becomes its own text box, so a card with three paragraphs edits awkwardly. Mark the card's wrapping box for merging and its paragraphs become one text box with paragraph breaks. Per paragraph it keeps size, colour, family, bold, italic, underline; alignment and line height come from the first paragraph (PowerPoint sets them per box), so don't merge paragraphs with different alignment. No nested merges; no bordered boxes inside a merged one.

**Fonts:** the converter measures text with the webfont; PowerPoint renders with the fonts installed on the machine. Differences cause overflow or drift; check every slide, and prefer fonts the recipient has.

## When the person insists on editable PPTX for a finished visual deck

Try Route B first. If Route B also can't serve, tell them plainly what each option costs:

> Desainnya memakai [gradien / ilustrasi rumit / animasi]; langsung jadi PowerPoint yang bisa diedit akan gagal. Dua pilihan:
> A. **PDF** (saran saya): tampilan 100% sama, bisa dibuka dan dicetak, teks tidak bisa diedit.
> B. **Saya susun ulang versi yang bisa diedit**: warna, susunan, dan teks tetap; gradien, ilustrasi rumit, dan bayangan disederhanakan.

If B: you rewrite (never ask the person to learn the constraints). Keep the colour system, hierarchy, copy, layout skeleton and rhythm; downgrade gradients to solid colours or blocks, complex drawings to images or simple shapes, shadows to nothing or a hairline, custom fonts toward common ones. Then give a before and after list ("judul gradien ungu → warna primer polos; bayangan kartu → garis 2pt"), and deliver both: the visual PDF and the editable PPTX. Advise against B when the deck's value is its animation, when it has more than 30 slides, or when it depends on precise drawn graphics.

## Route B: six traps when reading rendered coordinates

1. **Source whitespace becomes real line breaks.** Line breaks and indentation between tags are text nodes the browser collapses; PowerPoint doesn't. Collapse whitespace during extraction and drop empty runs.
2. **A container's computed size is often inherited** (16px from the page while the number inside is 132px). Base spacing and leading on the largest run inside.
3. **Counting lines by rectangle tops fails**: mixed sizes on one line give several rectangles with different tops. Cluster by overlapping vertical ranges.
4. **One box with several sizes can't share one line spacing**; adjacent lines collide. Split at each line break into separate text boxes (more boxes, but correct).
5. **Shrink-wrapped boxes have zero slack**, so a slightly different font metric pushes the last word to a new line. If the line didn't wrap in HTML, turn word wrap off for that box.
6. **Don't set exact line spacing on single-line paragraphs**: the measured line box is taller than the real leading and pushes text down. Set spacing only where text actually wrapped.

**Inheriting a client template:** open their file as the base, delete its sample slides but keep masters, layouts, theme and palette; add each slide on their official layout and remove the empty placeholders it brings. Don't draw shared elements such as the logo into the content: skip them in the HTML and let the layout provide them, so when the client edits the master every slide follows. Watch for a hidden full-slide solid rectangle in a layout, same colour as the ground, that covers whatever sits under it; remove it. A 1920px HTML canvas on a 1920pt slide gives a scale of exactly 1.

**Fonts:** a brand font you don't have installed (a commercial system font) stays in the PPTX only; lay out the HTML and PDF in a font you really have, or the layout jumps.

**Verify:** never trust quick-look thumbnails; check in a real presentation engine and in the target app. The script compares the page's rendered text with the PPTX text slide by slide and reports missing pieces automatically, because lost text fails silently; also compare image and shape counts per slide.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
