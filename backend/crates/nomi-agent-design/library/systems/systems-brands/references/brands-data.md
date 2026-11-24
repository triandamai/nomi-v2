# Brands: data and backend platforms

Database and backend platforms, mostly dark consoles: what makes each work and the rare lesson for a small business.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## Supabase
`supabase` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Deep black (#171717), #fafafa text, emerald (#3ecf8e) with dark label, 6/8/16 radii, terminal-born.
**Tokens:** background #171717 · foreground #fafafa (17) · card #1c1c1c · muted #1c1c1c · muted-foreground #898989 (5.1) · border #2e2e2e · primary #3ecf8e (9.0 on ground) · primary-foreground #0f0f0f (9.6) · destructive #f03c35 (4.6) · radius 6/8/16 · type 16/18/24/32/36/72 · body line 1.5
**Fixed:** destructive #dc2626 → #f03c35 (3.7:1).
**Fonts:** Circular (licensed) → Outfit
**Learn:** Tech only. Learn the one-green-on-dark discipline.

## Sentry
`sentry` · devtool · VoltAgent awesome-design-md via open-design

**Works:** Purple-black grounds (#1f1633), white text, muted purple accent, 6/8/12 radii. Violet grounds are out for Taste.
**Tokens:** background #081d36 · foreground #ffffff (17) · card #061425 · muted #061425 · muted-foreground #9d96b3 (6.0) · border #362d59 · primary #2970c3 (3.4 on ground) · primary-foreground #ffffff (5.0) · destructive #f44038 (4.5) · radius 6/8/12 · type 16/20/24/30/60/88 · body line 1.5
**Fixed:** background #1f1633 is violet (OKLCH hue 296) → #081d36; card #150f23 is violet (OKLCH hue 296) → #061425; muted #150f23 is violet (OKLCH hue 296) → #061425; primary #6a5fc1 is violet (OKLCH hue 285) → #2970c3; destructive #dc2626 → #f44038 (3.5:1).
**Fonts:** Dammit Sans / Rubik → Epilogue
**Learn:** Reference only; nothing here for a small business except chart clarity.
**Upstream now:** VoltAgent's current file gives primary #150f23; faces Sentri Display, Rubik.

## PostHog
`posthog` · saas, devtool · VoltAgent awesome-design-md via open-design

**Works:** Sage-tinted cream (#fdfdf8), olive-grey ink (#4d4f46), orange (#f54e00), small 4/6/8 radii, playful illustration; anti-corporate.
**Tokens:** background #fdfdf8 · foreground #4d4f46 (8.2) · card #eeefe9 · muted #e5e7e0 · muted-foreground #65675e (5.6) · border #bfc1b7 · primary #d54403 (4.4 on ground) · primary-foreground #ffffff (4.5) · destructive #dc2626 (4.7) · radius 4/6/8 · type 16/20/24/30/36/44 · body line 1.5 · display tracking -0.025em
**Fixed:** primary #f54e00 → #d54403 so its label #ffffff holds 4.5 (3.5:1).
**Fonts:** IBM Plex Sans (Google Fonts) → keep IBM Plex Sans
**Learn:** A warm cream ground with olive ink and a cheeky voice fits an indie product.
**Upstream now:** VoltAgent's current file gives primary #f7a501 on canvas #eeefe9; faces IBM Plex Sans Variable, ui-monospace, Source Code Pro.

## Sanity
`sanity` · saas, agency · VoltAgent awesome-design-md via open-design

**Works:** Near-white (#f8f8f7), #171717 ink, red (#f03e2f), 6/10/16 radii; the upstream page itself runs dark.
**Tokens:** background #f8f8f7 · foreground #171717 (17) · card #ffffff · muted #fff1ef · muted-foreground #6e6e77 (4.8) · border #e4e4e7 · primary #e02e1f (4.3 on ground) · primary-foreground #ffffff (4.6) · destructive #dc2626 (4.5) · radius 6/10/16 · type 16/18/22/32/46/64 · body line 1.55 · display tracking -0.02em
**Fixed:** muted-foreground → #6e6e77 so it also holds 4.5 on muted; primary #f03e2f → #e02e1f so its label #ffffff holds 4.5 (3.9:1).
**Fonts:** Waldenburg (licensed) → Schibsted Grotesk
**Learn:** One red accent on a near-white ground is a solid agency default.

## MongoDB
`mongodb` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Teal-black (#001e2b), white type, neon green (#00ed64) with a dark label, serif display (Value Serif) beside a geometric sans, 4/8/16 radii.
**Tokens:** background #001e2b · foreground #ffffff (17) · card #1c2d38 · muted #1c2d38 · muted-foreground #b8c4c2 (9.6) · border #3d4f58 · primary #00ed64 (11 on ground) · primary-foreground #001e2b (11) · destructive #f44038 (4.6) · radius 4/8/16 · type 16/20/24/36/64/96 · body line 1.5
**Fixed:** destructive #dc2626 → #f44038 (3.6:1).
**Fonts:** Value Serif / Euclid (licensed) → Newsreader + Hanken Grotesk
**Learn:** Learn the serif-plus-geometric pairing for a data story; the dark ground stays in tech.

## ClickHouse
`clickhouse` · devtool · VoltAgent awesome-design-md via open-design

**Works:** Pure black, white type, acid yellow (#faff69) with a black label, tiny 4/8 radii, terminal density.
**Tokens:** background #000000 · foreground #ffffff (21) · card #141414 · muted #141414 · muted-foreground #a0a0a0 (8.0) · border #414141 · primary #faff69 (20 on ground) · primary-foreground #151515 (17) · destructive #e02b29 (4.5) · radius 4/8/8 · type 16/20/24/36/72/96 · body line 1.5
**Fixed:** destructive #dc2626 → #e02b29 (4.3:1).
**Fonts:** Inter (banned) → Chivo
**Learn:** Tech only.

## HashiCorp
`hashicorp` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Dark sections (#15181e) for product, white for information, blue (#1060ff), 5/8 radii, a colour per product line.
**Tokens:** background #15181e · foreground #efeff1 (15) · card #0d0e12 · muted #0d0e12 · muted-foreground #b2b6bd (8.7) · border #b2b6bd · primary #1060ff (3.5 on ground) · primary-foreground #ffffff (5.1) · destructive #f03c35 (4.6) · radius 5/8/8 · type 16/20/26/42/52/82 · body line 1.63
**Fixed:** destructive #dc2626 → #f03c35 (3.7:1).
**Fonts:** HashiCorp Sans (licensed) → SUSE
**Learn:** One colour per service line is a useful idea for a firm with several services.
**Upstream now:** VoltAgent's current file gives primary #000000 on canvas #000000; faces hashicorpSans.

## Composio
`composio` · devtool, ai · VoltAgent awesome-design-md via open-design

**Works:** Near-black (#090b14), white text, a violet-blue accent (#6d5dfc) that Taste turns blue, 10/14/22 radii, dense panels.
**Tokens:** background #090b14 · foreground #f8fafc (19) · card #121626 · muted #0e2440 · muted-foreground #94a3b8 (7.7) · border #27324a · primary #0474e0 (4.3 on ground) · primary-foreground #ffffff (4.6) · destructive #fb7185 (7.3) · radius 10/14/22 · type 15/17/22/32/48/64 · body line 1.55 · display tracking -0.02em
**Fixed:** muted #1b2140 is violet (OKLCH hue 273) → #0e2440; primary #6d5dfc is violet (OKLCH hue 281) → #137ae6; primary #137ae6 → #0474e0 so its label #ffffff holds 4.5 (4.2:1).
**Fonts:** abcDiatype (licensed) → Geist
**Learn:** Tech only.
**Upstream now:** VoltAgent's current file gives primary #0007cd on canvas #0f0f0f; faces abcDiatype, JetBrains Mono.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
