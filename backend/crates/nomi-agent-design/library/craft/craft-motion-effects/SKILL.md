---
name: craft-motion-effects
description: "The catalogue of bigger motion effects (text reveals, cursor and hover tricks, scroll-driven scenes, generated motion assets, art-styled animation, chat and quote reveals) with where Taste allows each, how to draw its still, and how to note it. Read it before dealing a moment or an effect."
license: "MIT (Motion.Lab, yangxu128 and contributors; no copyright line upstream); MIT, Copyright (c) 2026 Lin Zhihuang (oil-motion); MIT, Copyright (c) 2026 alchaincyf (huashu-art-motion); MIT, Copyright (c) 2026 GreenSock (gsap-plugins); Apache-2.0, nexu-io/html-anything (vfx-text-cursor); chat-motion-overlay is open-design's own; via open-design (Apache-2.0)"
source: "https://github.com/yangxu128/Motion.Lab @fbfda03; https://github.com/oil-oil/oil-motion @07e31f2; https://github.com/alchaincyf/huashu-art-motion @d861767; https://github.com/greensock/gsap-skills @aed9cfd; https://github.com/nexu-io/html-anything; open-design@5fdcc3d"
---
# Motion effects: what exists, where it may go

The bigger effects you will be asked for, sorted by whether Taste lets this page carry them, with the still you draw and the note you write for each.

Small interface motion is `craft-motion-notes`. This skill covers the showy end: effect libraries list 160 of them, and almost all are wrong for a bakery. Taste's budget comes first (`taste-photos-motion`): every page gets the hero entrance, one or two dealt moments and the hover layer; only software, gaming and tech hardware may add one effect or scene.

## The gate

Before offering any effect, answer three questions in the plan:
1. **Kind:** is this a tech page (software, gaming, hardware)? If not, only the photo- and type-led moments in `taste-photos-motion` exist.
2. **Job:** what does the viewer understand because it moves? "It looks premium" is not a job.
3. **Budget:** is another moving thing on the same screen? Never two; a calm section between beats; the back third of the page calm.

## Catalogue by family

| Family | Examples | Taste |
|---|---|---|
| Basic entrances | fade, fade-up, slide, scale-in | Hero entrance only; never scroll fade-ins on sections |
| Emphasis loops | pulse, heartbeat, shake, jello, swing, flash | Status pulse on a live indicator (tech); a single error shake; nothing decorative |
| Text reveals | typewriter, split characters, mask reveal, word highlight, scramble, letters assemble, count-up | `word-highlight` on any kind; letters-assemble, counter-rolls on tech only; scramble and glitch never on other kinds |
| Text decoration | gradient, rainbow, 3D, shadow shift, outlined-to-fill | Never: Taste bans gradient text; type stays live and flat |
| Hover and cursor | lift, underline draw, icon nudge | The hover layer, every kind |
| | magnetic button, 3D tilt, spotlight border, cursor trail, blob cursor, image distort, click burst | Never: not in Taste's hover layer; custom cursors banned |
| Scroll | reveal-once, pinned chapters, photo chapters, line carries colour, giant type pass, iris opens | Only as one of the page's dealt moments, per the kind table in `taste-photos-motion` |
| | parallax, horizontal scroll hijack, card stacking, zoom on scroll, smooth-scroll inertia | Never |
| Ambient and generative | particles, starfield, flow field, sine waves, confetti, shader ground | Tech only, as the page's one effect; confetti only as an in-app success moment, once |
| Generated motion assets | an object that rotates, opens or transforms as you scroll; a product that assembles | Tech only, or a product page when the brief supplies the footage; see below |
| Art-styled animation | ink, woodcut, paper cut-out, painterly, pixel | An explainer or campaign film the person asked for; never a website ground |
| Quote and chat reveals | typed quote with a cursor and light flares; chat bubbles arriving one by one | Video intros and social clips; on a page only as tech's effect or the word-highlight moment |

## How to draw each on the canvas

The canvas shows the finished, resting state; the note carries the motion.

- **Text reveals:** draw the text complete in its final ink. Note: `Moment · word-highlight in tentang-kami · words light from muted to ink as the statement scrolls through the middle third · reduced: full ink`.
- **Typed quote (video intro):** draw the final frame: the quote complete, the cursor bar after the last word in the one accent. Note the per-character interval and that the cursor blinks only while typing. Reduced: show the whole quote.
- **Counters:** draw the final number. Note the start value (zero only if zero is honest), that it runs once on entry, and tabular figures so the width never jumps.
- **Pinned or photo chapters:** draw the chapters side by side as Taste describes; note pin, order and the phone fallback.
- **Ambient or shader effect:** draw its poster: the effect frozen at a calm, good-looking moment in flat palette colours (shapes, lines, dots as paths and ellipses). Note it by name, where it sits, its loop length, the pause button if it runs over 5 seconds, and that it stops off screen.
- **Chat reveal:** draw the full conversation as it ends, bubbles in reading order, one beat per message; note the interval between bubbles, typing indicator yes or no, and which side is the business.

## Generated motion assets (from oil-motion)

Some motion can't be made by moving flat elements: a coffee grinder opening to show its burrs, a shoe rotating, a box unfolding. These are generated or filmed clips that the page plays or scrubs. Plan them like this:

1. **Separate what the asset must do from what the page does.** Moving, scaling, cropping and timing the whole picture are the page's job; joints, materials, contact and occlusion changing are the asset's job. Only the second needs a generated clip.
2. **Choose how the person drives it:**
   - *scrub*: the scroll or drag position is the clock; stop scrolling and the frame stops.
   - *segment play*: an input picks the next state and that segment plays; reversing plays back from the current frame, never a hard cut.
   - *autonomous*: it plays on its own; input only starts, pauses or switches.
3. **Lock the ratio from the real container** (16:9 for a wide hero, 9:16 for a phone-height panel, 1:1 for a card) before anything is made.
4. **Keyframes, one change each:** K0 → K1 → K2, every segment carrying one meaningful change. Draw the keyframes as a row of frames, each a photo placeholder with its note: `Keyframe · K1 · grinder lid lifted, burrs visible, same light as K0`.
5. **Background:** say whether the clip owns its background (lighting and shadows continuous) or sits transparent on the page ground.
6. **One pilot segment first,** placed on the real page, before the rest is produced.

Never fake an asset that doesn't exist; when the brief has no footage and no tool can produce it, use photographs.

## Art-styled animation (from huashu-art-motion)

For an explainer or campaign film in an art style:
- **Pick one visual language** that fits the subject (ink wash, woodcut, paper cut-out, shadow puppet, Bauhaus geometry, pixel art, a clean keynote-style UI explainer) and keep its marks, palette, lighting and edges consistent across every shot.
- **The subject explains itself:** a concrete object acting out the idea beats a sequence of title cards.
- **Shot list:** time, what's in the frame, the action, the camera, words on screen, how it's made.
- **One frame first:** lock the key composition as a still before any movement; this is the poster.
- **Animate inside the world,** not only the camera across still paintings; seed any texture once so it doesn't flicker.
- **Wayang and batik motifs** are culturally loaded: use them only when the person's brand is rooted in them, and never as generic "Indonesian" decoration.

## Quote, title and chat clips (video frames)

For social clips and video intros the person asks for (16:9 or 9:16):
- One idea per frame; the text is the person's real words.
- One accent colour; dark grounds are fine here because it is a video frame, not their website.
- Leave safe margins for platform chrome (captions, buttons) and keep the focal text out of them.
- Reveal timing is not reading time: hold the finished frame long enough to read twice.

## Notes for Koda

Each effect note names the effect, its section, trigger, timing source (`motion-timing` or the beat map), the poster frame id, the reduced-motion still, and the performance guard (pauses off screen, one effect per page). Effects never block taps, scrolling or reading.

Adapted from Motion.Lab (MIT, yangxu128 and contributors), oil-motion (MIT, Copyright (c) 2026 Lin Zhihuang), huashu-art-motion (MIT, Copyright (c) 2026 alchaincyf), GreenSock's gsap-skills (MIT, Copyright (c) 2026 GreenSock), html-anything's vfx-text-cursor (Apache-2.0) and open-design's chat-motion-overlay, via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
