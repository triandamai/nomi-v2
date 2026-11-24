---
name: style-nested-agency
description: "Clean agency look built from container hierarchy: a white page shell with visible side rails and corner squares, a large dark rounded block nested inside it, and lighter cards inset within that block, so packages and services feel curated."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Nested Agency

Clean agency look built from container hierarchy: a white page shell with visible side rails and corner squares, a large dark rounded block nested inside it, and lighter cards inset within that block, so packages and services feel curated.

## The look

Three levels, always in this order: outer light shell (rails at its edges), one major dark section with a big radius, inner cards inside it. Hierarchy comes from that nesting and the jump from light to deep navy, not from colour. Headlines are clean sans with one serif italic phrase; a soft periwinkle accent appears only on pills and the key button.

## Suits / avoid

- **Suits:** digital and social-media agencies, consultants, accountants and tax services, wedding organisers with packages, photography packages, IT services, training providers with tiers.
- **Avoid for:** single-product shops, menus, dense data; anything with fewer than three things to group.

## Tokens

| Token | Light | Role |
|---|---|---|
| `$color-background` | #F4F4F1 | Page outside the shell |
| `$color-foreground` | #1A2440 | Text on light (deep navy ink) |
| `$color-card` / `-foreground` | #FFFFFF / #1A2440 | Outer shell |
| `$color-chapter` / `-foreground` | #0F1424 / #F4F4F1 | Nested dark block |
| `$color-chapter-card` | #182036 | Cards inside the dark block |
| `$color-chapter-border` | #FFFFFF14 | Strokes inside the dark block |
| `$color-muted-foreground` | #5B6478 | Body on light (on dark use #A9B0C2) |
| `$color-primary` / `-foreground` | #0A3BBF / #FFFFFF | Primary button |
| `$color-accent` / `-foreground` | #DCE3FF / #0A3BBF | Pills, active dot |
| `$color-border` | #1A24401A | Rails, shell stroke |
| `$color-ring` | #0A3BBF | Focus |

Radii: `$radius-md` 8 · `$radius-lg` 12 · `$radius-xl` 20 (inner cards) · `$radius-2xl` 32 (dark block); pills 999. Fonts: `$font-heading` and `$font-body` Inter 500/400, with Newsreader italic 400 for one phrase per heading at most.

## Type

- Headline 48–64px, 500, letter spacing −3%; one phrase in Newsreader italic ("yang *benar-benar* jalan").
- Section 32px/500. Body 16px/26. Labels 11px uppercase, tracking 10%, used once per block.
- Prices 32px/500 with a 14px unit: "Rp 4.500.000 / bulan".

## Shape, spacing, depth

- Outer shell: max width 1200, fill `$color-card`, left and right strokes only (`$color-border`), 6px squares `$color-foreground` at 30% on the four corners; padding 24.
- Dark block: inset 24 from the shell, radius 32, padding 48 (24 phone).
- Inner cards: radius 20, fill `$color-chapter-card`, stroke 1 `$color-chapter-border`, inner shadow y 1 blur 0 #FFFFFF0F.
- Gaps: 24 between inner cards, 64–96 between shell sections. Max nesting: 3 levels. Never a card inside an inner card.

## Layout signatures

1. **Hero on the shell:** left-aligned headline, 2-line body, primary + secondary buttons, a pill above only if it says something new ("Slot Agustus tinggal 2").
2. **Dark package block:** heading row (title left, monthly/yearly toggle right), then 3 inner cards in a row: name, who it is for (1 line), price, 4–5 outcomes, button. The recommended card gets stroke 1 `$color-primary` and a `$color-accent` pill.
3. **Work strip:** on the light shell, 2–3 project placeholders with captions.
4. **Closing block:** smaller dark rounded block with one ask and the contact "halo@agensikopi.example".

## Components

- Primary: fill `$color-primary`, height 44, radius 999, 14px/500. Secondary on dark: stroke 1 #FFFFFF33, text `$color-chapter-foreground`.
- Toggle: two-segment pill, active segment `$color-card` on `$color-chapter-card`.

## Copy voice

Clear, outcome-first, not hype. "Konten 12 kali sebulan, dijadwalkan dan dilaporkan." Plan names describe the buyer: "Usaha baru", "Usaha tumbuh".

## Tells

- Outer light shell with visible side rails and corner squares.
- One big deep-navy rounded block nested inside it.
- Inner cards with faint strokes inside that block, never deeper.
- A single serif italic phrase inside a sans headline.
- Accent only on pills and the primary button.

## Note for Koda

The source uses gradient-border wrappers on inner cards; canvas uses a 1px faint stroke plus a top inner highlight. Koda may use a 1px white-to-transparent gradient border.

Adapted from mengto/skills: agent-skills/web-design/nested-container-clean-agency, agent-skills/web-design/nested-container-frames (MIT, Copyright (c) 2026 Meng To).
