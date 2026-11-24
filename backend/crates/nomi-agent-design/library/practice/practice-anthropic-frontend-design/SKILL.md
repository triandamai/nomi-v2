---
name: practice-anthropic-frontend-design
description: "How a studio design lead gets to a distinctive design: ground it in the client's subject, write a short token plan, test it against the five clusters AI design falls into, revise, then spend boldness in one place and cut one accessory. Read when a plan feels generic."
license: "Apache-2.0 (anthropics/skills); via open-design (Apache-2.0)"
source: "https://github.com/anthropics/skills@e4bb5d9 (skills/frontend-design); open-design@5fdcc3d"
---
# Practice: distinctive design from the subject (Anthropic frontend-design)

How a studio design lead gets to a design nobody mistakes for a template: ground it in the client's subject, write a short token plan, check it against the five looks AI design falls into, revise, then build with restraint and one bold move.

Act like the design lead at a studio known for giving every client its own identity. This client has already turned down proposals that felt templated and is paying for a point of view. Taste Skill is still your rulebook: it decides the kind, the palette and the type by trade. This file is the thinking that keeps those choices specific to this client.

## 1. Ground the design in the subject

- If the brief doesn't say what is sold, to whom, and what the page must do, name one concrete subject, audience and main job yourself as a proposal and say it back to the person ("Saya anggap ini toko roti rumahan di Kotagede, pembelinya keluarga sekitar, tugas halaman: pesan kue untuk acara").
- Distinctive choices come from the subject's world: its trade, materials, tools, vernacular and local words. A booking page for a batik studio and a dashboard for a koperasi treasurer should not look related.
- Use the brief's real content everywhere. Never draw with placeholder text (`taste-copy`).

## 2. Open with the most characteristic thing

The hero is the first thing seen. Open with the most characteristic thing in this subject's world, in the form that fits it: the product, the room, the work, the real UI, a headline built from type. Be deliberate:

- "A big number with a small label, supporting stats and a gradient accent" is the default AI hero. Use it only if it truly is the best option (it rarely is; gradients can't be drawn here anyway).
- Taste wins on what a hero may hold for each kind (`taste-core`, `taste-sections`).

## 3. Typography carries the personality

- One family or two. If two, make them clearly different (a serif display with a sans body, not two similar sans).
- Choose faces on purpose, never the family you'd reach for on any project. Set a clear scale with intentional weights and spacing; when type is the headline or the visual, make the type treatment part of the design, not a neutral carrier. Taste's pairings by kind are the menu (`taste-type`).
- Lines under 80 characters (Taste: 62 at 1440). A serif body may run slightly longer and gets a little more line height than a sans.
- Avoid the commonest typographic tells:
  - one word or phrase in a heading set in italic, bold or another colour;
  - all-caps labels;
  - small labels above content that add nothing.

## 4. Visual structure is information

Outlines, borders, numbers, eyebrows, dividers and labels must say something about the content, not decorate it.

- Numbered markers (01 / 02 / 03) only when the content really is a sequence: steps to order, a timeline. Check before you number.
- A divider separates two different things; it isn't texture.
- A card exists because it is one clickable or comparable thing (`taste-cards-icons`).

## 5. Motion: one orchestrated moment

Motion nobody triggered is used sparingly and only to draw attention. One page-load sequence or one reveal lands better than scattered effects. Fade-and-slide-up on every section and hover effects on every card read as AI-made. Motion that answers the person (opening, expanding, confirming) is welcome when it shows what changed.

On the canvas you draw the finished state and annotate the motion for Koda. Taste wins on what motion each kind may carry (`taste-photos-motion`).

## 6. The five clusters AI design falls into

Check your plan against these. Each is fine for some briefs, but they appear whatever the subject, so they are defaults, not choices:

| # | Cluster | What it looks like |
|---|---|---|
| 1 | Warm cream editorial | Cream ground near `#F4F1EA`, high-contrast serif display, terracotta or clay accent near `#D97757` |
| 2 | Black with one acid accent | Near-black ground with a single bright acid-green or vermilion accent |
| 3 | Broadsheet | Hairline rules, zero radius, dense newspaper-like columns |
| 4 | SaaS card kit | Everything chopped into identical rounded cards, one radius on everything whatever the hierarchy, the same soft grey shadow under each, gradient washes as decoration |
| 5 | Template chrome | Tracked-out ALL-CAPS eyebrow above every heading; meta joined with middle dots ("A · B · C"); labels like "WORD — fragment"; tinted near-black standing in for black; a monospace face for small data labels; "→" added to every link and button |

- Where the brief pins down a look, follow it exactly, even if it is one of these. The brief's own words win.
- Where an axis is free, don't spend that freedom on one of these defaults.
- Note for Nomi: Taste's palettes may land near cluster 1 for a bakery (a flour ground and an oven-tile terracotta). That is fine when the trade answers it and you can say why on plan line 2; it is a tell when it comes from habit.

## 7. Work in two passes

**Pass 1, the plan.** Before any frame, write a compact system:

- **Colour:** the base palette as 4-6 named hex values with their token roles (`$color-background`, `$color-foreground`, `$color-muted`, `$color-primary`, `$color-border`, maybe `$color-inverse`).
- **Type:** the faces and their roles (`$font-heading`, `$font-body`), with the scale.
- **Layout:** the concept in one sentence per screen and a rough block sketch; say the alignment (left, centred) and why.
- **Principles:** the two or three things that make this page this client's.

**Review the plan against the brief.** For each part ask: would I have produced this for any similar brief? Run a similar prompt in your head ("toko roti lain di kota lain"): if you land in the same place, that part is a default. Revise it and say what you changed and why. Only then draw the tokens, components and screens.

On the canvas, the plan lives in Taste's plan lines 1 and 2 (`taste-core`); this review is the check you run on line 2 before you deal it.

## 8. Restraint and self-critique

- **Spend boldness in one place.** One element is the memorable thing (the hero photograph at full width, a huge set name, the one moment); everything around it stays quiet and disciplined.
- Cut decoration that doesn't serve the brief.
- Build to a quality floor without announcing it: frames at 390, 768 and 1440, a visible focus state on every control, contrast that passes, a harmonious palette.
- Critique as you go: look at the saved preview, not your JSON. A picture shows what a scene graph hides.
- Chanel's rule: before leaving the house, look in the mirror and take one accessory off. Before you save, remove one thing.
- Keep a short note in your plan of what you tried in this conversation, so the next design deals something new.

## 9. Words are design content

Words exist to make the page easier to understand and use. Bring the same restraint to them as to spacing and colour.

- Write from the visitor's side, in words they use: "Atur notifikasi", not "Konfigurasi webhook". Describe what a thing is or does; don't sell it. Specific beats clever.
- Active voice. A button says exactly what happens: "Simpan perubahan", not "Kirim". An action keeps its name through the flow: the button "Terbitkan" leads to the toast "Sudah terbit".
- Failure and emptiness give direction, not mood. Say what went wrong and how to fix it, in the interface's voice. Errors don't apologise and are never vague: "Nomor WhatsApp kurang satu digit. Periksa lagi." An empty screen invites the next action: "Belum ada pesanan. Bagikan tautan toko Anda."
- Plain verbs, sentence case, no filler; tone matched to the brand and the buyer. Each piece of text does one job.
- `taste-copy` has the full rules; where they're stricter, Taste wins.

## Quick check before you save

- [ ] Can I name the subject, buyer and one main job in one line?
- [ ] Does the hero show the most characteristic thing in this subject's world?
- [ ] One or two typefaces, chosen for this client, with a clear scale?
- [ ] No single-word accent in a heading, no all-caps labels, no empty eyebrows?
- [ ] Every number, divider and card means something?
- [ ] Which of the five clusters is this closest to, and is that a choice I can defend from the brief?
- [ ] One bold thing, everything else quiet; one accessory removed?
- [ ] Every button says what happens; errors and empty states say what to do?

Adapted from anthropics/skills frontend-design (Apache-2.0, anthropics/skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
