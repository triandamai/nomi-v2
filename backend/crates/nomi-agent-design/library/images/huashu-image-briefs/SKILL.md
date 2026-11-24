---
name: huashu-image-briefs
description: "How huashu sources and generates images: when a picture is content, the order of sources, generated versus real, writing a Gemini prompt (concrete, matched to the page, mood not layout), picking among candidates, edits and size templates. Read it before generating any image."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Image briefs

How huashu sources and generates images: when a picture is content, the order of sources, generated versus real, how to write a generation prompt (concrete, matched to the page, mood not layout), picking the best of several, edits from an official reference and size templates.

`images-visual-briefs` covers the caption inside a canvas placeholder and the brief a person can paste elsewhere; this file is how you yourself decide on, prompt for and choose a generated image (Gemini). On the canvas an image is still a sized, captioned placeholder (`pen-format`); generated images go into decks, videos, covers and the built site.

## Where pictures belong

1. **Every direction names its hero material first** (`huashu-directions`): real photo, generated image, data drawn as image, a recreated object (a receipt, a label, a menu card), giant type. Don't let all three directions be pure typography; when you do use type, make the type itself the picture.
2. **Pictures the content needs come before the design.** A page introducing a dish, a place, a person, a product: the picture is content. A parrot site without parrots is a failure; an app page without the app's screens is a failure. When unsure, treat it as needed.
3. **Honesty test:** remove the picture; is information lost? If not, it's decoration: leave it out. No stock "inspiration" photos.
4. **Where a picture belongs, put a real one:** avatars, dishes, products, rooms, covers, characters, hero visuals. Never an emoji, a drawn person or a grey box labelled "gambar"; viewers recognise an unfinished AI draft instantly. Geometry, icons, charts, maps and textures are drawn, not generated.

A real test: the same app specimen with drawn objects (a cardboard box, a potted plant) swapped for generated photos, nothing else changed, became far more believable.

## Order of sources

1. **The person's own material** (their photos, products, shop): always first for their own business. Never invented.
2. **Official sources** for named brands and products (`huashu-brand-asset-protocol`).
3. **Generation with Gemini**, for "what this kind of thing looks like".
4. **Openly licensed photos** (Wikimedia Commons, Unsplash, Pexels, Openverse), credited with author and licence where required.
5. **None of these work:** tell the person in one line what's missing and what would fill it; leave those spots marked as missing in the first line of the handover. That is unfinished, not a style.

**Generated or real?** Generate when you need "this kind of thing" (a bowl of soto, a living room, a person running): it can match the page's colour and light. Use a real photo when you need "this specific thing" (a real place, an artefact, a named product, a real person): a generated one would be fake. Never generate a named product from nothing: give the official image as the reference and generate a variant for the scene. Never pass stock or generated images off as the client's own (Taste's house rules).

If generation fails (quota, refusal), step down the list, or pick an easier subject (can't get the finished dish, show its ingredients). Never fall back to emoji or flat colour.

## Writing the prompt

- **Concrete subject, angle, light, material, setting.** "Semangkuk soto ayam baru matang, difoto dari atas, mangkuk keramik putih bermotif biru, cahaya jendela pagi dari kiri, meja kayu jati tua, uap tipis" beats "foto makanan premium". Never adjectives like "high-end", "premium", "minimal" on their own.
- **Match the page:** write the page's main colours, light or dark, and aspect ratio into the prompt so the image belongs to the design ("dominan hijau lumut dan krem, terang, rasio 4:3").
- **Mood, not layout.** Short prompts beat long ones. Describe the feeling ("hangat seperti Minggu pagi"), name a concrete aesthetic ("editorial majalah kuliner tahun 70-an"), say what the viewer should feel. Don't prescribe colour percentages, element positions, or list every object: that kills variety.
- **Anchor the treatment** for sets: one shared style phrase ("arsip foto museum tahun 60-an", "ilustrasi gouache datar yang konsisten") so every image in a set matches.
- **A persona line helps finish:** "Anda kurator majalah desain yang menyiapkan foto pameran."
- **For objects you'll cut out:** one sheet of 4 × 2 items on plain white with generous space between (about 60px), one unified style; then cut each item to its own transparent image. White grounds cut cleanly (grey is moodier but hard to cut); 4 × 2 rather than 5 × 5, whose last row gets squeezed.
- **No text in the image** unless it's the point; type is set in the design.

## Choosing and editing

- **Two or three per slot, then look and pick.** Generation costs the person's quota: start with one per slot, add another only if it isn't right.
- Score every candidate like any asset: 8/10 or better, or don't use it (the 5-10-2-8 bar in `huashu-brand-asset-protocol`). A 7/10 image is a deduction; an honest placeholder is better.
- **Open it and check the four corners:** a platform watermark ("AI generated" marks) disqualifies it. Check hands, faces, text-like scribbles, impossible objects.
- Two images on one page must agree in light, composition and style.
- Save the file you chose; a returned temporary link expires.
- For edits (change background, lighting, season, angle) always start from the real or official image as the reference; keep the product identical.
- Where generated imagery needs a disclosure on the platform, say so to the person.

## Size templates

Combine: [style phrase] + [template] + [the specific content].

| Use | Size | Brief essentials |
|---|---|---|
| Article cover or header | 2.35:1 (1200 × 510) | Strong single focal point, little or no text (the title sits over it), moderate saturation, still readable as a thumbnail |
| Article illustration | 16:9 (1200 × 675), 1:1, 4:3 | One concept serving the argument, not decoration; light or dark to match the page |
| Infographic | 1080 × 1920 tall, 1920 × 1080, 1080 × 1080 | Clear hierarchy (title, key data, detail), a reading path, accurate data only |
| Slide image | 16:9 (1920 × 1080) | One message, room for type, projection-friendly contrast |
| Landing hero | 1440 wide, first screen | The product or place itself, the core value in five seconds |
| App screen imagery | 390 wide screens | Avatars, dishes, products at real proportions |
| Instagram feed | 4:5 (1080 × 1350) or 1:1 | Wins attention in a feed, text under 20% of the image, real texture and atmosphere |
| Reels, Stories, status | 9:16 (1080 × 1920) | Keep faces and text out of the top and bottom safe areas |
| Social carousel | 3:4 (1080 × 1440) | The cover decides the tap |

Styles whose soul is generated imagery (particles, cinematic volumetric light, ink wash, concept art) are listed in `huashu-design-styles`; offer them only when generation is available for this job.

## Script (run later in Nomi's render worker)

`gen_image.py` generates one image per run into a given file and size, optionally from reference images, and only accepts an image that was really created in this run (an agent without generation ability would otherwise assemble a fake with code). Run it once per candidate, one at a time, then look and choose. In Nomi the generator is Gemini; keys come from the environment, never from the conversation.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
