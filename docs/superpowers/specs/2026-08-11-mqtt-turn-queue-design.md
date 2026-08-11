# MQTT Pub/Sub Infra + Turn-Loop Decoupling

Date: 2026-08-11
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-03-llm-provider-streaming-design.md` (sub-project 1 — `LlmProvider::complete_stream()`, `StreamEvent`, `collect_stream()`, all reused directly here). Combines sub-projects 2 ("MQTT/pub-sub infra") and 3 ("turn-loop decoupling") of 5 in the realtime chat effort — see the remaining two (WebSocket bridge, frontend realtime rendering), specced separately.

## Purpose

Sub-project 1 made LLM calls streamable but nothing consumes that stream — the turn loop still runs synchronously inside the HTTP request via the `crate::llm::complete()` free function, which drains a `complete_stream()` internally and returns one `LlmResponse`, and `POST /api/sessions/:id/messages` blocks until the whole turn finishes. This design moves turn processing out of the HTTP request entirely and onto a durable, separately-deployable worker that streams live token deltas onto an MQTT broker as they arrive — the piece a future WebSocket bridge (sub-project 4) will subscribe to.

Driving constraint: the backend is expected to run as multiple instances eventually. A durable Postgres-backed job queue (rather than an in-process `tokio::spawn`) means turn processing survives a process restart and can run on dedicated worker instances independent of whichever instance is holding a given client's connection — MQTT is the cross-instance bridge between whichever worker processed a turn and whichever instance eventually holds the WebSocket for it.

**This sub-project accepts a temporary regression**: `POST /messages` no longer returns `assistant_message` — until the WebSocket bridge and frontend rendering (sub-projects 4-5) land, the chat UI will not show new replies. This is mid-effort infrastructure, not a shippable state on its own.

## 1. New Table: `turn_jobs`

```sql
CREATE TABLE turn_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id UUID NOT NULL,
    text TEXT NOT NULL,
    org_id_hint UUID,
    status TEXT NOT NULL DEFAULT 'pending', -- pending | processing | completed | failed
    claimed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX turn_jobs_pending_idx ON turn_jobs (created_at) WHERE status = 'pending';
```

One row per inbound message that needs turn processing. `sender_channel_identity_id`, `text`, `org_id_hint` are exactly the arguments `handle_inbound_message` takes today beyond `session_id` — carried through the queue instead of a function call.

## 2. HTTP Server: Ingest Only

`POST /api/sessions/:id/messages` (`backend/src/routes/sessions.rs::send_message`) changes to:

1. `authorize_session_access` (unchanged).
2. Bootstrap identity/session (unchanged — reuses `turn::bootstrap::bootstrap_identity_and_session`, already called via `handle_inbound_message` today; the ingest phase calls it directly).
3. In one transaction: insert the user message into `messages` (same as today's `lock::insert_inbound_message`), insert a `turn_jobs` row, `NOTIFY turn_jobs_channel`.
4. Return `202 Accepted` with `{ user_message }` — no `assistant_message`.

The HTTP server no longer touches `LlmProvider`/`EmbeddingProvider` for this endpoint. `AppState.provider`/`AppState.embedding_provider` (and the admin settings routes' live in-memory swap of them, `backend/src/routes/settings.rs:158,221`) become dead for turn-processing purposes — the worker is a separate process and re-reads settings from the database itself (see §4). The admin settings PUT handlers keep persisting to the encrypted `provider_settings` row (unchanged), but the `*state.provider.write().await = new_provider` in-memory swap can be removed as part of this work — nothing reads it anymore.

## 3. MQTT: Broker and Client Wrapper

**Broker:** EMQX, added to `backend/docker-compose.yml` as a new service alongside `postgres`, using its default ports/config for local dev.

**Rust client:** `rumqttc` (async, tokio-native — the standard choice for a Rust MQTT client). New dependency in `backend/Cargo.toml`.

**Wrapper** (`backend/src/realtime/mqtt.rs`, new module):

```rust
pub struct MqttPublisher { client: rumqttc::AsyncClient }

impl MqttPublisher {
    pub fn connect(broker_url: &str, client_id: &str) -> (Self, /* spawned eventloop handle */);
    pub async fn publish(&self, session_id: Uuid, envelope: &StreamEnvelope) -> Result<(), MqttError>;
}
```

`connect` constructs the `rumqttc::AsyncClient`/`EventLoop` pair and spawns a background task that drives the `EventLoop` (required by `rumqttc` for the client to make progress) for the lifetime of the process. Only the worker constructs an `MqttPublisher` and calls `publish` — the HTTP server never touches MQTT, consistent with §2 (it only inserts a `turn_jobs` row and `NOTIFY`s).

**Topic:** `chat/{session_id}/stream` — one topic per session, carrying every event for every turn in that session.

**Envelope** (published as JSON, QoS 0):

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
}
```

`StreamEvent`/`PartialBlock` (`backend/src/llm/types.rs`, sub-project 1) gain `Serialize`/`Deserialize` derives to support this — no shape change, additive derives only. QoS 0 (at-most-once): a dropped delta is a UX hiccup for a future subscriber, never a correctness issue, since `messages`/`turn_jobs` in Postgres remain the source of truth regardless of what MQTT delivered.

## 4. Worker Binary

New `backend/src/bin/worker.rs`, linking the same `nomi_orchestrator` lib crate as `main.rs`.

**Startup:** same bootstrap as the HTTP server today — connect to Postgres, run migrations (idempotent, safe from either binary), construct an `MqttPublisher`. LLM/embedding provider construction (`build_llm_provider_from_settings_or_env`, `build_embedding_provider_from_settings_or_env`, currently private functions in `backend/src/main.rs`) moves to a new shared `backend/src/bootstrap/providers.rs` in the lib crate so both binaries call the same code instead of duplicating it. Unlike the HTTP server, the worker does **not** hold these in a hot-swappable `RwLock`: it re-reads and rebuilds the provider from `provider_settings` before processing each claimed job. This trades a small settings SELECT + decrypt per job (negligible next to an LLM call) for not needing any cross-process cache-invalidation mechanism — a deliberate simplification, not an oversight.

**Job discovery:** a `sqlx::postgres::PgListener` on a `turn_jobs_channel` (the same channel the HTTP server `NOTIFY`s) wakes the poll loop immediately when a job is inserted. A 5-second timer is the fallback in case a `NOTIFY` is ever missed (e.g. the listener reconnecting) — bounded worst-case latency, not silent starvation.

**Claim:**

```sql
WITH claimed AS (
    SELECT id FROM turn_jobs
    WHERE status = 'pending'
    ORDER BY created_at
    FOR UPDATE SKIP LOCKED
    LIMIT 1
)
UPDATE turn_jobs SET status = 'processing', claimed_at = now()
WHERE id IN (SELECT id FROM claimed)
RETURNING *;
```

`SKIP LOCKED` means multiple worker processes can run this same query concurrently and each claims a different job — this is what makes horizontal worker scaling safe without any additional coordination.

**Process:** acquire the session's advisory lock (`turn::lock::acquire_session_lock`, unchanged) — this still serializes concurrent turns for the same session exactly as today, just now enforced across worker processes instead of within one HTTP process — then run `turn::process_turn` (see §5). On success: mark the job `completed`, publish `TurnCompleted`. On failure: existing `TurnFailed` `agent_events` insert (unchanged), mark the job `failed` with the error, publish `TurnFailed`.

## 5. Turn Loop Changes

`turn::handle_inbound_message` is split:

- **Ingest** (bootstrap + insert message) moves into the HTTP handler (§2) — it no longer lives in the `turn` module's public entry point.
- **Everything from `lock::acquire_session_lock` onward** (routing, chitchat/subagent dispatch, the `TurnFailed` event on error, lock release) becomes `turn::process_turn(pool, mqtt, provider, embedding_provider, turn_job_id, session_id, sender_channel_identity_id, text, org_id_hint)` — same logic as today's `handle_inbound_message` body, unchanged, just renamed and now the worker's entry point instead of the HTTP handler's.

**Only `chitchat::run_chitchat_turn` changes internally.** Its single LLM call:

```rust
// today:
let response = crate::llm::complete(provider, request).await.map_err(TurnError::LlmCallFailed)?;

// becomes:
let stream = provider.complete_stream(request).await.map_err(TurnError::LlmCallFailed)?;
let published = stream.then(|event_result| async {
    if let Ok(event) = &event_result {
        mqtt.publish(session_id, &StreamEnvelope::Delta { turn_job_id, event: event.clone() }).await.ok();
        // best-effort — a publish failure is logged, never fails the turn (see §6)
    }
    event_result
});
let response = collect_stream(Box::pin(published)).await.map_err(TurnError::LlmCallFailed)?;
```

`collect_stream` (sub-project 1, unchanged) still does all response assembly — this is a tee, not a reimplementation. Everything after this call (persisting `reply_text`, the `ChitchatReply` `agent_events` row, memory extraction) is unchanged.

`memory.rs`, `routing.rs`, `tools.rs` keep calling `crate::llm::complete()` exactly as today — intent classification, memory extraction, and the money-agent's tool loop are never shown to a user token-by-token, so there's nothing to stream for them (same rationale as sub-project 1's design).

## 6. Error Handling

- **Worker crash mid-job:** a job claimed (`status='processing'`) when its worker dies stays stuck — no reaper/timeout-based requeue in this sub-project. Documented limitation: there's no real multi-instance deployment yet to actually trigger this, and building a stuck-job reaper without being able to exercise it under real conditions risks solving the wrong problem. Natural follow-up once workers run at scale.
- **MQTT publish failures:** logged, never fail the turn. Postgres (`messages`, `turn_jobs`) is always the durable source of truth; MQTT is a best-effort live side-channel. This matches the project's existing fail-open philosophy elsewhere (e.g. `chitchat.rs`'s memory extraction).
- **Missed `NOTIFY`:** bounded by the 5-second poll fallback (§4).
- **Turn failure** (LLM error, etc.): unchanged `TurnFailed` `agent_events` insert, plus the job row marked `failed` and a `TurnFailed` MQTT event for a future subscriber to stop showing a typing indicator.

## 7. Testing Approach

- **Claim concurrency:** two simulated workers (two `PgPool` connections) racing the claim query against the same set of pending jobs — assert every job is claimed by exactly one, proving `SKIP LOCKED` behaves correctly against this project's real Postgres test setup (matching existing DB-integration test patterns, e.g. `tests/app_state_reload.rs`).
- **MQTT round trip:** against the compose EMQX instance, subscribe to `chat/{session_id}/stream`, run a `FakeLlmProvider`-backed chitchat turn through `turn::process_turn`, assert the exact ordered `StreamEnvelope::Delta` sequence (matching `FakeLlmProvider::complete_stream`'s single-shot stream from sub-project 1) followed by `TurnCompleted`.
- **HTTP ingest:** update `send_message`'s existing tests to assert `202` + `user_message`-only body, and that a `turn_jobs` row exists afterward with `status='pending'`.
- **Existing turn-loop tests** (chitchat, routing, memory, tools — e.g. `turn_chitchat.rs`, `turn_money_agent.rs`) continue to drive `turn::process_turn` directly, bypassing HTTP and the queue entirely, same as they drive `handle_inbound_message` today — only the function name and the split-out ingest arguments change, not the test structure.

## Out of Scope

- The WebSocket bridge and any client actually subscribing to `chat/{session_id}/stream` (sub-project 4).
- Frontend changes of any kind (sub-project 5) — `POST /messages` returning `202` without `assistant_message` is a known, accepted regression until then.
- A stuck-job reaper for crashed workers (§6) — deferred until multi-instance deployment is real.
- MQTT topic ACLs / broker-level auth — not needed yet since only backend-internal processes (worker, future WS-bridge instances) talk to the broker; no external/browser client connects to MQTT directly at any point in this effort.
