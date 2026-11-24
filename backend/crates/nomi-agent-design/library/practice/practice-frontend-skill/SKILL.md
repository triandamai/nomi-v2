---
name: practice-frontend-skill
description: "OpenAI's playbook for restrained, image-led pages and calm app screens: a visual thesis first, the first screen as a poster, cardless layouts, utility copy in apps, and seven classic failures to reject. Read when you plan a landing page or app surface."
license: "Apache-2.0 (openai/skills, skills/.curated/frontend-skill/LICENSE.txt); via open-design (Apache-2.0)"
source: "https://github.com/openai/skills@82d2c5b (skills/.curated/frontend-skill; removed upstream later in 11c6438); open-design@5fdcc3d"
---
# Restrained composition for pages and apps

Start from composition, not components: one big idea, strong real imagery, sparse copy, rigorous spacing, a few memorable moments, and no card where plain layout would do.

Much of this agrees with Taste. Where it doesn't (scroll-linked effects, carousels, "brand as the loudest text"), **Taste wins**, as marked below.

## Write three things before drawing

- **Visual thesis:** one sentence on mood, material and energy. "Tenang dan hangat: kayu, tepung, cahaya subuh; foto besar, sedikit kata."
- **Content plan:** hero, support, detail, close.
- **Interaction thesis:** two or three moments that change how the page feels (in Taste's terms: the hero entrance, one or two dealt moments, the hover layer).

Each section gets one job, one dominant visual idea, and one main takeaway or action.

## Beautiful defaults

- Composition first, components second.
- A full-bleed hero or a full-frame visual anchor.
- Copy short enough to scan in seconds.
- Use space, alignment, scale, cropping and contrast before adding boxes or decoration.
- Two typefaces at most, one accent colour.
- Cardless by default: sections, columns, dividers, lists and media blocks instead.
- The first screen is a poster, not a document.

## Landing pages

Default sequence: **hero** (the brand or product, the promise, the action, one dominant picture), **support** (one concrete feature, offer or proof), **detail** (atmosphere, how it works, depth, the story), **close** (order, visit, contact). Taste's section list for the kind extends this; keep the one-job rule.

Hero rules:

- One composition. A full-bleed photo or dominant visual plane that runs edge to edge (no inset, framed or rounded hero box); only the text column is constrained to the content edges.
- Order of attention: brand, then headline, then body, then action. **Taste wins on wording:** the brand sits in the nav and logo; the h1 names what is sold (`taste-copy`). The rule's point stays: the brand must not disappear.
- No hero cards, stat strips, logo clouds, pill clusters or floating dashboards.
- The headline about 2-3 lines at 1440 and readable at a glance on 390.
- A narrow text column anchored to a calm tonal area of the photo; strong contrast and clear tap targets for any text over a picture.
- **Viewport budget:** the nav counts against the first screen. Nav plus hero content fit in 900 at 1440 and in the first screen at 390; overlay the nav on the hero instead of stacking it.
- Two tests: if the first screen still works with the photo removed, the photo is too weak; if the brand vanishes when you hide the nav, the hierarchy is too weak.

## Apps

Restraint in the Linear manner: calm surface hierarchy, strong type and spacing, few colours, dense but readable information, little chrome, cards only when the card is the interaction.

Organise every app screen around: the main workspace, navigation, secondary context or an inspector, and one accent for action or state.

Avoid: dashboard mosaics of cards, thick borders around every region, decorative fills behind routine UI, competing accents, ornamental icons that don't help scanning. If a panel can become plain layout without losing meaning, remove the card.

## Imagery

Imagery does narrative work.

- At least one strong, real photograph for brands, places, editorial pages and lifestyle products; in-situ photos over abstract fills or fake 3D.
- Choose or crop for a calm tonal area where text sits.
- No photos with signage, logos or lettering fighting the interface.
- Several moments need several photos, never one collage.
- The first screen needs a real visual anchor; texture alone is not enough.

On the canvas these are photo placeholders at their final size and ratio with their note (`pen-format`).

## Copy

- Product language, never design commentary or prompt language on the page.
- The headline carries the meaning; supporting copy is usually one short sentence.
- No repetition between sections; each section explains, proves, deepens or converts.
- If deleting 30% of the copy improves the page, keep deleting.

**Utility copy in app screens:** orientation, status and action over promise and mood. Start with the working surface (figures, charts, filters, tables, status), no hero unless asked. Headings say what the area is or what can be done: "Pesanan hari ini", "Stok menipis", "Terakhir sinkron 09.12". Supporting text explains scope, behaviour, freshness or why it matters, in one sentence. If a line could sit in an ad, rewrite it until it sounds like product UI. Test: can someone scanning only headings, labels and numbers understand the screen?

## Motion

Motion creates presence and hierarchy, not noise: noticeable, smooth on phones, fast, restrained, consistent, and removed if ornamental. **Taste wins on which motions:** one hero entrance, one or two dealt moments, the hover layer, annotated for Koda. No scroll-linked effects, parallax, scroll fade-ins or carousels, even "narrative" ones.

## Hard rules

- No cards by default; no hero cards.
- No boxed or centre-column hero when the brief calls for full bleed.
- One dominant idea per section; no section that needs many small UI devices to explain itself.
- No filler copy.
- No split-screen hero unless the text sits on a calm, unified side.
- Two typefaces and one accent, unless the product already has a strong system.

## Reject these failures

1. A generic card grid as the first impression.
2. A beautiful image with a weak brand.
3. A strong headline with no clear action.
4. Busy imagery behind text.
5. Sections repeating the same mood statement.
6. A carousel with no narrative purpose.
7. An app screen made of stacked cards instead of layout.

## Litmus checks

Brand unmistakable on the first screen? One strong visual anchor? Understood by scanning headings alone? One job per section? Cards actually necessary? Motion improves hierarchy? Still premium with every decorative shadow removed?

Adapted from OpenAI's frontend-skill (Apache-2.0, openai/skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
