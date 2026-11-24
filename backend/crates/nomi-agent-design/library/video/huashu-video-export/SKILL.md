---
name: huashu-video-export
description: "When and how a video composition becomes MP4 and GIF: the formats to deliver, the two-stage render, the HyperFrames choices, what each export script does, the pre-flight list, hard checks on the rendered file and the usual follow-up requests. Read it before rendering or delivering any video."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Video export and verification

When and how a video composition becomes MP4 and GIF: the formats to deliver, the two-stage render, the HyperFrames choices, what each export script does, the pre-flight list, hard checks on the rendered file and the usual follow-up requests.

The composition contract (structure, timeline registry, check gates) is in `video-hyperframes` and `video-hyperframes-composition`; this file adds huashu's export judgement and checks. HyperFrames is only the renderer: the narrative, easing and two-track audio rules (`huashu-animation`, `huashu-audio`) always lead.

## When to export

Only when the animation runs through completely, the key moments are checked as stills, and the person has seen it once and is happy. Never export with known bugs: changes after rendering cost more.

## What to deliver

| Format | Spec | For | Typical size (30s) |
|---|---|---|---|
| MP4 | 1920 × 1080, H.264, high quality (CRF about 18) | Instagram, WhatsApp status, YouTube, websites | 1 to 2 MB |
| MP4 60fps | Same, true 60fps from frame-by-frame rendering | Portfolios, platforms that favour 60fps | 1.5 to 3 MB |
| GIF | 960 wide, 15fps, two-pass palette | Chat previews, READMEs | 2 to 4 MB |
| Vertical | 1080 × 1920 (re-laid out, not cropped) | Reels, TikTok, Shorts, status | |

A GIF holds 256 colours: scan the whole clip for its own palette first, then encode with it, updating only changed areas; one pass muddies subtle palettes. Dithering keeps fades smooth at a slightly larger size.

## Render in two stages

1. **Quick render at 30fps**; look at it and at the acceptance frames (`huashu-cinematic` storyboard).
2. **Final render at 60fps** only after that passes (600 frames of 1080p take about 20 seconds).

The composition check must pass with zero errors before any render. The only allowed exception: the contrast gate on dark cinematic designs whose low-contrast watermarks and labels are intentional. Light informational videos keep it; there a contrast error is a real readability problem.

**Transparent overlays** (lower thirds, captions, stickers to lay over other footage): render with a transparent ground (ProRes 4444 MOV for editing, large; WebM with alpha for the web; PNG frames for editors). No ground colour in the composition.

**Narration-driven explainers** use the separate narration pipeline (voice generated per section with measured timings, then mixed), not the plain composition render.

## What each script does (run later in Nomi's render worker)

- `render-video.js`: records an HTML animation to MP4 in a real browser at 25fps. A throwaway warm-up pass caches fonts and assets; a fresh recording pass starts at time zero after the page signals ready, trims the loading frames, and hides human-only chrome (progress bars, replay buttons) automatically. Use it for animations that can't be seeked frame by frame.
- `render-video-seek.js`: seeks frame by frame and captures each, then encodes: true 60fps, deterministic, no black frames at the start. For animations driven by a seekable clock.
- `convert-formats.sh`: makes a 60fps MP4 by frame duplication (plays everywhere) and a palette-optimised GIF. Motion-interpolated 60fps is opt-in and must be tested in the target player first: interpolated files have failed to open in some players.
- `add-music.sh`: lays a music bed under a silent MP4, trimmed to length, 0.3s fade in, 1s fade out, video copied without re-encoding.
- `sfx-cues.sh`: places effects from a cue table (second, file, level) onto a video; edit the table, rerun.
- `verify-video.sh`: hard checks on the rendered file (below); a non-zero exit means don't deliver.

## Pre-flight

- Plays through with no errors.
- Frame 0 is the complete initial state, not blank or loading; the last frame is a settled final state, not a loop back to the start or a fade to black.
- Fonts, images and symbols all render (no boxes).
- The duration in the export matches the composition.
- No human-only chrome in the picture (progress bars, timecodes, replay buttons).
- Sound: SFX and music present unless the person declined (`huashu-audio`).
- Brand film for a named brand: the asset protocol was followed (real logo, real product, real UI).

## Hard checks on the rendered file

Don't eyeball a final render; measure it:

| Check | Pass |
|---|---|
| Resolution and frame rate | As specified |
| Duration | Within 2% of the plan |
| Audio stream | Present (no audio track = half finished), unless a deliberate silent intermediate |
| Black frames at start and end | None (they reveal a late recording start or a loop jump), unless a deliberate dark opening |
| Loudness | Around -14 LUFS (±4) for finished pieces |
| Size | Reasonable for the format |

Then extract and look at frame 0, the last frame, and the first frame after each scene change. An AI video review, if available, can watch the final render against the director's notes (black or broken frames, cut-off text, overlaps, story continuity, the hero carried through, dead stretches, cue timing, empty composition); treat its findings as opinions to confirm frame by frame. It cannot hear the audio, misses one- or two-frame glitches, and may call a fast cross-fade a hard cut.

## Telling the person

Short: what was delivered, its length and formats, the caveats, the next step. "Video 22 detik siap: MP4 (1080p dan 60fps) dan GIF untuk WhatsApp. Musik dan efek suara sudah ada. Catatan: foto produk di detik 6 masih foto sementara."

## Common follow-ups

| They say | Do |
|---|---|
| "Terlalu besar" | MP4: raise CRF to 23 to 28. GIF: 600 wide or 10fps |
| "GIF-nya buram" | 1280 wide, or suggest MP4 (most chat apps play it) |
| "Mau versi vertikal" | Re-lay out the composition at 1080 × 1920 and render again; never crop the landscape |
| "Latarnya transparan" | MP4 can't; deliver WebM with alpha or MOV |
| "Tambah logo di pojok" | Add it in the composition, on the overlay layer, colour adapted per scene |
| "Tanpa kompresi" | Lossless settings, about ten times larger |

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
