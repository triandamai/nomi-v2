---
name: huashu-cinematic
description: "Camera language as a budget, the shot vocabulary, zoom tiers and durations, how to join shots without nausea, openings and endings, transitions, parallax, scene-based storytelling and launch-film director's notes. Read it before any shot-level camera move or any video longer than a few seconds."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Cinematic: camera, scenes and director's notes

Camera language as a budget, the shot vocabulary, zoom tiers and durations, how to join shots without nausea, openings and endings, transitions, parallax, scene-based storytelling and launch-film director's notes.

`huashu-animation` covers how the actors move; this covers the camera and the film's structure. Storyboard method, the eleven frame laws, shot sizes, the energy arc and transition choices are in `references/storyboard.md`. `video-frame-templates` covers formats, hooks and safe areas for short social videos; this adds the camera and directing layer.

## 0. The camera is a budget, not an effect

Most AI camera work adds a zoom wherever one fits; that is the shared source of nausea and cheapness. Two axioms:

- **Visibility:** at every moment what the viewer should see is inside the frame, with an 8% safe margin. A shot that breaks this gets a lower zoom, is merged, or isn't shot.
- **Comfort budget:** every camera change spends attention. When the budget is spent, even a great shot isn't taken.

| Budget | Value |
|---|---|
| Gap between camera changes | 2.6 to 3.0s or more (restrained: 5s) |
| Changes in any 15-second window | 4 to 5 at most (restrained: 3); over budget, cut the weakest motive, don't squeeze gaps |
| Smallest zoom | 1.25×; less is perceived as wobble (except the 1.06× establishing push) |
| Largest zoom | 2.3× (restrained 1.8×); beyond that change the source material |
| Shots per minute | 4 restrained, 6 normal |

Cinematic means camera × light × rhythm × sound, each passable, not one maxed out. **Rhythm prefers slow:** every piece of user feedback pointed to "slower, stay longer", none to "faster". Unsure of a duration, choose the longer; unsure whether to move, don't.

## 1. Shot vocabulary

Ask of every move: **what question does this answer for the viewer?** No answer, no move.

| Move | Motive | Parameters | Never |
|---|---|---|---|
| Push in | "Look here next": one UI element, a number, a key word; rising tension | Tiers 1.3, 1.45, 1.8, 2.3; arrive 0.15s before the action | Under 1.25×; during a scroll, page change or video playing; mid-sentence in narration |
| Pull out | Reveal the whole: "it belongs to something bigger"; the closing shot | Closing: 0.55s pull plus 0.8s or more of still wide shot | Out-in-out pumping |
| Pan | Between two mid-distance focal points (0.22 to 0.45 of frame apart) | Combined zoom 1.25× or more; prefer diagonal (X and Y at different frequencies, 30 to 40px) | Focal points more than 0.45 apart: drop the shot |
| Orbit | A single hero's material close-up; the most "physical" shot | Rotation mostly around the vertical axis, perspective about 1100 to 1200px | More than once per film; dense text (face it straight on) |
| Dolly zoom | A "worldview flip" reveal: subject fixed, context changes | Subject pinned, background scaling 1 to 2.5 with reduced opacity | More than once; without a real narrative drop it's showing off |
| Still | Reading, real-speed interaction, dense frames; the default when out of budget | Wordmark hold 1s or more; 0.5s still after batch motion; opening hero arc 3s or more | Never wrong. Not moving is a directing decision |

Speed comes from acceleration, not constant speed: a short burst and a long settle feel fast; halving the duration feels cheap. Camera angle serves readability: dense frames face on; text close-ups from the side (rotate mainly around the vertical axis); no global tilt; no handheld shake in product films.

**Zoom or dolly:** zoom (scaling, no parallax) is clean and informational, for UI, documents, charts. Dolly (perspective with layers moving forward, parallax) is spatial and cinematic, for hero shots with real foreground, subject and background. Eight or fewer elements in 3D: real depth layers. Twenty or more: fake depth with shadows, blur and lightness. In between: only if the shot needs parallax.

## 2. Camera rig rules (for the composition)

- **Camera moves and element animations never share a transform.** All camera motion lives on one camera layer around a world layer; element entrances live inside the world. Subtitles and fixed labels live on a separate overlay layer that doesn't move with the camera.
- Push-in target = the transform origin at the target's centre.
- The world is bigger than the frame by the largest pan plus 8%, so pans never reveal empty edges; background layers fill that bleed too. Check the end points of every pan.
- An overflow clip, a filter, reduced opacity or a clip path on any layer between camera and 3D content flattens the 3D instantly; put such effects only on the innermost elements.
- **Text blurs when a page is scaled up with a transform** (the browser rasterises at layout size, then enlarges the bitmap; beyond 2× it's unusable). Zoom the world layer with layout-level zoom instead, so text re-rasterises sharp at any magnification. This is the only allowed animated layout property, on the camera layer only. Capture source screenshots at 2× and close-up crops at 4×, cross-fading to the sharp crop during a push.
- If a tracked target leaves the frame, keep the zoom and nudge along the overflowing axis only: the camera yields, the target never leaves.

## 3. Camera easing and duration

| Move | Easing |
|---|---|
| Deliberate push or pull with clear ends | Strong ease-in-out (both ends steady); never linear, never spring overshoot |
| Following an action already under way | Quick start, very long brake ("catching up", not cutting) |
| Idle drift | Gentle sine back and forth; the only constant camera motion allowed |

**Duration from magnitude:** duration = 0.55 × |ln(zoom_to ÷ zoom_from)| ÷ ln 2 seconds, clamped to 0.30 to 0.94. So 1× to 2× takes 0.55s; 1× to 1.3× about 0.30s; 0.78× to 2.6× tops out at 0.94s. Fixed durations are the mark of an amateur. Framing: zoom = 0.8 ÷ the target's largest normalised side, so content fills 80% and breathes. Arrive 0.15s before the action, stay 1.2s after. An isolated action under 1.2s doesn't deserve its own shot.

## 4. Joining shots (where nausea really comes from)

Nausea comes from how shots join, mainly out-in-out pumping and jumps between distant focal points. For neighbouring shots under 1.5s apart, by focal distance:

| Distance | Join |
|---|---|
| Under 0.22 (close) | **Merge**: one shot framing both targets |
| 0.22 to 0.45 | **Pan**: a slightly wider zoom, one move across |
| Over 0.45 (far, diagonal) | **Drop** the weaker shot; never two far focal points in a row |

If the gap is shorter than the transition, go straight from the current zoom to the next shot's; returning to 1× and pushing again is the pump. Only with 1.5s or more between shots may you pull out and push in again.

## 5. Openings and endings

1. **Establishing push:** a film longer than 14 seconds whose first real shot comes after 7 seconds opens with a 1.06× centre push easing back to wide over 3 seconds: "the camera is alive", not "the slideshow started".
2. **Wide closing rule:** end on a still wide shot: 0.55s pull out plus at least 0.8s still. Never stop while pushed in. Compatible with the hard stop: the held frame is wide.
3. **Move on pause:** with narration, a camera move landing mid-sentence snaps earlier to the nearest silence (at most 0.8s, only earlier, never later). Use the narration timeline's sentence gaps.

## 6. Transitions

Choose by the energy drop at the seam; one technique per seam; transition frames come out of the neighbouring shots' time. Admired launch films never cut bare.

| Technique | When | Parameters |
|---|---|---|
| Flash wash | High to high, a strong section change | White peak 2 to 4 frames, 5 frames ramp each side |
| Dip to dark | High to low, mood drops | To near-black at 90%, 0.6s total; don't linger |
| Defocus hand-off | Level topic change | Out-going blur 0 to 8px overlapping in-coming 8 to 0, 8 frames or more |
| Title card | Chapter break | Hold 1s or more, 0.3s each side; two per film at most |
| Whip pan | Low to high, energy surge | Hold 20 frames each end, 8-frame whip across 1.5 screens, fast enough to blur fully |
| Mask wipe | Through one interface into another | The mask's shape must come from the frame (a window, a card's corner); an invented shape is slop |
| Hidden cuts | Unnoticed | A white flash straddling the cut; a foreground element sweeping across; a cut at peak motion blur |
| Travel | Continuous space | A shared element moving into its place in the next shot (keep its identity); pushing through the counter of a big letter (at least a third of frame height) |

A hard cut is allowed only when wrapped in one of these.

**Parallax:** at most four layers, speed factors around 0.35 (far), 0.7 (mid), 1.4 (near), neighbours at least 2× apart (1.2× is unreadable), all derived from one camera position. **Motion-derived signals:** blur only while moving (triggered above a zoom speed of 0.6 per second, at most 10px); still frames always sharp; lagged followers (shadow 2 frames behind, ghost 4); a tiny 1 to 2 swing settle (3px at most) only after whip pans or hard stops.

## 7. Scene-based, not step-based

Demoing a workflow as "step 1 fades in, step 2 fades in", four boxes on one screen, is PowerPoint. Cut it into scenes, each a full-screen shot focusing on one thing, joined by dissolves, focus pulls and morphs. A tested shape for a workflow demo, about 22 seconds (under 18 is over before the viewer settles, over 25 loses them): Invoke 3 to 4s (the user's input) → Process 5 to 6s (the core flow in its own visual language) → Insight 4 to 5s (the key result visualised) → Output 3 to 4s (the real artefact) → Hero reveal 4 to 5s (big type, the value).

- **Each demo has its own visual language.** Two products demoed with the same template tell viewers the products are the same. Cover the text: can you tell which demo it is?
- **Real generated assets, never emoji or drawn props** (`huashu-image-briefs`).
- **Sound on two tracks** (`huashu-audio`); in a browser, start only after a play click, with a mute control.
- **Embedded in a page or deck:** the default view is a static, readable summary; play is a small secondary control, never a black screen with a big play button.

For 20 or more similar assets (screens, covers, works) showing breadth, use the **gallery ripple**: cards spread from the centre with delay by distance (about 1.7s, expo-out, while the canvas zooms 1.25 to 0.94), then four focus beats of 1.7s with 0.6s rests where the background dims, desaturates and darkens and one card enlarges to centre, over a slow drift of sine plus linear motion. Only with 20 or more consistent assets that still read when enlarged, in landscape. Not for feature demos, data or stories.

## 8. Launch films: director's notes first

For a brand film of 20 seconds or more with a thesis or slogan reveal, likely to be paused, screenshotted or compared to a named reference ("level Apple"): after the three-direction gate, write the director's notes before any composition. Thinking is writing the notes; building is translating them. Five parts:

1. **Director's statement:** what this film is not; the one-line thesis; five to eight named references with years and what each teaches; three audiences and the promise to each; the rhythm philosophy and the second of the emotional climax (often around 75 to 85% of the length, not the last second); a list of things this film will not do.
2. **Visual system:** eight to ten colours with role and maximum share; six or more type levels with face, weight, size, tracking, use; canvas, margins, grid, safe areas; up to four easings, a duration dictionary, stagger rules, scene transitions; recurring chrome with timing; the music curve and an SFX dictionary of ten or more cues; a per-shot anti-slop checklist. Every later decision derives from this; never invent values in the shot list.
3. **Story arc:** setup (first fifth), escalation (middle), payoff (last quarter), with the climax marked.
4. **Shot by shot** (about 60% of the notes): 12 to 15 shots for 30 seconds, each with eleven fields: timecode, function, visual, camera, type, animation, audio, chrome, anti-slop, why (link from the previous shot and hook to the next). Reread: if a shot can be deleted, delete it.
5. **Production manifest:** fonts, tokens, music brief, SFX list with times and levels, 12 to 15 keyframe times to verify, render settings, deliverables.

Write as a director ("Ini hero shot-nya: kalau penonton berhenti di mana pun, saya mau di sini"), cite specific works with years, trace every decision to one first principle, and reread after a break. When directions are unclear, build several versions in parallel from deliberately different directors (different cultures, decades, media), each independent, compared only at the end.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
