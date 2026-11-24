# Style: Tech Green Dark

A calm operational dark mode for dashboards and tools: near-black surfaces in three steps, framed modules with corner brackets and mono system labels, and one controlled green as the signal for live, active and progress.

## The look

A structured module grid on black. Panels lift by tone, not glow; thin strokes and small corner brackets give each one an instrumented feel. Mono codes sit in header rails. Charts are grey with green only on the active series. Green underglow appears under one or two key modules, never washing the page.

## Suits / avoid

- **Suits:** dashboards for small teams (orders, stock, cashflow, queue), IoT monitoring (aquaculture ponds, greenhouses, solar), IT and network services, delivery and courier tracking, esports teams.
- **Avoid for:** food, beauty, wellness, heritage; first-time customers who need warmth.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #080A0F | Page |
| `$color-foreground` | #F2F5F7 | Text |
| `$color-card` / `-foreground` | #10131A / #F2F5F7 | Modules |
| `$color-popover` | #121620 | Raised panels, menus |
| `$color-muted` | #171B24 | Chart tracks, table header |
| `$color-muted-foreground` | #929AA7 | Labels, secondary |
| `$color-primary` / `-foreground` | #91FF4F / #0A1205 | Primary button, active series |
| `$color-accent` / `-foreground` | #91FF4F1A / #C8FFA6 | Selected row, active nav |
| `$color-border` | #FFFFFF24 | Strokes (14%) |
| `$color-ring` | #91FF4F | Focus |
| `$color-warning` / `-foreground` | #F5B84233 / #F5B842 | Low stock, delays |
| `$color-destructive` | #FF5A5F | Errors |

Radii: `$radius-sm` 4 · `$radius-md` 6 · `$radius-lg` 8 · `$radius-xl` 12 (modules). Fonts: `$font-heading` and `$font-body` Inter 400–700 (or Geist), custom `$font-mono` JetBrains Mono.

## Type

- Page title 32–48px, 700, letter spacing −6%. Big numbers 32–40px/600, tabular.
- Module titles 15px/600. Body and table 13–14px/20.
- Mono labels 9–11px uppercase, tracking 12–16%, `$color-muted-foreground` ("SYS-02 · KOLAM A").

## Shape, spacing, depth

- Module: radius 12, fill `$color-card`, stroke 1 `$color-border`, padding 20, inner shadow y 1 #FFFFFF0A.
- Corner brackets: 10px L-marks (10×1 and 1×10 rectangles) in `$color-primary` at 50% on the top-left and bottom-right of key modules only.
- Underglow for at most two modules: drop shadow y 8 blur 32 #91FF4F1F.
- Gap 16 between modules, page padding 24 (16 phone). Optional faint page guides: 1px #FFFFFF0A vertical lines at the content edges.

## Layout signatures

1. **Header rail:** mono section code left, date "26 JUL 2026 · 08.40" and a live dot right (only if data is live), bottom stroke.
2. **Stat row:** 3–4 modules: mono label, big number, one-line delta ("+12 pesanan dari kemarin"), footer status row.
3. **Main module (span 8):** chart with grey bars, the current day in `$color-primary`; axis labels in mono.
4. **Side module (span 4):** list of alerts with icon + label + status ("Stok gula pasir tinggal 3 kg").
5. **Table:** header `$color-muted`, rows 44 tall, selected row `$color-accent`.

Phone: stat row becomes a 2×2 grid; the chart module goes full width; the table becomes a list of rows.

## Components

- Primary: fill `$color-primary`, text `$color-primary-foreground`, height 40, radius 8, 14px/600. Secondary: stroke 1 `$color-border`, text `$color-foreground`.
- Status chip: radius 999, 11px mono, fill at 10% of its colour.
- Progress: 4px track `$color-muted`, fill `$color-primary`.

## Copy voice

Operational, short, numbers first. "3 pesanan menunggu kurir." No hacker jargon.

## Tells

- Three dark tones carrying panel hierarchy with thin strokes.
- Green only on active data, primary action and progress.
- Mono system codes in header rails and labels.
- Small green corner brackets on the key modules.
- A soft green underglow under one or two modules, nowhere else.

## Note for Koda

Card shells in the source use faint glass and gradient borders; canvas uses solid fills and a coloured drop shadow. Koda may add a 1px gradient border on key modules. Live dots pulse only if data is truly live.

Adapted from mengto/skills: agent-skills/web-design/tech-green-dark-mode-modern (MIT, Copyright (c) 2026 Meng To).
