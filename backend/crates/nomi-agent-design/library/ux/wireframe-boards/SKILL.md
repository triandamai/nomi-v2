---
name: wireframe-boards
description: "How to draw a lo-fi flow board (phone frames in a row, numbered verb-first steps, solid connector paths, sticky notes) and an annotated screen (numbered pins plus a spec panel) in .pen terms. Read when a flow is unclear, the person asks for an alur, or a screen needs redline notes."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (design-templates/wireframe-mobile-flow/SKILL.md, design-templates/wireframe-annotated/SKILL.md)"
---
# Wireframe boards

Two lo-fi boards you can add to a preview as their own top-level frames. They show structure and flow, not polish: the flow board answers "how does a customer get from A to B", the annotated screen answers "what does each part do". They are notes for the person and Koda, never pages Koda builds; say so in the handoff.

## Shared rules

- Greys only, plus **one** accent used only for connectors, step numbers and pins. Board colours are canvas annotation colours: give them their own tokens (`$color-board-accent`, `$color-board-note`) so they stay apart from the brand's.
- Blocks must read at thumbnail size: content blocks `#D4D4D4` with a `#9A9A9A` 1px stroke on white screens, never near-white on white.
- Use the real headings and button labels you already wrote (Taste: copy before drawing); grey bars stand in only for running text. Photo spots are a grey rectangle with a diagonal cross (one `path`, geometry "M0 0L100 100M100 0L0 100", 1px stroke) and a 13px note ("Foto · kue lapis, top-down").
- Ids start with `wf-` so they never collide with the design's ids.
- Budget: a four-phone board is about 60–80 elements of the preview's 300. Keep greyboxes coarse.

## The flow board

A top-level frame named `Alur · Pesan kue`, white fill, `layout: "none"`, wide enough for its phones.

1. **Header** at the top: the title (28px, `$font-heading`), a tag frame "Wireframe v0.1 · Phone" with a 1px solid stroke and `rotation: -2` (upstream uses a dashed tag; dashes can't be drawn here), and a dateline at the right "26 Jul 2026 · 390 · lo-fi" in 13px.
2. **Phones in a row:** 3–4 frames 260 wide × about 540 tall, `cornerRadius: 32`, a 2px `#1A1A1A` stroke, a 20px status bar strip, 120 apart, all top-aligned. Each holds the greybox of one step: hero photo cross, title and price bars, list rows (thumbnail cross plus two bars), chips, the bottom bar, a sticky action bar, a confirmation tick.
3. **Connectors:** between each pair of phones a `path` about 100×24 in the gap, vertically centred on the phones: a line with an arrowhead, geometry "M0 12H96M84 4L96 12L84 20", stroke 2 in the accent, round caps and joins, no fill. Dashed arrows are not drawable; a solid line is the convention here.
4. **Step labels** above each connector: a 20×20 frame with `cornerRadius: 10` in the accent holding the step number (13/600, white, centred), then a verb-first label in 13/500: "1 Pilih kue", "2 Isi alamat", "3 Bayar QRIS", "4 Lihat status". The label names the tap that moves the flow on.
5. **Sticky notes:** 1–2 frames 180 wide, `layout: "vertical"`, padding 12, filled with the note tint (a pale yellow such as `#FCE9A6`), a soft shadow (y 4, blur 12, `#0000001A`), `rotation` between -2 and 2, placed beside the screen they talk about. One intent each, 14px: "Checkout cukup satu layar", "Ongkir tampil sebelum bayar".

Check: the flow reads left to right, every phone is whole at a glance, each connector has exactly one numbered label, the accent appears nowhere else.

## The annotated screen

A top-level frame named `Catatan · Beranda — Desktop`, `layout: "horizontal"`, gap 48, two children:

1. **The screen**, greyboxed (or the real screen when you annotate a finished design), inside a wrapper frame with `layout: "none"`, so the pins can sit on top at their own `x`/`y`. The screen content keeps its own vertical layout inside the wrapper at 0,0.
2. **Pins:** a 24×24 frame with `cornerRadius: 12`, `justifyContent` and `alignItems` centred, filled with the accent and holding its number (13/600, white) (an ellipse can't hold text), placed on the top-left corner of the region it marks, numbered in reading order top to bottom. One pin per region: nav, hero, each section, footer.
3. **The spec panel:** a frame 360 wide, `layout: "vertical"`, gap 16, padding 24, `#F4F4F4` fill, a title "Catatan" (20px), then one row per pin: the same accent number frame and a 14px note with numbers in it:
   - "1 Nav tetap di atas, 64 tinggi; di ponsel menjadi sheet penuh."
   - "2 Hero mengisi layar pertama; satu tombol 'Pesan lewat WhatsApp', 48 tinggi."
   - "3 Menu 2 kolom di ponsel, 4 di desktop; harga selalu tampil."
   - "4 Lokasi: jalan dan kecamatan saja, tombol 'Petunjuk arah'."
   - "5 Penutup dan footer satu bagian; tautan Kebijakan privasi."

Every pin has exactly one row and every row one pin. Annotate the sections your plan really has; the upstream template's logo strip and three feature cards are exactly what Taste bans on a place page.

## When to use which

- A new app with several screens, or a person who asks "alurnya gimana?": a flow board first, then the design.
- A finished screen whose behaviour needs explaining (sticky parts, what collapses on a phone, what is a link): an annotated screen beside it.
- Never instead of states: `ux-states` frames still go beside the real screens.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): design-templates/wireframe-mobile-flow/SKILL.md, design-templates/wireframe-annotated/SKILL.md. Changed: rewritten for the OpenPencil canvas.
