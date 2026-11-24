---
name: huashu-critique
description: "Huashu's original expert critique: a concept score that can veto, five execution dimensions with full scoring tables, weights by deliverable, the ten most common problems with fixes, and the report format. Read it when the person asks whether a design is good, or before you judge your own work."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Huashu critique: concept first, then five dimensions

Huashu's original expert critique: a concept score that can veto, five execution dimensions with full scoring tables, weights by deliverable, the ten most common problems with fixes, and the report format.

**Relation to other skills:** `ux/design-critique` is a version adapted from open-design, itself inspired by this guide; this file is the original with its full scoring tables. Use either; don't run both on one design. `design-loop` §4 scores your own versions to a target in rounds; this critique is the deeper read for when the person asks "bagus nggak?" or for a final judgement. Judge as a critic who didn't build it (`design-loop` §6).

## 0. Concept (the highest weight, scored first)

First ask whether the design has an idea at all, then how well it is done. Execution is an amplifier: amplifying an empty concept makes it emptier.

| Score | Means |
|---|---|
| 9 to 10 | One idea grown from the person's own content; the visual motif can't be swapped |
| 7 to 8 | A clear intent; the motif fits, but a similar subject could almost use it |
| 5 to 6 | Style without concept: pretty, says nothing |
| 3 to 4 | A generic template reskinned; the concept layer is zero |
| 1 to 2 | Not even the style fits; pure decoration |

Questions:
- What does this design say? Can you say its idea in one sentence? If not, there isn't one.
- Cover all the text and logos: can you still tell the subject? If not, the visuals aren't expressing anything. (For type-led design, ask instead: would this type treatment work for another subject?)
- **Swap in another client's or product's name: does it still work? If yes, it's a template; concept is 5 at most.**
- Does the form have a motif that comes from the content (the five form questions in `huashu-workflow`), rather than drawn from a style list?

**Veto:** with concept at 5 or below, the total is capped at 6.0. The other five dimensions are execution; polish can't rescue a design with no idea, it only shines the template.

## 1. Philosophy alignment

| Score | Means |
|---|---|
| 9 to 10 | Embodies the core of the chosen direction; every detail has a reason in it |
| 7 to 8 | Right overall, core traits present, a detail or two off |
| 5 to 6 | Intent visible, but other styles leak in |
| 3 to 4 | Surface imitation, the core not understood |
| 1 to 2 | Unrelated to the chosen direction |

Check: does it use the chosen designer's or style's signature moves? Do colour, type and layout belong to that system? Any self-contradiction (a Kenya Hara direction stuffed with content)?

## 2. Visual hierarchy

| Score | Means |
|---|---|
| 9 to 10 | The eye follows the intended path; zero friction |
| 7 to 8 | Clear primary and secondary; one or two muddy spots |
| 5 to 6 | Title and body distinct, the middle levels confused |
| 3 to 4 | Everything flat, no entry point |
| 1 to 2 | Chaos; nobody knows where to look |

Check: headline at least 2.5 times the body; three or four clear levels through size, weight and colour; whitespace leading the eye; the squint test still shows the order.

## 3. Craft

| Score | Means |
|---|---|
| 9 to 10 | Pixel-exact alignment, spacing and colour |
| 7 to 8 | Refined, one or two tiny alignment or spacing slips |
| 5 to 6 | Mostly aligned, spacing inconsistent, colour unsystematic |
| 3 to 4 | Obvious misalignment, messy spacing, too many colours |
| 1 to 2 | Rough, looks like a sketch |

Check: one spacing system (an 8-point grid); equal gaps for equal things; colours controlled (usually three or four); at most two families; exact edges.

## 4. Functionality

| Score | Means |
|---|---|
| 9 to 10 | Every element serves the goal; zero redundancy |
| 7 to 8 | Clearly goal-led, a little removable decoration |
| 5 to 6 | Usable, but decoration distracts |
| 3 to 4 | Form over function; the user hunts for information |
| 1 to 2 | Drowned in decoration |

Check: would deleting any element make it worse (if not, delete it)? Is the main action or key message in the most visible place? Anything added "because it looks nice"? Does density fit the medium (a slide lighter, a PDF denser)?

## 5. Originality

| Score | Means |
|---|---|
| 9 to 10 | Fresh: a distinct expression inside its framework |
| 7 to 8 | Its own ideas, not a template |
| 5 to 6 | Middle of the road, template-like |
| 3 to 4 | Heavy clichés (a glowing gradient orb for "AI") |
| 1 to 2 | Pure template or stock collage |

Check: clichés avoided (the list below); personal expression while following the direction; at least one "unexpected but right" decision.

## Weights by deliverable

Concept is the first gate everywhere and isn't traded off here.

| Deliverable | Most important | Next | May relax |
|---|---|---|---|
| Social post, cover image | Originality, hierarchy | Philosophy | Functionality (no interaction) |
| Infographic | Functionality, hierarchy | Craft | Originality (accuracy first) |
| Slide deck | Hierarchy, functionality | Craft | Originality (clarity first) |
| PDF, report | Craft, functionality | Hierarchy | Originality (professional first) |
| Landing page, company site | Functionality, hierarchy | Originality | Nothing: all of it |
| App UI | Functionality, craft | Hierarchy | Philosophy (usability first) |
| Promo video | Concept, hierarchy per frame | Craft (timing, easing) | Functionality |
| Social carousel | Originality, hierarchy | Philosophy | Craft (mood first) |

## The ten most common problems

1. **AI-tech cliché:** gradient orbs, digital rain, blue circuit boards, robot faces. Viewers are tired of them. Fix: a metaphor from the subject (conversation itself, not a chat-bubble icon).
2. **Weak size hierarchy:** headline under 2.5 times body. Fix: headline at least 3 times (body 16, headline 48 to 64).
3. **Too many colours:** five or more with no ranking. Fix: one primary, one secondary, one accent, plus greys.
4. **Inconsistent spacing.** Fix: an 8-point system (8, 16, 24, 32, 48, 64).
5. **Not enough whitespace:** every space filled; reading tires. Fix: at least 40% whitespace (60% for minimal styles).
6. **Too many fonts:** three or more. Fix: two at most, variety from weight and size.
7. **Mixed alignment:** left, centre and right together. Fix: one alignment (usually left) throughout.
8. **Decoration over content:** patterns, gradients, shadows stealing the stage. Fix: "if I delete this, is it worse?" If not, delete.
9. **Cyber-neon abuse:** dark navy (#0D1117) with neon glow, the biggest cliché of all (the person's own brand may override). Fix: a palette derived from the subject (`huashu-design-styles`).
10. **Density wrong for the medium:** a slide full of paragraphs, a cover with ten elements. Fix: slide, one key point; cover, one focal point; infographic, layered; PDF, denser but navigable.

## The report

Write it in the person's language. Shape:

> **Penilaian desain: 6,8 / 10** (bagus; konsep di atas 5, jadi tidak dibatasi)
>
> - Konsep 7: "kopi dijemur pelan, halaman juga pelan"; motif terasa, tapi bisa dipakai kopi lain
> - Kesesuaian arah 8: whitespace dan satu foto per layar konsisten
> - Hierarki 6: judul hanya 2× teks isi di layar harga
> - Kerapian 7: jarak kartu menu tidak sama (24 vs 32)
> - Fungsi 6: tombol "Pesan" kalah dari foto di layar pertama
> - Orisinalitas 7: kolom judul tegak segar untuk kedai kopi
>
> **Pertahankan:** foto penjemuran di layar pertama; palet dari kerak kopi.
> **Perbaiki** (urut dari terberat):
> 1. *Tombol utama tenggelam* (penting). Sekarang: tombol 40px abu-abu di bawah foto. Masalah: tindakan utama tidak terlihat. Perbaikan: tombol 48px warna primer, naik ke bawah judul.
> 2. ...
> **Kalau cuma 5 menit:** (1) besarkan judul harga ke 3×; (2) samakan jarak kartu ke 24; (3) pindahkan tombol Pesan.

Severity marks: fatal (blocks the goal), important (clearly hurts), refinement. Give every fix a concrete value ("48px", "jarak 24"), never "make it better". Scores are judgements; say which points you measured.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
