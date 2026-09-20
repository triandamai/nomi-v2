# Scheduled Reminders — Design

**Goal:** Let a user ask Nomi something like "remind me tomorrow at 11 to take a bath" and have
the system fire that reminder at the right time — delivering it as a chat message (and, later,
a real push notification) via whichever agent the request names.

**Architecture:** A new engine-level tool set (`create_reminder`, `list_reminders`,
`cancel_reminder`), gated behind a new `supports_reminders()` opt-in on `SubAgent` and enabled on
`ChitchatAgent`, writes/reads/cancels rows in a new `scheduled_jobs` table — no delegation
round-trip, so the reply comes back in the same turn. A new `scheduler_worker.rs` background loop,
closely modeled on `delegation_worker.rs`, polls for jobs whose `run_at` has passed, runs a real
agent turn with the row's `target_agent_type` as the actor and `prompt` as its task, and delivers
the result as a normal chat message. A small `NotificationDelivery` trait gives future push
integrations (Telegram, WhatsApp) a seam to plug into without touching the scheduler; v1 ships a
log-only stub.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, `tokio::spawn` background worker (same pattern as `worker.rs` /
`delegation_worker.rs`, embedded in the main server process by default).

## Context

Today, the only way an agent does more than reply in-turn is `delegate_to_agent`
(`crates/nomi-agent-core/src/delegation.rs`), which hands a task to a specialist immediately, and
`agent_delegations` + `delegation_worker.rs` pick it up right away via `LISTEN`/`NOTIFY`. There is
no notion of a task deferred to a future time, recurring or not, and no outbound push
infrastructure anywhere in the product — all message delivery today is a `messages` row insert plus
an MQTT/WebSocket relay to the web frontend.

This project adds the missing piece: a durable, time-triggered version of "hand a task to an
agent," reusing as much of the existing delegation machinery's shape as fits.

## Data Model

```sql
-- 0028_scheduled_jobs.sql
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

- `label` is the short human phrase ("take a bath") `list_reminders`/`cancel_reminder` show back to
  the user; `prompt` is the literal instruction handed to `target_agent_type` at fire time. These
  are allowed to diverge — the creating agent may phrase a richer instruction than the label — but
  for the common case they will read almost identically.
- `recurrence_weekday`/`recurrence_day_of_month` are populated only for the matching `recurrence`
  value; both NULL for `recurrence = 'daily'` or one-time (`recurrence IS NULL`) jobs. Not enforced
  by a `CHECK` cross-column constraint — validated in the tool's Rust input parsing instead,
  matching how `parse_todo_items`/`parse_table_input` validate today rather than pushing
  conditional logic into SQL.
- One-time jobs move to `status = 'completed'` after firing once. Recurring jobs stay `active`
  indefinitely: after each fire, the worker advances `run_at` to the next occurrence and resets
  `claimed_at` to `NULL` — no new row per occurrence.
- `session_id` is a real FK (unlike `agent_plans.agent_session_id`) because reminders are always
  created from — and delivered back into — one specific conversational session; there is no
  sentinel-id case here to accommodate.
- `timezone` follows the exact precedent of `0022_accent_color.sql` extending this same table:
  `NOT NULL DEFAULT` so every existing row is valid with no backfill step. IANA name (e.g.
  `America/New_York`); validity is not checked at the DB layer, matching `theme`'s own
  `CHECK`-only-for-a-closed-enum precedent — an arbitrary IANA string isn't a closed enum, so no
  `CHECK` is added, and an invalid value simply falls back to UTC math wherever it's used.

## Backend: `supports_reminders()` and the Three Tools

```rust
// SubAgent trait — new default method, alongside supports_todos()/supports_plans()
fn supports_reminders(&self) -> bool {
    false
}
```

Enabled on `ChitchatAgent` only for v1 (the conversational front door where "remind me..." requests
land). Other agents opt in later by flipping the flag, same as `supports_todos()`.

```rust
// engine.rs — new tool name consts, alongside WRITE_PLAN_TOOL_NAME
pub const CREATE_REMINDER_TOOL_NAME: &str = "create_reminder";
pub const LIST_REMINDERS_TOOL_NAME: &str = "list_reminders";
pub const CANCEL_REMINDER_TOOL_NAME: &str = "cancel_reminder";
```

**`create_reminder`** — registered only when `agent.supports_reminders()`, with `target_agent`'s
`enum` built from `registry.delegatable_agent_types(agent.agent_type().as_ref())`, the exact same
helper `delegate_tool_definition` already uses:

```rust
ToolDefinition {
    name: CREATE_REMINDER_TOOL_NAME.to_string(),
    description: "Schedule a reminder to fire at a specific future time. Resolve any relative \
                   time the user gives (\"tomorrow\", \"in an hour\") to an absolute ISO 8601 \
                   datetime yourself, using the current date/time and the user's timezone given \
                   in your system prompt.".to_string(),
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
```

**`list_reminders`** — no input; scoped to `user_id` (not just `session_id` — a reminder created in
one session should still be listable/cancellable from another, since it's the user's reminder, not
the session's). Returns active jobs as `{ id, label, run_at, recurrence }`.

**`cancel_reminder`** — `{ reminder_id: string }`. The calling LLM is expected to have called
`list_reminders` earlier in the same turn to learn a valid id from the user's description; the tool
itself does no fuzzy matching.

Dispatch (new branches in `resolve_tool_batch`'s chain, alongside the `WRITE_PLAN_TOOL_NAME` arm):

1. `create_reminder`: parse and validate input (reject if `recurrence = 'weekly'` and
   `recurrence_weekday` missing, or `'monthly'` and `recurrence_day_of_month` missing — an error
   string back to the LLM, same pattern as `parse_todo_items`), then
   `INSERT INTO scheduled_jobs (...) VALUES (...)`, then return a confirmation string ("Reminder
   set for {run_at}.") as the tool result. No content block, no extra posted message — the agent's
   own reply text (which the LLM writes right after this tool result) is the user-visible
   confirmation, exactly like `complete_task`.
2. `list_reminders`: `SELECT id, label, run_at, recurrence FROM scheduled_jobs WHERE user_id = $1
   AND status = 'active' ORDER BY run_at`, formatted into a compact string for the tool result
   (the LLM reads this to answer the user or to pick an id for a follow-up `cancel_reminder` call).
3. `cancel_reminder`: `UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() WHERE
   id = $1 AND user_id = $2 AND status = 'active'` — the `user_id` match is the authorization
   check (mirrors `authorize_session_access`'s ownership-scoping in spirit, but inline since this
   is a tool call, not an HTTP route). Zero rows affected → error string back to the LLM ("no
   active reminder with that id").

All three join `is_gateable`'s existing exclusion list (never permission-gated), same as
`WRITE_PLAN_TOOL_NAME` — none of these touch anything a user would want an approve/deny prompt for.

## Backend: System Prompt Time/Timezone Injection

`create_reminder`'s correctness depends on the LLM resolving "tomorrow at 11" itself, which needs
today's date and the user's timezone in context. In `run_agent_turn`, when `agent.supports_reminders()`
is true, append to the system prompt (same pattern already used for the memories/personality
injections just below it):

```
format!(
    "{base}\n\nCurrent date/time: {now} ({tz}). When scheduling a reminder, resolve relative \
     times against this.",
    now = now_in_tz.to_rfc3339(),
    tz = user_timezone,
)
```

`user_timezone` is read from `user_preferences.timezone` (default `'UTC'` for users who never set
one) via one extra query, gated the same way `uses_personality()`'s lookup already is — only run
when `supports_reminders()` is true, not on every turn. Converting UTC "now" into that IANA
timezone's wall-clock time (so the injected string reads as the user's actual local time, not UTC
labeled with a timezone name) needs the `chrono-tz` crate, not currently a dependency anywhere in
this workspace — added to `nomi-agent-core/Cargo.toml`. An unparseable/unknown timezone string
(shouldn't happen given the column's `'UTC'` default and the frontend always sending a real
`Intl`-resolved IANA name, but not enforced by a DB constraint) falls back to UTC rather than
failing the turn.

## Backend: `scheduler_worker.rs`

Modeled directly on `delegation_worker.rs`, with two differences: no `LISTEN`/`NOTIFY` (the trigger
is elapsed time, not a row insert — a new row's `run_at` is essentially never in the past at
creation time, so there's nothing for a notify to usefully wake early), and recurrence advancement
on completion.

```rust
const POLL_INTERVAL: Duration = Duration::from_secs(30);

struct ClaimedJob {
    id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    target_agent_type: String,
    prompt: String,
    recurrence: Option<String>,
    recurrence_weekday: Option<i16>,
    recurrence_day_of_month: Option<i16>,
    run_at: DateTime<Utc>,
}

async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, sqlx::Error> {
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
```

Loop shape: `loop { tokio::time::sleep(POLL_INTERVAL).await; while let Some(job) = claim_next(...) { ... } }`
— a plain sleep-poll, not `delegation_worker.rs`'s `timeout(POLL_FALLBACK_INTERVAL, listener.recv())`,
since there's no listener to race against.

Per claimed job:

1. Look up `target_agent_type` in the registry; unknown target → log a warning, mark the job
   `status = 'cancelled'` (it can never fire correctly), continue. This mirrors delegation's
   unknown-target handling, but there's no "delegation" row status of `'failed'` to reuse — the
   closest equivalent state here is `cancelled`, since retrying an unknown agent type will never
   succeed.
2. Build a single-message turn: `vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: job.prompt.clone() }] }]`, run `nomi_agent_core::run_agent_turn` with `agent_session_id == session_id` (same sentinel convention `nomi_turn::run_locked_turn` already uses for non-agent_sessions-backed turns).
3. On `Ok(LoopOutcome::Reply { text, .. })` or `Ok(LoopOutcome::Completed { summary: text, .. })`:
   post `text` directly as a `messages` row with `agent_display_name = agent.display_name()` (the
   *target agent's own* name, not "Supervisor" — unlike a delegation, which is background work
   reported back through the supervisor persona mid-conversation, a fired reminder is the literal
   deliverable the user asked for, so it should read as that agent speaking, not as a status
   report). Publish `StreamEnvelope::MessageCreated`. Call
   `notification_delivery.deliver(job.user_id, &text).await` (best-effort, error only logged — see
   below). Then: if `job.recurrence.is_some()`, compute the next `run_at` (see below) and
   `UPDATE scheduled_jobs SET run_at = $2, claimed_at = NULL, last_fired_at = now() WHERE id = $1`;
   otherwise `UPDATE scheduled_jobs SET status = 'completed', last_fired_at = now() WHERE id = $1`.
4. On `Ok(LoopOutcome::AwaitingApproval { .. })`: post a notice message ("Your reminder needed a
   tool approval it can't get automatically, so it didn't complete.") and treat the job the same as
   a one-time failure (`status = 'completed'` if one-time, or advance `run_at` normally if
   recurring) — reminders run unattended, so there is no user available to approve a gated tool
   call the way there is in a live conversational turn; retrying at the same `run_at` would just
   hit the same wall.
5. On `Err(e)`: log a warning, post a short apology message the same way delegation's failure path
   does, and advance/complete the job the same as step 4 (never leave a job permanently stuck
   `claimed_at`-set-forever — a transient failure gets one more shot next cycle for recurring jobs,
   or is simply dropped for one-time ones, matching delegation's "mark it done, don't retry"
   philosophy for the request-response shape this shares).

**Next-occurrence computation** (pure function, unit-testable without a DB):

```rust
fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            // `weekday` is 0=Sunday..6=Saturday (matches the tool schema's description).
            // chrono's Weekday::num_days_from_sunday() uses the same convention, so the two
            // compare directly with no offset translation.
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
            // Clamp to the target month's actual last day (e.g. day_of_month = 31 scheduled
            // from a 30-day or February start lands on that month's last valid day instead of
            // rolling over into the following month).
            let last_day_of_month = NaiveDate::from_ymd_opt(year, month + 1, 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            from.with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}
```

Operates on the job's stored `run_at` (already an absolute UTC instant, computed by the LLM against
the user's timezone at creation time) plus the cadence fields — it does not need to re-consult
`user_preferences.timezone` itself, since `run_at`'s wall-clock-in-that-timezone shape was already
fixed at creation. The `with_year`/`with_month`/`with_day` chain operates on `DateTime<Utc>`
directly rather than converting through the user's local timezone — this is a deliberate
simplification: it preserves `run_at`'s UTC wall-clock time-of-day exactly (so "11:00 user-local"
stays whatever UTC instant that mapped to at creation), at the cost of not re-adjusting across a
DST transition that falls between fires. Acceptable for v1 given daily/weekly/monthly reminders
tolerate an hour of drift far better than a missed or double fire would tolerate a more complex
timezone-aware recomputation; revisit if this proves wrong in practice.

## Backend: Delivery Abstraction

```rust
// New, small — crates/nomi-agent-core/src/notification.rs
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
```

Threaded into `scheduler_worker::run(...)` as one more parameter, constructed once in `main.rs`
(`LogOnlyDelivery` for v1, same as every other worker dependency built once and cloned/moved into
the spawned task). The in-app `messages` insert in step 3 above always happens regardless of what
this call does — `deliver` is a strictly additive notification path, never a gate on the baseline
chat delivery.

## Backend: Wiring

- New `crates/nomi-server/src/scheduler_worker.rs`, spawned in `main.rs` the same way
  `delegation_worker::run` is today — its own `tokio::spawn`, its own cloned `pool`/`s3`/
  `http_client`/`database_url`/`project_storage`, under the same `run_worker_inline` flag (no
  separate env var; it's cheap enough to always run alongside the other two embedded workers).
- `run_agent_turn` gains the `supports_reminders()` tool-registration branch and the system-prompt
  time/timezone injection described above — both additive, no signature change to the function
  itself (it already takes everything needed: `conn` for the `user_preferences` lookup, `agent`
  for the flag check).

## Frontend: Timezone Preference

`user_preferences.timezone` is set once on first load via
`Intl.DateTimeFormat().resolvedOptions().timeZone`, sent to the existing preferences-update
endpoint alongside `theme`/`accent_color`, and editable later in the same settings section as those
two — a plain `<select>` of IANA names is enough for v1, no searchable-timezone-picker component.

## Testing

- `#[sqlx::test]` coverage for the three new tools (create validates recurrence field combinations;
  list returns only the calling user's active jobs, ordered by `run_at`; cancel is a no-op on
  another user's job or an already-cancelled one).
- Unit tests (no DB) for `next_occurrence` covering: daily rolls forward exactly 24h; weekly rolls
  to the correct weekday including the "today is already that weekday" case; monthly clamps to a
  shorter month's last day (e.g. `day_of_month = 31` scheduled from January lands on Feb 28/29, not
  an invalid date).
- `#[sqlx::test]` coverage for `claim_next`'s `FOR UPDATE SKIP LOCKED` query: a job with
  `run_at` in the future is never claimed; a claimed job is not claimed again until `claimed_at`
  is cleared.
- An integration-style test for the worker's one-time-vs-recurring branch after a successful fire
  (asserts `status = 'completed'` vs. `run_at` advanced + `claimed_at` cleared), following the same
  shape as existing `delegation_worker` coverage if any exists, otherwise a new direct test against
  the worker's claim+fire logic factored out for testability (mirroring how `next_occurrence` above
  is kept as a pure, separately testable function rather than inlined into the loop).

## Out of Scope

- **Real push delivery** (Telegram Bot API, WhatsApp Business API, web push, email). `LogOnlyDelivery`
  is the only concrete `NotificationDelivery` implementation shipped by this spec; real
  integrations are future work behind the same trait.
- **Full cron/RRULE recurrence syntax.** Only the three simple cadences (daily / weekly-on-a-weekday
  / monthly-on-a-day-of-month) are supported.
- **Other agents opting into `supports_reminders()`** beyond `ChitchatAgent` — the capability flag
  exists on the trait from day one, but only chitchat turns it on for v1.
- **Editing an existing reminder's time/prompt.** v1 only supports create/list/cancel — changing a
  reminder is cancel-then-recreate.
- **Reminder history/audit trail of past fires.** `last_fired_at` tracks only the most recent fire;
  there is no `scheduled_job_fires` log table in this spec.
