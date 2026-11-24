---
name: craft-style-industrial-brutalist
description: "A style card for Industrial Brutalism in two modes, Swiss print (light) and tactical telemetry (dark): rigid visible grids, square corners, giant type against dense small data, one hazard red. Read it when offering directions to workshops, hardware, data tools or bold editorial brands."
license: "MIT, Copyright (c) 2026 Leonxlnx (Taste Skill family: brutalist-skill), via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (industrial-brutalist-ui, as vendored in open-design@5fdcc3d, 2026)"
---
# Style: Industrial Brutalist

A raw, mechanical direction built from a visible grid, square corners, huge type set against tight clusters of small data, and a single hazard red; it comes in two modes you never mix, a light Swiss print mode and a dark telemetry mode for technical products.

This is not `style-documentary-brutalist` (photo collage, billboard caps, black and warm-white cuts for barbershops and streetwear). This one is engineered: blueprints, manuals, instrument panels.

## Pick one mode per project

| | Swiss industrial print | Tactical telemetry |
|---|---|---|
| Ground | Unbleached paper #F2F1EC | Dead screen #0E0E0E |
| Ink | Carbon #111111 | Phosphor white #EAEAEA |
| Accent | Hazard red #D7261E | Hazard red #FF3B30 |
| Optional | none | One terminal green #4AF626 element (one live readout or status) |
| Type | Heavy neo-grotesk display, plain sans body | Mono for data and labels, grotesk display |
| Suits (Taste kind) | Bengkel, contractors, printers, furniture makers, hardware shops, architects, a bold editorial magazine (product, service) | Software, developer tools, monitoring, IoT, drones, fintech back-offices (software, tech) |

Taste decides the mode: a dark page is allowed only on software, gaming and tech hardware, and mono only where the design shows code or technical data. A bengkel or a printer gets the print mode, never telemetry.

## Tokens (print mode; telemetry swaps ground and ink)

| Token | Value | Role |
|---|---|---|
| `$color-background` | #F2F1EC | Paper |
| `$color-foreground` | #111111 | Ink, rules, display type |
| `$color-card` / `-foreground` | #F2F1EC / #111111 | Cells are the ground itself |
| `$color-muted-foreground` | #4D4B46 | Data labels (7:1 on paper) |
| `$color-primary` / `-foreground` | #111111 / #F2F1EC | Button |
| `$color-signal` | #D7261E | Strike-through, one thick rule, one vital number |
| `$color-border` | #111111 | Grid rules, 1px; section rules 2px |
| `$color-ring` | #D7261E | Focus |

All radii 0. No shadows at all.

## Type

- Display: Archivo (wdth axis, condensed end) or Mona Sans from Taste's table, at very large sizes with tracking about -4% and line height 0.9.
- **Taste wins on weight and case:** the source wants uppercase black weights. Taste sets headlines at 400 (500 on tech) in sentence case. The brutal mass comes from size, condensed width and tight tracking, not from bold or capitals. Archivo at 400 and 900px tall still reads as a block.
- Data and labels: telemetry uses JetBrains Mono or IBM Plex Mono 13/500 in sentence case (Taste bans tracked caps); print mode uses the body sans at 13/500.
- Optional serif disruption (Source Serif 4) once, for a pulled quote. The source's halftone or dithered serif can't be drawn; set it plain.

## Layout signatures

1. **Blueprint grid:** a 12-column grid drawn as 1px rules; content sits in the cells. Draw cells as frames with a stroke on the sides that need a rule (`thickness: { "right": 1, "bottom": 1 }`) so lines never double.
2. **Bimodal density:** each section pairs one vast area (a giant numeral, a two-word headline, a full-bleed photo of the workshop) with one tight cluster of data rows (spec, unit, value).
3. **Full-width rules** (2px ink) separate operating units; a single thick red rule may mark the one critical section.
4. **Crosshairs** (`+`, 12px paths) at a few grid intersections, and registration marks in a corner of the hero. Use them sparingly: they are decoration, and `slop-audit` asks each one for a job (alignment cue, section start).
5. **Spec rows:** label left in muted, value right in ink, tabular numbers, a hairline under each: "Kapasitas mesin · 2.400 lembar/jam".

## Components

- Button: 44 tall, radius 0, ink fill, label 15/500; hover note: fill flips to red. Secondary: 1px ink outline.
- Status (telemetry): a 8px square in green or red plus a mono label: "Aktif · 14:02 WIB".
- Data table: no zebra, hairline rows, numbers right-aligned in mono.

## What the canvas draws instead

| Source effect | Canvas | Koda |
|---|---|---|
| CRT scanlines, phosphor glow, noise over the page | Flat ground | Not on print mode; on telemetry it counts as the page's one effect and must be annotated (`taste-photos-motion`) |
| Halftone or dithered photos | Plain photo placeholder with its note | Taste forbids photo filters |
| ASCII framing `[ LAYANAN ]`, `>>>` | Not used: Taste counts them as fake system markers | Do not add |
| Random strings (REV 2.6, UNIT/D-01) as decoration | Only real codes from the brief (a real model number, a real batch) | Never invent |

## Copy voice

Short, factual, numbered where order matters. "Cetak offset, potong, jilid. Selesai Jumat." Real units, real tolerances, real hours.

## Tells

- A visible grid of 1px rules and square corners everywhere.
- One enormous element per section beside a tight block of small data.
- Exactly one red, used like a hazard marking.

Adapted from Taste Skill's industrial-brutalist-ui (MIT, Copyright (c) 2026 Leonxlnx), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
