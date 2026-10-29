# frontend-new: rules for working here

The new Nomi frontend, rebuilt from the pen.dev design system (`Nomi.pen`: the sheets
*Design System / Nomi*, *Chat responses* and *Build*, plus the flows). `frontend/` is the
old app: read it for business logic and API usage, never import from it or edit it.

## Boundaries
- Change files only inside `frontend-new/` (and `.github/workflows/frontend-new.yml`).
  Never touch `backend/`, `frontend/`, `native/`, `relay/` or the other workflows.
- The backend API is used exactly as it is. Anything the UI needs that the API doesn't have goes
  in `BACKEND_TODO.md` (checked against the real backend routes first) and is left out of the
  UI, never faked.

## Design system
"An instrument, not a toy." Every value comes from a design token; the pen file is the source
of truth for look, the prototype (`marioprasetyo/nomi-prototype`) for behaviour and copy.
- Match the pen 1:1: sizes, spacing, colours, icons and copy as drawn. Where the rules below and
  the pen disagree (a 10px citation number, a 13px icon, an inline link without an underline),
  the pen wins and the component's comment says so. Check every component against an export of
  its pen frame before it ships.
- One accent (mint), under 10% of a view: it marks the single thing to act on. Neutrals do the
  hierarchy. Inverse buttons for panel actions.
- Mono (Geist Mono) for anything a machine produced: paths, IDs, URLs, tokens, money, costs,
  timestamps, model names. Everything else is Geist.
- Borders over shadows. Shadows only for things that float: menus, toasts, dialogs.
- Honest states: show the real tool and target, elapsed time and the current step when the
  duration is unknown, no fake progress, cost always visible. Ship empty, loading, error and
  populated states together.
- Mascots are identity, not decoration: one per agent, flat colour, only where that agent speaks
  or works.
- Type: sizes 28 / 15 / 13 / 12 / 11 (max four per screen; `text-micro` 10 only where the pen
  uses it), weights 400 / 500 / 600, sentence case everywhere; uppercase only in the mono labels.
- Spacing 2 / 4 / 6 / 8 / 12 / 16 / 20 / 24 / 32 / 48; radius 6 (buttons, chips, rows, inputs)
  and 10 (panels, composer, callouts). Lucide icons, stroke 1.5: 16px in nav and buttons, 14px
  inline, 12px in chips (13px where the pen draws it).
- Don't: gradients, glass, glows or a second accent hue; emoji or illustrations in UI chrome;
  shadows on flat cards or rows; progress bars that creep toward 90%; "magic", "effortless",
  "AI-powered"; Title Case or ALL CAPS outside the mono label; pure #000 or #FFF surfaces.
- Every interactive element has hover and `focus-visible` states and a 44px target on phones.
  Check every screen at 1512 wide and 390 wide, in dark and light.

## Code
- Svelte 5 runes, TypeScript strict, Tailwind v4 with the design tokens only (no raw colours,
  no `dark:` variant: the tokens carry the theme).
- Tokens live in `src/lib/ds/tokens/tokens.css`, exported from the pen file into `tokens.json`
  (`tokens.test.ts` fails when the two drift). Tailwind knows nothing else: colours are the token
  names without `nm-` (`bg-surface`, `text-text-2`, `border-border-strong`, `bg-accent`,
  `text-accent-ink`, `bg-crew-koda-bg`), sizes `text-display / body / ui / caption / label / micro`,
  `rounded-xs / sm / md`, `shadow-float` (floating layers only), `leading-code`, `tracking-label / display`,
  `font-sans / mono`. Spacing keeps Tailwind's 0.25 steps (1px each quarter): the design's scale
  is 2, 4, 6, 8, 12, 16, 20, 24, 32, 48 (`0.5 1 1.5 2 3 4 5 6 8 12`), plus the pen's own paddings
  where it draws them (10 → `2.5`, 14 → `3.5`, 9 → `2.25`).
- The pen draws strokes inside the frame, so they never move the content: a box with a 1px border
  takes 1px off the pen's padding (pen 16 → `p-3.75`, 14 → `3.25`, 24 → `5.75`), and comes out the
  pen's size. Explicit line heights round to whole pixels like the pen's (`leading-*` tokens).
- `src/lib/ds/lint` checks every `.svelte` file: no raw colours, no `dark:`, no shadows other
  than `shadow-float`, no arbitrary values, and no class Tailwind doesn't generate. It reads the
  markup, so write classes in the markup, not in script variables.
- Theme: light, dark or system in the `nm_theme` cookie, put on `<html data-theme>` by the server.
  Any `[data-theme]` subtree re-themes everything inside it.
- `/design-system` is the gallery: each design sheet rebuilt from the real components, to compare
  with the pen file (`?theme=dark|light`). On in `npm run dev`, off in production unless
  `PUBLIC_DS_GALLERY=1`. Its copy is documentation, not app text, so it isn't translated.
- Design-system components live in `src/lib/ds/<section>/` and are presentational: no `$app/*`,
  no API types. Links, active state and actions come in as props. Pages and flows adapt API data
  to those props.

## Server
- Every backend call from a load or action goes through `apiFetch(event, path, init)`
  (`src/lib/server/api.ts`): it sends the person's token and renews the sign-in at most once per
  request (the hook renews it ahead of expiry; a 401 renews and retries once). When renewal fails
  they're signed out and sent to `SIGN_IN_PATH`.
- `server.js` runs in production: uploads stream straight to the backend (`ws-proxy/upload-proxy.js`),
  live updates go through the WebSocket relay (`ws-proxy/ws-proxy.js`: `/chat/:id/ws`,
  `/projects/:id/ws`, `/activity/ws`), everything else is SvelteKit. The relay refuses other sites'
  pages and renews an expired token before it gives up on a connection. `vite dev` and
  `vite preview` run the same relay (`vite-plugins/dev-ws-proxy.ts`).

## Tests (test-first, always)
- Write the failing test, then the code, then refactor. Nothing ships without a test.
- `*.svelte.test.ts` runs in real Chromium (Vitest browser mode, `vitest-browser-svelte`) with
  the app's CSS: components, roles, keyboard, focus, ARIA, events.
- Every other `*.test.ts` (and `ws-proxy/*.test.js`) runs in Node: pure modules, server code,
  the relay, message files.
- `e2e/*.e2e.ts` (Playwright) runs the production server on 4175 against a backend: the smoke
  tests use the stub in `e2e/stub-api.mjs`.
- Gates, chained so a failure stops the chain:
  `npm run check && npm run test:unit && npm run test:components && npm run build && npm run test:e2e`.

## i18n
- Every string the person reads is a message in `messages/en.json` and `messages/id.json`
  (Paraglide). Add keys at the end of both files with the Indonesian written at the same time;
  `src/lib/messages.test.ts` fails on a missing key, an empty string or a dropped placeholder.

## Running
- `npm run dev` serves on http://localhost:5175 (the old frontend keeps 5173),
  `npm run preview` on 4175.
