---
name: style-tactile-hardware
description: "Clean skeuomorphism: controls and panels that look made, with light from the top, a bright top edge, a darker lower edge, stacked shadows and pressed states that sink, either as a dark industrial shell on a light page or as soft light hardware."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Tactile Hardware

Clean skeuomorphism: controls and panels that look made, with light from the top, a bright top edge, a darker lower edge, stacked shadows and pressed states that sink, either as a dark industrial shell on a light page or as soft light hardware.

## The look

Two variants, same physics. **Dark shell:** a deep charcoal rounded console sits on a light page; inside, modules look moulded with inner highlights and inset wells; one signal colour lights status. **Light hardware:** pale blue-grey plastic surfaces, raised buttons and carved inputs. Either way it stays sharp and modern, never leather, chrome or puffy neumorphism.

## Suits / avoid

- **Suits:** kasir and POS apps, calculators (cicilan, ongkir, kalori), booking widgets, timers and queue screens, audio and coffee-gear shops, smart-home and farm IoT panels, small tools inside a Nomi app.
- **Avoid for:** long reading pages, editorial brands, salons and clinics wanting calm.

## Tokens

| Token | Dark shell | Light hardware | Role |
|---|---|---|---|
| `$color-background` | #E9E9E6 | #E9EEF5 | Page |
| `$color-sidebar` | #0A0A0C | — | Master console |
| `$color-card` / `-foreground` | #1C1C20 / #F5F5F5 | #F8FAFC / #334155 | Raised modules |
| `$color-muted` | #121215 | #D5DCE8 | Inset wells, pressed fill |
| `$color-foreground` | #F5F5F5 | #1E293B | Text |
| `$color-muted-foreground` | #A3A3A3 | #64748B | Labels |
| `$color-primary` / `-foreground` | #F5F5F5 / #0A0A0C | #334155 / #F8FAFC | Primary control |
| `$color-border` | #2A2A30 | #CFD7E4 | Module edges |
| `$color-input` | #3A3A40 | #CFD7E4 | Input edges |
| `$color-ring`, `$color-signal` | #F5A524 | #2563EB | Status light, active marker |
| `$color-success` | #3DDC84 | #16A34A | "Lunas", online |

The source's signal was cyan #22D3EE; on dark, cyan is a stock "futuristic" tell, so prefer an amber or the brand colour. Radii: `$radius-md` 8 · `$radius-lg` 12 (controls) · `$radius-xl` 16 · `$radius-2xl` 22 (panels), knobs 999. Fonts: `$font-heading` and `$font-body` Inter 300/400 or Geist, custom `$font-mono` Geist Mono for readouts.

## Depth recipes (shadow effects only)

- **Raised (dark):** inner shadow y 1 blur 0 #FFFFFF0F; drop shadow y 12 blur 24 #00000080.
- **Raised (light):** inner shadow y 1 blur 0 #FFFFFFB8; inner shadow y −1 blur 0 #4F5D7A3D; drop shadows y 18 blur 34 #1F29372E and y 5 blur 12 #1F29371F.
- **Pressed / inset well:** fill `$color-muted`; inner shadow y 2 blur 4 #00000099 (dark) or y 4 blur 10 #1F293738 (light); bottom inner highlight y −1 #FFFFFF0D.
- **Signal light:** 8px ellipse `$color-signal`, inner shadow y 1 #FFFFFF80, drop shadow 0 0 blur 8 at the signal colour, 50% alpha.
- Always light from the top. Tinted shadows, never pure black on light pages. One material per component.

## Layout signatures

1. **Console:** one master frame radius 22–28 on the light page, padding 24, drop shadow y 24 blur 48 #00000040.
2. **Split:** bold copy left on the page; stacked modules right inside the console.
3. **Modules:** readout well (mono 32px digits "Rp 249.000"), keypad of 44–56px raised keys in a 3×4 grid gap 8, status row with signal light ("Printer terhubung").
4. **Inputs:** carved wells, height 44, label above in 11px caps.

## Components

- Key / button: raised recipe, height 48, radius 12, label 15px/500; pressed state uses the inset recipe and moves 1px down.
- Toggle: track as inset well 44×24, knob raised 20px circle.
- Progress: inset well 6px tall, fill `$color-signal`.

## Copy voice

Short instrument labels and exact readouts. "Total", "Bayar", "Kembalian Rp 1.000". No marketing on controls.

## Tells

- Every surface has a bright top edge and a darker bottom; light comes from above.
- Pressed states are inset wells, not colour swaps.
- A dark console on a light page (or soft plastic on blue-grey).
- One small glowing status light as the only colour.
- Mono numeric readouts in recessed wells.

## Note for Koda

The source builds surfaces with vertical gradients and gradient borders; the canvas fakes them with inner highlight plus shadows. Koda should add a subtle top-to-bottom gradient on raised surfaces. Press transitions 160–240ms.

Adapted from mengto/skills: agent-skills/web-design/high-contrast-skeuomorphic-clean, agent-skills/web-design/skeuomorphic-ui (MIT, Copyright (c) 2026 Meng To).
