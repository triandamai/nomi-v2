---
name: video-frame-templates
description: "How you design a short video (format, length, hook, pacing, safe areas, captions, type, palette, transitions, end card, audio brief) and draw it as a storyboard of key frames with motion notes; read before any reel, promo clip, intro or outro."
license: "Apache-2.0, Copyright nexu-io (open-design, and its html-anything frame templates); Apache-2.0, Copyright HeyGen (HyperFrames); MIT, Copyright (c) 2026 Zara Zhang (8-bit Orbit look)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/hyperframes; https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (motion-frames, video-shortform, sprite-animation, audio-jingle, webgl-experience); https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (video-hyperframes, frame-glitch-title, frame-light-leak-cinema, frame-liquid-bg-hero, frame-logo-outro, frame-macos-notification, frame-data-chart-nyt, frame-flowchart-sticky, 8-bit-orbit-video-template, after-hours-editorial-template, swiss-user-research-video-template, weread-year-in-review-video-template); https://github.com/heygen-com/hyperframes; https://github.com/nexu-io/html-anything; https://github.com/zarazhangrui/beautiful-html-templates/tree/main/templates/8-bit-orbit"
---
# Video frame templates

How you design a short video and draw it now as a storyboard of key frames, one artboard per scene with a motion note, so the same frames can later be rendered to MP4.

Read this before a reel, a promo clip, a launch teaser, an intro card or an end card. The frame-by-frame looks (title card, chapter card, end card, data chart, flowchart, notification, multi-scene templates) are in [references/video-frame-catalogue.md](references/video-frame-catalogue.md); named styles, palettes, video type and the transition menu are in [references/video-styles-and-palettes.md](references/video-styles-and-palettes.md). Tools that generate footage, voice or music live in [images-media-tools](../../images/images-media-tools/SKILL.md).

## 1. Decide the video before you draw

Write these five lines in your plan first:

1. **Where it plays and its size.** Reels, TikTok, Shorts, WhatsApp Status, Stories: vertical 1080x1920. YouTube, a website hero, a presentation: landscape 1920x1080. Never draw both unless asked; a re-cut is a new storyboard.
2. **Length.** Teaser or intro 5-10 s. Promo reel 15-30 s. Explainer 30-60 s. Shorter wins: a reel that says one thing in 15 s beats one that says four in 45.
3. **The one message and the one action.** "Kopi Susu Gula Aren sekarang ada" and "Pesan lewat WhatsApp". Everything else supports these two.
4. **The arc.** Hook (first 1.5-2 s) → two to four proof beats → the offer → the end card. One idea per scene.
5. **The look.** Kind and palette from Taste (`taste-core`), then a video style that fits it (references file). Light ground for food, wellness, kids, crafts; dark for tech, gaming, cinema, nightlife, premium.

All copy is real and in the person's language, written before you draw. No invented prices, discounts, reviews, follower counts, customer logos or phone numbers: use what the brief gives, or leave a clearly marked slot (`[harga dari kamu]`).

## 2. Hook, pacing and reading time

- **Hook in the first frame.** One of: the product big and close, a real number from the brief, a question the buyer is already asking ("Capek antre kopi pagi-pagi?"), or the result first. Never a logo intro: the logo belongs at the end.
- **Scene length.** 2.5-4 s a scene; hype cuts 1.5-2.5 s; a hold on the offer or end card 3 s. Multi-page templates keep each page under 3 s.
- **Reading time.** A line on screen for 3 s must be readable in 2. That means 3-7 words a line, one line of copy a scene plus at most one small label.
- **Every scene has three phases.** Build (first 30%: elements enter, most important first), breathe (middle 40%: everything visible, one small ambient move or stillness), resolve (last 30%: the transition takes it away).
- **Vary the rhythm.** The slowest scene about three times the fastest. Not every element from below; not the same stagger every scene. Stillness after motion reads as confidence.
- **Count scenes from the length:** 15 s ≈ 5-6 scenes, 30 s ≈ 8-10, 60 s ≈ 14-18 (then draw the key moments, not every cut).

## 3. Composing a frame (video is not a web page)

- **Fill the frame.** A headline spans 60-80% of the width. Web-sized type disappears on a phone.
- **Anchor to edges or split the frame,** not one centred block floating in space: words left and product right, a top bar of metadata over a full-width picture, a number panel beside a photo.
- **Two focal points and three layers** in every scene: a ground (a colour panel, oversized type at 4-8% alpha bleeding off the edge, hairline rules), the content, and one accent (a rule, a label, a price tag).
- **Draw the hero moment:** the instant when everything in the scene has entered and nothing has left. Position every element there. The motion note then says where each one comes from.
- **Same element, same place, same id** across scenes (logo lockup, caption bar, price tag) so the eye and the render both follow it.

## 4. Safe areas

| Canvas | Keep text and faces inside | Why |
|---|---|---|
| 1920x1080 | 96 px from every edge; captions sit 80-120 px above the bottom | TV and player overscan, progress bars |
| 1080x1920 | x 64-944, y 220-1540 | Platform UI covers the top ~220 (account, sound), the bottom ~380 (caption, buttons) and a right rail ~136 (like, share) |

Pictures may bleed to the edge; words and the product's face may not. Draw the safe area once as a reusable guide frame (1 px stroke at low alpha, no fill, named `Guide · safe area (not rendered)`) and place it with `ref` in every scene.

## 5. Captions

Most reels play muted: if anyone speaks, caption it; if no one does, the on-screen line is the caption.

- **Group words by energy:** hype 2-3 words a group, conversational 3-5, calm 4-6. Break at sentence ends and pauses. One group on screen at a time.
- **Position:** landscape centred 80-120 px above the bottom; portrait in the lower middle, about 600-700 px from the bottom, clear of the button zone. Never over a face or the product.
- **Style by tone:** hype in a heavy condensed face 72-96 px; corporate a clean sans 56-72; tutorial mono or plain sans 48-64; storytelling a serif 44-56; social a rounded bold 56-80 on coloured pills.
- **Emphasis:** the brand or product name, numbers and the call to action get the accent colour or a heavier weight. One emphasised word a group.
- Width at most 1600 (landscape) or 900 (portrait), with room if an emphasised word grows. Draw the caption bar once as a reusable frame.

## 6. Type in motion

- Sizes for 1080p: headline at least 60 px (on vertical, 96-160), body 28 and up, labels at least 20; 16 px is the floor only for small landscape data labels.
- Weight contrast must be obvious in motion: a light 300 against a black 900, not 400 against 700.
- Pair across categories (serif with sans, sans with mono) or one family at two weights; never two similar sans. One expressive face a scene.
- Tighten display tracking slightly (about -3 to -5% of the size); on dark grounds drop body weight one step and open line height a little.
- Time is hierarchy: what appears first is read as most important. How a word enters is part of the type choice: a 0.1 s slam and a 2 s fade say different things.
- Numbers in columns or counters use tabular figures (note it for the render).

## 7. Palette

Declare ground, ink and one accent before drawing, as tokens: `$color-background`, `$color-foreground`, `$color-primary` (plus `$color-muted-foreground` for labels). One ground across all scenes; a ground change marks a new chapter. Tint neutrals toward the accent instead of pure black or white. Text passes 4.5:1, or 3:1 at 24 px and up. The client's own colours, then Taste's palette for the trade, come before any mood palette in the references file.

## 8. Transitions and motion

- **Every scene enters, nothing exits early.** Each element arrives with its own entrance; the transition is the exit. Only the last scene may fade out.
- **One primary transition** for most cuts plus one or two accents saved for the topic change and the reveal. Calm pieces 0.5-0.8 s, standard 0.3-0.5 s, high energy 0.15-0.3 s.
- **Meaning:** a crossfade says "this continues", a push says "next point", a hard cut says "wake up", a slow dissolve says "drift with me", a dip to the ground colour says "the end".
- **Easing in words:** entrances start fast and land softly; exits start slow and accelerate away; moves between positions ease both ends. Overshoot only for playful pieces. Entrances take longer than exits (0.4 s in, 0.25 s out).
- First move 0.1-0.3 s after the scene starts. Stagger by importance, whole stagger under 0.5 s.
- **Effects belong to tech trades.** Glitch, chromatic split, liquid colour fields, shader-style warps and real-time 3D are for software, gaming, hardware and fintech pieces only (Taste rule 8); a cafe, bakery, shop or service gets photographs, type and clean cuts.

## 9. End card

The last 2.5-3 s: the logo lockup (the person's real mark, drawn as `path` or set as type; never a copy of someone else's), the brand name, one line, and the action ("Pesan di WhatsApp", "@handle", "Buka setiap hari 07.00-22.00" only if the brief says so). It assembles, then holds still: no loop. Reuse it across every video for this brand.

## 10. Audio brief (written, not drawn)

Put an audio line in the notes so the person or a tool can make it later:

- **Music bed:** genre, tempo (bpm), 3-5 instruments, vocals or none, mood arc. Under 30 s is one section; 30-90 s intro plus body. Example: "akustik pop hangat, 96 bpm, gitar, shaker, piano listrik, tanpa vokal, naik di detik 9 saat harga muncul".
- **Voiceover:** the final script, word for word (it is read literally), pace, voice, and how to say names ("Gula Aren: GU-la A-ren").
- **Sound effects:** one well-described layer at a time: texture (whoosh, pop, pour, click), envelope (sharp hit or soft swell), the scene and second it lands.
- Cues follow the cut: a hit on the reveal, silence just before it, the bed resolving on the end card.

## 11. Drawing the storyboard on the canvas

- **One artboard per scene or key moment.** Top-level frame named `S01 · Hook · 0.0-2.5 s`, width 1920 or 1080, `layout: vertical`, `fill` set, x/y 120 apart (vertical scenes at x 0, 1200, 2400…; landscape at x 0, 2040, 4080…; wrap to a second row after five).
- Inside it, first the **scene frame** at the exact size (1920x1080 or 1080x1920), `clip: true`, `layout: none` where you place by hand; then the **note strip** below it (fill `$color-muted`, text `$color-muted-foreground`, 24 px), with one line each:
  - `Masuk:` what enters, in order, from where and how (`judul naik 40 px, 0,5 s; foto membesar 1,05→1; label harga muncul 0,3 s kemudian`)
  - `Tahan:` what breathes or stays still
  - `Transisi:` the exit into the next scene and its length
  - `Teks/caption:` the caption groups and their timing, if any
  - `Audio:` the cue in this scene
- **Pictures and footage** are captioned placeholder frames at their real size and tone: `Photo · gelas es kopi susu di meja kayu, close-up, cahaya jendela` or `Clip · tangan menuang susu ke kopi, slow motion, 2 s`. The note is for the person and the render, never on-screen copy.
- **Effects become flat stand-ins plus a note:** a glow is a low-alpha ellipse in the accent; a light leak two warm ellipses at 15-25% alpha off one corner; a glitch two copies of the title offset 3 px in the accent pair. No gradients, blur or grain on the canvas: write `Effect · …` in the note.
- **Budget.** 300 elements across the preview: a 6-scene storyboard gets 30-40 nodes a frame, a 12-scene one about 20. Make the repeated parts reusable frames (caption bar, logo lockup, price tag, scene counter, safe-area guide, note strip) and place them with `ref`.
- **Tokens only,** the same set as the person's site or brand, so the video sits beside it.

### Worked example: reel promo kopi susu, 9:16, 15 s

| Scene | Time | Frame | Motion note |
|---|---|---|---|
| S01 Hook | 0-2.5 | `Clip · es kopi susu dituang, close-up`, line "Kopi susu baru di Kedai Senja" top-left in the safe area | footage slowly pushes in; words rise line by line, 0.5 s each |
| S02 Rasa | 2.5-5 | split: photo of gula aren top, "Manis dari gula aren asli" below | push transition up 0.4 s; photo first, text 0.2 s later |
| S03 Proses | 5-8 | three small steps as type: "Seduh · Tuang · Aduk" | each word slams in on the beat, 0.15 s apart |
| S04 Harga | 8-11 | price tag from the brief, large, beside the glass | hard cut; price scales from 0.9, the bed lifts |
| S05 Ajak | 11-12.5 | "Pesan lewat WhatsApp" with the arrow | crossfade 0.3 s |
| S06 End card | 12.5-15 | logo lockup, name, `@handle`, hours if given | lockup assembles 1 s, then holds; bed resolves |

## 12. Before you save

- One message, one action, a hook in the first frame, an end card last.
- Every scene: hero moment drawn, inside the safe area, real copy, a note strip with all five lines.
- Type sizes are video sizes; captions grouped and placed clear of faces and platform UI.
- One ground, one accent, contrast passes; effects only on tech trades.
- Under 300 elements, repeated parts are `ref`s, ids stable across scenes.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), with HeyGen's HyperFrames (heygen-com/hyperframes, Apache-2.0), the html-anything frame templates (nexu-io/html-anything, Apache-2.0) and Zara Zhang's 8-bit Orbit (MIT). Changed: rewritten for the OpenPencil canvas.
