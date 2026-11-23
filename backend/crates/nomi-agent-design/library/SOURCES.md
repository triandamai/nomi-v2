# Where Sora's design library comes from

Sora's library is adapted from the open projects below and rewritten for drawing on the
OpenPencil canvas. It isn't served to Sora (`read_design_skill` reads only the skills: `<area>/<skill>/SKILL.md` and its `references/`); it
records each source, its licence, and how we may use it. Keep it up to date when a skill is
added or re-synced.

| Folder / topic | Source | Version read | Licence | How we use it |
|---|---|---|---|---|
| `design/pen-format` | `@open-pencil/pen` 0.16 (what the Design Studio parses) and Nomi's own `design_artifact.rs` | 0.16.0 | Written for Nomi | The canvas format |
| `taste/` | Taste Skill, [github.com/Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill) (`design-taste-frontend`) | 2.5.1 (7 Nov 2026) | MIT, Copyright (c) 2026 Leonxlnx | Adapted: Sora's design rulebook |
| `styles/` | [github.com/mengto/skills](https://github.com/mengto/skills): the UI, workflow and web-design skills named in each file | main, 22 Nov 2026 | MIT, Copyright (c) 2026 Meng To | Adapted: anti-slop audit, design loop, styles to offer as directions |
| `components/` | shadcn/ui's agent skill (`skills/shadcn`), [github.com/shadcn-ui/ui](https://github.com/shadcn-ui/ui) | main, Nov 2026 | MIT, Copyright (c) 2023 shadcn | Adapted: token roles, design-system page, state matrix |
| `components/` | shadcn-svelte docs, [shadcn-svelte.com](https://www.shadcn-svelte.com/) | Nov 2026 | MIT, Copyright (c) 2023 Hunter Johnston, Copyright (c) 2023 CokaKoala | Component names, parts and variants Koda builds with |
| `references/` | beUI, [beui.dev](https://beui.dev) (free library, source github.com/starc007/ui-components) | Nov 2026 | MIT, Copyright (c) 2026 Saurabh Chauhan. beUI Pro is a separate paid licence: never used | Reference and learning only: patterns described in our own words |
| `references/` | Beautiful UI, [beautifului.dev](https://www.beautifului.dev/) | Nov 2026 | MIT, Copyright (c) 2026 Shane Levine | Reference and learning only |
| `references/` | rare-ui, [github.com/swamimalode07/rare-ui](https://github.com/swamimalode07/rare-ui) / rareui.com | Nov 2026 | MIT + Commons Clause License Condition v1.0 + Attribution, Copyright (c) 2026 Swami Malode | **reference only**: ideas described in our own words; no code, text or values copied or ported, and Koda never ships its components (it would need a visible rareui.com credit in every app, and the components may not be redistributed or ported) |

## The MIT notice

The MIT-licensed sources above (Taste Skill, mengto/skills, shadcn/ui, shadcn-svelte, beUI's
free library, Beautiful UI) are each used under this licence, with their copyright lines as
listed in the table:

> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the "Software"), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or
> substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
> BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
> NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
> DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
