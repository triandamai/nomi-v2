# Brands: media, entertainment and retro

Media, entertainment, consumer-hardware and retro-web systems: photography-led layouts and editorial grids, mostly reference only.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## Apple
`apple` · product, hardware, retail · VoltAgent awesome-design-md via open-design

**Works:** White and #f5f5f7 bands alternating, #1d1d1f ink, blue (#0071e3) link-only, 17px body, 8/12/18 radii, product photography with all the expression.
**Tokens:** background #ffffff · foreground #1d1d1f (17) · card #f5f5f7 · muted #fbfbfd · muted-foreground #6e6e73 (5.1) · border #d2d2d7 · primary #0071e3 (4.7 on ground) · primary-foreground #ffffff (4.7) · destructive #dc2626 (4.8) · radius 8/12/18 · type 17/21/28/40/56/80 · body line 1.47 · display tracking -0.015em
**Checked:** every pair passes as shipped.
**Fonts:** SF Pro (licensed) → Instrument Sans
**Learn:** Alternating white and pale-grey bands with one product photo each is the cleanest product page there is.
**Upstream now:** VoltAgent's current file gives primary #0066cc on canvas #ffffff; faces SF Pro Display, SF Pro Text.

## NVIDIA
`nvidia` · hardware, game · VoltAgent awesome-design-md via open-design

**Works:** Black and white, green (#76b900) with black label, near-square 2/4 radii, dense spec type.
**Tokens:** background #000000 · foreground #ffffff (21) · card #1a1a1a · muted #1a1a1a · muted-foreground #848484 (5.6) · border #5e5e5e · primary #76b900 (8.7 on ground) · primary-foreground #000000 (8.7) · destructive #e52020 (4.6) · radius 2/2/4 · type 16/18/22/24/32/36 · body line 1.5
**Fixed:** muted-foreground → #848484 so it also holds 4.5 on muted.
**Fonts:** NVIDIA Sans (licensed) → Archivo
**Learn:** Tech only.

## PlayStation
`playstation` · game, product · VoltAgent awesome-design-md via open-design

**Works:** A vertical channel: black hero, white editorial panels, deep blue footer; blue (#0070cc) action, 6/12/24 radii.
**Tokens:** background #000000 · foreground #ffffff (21) · card #ffffff · muted #f5f7fa · muted-foreground #6f6f6f (4.2) · border #cccccc · primary #0070cc (4.2 on ground) · primary-foreground #ffffff (5.0) · destructive #dc3249 (4.6) · radius 6/12/24 · type 18/22/28/35/44/54 · body line 1.5 · display tracking -0.1px
**Fixed:** muted-foreground → #6f6f6f so it also holds 4.5 on muted; destructive #c81b3a → #dc3249 (3.7:1).
**Fonts:** SST (licensed) → Mona Sans
**Learn:** The "one dark masthead, light body, coloured footer" rhythm.
**Upstream now:** VoltAgent's current file gives primary #0070d1; faces PlayStation SST.

## Spotify
`spotify` · game, studio · VoltAgent awesome-design-md via open-design

**Works:** Near-black layers (#121212, #181818), white type, green (#1ed760) with black label, pill buttons, art supplies colour.
**Tokens:** background #121212 · foreground #ffffff (19) · card #181818 · muted #1f1f1f · muted-foreground #b3b3b3 (8.9) · border #4d4d4d · primary #1ed760 (9.8 on ground) · primary-foreground #000000 (11) · destructive #f3727f (6.7) · radius 9999/6/8 · type 16/18/20/24/24/24 · body line 1.50
**Checked:** every pair passes as shipped.
**Fonts:** Spotify Mix (licensed) → Figtree
**Learn:** Tech/media only.

## SpaceX
`spacex` · hardware · VoltAgent awesome-design-md via open-design

**Works:** Pure black, pale #f0f0fa text, outline buttons, uppercase spaced D-DIN, 4px radii.
**Tokens:** background #000000 · foreground #f0f0fa (19) · card #000000 · muted #000000 · muted-foreground #f0f0fa (19) · border #f0f0fa · primary #f0f0fa (19 on ground) · primary-foreground #000000 (19) · destructive #e02b29 (4.5) · radius 4/4/4 · type 16/20/24/32/40/48 · body line 1.5 · display tracking 0.02em
**Fixed:** destructive #dc2626 → #e02b29 (4.3:1).
**Fonts:** D-DIN (OFL) → Chivo
**Learn:** Tech only; uppercase tracking is against Taste for a place.
**Upstream now:** VoltAgent's current file gives primary #000000; faces D-DIN-Bold, D-DIN.

## The Verge
`theverge` · agency, studio · VoltAgent awesome-design-md via open-design

**Works:** Near-black, white type, hot pink (#ff00a8) and mint as signals, a brutally heavy display face, 4/8/12 radii.
**Tokens:** background #050505 · foreground #ffffff (20) · card #111111 · muted #250f1c · muted-foreground #9ca3af (8.0) · border #2b2b2b · primary #df0594 (4.5 on ground) · primary-foreground #ffffff (4.6) · destructive #ef4444 (5.4) · radius 4/8/12 · type 16/19/26/42/66/96 · body line 1.48 · display tracking -0.035em
**Fixed:** muted #211022 is violet (OKLCH hue 325) → #250f1c; primary #ff00a8 → #df0594 so its label #ffffff holds 4.5 (3.6:1).
**Fonts:** Manuka/PolySans (licensed) → Anybody + Onest
**Learn:** Reference for an editorial site; too loud for a small business.

## WIRED
`wired` · agency, school · VoltAgent awesome-design-md via open-design

**Works:** Paper white, black, red (#e10600), zero radius, hairline rules, dense grid, a wide heavy display face.
**Tokens:** background #ffffff · foreground #000000 (21) · card #f2f2f2 · muted #fff0f0 · muted-foreground #666666 (5.7) · border #d0d0d0 · primary #e10600 (5.0 on ground) · primary-foreground #ffffff (5.0) · destructive #c40000 (6.3) · radius 0/0/0 · type 17/20/28/44/68/96 · body line 1.55 · display tracking -0.02em
**Checked:** every pair passes as shipped.
**Fonts:** Druk Wide (licensed) → Archivo + Source Serif 4
**Learn:** Square corners and hairline rules suit a newsletter or local news page.
**Upstream now:** VoltAgent's current file gives primary #000000 on canvas #ffffff.

## aljazeera
`aljazeera` · school, community · VoltAgent awesome-design-md only

**Works:** Flat white, black ink, one orange (#fa9000) as logo block and eyebrow bar, a live red for breaking marks, dense Roboto stacks.
**Tokens:** background #ffffff · foreground #000000 (21) · card #f0f0f0 · muted #f0f0f0 · muted-foreground #595959 (7.0) · border #d8d8d8 · primary #ad630a (4.6 on ground) · primary-foreground #ffffff (4.6) · destructive #c31833 (6.0) · radius 4/10/18
**Fixed:** primary #fa9000 → #ad630a so its label #ffffff holds 4.5 (2.3:1).
**Fonts:** Roboto (banned) → Schibsted Grotesk
**Learn:** Orange fails on white (Taste *): ink buttons; orange as a bar and mark.

## Totality Festival
`totality-festival` · events · open-design's own brand file

**Works:** Deep space (#070711) with violet glow and amber flashes, 10/16/24 radii. Taste rejects violet grounds.
**Tokens:** background #070711 · foreground #f8f7ff (19) · card #061426 · muted #001d42 · muted-foreground #9d8ad4 (6.7) · border #34265e · primary #f072be (7.5 on ground) · primary-foreground #13051f (7.3) · destructive #ff4d8d (6.4) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** card #111126 is violet (OKLCH hue 282) → #061426; muted #1e1540 is violet (OKLCH hue 289) → #001d42; primary #c084fc is violet (OKLCH hue 306) → #f072be.
**Fonts:** Inter (banned) → Mona Sans
**Learn:** Event reference only.

## nintendo-2001
`nintendo-2001` · game · VoltAgent awesome-design-md only

**Works:** Periwinkle metal panels (#7a8aba), amber-glowing nav, outlined heavy display, white content cards. Retro reference.
**Tokens:** background #7a8aba · foreground #010101 (6.1) · card #ffffff · border #5a5f8c · primary #010101* · primary-foreground #7a8aba · brand #e60012 · radius 4/6/10
**Fixed:** foreground #21242e → #010101 (4.6:1 was under 7); primary #e60012 is 1.4:1 on the ground (under 3): buttons and links take the ink, #e60012 becomes $color-brand for small marks and one band (Taste *).
**Fonts:** Arial Black → Archivo
**Learn:** Retro reference only; violet-blue grounds are out for Taste.

## dell-1996
`dell-1996` · retro · VoltAgent awesome-design-md only

**Works:** A literal black page frame, flat sage/salmon/sky ribbon cards, Helvetica Black titles over Times body, red (#e91d2a), GIF stickers.
**Tokens:** background #ffffff · foreground #000000 (21) · card #ffffff · primary #e51627 (4.7 on ground) · primary-foreground #ffffff (4.7) · radius 0
**Fixed:** primary #e91d2a → #e51627 so its label #ffffff holds 4.5 (4.5:1).
**Fonts:** Helvetica Black + Times → Archivo + Source Serif 4
**Learn:** Retro reference only.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
