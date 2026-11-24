---
name: huashu-polish-loop
description: "The polish loop (look at your own renders screen by screen, write the three worst problems, fix, look again), how many rounds, when to rebuild a screen, and verification: check your checker first, then the render. Read it before showing any design, deck or video to the person."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# The polish loop and verification

The polish loop (look at your own renders screen by screen, write the three worst problems, fix, look again), how many rounds, when to rebuild a screen, and verification: check your checker first, then the render.

This extends `design-loop` §5 (rounds to a score) and `slop-audit`: those say what to score and remove; this says how to look. The first version is only a draft. The biggest difference between a designer and someone who writes markup is that the designer sits back, looks, sees what's wrong, fixes it and looks again. The same model that revises three times against its own screenshots delivers a full grade better than one that hands over the first pass.

## How to look

- Look at the **render**, one screen at a time at full size: the canvas export at 1440 and at 390 for sites and apps, every slide for decks, keyframes for video. A whole long page shrunk into one image blurs type and spacing; problems vanish.
- Slides don't scroll: one slide is one screen. Video: look at the planned acceptance frames and the first second after every scene change.
- Numbers a tool prints (shapes per screen, word counts, how much is empty ground) only tell you where to look first; they never replace looking. A quiet, whitespace-led direction will show a high share of empty ground: that is its intent; don't fill it to please a number.

## What to look for (eight questions, every round)

1. **First three seconds:** would someone who never saw this project know within three seconds what it is and for whom? Is something unique to this subject's world in the first glance?
2. **Hero material:** is the hero material on the direction card there? Is every picture the content needs present (an app page shows the app; a data page makes the data the picture)?
3. **An event on every screen:** scrolling down, does each screen have something that makes you pause, or is it "heading plus three cards" to the end?
4. **Hierarchy:** squint at each screen; does the most important thing jump out first?
5. **Craft:** alignment, spacing rhythm, a single word alone on a last line, ugly breaks, overflow, broken assets, an element not from this system.
6. **Phone:** the same questions at 390. Not "nothing broke" but "it looks designed on a phone too".
7. **Against the card:** beside the direction card and its reference, are the borrowed rules in place? Where did it drift?
8. **Take one thing off:** which single element, removed, makes this screen better?

Write down the **three worst** answers, precisely: which screen, which element, what's wrong ("Layar 3, kartu harga: tiga kartu sama persis, tidak ada yang jadi pusat").

## What to fix

- Only the three worst per round, then render and look again. Ten fixes in one round hide which one worked and break good parts.
- Structural first (first screen doesn't stand, hero missing, a whole screen mediocre), craft second.
- A screen that stays mediocre whatever you do: don't patch, **rebuild it** with a different way of organising the information. First ask: if a top studio took this job, what would they spend three extra days on, on this screen?

## How many rounds

| Stage | Rounds |
|---|---|
| Each of the three direction drafts | 1: look, fix the three worst. It's for choosing, not finishing |
| The chosen direction's full version | At least 2; until a careful look finds no three things worth fixing, usually 3 to 5 |
| The person is in a hurry | 1, and say only one round was done |

Still making big changes at round five means the problem is the direction or the structure, not polish: stop, tell the person what you see and offer two ways forward. Keep every round's render; at handover show the first and last first screen side by side.

If you can't see your renders in this run, don't skip: check every fact you can (sizes, contrast, element counts, the steady-track checklist in `huashu-workflow`) and say honestly "versi ini belum dipoles dengan melihat render".

## Verification answers "is it broken"; polish answers "is it good"

Do both.

### Rule zero: verify your verifier

Before trusting any render, confirm the renderer itself renders correctly. Two real failures: a quick-look thumbnail passed four line-spacing bugs that made text overlap in real PowerPoint (thumbnails take a simplified path); an office suite on one machine turned every non-Latin character into boxes, which looked like a broken file. **Control experiment:** give the same renderer a file you know is correct (the client's own template). If that breaks too, it's the environment, not your file; this saves hours in the wrong direction.

Trust ranking for slide files: quick-look thumbnails, never as acceptance; a headless office suite, layout trustworthy but fonts may vanish; a real presentation engine exporting to PDF, the best automatable check; the target app itself (PowerPoint, Keynote, Google Slides), the final word.

Add a check no renderer can fool: compare the text word for word between source and output (ignoring whitespace), and count images and shapes per page. Renderers fool eyes, not counts. The comparison source must be independent of the extraction: comparing the export against a list made by the same extraction shares its blind spots.

### The checklist

- It opens and renders; no errors.
- Every viewport the design claims (1440, 768, 390).
- Every interaction and state: tabs, toggles, open and closed, empty, error, success. AI-written interactions often have bugs that static images hide.
- Decks: every slide, no blank page, no overflow, fonts actually loaded (webfonts need a moment before capture), no leftover placeholder text.
- Video: the checks in `huashu-video-export`.
- Light and dark, if both exist.

## The scripts (run later in Nomi's render worker)

- `polish.py` takes a page and saves full-size screenshots screen by screen (plus a 390 phone pass), counts shapes, words and empty ground per screen, and appends a round to a polish log; it warns about "several screens of only text and boxes" and rounds logged without notes.
- `design_lint.py` measures only checkable facts: red for broken images, text too small, low contrast, console errors; yellow for template chrome (all-caps eyebrows, "A · B · C" chains, trailing arrows, one recoloured word); a hint for look families. Its `compare` mode says whether three directions are reskins of one skeleton, and its ledger remembers past deliveries' looks so new directions avoid them. It gives no score: taste is yours.

When a check reports the opposite of what your eyes see (it once called four clearly different app directions "the same" because it counted the phone bodies as identical cards), first find what it measured wrong before trusting or tuning it.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
