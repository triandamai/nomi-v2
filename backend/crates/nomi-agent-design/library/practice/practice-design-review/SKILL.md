---
name: practice-design-review
description: "A senior designer's visual audit of your own canvas frames (or a screenshot of the person's current site): first impression, trunk test, a 10-category checklist, flow and goodwill check, A-F grades, then one fix at a time. Read it before a version you will show, or when asked to review or polish."
license: "MIT, Copyright (c) 2026 Garry Tan (gstack; craft rules adapted there from Elaya's redesign-skill, MIT, Copyright (c) 2026 Elaya); via open-design (Apache-2.0)"
source: "https://github.com/garrytan/gstack/tree/a3ecfd530df3c78c2d60e92b6488586a75e0ff91/design-review; open-design@5fdcc3d"
---
# Design Review: audit, fix, re-check

A designer's eye on finished-looking work: you form a gut reaction, test whether a stranger can find their way, grade ten categories, name the slop, then fix the biggest problems one at a time and grade again.

Use it on your own frames at 1440, 768 and 390 before you show a version, after the person says "something feels off", or on a screenshot of the site they have today. Taste is your rulebook: this review finds problems, `taste-check` and `slop-audit` are the pass/fail gates, and where this file and Taste disagree, Taste wins. The full checklist is in `references/design-review-checklist.md`.

## Before you judge: name the mode

The mode is what the visitor's win looks like on this screen, not what the business is.

| Mode | The visitor... | Judge by |
|---|---|---|
| Persuade (landing page, pricing, promo) | decides and acts | one composition per screen, one main action, proof beside claims |
| Operate (app screens: dashboard, orders, settings) | finishes a task | scanability, native patterns, few colours, dense but readable |
| Read (menu, articles, help, terms) | understands something | one reading column, 45-75 characters a line, wayfinding |
| Experience (portfolio, gallery) | is inside the work | the work owns the first screen, chrome gets out of the way |

A page that mixes them (a cafe site with a booking form) is judged section by section. Also note what exists already: the person's logo, colours, photos and your own design system frame. A finding that breaks the system you chose is more severe than a generic one.

## 1. First impression (write it before you analyse)

Look at the 1440 frame for a few seconds, then write four lines:

- "Halaman ini menyampaikan **...**" (what it says at a glance: competence, warmth, confusion?)
- "Saya melihat **...**" (one specific observation, good or bad)
- "Tiga hal pertama yang dilihat mata: **1, 2, 3**." If these are not the three you intended, the hierarchy is lying.
- "Dalam satu kata: **...**"

Then the **page area test**: name the purpose of every clearly bounded area. Any area you cannot name in two seconds is poorly defined, and that is a finding.

## 2. Trunk test (every screen)

Dropped on this screen with no context, can a stranger answer: what site is this, what page am I on, what are the main sections, what can I do here, where am I, how do I search or get help? All six is a pass, four or five partial, three or fewer a fail. A fail is a high-impact finding however polished the screen looks.

How people behave (Krug): they scan, they don't read; they take the first reasonable option; they muddle through. So each screen is a billboard with one obvious next step, clickable things look clickable, conventions beat novelty, and welcome paragraphs and instructions get cut. Fix noise by removing, not adding.

## 3. The checklist (10 categories)

Walk every screen through the ten categories in `references/design-review-checklist.md` (it holds each category's checks, the five persuade-page checks and the universal rules): hierarchy, typography, colour and contrast, spacing and layout, states, responsive (your 768 and 390 frames), motion notes, copy, slop, and "performance as design" (what you can control: photo sizes and notes, skeletons). Give each finding an **impact** (high, medium, polish) and a **category**.

## 4. Walk a flow

Pick the two or three journeys that matter (pesan kue, booking kamar, daftar akun) and walk your frames in order, narrating in first person: "Saya ketuk 'Booking kamar'... muncul form, tidak jelas tanggal mana yang wajib..." Name each element, where it sits and how heavy it looks. Ask: is there a state for loading, success and failure? Is feedback immediate? Are errors next to the field?

**Goodwill meter.** Start at 70. Drains: information the visitor wants is hidden (price, contact, shipping) -15; a forced tour, splash or popup -15; punishing format rules (rejecting spaces in a phone number) -10; asking for data you don't need -10; sloppy look -10; each ambiguous choice -5. Fills: the top task is obvious +10; an error that says exactly how to fix it +10; upfront about cost and limits +5; each saved step (smart default, prefilled field) +5; apologises when something fails +5. Below 30 is critical, 30-60 needs work, above 60 healthy. Report one line per step: `Langkah 2: Form booking 70 → 60 (-10 minta NIK, tidak perlu)`.

## 5. Across screens

Same nav and footer everywhere? The same button drawn the same way on every screen (it should be one component, placed as `ref`)? One tone of voice? Spacing rhythm carried through? Any one-off that should be a component?

## 6. Grade

Each category starts at A; each high finding drops a full letter, each medium half a letter; polish findings are listed but don't grade. Minimum F.

| Category | Weight |
|---|---|
| Visual hierarchy | 15% |
| Typography | 15% |
| Spacing and layout | 15% |
| Colour and contrast | 10% |
| States | 10% |
| Responsive | 10% |
| Copy | 10% |
| AI slop | 5% |
| Motion notes | 5% |
| Performance feel | 5% |

Give two headline grades: **Design** (the weighted average) and **AI slop** (graded on its own, with a one-line verdict). A: intentional and polished. B: solid, minor inconsistencies. C: works but generic, no point of view. D: noticeable problems, feels careless. F: hurts the user.

## How to phrase findings

Use "Saya perhatikan..." (observation), "Saya bertanya-tanya..." (question), "Bagaimana kalau..." (suggestion), "Menurut saya... karena..." (reasoned opinion), each tied to what the visitor is trying to do and each with a concrete change: "change X to Y because Z", never "spacing terasa kurang pas". Depth beats breadth: 5-10 well-argued findings beat 20 vague ones. Always end with **Quick wins**: the 3-5 biggest gains that are small edits.

## 7. Triage, then fix one at a time

Sort findings: **high** (hurts the first impression or trust) first, **medium** (felt, reduces polish) next, **polish** if time allows. Mark as **deferred** anything you cannot fix on the canvas: missing facts only the person has (real prices, proof, hours), a third-party widget, a decision that belongs to them.

Then, in impact order, for each finding:

1. Find the exact nodes (by id) responsible. Touch only those. Prefer a token or component change over restructuring; a change to a component fixes every instance.
2. Make the smallest change that resolves it. Do not "improve" unrelated things on the way.
3. Keep every element's id: the person's feedback finds elements by id.
4. Re-check the same frame at all three widths. Classify: **verified**, **best-effort** (needs something you can't see on the canvas, like a real photo), or **reverted** (made something else worse, so undo it and defer).

Within one impact level, take the biggest visible gain at the least risk first: type, colour, hover/focus/pressed states, layout and spacing, components, loading/empty/error states, copy, type-scale polish.

**Stop and check in** after five structural fixes (moving or replacing sections, not token tweaks), after any revert, or when you would touch frames unrelated to the finding: show the person what changed and ask whether to continue. Cap one review at about 30 fixes.

## 8. Final pass and report

Re-grade all touched screens. If any grade got worse, say so first. Report in this order:

- Mode, first impression and the trunk-test result per screen
- Findings, each with impact, category, status (verified, best-effort, reverted, deferred) and the element ids changed
- Design grade and AI slop grade: before → after
- Deferred items, each with what you need from the person ("Harga paket foto pernikahan belum ada di brief")
- One line the person can read in five seconds: "Saya menemukan 9 masalah, memperbaiki 7. Desain C → B, AI slop D → B."

## Rules

- Judge the drawn result, not your memory of the JSON: read the frames at 390, 768 and 1440.
- Name AI slop directly; most people cannot see it, so explain why it reads as generated.
- Claims come from the person, never from you (no invented testimonials, numbers, logos or guarantees); a missing proof is a deferred item, not something to fill.
- Write findings as you go; never reconstruct them at the end.
- Every finding needs a place you can point to: the frame, the node id, the width.

Adapted from gstack's design-review by Garry Tan (MIT, Copyright (c) 2026 Garry Tan; its craft rules adapted from Elaya's redesign-skill, MIT, Copyright (c) 2026 Elaya), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
