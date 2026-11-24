# Generic styles: bergside-only skills

The sixteen bergside/awesome-design-skills styles that open-design never copied, each with a token set built from the skill's own colours and description, checked and mapped to Sora's names.

bergside ships only primary, secondary, surface and text (most with a placeholder #3B82F6/#8B5CF6 pair), so grounds come from the description where it names one. Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## basic
Print-inspired books and reports. Suits school, studio. Canvas fonts Newsreader + Hanken Grotesk (skill: Oswald, Nunito).

Editorial grids and expressive type on white with a navy ink; the violet primary goes rose.
**Tokens:** background #ffffff · foreground #0a1829 (18) · primary #d5259e (4.6 on ground) · primary-foreground #ffffff (4.6)
**Fixed:** primary #a855f7 is violet (OKLCH hue 304) → #e133a7; primary #e133a7 → #d5259e so its label #ffffff holds 4.5 (4.0:1).

## codex
Blank edge-to-edge canvas, black only. Suits devtool, studio. Canvas fonts Geist (skill: Open Sans, JetBrains Mono).

Black is the only fill, divider and card tone; typography carries everything. Open Sans swapped (banned).
**Tokens:** background #ffffff · foreground #111827 (18) · primary #000000 (21 on ground) · primary-foreground #ffffff (21)
**Checked:** every pair passes as shipped.

## fiction
Children's-book cartoon, thick outlines. Suits school, bakery, retail. Canvas fonts Gabarito + Figtree (skill: Cossette Texte, JetBrains Mono).

Warm cream ground (the skill's secondary), big rounded shapes, thick black outlines, saturated colour blocks.
**Tokens:** background #ffe9ce · foreground #222222 (13) · card #ffffff · primary #222222 (13 on ground) · primary-foreground #ffe9ce (13)
**Checked:** every pair passes as shipped.

## geometric
Precise shapes, neutral, structured. Suits saas, workshop. Canvas fonts Epilogue (skill: Inter, JetBrains Mono).

Still carries bergside's placeholder blue/violet pair; the structure is the content.
**Tokens:** background #ffffff · foreground #111827 (18) · primary #2970e4 (4.6 on ground) · primary-foreground #ffffff (4.6)
**Fixed:** primary #3b82f6 → #2970e4 so its label #ffffff holds 4.5 (3.7:1).

## immersive
Exhibit-style story on deep green. Suits events, school. Canvas fonts Archivo (skill: Oswald, JetBrains Mono).

One continuous brand-green canvas with gamified storytelling; on a place use the green only as the one brand band.
**Tokens:** background #00592b · foreground #ffffff (8.5) · primary #ffffff (8.5 on ground) · primary-foreground #00592b (8.5)
**Checked:** every pair passes as shipped.

## impeccable
Graphic poster, cream and burnt orange. Suits agency, cafe, studio. Canvas fonts Archivo + Work Sans (skill: Chakra Petch, JetBrains Mono).

Alternating cream and burnt-orange sections with an amber brand colour. The amber is too light on cream (Taste *).
**Tokens:** background #fff8ec · foreground #111827 (17) · primary #111827* · primary-foreground #fff8ec · brand #cc8800 · destructive #dc2626 (4.6)
**Fixed:** primary #cc8800 is 2.8:1 on the ground (under 3): buttons and links take the ink, #cc8800 becomes $color-brand for small marks and one band (Taste *).

## matrix
Dark-only cyber, monospace. Suits devtool, game. Canvas fonts Geist Mono + Geist (skill: Space Mono).

Single deep monochrome surface and mono type. Tech only.
**Tokens:** background #0b0c14 · foreground #e8f5ef (17) · primary #2db58a (7.5 on ground) · primary-foreground #0b0c14 (7.5)
**Checked:** every pair passes as shipped.

## power
Monochrome dark, big headings. Suits hardware, agency. Canvas fonts Mona Sans (skill: Oswald, JetBrains Mono).

Black, white and nothing else, with very large headings. Tech only.
**Tokens:** background #000000 · foreground #ffffff (21) · primary #fafafa (20 on ground) · primary-foreground #000000 (20)
**Checked:** every pair passes as shipped.

## pulse
Thick borders, orange motion. Suits wellness, events. Canvas fonts Archivo (skill: Limelight, JetBrains Mono).

Orange text on peach fails badly as shipped; the ink is pushed to a deep rust.
**Tokens:** background #ffedd5 · foreground #8f3101 (7.0) · card #fdba74 · primary #ca4b02 (4.1 on ground) · primary-foreground #ffffff (4.6)
**Fixed:** foreground #ea580c → #8f3101 (3.1:1 was under 7); primary #ea580b → #ca4b02 so its label #ffffff holds 4.5 (3.6:1).

## riso
Two-colour risograph on off-white paper. Suits studio, bakery, events. Canvas fonts Bricolage Grotesque + Figtree (skill: Space Grotesk, Overpass Mono).

A single warm paper surface with fluorescent pink as the only action and blue (#2c40a7) as second ink. Space Grotesk swapped (banned).
**Tokens:** background #faf6ee · foreground #111827 (16) · primary #da178f (4.3 on ground) · primary-foreground #ffffff (4.7)
**Fixed:** primary #f237a1 → #da178f so its label #ffffff holds 4.5 (3.6:1).

## roku
Purple app dashboard, top bar. Suits saas. Canvas fonts Onest (skill: Inter, JetBrains Mono).

Top-bar navigation and card dashboard; the purple goes rose for Taste.
**Tokens:** background #ffffff · foreground #09090b (20) · primary #c61591 (5.4 on ground) · primary-foreground #ffffff (5.4) · destructive #db3035 (4.7)
**Fixed:** primary #9333ea is violet (OKLCH hue 302) → #c61591; destructive #ef4444 → #db3035 (3.8:1).

## sega
Arcade: pixel type, 0px corners, pressed pills. Suits game. Canvas fonts Anybody + Onest (skill: VT323, JetBrains Mono).

Chunky pill buttons pressing into offset blocks, rail-framed cards, VT323 pixel type for logos only.
**Tokens:** background #ffffff · foreground #111827 (18) · primary #075bb0 (6.7 on ground) · primary-foreground #ffffff (6.7)
**Fixed:** primary #4502ff is violet (OKLCH hue 275) → #075bb0.

## sketch
Pencil sketch on cream, soft teal. Suits school, cafe, studio. Canvas fonts Gabarito + Figtree (skill: Delicious Handrawn, JetBrains Mono).

Warm cream paper, hand-written display, dashed card outlines, rounded pills. The teal needs darkening for a white label.
**Tokens:** background #f4ede0 · foreground #111827 (15) · card #ffffff · primary #028673 (3.9 on ground) · primary-foreground #ffffff (4.5)
**Fixed:** primary #1dad97 → #028673 so its label #ffffff holds 4.5 (2.8:1).

## square
Delicate refined minimal. Suits law, studio. Canvas fonts Instrument Serif + Instrument Sans (skill: Google Sans, Anonymous Pro).

Placeholder blue/violet palette; the idea is the delicate type. Google Sans swapped.
**Tokens:** background #ffffff · foreground #111827 (18) · primary #2970e4 (4.6 on ground) · primary-foreground #ffffff (4.6)
**Fixed:** primary #3b82f6 → #2970e4 so its label #ffffff holds 4.5 (3.7:1).

## stitch
Enterprise drag-and-drop, deep teal and orange. Suits saas, workshop. Canvas fonts Hanken Grotesk (skill: Oswald, Ubuntu).

Stone ground, deep teal actions, an orange (#ff5f03) second colour for drag targets. Every pair passes except the red on the stone ground.
**Tokens:** background #edeade · foreground #111827 (15) · primary #072c2c (12 on ground) · primary-foreground #ffffff (15) · destructive #d0141a (4.6)
**Fixed:** destructive #dc2626 → #d0141a (4.0:1).

## terracotta
Sun-baked clay editorial with a display serif. Suits cafe, bakery, hotel, wedding. Canvas fonts Fraunces + Figtree (skill: DM Serif Display, JetBrains Mono).

Warm cream surfaces, ink-brown serif headlines and one terracotta accent. Ground and ink taken from its description (its tokens say white and grey-navy). The accent needs darkening.
**Tokens:** background #f3e9d8 · foreground #2b1d14 (14) · card #ffffff · primary #b65b2d (3.9 on ground) · primary-foreground #ffffff (4.6)
**Fixed:** primary #c56a3c → #b65b2d so its label #ffffff holds 4.5 (3.8:1).

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside; skills authored with typeui.sh). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
