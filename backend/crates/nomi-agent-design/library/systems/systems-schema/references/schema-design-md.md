# The DESIGN.md format

How a DESIGN.md is built in VoltAgent's collection (Google Stitch's format with nine extended sections) and in open-design's packages, so you can read any of them quickly and write your own in the same shape.

## What a DESIGN.md is

A plain Markdown file that describes how a product should look, written for agents. It sits beside AGENTS.md: AGENTS.md tells a coding agent how to build the project, DESIGN.md tells a design or coding agent how it should look and feel. Google Stitch introduced the format. VoltAgent's awesome-design-md collects 84 of them, read from public websites. open-design packages the same idea with a compiled token file.

## The frontmatter (Stitch format)

74 of VoltAgent's 84 files open with a YAML block. Its keys, in order:

| Key | Holds |
|---|---|
| `version` | Format version (`alpha`) |
| `name` | System name ("Oura Analysis") |
| `description` | One paragraph: the system's character in a sentence or two |
| `colors` | Named colours by role: `primary`, `on-primary`, `canvas`, `surface`, `ink`, `muted`, `hairline`, `error`, plus brand-named tints (`clay`, `ochre`, `sage`…) |
| `typography` | Named styles (`display`, `display-2`, `heading-1`…, `body`, `ui`, `eyebrow`), each with `fontFamily`, `fontSize`, `fontWeight`, `lineHeight`, `letterSpacing` |
| `rounded` | The radius ladder by name: `none`, `xs`, `sm`, `md`, `lg`, `xl`, `pill`, `full` |
| `spacing` | The spacing ladder by name |
| `components` | Named components (`nav-bar`, `hero`, `button-primary`, `card`…) whose properties reference the tokens above as `{colors.canvas}`, `{typography.ui}`, `{rounded.none}` |

The reference syntax is the useful idea: a component never holds a raw value, only a pointer to a token. That is exactly how your `.pen` components use `$color-*`.

## The nine sections

Each file follows these sections (the frontmatter files group them under Overview, Colors, Typography, Layout, Elevation & Depth, Shapes, Components, Do's and Don'ts):

| # | Section | What it captures |
|---|---|---|
| 1 | Visual Theme & Atmosphere | Mood, density, design philosophy, in a paragraph or two |
| 2 | Color Palette & Roles | Each colour with a semantic name, hex and job |
| 3 | Typography Rules | Families, the full hierarchy as a table (role, size, weight, line height, tracking), principles, font substitutes |
| 4 | Component Stylings | Buttons, cards, inputs, navigation, chips and signature components, with their states |
| 5 | Layout Principles | Spacing scale, grid and container, whitespace philosophy |
| 6 | Depth & Elevation | Shadow system and surface hierarchy |
| 7 | Do's and Don'ts | Guardrails and anti-patterns |
| 8 | Responsive Behavior | Breakpoints, touch targets, how things collapse |
| 9 | Agent Prompt Guide | A quick colour reference and ready-to-use prompts |

The newer files add Shapes (radius ladder, photo geometry), a font-substitute note (licensed faces and their open stand-ins), and illustrative examples.

## How open-design packages it

An open-design package is a folder with three required files:
- `manifest.json`: id (equal to the folder name), name, category, a one-line description, and provenance.
- `DESIGN.md`: the prose, with at least seven substantive H2 sections. Names and order are free; it opens with `# Name` and a `> Category:` line.
- `tokens.css`: one `:root` block declaring every shared token (see the schema in this skill's SKILL.md). Agents paste it and reference values through variables, never re-typing hex.

Richer packages add `USAGE.md` (read order), `components.html` (a fixture proving the tokens alone render the system), derived `design-tokens.json` and `tailwind-v4.css`, previews and source evidence. The derived files must agree with tokens.css; when DESIGN.md and tokens.css disagree, tokens.css is canonical.

## Reading one quickly

1. Read the description or section 1 for character.
2. Read the colour roles and find the three that matter: ground, ink, the one action colour.
3. Read the type hierarchy for the ratio between levels and the display tracking.
4. Read Do's and Don'ts: they hold the rules that make the system recognisable.
5. Skip the agent prompts; you work from tokens, not prompts.

Adapted from VoltAgent/awesome-design-md (@fcc7028, MIT, Copyright (c) 2026 VoltAgent) and open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
