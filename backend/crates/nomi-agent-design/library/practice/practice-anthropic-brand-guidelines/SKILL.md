---
name: practice-anthropic-brand-guidelines
description: "How to apply a client's existing brand consistently: a short brand sheet, colours mapped to tokens, heading and body faces with fallbacks, text colour by background, accents on shapes. Read when the person already has a brand or brand book."
license: "Apache-2.0 (anthropics/skills, skills/brand-guidelines/LICENSE.txt); via open-design (Apache-2.0)"
source: "https://github.com/anthropics/skills@e4bb5d9 (skills/brand-guidelines); open-design@5fdcc3d"
---
# Applying a client's brand

When the person already has a brand (a logo, colours, a font, maybe a brand book from a designer), capture it in a short brand sheet and apply it the same way on every screen.

The source skill is Anthropic's own brand sheet and how it gets applied to documents. What you keep is the shape of a good brand sheet and the application rules; the client's brand replaces the values. Never use Anthropic's colours or fonts as a client brand.

## 1. Collect the brand sheet

Ask for or find (logo files, signage, packaging, Instagram, an old site, a brand PDF):

| Part | What to record | Example (a batik shop) |
|---|---|---|
| Main colours | a dark for text and dark grounds, a light for grounds and text on dark, one or two greys | Indigo tua #1F2A44 · Kain #F5F0E6 · Abu #B5AEA3, Abu muda #E7E2D8 |
| Accent colours | one primary accent, at most two more, with their order | Soga #A0522D utama · Hijau daun #5E7A4C kedua |
| Heading face | family, weights, the fallback | a serif the brand uses, falling back to a common serif |
| Body face | family, the fallback | a readable sans or serif |
| Logo | versions (full, mark only, one colour), clear space, smallest size, which grounds it may sit on | full logo on Kain, white mark on Indigo |
| Don'ts | what the brand forbids | no stretching the logo, no batik pattern behind text |

Write it as the first part of the design system frame and in the handoff (`practice-design-md`). Mark anything you inferred from a photo instead of a file.

## 2. Map it to tokens

- The dark becomes `$color-foreground` (and the dark ground if the trade allows dark sections); the light becomes `$color-background`; greys become `$color-muted`, `$color-muted-foreground`, `$color-border`.
- The primary accent becomes `$color-primary` with a label colour that passes 4.5:1; second and third accents become extra token pairs only if the screens need them.
- Fonts become `$font-heading` and `$font-body`. Name the brand's real font even if the Studio shows a stand-in; Koda loads it. Note the fallback.

## 3. Apply it

- **Headings in the heading face** from a size threshold up (the source uses 24 pt and larger); everything smaller is body face, so labels and buttons stay readable.
- **Text colour follows the background:** dark text on light grounds, light text on dark, chosen by contrast, never by habit.
- **Accents go on shapes,** not long text: buttons, small markers, icons, chart series, a divider. Where several accents exist, use them in the brand's order and keep the primary for the main action.
- **Keep the hierarchy** the brand implies; don't invent a new heading scale per page.
- **Logo:** use the supplied file as is, on an allowed ground, with its clear space; never redraw, recolour or stretch it (`practice-own-assets`).

## Where Taste overrides a brand sheet

- The brand's colours still sit inside a 90-95% neutral interface; cycling three accents across shapes (as the source does on slides) becomes one accent for actions plus colour from photographs.
- A brand colour that fails contrast as a button fill keeps its role but gets a darker step for small text and labels; say so in the handoff.
- If the brand's own colour is violet or lavender, it is the client's real colour and replaces the palette (Taste: the client's own colours replace the palette), but keep it to the logo and the main action, not grounds.
- A brand font that isn't licensed for the web gets the nearest licensed face, noted for the person.

Adapted from Anthropic's brand-guidelines skill (Apache-2.0, anthropics/skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
