---
name: craft-motion-notes
description: "What goes inside a motion note for Koda: trigger and values, the stronger named curves, the vocabulary for naming an effect, recipes for common components and the checks a note must pass; the format and durations are in motion-timing. Read it whenever you annotate motion."
license: "MIT, Copyright (c) 2026 Emil Kowalski (animate, review-animations, emil-design-eng); MIT, Copyright (c) 2026 Matthew Blode (ui-animation); MIT, Copyright (c) 2026 GreenSock (gsap-* principles); via open-design (Apache-2.0)"
source: "https://github.com/emilkowalski/skills @e8a175d and @a47903a; https://github.com/mblode/agent-skills @5a781a8; https://github.com/greensock/gsap-skills @aed9cfd; open-design@5fdcc3d"
---
# Motion notes for Koda

What goes inside a motion note so Koda can build it without guessing: the trigger and values, the right curve, the name of the effect, recipes for common components, and the checks a note must pass.

Decide first with `craft-motion-design` (should it move, and why). The note's format, durations and defaults are in `motion-timing`; this skill does not restate them.

## The note format

Use the one-line format defined in `motion-timing` exactly (element id, trigger, what moves from → to, duration and curve or spring with the phone duration, reduced substitute, why), in a `motion-note` frame beside the screen and in the handoff. Taste's page-level summary line opens the list (`Motion · hero entrance: … · moment: … · hover: …`, from `taste-photos-motion`); a component in the design system gets one note covering all its states.

This skill adds what goes inside the fields:
- **Element:** the canvas id, so Koda finds it.
- **Trigger:** one of load (once), tap, hover, focus, press, release, scroll into view (once), state change, drag.
- **From → to:** transform and opacity values and the origin: `translateY 100% → 0, opacity 0 → 1, origin bottom`. Use a name from the vocabulary below when one fits.
- **Duration and curve:** from `motion-timing`; the curve names below when a stronger one is wanted.
- **Why:** one purpose word from `craft-motion-design` (feedback, orientation, continuity, state, explanation, delight).

Bad notes: "make it smooth and nice", "add a subtle animation", "fade in on scroll".

## Curves

`motion-timing` sets the default: a standard ease-out for opacity and colour, a critically damped spring for position, scale and drag. Emil Kowalski's skills argue the built-in curves are too soft; where a motion should feel decisive, name one of these stronger variants and Koda maps it to a token. Never invent a curve.

| Name | Use | Value |
|---|---|---|
| ease-out strong | Entrances, exits, hovers that move | cubic-bezier(0.23, 1, 0.32, 1) |
| ease-in-out strong | Something already on screen moving from A to B | cubic-bezier(0.77, 0, 0.175, 1) |
| drawer curve | Sheets, drawers, side panels (very steep start, so a longer time doesn't feel slow) | cubic-bezier(0.32, 0.72, 0, 1) |
| expo out | A dramatic reveal (hero, a dealt moment) | cubic-bezier(0.19, 1, 0.22, 1) |
| linear | Constant motion and progress only: a spinner, a hold-to-confirm fill, scroll-scrubbed motion | linear |

Never ease-in on interface motion: it starts slowly at the exact moment the person is watching.

## Springs

A spring replaces a duration when the motion follows a finger (drag, swipe to dismiss), should feel alive, or can be reversed mid-way. Bounce follows `motion-timing`: none in business apps, a little only for a playful brand, and only on drag release or rare moments.

## Recipes for common components

Durations come from `motion-timing`; the shape of each is fixed:

| Component | Note shape |
|---|---|
| Button | press: scale .97 (Taste: 1px down and .985 on the page layer), hover: 1px up; release snaps back |
| Dropdown, popover, select | origin at the trigger; scale .95 + opacity 0 → 1; ease-out strong; exit faster |
| Tooltip | as popover at scale .97; after the first opens, neighbours open instantly with no animation |
| Dialog | origin centre; scale .96 + opacity; backdrop fades with the same timing |
| Sheet, drawer | translate 100% of its own size from its edge; drawer curve; follows the finger; momentum dismiss |
| Toast | from the edge it lives on; leaves the same way; pauses its timer while hidden or hovered |
| Accordion | height and opacity together, short; the one place height may animate |
| List entrance (occasional screens) | 30-50ms stagger between items, total under 300ms, most important first; never blocks taps |
| Hold to confirm (delete) | fill grows linear while held; snaps back fast on release; button pressed .97 |
| Tabs | the active background slides under the labels; or a clipped copy reveals the new active label |
| Number that changes | the new figure rolls in (odometer) when the person drives it, pops in when it arrives on its own |
| Error on a field | a short horizontal shake once, then the message; reduced: no shake |
| Success | the button morphs into "Tersimpan" with a check, holds, returns |

## Vocabulary: name the effect, don't describe it

| You mean | Write |
|---|---|
| A panel grows out of the button you tapped | origin-aware scale-in |
| The button itself becomes the form | container morph |
| A thumbnail flies into the detail page | shared element transition |
| One image fades over another in place | crossfade |
| Content slides left going forward, right going back | direction-aware transition |
| Items appear one after another | stagger |
| Content is uncovered by a moving edge | clip reveal (wipe) |
| Resistance when dragging past the end | rubber-banding |
| A slider that clicks into fixed stops | detents |
| Everything around the focused card fades back | peripheral de-emphasis |
| Animation tied to the scroll position itself | scroll-driven (scrubbed) |
| Elements reveal once as they enter the screen | scroll reveal (once) |
| A line draws itself | line drawing (stroke reveal) |
| Overshoot then settle | spring with bounce |

If nothing fits, describe it in these words ("a stagger of origin-aware scale-ins") rather than inventing a name.

## Sequencing several things (timeline thinking)

For the hero entrance, a moment or a demo, write the sequence as a numbered list with relative starts, the way an animation timeline works:

1. `hero-title` lines rise, each line after the last.
2. `hero-lead` and `hero-actions` start when the title's last line starts, fade up 12px.
3. `hero-photo` settles from 1.05 to 1 together with step 1.

Relative words: "with 1", "after 1", "0.2s before 1 ends". Labels for beats in a longer piece. Everything respects one master order; nothing independent loops alongside it.

## Scroll motion (tech pages and dealt moments only)

- Scroll reveal runs once and never replays on scroll up.
- Scrubbed motion (tied to the scrollbar) uses linear and no duration; pinning holds only the one section that needs it.
- Never on above-the-fold content, product UI the person uses, or every section.
- Phones and reduced motion get the finished still (Taste).

## Reduced motion

Reduced means fewer and gentler, not zero: keep opacity and colour changes that help understanding; remove travel, scale, parallax, loops and auto-playing sequences. Every note has a `reduced:` field, even if it says "no change (colour only)".

## Hover and touch

Hover notes apply to mouse and trackpad only; on touch the same element gets its press state. Never put anything essential behind hover.

## Check every note before handing off

- [ ] It has a purpose word and passes the frequency test.
- [ ] Nothing animates on a keyboard action or a many-times-a-day action.
- [ ] Only transform and opacity move (accordion height is the exception; colour may change).
- [ ] Entrances start from about .95, never from zero size; popovers have their origin at the trigger.
- [ ] Interface motion stays within the `motion-timing` budget; exits are faster than entrances.
- [ ] No ease-in; curves are from the named list; springs only where something follows a finger or must stay interruptible.
- [ ] Paired elements share timing; open and close both noted.
- [ ] Rapidly repeated actions can reverse from where they are.
- [ ] Every note has a reduced-motion fallback.
- [ ] It fits Taste's budget for the page kind.

When in doubt whether a motion feels right, the strongest note is often to delete it.

Adapted from Emil Kowalski's animate, review-animations and design-engineering skills (MIT, Copyright (c) 2026 Emil Kowalski), Matthew Blode's ui-animation (MIT, Copyright (c) 2026 Matthew Blode) and GreenSock's gsap-skills (MIT, Copyright (c) 2026 GreenSock), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
