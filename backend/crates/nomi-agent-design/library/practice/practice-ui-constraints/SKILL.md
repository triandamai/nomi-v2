---
name: practice-ui-constraints
description: "Must, should and never constraints that keep many small UI pieces coherent, accessibility as canvas checks, cheap motion, titles, share cards and icons, and an evidence-first audit of an existing screen. Read when you build components or audit a screen."
license: "MIT, Copyright (c) 2026 Julien Thibeaut (ibelick/ui-skills); via open-design (Apache-2.0)"
source: "https://github.com/ibelick/ui-skills@7ef0076 (skills baseline-ui, fixing-accessibility, fixing-motion-performance, fixing-metadata, improve-ui + references/plan-template); open-design@5fdcc3d"
---
# UI constraints: a baseline against slop

Hold every screen to a short list of musts and nevers, so dozens of small pieces drawn over a conversation still look like one product.

Rules from the source written for code (Tailwind classes, React libraries, hooks) are translated here into what you draw and what you annotate for Koda.

## Baseline

**Components**
- Use the project's existing components first: the design system frame's reusable frames (`components-kit`). Never mix two families of the same control on one surface (two different dropdown styles in one form).
- Anything with keyboard or focus behaviour (menus, dialogs, tabs, comboboxes) is drawn as its shadcn component so Koda uses an accessible primitive; never invent a custom control that needs hand-built keyboard behaviour.
- Icon-only buttons get a label in a canvas note ("Label: Tutup") so Koda names them.

**Interaction**
- Destructive or irreversible actions open an `Alert Dialog / Destructive / Default`, never a one-tap delete.
- Loading states use structural skeletons shaped like the content.
- Full-height screens use the real visible height on phones (annotate "tinggi layar dinamis"), and fixed bars respect the notch and home-indicator areas.
- Errors appear next to where the action happened, not in a banner far away.
- Never block pasting into fields (note it on OTP and password fields).

**Motion**
- No animation unless it is asked for or Taste's layer calls for it (hero entrance, the moment, hover).
- Animate only movement and opacity; never width, height, position or padding.
- Ease-out on entrance; interaction feedback no longer than 200 ms.
- Looping motion pauses when off screen; reduced-motion respected.
- No custom easing curves unless asked; avoid animating large photos or whole screens.

**Typography**
- Headings set to balanced wrapping, paragraphs to "pretty" wrapping (annotate for Koda; on the canvas, avoid one-word last lines by setting widths).
- Tabular figures for data, prices and timers.
- Dense UI truncates or clamps lines, with the full text reachable.
- Don't change letter-spacing unless asked. Taste's display headings take a small negative tracking by design; that counts as asked.

**Layout**
- A fixed layer order (base, sticky bar, dropdown, sheet, dialog, toast); never an arbitrary one.
- Square elements (avatars, icon buttons) drawn with equal width and height.

**Design defaults**
- No gradients unless asked (the canvas can't draw them anyway); never purple or multicolour gradients; no glow as the main affordance.
- One shadow scale; empty states get one clear next action; one accent per view; existing tokens before new ones.

## Accessibility as canvas checks

In priority order:

1. **Names:** every control has a visible label or a noted accessible name; links say where they go (never "klik di sini"); decorative icons are marked decorative.
2. **Keyboard:** every interactive element has a focus state drawn (`$color-ring`); nothing is reachable only by hover.
3. **Dialogs:** draw where focus starts inside a dialog and that Escape or a close button dismisses it; focus returns to what opened it (annotate).
4. **Semantics:** headings in order, lists drawn as lists, tables with header rows.
5. **Forms:** errors under their fields; required fields marked; helper text under the field it helps; a disabled submit says why ("Isi nomor HP dulu").
6. **Announcements:** critical errors aren't only in a toast; loading has visible status text.
7. **Contrast and states:** text and icons pass contrast; disabled isn't shown by colour alone; focus outlines never removed without a visible replacement.
8. **Media and motion:** images have alt text in the note (or "decorative"); no autoplay with sound.

Make minimal, targeted fixes; don't redesign a whole screen to fix one label.

## Motion that stays cheap (notes for Koda)

- Moving and fading are cheap; colour, border, shadow and image changes cost more; size and position changes cost the most.
- Never drive animation from scroll position; scroll-linked motion is out under Taste anyway.
- Measure once, then move; prefer one-shot effects over continuous motion; stop or pause off screen.
- If a moment is too expensive, downgrade the technique (a fade instead of a size change) rather than removing it.

## Titles, share cards and icons (what you design)

- **Page title format:** consistent across the site: "Menu — Roti Bu Sri", short, no keyword stuffing. Write one per screen in the handoff, plus a one-sentence plain description for shareable pages.
- **Share card:** a 1200×630 frame per shareable page (at least the home page): the business name, one line, one real photo, readable as a small thumbnail in WhatsApp. Stable ratio; no tiny text.
- **Icons:** a favicon that reads at 16 and 32, a 180×180 home-screen icon with no transparency, and a browser theme colour chosen on purpose (usually `$color-background` or the brand's dark).
- Preview and staging pages are not indexed (note for Koda).

## Auditing an existing surface (evidence first)

When the person asks "apa yang kurang dari desain ini?" about an existing screen or your earlier version:

1. **Pick one surface** (one task's screens), say which.
2. **Rebuild its system:** the tokens, components and rules that really govern it. Write: surface audited, design sources, documented decisions, which components own what, explicit exceptions ("tidak ada" unless documented).
3. **Prove each finding** three ways: the rule it breaks (a documented decision or a direct contradiction on the same task), that the rule applies to this element, and the one correction that follows. Repetition or "it looks different" is a candidate, not a finding.
4. **Try to falsify each finding** and drop it if the rule doesn't govern it, if the difference is deliberate, if more than one fix is possible, if the fix invents intent, or if another finding covers it.
5. **Report at most three,** ordered by confidence, impact, reach and cost:

| # | Problem | Evidence | Change | Scope | Confidence |
|---|---|---|---|---|---|
| 1 | Two button heights in one form (40 and 44) | `Button / Primary` is 44; the "Batal" button is a detached copy at 40 | replace with `Button / Outline / Default` ref | booking form, 390 and 1440 | high |

Then name one to improve first. If nothing survives, say so plainly; no finding is better than an unsupported one.

6. **For each change the person picks,** write a short plan: the outcome, the evidence, the decision and why, what to reuse (token, component, an existing screen as the exemplar), the frames that change and what must stay the same, what else inherits the change, how you'll check it (states, long content, 390/768/1440), and when to stop and ask (an assumption fails or the scope grows).

Adapted from ibelick/ui-skills (MIT, Copyright (c) 2026 Julien Thibeaut), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
