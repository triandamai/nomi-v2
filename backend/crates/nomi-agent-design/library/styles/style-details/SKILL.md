---
name: style-details
description: "Small structural details that make a style feel made rather than generated — 01/02 number markers, container lines with corner squares, L-brackets, chamfered corners, layered shadow recipes and one icon family — with the rules that stop them turning into decoration."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style Details

Small structural details that make a style feel made rather than generated — 01/02 number markers, container lines with corner squares, L-brackets, chamfered corners, layered shadow recipes and one icon family — with the rules that stop them turning into decoration.

Use a detail only when the chosen style calls for it, and the same way everywhere on the page. One or two details per page is plenty.

## Number markers (01, 02, 03)

- Secondary structure for real sequences: process steps, chapters, numbered services. Not for features that have no order.
- Always two digits. One style per section.
- Place them predictably: card corner, section gutter, timeline rail, or left of a heading on the same baseline.
- Mono or narrow caps, 11–13px, `$color-muted-foreground`. They must never compete with the heading or the button.

## Container lines and corner squares

- Two 1px vertical rectangles at the left and right edges of the main content container (e.g. max 1120, side padding 20–48), running the full page height behind content.
- Colour: foreground at 14% (#18181B24 on light, #FFFFFF24 on dark).
- Corner squares 4–8px, foreground at 28%, centred on real container or section corners, never at random spots.
- Same container width for every section; the content edge must line up exactly with the line. Page level only, not on nested components.
- Phone: keep at least 16px between the line and text.

## L-shaped corner brackets

- At each corner of a frame: two rectangles, e.g. 18×1 and 1×18, in a stronger line colour (foreground at 34%).
- Build one reusable bracket-corner component and rotate/place four copies; one size on the whole page.
- Brackets replace shadows: a framed section has a 1px border and brackets, no drop shadow.

## Chamfered (diagonal-cut) corners

- Draw the shape as a `path` (an octagon or a rectangle with two opposite cuts), since cornerRadius only rounds.
- Cut sizes: 6–10px on buttons, 12–18px on cards, 20–32px on large panels; one cut scale per component type.
- Mirror pairs: top-left + bottom-right, or top-right + bottom-left.
- The border follows the same cut (stroke on the path). Keep text padding after the cut and button height ≥ 44.
- Do not mix chamfers and round pills at the same level of hierarchy. Fits technical styles (`style-tech-green-dark`, `style-framed-grid`), rarely others.

## Shadow recipes (neutral, layered)

Use one strength per component state. Pair with a clean fill and the style's radius. Never tint these strongly; never stack two recipes on one node.

| Recipe | Use for | Effects |
|---|---|---|
| Small | Inputs, pills, compact cards | Stroke 1 inside #191C2114 (replaces the 1px ring); drop y 2 blur 3 spread −1 #0000001A; drop y 1 blur 0 #191C2105 |
| Medium | Default cards, popovers, panels | Stroke 1 inside #0000000F; drops y 1 blur 1 #0000000F, y 3 blur 3 spread −1.5 #0000000F, y 6 blur 6 spread −3 #0000000F, y 12 blur 12 spread −6 #0000000F, y 24 blur 24 spread −12 #0000000F |
| Large | Hero media, one feature callout, dialogs | Drops y 3 blur 2 #00000009, y 7 blur 5 #0000000C, y 13 blur 10 #0000000F, y 22 blur 18 #00000012, y 42 blur 33 #00000016, y 100 blur 80 #0000001F |

If a node cannot take six effects, keep the first, middle and last layer of the recipe. Do not put Large on dense lists or small controls. In very low-contrast layouts, a clear border beats a shadow.

## Icons

- One icon family per product, one stroke width (1.5 or 1.75) and one size per role (16 inline, 20 in buttons, 24 in nav). Icons are drawn as `path` nodes.
- Pair icons with labels for anything unfamiliar; icon-only is for universal actions (close, search, back).
- Put icons in flow beside text. Do not wrap every feature icon in a rounded-square tile.
- A duotone look (a primary glyph plus a 30–40% secondary fill) is fine if the whole set uses it; never mix duotone with plain outline icons in one control set.
- The icon colour follows the text it sits with; accent-coloured icons only where the accent means something (active, status).

Adapted from mengto/skills: agent-skills/web-design/number-details, agent-skills/web-design/container-lines, agent-skills/web-design/framed-grid-layout, agent-skills/web-design/corner-diagonals, agent-skills/web-design/beautiful-shadows, agent-skills/web-design/solar-duotone-bold (MIT, Copyright (c) 2026 Meng To).
