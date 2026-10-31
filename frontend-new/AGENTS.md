# frontend-new: rules for every agent

The new Nomi frontend: SvelteKit 2, Svelte 5 runes, Tailwind v4, rebuilt from the pen.dev design
system in `Nomi.pen`. These rules apply to every agent and every person working in this folder
(Claude Code reads them through `CLAUDE.md`). Read them before you change anything.

`frontend/` is the old app: read it for business logic and API usage, never import from it or
edit it. The two run side by side against the same backend until the cutover.

---

## 0. The one rule: the pen, 1:1

The pen file is the source of truth for how everything looks. The gallery at `/design-system`
rebuilds its three design sheets from the real components, and CI holds it there.

- **Match the pen exactly**: sizes, paddings, gaps, alignment, colours, strokes, radii, type,
  icons and copy as drawn. Read the frame before you write the component; measure, don't guess.
- **The pen wins** over these rules and over your taste. Where it breaks a rule here (a 10px
  citation number, a 13px icon, an inline link without an underline), follow the pen and say so in
  the component's comment.
- **Only an obvious slip in the pen** (copy pasted from another frame, text spilling out of its
  box) may be left out, and only by listing it in [§9](#9-known-pen-slips-we-dont-copy) with the
  reason. Never silently.
- **Prove it**: `e2e/pen-parity.e2e.ts` fails when any section or board of the gallery moves off
  the pen's bounds by a single pixel. `scripts/pen/` shows you where (see [§6](#6-proving-11)).
- The design sheets: *Design System / Nomi* (`YOYH1`, foundations and core components),
  *Design System / Chat responses* (`Ljvtr`), *Design System / Build* (`DcG0H`); plus 40 flows,
  each with a light twin, and the app screens. The prototype `marioprasetyo/nomi-prototype` is the
  reference for behaviour, timings and copy where the pen is silent.

## 1. Boundaries

- Change files only inside `frontend-new/` (and `.github/workflows/frontend-new.yml`). Never
  touch `backend/`, `frontend/`, `native/`, `relay/` or the other workflows.
- The backend API is used exactly as it is. Anything the UI needs that the API doesn't have goes in
  `BACKEND_TODO.md` (checked against the real backend routes first) and is left out of the UI,
  never faked.
- Never commit secrets or `.env` files (`.env.example` holds placeholders only).

---

## 2. The visual language

### Character: "An instrument, not a toy"

Calm, dense, precise. Neutral surfaces do the hierarchy; colour is rare and always means
something. It should feel like a well-made tool for people's money, plans and projects, not a
playful chat toy.

- **One accent, under 10% of a view.** Mint (`accent`) marks the single thing to act on: the
  primary button, the live dot, the chosen option. Everything else is neutral. Inside panels the
  main action is **inverse** (text-coloured fill), so mint stays rare.
- **Mono for anything a machine produced**: paths, IDs, URLs, tokens, money amounts with units,
  costs, timestamps, durations, model names, versions, keyboard keys. Everything a person wrote or
  reads as prose is Geist.
- **Borders over shadows.** Edges come from 1px strokes. Shadows only on things that truly
  float: menus, toasts, dialogs, the floating ask bar.
- **Honest states.** Show the real tool and target, elapsed time and the current step when the
  duration is unknown. No fake progress, no spinners that pretend. Cost always visible. Empty,
  loading, error and populated states ship together.
- **Mascots are identity, not decoration.** One per crew member, flat colour, only where that
  member speaks or works.

### Colour (tokens, themed for dark and light)

Values live in `src/lib/ds/tokens/tokens.css` (exported from the pen, written with `light-dark()`).
Tailwind knows only these names (drop the `nm-` prefix: `bg-surface`, `text-text-2`, …).

| Token | Role |
|---|---|
| `bg` | The page. Never pure black or white. |
| `surface`, `surface-2` | Raised areas: cards, headers, chips; `surface-2` for fills inside them, hover, tracks. |
| `border`, `border-strong` | Dividers and quiet outlines; inputs, buttons, emphasis. |
| `text`, `text-2`, `text-3` | Primary text; secondary (body copy, icons); tertiary (meta, captions, placeholders). |
| `accent`, `accent-ink`, `accent-bg` | The one action and live state; text on accent; the quiet accent tint. |
| `danger`, `danger-bg`, `danger-border` | Errors and destructive actions. |
| `warning`, `warning-bg` | Needs attention, waiting, interrupted. |
| `focus` | Focus ring and the field glow. |
| `overlay` | Dialog scrim. `shadow`: the colour of the one shadow. |
| `code-*`, `diff-*`, `heat-*`, `kb-*`, `peer-1/2(-bg)` | Syntax, diffs, heatmaps, the phone keyboard, teammates in a shared project. |

**Crew colours** (`crew-<member>`, `crew-<member>-bg`, `crew-<member>-ink` for nomi, koda, dana,
kala, rena, tara, maya, raga, personality, supervisor): only on surfaces that belong to that member
(their cards, tiles, who-it's-from markers, their usage). Never for buttons, links, focus, live
state, errors or page backgrounds.

### Type

Geist for everything, Geist Mono for machine output (§ above). Glyphs Geist Mono lacks (⌘ ⇧ ↵)
fall back to the system font, as in the pen. Sentence case everywhere; capitals only in the mono
section labels (`tracking-label`, CSS `uppercase`). Weights 400, 500, 600 only. At most four sizes
on a screen.

| Class | Size / line | Use |
|---|---|---|
| `text-display` | 28 / 36, −0.4 | Page and section titles, the greeting |
| `text-heading` | 20 / 26 | Large card titles where the pen draws them |
| `text-body` | 15 / 19 | Message body, page intro |
| `text-ui` | 13 / 17 | Default UI: buttons, nav, rows, labels |
| `text-caption` | 12 / 16 | Descriptions, secondary rows |
| `text-label` | 11 / 14 | Mono meta, kbd, section labels |
| `text-micro`, `text-nano` | 10 / 13, 9 / 12 | Only where the pen draws them (citations, tiny badges) |
| `text-phone-title / phone-heading / phone / phone-sm` | 17/22, 18/23, 16/21, 14/18 | Phone screens (16pt body so iOS doesn't zoom) |

Line heights are Geist's natural height rounded to a whole pixel, built into each size. Where the
pen sets an explicit line height use the leading token, rounded the same way: `leading-tight`
1.25, `firm` 1.3, `snug` 1.35, `cozy` 1.4, `comfy` 1.45, `normal` 1.5, `relaxed` 1.6 (prose),
`code` 1.7.

### Space, radius, elevation

- **Spacing** 2 / 4 / 6 / 8 / 12 / 16 / 20 / 24 / 32 / 48 (Tailwind `0.5 1 1.5 2 3 4 5 6 8 12`),
  plus the pen's own paddings where it draws them (9 → `2.25`, 10 → `2.5`, 14 → `3.5`, 28 → `7`).
  Tailwind's quarter steps are 1px each.
- **Radius** `xs` 4 (kbd, tags), `sm` 6 (buttons, chips, rows, inputs), `inner` 8 (a button inside
  a 10–12 container on phones), `md` 10 (panels, cards, composer, callouts, boards), `lg` 12,
  `xl` 14, `2xl` 16, `3xl` 18 (phone cards and bubbles), `device` 44, `full` for pills.
- **Shadows**: `shadow-float` (0 8 24) only on floating layers; `shadow-focus` is the field's focus
  glow, not elevation. Nothing else casts a shadow.

### Icons and mascots

- Lucide only, stroke 1.5, through `foundations/Icon.svelte`: 16px in nav and buttons, 14px inline
  with text, 12px in chips; 13, 15, 18 and the phone's 20–26 only where the pen draws them.
  Decorative unless it carries meaning alone (then it gets a label).
- Mascots (`agents/Mascot.svelte`) at 96 / 32 / 20 / 16, flat, from the pen's paths.

### Layout and screens

- Desktop screens are 1512 × 862 at 1×; phones 390 wide. Check every screen at both, in dark and
  light.
- Shells (`src/lib/ds/shells`): **App** (sidebar 248, top bar 52, content padded 32–56, crew panel
  300–380), **Workspace** (project bar 52, agent chat 440 resizable 360–720, preview never under
  480), **Focus** (one column 480–720: onboarding, dialogs, checkout).
- Phones: one column, sheets instead of popovers, 44pt targets (`pointer-coarse:`), 16pt body,
  the composer above the keyboard, banners at the top.

### Interaction and states

- Every interactive element has hover, `focus-visible` (the 2px focus ring, offset 2, or 3px on
  buttons), active and disabled states, and a 44px target on coarse pointers.
- Loading shows real progress in words ("Deploying · 00:14"). Spinners only for the current step,
  and `motion-reduce` stops them. No progress bars that creep toward 90%.
- Errors say what happened in plain words, quote the provider's own reason, and offer the next
  move. Never guess a cost on failure.

### Copy

Plain, specific, sentence case. Name the real thing ("Koda is building", "deploy #2"). No
"magic", "effortless" or "AI-powered". Every string is a Paraglide message in English and
Indonesian (§4).

### Don't

Gradients, glass, glows or a second accent hue · emoji or illustrations in UI chrome · shadows on
flat cards or rows · fake progress · Title Case or ALL CAPS outside the mono label · pure #000 or
#FFF surfaces · raw colours or arbitrary values in classes.

---

## 3. From the pen to code

These are the translations that make the gallery land on the pen. Each one cost a sweep to find.

**Strokes are drawn inside the frame and never move content.** In the pen a 1px stroke takes no
room; a CSS `border` does. So:
- A container that isn't interactive (card, panel, menu, board, list) draws its stroke as
  `outline -outline-offset-1 outline-border` (or `-border-strong`, `-accent`…) and keeps the pen's
  paddings exactly. The pen's own HTML export does the same.
- Buttons, links, inputs and labels keep `border` (their outline is the focus ring) and take 1px
  off the pen's padding on each bordered side: 16 → `3.75`, 14 → `3.25`, 12 → `2.75`, 10 → `2.25`.
  A fixed height (`h-7.5`) already includes the border.
- A side stroke (`border-t`/`border-b` on a header, footer or row) takes 1px off that side's
  padding: a header `[10,14]` with a bottom stroke is `pt-2.5 pb-2.25`. A divider the pen draws as
  its own 1px rectangle *is* layout: keep a plain `border-t` and the full padding. Check which one
  the pen uses.
- Images and phone frames: draw the stroke over the edge (`outline`), as the pen does. The phone
  bezel is an 8px outline offset −4 around a 330 × 640 frame.

**Read layout, don't eyeball it.** Pen `gap` → `gap-*`; `padding [t,r,b,l]` → per-side classes;
`width: fill_container` → `flex-1 min-w-0` or full width; a fixed width → `w-*`; no `alignItems`
means top (`items-start`), not centre. A text frame with `textAlign: right` and fill width is a
right-aligned note. Line heights the pen leaves "normal" come from the size token; explicit ones
use the leading tokens (they round like the pen does).

**Draw what the pen draws open.** Menus, dialogs, tooltips and filled-in cards appear open on
the sheets. The components take `inline` (Menu, Dialog, ShareDialog, MoveDialog, RegenerateMenu,
AttachMenu, ChatMenu) or initial state (`initialReasons`, `initialChosen`…) for the gallery;
states without a pointer use `data-hover`, `data-focus`, `data-active`, `data-dragging`.

**Mirror the pen's components.** When the pen has a part as its own component (Segment, Tab, Menu
Item, Tooltip, Dialog), so does the code (`Segment`, `Tab`, `MenuRow`, `MenuPanel`,
`TooltipBubble`, `DialogPanel`), and the interactive wrapper composes it.

**A value the pen uses that no token has**: add a named token in `src/app.css` with a comment
saying where the pen draws it (as `text-micro`, `radius-inner`, `leading-firm`). Values that only
belong to a gallery illustration of something that isn't Nomi's UI (the phone's keyboard, a demo
app's heading) go in that file's scoped `<style>` with a comment.

---

## 4. Components and code

- **Where**: `src/lib/ds/<section>/` (`actions`, `inputs`, `navigation`, `identity`, `status`,
  `conversation`, `lists`, `overlays`, `shells`, `chrome`, `agents`, `foundations`, `chat`,
  `build`, `mobile`). Each file starts with a comment naming the pen frame it implements
  (`// Design System / Nomi → Overlays / Menu: …`) and anything it does differently on purpose.
- **Presentational only**: no `$app/*`, no API types, no fetching. Links, active state, data and
  actions come in as props and callbacks; pages and flows adapt API data to those props.
- **Svelte 5 runes, TypeScript strict.** `$props()` with a `Props` type; `$bindable` for
  two-way values; `$props.id()` for ids; initial state copied with a closure
  (`const initial = () => prop; let x = $state(initial())`); snippets for slots. Export shared
  types from `<script module>`. Never name a variable `state`.
- **Tailwind v4 with the design tokens only.** No raw colours, no arbitrary values (`[…]`), no
  `dark:` (the tokens carry the theme), no shadows other than `shadow-float`/`shadow-focus`.
  Write classes in the markup (class strings or `class={[…]}` arrays), not in script variables:
  `src/lib/ds/lint` reads the markup and fails the unit tests otherwise.
- **Theme**: light, dark or system from the `nm_theme` cookie on `<html data-theme>`; any
  `[data-theme]` subtree re-themes everything inside it (the light phone preview in the dark
  workspace).
- **Accessibility**: real elements and roles (radiogroup, tablist, menu, dialog, tree, switch,
  meter, status/alert), full keyboard support as the platform does it, visible `focus-visible`,
  names for icon-only controls, decorative things `aria-hidden`, `motion-reduce` respected.
- **i18n**: every string a person reads is a message in `messages/en.json` and `messages/id.json`
  (Paraglide), added at the end of both files with the Indonesian written at the same time.
  `src/lib/messages.test.ts` fails on a missing key, an empty string or a dropped placeholder.
  The gallery's own copy is documentation and isn't translated.
- **Server**: every backend call from a load or action goes through `apiFetch(event, path, init)`
  (`src/lib/server/api.ts`), which renews the sign-in at most once per request. `server.js` runs
  in production: uploads stream to the backend (`ws-proxy/upload-proxy.js`), live updates go
  through the WebSocket relay (`ws-proxy/ws-proxy.js`), which refuses other sites' pages and
  renews an expired token before giving up. `vite dev`/`preview` run the same relay.

## 5. The docs and the gallery (`/design-system`)

`/design-system` is the design system's documentation for every team (Get started per role,
Foundations, Content, Components, Patterns, Tokens, Resources), built from the real components in
`src/routes/design-system/(docs)` and `src/lib/docs`. The three pen sheets sit beside it in
`(sheets)`.

- **Every component has a docs entry.** Adding or changing a component in `src/lib/ds` means
  adding or updating its `ComponentDoc` in `src/lib/docs/entries/<category>.ts` (summary, when to
  use it, do/don't, writing, accessibility) in the same PR; a test fails on a component without
  one. Props are never written by hand: the docs read them from the type given to `$props()` at
  build time (`src/lib/docs/api`), and the comments above it become "Notes from the source".
- Live examples are `src/lib/docs/examples/<component-slug>/<nn>-<name>.svelte`, first line
  `<!-- Title: description -->`; what renders is what the page shows as code.
- Patterns are the Chat responses and Build sheet sections (`src/lib/docs/patterns.ts`), with
  their guidance in `src/lib/docs/pattern-docs/`. Guide pages are listed in `src/lib/docs/nav.ts`.
- Internal links go through `docHref` so they work in the static copy as well.

### The sheets

- `/design-system/nomi`, `/chat`, `/build` rebuild the three sheets section by section with the
  real components, laid out like the pen: 64 around the sheet and between sections, the pen's DS
  header (`SheetHeader`, `NomiHeader`), each section a `GallerySection` (28px title, the line
  beside it).
- Boards are `GalleryBoard`, matching the pen's three board styles: default (Chat sheet: note
  right-aligned, 20 between parts), `inline` (Build sheet: note after the title, tops aligned),
  `nomi` (Nomi sheet: note after the title, centred, 16 apart); plus `phone` (430 wide),
  `spacious` (28 apart) and `tight` (16 apart, note right-aligned).
- On in `npm run dev` and wherever `PUBLIC_DS_GALLERY=1` (CI, e2e, review builds); off in
  production. `?theme=dark|light` forces a theme.
- Adding a section: read the pen section's structure, build or reuse the components (tests first),
  add the section in the pen's order, add its bounds to `pen/bounds.json` (pen/README.md), and run
  the guard. A new Chat or Build section is also a pattern: add it to `src/lib/docs/patterns.ts`
  with its guidance.

## 6. Proving 1:1

1. **Read the frame** with the Pencil MCP tools (`mcp__pencil__execute` with `Get(id,
   {depth})`, see pen/README.md): every frame's padding, gap, alignment, stroke, fill, radius and
   text style, and `ctx.bounds` for exact positions. Never read the `.pen` file directly.
2. **Write the test first** (component tests run in real Chromium, so assert the geometry the pen
   gives you: heights, insets, computed styles; `src/lib/ds/pen.svelte.test.ts` collects them).
3. **Compare** with the dev server running (`npm run dev`):
   - `npm run pen:report` — every section and board against `pen/bounds.json`.
   - `npm run pen:compare -- <sheet> <section-slug>` — every text in a section against the pen's
     own HTML export (`pen/export/`); the first line marked ◀ is where the drift starts.
   - `npm run pen:boxes -- <sheet> '<css selector>'` — an element's boxes, to set next to the pen's
     frame bounds.
4. **CI**: `e2e/pen-parity.e2e.ts` asserts every section's offset and height and every board's
   box on all three sheets. It must stay green; if the pen itself changed, re-export
   `pen/bounds.json` and `pen/export/` in the same PR. The e2e browser and the `pen:*` tools run
   Chromium with `--font-render-hinting=none`, so Linux measures text as the Mac the pen is drawn
   on does (hinted, Linux rounds every glyph to whole pixels and lines wrap early). Keep it.
5. Look at it in both themes, at 1512 and 390.

The HTML export is a good reference, not a perfect one: it collapses trailing spaces between
inline runs (code tokens, inline prose), mis-sizes negative gaps and a few fill widths. When
`compare` flags something odd, check the pen's own `ctx.bounds` before changing a component.

## 7. Tests and gates (test-first, always)

- Write the failing test, then the code, then refactor. Nothing ships without a test.
- `*.svelte.test.ts`: real Chromium (Vitest browser mode) with the app's CSS: components, roles,
  keyboard, focus, ARIA, events, and pen geometry.
- Every other `*.test.ts` (and `ws-proxy/*.test.js`): Node — pure modules, server code, the relay,
  messages, tokens, the class lint.
- `e2e/*.e2e.ts` (Playwright): the production server on 4175 against the stub backend in
  `e2e/stub-api.mjs`, including the pen-parity guard.
- Gates, chained so a failure stops the chain (CI runs the same):
  `npm run check && npm run test:unit && npm run test:components && npm run build && npm run test:e2e`.
  Never report a gate as passing without having run it.

## 8. Git and pull requests

- Never commit to `main`. One branch per task (`feat/…`, `fix/…`), stacked on the previous one
  while it's unmerged; rebase a stack with `git rebase --update-refs`.
- One PR per branch against `main` (or the branch below it in a stack), describing the changes,
  the tests and how it was validated. Wait for green CI and fix failures on the branch.

## 9. Known pen slips we don't copy

Each of these is a deliberate difference, kept because the pen looks wrong, not because it was
hard. Add to this list instead of deviating silently.

- Crew-working banner: the button reads "Stop all"; the pen's says "Team and invites" (pasted
  from another frame).
- Move to a project: the search says "Search projects"; the pen reuses the generic "Search chats,
  memory, files ⌘K".
- Sidebar recents: a long title is truncated on one line; in the pen it wraps out of its 30px row.
- Apostrophes are typographic (’) in messages; the pen's are straight.
- Inline prose uses the font's own spaces; the pen draws inline examples as frames 4px apart.
- A wrapped user bubble can be up to 2px wider than the pen's (CSS sizes a wrapped block to its
  max width, not its longest line).
- Copied although it may be a slip: the standard menu keeps its items 220 wide inside the 236
  panel, as drawn.

## 10. Running

- `npm run dev`: http://localhost:5175 (the gallery at `/design-system`; the old frontend keeps
  5173). `npm run preview`: 4175.
- `npm run build:gallery`: the gallery alone as static files in `build-gallery/`, working from
  any path on any static host (`e2e/gallery-static.e2e.ts` serves it under a nested path). This is
  the live design-system link shared with the team: rebuild and republish it after the design
  system changes.
