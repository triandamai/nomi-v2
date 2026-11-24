# Brands: design tools and playful apps

Design tools and playful consumer apps: black-and-white canvases, cream craft, chunky pressed buttons, and what a studio or class can learn.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## Figma
`figma` · studio, agency · VoltAgent awesome-design-md via open-design

**Works:** Pure black and white chrome, black buttons, a variable font using unusual weights (320–540), pill buttons (50) with 8px cards; colour arrives only in product shots.
**Tokens:** background #ffffff · foreground #000000 (21) · card #ffffff · muted #ffffff · muted-foreground #000000 (21) · border #000000 · primary #000000 (21 on ground) · primary-foreground #ffffff (21) · destructive #dc2626 (4.8) · radius 50/8/8 · type 16/20/26/26/64/86 · body line 1.40 · display tracking -0.02em
**Checked:** every pair passes as shipped.
**Fonts:** figmaSans (licensed) → Familjen Grotesk
**Learn:** A designer portfolio: black-and-white chrome so the work is the only colour.

## Framer
`framer` · studio, agency · VoltAgent awesome-design-md via open-design

**Works:** Pure black, white type, blue (#0099ff) links, huge display (110) at -0.05em, 8/12/20 radii.
**Tokens:** background #000000 · foreground #ffffff (21) · card #090909 · muted #090909 · muted-foreground #ffffff (21) · border #0099ff · primary #0c78c9 (4.5 on ground) · primary-foreground #ffffff (4.6) · destructive #e02b29 (4.5) · radius 8/12/20 · type 15/18/22/32/62/110 · body line 1.3 · display tracking -0.05em
**Fixed:** primary #0099ff → #0c78c9 so its label #ffffff holds 4.5 (3.0:1); destructive #dc2626 → #e02b29 (4.3:1).
**Fonts:** GT Walsheim (licensed) → Mona Sans
**Learn:** Tech only for the dark; the giant tight headline is the transferable bit.
**Upstream now:** VoltAgent's current file gives primary #ffffff on canvas #090909; faces GT Walsheim Framer Medium, GT Walsheim Medium, Inter.

## Canva
`canva` · studio, school, retail · open-design's own brand file

**Works:** White and #f4f5f7, #0e1318 ink, purple (#7d2ae8) to teal brand, generous padding, 8/12/16 radii. Taste turns the purple blue.
**Tokens:** background #ffffff · foreground #0e1318 (19) · card #f4f5f7 · muted #f4f5f7 · muted-foreground #5f6368 (6.0) · border #e1e3e6 · primary #0668c8 (5.5 on ground) · primary-foreground #ffffff (5.5) · destructive #db333c (4.6) · radius 8/12/16 · type 14/16/18/24/36/64 · body line 1.5 · display tracking -0.02em
**Fixed:** primary #7d2ae8 is violet (OKLCH hue 296) → #0668c8; destructive #ff5757 → #db333c (3.1:1).
**Fonts:** Canva Sans (licensed) → Figtree
**Learn:** Friendly, roomy cards with a plain ink suit a print shop or class.

## Webflow
`webflow` · agency, studio · VoltAgent awesome-design-md via open-design

**Works:** White, #080808 ink, blue (#146ef5), small 4/8 radii, a secondary set of purple, pink, green, orange for product accents.
**Tokens:** background #ffffff · foreground #080808 (20) · card #ffffff · muted #ffffff · muted-foreground #5a5a5a (6.9) · border #d8d8d8 · primary #146ef5 (4.6 on ground) · primary-foreground #ffffff (4.6) · destructive #ea1533 (4.5) · radius 4/4/8 · type 16/20/24/32/56/80 · body line 1.6 · display tracking -0.01em
**Fixed:** destructive #ee1d36 → #ea1533 (4.3:1).
**Fonts:** WF Visual Sans (licensed) → Instrument Sans
**Learn:** Keep one accent; secondary colours only inside product screenshots.
**Upstream now:** VoltAgent's current file gives primary #080808 on canvas #ffffff; faces WF Visual Sans Variable, WFVisualSans-Mono.

## Clay
`clay` · studio, agency, cafe · VoltAgent awesome-design-md via open-design

**Works:** Warm cream (#f7eee6), oat borders, brown-black ink, clay accent (#b46a46), big 14/22/34 radii, colourful illustrations.
**Tokens:** background #f7eee6 · foreground #2b211c (14) · card #fff8f1 · muted #ead6c7 · muted-foreground #6c5c52 (5.6) · border #dac8b9 · primary #ab613d (4.1 on ground) · primary-foreground #ffffff (4.7) · destructive #b54949 (4.6) · radius 14/22/34 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground #8a7a70 → #78685e (3.6:1); muted-foreground → #6c5c52 so it also holds 4.5 on muted; primary #b46a46 → #ab613d so its label #ffffff holds 4.5 (4.1:1); destructive #b84c4c → #b54949 (4.4:1).
**Fonts:** Plain (licensed) → Bricolage Grotesque + Figtree
**Learn:** Oat borders and cream cards feel hand-made; darken the clay accent and the grey.
**Upstream now:** VoltAgent's current file gives primary #0a0a0a on canvas #fffaf0; faces Plain Black, Inter.

## Duolingo
`duolingo` · school, community · open-design's own brand file

**Works:** White, #3c3c3c ink, owl green (#58cc02), thick 2–3px borders and a 4px bottom shadow on every control, 12/16/20 radii.
**Tokens:** background #ffffff · foreground #3c3c3c (11) · card #f7f7f7 · muted #f7f7f7 · muted-foreground #717171 (4.9) · border #e5e5e5 · primary #3b850e (4.6 on ground) · primary-foreground #ffffff (4.6) · destructive #df2a33 (4.6) · radius 12/16/20 · type 15/18/24/32/40/56 · body line 1.5 · display tracking -0.01em
**Fixed:** muted-foreground #777777 → #747474 (4.5:1); muted-foreground → #717171 so it also holds 4.5 on muted; primary #58cc02 → #3b850e so its label #ffffff holds 4.5 (2.1:1); destructive #ff4b4b → #df2a33 (3.3:1).
**Fonts:** Feather Bold / din-round → Nunito
**Learn:** The green needs darkening for a white label. The chunky pressed button suits a kids' class.

## Discord
`discord` · community, game · open-design's own brand file

**Works:** Grey-black chat layers (#313338, #2b2d31), #dbdee1 text, blurple (#5865f2), 4/8/16 radii.
**Tokens:** background #313338 · foreground #dbdee1 (9.4) · card #2b2d31 · muted #1e1f22 · muted-foreground #949ba4 (4.5) · border #ffffff · primary #dbdee1* · primary-foreground #313338 · brand #1175dc · destructive #ff706a (4.7) · radius 4/8/16 · type 16/18/20/24/32/56 · body line 1.375 · display tracking -0.02em
**Fixed:** primary #5865f2 is violet (OKLCH hue 274) → #0477e5; primary #0477e5 → #1175dc so its label #ffffff holds 4.5 (4.4:1); primary #1175dc is 2.8:1 on the ground (under 3): buttons and links take the ink, #1175dc becomes $color-brand for small marks and one band (Taste *); destructive #f23f43 → #ff706a (3.4:1).
**Fonts:** gg sans (licensed) → Gabarito + Figtree
**Learn:** Community app reference only.

## jobright
`jobright` · saas, school · VoltAgent awesome-design-md only

**Works:** White with pale mint bands (#ecffe0), black ink, mint (#00f0a0) with black label, condensed bold display, big 24–40 radii.
**Tokens:** background #ffffff · foreground #000000 (21) · card #f2f4f5 · muted #ecffe0 · muted-foreground #68727e (4.9) · border #e5e7eb · primary #000000* · primary-foreground #ffffff · brand #00f0a0 · radius 12/16/24
**Fixed:** muted-foreground #86909c → #6b7581 (3.2:1); muted-foreground → #68727e so it also holds 4.5 on muted; primary #00f0a0 is 1.5:1 on the ground (under 3): buttons and links take the ink, #00f0a0 becomes $color-brand for small marks and one band (Taste *).
**Fonts:** Titillium Web + Inter → Archivo + Work Sans
**Learn:** Mint fails on white (Taste *): ink buttons, mint for bands.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
