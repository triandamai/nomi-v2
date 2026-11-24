---
name: craft-diagrams
description: "How to design diagrams that appear on a page or in an app (process steps, flows, how-it-works, simple system maps): when Taste allows them, the shapes, connectors and labels drawn as frames and paths, and when a hand-drawn whiteboard look fits. Read it before drawing any diagram card."
license: "Apache-2.0, nexu-io (html-anything: frame-flowchart-sticky, after HeyGen HyperFrames' flowchart); MIT, Copyright (c) 2025 Muthukumaran Navaneethakrishnan (hand-drawn-diagrams, topic only); via open-design (Apache-2.0)"
source: "open-design@5fdcc3d skills frame-flowchart-sticky (upstream github.com/nexu-io/html-anything) and hand-drawn-diagrams (catalogue stub for github.com/muthuishere/hand-drawn-diagrams)"
---
# Diagrams on pages and in apps

How you draw a diagram that a visitor or app user reads (how an order flows, the steps of a service, what connects to what) so it explains one thing at a glance, built from frames, paths and live text.

Planning boards you draw for the person (user flows, site maps, annotated screens with sticky notes) are in `wireframe-boards`; a whiteboard flowchart as a video frame is in `video-frame-templates`. This skill is for diagrams that ship on the page or inside the app.

## Where diagrams may appear

Taste's kind table (`taste-core`): software and tech pages may carry 0-2 diagram cards (Koda may pulse them, annotated); product and service pages 0-1, still; a place never. A diagram must explain something a photo or the UI can't: a sequence, a relationship, a path through a system. "Our process" with four generic icons is not that.

## Pick the shape

| What you explain | Diagram |
|---|---|
| Steps in order (booking, ordering, onboarding) | A horizontal row of 3-5 numbered steps on desktop, a vertical list on the phone; numbers only because order is the content |
| A decision ("Sudah punya toko online?") | A flow with one split and both branches labelled |
| Many sources into one place (orders from WhatsApp, Shopee, Instagram into one inbox) | Converging lines into one node: the most common software diagram for Nomi's clients |
| One thing feeding several | Diverging lines from one node |
| A loop (restock, reorder) | A cycle of 3-4 nodes, arrows in one direction |
| Parts of a system | Boxes for parts, lines for data, at most 7 nodes |

Five to twelve nodes at most on any diagram; if it needs more, split it or simplify the story.

## Draw it

- **Nodes:** reusable frames (`Diagram / Node`), one size per role, the card radius from the ladder, a hairline stroke or a `$color-muted` fill. Inside: a real brand icon for real products (Taste house rule 7: WhatsApp, Shopee, Google Sheets as full-colour icon tiles with their note, never a drawn look-alike) or one outline glyph, a title (16/500) and at most one short line.
- **Connectors:** `path` nodes with stroke only, 1.5-2px, `$color-muted-foreground` or the ink, round caps; gentle curves (a cubic `C` segment) between nodes in different rows, straight lines in one row. Lines never cross; reroute or reorder nodes until they don't.
- **Arrows:** a small filled triangle `path` (8px) at the line's end, the same colour.
- **Conditional branches:** the canvas has no dashed stroke; use a lighter tone and a label on the line ("jika belum bayar").
- **Labels on lines:** 13px text on a small background-coloured frame so the line doesn't run through the words.
- **Layout:** a `layout: "none"` frame so nodes sit on the grid by coordinates; nodes snap to columns; equal gaps.
- **Highlight:** one node or path in the accent to show where the person's product sits ("Nomi" in the middle); everything else neutral.
- **Phone:** redraw vertically; never shrink a desktop diagram to 390.

## The hand-drawn look

Whiteboard diagrams (hand-drawn strokes, sticky-note colours, a marker font, slight rotation) feel informal and friendly. On a website, Taste overrules most of it: nothing tilted, no extra font families outside the pairing, one accent. Use the sketch look only when it is the brand's own voice (a kids' tutoring brand, a creative workshop) and then: straight-on notes, the pairing's text face, pastel fills from the token set, slightly irregular hand-drawn paths for connectors. Otherwise keep diagrams clean and geometric. The whiteboard style belongs on planning boards (`wireframe-boards`), where it signals "draft".

## Motion notes

On software pages a diagram card may pulse: a dot travels along each incoming line in turn, one loop of a few seconds, pausing off screen; elsewhere it is still. Note it with `craft-motion-notes`; reduced motion shows the still. A line-drawing reveal (connectors draw in order, nodes fade in as each line arrives) may be the page's dealt moment on a tech page.

## Check

- It explains one thing; a stranger gets it in five seconds.
- Real icons for real products; no invented integrations or logos.
- No crossing lines, labels readable at 13px, equal node sizes.
- Drawn separately at 390.

Adapted from html-anything's frame-flowchart-sticky template (Apache-2.0, nexu-io; after HeyGen HyperFrames' flowchart) and the topic of hand-drawn-diagrams (MIT, Copyright (c) 2025 Muthukumaran Navaneethakrishnan), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
