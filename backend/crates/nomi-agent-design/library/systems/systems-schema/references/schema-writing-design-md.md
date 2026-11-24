# Writing a DESIGN.md for Koda

How you write a DESIGN.md once the person approves a design, so Koda can build it in SvelteKit, Tailwind v4 and shadcn-svelte without guessing: the frontmatter, the nine sections, and the rules that keep it true to the canvas.

## When and why

Write it after the person approves a direction and the design system frame is final, alongside the handoff (`components-handoff`). The canvas shows what to build. The DESIGN.md says why and what must never change, so the next page Koda adds still looks like the first. It describes the person's design only: no source system, no brand name other than the person's own.

## The frontmatter

Open with the Stitch-format block, with values copied from your `.pen` variables, never re-invented:

| Key | What you write |
|---|---|
| `version` | `alpha` |
| `name` | The person's business name ("Kedai Kopi Senja") |
| `description` | One or two sentences of character, as on plan line 2 ("Warm flour-white paper, espresso ink and one terracotta, set in Fraunces and Figtree; calm, local, made by hand.") |
| `colors` | Every `$color-*` token by its shadcn name (`background`, `foreground`, `card`, `primary`, `primary-foreground`, `muted`, `muted-foreground`, `border`, `input`, `ring`, `destructive`, and any custom pair such as `brand`, `success`, `warning`), light values; then a `dark` group if the design has one |
| `typography` | Each text style from the foundations frame (`display`, `h1`, `h2`, `h3`, `lead`, `body`, `small`, `label`) with family, size, weight, line height and tracking, at 1440. Phone sizes go in section 8 |
| `rounded` | `sm`, `md`, `lg`, `xl`, `2xl`, `pill` as on the canvas |
| `spacing` | `space-1`…`space-12` and the section padding at 1440, 768 and 390 |
| `components` | One entry per reusable frame family (`button-default`, `button-outline`, `input`, `card`, `dialog`…) whose properties point at tokens (`{colors.primary}`, `{rounded.lg}`, `{typography.label}`) |

Every value must match the canvas exactly. If you change a token on the canvas, change it here in the same save.

## The nine sections

1. **Visual theme and atmosphere.** Two short paragraphs: the mood, the density, what carries the colour (photos, product, one band), and the one idea the design is built on.
2. **Colour palette and roles.** A table: token, light hex, dark hex, job, and the contrast it holds ("primary #a3472b, label #ffffff 5.9:1, 4.8:1 on background"). Name the accent's limits ("primary button, links, focus ring and one band; nowhere else").
3. **Typography rules.** Families as real Google Fonts with the weights loaded (at most four files), then the hierarchy table at 1440 and 390, tracking on headings, sentence case, line length 62 characters.
4. **Component stylings.** For each component used: its shadcn-svelte name, variants, sizes, the 44 or 48 control height, radius token, and every state (Default, Hover, Focus, Active, Disabled, Loading, Error) with what changes. Point at the frame names (`Button / Default / Hover`).
5. **Layout principles.** Container width, grid columns at each breakpoint, side padding, section padding, the ground order down the page, and any composition the hero uses (from `taste-layout`).
6. **Depth and elevation.** The four levels and which components use each; borders vs shadows; the scrim for overlays.
7. **Do's and don'ts.** Six to ten specific rules that keep the design recognisable ("cards never pure white on the cream page", "no second accent", "photos keep a 4:5 ratio").
8. **Responsive behaviour.** Breakpoints (390, 768, 1440), what stacks and in what order, phone type sizes, tap targets of 44 and more, what the navigation becomes.
9. **Build notes for Koda.** Tailwind v4 mapping (each `$color-*` becomes `--<name>` in the `@theme` block; radius and spacing likewise), the shadcn-svelte components to install, custom tokens that aren't shadcn defaults, the motion note, the fonts to load, and what the canvas couldn't show (an intended gradient, a photo crop, an animation).

## Rules

- Write in plain English sentences, short and specific. Indonesian copy examples stay as they are on the canvas.
- Values only from the canvas. A value that isn't on the canvas is a gap: add it there first.
- No source names. Never "inspired by" a company, never a system id from this library, never a licensed face's name as the font to load.
- One file per design, at the project root, beside AGENTS.md. Keep it under about 12 KB; link to the canvas frames instead of repeating them.
- When the person changes the design later, update the DESIGN.md in the same turn.

Adapted from VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
