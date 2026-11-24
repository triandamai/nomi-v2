---
name: video-hyperframes-composition
description: "The exact HyperFrames composition format you write for a video: files, required structure, data attributes, the timeline registry and seek contract, variables for tokens, media and fonts, and what the linter checks. Read it before writing or editing any video HTML."
license: "Apache-2.0, Copyright 2026 HeyGen, Inc."
source: "https://github.com/heygen-com/hyperframes @574b5e3d3bbf526809d268a6728f9c5ce52e18bb"
---
# The composition format

A composition is a normal HTML document. Data attributes declare the canvas, what shows when, and the scene files. A paused GSAP timeline, registered on `window`, holds the motion. The renderer seeks that timeline to `frame / fps` for every frame, so whatever you write must look right when it is seeked straight to any time, in any order.

## Files in a video version

```text
index.html                 the root: declares variables, mounts scenes, media and audio
compositions/intro.html    one scene per file, everything inside <template>
compositions/menu.html
assets/                    photos, footage, music, logo (svg/png)
assets/fonts/              .woff2 files for any font that isn't bundled
frame.md                   the brand translated for the frame (video-hyperframes-frame-md)
```

A very short piece, such as a logo sting or a single title, may live in `index.html` alone. More than 3 timed elements in one file triggers a density warning (`timeline_track_too_dense`), so give each chapter its own scene file.

## The root: `index.html`

```html
<!doctype html>
<html lang="id" data-composition-variables='[
  {"id":"color-primary","type":"color","label":"Primary","default":"#1F6F5C"},
  {"id":"headline","type":"string","label":"Headline","default":"Es Kopi Pandan"}]'>
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=1080, height=1920" />
  <script src="https://cdn.jsdelivr.net/npm/gsap@3.14.2/dist/gsap.min.js"></script>
  <style>html, body { margin: 0; width: 1080px; height: 1920px; overflow: hidden; }</style>
</head>
<body>
  <div id="root" data-composition-id="main" data-start="0" data-duration="12"
       data-width="1080" data-height="1920" data-fps="30">
    <div id="intro" data-composition-id="intro" data-composition-src="compositions/intro.html"
         data-start="0" data-duration="4" data-track-index="1" data-track-kind="graphics"
         data-width="1080" data-height="1920"></div>
  </div>
  <script>
    window.__timelines = window.__timelines || {};
    window.__timelines["main"] = gsap.timeline({ paused: true });
  </script>
</body>
</html>
```

- `<!doctype html>`, `<html>` and `<body>` are required. A bare fragment fails `root_composition_missing_html_wrapper`.
- The root carries `data-composition-id`, `data-start="0"`, `data-width`, `data-height` and `data-duration`. The `data-duration` value is the video's length; scripts and variables can't change it.
- Keep `<meta viewport>` and the `html, body` size equal to the root's size, or lint warns `root_dimensions_mismatch`.
- Load GSAP with one pinned `<script>` before any animation script. Lint fails `missing_gsap_script` without it.
- The root holds only scene hosts, media and audio.

## A scene: `compositions/<id>.html`

```html
<template>
  <style>
    #root { position: absolute; inset: 0; background: var(--color-background); }
    #intro-title { font: 800 120px/1 var(--font-heading); color: var(--color-foreground); }
  </style>
  <div id="root" data-composition-id="intro" data-width="1080" data-height="1920">
    <h1 id="intro-title" class="clip" data-start="0.3" data-duration="3.4"
        data-var-text="headline">Es Kopi Pandan</h1>
  </div>
  <script>
    const tl = gsap.timeline({ paused: true });
    tl.fromTo("#intro-title", { opacity: 0, filter: "blur(8px)" },
      { opacity: 1, filter: "blur(0px)", duration: 0.2, ease: "power4.out" }, 0.3);
    window.__timelines["intro"] = tl;
  </script>
</template>
```

- Styles, markup and script all sit **inside** `<template>`. Anything outside it is dropped. Only scene files use `<template>`; the root never does.
- The host's id, the scene root's `data-composition-id` and the timeline key are the **same id**. If the host id differs, each render waits 45 s for that scene and then shows still frames.
- Style the scene root as `#root`, sized with `inset: 0`, never with fixed pixels or a class.
- Keep ids unique across the assembled page by prefixing them (`#intro-title`, `#menu-item-1`). A scene's timeline can't reach elements outside its own file.
- A scene's clock starts at 0 at its host's `data-start`. The host's `data-duration` is the scene's slot: a shorter timeline holds its last frame, and the scene goes blank when its slot ends.

## The timeline and seek contract

- One `gsap.timeline({ paused: true })` per composition. Add every tween at an **absolute** second (`tl.to(el, {…}, 2.4)`), then register it last: `window.__timelines["<id>"] = tl`.
- Use `fromTo`, not `from`, inside scenes. The host re-seeks the scene every time it shows, and `from` records its start state only once.
- Never `tl.play()`. Never `add()` a scene's timeline to the root's, because the runtime drives each scene and nesting seeks it twice.
- If you build inside `document.fonts.ready.then(…)`, register at the end of that callback. Registering first renders blank (`gsap_timeline_registered_before_async_build`).
- Measure layout once before building tweens, never inside `onUpdate`. Two timelines must never animate the same property of one element.
- Repeats are finite: `repeat: Math.max(0, Math.floor(duration / cycle) - 1)`. Randomness uses a seeded PRNG (mulberry32).
- Transforms on a `<span>` need `display: inline-block`.
- A still composition, with no motion, gets `data-no-timeline` on its root, or every render waits 45 s.
- Canvas, Three.js, Lottie, CSS keyframes and other runtimes have their own rules (`references/lint-and-runtimes.md`).

## Attributes

| Attribute | Meaning |
|---|---|
| `data-start`, `data-duration` | Seconds, local to the file. `data-start="intro + 0.5"` starts 0.5 s after `intro` ends (the spaces are required, and a typo silently resolves to 0). |
| `class="clip"` | On every timed DOM element and image |
| `data-track-index` | The Studio lane only. Paint order is CSS `z-index`. Clips that never overlap share a lane. |
| `data-track-kind` | `graphics` on scene and overlay hosts. `captions` on the one host that holds every caption group. |
| `data-composition-src` | Mounts a scene or registry block file |
| `data-variable-values` | A JSON object that sets one mount's variables |
| `data-fps` | On the root: the render frame rate (30 if absent) |
| `data-media-start` | Offset into the source media (a trim) |
| `data-playback-rate` | A constant speed, 0.1–10 |
| `data-has-audio="true"` | The `<video>` keeps its own sound |
| `data-volume`, `data-fade-in`, `data-fade-out` | Gain (1 = 0 dB) and edge fades on media |
| `data-hidden` | Hidden in preview and render, reversibly |
| `data-no-timeline` | A root or host with no timeline to wait for |
| `data-motion-blur="on"` | On the root: render-time motion blur for fast movers |

Layout-audit opt-outs: use the narrowest one, on the smallest element. `data-layout-allow-overflow` covers decorative bleed. `data-layout-bleed="true"` covers one intended crop of primary text. `data-layout-allow-overlap` covers deliberate text layering. `data-layout-allow-occlusion` covers an element meant to cover text. `data-layout-ignore` marks decoration that is never audited.

## Variables: where your tokens go

Declare an array of `{id, type, label, default}` on `<html>` with `data-composition-variables`. The types are `string`, `number`, `color`, `boolean` and `enum`. Each id becomes a CSS custom property, so Nomi's tokens map one to one:

| Your token | Variable id | In CSS |
|---|---|---|
| `$color-primary`, `$color-background`, `$color-foreground`, `$color-muted-foreground`… | `color-primary`… (type `color`) | `var(--color-primary)` |
| `$font-heading`, `$font-body` | `font-heading`, `font-body` (type `string`) | `font-family: var(--font-heading)` |
| `$space-n`, `$radius-*` | Scale them for the frame first (`video-hyperframes-frame-md`). Declare them as `number` only if the person should edit them. Otherwise write them as constants in `:root`. | `var(--space-4)` |

- Bind text and media without script: `data-var-text="headline"`, `data-var-src="hero-photo"`. Keep a real fallback `src` on media.
- For logic, read `window.__hyperframes.getVariables()` once at init.
- Copy, prices, brand colours and swappable photos should all be variables, so the person can edit them in the Studio's Variables panel.
- A variable's id must exist before you bind to it (`unknown_variable_binding`).

## Fonts

- **Bundled, so they resolve by name:** Inter, Montserrat, Outfit, Nunito, Oswald, League Gothic, Archivo Black, Space Mono, IBM Plex Mono, JetBrains Mono, EB Garamond, Playfair Display, Source Code Pro, Noto Sans JP, Roboto, Open Sans, Lato, Poppins. Indonesian uses Latin script, so all of these cover it.
- **Any other family** needs `@font-face { font-family: "Fraunces"; src: url("assets/fonts/fraunces-700.woff2"); font-weight: 700; }` in the file that uses it, or lint fails `font_family_without_font_face`. A `local()`-only face doesn't exist on the render worker.
- System fonts (Helvetica, Arial and the like) are swapped for the nearest bundled family, so text can reflow. Name a real family.

## Media

- Use `id`, `class="clip"`, `data-start` and `data-track-index` on every clip. A `<video>` with sound gets `playsinline data-has-audio="true"`; silent footage gets `muted playsinline`. Every `<audio>` has an `id` and `data-volume`.
- Paths are `assets/...`, relative, with no leading `/`. Use no `crossorigin`, no base64, no placeholder URL and no `preload="none"`.
- A full-frame photo or video is `position: absolute; left: 0; top: 0`, the canvas's width and height, with `object-fit: cover` for a photo meant to fill and `contain` for a screenshot or anything with readable text.
- A `<video>` or `<audio>` without `data-duration` plays its full length. An `<img>` without one lasts 3 s.
- Time the `<video>` itself, never its wrapper (`video_nested_in_timed_element`). Never call `play()`, `pause()` or set `currentTime`.
- Grades, grain and blur on footage are done by the worker's media treatment, never with CSS filters on the video.
- Raise the root's `data-duration` to cover a new clip's end.

## Timing rules that bite

- A clip shows over `[start, start + duration)`, so an end state landing exactly on its end is never rendered.
- A clip nested inside another timed element counts `data-start` from its composition, and shows only while its parent shows.
- An untimed full-bleed background needs `position: absolute; inset: 0`, or it collapses.
- Put a background fill on a full-bleed child, not on the composition root, when the video has shader transitions.

## Before you save

1. Doctype, `<html>`, `<body>`; the root has id, size, `data-start="0"` and `data-duration`.
2. Every host id = its scene's `data-composition-id` = its timeline key.
3. Every timed element has `id`, `class="clip"`, `data-start` and `data-duration`.
4. GSAP is loaded once. Each timeline is paused, built with absolute times and `fromTo`, and registered last.
5. No clock, randomness, CSS transition, infinite repeat or media control.
6. Fonts are bundled or have `@font-face`. Assets are `assets/...` files.
7. Words sit inside title-safe, at or above the size floor (`video-hyperframes-frame-md`).

The linter's rules and fixes, and the other animation runtimes, are in `references/lint-and-runtimes.md`.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
