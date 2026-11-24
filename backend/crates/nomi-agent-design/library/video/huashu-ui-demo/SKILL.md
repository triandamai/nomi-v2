---
name: huashu-ui-demo
description: "The playbook for a video whose hero is a product interface (the app or site you designed for the person): screenshot camera versus rebuilt UI, the eight ways to show an interface, rhythm rules and where each sits in the film. Read it before animating any product UI."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# UI demo videos

The playbook for a video whose hero is a product interface (the app or site you designed for the person): screenshot camera versus rebuilt UI, the eight ways to show an interface, rhythm rules and where each sits in the film.

**The claim:** a product video's quality comes from **real UI plus cinematic camera**, not effects. Viewers must recognise "that's the product". One real screenshot and one restrained push beat ten layers of fake interface. Read `huashu-cinematic` for camera vocabulary and `huashu-animation` for element motion; this file decides how the UI performs.

Two axioms over everything: the cursor and the action happening must be inside the frame with an 8% margin at all times; the visual language grows from the product's own tokens (fonts, radii, palette, grid), which for Nomi are the design system frame you drew.

## Screenshot, rebuild, or both

| Path | How | Effort | When |
|---|---|---|---|
| Screenshot camera | Real screenshots in a browser or phone frame; the camera moves | Hours | The UI is looked at, nothing inside it changes |
| Rebuild | The screen rebuilt as live elements from the screenshot | Days | Elements need their own timeline: building up, typing, state changes |
| Hybrid | The full screenshot as base; only the moving elements cut out and animated in place | Half a day | 90% still, 10% alive. **Usually right** |

Hybrid steps: a full 2× screenshot in a device frame; cut the moving elements as transparent slices with their coordinates; cover their baked-in originals with patches of the page colour; animate the slices; **every slice ends exactly in its real slot** (a card hovering above the grid instead of landing reads fake). For close-ups use 4× slices cross-faded in over 6 frames. A device frame says "real software"; past 2× zoom it leaves the frame and plain slices are fine. A rebuild must match the screenshot area by area (type, radius, spacing, icons) and be compared side by side: a "roughly similar" interface hurts more than a screenshot, because people know their own screens. No real screenshot: ask for one or export the canvas; never a mockup generator.

## Eight ways to show an interface

One shot shows one way; each way is the hero once per film.

1. **Hero showcase:** one card or panel becomes the film's protagonist: spotlight, push, float, settle. Side angle (rotate mostly around the vertical axis, about 34°, a little tilt), a still beat at 0.78× before pushing to about 2.6×; rise with slight overshoot, hover with a gentle bob, settle with a tiny press; shadows grow with the height; an outline light sweeping twice (fast and bright, then slow and dim) only for this hero, once. About 3.3 seconds from lock to landing; first drafts are always too fast.
2. **Build-up:** the interface appears in grammar order: frame, grey skeleton, content blocks with stagger, data last. The swap from skeleton to real is one quick decisive beat; the layouts at each stage are identical. Words rise one by one; the last word lands half a beat late as the full stop. Rows embedding into a list tilt flat as they land (tilt, not just drop). A document block by block with one caret only. Count blocks against the time first; too many, cut blocks, don't speed up.
3. **User typing:** per character at human speed (3 frames per character, 2 in a terminal), cursor solid while typing, blinking after; one scripted typo with backspace; 0.37s before the page responds. Code colours token by token as typed. On a screenshot, cover the baked placeholder first. Never use the AI chunk reveal for a user.
4. **Cursor as actor:** arcs, a click with two concentric ripples starting 3 frames apart (expand fast, fade evenly), a slight shrink before the click, the hover target lighting on the same frame. The camera moves first if the target is off-frame. A click followed by a 16-frame push into the detail is the standard hand-off.
5. **State changes on the timeline:** a dialog: background dims and blurs, panel drops with a small 8px overshoot, options staggered. Tabs: the indicator slides, old content sinks out, new rises; indicator first, content half a beat after. Route push: new page from the right, old page drifts back 30% and darkens. The camera holds still during the change. One state change per shot.
6. **3D tour:** either the camera is still and the page glides past tilted about 60° (55 to 65°; content needn't be read), or the pages float still and the camera grazes across them (content readable). Never both. A three-column wall of pages with columns looping at clearly different speeds (25% apart or more) states "many pages" at the close.
7. **Scroll and brake:** one fast scroll on a single expo curve, motion blur only while moving, stopping on the target row, which lifts slightly (1.03) while the rest dims (not to zero). No push during the scroll; at most one gentle push after it stops. Brakes count against the camera budget.
8. **Callouts:** leader lines drawing in (12 to 18 frames, text 5 frames after the line arrives); a highlight box with the outside dimmed about 30%; a loupe holding a sharp 2 to 3× slice; a before and after slider (a fast flick, a pause, a slow sweep, speed ratio about 5:1), with the same layout and angle on both sides and a real "before". Callouts are a guide, styled one layer apart from the product UI.

## Placing them in the film

| Part | Ways |
|---|---|
| Brand opening (8 to 12%) | None: no UI steals the opening |
| Hero portrait (12 to 15%) | 1 or 2, the slowest, highest-craft shot |
| Feature climb (55 to 65%) | 3, 4, 5, 7, 8 alternating, one feature per shot; callouts make good low-energy breaths |
| Close (13 to 16%) | 6 as the group shot, then sign-off |

Sound hooks, added after picture lock: hero rise whoosh, sweep sparkle, landing snap; skeleton-to-real pop, light ticks per row; keyboard sound exactly as long as the typing; the click is the loudest sound in the film, ripples silent; dialog soft pop, route push quick whoosh; tours near silent, a dull thud at the brake.

## Rhythm rules

1. Interaction at human speed; first versions are always too fast.
2. Batch motion ends with 0.5s of stillness.
3. Exits are staggered in reading order; everything vanishing at once reads as a crash.
4. One motion per shot.
5. Holds are truly still: no drifting zoom or opacity.

Three directions for a UI video are not three skins: they are three camera narratives of the same screens (a hero showcase; a build-up telling "it generates"; a tour and scroll telling scale), each shown as two or three keyframes.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
