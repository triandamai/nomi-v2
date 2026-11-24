# Brands: AI products

AI-product systems: parchment salons, clinical whites and dark terminals, with what a studio, school or clinic can learn.

**Learn, don't copy.** These cards teach how a system works. Never reproduce a brand's identity for another business, and never name the brand in the person's design. Take structure and character, then set the person's own colours and type.

Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule).

## Claude (Anthropic)
`claude` · studio, law, school, wedding · VoltAgent awesome-design-md via open-design

**Works:** Parchment (#f5f4ed) instead of white, warm near-black ink, a serif display at 400 with a quiet sans, terracotta (#c96442) for the one action, 8/12/16 radii, unhurried 1.6 body line.
**Tokens:** background #f5f4ed · foreground #141413 (17) · card #faf9f5 · muted #e8e6dc · muted-foreground #5e5d59 (6.0) · border #f0eee6 · primary #b75230 (4.5 on ground) · primary-foreground #faf9f5 (4.7) · destructive #b53333 (5.5) · radius 8/12/16 · type 16/20/25/32/52/64 · body line 1.6
**Fixed:** primary #c96442 → #b75230 so its label #faf9f5 holds 4.5 (3.7:1).
**Fonts:** Anthropic Serif/Sans (licensed) → Newsreader + Hanken Grotesk
**Learn:** A bookshop, school or law office: parchment, serif headlines and one earthy accent; darken the accent for its label.
**Upstream now:** VoltAgent's current file gives primary #cc785c on canvas #faf9f5; faces Copernicus, StyreneB, JetBrains Mono.

## Claude (bergside reading)
`claude-bergside` · studio, law, school · bergside awesome-design-skills only

**Works:** bergside's reading of the same brand: warm ivory parchment, near-black slate ink, an almost achromatic palette where #141413 is the primary.
**Tokens:** background #faf9f6 · foreground #141413 (18) · card #ffffff · primary #141413 (18 on ground) · primary-foreground #faf9f6 (18)
**Checked:** every pair passes as shipped.
**Fonts:** Anthropic Sans (licensed) → Newsreader + Hanken Grotesk
**Learn:** Duplicate direction of the card above; use it only to confirm that ink-as-primary works on parchment.

## OpenAI
`openai` · saas, clinic, law · open-design's own brand file

**Works:** True white, #0d0d0d ink, a teal-green (#10a37f) accent, a serif (Signifier) for headlines, 12/16 radii, quiet and restrained.
**Tokens:** background #ffffff · foreground #0d0d0d (19) · card #f5f5f5 · muted #fafafa · muted-foreground #6e6e6e (5.1) · border #e5e5e5 · primary #0b8567 (4.6 on ground) · primary-foreground #ffffff (4.6) · destructive #df313a (4.5) · radius 12/16/16 · type 16/18/20/28/40/56 · body line 1.65 · display tracking -0.02em
**Fixed:** primary #10a37f → #0b8567 so its label #ffffff holds 4.5 (3.2:1); destructive #ef4146 → #df313a (3.8:1).
**Fonts:** Signifier / Söhne (licensed) → Source Serif 4 + Libre Franklin
**Learn:** Darken the green for a white label. Serif headline over a clean white page suits a clinic.

## Cohere
`cohere` · saas, fintech · VoltAgent awesome-design-md via open-design

**Works:** White, black ink, blue (#1863dc), cards at a generous 22px radius, a grotesk display; muted grey fails and needs darkening.
**Tokens:** background #ffffff · foreground #000000 (21) · card #fafafa · muted #fafafa · muted-foreground #72727e (4.7) · border #d9d9dd · primary #1863dc (5.4 on ground) · primary-foreground #ffffff (5.4) · destructive #db3035 (4.7) · radius 8/22/22 · type 16/18/24/32/60/72 · body line 1.5 · display tracking -0.02em
**Fixed:** muted-foreground #93939f → #757581 (3.0:1); muted-foreground → #72727e so it also holds 4.5 on muted; destructive #ef4444 → #db3035 (3.8:1).
**Fonts:** Cohere Text / Unica77 (licensed) → Host Grotesk
**Learn:** Generous rounded cards on white feel organised for a consulting offer.
**Upstream now:** VoltAgent's current file gives primary #17171c on canvas #ffffff; faces CohereText, Unica77 Cohere Web, CohereMono.

## Mistral AI
`mistral-ai` · cafe, bakery, studio · VoltAgent awesome-design-md via open-design

**Works:** Cream-to-amber grounds (#fffaeb, #fff0c2), near-black ink, burnt orange (#fa520f), near-square 2/4 radii, block-pixel motifs.
**Tokens:** background #fffaeb · foreground #1f1f1f (16) · card #fff0c2 · muted #fff0c2 · muted-foreground #3d3d3d (10) · border #f0e8c8 · primary #d64209 (4.3 on ground) · primary-foreground #ffffff (4.5) · destructive #dc2626 (4.6) · radius 2/2/4 · type 16/24/32/48/56/82 · body line 1.5 · display tracking -0.025em
**Fixed:** primary #fa520f → #d64209 so its label #ffffff holds 4.5 (3.3:1).
**Fonts:** Arial (system) → Archivo
**Learn:** Warm amber grounds and burnt orange suit a bakery; darken the orange for its white label.

## ElevenLabs
`elevenlabs` · studio, agency · VoltAgent awesome-design-md via open-design

**Works:** White and #f5f5f5, black ink and black buttons, a light display serif (Waldenburg) at 300, 8/16/24 radii, faint shadows.
**Tokens:** background #ffffff · foreground #000000 (21) · card #f5f5f5 · muted #f5f2ef · muted-foreground #746e66 (5.0) · border #e5e5e5 · primary #000000 (21 on ground) · primary-foreground #ffffff (21) · destructive #dc2626 (4.8) · radius 8/16/24 · type 16/20/24/32/36/48 · body line 1.5 · display tracking -0.02em
**Fixed:** muted-foreground → #746e66 so it also holds 4.5 on muted.
**Fonts:** Waldenburg (licensed) → Instrument Serif + Instrument Sans
**Learn:** Black buttons and a light display serif make a sound studio or podcast page feel premium.
**Upstream now:** VoltAgent's current file gives primary #292524 on canvas #f5f5f5; faces Waldenburg, Inter.

## Hugging Face
`huggingface` · community, school · VoltAgent awesome-design-md via open-design

**Works:** Off-white (#fafafa), #0d1117 ink, yellow (#ffd21e) with dark label, monospace voice, 4/6/8 radii, emoji as logo.
**Tokens:** background #fafafa · foreground #0d1117 (18) · card #ffffff · muted #fff4cc · muted-foreground #686f7d (4.8) · border #e5e7eb · primary #0d1117* · primary-foreground #fafafa · brand #ffd21e · destructive #dc2626 (4.6) · radius 4/6/8 · type 15/18/24/32/40/48 · body line 1.55 · display tracking -0.01em
**Fixed:** muted-foreground → #686f7d so it also holds 4.5 on muted; primary #ffd21e is 1.4:1 on the ground (under 3): buttons and links take the ink, #ffd21e becomes $color-brand for small marks and one band (Taste *).
**Fonts:** IBM Plex Mono → Reddit Sans
**Learn:** Yellow fails on white (Taste *). Learn the friendly community voice, not the mono face.

## MiniMax
`minimax` · saas, product · VoltAgent awesome-design-md via open-design

**Works:** White, #222222 ink, slate muted (#45515e), blue (#1456f0), colourful product cards, 8/13/20 radii.
**Tokens:** background #ffffff · foreground #222222 (16) · card #fafafa · muted #f0f0f0 · muted-foreground #45515e (8.1) · border #e5e7eb · primary #1456f0 (5.8 on ground) · primary-foreground #ffffff (5.8) · destructive #dc2626 (4.8) · radius 8/13/20 · type 16/20/24/32/48/80 · body line 1.5 · display tracking -0.01em
**Checked:** every pair passes as shipped.
**Fonts:** Outfit (Google Fonts) → keep Outfit
**Learn:** Colour lives in the product cards, not the chrome.
**Upstream now:** VoltAgent's current file gives primary #0a0a0a on canvas #ffffff; faces DM Sans.

## Ollama
`ollama` · devtool, studio · VoltAgent awesome-design-md via open-design

**Works:** White, black, nothing else; pill (9999) buttons and inputs, 12px cards, no shadow, no colour.
**Tokens:** background #ffffff · foreground #000000 (21) · card #fafafa · muted #fafafa · muted-foreground #737373 (4.7) · border #e5e5e5 · primary #000000 (21 on ground) · primary-foreground #ffffff (21) · destructive #dc2626 (4.8) · radius 9999/12/9999 · type 16/18/24/30/36/48 · body line 1.5
**Checked:** every pair passes as shipped.
**Fonts:** SF Pro Rounded (system) → Nunito
**Learn:** Proof that black on white with pill controls is a complete system for a one-product shop.

## Perplexity AI
`perplexity` · ai, saas · open-design's own brand file

**Works:** Dark (#0f0f10), #f0f0f0 text, a violet (#a855f7) accent Taste turns rose, 4/8/12 radii, no gradient hero.
**Tokens:** background #0f0f10 · foreground #f0f0f0 (17) · card #19191a · muted #19191a · muted-foreground #9b9b9b (6.9) · border #2e2e30 · primary #d5259e (4.2 on ground) · primary-foreground #ffffff (4.6) · destructive #ef4444 (5.1) · radius 4/8/12 · type 15/16/18/22/28/32 · body line 1.65 · display tracking -0.01em
**Fixed:** primary #a855f7 is violet (OKLCH hue 304) → #e133a7; primary #e133a7 → #d5259e so its label #ffffff holds 4.5 (4.0:1).
**Fonts:** Inter (banned) → Geist
**Learn:** Learn "content leads, chrome disappears"; dark stays tech only.

## Replicate
`replicate` · ai, agency · VoltAgent awesome-design-md via open-design

**Works:** White body, #202020 ink, orange-red (#ea2804), all-pill radii, a huge display scale to 128px.
**Tokens:** background #ffffff · foreground #202020 (16) · card #f8f8f8 · muted #f8f8f8 · muted-foreground #646464 (5.9) · border #202020 · primary #e22e12 (4.5 on ground) · primary-foreground #ffffff (4.5) · destructive #dc2626 (4.8) · radius 9999/9999/9999 · type 16/18/30/48/72/128 · body line 1.5 · display tracking -0.025em
**Fixed:** primary #ea2804 → #e22e12 so its label #ffffff holds 4.5 (4.4:1).
**Fonts:** rb-freigeist (licensed) → Bricolage Grotesque + Figtree
**Learn:** A loud poster scale is for one hero line; keep body plain.

## Runway
`runwayml` · studio, agency · VoltAgent awesome-design-md via open-design

**Works:** Black ground, white type and white buttons with black label, 4/8/16 radii; full-bleed film stills are the interface.
**Tokens:** background #000000 · foreground #ffffff (21) · card #1a1a1a · muted #030303 · muted-foreground #767d88 (5.1) · border #27272a · primary #ffffff (21 on ground) · primary-foreground #000000 (21) · destructive #e02b29 (4.5) · radius 4/8/16 · type 16/20/24/36/40/48 · body line 1.4 · display tracking -0.025em
**Fixed:** destructive #dc2626 → #e02b29 (4.3:1).
**Fonts:** ABC Normal (licensed) → Host Grotesk
**Learn:** For a videographer: let stills fill frames; keep chrome to white text.

## Together AI
`together-ai` · ai, saas · VoltAgent awesome-design-md via open-design

**Works:** White, black ink, near-black navy buttons (#010120), pastel painterly hero art, 4/8 radii.
**Tokens:** background #ffffff · foreground #000000 (21) · card #ffffff · muted #ffffff · muted-foreground #666666 (5.7) · border #ebebeb · primary #010120 (20 on ground) · primary-foreground #ffffff (20) · destructive #dc2626 (4.8) · radius 4/8/8 · type 16/18/22/28/40/64 · body line 1.3 · display tracking -0.03em
**Checked:** every pair passes as shipped.
**Fonts:** The Future (licensed) → Familjen Grotesk
**Learn:** Gradients can't be drawn here; take the ink buttons and the calm body.
**Upstream now:** VoltAgent's current file gives primary #000000 on canvas #ffffff; faces The Future, PP Neue Montreal Mono.

## VoltAgent
`voltagent` · devtool, ai · VoltAgent awesome-design-md via open-design

**Works:** Near-black green-tinted ground (#07110c), mint-white text, electric green (#7cff6b) with a dark label, 8/12/18 radii.
**Tokens:** background #07110c · foreground #f4fff8 (19) · card #101c16 · muted #14281e · muted-foreground #8ba493 (7.2) · border #254132 · primary #7cff6b (15 on ground) · primary-foreground #07110c (15) · destructive #fb7185 (7.1) · radius 8/12/18 · type 15/17/22/32/46/64 · body line 1.56 · display tracking -0.02em
**Checked:** every pair passes as shipped.
**Fonts:** Inter (banned) → Geist
**Learn:** Tech only.
**Upstream now:** VoltAgent's current file gives primary #00d992 on canvas #101010; faces Inter, SFMono-Regular.

## xAI
`x-ai` · devtool, ai · VoltAgent awesome-design-md via open-design

**Works:** Charcoal (#1f2228), white text and white buttons, mono display, square 0/0/4 radii.
**Tokens:** background #1f2228 · foreground #ffffff (16) · card #ffffff · muted #ffffff · muted-foreground #757575 (3.5) · border #ffffff · primary #ffffff (16 on ground) · primary-foreground #1f2228 (16) · destructive #fc483e (4.7) · radius 0/0/4 · type 16/20/24/30/40/64 · body line 1.5
**Fixed:** muted-foreground → #757575 so it also holds 4.5 on muted; destructive #dc2626 → #fc483e (3.3:1).
**Fonts:** Geist Mono / Universal Sans → Geist Mono + Geist
**Learn:** Tech only; mono is never for service or place.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
