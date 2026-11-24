---
name: huashu-animation
description: "Huashu's motion grammar for video: the motion designer's taste, Slow-Fast-Boom-Stop, easing as physics, eight motion-language rules, the techniques, three recipes, the visual density clause, the timeline principles and the pitfalls that break renders. Read it before animating any video composition."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Animation: motion as physics

Huashu's motion grammar for video: the motion designer's taste, Slow-Fast-Boom-Stop, easing as physics, eight motion-language rules, the techniques, three recipes, the visual density clause, the timeline principles and the pitfalls that break renders.

**Scope:** rendered video only (HyperFrames compositions). It never applies to a site or app: there Taste rule 5 and `ux/motion-timing` decide (hover micro-interactions, one hero entrance, no scroll fade-ins). The composition format and render contract are in `video-hyperframes` and `video-hyperframes-composition`; this file is about how things should move. No colour values here: colour comes from the brand spec or the chosen direction.

## Who you are when animating

A motion designer who has studied Anthropic, Apple, Pentagram and Field.io. You aren't tweening properties; you are simulating a physical world so the viewer's subconscious believes these are objects with weight and inertia. Three beliefs:

1. **Motion is physics, not curves.** Linear is a number; expo-out is an object. Every easing answers: how heavy is this, how much friction?
2. **Timing matters more than curve shape.** Even pacing is a tech demo; rhythm is narrative. Slowing down at the right moment beats the right easing at the wrong moment.
3. **Giving the viewer time is harder than showing off.** Pausing half a second before a key result is craft, not compromise. AI animation defaults to no pauses and full density; that is a beginner.

Beauty, and how to recognise it: **weight** (an element lands rather than stops); **courtesy** (a felt pause of 300ms or more before key information); **restraint** (one 120% moment, the rest just right; effects everywhere signal cheapness); **hand feel** (arcs not lines, irregular not metronomic); **respect** (show the process and the fix, not magic).

The only metric: the viewer's first reaction. "Lancar" is good (you made PowerPoint). "Kok kayak benar-benar terangkat dari meja" is great. "Ini nggak kelihatan bikinan AI" is the bar. "Mau saya screenshot" is the best. When rules don't cover a case or conflict, decide from this section; to break a rule, name which beauty it serves.

## 1. Slow-Fast-Boom-Stop

| Part | Share | Pace | Job |
|---|---|---|---|
| S1 Trigger | ~15% | Slow | Reaction time, realism |
| S2 Generate | ~15% | Medium | The first striking visual |
| S3 Process | ~40% | Fast | Control, density, detail |
| S4 Boom | ~20% | Peak | Camera pulls back, a pop-out, panels flood in |
| S5 Landing | ~10% | Still | Logo, then a hard stop |

For 15 seconds: 2, 2, 6, 3, 2. Never even pacing, never constant high density, never a fade-out ending: **stop dead and hold the final frame.** Self-check: five thumbnails, one per part; if they look alike, there's no rhythm. For multi-shot films over 20 seconds use the energy arc in `huashu-cinematic`, with this feel inside each shot.

## 2. Easing

| Use | Easing |
|---|---|
| Cards rising, panels entering, overlays (the default) | **Expo-out** (sharp start, steady settle; plain ease-out is not sharp enough) |
| Toggles, buttons popping, emphasis | Overshoot (back-out) |
| Things falling into place, small UI bounces | Spring (elastic-out) |
| Continuous symmetric motion (a cursor path) | Ease-in-out |
| Only proxy-driven motion and a deliberately constant camera drift | Linear, never on an element |

Most promo motion is too fast and too hard. A custom anticipation curve that dips below zero goes only on position, scale or rotation, never opacity or colour.

## 3. Motion language: eight rules

1. **No pure black or pure white grounds.** A tinted neutral (warm or cool) reads as paper or desk, not machine.
2. **Never linear on elements.**
3. **Slow-Fast-Boom-Stop.**
4. **Show the process, not a magic result.** Dragging a slider, an error and its fix, red deletions and green insertions. The product is a collaborator, not a magician; showing the process is where recognition comes from.
5. **Hand-drawn cursor paths:** accelerate, arc, slow and correct, click. Straight interpolation feels robotic. Add a tiny deterministic wobble (two sine waves of unrelated frequency), never runtime noise.
6. **The logo morphs in.** The previous visual collapses into a colour block that unfolds into the wordmark (a quick 150ms cut with a brief blur from 6px to 0). Never a plain fade-in.
7. **Serif plus sans:** serif for brand and narration, sans and mono for UI and data.
8. **Focus change = background dims, foreground sharpens, a flash leads the eye.** Opacity alone isn't enough: blur the background 4 to 8px so it truly steps back.

## 4. Techniques

- **Shared element:** a button swells into an input field; the same element changes state, not two elements cross-fading.
- **Breathing expansion:** first 40% widen only, last 60% grow height ("unfold, then fill").
- **Staggered rise:** list items, rows, cards enter 30ms apart, rising 10px.
- **Pause 0.5s before a key result.**
- **AI output is a chunk reveal:** two to five characters at a time at irregular (precomputed) intervals, like streamed tokens. **User typing is different:** steady per character (about 3 frames per character at 30fps), cursor solid while typing and blinking after. Never mix the two.
- **Anticipation, action, follow-through:** a small reverse move before (a button shrinks slightly), the action, an after-swing (a card settles with a slight bounce).
- **3D float:** perspective with layers at different depths; tilt about 8° on one axis and -4° on the other (over 10° looks like falling over, under 5° like a skew).
- **Diagonal drift:** move X and Y together at different frequencies so the loop never looks regular.
- **Variable-font weight change** (thin to bold) reads more cinematic than scaling.

## 5. Three recipes (pick one, don't mix)

| | A · Keynote drama | B · One-take tool | C · Office narrative |
|---|---|---|---|
| For | Big launches, hero films | Developer tools, productivity, flow | Business software, documents, calendars |
| Rhythm | Strong Slow-Fast-Boom-Stop arc | Steady flow, no clear peak | Several hard-cut scenes with dolly in and out |
| Easing | Expo-out, a little overshoot | Spring and expo-out | Overshoot for toggles, expo-out for panels |
| SFX density | High (~4 per 10s), pitched to the music | Zero: the music drives the cut | Medium (~3 per 10s), mostly clicks |
| Music | Minimal precise electronic | Lo-fi, 85 to 90 BPM | Light jazz, minor, 90 to 95 BPM |
| Signature | Fast pull-back, drop, logo morph, one airy tone, hard stop | Key UI actions land on kick and snare | One "hero moment": a 3D pop-out, once only |

## 6. Anti-patterns

| Don't | Do |
|---|---|
| One transition for everything | Expo-out with per-element stagger |
| Every entrance an opacity fade | Fade with a 10px rise and anticipation |
| Logo fades in | Morph, converge, collapse and unfold |
| Straight cursor | Arc with a little wobble |
| Typing one character at a fixed interval | Chunk reveal for AI, human rhythm for users |
| No pause before results | Hold 0.5s |
| Focus by opacity only | Opacity, brightness and blur |
| Everything at one speed | Slow-Fast-Boom-Stop |
| Fade-out ending | Hold the last frame |

## 7. The visual density clause

A brief that only describes story and camera gets back a wireframe: an animation that passed every motion check but looked like "three plain dark blocks and big type" was rejected as boring. Every animation brief must state: **a density standard** (how much detail per frame: UI rows, legends, textures, secondary elements; and "pause any frame, put it beside the benchmark, it must not embarrass"); **a benchmark** (a named finished piece whose craft to match); **the ambient layer** (a ground line, soft shadows on components, ground texture, a small companion element, idle micro-motion on still parts: breathing, a blinking caret, a slow push). The ambient layer, not the hero, is the main cure for emptiness. When work is rejected, ask which layer failed: if motion passed, re-skin; don't re-choreograph.

Timing guide: micro-interactions 0.1 to 0.3s; UI transitions 0.3 to 0.8s; one narrative beat 2 to 10s, never more than 10s per idea. Readers need about 300 to 500ms per word: leave a sentence on screen at least 3 seconds. A static frame longer than 5 seconds gets dull.

## 8. Timeline principles (seek-safe rendering)

The renderer seeks to each frame and captures it, so every state must be a pure function of time. Previews often look right while renders break.

- One paused timeline built synchronously; the renderer drives it. Duration comes from the composition, not the timeline length; repeats are finite.
- **No wall-clock motion on the render path:** no CSS transitions with class toggles, no CSS keyframe animations, no browser animation calls. Express everything as timeline tweens.
- **No runtime randomness or clocks:** precompute "random" values from a seeded generator before building the timeline.
- **Animate transforms and opacity, never layout** (width, height, top, left, margins, font size, letter spacing): layout snaps to whole pixels and slow eases visibly judder. Use scale for size and offsets for position. The one exception is the camera layer's zoom (`huashu-cinematic`).
- Blur and filters through variables on the few elements that actually move them; keep large-area blur at 24px or less.
- Write holds as explicit labelled gaps; end with no tweens for the last 0.5 to 1s (the hard stop). Ease the music volume down instead.
- A from-to tween whose start state is visible renders that state immediately: sparks, ripples and click rings appear seconds early. Mark it not to render immediately, or set hidden then tween.
- Proxy-driven scenes (a cursor path, a chunk reveal) need the first frame drawn explicitly; at time zero the update may not fire.
- In sub-compositions use from-to (both ends explicit), not from.

## 9. Pitfalls

- Every absolutely positioned element's parent is explicitly positioned; otherwise it floats to the wrong frame.
- Every character on screen exists in the chosen font (rare symbols such as ⏎ ⌘ ␣ become boxes; draw keys as labelled boxes). Emoji often render empty.
- Grid columns come from the data count, not a hard-coded number.
- Scene changes overlap (cross-fade), never "out, then in" with an empty gap; a 1-second blank reads as a frozen video.
- Measure text positions only after fonts load.
- Frame 0 is the complete initial layout, never blank; the last frame is the final state, never a loop back to the start.
- **No fake player chrome in the picture:** no progress bars, timecodes, chapter counters "03/06", version numbers or credit strips. Every element belongs to a scene's story; if it belongs to no scene, delete it.
- Elements that appear across scenes (labels, a watermark) never hard-code a colour: on a dark scene they vanish silently. Check one frame per scene.
- Dark cinematic designs deliberately use low-contrast watermarks and labels; the contrast check can be skipped for those only, every other check must pass. Light informational videos keep the contrast check.

## Before delivery (60 seconds)

Slow-Fast-Boom-Stop or an energy arc; expo-out default; overshoot on toggles; 30ms stagger; 0.5s holds; chunk reveal for AI text; blur on focus changes; logo morphs; tinted grounds; serif and sans; hard stop; arced cursor; SFX density fits the recipe; music 6 to 8 dB under SFX (`huashu-audio`).

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
