# Scheduled Reminders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a user ask Nomi to schedule a reminder ("remind me tomorrow at 11 to take a bath"),
store it durably with a target agent and optional recurrence, and have a background worker fire it
at the right time, delivering the result as a chat message (with a pluggable, stubbed-for-now push
notification seam alongside it).

**Architecture:** Three new engine-level tools (`create_reminder`, `list_reminders`,
`cancel_reminder`), gated behind a new `supports_reminders()` opt-in on `SubAgent` and enabled on
`ChitchatAgent`, read/write a new `scheduled_jobs` table directly (no delegation round-trip). A new
`scheduler_worker.rs` background loop in `nomi-server`, modeled on the existing
`delegation_worker.rs`, polls for due jobs, runs a real agent turn per job, and delivers the result
as a normal chat message — plus a best-effort call through a new `NotificationDelivery` trait
(`LogOnlyDelivery` for v1).

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, `tokio::spawn` background worker embedded in the main server process.

**Spec:** `docs/superpowers/specs/2026-09-20-scheduled-reminders-design.md`

## Global Constraints

- New engine tools (`create_reminder`, `list_reminders`, `cancel_reminder`) are never
  permission-gated — added to `is_gateable`'s exclusion list alongside `WRITE_PLAN_TOOL_NAME`.
- Only `ChitchatAgent` opts into `supports_reminders()` in this plan; the trait flag defaults to
  `false` for every other agent.
- Only three recurrence cadences exist: `daily`, `weekly`, `monthly` — no cron/RRULE syntax.
- The in-app chat message delivery (DB insert + MQTT `MessageCreated` publish) always happens on a
  successful fire, regardless of `NotificationDelivery`'s outcome.
- `user_preferences.timezone` defaults to `'UTC'` — every existing row stays valid, no backfill.
- All new backend tests use `#[sqlx::test(migrations = "../../migrations")]`, matching every
  existing test in `nomi-agent-core/tests/engine.rs`.

---

## File Structure

**New files:**
- `backend/migrations/0028_scheduled_jobs.sql` — table + index + `user_preferences.timezone` column.
- `backend/crates/nomi-agent-core/src/notification.rs` — `NotificationDelivery` trait + `LogOnlyDelivery`.
- `backend/crates/nomi-agent-core/src/reminders.rs` — `create_reminder`/`list_reminders`/`cancel_reminder`/`get_user_timezone`, called from `engine.rs`'s dispatch.
- `backend/crates/nomi-agent-core/tests/scheduled_jobs_schema.rs` — schema-level contract test.
- `backend/crates/nomi-agent-core/tests/reminders.rs` — direct tests of the `reminders.rs` functions.
- `backend/crates/nomi-server/src/scheduler_worker.rs` — `next_occurrence`, `claim_next`, `process_claimed_job`, `run`.
- `backend/crates/nomi-server/tests/scheduler_worker.rs` — tests for the above.

**Modified files:**
- `backend/crates/nomi-agent-core/src/subagent.rs` — new `supports_reminders()` default method.
- `backend/crates/nomi-agent-core/src/engine.rs` — tool name consts, tool definitions, dispatch branches, `is_gateable`, system-prompt time/timezone injection, tool registration.
- `backend/crates/nomi-agent-core/src/lib.rs` — export new module items.
- `backend/crates/nomi-agent-core/Cargo.toml` — add `chrono-tz`.
- `backend/crates/nomi-agent-core/tests/engine.rs` — reuse existing `TestAgent`/`seed_session`/`seed_agent_session` helpers for the new end-to-end tool tests.
- `backend/crates/nomi-agent-chitchat/src/lib.rs` — `supports_reminders() -> true`.
- `backend/crates/nomi-server/src/lib.rs` — `pub mod scheduler_worker;`.
- `backend/crates/nomi-server/src/main.rs` — construct `LogOnlyDelivery`, spawn `scheduler_worker::run`.
- `backend/crates/nomi-server/src/routes/profile.rs` — `timezone` in `PreferencesResponse`/`UpdatePreferencesRequest`, `get_preferences`, `put_preferences`.
- `backend/crates/nomi-server/Cargo.toml` — add `chrono-tz`.
- `backend/crates/nomi-server/tests/*` (whichever file covers `/api/preferences` today — see Task 8) — timezone coverage.
- `frontend/src/lib/types.ts` — `timezone` on `Preferences`.
- `frontend/src/routes/(app)/preferences/+page.server.ts` — `updateTimezone` action.
- `frontend/src/routes/(app)/preferences/+page.svelte` — timezone section (auto-detect + manual select).

---

### Task 1: `scheduled_jobs` table and `user_preferences.timezone` column

**Files:**
- Create: `backend/migrations/0028_scheduled_jobs.sql`
- Test: `backend/crates/nomi-agent-core/tests/scheduled_jobs_schema.rs`

**Interfaces:**
- Produces: the `scheduled_jobs` table (columns: `id`, `session_id`, `user_id`,
  `created_by_agent_type`, `target_agent_type`, `label`, `prompt`, `run_at`, `recurrence`,
  `recurrence_weekday`, `recurrence_day_of_month`, `status`, `claimed_at`, `last_fired_at`,
  `created_at`, `cancelled_at`) and `user_preferences.timezone TEXT NOT NULL DEFAULT 'UTC'`, both
  consumed by every later task in this plan.

- [ ] **Step 1: Write the migration**

```sql
-- backend/migrations/0028_scheduled_jobs.sql
CREATE TABLE scheduled_jobs (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id               UUID NOT NULL REFERENCES sessions(id),
    user_id                  UUID NOT NULL REFERENCES users(id),
    created_by_agent_type    TEXT NOT NULL,
    target_agent_type        TEXT NOT NULL,
    label                    TEXT NOT NULL,
    prompt                   TEXT NOT NULL,
    run_at                   TIMESTAMPTZ NOT NULL,
    recurrence               TEXT CHECK (recurrence IN ('daily', 'weekly', 'monthly')),
    recurrence_weekday       SMALLINT CHECK (recurrence_weekday BETWEEN 0 AND 6),
    recurrence_day_of_month  SMALLINT CHECK (recurrence_day_of_month BETWEEN 1 AND 31),
    status                   TEXT NOT NULL DEFAULT 'active'
                                 CHECK (status IN ('active', 'cancelled', 'completed')),
    claimed_at               TIMESTAMPTZ,
    last_fired_at            TIMESTAMPTZ,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    cancelled_at             TIMESTAMPTZ
);

CREATE INDEX scheduled_jobs_due_idx ON scheduled_jobs (run_at)
    WHERE status = 'active' AND claimed_at IS NULL;

ALTER TABLE user_preferences ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
```

- [ ] **Step 2: Write the failing schema test**

```rust
// backend/crates/nomi-agent-core/tests/scheduled_jobs_schema.rs
use sqlx::PgPool;
use uuid::Uuid;

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_scheduled_job_defaults_to_active_status(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;

    let status: String = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'take a bath', 'Remind the user to take a bath.', now() + interval '1 day') \
         RETURNING status",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(status, "active");
}

#[sqlx::test(migrations = "../../migrations")]
async fn recurrence_must_be_one_of_the_three_known_cadences(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;

    let result = sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, recurrence) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'x', 'y', now(), 'yearly')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await;

    assert!(result.is_err(), "'yearly' is not one of the allowed recurrence values");
}

#[sqlx::test(migrations = "../../migrations")]
async fn user_preferences_timezone_defaults_to_utc(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let timezone: String = sqlx::query_scalar(
        "INSERT INTO user_preferences (user_id) VALUES ($1) RETURNING timezone",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(timezone, "UTC");
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test -p nomi-agent-core --test scheduled_jobs_schema`
Expected: FAIL — `scheduled_jobs` does not exist / `user_preferences` has no `timezone` column.

- [ ] **Step 4: Run migrations and re-run the tests**

The migration file from Step 1 already exists on disk; `#[sqlx::test]` applies every migration
under `../../migrations` automatically per test, so no separate `sqlx migrate run` is needed for
the test suite itself. Run: `cd backend && cargo test -p nomi-agent-core --test scheduled_jobs_schema`
Expected: PASS (all 3 tests).

- [ ] **Step 5: Commit**

```bash
git add backend/migrations/0028_scheduled_jobs.sql backend/crates/nomi-agent-core/tests/scheduled_jobs_schema.rs
git commit -m "feat: add scheduled_jobs table and user_preferences.timezone column"
```

---

### Task 2: `NotificationDelivery` trait and `LogOnlyDelivery`

**Files:**
- Create: `backend/crates/nomi-agent-core/src/notification.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `pub trait NotificationDelivery { async fn deliver(&self, user_id: Uuid, message: &str) -> Result<(), String>; }` and `pub struct LogOnlyDelivery;` implementing it — consumed by Task 7's `scheduler_worker.rs`.

- [ ] **Step 1: Write the failing test**

```rust
// bottom of backend/crates/nomi-agent-core/src/notification.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_only_delivery_always_succeeds() {
        let delivery = LogOnlyDelivery;
        let result = delivery.deliver(Uuid::new_v4(), "test message").await;
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p nomi-agent-core notification::tests`
Expected: FAIL with a compile error — `notification` module / `NotificationDelivery` / `LogOnlyDelivery` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/crates/nomi-agent-core/src/notification.rs
use async_trait::async_trait;
use uuid::Uuid;

/// Seam for pushing a fired reminder's result somewhere beyond the in-app chat message that
/// `scheduler_worker.rs` always posts unconditionally. `LogOnlyDelivery` is the only concrete
/// implementation today — real Telegram/WhatsApp/web-push integrations are future work that
/// implement this same trait with no changes needed to the scheduler itself.
#[async_trait]
pub trait NotificationDelivery: Send + Sync {
    async fn deliver(&self, user_id: Uuid, message: &str) -> Result<(), String>;
}

pub struct LogOnlyDelivery;

#[async_trait]
impl NotificationDelivery for LogOnlyDelivery {
    async fn deliver(&self, user_id: Uuid, message: &str) -> Result<(), String> {
        tracing::info!(%user_id, %message, "notification delivery not yet configured — logging only");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_only_delivery_always_succeeds() {
        let delivery = LogOnlyDelivery;
        let result = delivery.deliver(Uuid::new_v4(), "test message").await;
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 4: Wire the module into `lib.rs`**

In `backend/crates/nomi-agent-core/src/lib.rs`, add `pub mod notification;` alongside the other
`pub mod` lines, and add `NotificationDelivery, LogOnlyDelivery` to the `pub use` re-export list
(the file currently re-exports from `content_block`, `dynamic_agent`, `engine`, `error`,
`registry`, `subagent`, `tool_catalog` — add a new line: `pub use notification::{LogOnlyDelivery, NotificationDelivery};`).

- [ ] **Step 5: Run test to verify it passes**

Run: `cd backend && cargo test -p nomi-agent-core notification::tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-agent-core/src/notification.rs backend/crates/nomi-agent-core/src/lib.rs
git commit -m "feat: add NotificationDelivery trait and log-only stub implementation"
```

---

### Task 3: `supports_reminders()` flag and the `reminders.rs` module

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs`
- Create: `backend/crates/nomi-agent-core/src/reminders.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs`
- Test: `backend/crates/nomi-agent-core/tests/reminders.rs`

**Interfaces:**
- Consumes: the `scheduled_jobs` table and `user_preferences.timezone` column from Task 1.
- Produces:
  - `SubAgent::supports_reminders(&self) -> bool` (default `false`), consumed by Task 4.
  - `pub async fn get_user_timezone(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> String`
  - `pub async fn create_reminder(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, created_by_agent_type: &str, input: &serde_json::Value) -> Result<String, String>`
  - `pub async fn list_reminders(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Result<String, String>`
  - `pub async fn cancel_reminder(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: &serde_json::Value) -> Result<String, String>`

  All four consumed by Task 4's `engine.rs` dispatch branches.

- [ ] **Step 1: Add `supports_reminders()` to the `SubAgent` trait**

In `backend/crates/nomi-agent-core/src/subagent.rs`, add this default method right after
`supports_plans()`:

```rust
    /// When true, run_agent_turn gives this agent the engine-level `create_reminder`,
    /// `list_reminders`, and `cancel_reminder` tools for scheduling deferred/recurring tasks —
    /// see docs/superpowers/specs/2026-09-20-scheduled-reminders-design.md. Off by default, same
    /// reasoning as supports_plans(): only the conversational front door needs this.
    fn supports_reminders(&self) -> bool {
        false
    }
```

- [ ] **Step 2: Write the failing tests for `reminders.rs`**

```rust
// backend/crates/nomi-agent-core/tests/reminders.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::reminders::{cancel_reminder, create_reminder, get_user_timezone, list_reminders};

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_timezone_defaults_to_utc_when_no_preferences_row_exists(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let tz = get_user_timezone(&mut conn, user_id).await;

    assert_eq!(tz, "UTC");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_timezone_returns_the_stored_value(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'America/New_York')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let tz = get_user_timezone(&mut conn, user_id).await;

    assert_eq!(tz, "America/New_York");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_inserts_a_one_time_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let input = serde_json::json!({
        "run_at": "2026-09-21T11:00:00-04:00",
        "label": "take a bath",
        "prompt": "Remind the user to take a bath.",
        "target_agent": "chitchat"
    });

    let result = create_reminder(&mut conn, session_id, user_id, "chitchat", &input).await;
    assert!(result.is_ok(), "{result:?}");

    let (label, recurrence): (String, Option<String>) =
        sqlx::query_as("SELECT label, recurrence FROM scheduled_jobs WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(label, "take a bath");
    assert_eq!(recurrence, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_rejects_weekly_recurrence_without_a_weekday(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let input = serde_json::json!({
        "run_at": "2026-09-21T11:00:00-04:00",
        "label": "check spending",
        "prompt": "Check recent spending.",
        "target_agent": "chitchat",
        "recurrence": "weekly"
    });

    let result = create_reminder(&mut conn, session_id, user_id, "chitchat", &input).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("recurrence_weekday"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_reminders_returns_only_this_users_active_jobs(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let (_, other_user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "mine", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    create_reminder(
        &mut conn,
        session_id,
        other_user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "not mine", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();

    let listing = list_reminders(&mut conn, user_id).await.unwrap();

    assert!(listing.contains("mine"));
    assert!(!listing.contains("not mine"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_marks_it_cancelled(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    let id: Uuid = sqlx::query_scalar("SELECT id FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = cancel_reminder(&mut conn, user_id, &serde_json::json!({"reminder_id": id.to_string()})).await;
    assert!(result.is_ok(), "{result:?}");

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_rejects_another_users_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let (_, other_user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    create_reminder(
        &mut conn,
        session_id,
        user_id,
        "chitchat",
        &serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "p", "target_agent": "chitchat"}),
    )
    .await
    .unwrap();
    let id: Uuid = sqlx::query_scalar("SELECT id FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = cancel_reminder(&mut conn, other_user_id, &serde_json::json!({"reminder_id": id.to_string()})).await;

    assert!(result.is_err());
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "active", "another user's cancel attempt must not change this job's status");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-agent-core --test reminders`
Expected: FAIL — `nomi_agent_core::reminders` module does not exist.

- [ ] **Step 4: Write the implementation**

```rust
// backend/crates/nomi-agent-core/src/reminders.rs
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

pub async fn get_user_timezone(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> String {
    sqlx::query_scalar("SELECT timezone FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "UTC".to_string())
}

/// Parses and validates the recurrence-related fields together, since `recurrence_weekday`/
/// `recurrence_day_of_month` are each required exactly when their matching `recurrence` value is
/// given — not enforced by a DB CHECK constraint (see the design spec's Data Model section), so
/// it's validated here instead, the same way `parse_todo_items`/`parse_table_input` validate
/// their own tools' input in engine.rs.
fn parse_recurrence(input: &serde_json::Value) -> Result<(Option<String>, Option<i16>, Option<i16>), String> {
    let recurrence = input.get("recurrence").and_then(|v| v.as_str());
    match recurrence {
        None => Ok((None, None, None)),
        Some("daily") => Ok((Some("daily".to_string()), None, None)),
        Some("weekly") => {
            let weekday = input
                .get("recurrence_weekday")
                .and_then(|v| v.as_i64())
                .ok_or("recurrence_weekday is required when recurrence is weekly")?;
            if !(0..=6).contains(&weekday) {
                return Err("recurrence_weekday must be between 0 and 6".to_string());
            }
            Ok((Some("weekly".to_string()), Some(weekday as i16), None))
        }
        Some("monthly") => {
            let day = input
                .get("recurrence_day_of_month")
                .and_then(|v| v.as_i64())
                .ok_or("recurrence_day_of_month is required when recurrence is monthly")?;
            if !(1..=31).contains(&day) {
                return Err("recurrence_day_of_month must be between 1 and 31".to_string());
            }
            Ok((Some("monthly".to_string()), None, Some(day as i16)))
        }
        Some(other) => Err(format!("recurrence must be daily, weekly, or monthly, got '{other}'")),
    }
}

pub async fn create_reminder(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    created_by_agent_type: &str,
    input: &serde_json::Value,
) -> Result<String, String> {
    let run_at_str = input.get("run_at").and_then(|v| v.as_str()).ok_or("run_at is required")?;
    let run_at = chrono::DateTime::parse_from_rfc3339(run_at_str)
        .map_err(|e| format!("run_at must be a valid ISO 8601 datetime: {e}"))?
        .with_timezone(&chrono::Utc);
    let label = input.get("label").and_then(|v| v.as_str()).ok_or("label is required")?.to_string();
    let prompt = input.get("prompt").and_then(|v| v.as_str()).ok_or("prompt is required")?.to_string();
    let target_agent = input.get("target_agent").and_then(|v| v.as_str()).ok_or("target_agent is required")?.to_string();
    let (recurrence, recurrence_weekday, recurrence_day_of_month) = parse_recurrence(input)?;

    sqlx::query(
        "INSERT INTO scheduled_jobs \
             (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, \
              recurrence, recurrence_weekday, recurrence_day_of_month) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(created_by_agent_type)
    .bind(&target_agent)
    .bind(&label)
    .bind(&prompt)
    .bind(run_at)
    .bind(&recurrence)
    .bind(recurrence_weekday)
    .bind(recurrence_day_of_month)
    .execute(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(format!("Reminder set for {} ({}).", run_at.to_rfc3339(), label))
}

pub async fn list_reminders(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Result<String, String> {
    let rows: Vec<(Uuid, String, chrono::DateTime<chrono::Utc>, Option<String>)> = sqlx::query_as(
        "SELECT id, label, run_at, recurrence FROM scheduled_jobs \
         WHERE user_id = $1 AND status = 'active' ORDER BY run_at",
    )
    .bind(user_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No active reminders.".to_string());
    }

    let mut out = String::new();
    for (id, label, run_at, recurrence) in rows {
        let cadence = recurrence.map(|r| format!(", repeats {r}")).unwrap_or_default();
        out.push_str(&format!("- {id}: \"{label}\" at {}{cadence}\n", run_at.to_rfc3339()));
    }
    Ok(out)
}

pub async fn cancel_reminder(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    input: &serde_json::Value,
) -> Result<String, String> {
    let reminder_id_str = input.get("reminder_id").and_then(|v| v.as_str()).ok_or("reminder_id is required")?;
    let reminder_id: Uuid = reminder_id_str.parse().map_err(|_| "reminder_id must be a valid UUID".to_string())?;

    let result = sqlx::query(
        "UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() \
         WHERE id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(reminder_id)
    .bind(user_id)
    .execute(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("no active reminder with that id".to_string());
    }
    Ok("Reminder cancelled.".to_string())
}
```

- [ ] **Step 5: Wire the module into `lib.rs`**

In `backend/crates/nomi-agent-core/src/lib.rs`, add `pub mod reminders;` alongside the other
`pub mod` lines. Unlike `notification`, this module's functions are called by fully-qualified path
(`nomi_agent_core::reminders::create_reminder`, as the test file above already does) rather than
re-exported flat — matching how `memory`, `permissions`, and `personality` are used elsewhere in
this crate (`crate::memory::try_retrieve_memories(...)` in `engine.rs`), since these are
implementation-detail helper functions, not part of the crate's small top-level public API.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd backend && cargo test -p nomi-agent-core --test reminders`
Expected: PASS (all 7 tests)

- [ ] **Step 7: Commit**

```bash
git add backend/crates/nomi-agent-core/src/subagent.rs backend/crates/nomi-agent-core/src/reminders.rs backend/crates/nomi-agent-core/src/lib.rs backend/crates/nomi-agent-core/tests/reminders.rs
git commit -m "feat: add supports_reminders() flag and reminders create/list/cancel functions"
```

---

### Task 4: Wire `create_reminder`/`list_reminders`/`cancel_reminder` into the engine tool loop

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/engine.rs`
- Modify: `backend/crates/nomi-agent-core/Cargo.toml`
- Modify: `backend/crates/nomi-agent-core/tests/engine.rs`

**Interfaces:**
- Consumes: `SubAgent::supports_reminders()` (Task 3), `reminders::{create_reminder, list_reminders,
  cancel_reminder, get_user_timezone}` (Task 3), `registry.delegatable_agent_types(...)` (existing,
  used identically to `delegate_tool_definition`).
- Produces: `CREATE_REMINDER_TOOL_NAME`, `LIST_REMINDERS_TOOL_NAME`, `CANCEL_REMINDER_TOOL_NAME`
  consts, and the fact that `run_agent_turn` now injects current time/timezone into the system
  prompt whenever `supports_reminders()` is true — consumed by Task 5 (enabling the flag on
  `ChitchatAgent`) and observable by any later agent that opts in.

- [ ] **Step 1: Add the `chrono-tz` dependency**

In `backend/crates/nomi-agent-core/Cargo.toml`, add to `[dependencies]`:

```toml
chrono-tz = "0.10"
```

- [ ] **Step 2: Write the failing end-to-end tests**

Append to `backend/crates/nomi-agent-core/tests/engine.rs` (reusing this file's existing
`TestAgent`, `seed_session`, `seed_agent_session`, `text_response`, `tool_use_response` helpers —
`TestAgent` needs one addition first):

```rust
// In TestAgent's `impl SubAgent for TestAgent` block, alongside the existing supports_todos()/
// supports_plans() overrides:
    fn supports_reminders(&self) -> bool {
        true
    }
```

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_is_only_available_to_agents_that_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "create_reminder", serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "x", "prompt": "y", "target_agent": "test"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &PersonalityAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
    let job_count: i64 = sqlx::query_scalar("SELECT count(*) FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 0, "PersonalityAwareTestAgent never opted into supports_reminders()");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_inserts_a_job_and_returns_a_confirmation(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "create_reminder", serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "Remind the user to take a bath.", "target_agent": "test"})),
        text_response("I'll remind you tomorrow at 11!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "I'll remind you tomorrow at 11!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 }
    );
    let label: String = sqlx::query_scalar("SELECT label FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(label, "take a bath");
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_updates_status(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let reminder_id: Uuid = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'test', 'test', 'x', 'y', now() + interval '1 day') RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "cancel_reminder", serde_json::json!({"reminder_id": reminder_id.to_string()})),
        text_response("Cancelled.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(reminder_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}

#[sqlx::test(migrations = "../../migrations")]
async fn system_prompt_carries_current_time_and_timezone_when_reminders_are_supported(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'America/New_York')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("ok", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let received = provider.received_requests.lock().unwrap();
    let sent_system_prompt = received[0].system.as_deref().unwrap_or_default();
    assert!(sent_system_prompt.contains("America/New_York"));
}
```

The last test reads `FakeLlmProvider`'s own public `received_requests: Mutex<Vec<LlmRequest>>`
field directly (`backend/crates/nomi-test-support/src/lib.rs:29`) — `LlmRequest.system: Option<String>`
(`backend/crates/nomi-llm/src/types.rs:45`) holds the exact string sent as the system prompt. No
new test-fixture method is needed.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-agent-core --test engine create_reminder`
Expected: FAIL — `create_reminder`/`cancel_reminder` tool names not recognized (tool calls come
back as unrecognized-tool errors, not the expected outcomes), and `supports_reminders` doesn't
exist on `TestAgent` yet (compile error until Step 2's trait method is confirmed present from
Task 3).

- [ ] **Step 4: Add tool name consts and tool definitions**

In `backend/crates/nomi-agent-core/src/engine.rs`, alongside the existing tool name consts:

```rust
pub const CREATE_REMINDER_TOOL_NAME: &str = "create_reminder";
pub const LIST_REMINDERS_TOOL_NAME: &str = "list_reminders";
pub const CANCEL_REMINDER_TOOL_NAME: &str = "cancel_reminder";
```

Alongside `write_plan_tool_definition()`:

```rust
fn create_reminder_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: CREATE_REMINDER_TOOL_NAME.to_string(),
        description: "Schedule a reminder to fire at a specific future time. Resolve any \
                       relative time the user gives (\"tomorrow\", \"in an hour\") to an \
                       absolute ISO 8601 datetime yourself, using the current date/time and the \
                       user's timezone given in your system prompt.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "run_at": {"type": "string", "description": "Absolute ISO 8601 datetime, e.g. 2026-09-21T11:00:00-04:00"},
                "label": {"type": "string", "description": "Short human label, e.g. \"take a bath\""},
                "prompt": {"type": "string", "description": "The instruction the target agent will act on when this fires"},
                "target_agent": {"type": "string", "enum": targets, "description": "Which agent takes this job when it fires"},
                "recurrence": {"type": "string", "enum": ["daily", "weekly", "monthly"], "description": "Omit for a one-time reminder"},
                "recurrence_weekday": {"type": "integer", "minimum": 0, "maximum": 6, "description": "Required when recurrence is weekly (0 = Sunday)"},
                "recurrence_day_of_month": {"type": "integer", "minimum": 1, "maximum": 31, "description": "Required when recurrence is monthly"}
            },
            "required": ["run_at", "label", "prompt", "target_agent"]
        }),
    }
}

fn list_reminders_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: LIST_REMINDERS_TOOL_NAME.to_string(),
        description: "List your active reminders.".to_string(),
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
    }
}

fn cancel_reminder_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: CANCEL_REMINDER_TOOL_NAME.to_string(),
        description: "Cancel an active reminder by id. Call list_reminders first if you don't \
                       already know the id.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "reminder_id": {"type": "string", "description": "The reminder's id, from a prior list_reminders call"}
            },
            "required": ["reminder_id"]
        }),
    }
}
```

- [ ] **Step 5: Register the tools in `run_agent_turn` and add the system-prompt injection**

In `run_agent_turn`, right after the existing `if agent.supports_plans() { ... }` block:

```rust
    if agent.supports_reminders() {
        let targets = registry.delegatable_agent_types(agent.agent_type().as_ref());
        tools.push(create_reminder_tool_definition(&targets));
        tools.push(list_reminders_tool_definition());
        tools.push(cancel_reminder_tool_definition());
    }
```

Then, right after the existing `uses_personality()` block that builds `system_prompt` (the block
ending `} else { system_prompt };` around what is currently line 213-220), add:

```rust
    let system_prompt = if agent.supports_reminders() {
        let timezone_name = crate::reminders::get_user_timezone(conn, user_id).await;
        let tz: chrono_tz::Tz = timezone_name.parse().unwrap_or(chrono_tz::UTC);
        let now = chrono::Utc::now().with_timezone(&tz);
        format!(
            "{system_prompt}\n\nCurrent date/time: {} ({timezone_name}). When scheduling a \
             reminder, resolve relative times against this.",
            now.to_rfc3339(),
        )
    } else {
        system_prompt
    };
```

(`chrono_tz` needs `use chrono_tz;` — or fully-qualified as written above, matching this file's
existing style of qualifying less-common types inline rather than adding new top-of-file `use`
lines for a single call site.)

- [ ] **Step 6: Add the three dispatch branches**

In the tool-execution `for block in tool_use_blocks` loop, add three new `else if` branches
immediately after the existing `WRITE_PLAN_TOOL_NAME` branch (before the `already_decided` branch):

```rust
            } else if name.as_str() == CREATE_REMINDER_TOOL_NAME && agent.supports_reminders() {
                match crate::reminders::create_reminder(conn, session_id, user_id, agent.agent_type().as_ref(), input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == LIST_REMINDERS_TOOL_NAME && agent.supports_reminders() {
                match crate::reminders::list_reminders(conn, user_id).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == CANCEL_REMINDER_TOOL_NAME && agent.supports_reminders() {
                match crate::reminders::cancel_reminder(conn, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
```

- [ ] **Step 7: Exclude the three tools from `is_gateable` and from activity-posting**

Update `is_gateable`:

```rust
fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != SHOW_TABLE_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
        && tool_name != WRITE_PLAN_TOOL_NAME
        && tool_name != CREATE_REMINDER_TOOL_NAME
        && tool_name != LIST_REMINDERS_TOOL_NAME
        && tool_name != CANCEL_REMINDER_TOOL_NAME
}
```

Update the `should_post` computation (so these three don't get a duplicate activity message the
way `WRITE_PLAN_TOOL_NAME` already doesn't — their own tool-result text is enough, surfaced via the
agent's own reply):

```rust
            let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME
                    && name.as_str() != WRITE_PLAN_TOOL_NAME
                    && name.as_str() != CREATE_REMINDER_TOOL_NAME
                    && name.as_str() != LIST_REMINDERS_TOOL_NAME
                    && name.as_str() != CANCEL_REMINDER_TOOL_NAME);
```

- [ ] **Step 8: Export the new tool name consts from `lib.rs`**

In `backend/crates/nomi-agent-core/src/lib.rs`, extend the existing `pub use engine::{...}` list:

```rust
pub use engine::{
    resolve_tool_batch, run_agent_turn, LoopOutcome, ToolBatchOutcome, CANCEL_REMINDER_TOOL_NAME,
    COMPLETE_TASK_TOOL_NAME, CREATE_REMINDER_TOOL_NAME, DELEGATE_TOOL_NAME, LIST_REMINDERS_TOOL_NAME,
    SHOW_TABLE_TOOL_NAME, UPDATE_TODOS_TOOL_NAME, WRITE_PLAN_TOOL_NAME,
};
```

- [ ] **Step 9: Run tests to verify they pass**

Run: `cd backend && cargo test -p nomi-agent-core --test engine`
Expected: PASS (all tests in the file, including the 4 new ones and every pre-existing test still
passing unchanged).

- [ ] **Step 10: Commit**

```bash
git add backend/crates/nomi-agent-core/src/engine.rs backend/crates/nomi-agent-core/src/lib.rs backend/crates/nomi-agent-core/Cargo.toml backend/crates/nomi-agent-core/tests/engine.rs backend/Cargo.lock
git commit -m "feat: wire create_reminder/list_reminders/cancel_reminder into the tool loop"
```

---

### Task 5: Enable `supports_reminders()` on `ChitchatAgent`

**Files:**
- Modify: `backend/crates/nomi-agent-chitchat/src/lib.rs`

**Interfaces:**
- Consumes: `SubAgent::supports_reminders()` default method (Task 3).
- Produces: nothing new consumed by later tasks — this is the leaf wiring step.

- [ ] **Step 1: Write the failing test**

`nomi-agent-chitchat` currently has no test file (check `backend/crates/nomi-agent-chitchat/tests/`
first — if a test file already exists, add to it; otherwise create one):

```rust
// backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs
use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::SubAgent;

#[test]
fn chitchat_supports_reminders() {
    assert!(ChitchatAgent.supports_reminders());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p nomi-agent-chitchat chitchat_supports_reminders`
Expected: FAIL — `supports_reminders()` returns the trait default `false`.

- [ ] **Step 3: Implement**

In `backend/crates/nomi-agent-chitchat/src/lib.rs`, add to the `impl SubAgent for ChitchatAgent`
block, alongside the existing `can_delegate()` override:

```rust
    fn supports_reminders(&self) -> bool {
        true
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd backend && cargo test -p nomi-agent-chitchat chitchat_supports_reminders`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-agent-chitchat/src/lib.rs backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs
git commit -m "feat: let chitchat create and manage reminders"
```

---

### Task 6: `next_occurrence` pure function

**Files:**
- Create: `backend/crates/nomi-server/src/scheduler_worker.rs` (this task only adds `next_occurrence`
  and its tests; Task 7 adds the rest of the file)
- Modify: `backend/crates/nomi-server/src/lib.rs`
- Modify: `backend/crates/nomi-server/Cargo.toml`

**Interfaces:**
- Consumes: nothing from earlier tasks (pure date math).
- Produces: `pub(crate) fn next_occurrence(from: DateTime<Utc>, recurrence: &str, weekday: Option<i16>, day_of_month: Option<i16>) -> DateTime<Utc>`, consumed by Task 7's `process_claimed_job`.

- [ ] **Step 1: Add `chrono-tz` to `nomi-server` (needed by Task 8's timezone validation, adding now to keep the dependency change in one place)**

In `backend/crates/nomi-server/Cargo.toml`, add to `[dependencies]`:

```toml
chrono-tz = "0.10"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/crates/nomi-server/src/scheduler_worker.rs
use chrono::{DateTime, Datelike, Utc};

pub(crate) fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn daily_advances_by_exactly_24_hours() {
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "daily", None, None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 21, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_advances_to_the_next_matching_weekday() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 3 = Wednesday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(3), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 23, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_rolls_a_full_week_when_today_already_matches() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 0 = Sunday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(0), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 27, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_advances_to_the_next_month_same_day() {
        let from = Utc.with_ymd_and_hms(2026, 9, 15, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(15));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 15, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_clamps_to_the_target_months_last_day() {
        // day_of_month = 31 scheduled from January lands on February's last valid day (28, 2026
        // is not a leap year), not an invalid date and not rolling into March.
        let from = Utc.with_ymd_and_hms(2026, 1, 31, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(31));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 28, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_wraps_from_december_into_january_of_the_next_year() {
        let from = Utc.with_ymd_and_hms(2026, 12, 10, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(10));
        assert_eq!(next, Utc.with_ymd_and_hms(2027, 1, 10, 11, 0, 0).unwrap());
    }
}
```

Add `mod scheduler_worker;` (private for now — `pub mod` happens in Step 5 once the rest of the
file exists) to `backend/crates/nomi-server/src/lib.rs`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-server scheduler_worker::tests`
Expected: FAIL/panic — `next_occurrence` is `todo!()`.

- [ ] **Step 4: Implement `next_occurrence`**

Replace the `todo!()` body:

```rust
pub(crate) fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            // `weekday` is 0=Sunday..6=Saturday (matches the create_reminder tool schema's
            // description). chrono's Weekday::num_days_from_sunday() uses the same convention.
            let target = weekday.unwrap_or(0) as u32;
            let mut candidate = from + chrono::Duration::days(1);
            while candidate.weekday().num_days_from_sunday() != target {
                candidate += chrono::Duration::days(1);
            }
            candidate
        }
        "monthly" => {
            let day = day_of_month.unwrap_or(1) as u32;
            let (mut year, mut month) = (from.year(), from.month());
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .with_day(1)
                .unwrap()
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            // Step through day=1 first: with_month()/with_year() reject a result that isn't a
            // real calendar date, and `from`'s own day-of-month (e.g. 31) may not exist in the
            // target month — day=1 always does, in every month.
            from.with_day(1).unwrap().with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test -p nomi-server scheduler_worker::tests`
Expected: PASS (all 6 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-server/src/scheduler_worker.rs backend/crates/nomi-server/src/lib.rs backend/crates/nomi-server/Cargo.toml backend/Cargo.lock
git commit -m "feat: add next_occurrence recurrence date math for scheduled reminders"
```

---

### Task 7: `scheduler_worker.rs` claim loop, job processing, and wiring into `main.rs`

**Files:**
- Modify: `backend/crates/nomi-server/src/scheduler_worker.rs`
- Modify: `backend/crates/nomi-server/src/lib.rs`
- Modify: `backend/crates/nomi-server/src/main.rs`
- Test: `backend/crates/nomi-server/tests/scheduler_worker.rs`

**Interfaces:**
- Consumes: `next_occurrence` (Task 6), `NotificationDelivery`/`LogOnlyDelivery` (Task 2),
  `nomi_agent_core::run_agent_turn`/`LoopOutcome` (existing, same as `delegation_worker.rs`),
  `crate::build_agent_registry` (existing), `crate::bootstrap::{build_llm_provider_for_user,
  build_embedding_provider_from_settings_or_env}` (existing).
- Produces: `pub async fn run(pool: PgPool, mqtt: MqttPublisher, s3: Option<S3Config>, settings_key:
  [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: LocalFsStore,
  notification: Arc<dyn NotificationDelivery>)`, spawned from `main.rs`. Nothing later in this plan
  consumes this directly (it's the terminal wiring for the backend half of the feature).

- [ ] **Step 1: Write the failing tests**

```rust
// backend/crates/nomi-server/tests/scheduler_worker.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;
use nomi_agent_core::{LogOnlyDelivery, NotificationDelivery};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

async fn insert_job(
    pool: &PgPool,
    session_id: Uuid,
    user_id: Uuid,
    run_at_offset: &str,
    recurrence: Option<&str>,
) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, recurrence) \
         VALUES ($1, $2, 'chitchat', 'chitchat', 'take a bath', 'Remind the user to take a bath.', now() + $3::interval, $4) \
         RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(run_at_offset)
    .bind(recurrence)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_does_not_claim_a_future_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    insert_job(&pool, session_id, user_id, "1 hour", None).await;

    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(claimed.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_claims_a_due_job_and_sets_claimed_at(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", None).await;

    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(claimed.is_some());
    assert_eq!(claimed.unwrap().id, id);
    let claimed_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT claimed_at FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(claimed_at.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_does_not_reclaim_an_already_claimed_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    insert_job(&pool, session_id, user_id, "-1 hour", None).await;

    let first = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();
    assert!(first.is_some());
    let second = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap();

    assert!(second.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_one_time_job_is_marked_completed_after_a_successful_fire(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", None).await;
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();

    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![ContentBlock::Text { text: "Time for a bath!".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let (status, last_fired_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, last_fired_at FROM scheduled_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "completed");
    assert!(last_fired_at.is_some());

    let message_content: String = sqlx::query_scalar("SELECT content FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_content, "Time for a bath!");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_recurring_job_has_its_run_at_advanced_and_claim_cleared_after_a_fire(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let id = insert_job(&pool, session_id, user_id, "-1 hour", Some("daily")).await;
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();
    let run_at_before_fire = claimed.run_at;

    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![ContentBlock::Text { text: "Time for a bath!".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let (status, claimed_at, run_at): (String, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT status, claimed_at, run_at FROM scheduled_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "active", "a recurring job stays active after firing");
    assert!(claimed_at.is_none(), "claimed_at must be cleared so the next poll can claim it again");
    assert_eq!(run_at, run_at_before_fire + chrono::Duration::days(1));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_target_agent_type_cancels_the_job(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'chitchat', 'not_a_real_agent', 'x', 'y', now() - interval '1 hour')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    let claimed = nomi_server::scheduler_worker::claim_next(&pool).await.unwrap().unwrap();
    let id = claimed.id;

    let provider = FakeLlmProvider::sequence(vec![]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(nomi_agent_chitchat::ChitchatAgent)]);
    let notification: std::sync::Arc<dyn NotificationDelivery> = std::sync::Arc::new(LogOnlyDelivery);

    nomi_server::scheduler_worker::process_claimed_job(&pool, None, None, &provider, &embedding_provider, &registry, notification.as_ref(), claimed)
        .await;

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}
```

`process_claimed_job` takes `mqtt: Option<&MqttPublisher>` (implemented in Step 3 below), matching
`run_agent_turn`'s own `mqtt: Option<(&MqttPublisher, Uuid)>` pattern — these tests pass `None`,
so they need no live MQTT broker.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-server --test scheduler_worker`
Expected: FAIL — `claim_next`/`process_claimed_job` are not `pub` yet / don't exist.

- [ ] **Step 3: Implement `claim_next`, `process_claimed_job`, and `run`**

Full contents of `backend/crates/nomi-server/src/scheduler_worker.rs` (replacing/extending the
Task 6 version):

```rust
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, LoopOutcome, NotificationDelivery};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};

const POLL_INTERVAL: Duration = Duration::from_secs(30);
const REMINDER_MAX_TOKENS: u32 = 1024;

pub(crate) struct ClaimedJob {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub target_agent_type: String,
    pub prompt: String,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i16>,
    pub recurrence_day_of_month: Option<i16>,
    pub run_at: DateTime<Utc>,
}

pub(crate) async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String, Option<String>, Option<i16>, Option<i16>, DateTime<Utc>)> =
        sqlx::query_as(
            "WITH claimed AS ( \
                 SELECT id FROM scheduled_jobs \
                 WHERE status = 'active' AND run_at <= now() AND claimed_at IS NULL \
                 ORDER BY run_at \
                 FOR UPDATE SKIP LOCKED \
                 LIMIT 1 \
             ) \
             UPDATE scheduled_jobs SET claimed_at = now() \
             WHERE id IN (SELECT id FROM claimed) \
             RETURNING id, session_id, user_id, target_agent_type, prompt, recurrence, \
                       recurrence_weekday, recurrence_day_of_month, run_at",
        )
        .fetch_optional(pool)
        .await?;

    Ok(row.map(
        |(id, session_id, user_id, target_agent_type, prompt, recurrence, recurrence_weekday, recurrence_day_of_month, run_at)| ClaimedJob {
            id,
            session_id,
            user_id,
            target_agent_type,
            prompt,
            recurrence,
            recurrence_weekday,
            recurrence_day_of_month,
            run_at,
        },
    ))
}

pub(crate) fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            let target = weekday.unwrap_or(0) as u32;
            let mut candidate = from + chrono::Duration::days(1);
            while candidate.weekday().num_days_from_sunday() != target {
                candidate += chrono::Duration::days(1);
            }
            candidate
        }
        "monthly" => {
            let day = day_of_month.unwrap_or(1) as u32;
            let (mut year, mut month) = (from.year(), from.month());
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            from.with_day(1).unwrap().with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}

async fn finish_one_time_or_advance_recurring(pool: &PgPool, job: &ClaimedJob) {
    match &job.recurrence {
        Some(recurrence) => {
            let next_run_at = next_occurrence(job.run_at, recurrence, job.recurrence_weekday, job.recurrence_day_of_month);
            let _ = sqlx::query("UPDATE scheduled_jobs SET run_at = $2, claimed_at = NULL, last_fired_at = now() WHERE id = $1")
                .bind(job.id)
                .bind(next_run_at)
                .execute(pool)
                .await;
        }
        None => {
            let _ = sqlx::query("UPDATE scheduled_jobs SET status = 'completed', last_fired_at = now() WHERE id = $1")
                .bind(job.id)
                .execute(pool)
                .await;
        }
    }
}

/// Runs one claimed job to completion: looks up its target agent, runs a real agent turn with
/// `job.prompt` as the task, posts the result as a chat message authored by the target agent
/// itself (not "Supervisor" — see the design spec's Firing section for why), calls
/// `notification.deliver` best-effort, then either completes (one-time) or advances `run_at`
/// (recurring). Exposed as a standalone function (rather than inlined into `run`'s loop) so it's
/// directly testable without spinning the infinite polling loop.
pub(crate) async fn process_claimed_job(
    pool: &PgPool,
    mqtt: Option<&MqttPublisher>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn nomi_llm::LlmProvider,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    registry: &AgentRegistry,
    notification: &dyn NotificationDelivery,
    job: ClaimedJob,
) {
    let Some(agent) = registry.find(&job.target_agent_type) else {
        tracing::warn!(job_id = %job.id, target = %job.target_agent_type, "scheduler worker: unknown target agent, cancelling job");
        let _ = sqlx::query("UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() WHERE id = $1")
            .bind(job.id)
            .execute(pool)
            .await;
        return;
    };

    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!(job_id = %job.id, error = %e, "scheduler worker: failed to acquire connection");
            return;
        }
    };

    let messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: job.prompt.clone() }] }];

    let outcome = nomi_agent_core::run_agent_turn(
        &mut conn,
        mqtt.map(|m| (m, job.id)),
        s3,
        provider,
        embedding_provider,
        registry,
        agent.as_ref(),
        job.session_id,
        job.session_id,
        job.user_id,
        messages,
        REMINDER_MAX_TOKENS,
    )
    .await;

    match outcome {
        Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
            let message_id: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(job.session_id)
            .bind(&text)
            .bind(agent.display_name().as_ref())
            .fetch_one(&mut *conn)
            .await
            .ok();
            if let (Some(message_id), Some(publisher)) = (message_id, mqtt) {
                let _ = publisher.publish(job.session_id, &StreamEnvelope::MessageCreated { message_id }).await;
            }
            let _ = notification.deliver(job.user_id, &text).await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
        Ok(LoopOutcome::AwaitingApproval { .. }) => {
            tracing::info!(job_id = %job.id, "scheduler worker: fired reminder needed a tool approval it can't get unattended");
            let notice = "Your reminder needed a tool approval it can't get automatically, so it didn't complete.";
            let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3)")
                .bind(job.session_id)
                .bind(notice)
                .bind(agent.display_name().as_ref())
                .execute(&mut *conn)
                .await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
        Err(e) => {
            tracing::warn!(job_id = %job.id, error = %e, "scheduler worker: fired reminder failed");
            let sorry = format!("I wasn't able to complete your reminder — {e}.");
            let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3)")
                .bind(job.session_id)
                .bind(&sorry)
                .bind(agent.display_name().as_ref())
                .execute(&mut *conn)
                .await;
            finish_one_time_or_advance_recurring(pool, &job).await;
        }
    }
}

/// Runs the scheduler-worker loop forever: every `POLL_INTERVAL`, claims and fires every due
/// `scheduled_jobs` row. Poll-only, unlike `delegation_worker.rs`'s LISTEN/NOTIFY — the trigger
/// here is elapsed time, not a row insert, so there's nothing useful for a notify to wake early.
pub async fn run(
    pool: PgPool,
    mqtt: MqttPublisher,
    s3: Option<nomi_storage::S3Config>,
    settings_key: [u8; 32],
    http_client: reqwest::Client,
    project_storage: nomi_storage::LocalFsStore,
    notification: Arc<dyn NotificationDelivery>,
) {
    tracing::info!("scheduler worker: polling for due reminders every {:?}", POLL_INTERVAL);
    let registry = crate::build_agent_registry(project_storage);

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        loop {
            let claimed = match claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "scheduler worker: failed to claim next job");
                    break;
                }
            };

            let provider = build_llm_provider_for_user(&pool, claimed.user_id, &settings_key, http_client.clone()).await;
            let embedding_provider = build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            process_claimed_job(&pool, Some(&mqtt), s3.as_ref(), provider.as_ref(), embedding_provider.as_ref(), &registry, notification.as_ref(), claimed).await;
        }
    }
}
```

Note: `next_occurrence` and its `#[cfg(test)] mod tests` block from Task 6 already live in this
file — this step's full listing above supersedes the Task 6 version by adding everything else
around it; do not duplicate `next_occurrence`, just add the new code alongside it.

- [ ] **Step 4: Make the module public and wire into `main.rs`**

In `backend/crates/nomi-server/src/lib.rs`, change `mod scheduler_worker;` (added in Task 6) to
`pub mod scheduler_worker;`.

In `backend/crates/nomi-server/src/main.rs`, inside the `if run_worker_inline { ... }` block,
right after the existing delegation worker `tokio::spawn` (after the line ending
`tracing::info!("embedded worker enabled ...")` stays where it is — add the new spawn just before
that final `tracing::info!` line, alongside the other two):

```rust
        let scheduler_mqtt_client_id = format!("nomi-orchestrator-scheduler-{}", uuid::Uuid::new_v4());
        let scheduler_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &scheduler_mqtt_client_id);
        let scheduler_pool = pool.clone();
        let scheduler_s3 = s3.clone();
        let scheduler_http_client = http_client.clone();
        let scheduler_project_storage = project_storage.clone();
        let notification: std::sync::Arc<dyn nomi_agent_core::NotificationDelivery> = std::sync::Arc::new(nomi_agent_core::LogOnlyDelivery);
        tokio::spawn(async move {
            nomi_server::scheduler_worker::run(scheduler_pool, scheduler_mqtt, scheduler_s3, settings_key, scheduler_http_client, scheduler_project_storage, notification).await;
        });
```

(This drops the `database_url` parameter that `worker::run`/`delegation_worker::run` both take —
those need it to construct their own `PgListener`; `scheduler_worker::run` has no listener, so it
never needs a raw connection string. Its signature intentionally omits that parameter.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test -p nomi-server --test scheduler_worker`
Expected: PASS (all 6 tests)

Then run the full workspace build to catch any `main.rs` wiring mistakes:

Run: `cd backend && cargo build`
Expected: builds clean.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-server/src/scheduler_worker.rs backend/crates/nomi-server/src/lib.rs backend/crates/nomi-server/src/main.rs backend/crates/nomi-server/tests/scheduler_worker.rs
git commit -m "feat: add scheduler worker that claims and fires due reminders"
```

---

### Task 8: Timezone preference — backend route and frontend settings UI

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/profile.rs`
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/routes/(app)/preferences/+page.server.ts`
- Modify: `frontend/src/routes/(app)/preferences/+page.svelte`
- Test: wherever `/api/preferences` is already covered today (check
  `backend/crates/nomi-server/tests/` for a file testing `get_preferences`/`put_preferences` before
  creating a new one — extend it if found).

**Interfaces:**
- Consumes: `user_preferences.timezone` column (Task 1).
- Produces: `GET /api/preferences` and `PUT /api/preferences` both handle `timezone`; nothing later
  in this plan depends on this task (it's the final leaf).

- [ ] **Step 1: Find or create the backend test file, and write the failing tests**

Run `grep -rl "put_preferences\|get_preferences" backend/crates/nomi-server/tests/` first. If a
file is found, add these two tests to it; otherwise create
`backend/crates/nomi-server/tests/preferences_routes.rs` following this repo's existing route-test
pattern (an authenticated request built the same way other route tests in this directory construct
one — check one sibling test file for the exact `AuthClaims`/router-building boilerplate and reuse
it rather than inventing a new harness).

```rust
#[tokio::test]
async fn get_preferences_defaults_timezone_to_utc() {
    // ... existing harness setup for an authenticated GET /api/preferences request ...
    // assert response body's `timezone` field is "UTC" when no user_preferences row exists yet
}

#[tokio::test]
async fn put_preferences_rejects_an_invalid_timezone_string() {
    // ... existing harness setup ...
    // PUT /api/preferences with {"timezone": "Not/A_Real_Zone"}
    // assert 400 response
}

#[tokio::test]
async fn put_preferences_saves_a_valid_timezone() {
    // ... existing harness setup ...
    // PUT /api/preferences with {"timezone": "America/New_York"}
    // assert 200 and response body's `timezone` field is "America/New_York"
    // assert a second GET /api/preferences also returns "America/New_York"
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-server preferences`
Expected: FAIL — `timezone` field doesn't exist on `PreferencesResponse`/`UpdatePreferencesRequest` yet.

- [ ] **Step 3: Implement the backend route changes**

In `backend/crates/nomi-server/src/routes/profile.rs`:

```rust
#[derive(Serialize)]
pub struct PreferencesResponse {
    pub theme: String,
    pub accent_color: String,
    pub timezone: String,
}
```

```rust
#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub accent_color: Option<String>,
    pub timezone: Option<String>,
}
```

`get_preferences`:

```rust
    let row: Option<(Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT theme, accent_color, timezone FROM user_preferences WHERE user_id = $1")
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to load preferences");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preferences")
            })?;

    let (theme, accent_color, timezone) = row.unwrap_or((None, None, None));
    Ok(Json(PreferencesResponse {
        theme: theme.unwrap_or_else(|| "system".to_string()),
        accent_color: accent_color.unwrap_or_else(|| "green".to_string()),
        timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
    }))
```

`put_preferences` — add validation right after the existing `accent_color` validation block:

```rust
    if let Some(timezone) = &req.timezone {
        if timezone.parse::<chrono_tz::Tz>().is_err() {
            return Err((StatusCode::BAD_REQUEST, "timezone must be a valid IANA timezone name"));
        }
    }
```

Then extend both the UPDATE and the INSERT-fallback to carry `timezone` through, matching the
existing `theme`/`accent_color` COALESCE shape exactly:

```rust
    let updated: Option<(String, String, String)> = sqlx::query_as(
        "UPDATE user_preferences SET \
            theme = COALESCE($2, theme), \
            accent_color = COALESCE($3, accent_color), \
            timezone = COALESCE($4, timezone), \
            updated_at = now() \
         WHERE user_id = $1 \
         RETURNING theme, accent_color, timezone",
    )
    .bind(claims.sub)
    .bind(&req.theme)
    .bind(&req.accent_color)
    .bind(&req.timezone)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    let (theme, accent_color, timezone) = match updated {
        Some(row) => row,
        None => sqlx::query_as(
            "INSERT INTO user_preferences (user_id, theme, accent_color, timezone) \
             VALUES ($1, COALESCE($2, 'system'), COALESCE($3, 'green'), COALESCE($4, 'UTC')) \
             RETURNING theme, accent_color, timezone",
        )
        .bind(claims.sub)
        .bind(&req.theme)
        .bind(&req.accent_color)
        .bind(&req.timezone)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to save preferences");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
        })?,
    };

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    Ok(Json(PreferencesResponse { theme, accent_color, timezone }))
```

- [ ] **Step 4: Run backend tests to verify they pass**

Run: `cd backend && cargo test -p nomi-server preferences`
Expected: PASS

- [ ] **Step 5: Update the frontend type**

In `frontend/src/lib/types.ts`:

```ts
export interface Preferences {
	theme: Theme;
	accent_color: AccentColor;
	timezone: string;
}
```

- [ ] **Step 6: Update `+page.server.ts`'s load default and add the `updateTimezone` action**

In `frontend/src/routes/(app)/preferences/+page.server.ts`:

```ts
export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = response.ok
		? ((await response.json()) as Preferences)
		: { theme: 'system', accent_color: 'green', timezone: 'UTC' };
	return { preferences };
};
```

Add to the `actions` object, alongside `updateAccentColor`:

```ts
	updateTimezone: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const timezone = data.get('timezone');
		if (typeof timezone !== 'string' || timezone.length === 0) {
			return fail(400, { error: 'Invalid timezone.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ timezone }),
		});
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to save preference.' });
		}
		return { success: true };
	},
```

- [ ] **Step 7: Add the timezone section to `+page.svelte`**

Add this new `<section>` after the existing "Color" section, plus an `onMount` that auto-submits
the detected timezone exactly once, the first time the page loads with no timezone saved yet (i.e.
the server still had to fall back to the `'UTC'` default because no `user_preferences` row
existed) — detected via a `data.preferences` value passed from the load function, not by guessing
from the timezone string itself, since a user's real IANA zone can legitimately be `'UTC'` too:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import { onMount } from 'svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let timezoneForm: HTMLFormElement | undefined = $state();
	let detectedTimezone = $state('');

	onMount(() => {
		detectedTimezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
		if (!data.preferences.hasStoredTimezone && detectedTimezone) {
			timezoneForm?.requestSubmit();
		}
	});
	// ... existing THEME_OPTIONS/ACCENT_COLOR_OPTIONS unchanged ...
</script>
```

```svelte
	<section class="mt-8">
		<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Timezone</h2>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
			Used to schedule reminders at the right local time.
		</p>
		<form
			bind:this={timezoneForm}
			method="POST"
			action="?/updateTimezone"
			use:enhance
			class="mt-3"
		>
			<input type="hidden" name="timezone" value={detectedTimezone || data.preferences.timezone} />
			<select
				class="m3-timezone-select"
				value={data.preferences.timezone}
				onchange={(e) => {
					detectedTimezone = e.currentTarget.value;
					timezoneForm?.requestSubmit();
				}}
			>
				{#each Intl.supportedValuesOf('timeZone') as tz (tz)}
					<option value={tz}>{tz}</option>
				{/each}
			</select>
		</form>
	</section>
```

`data.preferences.hasStoredTimezone` needs a matching addition to the load function's returned
shape — since `PreferencesResponse` itself has no way to distinguish "explicitly saved as UTC" from
"defaulted to UTC because no row exists," add a second field to the backend response for this:

```rust
// PreferencesResponse gains one more field
pub struct PreferencesResponse {
    pub theme: String,
    pub accent_color: String,
    pub timezone: String,
    pub has_stored_timezone: bool,
}
```

Set `has_stored_timezone: timezone.is_some()` in `get_preferences` (using the `Option<String>`
`timezone` variable from the row, before it's defaulted) — and `true` unconditionally in
`put_preferences`'s response (a PUT always means a row now exists). Update
`frontend/src/lib/types.ts`'s `Preferences` interface to add `has_stored_timezone: boolean`, and
reference it as `data.preferences.has_stored_timezone` in the `+page.svelte`/`onMount` snippet
above (not `hasStoredTimezone` — match the snake_case the JSON API actually returns, consistent
with `accent_color` staying snake_case in this same interface today).

- [ ] **Step 8: Add a minimal style rule for the new `<select>`**

Add to `+page.svelte`'s `<style>` block:

```css
	.m3-timezone-select {
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		max-width: 320px;
	}
```

- [ ] **Step 9: Manually verify in a browser**

Run the frontend dev server, sign in, open `/preferences`, confirm: (a) on first load with no
stored timezone, the page auto-submits the browser's detected zone and the `<select>` reflects it
after reload; (b) picking a different zone from the dropdown saves it and persists across a reload.
This repo has no component test harness for `.svelte` files — this manual check plus
`svelte-check` is the established standing verification approach (see the `agent_plans` design
spec's own Testing section for the same precedent).

Run: `cd frontend && npx svelte-check`
Expected: no new type errors.

- [ ] **Step 10: Commit**

```bash
git add backend/crates/nomi-server/src/routes/profile.rs frontend/src/lib/types.ts frontend/src/routes/\(app\)/preferences/+page.server.ts frontend/src/routes/\(app\)/preferences/+page.svelte
git commit -m "feat: add timezone preference with browser auto-detect"
```

If Step 1 found and extended an existing preferences test file rather than creating a new one, add
that file's path to the `git add` above instead of `backend/crates/nomi-server/tests/preferences_routes.rs`.

---

## Self-Review Notes

- **Spec coverage:** every section of the spec has a task — Data Model → Task 1; `supports_reminders()`
  + three tools → Tasks 3-4; system prompt injection → Task 4; `ChitchatAgent` opt-in → Task 5;
  `next_occurrence` → Task 6; scheduler worker + delivery wiring → Tasks 2 and 7; frontend timezone
  → Task 8.
- **Placeholder scan:** the only `todo!()` in this plan is Task 6 Step 2's intentional
  write-the-test-first placeholder, replaced by real code in Step 4 of the same task — not a gap.
- **Type consistency:** `ClaimedJob`, `next_occurrence`, `claim_next`, and `process_claimed_job`
  are defined once in Task 6/7 and referenced with matching names and signatures throughout Task 7's
  own tests; `reminders::{create_reminder, list_reminders, cancel_reminder, get_user_timezone}`
  defined in Task 3 are called with matching signatures from Task 4's `engine.rs` dispatch.
- Both spots that initially needed an unverified test-fixture API (Task 4's system-prompt
  assertion, Task 7's mqtt-free test harness) were resolved against the actual code before this
  plan was finalized: `FakeLlmProvider.received_requests` (a public field) and
  `process_claimed_job`'s own `Option<&MqttPublisher>` parameter — no open lookups remain.
