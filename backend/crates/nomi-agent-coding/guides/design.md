# Design: make it look made by a person with taste

Every project ships with a deliberate, modern look, even a two-screen todo app. The bar: someone
who opens it should not be able to tell it was generated. Decide the look before writing
components, write it down as tokens in CSS, then build every screen from those tokens.

## What reads as "AI slop" (never do these)

- Purple/indigo-to-pink or blue-to-purple gradients: on the hero, on buttons, on text
  (`bg-clip-text text-transparent` headlines). No gradient text at all.
- Glowing blurred blobs, floating orbs, aurora backgrounds, grain-on-gradient heroes.
- Glassmorphism everywhere (`backdrop-blur` cards on a gradient).
- Emoji as icons or decoration (🚀 ✨ 💡 🔥), sparkle icons, "AI-powered" badges.
- The template landing page: centered "Welcome to X" + subtitle + two buttons, then a row of
  three identical icon cards, then fake testimonials, then a four-column footer. A tool is not a
  landing page: its first screen is the tool.
- Everything in a `rounded-2xl shadow-lg` card, and cards inside cards. Identical card grids.
- Default Tailwind look: Inter/system font, `slate` greys, `indigo-600` buttons, `gray-50` page.
- Marketing filler copy: "Unlock", "Seamless", "Elevate", "Revolutionize", "Effortlessly",
  "Your all-in-one…", "Supercharge". Fake numbers ("10k+ happy users"), lorem ipsum, "John Doe".
- Hover effects on everything (scale-105, glow, bounce); spinning or pulsing decoration.
- Dark mode as pure black with neon accents.

## What to do instead

### 1. Pick one direction that fits the subject
Choose from these (or a clear variation) and stick to it on every screen:

- **Editorial**: serif display face, generous margins, hairline rules, an off-white paper
  background, one ink colour plus one accent. Good for journals, reading lists, recipes, notes.
- **Swiss utility**: a sharp grotesk, strict grid, left-aligned everything, dense but calm
  tables, one saturated accent used only for the primary action and selection. Good for trackers,
  dashboards, finance, admin tools.
- **Soft tactile**: warm tinted neutrals, large radii used consistently, subtle inner borders
  instead of shadows, friendly rounded sans. Good for habit apps, personal tools, kids, wellness.
- **Bold graphic**: big confident type, flat blocks of colour, thick borders, no shadows,
  playful but disciplined. Good for games, events, portfolios, small shops.

### 2. Tokens first (Tailwind v4 `@theme`)
Define colour, type and radius once in the main CSS file and use only these utilities after:

```css
@import 'tailwindcss';

@theme {
  --font-display: 'Fraunces', ui-serif, Georgia, serif;
  --font-sans: 'Inter Tight', ui-sans-serif, system-ui, sans-serif;

  /* Neutrals tinted toward the accent's hue, not plain grey. */
  --color-paper: oklch(98.5% 0.008 85);
  --color-surface: oklch(96% 0.01 85);
  --color-line: oklch(89% 0.012 85);
  --color-muted: oklch(52% 0.02 85);
  --color-ink: oklch(22% 0.02 85);
  /* One accent. Use it for the primary action, links and the selected state. Nothing else. */
  --color-accent: oklch(55% 0.15 35);
  --color-accent-ink: oklch(98% 0.01 35);

  --radius-card: 0.75rem;
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-paper: oklch(18% 0.012 85);
    --color-surface: oklch(22% 0.014 85);
    --color-line: oklch(32% 0.014 85);
    --color-muted: oklch(70% 0.02 85);
    --color-ink: oklch(94% 0.01 85);
    --color-accent: oklch(70% 0.14 35);
    --color-accent-ink: oklch(18% 0.02 35);
  }
}

body { @apply bg-paper text-ink font-sans antialiased; }
```

Change the hue (85 and 35 above), the fonts and the radius to fit the direction; never ship
these exact values for every project. Load fonts with one `<link>` to Google Fonts in the HTML
head (`display=swap`, only the weights you use). Good pairings: Fraunces + Inter Tight,
Instrument Serif + Instrument Sans, Bricolage Grotesque + Geist, Space Grotesk + IBM Plex Sans,
Familjen Grotesk + Inter, DM Serif Display + DM Sans. One display face, one text face, at most.

### 3. Type does the hierarchy
- A real scale with contrast: page titles 2.25–3.5rem with tight tracking (`tracking-tight`,
  `leading-[1.05]`), section heads ~1.25rem, body 1rem, meta 0.8125rem in `text-muted`.
- Weight and size make the hierarchy, not boxes and colours.
- Numbers in tables, prices, timers and stats use `tabular-nums`.
- Reading text at most ~65ch wide.

### 4. Layout
- Mobile first: 16px side gutters, 44px minimum touch targets, the main action reachable with a
  thumb (a sticky bottom bar on phones is fine). Then widen to a real grid, not a stretched phone.
- Left-align text. Centre only short, standalone things (an empty state, a single form).
- Use whitespace and 1px `border-line` dividers to group things; reach for a card only when
  something is genuinely a separate object (a draggable item, a product). Lists are rows, not
  cards.
- One radius scale (e.g. 6px controls, 12px panels) used everywhere. Shadows: none, or one
  subtle one for things that float (menus, dialogs).

### 5. Colour
- Neutrals carry the page; the accent appears in a few places per screen.
- Success/warning/error are muted versions that sit with the palette, used only for state.
- Check contrast: body text and controls must meet WCAG AA in light and dark.

### 6. Details that make it feel finished
- Icons: one consistent set of inline SVGs (24px grid, 1.5–2px stroke, `currentColor`), sized to
  the text next to them. Never emoji.
- Every interactive element has hover, `focus-visible` (a 2px accent ring) and disabled states.
- Empty states say what goes here and offer the action to add the first one. Loading shows the
  shape of what's coming (skeleton rows), not a centred spinner on a blank page. Errors say what
  happened and what to do.
- Motion is quick and quiet: 150–250ms, ease-out, opacity/transform only; respect
  `prefers-reduced-motion`.
- Copy is specific and plain: name things the way the user would ("Add a book", "3 left this
  week"), sentence case, no exclamation marks. Seed demo data that looks real for the subject.
- A favicon and a proper `<title>`.

## Before you call complete_task
Look at what you built against this list: no gradient text or purple gradients, no emoji icons,
no card-in-card, no default indigo/slate, tokens defined in `@theme` and used everywhere, a
display font loaded, dark mode works, it works at 375px wide, every button has a focus state,
empty/loading/error states exist. Fix anything that fails before finishing.
