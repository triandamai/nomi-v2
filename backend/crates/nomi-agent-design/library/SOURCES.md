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

### Added with library v2 (Nov 2026)

| Folder / skills | Source | Version read | Licence | How we use it |
|---|---|---|---|---|
| `ux/`, `practice/` (open-design's own: design-brief, research-decision-room, faq-page, own-assets, login-flow), `templates/`, parts of `craft/`, `decks/decks-templates`, `video/video-frame-templates`, `images/` | open-design, [github.com/nexu-io/open-design](https://github.com/nexu-io/open-design) | `5fdcc3d` (23 Nov 2026) | Apache-2.0, Copyright 2026 Open Design contributors (no NOTICE file) | Adapted; each file says what it came from. Changed: rewritten for the OpenPencil canvas |
| `decks/decks-templates` (one source among several) | guizang-ppt, **only the copy bundled in open-design** at `5fdcc3d` | MIT snapshot | MIT, Copyright (c) 2026 op7418 | Upstream github.com/op7418/guizang-ppt-skill switched to **AGPL-3.0 on 28 May 2026**: never read, copied or synced from it |
| `design/huashu-*`, `ux/huashu-critique`, `decks/huashu-slide-decks`, `video/huashu-*`, `images/huashu-image-briefs` | [alchaincyf/huashu-design](https://github.com/alchaincyf/huashu-design) | `5906569` (23 Nov 2026, after its 14 May 2026 switch to MIT) | MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生) | Translated to English and adapted; its scripts are described, not copied |
| `ux/design-critique` (inspired by) | huashu-design via open-design | as above | MIT | Adapted |
| `decks/decks-html-ppt`, `decks/decks-storytelling`, `decks/decks-runtime-notes` | [lewislulu/html-ppt-skill](https://github.com/lewislulu/html-ppt-skill) | `fd16290` | MIT, Copyright (c) 2026 lewis | Adapted; the deck runtime will be served as an asset with this notice |
| Deck runtime (`backend/crates/nomi-agent-core/assets/deck-runtime/html-ppt/`: runtime.js, base.css, animations.css, 36 themes) | [lewislulu/html-ppt-skill](https://github.com/lewislulu/html-ppt-skill) `fd16290` | `fd16290` | MIT, Copyright (c) 2026 lewis; its LICENSE is served and published with every deck | Copied unchanged, plus Nomi's own `nomi-bridge.css`; served beside each preview under `_runtime/html-ppt/` |
| `video/video-hyperframes*` | [heygen-com/hyperframes](https://github.com/heygen-com/hyperframes) | `574b5e3` (23 Nov 2026) | Apache-2.0, Copyright 2026 HeyGen, Inc. (no NOTICE file) | Adapted; community skills not ported |
| `systems/` | open-design `design-systems/` at `5fdcc3d`; [VoltAgent/awesome-design-md](https://github.com/VoltAgent/awesome-design-md); [bergside/awesome-design-skills](https://github.com/bergside/awesome-design-skills); tw93/kami; Tom-Opencart/tom-modern-html-style-rule; cloudflare/kumo | Nov 2026 | Apache-2.0 (open-design, incl. its 15 own brand packages); MIT (c) 2026 VoltAgent; MIT (c) 2026 Bergside; MIT (c) 2026 Tw93; MIT (c) 2026 Tom-Opencart; MIT (c) 2026 Cloudflare, Inc. | Brand systems are **learn, don't copy**: never another business's identity. Colours fixed for contrast; violet swapped per Taste |
| `ux/starter-systems` | open-design fixtures and bergside tokens | as above | Apache-2.0; MIT (c) 2026 Bergside | Contrast-checked starter sets |
| `craft/craft-style-*`, `craft-taste-variants`, `craft-redesign`, `craft-screenshot-to-design`, `craft-brand-mark` | Leonxlnx skills (minimalist, soft, brutalist, taste v1, gpt-taste, output, stitch, image-to-code, brandkit, redesign) via open-design | Nov 2026 | MIT, Copyright (c) 2026 Leonxlnx | Adapted; Taste wins where they differ |
| `craft/craft-polish`, `craft-interface-guidelines`, `craft-microcopy`, `craft-motion-*`, `craft-visual-scenes`, `craft-charts`, `craft-diagrams` | impeccable (Apache-2.0, Paul Bakaus); anthropics/skills frontend-design, algorithmic-art (Apache-2.0); emilkowalski/skills (MIT (c) 2026 Emil Kowalski; open-design's copies wrongly name another holder); ui-animation (MIT (c) Matthew Blode); gsap-skills (MIT (c) GreenSock); vercel-labs web-interface-guidelines (MIT (c) 2025) and writing-guidelines (MIT (c) 2026); oil-motion (MIT (c) Lin Zhihuang); Motion.Lab (MIT per its README, yangxu128 and contributors); html-anything (Apache-2.0, open-design); snow-d3 (MIT (c) Jiannanya); hand-drawn-diagrams (MIT (c) 2025 Muthukumaran N.); MiniMax, CloudAI-X (MIT); Flutter docs (BSD-3) | Nov 2026 | as listed | Design judgement only; no code |
| `practice/` | coreyhaines31/marketingskills `5b2b400` (MIT (c) 2025 Corey Haines); ComposioHQ/awesome-claude-skills (Apache-2.0 per its README; no LICENSE file); garrytan/gstack `a3ecfd5` (MIT (c) 2026 Garry Tan); smixs/creative-director-skill `ac06eb2` (CC-BY-4.0 (c) 2026 Serge Shima); meodai/skill.color-expert `5bf7c77` (CC-BY-4.0, its own SKILL.md only); anthropics/skills `e4bb5d9` (Apache-2.0: frontend-design, canvas-design, theme-factory, brand-guidelines); openai/skills `82d2c5b` (Apache-2.0); nextlevelbuilder ui-ux-pro-max `f2f7266` (MIT (c) 2024 Next Level Builder); ibelick/ui-skills `7ef0076` (MIT (c) 2026 Julien Thibeaut); wholiver swiftui `2c82638` (MIT (c) 2026); raintree apple-hig-skills `a0f1e4e` (MIT (c) 2025 Raintree; skill files only, Apple's text not used); ehmo/platform-design-skills `dc2be82` (MIT (c) 2026); google-labs-code/stitch-skills `62fdcd6` (Apache-2.0) | Nov 2026 | as listed | Adapted; CC-BY-4.0 items credited in their files |
| `decks/decks-templates`, `templates/*` (one source among several) | kami (MIT), anthropics/financial-services (Apache-2.0), Zara Zhang (MIT), marketingskills (MIT), last30days (MIT), aster-atelier (MIT), 8-bit Orbit (MIT); YouMind prompt collections (CC-BY-4.0) studied, not copied | Nov 2026 | as listed | Adapted |

**Not ported, and why:** multica (Apache-2.0 plus its own conditions; a local-daemon design for agent CLIs, nothing for a hosted service); WordPress/agent-skills (GPL-2.0-or-later); anthropics doc-coauthoring and docx/pdf/pptx, openai slides/doc/imagegen/sora/speech, NanoBanana-PPT (no licence file: described from open-design's stubs in our own words only); Shpigford/screenshots (repository gone); hyperframes community skills (community-maintained); open-design's webgpu effects. Remotion (named in `images/images-media-tools`) may need a company licence for business use.

## The MIT notice

The MIT-licensed sources above (Taste Skill, mengto/skills, shadcn/ui, shadcn-svelte, beUI's
free library, Beautiful UI, and every MIT source in the v2 table) are each used under this licence, with their copyright lines as
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

## The Apache License 2.0

open-design, hyperframes, anthropics/skills, openai/skills, impeccable, html-anything,
ComposioHQ/awesome-claude-skills and stitch-skills are used under the Apache License, Version 2.0
(https://www.apache.org/licenses/LICENSE-2.0), with their copyright lines as listed above. Every
file adapted from them was changed: rewritten for Nomi's canvas, decks and video, and each says
so in its last line.

## Creative Commons Attribution 4.0

creative-director-skill (Serge Shima) and skill.color-expert (meodai) are adapted under CC BY 4.0
(https://creativecommons.org/licenses/by/4.0/); changes: rewritten for Nomi's canvas.
