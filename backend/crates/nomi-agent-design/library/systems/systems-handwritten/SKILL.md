---
name: systems-handwritten
description: "The ten hand-written systems (Neutral Modern, Warm Editorial, kami, Tom Modern, Atelier Zero, HUD, Mission Control, Trading Terminal, Urdu Modern, Cloudflare Kumo) with tokens checked; read it for a principled starter or a print, RTL or dense-app direction."
license: "Apache-2.0 (open-design, Copyright (c) 2026 nexu-io); MIT, Copyright (c) 2026 Tw93 (kami); MIT, Copyright (c) 2026 Tom-Opencart (tom-modern); MIT, Copyright (c) 2026 Cloudflare, Inc. (kumo)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/design-systems; https://github.com/tw93/kami; https://github.com/Tom-Opencart/tom-modern-html-style-rule; https://github.com/cloudflare/kumo (0c58325, 2.9.2)"
---
# Systems: hand-written

The ten systems a person wrote rather than synced from a catalogue, with their tokens checked. Read it when you want a principled starter, a printed-feeling direction, a right-to-left page, or a dense app screen.

## Why these are different

The brand and generic packages were read off websites or generated in bulk. These were authored with intent. Each states rules (what never to do, how many accents, which radius range), and those rules are the most useful part. Several ship a tokens.css that reuses one of open-design's shared templates and disagrees with the author's DESIGN.md. The reference gives both, and for a direction the author's own palette is the one with the character.

| System | Source | Use it for |
|---|---|---|
| Neutral Modern (`default`) | open-design starter | The floor before a brand exists; B2B tools, utility pages |
| Warm Editorial | open-design starter | Serif-led long-form on warm paper, one terracotta |
| kami | tw93/kami, MIT | Printed-feeling reports, price lists, decks: one serif, ink-blue, no italic |
| Tom Modern | Tom-Opencart, MIT | Sharp editorial-technical pages with hard shadows, the anti-template |
| Atelier Zero | open-design | Studio or venue pages that feel like a print annual |
| HUD | open-design | Game and hardware reference (phosphor on black) |
| Mission Control | open-design | Fleet, warehouse or ops status boards |
| Trading Terminal | open-design | Dense data screens with a gain/loss colour pair |
| Urdu Modern | open-design | Right-to-left and bilingual pages (Arabic script) |
| Cloudflare Kumo | cloudflare/kumo, MIT | App and admin screens with full light and dark sets |

## Rules worth keeping from them

- One accent, used for links, the primary action and at most one hero element a page (Neutral Modern, Warm Editorial, Tom Modern).
- A radius range, not just values: never under 8 or over 24 (Warm Editorial), or zero everywhere (Tom Modern).
- Shadow only on hover, or a hard offset shadow with no blur: both are drawable on the canvas.
- When the accent can't darken further (kami's ink-blue), hover stays equal to rest; draw that matrix column identical.
- Tabular figures wherever numbers stack (kami, Trading Terminal); signs with colour, never colour alone.
- A second script gets its own size and line height and the layout mirrors for RTL (Urdu Modern).

## References

| File | Holds |
|---|---|
| `references/handwritten-systems.md` | All ten systems: character, tokens and the author's own palette in Sora's names, contrast fixes, font stand-ins, and the lesson |

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and tw93/kami (MIT, Copyright (c) 2026 Tw93), Tom-Opencart/tom-modern-html-style-rule (MIT, Copyright (c) 2026 Tom-Opencart), cloudflare/kumo (MIT, Copyright (c) 2026 Cloudflare, Inc.). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
