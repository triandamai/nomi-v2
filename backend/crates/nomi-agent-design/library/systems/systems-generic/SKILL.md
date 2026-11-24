---
name: systems-generic
description: "The 73 generic style packages (cafe, friendly, luxury, retro, brutalism…) with tokens from tokens.css mapped to Sora's names, contrast fixed, violet swapped and Taste-approved Google Fonts; read it when a brief names a mood rather than a product."
license: "Apache-2.0 (open-design, Copyright (c) 2026 nexu-io); MIT, Copyright (c) 2026 Bergside (skills authored with typeui.sh)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/design-systems; https://github.com/bergside/awesome-design-skills (f631a09, 28 Jun 2026)"
---
# Systems: generic styles

The 73 generic style packages with their tokens mapped to your names, contrast fixed, violet swapped and Taste-approved Google Fonts. Read it when a brief names a mood ("friendly", "luxury", "retro") rather than a product.

## Where they come from, and what to trust

- **57 open-design packages.** open-design took them from bergside/awesome-design-skills (50 still in bergside's catalogue; application, dashboard, elegant, energetic, luxury, publication and simple were removed upstream later), then compiled a tokens.css for each.
- **tokens.css wins.** Each package's DESIGN.md still repeats bergside's original palette (often Poppins with a placeholder #3B82F6/#8B5CF6 pair), which contradicts its tokens. The cards use the tokens and quote bergside's palette on each line so you can see the difference.
- **Shared palettes.** The 57 packages share 29 token sets: cafe, artistic, elegant, refined and storytelling are identical; seven cool-blue packages are identical; and so on. The references group packages by palette family. Packages in a family differ in character, type and layout habits, not in colour.
- **16 bergside-only skills** (basic, codex, fiction, geometric, immersive, impeccable, matrix, power, pulse, riso, roku, sega, sketch, square, stitch, terracotta) have only four colours each. Their token sets are built from the skill's own colours and, where it names one, the ground from its description.
- bergside's `claude` skill is a brand reading and lives in `systems-brands` (brands-ai).

## What was changed

- **Contrast.** Foreground holds 7:1; muted-foreground 4.5:1 on background and on muted; the primary's label 4.5:1; destructive 4.5:1. An accent under 3:1 on the ground takes Taste's * rule: buttons in ink, the accent as `$color-brand`.
- **Violet.** Any accent, ground or ink with an OKLCH hue of 270–330 is moved to 255 (blue) or 345 (rose), keeping lightness and chroma.
- **Effects.** Glass, blur, gradients, glow, grain and neumorphic double shadows can't be drawn on the canvas. The families that rely on them (soft clay, glass, gradient, neon) keep their palette and radii and drop the effect; on tech only, note the intended effect for Koda.
- **Fonts.** Inter, Poppins, Roboto, Montserrat, Open Sans, Space Grotesk, Plus Jakarta Sans and system stacks are banned by Taste; each package gets canvas fonts from the `taste-type` table. Novelty faces in the source (Comic Sans, Press Start 2P, Bangers, VT323) are logo-only at most.

## Picking one

1. Match the mood word in the brief to a package, then read its family's reading and tokens.
2. A place (cafe, salon, homestay) uses only light families: warm, calm, and the light bright ones with the accent darkened. Dark families are for software, AI and games.
3. Swap the accent for the person's own colour and recheck it (`taste-color`); keep the family's ground, ink roles and radius ladder.

## References

| File | Holds |
|---|---|
| `references/generic-warm.md` | Cream-walnut (artistic, cafe, elegant, refined, storytelling), aged paper (paper, spacious, vintage), editorial, premium, soft clay (claymorphism, neumorphism, skeumorphism), sunny retro (dithered, neobrutalism, retro), levels |
| `references/generic-bright.md` | Yellow-orange (colorful, creative, doodle, energetic, friendly, vibrant), brutalism, pacman, tetris, lingo, expressive, gradient, glassmorphism |
| `references/generic-calm.md` | Cool blue (bento, contemporary, corporate, flat, perspective, professional, simple), application, dashboard, enterprise, modern, material, ant, shadcn, ink on white (bold, clean, minimal, mono), publication |
| `references/generic-dark.md` | Agentic, violet night (cosmic, fantasy, neon), blue night (dramatic, futuristic, sleek), luxury |
| `references/generic-bergside.md` | The 16 bergside-only skills |

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
