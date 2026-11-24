---
name: practice-anthropic-canvas-design
description: "The design-philosophy method for a single composed piece: name a visual movement, write how it shows in space, colour, scale and type, hide a quiet reference to the subject, then compose and refine without adding. Read for a poster, a type-led hero or a statement moment."
license: "Apache-2.0 (anthropics/skills); via open-design (Apache-2.0)"
source: "https://github.com/anthropics/skills@e4bb5d9 (skills/canvas-design); open-design@5fdcc3d"
---
# Practice: composing one strong piece (Anthropic canvas-design)

The design-philosophy method for one composed, mostly visual piece: name a visual movement, write how it shows in space, colour, scale and type, weave in a quiet reference to the subject, compose it with almost no words, then refine what is there instead of adding.

## Where you use it

The source makes art posters as PNG or PDF. You draw on the OpenPencil canvas, so use the method for the few places a design is one composition rather than a layout of parts:

| Where | What it becomes |
|---|---|
| A poster or flyer frame the person asks for (promo for a grand opening, an event, a menu board, an Instagram post at 1080x1350) | One top-level frame at the asked size, composed with this method |
| A type-led hero (a place or service with no strong photograph, a studio name set huge) | The hero's composition and type treatment |
| A statement moment section (one sentence set as the page's moment) | Its scale, placement and rhythm |
| A cover screen for a portfolio, a case study, a launch page | Its first screen |

Everywhere else, Taste's sections and rules decide. Taste wins on pages: on a place, product or service page there are no patterns, textures, grain or decorative marks on the interface (`taste-color`, `taste-photos-motion`), photographs carry the page, and moments are photo- or type-led. A standalone poster frame isn't a page, so the visual language below may go further there, still in solid fills only.

## Step 1. Write the visual philosophy

Before drawing, write a short philosophy (in your plan, not on the canvas): an aesthetic stance that the piece will express visually, about 90% image and form, 10% essential words.

1. **Name the movement** in one or two words: "Beton Puisi", "Sunyi Kromatik", "Ritme Pasar".
2. **Say how it shows** in four to six short paragraphs, each aspect once:
   - space and form;
   - colour and material;
   - scale and rhythm;
   - composition and balance;
   - visual hierarchy.
3. **Leave room.** Be specific about the direction, brief enough that the drawing still makes real choices.
4. **Hold the craft bar.** The piece must look as if it took many hours from someone at the top of the field: every alignment refined, every gap deliberate. Write that standard into the philosophy and judge the result by it.

Condensed examples of philosophies, as a menu of directions:

| Movement | Stance | How it shows |
|---|---|---|
| Concrete poetry | Communication through monumental form and bold geometry | Massive colour blocks, sculptural type (one huge word, tiny labels), hard spatial divisions; words as rare, powerful gestures, never paragraphs |
| Chromatic language | Colour as the information system | Geometric colour zones that carry meaning; minimal small sans labels; words only to anchor what colour already shows |
| Quiet contemplation | Breathing room and attention | Vast empty space, one photograph or object dominating, type whispered small at an edge; short phrases only |
| Organic systems | Natural clustering and modular growth | Rounded forms, grouped clusters, colour from nature; information shown by arrangement, labels floating in space |
| Geometric silence | Pure order and restraint | Strict grid, one bold photograph or stark shape, dramatic empty zones; small precise type; structure speaks |

Two of the source's examples lean on paper grain and ink bleed; the canvas has solid fills only (`pen-format`), so texture becomes a photo placeholder note or is left out.

## Step 2. Find the quiet reference

Before composing, pick the subtle thread from the brief: the subject woven into the form, not announced. Someone who knows the subject feels it; others just see a strong composition. Like a musician quoting another song: only those who know will catch it.

Examples for Nomi's clients:

- A batik studio's opening poster: the grid's proportions follow a parang motif's diagonal rhythm, drawn as plain bars, never a copied batik pattern.
- A kopi roastery: the colour zones step from green bean to dark roast in the order of a roast.
- A tailor (penjahit): a single long rule across the piece, ticked like a measuring tape, with the shop name sitting on it.

## Step 3. Compose

- **One frame, highly visual.** Repeating shapes and perfect geometry; marks that build meaning through patient repetition reward a longer look.
- **Limited palette** that feels intentional: three or four colours, all tokens or named hexes in the plan. Taste wins on hue: never violet; a place's poster uses its trade palette or the client's colours (`taste-color`).
- **Text is an element of the composition.** Context sets its voice: a gig poster can shout in huge heavy type; a ceramics studio whispers small and light. Mostly, the type is thin and quiet.
- **Typography is part of the art.** Bring the type into the composition (set it huge, cropped by the frame edge only if the frame clips on purpose, aligned to the geometry), not typeset as an afterthought. One or two families: the source says to try different fonts, but Taste wins: two at most, from `taste-type`.
- **Nothing overlaps, nothing falls off.** Every element sits inside the frame with proper margins and clear separation. This is not negotiable. (A deliberate crop at the frame edge is a choice you note; an accidental one is a defect.)
- **Words stay few:** the name, the one fact (date, place, price), the action. "Buka 12 Juli · Jl. Kaliurang, Sleman" carries a grand-opening poster; nothing else needs saying.
- **Sophistication always.** Even for a game, a kids' class or a festival, the piece is designed, never cartoonish or amateur.

On the canvas: build the composition as a frame with `layout: "none"` where you place shapes by hand, rectangles and ellipses for the geometry, `path` nodes for drawn marks, text nodes with `fixed-width` growth for every line that wraps, and photo placeholders (tone plus note) where a photograph belongs.

## Step 4. Refine, don't add

Assume the person has already said: "Belum cukup. Harus rapi sempurna, seperti mau dipajang di museum."

- Don't add graphics. Refine what exists until it's crisp and fully true to the philosophy.
- If your instinct is to draw a new shape or add an effect, stop and ask instead: "How can I make what's already here more of a piece of art?"
- Take a second full pass: alignments, the spacing between every pair of elements, the type's size steps, the colour balance. Then look at the preview, not the JSON.

## More than one frame

When the person wants a set (a poster plus feed posts, a series of three covers), make each frame a distinct twist on the same philosophy, like pages of one book: same movement, same palette and type, a new arrangement and a new detail in each. Together they should almost tell a story.

## Check

- [ ] Philosophy named and written; each aspect said once.
- [ ] The quiet reference is there and not announced.
- [ ] Mostly visual; words are the name, one fact, one action.
- [ ] Palette of three or four, no violet; one or two type families.
- [ ] Nothing overlaps, nothing falls off; margins even.
- [ ] Second pass done by refining, not adding.
- [ ] On a page section: Taste's limits for the kind still hold.

Adapted from anthropics/skills canvas-design (Apache-2.0, anthropics/skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
