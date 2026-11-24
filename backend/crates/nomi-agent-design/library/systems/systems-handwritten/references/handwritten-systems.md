# Hand-written systems

The ten open-design packages written by a person rather than synced from a catalogue, with their tokens in Sora's names, checked, and what each teaches.

Token names are Sora's `$color-<name>`. The number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. Several of these packages ship a tokens.css that reuses a shared template and disagrees with their own DESIGN.md. Where that happens, both readings are given: the tokens line (what open-design compiled) and the own line (what the author wrote). For a direction, take the own palette; it carries the character.

## Neutral Modern (`default`)
open-design starter · suits saas, clinic, retail, any brief without a mood

Calm, functional, content first: a near-white floor, white cards with a 1px border and no shadow, cobalt for primary actions and one hero element a screen, 8px buttons and inputs, 12px cards.
**Tokens:** background #fafafa · foreground #111111 (18) · card #ffffff · muted-foreground #6b6b6b (5.1) · border #e5e5e5 · primary #2f6feb (4.4 on ground) · primary-foreground #ffffff (4.6) · destructive #dc2626 (4.6) · radius 8/12/16 · type 16/20/24/32/48/64 · body line 1.5.
**Checked:** every pair passes as shipped (tokens and DESIGN.md agree).
**Fonts:** Inter (banned by Taste) → Host Grotesk, or Hanken Grotesk for a service.
**Learn:** This is the floor every other system departs from. Use it when the person has no brand yet, then change the primary to their colour and pick a face from taste-type.

## Warm Editorial (`warm-editorial`)
open-design starter · suits studio, school, bakery, law, long-form

A serif-led magazine page: warm off-white paper, near-black warm ink, terracotta for links, primary actions and at most one hero element a page, forest green for dividers and tags. Cards are off-white with a forest hairline at 8% and a 16px radius; no radius under 8 or over 24; shadow only on hover.
**Tokens:** background #fbf6ee · foreground #201914 (16) · card #fffdf8 · muted #f1e3cf · muted-foreground #71645a (5.3) · border #ded2c3 · primary #9b5b32 (5.0 on ground) · primary-foreground #ffffff (5.3) · destructive #b33a3a (5.4) · radius 10/16/24 · type 17/20/28/42/64/88 · body line 1.62 · display tracking -0.025em.
**Own palette:** background #faf7f2 · foreground #1c1a17 (16) · muted-foreground #786f68 (4.6) · primary #bd4e2c (4.6 on ground) · primary-foreground #faf7f2 (4.6) · second accent #2f5b4f (forest).
**Fixed:** tokens muted-foreground darkened so it holds 4.5 on muted too; own muted-foreground #8a817a → #786f68 (was 3.6:1); own terracotta #c0512f → #bd4e2c (its paper label was 4.4:1).
**Fonts:** GT Sectra + Söhne (licensed) → Newsreader + Hanken Grotesk.
**Learn:** The upstream tells agents to lean harder on serif and whitespace for "serious" and to pick another system for "modern". That is a good honesty rule for directions.

## kami (`kami`)
tw93/kami, MIT · suits law, school, studio, reports, decks

One sentence: warm parchment, ink-blue accent, serif carries hierarchy, no cool greys, no hard shadows. It is a constraint system for printed-feeling deliverables. Sans equals serif (a single serif family at one weight, 500), no italic anywhere, tabular figures wherever numbers stack, tags as solid tints of the ink blue, and an 8px default radius (12 featured, 16–24 hero).
**Tokens:** background #f5f4ed · foreground #141413 (17) · card #faf9f5 · muted #e8e6dc · muted-foreground #504e49 (7.5) · border #e8e6dc · primary #1b365d (11 on ground) · primary-foreground #faf9f5 (12) · destructive #8a3a30 (7.0) · radius 4/8/12 · type 14/17/22/32/48/96 · body line 1.55 · display tracking -1.2px.
**Checked:** every pair passes; the strongest-contrast system in the collection.
**Fonts:** Charter / YuMincho / TsangerJinKai (CJK) → Source Serif 4 for both jobs (Taste allows one family), or Newsreader.
**Learn:** For a notary, school report or price list: one serif, one ink-blue, and tinted tags instead of coloured badges. Ink-blue is too dark to darken further on hover, so kami keeps hover equal to rest; draw that column identical in the state matrix.

## Tom Modern (`tom-modern`)
Tom-Opencart/tom-modern-html-style-rule, MIT · suits agency, devtool, workshop, studio

Editorial-technical: structured, sharp, restrained, anti-generic. A neutral near-white canvas (never warm), deep ink, one vermillion carrying all emphasis, Geist for everything with Geist Mono for labels and metadata, every corner 0px, hard offset shadows instead of soft ones, ink-filled primary buttons, uppercase mono tab labels with an accent underline.
**Tokens:** background #fafafa · foreground #27272a (14) · card #ffffff · muted-foreground #71717a (4.6) · border #969696 · primary #cc4901 (4.5 on ground) · primary-foreground #ffffff (4.6) · destructive #dc2626 (4.6) · radius 0/0/0 · type 15/18/22/28/38/48 · body line 1.65 · display tracking -0.04em.
**Fixed:** primary #ff5a00 → #cc4901 so its white label holds 4.5 (was 3.1:1). The original stays as `$color-brand` for accent words and marks.
**Fonts:** Geist + Geist Mono (Google Fonts) → keep. Mono labels only on a tech page; Taste keeps labels sentence case, so drop the uppercase.
**Learn:** Hard shadows on a sharp grid are drawable on the canvas (one shadow, blur 0, offset 4/4), which makes this the best "anti-template" direction for a workshop or agency.

## Atelier Zero (`atelier-zero`)
open-design editorial · suits studio, agency, hotel, wedding

A small high-craft studio's annual report as a website: warm handmade-paper canvas, collage imagery of plaster and architecture, oversized display type, hairline rules, Roman-numeral section markers and tiny editorial notes. Coral is the only spark; mustard (a single star in the nav) and olive are quiet companions. Pill (999) buttons, bone-fill cards at 18px.
**Tokens:** background #ffffff · foreground #111111 (19) · card #f7f7f7 · muted #eeeeee · muted-foreground #6a6a6a (5.4) · border #d9d9d9 · primary #111111 · primary-foreground #ffffff (19) · radius 4/8/12. The tokens are the shared ink-on-white template, not the author's palette.
**Own palette:** background #efe7d2 (paper) · card #f7f1de (bone) · foreground #15140f (15) · primary #c84e3b (3.7 on ground) · primary-foreground #ffffff (4.6) · mustard #e9b94a · olive #6e7448.
**Fixed:** coral #ed6f5c → #c84e3b so its white label holds 4.5 (was 3.0:1).
**Fonts:** a grotesk display at very large sizes → Instrument Serif + Instrument Sans, or Mona Sans for the oversized display.
**Learn:** Roman numerals, hairline rules and tiny notes make a studio page feel printed without a single effect; all of it draws as text and 1px strokes.

## HUD (`hud`)
open-design themed · suits game, hardware

A combat pilot's head-up display: phosphor green on near-black, all-caps data overlays, angular geometry, glow instead of shadow, monospace readouts.
**Tokens:** the shared dark-blue template: background #090b12 · foreground #f8fafc (19) · primary #60a5fa · primary-foreground #06101d (7.5) · radius 10/16/24.
**Own palette:** background #0a0a0a · card #111316 · border #1e2328 · foreground and primary #00ff41 (15) · primary-foreground #0a0a0a · standby #7fff00 · grid #5a9a5a · warning #ffb800 · destructive #ff3b3b (5.6).
**Fonts:** JetBrains Mono + Inter → JetBrains Mono + Geist.
**Learn:** Game and hardware reference only. Its one transferable rule: every element is functional or it doesn't exist.

## Mission Control (`mission-control`)
open-design themed · suits devtool, hardware, logistics dashboards

A deep-space command centre: dark, information-dense, amber telemetry on navy as the core pair, everything else subdued, monospace precision.
**Tokens:** the shared dark-blue template (as HUD), destructive #ff4757 (5.9).
**Own palette:** background #0b1120 · card #111827 · border #1e3a5f · foreground #e8f0fe (16) · muted-foreground #8ba3c7 (7.3) · primary #ffb800 (11 on ground) · primary-foreground #0b1120 · healthy #00d4ff · success #26de81 · warning #ff9f43 · destructive #ff4757 (5.6).
**Fonts:** Inter + JetBrains Mono → Geist + Geist Mono.
**Learn:** A status board for a delivery fleet or warehouse app: one data colour for values and one for health, everything else grey.

## Trading Terminal (`trading-terminal`)
open-design themed · suits fintech, devtool

A Bloomberg-style terminal: dark only, data-dense, tabular layouts without decorative spacing, cyan for buy or gain and coral for sell or loss, readable from two metres.
**Tokens:** background #070b12 · foreground #f8fafc (19) · card #101826 · muted-foreground #8492a6 (6.2) · primary #38bdf8 (9.2 on ground) · primary-foreground #03111a (8.9) · destructive #ef4444 (5.2) · radius 4/8/12.
**Own palette:** background #0d0d0d · card #141414 · border #2a2a2a · foreground #ffffff (19) · muted-foreground #aaaaaa (8.4) · primary #00d4aa (10) · primary-foreground #0d0d0d · loss #ff4757 (5.8) · warning #ffb800.
**Fonts:** Inter + JetBrains Mono → IBM Plex Sans + JetBrains Mono for figures.
**Learn:** Gain and loss as a fixed colour pair with a sign (+/−) too, never colour alone. That carries to any sales report screen.

## Urdu Modern (`urdu`)
open-design editorial · suits school, community, publishing; the model for any right-to-left or bilingual page

Digital heritage: calm, spacious, respectful, built for Nastaliq script with native RTL and bilingual harmony. Typography is the protagonist; colours echo the Indus region's earth tones. Nastaliq needs a much taller line height and a larger size than Latin text, and weights must never animate.
**Tokens:** background #f3faf4 · foreground #15261a (15) · card #ffffff · muted #e5f5e7 · muted-foreground #5f7164 (4.9) · border #d4e3d7 · primary #0d863d (4.4 on ground) · primary-foreground #ffffff (4.7) · radius 10/16/24.
**Own palette:** background #f4f1ea (parchment) · foreground #1a202c (14) · muted-foreground #4a5568 (6.7) · border #e2e8f0 · primary #0f595e (7.1 on ground) · primary-foreground #ffffff (8.0) · accent #c05621 (rust, large text and marks only).
**Fixed:** tokens muted-foreground #6b7d70 → #5f7164 (was 4.1:1); tokens primary #1f8f46 → #0d863d (its label was 4.1:1). The own palette passes.
**Fonts:** Noto Nastaliq Urdu (Google Fonts) for Urdu, Inter for Latin → keep Noto Nastaliq Urdu, pair it with Hanken Grotesk.
**Learn:** For a page with Arabic-script text (a pesantren or Arabic course), mirror the layout for RTL, give the script its own size and line height, and keep both languages on one baseline grid.

## Cloudflare Kumo (`cloudflare-kumo`)
cloudflare/kumo 2.9.2, MIT, Copyright (c) 2026 Cloudflare, Inc.; open-design integration · suits saas, devtool, dashboards

A compact, semantic component language: layered neutral surfaces, precise type, accessible controls, automatic light and dark roles, and chart palettes chosen per task. It is an official open-source library, so its tokens are real, not inferred.
**Tokens (light):** background #fbfbfb · foreground #18181b (17) · card #ffffff · muted #f2f2f2 · muted-foreground #6d6d6d (5.0) · border #e9e9e9 · primary #056dff (4.4 on ground) · primary-foreground #ffffff (4.5) · destructive #e70627 (4.6) · radius 4/6/8 · type 14/16/20/24/30/36 · body line 1.5.
**Fixed:** muted-foreground #737373 → #6d6d6d so it also holds 4.5 on muted; destructive #fb2c36 → #e70627 (was 3.7:1). The dark set (background #0f0f0f-ish oklch 10%, card oklch 17%) is defined in oklch; convert per pair and recheck.
**Fonts:** Inter (banned by Taste) → Geist.
**Learn:** The best app-screen reference in the collection: dense but legible, every state defined, light and dark as two real sets. Use it for a person's admin or dashboard, never for their public page.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and tw93/kami (MIT, Copyright (c) 2026 Tw93), Tom-Opencart/tom-modern-html-style-rule (MIT, Copyright (c) 2026 Tom-Opencart), cloudflare/kumo (MIT, Copyright (c) 2026 Cloudflare, Inc.). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
