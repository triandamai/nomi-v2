---
name: practice-anthropic-theme-factory
description: "Ten named colour themes with moods and trades as a menu of starting points, how to map one onto shadcn tokens, show options and wait for a pick, and build a custom theme in the same shape. Read when the person wants to choose a look from options."
license: "Apache-2.0 (anthropics/skills, skills/theme-factory/LICENSE.txt); via open-design (Apache-2.0)"
source: "https://github.com/anthropics/skills@e4bb5d9 (skills/theme-factory, SKILL.md and themes/); open-design@5fdcc3d"
---
# Theme factory: a menu of colour themes

Offer a few named themes, let the person pick, then apply the chosen one consistently through the tokens, or build a custom theme in the same shape.

**Taste wins on choice.** Taste deals the palette by trade and kind and puts the client's own colours first (`taste-color`). Use these themes as a menu of named starting points when the person wants to compare looks, and adjust their values to Taste's rules: a 90-95% neutral interface, one accent, contrast checked, never violet or lavender.

## The flow

1. **Show two to four options,** not all ten: the ones that fit the trade. On the canvas, a small frame per theme: four swatches with names and hex, a heading and a line of body copy in the pairing, and a primary button.
2. **Ask which one,** and wait for an explicit choice.
3. **Apply it:** map its colours to roles (below), keep it everywhere, check contrast.
4. If none fits, **make a custom theme** in the same shape: a name that describes the feeling, four colours with names, hex and roles, a type pairing, and what it's good for. Show it for review before using it.

## The ten themes

| Theme | Colours (name, hex, job) | Feel | Suits |
|---|---|---|---|
| Ocean Depths | Deep Navy #1a2332 base · Teal #2d8b8b accent · Seafoam #a8dadc light accent · Cream #f1faee text/light ground | calm, professional | finance, consulting, clinics |
| Sunset Boulevard | Burnt Orange #e76f51 accent · Coral #f4a261 second accent · Warm Sand #e9c46a highlight · Deep Teal-Slate #264653 dark text | warm, energetic | events, lifestyle, creative services |
| Forest Canopy | Forest Green #2d4a2b dark · Sage #7d8471 muted accent · Olive #a4ac86 light accent · Ivory #faf9f6 ground | natural, grounded | outdoor, wellness, organic products |
| Modern Minimalist | Charcoal #36454f dark · Slate Gray #708090 accent · Light Gray #d3d3d3 dividers · White #ffffff ground | clean, versatile | architecture, studios, business |
| Golden Hour | Mustard #f4a900 accent · Terracotta #c1666b second · Warm Beige #d4b896 ground · Chocolate #4a403a text | warm, inviting | restaurants, hospitality, artisan goods |
| Arctic Frost | Ice Blue #d4e4f7 light · Steel Blue #4a6fa5 accent · Silver #c0c0c0 metal · Crisp White #fafafa ground | cool, precise | healthcare, clean tech |
| Desert Rose | Dusty Rose #d4a5a5 soft primary · Clay #b87d6d accent · Sand #e8d5c4 ground · Deep Burgundy #5d2e46 dark | soft, refined | fashion, beauty, weddings, boutiques |
| Tech Innovation | Electric Blue #0066ff accent · Neon Cyan #00ffff highlight · Dark Gray #1e1e1e ground · White #ffffff text | bold, high contrast | software launches only |
| Botanical Garden | Fern Green #4a7c59 · Marigold #f9a620 accent · Terracotta #b7472a warm · Cream #f5f3ed ground | fresh, lively | garden shops, food, farm-to-table |
| Midnight Galaxy | Deep Purple #2b1e3e · Cosmic Blue #4a4e8f · Lavender #a490c2 · Silver-lilac #e6e6fa | dramatic | not for you as given (below) |

The source pairs each theme with system fonts for slides (DejaVu, FreeSans, FreeSerif). On the web, replace them with the kind's pairing in `taste-type`; keep only the intent (serif headings for Sunset, Botanical and Forest; sans for the rest).

## Where Taste changes a theme

- **Midnight Galaxy** is built on violet and lavender: Taste bans both. Don't offer it; for a dark, dramatic gaming or nightlife look, use a near-black with one non-violet accent.
- **Tech Innovation** has a dark ground and neon cyan: only on software, gaming or tech pages (dark pages are never allowed on a place, product or service), and the cyan stays a rare highlight, never body text (it fails contrast on white).
- **Sunset Boulevard, Golden Hour, Botanical Garden** each carry two or three saturated colours: keep one as `$color-primary`, push the others into photographs, small details or nothing. The interface stays neutral.
- **Cream-and-terracotta looks** (Golden Hour, Botanical) are one of the predictable AI looks (`practice-design-consultation`): use them only when the business's real material points there.
- **Mustard, marigold, coral and seafoam fail as text** on light grounds: never use them for labels; a button in them needs a dark label, checked at 4.5:1.

## Mapping a theme to tokens

| Theme part | Token |
|---|---|
| the light ground colour | `$color-background` (and one step off it for `$color-card` or `$color-muted`) |
| the dark text colour | `$color-foreground` |
| the accent | `$color-primary`, with `$color-primary-foreground` chosen for contrast; `$color-ring` near it |
| the muted accent | `$color-muted-foreground` if it passes 4.5:1, otherwise a darker step |
| the light accent | `$color-accent` (hover wash, selected row) |
| dividers | `$color-border`, `$color-input` |

Then build the dark theme as its own set (`practice-color-expert`), and check every pair.

## Example of a custom theme

> **Kopi Pagi** — hangat dan tenang, untuk kedai kopi.
> Krem susu #F6F1E9 latar · Cokelat espresso #2E2119 teks · Hijau kemasan #2F5D3A aksi utama · Abu hangat #8A7F74 teks pendukung.
> Judul serif, teks sans. Cocok untuk kedai, roastery, toko roti.

Adapted from Anthropic's theme-factory skill (Apache-2.0, anthropics/skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
