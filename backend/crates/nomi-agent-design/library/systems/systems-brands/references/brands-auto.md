# Brands: automotive

Automotive systems: photo-led heroes, square or sharp corners and one signal colour, with what a bengkel or dealer can learn.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## BMW
`bmw` · workshop, hardware · VoltAgent awesome-design-md via open-design

**Works:** Alternating dark photo heroes and white content, #262626 ink, blue (#1c69d4), zero radius, light display weight.
**Tokens:** background #ffffff · foreground #262626 (15) · card #f5f5f5 · muted #f5f5f5 · muted-foreground #6f6f6f (5.0) · border #e0e0e0 · primary #1c69d4 (5.2 on ground) · primary-foreground #ffffff (5.2) · destructive #dc2626 (4.8) · radius 0/0/0 · type 16/18/24/32/44/60 · body line 1.15
**Fixed:** muted-foreground → #6f6f6f so it also holds 4.5 on muted.
**Fonts:** BMW Type (licensed) → Archivo
**Learn:** A bengkel or car dealer: square corners, one blue, big car photos.

## BMW M
`bmw-m` · workshop, hardware · VoltAgent awesome-design-md via open-design

**Works:** Pale grey (#f7f8fa) or black canvases, uppercase headlines, M-blue (#0066b1), 4/8/14 radii; energy comes from photos.
**Tokens:** background #f7f8fa · foreground #111827 (17) · card #ffffff · muted #eef3f8 · muted-foreground #5e6e85 (4.9) · border #d7dee8 · primary #0066b1 (5.6 on ground) · primary-foreground #ffffff (5.9) · destructive #e4002b (4.6) · radius 4/8/14 · type 16/18/24/36/56/78 · body line 1.5 · display tracking -0.025em
**Fixed:** muted-foreground #64748b → #617188 (4.5:1); muted-foreground → #5e6e85 so it also holds 4.5 on muted.
**Fonts:** BMW Type Next (licensed) → Archivo
**Learn:** Take the photo-led energy; Taste keeps headlines sentence case.
**Upstream now:** VoltAgent's current file gives primary #ffffff on canvas #000000; faces BMWTypeNextLatin, BMWTypeNextLatin Light.

## Bugatti
`bugatti` · hardware · VoltAgent awesome-design-md via open-design

**Works:** Pure or navy black, only white at rest, one typographic moment over full-bleed video.
**Tokens:** background #07111f · foreground #f7fbff (18) · card #101d2f · muted #16283f · muted-foreground #8fa1b7 (7.2) · border #2c405c · primary #f7fbff* · primary-foreground #07111f · brand #1b4d8f · destructive #e53d3a (4.6) · radius 8/14/22 · type 16/19/26/42/64/88 · body line 1.58 · display tracking -0.015em
**Fixed:** primary #1b4d8f is 2.3:1 on the ground (under 3): buttons and links take the ink, #1b4d8f becomes $color-brand for small marks and one band (Taste *); destructive #c1121f → #e53d3a (3.0:1).
**Fonts:** Bugatti Display (licensed) → Mona Sans
**Learn:** Tech only.
**Upstream now:** VoltAgent's current file gives primary #ffffff on canvas #000000; faces Bugatti Display, Bugatti Monospace.

## Ferrari
`ferrari` · hardware, product · VoltAgent awesome-design-md via open-design

**Works:** Black ground, warm white type, red (#dc0000), 4/8/14 radii, art-institution pacing.
**Tokens:** background #0b0b0b · foreground #fffaf0 (19) · card #171717 · muted #23130f · muted-foreground #a89f91 (7.5) · border #342a24 · primary #dc0000 (3.8 on ground) · primary-foreground #ffffff (5.2) · destructive #ff3b30 (5.5) · radius 4/8/14 · type 16/19/26/40/62/88 · body line 1.5 · display tracking -0.03em
**Checked:** every pair passes as shipped.
**Fonts:** Ferrari Sans (licensed) → Archivo
**Learn:** Gravitas through pacing; red as a single mark.
**Upstream now:** VoltAgent's current file gives primary #da291c on canvas #181818; faces FerrariSans.

## Lamborghini
`lamborghini` · hardware · VoltAgent awesome-design-md via open-design

**Works:** True black, cream text, gold (#d4af37) with black label, sharp 2/6/10 radii, angular lines.
**Tokens:** background #050505 · foreground #f8f4e6 (19) · card #141414 · muted #211b0b · muted-foreground #9b927d (6.6) · border #3a321d · primary #d4af37 (9.7 on ground) · primary-foreground #050505 (9.7) · destructive #e63946 (4.9) · radius 2/6/10 · type 16/18/26/42/64/90 · body line 1.48 · display tracking -0.025em
**Checked:** every pair passes as shipped.
**Fonts:** Lamborghini (licensed) → Anybody + Onest
**Learn:** Tech only.

## Renault
`renault` · workshop, retail · VoltAgent awesome-design-md via open-design

**Works:** Warm white (#f7f4ef), black ink, yellow (#ffcc33) with black label, 4/8/14 radii, aurora gradient heroes.
**Tokens:** background #f7f4ef · foreground #111111 (17) · card #ffffff · muted #fff4bf · muted-foreground #6f6b63 (4.8) · border #dfd8ca · primary #111111* · primary-foreground #f7f4ef · brand #ffcc33 · destructive #d5281c (4.6) · radius 4/8/14 · type 16/19/26/38/58/82 · body line 1.5 · display tracking -0.03em
**Fixed:** primary #ffcc33 is 1.4:1 on the ground (under 3): buttons and links take the ink, #ffcc33 becomes $color-brand for small marks and one band (Taste *); destructive #d92d20 → #d5281c (4.4:1).
**Fonts:** NouvelR (licensed) → Archivo
**Learn:** Yellow fails on light (Taste *): ink buttons, yellow as a band.
**Upstream now:** VoltAgent's current file gives primary #ffed00 on canvas #ffffff; faces NouvelR.

## Tesla
`tesla` · hardware, product · VoltAgent awesome-design-md via open-design

**Works:** White, #171a20 ink, blue (#3e6ae1) action, 4/12 radii, tiny type over full-screen photos.
**Tokens:** background #ffffff · foreground #171a20 (17) · card #f4f4f4 · muted #f4f4f4 · muted-foreground #5c5e62 (6.5) · border #d0d1d2 · primary #3e6ae1 (4.8 on ground) · primary-foreground #ffffff (4.8) · destructive #dc2626 (4.8) · radius 4/12/12 · type 14/16/17/22/28/40 · body line 1.43
**Checked:** every pair passes as shipped.
**Fonts:** Universal Sans (licensed) → Host Grotesk
**Learn:** Subtraction: one photo, one line, two buttons.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
