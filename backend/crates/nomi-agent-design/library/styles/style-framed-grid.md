# Style: Framed Grid

A precise, technical light layout: every section is a rectangular frame on one 12-column grid, with one border colour, L-shaped corner brackets of one size, equal gaps, and a single hot accent, so price lists and schedules read like a well-made spec sheet.

## The look

Pale grey-white page, thin 1px frames snapped to a strict grid, crisp L-brackets at every frame corner. Headlines are large and light with tight tracking; everything else is small and orderly. Nothing floats, nothing is soft. One red-orange accent marks the live or chosen thing.

## Suits / avoid

- **Suits:** price lists and menus (warung, bengkel service menu, laundry kiloan), class and workshop schedules, event rundowns, catalogues, spec comparison, small-team dashboards and reports.
- **Avoid for:** luxury, wellness, romance, kids' brands; anything that needs warmth or softness.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F7F7F4 | Page |
| `$color-foreground` | #141414 | Text |
| `$color-card` / `-foreground` | #FFFFFF9E / #141414 | Frame surface (white at 62%) |
| `$color-muted` | #EDEDEA | Header rows, inactive tabs |
| `$color-muted-foreground` | #777777 | Labels, secondary |
| `$color-primary` / `-foreground` | #141414 / #F7F7F4 | Primary button |
| `$color-border` | #18181B24 | Frame lines (14%) |
| `$color-bracket` | #18181B57 | Corner brackets (34%) |
| `$color-ring`, `$color-signal` | #FF3A1A | Live dot, chosen row, one highlight |
| `$color-destructive` | #C21F0A | Errors |

Radii: all 0 (frames are rectangles); only status dots are round. Fonts: `$font-heading` and `$font-body` Inter (300 for display, 400–500 for UI), or Geist for a less default voice.

## Type

- Display 56–96px, weight 300, letter spacing −5% to −6%, line height 1.0.
- Frame titles 18px/500. Body 14–15px/22. Labels 11px uppercase, tracking 8%.
- Prices and times use tabular figures, right-aligned.

## Shape, spacing, depth

- Page padding 16 (`$space-4`); gap between frames 16; frame padding 24 desktop, 16 phone.
- Frame: stroke 1 inside `$color-border`, no shadow, no radius.
- Brackets: at each corner two rectangles, 18×1 and 1×18, `$color-bracket`. Make one reusable bracket-corner component and place four per frame; same size everywhere.
- One border weight on the whole page.

## Layout signatures

1. **Parent grid:** 12 columns; frames span 12, 8+4, 6+6 or 4+4+4. Every frame edge lines up with frames above and below. Phone: all span 12.
2. **Title frame (span 12):** display headline left, a 4-column block right with address, hours, and one button.
3. **Price list frame (span 8):** rows of name / note / price, each with a bottom hairline; group headings in caps ("SERVIS RINGAN").
4. **Side frame (span 4):** "Antrean sekarang: 3 motor" with a `$color-signal` dot, or opening hours.
5. **Schedule frame (span 6+6):** day columns with time rows.

## Components

- Buttons: height 44, radius 0, primary filled `$color-primary`; secondary stroke 1 `$color-foreground`.
- Status dot: 6px `$color-signal` with drop shadow 0 0 blur 10 #FF3A1A99 (the only glow).
- Tabs: text 13px; active has 2px `$color-foreground` bottom stroke.

## Copy voice

Terse labels, exact numbers. "Ganti oli + cek rem · 30 mnt · Rp 85.000". Headings name things ("Daftar harga"), never sell.

## Tells

- Every section is a 1px rectangle on one shared grid; edges align across rows.
- L-shaped corner brackets of identical size on every frame.
- Very light, very large display type next to tiny caps labels.
- Equal 16px gaps everywhere; no floating cards, no shadows.
- One red-orange dot or row as the only colour.

## Note for Koda

The source adds a 135° diagonal stripe texture under 5% opacity on the page; skip it on canvas, Koda may add it. The layout must read the same without it.

Adapted from mengto/skills: agent-skills/web-design/framed-grid-layout (MIT, Copyright (c) 2026 Meng To).
