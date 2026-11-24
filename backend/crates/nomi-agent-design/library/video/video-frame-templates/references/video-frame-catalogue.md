# Video frame catalogue

The single frames and short multi-scene templates you can build a video from, each with its job, its composition, and how to draw it as a key frame with a motion note.

Every frame here is drawn flat: grain, glows, glass, scanlines and colour fringes become a solid base plus a build note for the render. Effects marked **tech only** follow Taste rule 8.

## Single frames

### Title card (clean)
- **Job:** the opening name of the piece or a chapter.
- **Composition:** 1920x1080 or 1080x1920; title centred or on the left third at 120-200 px, a small kicker above, one supporting line below; ground from the palette.
- **Motion note:** `title rises 24 px and fades in, 0.5 s, kicker 0.15 s later`.

### Glitch title (tech only)
- **Job:** a hero title or transition for software, gaming, security.
- **Composition:** near-black ground (#070708 or CRT grey #0D0E10), a big bold grotesk or mono title in near-white, centred.
- **Drawing:** draw the title once, solid; optionally two thin offset copies in cyan and magenta at low opacity if the person wants the look shown. Grid and scanlines are a note.
- **Motion note:** `title slices jitter sideways for 0.8 s then lock; RGB split settles`.

### Film light-leak chapter card
- **Job:** the opening of a documentary, a personal short, a brand story (a family warung's 30 years).
- **Composition:** 2.39:1 letterbox inside 1920x1080 (black bars 140 top and bottom, picture 1920x800), a deep warm ground (red-brown #1A0D08, ink green #0A1410), a large serif title, a small chapter line and date.
- **Drawing:** the bars as black rectangles; the warm leak as one flat warm tone block near a corner with a note; never a cold blue leak.
- **Motion note:** `warm light leak drifts across top right, film grain 14%, title fades up over 1.2 s`.

### Colour-field hero (tech only for the liquid version)
- **Job:** an opening frame or a hero background behind a product name.
- **Composition:** full-frame field of 3-5 large soft colour shapes from the palette, the name and one line on top.
- **Drawing:** two or three large ellipses in flat palette colours, the text on a calm area; for a non-tech brand use a flat ground and a photograph instead.
- **Motion note:** `colour shapes breathe and drift, 8-14 s loop`.

### Logo outro / end card
- **Job:** the last 2-4 seconds: brand, line, action.
- **Composition:** dark or brand-colour ground; the logo centred (the person's real mark, never redrawn); brand name below at 48-72 px; a tagline at 24-28 px, softer; a call to action (`Pesan lewat WhatsApp`, a handle, a site).
- **Drawing:** the logo as a placeholder frame with `Logo · file from the person` if you don't have it as a path.
- **Motion note:** `logo pieces assemble 1.2 s, soft bloom, name types in at 1.4 s, tagline fades in at 1.8 s, hold 1.5 s`.

### Notification banner
- **Job:** an announcement shown as a desktop or phone notification, in a corner of a video or as its own card ("Pesanan baru masuk · Rp 245.000").
- **Composition:** a rounded banner (radius ~14) about 480x120 (or 480x180 with body text), app icon left, app name and time on top, title and one line of text.
- **Drawing:** a light card (#F5F5F7) or dark card (#1C1C1E) with a hairline border and a soft shadow; the frosted glass is a note.
- **Motion note:** `slides in from the top right, 0.35 s, holds 2.5 s, slides out`.

### Data chart frame
- **Job:** one finding as a chart, for a report video or a social cut.
- **Composition:** warm white ground (#F7F5EE) or ink ground (#0E0E0E); a small caps kicker in the accent naming source and range; a serif headline that states the conclusion from the person's own data; the chart takes 55-65% of the frame; key points labelled in small mono (`Juni · 1.240 pesanan`).
- **Drawing:** bars as rectangles in ink with one accent bar; a line as a `path` with dots as small ellipses.
- **Motion note:** `bars grow from the baseline 0.6 s staggered; the headline number counts up`.

### Sticky-note flowchart
- **Job:** explaining a process: ordering, onboarding, how a service works.
- **Composition:** whiteboard ground (#F4EDE1 or #F0F2F4); 4-7 notes of about 240x180 in four colours (yellow, peach, mint, sky-blue); each note an icon, a big title and one line; connecting arrows with short labels.
- **Drawing:** notes as frames without rotation (note the slight tilt for the render), arrows as `path` lines with a small head, handwriting face named for the build.
- **Motion note:** `notes pop in along the flow, arrows draw between them, 0.3 s each`.

### Motion poster frame (loop)
- **Job:** a single looping composition: a launch teaser, a background for a screen in the shop.
- **Composition:** 16:9; layers back to front: a quiet ground, two or three thin concentric rings from a focal point, the focal object (a globe, a product, a number), a ring of words or labels around it, a ticking element (a timer, a date).
- **Drawing:** rings as stroked ellipses, labels as small text placed by hand.
- **Motion note:** `type ring rotates slowly, timer ticks, labels drift; seamless 10 s loop`.

### Explainer beat with a sprite
- **Job:** one beat of an educational vertical video ("1945 · Proklamasi", "Sejarah tempe").
- **Composition:** cream ground (#F5EFE2); a small mono top bar (series name, index); a big year or number; a one-line headline; a simple mascot or object; a short caption.
- **Drawing:** the sprite as a simple shape built from rectangles or a placeholder with a note.
- **Motion note:** `sprite bobs on a 2-frame loop, year counts up, headline slides in`.

### One-shot clip from a video model
- Text-to-video models are best at one shot with one idea, up to about 10 seconds. Plan one shot per clip: subject, setting, camera move, light, length, ratio. You can't generate it; write the shot brief in the scene frame's placeholder (see `../../../images/images-visual-briefs/SKILL.md`).
- The HTML-to-MP4 renderer is not a camera: treat its scenes as motion-design cards, not photoreal footage.

## Multi-scene templates (three scenes, about 3 seconds each)

| Template | Job | Scenes | Look |
|---|---|---|---|
| **Pixel orbit** | A game, hackathon or playful tech teaser | Title · feature · call to action | Retro pixel arcade: deep navy, neon pink, cyan, yellow, pixel display face. Tech or gaming only |
| **After-hours editorial** | Fashion, beauty, a premium launch | Three pages: chapter title · story line · lockup | Dark editorial, large serif, staged text reveals, chapter wipes; grain and vignette as notes. Dark only for luxury or nightlife |
| **Swiss research story** | Sharing customer research or survey results | Framing title · participant breakdown (donut) · pattern + evidence panel | Warm paper, strict spacing, thin rules, restrained motion |
| **Year in review (vertical)** | An annual recap: reading, sales, a member's year ("Tahun kamu di Kopi Kenangan Senja") | Opening · totals (time, active days) · shelf or top items · highlights and keywords · persona · share card | 9:16, bold numbers, one card per stat; uses only the person's real data |

Every scene holds 3 seconds or less; motion is deterministic (no random loops), so the render is the same every time. Draw one frame per scene, ids shared across scenes for repeated parts (the logo, the progress dots), with the motion and transition note at the bottom of each.

Note: the year-in-review template comes from a Chinese reading app's annual report; think of it as a Wrapped-style recap.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus the html-anything frame templates (nexu-io/html-anything, Apache-2.0), HeyGen's HyperFrames (heygen-com/hyperframes, Apache-2.0) and Zara Zhang's 8-bit Orbit look (MIT, Copyright (c) 2026 Zara Zhang). Changed: rewritten for the OpenPencil canvas.
