# Nomi Expressive Redesign

Date: 2026-10-05
Status: Implemented
Related: `docs/superpowers/specs/2026-08-29-md3-component-library-design.md` (the component library this builds on), design canvas "Nomi Redesign" (preview approved before implementation).

## Purpose

Give Nomi a visual identity of its own on top of Material 3 Expressive. The previous look was a straight SchemeVibrant green with a condensed display face. It read as generic. This redesign keeps the M3 token architecture and every existing component API, and changes what the tokens say and how the components move.

## Principles

1. **Gradient is a material, not a background.** Gradients paint objects that act: Nomi's mark, the send button, the "New chat" FAB, an agent that is working, progress. Page grounds and containers stay flat. At most one gradient button per screen.
2. **Shape carries identity.** Every agent owns one M3 Expressive shape and one gradient (`$lib/components/m3/shapes.ts`):

   | Agent | Shape | Gradient |
   |---|---|---|
   | Nomi (chitchat, dynamic agents, fallback) | cookie 9 | glow `#E4F76A → #5BE08F → #1DB9A0` |
   | Money | sunny 8 | ember `#FFD27A → #FF7A4A` |
   | Coding | cookie 6 | tide `#C4F3EA → #3FB8C8` |
   | Planning | clover 4 | sky `#D6ECFF → #7AB0FF` |
   | Personality / memory | flower 5 | bloom `#FFD9E2 → #FF8FA8` |

3. **Motion explains state.** Spatial changes (shape, size, position) use overshooting spring curves (`--nomi-motion-spatial-*`); color and opacity use non-overshooting effect curves (`--nomi-motion-effects-*`). Shapes only spin or morph while something is actually working. Everything respects `prefers-reduced-motion`.

## Tokens (`frontend/src/lib/styles/material3.css`)

- **Color:** the default ("green") light and dark schemes were regenerated around seed `#1FC46B`, with an ember tertiary (`#A33A12` / `#FFB596`) reserved for approvals. The user-selectable accent schemes are unchanged.
- **Type:** Bricolage Grotesque for display and headline (real 800 weight), Instrument Sans for title, body and label, and JetBrains Mono for meta text (`.nomi-meta`, timestamps, counters).
- **New tokens:** `--nomi-gradient-*` with matching `--nomi-on-gradient-*` inks (4.5:1 across every stop), `--nomi-shape-bubble-start|end`, `--nomi-motion-*`, and `--nomi-color-stage` / `--nomi-color-on-stage` (a dark panel that stays dark in both themes).

## Components (`frontend/src/lib/components/m3/`)

**New:**
- `AgentShape`: the agent's shape filled with its gradient, with optional `working` spin/breathe animation and Nomi's face.
- `LoadingIndicator`: the M3 Expressive morphing loader, interpolated per frame. Every shape shares one point count, so any pair can morph.
- `WavyProgress`: M3 Expressive wavy linear progress.
- `SendButton`: the cookie-shaped gradient send button. It becomes the loader while a turn is in flight.
- `Chip` (assist / filter / suggestion).
- `Switch`: form-friendly via a hidden input.
- `ButtonGroup`: the connected group; the selected segment springs to fully round.
- `SplitButton`.
- `Snackbar`.

**Changed:**
- `Button`:
  - adds a `gradient` variant;
  - sizes follow the Expressive scale (S 40, M 56, L 96, XL 136);
  - corners tighten on press.
- `IconButton`: adds `selected`, which sets `aria-pressed` and morphs the button from round to square.
- `Card`: radius raised to XL, plus a new `tonal` variant.
- Menus, lists and fields: rounder corners and a primary-container selection color.

## Screens

- **Sidebar:**
  - the Nomi wordmark;
  - a gradient "New chat" FAB;
  - Home / Chats / Projects / Memory with M3 pill indicators;
  - collapses to a proper navigation rail.
- **Home:**
  - greeting in the user's timezone;
  - a composer that starts the chat *and* sends the first message (the `newChat` action now accepts optional `text`);
  - suggestion chips;
  - a "Your crew" card;
  - recent chats.
- **Chat:**
  - assistant turns are editorial (no bubble, agent shape as avatar);
  - user turns are primary bubbles with a tight corner;
  - the working state is the morphing loader;
  - the composer is a floating container with an auto-growing textarea (Enter sends, Shift+Enter breaks the line).
- **Content blocks:**
  - approval in ember;
  - to-do with wavy progress and real check marks;
  - plan and file blocks with agent shapes;
  - all emoji glyphs removed.
- **Auth:** login and register share `AuthShell`, a dark stage with the agent crew.
- **Other pages:** account pages share one display-size page title, and Preferences uses the connected button group and morphing swatches.
