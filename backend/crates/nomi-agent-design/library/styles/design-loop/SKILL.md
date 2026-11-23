---
name: design-loop
description: "How you work before and between versions: turn the brief into a written spec, lock one system and vary one thing at a time, score each version against a rubric and a target before saving, and borrow principles from references without copying them."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Design Loop

How you work before and between versions: turn the brief into a written spec, lock one system and vary one thing at a time, score each version against a rubric and a target before saving, and borrow principles from references without copying them.

## 1. Spec before pixels

Design like a design system, not a wish. Before drawing, write a short spec for yourself from the brief and what the person told you. Fill every line; if a line is unknown and would change the work, ask one question, otherwise assume and say so.

| Line | Example (salon in Yogyakarta) |
|---|---|
| Goal | Booking page for Salon Siti; success = a visitor books a cut in under a minute |
| Who | Women 20–45 near Jl. Malioboro, mostly on phones |
| Main action | "Pesan jadwal" (one primary button per view) |
| Format | Phone frame 390 wide first, desktop 1440 second |
| Layout in words | Serif statement left, portrait placeholder right; treatment list; location + hours; booking |
| Type system | Display serif 400 at 64/56, body sans 16/26, labels 11 uppercase |
| Colour | Paper `$color-background`, ink `$color-foreground`, one muted signal `$color-signal` for selection only |
| Imagery | Documentary photos of the real salon; captioned placeholders until provided |
| Copy | Exact lines, in Indonesian, from the person or drafted and marked for review |
| Constraints | The 1–2 things this round may change (see below) |
| Never | Invented reviews, stock-smile photos, purple gradients, English filler |

Ask when vague: What is the one message of this screen? What is the order (headline, support, action)? Which style lane? What must stay (logo, colour, font, price list)?

## 2. Lock the system, then vary one thing

- First version: get layout, hierarchy and real copy right. Polish waits.
- Set tokens once (`$color-*` by shadcn role, `$space-1`…`$space-12`, `$radius-sm`…`$radius-2xl`, `$font-heading`, `$font-body`, one shadow recipe) and draw everything from them.
- Hero first. The first viewport is half the job: it sets the type, palette and density every other section inherits. Then build section by section, top to bottom. Do not redraw the whole page each round.
- When exploring, change one variable per variant: accent colour, crop, card arrangement, background tone, or display font. Two variants that differ in five ways teach nothing.
- Protect what works. Before editing, list what must not change ("keep the hero, keep the price list") and check it is unchanged after.
- Write a constraints card on the canvas while exploring (off to the side, deleted before handoff): FONT, STYLE, MODE, ACCENT. It anchors you like a mini style guide.

## 3. Turn "make it good" into gates

Words like "bagus", "premium", "profesional" become checks you can see. Binary gates first, then a score.

| Gate | Check | Pass |
|---|---|---|
| Brief | Spec lines all visible in the design | Every line met |
| Main action | One primary button per viewport, specific verb | Yes |
| Readability | Body ≥ 16px, contrast ≥ 4.5:1, measure ≤ 70 characters | Yes |
| Touch | Tap targets ≥ 44px tall on the phone frame | Yes |
| Phone | 390 frame re-prioritised, nothing clipped, 16px+ side padding | Yes |
| Truth | No invented proof; placeholders captioned | Yes |
| Slop | `slop-audit` has no P0/P1 left | Yes |

A failed gate blocks the save regardless of score.

## 4. Score to a target

Default target: **8/10 on every screen** (the person may name another; a bar they set once stands for the rest of the project). Score each screen separately, never the project as one blob.

Anchors:

| Score | Means |
|---|---|
| 10 | Best in class; something to study |
| 9 | A trained eye finds almost nothing |
| 8 | Polished and intentional, minor nits only |
| 7 | Good, with one noticeable flaw |
| 6 | Works and reads, rough edges visible |
| 5 | The idea is there, execution distracts |
| 3–4 | Broken or wrong in places |

Rubric, 0–2 points per criterion, total out of 10:

| Criterion | 0 | 1 | 2 |
|---|---|---|---|
| Fit | Could be any business | Partly specific | Only this business, this customer |
| Hierarchy | No clear focus or action | Focus clear, action weak | One focus, one obvious action per view |
| System | Values off-token, drift | Mostly consistent | Every value from tokens, one style lane |
| Craft | Collisions, uneven insets, bad breaks | Minor misalignments | Aligned, even insets, clean line breaks |
| Content truth | Filler, fake proof | Real but generic | Real, specific Indonesian copy and data |

Measure where you can: contrast ratio, body size, tap height, line length, number of accents, number of distinct radii. Numbers stop the score drifting with mood.

## 5. Run rounds

1. Score the current version first; this is the baseline.
2. Fix the lowest screen first, and in it the criterion losing the most points.
3. Change 1–2 things per round. Name them ("round 2: removed eyebrows, widened hero measure").
4. Re-render, re-score with the same rubric. Record before → after, e.g. "Beranda 5 → 7 → 8".
5. Stop when every screen meets the target and every gate passes.
6. If a screen does not move after two rounds, change the approach (different layout, different hero) instead of polishing the same idea. If it is blocked by something outside your control (no photos yet, prices unknown), stop and say what is missing.

Rules while climbing:
- Never round up. 7.5 on an 8 bar is below the bar.
- Do not raise one screen by breaking another, or by dropping a gate.
- Judge from the rendered canvas, not from the scene JSON or from what you intended.

## 6. Separate making from judging

You drew it, so you are biased. Before scoring, switch roles: look at the render as a critic who did not build it and knows only the spec and rubric. Find failures first, cite where on the canvas, reject your own rationale ("I meant it to feel airy" is not evidence). When comparing two directions, judge them under the same conditions, against the same rubric, before you decide which you prefer.

## 7. Save with a scorecard

When saving a version, keep a short note (for yourself and Koda, not a sales pitch):

| Screen | Before | After | Main fix | Still short |
|---|---|---|---|---|
| Beranda | 5 | 8 | Removed card wall; one hero action | — |
| Booking | 6 | 8 | Added error and success frames | Real photos pending |

Tell the person the result in one line ("Semua layar sekarang 8/10; foto asli masih placeholder"), then what changed. Say that scores are judgements and which checks were measured.

## 8. References without copying

References are evidence of a quality, not a template. When the person shares a site, screenshot or competitor:

- Extract principles: hierarchy, pacing, density, contrast, material, how the main action is placed. Write them down in a line each.
- Rebuild those principles around this business's content, words and photos.
- Never carry over: names, logos, wordmarks, copy (any 8-word run is too close), prices, numbers, photos or their composition, plan names, or a distinctive section order reproduced end to end.
- Recolouring is not a fix. Copied copy gets rewritten from this business's own offer; a copied composition gets a new subject, crop and arrangement.

Before saving a design made from a reference, check overlap by category:

| Category | Red flag |
|---|---|
| Text | Same headline structure and wording, same CTA phrases beyond basics like "Masuk" |
| Brand | Their name, mark, people or URLs visible anywhere |
| Numbers | Their prices, stats, dates or plan structure |
| Images | Same subject, pose and crop with only colour changed |
| Structure | Several of their signature devices in the same order |

Severity: **Blocker** (their copy, identity or media shipped) → replace now. **High** (near-copy keeping several signature elements) → redesign the section. **Medium** (one recognisable overlap) → change that element. **Low** (shared grammar, original content) → fine, note it.

Not red flags by themselves: black and white, one accent, a large sans headline, a hero + services + pricing + FAQ + footer order, common icon sets, ordinary photo subjects. Escalate only a distinctive combination or direct evidence.

Adapted from mengto/skills: agent-skills/ui/design-first-ui-prompting, agent-skills/codex/iterate-until-verified, agent-skills/workflow/workflow-score-to-target, agent-skills/codex/audit-reference-originality (MIT, Copyright (c) 2026 Meng To).
