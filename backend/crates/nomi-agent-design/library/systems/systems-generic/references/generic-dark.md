# Generic styles: dark

Dark generic packages with tokens checked; in Taste a dark page is tech only, so these seed software, AI and game work.

Values are from each package's tokens.css, which wins over its DESIGN.md (the DESIGN.md often repeats bergside's original palette). Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule). "Canvas fonts" are real Google Fonts from taste-type: the packages' own Inter, Poppins, Roboto, Montserrat and system faces are banned by Taste.

## Agentic

Navy-black (#0b1020), white text, a sky accent (#60a5fa) with a dark label, 8/12/20 radii. Tech only.

**Tokens:** background #0b1020 · foreground #f8fafc (18) · card #131b2f · muted #182343 · muted-foreground #8ea0b8 (7.1) · border #293653 · primary #60a5fa (7.4 on ground) · primary-foreground #06111f (7.5) · destructive #fb7185 (7.0) · radius 8/12/20 · type 15/17/22/32/48/66 · body line 1.55 · display tracking -0.02em
**Checked:** every pair passes as shipped.

- **agentic**: AI-first delegated task flows. Suits ai, saas. Canvas fonts Geist (tokens: Inter). bergside: primary #ff5701, surface #ffffff, text #111827; fonts Playfair Display, JetBrains Mono. bergside: orange #FF5701 on white with Playfair Display.

## Violet night (three packages)

Near-black with violet layers, a lavender accent (#c084fc). Violet is out for Taste: the accent goes rose and the layers navy. Tech and game reference only.

**Tokens:** background #070711 · foreground #f8f7ff (19) · card #061426 · muted #001d42 · muted-foreground #9d8ad4 (6.7) · border #34265e · primary #f072be (7.5 on ground) · primary-foreground #13051f (7.3) · destructive #ff4d8d (6.4) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** card #111126 is violet (hue 282) → #061426; muted #1e1540 is violet (hue 289) → #001d42; primary #c084fc is violet (OKLCH hue 306) → #f072be.

- **cosmic**: sci-fi neon space. Suits game, ai. Canvas fonts Sora + Figtree (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Audiowide, JetBrains Mono.
- **fantasy**: game fantasy, rich colour. Suits game. Canvas fonts Anybody + Onest (tokens: Georgia / Inter). bergside: primary #0250cc, surface #ffffff, text #111827; fonts New Rocker, IBM Plex Mono. bergside: royal blue and gold on white.
- **neon**: electric glow (flattened). Suits game, ai. Canvas fonts Mona Sans (tokens: Inter). bergside: primary #bbf351, surface #ffffff, text #111827; fonts STIX Two Text, Roboto.

## Blue night (three packages)

Navy-black (#090b12), white text, a sky accent with a dark label, 10/16/24 radii. All pairs pass. Tech only.

**Tokens:** background #090b12 · foreground #f8fafc (19) · card #121722 · muted #1b2233 · muted-foreground #94a3b8 (7.7) · border #2a3447 · primary #60a5fa (7.7 on ground) · primary-foreground #06101d (7.5) · destructive #fb7185 (7.3) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Checked:** every pair passes as shipped.

- **dramatic**: theatrical high contrast. Suits agency, ai. Canvas fonts Mona Sans (tokens: Inter). bergside: primary #8b5cf6, surface #09090b, text #fafafa; fonts Outfit, JetBrains Mono.
- **futuristic**: tech-forward sleek. Suits ai, hardware. Canvas fonts Sora + Figtree (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Audiowide, Roboto.
- **sleek**: minimal dark with subtle motion. Suits saas, devtool. Canvas fonts Geist (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Inter, JetBrains Mono.

## Luxury

Warm black (#080706), cream text, gold (#c6a15b) with a black label, 8/14/24 radii, a Didone display. A dark page is tech only in Taste, so a hotel or jeweller takes the gold-and-cream logic onto a light ivory ground.

**Tokens:** background #080706 · foreground #fff8ea (19) · card #151310 · muted #241e14 · muted-foreground #9f927c (6.6) · border #3a3020 · primary #c6a15b (8.3 on ground) · primary-foreground #080706 (8.3) · destructive #d85a52 (5.3) · radius 8/14/24 · type 16/19/28/44/68/96 · body line 1.6 · display tracking -0.015em
**Checked:** every pair passes as shipped.

- **luxury**: warm black, cream and gold. Suits hotel, wedding, retail. Canvas fonts EB Garamond + Jost (tokens: Didot / Avenir Next). bergside: removed from its current catalogue.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside; skills authored with typeui.sh). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
