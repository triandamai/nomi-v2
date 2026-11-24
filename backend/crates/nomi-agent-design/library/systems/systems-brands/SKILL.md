---
name: systems-brands
description: "Cards for the 99 brand-inspired design systems (fintech, cafes, cars, AI tools…): why each works, its tokens in Sora's names with contrast fixed, and what a small business can learn; read one when a brief resembles a product you'd learn from, never to copy it."
license: "Apache-2.0 (open-design, Copyright (c) 2026 nexu-io); MIT, Copyright (c) 2026 VoltAgent; MIT, Copyright (c) 2026 Bergside"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/design-systems; https://github.com/VoltAgent/awesome-design-md (fcc7028, 28 Oct 2026); https://github.com/bergside/awesome-design-skills (f631a09, 28 Jun 2026)"
---
# Systems: brand-inspired

Cards for the 99 brand-inspired design systems: why each works, its tokens in your names with contrast fixed, and what a small business can learn. Read one when a brief resembles a product worth learning from, never to copy it.

## Learn, don't copy

These systems describe real companies' public websites. They are aesthetic readings, not official assets, and they are here so you understand how good systems are built.

- Take the **structure and character**: how the ground, ink and accent share the work, the radius ladder, the type scale ratio, the density, the component habits.
- Then set the **person's own** colours (from their shopfront, logo, packaging or photos) and a face from `taste-type`. The card's palette is a worked example, not a palette to deal.
- Never reproduce a brand's name, mark, signature colour pairing, custom typeface, slogan or imagery for another business, and never name the brand anywhere in the person's design or handoff.
- Taste wins every conflict: dark pages are tech only, violet turns blue or rose, Inter and its kin are swapped, labels stay sentence case.

## How to read a card

Each card gives:
- **Where it came from:** the package id, best-fit trades, and its source: VoltAgent awesome-design-md via open-design (70), open-design's own brand file (15), VoltAgent only (13), or bergside only (1). "Upstream ships an Indonesian DESIGN-id.md" marks the 23 open-design packages with an Indonesian translation (airbnb, airtable, apple, cohere, discord, duolingo, figma, github, ibm, nike, notion, nvidia, openai, pinterest, playstation, shopify, slack, spotify, starbucks, stripe, uber, vercel, webflow). They are useful phrasing when you explain a direction in Bahasa Indonesia.
- **Works:** what makes the system hold together.
- **Tokens:** open-design's tokens.css (or VoltAgent's frontmatter for VoltAgent-only systems) mapped to `$color-*`, plus radius sm/md/lg, the type ladder from body to display, body line height and display tracking. The number after a colour is its contrast with background. After primary-foreground, it is the contrast with primary.
- **Fixed / Checked:** every pair below 4.5:1 for text (7:1 for foreground), every violet hue, and every accent under 3:1 on the ground, with the replacement. An accent marked * becomes `$color-brand` while buttons take the ink.
- **Fonts:** the system's face, usually licensed, and a Google Fonts stand-in from `taste-type`.
- **Learn:** the lesson for an Indonesian small business, or "reference only" when there isn't one.
- **Upstream now:** VoltAgent has rewritten 47 of the 70 shared files since open-design copied them. Where the primary changed, the card notes the current value so you can see both readings.

## Turning a card into the person's system

1. Read the card's Works line and pick the two or three habits that fit the brief (for example "navy ink, one accent, 4/6/8 radii").
2. Build tokens with `components-kit`: ground from the person's place or product, ink near-black (warm or cool to match), accent from their brand, darkened until its label passes 4.5:1 and it passes 3:1 on the ground.
3. Keep the card's ratios, not its values: the radius ladder steps, the type scale ratio, the section rhythm.
4. Write the direction on plan line 2 by character, and the source nowhere.

## References

| File | Holds |
|---|---|
| `references/brands-commerce.md` | Airbnb, Nike, Shopify, Starbucks, Meta Store, Pinterest, Xiaohongshu, mydealz, DJI, Oura, Uber |
| `references/brands-enterprise.md` | WeChat, Vodafone, HP, Oracle, SAP, IBM, Cisco, Webex |
| `references/brands-fintech.md` | Stripe, Coinbase, Binance.US, Kraken, Revolut, Wise, Mastercard, Mercury, NerdWallet, 1win |
| `references/brands-saas.md` | Notion, Slack, Linear, Cal.com, Intercom, Zapier, Airtable, Miro, Mintlify, Loom, Superhuman, Arc |
| `references/brands-devtools.md` | Vercel, GitHub, Cursor, Expo, Raycast, Warp, Lovable, OpenCode, Resend |
| `references/brands-data.md` | Supabase, Sentry, PostHog, Sanity, MongoDB, ClickHouse, HashiCorp, Composio |
| `references/brands-ai.md` | Claude (and bergside's reading of it), OpenAI, Cohere, Mistral AI, ElevenLabs, Hugging Face, MiniMax, Ollama, Perplexity, Replicate, Runway, Together AI, VoltAgent, xAI |
| `references/brands-design.md` | Figma, Framer, Canva, Webflow, Clay, Duolingo, Discord, Jobright |
| `references/brands-media.md` | Apple, NVIDIA, PlayStation, Spotify, SpaceX, The Verge, WIRED, Al Jazeera, Totality Festival, Nintendo 2001, Dell 1996 |
| `references/brands-auto.md` | BMW, BMW M, Bugatti, Ferrari, Lamborghini, Renault, Tesla |

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
