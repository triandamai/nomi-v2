# Motion: eases, titles, transitions and pacing

These numbers were measured frame by frame on HeyGen's own launch films. They are defaults. If the person gave a reference video, or a beat does something else on purpose, follow that for that beat.

## Moves

- **Duration:** a fast move takes 0.3–0.6 s, a calm arrival 0.6–0.9 s. One move is one tween.
- **Arrive slow, leave clean:** arrivals end on a long tail, and landing takes longer than leaving. An exit is one smooth motion. Never slide, stop, then leave. If entrances look like glitches, slowing the whole video by about a second often fixes it.
- **Ease by role:**

| Role | Ease |
|---|---|
| An element arriving | `power4.out`, `expo.out` |
| A camera move, a colour flood | `power2.inOut` |
| An exit, or a zoom through | `power3.in` |
| Rotation, a progress bar, constant speed that is the point | `none` (linear) |

  Using the same `power2.out` at 0.5 s on everything is the flat default that makes a video feel generated.
- **Springs** are named by damping ratio. 1.0 never bounces, 0.7–0.85 reads premium, 0.4–0.6 playful, and under 0.3 looks like a cartoon. Position and scale may bounce; opacity and colour never do. The closest built-ins are `back.out(1.4)` and `elastic.out(1, 0.5)`.
- **Overlap and stagger:** start the next element when the previous one is 30–50% done. Letters go a frame apart, words 60–120 ms apart, list items 40–80 ms apart.
- **Motion blur on fast moves:** 6–12 px on text-sized elements and 15–20 px full-frame, peaking at the fastest frame. Use horizontal blur only on a horizontal move. For many fast 2D movers, `data-motion-blur="on"` on the root adds blur at render time. It costs render time, so use it only when the person wants a filmic look.

## Titles

- A landed title is **dead still**. The motion moves on to the next element, the camera or the world behind it.
- Vary the entrance within a film. Don't fade and slide up every title. Choices:
  - per word, `blur(6px)` → 0 and opacity 0 → 1 in 0.13 s, 0.067 s apart, with no translate;
  - per word, from light grey to ink, 0.07–0.1 s apart;
  - the whole line blurring in, 8 px → 0 in 0.13–0.2 s;
  - per letter for a name, 0.1 s apart, opacity 0.3 → 1;
  - one word rolling through options like a slot machine, about 0.1 s a roll ("Kopi / Teh / Cokelat").
- **Holds:** a line the viewer must read holds 0.5 s plus 0.3 s a word. The film's last line holds at least 2 s. Exit with a blur-out and fade of 0.13–0.25 s.
- **Size:** a title is 4–9% of the frame height, one or two lines. Quiet lines are 2.5–4%, against hero words that fill 60–90% of the width. Highlight one word in the accent colour, and only when that word carries the meaning.
- **Counters:** a count-up runs 1.2–1.6 s, then holds, using `font-variant-numeric: tabular-nums` so the digits don't jitter. Never count up a price; show it.

## Rhythm and pacing

- Contrast speeds. Montages cut every 0.13–0.8 s, a demo holds one framing for 7–10 s while the UI works, and shots average 1.8–3 s with motion inside them.
- Hold 0.3–0.75 s before the payoff, and play the payoff at normal speed.
- Show something new every 2–4 s.
- **9:16 social** runs faster than landscape. Use shots of 1–2 s and a new beat every 2–4 s, and put the payoff before the viewer swipes. Frame 0 is the thumbnail: a face, the dish or one bold line, never black and never a logo alone.
- **A cafe TV menu** runs slower. Each item holds 5–8 s with one gentle move (a 3–5% push on the photo). There is no hard cut every second, and the last frame matches the first so the loop is invisible.
- **Music:** frames per beat = fps × 60 / BPM. Compute each beat from its index and round it; never keep adding a rounded beat. Cut on the beat and change the layout on the bar.
- **Cuts:** cut at the fastest frame of a move, with the same direction and speed on both sides.

## Transitions with a cause

Use 3–5 kinds in a film, each caused by something on screen. At most a third of the seams between beats are hard cuts. Two worlds never crossfade into each other.

| Transition | How it is built |
|---|---|
| Object becomes the world | A card or photo grows past the frame in about 0.65 s (`power2.inOut`). What was around it blurs about 15 px and fades in 0.3 s, and the next scene is inside it. |
| Zoom through | The camera tracks to the item (0.6 s), then the item scales from about 8% to 250% of the frame in 0.35 s with a strong ease-in. Its fill becomes the next frame. |
| Colour flood | The hero object stays while the whole ground changes colour in about 0.4 s (`power2.inOut`) |
| Whip | A 0.13–0.3 s horizontal smear. The arriving element lands blurred and sharpens in about 0.13 s. |
| Iris | `clip-path: circle()` opens from a point onto the next scene in about 0.15 s |
| Blur swap on one stage | Out: opacity → 0 with about 8 px blur in 0.13–0.2 s. In: blur → 0 in 0.13 s. The ground never cuts. |
| Dip cut | The outgoing frame darkens about 40% for 3 frames before a hard cut, so the cut reads as meant |

Ready-made versions are in the registry (`video-hyperframes-registry`). Use at most 2 shader transitions in a video.

## The right mechanism

| The ask | Mechanism |
|---|---|
| Punch-in, zoom, pan, Ken Burns on a photo | Tween `scale` and `x`/`y` on a wrapper inside the clip, never on the timed media element |
| Crop or mask reveal | Tween `clip-path` or a mask on an inner wrapper |
| Footage at a constant speed | `data-playback-rate` on the media, 0.1–10 |
| A speed ramp | A `rate` lane in `data-automation` on the media |
| A freeze mid-clip | Export a still and place it as its own clip |

## When the person says

| They say | You change |
|---|---|
| "Terlalu cepat" (too fast) | Slow every zoom and pan to 0.7× its speed, nothing else |
| "Percepat" (speed it up) | Shorten the content and the beat by the same amount, and move later beats up |
| "Tulisannya goyang" (the text moves) | Land the titles, then hold them dead still until they leave |
| "Kayak slide" (feels like slides) | Enter the next scene through an object on screen, or cut a move mid-flight |
| "Terlalu ramai" (too busy) | Remove a third of the motion and keep the moves that lead the eye |

## Defaults that look generated

- breathing, floating or pulsing loops standing in for motion on titles and UI;
- titles that keep sliding or scaling after they land, or a slow creeping push late in a scene;
- bounce on UI, click rings, and every scene entering the same way;
- every element fading and sliding up;
- the same whoosh on every cut.

## Common faults

| Symptom | Usual cause |
|---|---|
| Stutter or drift between renders | Layout measured inside `onUpdate`, `requestAnimationFrame` or clock time, or two timelines on one property |
| A jump at a cut | The outgoing and incoming states differ at the seam |
| An ease looks wrong at a hand-off | The speeds don't match. Mirror the ease: `power3.in` out, `power3.out` in. |
| Something drifts after seeking back | `from()` used inside a scene. Use `fromTo()`. |

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
