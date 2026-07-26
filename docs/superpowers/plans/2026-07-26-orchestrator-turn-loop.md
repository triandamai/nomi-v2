# Orchestrator Turn Loop & Chitchat Agent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `handle_inbound_message`, the turn-loop skeleton described in `docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md`: identity/org/session bootstrap, a session-scoped advisory lock on a pinned connection, the two-transaction durability split, a (currently always-empty) sub-agent routing check, and a chitchat reply using the `LlmProvider` trait from the just-merged LLM provider abstraction.

**Architecture:** A new `backend/src/turn/` module, built bottom-up across 5 tasks: shared types + identity/org/session bootstrap, the advisory-lock + inbound-message-durability helpers, the sub-agent routing-check stub, the chitchat turn (history fetch + LLM call + reply persist), and finally the `handle_inbound_message` entry point that wires all four together with the failure-handling semantics from the design doc. No schema changes — every table this plan touches (`users`, `channel_identities`, `organizations`, `memberships`, `sessions`, `messages`, `agent_sessions`, `agent_events`) already exists.

**Tech Stack:** Rust, `sqlx` (Postgres, already in the crate), `tokio`, `uuid`, `serde_json`, `thiserror`, the crate's own `llm` module (`LlmProvider`, `LlmRequest`, `LlmMessage`, `ContentBlock`, `LlmRole`, `LlmResponse`, `LlmError`) from the merged provider-abstraction plan. No new Cargo.toml dependencies.

## Global Constraints

- No schema changes. Every table used here already exists (migrations 0002–0006).
- Personal-org auto-bootstrap uses exactly: `organizations.is_personal = true`, `organizations.name = 'Personal'` (literal — no user profile data exists yet to derive a better name from), `memberships.role = 'owner'`. This matches `docs/superpowers/specs/2026-07-25-multi-tenant-rbac-design.md` §3: "an organizations row is auto-created with is_personal = true, and a memberships row makes that user its sole owner."
- The chitchat system prompt is exactly: `"You are a helpful, friendly assistant chatting with the user. Keep replies concise."` (verbatim from the design doc §3).
- History: the last 20 messages for a session, oldest-first, each mapped to one `LlmMessage` with a single `ContentBlock::Text` block. `sender_channel_identity_id IS NOT NULL` → `LlmRole::User`; `NULL` → `LlmRole::Assistant`.
- `max_tokens` for chitchat completion requests is `1024`. The design doc explicitly defers "prompt/max_tokens tuning" as future work; this is this plan's own working default, not a spec-mandated value.
- The advisory lock uses exactly `SELECT pg_advisory_lock(hashtext($1))` / `SELECT pg_advisory_unlock(hashtext($1))`, binding `session_id.to_string()` as the text parameter (per `docs/superpowers/specs/2026-07-22-sub-agent-lifecycle-design.md`). It is acquired on a single `PoolConnection<Postgres>` checked out once via `pool.acquire()` and held for the entire acquire → Txn A → routing-check → Txn B → release span — never re-acquired from the pool mid-turn. `use sqlx::Acquire;` is required in scope to call `.begin()` on a `PoolConnection<Postgres>`.
- Txn A (inbound message insert) always commits independently of Txn B (chitchat reply). A Txn B failure must never roll back or delete the Txn A message.
- The advisory lock is released via an **explicit call on both the success and failure paths** — not an RAII/scope guard — per the design doc's stated preference ("to keep the release point visible and testable").
- On a Txn B failure: a best-effort `TurnFailed` row is inserted into `agent_events` **outside** Txn B (a separate, always-attempted `execute` call whose own failure is silently ignored — it must never replace or mask the original `TurnError` returned to the caller).
- Out of scope for this plan (per the design doc): live MQTT publishing, RAG/memory retrieval, real sub-agent spawning/execution, a real channel adapter (Telegram/WhatsApp webhook). The sub-agent routing check is implemented and tested as a query, but its result is always discarded and the turn always falls through to chitchat in this slice — nothing in this plan ever populates `agent_sessions`.
- **Known limitation, explicitly not handled by this plan:** two truly concurrent *first-contact* messages from the same brand-new `(channel, channel_user_id)` can race on `channel_identities`' `UNIQUE (channel, channel_user_id)` constraint during bootstrap (bootstrap runs before the session lock is acquired, so it isn't serialized by it). This surfaces as `TurnError::Db` from the losing call; there is no retry. This is acceptable for this slice (a real channel adapter delivers one sender's messages serially) but should be called out to whoever builds the real channel adapter.

## File Structure

- `backend/src/turn/types.rs` — `TurnOutcome`, `TurnError` (shared by every other file in this module).
- `backend/src/turn/bootstrap.rs` — `BootstrapResult`, `bootstrap_identity_and_session`.
- `backend/src/turn/lock.rs` — `acquire_session_lock`, `release_session_lock`, `insert_inbound_message` (Txn A).
- `backend/src/turn/routing.rs` — `find_active_agent_session` (the routing-check stub).
- `backend/src/turn/chitchat.rs` — `run_chitchat_turn` (history fetch + LLM call + Txn B).
- `backend/src/turn/mod.rs` — built incrementally; ends with `handle_inbound_message`, the public entry point.
- `backend/src/lib.rs` — add `pub mod turn;` (Task 1).
- `backend/tests/turn_bootstrap.rs`, `turn_lock.rs`, `turn_routing.rs`, `turn_chitchat.rs`, `turn_handle_inbound_message.rs` — one per task.
- `backend/tests/support/mod.rs` — `FakeLlmProvider`, a hand-written `LlmProvider` test double (fixed success/failure outcome, optional artificial delay, records every request it received). Created in Task 4, reused by Task 5.

---

### Task 1: Turn types + identity/org/session bootstrap

**Files:**
- Create: `backend/src/turn/types.rs`
- Create: `backend/src/turn/bootstrap.rs`
- Create: `backend/src/turn/mod.rs`
- Modify: `backend/src/lib.rs` (add `pub mod turn;`)
- Test: `backend/tests/turn_bootstrap.rs`

**Interfaces:**
- Produces: `TurnError` (`Db(#[from] sqlx::Error)`, `LlmCallFailed(#[from] crate::llm::LlmError)`), `TurnOutcome { session_id: Uuid, reply: String }`, `BootstrapResult { user_id: Uuid, org_id: Uuid, sender_channel_identity_id: Uuid, session_id: Uuid }`, `async fn bootstrap_identity_and_session(pool: &PgPool, channel: &str, chat_type: &str, chat_id: &str, sender_channel_user_id: &str) -> Result<BootstrapResult, TurnError>`. All later tasks import `TurnError` from `super::types`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_bootstrap.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::bootstrap::bootstrap_identity_and_session;

#[sqlx::test]
async fn new_sender_creates_user_org_membership_identity_and_session(pool: PgPool) {
    let result = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
        .bind(result.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_count, 1);

    let (is_personal, name): (bool, String) =
        sqlx::query_as("SELECT is_personal, name FROM organizations WHERE id = $1")
            .bind(result.org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(is_personal);
    assert_eq!(name, "Personal");

    let role: String = sqlx::query_scalar(
        "SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2",
    )
    .bind(result.org_id)
    .bind(result.user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(role, "owner");

    let (channel, channel_user_id): (String, String) = sqlx::query_as(
        "SELECT channel, channel_user_id FROM channel_identities WHERE id = $1",
    )
    .bind(result.sender_channel_identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(channel, "telegram");
    assert_eq!(channel_user_id, "tg-user-1");

    let (org_id, channel, chat_type, chat_id): (Uuid, String, String, String) = sqlx::query_as(
        "SELECT org_id, channel, chat_type, chat_id FROM sessions WHERE id = $1",
    )
    .bind(result.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(org_id, result.org_id);
    assert_eq!(channel, "telegram");
    assert_eq!(chat_type, "dm");
    assert_eq!(chat_id, "chat-1");
}

#[sqlx::test]
async fn existing_sender_reuses_identity_org_and_session(pool: PgPool) {
    let first = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();
    let second = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(first, second);

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions").fetch_one(&pool).await.unwrap();
    assert_eq!(session_count, 1);
}

#[sqlx::test]
async fn existing_sender_new_chat_creates_a_new_session_under_the_same_personal_org(pool: PgPool) {
    let first = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();
    let second = bootstrap_identity_and_session(&pool, "telegram", "group", "chat-2", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(first.user_id, second.user_id);
    assert_eq!(first.org_id, second.org_id);
    assert_eq!(first.sender_channel_identity_id, second.sender_channel_identity_id);
    assert_ne!(first.session_id, second.session_id);

    let session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions").fetch_one(&pool).await.unwrap();
    assert_eq!(session_count, 2);
}

#[sqlx::test]
async fn resolves_the_personal_org_even_when_the_user_also_belongs_to_a_named_org(pool: PgPool) {
    let bootstrapped = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    let acme_org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Acme', false) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(acme_org_id)
        .bind(bootstrapped.user_id)
        .execute(&pool)
        .await
        .unwrap();

    let second = bootstrap_identity_and_session(&pool, "telegram", "group", "chat-2", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(second.org_id, bootstrapped.org_id);
    assert_ne!(second.org_id, acme_org_id);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_bootstrap`
Expected: FAIL to compile — `nomi_orchestrator::turn` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/types.rs
use uuid::Uuid;

use crate::llm::LlmError;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("llm call failed: {0}")]
    LlmCallFailed(#[from] LlmError),
}
```

```rust
// backend/src/turn/bootstrap.rs
use sqlx::PgPool;
use uuid::Uuid;

use super::types::TurnError;

#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapResult {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub session_id: Uuid,
}

pub async fn bootstrap_identity_and_session(
    pool: &PgPool,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
) -> Result<BootstrapResult, TurnError> {
    let existing: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, user_id FROM channel_identities WHERE channel = $1 AND channel_user_id = $2",
    )
    .bind(channel)
    .bind(sender_channel_user_id)
    .fetch_optional(pool)
    .await?;

    let (sender_channel_identity_id, user_id, org_id) = match existing {
        Some((identity_id, user_id)) => {
            let org_id: Uuid = sqlx::query_scalar(
                "SELECT o.id FROM organizations o \
                 JOIN memberships m ON m.org_id = o.id \
                 WHERE m.user_id = $1 AND o.is_personal = true",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            (identity_id, user_id, org_id)
        }
        None => {
            let mut tx = pool.begin().await?;

            let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
                .fetch_one(&mut *tx)
                .await?;

            let org_id: Uuid = sqlx::query_scalar(
                "INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id",
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(org_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            let identity_id: Uuid = sqlx::query_scalar(
                "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(user_id)
            .bind(channel)
            .bind(sender_channel_user_id)
            .fetch_one(&mut *tx)
            .await?;

            tx.commit().await?;
            (identity_id, user_id, org_id)
        }
    };

    let existing_session: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM sessions WHERE channel = $1 AND chat_id = $2")
            .bind(channel)
            .bind(chat_id)
            .fetch_optional(pool)
            .await?;

    let session_id = match existing_session {
        Some(id) => id,
        None => {
            sqlx::query_scalar(
                "INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(org_id)
            .bind(channel)
            .bind(chat_type)
            .bind(chat_id)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(BootstrapResult { user_id, org_id, sender_channel_identity_id, session_id })
}
```

```rust
// backend/src/turn/mod.rs
pub mod bootstrap;
pub mod types;

pub use types::{TurnError, TurnOutcome};
```

```rust
// backend/src/lib.rs — add this line
pub mod turn;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_bootstrap`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/types.rs backend/src/turn/bootstrap.rs backend/src/turn/mod.rs backend/src/lib.rs backend/tests/turn_bootstrap.rs
git commit -m "feat: add turn-loop types and identity/org/session bootstrap"
```

---

### Task 2: Advisory lock + Txn A (inbound message durability)

**Files:**
- Create: `backend/src/turn/lock.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod lock;`)
- Test: `backend/tests/turn_lock.rs`

**Interfaces:**
- Consumes: `TurnError` from `super::types` (Task 1).
- Produces: `async fn acquire_session_lock(pool: &PgPool, session_id: Uuid) -> Result<PoolConnection<Postgres>, TurnError>`, `async fn release_session_lock(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<(), TurnError>`, `async fn insert_inbound_message(conn: &mut PoolConnection<Postgres>, session_id: Uuid, sender_channel_identity_id: Uuid, text: &str) -> Result<Uuid, TurnError>`. Task 5 calls all three on the same connection.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_lock.rs
use std::time::{Duration, Instant};

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::lock::{acquire_session_lock, insert_inbound_message, release_session_lock};

async fn seed_session(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, identity_id)
}

#[sqlx::test]
async fn acquire_and_release_round_trip(pool: PgPool) {
    let (session_id, _) = seed_session(&pool).await;
    let mut conn = acquire_session_lock(&pool, session_id).await.unwrap();
    release_session_lock(&mut conn, session_id).await.unwrap();
}

#[sqlx::test]
async fn insert_inbound_message_persists_a_durable_row(pool: PgPool) {
    let (session_id, identity_id) = seed_session(&pool).await;
    let mut conn = acquire_session_lock(&pool, session_id).await.unwrap();

    let message_id = insert_inbound_message(&mut conn, session_id, identity_id, "hello")
        .await
        .unwrap();

    release_session_lock(&mut conn, session_id).await.unwrap();

    let content: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content, "hello");
}

#[sqlx::test]
async fn advisory_lock_serializes_two_concurrent_acquires_for_the_same_session(pool: PgPool) {
    let (session_id, _) = seed_session(&pool).await;

    let mut conn1 = acquire_session_lock(&pool, session_id).await.unwrap();

    let pool2 = pool.clone();
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        let mut conn2 = acquire_session_lock(&pool2, session_id).await.unwrap();
        let waited = start.elapsed();
        release_session_lock(&mut conn2, session_id).await.unwrap();
        waited
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    release_session_lock(&mut conn1, session_id).await.unwrap();

    let waited = handle.await.unwrap();
    assert!(waited >= Duration::from_millis(150), "second acquire should have blocked on the first, waited {waited:?}");
}

#[sqlx::test]
async fn locks_for_different_sessions_do_not_contend(pool: PgPool) {
    let (session_a, _) = seed_session(&pool).await;
    let session_b = Uuid::new_v4();

    let mut conn_a = acquire_session_lock(&pool, session_a).await.unwrap();

    let pool2 = pool.clone();
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        let mut conn_b = acquire_session_lock(&pool2, session_b).await.unwrap();
        let waited = start.elapsed();
        release_session_lock(&mut conn_b, session_b).await.unwrap();
        waited
    });

    let waited = handle.await.unwrap();
    release_session_lock(&mut conn_a, session_a).await.unwrap();

    assert!(waited < Duration::from_millis(100), "different sessions should not contend, waited {waited:?}");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_lock`
Expected: FAIL to compile — `nomi_orchestrator::turn::lock` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/lock.rs
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use super::types::TurnError;

pub async fn acquire_session_lock(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<PoolConnection<Postgres>, TurnError> {
    let mut conn = pool.acquire().await?;
    sqlx::query("SELECT pg_advisory_lock(hashtext($1))")
        .bind(session_id.to_string())
        .execute(&mut *conn)
        .await?;
    Ok(conn)
}

pub async fn release_session_lock(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<(), TurnError> {
    sqlx::query("SELECT pg_advisory_unlock(hashtext($1))")
        .bind(session_id.to_string())
        .execute(&mut **conn)
        .await?;
    Ok(())
}

pub async fn insert_inbound_message(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    text: &str,
) -> Result<Uuid, TurnError> {
    let mut tx = conn.begin().await?;
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(message_id)
}
```

```rust
// backend/src/turn/mod.rs — add this line among the existing pub mod declarations
pub mod lock;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_lock`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/lock.rs backend/src/turn/mod.rs backend/tests/turn_lock.rs
git commit -m "feat: add session advisory lock and inbound-message durability (Txn A)"
```

---

### Task 3: Sub-agent routing check (stub)

**Files:**
- Create: `backend/src/turn/routing.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod routing;`)
- Test: `backend/tests/turn_routing.rs`

**Interfaces:**
- Consumes: `TurnError` from `super::types` (Task 1).
- Produces: `async fn find_active_agent_session(conn: &mut PoolConnection<Postgres>, session_id: Uuid, sender_channel_identity_id: Uuid) -> Result<Option<Uuid>, TurnError>`. Task 5 calls this between Txn A and Txn B and discards the result (nothing populates `agent_sessions` in this plan).

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_routing.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::routing::find_active_agent_session;

async fn seed_session_and_speaker(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, identity_id)
}

#[sqlx::test]
async fn no_active_agent_session_returns_none(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}

#[sqlx::test]
async fn an_active_agent_session_returns_its_id(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, Some(agent_session_id));
}

#[sqlx::test]
async fn a_completed_agent_session_is_not_returned(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'booking', 'completed', now())",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}

#[sqlx::test]
async fn an_active_agent_session_for_a_different_speaker_is_not_returned(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let other_user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let other_identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u2') RETURNING id",
    )
    .bind(other_user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(session_id)
    .bind(other_identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_routing`
Expected: FAIL to compile — `nomi_orchestrator::turn::routing` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/routing.rs
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use super::types::TurnError;

pub async fn find_active_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
) -> Result<Option<Uuid>, TurnError> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM agent_sessions WHERE session_id = $1 AND sender_channel_identity_id = $2 AND status = 'active'",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .fetch_optional(&mut **conn)
    .await?;
    Ok(id)
}
```

```rust
// backend/src/turn/mod.rs — add this line among the existing pub mod declarations
pub mod routing;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_routing`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/routing.rs backend/src/turn/mod.rs backend/tests/turn_routing.rs
git commit -m "feat: add sub-agent routing check (stub, always falls through to chitchat)"
```

---

### Task 4: Chitchat turn (history + LLM call + Txn B)

**Files:**
- Create: `backend/src/turn/chitchat.rs`
- Create: `backend/tests/support/mod.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod chitchat;`)
- Test: `backend/tests/turn_chitchat.rs`

**Interfaces:**
- Consumes: `TurnError` from `super::types` (Task 1); `LlmProvider`, `LlmRequest`, `LlmMessage`, `LlmRole`, `ContentBlock`, `LlmResponse`, `LlmError` from `crate::llm` (already merged).
- Produces: `async fn run_chitchat_turn(conn: &mut PoolConnection<Postgres>, provider: &dyn LlmProvider, session_id: Uuid) -> Result<String, TurnError>`. Task 5 calls this after the routing check. Also produces `FakeLlmProvider` in `tests/support/mod.rs` (`FakeLlmProvider::success(LlmResponse)`, `FakeLlmProvider::failure(impl Into<String>)`, `.with_delay(Duration)`, `pub received_requests: Mutex<Vec<LlmRequest>>`), reused by Task 5's tests.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/support/mod.rs
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nomi_orchestrator::llm::{LlmError, LlmProvider, LlmRequest, LlmResponse};

enum FakeOutcome {
    Success(LlmResponse),
    Failure(String),
}

pub struct FakeLlmProvider {
    outcome: FakeOutcome,
    delay: Option<Duration>,
    pub received_requests: Mutex<Vec<LlmRequest>>,
}

impl FakeLlmProvider {
    pub fn success(response: LlmResponse) -> Self {
        Self { outcome: FakeOutcome::Success(response), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeOutcome::Failure(message.into()), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        self.received_requests.lock().unwrap().push(request);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        match &self.outcome {
            FakeOutcome::Success(response) => Ok(response.clone()),
            FakeOutcome::Failure(message) => Err(LlmError::ProviderError(message.clone())),
        }
    }
}
```

```rust
// backend/tests/turn_chitchat.rs
mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, LlmRole, StopReason};
use nomi_orchestrator::turn::chitchat::run_chitchat_turn;

use support::FakeLlmProvider;

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test]
async fn persists_reply_and_chitchat_reply_event_on_success(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Hello there!"));

    let reply = run_chitchat_turn(&mut conn, &provider, session_id).await.unwrap();
    assert_eq!(reply, "Hello there!");

    let (sender, content): (Option<Uuid>, String) = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM messages WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sender, None);
    assert_eq!(content, "Hello there!");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM agent_events WHERE session_id = $1 AND event_type = 'ChitchatReply'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload, serde_json::json!({"input_tokens": 10, "output_tokens": 5}));
}

#[sqlx::test]
async fn persists_nothing_when_the_provider_call_fails(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::failure("provider unavailable");

    let result = run_chitchat_turn(&mut conn, &provider, session_id).await;
    assert!(result.is_err());

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 0);

    let event_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(event_count, 0);
}

#[sqlx::test]
async fn keeps_only_the_last_20_messages_ordered_oldest_first(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    for i in 0..25 {
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(identity_id)
        .bind(format!("seq-{i}"))
        .bind(base + chrono::Duration::milliseconds(i))
        .execute(&pool)
        .await
        .unwrap();
    }

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));

    run_chitchat_turn(&mut conn, &provider, session_id).await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages.len(), 20);
    for (offset, message) in sent.messages.iter().enumerate() {
        let expected_seq = 5 + offset;
        match &message.content[0] {
            ContentBlock::Text { text } => assert_eq!(text, &format!("seq-{expected_seq}")),
            _ => panic!("expected a text block"),
        }
    }
}

#[sqlx::test]
async fn maps_sender_presence_to_role_correctly(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, 'hi', $3)",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(base)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, NULL, 'hello back', $2)",
    )
    .bind(session_id)
    .bind(base + chrono::Duration::milliseconds(1))
    .execute(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));

    run_chitchat_turn(&mut conn, &provider, session_id).await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages[0].role, LlmRole::User);
    assert_eq!(sent.messages[1].role, LlmRole::Assistant);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_chitchat`
Expected: FAIL to compile — `nomi_orchestrator::turn::chitchat` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/chitchat.rs
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

use super::types::TurnError;

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";
const CHITCHAT_HISTORY_LIMIT: i64 = 20;
const CHITCHAT_MAX_TOKENS: u32 = 1024;

pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    session_id: Uuid,
) -> Result<String, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(CHITCHAT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    let messages: Vec<LlmMessage> = rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect();

    let request = LlmRequest {
        system: Some(CHITCHAT_SYSTEM_PROMPT.to_string()),
        messages,
        tools: vec![],
        max_tokens: CHITCHAT_MAX_TOKENS,
    };

    // The LLM call happens outside any DB transaction: holding a transaction open across a
    // slow network round trip would needlessly extend how long this connection's locks are held.
    let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;

    let reply_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    let mut tx = conn.begin().await?;

    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
        .bind(session_id)
        .bind(&reply_text)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'ChitchatReply', $2)")
        .bind(session_id)
        .bind(serde_json::json!({
            "input_tokens": response.input_tokens,
            "output_tokens": response.output_tokens,
        }))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(reply_text)
}
```

```rust
// backend/src/turn/mod.rs — add this line among the existing pub mod declarations
pub mod chitchat;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_chitchat`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/chitchat.rs backend/src/turn/mod.rs backend/tests/support/mod.rs backend/tests/turn_chitchat.rs
git commit -m "feat: add chitchat turn (history fetch, LLM call, reply persist)"
```

---

### Task 5: `handle_inbound_message` entry point + error handling + integration tests

**Files:**
- Modify: `backend/src/turn/mod.rs` (add `handle_inbound_message`)
- Test: `backend/tests/turn_handle_inbound_message.rs`

**Interfaces:**
- Consumes: `bootstrap::bootstrap_identity_and_session` (Task 1), `lock::{acquire_session_lock, release_session_lock, insert_inbound_message}` (Task 2), `routing::find_active_agent_session` (Task 3), `chitchat::run_chitchat_turn` (Task 4), `FakeLlmProvider` from `tests/support/mod.rs` (Task 4).
- Produces: `pub async fn handle_inbound_message(pool: &PgPool, provider: &dyn LlmProvider, channel: &str, chat_type: &str, chat_id: &str, sender_channel_user_id: &str, text: &str) -> Result<TurnOutcome, TurnError>` — the plan's final public entry point.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_handle_inbound_message.rs
mod support;

use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::handle_inbound_message;

use support::FakeLlmProvider;

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test]
async fn new_sender_gets_bootstrapped_and_receives_a_chitchat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("Hi! How can I help?"));

    let outcome = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "hello")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi! How can I help?");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(outcome.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // inbound + reply

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let org_count: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations").fetch_one(&pool).await.unwrap();
    assert_eq!(org_count, 1);
}

#[sqlx::test]
async fn existing_sender_reuses_identity_and_session_across_two_calls(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("ok"));

    let first = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "first")
        .await
        .unwrap();
    let second = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "second")
        .await
        .unwrap();

    assert_eq!(first.session_id, second.session_id);

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 4); // 2 inbound + 2 replies

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
}

#[sqlx::test]
async fn inbound_message_is_durable_even_when_the_provider_call_fails(pool: PgPool) {
    let provider = FakeLlmProvider::failure("provider unavailable");

    let result = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "hello")
        .await;
    assert!(result.is_err());

    let (content,): (String,) = sqlx::query_as(
        "SELECT content FROM messages m JOIN sessions s ON m.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "hello");

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events e JOIN sessions s ON e.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}

#[sqlx::test]
async fn an_active_agent_session_does_not_block_the_chitchat_fallback_in_this_slice(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("still chatting"));

    let first = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "hi")
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT ci.id FROM channel_identities ci WHERE ci.channel = 'telegram' AND ci.channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(first.session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let second = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "still there?")
        .await
        .unwrap();

    assert_eq!(second.reply, "still chatting");
}

#[sqlx::test]
async fn concurrent_messages_for_the_same_session_are_serialized(pool: PgPool) {
    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    handle_inbound_message(&pool, &bootstrap_provider, "telegram", "dm", "chat-1", "tg-1", "bootstrap")
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok")).with_delay(Duration::from_millis(200));

    let call1 = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "first");
    let call2 = handle_inbound_message(&pool, &provider, "telegram", "dm", "chat-1", "tg-1", "second");
    let (result1, result2) = tokio::join!(call1, call2);
    result1.unwrap();
    result2.unwrap();

    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT m.sender_channel_identity_id, m.content FROM messages m \
         JOIN sessions s ON m.session_id = s.id \
         WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1' \
         ORDER BY m.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // bootstrap turn (2 rows) + two serialized turns (2 rows each) = 6, never interleaved.
    assert_eq!(rows.len(), 6);
    let (first_inbound, first_reply, second_inbound, second_reply) =
        (&rows[2], &rows[3], &rows[4], &rows[5]);
    assert!(first_inbound.0.is_some());
    assert!(first_inbound.1 == "first" || first_inbound.1 == "second");
    assert!(first_reply.0.is_none());
    assert_ne!(first_inbound.1, second_inbound.1);
    assert!(second_inbound.0.is_some());
    assert!(second_reply.0.is_none());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_handle_inbound_message`
Expected: FAIL to compile — `handle_inbound_message` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/mod.rs — final version
pub mod bootstrap;
pub mod chitchat;
pub mod lock;
pub mod routing;
pub mod types;

pub use types::{TurnError, TurnOutcome};

use sqlx::PgPool;

use crate::llm::LlmProvider;

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    // Sub-agent routing is scaffolded but not yet implemented: nothing populates
    // agent_sessions in this plan, so every turn falls through to chitchat. See
    // docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md §2 step 5.
    let _active_agent_session_id =
        routing::find_active_agent_session(&mut conn, session_id, sender_channel_identity_id).await?;

    let result = chitchat::run_chitchat_turn(&mut conn, provider, session_id).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            // Best-effort: a failed event write here must never mask the original error.
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

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
```

Note on the success path: `release_lock_ignoring_errors` is used on both branches so a lock-release failure (rare, but possible if the connection has already gone bad) never masks the turn's own outcome — this matches the design's requirement that the lock is released "always, on both the success and failure paths," implemented as an explicit call rather than a scope guard. On the success path this discards a hypothetical release error; if this matters later (e.g. surfacing it as a warning log), that is future hardening, not part of this plan's scope.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_handle_inbound_message`
Expected: PASS (5 tests)

- [ ] **Step 5: Run the full test suite**

Run: `cd backend && cargo test`
Expected: PASS, no regressions (all prior suites plus the new `turn_*` suites).

- [ ] **Step 6: Commit**

```bash
git add backend/src/turn/mod.rs backend/tests/turn_handle_inbound_message.rs
git commit -m "feat: wire up handle_inbound_message turn-loop entry point"
```
