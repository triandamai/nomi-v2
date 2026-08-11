# MQTT Pub/Sub Infra + Turn-Loop Decoupling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move turn processing out of the HTTP request onto a durable Postgres-backed job queue, processed by a separate worker binary that streams live token deltas onto an MQTT broker via `complete_stream()` + `collect_stream()` (sub-project 1), while `POST /messages` becomes ingest-only.

**Architecture:** A new `turn_jobs` table plus `FOR UPDATE SKIP LOCKED` claiming makes the queue durable and safe for multiple concurrent workers. A new `worker` binary (sharing the existing `nomi_orchestrator` lib crate with the HTTP server) does all turn processing; the HTTP server only ingests. Only `chitchat::run_chitchat_turn` changes its LLM call to tee `complete_stream()`'s events through an MQTT publish before feeding them into the unchanged `collect_stream()`. Everything else (routing, memory, subagent tool-calling, `handle_inbound_message`'s existing test-facing behavior) is preserved unchanged alongside the new queue-based path.

**Tech Stack:** Rust/axum backend (existing), `rumqttc` (async MQTT client), EMQX (new docker-compose service), `sqlx::postgres::PgListener` (Postgres `LISTEN`/`NOTIFY`).

## Global Constraints

- Every task must leave `cargo build` and `cargo test` green — no task may leave the crate in a non-compiling state.
- `handle_inbound_message` (`backend/src/turn/mod.rs`) keeps its exact current signature and behavior — every existing test in `tests/turn_handle_inbound_message.rs` and `tests/turn_subagent_state_machine.rs` must continue passing unmodified. It is a distinct, still-supported code path from the new ingest/queue/worker path, sharing internal routing/dispatch logic but not the queue.
- MQTT publish failures are always best-effort: logged, never propagated as a `TurnError`, never fail a turn. Postgres (`messages`, `turn_jobs`, `agent_events`) is always the durable source of truth.
- `POST /api/sessions/:id/messages` returns `202 Accepted` with `{ user_message }` only — no `assistant_message`. This is an intentional, accepted regression until the WebSocket bridge and frontend rendering sub-projects land (see the design doc's Purpose section).
- The worker re-reads LLM/embedding provider settings from the database before processing each claimed job — it does not hold a hot-swappable in-process cache the way `AppState` does.

---

## Task 1: `turn_jobs` table and the `turn::queue` module

**Files:**
- Create: `backend/migrations/0011_turn_jobs.sql`
- Create: `backend/src/turn/queue.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod queue;`)
- Test: `backend/tests/turn_queue.rs` (new)

**Interfaces:**
- Produces: `pub struct ClaimedJob { pub id: Uuid, pub session_id: Uuid, pub sender_channel_identity_id: Uuid, pub text: String, pub org_id_hint: Option<Uuid> }`.
- Produces: `pub async fn enqueue(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, session_id: Uuid, sender_channel_identity_id: Uuid, text: &str, org_id_hint: Option<Uuid>) -> Result<Uuid, TurnError>`.
- Produces: `pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, TurnError>`.
- Produces: `pub async fn mark_completed(pool: &PgPool, job_id: Uuid) -> Result<(), TurnError>`.
- Produces: `pub async fn mark_failed(pool: &PgPool, job_id: Uuid, error: &str) -> Result<(), TurnError>`.

- [ ] **Step 1: Write the migration**

Create `backend/migrations/0011_turn_jobs.sql`:

```sql
CREATE TABLE turn_jobs (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID NOT NULL,
    text                        TEXT NOT NULL,
    org_id_hint                 UUID,
    status                      TEXT NOT NULL DEFAULT 'pending',
    claimed_at                  TIMESTAMPTZ,
    completed_at                TIMESTAMPTZ,
    error                       TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX turn_jobs_pending_idx ON turn_jobs (created_at) WHERE status = 'pending';
```

- [ ] **Step 2: Write the failing test**

Create `backend/tests/turn_queue.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::queue;

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('t', true) RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, 'telegram', 'dm', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn enqueue_then_claim_returns_the_job(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();

    let claimed = queue::claim_next(&pool).await.unwrap().expect("expected a pending job");
    assert_eq!(claimed.id, job_id);
    assert_eq!(claimed.session_id, session_id);
    assert_eq!(claimed.sender_channel_identity_id, sender_id);
    assert_eq!(claimed.text, "hello");

    let status: String = sqlx::query_scalar("SELECT status FROM turn_jobs WHERE id = $1")
        .bind(job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "processing");
}

#[sqlx::test]
async fn claim_next_returns_none_when_no_pending_jobs(pool: PgPool) {
    let claimed = queue::claim_next(&pool).await.unwrap();
    assert!(claimed.is_none());
}

#[sqlx::test]
async fn two_concurrent_claims_never_return_the_same_job(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();

    let (a, b) = tokio::join!(queue::claim_next(&pool), queue::claim_next(&pool));
    let (a, b) = (a.unwrap(), b.unwrap());

    // Exactly one of the two concurrent claims won the single pending job; the other found nothing.
    let winners: Vec<_> = [a, b].into_iter().flatten().collect();
    assert_eq!(winners.len(), 1);
    assert_eq!(winners[0].id, job_id);
}

#[sqlx::test]
async fn mark_completed_and_mark_failed_update_status(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();
    queue::claim_next(&pool).await.unwrap();

    queue::mark_completed(&pool, job_id).await.unwrap();
    let (status, completed_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, completed_at FROM turn_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "completed");
    assert!(completed_at.is_some());

    let mut tx = pool.begin().await.unwrap();
    let job_id_2 = queue::enqueue(&mut tx, session_id, sender_id, "hello again", None).await.unwrap();
    tx.commit().await.unwrap();
    queue::claim_next(&pool).await.unwrap();

    queue::mark_failed(&pool, job_id_2, "boom").await.unwrap();
    let (status, error): (String, Option<String>) =
        sqlx::query_as("SELECT status, error FROM turn_jobs WHERE id = $1")
            .bind(job_id_2)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "failed");
    assert_eq!(error.as_deref(), Some("boom"));
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test --test turn_queue`
Expected: FAIL to compile — `turn_jobs` table doesn't exist yet (migration not applied to a fresh `sqlx::test` database until Step 1's file is picked up) and `nomi_orchestrator::turn::queue` doesn't exist.

- [ ] **Step 4: Implement**

Create `backend/src/turn/queue.rs`:

```rust
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::types::TurnError;

pub struct ClaimedJob {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub text: String,
    pub org_id_hint: Option<Uuid>,
}

pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<Uuid, TurnError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text, org_id_hint) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .bind(org_id_hint)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('turn_jobs_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, TurnError> {
    let row: Option<(Uuid, Uuid, Uuid, String, Option<Uuid>)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM turn_jobs \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE turn_jobs SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, sender_channel_identity_id, text, org_id_hint",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, session_id, sender_channel_identity_id, text, org_id_hint)| ClaimedJob {
        id,
        session_id,
        sender_channel_identity_id,
        text,
        org_id_hint,
    }))
}

pub async fn mark_completed(pool: &PgPool, job_id: Uuid) -> Result<(), TurnError> {
    sqlx::query("UPDATE turn_jobs SET status = 'completed', completed_at = now() WHERE id = $1")
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_failed(pool: &PgPool, job_id: Uuid, error: &str) -> Result<(), TurnError> {
    sqlx::query("UPDATE turn_jobs SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(job_id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}
```

Add to `backend/src/turn/mod.rs`'s module list (near the top, alphabetically with the others):

```rust
pub mod queue;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test --test turn_queue`
Expected: all 4 tests pass.

- [ ] **Step 6: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all existing tests still pass (purely additive).

- [ ] **Step 7: Commit**

```bash
git add backend/migrations/0011_turn_jobs.sql backend/src/turn/queue.rs backend/src/turn/mod.rs backend/tests/turn_queue.rs
git commit -m "feat: add turn_jobs table and turn::queue module"
```

---

## Task 2: `StreamEvent` serde support and `StreamEnvelope`

**Files:**
- Modify: `backend/src/llm/types.rs`
- Create: `backend/src/realtime/mod.rs`
- Modify: `backend/src/lib.rs` (add `pub mod realtime;`)
- Test: `backend/tests/realtime_envelope.rs` (new)

**Interfaces:**
- Modifies: `StreamEvent` (`backend/src/llm/types.rs`) now derives `Serialize, Deserialize, Clone` in addition to its existing `Debug, PartialEq`. `PartialBlock` already derives `Clone`; give it `Serialize, Deserialize` too.
- Produces: `pub enum StreamEnvelope { Delta { turn_job_id: Uuid, event: StreamEvent }, TurnCompleted { turn_job_id: Uuid, message_id: Uuid }, TurnFailed { turn_job_id: Uuid, error: String } }` (derives `Debug, Clone, PartialEq, Serialize, Deserialize`, `#[serde(tag = "kind")]`) in `backend/src/realtime/mod.rs`.

- [ ] **Step 1: Write the failing test**

Create `backend/tests/realtime_envelope.rs`:

```rust
use uuid::Uuid;

use nomi_orchestrator::llm::{PartialBlock, StopReason, StreamEvent};
use nomi_orchestrator::realtime::StreamEnvelope;

#[test]
fn a_delta_envelope_round_trips_through_json() {
    let turn_job_id = Uuid::new_v4();
    let envelope = StreamEnvelope::Delta {
        turn_job_id,
        event: StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text },
    };

    let json = serde_json::to_string(&envelope).unwrap();
    assert!(json.contains("\"kind\":\"Delta\""));

    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_turn_completed_envelope_round_trips_through_json() {
    let envelope = StreamEnvelope::TurnCompleted { turn_job_id: Uuid::new_v4(), message_id: Uuid::new_v4() };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_turn_failed_envelope_round_trips_through_json() {
    let envelope = StreamEnvelope::TurnFailed { turn_job_id: Uuid::new_v4(), error: "boom".to_string() };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_text_delta_event_carries_its_text_through_json() {
    let envelope = StreamEnvelope::Delta {
        turn_job_id: Uuid::new_v4(),
        event: StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 3, output_tokens: 2 },
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test realtime_envelope`
Expected: FAIL to compile — `nomi_orchestrator::realtime` doesn't exist, and `StreamEvent`/`PartialBlock` don't implement `Serialize`/`Deserialize` yet.

- [ ] **Step 3: Implement**

In `backend/src/llm/types.rs`, change:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PartialBlock {
```

to:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PartialBlock {
```

and change:

```rust
#[derive(Debug, PartialEq)]
pub enum StreamEvent {
```

to:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StreamEvent {
```

(`StopReason` already derives `Serialize, Deserialize` — confirm this while editing; if it doesn't, add them the same way, since `StreamEvent::Done` embeds a `StopReason`.)

Create `backend/src/realtime/mod.rs`:

```rust
pub mod mqtt;

pub use mqtt::{MqttError, MqttPublisher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::llm::StreamEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
}
```

Add to `backend/src/lib.rs`:

```rust
pub mod realtime;
```

This task does not yet create `backend/src/realtime/mqtt.rs` (Task 3 does) — to keep this task compiling on its own, temporarily stub it:

```rust
// backend/src/realtime/mqtt.rs (temporary stub, replaced in Task 3)
#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    #[error("mqtt not yet implemented")]
    NotImplemented,
}

pub struct MqttPublisher;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test realtime_envelope`
Expected: all 4 tests pass.

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass (the added derives are additive; no existing code depended on `StreamEvent` NOT implementing `Serialize`/`Deserialize`/`Clone`).

- [ ] **Step 6: Commit**

```bash
git add backend/src/llm/types.rs backend/src/realtime/mod.rs backend/src/realtime/mqtt.rs backend/src/lib.rs backend/tests/realtime_envelope.rs
git commit -m "feat: add Serialize/Deserialize to StreamEvent and a StreamEnvelope wire type"
```

---

## Task 3: EMQX broker and the `MqttPublisher` wrapper

**Files:**
- Modify: `backend/docker-compose.yml`
- Modify: `backend/Cargo.toml`
- Modify: `backend/src/realtime/mqtt.rs` (replace Task 2's stub)
- Test: `backend/tests/realtime_mqtt.rs` (new)

**Interfaces:**
- Produces: `pub struct MqttPublisher` (derives `Clone`), `pub fn MqttPublisher::connect(broker_host: &str, broker_port: u16, client_id: &str) -> Self`, `pub async fn MqttPublisher::publish<T: serde::Serialize>(&self, session_id: Uuid, envelope: &T) -> Result<(), MqttError>`.
- Produces: `pub enum MqttError` — replaces Task 2's placeholder variant with `Publish(#[from] rumqttc::ClientError)` and `Serialize(#[from] serde_json::Error)`.

- [ ] **Step 1: Add EMQX to docker-compose and the `rumqttc` dependency**

In `backend/docker-compose.yml`, add a new service alongside `postgres`:

```yaml
services:
  postgres:
    image: pgvector/pgvector:pg16
    environment:
      POSTGRES_USER: nomi
      POSTGRES_PASSWORD: nomi
      POSTGRES_DB: nomi
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  emqx:
    image: emqx/emqx:5
    ports:
      - "1883:1883"
      - "18083:18083"
    volumes:
      - emqxdata:/opt/emqx/data

volumes:
  pgdata:
  emqxdata:
```

(Port 1883 is the standard MQTT port the client connects to; 18083 is EMQX's admin dashboard, useful for manually inspecting topics during development.)

```bash
cd backend
cargo add rumqttc
```

- [ ] **Step 2: Write the failing test**

Create `backend/tests/realtime_mqtt.rs`:

```rust
use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use uuid::Uuid;

use nomi_orchestrator::realtime::MqttPublisher;

// Assumes the EMQX service from backend/docker-compose.yml is running on localhost:1883,
// matching how sqlx::test assumes a running local Postgres.
const BROKER_HOST: &str = "localhost";
const BROKER_PORT: u16 = 1883;

#[tokio::test]
async fn a_published_envelope_is_received_by_a_subscriber_on_the_session_topic() {
    let session_id = Uuid::new_v4();
    let topic = format!("chat/{session_id}/stream");

    let mut sub_options = MqttOptions::new(format!("test-sub-{session_id}"), BROKER_HOST, BROKER_PORT);
    sub_options.set_keep_alive(Duration::from_secs(5));
    let (subscriber, mut sub_eventloop) = AsyncClient::new(sub_options, 16);
    subscriber.subscribe(&topic, QoS::AtMostOnce).await.unwrap();

    // Drain the SubAck before publishing, so we don't race the subscribe confirmation.
    loop {
        match sub_eventloop.poll().await.unwrap() {
            Event::Incoming(Packet::SubAck(_)) => break,
            _ => continue,
        }
    }

    let publisher = MqttPublisher::connect(BROKER_HOST, BROKER_PORT, &format!("test-pub-{session_id}"));
    #[derive(serde::Serialize)]
    struct Payload {
        value: String,
    }
    publisher.publish(session_id, &Payload { value: "hello".to_string() }).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Event::Incoming(Packet::Publish(publish)) = sub_eventloop.poll().await.unwrap() {
                return publish.payload;
            }
        }
    })
    .await
    .expect("timed out waiting for the published message");

    let payload: serde_json::Value = serde_json::from_slice(&received).unwrap();
    assert_eq!(payload["value"], "hello");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && docker compose up -d emqx && cargo test --test realtime_mqtt`
Expected: FAIL — `MqttPublisher::connect`/`publish` don't have this signature yet (Task 2's stub is a unit struct with no methods).

- [ ] **Step 4: Implement**

Replace `backend/src/realtime/mqtt.rs`:

```rust
use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    #[error("mqtt publish failed: {0}")]
    Publish(#[from] rumqttc::ClientError),
    #[error("failed to serialize stream envelope: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    pub fn connect(broker_host: &str, broker_port: u16, client_id: &str) -> Self {
        let mut options = MqttOptions::new(client_id, broker_host, broker_port);
        options.set_keep_alive(Duration::from_secs(30));
        let (client, mut eventloop) = AsyncClient::new(options, 64);

        // rumqttc requires the EventLoop to be polled continuously to make progress (send
        // outgoing publishes, receive acks) — this task drives it for the process's lifetime.
        // A connection error just gets retried on the next poll; it never surfaces to publish()
        // callers directly (matching this project's fail-open policy for MQTT — see Task 5).
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    tracing::warn!(error = %e, "mqtt eventloop error, retrying");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });

        Self { client }
    }

    pub async fn publish<T: Serialize>(&self, session_id: Uuid, envelope: &T) -> Result<(), MqttError> {
        let payload = serde_json::to_vec(envelope)?;
        let topic = format!("chat/{session_id}/stream");
        self.client.publish(topic, QoS::AtMostOnce, false, payload).await?;
        Ok(())
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && docker compose up -d emqx && cargo test --test realtime_mqtt`
Expected: the test passes. If `rumqttc`'s exact API (constructor names, `Event`/`Packet` variants) differs slightly from what's written here, check `cargo doc --open -p rumqttc` (or docs.rs for the installed version) and adjust — the shape (options → `AsyncClient::new` → spawn a polling task → `client.publish(...)`) is stable across recent `rumqttc` versions even if exact signatures shift.

- [ ] **Step 6: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add backend/docker-compose.yml backend/Cargo.toml backend/Cargo.lock backend/src/realtime/mqtt.rs backend/tests/realtime_mqtt.rs
git commit -m "feat: add EMQX broker and MqttPublisher wrapper"
```

---

## Task 4: Shared provider bootstrap module

**Files:**
- Create: `backend/src/bootstrap/mod.rs`
- Create: `backend/src/bootstrap/providers.rs`
- Modify: `backend/src/lib.rs` (add `pub mod bootstrap;`)
- Modify: `backend/src/main.rs`

**Interfaces:**
- Produces: `pub async fn build_llm_provider_from_settings_or_env(pool: &PgPool, settings_key: &[u8; 32], http_client: reqwest::Client) -> Arc<dyn LlmProvider>`.
- Produces: `pub async fn build_embedding_provider_from_settings_or_env(pool: &PgPool, settings_key: &[u8; 32], http_client: reqwest::Client) -> Arc<dyn EmbeddingProvider>`.

This task is a pure refactor: it moves two existing private functions from `backend/src/main.rs` into the shared lib crate, unchanged, so Task 7's `worker.rs` can call them too instead of duplicating ~70 lines. No behavior changes; no new tests — existing tests (especially `backend/tests/app_state_reload.rs` and anything exercising server startup indirectly) are the regression check.

- [ ] **Step 1: Create the shared module**

Create `backend/src/bootstrap/mod.rs`:

```rust
pub mod providers;

pub use providers::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
```

Create `backend/src/bootstrap/providers.rs` — this is `backend/src/main.rs`'s current `llm_provider_kind_from_str`, `embedding_provider_kind_from_str`, `build_llm_provider_from_settings_or_env`, and `build_embedding_provider_from_settings_or_env` functions (currently `backend/src/main.rs:10-97`), moved verbatim with `pub` added and imports adjusted for the lib crate:

```rust
use std::env::var;
use std::sync::Arc;

use sqlx::PgPool;

use crate::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use crate::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use crate::settings;

fn llm_provider_kind_from_str(s: &str) -> ProviderKind {
    match s {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM provider: {other} (expected anthropic, openai, gemini, or fake)"),
    }
}

fn embedding_provider_kind_from_str(s: &str) -> EmbeddingProviderKind {
    match s {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown embedding provider: {other} (expected openai or fake)"),
    }
}

pub async fn build_llm_provider_from_settings_or_env(
    pool: &PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    let row = settings::get_settings(pool, "llm").await.expect("failed to query provider_settings");

    let model_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored llm api key");
            ModelConfig {
                provider: llm_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider =
                llm_provider_kind_from_str(&var("LLM_PROVIDER").expect("LLM_PROVIDER must be set"));
            let (model_id, api_key) = match provider {
                ProviderKind::Fake => (String::new(), String::new()),
                _ => (
                    var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
                    var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
                ),
            };
            ModelConfig { provider, model_id, api_key, base_url: var("LLM_BASE_URL").ok() }
        }
    };

    Arc::from(build_provider(model_config, http_client))
}

pub async fn build_embedding_provider_from_settings_or_env(
    pool: &PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn EmbeddingProvider> {
    let row = settings::get_settings(pool, "embedding").await.expect("failed to query provider_settings");

    let embedding_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored embedding api key");
            EmbeddingConfig {
                provider: embedding_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider = embedding_provider_kind_from_str(
                &std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            );
            let (model_id, api_key) = match provider {
                EmbeddingProviderKind::Fake => (String::new(), String::new()),
                EmbeddingProviderKind::OpenAi => (
                    std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
                    std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
                ),
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    Arc::from(build_embedding_provider(embedding_config, http_client))
}
```

Add to `backend/src/lib.rs`:

```rust
pub mod bootstrap;
```

- [ ] **Step 2: Update `main.rs` to use the shared module**

In `backend/src/main.rs`, delete the four functions that were just moved (`llm_provider_kind_from_str`, `embedding_provider_kind_from_str`, `build_llm_provider_from_settings_or_env`, `build_embedding_provider_from_settings_or_env` — lines 10-97 of the current file) and their now-unused imports (`nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind}`, `nomi_orchestrator::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind}`).

Add this import instead:

```rust
use nomi_orchestrator::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
```

The two call sites inside `main()` (currently `build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await` and the embedding equivalent) are unchanged — they now resolve to the imported functions instead of local ones.

- [ ] **Step 3: Verify it builds and existing tests pass**

Run: `cd backend && cargo build && cargo test`
Expected: builds clean, all existing tests pass unchanged (this task changes no runtime behavior — same functions, same call sites, different module).

- [ ] **Step 4: Commit**

```bash
git add backend/src/bootstrap/mod.rs backend/src/bootstrap/providers.rs backend/src/lib.rs backend/src/main.rs
git commit -m "refactor: extract provider bootstrap into a shared bootstrap module"
```

---

## Task 5: `turn::ingest_inbound_message`

**Files:**
- Create: `backend/src/turn/ingest.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod ingest;`)
- Test: `backend/tests/turn_ingest.rs` (new)

**Interfaces:**
- Consumes: `bootstrap::bootstrap_identity_and_session` (existing, unchanged), `queue::enqueue` (Task 1).
- Produces: `pub struct IngestResult { pub session_id: Uuid, pub sender_channel_identity_id: Uuid, pub user_message_id: Uuid, pub turn_job_id: Uuid }` and `pub async fn ingest_inbound_message(pool: &PgPool, channel: &str, chat_type: &str, chat_id: &str, sender_channel_user_id: &str, text: &str, org_id_hint: Option<Uuid>) -> Result<IngestResult, TurnError>`.

- [ ] **Step 1: Write the failing test**

Create `backend/tests/turn_ingest.rs`:

```rust
use sqlx::PgPool;

use nomi_orchestrator::turn::ingest::ingest_inbound_message;

#[sqlx::test]
async fn ingest_bootstraps_persists_the_message_and_enqueues_a_job(pool: PgPool) {
    let result = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await
        .unwrap();

    let (content, sender_id): (String, Option<uuid::Uuid>) =
        sqlx::query_as("SELECT content, sender_channel_identity_id FROM messages WHERE id = $1")
            .bind(result.user_message_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(content, "hello");
    assert_eq!(sender_id, Some(result.sender_channel_identity_id));

    let (job_session_id, job_text, status): (uuid::Uuid, String, String) =
        sqlx::query_as("SELECT session_id, text, status FROM turn_jobs WHERE id = $1")
            .bind(result.turn_job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(job_session_id, result.session_id);
    assert_eq!(job_text, "hello");
    assert_eq!(status, "pending");
}

#[sqlx::test]
async fn ingest_reuses_the_same_session_and_identity_across_two_calls(pool: PgPool) {
    let first = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "first", None).await.unwrap();
    let second = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "second", None).await.unwrap();

    assert_eq!(first.session_id, second.session_id);
    assert_eq!(first.sender_channel_identity_id, second.sender_channel_identity_id);
    assert_ne!(first.turn_job_id, second.turn_job_id);

    let job_count: i64 = sqlx::query_scalar("SELECT count(*) FROM turn_jobs WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 2);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test turn_ingest`
Expected: FAIL to compile — `nomi_orchestrator::turn::ingest` doesn't exist yet.

- [ ] **Step 3: Implement**

Create `backend/src/turn/ingest.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use super::bootstrap;
use super::queue;
use super::types::TurnError;

#[derive(Debug, Clone, PartialEq)]
pub struct IngestResult {
    pub session_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub user_message_id: Uuid,
    pub turn_job_id: Uuid,
}

pub async fn ingest_inbound_message(
    pool: &PgPool,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<IngestResult, TurnError> {
    let bootstrap::BootstrapResult { sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut tx = pool.begin().await?;

    let user_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .fetch_one(&mut *tx)
    .await?;

    let turn_job_id = queue::enqueue(&mut tx, session_id, sender_channel_identity_id, text, org_id_hint).await?;

    tx.commit().await?;

    Ok(IngestResult { session_id, sender_channel_identity_id, user_message_id, turn_job_id })
}
```

Add to `backend/src/turn/mod.rs`'s module list:

```rust
pub mod ingest;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test turn_ingest`
Expected: both tests pass.

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass (purely additive — `handle_inbound_message` is untouched).

- [ ] **Step 6: Commit**

```bash
git add backend/src/turn/ingest.rs backend/src/turn/mod.rs backend/tests/turn_ingest.rs
git commit -m "feat: add turn::ingest_inbound_message for the HTTP ingest phase"
```

---

## Task 6: `run_locked_turn` extraction, `turn::process_turn`, and chitchat streaming

**Files:**
- Modify: `backend/src/turn/mod.rs`
- Modify: `backend/src/turn/chitchat.rs`
- Modify: `backend/tests/turn_chitchat.rs` (add one parameter to 6 existing call sites)
- Test: `backend/tests/turn_process.rs` (new)

**Interfaces:**
- Consumes: `queue::ClaimedJob` (Task 1), `realtime::{MqttPublisher, StreamEnvelope}` (Tasks 2-3), `llm::{collect_stream, LlmProvider}` (sub-project 1).
- Produces: `pub async fn process_turn(pool: &PgPool, mqtt: &MqttPublisher, provider: &dyn LlmProvider, embedding_provider: &dyn EmbeddingProvider, turn_job_id: Uuid, session_id: Uuid, sender_channel_identity_id: Uuid, user_id: Uuid, text: &str) -> Result<TurnOutcome, TurnError>` in `backend/src/turn/mod.rs`.
- Modifies: `chitchat::run_chitchat_turn`'s signature gains a `mqtt: Option<(&MqttPublisher, Uuid)>` parameter (second positional parameter, right after `conn`) — `None` from `handle_inbound_message`'s path, `Some((mqtt, turn_job_id))` from `process_turn`'s path.
- Modifies: `handle_inbound_message`'s body is refactored to call the new shared `run_locked_turn` helper instead of inlining the routing/dispatch match — its signature, behavior, and every existing test pass unchanged (Global Constraints).

- [ ] **Step 1: Write the failing test**

Create `backend/tests/turn_process.rs`:

```rust
mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::realtime::MqttPublisher;
use nomi_orchestrator::turn::ingest::ingest_inbound_message;
use nomi_orchestrator::turn::process_turn;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test]
async fn process_turn_produces_a_reply_for_an_already_ingested_message(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::success(canned_response("hi there"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-process-{}", Uuid::new_v4()));

    let user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM channel_identities WHERE id = $1",
    )
    .bind(ingested.sender_channel_identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let outcome = process_turn(
        &pool,
        &mqtt,
        &provider,
        &embedder,
        ingested.turn_job_id,
        ingested.session_id,
        ingested.sender_channel_identity_id,
        user_id,
        "hello",
    )
    .await
    .unwrap();

    assert_eq!(outcome.reply, "hi there");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(ingested.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // ingest's inbound insert + process_turn's reply insert
}

#[sqlx::test]
async fn process_turn_records_a_turn_failed_event_on_llm_failure(pool: PgPool) {
    let ingested = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();

    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let mqtt = MqttPublisher::connect("localhost", 1883, &format!("test-process-{}", Uuid::new_v4()));

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = process_turn(
        &pool,
        &mqtt,
        &provider,
        &embedder,
        ingested.turn_job_id,
        ingested.session_id,
        ingested.sender_channel_identity_id,
        user_id,
        "hello",
    )
    .await;

    assert!(result.is_err());

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events WHERE session_id = $1",
    )
    .bind(ingested.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}
```

(This test connects to the EMQX instance from Task 3's docker-compose — same assumption as `tests/realtime_mqtt.rs`.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && docker compose up -d emqx && cargo test --test turn_process`
Expected: FAIL to compile — `nomi_orchestrator::turn::process_turn` doesn't exist yet.

- [ ] **Step 3: Implement**

In `backend/src/turn/chitchat.rs`, add imports and change the function signature and its LLM call. Add to the existing `use` lines:

```rust
use crate::realtime::{MqttPublisher, StreamEnvelope};
```

Change:

```rust
pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
```

to:

```rust
pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
```

Change the LLM call (currently `let response = crate::llm::complete(provider, request).await.map_err(TurnError::LlmCallFailed)?;`, with the comment above it about not holding a transaction open) to:

```rust
    // The LLM call happens outside any DB transaction: holding a transaction open across a
    // slow network round trip would needlessly extend how long this connection's locks are held.
    let stream = provider.complete_stream(request).await.map_err(TurnError::LlmCallFailed)?;
    let response = match mqtt {
        Some((publisher, turn_job_id)) => {
            use futures_util::StreamExt;
            let published = stream.then(move |event_result| async move {
                if let Ok(event) = &event_result {
                    let envelope = StreamEnvelope::Delta { turn_job_id, event: event.clone() };
                    // Best-effort: an MQTT publish failure never fails the turn (Global Constraints).
                    let _ = publisher.publish(session_id, &envelope).await;
                }
                event_result
            });
            crate::llm::collect_stream(Box::pin(published)).await.map_err(TurnError::LlmCallFailed)?
        }
        None => crate::llm::collect_stream(stream).await.map_err(TurnError::LlmCallFailed)?,
    };
```

Everything after this (extracting `reply_text`, persisting it, the `ChitchatReply` event, memory extraction) is unchanged.

In `backend/src/turn/mod.rs`, replace the `handle_inbound_message` function's routing/dispatch match (the `let result = match routing_outcome { ... };` block, currently lines 72-94) and the two `chitchat::run_chitchat_turn(...)` call sites inside it with calls to a new shared private helper. The full new `backend/src/turn/mod.rs` becomes:

```rust
pub mod bootstrap;
pub mod chitchat;
pub mod ingest;
pub mod lock;
pub mod memory;
pub mod money_agent;
pub mod queue;
pub mod routing;
pub mod subagent;
pub mod tools;
pub mod types;

pub use types::{TurnError, TurnOutcome};

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use crate::realtime::{MqttPublisher, StreamEnvelope};

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;

enum RoutingOutcome {
    Continue(Uuid),
    NeedsClassification,
    FallbackToChitchat,
}

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    let result = run_locked_turn(&mut conn, None, provider, embedding_provider, session_id, sender_channel_identity_id, user_id, text).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// The worker's entry point (see backend/src/bin/worker.rs): processes an already-ingested
/// message (see turn::ingest::ingest_inbound_message) — bootstrap and the inbound message
/// insert have already happened, so this only acquires the session lock and runs routing/
/// dispatch, threading `mqtt`/`turn_job_id` through to chitchat for live delta publishing.
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = run_locked_turn(
        &mut conn,
        Some((mqtt, turn_job_id)),
        provider,
        embedding_provider,
        session_id,
        sender_channel_identity_id,
        user_id,
        text,
    )
    .await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            // Best-effort: an MQTT publish failure never changes the turn's outcome.
            let _ = mqtt
                .publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id, error: err.to_string() })
                .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// Shared by handle_inbound_message and process_turn: routing/classification and
/// chitchat/subagent dispatch, assuming the session lock is already held by the caller and
/// the inbound message has already been persisted (by the caller, before this runs).
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else if details.agent_type == money_agent::MONEY_AGENT_TYPE {
                RoutingOutcome::Continue(agent_session_id)
            } else {
                RoutingOutcome::FallbackToChitchat
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue(agent_session_id) => {
            run_subagent_turn(conn, provider, &money_agent::MoneyAgent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::FallbackToChitchat => {
            chitchat::run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id, text).await
        }
        RoutingOutcome::NeedsClassification => match routing::classify_intent(provider, text).await {
            routing::Intent::Money => {
                let agent_session_id = routing::spawn_agent_session(
                    conn,
                    session_id,
                    sender_channel_identity_id,
                    money_agent::MONEY_AGENT_TYPE,
                )
                .await?;
                run_subagent_turn(conn, provider, &money_agent::MoneyAgent, session_id, agent_session_id, user_id).await
            }
            routing::Intent::Chitchat => {
                chitchat::run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id, text).await
            }
        },
    }
}

async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    agent: &dyn subagent::SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<String, TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = tools::run_tool_calling_loop(
        conn,
        provider,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        SUBAGENT_MAX_TOKENS,
    )
    .await?;

    match outcome {
        tools::LoopOutcome::Reply(reply_text) => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&reply_text)
                .execute(&mut *tx)
                .await?;
            sqlx::query("UPDATE agent_sessions SET last_activity_at = now() WHERE id = $1")
                .bind(agent_session_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(reply_text)
        }
        tools::LoopOutcome::Completed { status, summary } => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&summary)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type(), &status, &summary).await?;

            Ok(summary)
        }
    }
}

async fn fetch_recent_messages(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<Vec<LlmMessage>, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(SUBAGENT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect())
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
```

(Note: `run_locked_turn` takes `conn: &mut PoolConnection<Postgres>` directly rather than the `&mut conn` reborrow pattern the original inline code used implicitly — `find_active_agent_session` and friends already accept `&mut PoolConnection<Postgres>` per their existing signatures in `routing.rs`, unchanged.)

In `backend/tests/turn_chitchat.rs`, add `None,` as the second argument to all 6 existing `run_chitchat_turn(&mut conn, ...)` calls, e.g. change:

```rust
let reply = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi")
```

to:

```rust
let reply = run_chitchat_turn(&mut conn, None, &provider, &embedder, session_id, user_id, "hi")
```

Apply the same one-argument insertion to the other 5 call sites in that file (lines 88, 135, 183, 208, 236 per the current file).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && docker compose up -d emqx && cargo test --test turn_process --test turn_chitchat --test turn_handle_inbound_message --test turn_subagent_state_machine`
Expected: all pass — the two new `turn_process.rs` tests, all 6 `turn_chitchat.rs` tests (now compiling with the extra `None`/`mqtt` argument), and every existing `handle_inbound_message`-based test (Global Constraints: unchanged behavior).

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && docker compose up -d emqx && cargo test`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/turn/mod.rs backend/src/turn/chitchat.rs backend/tests/turn_chitchat.rs backend/tests/turn_process.rs
git commit -m "feat: add turn::process_turn, extract run_locked_turn, stream chitchat via MQTT"
```

---

## Task 7: Worker binary

**Files:**
- Create: `backend/src/bin/worker.rs`
- Test: covered by Task 6's `turn_process.rs` (the worker's core logic, `process_turn`) plus a smoke-level integration check in this task's Step 3.

**Interfaces:**
- Consumes: `bootstrap::{build_llm_provider_from_settings_or_env, build_embedding_provider_from_settings_or_env}` (Task 4), `turn::queue::{claim_next, mark_completed, mark_failed}` (Task 1), `turn::process_turn` (Task 6), `realtime::{MqttPublisher, StreamEnvelope}` (Tasks 2-3).

A `#[sqlx::test]`-based automated test cannot easily exercise a `loop { }` binary's main function directly (it runs forever); this task's correctness is verified by Task 6's `process_turn` tests (the logic the worker calls) plus a manual smoke test in Step 3. This matches how `backend/src/main.rs`'s `main()` itself has no direct test today either — it's an assembly of already-tested pieces.

- [ ] **Step 1: Implement**

Create `backend/src/bin/worker.rs`:

```rust
use std::env::var;
use std::time::Duration;

use nomi_orchestrator::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
use nomi_orchestrator::realtime::{MqttPublisher, StreamEnvelope};
use nomi_orchestrator::settings;
use nomi_orchestrator::turn::queue;

const NOTIFY_CHANNEL: &str = "turn_jobs_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nomi_orchestrator=debug,info".into()),
        )
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");
    let mqtt_broker_host = var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port: u16 = var("MQTT_BROKER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    tracing::info!("connected to database");
    sqlx::migrate!("./migrations").run(&pool).await.expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let http_client = reqwest::Client::new();
    let mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, "nomi-worker");

    let mut listener = sqlx::postgres::PgListener::connect(&database_url).await.expect("failed to connect listener");
    listener.listen(NOTIFY_CHANNEL).await.expect("failed to LISTEN on turn_jobs_channel");
    tracing::info!("listening for new turn jobs");

    loop {
        // Wake on NOTIFY, or on the fallback interval if a NOTIFY is ever missed — either way,
        // fall through to draining every currently-pending job before waiting again.
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

        loop {
            let claimed = match queue::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "failed to claim next turn job");
                    break;
                }
            };

            let provider = build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let user_id: Result<uuid::Uuid, sqlx::Error> =
                sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
                    .bind(claimed.sender_channel_identity_id)
                    .fetch_one(&pool)
                    .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, job_id = %claimed.id, "failed to resolve user_id for claimed job");
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };

            let result = nomi_orchestrator::turn::process_turn(
                &pool,
                &mqtt,
                provider.as_ref(),
                embedding_provider.as_ref(),
                claimed.id,
                claimed.session_id,
                claimed.sender_channel_identity_id,
                user_id,
                &claimed.text,
            )
            .await;

            match result {
                Ok(outcome) => {
                    let _ = queue::mark_completed(&pool, claimed.id).await;
                    // process_turn's TurnOutcome doesn't carry the persisted reply message's id
                    // (only its text) — the terminal envelope uses the turn_job_id as the
                    // correlation key; a future WebSocket bridge looks up the actual message row
                    // by session_id once it sees this event, rather than needing message_id here.
                    let _ = mqtt
                        .publish(
                            claimed.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: uuid::Uuid::nil() },
                        )
                        .await;
                    tracing::info!(job_id = %claimed.id, reply_len = outcome.reply.len(), "turn job completed");
                }
                Err(e) => {
                    // process_turn already published TurnFailed and recorded the agent_events row
                    // internally (see turn::process_turn) — this only updates the job's own status.
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    tracing::warn!(job_id = %claimed.id, error = %e, "turn job failed");
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify it builds**

Run: `cd backend && cargo build --bin worker`
Expected: builds clean.

- [ ] **Step 3: Manual smoke test**

```bash
cd backend
docker compose up -d postgres emqx
export DATABASE_URL=postgres://nomi:nomi@localhost:5432/nomi
sqlx migrate run --database-url "$DATABASE_URL"
export SETTINGS_ENCRYPTION_KEY=$(openssl rand -hex 32)
export LLM_PROVIDER=fake
export EMBEDDING_PROVIDER=fake
cargo run --bin worker &
WORKER_PID=$!
sleep 2

# Seed an org/session/channel-identity and enqueue one job directly via psql, simulating what
# ingest_inbound_message would do from the HTTP side — confirms the running worker (already
# LISTENing) picks it up without needing a restart.
psql "$DATABASE_URL" <<'SQL'
WITH org AS (
    INSERT INTO organizations (name, is_personal) VALUES ('smoke', true) RETURNING id
), usr AS (
    INSERT INTO users DEFAULT VALUES RETURNING id
), identity AS (
    INSERT INTO channel_identities (user_id, channel, channel_user_id)
    SELECT id, 'smoke', 'smoke-user' FROM usr RETURNING id, user_id
), sess AS (
    INSERT INTO sessions (org_id, channel, chat_type, chat_id)
    SELECT org.id, 'smoke', 'dm', 'smoke-chat' FROM org RETURNING id
)
INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text)
SELECT sess.id, identity.id, 'hello from the smoke test' FROM sess, identity;

SELECT pg_notify('turn_jobs_channel', '');
SQL
```

Expected: worker logs show `listening for new turn jobs` on startup; within roughly a second of the `pg_notify` above (bounded by the LISTEN wakeup, not the 5s poll fallback), the worker logs `turn job completed`. Confirm via `psql "$DATABASE_URL" -c "SELECT status FROM turn_jobs ORDER BY created_at DESC LIMIT 1"` that the row shows `completed`. Kill the worker (`kill $WORKER_PID`) once confirmed. This step is manual verification, not an automated test — record the observed log lines and the final job status in the task report.

- [ ] **Step 4: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass (this task adds a new binary target; it doesn't change the lib crate's compiled test surface).

- [ ] **Step 5: Commit**

```bash
git add backend/src/bin/worker.rs
git commit -m "feat: add worker binary that claims and processes turn jobs"
```

---

## Task 8: HTTP route becomes ingest-only

**Files:**
- Modify: `backend/src/routes/sessions.rs`
- Modify: `backend/tests/sessions_routes.rs`

**Interfaces:**
- Modifies: `send_message`'s response type from `SendMessageResponse { user_message, assistant_message }` to `IngestMessageResponse { user_message }`, and its status code from `200`/implicit-`502`-on-failure to `202` on success.

- [ ] **Step 1: Update the failing tests**

In `backend/tests/sessions_routes.rs`, find the test(s) exercising `POST /api/sessions/:id/messages` for the success path (asserting on `assistant_message`) and the failure path (asserting `502`/`turnFailed`). Update them to match the new contract — read the existing test(s) in that file first to match their exact setup/helper style, then change their assertions to:

```rust
// success path
assert_eq!(response.status(), StatusCode::ACCEPTED);
let body: serde_json::Value = /* existing JSON-parsing helper from this file */;
assert!(body.get("user_message").is_some());
assert!(body.get("assistant_message").is_none());
```

The failure-path test (today asserting a `502` when the fake provider's `SIMULATE_FAILURE_SENTINEL` is used) no longer applies at the HTTP layer — ingest always succeeds once the message is validated and the session exists, regardless of whether the LLM call will later fail in the worker. Delete that specific assertion/test case, or repurpose it to assert that ingest still succeeds (`202`) even when the *text* contains the failure sentinel (the failure now surfaces only in the worker's processing, covered by Task 6's `turn_process.rs::process_turn_records_a_turn_failed_event_on_llm_failure`).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test sessions_routes`
Expected: FAIL — current `send_message` still returns `200`/`assistant_message` and a `502` on the failure-sentinel case.

- [ ] **Step 3: Implement**

Replace `send_message` in `backend/src/routes/sessions.rs`:

```rust
#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct IngestMessageResponse {
    pub user_message: MessageItem,
}

#[tracing::instrument(skip(state, claims, req), fields(session_id = %session_id, user_id = %claims.sub, text_len = req.text.len()))]
pub async fn send_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<IngestMessageResponse>), (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    if req.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text must not be empty"));
    }

    let (channel, chat_type, chat_id): (String, String, String) =
        sqlx::query_as("SELECT channel, chat_type, chat_id FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "session lookup failed");
                (StatusCode::NOT_FOUND, "session not found")
            })?;

    tracing::debug!(channel = %channel, chat_type = %chat_type, "ingesting inbound message");
    let ingested = crate::turn::ingest::ingest_inbound_message(
        &state.pool,
        &channel,
        &chat_type,
        &chat_id,
        &claims.sub.to_string(),
        &req.text,
        Some(claims.active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "ingest failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to ingest message")
    })?;
    tracing::info!(turn_job_id = %ingested.turn_job_id, "message ingested and queued");

    let (created_at,): (DateTime<Utc>,) = sqlx::query_as("SELECT created_at FROM messages WHERE id = $1")
        .bind(ingested.user_message_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to fetch persisted user message");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch persisted user message")
        })?;

    let user_message = MessageItem {
        id: ingested.user_message_id,
        sender: "user".to_string(),
        content: req.text,
        created_at,
    };

    Ok((StatusCode::ACCEPTED, Json(IngestMessageResponse { user_message })))
}
```

Delete the now-unused `SendMessageResponse` struct (replaced by `IngestMessageResponse`). `crate::turn::handle_inbound_message` is no longer called from this file — leave `use crate::turn::bootstrap::bootstrap_identity_and_session;` as-is (still used by `create_session`), and note `send_message` no longer needs `state.provider`/`state.embedding_provider` at all.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test sessions_routes`
Expected: pass.

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && docker compose up -d emqx && cargo test`
Expected: all tests pass. (`frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` will now receive a response without `assistant_message` — per Global Constraints this is an accepted, documented regression; no frontend changes are in scope for this plan.)

- [ ] **Step 6: Commit**

```bash
git add backend/src/routes/sessions.rs backend/tests/sessions_routes.rs
git commit -m "feat: make POST /messages ingest-only, returning 202 with just user_message"
```

---

## Task 9: Remove the dead in-memory provider hot-swap

**Files:**
- Modify: `backend/src/app.rs`
- Modify: `backend/src/routes/settings.rs`
- Modify: `backend/src/main.rs`
- Modify: `backend/tests/app_state_reload.rs`
- Modify: `backend/tests/settings_routes.rs` (if it asserts on the in-memory swap specifically)

**Interfaces:**
- Modifies: `AppState` loses its `provider: Arc<RwLock<Arc<dyn LlmProvider>>>` and `embedding_provider: Arc<RwLock<Arc<dyn EmbeddingProvider>>>` fields — nothing in the HTTP server reads them anymore after Task 8 (the worker re-reads settings directly per Task 7).

- [ ] **Step 1: Check what the affected tests actually assert**

Read `backend/tests/app_state_reload.rs` and `backend/tests/settings_routes.rs` in full before editing. `app_state_reload.rs`'s existing test (`swapping_the_provider_lock_is_visible_to_the_next_reader`) constructs its own standalone `Arc<RwLock<Arc<dyn LlmProvider>>>` to test the RwLock-swap *mechanism* in isolation — it does not go through `AppState` directly (confirmed: it builds `let lock: Arc<RwLock<Arc<dyn LlmProvider>>> = Arc::new(RwLock::new(provider));` itself). If that's the full extent of the file, delete it entirely — it exists solely to test a mechanism this task removes. If `settings_routes.rs` has assertions specifically checking that `state.provider` changed after a `PUT`, remove only those specific assertions (the rest of each settings test — verifying the row was persisted to `provider_settings` — is unaffected and must stay).

- [ ] **Step 2: Remove the fields from `AppState`**

In `backend/src/app.rs`, change:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub provider: Arc<RwLock<Arc<dyn LlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Arc<dyn EmbeddingProvider>>>,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
}
```

to:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
}
```

Remove the now-unused imports in that file (`std::sync::Arc`, `tokio::sync::RwLock`, `crate::embedding::EmbeddingProvider`, `crate::llm::LlmProvider`) if nothing else in `app.rs` uses them — check with `cargo build` after this edit; keep any that are still needed elsewhere in the file.

- [ ] **Step 3: Update `main.rs`'s `AppState` construction**

In `backend/src/main.rs`, remove the `provider`/`embedding_provider` construction lines and their fields from the `AppState { ... }` literal:

```rust
let state = nomi_orchestrator::app::AppState {
    pool,
    jwt_secret,
    http_client,
    settings_key,
};
```

Also remove the now-unused `provider`/`embedding_provider` local `let` bindings and their calls to `build_llm_provider_from_settings_or_env`/`build_embedding_provider_from_settings_or_env` — the HTTP server binary no longer needs these at all (only the worker binary does, per Task 7). Remove the now-unused `use nomi_orchestrator::bootstrap::{...}` import this leaves behind in `main.rs`.

- [ ] **Step 4: Update the settings routes**

In `backend/src/routes/settings.rs`, remove the `*state.provider.write().await = new_provider;` line (and its `tracing::info!("live llm provider swapped")` line) from `put_llm_settings`, and the equivalent `*state.embedding_provider.write().await = new_provider;` line from `put_embedding_settings`. The rest of each handler (validation, `settings::upsert_settings`, the response) is unchanged. Remove the now-unused `build_provider`/`ProviderKind`/`Arc<dyn LlmProvider>` construction of `new_provider` in each handler if nothing else in the function uses it, and any now-unused imports this leaves behind.

- [ ] **Step 5: Apply Step 1's test changes**

Delete `backend/tests/app_state_reload.rs` (per Step 1, if confirmed to test only the now-removed mechanism) or trim `backend/tests/settings_routes.rs`'s swap-specific assertions, per what Step 1 found.

- [ ] **Step 6: Run the full backend test suite**

Run: `cd backend && cargo build && cargo test`
Expected: builds clean, all remaining tests pass.

- [ ] **Step 7: Commit**

```bash
git add backend/src/app.rs backend/src/main.rs backend/src/routes/settings.rs backend/tests/app_state_reload.rs backend/tests/settings_routes.rs
git commit -m "refactor: remove dead in-memory provider hot-swap now that the worker owns turn processing"
```
