# Captions and cards over footage

These rules cover burned-in captions over a phone video, and cards, lower thirds and split layouts over a person speaking. The placement numbers are for 1920×1080 and 1080×1920; scale them for other sizes.

## Words and timings

- The worker transcribes with `npx hyperframes transcribe <file> --json`. It writes `assets/transcript.json` with every word's start and end, as `[{ "id": "w0", "text": "Halo", "start": 0.0, "end": 0.42 }, …]`.
- **Name the language.** The default model, `small.en`, **translates** non-English speech into English. For Indonesian, use `--model small --language id`. For speech over loud music, ask for a larger model.
- Read the transcript before using it. More than 20% `♪` or `�` tokens, or nonsense words, means it failed. Retry with a larger model, or ask the person for the text.
- Whisper invents "Thank you." over silence and repeats lines over a silent tail. Drop words that start inside a final silence.
- Never put two words in one timed entry. Match transcript words to caption words by position, not by text, so a repeated word ("enak, enak banget") takes the right time.
- Keep the person's spelling of names, dishes and slang ("mantul", "es teh manis"). Fix only what the model misheard.

## Structure

- One scene host with `data-track-kind="captions"` holds every caption group, so the Studio shows them on one captions track.
- Look for a ready-made style first: `catalog --tag caption-style` (`video-hyperframes-registry`). An installed style is ordinary source you can restyle with the brand's tokens.

## Groups

- Use 2–3 words for high energy, 3–5 for conversation and 4–6 for calm. Break at a sentence end, at a pause of 150 ms or more, or at the word limit.
- One group is visible at a time. A group opens at or before its first word's start and closes at or after its last word's end. A caption at exactly 0 s looks pre-existing, so start 0.1–0.3 s in.
- Every group ends with a **hard kill** after its exit, or a stale group can stay on screen: fade to 0 over 0.12 s ending at its end time, then `tl.set(group, { opacity: 0, visibility: "hidden" }, end)`. `tl.set` is an instant flip, so it is allowed on a group; never tween `visibility` on a `.clip`.
- Fade the group or its words, not both, because the opacities multiply and read as a pop.

## Fit and place

- Size each group to fit with `window.__hyperframes.fitTextFontSize(text, { fontFamily, fontWeight, maxWidth })`, which returns `{ fontSize }`. Defaults: `maxWidth` 1600 landscape and 900 portrait, base size 78, minimum 42. When one word scales up for emphasis, divide `maxWidth` by that scale.
- Use a full-width absolute container, centred with `text-align`. Never use `left: 50%; transform: translateX(-50%)`, which clips at the frame edges.
- Set `overflow: visible` on the container. `hidden` cuts off scaled words and glows.
- **Landscape:** 80–120 px from the bottom.
- **Portrait 1080×1920:** the bottom edge of the caption sits at or above y 1248, the lowest point every app's UI leaves clear. Keep captions inside the 80% title-safe box and never over the speaker's face.
- Position every group absolutely. In a flex column, a hidden group still takes space and pushes the next one down.
- **Over footage,** choose the blend by the brightness (0–255) behind the text. Under 60, use `screen`. From 60 to 180, use `overlay`. Over 180, use `normal` with an opaque colour. Plain subtitles over anything brighter than 60 get a scrim sized to the text (30–40% black), never a full-width bar.
- Before placing captions, check whether the footage already has burned-in subtitles somewhere in the clip.

## Cards over a talking head

| Layout | Landscape (x, y, w×h) | Portrait (x, y, w×h) |
|---|---|---|
| Split, video beside the card | 960, 0, 960×1080 | 0, 960, 1080×960 |
| Stack, video above | 14, 14, 1892×548 | 0, 0, 1080×844 |
| Overlay | full frame | full frame |

Picture-in-picture sits 28 px from the edges: a 16:9 source in landscape at 1432, 28, 460×258, and a 9:16 source in portrait at 738, 28, 312×555.

- A lower third is the bottom 30% of the frame. A side panel is the right 42% in landscape, or the bottom 40% in portrait.
- Portrait type is landscape type × 1.3 (titles × 1.35, numbers × 1.4). Padding drops from 40–64 px to 24–36 px.
- Size each box to its source's aspect. `object-fit: cover` crops a portrait source in a landscape box.
- Cards fade in over 0.4 s (`power2.out`) and out over 0.35 s (`power2.in`). Re-frame the video (0.5–0.7 s) only in a gap between cards, with transforms on a wrapper.
- A static card goes dead after about 8 s; past about 15 s it needs a staged reveal. As a start, use one card every 6–8 s in a video under a minute.
- Lower-third and overlay scenes need a transparent root, or they hide the speaker.
- Glass over bright footage goes white on white. Put a dark scrim behind it (about `rgba(0,0,0,.55)`) and give the card a dark base.
- Clamp every card end, and the root `data-duration`, to the footage's real length, or the render ends in black.

## Text behind the speaker

This puts a big word between the footage and a cut-out of the person. Stack the video, then the caption, then the cut-out, ordered by `z-index`. The worker makes the cut-out with `remove-background`. Mark the cut-out `data-layout-allow-occlusion`, because the checker treats a video as opaque. The word reads best 30–55% covered: under 15% it floats, and over 65% it is eaten. Use this only for a punchy word or two, not for running captions.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
