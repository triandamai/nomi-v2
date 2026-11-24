---
name: huashu-audio
description: "Huashu's audio rules for video: two tracks (SFX and music) by default, levels, frequency separation, fades, SFX density and cue priority, music choice, the SFX library's categories, and narration-driven video without becoming a slideshow with a voice. Read it before any video's sound."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Audio: two tracks, then narration

Huashu's audio rules for video: two tracks (SFX and music) by default, levels, frequency separation, fades, SFX density and cue priority, music choice, the SFX library's categories, and narration-driven video without becoming a slideshow with a voice.

**A silent video is half finished.** Deliver with sound effects and music unless the person says "tanpa suara", "visual saja" or "saya isi suara sendiri". Music alone is a third of the job: viewers sense "it moves but nothing answers", and that is where cheapness comes from.

## Two tracks, designed separately

| Track | Job | Time scale | Sync with picture | Frequency |
|---|---|---|---|---|
| SFX (beats) | Marks each visual beat | 0.2 to 2s | Frame-exact | Above about 800 Hz |
| Music (bed) | Mood, space | 20 to 60s continuous | By section | Below about 4 kHz |

**Levels:** music at 0.40 to 0.50 of full scale, SFX at 1.0; music peaks 6 to 8 dB under SFX peaks. SFX stand out by the difference, not loudness. Mix without normalising (normalising flattens the dynamics).

**Frequency separation:** the ear is most sensitive at 2 to 5 kHz. Cut the music above 4 kHz and the SFX below 800 Hz so each owns its band; SFX clarity jumps a level.

**Fades:** music in over 0.3s, out over the last 1.5s; SFX carry their own envelope.

## SFX cues

**Density per 10 seconds:** calm, focused products 0 to 3 (music leads); lively, information-rich ones 6 to 9 (SFX drive the rhythm); balanced office tools about 4. Don't fill every beat: **deleting 30 to 50% of cues makes the rest more dramatic.**

| Priority | Cues |
|---|---|
| Must | Typing; a click or selection (a decision); a focus change; the logo reveal |
| Recommended | Elements entering or leaving (dialogs, cards); success; AI starting and finishing; major scene changes |
| Optional (more gets messy) | Hover; progress ticks; ambient decoration |

**Alignment:** clicks, focus changes and logo landings on the same frame; quick whooshes 1 to 2 frames early (anticipation); impacts and landings 1 to 2 frames late (physics). At most two SFX at one moment (three if music is under 0.3); before a brand impact clear everything for 0.2s. Keep SFX under 0.5s and at least 0.2s apart.

**Match the picture:** warm paper looks get wooden, soft sounds (paper snap, soft click); cool dark tech gets metallic, digital ones (beep, pulse); playful hand-drawn looks get cartoon sounds (pop, boing). Advanced: design the SFX timeline first and fit the motion to it; picture chasing sound is off by a frame and feels wrong.

**The library** (37 short effects in eight categories; the cue table lists second, file, level): keyboard (single key, fast typing, backspace, space, enter); UI (click, soft click, focus, faint hover, finger tap, toggle on); transition (whoosh, fast whoosh, horizontal swipe, slide in, dissolve); container (card snap, card flip, stack collapse, dialog open); feedback (success chime, error tone, notification pop, achievement); progress (loading tick, step complete, AI generation start); impact (logo reveal short and cinematic, stamp, drop thud); magic (sparkle, AI processing hum, transform); terminal (command execute, output appear, cursor blink). Choose by the action: tactile action, keyboard or UI; elements entering, transition; cards and dialogs, container; status, feedback; time passing, progress; brand moments, impact; AI change, magic; code, terminal. To generate a new effect, describe the physical sound (object and action), the character (crisp, minimal, cinematic) and exclusions (no reverb, clean studio): not "click sound" but "crisp UI button click, clean modern interface, high pitched, no reverb".

## Music

Choose by character: launch or tech demo, minimal synth and piano; tutorial or tool walkthrough, warm and nearly invisible lo-fi; education, curious and bright; ads and promos, upbeat with a build and drop. Use music without a strong beat under SFX-heavy films, or the beats fight. **Skip music** when the video is under 10 seconds, the product is meditative, there is narration or live sound, or SFX density is already high; then three to five carefully placed effects carry it.

| Recipe | Length | Music | SFX |
|---|---|---|---|
| Launch hero | ~25s | Tech bed at 45% | ~6 per 10s: four typing ticks, enter, four card snaps staggered 0.2s, a click, a whoosh, four focus tones, a 1.5s logo thud |
| Tool walkthrough | 30 to 45s | Tutorial bed at 50% | 0 to 2 per 10s, only at decisive moments (saved, done) |
| AI generation | 15 to 20s | Tech bed or none | ~8 per 10s: type and enter, a looping processing hum, complete, sparkle |
| Pure mood shot | 10 to 15s | None | 3 to 5 hero cues |

**Check:** SFX peaks 6 to 8 dB over music; the band split applied; fades in place; density fits the product; every cue within one frame of its beat; the logo sound about 1.5s long. Listen with music muted (do SFX alone have rhythm?) and with SFX muted (does the music alone rise and fall?). Each track must work alone. Confirm the final file actually has an audio stream.

## Narration-driven video

For explainers of a minute or more: write the script first, generate the voice, and let the **measured** durations drive the picture. Under 60 seconds, a silent animation plus a separate voice track is enough.

**The iron rule: never make a slideshow with a voice.** The failure: seven scenes each with its own layout, text fading up, whole-page opacity swaps. Viewers say "seperti PowerPoint". Instead:

1. **One continuous motion story.** Pick one or two hero elements present from start to end; every cue is a change of their state (position, size, colour, perspective, form), not a new element. Scenes exist in the script, not on screen.
2. **No hard swaps between scenes:** the hero morphs from one state to the next; supporting elements enter with expo-out and stagger and leave overlapping the next section.
3. **Every frame moves:** a hero breathing (scale 1 to 1.02 over 5 seconds), a slow background drift, a slight float on cards. A fully still frame is a slide.
4. **Expo-out, 30ms stagger, a 0.3 to 0.5s hold before key cues, a hard 1s final hold.**

**Script format:** sections headed with a short id; cue markers placed in the middle of the key sentence, where the picture should trigger; short sentences (long ones sound flat in synthetic voices). Listen to the whole voice track before animating: it sets the quality ceiling. Before building, answer: what is the hero, what is its state in each section, how does it morph?

**Subtitles by default** (long narration without them loses viewers): no background bar or blur; dark ink with a light halo on light grounds, white with a dark halo on dark; about 32px at 1080p in a clean sans; about 90px above the bottom; one line of at most about 40 Latin characters; split at sentence ends first, then at commas, never mid-phrase ("Pesan sekarang, / bayar di tempat", not "Pesan sekarang, bayar / di tempat"). If a line comes out too long, move a cue in the script rather than tweaking the splitter. Camera moves snap to the voice's silences (`huashu-cinematic`, move on pause).

Viewer test: "PowerPoint with a voice" fails; "the picture changes with the voice" lacks a continuous story; "something moves" passes; "I want to watch to the end" is good; "I want to screenshot this part" is great.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
