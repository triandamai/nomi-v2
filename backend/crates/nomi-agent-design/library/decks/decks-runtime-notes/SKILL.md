---
name: decks-runtime-notes
description: "Technical note on the deck runtime Nomi serves: navigation keys, entry animations (no fragments), presenter view, theming hooks, and the exact class and attribute names your deck HTML must use. Read when writing a deck's HTML."
license: "MIT, Copyright (c) 2026 lewis <sudolewis@gmail.com>; pptx guidance Apache-2.0 (Anthropic)"
source: "https://github.com/lewislulu/html-ppt-skill @fd1629067909ff55b36b905476de4e5f26062a1f"
---
# The deck runtime: what it gives you and the names it needs

This is a technical note on the deck runtime Nomi serves. It covers navigation keys, entry animations (there are no fragments), presenter view, theming hooks, and the exact class and attribute names your deck HTML must use.

The runtime is html-ppt's: `runtime.js` (keys, presenter, overview, themes, logo), `base.css` (tokens, slide system, primitives, print), `animations.css`, and optionally `fx-runtime.js` with its FX modules. Nomi serves these once. You link them; you never copy or edit them. Names below are exact. A wrong name silently does nothing.

## Document skeleton

- `<html lang="id">`. Put `data-themes` here or on `body`.
- In the head: the runtime stylesheets, then one theme stylesheet with `id="theme-link"`, then your deck's own `<style>` (your tokens and the layout CSS).
- `<body>`, then `<div class="deck">`, then one `<section class="slide" data-title="…">` per page.
- The runtime script goes last in `body`. Without it there are no keys, no presenter view and no per-page logo in PDF.
- **One file per deck.** A one-slide file for a standalone render uses `<body class="single">` with one `section.slide.is-active`.

## Canvas

- The default canvas is 1920×1080, scaled to fit the window through `--deck-scale`. Never set the slide size yourself.
- **`.deck` attributes:** `data-w` and `data-h` set a custom canvas, such as `data-w="1080" data-h="1350"` for a 4:5 carousel.
- **Opt-out:** `body[data-fit="fluid"]` follows the viewport instead. Avoid it, because export expects the fixed canvas.
- **Slide padding** is 72px 96px (from `.slide`). Slides are flex columns, vertically centred.

## Slide state (set by the runtime, never by you)

- `.is-active` marks the visible slide and `.is-prev` marks slides before it. Only a single-slide file sets `is-active` by hand.
- **URL:** `#/N` deep-links to slide N (1-based). `?preview=N` renders slide N alone with no chrome, and the runtime then sets `data-preview="1"` on `html` and `body`. Presenter previews and renders use this.

## Keys and touch

| Key | Action |
|---|---|
| → Space PageDown Enter | Next slide |
| ← PageUp Backspace | Previous slide |
| Home / End | First / last slide |
| F | Fullscreen |
| S | Presenter window |
| N | Notes drawer |
| O | Overview grid |
| T | Cycle themes |
| A | Demo animation (authoring aid, not for presenting) |
| Esc | Close the overview and drawer |

- **In the presenter window:** ← → (synced to the audience window), R resets the timer, Esc closes the window.
- **Touch:** swipe left or right on phones and tablets. Put `data-no-swipe` on anything the person must drag or scroll. Links, buttons, inputs, `.overview` and `.notes-overlay` are already excluded.

## Notes and presenter view

- **Notes:** `<aside class="notes">` (or `div.notes`, or `.speaker-notes`) inside each slide. These are always hidden from the audience. Inline `<strong>`, `<em>` and `<code>` are styled in the presenter window. The export puts notes into PPTX speaker notes.
- **Presenter window:** S opens a popup with four draggable, resizable cards: CURRENT and NEXT (iframes of this file at `?preview=N`), the script, and a timer with prev/next buttons. Card layout persists in `localStorage`.
- **How the windows talk:**

  | Link | Mechanism |
  |---|---|
  | Audience ↔ presenter windows | `BroadcastChannel` named `html-ppt-presenter-<pathname>`, messages `go` and `theme` |
  | Presenter → preview iframes | `postMessage` with `preview-goto`, `preview-theme` and `preview-ready` |

  Navigation doesn't reload the iframes.
- **Labels:** `data-title` labels a slide in the overview and presenter view. Without it, the runtime uses the slide's first h1–h3.

## Chrome (optional, inside a slide or the deck)

- **Header and footer:** `.deck-header` and `.deck-footer`, absolutely placed at the top and bottom, 40px insets.
- **Slide number:** `.slide-number`. The runtime fills `data-current` and `data-total`, and the CSS prints "n / total".
- **Auto-created if missing:** `.progress-bar` (with an inner `span`), `.notes-overlay` and `.overview`.
- **Indicators** you may add: `.theme-indicator` and `.anim-indicator`. The runtime writes the current theme or animation name into them.
- **Print hides all chrome.** Never put content there.

## Logo

- **On `body`:** `data-logo` (path relative to the deck), `data-logo-position` (`top-left`, `top-right` (the default), `bottom-left` or `bottom-right`), `data-logo-size` (height, default 44px), `data-logo-opacity` (default .9) and `data-logo-alt`.
- **Per slide:** `data-no-logo` on a `section.slide` hides the logo there.
- **By hand:** `img.deck-logo[data-pos]` inside `.deck` works without JS. Fine-tune with `--logo-inset-x` and `--logo-inset-y`.
- **In print,** the runtime sets `data-logo-print` on `.deck` and the logo is painted on every page.

## Theming hooks

- **Theme cycling:** `data-themes="a,b,c"` (on `html` or `body`) lists the themes T cycles through. `data-theme-base` is the folder holding `<name>.css`. If it's missing, the runtime takes the folder from `#theme-link`'s href.
- **What the runtime reads:** the runtime sets `html[data-theme]` to the current name. Theme files override these custom properties, and `base.css` and the FX read them:
  - Colour: `--bg`, `--bg-soft`, `--surface`, `--surface-2`, `--border`, `--border-strong`, `--text-1`, `--text-2`, `--text-3`, `--accent`, `--accent-ink`, `--accent-2`, `--accent-3`, `--good`, `--warn`, `--bad`, `--grad`, `--grad-soft`
  - Shape and type: `--radius`, `--radius-sm`, `--radius-lg`, `--shadow`, `--shadow-lg`, `--font-sans`, `--font-serif`, `--font-mono`, `--font-display`, `--letter-tight`, `--letter-normal`, `--ease`
- **Nomi tokens:** you write the deck's tokens with Nomi's names (`--color-background`, `--color-primary`, `--color-primary-foreground`, `--font-heading`, `--font-body`, `--radius-lg`, `--space-n` …), then point the runtime's names at them, for example `--accent: var(--color-primary)` and `--accent-ink: var(--color-primary-foreground)`. The full role map is in `decks-html-ppt/references/themes.md`.
- **Gradients:** set `--grad` and `--grad-soft` to a solid colour too. Base classes use them, and gradients don't survive editable PPTX.

## Classes from `base.css` you can use

| Group | Classes |
|---|---|
| Type | `.h1` (72) `.h2` (54) `.h3` (32) `.h4` (22) `.lede` `.eyebrow` `.kicker` `.dim` `.dim2` `.mono` `.serif`. Override the sizes to the deck scale in `decks-html-ppt`; base `.lede` at 22 is too small for body |
| Layout | `.stack` `.row` `.row.wrap` `.grid` with `.g2` `.g3` `.g4` `.center` `.fill` `.tc` `.tl` `.tr` `.mt-s/m/l` `.mb-s/m/l` `.sp-t` `.sp-b` `.nowrap` `.hidden` |
| Cards | `.card` `.card-soft` `.card-outline` `.card-hover`, and `.pill` `.pill-accent` `.divider` |
| Images | `figure.img-frame` (crop, `cover`), `.img-frame.contain` (letterbox), `.img-frame.fill` (full height); custom properties `--img-ratio` (default 16/10) and `--img-pos`; overlays `.img-scrim`, `.img-cap`, `.img-tag` |

**Avoid:** `.gradient-text` (lost in PPTX), `.card-accent` and `.divider-accent` (edge stripes and title rules), and `.uppercase` and the tracked capitals of `.eyebrow` and `.kicker` (Taste wants sentence case; override them).

**Not in `base.css`:** the per-layout classes (`.steps`, `.flow`, `.tier`, `.gantt`, `.rm`, `.split`, `.gal`, `.cmp`, `.btn` and so on) live in each source layout's own CSS. Write your layout CSS in the deck's `<style>`, scoped to your own class names.

## Animations

- **`data-anim="<name>"`** re-plays every time the slide becomes active. This is the one to use. `class="anim-<name>"` plays only once, on page load.
- **Names:** `fade-up` `fade-down` `fade-left` `fade-right` `rise-in` `drop-in` `zoom-pop` `blur-in` `glitch-in` `typewriter` `neon-glow` `shimmer-sweep` `gradient-flow` `stagger-list` `path-draw` `morph-shape` `parallax-tilt` `card-flip-3d` `cube-rotate-3d` `page-turn-3d` `perspective-zoom` `marquee-scroll` `kenburns` `confetti-burst` `spotlight` `ripple-reveal`.
- **Stagger:** `stagger-list` on a list or grid parent staggers its children (up to 9). `path-draw` goes on an `<svg>`.
- **Counters:** `<span class="counter" data-to="1250">1250</span>` counts up on enter. `data-dur` is in ms (default 1200). Write the real value as the text too, so a still render without JS shows it.
- **Other attributes:** `data-anim-target` marks which element the A key animates.
- **No fragments.** Nothing waits for a click inside a slide. Split the slide instead.
- `prefers-reduced-motion` disables all `anim-*` classes. Never override that.

## Canvas FX (tech decks only)

- **Setup:** load `fx-runtime.js`, then add `data-fx="<name>"` to an element with an explicit height. The effect starts on slide enter and stops on leave.
- **Names:** `particle-burst` `confetti-cannon` `firework` `starfield` `matrix-rain` `knowledge-graph` `neural-net` `constellation` `orbit-ring` `galaxy-swirl` `word-cascade` `letter-explode` `chain-react` `magnetic-field` `data-stream` `gradient-blob` `sparkle-trail` `shockwave` `typewriter-multi` `counter-explosion`.
- **Parameters:** `data-fx-text-value`, `data-fx-line1`… and `data-fx-to`. Colours come from `--accent` and `--accent-2`. `window.__hpxReinit(el)` restarts an effect.
- **Export:** FX export as a still picture at best. Never put information in them.

## Print and export behaviour

- **`@media print`:** each `.slide` is one page at the full viewport, chrome hidden, logo on every page except slides with `data-no-logo`.
- **Animations:** there is no print rule for them, so the render worker must wait for entry animations to finish (or force reduced motion) before capturing.
- **Dependencies:** charts in the source layouts used Chart.js and code used highlight.js, both from a CDN. Neither is part of the runtime. Draw charts from boxes and text, and colour code by hand with spans.
- The editable-PPTX rules (text in h1–h6, p and li; fills on wrapping boxes; no gradients or background-image; images as `<img>`) are in `decks-html-ppt`.

Adapted from html-ppt-skill (MIT), and Anthropic's pptx skill (Apache-2.0). Changed: rewritten for Nomi's decks.
