---
name: taste-photos-motion
description: "How to draw Taste's photographs as sized placeholders with a canvas note, how many go where by kind, and how motion, moments and effects become annotations for Koda, including which pages may have them at all."
license: "MIT, Copyright (c) 2026 Leonxlnx"
source: "https://github.com/Leonxlnx/taste-skill (2.5.1)"
---
# Taste: photographs, motion and effects on a static canvas

How to draw Taste's photographs as sized placeholders with a canvas note, how many go where by kind, and how motion, moments and effects become annotations for Koda, including which pages may have them at all.

## Photographs as placeholders

The canvas holds no images. A photo is a frame (or rectangle) at its exact final size and ratio on the grid, filled with a solid tone, with one short note inside:

`Photo · baker's hands shaping dough, morning window light, close crop`

- **The note is a canvas note, not page copy:** name the layer `photo-note`, set it 13-14px in a colour that reads on the tone, top-left inset 16. It says subject, framing and light. On the built page no caption, label or credit sits under or on any picture (house rule 1).
- **The tone** is the photo's real dominant colour (crust brown, leaf green, a dim room), never a flat grey "empty box": the composition must read as finished (house rule 2). A placeholder that will carry words (hero and one closing band on tech, product, service) has a darker or lighter calm area the note names: "calm dark sky across the left half, phone crop too".
- **What the photos show:** the client's real product, place, people and work, in daylight, one light and one grade across the set. A place hero is the place itself: the room from the door, the counter with the machine, bread on shelves, the shopfront with customers. A product is shown whole and big where it leads, then its use, parts and the people with it. A remote guesthouse leads with its region's landscape.
- **Never ask for:** mood stock (curtains, plant shadows, sky, fog, a forest), latte art from above, the croissant-and-coffee still life, a made bed, flour thrown on black, a whole figure posing, stock handshakes, people at screens, another business's sign or logo, a landmark passed off as the client's place, AI-looking composites.
- **Never on a photo:** filters, duotone, halftone, grain, a scrim or gradient box, rounded corners on a place, words on a place's photo.
- In the handoff, list every photo slot with its note as the shot list (when the person has no photos: shopfront straight on, the room from the door, the counter, three products on one surface, hands at work, one detail, each room; window light, no flash).

## How many, where

| Kind | Photos |
|---|---|
| Place | full page 12-20 (2-3 a screen, at most 6 screens), compact 8-12 (4-5 screens), short page under 8 (3 screens). By section: hero 1, visit 1, menu 1, rooms 2-6, goods 4, the place 3 (or a wall), orders 2-3 or booking 1, close 1 (1:1, never used above). |
| Product | two or three a screen; the product whole in three sections at most, never one view twice; every headline promise pictured; every colour offered shown once. |
| Service | one or two a screen: the work, the team at work, the rooms. |
| Software | real people or the real product only; the UI leads. |
| Gaming | key art and in-engine shots; without art, the game's name large over stand-in photos of its setting. |

Ratios: a place 4:5, 4:3, 1:1; others add 3:2. One ratio per row, equal sizes in a row, one dominant picture per screen or a row of three or four. Full bleed only as a hero or section layer. No photo taller than 708 outside the hero. Never the same photo twice (a crop of one is the same photo).

## Motion: what every page has (annotate it, never fake it)

Taste gives every page three layers of motion and nothing else. Draw the resting, finished state; write the motion as a short annotation (a `motion-note` text frame beside the screen frame, outside the page, or in the handoff), one line each with its purpose:

1. **Hero entrance, once on load** (every kind): the h1 rises line by line (700ms a line, 80ms apart), then lead, buttons and facts fade up 12px (520ms each, 60ms apart); a photo settles from 1.05 to 1 over 1.1 s. About 1.1 s in all, ease-out, never hides text for long, never repeats on scroll.
2. **One dealt moment** in a section of its own; two on a long page (eight sections or more), never three.
3. **The hover and press layer:** buttons step up 1px on hover and press 1px down at scale .985; a linked card lifts 2px; a photo in a link zooms 1.03 inside its frame over 600ms; text links draw an underline; arrows move 3px; the nav panel opens after 150ms with a 6px drop. Design for it: on a place every photo that leads somewhere is a link, and each section after the hero holds one such link.

Annotation example: `Motion · hero entrance: name first, then the action · photo-chapters in Kamar: three rooms in the space of one · hover: every link answers`.

## Moments by kind (pick one or two, write the section and why)

| Kind | Moments allowed |
|---|---|
| Place | `photo-chapters` (3-5 full-width photos uncovered one over the last as you scroll, each chapter's title and two lines on a panel at its lower left; the section is 3 or 4 screens), `picture-develops` (one large photo develops from a faint grey print, 1.6 s, once), `word-highlight` (one statement of 12-40 words lights up word by word as you scroll) |
| Product, service | the same, plus `pinned-chapters` (2-4 steps listed beside one frame that changes per step; never on the same page as photo-chapters) |
| Software, gaming, tech | the above except photo-chapters and picture-develops, plus counter-rolls, letters-assemble, marquee-row (real words, never logos), line-carries-colour, iris-opens, giant-type-pass, liquid-type and liquid-hover |

- Draw a moment's section at rest, finished: photo-chapters as its chapters side by side (each photo over its words), pinned-chapters as rows, word-highlight as the statement in full ink. Never in a card, a plan, pricing, an FAQ or the footer.
- On phones and under reduced motion every moment shows that finished still (word-highlight, picture-develops and photo-chapters still run on phones).

## Effects and scenes: tech pages only

- **Never** on a place, product brand, shop, service firm, agency or studio (house rule 8): no shader, WebGL, canvas, autoplay video, gradient, glass, grain, blend modes, loops, counters, marquees, custom cursors, scroll animation of any kind beyond the dealt moment.
- **Software, gaming, hardware** may carry, by deal: one effect (a live ground in the hero or in one framed visual under an h2, a WebGL part, an object, or one big scene in its own two-screen section), one timed loop, one card loop per card group (2.5-4 s cycles, restarting on hover, with a pause button when over 5 s), diagram pulses, one count. Never two moving things in one screen, a calm section between beats, the back third calm. Draw the effect's still (its poster) in flat palette colours and annotate it by name and placement: `Effect · soft grain ground behind the app window, still in cards, pause button bottom-right`.
- A product render only when the brief asks for 3D; otherwise photos. An effect never stands where the product, the device or the game's world should be.

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
