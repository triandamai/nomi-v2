# Frontend Chat MVP

Date: 2026-07-27
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-25-frontend-chat-design.md` (the full, longer-term frontend vision — this spec is a deliberately smaller first slice of it, scoped to what the backend actually supports today), `docs/superpowers/specs/2026-07-27-chat-api-design.md` (the REST API this frontend consumes), `docs/superpowers/plans/2026-07-26-auth-claims-layer.md` (the JWT/claims model this frontend authenticates against)

## Purpose

`2026-07-25-frontend-chat-design.md` designed a full-featured chat frontend assuming WebSocket/MQTT streaming, an always-expanded reasoning timeline, an org switcher, `/org/:id/members`, `/admin`, and group chat. None of that exists server-side yet — the backend only exposes request/response REST endpoints (`docs/superpowers/specs/2026-07-27-chat-api-design.md`): session create/list, and paginated message history/send, where the send endpoint's HTTP response itself carries the assistant's reply synchronously (no push channel exists).

This spec designs a deliberately smaller first slice: a working SvelteKit app that lets a user register, log in, create a chat, and exchange messages with the orchestrator — styled after the "Sand" reference UI (clean sidebar + main pane + pill input bar) — without inventing frontend behavior the backend can't back up. Longer-term features (reasoning timeline, streaming, org management, admin) remain in the original design doc as future slices, each to be re-scoped against the backend capability that exists when they're picked up.

## 1. Architecture

A new `frontend/` directory at the repo root, sibling to `backend/` — a standalone SvelteKit project using SvelteKit's default SSR (not a client-only SPA), styled with Tailwind CSS, using `pnpm` as the package manager.

**Why SSR, not a client-only SPA:** `load` functions and form actions run server-side, so the JWT never needs to live in client-accessible storage — it's read from an httpOnly cookie inside server-side code and attached as a Bearer token when calling the backend. Route protection becomes a server-side `+layout.server.ts` guard.

**Auth token handling:**
- Login/register are SvelteKit form actions. On success, they call the backend's `/api/auth/login` or `/api/auth/register`, then set `access_token` and `refresh_token` as httpOnly cookies (`path: '/'`, `sameSite: 'lax'`) on the SvelteKit response. They also set a third, non-httpOnly `user_email` cookie to the email the user just submitted — purely for display (the greeting, below); `GET /api/whoami` returns only `Claims { sub, active_org_id, permissions, exp, iat }`, with no email or display name, so there is no backend endpoint to fetch it from later. This cookie carries no secret and isn't used for any authorization decision.
- `src/hooks.server.ts` reads the `access_token` cookie into `event.locals.accessToken` on every request.
- A shared `apiFetch(fetch, path, options, locals)` helper (in `src/lib/server/api.ts`) wraps every backend call, attaching `Authorization: Bearer <accessToken>`. On a `401` response, it calls `POST /api/auth/refresh` with the `refresh_token` cookie, retries the original call once with the new access token (updating the cookie), and on continued failure clears both cookies and redirects to `/login`.
- **Route protection is a UX nicety, not the real security boundary** — same principle as the original frontend design doc: the Rust backend independently re-validates the token (and, for session-scoped endpoints, current org membership) on every request regardless of what the SvelteKit layer does. A `+layout.server.ts` for the authenticated route group redirects to `/login` if `locals.accessToken` is absent, purely so logged-out users don't see a flash of protected UI before the backend would reject the call anyway.

**Environment:** `API_URL` (e.g. `http://localhost:8080`), read via `$env/static/private` — never exposed to the client bundle.

## 2. Screens & Routing

- **`/login`** — email + password form. On submit: `POST /api/auth/login`, set cookies, redirect to `/`.
- **`/register`** — email + password + org name. On submit: `POST /api/auth/register` with `org: { mode: "create", name }` (the only mode exposed in this UI — "join via invite code" has no backend endpoint to generate a code yet, so it's omitted here entirely rather than built against a feature that doesn't exist). Sets cookies, redirects to `/`.
- **`(app)` route group** — the authenticated shell. Its `+layout.server.ts`:
  - Redirects to `/login` if no access token cookie is present.
  - Loads the session list once via `GET /api/sessions`, shared by every page under this group (no duplicate fetches between `/` and `/chat/[id]`).
  - Its `+layout.svelte` renders the persistent `Sidebar` around whichever child page is active.
- **`/` (inside `(app)`)** — the empty state: "Hi, `<email>`!" greeting (from the `user_email` cookie set at login/register — see Architecture) and a prominent "New Chat" button. No feature cards (image/file/voice) — those map to capabilities this backend doesn't have, and a placeholder grid would just be visual noise implying features that don't work.
- **`/chat/[sessionId]` (inside `(app)`)** — the conversation view: message list (`GET /api/sessions/:id/messages`, oldest-first) + a pinned `MessageInput` bar.

## 3. Components

- **`Sidebar`** (`src/lib/components/Sidebar.svelte`) — brand row, "New Chat" button (form action → `POST /api/sessions` → redirect to `/chat/[new session_id]`), a scrollable list of `SessionListItem`s, and a user row at the bottom (email + logout button, which calls `POST /api/auth/logout` and clears cookies).
- **`SessionListItem`** — one row per session: last-message preview text (truncated), a relative timestamp (`updated_at`), and a small pulsing dot when `agent_active: true`. No channel badge (every session is `"web"` in this slice — the old design's per-row channel badge only matters once other channels are wired to a shared inbox, which isn't in scope here).
- **`MessageBubble`** — two variants (`sender: "user"` right-aligned, `sender: "assistant"` left-aligned), plain text content + timestamp. No reasoning/tool-call timeline — the API doesn't expose that data (`MessageItem` is just `{id, sender, content, created_at}`), so rendering one would mean fabricating UI for data that doesn't exist.
- **`MessageInput`** — a pinned bottom pill: text field + send button only (styled after the Sand reference's input bar chrome). No attachment icon, slash-command palette, or mode chip — all deferred, matching what `POST .../messages` actually accepts (`{text}` only).
- **`EmptyState`** — the "Hi, `<email>`!" + "New Chat" view described above.

## 4. Data Flow

1. **New Chat:** `Sidebar`'s "New Chat" button is a form action posting to a `+page.server.ts` action that calls `POST /api/sessions`, then redirects to `/chat/[session_id]`. This also invalidates the `(app)` layout's session-list load (via SvelteKit's `depends('app:sessions')` / `invalidate`), so the new session appears in the sidebar immediately.
2. **Opening a chat:** `/chat/[sessionId]/+page.server.ts`'s `load` calls `GET /api/sessions/:id/messages` and renders the returned messages oldest-first (matches the API's own ordering).
3. **Sending a message:** `MessageInput`'s form (progressive enhancement via `use:enhance`) posts to a form action that calls `POST /api/sessions/:id/messages`. Since that endpoint's response already contains both `user_message` and `assistant_message` synchronously, the action returns both and the page appends them to the rendered list in one round trip. The action also invalidates `app:sessions` so the sidebar's preview/ordering reflects the new last message.
4. **No polling, no streaming.** The send endpoint's response is the only mechanism for learning the assistant's reply, by the backend's own design (per `2026-07-27-chat-api-design.md`: "this HTTP response is how the caller learns the assistant's reply") — so there is nothing for the frontend to poll or subscribe to yet.

## 5. Error Handling

- **Login/register failures** (`401` invalid credentials, `409` email taken, `400` malformed): shown as inline form errors beneath the relevant field, not toasts.
- **`send_message` returning `502`** (turn failure): rendered as a small inline "no reply yet" note attached to the user's message bubble, not a page-level error banner — the inbound message is already durable server-side (Txn A, per the turn-loop design), so this must read as "no reply yet," not "message lost." No auto-retry; the user can just send again.
- **`404`** (session not found, or belongs to another org): redirect to `/` with a small flash message ("that chat isn't available").
- **Token refresh failure** (refresh token invalid/expired): clear both cookies, redirect to `/login`.

## 6. Testing Approach

A small, pragmatic set of Playwright end-to-end tests against a real running backend + Postgres (matching this project's existing "test against the real thing, not mocks" convention) rather than exhaustive component-level coverage, since this is a first slice:
- Register → land on the empty-state `/` with the greeting showing the registered email.
- New chat → send a message → see both the user bubble and the assistant's reply appear.
- Visiting `/chat/[id]` or `/` while logged out → redirected to `/login`.
- A `502` from a forced turn failure → the inline "no reply yet" note appears, and the session's message history still shows the user's message on reload.

## Out of Scope

- Reasoning/tool-call timeline — no data for it in the current API; would require a backend change to expose `agent_events` per message first.
- Real-time streaming/WebSocket — the backend has no push channel yet.
- Org switcher, `/org/:id/members`, `/admin` — all still pending from the original frontend design; this app currently assumes one org per user (the one created at registration).
- Group chat creation, unread counts, slash-command palette, per-turn routing mode chip, attachments/image/file/voice — none of these have backend support.
- "Join org via invite code" registration path — no backend endpoint exists yet to generate a code.
