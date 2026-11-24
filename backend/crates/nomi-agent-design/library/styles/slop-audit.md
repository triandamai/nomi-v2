# Slop Audit

What makes a design read as AI-made, and the audit you run on your own canvas before you save a version: name each suspect element, ask what job it does, delete what has none, and fix the rest with the system you already chose.

## The core judgment

Slop is not a colour, a font, a card or a shadow. It is a choice made by reflex instead of for this business. Treat an element as suspect when several of these are true:

- it could be pasted into an unrelated business unchanged (a salon page and a bengkel page would share it)
- it repeats a familiar generated pattern (eyebrow, icon tile, bento, "Trusted by" row)
- it conflicts with the type, colour, radius or spacing you set elsewhere on the canvas
- it carries no information, state, action, hierarchy or brand meaning
- it competes with the content, weakens trust, or makes the page harder to use

Do not guess whether something "looks AI". Judge the visible result. Keep the chosen direction: an audit removes what has no job, it does not replace a bold style with a safe neutral one.

## When to run it

- Before every saved version of a page or screen. Not after the person complains.
- After adding any section, component or effect: run the quick gates below on that piece.
- Before presenting 2–4 style directions: each one must pass on its own, not just look different.

Audit the rendered canvas (a screenshot or export at desktop 1440 and phone 390 widths), not your memory of the JSON. Sizes, collisions and contrast only show when drawn.

## Quick gates (every new piece)

| Gate | Pass when |
|---|---|
| System | Uses only your `$tokens`: type roles, colours, spacing steps, radii, stroke, shadow |
| Hierarchy | The most important thing is obvious without an eyebrow, pill or number |
| Composition | Edges align, related items sit closer than unrelated ones, insets look even |
| Type | Roles are distinct, body ≥ 16px, labels ≥ 11px, no forced line breaks in body copy |
| Colour and material | Each accent, border and shadow has a job you can name |
| Truth | Copy, prices, names, reviews and photos are real, given by the person, or marked as placeholders |
| States | Selected, empty, error, disabled and success exist where the flow needs them |

## The removal test

For every suspect element:

1. Name it precisely: tracked eyebrow, icon tile, nested card, hero pill, fake rating, double edge, mono micro-label.
2. Say the job it does.
3. Remove it in your head. If meaning, action, hierarchy and character survive and the page gets clearer, delete it.
4. If deleting loses something real, make the smallest fix with the existing system.
5. Add a replacement only if the page truly needs one. Never compensate with a new effect.

Default to subtraction.

## Checklist by area

### Direction
- No clear business, audience or main action in the first viewport.
- The style could belong to any startup if the name were swapped.
- Two or more style lanes mixed with no lead (beige editorial hero, then neon dashboard cards).
- A section that does not inherit the page's type, palette, radius or density looks imported.
- Several focal points per viewport, none dominant.

### Composition and space
- Every section has the same vertical padding regardless of content. Vary it: 64–96px for reading sections, 40–56px for dense lists.
- Headings sit closer to the previous section than to their own content.
- One column runs far longer than its partner and leaves a dead block.
- Body text touches the frame edge; phone frames need ≥ 16px side padding.
- Lines longer than about 70 characters; centred paragraphs longer than two lines.
- A dashboard drawn with marketing spacing (huge gaps, three numbers per screen).
- The phone layout is the desktop stacked in one column with nothing re-prioritised.

### Containers and shape
- Every block is a card. Use proximity, alignment and a 1px divider first.
- Cards inside cards with no real hierarchy.
- A bento grid for content of equal importance.
- Identical icon + heading + two-line description tiles, three or six in a row.
- Extreme radii on small cards (24px on a 120px tile); radii that drift between siblings.
- A stroke and a broad soft shadow both drawing the same edge. Pick one.
- A thick accent stripe on one side of a rounded card.
- Pills for things that are not status, tag, filter or action.

### Typography
- A tracked uppercase eyebrow above almost every heading.
- Heading, subheading and body too close in size; use at least a 1.25 step between roles.
- Display tracking so tight letters touch; body tracking widened.
- Long copy in all caps; tiny mono labels making ordinary content look "technical".
- One default sans at one weight for every role. If you use Inter or a system sans, it must be a decision, not the fallback.
- Decorative italics used only to make a headline feel editorial.
- Buttons whose label wraps to two lines.
- A single weak word left alone on the last line of a headline.

### Colour and material
- Three or more accents with no roles. One accent, plus semantic colours for state.
- Grey text on a coloured fill that looks washed out; contrast below 4.5:1 for body, 3:1 for large text.
- A warm cream background picked only because it reads as tasteful.
- Shadows on everything; shadows that are muddy, wide and unrelated to any elevation model.
- Dark mode as black plus neon instead of a full tonal ramp (page, panel, raised, stroke, text, muted).
- Colour as the only signal of state; add an icon, label or weight change.
- On this canvas there are no gradients or blur. Do not fake them with stacks of semi-transparent rectangles.

### Imagery and icons
- A photo placeholder with no caption. Every placeholder names the photo it stands for, e.g. "Foto: Siti Rahayu memotong rambut pelanggan, cahaya jendela".
- Abstract shapes posing as an illustration with no link to the business.
- A fake dashboard, browser window or phone mockup that shows nothing true about the product.
- A rounded-square icon tile above every feature heading.
- Icons from more than one family, or with mixed stroke weights.
- Invented logos of partners or customers.

### Components and states
- Two or three primary buttons of equal weight in one view.
- Tabs, nav or filters with no clear selected state, or a selected state that is just an uneven outline.
- A status pill or pulsing dot that implies live data the page does not have.
- Buttons with different heights, padding or radii for the same job. Tap targets under 44px.
- Destructive actions styled like routine ones.
- Missing empty, error, loading and success frames for a flow that needs them (booking, checkout, form).

### Copy, proof and structure
- Headlines that fit any business: "Solusi terbaik untuk kebutuhan Anda", "Tingkatkan bisnis Anda ke level berikutnya".
- Vague verbs on buttons ("Pelajari lebih lanjut", "Kirim") when a specific one exists ("Pesan meja", "Cek jadwal kosong").
- Labels, helper text and headings that repeat each other.
- The default sequence regardless of the business: hero, logo wall, three stats, bento features, testimonials, FAQ, CTA. Choose sections from how this person's customer actually decides.
- Invented proof: "4,9 dari 2.000+ pelanggan", "Dipercaya 500 bisnis", testimonials from "Budi S., CEO". Use proof the person gave you or a marked placeholder ("[Ulasan pelanggan asli]").
- Prices with no unit or period ("Rp 249.000" — per bulan? per sesi?).
- Empty states with marketing copy instead of the next action.
- Lorem ipsum, "Your Name", English filler on an Indonesian site.

## High-signal clusters

Do not count single matches. Look for a cluster with one root cause and fix the cause once.

- **Generic AI SaaS:** tracked eyebrow, oversized sentence headline, purple or blue accent everywhere, icon tiles, bento, vague copy.
- **Card apocalypse:** section card around feature cards around pills, double edges, 24px radii everywhere.
- **Fake sophistication:** mono micro-labels, decorative grid, fake dashboard, invented metrics, technical copy with no evidence.
- **Reference mismatch:** a section with alien fonts, radii or accent, borrowed from another style.
- **Polish before truth:** beautiful surfaces over unclear copy, weak actions, missing states, unverified proof.

## Priority

- **P0:** blocks the task, unreadable text, or deceptive proof.
- **P1:** hurts comprehension, trust, navigation or the main action.
- **P2:** repeated slop or inconsistency that weakens hierarchy and identity.
- **P3:** minor polish.

Fix all P0 and P1 before saving. Fix P2 if it is a cluster.

## Write it down

Before saving, note the five to eight highest findings in a short table: priority, pattern, where on the canvas, harm, fix. Group repeated instances into one row. End with the single removal that improves the page most. Keep it to yourself unless the person asks; tell them only what changed and why in one or two lines.

## Name the fix precisely

Precise names keep revisions surgical:

- "Remove the tracked eyebrow; the heading already carries the hierarchy."
- "Flatten the nested cards; group with 12px gaps and one divider."
- "Replace the icon tiles with the real menu photos, captioned placeholders for now."
- "Make the status static; there is no live data."
- "Adapt the pricing block to the page's radius, palette and type."

## Boundaries

- Do not treat a technique as slop by itself. A serif, a dark theme, a card or a single font can be right.
- Do not remove useful density, humour, asymmetry or edge because it is unusual.
- Do not add sections, colours, fonts or effects during a cleanup unless they fix a real loss.
- Never invent customers, ratings, awards, metrics or activity.

Adapted from mengto/skills: agent-skills/ui/no-ai-design-slop, agent-skills/ui/audit-ai-design-slop (MIT, Copyright (c) 2026 Meng To).
