# Personality Customization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a user change nomi's conversational personality by asking in chat, with every change recorded as an auditable `agent_events` row, and have that personality (and nomi's existing memory of the user) apply consistently across every chat context the same person uses — DM or group, any `chat_id`.

**Architecture:** A new `PersonalityAgent` (`nomi-agent-personality` crate), the second real `SubAgent` after `MoneyAgent`, exposes one tool (`set_personality`) that upserts a per-`user_id` row in a new `user_personality` table and logs a `PersonalityChanged` audit event — both through one new access-point module, `nomi_agent_core::personality`, mirroring the existing `nomi_agent_core::memory` module. `SubAgent` gains a `uses_personality()` opt-in flag (mirroring `uses_memory()`); `run_agent_turn` folds the stored personality into any opted-in agent's system prompt. `ChitchatAgent` opts in — this is the actual point of the feature, since chitchat is the agent that talks to the user day to day. Cross-chat consistency requires zero new code: `channel_identities` already resolves `(channel, channel_user_id)` to one stable `user_id` regardless of `chat_type`/`chat_id`, and the new table is keyed by that same `user_id`, exactly like `memory_items` already is.

**Tech Stack:** Rust, Cargo workspace, sqlx (Postgres, `#[sqlx::test(migrations = "../../migrations")]`), async-trait, serde_json.

**Spec:** `docs/superpowers/specs/2026-08-28-personality-customization-design.md`

## Global Constraints

- Personality is per-`user_id` only — no per-org, per-session, or per-chat-context personality (spec §3, §Out of Scope).
- No moderation, content filtering, or length validation on personality descriptions beyond non-empty (spec §2, §Error Handling) — this iteration has no extra guardrails.
- No new audit table — every personality change is a `PersonalityChanged` row in the existing `agent_events` table (spec §3).
- `user_personality` holds only the *current* description per user (single row per `user_id`, upserted) — history lives entirely in `agent_events` (spec §3).
- All reads/writes to `user_personality` go through one access point, `nomi_agent_core::personality` — no other crate writes SQL against that table directly (spec §3).
- Every new/modified crate follows this workspace's existing `#[sqlx::test(migrations = "../../migrations")]` pattern; that path is two directories up to `backend/migrations/` from any crate under `backend/crates/*`.
- Never use `FakeLlmProvider::success(...)` for a test that drives a full `handle_inbound_message` call — use `FakeLlmProvider::sequence(vec![...])` with one explicit response per LLM call the turn will make (classification, then each tool-loop turn, then memory extraction if the terminal agent's `uses_memory()` is `true`). A `success()` provider silently reuses the same canned text for every call, which previously planted a spurious extracted memory (see `docs/todo/reinforce-memory-usage-gap.md`'s resolution).

---

### Task 1: `user_personality` migration

**Files:**
- Create: `backend/migrations/0013_user_personality.sql`

**Interfaces:**
- Produces: a `user_personality` table with columns `user_id UUID PRIMARY KEY REFERENCES users(id)`, `description TEXT NOT NULL`, `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`, consumed by Task 2's `nomi_agent_core::personality` module.

- [ ] **Step 1: Write the migration**

```sql
CREATE TABLE user_personality (
    user_id     UUID PRIMARY KEY REFERENCES users(id),
    description TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Save as `backend/migrations/0013_user_personality.sql`.

- [ ] **Step 2: Verify the migration applies cleanly**

Run any existing test that uses the shared migrations path — this exercises every migration in `backend/migrations/` including the new one:

```bash
cd backend && cargo test -p nomi-agent-core --test engine -- --test-threads=1
```

Expected: all existing tests still PASS (proves `0013_user_personality.sql` applies without error alongside every prior migration).

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/0013_user_personality.sql
git commit -m "feat: add the user_personality table"
```

---

### Task 2: `nomi_agent_core::personality` module, `uses_personality()`, and system-prompt fold-in

This task widens `SubAgent::execute_tool`'s signature to also pass `session_id`/`agent_session_id` (needed so `PersonalityAgent`'s tool can write a properly-linked `PersonalityChanged` audit event) — every existing implementor (`ChitchatAgent`, `MoneyAgent`) and every direct call site must be updated in the same commit for the workspace to compile.

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs`
- Modify: `backend/crates/nomi-agent-core/src/engine.rs`
- Create: `backend/crates/nomi-agent-core/src/personality.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs`
- Modify: `backend/crates/nomi-agent-core/tests/engine.rs`
- Create: `backend/crates/nomi-agent-core/tests/personality.rs`
- Modify: `backend/crates/nomi-agent-chitchat/src/lib.rs`
- Modify: `backend/crates/nomi-agent-money/src/lib.rs`
- Modify: `backend/crates/nomi-agent-money/tests/turn_money_agent.rs`

**Interfaces:**
- Produces (used by Task 3): `nomi_agent_core::personality::get_current_personality(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String>` and `nomi_agent_core::personality::set_personality(conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, new_description: &str) -> Result<(), TurnError>`.
- Produces (used by Task 3): `SubAgent::uses_personality(&self) -> bool` (default `false`).
- Produces (used by Tasks 3, 4): the widened trait method `async fn execute_tool(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, name: &str, input: Value) -> Result<String, String>`.

#### Step 1: Widen `SubAgent::execute_tool` and add `uses_personality()`

- [ ] Edit `backend/crates/nomi-agent-core/src/subagent.rs` — replace the trait's `execute_tool` signature and add the new method:

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String>;

    /// Fed verbatim into the intent classifier's prompt. Never called for the agent that
    /// returns `true` from `is_default()` — that agent is the fallback, not something the
    /// classifier picks between.
    fn intent_label(&self) -> &'static str;
    fn intent_description(&self) -> &'static str;

    /// Exactly one registered agent must return `true`. See `AgentRegistry::new`.
    fn is_default(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` retrieves relevant memories before the first LLM call
    /// (folded into the system prompt) and extracts+stores new memories after a `Reply`
    /// outcome (never after `Completed` — a completed task summary isn't a conversational
    /// reply worth remembering facts from).
    fn uses_memory(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` folds the user's currently stored personality (if any)
    /// into the system prompt for this turn. See `nomi_agent_core::personality`.
    fn uses_personality(&self) -> bool {
        false
    }
}
```

#### Step 2: Update every existing implementor and call site to compile against the widened signature

- [ ] Edit `backend/crates/nomi-agent-core/src/engine.rs` — update the call site (around line 161):

```rust
                } else {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(text) => (text, false),
                        Err(err) => (err, true),
                    }
                };
```

- [ ] Edit `backend/crates/nomi-agent-chitchat/src/lib.rs` — update `execute_tool`'s signature and body:

```rust
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        _name: &str,
        _input: Value,
    ) -> Result<String, String> {
        // Unreachable: tools() returns an empty list, so the engine never calls this for
        // chitchat (the only tool it could ever see is complete_task, which the engine
        // handles itself before reaching an agent's execute_tool).
        Err("chitchat has no tools".to_string())
    }
```

- [ ] Edit `backend/crates/nomi-agent-money/src/lib.rs` — update `execute_tool`'s signature (body unchanged, `session_id`/`agent_session_id` unused):

```rust
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "list_transactions" => list_transactions(conn, user_id, input).await,
            "summarize_budget" => summarize_budget(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

- [ ] Edit `backend/crates/nomi-agent-money/tests/turn_money_agent.rs` — every direct call site passes `user_id` as `execute_tool`'s second argument; insert two `Uuid::new_v4()` arguments before it. Run this from the `backend` directory:

```bash
sed -i '' 's/\.execute_tool(&mut conn, user_id,/.execute_tool(\&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id,/g' crates/nomi-agent-money/tests/turn_money_agent.rs
```

Then verify every occurrence was updated:

```bash
grep -n "execute_tool" crates/nomi-agent-money/tests/turn_money_agent.rs
```

Expected: all 7 call sites now read `.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, ...)`.

- [ ] Edit `backend/crates/nomi-agent-core/tests/engine.rs` — update `TestAgent`'s `execute_tool` (the only other implementor in the workspace):

```rust
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<String, String> {
        match name {
            "echo" => Ok(format!("echoed: {input}")),
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

- [ ] Verify the whole workspace still compiles:

```bash
cd backend && cargo build --workspace
```

Expected: builds cleanly with no errors.

#### Step 3: Write the failing tests for `nomi_agent_core::personality`

- [ ] Create `backend/crates/nomi-agent-core/tests/personality.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::personality::{get_current_personality, set_personality};

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

async fn seed_agent_session(pool: &PgPool, session_id: Uuid) -> (Uuid, Uuid) {
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
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'personality', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, agent_session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_none_when_no_personality_is_set(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = get_current_personality(&mut conn, user_id).await;
    assert_eq!(result, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn first_change_upserts_the_row_and_records_a_null_old_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, session_id, agent_session_id, user_id, "Be sarcastic and blunt.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic and blunt.".to_string()));

    let (event_type, payload): (String, serde_json::Value) =
        sqlx::query_as("SELECT event_type, payload FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_type, "PersonalityChanged");
    assert_eq!(payload["old_description"], serde_json::Value::Null);
    assert_eq!(payload["new_description"], "Be sarcastic and blunt.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn second_change_updates_the_same_row_and_records_the_prior_value_as_old_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, session_id, agent_session_id, user_id, "Be sarcastic and blunt.").await.unwrap();
    set_personality(&mut conn, session_id, agent_session_id, user_id, "Be warm and encouraging.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be warm and encouraging.".to_string()));

    let row_count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row_count, 1);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'PersonalityChanged'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 2);

    let (payload,): (serde_json::Value,) = sqlx::query_as(
        "SELECT payload FROM agent_events WHERE agent_session_id = $1 AND event_type = 'PersonalityChanged' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload["old_description"], "Be sarcastic and blunt.");
    assert_eq!(payload["new_description"], "Be warm and encouraging.");
}
```

- [ ] Run it to verify it fails to compile (the `personality` module doesn't exist yet):

```bash
cd backend && cargo test -p nomi-agent-core --test personality
```

Expected: FAIL with `unresolved import 'nomi_agent_core::personality'` or similar.

#### Step 4: Implement `nomi_agent_core::personality`

- [ ] Create `backend/crates/nomi-agent-core/src/personality.rs`:

```rust
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::error::TurnError;

pub async fn get_current_personality(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten()
}

/// Upserts `user_personality` and records a `PersonalityChanged` `agent_events` row with
/// `{old_description, new_description}` (`old_description` is `null` on a user's first-ever
/// change) — both in one transaction. The literal `"personality"` agent_type below is
/// `nomi-agent-personality`'s `PERSONALITY_AGENT_TYPE`, duplicated here because
/// `nomi-agent-core` sits below `nomi-agent-personality` in the dependency graph (the same
/// convention every other agent_type string in this crate already follows).
pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError> {
    let mut tx = conn.begin().await?;

    let old_description: Option<String> =
        sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

    sqlx::query(
        "INSERT INTO user_personality (user_id, description, updated_at) VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET description = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_description)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) \
         VALUES ($1, $2, 'personality', 'PersonalityChanged', $3)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(serde_json::json!({"old_description": old_description, "new_description": new_description}))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
```

- [ ] Edit `backend/crates/nomi-agent-core/src/lib.rs` — add the module:

```rust
pub mod engine;
pub mod error;
pub mod memory;
pub mod personality;
pub mod registry;
pub mod subagent;

pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME};
pub use error::TurnError;
pub use registry::AgentRegistry;
pub use subagent::SubAgent;
```

- [ ] Run the personality tests to verify they pass:

```bash
cd backend && cargo test -p nomi-agent-core --test personality
```

Expected: 3 tests PASS.

- [ ] Commit:

```bash
git add backend/crates/nomi-agent-core/src/subagent.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-core/src/personality.rs backend/crates/nomi-agent-core/src/lib.rs \
        backend/crates/nomi-agent-core/tests/engine.rs backend/crates/nomi-agent-core/tests/personality.rs \
        backend/crates/nomi-agent-chitchat/src/lib.rs backend/crates/nomi-agent-money/src/lib.rs \
        backend/crates/nomi-agent-money/tests/turn_money_agent.rs
git commit -m "feat: add nomi_agent_core::personality and widen SubAgent::execute_tool"
```

#### Step 5: Write the failing tests for the system-prompt fold-in

- [ ] Edit `backend/crates/nomi-agent-core/tests/engine.rs` — add a second test agent right after `TestAgent`'s `impl SubAgent for TestAgent` block:

```rust
struct PersonalityAwareTestAgent;

#[async_trait::async_trait]
impl SubAgent for PersonalityAwareTestAgent {
    fn agent_type(&self) -> &'static str {
        "personality-aware-test"
    }
    fn system_prompt(&self) -> &'static str {
        "test prompt"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: serde_json::Value,
    ) -> Result<String, String> {
        Err(format!("unknown tool: {name}"))
    }
    fn intent_label(&self) -> &'static str {
        "personality-aware-test"
    }
    fn intent_description(&self) -> &'static str {
        "test agent"
    }
    fn uses_personality(&self) -> bool {
        true
    }
}
```

Then add two new tests at the end of the file:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn a_stored_personality_is_folded_into_the_system_prompt_when_uses_personality_is_true(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_personality (user_id, description) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Be sarcastic and blunt.")
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Fine.", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedding_provider,
        &PersonalityAwareTestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stored_personality_is_not_folded_in_for_an_agent_that_does_not_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_personality (user_id, description) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Be sarcastic and blunt.")
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Fine.", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedding_provider,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(requests[0].system.as_ref().unwrap(), "test prompt");
}
```

- [ ] Add the missing import at the top of the file (`FakeLlmProvider`/`FakeEmbeddingProvider` are already imported; nothing else is needed since `PersonalityAwareTestAgent` reuses `ToolDefinition`, `PoolConnection`, `Postgres`, `Uuid` already imported for `TestAgent`).

- [ ] Run the two new tests to verify they fail (the fold-in doesn't exist yet):

```bash
cd backend && cargo test -p nomi-agent-core --test engine a_stored_personality
```

Expected: `a_stored_personality_is_folded_into_the_system_prompt_when_uses_personality_is_true` FAILS (system prompt doesn't contain the personality text); `a_stored_personality_is_not_folded_in_for_an_agent_that_does_not_opt_in` PASSES already (nothing folds anything in yet).

#### Step 6: Implement the fold-in

- [ ] Edit `backend/crates/nomi-agent-core/src/engine.rs` — insert this block immediately after the existing memory fold-in (`let system_prompt = if memories.is_empty() { ... } else { ... };`) and before `for _ in 0..MAX_TOOL_TURNS {`:

```rust
    let system_prompt = if agent.uses_personality() {
        match crate::personality::get_current_personality(conn, user_id).await {
            Some(p) => format!("{system_prompt}\n\nAdopt this personality in your replies: {p}"),
            None => system_prompt,
        }
    } else {
        system_prompt
    };
```

- [ ] Run the two new tests again to verify they now pass:

```bash
cd backend && cargo test -p nomi-agent-core --test engine a_stored_personality
```

Expected: both PASS.

- [ ] Run the whole crate's test suite to check nothing else broke:

```bash
cd backend && cargo test -p nomi-agent-core
```

Expected: all tests PASS.

- [ ] Commit:

```bash
git add backend/crates/nomi-agent-core/src/engine.rs backend/crates/nomi-agent-core/tests/engine.rs
git commit -m "feat: fold a user's stored personality into opted-in agents' system prompts"
```

---

### Task 3: `nomi-agent-personality` crate

**Files:**
- Create: `backend/crates/nomi-agent-personality/Cargo.toml`
- Create: `backend/crates/nomi-agent-personality/src/lib.rs`
- Create: `backend/crates/nomi-agent-personality/tests/personality_agent.rs`

**Interfaces:**
- Consumes: `nomi_agent_core::{SubAgent, personality::set_personality}` (Task 2).
- Produces (used by Tasks 4, 5, 6): `pub const PERSONALITY_AGENT_TYPE: &str = "personality";` and `pub struct PersonalityAgent;` implementing `SubAgent`.

Note: `PersonalityAgent::uses_personality()` returns `true` so that, once a personality is already set from an earlier turn, the agent's *own* replies (e.g. mid-conversation clarifying questions) are already in-character. Because `run_agent_turn` computes the system prompt once at the top of a turn (Task 2, Step 6), a `set_personality` call happening *during* that same turn does not retroactively change that turn's own `complete_task` confirmation text — the new personality takes effect starting with the *next* turn. This matches the spec's testable acceptance criteria (spec §5's e2e test asserts the personality shows up in the following chitchat turn, not the same-turn confirmation) and requires no extra engine changes.

The workspace root's `Cargo.toml` has `members = ["crates/*"]`, so this new crate is picked up automatically — no root manifest edit needed.

- [ ] **Step 1: Write the crate manifest**

Create `backend/crates/nomi-agent-personality/Cargo.toml`:

```toml
[package]
name = "nomi-agent-personality"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
async-trait = "0.1"
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
nomi-test-support = { path = "../nomi-test-support" }
```

- [ ] **Step 2: Write the failing tests**

Create `backend/crates/nomi-agent-personality/tests/personality_agent.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_personality::PersonalityAgent;

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

async fn seed_agent_session(pool: &PgPool, session_id: Uuid) -> (Uuid, Uuid) {
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
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'personality', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, agent_session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_upserts_and_records_an_audit_event(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic and blunt."}),
        )
        .await
        .unwrap();

    assert!(result.contains("Be sarcastic and blunt."));

    let stored: String = sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "Be sarcastic and blunt.");

    let event_type: String =
        sqlx::query_scalar("SELECT event_type FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_type, "PersonalityChanged");
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_rejects_an_empty_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "   "}),
        )
        .await;

    assert!(result.is_err());

    let row_count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row_count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_rejects_a_missing_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_tool_name_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "delete_everything", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
```

Run it to verify it fails to compile (the crate doesn't exist yet):

```bash
cd backend && cargo test -p nomi-agent-personality
```

Expected: FAIL (crate/package not found).

- [ ] **Step 3: Implement `PersonalityAgent`**

Create `backend/crates/nomi-agent-personality/src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const PERSONALITY_AGENT_TYPE: &str = "personality";

const PERSONALITY_SYSTEM_PROMPT: &str =
    "You help the user customize nomi's personality — the tone, style, and manner nomi should \
     adopt in future replies. When the user describes how they want nomi to talk or behave, call \
     set_personality with a concise (one or two sentence) description of that personality, written \
     in the second person as an instruction (e.g. 'Be sarcastic and blunt, never overly polite.'). \
     Confirm the change back to the user in a friendly way, in the new personality if one was just \
     set. When you're done, call complete_task.";

pub struct PersonalityAgent;

#[async_trait]
impl SubAgent for PersonalityAgent {
    fn agent_type(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        PERSONALITY_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "set_personality".to_string(),
            description: "Set nomi's personality — how it should talk and behave in future replies — for this user."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "A concise instruction describing the personality, e.g. 'Be sarcastic and blunt.'"
                    }
                },
                "required": ["description"]
            }),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "set_personality" => set_personality(conn, session_id, agent_session_id, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "The user wants to change how nomi talks or behaves — its tone, style, or personality"
    }

    fn uses_personality(&self) -> bool {
        true
    }
}

async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let description = input
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "description is required and must not be empty".to_string())?;

    nomi_agent_core::personality::set_personality(conn, session_id, agent_session_id, user_id, description)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Personality updated to: {description}"))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

```bash
cd backend && cargo test -p nomi-agent-personality
```

Expected: 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-agent-personality
git commit -m "feat: add the PersonalityAgent sub-agent"
```

---

### Task 4: `ChitchatAgent` opts into personality

This is the point of the whole feature: once a personality is set, every future chitchat reply picks it up automatically.

**Files:**
- Modify: `backend/crates/nomi-agent-chitchat/src/lib.rs`
- Modify: `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs`

**Interfaces:**
- Consumes: `SubAgent::uses_personality()` (Task 2), `nomi_agent_core::personality` fold-in behavior (Task 2).

- [ ] **Step 1: Write the failing test**

Add to the end of `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs`:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn a_stored_personality_is_folded_into_chitchats_system_prompt(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    sqlx::query("INSERT INTO user_personality (user_id, description) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Be sarcastic and blunt.")
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Fine, whatever."), text_response("NONE")]);
    let embedder = FakeEmbeddingProvider::success(make_embedding(0.5));

    run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedder,
        &ChitchatAgent,
        Uuid::new_v4(),
        Uuid::new_v4(),
        user_id,
        vec![user_message("hi")],
        CHITCHAT_MAX_TOKENS,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}
```

Run it to verify it fails (`ChitchatAgent::uses_personality()` is still `false`, so nothing is folded in):

```bash
cd backend && cargo test -p nomi-agent-chitchat --test chitchat_agent a_stored_personality
```

Expected: FAIL — `assert!` on the `contains(...)` check fails.

- [ ] **Step 2: Flip the flag**

Edit `backend/crates/nomi-agent-chitchat/src/lib.rs` — add `uses_personality()` to the existing `impl SubAgent for ChitchatAgent` block, right after `uses_memory()`:

```rust
    fn uses_memory(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }
```

- [ ] **Step 3: Run the test to verify it passes**

```bash
cd backend && cargo test -p nomi-agent-chitchat --test chitchat_agent a_stored_personality
```

Expected: PASS.

- [ ] **Step 4: Run the whole crate's test suite**

```bash
cd backend && cargo test -p nomi-agent-chitchat
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-agent-chitchat/src/lib.rs backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs
git commit -m "feat: fold the user's personality into chitchat replies"
```

---

### Task 5: Register `PersonalityAgent` in `nomi-server`

**Files:**
- Modify: `backend/crates/nomi-server/Cargo.toml`
- Modify: `backend/crates/nomi-server/src/lib.rs`

**Interfaces:**
- Consumes: `nomi_agent_personality::PersonalityAgent` (Task 3).

- [ ] **Step 1: Add the dependency**

Edit `backend/crates/nomi-server/Cargo.toml` — add this line under `[dependencies]`, next to the other agent crates:

```toml
nomi-agent-personality = { path = "../nomi-agent-personality" }
```

So the `[dependencies]` block reads (only the agent-crate lines shown for context):

```toml
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-agent-money = { path = "../nomi-agent-money" }
nomi-agent-chitchat = { path = "../nomi-agent-chitchat" }
nomi-agent-personality = { path = "../nomi-agent-personality" }
```

- [ ] **Step 2: Register the agent**

Edit `backend/crates/nomi-server/src/lib.rs`:

```rust
pub fn build_agent_registry() -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_personality::PersonalityAgent),
    ])
}
```

- [ ] **Step 3: Verify the workspace builds**

```bash
cd backend && cargo build --workspace
```

Expected: builds cleanly.

- [ ] **Step 4: Run `nomi-server`'s existing test suite**

```bash
cd backend && cargo test -p nomi-server
```

Expected: all existing tests still PASS (nothing in `nomi-server`'s tests depends on the exact set of registered agents beyond what `session_ws.rs` builds inline, which is untouched).

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-server/Cargo.toml backend/crates/nomi-server/src/lib.rs
git commit -m "feat: register PersonalityAgent in the composition root"
```

---

### Task 6: End-to-end tests in `nomi-turn`

Proves the two things the spec cares about most: a personality change is recorded and shows up in the very next chitchat reply, and — the concrete verification of §1's "already solved" claim — a personality set in one chat context is visible from a *different* chat context for the same person.

**Files:**
- Modify: `backend/crates/nomi-turn/Cargo.toml`
- Modify: `backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs`

**Interfaces:**
- Consumes: `nomi_agent_personality::PersonalityAgent` (Task 3), `handle_inbound_message` (existing, unchanged signature).

- [ ] **Step 1: Add the dev-dependency**

Edit `backend/crates/nomi-turn/Cargo.toml` — add this line under `[dev-dependencies]`:

```toml
nomi-agent-personality = { path = "../nomi-agent-personality" }
```

So `[dev-dependencies]` reads:

```toml
[dev-dependencies]
nomi-agent-money = { path = "../nomi-agent-money" }
nomi-agent-chitchat = { path = "../nomi-agent-chitchat" }
nomi-agent-personality = { path = "../nomi-agent-personality" }
nomi-test-support = { path = "../nomi-test-support" }
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Write the two new tests**

Edit `backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs` — add the import and a `tool_use_response` helper near the top (after the existing `canned_response` helper):

```rust
use nomi_agent_personality::PersonalityAgent;

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 10,
        output_tokens: 5,
    }
}
```

Then add these two tests at the end of the file:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn a_personality_change_is_recorded_and_folded_into_the_next_chitchat_reply(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Turn 1: classified as "personality"; the agent calls set_personality, then complete_task.
    let personality_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Be sarcastic and blunt."})),
        tool_use_response(
            "t2",
            "complete_task",
            serde_json::json!({"status": "completed", "summary": "Done, I'll be blunt now."}),
        ),
    ]);
    let outcome = handle_inbound_message(
        &pool,
        &personality_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "be more sarcastic and blunt",
        None,
    )
    .await
    .unwrap();
    assert_eq!(outcome.reply, "Done, I'll be blunt now.");

    let event_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE event_type = 'PersonalityChanged'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_count, 1);

    // Turn 2: a plain chitchat message. The personality agent_session from Turn 1 already
    // completed, so this reclassifies fresh and falls through to chitchat.
    let chitchat_provider =
        FakeLlmProvider::sequence(vec![canned_response("chitchat"), canned_response("Sure thing."), canned_response("NONE")]);
    handle_inbound_message(
        &pool,
        &chitchat_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "what's up?",
        None,
    )
    .await
    .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_personality_set_in_one_chat_context_is_visible_in_a_different_chat_context_for_the_same_person(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Seed a personality change through a personal DM.
    let personality_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Talk like a 1920s detective."})),
        tool_use_response(
            "t2",
            "complete_task",
            serde_json::json!({"status": "completed", "summary": "Right-o, consider it done, see."}),
        ),
    ]);
    handle_inbound_message(
        &pool,
        &personality_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "dm-chat",
        "tg-user-1",
        "talk like a detective",
        None,
    )
    .await
    .unwrap();

    // Same sender (same channel_user_id), but a DIFFERENT chat_id and chat_type ("group") — a
    // distinct session, same person per channel_identities' (channel, channel_user_id)
    // resolution (proven by turn_bootstrap.rs's existing
    // existing_sender_new_chat_creates_a_new_session_under_the_same_personal_org).
    let chitchat_provider = FakeLlmProvider::sequence(vec![
        canned_response("chitchat"),
        canned_response("Say, what can I do for ya?"),
        canned_response("NONE"),
    ]);
    let group_outcome = handle_inbound_message(
        &pool,
        &chitchat_provider,
        &embedder,
        &registry,
        "telegram",
        "group",
        "group-chat",
        "tg-user-1",
        "hello there",
        None,
    )
    .await
    .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Talk like a 1920s detective."));

    let dm_session_id: Uuid =
        sqlx::query_scalar("SELECT id FROM sessions WHERE channel = 'telegram' AND chat_id = 'dm-chat'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(dm_session_id, group_outcome.session_id);
}
```

- [ ] **Step 3: Run the two new tests to verify they fail before the dependency is wired**

```bash
cd backend && cargo test -p nomi-turn --test turn_handle_inbound_message a_personality
```

Expected: FAIL to compile (`nomi_agent_personality` isn't a dependency of `nomi-turn` yet — this step is really just confirming Step 1 hasn't already silently been done; if Step 1 is already applied, skip straight to Step 4).

- [ ] **Step 4: Run the two new tests to verify they pass**

```bash
cd backend && cargo test -p nomi-turn --test turn_handle_inbound_message a_personality
```

Expected: both tests PASS.

- [ ] **Step 5: Run the whole workspace's test suite**

```bash
cd backend && cargo test --workspace
```

Expected: all tests PASS.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-turn/Cargo.toml backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs
git commit -m "test: prove personality changes are recorded and cross chat contexts for the same person"
```
