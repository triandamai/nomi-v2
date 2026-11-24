---
name: design-critique
description: "A five-dimension 0-10 critique with score bands and cited evidence, ending in Keep, Fix and Quick wins. Read when the person asks what is wrong with a preview, when comparing two variants, or when reviewing Koda's build after review_build."
license: "Apache-2.0, Copyright 2026 Open Design contributors; method inspired by alchaincyf/huashu-design, MIT, Copyright (c) 2026 alchaincyf"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (design-templates/critique/SKILL.md); upstream https://github.com/alchaincyf/huashu-design"
---
# Design critique

`design-loop` gives the 0–2 rubric and gates you pass before every save. This is the deeper review: five dimensions scored 0–10, each backed by evidence on the canvas, then three action lists. Use it:
- when the person asks "apa yang kurang?" or "kenapa kurang bagus?" about a saved version;
- before a change round, to pick what to fix first;
- to compare two variants under the same conditions;
- after Koda builds, together with `review_build`.

## The five dimensions

Score each on its own; never let one strong dimension lift another.

| Band | Means |
|---|---|
| 0–4 Broken | Fails its job here |
| 5–6 Functional | Works, rough or generic |
| 7–8 Strong | Clear and polished; an expert finds 2–3 misses |
| 9–10 Exceptional | Nothing to fix; something to study |

**1. Direction holds.** Does every decision follow the one frozen direction (`reference-contract`)?
Evidence: one palette, one pairing, one radius ladder, one button look, one voice across all screens; or a frame where a second style creeps in. 0–4 three styles at war; 5–6 one direction, half the parts drift; 7–8 coherent, drift at the edges (the 404, the empty state); 9–10 every part argues for the same idea.

**2. Hierarchy.** Can a stranger tell what to read first, second, third, and what to tap?
Evidence: the largest text is the most important thing on each frame; one filled primary per view; muted text really steps down. 0–4 everything shouts; 5–6 works on the hero, breaks on inner screens; 7–8 clear tiers, one collision; 9–10 the eye moves with no friction.

**3. Detail.** Alignment, spacing, line breaks, photo frames, state frames.
Evidence: blocks on the content edges, gaps from the spacing scale, no one-word last lines, the same price size everywhere, token values instead of raw hex. 0–4 visible tape and string; 5–6 most frames clean, one or two ragged; 7–8 polished, 2–3 misses; 9–10 nothing a trained eye would move.

**4. Function.** Does it do its job for this business's customers?
Evidence: the main action reachable in the first screen at 390; targets 44 tall; every state from `ux-states` drawn; contrast passing (`ux-accessibility`); real copy and prices; the flow ends on a real confirmation. 0–4 looks fine, can't be used; 5–6 the main path works, edges broken; 7–8 sturdy in normal use; 9–10 handles empty, error, long names and the phone without flinching.

**5. Distinctness.** Is there one thing that makes this business memorable, and does it serve them?
Evidence: a moment or a real photo idea only this client has, versus a page that could be any shop's. Under Taste this means one earned moment and the client's own material, never tilt, effects on a place, or novelty for its own sake. 0–4 the generic AI median; 5–6 competent, forgettable (fine for an admin app); 7–8 one memorable move, the rest solid; 9–10 several moves worth stealing, each serving the client.

## Scoring discipline

- **Cite evidence for every score:** the frame name, element id or token. "Hierarchy 5: on `Menu — Phone` the price (20/500) is louder than the drink name (16/400); the 'Pesan' button and 'Lihat lokasi' are both filled" beats "feels unclear".
- **Score the worst sustained band:** if one screen breaks hierarchy, two good ones don't lift it to 7.
- **Don't inflate:** 7 means strong, not acceptable. A mean above 8 is suspicious: look again.
- **Distinctness may be low** on a back-office screen; don't punish the right restraint.
- **All five, every time.**
- **Judge the render, not your intent.** You made it: switch roles first (`design-loop` §6) and read it as someone who knows only the brief and the contract.

## Keep, Fix, Quick wins

After the scores, three lists:
- **Keep** (3–5): what works and must survive the next round, by element ("Keep the full-bleed photo in `hero` and its 'Pesan lewat WhatsApp' button").
- **Fix** (3–6): must-dos, ordered by visible gain per minute; one sentence each, naming the element.
- **Quick wins** (3–5): small edits with outsized effect ("muted text `#7a6d63` → `#70645a` so it passes 4.5:1 on the muted band").

## How you present it

In chat, not on the canvas: a five-row table (Dimension, Score, Evidence), then the three lists, then one line on what you'll do next. Write it in the person's language. When the person has not asked, give a critique only as a short "what I'd fix next" after they have seen the version.

## Reviewing Koda's build

1. Run `review_build` first and report what it found: which colours, fonts and element ids from the handoff are in the project and which are missing.
2. It can't see spacing, sizes, layout or feel. For those, ask the person for screenshots of the running app at 390 and 1440 (or look at ones they sent).
3. Score the build on the same five dimensions against the approved version, not against what you'd do now.
4. Each Fix names the element id, what the approved design has and what the build shows: "`checkout-pay-button`: approved 48 tall full width, build 36 tall, label cut".
5. Never say the build matches the design without the `review_build` check, and say what neither check could see.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): design-templates/critique/SKILL.md, itself inspired by alchaincyf/huashu-design (MIT, © 2026 alchaincyf). Changed: rewritten for the OpenPencil canvas.
