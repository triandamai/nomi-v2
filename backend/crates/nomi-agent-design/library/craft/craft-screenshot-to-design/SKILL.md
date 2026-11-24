---
name: craft-screenshot-to-design
description: "How to read a picture (a screenshot of a site, a reference the person likes, a photo of a printed menu or a chat) into frames, tokens and real text on the canvas, section by section, without drifting from it. Read it whenever the person sends an image."
license: "MIT, Copyright (c) 2026 Leonxlnx (image-to-code-skill, stitch-skill); chat-motion-overlay is open-design's own (Apache-2.0); via open-design (Apache-2.0)"
source: "https://github.com/Leonxlnx/taste-skill (image-to-code, stitch-skill); https://github.com/nexu-io/open-design skills/chat-motion-overlay; open-design@5fdcc3d"
---
# Read a picture into a design

How you turn an image the person gives you into frames, tokens and text on the canvas: one section at a time, measured, with every unclear detail resolved in a fixed order rather than guessed.

The source skills generate an image first and then build code from it. You work the other way round and only the second half applies: the person's picture is the reference, the canvas is the translation. The method is the same: analyse deeply, extract exactly, never drift into a generic layout while translating.

## What the picture is for

Decide first, because it changes how faithful you are:

| The picture | Your job | Faithfulness |
|---|---|---|
| The person's own current site or app | Redesign it (`craft-redesign`): extract content and brand, then improve | Content and brand exact; layout may change |
| A site they like ("bikin seperti ini") | Learn its principles for their business | Rhythm, density, type mood; never its logo, words, photos or exact composition (Taste: never a copied site) |
| A photo of their printed menu, price list, brochure, signboard | Extract the real content and brand colours | Text and prices exact; layout is yours |
| A sketch on paper | Their intended structure | Order and grouping exact; style is yours |
| A screenshot of a WhatsApp chat or an order | Real wording and the flow customers use | Wording exact |

## 1. Split into sections before reading

Never read a long screenshot as one picture. Mark the section boundaries (a ground change, a large gap, a new heading) and treat each as its own unit, top to bottom. If a section is too small to read (tiny text, buttons you cannot measure), ask the person for a closer screenshot of that part instead of guessing. One clear crop per section beats one blurry full page.

## 2. Extract, per section

Write these down for each section before drawing it:

- **Text:** every readable heading, line, button label, nav item, price and footer label, verbatim. Text you cannot read is marked unknown, not invented.
- **Type:** serif or sans, the face if you recognise it (if not, the closest Taste pairing), relative sizes (h1 about 2.4x the lead), weights, line count of each heading, tracking (tight, normal, wide), case.
- **Spacing:** headline to line, text to buttons, card gaps, section top and bottom, side margins, card padding. Express each as a step on the spacing scale (`$space-*`), not as raw pixels.
- **Buttons and components:** height, radius, fill or outline, icon, primary versus secondary, card structure, dividers, borders, shadows, badges, input style.
- **Colours:** ground, panels, text levels, accent, button fills, borders, the tint of the photos. Pick values from the flattest area, never an anti-aliased edge; round to tokens by role (`components-kit`).
- **Grid:** columns, content edges, which blocks align to which, the container width.
- **Pictures:** each image's ratio, size on the grid and subject, so you can draw a placeholder with an honest note.
- **Rhythm:** how density and alignment change from section to section.

The goal is the logic of the spacing and scale, not exact pixel OCR: a 41px gap that is clearly "the same as the card gap" becomes `$space-10` everywhere.

## 3. Name the system in words

Before drawing, write the system the way `craft-taste-variants` section 6 describes: atmosphere, colours by name and job, type, components, layout. If the person can read it and say "ya, itu" you have understood the picture.

## 4. Draw, section by section

- Tokens first, then components, then sections in the picture's order.
- Media in fixed, repeatable frames: one ratio per row, the same radius for the same kind of image.
- Reuse ids: if you redraw the same section later, the same elements keep their ids.

## 5. When something is unclear: the order

1. Keep the visible design language (what you can see wins).
2. Keep the layout and spacing logic of the rest of the page.
3. Keep the component family (a new button looks like the buttons you saw).
4. Keep the mood and the level of polish.
5. Ask for a closer screenshot of that part.
6. Only then choose the plainest option that fits, and list it in the handoff as an assumption.

## 6. Do not drift

The common failure: the reference is distinctive, the canvas comes out generic. While translating, never:
- swap a distinctive section for a stock row of three cards;
- compress generous spacing into a tight default;
- flatten a strong type contrast into a polite scale;
- add boxes, pills, eyebrows or "Section 01" labels the reference didn't have;
- add boxes inside boxes: one framing move per section.

## 7. Taste still decides

A reference can break Taste rules: a centred dark hero on a bakery, a carousel, a gradient, a caption under every photo, a violet accent. Keep its principles, drop those parts, and tell the person in one line what you changed and why ("carousel diganti tiga foto tetap; pengunjung di HP jarang menggesernya").

## Chats and transcripts (from chat-motion-overlay)

When the picture is a chat (WhatsApp orders, customer questions you will turn into an FAQ, a conversation to show as proof on a page):
- Turn it into one message per line: speaker, side (left or right), exact words, time if shown.
- Use the avatars only to count participants and group messages; never crop or redraw a person's avatar from a screenshot. Draw a neutral avatar (initials on a tone) unless the person supplies the image.
- Keep exact wording where authenticity matters (a real customer message), and ask before showing a real customer's name or number on a page: blur it to initials by default.
- Taste: a chat screenshot is never a testimonial unless the person confirms the customer agreed to be shown.

## Checklist before you present

- Every section of the reference is accounted for (drawn, merged, or deliberately dropped with a reason).
- The readable text is exact; unknowns are listed.
- Type, spacing and colour come from tokens, not one-off values.
- At 390 the result still reads as the same design.
- Nothing from the reference that Taste forbids survived.

Adapted from Taste Skill's image-to-code and Stitch Design Taste (MIT, Copyright (c) 2026 Leonxlnx) and open-design's chat-motion-overlay, via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
