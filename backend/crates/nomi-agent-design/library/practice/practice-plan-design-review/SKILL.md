---
name: practice-plan-design-review
description: "How to review your plan before drawing: rate it 0-10 on seven passes (hierarchy, states, journey, slop risk, system, responsive and access, open decisions), fix gaps one decision at a time, score by the weakest pass. Read after writing a plan."
license: "MIT, Copyright (c) 2026 Garry Tan (gstack); via open-design (Apache-2.0)"
source: "https://github.com/garrytan/gstack/tree/a3ecfd530df3c78c2d60e92b6488586a75e0ff91/plan-design-review (SKILL.md.tmpl, sections/review-sections.md.tmpl, checklist); open-design@5fdcc3d"
---
# Plan design review: rate the plan before you draw

Rate your written plan 0-10 on seven passes, say what a 10 looks like for this plan, close the gaps one decision at a time, and only then draw.

`practice-design-review` audits frames you have already drawn. This skill reviews the plan (your Taste plan lines, or a brief the person wrote) before any frame exists, when fixing a gap costs one sentence instead of a redraw.

## Principles behind every score

1. Empty states are features: warmth, one action, context.
2. Every screen has a first, second and third thing.
3. Specific beats vibes: name the face, the spacing step, the component, the interaction.
4. Edge cases are user experiences: long names, zero results, errors, first visit versus the hundredth.
5. Generic is the enemy: template card grids, a stock hero, three feature cards.
6. Responsive is a design per width (390, 768, 1440), not "stacked on mobile".
7. Accessibility is in the plan (contrast, 44 targets, focus, reading order) or it won't be in the design.
8. Subtract by default: an element that doesn't earn its space goes.
9. Trust is earned at the pixel level.

How people behave (Krug): they scan, so each screen is a billboard with one obvious next step; they muddle through, so the default path must be the right one; every screen passes the trunk test (where am I, what is here, what can I do, how do I go back); clickable things look clickable; conventions beat novelty; every confusion drains goodwill; a phone raises every stake (thumb reach, 44 targets, no hover).

Trace every judgement to a broken principle, never "rasanya kurang pas". Imagine the user on a weak signal, one-handed, first time and thousandth time, and storyboard the journey, not single screens.

## Step 0: the overall rating

- Rate the plan's design completeness 0-10 with the reason: "3/10: rencana menyebut fitur, tapi tidak menyebut apa yang dilihat pembeli."
- Say what a 10 looks like for this plan.
- Note what already exists to reuse (the design system frame, components, the person's assets).
- If no design system exists yet, flag it: the system comes first (`practice-design-consultation`, `components-kit`).
- If the person wants, focus on a few passes; otherwise run all seven.

## The seven passes

For passes 1-6, write the score, list the gaps, propose a fix for each ("FIX TO 10"), get the person's yes, no or change for anything that alters what they asked for, then re-score with the reason. Your own craft fixes (a state you forgot) you just make. A declined fix stays recorded and keeps the score below 10.

### Pass 1: information architecture

Does the plan say what is seen first, second, third on every screen? Fix: one line per screen in order, plus the navigation between screens. If only three things could be shown, which three?

On a persuade page (landing, pricing, campaign) also settle: the one action, the first-screen fact, the proof the brief really gives (ask for it, never invent), the risk reducer (free trial, cancel anytime, cash on delivery, only if real), and the close.

### Pass 2: interaction states

Does the plan specify loading, empty, error, success and partial states? Fix: a table, describing what the user sees.

| Feature | Loading | Empty | Error | Success | Partial |
|---|---|---|---|---|---|
| Daftar pesanan | skeleton rows | "Belum ada pesanan hari ini" + Bagikan katalog | "Gagal memuat. Coba lagi" | list | some items unavailable, marked |

### Pass 3: journey and feeling

Does the plan think about how the user feels? Fix: a storyboard.

| Step | User does | Feels | Plan supports it with |
|---|---|---|---|
| 1 | opens the link from Instagram | curious, a little wary | the real cake photo and the price on screen one |
| 2 | picks a cake | wants certainty | size, servings, pickup times beside the button |

Think in three horizons: the first five seconds (gut), five minutes (use), and five years (would they come back and recommend it).

### Pass 4: slop risk

Does the plan describe specific, intentional design or generic patterns? Check the hard rejections; any one unresolved caps this pass below 8:

1. A generic card grid as the first impression.
2. A beautiful image with a weak brand.
3. A strong headline with no clear action.
4. Busy imagery behind text.
5. Sections repeating the same mood statement.
6. A carousel with no narrative purpose (Taste bans carousels anyway).
7. An app screen made of stacked cards instead of a layout.

Then answer yes or no, with evidence: Is the brand unmistakable on the first screen? Is there one strong visual anchor? Is the page understood by its headings alone? Does each section have one job? Are the cards necessary? Does the motion improve hierarchy? Would it still feel premium with every decorative shadow removed?

Fix vague lines with decisions: "kartu dengan ikon" becomes what makes these cards this business's; "hero section" becomes what makes this hero feel like this shop; "clean, modern" is replaced by the actual choices; "dipercaya ribuan pelanggan" needs the person's real proof beside it, or goes. Full slop list: `slop-audit`.

### Pass 5: design system alignment

Does every screen use the system's tokens and components? Fix: name them in the plan (`$color-primary`, `Card / Default`, `Button / Primary / Default`). Flag any new component: does it fit the existing vocabulary, and does it have a shadcn home (`components-kit`)?

### Pass 6: responsive and accessibility

Does the plan specify each width and access? Fix: what changes per width (the nav becomes a bottom bar on 390, the table becomes a list), contrast pairs at 4.5:1, 44 targets, focus states, reading order, labels on every field.

### Pass 7: open decisions

Unscored. List what will haunt the build if left open:

| Decision needed | If left open |
|---|---|
| What the empty cart looks like | Koda ships "No items found" |
| Phone navigation | the desktop nav hides behind a hamburger |
| What happens after payment | the user is left on a blank screen |

Ask the person each one that is theirs to decide, with your recommendation and why; decide your own craft ones.

## Completion summary

| Pass | Before | After |
|---|---|---|
| 1 Hierarchy | 4 | 9 |
| 2 States | 2 | 8 |
| 3 Journey | 5 | 8 |
| 4 Slop risk | 6 | 9 |
| 5 System | 7 | 10 |
| 6 Responsive and access | 3 | 8 |
| 7 Decisions | 3 resolved, 1 open | |

**The overall score is the lowest of passes 1-6,** before and after. Ready to draw at 8 or more with nothing open that blocks a screen. Add two short lists: not in scope, and what already exists to reuse.

## Rules

- Never skip or compress a pass because the plan "isn't visual"; design gaps are where builds break.
- One question per issue, with a recommendation and why; don't batch ten questions into one message.
- Keep the decisions you made consistent across passes; one fix answers every place it applies.

Adapted from gstack's plan-design-review by Garry Tan (MIT, Copyright (c) 2026 Garry Tan), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
