---
name: craft-polish
description: "The polish pass on a design that already exists: six modes (audit, critique, polish, animate, harden, present), a ranked checklist of what to fix first, and the invisible details that make an interface feel right. Read it after a first full version, before presenting."
license: "Apache-2.0, Copyright Anthropic (frontend-design, web-artifacts-builder) and Paul Bakaus (impeccable); MIT, Copyright (c) 2026 Emil Kowalski (design philosophy); MIT, Copyright (c) 2026 MiniMax (frontend-dev); via open-design (Apache-2.0)"
source: "open-design@5fdcc3d skills impeccable-design-polish (after github.com/pbakaus/impeccable), frontend-design (github.com/anthropics/skills), emil-design-eng (github.com/emilkowalski/skills@a47903a); catalogue entries artifacts-builder, web-artifacts-builder, frontend-dev"
---
# The polish pass

What you do to a design that already works: find the few changes with the most effect, make them, and harden the screens against real content, small phones and keyboards, without restarting or changing the person's direction.

Polish starts after a first full version exists (`design-loop`: layout, hierarchy and real copy first; polish waits). It is not a redesign: keep the content, brand and scenario unless the person asks otherwise.

## Six modes

Say which one you are in; the person may ask for just one.

| Mode | What you do | Output |
|---|---|---|
| **Audit** | Find the highest-impact problems in hierarchy, spacing, colour, type, states, phones, accessibility | A ranked list, top five first |
| **Critique** | Say what feels generic, overdesigned, underdesigned or inconsistent, and why | Short paragraphs per screen |
| **Polish** | Fix the top problems directly on the canvas | A new saved version |
| **Animate** | Add motion notes only where they help feedback or the story | Notes per `craft-motion-notes` |
| **Harden** | Long text, empty data, errors, 390 width, contrast, focus, tap targets | Extra state frames and fixes |
| **Present** | Final visual check, then a clear summary and the next decision for the person | The handoff message |

Rules for every mode: look at the rendered canvas, not your memory of the JSON; prefer a few decisive fixes over broad cosmetic churn; finish with a better saved version, not just a list.

## What to fix first

Work down this list; stop when a fix would change the direction.

1. **Hierarchy:** the one most important thing per screen is obvious in a squint. If two things compete, shrink one.
2. **Alignment to edges:** every block on the content edges or a column (`taste-layout`). Misalignments of 4-8px are the most common "feels off" cause.
3. **Spacing rhythm:** related things closer than unrelated things; the same gap for the same relationship everywhere; bottom padding of a section may be one step larger than its top so it does not feel cramped.
4. **Type:** one scale, used consistently (every card title the same size); headings tight, body loose; no widows on headings (rewrap or edit).
5. **Colour:** one accent with a job; greys from one family; contrast at Taste's levels (`taste-check`).
6. **Components:** one button pair, one card, one radius ladder; card rows one height, buttons pinned to card bottoms.
7. **States:** hover, press, focus, disabled, loading, empty, error, success drawn where the flow has them.
8. **Copy:** every heading a noun, buttons a verb and an object, no slop words (`taste-copy`, `craft-microcopy`).
9. **Phone:** the 390 frame redrawn, not squeezed.

## AI tells to remove

- Purple-blue glows or gradients with no product reason.
- The three-icon-card feature row as a default section.
- Oversized rounded cards everywhere, cards inside cards.
- Empty marketing adjectives; invented stats and reviews.
- Inconsistent spacing and type sizes.
- Decorative effects that don't help anyone understand anything.
- Eyebrow plus claim plus two-line lede plus three cards, repeated in every section (`slop-audit` has the full test).

## Invisible details that compound

Users never notice these individually; together they decide whether a design feels finished:
- **Optical alignment:** an icon beside text, a play triangle in a circle, an arrow in a button need 1-2px nudges to look centred. Trust the eye over the maths.
- **Concentric corners:** an inner element's radius is the outer radius minus the padding between them.
- **One light source:** every shadow falls the same way (down), with the same colour family.
- **Tabular figures** for prices and amounts that sit in columns, so digits line up: Rp 18.000 over Rp 249.000.
- **Non-breaking pairs:** a number never breaks from its unit ("5 hari", "Rp 18.000", "10 MB") and a brand name never breaks.
- **Real ellipsis and quotes:** "Memuat…" with the single character, curly quotes.
- **Hit areas:** every control at least 44 tall, 8 from the next target, even when the visible part is smaller (an icon button draws a 44 frame around a 20 icon).
- **Focus rings** visible on every control, in `$color-ring`, 2px outside, drawn in the design system.
- **Paired states:** if a menu opens with motion, it closes with motion; if hover has a state, focus and press have one too.
- **Good defaults over options:** the screen as it first loads should be the best state; most people never change a setting.

## Harden: content that breaks designs

Draw these variants for any list, card or table:
- A very long name ("Toko Kue dan Roti Ibu Sri Rahayu Ningsih Cabang Condongcatur") and a very short one ("Aji").
- Zero items, one item, and many (scroll or pagination).
- A price with seven digits; a missing photo (placeholder tone plus initials, never a broken image icon).
- An error from the server ("Gagal menyimpan. Periksa koneksi lalu coba lagi.").
- At 390: nothing wider than the frame, nothing overlapping, long words wrapped or truncated with an ellipsis and the full text reachable.

## Review it tomorrow

Before presenting a bigger version, look again after another task: imperfections you missed while drawing show up with fresh eyes. Compare it side by side with the benchmark sites Taste names and with the previous version.

## Present

End with: what changed (three bullets at most), what you assumed, and the one decision you need from the person. Do not list every tiny fix.

Notes on sources: open-design's `artifacts-builder`, `web-artifacts-builder` and `frontend-dev` entries are catalogue stubs (a description and an upstream link, no content), so nothing beyond their intent (build polished, multi-part interfaces with real states) is carried here.

Adapted from open-design's impeccable-design-polish (after Impeccable, Apache-2.0, Paul Bakaus), Anthropic's frontend-design (Apache-2.0) and Emil Kowalski's design-engineering skill (MIT, Copyright (c) 2026 Emil Kowalski), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
