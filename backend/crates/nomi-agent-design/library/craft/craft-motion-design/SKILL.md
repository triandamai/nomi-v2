---
name: craft-motion-design
description: "How to design for motion on a canvas that can't move: whether something should animate at all, what it is for, how it should feel, which states to draw so Koda can animate between them, and how to plan a short motion piece as a beat map. Read it before adding any motion note."
license: "MIT, Copyright (c) 2026 Emil Kowalski (emil-design-eng, review-animations, animate, emilkowalski-motion); MIT, Copyright (c) 2026 Matthew Blode (ui-animation); BSD-3-Clause, Copyright 2026 The Flutter Authors (flutter-animating-apps topic); open-design's motion-design (Apache-2.0); via open-design (Apache-2.0)"
source: "https://github.com/emilkowalski/skills @a47903a and @e8a175d; https://github.com/mblode/agent-skills @5a781a8; https://github.com/flutter/skills; open-design@5fdcc3d skills/motion-design, emilkowalski-motion"
---
# Designing for motion

How you decide what moves, why, and how it should feel, and what you draw so Koda can build it: the canvas holds still frames, so motion design here means choosing the states, the purpose and the character, then writing a precise note (`craft-motion-notes`).

Taste sets the motion budget of a website: the hero entrance, one or two dealt moments, the hover and press layer, effects only on tech pages (`taste-photos-motion`). This skill is the judgement inside that budget and for app screens, where Taste is silent. Durations and curves live in `motion-timing`; this skill never overrides them.

## Taste is trained

Good motion is mostly motion nobody notices. When a control does exactly what the person expected, they move on without a thought; that is the goal. Hundreds of small correct decisions (the right origin, a fast exit, no animation on something used all day) add up to an app that simply feels right. Study why the best apps feel the way they do; never add motion because it looks impressive in isolation.

## 1. Should it animate at all?

Ask how often the person will see it:

| How often | Decision | Examples in Nomi apps |
|---|---|---|
| Many times a day, or started from the keyboard | No animation | Search shortcut, switching tabs in the kasir, adding an item to a POS cart, list navigation |
| Tens of times a day | Nearly invisible: fast and subtle, or nothing | Hover on rows, menu open, toggles |
| Occasionally | Standard motion | Dialogs, sheets, toasts, a form step |
| Rarely or once | Room for delight | First product added, first order received, onboarding finished |

A "no" is a good answer. Delight scales inversely with frequency.

## 2. What is it for?

Name the purpose in one word before writing a note; if you can't, don't animate:

- **Feedback:** the interface heard the person (a press, a toggle, a saved state).
- **Orientation:** where something came from or went (a sheet from the bottom edge, a menu from its button).
- **Continuity:** the same thing changes instead of being replaced (a card that expands into its detail).
- **State:** making a change readable (a status moving from Menunggu to Lunas).
- **Explanation:** showing how something works (a hero demo on a software page, onboarding).
- **Delight:** only in the rare tier.

Data people read or act on never moves for style: a sales chart in the owner's dashboard doesn't wobble on hover.

## 3. Principles you design with

- **Continuity over teleporting.** An element present in both states transitions in place. Draw both states with the same ids so Koda can morph one into the other.
- **Emerge from the trigger.** Menus, popovers and sheets grow out of the control that opened them; only dialogs, which have no anchor, appear from the centre. Better still, the trigger becomes the surface: a search icon that expands into the search field.
- **Confirm in place.** The result of an action shows on the control that caused it: "Simpan" becomes "Tersimpan" with a check, holds, and reverts. Use a toast only for results with no on-screen origin (a payment arriving, a background export finishing).
- **Direction matches layout.** Forward slides left, back slides right; a sheet leaves the way it came.
- **Paired states move together.** If open animates, close animates; if hover has motion, focus and press get equal feedback. Elements that move together (dialog and its backdrop, tooltip and arrow) share timing.
- **Exit faster than enter.** The system's response is quick; the person's deliberate action may be slow (a hold-to-delete fills slowly, releases instantly).
- **Nothing appears from nothing.** Elements arrive from a slightly smaller size and transparent, never from zero size.
- **One entrance per container.** If a panel slides in, its content is already there; don't also stagger the contents.
- **Interruptible.** Anything the person can trigger twice in a second must be able to reverse from where it is.
- **Perceived speed.** Smooth, quick motion makes an app feel faster than a hard cut at the same load time; slow motion makes it feel broken.
- **Novelty budget.** Keep about 90% of a surface familiar and 10% special; never two high-novelty beats in a row.
- **One-shot flourishes stay one-shot.** An onboarding celebration does not replay on every visit.

## 4. Character: match the personality

Decide the feel once per product and write it on plan line 2 with the dials (`craft-taste-variants`):

| Product personality | Motion character |
|---|---|
| Kasir, stock, finance, admin | Crisp and fast, no bounce, mostly no motion |
| Booking, ordering, consumer app | Quick and smooth, a little softness on sheets |
| Kids, games, playful brands | Bouncier springs allowed on rare moments |
| Calm luxury (hotel, spa) | Slightly slower and softer, still never sluggish |

Everything in one product shares one motion language: the same curves and durations from `motion-timing`, no mixed physics.

## 5. What to draw

The canvas can't move, so draw the states Koda animates between:
- **Rest and the end state** of every transition, as separate frames side by side, same ids: menu closed and open; card and its expanded detail; button, pressed, loading, done.
- **The start state** only where it isn't obvious (a sheet below the screen edge, a skeleton before content).
- **Reduced motion:** nothing extra to draw; the note says what replaces movement (usually a fade, or the finished state at once).
- **Phones:** gestures (swipe to dismiss a sheet, pull to refresh) noted on the 390 frame; hover notes don't apply there.

## 6. Native apps (Flutter and mobile)

When the brief is a phone app rather than a web page, the same principles hold, with platform habits: pages push from the right and pop back; sheets rise from the bottom and follow the finger; a shared element (a product photo) flies from the list into the detail page; tab switches are instant or a quick fade. Note them in the same format; Koda maps them to the platform's built-in transitions.

## 7. A motion piece (hero demo, explainer, video)

For a software hero demo, an onboarding sequence or a short explainer (tech kinds only on a website, `taste-photos-motion`), plan before notes:

1. **One idea.** What should a viewer understand and remember?
2. **A representative frame first.** Design the single strongest still: hierarchy, crop, type, palette. A weak still is never rescued by motion; this frame is also the poster for reduced motion and phones.
3. **A beat map** in the plan, one line per beat: `time → what the viewer takes away → focal object → what changes → words on screen → exit`. Include reading holds; text needs time to be read, not just to appear.
4. **A recurring motif** (a line, a shape, the product's own UI element) that carries from beat to beat, so it is one piece, not four title cards.
5. **Three layers, not three effects:** the primary action carries the idea; a secondary reaction shows cause and weight (a label settling after its card); ambient life only if the scene needs it. Still moments are allowed.
6. **Alternate action and rest;** save the biggest change for the central idea; end on a held final frame.
7. **Real material:** the client's actual screens and data, cropped not stretched; axes and units stay correct through the whole movement.

Draw the beats as a row of frames (the storyboard) with the beat-map line under each, outside the page.

Adapted from Emil Kowalski's design-engineering, animate and review-animations skills (MIT, Copyright (c) 2026 Emil Kowalski), Matthew Blode's ui-animation (MIT, Copyright (c) 2026 Matthew Blode), the Flutter team's animating-apps topic (BSD-3-Clause) and open-design's motion-design, via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
