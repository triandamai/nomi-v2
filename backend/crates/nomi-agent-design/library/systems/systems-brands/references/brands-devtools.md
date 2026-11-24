# Brands: developer tools

Developer-tool systems: white galleries, warm editors and dark consoles, and the few lessons that carry to a small business.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## Vercel
`vercel` · devtool, saas, agency · VoltAgent awesome-design-md via open-design

**Works:** Overwhelmingly white, #171717 ink, a blue (#0070f3) link colour, very tight display tracking (-0.05em), 6/8/12 radii, shadows as hairline rings.
**Tokens:** background #ffffff · foreground #171717 (18) · card #ffffff · muted #ffffff · muted-foreground #666666 (5.7) · border #000000 · primary #0070f3 (4.6 on ground) · primary-foreground #ffffff (4.6) · destructive #dc2626 (4.8) · radius 6/8/12 · type 16/20/24/32/40/48 · body line 1.5 · display tracking -0.05em
**Checked:** every pair passes as shipped.
**Fonts:** Geist (OFL, Google Fonts) → keep Geist
**Learn:** Strict white plus tight Geist headlines suit a web agency; use the ring-as-border trick for cards.
**Upstream now:** VoltAgent's current file gives primary #171717 on canvas #ffffff; faces Geist, Geist Mono.

## GitHub
`github` · devtool, saas · open-design's own brand file

**Works:** White or #f6f8fa panes, #1f2328 ink, blue (#0969da) links, 6px radii, hairline borders instead of space; dense, functional.
**Tokens:** background #ffffff · foreground #1f2328 (16) · card #f6f8fa · muted #f6f8fa · muted-foreground #656d76 (5.2) · border #d0d7de · primary #0969da (5.2 on ground) · primary-foreground #ffffff (5.2) · destructive #cf222e (5.4) · radius 6/6/12 · type 14/16/20/24/28/32 · body line 1.5 · display tracking -0.01em
**Checked:** every pair passes as shipped.
**Fonts:** Segoe/system → IBM Plex Sans
**Learn:** A dense admin screen: panes and hairlines instead of shadows keep tables legible.

## Cursor
`cursor` · devtool, studio, workshop · VoltAgent awesome-design-md via open-design

**Works:** Warm off-white (#f2f1ed), deep warm brown-black (#26251e) text, orange (#f54e00) action, 8/10/12 radii; old-paper warmth on an editor.
**Tokens:** background #f2f1ed · foreground #26251e (14) · card #e6e5e0 · muted #ebeae5 · muted-foreground #26251e (14) · border #26251e · primary #d54403 (4.0 on ground) · primary-foreground #ffffff (4.5) · destructive #cb2953 (4.7) · radius 8/10/12 · type 16/19.2/22/26/36/72 · body line 1.5 · display tracking -0.03em
**Fixed:** primary #f54e00 → #d54403 so its label #ffffff holds 4.5 (3.5:1); destructive #cf2d56 → #cb2953 (4.5:1).
**Fonts:** Cursor Gothic (licensed) → Familjen Grotesk
**Learn:** Warm brown-black ink on paper is a good Taste-style ground for a craft workshop.

## Expo
`expo` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Cool white with #f6f7f9 panels, #0b0d12 ink, black (#000020) buttons, 8/14/22 radii, polished app screenshots.
**Tokens:** background #ffffff · foreground #0b0d12 (19) · card #f6f7f9 · muted #eef2ff · muted-foreground #686f7d (5.0) · border #dfe3ea · primary #000020 (21 on ground) · primary-foreground #ffffff (21) · destructive #d53841 (4.7) · radius 8/14/22 · type 16/18/22/32/48/68 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground → #686f7d so it also holds 4.5 on muted; destructive #e5484d → #d53841 (3.9:1).
**Fonts:** Inter (banned) → Instrument Sans
**Learn:** Ink buttons on cool white and rounded phone mockups suit an app studio.
**Upstream now:** VoltAgent's current file gives primary #000000 on canvas #ffffff; faces Inter, JetBrains Mono.

## Raycast
`raycast` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Blue-black ground (#07080a), white text, coral-red (#ff6363) action, 6/12/16 radii, macOS-native depth.
**Tokens:** background #07080a · foreground #f9f9f9 (19) · card #101111 · muted #101111 · muted-foreground #9c9c9d (7.3) · border #ffffff · primary #d34242 (4.4 on ground) · primary-foreground #ffffff (4.5) · radius 6/12/16 · type 16/18/20/24/40/64 · body line 1.6
**Fixed:** primary #ff6363 → #d34242 so its label #ffffff holds 4.5 (2.9:1).
**Fonts:** Inter (banned) → Geist
**Learn:** Dark, tech only. Learn the precise keyboard-shortcut chips.
**Upstream now:** VoltAgent's current file gives primary #ffffff on canvas #07080a; faces Inter.

## Warp
`warp` · devtool · VoltAgent awesome-design-md via open-design

**Works:** Warm near-black (#161412), warm parchment text (#faf9f6), a grey (#353534) action that fails on the ground, 6/12/14 radii.
**Tokens:** background #161412 · foreground #faf9f6 (17) · card #1f1d1b · muted #1f1d1b · muted-foreground #868584 (5.0) · border #e2e2e2 · primary #faf9f6* · primary-foreground #161412 · brand #353534 · destructive #ec3832 (4.5) · radius 6/12/14 · type 18/24/32/48/56/80 · body line 1.4 · display tracking -0.03em
**Fixed:** primary #353534 is 1.5:1 on the ground (under 3): buttons and links take the ink, #353534 becomes $color-brand for small marks and one band (Taste *); destructive #dc2626 → #ec3832 (3.8:1).
**Fonts:** Matter (licensed) → Familjen Grotesk
**Learn:** Taste: buttons in the light ink. Learn the warm black instead of blue-black.
**Upstream now:** VoltAgent's current file gives primary #f7f5f0 on canvas #2b2622; faces Inter, DM Mono.

## Lovable
`lovable` · saas, studio · VoltAgent awesome-design-md via open-design

**Works:** Creamy parchment (#f7f4ed) as both ground and card, #1c1c1c ink, hot pink action (#ff4d8d), 6/12/16 radii, approachable and almost analog.
**Tokens:** background #f7f4ed · foreground #1c1c1c (16) · card #f7f4ed · muted #f7f4ed · muted-foreground #5f5f5d (5.8) · border #eceae4 · primary #db2872 (4.2 on ground) · primary-foreground #ffffff (4.6) · destructive #d82122 (4.6) · radius 6/12/16 · type 16/18/20/36/48/60 · body line 1.5 · display tracking -0.025em
**Fixed:** primary #ff4d8d → #db2872 so its label #ffffff holds 4.5 (3.1:1); destructive #dc2626 → #d82122 (4.4:1).
**Fonts:** Camera Plain (licensed) → Rethink Sans
**Learn:** Parchment ground with one bright accent; darken the pink for its white label.

## OpenCode
`opencode-ai` · devtool · VoltAgent awesome-design-md via open-design

**Works:** Warm near-black (#201d1d) with warm off-white text, monospace everywhere, blue (#007aff) action, 4/6/8 radii, narrow 880 container.
**Tokens:** background #201d1d · foreground #fdfcfc (16) · card #302c2c · muted #302c2c · muted-foreground #9a9898 (5.8) · border #464343 · primary #0072ee (3.7 on ground) · primary-foreground #ffffff (4.5) · destructive #ff3b30 (4.7) · radius 4/6/8 · type 16/18/22/28/38/48 · body line 1.5
**Fixed:** primary #007aff → #0072ee so its label #ffffff holds 4.5 (4.0:1).
**Fonts:** Berkeley Mono (licensed) → JetBrains Mono
**Learn:** Reference for a developer tool only; monospace is never for service or place.
**Upstream now:** VoltAgent's current file gives primary #201d1d on canvas #fdfcfc; faces Berkeley Mono.

## Resend
`resend` · devtool, saas · VoltAgent awesome-design-md via open-design

**Works:** Pure black, near-white text, an orange (#ff801f) with black label, a serif display (Domaine) beside a grotesk, 4/8/16 radii, cinematic spacing.
**Tokens:** background #000000 · foreground #f0f0f0 (18) · card #000000 · muted #000000 · muted-foreground #777777 (4.7) · border #d6ebfd · primary #ff801f (8.4 on ground) · primary-foreground #000000 (8.4) · destructive #ff2047 (5.5) · radius 4/8/16 · type 16/20/24/36/56/96 · body line 1.5 · display tracking -0.05em
**Fixed:** muted-foreground #5c5c5c → #777777 (3.1:1).
**Fonts:** ABC Favorit / Domaine (licensed) → Instrument Serif + Instrument Sans
**Learn:** A serif display on a quiet ground lends luxury to a plain service; keep it light outside tech.
**Upstream now:** VoltAgent's current file gives primary #fcfdff on canvas #000000; faces Domaine Display, ABC Favorit, Inter.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
