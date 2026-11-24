---
name: video-hyperframes
description: "Read first for any video: how you plan, write, preview and hand off a HyperFrames composition for render, the render contract, formats and sizes, the motion, look and caption rules, and what fails a render."
license: "Apache-2.0, Copyright 2026 HeyGen, Inc."
source: "https://github.com/heygen-com/hyperframes @574b5e3d3bbf526809d268a6728f9c5ce52e18bb"
---
# Video with HyperFrames

HyperFrames turns HTML into video. The renderer never plays your page: it seeks one paused timeline to each frame's time, takes a screenshot, and FFmpeg encodes the frames. The same files give the same MP4 every time. That only works when everything on screen depends on timeline time alone.

How it works in Nomi:

1. You write the composition: `index.html`, plus scene files and assets. It is saved as a design version.
2. The Design Studio plays that version live in its sandboxed iframe. The person watches it there and comments.
3. When the person says OK, the render worker runs `npx hyperframes render` (Node 22, Chromium, FFmpeg) and returns the MP4.

Read the other video skills when you need them:

| You are about to | Read |
|---|---|
| Write or change composition HTML | `video-hyperframes-composition` (the exact format and what the linter checks) |
| Bring the brand's tokens into the frame | `video-hyperframes-frame-md` |
| Hand-build a transition, title, caption style, chart, lower third or logo sting | `video-hyperframes-registry` first |
| Make a deck someone presents live | `video-hyperframes-slideshow` |

## What people ask for

| Ask | Canvas | Length | Shape |
|---|---|---|---|
| Promo for Instagram Reels or TikTok | 1080×1920 (9:16) | 10–15 s | Hook in the first second, 3–5 shots of 2–3 s, end card with the one action |
| Menu video for a cafe's TV | 1920×1080 (16:9) | 30–60 s, loops | One item or category per 5–8 s, real photos and prices, last frame matches the first |
| Product launch clip | 16:9, or 9:16 for social | 20–45 s | Hook → problem → reveal → proof → payoff → end card (`references/story-and-storyboard.md`) |
| Captions over a phone video | The footage's own size, usually 1080×1920 | The clip's length | The footage plays, captions on one track (`references/captions.md`) |
| Animated logo | 1920×1080 or 1080×1080 | 3–6 s | The mark is performed once, then holds still for at least 1.5 s |
| Feed post | 1080×1080 (1:1) or 1080×1350 (4:5) | 6–15 s | Like a Reel, but the picture can use the whole frame |

Build only what was asked. Don't add a voiceover, captions or a second aspect ratio unless the request or the material calls for them. When the person gives a length, treat it as a range and don't pad it.

## Formats and timing

- **Size:** the root's `data-width` × `data-height`. A new video is 1920×1080 unless the platform or the person says otherwise. Never squeeze a landscape layout into 9:16; recompose it vertically.
- **fps:** 30 by default. Use 24 for a film look and 60 only for fast screen motion (it doubles the render time). The worker renders at the root's `data-fps`, or 30.
- **Length:** the root's `data-duration`, in seconds. Anything past it is cut. If the content ends early, the last frame holds. Frames = seconds × fps, so a 12 s Reel at 30 fps is 360 frames.
- **Output:** MP4 (H.264). A transparent WebM or MOV is only worth it when the person will lay the clip over other video, such as a logo sting for their own edits. Ask for it in the render hand-off.

## The production loop

1. **Plan.** Write the one thing the viewer should remember, the kind of business (Taste's kinds: place, product, service, software), the canvas, the length and the sound. For anything over about 15 s that tells a story, write a short spec with a number for every adjective: GOAL, OUTPUT, TIME, LOOK, SHOTS, BANNED (`references/story-and-storyboard.md`).
2. **Storyboard.** For a film over about 15 s, or when the person should approve the shots first, list one line per shot: its time window, what is on screen, the words and the sound. Wait for their go. Skip this for a short single-idea piece such as a logo sting, captions or a one-line title.
3. **Write valid composition HTML** (`video-hyperframes-composition`). The root `index.html` mounts one scene per file from `compositions/`. Put copy, brand colours and swappable photos in variables so the person can change them in the Studio.
4. **Wire seekable animation.** Each composition builds one paused GSAP timeline, places every tween at an absolute second, uses `fromTo`, and registers the timeline last: `window.__timelines["<composition-id>"] = tl`.
5. **Add media.** Use the person's own photos, footage, logo and music from `assets/`. Show real things, such as the real dish, the real shopfront and the real product screen. Draw only what has no source, and never draw a copy of a real logo.
6. **Lint.** Go through the contract below before saving. The worker runs `npx hyperframes lint` and `check --json`. Its findings come back by rule name, with a `Fix:` line. Fix every error and weigh every warning (the rule table is in `video-hyperframes-composition/references/lint-and-runtimes.md`).
7. **Preview.** The Studio plays the version. Tell the person what plays when, in a few lines, and stop. Passing lint is not a reason to render.
8. **Render** only when the person approves the preview or asks for the file. The worker runs the render, then checks with `ffprobe` that the duration matches the root's `data-duration` and the size matches `data-width`/`data-height`.

## The render contract

Break any of these and the render goes blank, freezes, waits or comes out different each time.

1. Every timed element has `class="clip"`, an `id`, `data-start` and `data-duration` in seconds. It shows over `[start, start + duration)`, so land its end state a little before its end.
2. The video's length is the root's `data-duration`. Raise it when a clip ends later.
3. Each composition builds one paused timeline and registers it after its tweens, under its own `data-composition-id`. Never call `play()`.
4. Use no `Date.now()`, no unseeded `Math.random()`, no `requestAnimationFrame` clock, no timers, no fetches while rendering and no `repeat: -1`. Everything visible is a function of timeline time. Seed any randomness with a small PRNG.
5. Never tween `display`, `visibility` or `autoAlpha` on a `.clip`. Animate a child, or the clip's `opacity`.
6. Never put a CSS `transform` on an element that GSAP transforms. Animate transforms (`x`, `y`, `scale`), not `left`, `top`, `width` or `height`. Use no CSS `transition`, because it runs on the browser clock.
7. Never `play()`, `pause()` or seek media. A `<video>` keeps its sound with `data-has-audio="true"`, otherwise it is `muted`. Music and voice are separate `<audio>` elements, each with an `id`. Don't add `crossorigin`.
8. Asset paths are relative, `assets/...` with no leading `/`, in scene files too. Use no base64 media and no placeholder URLs.
9. The bundled fonts resolve by name. Any other family needs `@font-face` pointing at a file in `assets/fonts/`.
10. A scene file puts its styles, markup and script inside `<template>`, and styles its root as `#root`. The host, the scene root and the timeline key share one id.
11. Keep content inside the 90% action-safe box, and words inside the 80% title-safe box. On 9:16 social, words stay in the band every app leaves clear (`video-hyperframes-frame-md`).
12. A composition file stays under 16 MB. Keep images, fonts and data as separate files.

## Motion in short

The full numbers are in `references/motion.md`.

- Arrivals use a long ease-out (`power4.out`, `expo.out`) and take 0.4–0.8 s. Exits are 40–50% shorter and accelerate (`power3.in`). Camera moves ease both ways (`power2.inOut`).
- Give each shot one hero move. Start the next element when the previous one is 30–50% done. Stagger words 60–120 ms apart.
- A landed title holds dead still. A line the viewer must read holds 0.5 s plus 0.3 s a word, and the last line holds at least 2 s.
- Use 3–5 kinds of transition in a film, each caused by something on screen. Don't fade and slide up on every element.
- Punch-ins, pans and Ken Burns moves go on a wrapper inside the clip, never on the timed `<video>` or `<img>` itself.

## Look in short

- Take the look from the brand: read its `frame.md` (`video-hyperframes-frame-md`). Colours and fonts are strict. Sizes, spacing and border weights are yours to scale for the frame.
- Taste's kinds still decide the look. A cafe, warung, bakery, shop or service gets photographs, flat colour and type. It never gets shader effects, glow, particles or a dark void. Software and tech may carry one effect, with a reason.
- Avoid these by default: a dark void with small things floating in it, glow and lens flares, purple or blue gradients, glass cards, everything centred, everything fading in at once, and a logo docked small in a corner.
- On video, full-screen linear gradients on dark grounds band, so use a solid colour or a radial glow. An accent at 5% opacity disappears in compression; use 15–25% for atmosphere.
- Real numbers only: prices, hours and claims come from the brief. Write prices as "Rp 25.000".

## Captions in short

The details are in `references/captions.md`.

- Transcribe in the speaker's language: `npx hyperframes transcribe <file> --model small --language id`. The default English model **translates** Indonesian speech into English.
- Show 2–5 words per group, one group at a time, and end each group with a hard hide. All groups sit on one captions track.
- On 1080×1920, the bottom of the caption stays at or above y 1248, and captions never cover the speaker's face.

## Words on screen

- Titles have at most 6 words a line, and one line shows at a time. Indonesian words run long, so on 9:16 budget 4–5 words a line. When copy doesn't fit, cut words; never shrink the type below the floor.
- Use the person's own words. Don't open on "Introducing" or "Perkenalkan" with nothing else on screen. Avoid hype like "terbaik", "dijamin" and "#1" unless the brief proves it. Never invent a phone number, a review or a statistic.
- The call to action names the real action: "Pesan lewat WhatsApp", "Mampir ke Jl. Kaliurang", "Cek menu di bio".

## What fails a render, and how to avoid it

| What goes wrong | Usual cause | Avoid it by |
|---|---|---|
| A scene is blank or frozen | Timeline registered before an async build finished, `visibility` tweened on a `.clip`, or host id ≠ scene id | Register the timeline last. Animate children. Use the same id three times. |
| The render waits 45 s per scene, then shows still frames | A composition that never registers a timeline | Add `data-no-timeline` to a still composition |
| The video ends in black | Captions or cards run past the footage's real length | Clamp ends and the root duration to the media length |
| The video is cut short | Root `data-duration` is too small | Raise it to cover the last clip |
| Text in the wrong font, or reflowed | A family that isn't bundled and has no `@font-face` | Ship the `.woff2` in `assets/fonts/` |
| Each render looks different | `Math.random`, `Date.now`, CSS transitions, `repeat: -1` | Use a seeded PRNG and finite repeats on the timeline |
| A photo or clip is missing | A leading `/` in the path, `crossorigin`, base64, a placeholder URL or a dead link | Use `assets/...` paths to files saved in the version |
| Footage is silent | A `<video>` with neither `data-has-audio="true"` nor `muted` | Set one of them on purpose |
| Footage stutters or freezes on seek | Phone video with keyframes more than ~1 s apart | Have it re-encoded with a keyframe every second (`-g 30`) |
| A deck renders as six seconds | A slideshow sent to `render` | Never render a deck to MP4 (`video-hyperframes-slideshow`) |
| A 3D or glass block is blank | WebGPU blocks need a browser the worker doesn't run | Skip blocks tagged `webgpu` (`video-hyperframes-registry`) |
| Render refused | Composition file over 16 MB | Split into scenes and keep assets as files |

## References

- `references/motion.md`: eases, durations, titles, holds, transitions with a cause, pacing for Reels, mechanisms, and what to change when the person says "too fast".
- `references/captions.md`: transcripts, groups, timing, placement and blend over footage, cards and lower thirds over a talking head.
- `references/story-and-storyboard.md`: the one thing, hooks, beat shares, the spec, the storyboard, sound, and the review pass before you hand off.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
