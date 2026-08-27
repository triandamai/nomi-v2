# Personality Versioning and Rollback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give every personality change an explicit, numbered version, and let a user roll back to any prior version — from chat (asking nomi) or from a new panel in the chat UI — without ever deleting or mutating history.

**Architecture:** A new `user_personality_versions` table holds an immutable, append-only history per user; `user_personality` keeps its existing role as an O(1) "current" pointer, gaining a `current_version` column. `nomi_agent_core::personality` gains `list_versions` (a read) and `rollback_to_version` (a thin wrapper that looks up an old description and calls the existing `set_personality`, which already becomes the single write path for every version — chat-driven or not). `set_personality`'s `session_id`/`agent_session_id` widen from `Uuid` to `Option<Uuid>` so a rollback triggered outside any chat turn (the web UI) can still write a valid audit event. `PersonalityAgent` gets two new tools (`list_personality_versions`, `rollback_personality`); the chat page gets a new dropdown panel mirroring the existing model-picker's interaction pattern, backed by two new HTTP routes.

**Tech Stack:** Rust, Cargo workspace, sqlx (Postgres), axum, SvelteKit 2 / Svelte 5, Tailwind, MD3 design tokens (CSS custom properties).

**Spec:** `docs/superpowers/specs/2026-08-28-personality-versioning-rollback-design.md`

## Global Constraints

- Personality versioning stays per-user only — no per-org or per-session scoping (spec §1, unchanged from the original personality feature).
- Rollback never mutates or deletes an existing `user_personality_versions` row — restoring an old version always creates a *new* version with that version's description (git-revert semantics, not git-reset) (spec §1, §3).
- `set_personality`'s `session_id`/`agent_session_id` parameters widen from `Uuid` to `Option<Uuid>`, `None` bound as SQL `NULL` for changes with no chat-turn context — every existing call site must be updated in the same task that changes the signature, the same discipline the prior feature's trait-widening task used (spec §2).
- `agent_events` gains no new columns and no new event type — rollbacks are recorded as ordinary `PersonalityChanged` events (payload gains `new_version`) (spec §2).
- Every new/modified Rust crate follows the existing `#[sqlx::test(migrations = "../../migrations")]` pattern; that path is two directories up to `backend/migrations/` from any crate under `backend/crates/*`.
- The frontend panel follows the chat page's existing model-picker interaction pattern exactly: a `$state` boolean toggling an absolute-positioned panel, SvelteKit form actions with `use:enhance`, MD3 CSS custom properties for styling — no new UI library or pattern (spec §5).
- Setting a brand-new personality by typing free text is **not** added to the frontend panel — that stays chat-only (spec §5, Out of Scope).
- No automated frontend tests are added for the panel — the codebase has no existing frontend test files for the model-picker feature either (`frontend/e2e` and `frontend/src` currently contain zero `.test.ts`/`.spec.ts` files), so there is no convention to follow yet. Verification for the frontend task is `npm run check` (svelte-check + TypeScript) plus careful self-review, not a test run.

---

### Task 1: `user_personality_versions` migration

**Files:**
- Create: `backend/migrations/0014_personality_versioning.sql`

**Interfaces:**
- Produces: `user_personality_versions` table (`id`, `user_id`, `version`, `description`, `created_at`, `UNIQUE (user_id, version)`) and `user_personality.current_version INT NOT NULL`, consumed by Task 2's widened `nomi_agent_core::personality` module.

- [ ] **Step 1: Write the migration**

```sql
CREATE TABLE user_personality_versions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    version     INT NOT NULL,
    description TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, version)
);

ALTER TABLE user_personality ADD COLUMN current_version INT NOT NULL DEFAULT 1;

INSERT INTO user_personality_versions (user_id, version, description, created_at)
SELECT user_id, 1, description, updated_at FROM user_personality;
```

Save as `backend/migrations/0014_personality_versioning.sql`. The backfill `INSERT` is required, not optional: without it, any user who already had a personality set before this migration would have `current_version = 1` (from the `DEFAULT 1`) but zero matching rows in `user_personality_versions` — `list_versions` would wrongly report empty history for someone who demonstrably has a personality set today.

- [ ] **Step 2: Verify the migration applies cleanly**

```bash
cd backend && cargo test -p nomi-agent-core --test personality -- --test-threads=1
```

Expected: all existing tests still PASS (proves the new migration applies without error alongside every prior one, including `0013_user_personality.sql`).

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/0014_personality_versioning.sql
git commit -m "feat: add the user_personality_versions table"
```

---

### Task 2: Widen `nomi_agent_core::personality` and add `list_versions`/`rollback_to_version`

**Files:**
- Modify: `backend/crates/nomi-agent-core/Cargo.toml`
- Modify: `backend/crates/nomi-agent-core/src/personality.rs`
- Modify: `backend/crates/nomi-agent-core/tests/personality.rs`

**Interfaces:**
- Produces (used by Task 3): `set_personality(conn, session_id: Option<Uuid>, agent_session_id: Option<Uuid>, user_id: Uuid, new_description: &str) -> Result<(), TurnError>` — widened from `Uuid, Uuid`.
- Produces (used by Tasks 3, 5): `pub struct PersonalityVersion { version: i32, description: String, created_at: DateTime<Utc>, is_current: bool }` and `list_versions(conn, user_id: Uuid, limit: i64) -> Result<Vec<PersonalityVersion>, TurnError>`.
- Produces (used by Tasks 3, 5): `pub enum RollbackError { Db(sqlx::Error), VersionNotFound(i32), SetPersonality(TurnError) }` and `rollback_to_version(conn, session_id: Option<Uuid>, agent_session_id: Option<Uuid>, user_id: Uuid, target_version: i32) -> Result<(), RollbackError>`.

#### Step 1: Add the `chrono` dependency

- [ ] Edit `backend/crates/nomi-agent-core/Cargo.toml` — add this line under `[dependencies]` (nothing in this crate currently imports `chrono` directly, only via sqlx's feature flag, which is not enough to write `DateTime<Utc>` in this crate's own public types):

```toml
chrono = { version = "0.4", features = ["serde"] }
```

#### Step 2: Update the 3 existing direct call sites for the widened `set_personality`

- [ ] Edit `backend/crates/nomi-agent-core/tests/personality.rs` — wrap `session_id`/`agent_session_id` in `Some(...)` at these 3 call sites (lines 57, 79, 80 in the current file):

```rust
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic and blunt.").await.unwrap();
```
(and the two equivalent calls in `second_change_updates_the_same_row_and_records_the_prior_value_as_old_description`, with their own description strings — same `Some(session_id), Some(agent_session_id)` wrapping, nothing else changes on those lines).

This is the only crate in the workspace with a direct (non-`nomi-agent-personality`) call site — confirmed via `grep -rn "set_personality(&mut conn" $(find . -name "*.rs" -not -path "*/target/*")` before writing this plan.

- [ ] Verify the crate still compiles (it will fail until Step 3 widens the function itself — this step alone is expected to leave the crate non-compiling; that's fine, proceed directly to Step 3):

```bash
cd backend && cargo build -p nomi-agent-core --tests 2>&1 | tail -20
```

Expected: a type error on `set_personality`'s call sites (`expected Option<Uuid>, found Uuid`) or, if Step 3 hasn't landed yet, `expected Uuid, found Option<Uuid>` — either way confirms the two sides are currently mismatched. This is a sanity check that the grep in this step's brief was exhaustive, not a gate to pass before continuing.

#### Step 3: Widen `set_personality` and add versioning to it

- [ ] Edit `backend/crates/nomi-agent-core/src/personality.rs` — replace the whole file with:

```rust
use chrono::{DateTime, Utc};
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

/// Upserts `user_personality`, appends a new row to `user_personality_versions`, and records a
/// `PersonalityChanged` `agent_events` row with `{old_description, new_description, new_version}`
/// (`old_description` is `null` on a user's first-ever change) — all in one transaction.
/// `session_id`/`agent_session_id` are `None` for changes with no chat turn behind them (e.g. a
/// rollback triggered from the web UI) — `agent_events.session_id`/`.agent_session_id` are
/// already nullable columns for exactly this reason (see chitchat's own NULL-sentinel handling
/// in `nomi-turn`). The literal `"personality"` agent_type below is `nomi-agent-personality`'s
/// `PERSONALITY_AGENT_TYPE`, duplicated here because `nomi-agent-core` sits below
/// `nomi-agent-personality` in the dependency graph (the same convention every other agent_type
/// string in this crate already follows).
pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError> {
    let mut tx = conn.begin().await?;

    let old_description: Option<String> =
        sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

    let max_version: Option<i32> =
        sqlx::query_scalar("SELECT MAX(version) FROM user_personality_versions WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;
    let new_version = max_version.unwrap_or(0) + 1;

    sqlx::query("INSERT INTO user_personality_versions (user_id, version, description) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(new_version)
        .bind(new_description)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO user_personality (user_id, description, current_version, updated_at) \
         VALUES ($1, $2, $3, now()) \
         ON CONFLICT (user_id) DO UPDATE SET description = $2, current_version = $3, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_description)
    .bind(new_version)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) \
         VALUES ($1, $2, 'personality', 'PersonalityChanged', $3)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(serde_json::json!({
        "old_description": old_description,
        "new_description": new_description,
        "new_version": new_version,
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersonalityVersion {
    pub version: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

/// Returns up to `limit` versions for `user_id`, most recent first. `is_current` is computed in
/// SQL via a join against `user_personality.current_version` — one query, no per-row lookup.
pub async fn list_versions(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<PersonalityVersion>, TurnError> {
    let rows: Vec<(i32, String, DateTime<Utc>, bool)> = sqlx::query_as(
        "SELECT v.version, v.description, v.created_at, v.version = up.current_version AS is_current \
         FROM user_personality_versions v \
         JOIN user_personality up ON up.user_id = v.user_id \
         WHERE v.user_id = $1 \
         ORDER BY v.version DESC \
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(version, description, created_at, is_current)| PersonalityVersion {
            version,
            description,
            created_at,
            is_current,
        })
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("personality version {0} not found for this user")]
    VersionNotFound(i32),
    #[error(transparent)]
    SetPersonality(#[from] TurnError),
}

/// Restores `user_id`'s personality to `target_version`'s description by creating a *new*
/// version with that description — never mutates or deletes `target_version`'s own row. The
/// lookup runs outside `set_personality`'s own transaction: version rows are immutable once
/// written, so a concurrent write racing this read can only mean the description copied forward
/// is still a genuine historical value, never a torn read.
pub async fn rollback_to_version(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    target_version: i32,
) -> Result<(), RollbackError> {
    let target_description: Option<String> = sqlx::query_scalar(
        "SELECT description FROM user_personality_versions WHERE user_id = $1 AND version = $2",
    )
    .bind(user_id)
    .bind(target_version)
    .fetch_optional(&mut **conn)
    .await?;

    let target_description = target_description.ok_or(RollbackError::VersionNotFound(target_version))?;

    set_personality(conn, session_id, agent_session_id, user_id, &target_description).await?;

    Ok(())
}
```

#### Step 4: Write the new tests

- [ ] Edit `backend/crates/nomi-agent-core/tests/personality.rs` — update the `use` line at the top to:

```rust
use nomi_agent_core::personality::{
    get_current_personality, list_versions, rollback_to_version, set_personality, RollbackError,
};
```

Then add these tests to the end of the file:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_returns_newest_first_with_the_current_one_marked(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v1 desc").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v2 desc").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v3 desc").await.unwrap();

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();

    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version, 3);
    assert_eq!(versions[0].description, "v3 desc");
    assert!(versions[0].is_current);
    assert_eq!(versions[1].version, 2);
    assert!(!versions[1].is_current);
    assert_eq!(versions[2].version, 1);
    assert!(!versions[2].is_current);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_respects_the_limit(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    for i in 1..=5 {
        set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, &format!("v{i}")).await.unwrap();
    }

    let versions = list_versions(&mut conn, user_id, 2).await.unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, 5);
    assert_eq!(versions[1].version, 4);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_returns_empty_for_a_user_with_no_personality_set(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();
    assert!(versions.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_creates_a_new_version_with_the_targets_description_rather_than_mutating_history(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic.").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be warm.").await.unwrap();

    rollback_to_version(&mut conn, Some(session_id), Some(agent_session_id), user_id, 1).await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic.".to_string()));

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version, 3);
    assert_eq!(versions[0].description, "Be sarcastic.");
    assert!(versions[0].is_current);
    // Version 1's own row is untouched — history is append-only.
    assert_eq!(versions[2].version, 1);
    assert_eq!(versions[2].description, "Be sarcastic.");
    assert!(!versions[2].is_current);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_to_an_unknown_version_returns_version_not_found(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic.").await.unwrap();

    let result = rollback_to_version(&mut conn, Some(session_id), Some(agent_session_id), user_id, 99).await;
    assert!(matches!(result, Err(RollbackError::VersionNotFound(99))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_with_no_turn_context_records_a_null_session_and_agent_session(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, None, None, user_id, "Be sarcastic.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic.".to_string()));

    let (session_id_col, agent_session_id_col): (Option<Uuid>, Option<Uuid>) = sqlx::query_as(
        "SELECT session_id, agent_session_id FROM agent_events \
         WHERE event_type = 'PersonalityChanged' AND payload->>'new_description' = 'Be sarcastic.'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(session_id_col, None);
    assert_eq!(agent_session_id_col, None);
}
```

- [ ] Run the crate's full test suite:

```bash
cd backend && cargo test -p nomi-agent-core
```

Expected: all tests PASS (the 2 pre-existing tests updated in Step 2, plus the 6 new ones above, plus everything else in the crate untouched).

- [ ] Commit:

```bash
git add backend/crates/nomi-agent-core/Cargo.toml backend/crates/nomi-agent-core/src/personality.rs \
        backend/crates/nomi-agent-core/tests/personality.rs
git commit -m "feat: add personality version history and rollback"
```

---

### Task 3: `PersonalityAgent` gets `list_personality_versions` and `rollback_personality`

**Files:**
- Modify: `backend/crates/nomi-agent-personality/src/lib.rs`
- Modify: `backend/crates/nomi-agent-personality/tests/personality_agent.rs`

**Interfaces:**
- Consumes: `nomi_agent_core::personality::{list_versions, rollback_to_version, RollbackError}` (Task 2).
- Produces (used by Task 4): two new tools on `PersonalityAgent`, `"list_personality_versions"` and `"rollback_personality"`.

#### Step 1: Write the failing tests

- [ ] Edit `backend/crates/nomi-agent-personality/tests/personality_agent.rs` — add these tests to the end of the file:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn list_personality_versions_reports_history_with_the_current_one_marked(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic."}),
        )
        .await
        .unwrap();
    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be warm."}),
        )
        .await
        .unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "list_personality_versions", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.contains("v2"));
    assert!(result.contains("Be warm."));
    assert!(result.contains("(current)"));
    assert!(result.contains("v1"));
    assert!(result.contains("Be sarcastic."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_personality_versions_with_no_history_says_so(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "list_personality_versions", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result, "No personality history yet.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_restores_an_old_version(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic."}),
        )
        .await
        .unwrap();
    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be warm."}),
        )
        .await
        .unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({"version": 1}))
        .await
        .unwrap();

    assert!(result.contains("version 1"));

    let stored: String = sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "Be sarcastic.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_to_an_unknown_version_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({"version": 99}))
        .await;

    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_rejects_a_missing_version(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
```

- [ ] Run the new tests to verify they fail (the tools don't exist yet):

```bash
cd backend && cargo test -p nomi-agent-personality --test personality_agent list_personality_versions
cd backend && cargo test -p nomi-agent-personality --test personality_agent rollback_personality
```

Expected: FAIL — `"unknown tool: list_personality_versions"` / `"unknown tool: rollback_personality"` errors surfaced as `Err(...)`, so `.unwrap()` panics.

#### Step 2: Implement the two new tools

- [ ] Edit `backend/crates/nomi-agent-personality/src/lib.rs` — replace the whole file with:

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
     set. If the user wants to see their past personalities or go back to an earlier one, call \
     list_personality_versions to show them the options, then rollback_personality with the \
     version they choose. Never guess a version number without listing first unless the user \
     gives one explicitly. When you're done, call complete_task.";

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
        vec![
            ToolDefinition {
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
            },
            ToolDefinition {
                name: "list_personality_versions".to_string(),
                description: "List your past personality descriptions, most recent first, so you can decide what to roll back to.".to_string(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            ToolDefinition {
                name: "rollback_personality".to_string(),
                description: "Roll back to a previous personality version. This creates a new version with that version's description rather than deleting anything.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "version": {
                            "type": "integer",
                            "description": "The version number to restore, from list_personality_versions."
                        }
                    },
                    "required": ["version"]
                }),
            },
        ]
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
            "list_personality_versions" => list_personality_versions(conn, user_id).await,
            "rollback_personality" => rollback_personality(conn, session_id, agent_session_id, user_id, input).await,
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

    nomi_agent_core::personality::set_personality(conn, Some(session_id), Some(agent_session_id), user_id, description)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Personality updated to: {description}"))
}

async fn list_personality_versions(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Result<String, String> {
    let versions = nomi_agent_core::personality::list_versions(conn, user_id, 10)
        .await
        .map_err(|e| e.to_string())?;

    if versions.is_empty() {
        return Ok("No personality history yet.".to_string());
    }

    let lines: Vec<String> = versions
        .iter()
        .map(|v| {
            let marker = if v.is_current { " (current)" } else { "" };
            format!("v{}{}: {}", v.version, marker, v.description)
        })
        .collect();

    Ok(lines.join("\n"))
}

async fn rollback_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let version = input
        .get("version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "version is required".to_string())? as i32;

    nomi_agent_core::personality::rollback_to_version(conn, Some(session_id), Some(agent_session_id), user_id, version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Rolled back to version {version}."))
}
```

- [ ] Run the new tests to verify they pass:

```bash
cd backend && cargo test -p nomi-agent-personality
```

Expected: all tests PASS (4 pre-existing + 5 new).

- [ ] Commit:

```bash
git add backend/crates/nomi-agent-personality/src/lib.rs backend/crates/nomi-agent-personality/tests/personality_agent.rs
git commit -m "feat: give PersonalityAgent list_personality_versions and rollback_personality tools"
```

---

### Task 4: End-to-end test — chat-driven rollback folds into the next chitchat reply

**Files:**
- Modify: `backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs`

**Interfaces:**
- Consumes: `PersonalityAgent`'s new tools (Task 3), `handle_inbound_message` (existing, unchanged signature).

- [ ] **Step 1: Write the test**

Add this test to the end of `backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs`:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_driven_rollback_is_folded_into_the_next_chitchat_reply(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Turn 1: set an initial personality (v1).
    let set_v1_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Be sarcastic and blunt."})),
        tool_use_response("t2", "complete_task", serde_json::json!({"status": "completed", "summary": "Done, sarcastic now."})),
    ]);
    handle_inbound_message(&pool, &set_v1_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "be sarcastic", None)
        .await
        .unwrap();

    // Turn 2: change it again (v2), so there's something to roll back from.
    let set_v2_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t3", "set_personality", serde_json::json!({"description": "Be warm and encouraging."})),
        tool_use_response("t4", "complete_task", serde_json::json!({"status": "completed", "summary": "Done, warm now."})),
    ]);
    handle_inbound_message(&pool, &set_v2_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "actually be warm", None)
        .await
        .unwrap();

    // Turn 3: roll back to v1 via chat — the agent lists versions first, then rolls back.
    let rollback_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t5", "list_personality_versions", serde_json::json!({})),
        tool_use_response("t6", "rollback_personality", serde_json::json!({"version": 1})),
        tool_use_response("t7", "complete_task", serde_json::json!({"status": "completed", "summary": "Rolled back to sarcastic."})),
    ]);
    let outcome = handle_inbound_message(
        &pool,
        &rollback_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "go back to how you were before",
        None,
    )
    .await
    .unwrap();
    assert_eq!(outcome.reply, "Rolled back to sarcastic.");

    // Turn 4: a plain chitchat message should now carry v1's personality again.
    let chitchat_provider =
        FakeLlmProvider::sequence(vec![canned_response("chitchat"), canned_response("Yeah, whatever."), canned_response("NONE")]);
    handle_inbound_message(&pool, &chitchat_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hey", None)
        .await
        .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}
```

This reuses `tool_use_response` and `PersonalityAgent`, both already added to this file by the prior personality-customization plan's Task 6 — no new imports needed.

- [ ] **Step 2: Run the test**

```bash
cd backend && cargo test -p nomi-turn --test turn_handle_inbound_message a_chat_driven_rollback
```

Expected: PASS. If it fails, the most likely cause is a mismatched number of `FakeLlmProvider::sequence` entries against the actual number of LLM calls the turn makes — count classification (1) + each tool-use turn (1 per tool call, including `list_personality_versions` and `rollback_personality`, since neither is `complete_task`) + the final `complete_task` call.

- [ ] **Step 3: Run the whole workspace's test suite**

```bash
cd backend && cargo test --workspace
```

Expected: all tests PASS.

- [ ] **Step 4: Commit**

```bash
git add backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs
git commit -m "test: prove a chat-driven personality rollback folds into the next chitchat reply"
```

---

### Task 5: HTTP routes — `GET /api/personality/history`, `POST /api/personality/rollback`

**Files:**
- Create: `backend/crates/nomi-server/src/routes/personality.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs`
- Modify: `backend/crates/nomi-server/src/app.rs`
- Create: `backend/crates/nomi-server/tests/personality_routes.rs`

**Interfaces:**
- Consumes: `nomi_agent_core::personality::{list_versions, rollback_to_version, RollbackError}` (Task 2) — both already available since `nomi-agent-core` and `chrono` are already `nomi-server` dependencies (confirmed in its existing `Cargo.toml`; no manifest change needed for this task).
- Produces (used by Task 6): `GET /api/personality/history` → `{"versions": [{"version": int, "description": string, "created_at": string, "is_current": bool}, ...]}`; `POST /api/personality/rollback` with body `{"version": int}` → `204 No Content` on success, `404` if the version doesn't exist.

#### Step 1: Implement the routes

- [ ] Create `backend/crates/nomi-server/src/routes/personality.rs`:

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use nomi_agent_core::personality::{list_versions, rollback_to_version, RollbackError};
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct PersonalityVersionItem {
    pub version: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Serialize)]
pub struct PersonalityHistoryResponse {
    pub versions: Vec<PersonalityVersionItem>,
}

pub async fn get_personality_history(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<PersonalityHistoryResponse>, (StatusCode, &'static str)> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;

    let versions = list_versions(&mut conn, claims.sub, 50)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load personality history"))?
        .into_iter()
        .map(|v| PersonalityVersionItem {
            version: v.version,
            description: v.description,
            created_at: v.created_at,
            is_current: v.is_current,
        })
        .collect();

    Ok(Json(PersonalityHistoryResponse { versions }))
}

#[derive(Deserialize)]
pub struct RollbackPersonalityRequest {
    pub version: i32,
}

pub async fn rollback_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<RollbackPersonalityRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;

    rollback_to_version(&mut conn, None, None, claims.sub, req.version)
        .await
        .map_err(|e| match e {
            RollbackError::VersionNotFound(_) => (StatusCode::NOT_FOUND, "version not found"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "failed to roll back personality"),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] Edit `backend/crates/nomi-server/src/routes/mod.rs` — add the module:

```rust
pub mod auth;
pub mod llm_models;
pub mod personality;
pub mod sessions;
pub mod settings;
```

- [ ] Edit `backend/crates/nomi-server/src/app.rs` — add the import (alongside the existing route-module imports) and two routes (alongside the existing `.route(...)` chain, order doesn't matter — placed here after the `llm` routes for locality):

```rust
use crate::routes::personality as personality_routes;
```

```rust
        .route("/api/llm/models", get(llm_models_routes::get_user_models))
        .route("/api/llm/selection", put(llm_models_routes::put_user_selection))
        .route(
            "/api/personality/history",
            get(personality_routes::get_personality_history),
        )
        .route("/api/personality/rollback", post(personality_routes::rollback_personality))
```

#### Step 2: Write the tests

- [ ] Create `backend/crates/nomi-server/tests/personality_routes.rs`:

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

async fn user_id_for_email(pool: &PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn history_is_empty_for_a_user_with_no_personality_set(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user1@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["versions"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_creates_a_new_current_version_and_history_reflects_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user2@example.com").await;

    let user_id = user_id_for_email(&pool, "user2@example.com").await;
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user_id, "Be sarcastic.").await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user_id, "Be warm.").await.unwrap();
    drop(conn);

    let (status, _) =
        json_request(router.clone(), "POST", "/api/personality/rollback", json!({"version": 1}), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let versions = body["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0]["version"], 3);
    assert_eq!(versions[0]["description"], "Be sarcastic.");
    assert_eq!(versions[0]["is_current"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rolling_back_to_an_unknown_version_returns_404(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user3@example.com").await;

    let (status, _) = json_request(router, "POST", "/api/personality/rollback", json!({"version": 99}), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_users_history_never_leaks_into_the_first_users_response(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token2 = {
        register_and_login(router.clone(), "user4@example.com").await;
        register_and_login(router.clone(), "user5@example.com").await
    };

    let user1_id = user_id_for_email(&pool, "user4@example.com").await;
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user1_id, "User 1's personality.").await.unwrap();
    drop(conn);

    let (_, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token2)).await;
    assert_eq!(body["versions"].as_array().unwrap().len(), 0);
}
```

- [ ] Run the new tests:

```bash
cd backend && cargo test -p nomi-server --test personality_routes
```

Expected: all 4 tests PASS.

- [ ] Run `nomi-server`'s full test suite:

```bash
cd backend && cargo test -p nomi-server
```

Expected: all tests PASS (no route ordering or state conflicts with the existing routes).

- [ ] Commit:

```bash
git add backend/crates/nomi-server/src/routes/personality.rs backend/crates/nomi-server/src/routes/mod.rs \
        backend/crates/nomi-server/src/app.rs backend/crates/nomi-server/tests/personality_routes.rs
git commit -m "feat: add personality history and rollback HTTP routes"
```

---

### Task 6: Frontend — personality history panel in the chat page

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/personality/history`, `POST /api/personality/rollback` (Task 5).

#### Step 1: Add the frontend types

- [ ] Edit `frontend/src/lib/types.ts` — add at the end of the file:

```ts
export interface PersonalityVersion {
	version: number;
	description: string;
	created_at: string;
	is_current: boolean;
}

export interface PersonalityHistoryResponse {
	versions: PersonalityVersion[];
}
```

#### Step 2: Load history and add the rollback action

- [ ] Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` — update the `import type` line and `load` function, and add a new `restorePersonality` action. Full file:

```ts
import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LlmModelsResponse, MessageItem, PersonalityHistoryResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, 'Could not load this chat.');
	}

	const { messages } = (await response.json()) as { messages: MessageItem[] };

	const modelsResponse = await apiFetch(fetch, cookies, '/api/llm/models');
	const models: LlmModelsResponse = modelsResponse.ok
		? ((await modelsResponse.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null };

	const personalityResponse = await apiFetch(fetch, cookies, '/api/personality/history');
	const personality: PersonalityHistoryResponse = personalityResponse.ok
		? ((await personalityResponse.json()) as PersonalityHistoryResponse)
		: { versions: [] };

	return { messages, models, personality };
};

export const actions: Actions = {
	// Named (not `default`) because this actions object also has selectAdminModel and
	// selectCustomModel — SvelteKit forbids mixing a `default` action with named actions in the
	// same file (throws "When using named actions, the default action cannot be used" at request
	// time), so the message-send form below must target this action explicitly.
	sendMessage: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: 'Message cannot be empty.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message } = (await response.json()) as { user_message: MessageItem };

		return { user_message };
	},

	selectAdminModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const adminModelId = data.get('admin_model_id');

		if (typeof adminModelId !== 'string') {
			return fail(400, { modelError: 'Invalid model selection.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({ kind: 'admin', admin_model_id: adminModelId }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || 'Failed to select model.' });
		}

		return { modelSelected: true };
	},

	selectCustomModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const label = data.get('label');
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (
			typeof label !== 'string' ||
			typeof provider !== 'string' ||
			typeof modelId !== 'string' ||
			typeof apiKey !== 'string' ||
			!label.trim() ||
			!provider.trim()
		) {
			return fail(400, { modelError: 'Label and provider are required.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({
				kind: 'custom',
				label,
				provider,
				model_id: modelId,
				api_key: apiKey,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || 'Failed to validate and save this model.' });
		}

		return { modelSelected: true };
	},

	restorePersonality: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const version = data.get('version');

		if (typeof version !== 'string' || !version.trim()) {
			return fail(400, { personalityError: 'Invalid version.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/personality/rollback', {
			method: 'POST',
			body: JSON.stringify({ version: Number(version) }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { personalityError: message || 'Failed to roll back personality.' });
		}

		return { personalityRestored: true };
	},
};
```

#### Step 3: Add the panel to the chat page

- [ ] Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`:

Add `personalityPanelOpen` alongside the other `$state` declarations near the top of the `<script>` block:

```ts
	let modelPickerOpen = $state(false);
	let personalityPanelOpen = $state(false);
	let showCustomForm = $state(false);
```

Change the toolbar row from a single `<div class="relative">` to two side-by-side ones — replace:

```svelte
		<div class="mb-2 flex justify-end">
			<div class="relative">
				<Button type="button" variant="outlined" onclick={() => (modelPickerOpen = !modelPickerOpen)}>
```

with:

```svelte
		<div class="mb-2 flex justify-end gap-2">
			<div class="relative">
				<Button type="button" variant="outlined" onclick={() => (personalityPanelOpen = !personalityPanelOpen)}>
					Personality
				</Button>
				{#if personalityPanelOpen}
					<div
						class="absolute right-0 bottom-full mb-2 w-80 p-3"
						style="background: var(--md-sys-color-surface-container-high); border-radius: var(--md-sys-shape-corner-extra-large); box-shadow: var(--md-sys-elevation-shadow-level3)"
					>
						{#if form?.personalityError}
							<p class="md-body-small mb-2" style="color: var(--md-sys-color-error)">{form.personalityError}</p>
						{/if}
						<p class="md-label-medium mb-1" style="color: var(--md-sys-color-on-surface-variant)">Personality history</p>
						{#if data.personality.versions.length === 0}
							<p class="md-body-medium px-2 py-1" style="color: var(--md-sys-color-on-surface-variant)">
								You haven't set a personality yet — just ask nomi to change it.
							</p>
						{:else}
							{#each data.personality.versions as version (version.version)}
								<div class="m3-personality-item" class:m3-personality-item--current={version.is_current}>
									<div class="min-w-0 flex-1">
										<p class="md-body-medium truncate" style="color: var(--md-sys-color-on-surface)">{version.description}</p>
										<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">
											v{version.version} · {new Date(version.created_at).toLocaleString()}
										</p>
									</div>
									{#if !version.is_current}
										<form
											method="POST"
											action="?/restorePersonality"
											use:enhance={() => {
												return async ({ update }) => {
													await update();
													personalityPanelOpen = false;
												};
											}}
										>
											<input type="hidden" name="version" value={version.version} />
											<Button type="submit" variant="text">Restore</Button>
										</form>
									{/if}
								</div>
							{/each}
						{/if}
					</div>
				{/if}
			</div>
			<div class="relative">
				<Button type="button" variant="outlined" onclick={() => (modelPickerOpen = !modelPickerOpen)}>
```

(The rest of the model-picker block — its own `{#if modelPickerOpen}` panel, closing `</div>` for this second `.relative` div, and the outer toolbar `</div>` — is unchanged; only the opening `<div class="mb-2 flex justify-end">` becomes `gap-2` and the new Personality button+panel block is inserted before the existing model-picker `<div class="relative">`.)

Add the new item styles to the `<style>` block at the bottom of the file, after `.m3-picker-input:focus`'s closing brace:

```css
	.m3-personality-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px;
		border-radius: var(--md-sys-shape-corner-small);
	}
	.m3-personality-item--current {
		background: var(--md-sys-color-secondary-container);
	}
```

(A new class rather than reusing `.m3-picker-item`: that class is `display: block` for the model list's plain buttons, while this list needs a flex row layout for the description+restore-button pair — reusing it would fight the existing rule's specificity instead of cleanly composing with it.)

#### Step 4: Verify

- [ ] Run svelte-check and the TypeScript check:

```bash
cd frontend && npm run check
```

Expected: no errors. `data.personality` is typed automatically from `load`'s return type via SvelteKit's generated `PageData` — no manual type annotation needed in the component.

- [ ] Self-review: re-read the diff and confirm — the model-picker's existing behavior (open/close, model selection, custom-key form) is completely unchanged; the new Personality button/panel only adds to the toolbar row, doesn't restructure it; the empty-state message renders when `data.personality.versions` is empty; the current version never shows a "Restore" button.

- [ ] Commit:

```bash
git add frontend/src/lib/types.ts frontend/src/routes/\(app\)/chat/\[sessionId\]/+page.server.ts \
        frontend/src/routes/\(app\)/chat/\[sessionId\]/+page.svelte
git commit -m "feat: add a personality history panel with rollback to the chat page"
```
