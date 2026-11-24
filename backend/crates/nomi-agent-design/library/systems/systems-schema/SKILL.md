---
name: systems-schema
description: "open-design's shared semantic token set mapped to Sora's $color, $space, $radius and $font names, and the steps to turn any package in the collection into a .pen design system frame; read it before building tokens from a system card."
license: "Apache-2.0 (open-design, Copyright (c) 2026 nexu-io); MIT, Copyright (c) 2026 VoltAgent"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/design-systems/_schema; https://github.com/VoltAgent/awesome-design-md (fcc7028, 28 Oct 2026)"
---
# Systems: the token schema

open-design's shared semantic token set mapped to your `$color`, `$space`, `$radius` and `$font` names, and the steps to turn any package into a `.pen` design system frame. Read it before building tokens from a system card.

## The shared contract

Every one of open-design's 152 packages ships a `tokens.css` declaring the same token names, so one structure covers them all. The schema sorts tokens by who decides the value:

| Layer | Who decides | Tokens |
|---|---|---|
| A1 identity | the brand, required | `--bg`, `--fg`, `--accent`, `--font-display`, `--font-body` |
| A1 structure | the brand, required | type scale `--text-xs`…`--text-4xl`, `--leading-body`, `--leading-tight`, `--tracking-display`, `--container-max`, `--section-y-desktop/tablet/phone`, gutters |
| A2 | the brand, with a schema default | `--accent-on`, `--accent-hover`, `--accent-active`, `--success`, `--warn`, `--danger`, `--font-mono`, `--space-1`…`--space-12`, `--radius-sm/md/lg/pill`, `--elev-flat/ring/raised`, `--focus-ring`, `--motion-fast/base`, `--ease-standard` |
| B slot | the brand, or an alias to a sibling | `--surface`, `--surface-warm`, `--fg-2`, `--muted`, `--meta`, `--border`, `--border-soft` |
| C extension | one brand only | e.g. tom-modern's `--tm-code-bg`, kami's tag tints |

The A2 defaults: accent label #ffffff, hover and active as the accent mixed 8% and 14% with black, success #16a34a, warn #eab308, danger #dc2626, a 4px spacing ladder (4, 8, 12, 16, 20, 24, 32, 48), radius 8/12/16 plus pill, elevation none / a 1px border ring / a soft 2px-8px shadow, focus a 3px accent ring at 30%, motion 150 and 200ms.

## Mapping to your names

open-design's "accent" is shadcn's **primary**, not `$color-accent` (the same trap Taste names in `taste-color`).

| open-design | Sora token(s) | Notes |
|---|---|---|
| `--bg` | `$color-background`, `$color-popover` | |
| `--surface` | `$color-card` | On a cream page, card is a step off the page, never pure white (Taste) |
| `--surface-warm` | `$color-muted`, `$color-secondary`, `$color-accent` | When aliased to surface, use surface |
| `--fg` | `$color-foreground`, `$color-card-foreground`, `$color-popover-foreground`, `$color-accent-foreground` | Holds 7:1 for reading text |
| `--fg-2` | `$color-secondary-foreground`, or a lead text style in foreground | Taste keeps one ink; a second tier is a style, not a colour, unless it passes 7:1 |
| `--muted`, `--meta` | `$color-muted-foreground` | 4.5:1 on background and on muted |
| `--border`, `--border-soft` | `$color-border`, `$color-input` | Hex with alpha is allowed here |
| `--accent` | `$color-primary`, `$color-ring` | Under 3:1 on the ground: primary = ink, the accent becomes `$color-brand` |
| `--accent-on` | `$color-primary-foreground` | 4.5:1 on primary |
| `--accent-hover`, `--accent-active` | the Hover and Active frames of the state matrix | No token: draw them as their own component states (`components-kit`) |
| `--danger` / `--success` / `--warn` | `$color-destructive` / `$color-success`+`-foreground` / `$color-warning`+`-foreground` | Success and warning are custom pairs: tell Koda |
| `--font-display` / `--font-body` / `--font-mono` | `$font-heading` / `$font-body` / `$font-mono` | Real Google Fonts from `taste-type`, never the package's licensed face |
| `--text-*`, `--leading-*`, `--tracking-display` | text styles: caption, small, body, lead, h3, h2, h1, display | Taste's sizes win; keep the package's ratio between levels |
| `--space-1`…`--space-12` | `$space-1`…`$space-12` | Same 4px ladder |
| `--radius-sm` / `--radius-md` / `--radius-lg` / `--radius-pill` | `$radius-lg` (controls) / `$radius-xl` (panels) / `$radius-2xl` (cards) / pill 9999 | Derive `$radius-sm` and `$radius-md` as the steps below; zero-radius systems set every step to 0 |
| `--elev-flat` / `--elev-ring` / `--elev-raised` | flat / hairline / soft | Add a deep level for dialogs; shadows are `effect` entries, blur only, no glass |
| `--focus-ring` | Focus state: 3px `$color-ring` at about 50% outside the control | |
| `--motion-*`, `--ease-standard` | a one-line motion note in the handoff | Not drawn |
| `--container-max`, `--section-y-*`, gutters | frame widths (1440, 768, 390), section padding, side padding | |

## Turning a package into a .pen design system frame

1. **Pick the reading.** Use the card's Tokens line (already mapped and checked), or for a hand-written system the author's own palette. Then replace the accent and ground with the person's colours.
2. **Write the variables** in `canvas.pen`, following `pen-format`: every `color-*` with a light value and a dark value (dark is its own set: card a step lighter than background, border and input low-alpha white, destructive lighter). Then `space-1`…`space-12`, `radius-sm`…`radius-2xl`, `font-heading`, `font-body`, and `font-mono` only for tech.
3. **Recheck contrast** with the new values: foreground 7:1, muted-foreground 4.5:1, label on primary 4.5:1, primary 3:1 on the ground, destructive 4.5:1, in both themes.
4. **Draw the foundations frame** (`Design system — Foundations`): a swatch row per role, each swatch captioned with its token and hex; the type styles at real size with line height; the radius ladder; the three elevations.
5. **Draw the components frame** with `components-kit`: Button, Input, Card and one overlay at minimum, each as reusable `<Component> / <Variant> / <State>` frames with token fills only. Carry the system's habits: pill or square buttons, hairline or shadow cards, a hard offset shadow for tom-modern, identical hover for kami.
6. **Note what didn't transfer** in a caption for Koda: the licensed face you replaced, an effect the canvas can't draw, a swapped violet.

## References

| File | Holds |
|---|---|
| `references/schema-design-md.md` | VoltAgent's DESIGN.md format: the Stitch frontmatter (colors, typography, rounded, spacing, components) and the nine sections, and how open-design's package format relates to it |
| `references/schema-writing-design-md.md` | How you write a DESIGN.md for an approved design, so Koda can build it in SvelteKit, Tailwind v4 and shadcn-svelte |

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
