# Multi-Agent Supervisor — Design

## Goal

Chitchat (the default conversational agent) can hand a task off to a specialist agent (money, personality, ...) to work on **in the background** — acknowledging immediately ("I'll check and get back to you") and continuing the conversation, rather than blocking on it. When the specialist finishes, a **supervisor** agent — sharing the same personality/"soul" as every other agent, but with a distinct job (coordination and reporting) — delivers the result into the open chat asynchronously, without the user having sent a new message. The supervisor is also directly reachable for explicit cross-agent status queries ("what are you all doing, give me a report"). Not every question triggers any of this: simple questions are still answered directly by whichever agent is already active, using its own tools/memory, exactly as today.

The chat UI gets a small indicator near the input area showing background agent activity for the current session; clicking it opens the `BottomSheet` component (already built) listing active/recent delegations.

## Current State (verified against the code, not assumed)

### Turn execution

- `nomi_agent_core::run_agent_turn` (`nomi-agent-core/src/engine.rs:42-196`) is the tool-calling loop: up to `MAX_TOOL_TURNS = 10` (line 13) LLM round-trips against `provider: &dyn LlmProvider`. Every tool call is `agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input).await` (line 170) — **synchronous, awaited inline, using the same connection the whole turn already holds**. A sentinel tool `complete_task` (`COMPLETE_TASK_TOOL_NAME`, line 14) is appended to every agent's tool list (line 55) and special-cased in the loop (line 167) to short-circuit with `LoopOutcome::Completed`.
- `nomi_turn::process_turn`/`run_locked_turn` (`nomi-turn/src/lib.rs:74-187`) wraps a turn in a **Postgres advisory lock held for the entire turn** (`lock::acquire_session_lock`, held across every LLM round-trip and tool call, released only at the very end). Any code that runs another agent's full turn synchronously from inside a tool call would (a) block the calling agent's own reply until the delegated work finishes, and (b) hold the session lock the whole time, blocking the user's next message too — exactly the two things this feature must avoid.
- `run_subagent_turn` (`nomi-turn/src/lib.rs:190-269`) is the only production caller of `run_agent_turn`. For the **default agent (chitchat)**, it passes `agent_session_id = session_id` as a sentinel — chitchat never gets a real `agent_sessions` row (`run_locked_turn`, line 179, with the comment at lines 174-178 confirming this explicitly). Downstream, `run_subagent_turn` itself checks `if agent_session_id == session_id { None } else { Some(agent_session_id) }` (lines 237-238) before binding to `agent_events.agent_session_id`, because that column has a foreign key into `agent_sessions` and the sentinel value doesn't correspond to a real row. **This exact sentinel pattern is the established codebase convention for "no real `agent_sessions` row" — this design reuses it rather than introducing a different one.**
- `agent_sessions` (`migrations/0005_agent_sessions.sql`) has `CREATE UNIQUE INDEX agent_sessions_one_active_per_speaker ON agent_sessions (session_id, sender_channel_identity_id) WHERE status = 'active'` — **regardless of `agent_type`**. A background delegation must NOT create a real `agent_sessions` row: doing so would occupy this slot for the whole delegation's lifetime, and any direct `classify_intent`-routed message from the same speaker to *any* specialist agent (not just the delegation's target) would then either collide on insert or, worse, get silently appended into the delegation's own in-flight tool-loop conversation via `find_active_agent_session` matching the delegation's row. **This is the concrete reason delegations get their own dedicated table instead of reusing `agent_sessions`, not just a simplification.**

### `SubAgent` / `AgentRegistry`

- `SubAgent` trait (`nomi-agent-core/src/subagent.rs:10-48`): `agent_type()`, `system_prompt()`, `tools()`, `execute_tool()`, `intent_label()`/`intent_description()`, `is_default()` (default `false`), `uses_memory()` (default `false`), `uses_personality()` (default `false`).
- `AgentRegistry::new` (`nomi-agent-core/src/registry.rs:12-28`) **panics** unless exactly one registered agent returns `true` from `is_default()`.
- `AgentRegistry::classification_prompt()` (registry.rs:50-72) builds the intent classifier's prompt from every *non-default* agent's `intent_label`/`intent_description` — fully generic, nothing to edit here when a new agent is registered.
- `nomi_server::build_agent_registry()` (`nomi-server/src/lib.rs:10-15`) is the single wiring point: currently `ChitchatAgent` (default), `MoneyAgent`, `PersonalityAgent`.
- `ChitchatAgent` (`nomi-agent-chitchat/src/lib.rs`): `tools()` returns `vec![]`, `uses_memory() = true`, `uses_personality() = true`, `is_default() = true`.
- `nomi_agent_core::personality::get_current_personality` (`nomi-agent-core/src/personality.rs:8`) is `pub` — reusable from any crate that already depends on `nomi-agent-core`.

### Transport — the key enabling fact

- `MqttPublisher::publish<T: Serialize>(&self, session_id: Uuid, envelope: &T)` (`nomi-realtime/src/mqtt.rs:42-47`) publishes to MQTT topic `chat/{session_id}/stream`. It is **not tied to any live HTTP request** — `MqttPublisher::connect` (mqtt.rs:20-40) spawns an independent, process-lifetime background task.
- `relay_session_stream` (`nomi-server/src/routes/sessions.rs:272-299`, backing the `session_stream` WS handler at line ~260) opens its own independent MQTT subscriber per WebSocket connection and forwards every publish on that topic **verbatim** into the socket. No server-side coupling to whatever request originally started the turn.
- `worker.rs` (`nomi-server/src/worker.rs`) already proves this pattern in production: it's a fully separate async task/process from the HTTP request handler, yet successfully pushes `StreamEnvelope::TurnCompleted`/`TurnFailed` into a session's open WebSocket. **No new transport infrastructure is needed for "background work finishes, pushes into the open chat" — only a new `StreamEnvelope` variant and a frontend handler for it.**
- `StreamEnvelope` (`nomi-realtime/src/lib.rs:10-16`) is a closed, `#[serde(tag = "kind")]` enum: `Delta`, `TurnCompleted`, `TurnFailed`.

### `turn_jobs` — the pattern to mirror, not to reuse directly

- Schema (`migrations/0011_turn_jobs.sql`): `id, session_id, sender_channel_identity_id, text, org_id_hint, status DEFAULT 'pending', claimed_at, completed_at, error, created_at`. One partial index: `turn_jobs_pending_idx ON turn_jobs (created_at) WHERE status = 'pending'`.
- Producer: `nomi_turn::queue::enqueue` (`nomi-turn/src/queue.rs:14-38`) — `INSERT` then `SELECT pg_notify('turn_jobs_channel', $1)`, both in the same transaction.
- Consumer: `worker.rs::run` (lines 19-106) — `PgListener` on `turn_jobs_channel`, wakes on `NOTIFY` or a 5-second fallback timeout, drains every pending job via `queue::claim_next` (`FOR UPDATE SKIP LOCKED`, queue.rs:40-63 — safe under concurrent workers), resolves `user_id`, builds fresh per-user LLM/embedding providers, runs the turn, marks the job `completed`/`failed`.
- Wired in by default: `main.rs` spawns `worker::run` inline unless `RUN_WORKER_INLINE=false` (main.rs, documented at worker.rs:13-18).
- This table models "raw inbound text from a channel," not "a task one agent handed to another with its own requesting/target agent context" — a new table is the right shape, but every structural element (status enum, `SKIP LOCKED` claiming, `LISTEN`/`NOTIFY` wake-up, a decoupled worker loop, MQTT result delivery) is copied from this one.

### Tool shape (for a concrete example)

- `ToolDefinition` (`nomi_llm`, used at `engine.rs:16-29`, `nomi-agent-money/src/lib.rs:34-56`): `{ name: String, description: String, input_schema: serde_json::Value }` — a raw JSON Schema object.
- `MoneyAgent::execute_tool` (`nomi-agent-money/src/lib.rs`) is a plain `match name { "list_transactions" => ..., "summarize_budget" => ..., other => Err(...) }`.

## Design

### 1. Delegation is a tool, not a new routing path

`classify_intent`'s existing behavior (one agent runs the whole turn, chosen by a single classification LLM call) is **unchanged** — a message that's clearly single-domain ("what's my balance") still routes straight to the money agent synchronously, exactly as today.

This adds a second, independent mechanism: any agent that opts in gets an extra tool, `delegate_to_agent`, alongside its own tools and `complete_task`. The LLM decides whether to call it — this is how "not every question needs delegation" falls out for free, using the exact same judgment mechanism the model already applies to every other tool choice.

- `SubAgent` gains two new default methods (`subagent.rs`):
  ```rust
  /// When true, run_agent_turn gives this agent an extra `delegate_to_agent` tool that hands
  /// a task to another registered specialist to run in the background.
  fn can_delegate(&self) -> bool {
      false
  }

  /// When false, this agent is never offered as a delegation target — used by the supervisor
  /// agent, which delivers/reports on delegated work rather than being delegated to itself.
  fn is_delegation_target(&self) -> bool {
      true
  }
  ```
- `ChitchatAgent` overrides `can_delegate() -> bool { true }`. No other existing agent changes.
- `AgentRegistry` gains one method (`registry.rs`):
  ```rust
  /// agent_type() of every non-default, delegation-eligible agent other than `excluding` —
  /// the valid target list for a delegate_to_agent tool call from that agent.
  pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<&'static str> {
      self.agents
          .iter()
          .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type() != excluding)
          .map(|a| a.agent_type())
          .collect()
  }
  ```

### 2. `run_agent_turn` gains a `registry: &AgentRegistry` parameter

Needed so the tool list can be built with the caller's own `agent_type()` excluded and the current set of valid delegation targets (the tool's JSON Schema `enum` is generated from the live registry, not hand-maintained). This is the only signature change to existing code in this feature.

- `engine.rs`: `run_agent_turn(conn, mqtt, provider, embedding_provider, registry: &AgentRegistry, agent, session_id, agent_session_id, user_id, messages, max_tokens)` — insert the new parameter (position: after `embedding_provider`, before `agent`, to keep it near the other by-reference context arguments).
- Tool list construction (currently `engine.rs:54-55`) becomes:
  ```rust
  let mut tools = agent.tools();
  tools.push(complete_task_tool_definition());
  if agent.can_delegate() {
      tools.push(delegate_tool_definition(registry, agent.agent_type()));
  }
  ```
- New `DELEGATE_TOOL_NAME` const and `delegate_tool_definition` function, same file, same shape as `complete_task_tool_definition`:
  ```rust
  pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";

  fn delegate_tool_definition(registry: &AgentRegistry, requesting_agent_type: &str) -> ToolDefinition {
      let targets = registry.delegatable_agent_types(requesting_agent_type);
      ToolDefinition {
          name: DELEGATE_TOOL_NAME.to_string(),
          description: format!(
              "Hand off a task to a specialist agent to work on in the background, and immediately \
               tell the user you'll follow up — do NOT wait for the result before replying. Use this \
               only when the request genuinely needs a specialist; answer anything else yourself. \
               Available specialists: {}.",
              targets.join(", "),
          ),
          input_schema: serde_json::json!({
              "type": "object",
              "properties": {
                  "target_agent": {"type": "string", "enum": targets, "description": "Which specialist to delegate to"},
                  "task": {"type": "string", "description": "What to ask the specialist to do, in your own words"}
              },
              "required": ["target_agent", "task"]
          }),
      }
  }
  ```
- **Only production call site to update:** `run_subagent_turn` (`nomi-turn/src/lib.rs:190-269`) gains a `registry: &AgentRegistry` parameter and threads it into its `run_agent_turn` call; its two call sites, both inside `run_locked_turn` (lines 169 and 179/183), already have `registry` in scope as one of `run_locked_turn`'s own parameters — passing it down is a one-argument addition at each site, not a new plumbing path.

### 3. Executing the delegation tool call — fast, non-blocking

Special-cased in `run_agent_turn`'s loop (`engine.rs`, alongside the existing `COMPLETE_TASK_TOOL_NAME` branch at line 167), **before** falling through to `agent.execute_tool()`:

```rust
let (result_text, is_error) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
    ...
} else if name.as_str() == DELEGATE_TOOL_NAME {
    let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
    let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
    match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type(), target_agent, task, user_id).await {
        Ok(ack) => (ack, false),
        Err(err) => (err, true),
    }
} else {
    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
        Ok(text) => (text, false),
        Err(err) => (err, true),
    }
};
```

This does **not** short-circuit the loop the way `complete_task` does — the delegating agent's own LLM call continues normally on the next iteration, now with a tool result telling it the delegation was created, so it composes its own next message (e.g. "I'll check and get back to you") within the same turn, same `MAX_TOOL_TURNS` budget, zero changes to the loop's control flow otherwise.

`create_delegation` (new module `nomi-agent-core/src/delegation.rs`, alongside `memory.rs`):

```rust
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_realtime::{MqttPublisher, StreamEnvelope};

pub async fn create_delegation(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    requesting_agent_type: &str,
    target_agent_type: &str,
    task: &str,
    user_id: Uuid,
) -> Result<String, String> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(requesting_agent_type)
    .bind(target_agent_type)
    .bind(task)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| format!("failed to create delegation: {e}"))?;

    let _ = sqlx::query("SELECT pg_notify('agent_delegations_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **conn)
        .await;

    if let Some((publisher, _)) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: id }).await;
    }

    Ok(format!(
        "Delegated to {target_agent_type}. Tell the user you'll follow up once it's done — do not wait for the result now."
    ))
}
```

No upfront validation that `target_agent_type` is a real, registered agent — matching `turn_jobs`' own philosophy of a cheap insert with validation/failure deferred to claim time, where the worker already has `registry` in scope to check.

The `mqtt` publish here is genuinely best-effort and may be skipped: `run_agent_turn`'s `mqtt` parameter is `None` when called via `handle_inbound_message` (the synchronous, non-web-chat ingestion path — confirmed at `nomi-turn/src/lib.rs:47`, always passes `None`). The web chat flow goes through the `turn_jobs` queue → `process_turn`, which always supplies `Some((mqtt, turn_job_id))` — this is the only path this feature's frontend cares about, and it always has `mqtt` available.

### 4. Migration: `agent_delegations`

`backend/migrations/0016_agent_delegations.sql`:

```sql
CREATE TABLE agent_delegations (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id             UUID NOT NULL REFERENCES sessions(id),
    user_id                UUID NOT NULL REFERENCES users(id),
    requesting_agent_type  TEXT NOT NULL,
    target_agent_type      TEXT NOT NULL,
    task                   TEXT NOT NULL,
    status                 TEXT NOT NULL DEFAULT 'pending',
    result                 TEXT,
    error                  TEXT,
    claimed_at             TIMESTAMPTZ,
    completed_at           TIMESTAMPTZ,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX agent_delegations_pending_idx ON agent_delegations (created_at) WHERE status = 'pending';
CREATE INDEX agent_delegations_session_idx ON agent_delegations (session_id, created_at DESC);
```

`user_id` is stored directly (unlike `turn_jobs`, which derives it from `sender_channel_identity_id` at claim time via a join) because `run_agent_turn` already has `user_id` directly in scope where `create_delegation` is called — no extra lookup needed.

### 5. `StreamEnvelope` gains one variant

`nomi-realtime/src/lib.rs`:

```rust
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
    AgentDelegationUpdated { delegation_id: Uuid },
}
```

One shared variant for both "a delegation was just created" (published by `create_delegation`, above) and "a delegation just completed or failed" (published by the worker, below) — the frontend's reaction to both is identical (`invalidateAll()`, which refetches messages *and* agent-activity together), so one signal covers both without over-specifying.

### 6. Background delegation worker

New file `nomi-server/src/delegation_worker.rs`, structurally mirroring `worker.rs` exactly (LISTEN/NOTIFY, `SKIP LOCKED` claiming, per-user provider construction) but:
- Does **not** acquire the conversational session's advisory lock (`lock::acquire_session_lock` is never called) — this is the whole point; it must never block the user's live conversation.
- Calls `nomi_agent_core::run_agent_turn` directly (not `nomi_turn::process_turn`/`run_locked_turn`), passing:
  - `agent_session_id = claimed.session_id` — the same sentinel value the codebase already uses for the default agent (see Current State above); this is fine to reuse here because the worker never calls `run_subagent_turn` (which is the only place that sentinel comparison happens) — the worker does its own result-handling instead (below).
  - `mqtt = None` — no live `Delta` streaming for delegated work in this first version (the user isn't watching this thread render token-by-token the way they watch their own live turn; only the final result matters for the UX described). A later pass can add streaming if it turns out to matter.
  - `messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: claimed.task }] }]` — the delegated agent starts fresh, with the task as its only "message."
  - `max_tokens` — reuse `nomi_turn`'s existing `SUBAGENT_MAX_TOKENS` constant value (1024) via a local const of the same value (no cross-crate export needed for one constant).

```rust
use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::{ContentBlock, LlmMessage, LlmRole, LoopOutcome};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};

const NOTIFY_CHANNEL: &str = "agent_delegations_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);
const DELEGATED_MAX_TOKENS: u32 = 1024;

struct ClaimedDelegation {
    id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    target_agent_type: String,
    task: String,
}

async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedDelegation>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM agent_delegations \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE agent_delegations SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, user_id, target_agent_type, task",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, session_id, user_id, target_agent_type, task)| ClaimedDelegation {
        id, session_id, user_id, target_agent_type, task,
    }))
}

pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String) {
    let mut listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "delegation worker: failed to connect LISTEN client; not started");
            return;
        }
    };
    if let Err(e) = listener.listen(NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "delegation worker: failed to LISTEN on agent_delegations_channel; not started");
        return;
    }
    tracing::info!("delegation worker: listening for new agent delegations");

    let registry = crate::build_agent_registry();

    loop {
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

        loop {
            let claimed = match claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "delegation worker: failed to claim next delegation");
                    break;
                }
            };

            let Some(agent) = registry.find(&claimed.target_agent_type) else {
                tracing::warn!(delegation_id = %claimed.id, target = %claimed.target_agent_type, "delegation worker: unknown target agent");
                fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, "unknown target agent").await;
                continue;
            };

            let provider = build_llm_provider_for_user(&pool, claimed.user_id, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let mut conn = match pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                    continue;
                }
            };

            let messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: claimed.task.clone() }] }];

            let outcome = nomi_agent_core::run_agent_turn(
                &mut conn,
                None,
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                agent,
                claimed.session_id,
                claimed.session_id,
                claimed.user_id,
                messages,
                DELEGATED_MAX_TOKENS,
            )
            .await;

            match outcome {
                Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
                    let phrased = nomi_agent_supervisor::phrase_delegation_result(
                        provider.as_ref(),
                        &mut conn,
                        claimed.user_id,
                        &claimed.target_agent_type,
                        &claimed.task,
                        &text,
                    )
                    .await
                    .unwrap_or(text.clone());

                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&phrased)
                        .execute(&mut *conn)
                        .await;

                    let _ = sqlx::query("UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1")
                        .bind(claimed.id)
                        .bind(&text)
                        .execute(&pool)
                        .await;

                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;
                }
                Err(e) => {
                    tracing::warn!(delegation_id = %claimed.id, error = %e, "delegation worker: delegated turn failed");
                    let sorry = format!(
                        "I wasn't able to get an answer from the {} agent — {}.",
                        claimed.target_agent_type, e
                    );
                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&sorry)
                        .execute(&mut *conn)
                        .await;
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                }
            }
        }
    }
}

async fn fail_and_notify(pool: &PgPool, mqtt: &MqttPublisher, delegation_id: Uuid, session_id: Uuid, error: &str) {
    let _ = sqlx::query("UPDATE agent_delegations SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(delegation_id)
        .bind(error)
        .execute(pool)
        .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id }).await;
}
```

Note: the exact re-export paths for `ContentBlock`/`LlmMessage`/`LlmRole`/`LoopOutcome` (`nomi_agent_core` vs. `nomi_llm`) need confirming against `nomi-agent-core/src/lib.rs`'s actual `pub use` list during implementation — the plan must verify this precisely rather than assume the module path shown above is exact.

Wired into `main.rs` identically to the existing worker, as a second inline-spawned task (own `MqttPublisher` instance, own client id):

```rust
if run_worker_inline {
    // ... existing turn-jobs worker spawn, unchanged ...

    let delegation_mqtt_client_id = format!("nomi-orchestrator-delegation-worker-{}", uuid::Uuid::new_v4());
    let delegation_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &delegation_mqtt_client_id);
    let delegation_pool = pool.clone();
    let delegation_http_client = http_client.clone();
    let delegation_database_url = database_url.clone();
    tokio::spawn(async move {
        nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, settings_key, delegation_http_client, delegation_database_url).await;
    });
}
```

Reuses the same `RUN_WORKER_INLINE` flag as the existing worker — no new environment variable; both workers are opt-out together, matching this codebase's existing "one flag controls whether this process does background work" convention.

### 7. New crate: `nomi-agent-supervisor`

`backend/crates/nomi-agent-supervisor/Cargo.toml`, mirroring `nomi-agent-money`'s shape exactly (dependencies: `nomi-agent-core`, `nomi-llm`, `async-trait`, `serde_json`, `sqlx`, `uuid`; dev-dependency `nomi-test-support`).

`src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const SUPERVISOR_AGENT_TYPE: &str = "supervisor";

const SUPERVISOR_SYSTEM_PROMPT: &str =
    "You are the coordinator among a small team of specialist agents. When asked what the team \
     is doing, or for a status report, use list_recent_agent_activity and summarize it plainly — \
     what was asked, of whom, and the outcome if it finished. You never do the specialist work \
     yourself; you only report on it.";

pub struct SupervisorAgent;

#[async_trait]
impl SubAgent for SupervisorAgent {
    fn agent_type(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        SUPERVISOR_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "list_recent_agent_activity".to_string(),
            description: "List recent and currently active background delegations for this chat session.".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<String, String> {
        match name {
            "list_recent_agent_activity" => list_recent_agent_activity(conn, session_id).await,
            other => Err(format!("supervisor has no tool named {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "The user is asking what the other agents are doing, wants a status update, or explicitly asks for a report across multiple specialists"
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn is_delegation_target(&self) -> bool {
        false
    }
}

async fn list_recent_agent_activity(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<String, String> {
    let rows: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT target_agent_type, task, status, result, error FROM agent_delegations \
         WHERE session_id = $1 ORDER BY created_at DESC LIMIT 10",
    )
    .bind(session_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No background delegations for this session yet.".to_string());
    }

    let mut out = String::new();
    for (target, task, status, result, error) in rows {
        out.push_str(&format!("- {target} ({status}): asked to \"{task}\"", target = target, status = status, task = task));
        if let Some(r) = result {
            out.push_str(&format!(" — result: {r}"));
        }
        if let Some(e) = error {
            out.push_str(&format!(" — error: {e}"));
        }
        out.push('\n');
    }
    Ok(out)
}

/// One-shot, tool-free completion that phrases a completed delegation's raw result in the
/// supervisor's voice, folding in the user's personality the same way run_agent_turn does for
/// any agent with uses_personality() == true (duplicated here, not shared, because this path
/// intentionally doesn't go through run_agent_turn's full tool loop — see the design doc).
pub async fn phrase_delegation_result(
    provider: &dyn nomi_llm::LlmProvider,
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    target_agent_type: &str,
    task: &str,
    raw_result: &str,
) -> Result<String, String> {
    let mut system = format!(
        "{SUPERVISOR_SYSTEM_PROMPT}\n\nDeliver this result from the {target_agent_type} agent to the user, in your \
         own voice, as if reporting back after they asked you to look into something. Be natural and brief.\n\n\
         What was asked: {task}\nRaw result: {raw_result}"
    );
    if let Some(p) = nomi_agent_core::personality::get_current_personality(conn, user_id).await {
        system = format!("{system}\n\nAdopt this personality in your reply: {p}");
    }

    let request = nomi_llm::LlmRequest { system: Some(system), messages: vec![], tools: vec![], max_tokens: 256 };
    let response = nomi_llm::complete(provider, request).await.map_err(|e| e.to_string())?;
    Ok(response
        .content
        .into_iter()
        .find_map(|block| match block {
            nomi_llm::ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or(raw_result.to_string()))
}
```

`nomi_llm::LlmRequest`/`complete` accepting an empty `messages: vec![]` with everything in `system` needs confirming against `nomi_llm`'s actual `complete`/provider implementations during implementation — every existing call site (`classify_intent`, memory extraction) always supplies at least one message, so an empty-messages call is not yet a proven path and the plan must verify it works (or seed `messages` with a single placeholder user message like `"Report back."` if providers require at least one).

### 8. Wiring: `build_agent_registry()` and `Cargo.toml`

`nomi-server/src/lib.rs`:

```rust
pub fn build_agent_registry() -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_personality::PersonalityAgent),
        Box::new(nomi_agent_supervisor::SupervisorAgent),
    ])
}
```

`nomi-server/Cargo.toml` gains `nomi-agent-supervisor = { path = "../nomi-agent-supervisor" }`, alongside the three existing agent crate dependencies.

### 9. New read endpoint for the chat UI

`GET /api/sessions/:id/agent-activity` — `nomi-server/src/routes/sessions.rs`, mirroring `session_stream`'s own auth pattern (`authorize_session_access`):

```rust
#[derive(Serialize)]
pub struct AgentActivityItem {
    pub id: Uuid,
    pub target_agent_type: String,
    pub task: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn list_agent_activity(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Vec<AgentActivityItem>>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let rows = sqlx::query_as!(
        AgentActivityItem,
        "SELECT id, target_agent_type, task, status, result, error, created_at, completed_at \
         FROM agent_delegations WHERE session_id = $1 ORDER BY created_at DESC LIMIT 20",
        session_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| { tracing::error!(error = %e, "failed to list agent activity"); (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agent activity") })?;
    Ok(Json(rows))
}
```

Registered in `app.rs` alongside the existing `/api/sessions/:id/ws` route: `.route("/api/sessions/:id/agent-activity", get(sessions_routes::list_agent_activity))`.

### 10. Frontend

- `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`: `load()` gains one more `apiFetch` call to `/api/sessions/{sessionId}/agent-activity`, exposed as `data.agentActivity` (empty array on a non-ok response, matching the existing tolerant pattern used for `data.models`).
- `+page.svelte`:
  - WS message handler (`onMount`, the existing `if (envelope.kind === 'Delta') { ... } else if (envelope.kind === 'TurnCompleted') { ... }` chain) gains one more branch: `else if (envelope.kind === 'AgentDelegationUpdated') { invalidateAll(); }`.
  - A small pill next to the existing "chat settings" `Menu` trigger (same row, `<div class="mb-2 flex justify-end">`), visible only when `data.agentActivity` has at least one `'pending'`/`'processing'` row — e.g. "Money agent working…" (singular) or "N agents working…" (plural) — clicking it opens a `BottomSheet` (already built) listing every item in `data.agentActivity` (target agent, task, status, result/error, timestamps), using `ListItem`s for each row.
  - No changes to the existing model/personality `Menu` — this is a separate, additive UI element.

## Out of Scope

- Live token-by-token streaming of a delegated agent's own thinking (`mqtt: None` for delegated `run_agent_turn` calls) — only the final result is delivered. Revisit later if it turns out to matter for longer-running delegations.
- Chaining: a delegated agent itself calling `delegate_to_agent` further (only chitchat gets `can_delegate() = true`; the supervisor and specialists don't, in this pass).
- Guarding against duplicate/overlapping delegations to the same target for the same session (no dedup) — matches `turn_jobs`' own lack of such a guard.
- Any change to `classify_intent`'s existing direct-routing behavior for single-domain messages — untouched.
- Tagging individual `messages` rows with which agent authored them (no new `messages` column) — the supervisor's delivered text is a normal assistant message; its content signals the voice, not a DB column. `agent_delegations` is the observability surface for "who did what," not `messages`.
- `agent_events` audit-trail rows for delegated tool calls — the delegation worker calls `run_agent_turn` with the sentinel `agent_session_id` (matching the existing chitchat convention), so any `ToolCalled` events during delegated execution hit the same existing `agent_events.agent_session_id` foreign-key mismatch chitchat's own tool calls would (silently swallowed via the existing `let _ = ...` in `log_tool_call` — a pre-existing codebase behavior, not newly introduced). `agent_delegations` is the dedicated observability surface for this feature instead.
