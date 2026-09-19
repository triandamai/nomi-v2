# Admin Command Center — Design

**Goal:** Replace the static, load-once `/admin/agents` page with a live-updating dashboard —
a table of every currently-active agent that adds/updates/removes rows in real time, a scrolling
activity feed of what agents have been doing, and a per-agent drill-down — all without ever
exposing conversation content to the admin viewing it.

**Architecture:** A new admin-only WebSocket endpoint subscribes to a wildcard MQTT topic
(`chat/+/stream`) that already receives every session's realtime events — no new message bus.
Two new `StreamEnvelope` variants (`AgentSessionStarted`/`AgentSessionEnded`) are published from
the three places `agent_sessions` rows are created/closed, closing the one real gap in today's
realtime coverage (in-flight phase changes are already published; session start/end isn't). The
relay filters server-side to a fixed allow-list of event kinds carrying only metadata (agent
identity, phase, tool *names*, timestamps) — message content (`Delta`, `MessageCreated`,
`MessageUpdated`) is never forwarded, so it cannot reach the admin browser even via devtools. The
existing `agent_events` table (already persists every relevant event) backs both the feed's
history and — filtered by `agent_session_id` — the drill-down, with no new storage.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, the existing MQTT/WS realtime relay (`nomi-realtime`, `rumqttc`), the
existing Node WS proxy (`frontend/ws-proxy/`, `frontend/server.js`, `frontend/vite-plugins/`).

## Context

`/admin/agents` (added in the dynamic-agents-and-live-status feature) queries
`agent_sessions WHERE status = 'active'` once, server-side, on page load — it never updates while
open. Two things make a live version non-trivial:

1. **The realtime relay is per-session today.** `MqttPublisher::publish(session_id, envelope)`
   writes to topic `chat/{session_id}/stream`; the only WS route (`GET /api/sessions/:id/ws`)
   subscribes to exactly one such topic. There is no existing "subscribe to everything" path.
2. **Nothing is published when an agent starts or stops.** `StreamEnvelope::AgentPhaseChanged`
   (already shipped) fires only *during* an active turn. `routing::spawn_agent_session`,
   `routing::complete_agent_session`, and `routing::mark_expired` — the only three places
   `agent_sessions` rows are created or closed, all called from one place,
   `nomi-turn/src/lib.rs`'s `run_locked_turn` — only write to Postgres today.

Both gaps are additive to what already exists; nothing about the current per-session realtime
path changes.

## Data Model

No new tables. `agent_events` (existing, from migration `0006_memory_and_events.sql`) already
persists `AgentSpawned`, `AgentCompleted`, `AgentCancelled`, `AgentExpired`, `ToolCalled`, and
`AgentReplied` rows with `session_id`, `agent_session_id`, `agent_type`, `event_type`, and a
`payload JSONB` — this backs the feed's history and the drill-down's per-agent slice.

## New `StreamEnvelope` Variants

```rust
// backend/crates/nomi-realtime/src/lib.rs — added to the existing enum
/// An agent_sessions row was created — an agent started working. `agent_display_name` and
/// `sender_label` are resolved once at spawn time (mirroring how `messages.agent_display_name`
/// is resolved at insert time) so the dashboard never needs a follow-up lookup per event.
AgentSessionStarted {
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: String,
    agent_display_name: String,
    sender_label: String,
},
/// An agent_sessions row was closed — `reason` is one of "completed" | "cancelled" | "expired",
/// matching `agent_sessions.status`'s possible values for a non-active row.
AgentSessionEnded { agent_session_id: Uuid, session_id: Uuid, reason: String },
```

`session_id` is included directly in these two variants (unlike `AgentPhaseChanged`, which only
carries `agent_session_id` — the admin relay injects `session_id` for that one by parsing it out
of the MQTT topic string at forward time, since the topic already encodes it and no producer-side
change is needed there).

## Backend: Publishing Session Start/End

`routing::spawn_agent_session`, `routing::complete_agent_session`, and `routing::mark_expired`
each gain an `mqtt: Option<(&MqttPublisher, Uuid)>` parameter (same shape already threaded through
every other engine/turn function) and publish the corresponding event after their existing
Postgres write, best-effort — matching this codebase's established "an MQTT publish failure never
fails the turn" convention used everywhere else in the realtime path.

`spawn_agent_session` gains an `agent_display_name: &str` parameter — its one call site (in
`run_locked_turn`) already holds `agent: Arc<dyn SubAgent>` and can pass
`agent.display_name().as_ref()` directly, so `routing.rs` itself never needs agent-identity logic.
`sender_label` is resolved inside `spawn_agent_session` with one query mirroring
`admin_dashboard::get_agents`'s existing email-or-`channel:channel_user_id` JOIN — self-contained,
no new parameter needed for it.

`complete_agent_session` and `mark_expired` publish `AgentSessionEnded` with `reason` set to
their own `status`/the literal `"expired"` respectively — no new data to resolve.

## Backend: Admin WebSocket Relay

`GET /api/admin/agents/ws` (new handler in `nomi-server/src/routes/admin_dashboard.rs`), gated by
the same `require_system_config_permission` check as every other admin route, before the
WebSocket upgrade (the same pattern `session_stream`'s handler already uses for its own
pre-upgrade `authorize_session_access` check).

The relay task subscribes to `chat/+/stream` (MQTT's single-level wildcard — matches every
session's topic in one subscription) instead of one session's topic. For each incoming publish,
it:

1. Parses `session_id` out of the topic string (`chat/{session_id}/stream`).
2. Deserializes the payload into `StreamEnvelope` and matches on its kind. Only
   `AgentSessionStarted`, `AgentSessionEnded`, and `AgentPhaseChanged` are forwarded;
   `Delta`, `MessageCreated`, `MessageUpdated`, and `AgentDelegationUpdated` are dropped
   unconditionally. This filter lives in the relay itself, not the frontend — message content is
   never serialized onto this connection in the first place. `ToolCalled`/`AgentReplied` are
   never forwarded live at all — they aren't `StreamEnvelope`/MQTT-published events today (they're
   `agent_events` rows written directly by the turn engine), and this spec deliberately doesn't
   add a new publish for them; see "Feed live updates" below for how the feed represents tool
   calls without one.
3. Re-serializes with `session_id` injected and forwards as a WS text frame — same
   `serde_json::Value`-based field injection, not a raw string patch.

**Feed live updates without a new publish per tool call:** publishing an MQTT event on every
`ToolCalled`/`AgentReplied` would mean editing `nomi-agent-core/src/engine.rs`'s hot path for a
purely observational feature. Instead, the relay's forwarded `AgentPhaseChanged { phase:
"calling_tool", detail: Some(tool_name), .. }` transitions **are** "a tool was just called" —
the feed renders those directly (e.g. *"Money agent called list_transactions"*) instead of
needing a dedicated event. `AgentSessionStarted`/`AgentSessionEnded` cover the spawn/finish feed
entries. This keeps the turn engine's hot path completely unchanged.

## Backend: Feed & Drill-Down History Endpoint

`GET /api/admin/agent-events?limit=50&session_id=<uuid>` (new handler, same file), admin-gated.
`session_id` is optional: omitted for the top-level feed's backfill (most recent 50 across all
sessions), provided for the drill-down's per-agent history. Queries `agent_events` for
`event_type IN ('AgentSpawned', 'AgentCompleted', 'AgentCancelled', 'AgentExpired', 'ToolCalled',
'AgentReplied')`, returning `id, session_id, agent_session_id, agent_type, event_type, created_at`
and, from `payload`, only `tool_name`/`is_error` (for `ToolCalled`) — **never** `input` or
`result`, which is where a tool call's actual data (file contents, transaction rows) lives. This
is the same content boundary as the live relay, enforced at the same layer (SQL projection, not a
frontend filter).

## Node WS Proxy

The browser cannot attach an `Authorization` header to a raw `WebSocket()`, so every WS route
goes through `frontend/ws-proxy/session-stream-proxy.js` today (cookie → bearer-token exchange,
reconnect-with-backoff), wired into both `frontend/vite-plugins/dev-ws-proxy.ts` (dev/preview) and
`frontend/server.js` (production, `adapter-node`). Today it hardcodes one path pattern
(`SESSION_WS_PATH`) and one upstream URL shape. This gets generalized into a small route table —
`{ pattern: RegExp, upstreamPath: (match) => string }[]` — so both `/chat/:id/ws` and the new
`/api/admin/agents/ws` share the same auth/reconnect logic via one `attachWsProxy(server, routes)`
function, instead of duplicating the ~150 lines of relay/backoff logic for a second route.

## Frontend

`/admin/agents` (`frontend/src/routes/admin/(protected)/agents/`) is rewritten in place — same
nav entry, same URL, no new page sitting confusingly next to "Dynamic Agents":

- **Live table** (top): seeded by the existing `GET /api/admin/agents` (unchanged — still the
  initial snapshot), then kept live by one shared WebSocket connection to
  `/api/admin/agents/ws`: `AgentSessionStarted` inserts a row, `AgentPhaseChanged` updates a row's
  phase/detail in place, `AgentSessionEnded` removes it. Same columns as today's static table
  (user, agent, channel, phase, started, last activity).
- **Activity feed** (below the table): seeded by `GET /api/admin/agent-events` (no `session_id`),
  prepended live from the same socket's `AgentSessionStarted`/`AgentSessionEnded`/
  `AgentPhaseChanged{phase: "calling_tool"}` frames, rendered as short lines (*"Weather Bot
  started"*, *"Money agent called list_transactions"*, *"Weather Bot finished (completed)"*).
- **Drill-down**: clicking a table row opens the existing `SideSheet` component
  (`frontend/src/lib/components/m3/SideSheet.svelte`, `open`/`children` props, already built for
  the agent-plan-artifacts feature) showing that row's agent identity, `session_id`, and its own
  feed — either fetched fresh via `GET /api/admin/agent-events?session_id=...` on open, or filtered
  client-side from frames already received on the shared socket (implementation detail for the
  plan to settle; both are correct, the fetch is simpler and the filter avoids a network round
  trip — no user-visible difference either way). No message content is fetched or rendered
  anywhere in the drill-down.

## Error Handling

- Every new MQTT publish (`AgentSessionStarted`/`AgentSessionEnded`) is best-effort, matching
  every existing publish call in this codebase — a broker hiccup never fails a turn.
- The admin WS relay's per-frame deserialize-and-filter step drops (does not forward, does not
  error) any envelope kind it doesn't recognize or that isn't on the allow-list — forward
  compatible with future `StreamEnvelope` variants that shouldn't reach this dashboard by default.
- If the admin socket drops, the frontend reconnects with the same backoff pattern
  `ChatThread.svelte` already implements, and on reconnect re-fetches `GET /api/admin/agents` +
  `GET /api/admin/agent-events` to reconcile anything missed while disconnected (same
  "`invalidateAll()` on reconnect" pattern `ChatThread.svelte` uses today).

## Testing

- Backend: an end-to-end WS test (same shape as `nomi-server/tests/session_ws.rs`'s
  `try_connect_ws`/`expected_frames_for` pattern — a real `tokio_tungstenite` client against a
  real bound TCP listener) asserting `AgentSessionStarted` and `AgentSessionEnded` frames arrive
  on `/api/admin/agents/ws` when a turn spawns and completes an agent session, and that a `Delta`
  published to the same session's topic during that turn never reaches the admin socket; a test
  asserting `GET /api/admin/agent-events`'s response for a `ToolCalled` row never contains
  `input`/`result` keys (only `tool_name`/`is_error`); admin-gating tests (403 for a non-admin,
  both for the two new HTTP endpoints and the WS upgrade) matching every other admin route's
  existing test shape.
- Frontend: `svelte-check` clean; a live look at the dashboard while a real turn runs (this is
  exactly the class of realtime, timing-dependent behavior where watching it actually work matters
  more than an assertion — same standing exception already used for this codebase's other
  WS-driven UI).

## Out of Scope

- **Editing/cancelling an agent from the dashboard** — this is a read-only observability view;
  no admin action (force-complete, kill) is added here.
- **Historical dashboards beyond the most recent N events** — no pagination, no date-range query,
  no export. The feed is a live tail with a shallow backfill, not an analytics view.
- **Non-admin visibility** — this entire feature stays behind
  `require_system_config_permission`, same as `/admin/agents` today; no user-facing equivalent is
  part of this spec (the existing per-session `agent-status` endpoint from the prior feature
  already covers a user's own live status within their own chat).
