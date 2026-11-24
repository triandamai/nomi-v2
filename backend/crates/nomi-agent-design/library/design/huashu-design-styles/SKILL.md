---
name: huashu-design-styles
description: "Huashu's colour derivation protocol (sample, converge, argue), print-like chroma levels, colour in cultural context, how to use its 92-style library by medium and temperature, and the styles that only work with generated images. Read it when choosing colour or a style for a deck, video or image."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Design styles and the colour protocol

Huashu's colour derivation protocol (sample, converge, argue), print-like chroma levels, colour in cultural context, how to use its 92-style library by medium and temperature, and the styles that only work with generated images.

For sites and apps, `taste-color` (palettes by trade) and `styles/styles-index` decide; use this protocol to justify a client's own colours or to fill gaps. For decks, video and images it leads. The style library is ammunition for when you have no idea, never a list you must pick from: when the person gives content, brand or references, the design grows from those.

## The colour protocol: sample, converge, argue

Every hex in a style entry is an example anchor, not a recipe. The same style on different content must end in different colours; copying an entry's hex is just making better-looking slop.

| Step | Do | Why |
|---|---|---|
| 1 Sample | Take the main colours from three sources, never invent: the brand's assets (logo, signage, packaging); the content's real photos (the product, the place); the subject's cultural memory (table below) | Invented colour is a draw from your defaults, always the same handful of trendy colours; sampled colour carries its own "why" |
| 2 Converge | Reduce to **two or three hues plus one neutral set**. Write the neutrals as a lightness ladder (for example L 0.15, 0.35, 0.65, 0.92, 0.98 in OKLCH). Keep hues at least 60° apart, or 0.3 apart in lightness | More colours means mess; a lightness ladder is already a hierarchy |
| 3 Argue | Write one sentence on why this colour, in your plan or handover: "Primer dari warna tanah liat gentong, saturasi diturunkan supaya terasa seperti tinta cetak" | **If you can't write it, you are copying a recipe.** The sentence is the anti-slop gate |

### Why low saturation looks more expensive

Ink on paper never reaches screen saturation: a narrower gamut, absorbent paper and reflected light all grey it down. Decades of print trained the eye to read that greyness as quality, so lowering chroma on screen borrows print's memory.

| Use | Chroma (OKLCH) | Effect |
|---|---|---|
| Large grounds | 0.01 to 0.04 | Paper-like, calm |
| Brand primary, emphasis | 0.08 to 0.15 | Ink-like: visible but not plastic |
| Small accents (buttons, links) | 0.15 to 0.22 | Lively, small areas only |
| Above 0.25 over a full field | Rarely | Screen-neon; only for deliberately "digital native" styles |

### Same hue, different culture

Choosing a colour is choosing a cultural coordinate. Indonesian anchors to sample from:

| Hue | Context A | Context B | The difference |
|---|---|---|---|
| Red | Brick and terracotta of old Javanese temples and roof tiles: orange-leaning, greyed, dark; heritage, warmth | Flag and promo red at full saturation: retail, urgency, "diskon" | Lower the chroma and you move from the price tag to the temple |
| Blue | Indigo of tenun ikat and batik indigo: deep, grey-violet; handmade, calm | "Tech blue" around #0066FF: SaaS, fintech | The second is your default; ask if you are just drawing lots |
| Green | Young rice field, pandan, tea leaf: yellow-leaning, soft; nature, kitchen | Neon terminal green: hacker | One drinks tea, one types code |
| Yellow | Turmeric and old batik sogan browns: brownish, greyed; tradition, print | Warning and delivery-brand yellow: loud, playful | Grey decides old page or safety helmet |
| White | Rice-paper and unbleached cotton off-white: warm, editorial | Pure #FFFFFF: lab, Swiss | Two percent of warmth separates them |

## The 92-style library

Six sections, chosen by **output form, not subject**: something clickable goes to Web (20); something you page through goes to Slides (20); one or a set of images where data is the hero, readable without interaction, goes to Infographic (20); mobile prototypes go to App (12); 3:4 social carousels go to Social carousel (8); **Chinese and East Asian (12) is cross-media** and may serve any form. A dashboard prototype is Web; a data slide inside a deck is Slides.

- **Temperature:** each style is bold, neutral or quiet. The library is deliberately bold-heavy, because your own bias drifts quiet and minimal. Never let all three directions share one temperature (all "off-white, whitespace, one accent").
- **Fidelity:** each style is rated for how much survives in plain HTML and CSS without generated images. 90% or more: build freely. 70 to 90%: the body works, some details downgrade. Under 70% (worn Memphis textures, hand-painted cross-sections): say in the deliverable which part became flat colour; don't pretend to the original texture.
- **Fonts:** each entry names open replacements for paid faces; check them against the overused list in `huashu-typography`, which wins where they conflict.
- **Photos:** styles that need real photography (dark gallery, white-box gallery, map-led) need the images gathered first.

The full index, one line per style with temperature and fit, is in `references/style-index.md`. The roulette in `huashu-directions` draws from it.

## Styles that need generated images

Their soul is generated visuals, 3D, particles, cinematic lighting or hand illustration; in plain HTML they become a badly degraded mock. Offer them only when image generation is confirmed for this job (Gemini, see `huashu-image-briefs`), mainly for single images, covers and video plates:

| Style | Soul | Why HTML can't |
|---|---|---|
| Active Theory | Real-time WebGL particles | No CSS equivalent |
| Field.io | Algorithmic generative graphics | Static vectors look frozen |
| Resn | Character illustration, game-like | Needs drawn assets |
| Zach Lieberman | Live creative-coding strokes | Needs live generation |
| Raven Kwok | Recursive fractal parameters | Complexity beyond CSS |
| Ash Thorp | Cinematic volumetric light, concept art | CSS light is a downgrade |
| Territory Studio | Sci-fi holographic interfaces | Needs layered glowing assets |
| Neo Shen | Ink-wash bleeding | Gradients are not ink |
| Sagmeister & Walsh | Hand-made objects, experimental type | The colour clash works in HTML, the handmade texture doesn't |

They aren't bad; their native medium is a generated image, not the browser.

## Default no-go zone (the person's own brand may override)

- The lazy GitHub-dark answer: an even dark navy ground (#0D1117) with generic cyan or violet neon glow. Only this combination is banned, not dark design: cinematic dark lighting, warm cyberpunk orange-teal, dark stages with an author's intent are all legitimate.
- The all-purpose violet gradient, emoji as icons, rounded cards with a coloured left border, unless the brand itself uses them.
- Personal signatures or watermarks on covers.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
