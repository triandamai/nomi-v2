---
name: motion-timing
description: "Durations, curves and springs for app motion, shorter on phones, what reduced motion keeps, and the exact one-line format for a motion note Koda can build from. Read when you annotate any interaction on an app screen."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (craft/animation-discipline.md)"
---
# Motion timing

The canvas is still, so motion is a note. Motion earns its place only when someone moves through space, time or state: a sheet opening, a screen changing, a save confirming. Never to decorate or to look premium. On marketing pages Taste's three layers (hero entrance, moments, hover) and its numbers in `taste-photos-motion` win; this file covers app screens and components.

## Durations

| Duration | Use |
|---|---|
| 50–100 ms | Press, toggle, hover feedback |
| 150 ms | Default state change (a check appears, a row selects) |
| 200–300 ms | Something enters: Dialog, Sheet, Dropdown Menu, Popover, Sonner toast |
| 300–500 ms | Screen to screen, a card growing into a page |
| over 500 ms | Only staged cross-screen moves, or Taste's hero entrance (about 1.1 s on a site); never a hero sequence on an app screen |

- Anything seen many times a session (hover, row select) stays at 200 ms or less.
- Phones run 20–30% shorter than desktop: a sheet 280 ms on desktop is about 210 ms on a phone.
- Exits run a little faster than entrances (a 250 ms dialog leaves in about 200 ms).

## Curve or spring

- **Curve** for opacity and colour, anything going between two known values: a standard ease-out that starts fast and settles (the common one is 0.2, 0, 0, 1).
- **Spring** for position, scale and anything dragged: critically damped (no bounce) for a business app; a little bounce (0.1–0.15) only for a playful brand.
- Motion confirms a change that already happened (the item is in the cart, then the badge moves); it never stands in for the change.

## Reduced motion

- Strip movement along an axis: slides, scales, rotations, parallax.
- Keep a crossfade of 150 ms or less where a state change still has to be seen.
- Every state change also shows without motion (colour, position, words).
- No flashing more than three times a second; loops stop when content lands; spinners stop at 60 s; anything moving past 5 s gets a pause.

## The motion note

Put it in a `motion-note` text frame beside the screen frame (outside the page) or in the handoff, one line per motion:

`Motion · <element id> · <trigger> · <what moves, from → to> · <duration> <curve or spring> (phone <duration>) · reduced: <substitute> · why: <purpose>`

Examples:
- `Motion · cart-sheet · tap "Keranjang" · slides up from below, 100% → 0 · 280ms ease-out (phone 210ms) · reduced: fades in 150ms · why: the cart comes from the bar they tapped`
- `Motion · order-row-status · payment confirmed · chip "Menunggu" → "Lunas", colour crossfade · 150ms ease-out · reduced: same · why: confirms the payment without a popup`
- `Motion · checkout-pay-button · tap · press to Active, then Loading "Memproses…" · 80ms, spinner until reply · reduced: no press scale · why: answers the tap at once`

## Folklore to ignore

- "Skeletons make waiting feel 11% faster": the study tested progress bars, not skeletons.
- "Doherty threshold = 400 ms": not in the paper.
- "Material 3's curve is 0.4, 0, 0.2, 1": that is the older Material 2 curve.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): craft/animation-discipline.md. Changed: rewritten for the OpenPencil canvas.
