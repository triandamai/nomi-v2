---
name: practice-swiftui-design
description: "How to design app screens that look crafted, not generated: six principles, ten banned app-slop patterns, a brand asset protocol, five design schools, layout patterns at 390 and a five-dimension review. Read when you design or review app screens."
license: "MIT, Copyright (c) 2026 wholiver; via open-design (Apache-2.0)"
source: "https://github.com/wholiver/swiftui-design-skill@2c82638 (SKILL.md, references anti-ai-slop, design-review, layout-patterns, templates/brand-spec); open-design@5fdcc3d"
---
# Crafted app screens

Design app screens from context, offer real directions, ban the generated look, and score every screen on five dimensions before you hand it over.

The source is written for native SwiftUI apps. Koda builds a SvelteKit web app, so read "native" as "feels like a good phone app at 390 wide". Platform details (bars, sheets, safe areas, sizes) are in `practice-apple-hig` and `practice-platform-design`; the screen rules in `taste-app-screens`. Where this source's visual advice disagrees with Taste, **Taste wins**.

## Six principles

1. **Start from context, never blank.** Is there a design system frame, a brand, an existing app, a style guide? If none, write a brand sheet first (below).
2. **Hypothesise early, iterate fast.** A clean placeholder for a photo beats a bad drawn illustration; a common font with good spacing beats a fancy font with bad spacing; an honest rough layout beats a polished generic one.
3. **Offer variations, not one final.** Two or three directions from different philosophies, each with a reason, visibly different, not colour swaps.
4. **A placeholder beats a bad asset.** No drawn clipart, no emoji as icons; one clean icon set (Lucide fits shadcn), photo placeholders with their note (`pen-format`).
5. **System first, don't fill.** Every element earns its place: does it serve the user's goal, can it merge with another, what if it goes? Space is a feature.
6. **No app slop** (below).

## Ten banned patterns

| Banned | Instead |
|---|---|
| Purple-to-blue gradient grounds | a solid warm or cool neutral `$color-background` |
| Emoji as icons | one stroke icon family, one stroke width |
| Rounded cards with a coloured left border | a plain card (`$color-card`, hairline or soft shadow) with clear internal hierarchy, or no card |
| A marketing hero inside an app | show the content immediately (today's orders, the menu, the balance) |
| Fonts that feel ported from somewhere else | a pairing chosen for the brand and kind (`taste-type`); the source's "use Apple's system font" becomes "use a pairing that suits the product", not a platform font |
| Neon on near-black "cyber" dark | a proper dark theme with tonal surfaces (`practice-color-expert`) |
| Symmetric three-column feature grids | content-driven layouts: a list, a two-up, one big item |
| Drawn "people working" illustrations | real photographs, the product's own UI, a large icon, or a labelled placeholder ("Ganti dengan foto Anda") |
| Default blue everywhere | the brand's own accent token |
| Random spacing | the `$space-*` scale on a 4/8 grid |

The source recommends a serif display font as a fix; Taste deals type by kind, so use a serif only where the pairing calls for it.

## Brand asset protocol

1. **Ask** for the logo files, colours, fonts and any guidelines.
2. **Look** in what they already sent (`practice-own-assets`); you don't search the web for a small business's brand.
3. **Get the real files,** not a screenshot of the logo.
4. **Verify** colours against the real source (the logo file, the shop paint photo), not memory.
5. **Write a brand sheet** (`practice-anthropic-brand-guidelines`): main and secondary colours, fonts, logo rules, spacing.

Good enough to start: the brand's real colours (up to five), a set of verified tokens, two font families, an 8-point spacing grid. With no brand at all: one accent (warm or cool by the trade, never purple-blue), a neutral ramp tinted to it, a display and a body face, the 8-point grid.

## Direction workflow (when the brief is vague)

1. Understand: what the app is, who uses it, where (behind a counter, on a motorbike stop, at a desk); reference apps they like; existing assets.
2. Propose three directions from different schools:

| School | Looks like | Fits |
|---|---|---|
| Informational | data first, charts, clear figures | sales and stock tools, finance |
| Editorial | strong type, generous space, photographs | catalogues, menus, content apps |
| Expressive | bold colour, asymmetry, lively moments | youth brands, games, events |
| Functional | dense utility, mono figures, little decoration | cashier (POS), admin, field tools |
| Warm minimal | soft neutrals, rounded shapes, calm | booking, wellness, community |

3. For each: 3-5 colours with hex, a type pairing, density (comfortable, compact, spacious) and one **signature detail** per screen, done at 120% effort (a beautifully set price, a satisfying order-status step, a photo crop).
4. The person picks; you lock the system.

## Layout patterns at 390

- **Screen:** a compact top bar with the title left-aligned, content that scrolls, the main navigation in a bottom bar; nothing important behind the notch or the home indicator.
- **Cards:** only when the card is the interaction; one radius (12-16 for cards, Taste's ladder), a subtle shadow or a hairline, not a heavy border.
- **Lists:** separators inset from the left edge (aligned with the text, not the icon); group with section headers, not just dividers. Long lists get a search or filter at the top.
- **Grids:** let content decide the columns (two photos across at 390 for a catalogue); equal heights within a row; never the three-column feature grid.
- **Headers:** left-aligned, compact; don't spend half the first screen on a title.

## Five-dimension review

Score each 1-10 on your own frames; generated-looking UI usually scores 3-5. Fix anything below 7, then re-score.

| Dimension | Ask | 8-10 looks like |
|---|---|---|
| Philosophy | one coherent direction, or a mix of trends? | every element supports the direction; recognisable without the logo |
| Hierarchy | is the important thing obvious? | instant first, second, third; the main action dominant |
| Craft | are the details right? | the spacing scale, the type scale, tokens only, one radius, one icon weight, no one-word last lines, consistent image ratios |
| Function | does it serve the real task? | main action visible, 44 targets, loading, error and empty states drawn, predictable back, labelled fields |
| Originality | real product or template? | no banned pattern, one signature detail per screen, distinctive type and colour |

Write the review as a table with a note per dimension, then "perlu diperbaiki" (each issue with its fix) and "yang sudah bagus".

**Shipping threshold:** every dimension 7 or more, none below 5. 1-3 needs a redesign; 4-5 is generic; 6-7 is professional; 8-10 is distinctive.

Adapted from wholiver's swiftui-design-skill (MIT, Copyright (c) 2026 wholiver), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
