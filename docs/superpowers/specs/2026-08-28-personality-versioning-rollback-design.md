# Personality Versioning and Rollback

Date: 2026-08-28
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-28-personality-customization-design.md` (the personality-customization feature this extends — already implemented and merged), `docs/superpowers/specs/2026-08-26-backend-cargo-workspace-design.md` (the `SubAgent`/`AgentRegistry` architecture `PersonalityAgent` already uses).

## Purpose

Every personality change is already recorded as a `PersonalityChanged` `agent_events` row, but `user_personality` only ever holds the *current* description — there is no way to see what a personality used to be, or to go back to it. This design adds an explicit, numbered version history per user and lets the user roll back to any prior version, either by asking nomi in chat or from a panel in the chat UI. Rolling back never deletes or mutates history — restoring an old version creates a *new* version with that old description, the same way `git revert` works, so the audit trail stays complete and honest no matter how many times a user changes their mind.

## 1. Data Model

One new table. `user_personality` (existing) is kept as-is for the hot read path — `get_current_personality` runs on every chitchat turn via the `uses_personality()` fold-in, and a single-row point lookup must stay cheap — plus one new column so "which version is current" doesn't require a second query:

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

The backfill `INSERT` is required, not optional: without it, `current_version DEFAULT 1` would claim version 1 exists for every user who already had a personality set before this migration, while `user_personality_versions` stayed empty for them — `list_versions` would wrongly report "no history" for a user who demonstrably has a current personality. The backfill makes that first row real. (Their pre-versioning `PersonalityChanged` audit events in `agent_events` are untouched and still exist; this only backfills the new versions table, not the audit log.)

Every version row is immutable once written — nothing ever `UPDATE`s or `DELETE`s a row in `user_personality_versions`. That immutability is what makes "rollback" safe to implement as "read an old row, write a new one" rather than needing any kind of soft-delete or restore-in-place logic.

## 2. `nomi_agent_core::personality` — Widened and Extended

**A signature change to already-shipped code.** The current `set_personality(conn, session_id: Uuid, agent_session_id: Uuid, user_id, new_description)` requires a real session/agent-session for its audit event. That's fine for chat-driven changes, but a rollback triggered from the web UI isn't part of any chat turn — it has neither. `agent_events.session_id` and `.agent_session_id` are already nullable columns (they support this exact case for chitchat's own default-agent sentinel handling), so both parameters widen from `Uuid` to `Option<Uuid>`, with `None` bound as SQL `NULL` when there's no turn context. `PersonalityAgent`'s existing call site (chat-driven `set_personality`) updates to pass `Some(session_id), Some(agent_session_id)` — it always has real values, so its behavior is unchanged.

```rust
pub async fn get_current_personality(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String>;
// unchanged

pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError>;
// Widened signature (was Uuid, Uuid). Behavior: same one transaction as before, plus version
// bookkeeping — reads old description (unchanged), computes next_version =
// COALESCE(MAX(version), 0) + 1 FROM user_personality_versions WHERE user_id = $1, inserts the
// new version row, upserts user_personality (description AND current_version), inserts the
// PersonalityChanged event (payload gains a new_version field), commits.
// Same benign TOCTOU on the MAX(version) read as the existing old_description read under
// concurrent same-user edits — already accepted for that read in the prior feature's final
// review; this inherits the same, not a new risk.

#[derive(Debug, Clone, PartialEq)]
pub struct PersonalityVersion {
    pub version: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

pub async fn list_versions(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<PersonalityVersion>, TurnError>;
// SELECT v.version, v.description, v.created_at, v.version = up.current_version AS is_current
// FROM user_personality_versions v JOIN user_personality up ON up.user_id = v.user_id
// WHERE v.user_id = $1 ORDER BY v.version DESC LIMIT $2
// One query, no N+1: "is this the current version" is computed in SQL via the join, not by a
// second round trip per row.

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("personality version {0} not found for this user")]
    VersionNotFound(i32),
    #[error(transparent)]
    SetPersonality(#[from] TurnError),
}

pub async fn rollback_to_version(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    target_version: i32,
) -> Result<(), RollbackError>;
// Looks up the description at target_version (a plain SELECT — no transaction needed for the
// lookup itself, since version rows are immutable; a concurrent write between the lookup and
// the call below can only mean we revert to a still-valid historical value, not a torn read).
// Returns RollbackError::VersionNotFound if no such version exists for this user. Otherwise
// delegates straight to set_personality with that description — rollback IS a personality
// change, just one whose new description happens to equal an old one. No separate write path,
// no duplicated transaction logic.
```

## 3. `PersonalityAgent` — Two New Tools

Two additions to the existing tool set (`set_personality` is unchanged):

```json
{
  "name": "list_personality_versions",
  "description": "List your past personality descriptions, most recent first, so you can decide what to roll back to.",
  "input_schema": { "type": "object", "properties": {} }
}
```
```json
{
  "name": "rollback_personality",
  "description": "Roll back to a previous personality version. This creates a new version with that version's description rather than deleting anything.",
  "input_schema": {
    "type": "object",
    "properties": {
      "version": { "type": "integer", "description": "The version number to restore, from list_personality_versions." }
    },
    "required": ["version"]
  }
}
```

`execute_tool` dispatches `"list_personality_versions"` to a handler that calls `nomi_agent_core::personality::list_versions(conn, user_id, 10)` (capped at 10 — enough for a real conversation, small enough to not bloat the LLM's context) and formats each row as one line of text (version number, description, and a relative/short date, with the current one marked). `"rollback_personality"` dispatches to a handler that validates the `version` field is present and calls `nomi_agent_core::personality::rollback_to_version(conn, Some(session_id), Some(agent_session_id), user_id, version)`, translating `RollbackError::VersionNotFound` into a tool-error string the LLM can relay back to the user (e.g. "I don't have a version 7 — you have versions 1 through 4.").

**System prompt addition** (appended to the existing prompt from the prior feature): "If the user wants to see their past personalities or go back to an earlier one, call `list_personality_versions` to show them the options, then `rollback_personality` with the version they choose. Never guess a version number without listing first unless the user gives one explicitly."

## 4. HTTP API (new routes in `nomi-server`)

Both routes are purely per-user — `claims.sub` (the authenticated user's `users.id`, already established as the same id `channel_identities.user_id`/`user_personality.user_id` use — see `web_identity::ensure_web_channel_identity`) is enough. No session, no org-scoping, no extra authorization check beyond a valid JWT, matching how personality has always been per-user-only, not per-org.

```rust
// routes/personality.rs (new file)

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
) -> Result<Json<PersonalityHistoryResponse>, (StatusCode, &'static str)>;
// GET /api/personality/history — calls list_versions(claims.sub, 50), maps to the response type.

#[derive(Deserialize)]
pub struct RollbackPersonalityRequest {
    pub version: i32,
}

pub async fn rollback_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<RollbackPersonalityRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)>;
// POST /api/personality/rollback — calls rollback_to_version(None, None, claims.sub, req.version).
// RollbackError::VersionNotFound -> 404; any other error -> 500. Success -> 204 No Content.
```

Wired into `app.rs` alongside the existing routes:
```rust
.route("/api/personality/history", get(personality_routes::get_personality_history))
.route("/api/personality/rollback", post(personality_routes::rollback_personality))
```

## 5. Frontend

A "Personality" button sits next to the existing model-picker button in the chat input toolbar (`(app)/chat/[sessionId]/+page.svelte`), toggling an absolute-positioned dropdown panel — same interaction pattern the model picker already uses (a `$state` boolean, a `Button` toggling it, a panel styled with the same MD3 surface/elevation tokens).

- `+page.server.ts`'s `load` fetches `/api/personality/history` alongside messages/models and exposes it as `data.personality.versions`.
- A new form action, `restorePersonality`, reads `version` from form data and `POST`s to `/api/personality/rollback`, returning `fail()` with a message on error (mirroring `selectAdminModel`'s shape) — on success, SvelteKit's `use:enhance` default `update()` re-runs `load`, so the panel reflects the new current version without a manual `invalidateAll()`.
- The panel lists versions newest-first: description text, a formatted `created_at`, visually marked if `is_current` (e.g. the same selected-state styling `.m3-picker-item--selected` already uses for the current model), and a small "Restore" button on every non-current row — each one its own tiny form (`hidden` input for `version`) posting to `?/restorePersonality`, matching how each model in the model picker is its own form today.
- Empty state ("You haven't set a personality yet — just ask nomi to change it.") when the list is empty.
- **Out of scope for this panel:** setting a brand-new personality by typing free text. That stays chat-only, unchanged from the original feature — this panel only views history and restores past versions.

## 6. Testing

Mirrors the rigor of the original personality-customization feature:

- `nomi-agent-core`: unit tests for `list_versions` (ordering, `is_current` correctness, limit respected) and `rollback_to_version` (creates a new version with the target's description rather than mutating history; `VersionNotFound` for an unknown version; `set_personality`'s existing behavior — old/new audit payload, upsert — still holds with `Option<Uuid>` args, both `Some` and `None`).
- `nomi-agent-personality`: unit tests for both new tools' `execute_tool` paths, including the "no such version" error string surfaced back through the tool-error path.
- `nomi-turn`: an end-to-end test proving a chat-driven `rollback_personality` tool call actually changes what's folded into the next chitchat reply's system prompt (same shape as the original feature's e2e test).
- `nomi-server`: integration tests for both new routes — history returns the right shape and `is_current` flag, rollback creates a new version and 404s on an unknown one, both scoped correctly to `claims.sub` (a second user's versions never leak into the first user's response).
- Frontend: exact test approach (unit vs. e2e, which existing convention to follow) is left to the implementation plan, which will inspect what test tooling the chat page's existing model-picker feature actually uses today before deciding.

## Error Handling

| Condition | Behavior |
|---|---|
| `rollback_personality` (tool or HTTP) called with a version number that doesn't exist for this user | `RollbackError::VersionNotFound` — tool path surfaces a clear message to the LLM; HTTP path returns 404. |
| `list_personality_versions`/`GET /api/personality/history` called with zero versions set | Empty list, not an error — same "no personality set" convention as the original feature. |
| Concurrent same-user personality changes (two rollbacks, or a chat change and a UI rollback, racing) | Same benign TOCTOU already accepted for `set_personality`'s `old_description` read — worst case, an audit payload's `old_description` or the computed `next_version` reflects a slightly stale read. The current-state row (`user_personality`) is always correct because the final upsert serializes on the row. Not hardened further here, consistent with the existing precedent. |

## Out of Scope

- Deleting or editing a past version — history is permanent and append-only by design.
- Setting a new personality directly from the frontend panel (stays chat-only).
- Per-org or per-session personality history (personality remains strictly per-user, unchanged from the original design).
- A retention/pruning policy for old versions — no cap on how many versions accumulate per user.
