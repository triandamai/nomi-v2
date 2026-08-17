# Frontend Realtime Rendering

Date: 2026-08-17
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-11-websocket-bridge-design.md` (sub-project 4 — the `GET /api/sessions/:id/ws` Rust endpoint this design consumes, already merged). Sub-project 5 of 5 in the realtime chat effort, and the last one.

## Purpose

Sub-project 4 gave the backend a WebSocket endpoint that relays a session's live turn events, but nothing in the browser uses it — the chat page still only shows whatever was in the database at the last full page load. Worse, the chat page's send flow is currently broken: `+page.server.ts`'s form action still destructures an `assistant_message` and checks for a `502` status from `POST /messages`, both of which stopped existing when that endpoint became ingest-only (202, `user_message` only). Today, sending a message shows nothing until a manual reload.

This design closes both gaps: it fixes the send flow for the ingest-only contract, and adds the missing link — SvelteKit relays the backend's WebSocket stream to the browser, and the chat page shows a typing indicator while a reply is in flight, then renders the finished reply once it's persisted.

**Topology.** Per the sub-project 4 design, the browser never talks to the Rust backend directly — every API call goes through SvelteKit's own server, which holds the JWT in an httpOnly cookie. This design keeps that shape on the realtime path too: the browser opens a real WebSocket to SvelteKit at `/chat/{sessionId}/ws`, and SvelteKit's server opens a second, independent WebSocket to the Rust backend's `/api/sessions/:id/ws`, forwarding frames verbatim between the two. Both legs are real WebSocket connections (not Server-Sent Events) — this channel is currently one-directional in practice (the browser never sends data over it; outgoing messages still go through the existing POST form), but a real WebSocket matches the original design intent and keeps the door open for future bidirectional use without a protocol change.

## 1. SvelteKit-side WebSocket Proxy

Three new files, one shared implementation:

- `frontend/src/lib/server/session-stream-proxy.ts` — the actual proxy logic, framework-agnostic: given a raw HTTP upgrade request and socket, extract `sessionId` from the URL path (`/chat/{sessionId}/ws`), read the `access_token` cookie, open a `ws` client connection to the Rust backend's `GET /api/sessions/:id/ws` with `Authorization: Bearer <token>`, and — once that upstream connection is open — accept the browser's upgrade and pipe text frames between the two verbatim in both directions. Never parses the frame payloads; it is a dumb pipe, matching the backend's own "verbatim, no reparsing" relay behavior.
- `frontend/server.js` — the production entrypoint (replaces plain `node build`). Creates an `http.Server`, delegates ordinary HTTP requests to the built SvelteKit `handler` from `adapter-node`'s output, and attaches an `upgrade` listener that calls into `session-stream-proxy.ts` for paths matching `/chat/:sessionId/ws`, letting everything else fall through untouched.
- `frontend/vite-plugins/dev-ws-proxy.ts` — a Vite plugin providing the same upgrade wiring for two other run modes that don't go through `server.js`: `configureServer` (used by `vite dev`) and `configurePreviewServer` (used by `vite preview`, which is what the existing Playwright e2e harness boots via `npm run build && npm run preview` — SvelteKit's own Vite plugin serves the production SSR build through `vite preview` via this same hook, independently of `adapter-node`'s `server.js`). Registered in `vite.config.ts`. All three entrypoints call the identical `session-stream-proxy.ts` function, so dev, e2e, and true production behave the same.

**Auth model:** no JWT verification happens in Node. The cookie's `access_token` is forwarded as-is to Rust as the Bearer token, exactly like every other server-side call in `frontend/src/lib/server/api.ts`. Rust's existing `authorize_session_access` remains the sole authority on whether the connection is allowed.

**Rejection mapping:** if the upstream Rust handshake fails, the browser's upgrade is rejected (never completed) with an application close code carrying the reason:
- Rust responded 401 → browser closes with code `4401`
- Rust responded 404 → browser closes with code `4404`
- Any other upstream failure (connection refused, timeout, EMQX down, etc.) → treated as transient; see §3.

A `sessionId` that isn't a well-formed UUID doesn't need its own validation in the proxy — Rust's own `Path<Uuid>` extractor rejects it before `authorize_session_access` ever runs, which the proxy already maps to `4404` like any other upstream rejection.

**Dependencies:** adds `ws` (runtime) and `@types/ws` (dev) to `frontend/package.json`. No other new dependencies.

## 2. Browser-side: Chat Page

`frontend/src/routes/(app)/chat/[sessionId]/+page.svelte` gains:

- A `WebSocket` connection to `/chat/{sessionId}/ws`, opened once when the page mounts and closed on unmount/navigation-away. Same-origin, so the cookie is sent automatically — no token handling in browser JS at all.
- A `pendingReply` boolean driving a typing-indicator bubble. No client-side accumulation of delta text: the chosen UX only needs to know "a reply is being generated" vs. "not," so the handler only inspects the envelope's `kind` field, never its `event`/`error` payload contents (except to decide whether the ⁠pendingReply flag flips).
- Message handling:
  - `kind: "Delta"` → `pendingReply = true` (idempotent; also correctly reflects a turn triggered from another tab/device on the same session, since the connection is session-scoped, not request-scoped)
  - `kind: "TurnCompleted"` → `pendingReply = false`, call SvelteKit's `invalidateAll()` to re-run the page's existing `load` function, which re-fetches `GET /api/sessions/:id/messages` — the single source of truth for what actually got persisted. No client-side text ever has to match the server's exactly, because the client never renders streamed text at all.
  - `kind: "TurnFailed"` → `pendingReply = false`, show a generic inline error ("Something went wrong — try sending again"), not the envelope's raw `error` string.
- Reconnect-with-backoff on unintentional close (1s → 2s → 4s → 8s → capped ~30s, small jitter), **except** when the close code is `4401` or `4404` (§1) — those are treated as terminal, showing "Couldn't connect to this chat — try reloading the page" instead of retrying forever against a condition that cannot self-resolve. On every reconnect after the first, call `invalidateAll()` once, since neither leg of this pipe replays missed events — this is how the UI reconciles anything that happened while disconnected, consistent with sub-project 4's "REST is the source of truth for catch-up" design.

`frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` changes:
- The `default` action's return type drops `assistant_message` (never exists on the ingest-only response) and the `response.status === 502` / `turnFailed` branch (never happens — ingest either 202s or fails outright before any turn runs). It now returns only `{ user_message }`.
- `+page.svelte`'s enhance callback calls `invalidateAll()` on a successful submission (replacing the old `update({ reset: true })` reliance on `form.user_message`/`form.assistant_message` merging) so the sent message appears via the same single code path as every other message.

## 3. Error Handling

| Condition | Behavior |
|---|---|
| Rust rejects upstream handshake (401/404) | Browser socket closed with `4401`/`4404`; browser stops retrying, shows a terminal error |
| Rust/EMQX unreachable, or a live upstream drop | Proxy retries the upstream leg with backoff, keeping the browser socket open; if retries exceed ~60s, proxy closes the browser socket with `1011` so the browser's normal (non-terminal) reconnect loop starts fresh |
| Malformed/non-JSON frame reaching the browser | Logged and dropped client-side; the proxy itself never parses frames, so this can only be a browser-side parse failure, and it never crashes the handler |
| Backgrounded/suspended tab | No special handling — surfaces as a normal close event, covered by the existing reconnect-then-`invalidateAll()` flow on resume |

## 4. Testing

**`session-stream-proxy.ts` — isolated Node tests (new).** This project has no Vitest today (Playwright only); adding it scoped to this one file is justified because Node-level WebSocket-proxy plumbing isn't something Playwright's browser-driven model exercises well, and a fast, real-components test (a bare `http.Server`, a fake upstream `ws` server standing in for Rust, a real `ws` client standing in for the browser) is a better fit than mocking. Covers: verbatim pass-through in both directions, the `4401`/`4404` rejection mapping, and upstream-drop-with-retry keeping the downstream socket alive across a simulated Rust reconnect.

**Playwright e2e (extends the existing fake-LLM harness in `playwright.config.ts`, which already boots the real backend with `LLM_PROVIDER=fake`/`EMBEDDING_PROVIDER=fake`):**
- Send a message → typing indicator appears → reply renders without a manual reload.
- A second turn on the same page still renders live, proving the connection survives across turns in a real browser (not just in the backend's own test suite).

**Explicitly not automated:**
- A deterministic `TurnFailed` path: the fake LLM provider (`backend/src/bootstrap/providers.rs`) has no failure-injection knob today, and adding one is backend scope this frontend-only design doesn't extend into. The inline-error UI for `TurnFailed` is verified manually, not in CI.
- True broker-loss mid-turn (killing EMQX mid-test) — flaky and heavy for CI. The isolated proxy tests already cover the retry/give-up logic, which is the part actually owned by this sub-project.

## Out of Scope

- Fixing the fake LLM provider to support failure injection (would enable automating the `TurnFailed` e2e case above) — separate, backend-scoped work if ever wanted.
- Any bidirectional use of the browser↔SvelteKit WebSocket (e.g., sending the outgoing message over it instead of the POST form) — the POST form flow works today and isn't being replaced.
- Multi-tab/multi-device coordination beyond what falls out naturally from the connection being session-scoped (§2) — no explicit "another tab is already viewing this session" handling is added.
