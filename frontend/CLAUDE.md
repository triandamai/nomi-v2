# Frontend UI rules

Nomi's own screens follow the same bar as the apps Koda builds
(`backend/crates/nomi-agent-coding/guides/design.md`): nothing that reads as AI-generated.

- No accent stripes on cards (a coloured left border or inset shadow), no uppercase eyebrow
  labels with icons on every card, no initial-letter squares decorating cards (an organization's
  avatar is fine), no glowing or
  pulsing status dots, no gradient text, no emoji as icons.
- A card is a title, a line or two of content and one quiet meta line. Lists are rows; reach for
  cards only when the items are objects people browse.
- Colour comes from the theme tokens in `src/lib/styles/material3.css`. Anything that should
  follow the person's theme uses the theme roles (`--md-sys-color-primary`, …), the action
  gradient (`--nomi-gradient-action`, `--nomi-on-action`) or `--nomi-chart` for single-series
  charts. Agent gradients (`--nomi-gradient-<tone>`) belong to agents only.
- Pages sit on `--nomi-color-ground`; cards and tables on `--md-sys-color-surface-container-lowest`
  (in light it's white with a touch of the theme, so it follows the person's theme); things inside
  a card on `surface-container-low`. Never paint a surface `#fff`/`white`.
- Every interactive element has hover and `focus-visible` states and a 44px target on phones;
  check every screen at 390px wide and in dark mode.
