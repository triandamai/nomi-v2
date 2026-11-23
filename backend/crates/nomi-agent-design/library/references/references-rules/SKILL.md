---
name: references-rules
description: "How you use reference libraries (beUI, rare-ui, Beautiful UI) and the person's own references: learn ideas, never copy, respect each licence, and turn what you borrow into shadcn components with the person's tokens and copy."
license: "Ideas from beUI (MIT, (c) 2026 Saurabh Chauhan), Beautiful UI (MIT, (c) 2026 Shane Levine) and rare-ui (reference only)"
source: "https://beui.dev, https://www.beautifului.dev, https://rareui.com"
---
# References: Rules of Use

How you use reference libraries (beUI, rare-ui, Beautiful UI) and the person's own references: learn ideas, never copy, respect each licence, and turn what you borrow into shadcn components with the person's tokens and copy.

## The libraries

| Library | Licence | What you may do |
|---|---|---|
| shadcn/ui, shadcn-svelte | MIT | The build kit. Koda installs these. |
| beUI (free tier) | MIT, Copyright (c) 2026 Saurabh Chauhan | Learn patterns and motion intent. |
| beUI Pro | Paid, not ours | Never look at, describe or use. |
| Beautiful UI | MIT, Copyright (c) 2026 Shane Levine | Learn AI-interface patterns. |
| rare-ui | MIT + Commons Clause + Attribution, Copyright (c) 2026 Swami Malode | Learn interaction ideas only. |

Rules:
- These are learning material for you. Your output is a canvas built from shadcn components, never a request to install a component from beUI, rare-ui or Beautiful UI.
- rare-ui forbids redistributing or porting its components, and any shipped part needs a visible link credit. So Koda never uses rare-ui code, and you never describe a rare-ui component closely enough to rebuild it. Take the idea ("confirm a delete in place"), not the component.
- If Koda ever ships code from beUI's free tier or Beautiful UI, the MIT copyright and licence notice must ship with it. Default: Koda does not; it builds the behaviour on shadcn.
- Never copy text, names, sample data, exact sizes, colours, timings or layouts from a reference. Never reproduce a library's brand or demo content.

## Borrowing an idea without copying

1. Name the job it does for this person ("show order status changes at a glance").
2. Rebuild it from shadcn parts (Badge, Item, Sonner, Drawer) and the kit's tokens.
3. Change the composition to fit the screen: different order, density and placement.
4. Use the person's brand tokens, radius and type, never the reference's look.
5. Write the copy in the person's voice, in Indonesian where the audience is ("Pesanan #1042 sudah dikirim").
6. Describe motion as your own one-line intent, not a reference's values.
7. Credit the library by name in `references-patterns` style notes when you explain your choice; never in the person's design.

## The person's own references

People share screenshots, sites they like, competitors, Instagram posts, a menu photo.
- Ask yourself what they like about it: colour mood, density, photo style, layout rhythm, tone. Say it back in one line ("Kamu suka nuansa hangat dan foto produk besar, ya").
- Take the qualities, not the page. Never trace a screenshot or rebuild another business's site, logo, product photos or copy.
- Their own material (logo, photos, prices, menu, brand colours) is theirs: use it directly and exactly, and keep prices and names as given.
- A competitor's site is a reference for what to beat, not a template.
- If a reference conflicts with usability (low contrast, tiny text), keep the mood and fix the problem, and say why in one line.

## Connected reference tools

If the session gives you a reference tool (a web reader, an image search, a design inspiration source), use it to understand patterns and current conventions, then follow the same rules: describe, recombine, re-token, re-write. Do not import assets from it into the design unless the tool says they are licensed for that use and the person agrees. If a tool is unavailable, work from what the person shared; do not guess what a site looks like.

Sources: beUI LICENSE (MIT, Copyright (c) 2026 Saurabh Chauhan), rare-ui LICENSE (MIT + Commons Clause + Attribution, Copyright (c) 2026 Swami Malode), Beautiful UI licence page (MIT, Copyright (c) 2026 Shane Levine), shadcn/ui (MIT, Copyright (c) 2023 shadcn), shadcn-svelte (MIT, Copyright (c) 2023 Hunter Johnston and CokaKoala).
