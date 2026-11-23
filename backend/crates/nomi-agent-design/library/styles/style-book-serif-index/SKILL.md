---
name: style-book-serif-index
description: "An archival reader: a warm paper book spread set inside a dark catalogue frame, serif-led pages, mono index rails for navigation, drop caps, margin notes and folio numbers, for businesses whose story or catalogue rewards slow reading."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Book Serif Index

An archival reader: a warm paper book spread set inside a dark catalogue frame, serif-led pages, mono index rails for navigation, drop caps, margin notes and folio numbers, for businesses whose story or catalogue rewards slow reading.

## The look

The screen is two zones. Outside: a near-black shell with thin frame lines, a mono sidebar index and tiny corner marks. Inside: an open book, two paper pages side by side with a visible centre crease. Typography leads; colour is restrained to paper, ink and one antique accent. It feels collected and scholarly, never dusty.

## Suits / avoid

- **Suits:** restaurant menus presented as a book, recipe collections, batik, tenun and craft catalogues with provenance, heritage brands, local history and tourism stories, writers and poets, course modules and study notes, wedding love-story pages.
- **Avoid for:** fast checkout, booking flows, dashboards, anything mobile-first and time-pressed.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F0E6D2 | Left page |
| `$color-card` / `-foreground` | #EADFC8 / #2A2119 | Right page (one step darker) |
| `$color-foreground` | #2A2119 | Body and headings |
| `$color-muted-foreground` | #6B5B48 | Captions, folios |
| `$color-border` | #CDBB98 | Rules, plate borders |
| `$color-primary` / `-foreground` | #2A2119 / #F0E6D2 | Checkout button |
| `$color-ring`, `$color-signal` | #7A2E2A | Oxblood: drop caps, active index, notes (or antique gold #A8873F) |
| `$color-sidebar` | #14110E | Outer dark shell |
| `$color-sidebar-foreground` | #D9CFBF | Index text on shell |
| `$color-sidebar-muted` | #8A7F70 | Inactive index items |
| `$color-sidebar-border` | #FFFFFF1A | Frame lines, rails |

Radii: `$radius-sm` 2 · `$radius-md` 4 (book spread); nothing rounder. Fonts: `$font-heading` Cormorant Garamond, `$font-body` EB Garamond, custom `$font-mono` JetBrains Mono.

## Type

- Display: **Cormorant Garamond** 500–600, 40–64px, line height 1.05.
- Body: **EB Garamond** 400, 17–18px/30, `$color-foreground`, measure 55–65 characters.
- Utility: **JetBrains Mono** 400, 10–11px uppercase, tracking 12–15%, for index, metadata and folios.
- Drop cap: display serif, 4 lines tall (about 96px), `$color-signal`.

## Shape, spacing, depth

- Shell: full frame, padding 24, inner frame stroke 1 `$color-sidebar-border` with 8px corner marks (two 8×1 and 1×8 rectangles per corner, `$color-sidebar-foreground` at 40%).
- Book spread: cornerRadius 4, drop shadow y 24, blur 48, #00000066.
- Crease: 1px vertical rectangle #0000002E at the page join, plus an inner shadow on each page toward the crease (left page x −14 blur 28 #0000001F; right page x 14 blur 28 #0000001F) to suggest the fold.
- Page padding 56 desktop, 24 phone. Paragraph gap 16.
- Image plates: stroke 1 `$color-border`, padding 8, caption below in mono.

## Layout signatures

1. **Index sidebar:** vertical frame 240 wide on the shell, groups of mono headings ("BAB I · ASAL USUL") with entries below; active entry `$color-signal` with a 4px square marker, others `$color-sidebar-muted`.
2. **Book spread:** horizontal frame, two pages fill_container, no gap, crease between. Left page: chapter label in mono, display heading, drop-cap paragraph. Right page: image plate (placeholder "Foto: kain tenun ikat Sumba, motif kuda, cahaya samping"), pull quote, margin note.
3. **Margin note:** small frame on the page edge, stroke 1 `$color-border` dashed, 13px italic serif, `$color-muted-foreground`.
4. **Folio footer:** mono 10px at each page bottom, "12" left, "Koleksi Rumah Tenun · 2026" right.

On phone: one page at a time; the index becomes a top mono row of chapter tabs; the crease disappears.

## Components

- Primary action: text link in display serif italic with a 1px `$color-signal` underline ("Pesan kain ini →"), or a small `$color-foreground` filled button radius 2 for checkout.
- Chapter tab: mono 11px; active underline 2px `$color-signal`.
- Price line: serif, dotted leader to the right "Rp 1.250.000".

## Copy voice

Measured, descriptive, a little literary, always factual. "Ditenun dua bulan oleh Ibu Rambu di Waingapu." Chapter names as an index: "Bab II — Pewarna Alami".

## Tells

- A dark shell with a light open book in it; the crease is visible.
- Mono index rail beside serif pages.
- Drop caps and folio numbers on pages.
- Image plates with thin borders and mono captions.
- One oxblood or antique-gold accent, nothing else coloured.

## Note for Koda

The source uses paper gradients, grain and edge shading; the canvas uses two flat page tones plus inner shadows at the crease. Koda may add a faint paper texture (low enough that text reads the same with it off) and slow masked text reveals.

Adapted from mengto/skills: agent-skills/web-design/book-serif-index (MIT, Copyright (c) 2026 Meng To).
