# Rich Content Blocks & Tool Approval Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace today's plain-text tool/action activity messages with structured content blocks (file diff/write, delete, live todo list, data/comparison table) rendered as real widgets, plus a Claude-Code-style tool permission subsystem with an in-chat approval card that pauses and resumes the turn loop.

**Architecture:** Add a nullable `content_blocks JSONB` column to `messages`; a new `ContentBlock` enum in `nomi-agent-core` (aliased `RichBlock` wherever it would collide with `nomi_llm::ContentBlock`) is built by the tool that owns the data and travels alongside the existing plain-text `content` fallback. `SubAgent::execute_tool` returns a new `ToolOutcome{display_text, block}` instead of a bare string. A pre-scan in `run_agent_turn` checks `tool_permission_rules` before executing any tool call in a response; an unmatched ("ask") tool call persists the exact paused conversation into `agent_sessions.state` (already existing, unused) and returns a new `LoopOutcome::AwaitingApproval`; a new `approval_resumes` queue (parallel to the existing `turn_jobs` queue, claimed by the same worker process) re-enters `run_agent_turn` once the user decides. `StreamEnvelope` gains `MessageCreated`/`MessageUpdated` (replacing `SessionActivity`) so the frontend patches one message instead of refetching the whole conversation.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend (`frontend/`), Postgres, MQTT (rumqttc) for realtime, `@codemirror/merge` (new) for the diff view.

**Spec:** `docs/superpowers/specs/2026-09-07-rich-content-blocks-design.md`

## Global Constraints

- `messages.content` stays `NOT NULL` and always populated — never remove or make it conditional.
- No new `agent_sessions.status` value — pause state lives entirely in the existing `state JSONB` column, merge-patched (never blindly overwritten), since it must coexist with the unrelated todo-list-pointer key.
- The approval/todo mechanism only works for agents with a **real** `agent_sessions` row — chitchat (the default agent) uses `session_id` as a sentinel `agent_session_id` with no real row, but chitchat also has zero tools, so this never actually arises; do not add special-casing for it.
- No block type for tool failures — errors stay plain-text (`⚠️ ...`), matching today.
- `path_pattern` glob matching is application code (a small `*`-only glob), never a SQL-level match.
- Package manager for `frontend/` is **npm** (`package-lock.json` is the lockfile CI/Docker actually use) — always run `npm install`/`npm run` there, never `pnpm`.
- Every new Rust file/function needs `#[cfg(test)]` unit tests or an integration test in `tests/`; every new pure TS/JS function needs a `vitest` test — this codebase has no Svelte component test harness, so new `.svelte` components are verified via `svelte-check` + live browser verification (Playwright/manual), not unit tests.

---

## Task 1: Migration — `content_blocks`, `tool_permission_rules`, `approval_resumes`

**Files:**
- Create: `backend/migrations/0023_content_blocks_and_permissions.sql`

**Interfaces:**
- Produces: `messages.content_blocks JSONB` (nullable), `tool_permission_rules` table, `approval_resumes` table — every later backend task binds against these exact names/columns.

- [ ] **Step 1: Write the migration**

```sql
-- backend/migrations/0023_content_blocks_and_permissions.sql

ALTER TABLE messages ADD COLUMN content_blocks JSONB;

CREATE TABLE tool_permission_rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    tool_name    TEXT NOT NULL,
    path_pattern TEXT,
    decision     TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX tool_permission_rules_lookup_idx ON tool_permission_rules (user_id, tool_name);

-- Parallel queue to `turn_jobs`, for resuming a turn after an approval decision instead of
-- ingesting a new inbound message. Claimed by the same worker process (see worker.rs), on its
-- own NOTIFY channel so the existing turn_jobs claim loop is untouched.
CREATE TABLE approval_resumes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id    UUID NOT NULL REFERENCES messages(id),
    decision      TEXT NOT NULL CHECK (decision IN ('approve', 'deny')),
    remember      BOOLEAN NOT NULL DEFAULT false,
    status        TEXT NOT NULL DEFAULT 'pending',
    claimed_at    TIMESTAMPTZ,
    completed_at  TIMESTAMPTZ,
    error         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX approval_resumes_pending_idx ON approval_resumes (created_at) WHERE status = 'pending';
```

- [ ] **Step 2: Run migrations against the dev database**

Run: `cd backend && sqlx migrate run` (or just let the next `cargo test`/app boot apply it — this repo's tests use `#[sqlx::test(migrations = "../../migrations")]`, which applies every migration to a fresh throwaway DB automatically).

Verify: `docker exec backend-postgres-1 psql -U nomi -d nomi -c "\d messages"` shows `content_blocks | jsonb`, and `\d tool_permission_rules` / `\d approval_resumes` show the new tables.

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/0023_content_blocks_and_permissions.sql
git commit -m "feat: add content_blocks column and tool permission/approval-resume tables"
```

---

## Task 2: `ContentBlock` and `ToolOutcome` types

**Files:**
- Create: `backend/crates/nomi-agent-core/src/content_block.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs`
- Test: inline `#[cfg(test)]` in `content_block.rs`

**Interfaces:**
- Produces: `nomi_agent_core::ContentBlock` (enum: `FileWrite`, `FileDelete`, `TodoList`, `Table`, `ApprovalRequest`), `nomi_agent_core::ToolOutcome { display_text: String, block: Option<ContentBlock> }`, `TodoItem { id: String, text: String, status: TodoStatus }`, `TodoStatus` (`Pending`|`InProgress`|`Done`), `TableColumn { key: String, label: String }`, `TableVariant` (`Data`|`Comparison`), `ApprovalStatus` (`Pending`|`Approved`|`Denied`) — every later task that builds or renders a block uses these exact names.
- Consumes: nothing new (only `serde`, `serde_json::Value`, `uuid::Uuid`, `chrono::{DateTime, Utc}`, already workspace dependencies of `nomi-agent-core`).

- [ ] **Step 1: Write the failing test**

```rust
// backend/crates/nomi-agent-core/src/content_block.rs (bottom of file, after the types below)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_write_serializes_with_a_kind_tag_and_snake_case_fields() {
        let block = ContentBlock::FileWrite {
            project_id: uuid::Uuid::nil(),
            path: "index.html".to_string(),
            content: "<h1>hi</h1>".to_string(),
            previous_content: Some("<h1>old</h1>".to_string()),
        };
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["kind"], "file_write");
        assert_eq!(json["path"], "index.html");
        assert_eq!(json["previous_content"], "<h1>old</h1>");
    }

    #[test]
    fn file_write_with_no_previous_content_serializes_previous_content_as_null() {
        let block = ContentBlock::FileWrite {
            project_id: uuid::Uuid::nil(),
            path: "new.js".to_string(),
            content: "x".to_string(),
            previous_content: None,
        };
        let json = serde_json::to_value(&block).unwrap();
        assert!(json["previous_content"].is_null());
    }

    #[test]
    fn todo_status_serializes_as_snake_case_strings() {
        assert_eq!(serde_json::to_value(TodoStatus::InProgress).unwrap(), "in_progress");
        assert_eq!(serde_json::to_value(TodoStatus::Done).unwrap(), "done");
    }

    #[test]
    fn approval_request_round_trips_through_json() {
        let block = ContentBlock::ApprovalRequest {
            id: uuid::Uuid::nil(),
            tool_name: "delete_file".to_string(),
            description: "Delete src/old.js".to_string(),
            input: serde_json::json!({"path": "src/old.js"}),
            status: ApprovalStatus::Pending,
            decided_at: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        let parsed: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, parsed);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nomi-agent-core content_block`
Expected: FAIL — `content_block` module doesn't exist yet (compile error).

- [ ] **Step 3: Write the implementation**

```rust
// backend/crates/nomi-agent-core/src/content_block.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableVariant {
    Data,
    Comparison,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

/// Structured widget data attached to a chat message alongside its plain-text `content` fallback
/// (see `messages.content_blocks`). Named `ContentBlock` to match the design spec; callers that
/// also need `nomi_llm::ContentBlock` (a different type — LLM message content, not chat-UI
/// widgets) in the same scope should import this one aliased, e.g.
/// `use nomi_agent_core::ContentBlock as RichBlock;` (see `engine.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentBlock {
    FileWrite {
        project_id: Uuid,
        path: String,
        content: String,
        previous_content: Option<String>,
    },
    FileDelete {
        project_id: Uuid,
        path: String,
    },
    TodoList {
        items: Vec<TodoItem>,
    },
    Table {
        variant: TableVariant,
        columns: Vec<TableColumn>,
        rows: Vec<Value>,
    },
    ApprovalRequest {
        id: Uuid,
        tool_name: String,
        description: String,
        input: Value,
        status: ApprovalStatus,
        decided_at: Option<DateTime<Utc>>,
    },
}

/// What `SubAgent::execute_tool` returns on success — `display_text` is the plain-text mirror
/// stored in `messages.content` (and used for anything that reads `content` directly: search,
/// memory extraction, notifications); `block` is the structured widget, when this call warrants
/// one. Most tools return `block: None`.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    pub display_text: String,
    pub block: Option<ContentBlock>,
}

impl ToolOutcome {
    /// Convenience for the common case (no block) — most agents' tools just wrap their existing
    /// return string in this.
    pub fn text(display_text: impl Into<String>) -> Self {
        Self { display_text: display_text.into(), block: None }
    }
}
```

- [ ] **Step 4: Register the module**

```rust
// backend/crates/nomi-agent-core/src/lib.rs
pub mod content_block;
pub mod delegation;
pub mod engine;
pub mod error;
pub mod memory;
pub mod personality;
pub mod prompts;
pub mod registry;
pub mod subagent;

pub use content_block::{ApprovalStatus, ContentBlock, TableColumn, TableVariant, TodoItem, TodoStatus, ToolOutcome};
pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME};
pub use error::TurnError;
pub use registry::AgentRegistry;
pub use subagent::SubAgent;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p nomi-agent-core content_block`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-agent-core/src/content_block.rs backend/crates/nomi-agent-core/src/lib.rs
git commit -m "feat: add ContentBlock and ToolOutcome types"
```

---

## Task 3: `SubAgent::execute_tool` returns `ToolOutcome` (trait + every agent crate)

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs` (trait signature)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (dispatch call site, `post_activity_message`, `describe_tool_activity` → `describe_tool_error`)
- Modify: `backend/crates/nomi-agent-chitchat/src/lib.rs`
- Modify: `backend/crates/nomi-agent-money/src/lib.rs`
- Modify: `backend/crates/nomi-agent-personality/src/lib.rs`
- Modify: `backend/crates/nomi-agent-supervisor/src/lib.rs`
- Modify: `backend/crates/nomi-agent-planning/src/lib.rs`
- Modify: `backend/crates/nomi-agent-coding/src/lib.rs` (real diff-capture logic)
- Create: `backend/crates/nomi-agent-coding/tests/write_file_block.rs`

**Interfaces:**
- Consumes: `ToolOutcome`, `ContentBlock` from Task 2.
- Produces: `SubAgent::execute_tool(...) -> Result<ToolOutcome, String>` — every later task that calls `execute_tool` (engine.rs's dispatch, and Task 6's resume path) uses this exact signature.

- [ ] **Step 1: Write the failing tests (coding agent's diff capture)**

```rust
// backend/crates/nomi-agent-coding/tests/write_file_block.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::CodingAgent;
use nomi_agent_core::{ContentBlock, SubAgent};
use nomi_storage::LocalFsStore;

async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let chat_id = Uuid::new_v4().to_string();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(org_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Test project') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, project_id)
}

fn temp_storage() -> LocalFsStore {
    let dir = std::env::temp_dir().join(format!("nomi-coding-test-{}", Uuid::new_v4()));
    LocalFsStore::at(dir)
}

#[sqlx::test(migrations = "../../migrations")]
async fn writing_a_new_file_produces_a_file_write_block_with_no_previous_content(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    let outcome = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileWrite { path, content, previous_content, .. }) => {
            assert_eq!(path, "index.html");
            assert_eq!(content, "<h1>hi</h1>");
            assert_eq!(previous_content, None);
        }
        other => panic!("expected a FileWrite block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn overwriting_an_existing_file_captures_its_previous_content_for_the_diff(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    let input = |content: &str| {
        serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": content})
    };

    agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file", input("<h1>old</h1>")).await.unwrap();

    let outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file", input("<h1>new</h1>"))
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileWrite { content, previous_content, .. }) => {
            assert_eq!(content, "<h1>new</h1>");
            assert_eq!(previous_content, Some("<h1>old</h1>".to_string()));
        }
        other => panic!("expected a FileWrite block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_file_produces_a_file_delete_block(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "old.js", "content": "x"}),
        )
        .await
        .unwrap();

    let outcome = agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "delete_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "old.js"}),
        )
        .await
        .unwrap();

    match outcome.block {
        Some(ContentBlock::FileDelete { path, .. }) => assert_eq!(path, "old.js"),
        other => panic!("expected a FileDelete block, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn read_file_and_list_files_never_produce_a_block(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(temp_storage());
    let mut conn = pool.acquire().await.unwrap();

    agent
        .execute_tool(
            &mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "a.txt", "content": "x"}),
        )
        .await
        .unwrap();

    let read_outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "read_file", serde_json::json!({"project_id": project_id.to_string(), "path": "a.txt"}))
        .await
        .unwrap();
    assert!(read_outcome.block.is_none());

    let list_outcome = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_files", serde_json::json!({"project_id": project_id.to_string()}))
        .await
        .unwrap();
    assert!(list_outcome.block.is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nomi-agent-coding --test write_file_block`
Expected: FAIL — compile error, `execute_tool` still returns `Result<String, String>`, `ContentBlock`/`outcome.block` don't exist on that type.

- [ ] **Step 3: Change the trait signature**

```rust
// backend/crates/nomi-agent-core/src/subagent.rs
use crate::content_block::ToolOutcome;
// ... (keep existing imports)

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String>;
```
(Only the return type of this one method changes — every other line in the file is unchanged.)

- [ ] **Step 4: Update the four mechanical agent crates**

```rust
// backend/crates/nomi-agent-chitchat/src/lib.rs — only the execute_tool signature changes
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        _name: &str,
        _input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        Err("chitchat has no tools".to_string())
    }
```

```rust
// backend/crates/nomi-agent-money/src/lib.rs — dispatcher only; list_transactions/summarize_budget bodies unchanged
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "list_transactions" => list_transactions(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "summarize_budget" => summarize_budget(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

```rust
// backend/crates/nomi-agent-personality/src/lib.rs — dispatcher only; the three helper fns unchanged
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "set_personality" => set_personality(conn, session_id, agent_session_id, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "list_personality_versions" => list_personality_versions(conn, user_id).await.map(nomi_agent_core::ToolOutcome::text),
            "rollback_personality" => rollback_personality(conn, session_id, agent_session_id, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

```rust
// backend/crates/nomi-agent-supervisor/src/lib.rs — dispatcher only; list_recent_agent_activity unchanged
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "list_recent_agent_activity" => list_recent_agent_activity(conn, session_id).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("supervisor has no tool named {other}")),
        }
    }
```

```rust
// backend/crates/nomi-agent-planning/src/lib.rs — dispatcher only; create_project/write_plan unchanged
// (per spec, planning's tools get no block in v1 — only coding's write_file/delete_file do)
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "create_project" => create_project(conn, session_id, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "write_plan" => write_plan(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

- [ ] **Step 5: Give the coding agent's `write_file`/`delete_file` real block-building logic**

```rust
// backend/crates/nomi-agent-coding/src/lib.rs

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "write_file" => self.write_file(conn, user_id, input).await,
            "read_file" => self.read_file(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "list_files" => list_files(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            "delete_file" => self.delete_file(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

Replace the `write_file` method body (inside `impl CodingAgent`) with:

```rust
    async fn write_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<nomi_agent_core::ToolOutcome, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        validate_path(path)?;
        let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?;
        let content_type = guess_content_type(path);

        let key = project_file_key(project_id, path);
        // Read before overwrite: this is the only point in the whole system where the file's
        // prior content is ever observable — capturing it here is what makes the diff view
        // possible later, since a second read after the write would just see the new content.
        let previous_content = self.storage.get_object(&key).await.map_err(|e| e.to_string())?;

        self.storage.put_object(&key, content, content_type).await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()",
        )
        .bind(project_id)
        .bind(path)
        .bind(content_type)
        .bind(content.len() as i32)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE projects SET status = 'building' WHERE id = $1 AND status = 'planning'")
            .bind(project_id)
            .execute(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(nomi_agent_core::ToolOutcome {
            display_text: format!("📝 Wrote `{path}`"),
            block: Some(nomi_agent_core::ContentBlock::FileWrite {
                project_id,
                path: path.to_string(),
                content: content.to_string(),
                previous_content,
            }),
        })
    }
```

Replace the `delete_file` method body with:

```rust
    async fn delete_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<nomi_agent_core::ToolOutcome, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        validate_path(path)?;

        self.storage.delete_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())?;
        sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
            .bind(project_id)
            .bind(path)
            .execute(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(nomi_agent_core::ToolOutcome {
            display_text: format!("🗑️ Deleted `{path}`"),
            block: Some(nomi_agent_core::ContentBlock::FileDelete { project_id, path: path.to_string() }),
        })
    }
```

`read_file`, `list_files`, `parse_project_id`, `owns_project` are all unchanged (they still return `Result<String, String>` / `Result<Uuid, String>` / `Result<bool, String>` as before — only wrapped with `.map(ToolOutcome::text)` at the call site).

- [ ] **Step 6: Update `engine.rs`'s dispatch to handle `ToolOutcome`**

Replace the tool-dispatch `if`/`else if`/`else` chain inside the `for block in &response.content` loop:

```rust
                let (result_text, is_error, rich_block) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false, None)
                } else if name.as_str() == DELEGATE_TOOL_NAME {
                    let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                    let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                    let rejection = registry.find(target_agent).and_then(|t| t.validate_delegation_task(task).err());
                    let (text, err) = match rejection {
                        Some(reason) => (reason, true),
                        None => match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type(), target_agent, task, user_id).await {
                            Ok(ack) => (ack, false),
                            Err(err) => (err, true),
                        },
                    };
                    (text, err, None)
                } else {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(outcome) => (outcome.display_text, false, outcome.block),
                        Err(err) => (describe_tool_error(name, input, &err), true, None),
                    }
                };

                log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

                if agent.surfaces_activity() && name.as_str() != COMPLETE_TASK_TOOL_NAME && name.as_str() != DELEGATE_TOOL_NAME {
                    post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &result_text, rich_block.as_ref()).await;
                }

                if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                    let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    return Ok(LoopOutcome::Completed { status, summary });
                }

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content: result_text,
                    is_error,
                });
```

Change `post_activity_message`'s signature and body to accept and persist the optional rich block, and update its two other call sites (🧠 thinking, 💭 thought) to pass `None`:

```rust
async fn post_activity_message(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    content: &str,
    block: Option<&crate::content_block::ContentBlock>,
) {
    let content_blocks = block.map(|b| serde_json::json!([b]));
    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3)")
        .bind(session_id)
        .bind(content)
        .bind(&content_blocks)
        .execute(&mut **conn)
        .await;
    if let Some(publisher) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
    }
}
```
(The two existing 🧠/💭 call sites become `post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &format!("🧠 {}", thought.trim()), None).await;` and the same for 💭 — just append `, None` to each existing call.)

Replace `describe_tool_activity` entirely with a slimmed error-only version (the success-path match arms are deleted — that logic now lives in each agent's own `display_text`):

```rust
/// Templated, non-LLM description of a failed tool call — kept generic in the engine (unlike a
/// success's display_text, which each tool builds itself) since failures never carry a block and
/// the "⚠️ couldn't X — {reason}" shape is the same regardless of which crate the tool lives in.
fn describe_tool_error(tool_name: &str, input: &serde_json::Value, result: &str) -> String {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("the file");
    match tool_name {
        "write_file" => format!("⚠️ Couldn't write `{path}` — {result}"),
        "read_file" => format!("⚠️ Couldn't read `{path}` — {result}"),
        "delete_file" => format!("⚠️ Couldn't delete `{path}` — {result}"),
        "list_files" => format!("⚠️ Couldn't list project files — {result}"),
        "create_project" => format!("⚠️ Couldn't create the project — {result}"),
        "write_plan" => format!("⚠️ Couldn't save the plan — {result}"),
        other => format!("⚠️ `{other}` failed — {result}"),
    }
}
```

- [ ] **Step 7: Run the coding-agent tests to verify they pass**

Run: `cargo test -p nomi-agent-coding --test write_file_block`
Expected: PASS (4 tests)

- [ ] **Step 8: Build and test the whole workspace**

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all existing tests still pass (fix any remaining call sites the compiler flags — e.g. any test file in `nomi-agent-money`/`nomi-agent-personality`/`nomi-agent-supervisor`/`nomi-agent-planning`/`nomi-agent-chitchat` that asserts on `execute_tool`'s return value as a raw string needs `.display_text` added, e.g. `result.display_text` instead of `result`).

- [ ] **Step 9: Commit**

```bash
git add backend/crates/nomi-agent-core/src/subagent.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-chitchat/src/lib.rs backend/crates/nomi-agent-money/src/lib.rs \
        backend/crates/nomi-agent-personality/src/lib.rs backend/crates/nomi-agent-supervisor/src/lib.rs \
        backend/crates/nomi-agent-planning/src/lib.rs backend/crates/nomi-agent-coding/src/lib.rs \
        backend/crates/nomi-agent-coding/tests/write_file_block.rs
git commit -m "feat: SubAgent::execute_tool returns ToolOutcome; coding agent captures file diffs"
```

---

## Task 4: Engine-level `show_table` and `update_todos` tools

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs` (`supports_todos()` default method)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (tool definitions, dispatch branches, upsert logic)
- Modify: `backend/crates/nomi-agent-coding/src/lib.rs` (`supports_todos() -> true`)
- Modify: `backend/crates/nomi-agent-planning/src/lib.rs` (`supports_todos() -> true`)
- Test: `backend/crates/nomi-agent-core/tests/engine.rs`

**Interfaces:**
- Consumes: `ContentBlock::{Table, TodoList}`, `TodoItem`, `TodoStatus`, `TableColumn`, `TableVariant` from Task 2; `agent_sessions.state` from Task 1.
- Produces: `SHOW_TABLE_TOOL_NAME`, `UPDATE_TODOS_TOOL_NAME` constants (both `pub` from `nomi_agent_core`) — Task 5/6 reference these names when deciding what's never permission-gated.

- [ ] **Step 1: Write the failing tests**

Add to `backend/crates/nomi-agent-core/tests/engine.rs` (uses the file's existing `seed_session`/`seed_agent_session`/`text_response`/`tool_use_response` helpers and `TestAgent` — give `TestAgent` a `supports_todos` override first, see Step 4):

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn show_table_is_available_to_every_agent_and_posts_a_table_block(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response(
            "t1",
            "show_table",
            serde_json::json!({
                "variant": "data",
                "columns": [{"key": "name", "label": "Name"}],
                "rows": [{"name": "Alice"}]
            }),
        ),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let content_blocks: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT content_blocks FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NULL ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let blocks = content_blocks.unwrap();
    assert_eq!(blocks[0]["kind"], "table");
    assert_eq!(blocks[0]["rows"][0]["name"], "Alice");
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_todos_upserts_the_same_message_instead_of_creating_a_new_one_each_time(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let todo_input = |status: &str| {
        serde_json::json!({"items": [{"id": "1", "text": "Write index.html", "status": status}]})
    };

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "update_todos", todo_input("pending")),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let provider2 = FakeLlmProvider::sequence(vec![
        tool_use_response("t2", "update_todos", todo_input("done")),
        text_response("ok again", StopReason::EndTurn),
    ]);
    run_agent_turn(&mut conn, None, &provider2, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let todo_message_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks IS NOT NULL AND content_blocks @> '[{\"kind\": \"todo_list\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(todo_message_count, 1, "the second update_todos call should update the same message, not create a second one");

    let status: String = sqlx::query_scalar(
        "SELECT content_blocks->0->'items'->0->>'status' FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"todo_list\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "done");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nomi-agent-core --test engine show_table` and `update_todos`
Expected: FAIL — `show_table`/`update_todos` aren't real tools yet, `TestAgent` has no `supports_todos`.

- [ ] **Step 3: Add `supports_todos()` to the trait**

```rust
// backend/crates/nomi-agent-core/src/subagent.rs — added after surfaces_activity()

    /// When true, run_agent_turn gives this agent the engine-level `update_todos` tool for
    /// maintaining a live multi-step task checklist. Off by default — most agents don't run
    /// long enough multi-step builds to need one.
    fn supports_todos(&self) -> bool {
        false
    }
```

- [ ] **Step 4: Give the test agent (and the real coding/planning agents) the flag**

```rust
// backend/crates/nomi-agent-core/tests/engine.rs — inside `impl SubAgent for TestAgent`, add:
    fn supports_todos(&self) -> bool {
        true
    }
```

```rust
// backend/crates/nomi-agent-coding/src/lib.rs — inside `impl SubAgent for CodingAgent`, add:
    fn supports_todos(&self) -> bool {
        true
    }
```

```rust
// backend/crates/nomi-agent-planning/src/lib.rs — inside `impl SubAgent for PlanningAgent`, add:
    fn supports_todos(&self) -> bool {
        true
    }
```

- [ ] **Step 5: Add the two tool name constants and their definitions in `engine.rs`**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — alongside the existing COMPLETE_TASK_TOOL_NAME/DELEGATE_TOOL_NAME
pub const SHOW_TABLE_TOOL_NAME: &str = "show_table";
pub const UPDATE_TODOS_TOOL_NAME: &str = "update_todos";

fn show_table_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: SHOW_TABLE_TOOL_NAME.to_string(),
        description: "Show the user a table of structured data — either a plain data table or a side-by-side comparison of a few items across criteria.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "variant": {"type": "string", "enum": ["data", "comparison"]},
                "columns": {
                    "type": "array",
                    "items": {"type": "object", "properties": {"key": {"type": "string"}, "label": {"type": "string"}}, "required": ["key", "label"]}
                },
                "rows": {"type": "array", "items": {"type": "object"}}
            },
            "required": ["variant", "columns", "rows"]
        }),
    }
}

fn update_todos_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: UPDATE_TODOS_TOOL_NAME.to_string(),
        description: "Set or replace your current multi-step task checklist, shown live to the user. Call this again with the full updated list whenever a step's status changes.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "text": {"type": "string"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "done"]}
                        },
                        "required": ["id", "text", "status"]
                    }
                }
            },
            "required": ["items"]
        }),
    }
}
```

- [ ] **Step 6: Register the tools in `run_agent_turn`**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — right after the existing
// `if agent.can_delegate() { ... }` block near the top of run_agent_turn:
    tools.push(show_table_tool_definition());
    if agent.supports_todos() {
        tools.push(update_todos_tool_definition());
    }
```

- [ ] **Step 7: Add the parsing/upsert helper functions**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — new functions, anywhere below run_agent_turn

fn parse_todo_items(input: &serde_json::Value) -> Result<Vec<crate::content_block::TodoItem>, String> {
    let items = input.get("items").and_then(|v| v.as_array()).ok_or("items is required")?;
    items
        .iter()
        .map(|item| {
            let id = item.get("id").and_then(|v| v.as_str()).ok_or("each item needs an id")?.to_string();
            let text = item.get("text").and_then(|v| v.as_str()).ok_or("each item needs text")?.to_string();
            let status = match item.get("status").and_then(|v| v.as_str()) {
                Some("pending") => crate::content_block::TodoStatus::Pending,
                Some("in_progress") => crate::content_block::TodoStatus::InProgress,
                Some("done") => crate::content_block::TodoStatus::Done,
                _ => return Err("each item's status must be pending, in_progress, or done".to_string()),
            };
            Ok(crate::content_block::TodoItem { id, text, status })
        })
        .collect()
}

fn todo_list_display_text(items: &[crate::content_block::TodoItem]) -> String {
    use crate::content_block::TodoStatus;
    let mut out = String::from("📋 To-do list:\n");
    for item in items {
        let mark = match item.status {
            TodoStatus::Done => "[x]",
            TodoStatus::InProgress => "[~]",
            TodoStatus::Pending => "[ ]",
        };
        out.push_str(&format!("{mark} {}\n", item.text));
    }
    out
}

/// Finds the most recent todo-list message for this agent session (tracked via
/// `agent_sessions.state->>'todo_message_id'`) and updates it in place; inserts a fresh message
/// and records its id into `state` the first time this agent session ever calls update_todos.
/// Unlike every other block-producing tool, this posts/updates the message itself rather than
/// returning a block for engine.rs's generic post-at-the-bottom-of-the-loop path, since that
/// path only ever inserts — it has no notion of "update this existing row instead".
async fn upsert_todo_list(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    items: Vec<crate::content_block::TodoItem>,
) -> Result<String, String> {
    let display_text = todo_list_display_text(&items);
    let block = crate::content_block::ContentBlock::TodoList { items };
    let content_blocks = serde_json::json!([block]);

    let existing_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT (state->>'todo_message_id')::uuid FROM agent_sessions WHERE id = $1",
    )
    .bind(agent_session_id)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    match existing_id {
        Some(message_id) => {
            sqlx::query("UPDATE messages SET content = $1, content_blocks = $2 WHERE id = $3")
                .bind(&display_text)
                .bind(&content_blocks)
                .bind(message_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
        }
        None => {
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(&display_text)
            .bind(&content_blocks)
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query("UPDATE agent_sessions SET state = state || jsonb_build_object('todo_message_id', $1::text) WHERE id = $2")
                .bind(message_id.to_string())
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    if let Some(publisher) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
    }

    Ok("todo list updated".to_string())
}

fn parse_table_input(
    input: &serde_json::Value,
) -> Result<(crate::content_block::TableVariant, Vec<crate::content_block::TableColumn>, Vec<serde_json::Value>), String> {
    use crate::content_block::{TableColumn, TableVariant};
    let variant = match input.get("variant").and_then(|v| v.as_str()) {
        Some("data") => TableVariant::Data,
        Some("comparison") => TableVariant::Comparison,
        _ => return Err("variant must be 'data' or 'comparison'".to_string()),
    };
    let columns: Vec<TableColumn> = input
        .get("columns")
        .and_then(|v| v.as_array())
        .ok_or("columns is required")?
        .iter()
        .map(|c| {
            let key = c.get("key").and_then(|v| v.as_str()).ok_or("each column needs a key")?.to_string();
            let label = c.get("label").and_then(|v| v.as_str()).ok_or("each column needs a label")?.to_string();
            Ok(TableColumn { key, label })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let rows = input.get("rows").and_then(|v| v.as_array()).ok_or("rows is required")?.clone();
    Ok((variant, columns, rows))
}

fn table_display_text(variant: &crate::content_block::TableVariant, row_count: usize) -> String {
    use crate::content_block::TableVariant;
    match variant {
        TableVariant::Data => format!("📊 Showed a table with {row_count} row(s)"),
        TableVariant::Comparison => format!("📊 Compared {row_count} item(s)"),
    }
}
```

- [ ] **Step 8: Wire the two new branches into the dispatch chain, and fix the posting condition**

In the same `if name.as_str() == COMPLETE_TASK_TOOL_NAME { ... } else if ... == DELEGATE_TOOL_NAME { ... } else { agent.execute_tool(...) }` chain from Task 3, insert two more branches before the final `else`:

```rust
                } else if name.as_str() == SHOW_TABLE_TOOL_NAME {
                    match parse_table_input(input) {
                        Ok((variant, columns, rows)) => {
                            let text = table_display_text(&variant, rows.len());
                            let block = crate::content_block::ContentBlock::Table { variant, columns, rows };
                            (text, false, Some(block))
                        }
                        Err(err) => (err, true, None),
                    }
                } else if name.as_str() == UPDATE_TODOS_TOOL_NAME {
                    match parse_todo_items(input) {
                        Ok(items) => match upsert_todo_list(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, items).await {
                            Ok(text) => (text, false, None),
                            Err(err) => (err, true, None),
                        },
                        Err(err) => (err, true, None),
                    }
                } else {
```
(This goes between the existing `DELEGATE_TOOL_NAME` arm and the final `else` that calls `agent.execute_tool`.)

Then change the posting condition right below (the `if agent.surfaces_activity() && ...` check) to also fire unconditionally for `show_table`, and to never double-post for `update_todos` (which already posted/updated inside `upsert_todo_list` above):

```rust
                let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                    || (agent.surfaces_activity()
                        && name.as_str() != COMPLETE_TASK_TOOL_NAME
                        && name.as_str() != DELEGATE_TOOL_NAME
                        && name.as_str() != UPDATE_TODOS_TOOL_NAME);
                if should_post {
                    post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &result_text, rich_block.as_ref()).await;
                }
```
(This replaces the `if agent.surfaces_activity() && name.as_str() != COMPLETE_TASK_TOOL_NAME && name.as_str() != DELEGATE_TOOL_NAME { post_activity_message(...) }` block from Task 3.)

- [ ] **Step 9: Run tests to verify they pass**

Run: `cargo test -p nomi-agent-core --test engine`
Expected: PASS (all tests, including the 2 new ones)

- [ ] **Step 10: Commit**

```bash
git add backend/crates/nomi-agent-core/src/subagent.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-core/tests/engine.rs backend/crates/nomi-agent-coding/src/lib.rs \
        backend/crates/nomi-agent-planning/src/lib.rs
git commit -m "feat: add engine-level show_table and update_todos tools"
```

---

## Task 5: Permission checking + pause (pre-scan, `LoopOutcome::AwaitingApproval`)

**Files:**
- Create: `backend/crates/nomi-agent-core/src/permissions.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs` (export `permissions`)
- Modify: `backend/crates/nomi-llm/src/types.rs` (serde derives on `LlmRole`/`ContentBlock`/`LlmMessage` — needed to persist paused conversation state as JSONB)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (pre-scan, deny handling, `LoopOutcome::AwaitingApproval`)
- Test: `backend/crates/nomi-agent-core/tests/engine.rs`

**Interfaces:**
- Consumes: `tool_permission_rules` from Task 1, `ApprovalStatus`/`ContentBlock::ApprovalRequest` from Task 2, `SHOW_TABLE_TOOL_NAME`/`UPDATE_TODOS_TOOL_NAME` from Task 4.
- Produces: `LoopOutcome::AwaitingApproval { message_id: Uuid }` — Task 6's resume path and Task 7's `TurnOutcome` handling match on this variant. `permissions::{PermissionDecision, check_tool_permission, remember_decision}` — Task 6's resume endpoint calls `remember_decision`.

- [ ] **Step 1: Add serde derives so `LlmMessage`/`ContentBlock`/`LlmRole` can round-trip through JSONB**

```rust
// backend/crates/nomi-llm/src/types.rs

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LlmRole {
```
(was `#[derive(Debug, Clone, PartialEq, Eq)]` — only the derive line changes, nothing else in the enum body)

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContentBlock {
```
(was `#[derive(Debug, Clone, PartialEq)]`)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmMessage {
```
(was `#[derive(Debug, Clone)]` — this is the *second* `#[derive(Debug, Clone)]` in the file, directly above `pub struct LlmMessage`; the one above `pub struct ToolDefinition` right after it stays untouched)

Run: `cargo build -p nomi-llm` — expect a clean build (every variant/field already implements `Serialize`/`Deserialize`: `String`, `Value`, `Uuid`, `Vec<T>`, `Option<T>` all do).

- [ ] **Step 2: Write the failing tests**

Add to `backend/crates/nomi-agent-core/tests/engine.rs`:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn an_unmatched_tool_call_pauses_the_turn_and_never_executes(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response("t1", "echo", serde_json::json!({"x": 1}))]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let message_id = match outcome {
        LoopOutcome::AwaitingApproval { message_id } => message_id,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };

    let content_blocks: serde_json::Value = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content_blocks[0]["kind"], "approval_request");
    assert_eq!(content_blocks[0]["status"], "pending");
    assert_eq!(content_blocks[0]["tool_name"], "echo");

    let state: serde_json::Value = sqlx::query_scalar("SELECT state FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state["paused_for_approval"], true);
    assert_eq!(state["pending_tool_use_id"], "t1");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_allow_rule_lets_the_tool_execute_without_pausing(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply { text: "Done!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 });
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_deny_rule_blocks_the_tool_but_lets_the_turn_continue(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'deny')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Okay, I won't.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply { text: "Okay, I won't.".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 });

    let requests = provider.received_requests.lock().unwrap();
    let second_request_text = format!("{:?}", requests[1].messages);
    assert!(second_request_text.contains("Denied by your permission rules."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn show_table_and_update_todos_are_never_gated(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // No permission rules at all — if these were gated the same way `echo` is, this would pause.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "show_table", serde_json::json!({"variant": "data", "columns": [], "rows": []})),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p nomi-agent-core --test engine`
Expected: FAIL — compile error (`LoopOutcome::AwaitingApproval` doesn't exist, `permissions` module doesn't exist).

- [ ] **Step 4: Write `permissions.rs`**

```rust
// backend/crates/nomi-agent-core/src/permissions.rs
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

/// `*`-only glob match (no `?`, no character classes) — enough for path prefixes like `src/*`
/// or `*.env`. Matched in application code, not SQL: Postgres has no glob operator.
fn glob_match(pattern: &str, text: &str) -> bool {
    fn helper(p: &[u8], t: &[u8]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some(b'*'), _) => helper(&p[1..], t) || (!t.is_empty() && helper(p, &t[1..])),
            (Some(pc), Some(tc)) if pc == tc => helper(&p[1..], &t[1..]),
            _ => false,
        }
    }
    helper(pattern.as_bytes(), text.as_bytes())
}

/// Looks up `tool_permission_rules` for `(user_id, tool_name)`. A rule with a `path_pattern`
/// only applies when the tool call's `input.path` matches it; a rule with `path_pattern = NULL`
/// matches any call to that tool. A pattern match beats a wildcard match when both exist. No
/// matching rule at all → `Ask`.
pub async fn check_tool_permission(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    tool_name: &str,
    input: &Value,
) -> PermissionDecision {
    let path = input.get("path").and_then(|v| v.as_str());

    let rules: Vec<(Option<String>, String)> = sqlx::query_as(
        "SELECT path_pattern, decision FROM tool_permission_rules WHERE user_id = $1 AND tool_name = $2",
    )
    .bind(user_id)
    .bind(tool_name)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_default();

    let mut pattern_match: Option<String> = None;
    let mut wildcard_match: Option<String> = None;

    for (pattern, decision) in rules {
        match (&pattern, path) {
            (Some(p), Some(path)) if glob_match(p, path) => pattern_match = Some(decision),
            (None, _) => wildcard_match = Some(decision),
            _ => {}
        }
    }

    match pattern_match.or(wildcard_match).as_deref() {
        Some("allow") => PermissionDecision::Allow,
        Some("deny") => PermissionDecision::Deny,
        _ => PermissionDecision::Ask,
    }
}

/// Persists an "always allow/deny" decision from an approval card so future calls to this same
/// tool (optionally scoped to a path pattern) skip the approval prompt.
pub async fn remember_decision(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    tool_name: &str,
    path_pattern: Option<&str>,
    decision: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, path_pattern, decision) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(tool_name)
        .bind(path_pattern)
        .bind(decision)
        .execute(&mut **conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_star_matches_any_suffix() {
        assert!(glob_match("src/*", "src/index.html"));
        assert!(glob_match("*.env", "backend/.env"));
        assert!(!glob_match("src/*", "lib/index.html"));
    }

    #[test]
    fn glob_exact_match_with_no_star() {
        assert!(glob_match("index.html", "index.html"));
        assert!(!glob_match("index.html", "index.htm"));
    }
}
```

```rust
// backend/crates/nomi-agent-core/src/lib.rs — add alongside the other `pub mod` lines
pub mod permissions;
```

- [ ] **Step 5: Add `LoopOutcome::AwaitingApproval`**

```rust
// backend/crates/nomi-agent-core/src/engine.rs
#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply { text: String, memory_ids_used: Vec<Uuid>, input_tokens: u32, output_tokens: u32 },
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
}
```

- [ ] **Step 6: Add the gateable-tool check and pending-action description helper**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — new functions

/// Tools that are never permission-gated: engine-level bookkeeping (complete_task,
/// delegate_to_agent, show_table, update_todos) — none of these touch anything a user would
/// want to approve/deny.
fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != SHOW_TABLE_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
}

fn describe_pending_action(tool_name: &str, input: &serde_json::Value) -> String {
    let path = input.get("path").and_then(|v| v.as_str());
    match (tool_name, path) {
        ("delete_file", Some(path)) => format!("Delete {path}"),
        ("write_file", Some(path)) => format!("Overwrite {path}"),
        (other, Some(path)) => format!("Run {other} on {path}"),
        (other, None) => format!("Run {other}"),
    }
}
```

- [ ] **Step 7: Insert the pre-scan**

In `run_agent_turn`, right after the `if agent.surfaces_activity() { ... }` 💭-thought-posting block and before `let mut tool_results = Vec::new();`, insert:

```rust
        // Permission pre-scan: before executing ANY tool call from this response, check whether
        // any of them need approval. If so, none of them run yet — the whole batch pauses on the
        // first one needing a decision. Resume (Task 6) re-enters this exact scan-then-execute
        // logic from scratch once a decision is made, so nothing here needs to track partial
        // progress within a batch.
        let pending_tool_use_blocks: Vec<ContentBlock> =
            response.content.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).cloned().collect();

        for block in &pending_tool_use_blocks {
            if let ContentBlock::ToolUse { id, name, input, .. } = block {
                if !is_gateable(name) {
                    continue;
                }
                let decision = permissions::check_tool_permission(conn, user_id, name, input).await;
                if !matches!(decision, permissions::PermissionDecision::Ask) {
                    continue;
                }

                let description = describe_pending_action(name, input);
                let approval_block = crate::content_block::ContentBlock::ApprovalRequest {
                    id: Uuid::new_v4(),
                    tool_name: name.clone(),
                    description: description.clone(),
                    input: input.clone(),
                    status: crate::content_block::ApprovalStatus::Pending,
                    decided_at: None,
                };
                let content_blocks = serde_json::json!([approval_block]);
                let message_id: Option<Uuid> = sqlx::query_scalar(
                    "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
                )
                .bind(session_id)
                .bind(format!("⏳ {description}"))
                .bind(&content_blocks)
                .fetch_one(&mut **conn)
                .await
                .ok();

                let Some(message_id) = message_id else {
                    return Err(TurnError::ToolLoopExceeded);
                };

                if let Some((publisher, _)) = mqtt {
                    let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
                }

                let state_patch = serde_json::json!({
                    "paused_for_approval": true,
                    "pending_approval_message_id": message_id.to_string(),
                    "pending_tool_use_id": id,
                    "tool_use_blocks": pending_tool_use_blocks,
                    "messages": messages,
                });
                let _ = sqlx::query("UPDATE agent_sessions SET state = state || $1 WHERE id = $2")
                    .bind(&state_patch)
                    .bind(agent_session_id)
                    .execute(&mut **conn)
                    .await;

                return Ok(LoopOutcome::AwaitingApproval { message_id });
            }
        }
```

- [ ] **Step 8: Handle the "deny" case in the execution chain**

Insert one more arm into the existing `if/else if` chain (from Tasks 3–4), between the `UPDATE_TODOS_TOOL_NAME` arm and the final `else`:

```rust
                } else if matches!(permissions::check_tool_permission(conn, user_id, name, input).await, permissions::PermissionDecision::Deny) {
                    ("Denied by your permission rules.".to_string(), true, None)
                } else {
```
(Everything reaching this arm has already fallen through `COMPLETE_TASK_TOOL_NAME`/`DELEGATE_TOOL_NAME`/`SHOW_TABLE_TOOL_NAME`/`UPDATE_TODOS_TOOL_NAME`, so it's necessarily a real, gateable domain tool — no extra `is_gateable` check needed here.)

Add `use crate::permissions;` near the top of `engine.rs` alongside the other `use crate::...` lines.

- [ ] **Step 9: Run tests to verify they pass**

Run: `cargo test -p nomi-agent-core --test engine`
Expected: PASS (all tests, including the 4 new ones)

- [ ] **Step 10: Build and test the whole workspace**

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all tests pass.

- [ ] **Step 11: Commit**

```bash
git add backend/crates/nomi-agent-core/src/permissions.rs backend/crates/nomi-agent-core/src/lib.rs \
        backend/crates/nomi-agent-core/src/engine.rs backend/crates/nomi-agent-core/tests/engine.rs \
        backend/crates/nomi-llm/src/types.rs
git commit -m "feat: permission-gate tool calls; pause the turn loop pending approval"
```

---

## Task 6: Resume subsystem — `resolve_tool_batch`, `approval_resumes` queue, resume worker loop, approval endpoint

This is the largest task: it (a) refactors Task 5's inline pre-scan/execution logic into a public, reusable `resolve_tool_batch` function so both a fresh turn and a resumed one share the exact same logic, (b) adds a small queue parallel to `turn_jobs` for resuming, (c) adds the worker loop that claims it, and (d) adds the HTTP endpoint that enqueues a resume. It also threads a real `message_id` through `TurnOutcome` (previously always `Uuid::nil()` in `TurnCompleted` — see Task 7, which depends on this).

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/error.rs` (`TurnError::ApprovalNoLongerPending`)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (extract `resolve_tool_batch`/`ToolBatchOutcome`, make `pub`)
- Modify: `backend/crates/nomi-agent-core/src/lib.rs` (export the two new public items)
- Create: `backend/crates/nomi-turn/src/approval.rs`
- Modify: `backend/crates/nomi-turn/src/lib.rs` (`TurnOutcome.message_id`, `finish_agent_turn`, `resume_paused_turn`, module declaration)
- Modify: `backend/crates/nomi-server/src/worker.rs` (second claim loop)
- Modify: `backend/crates/nomi-server/src/routes/sessions.rs` (`resolve_approval` handler)
- Modify: `backend/crates/nomi-server/src/app.rs` (route wiring)
- Test: `backend/crates/nomi-agent-core/tests/engine.rs`, `backend/crates/nomi-server/tests/` (new file)

**Interfaces:**
- Consumes: `LoopOutcome::AwaitingApproval`, `permissions::{check_tool_permission, remember_decision}`, `agent_sessions.state` shape from Task 5; `approval_resumes` table from Task 1.
- Produces: `nomi_agent_core::engine::{resolve_tool_batch, ToolBatchOutcome}` (pub), `nomi_turn::resume_paused_turn`, `TurnOutcome.message_id: Option<Uuid>` — Task 7 uses this field when publishing `TurnCompleted`.

- [ ] **Step 1: Extract `resolve_tool_batch` from `run_agent_turn` and make it `pub`**

Replace the pre-scan block and the tool-execution loop inside `run_agent_turn` (everything from the `let pending_tool_use_blocks = ...` line through the closing of the `for block in &response.content { ... }` execution loop, i.e. all of Task 5 Step 7 plus Tasks 3–4's execution chain) with a single call:

```rust
        let pending_tool_use_blocks: Vec<ContentBlock> =
            response.content.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).cloned().collect();

        match resolve_tool_batch(conn, mqtt, registry, agent, session_id, agent_session_id, user_id, &pending_tool_use_blocks, &messages, None).await? {
            ToolBatchOutcome::AwaitingApproval { message_id } => return Ok(LoopOutcome::AwaitingApproval { message_id }),
            ToolBatchOutcome::Completed { status, summary } => return Ok(LoopOutcome::Completed { status, summary }),
            ToolBatchOutcome::Resolved(tool_results) => {
                messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
            }
        }
```

Then add the extracted logic as its own `pub` function and `pub enum` (anywhere below `run_agent_turn`), combining Task 5's pre-scan with Tasks 3–4's execution chain, parameterized by `tool_use_blocks`/`conversation_so_far`/`already_decided` instead of reading `response.content`/`messages` directly:

```rust
/// The result of resolving one response's worth of tool_use blocks.
#[derive(Debug)]
pub enum ToolBatchOutcome {
    Resolved(Vec<ContentBlock>),
    Completed { status: String, summary: String },
    AwaitingApproval { message_id: Uuid },
}

/// Resolves every ToolUse block in `tool_use_blocks`, in order. `already_decided`, when set, is
/// `(tool_use_id, approved)` for a block whose approval was just resolved externally (a user
/// clicked Approve/Deny on its card) — that one block skips the permission check and uses the
/// given decision directly; every other block still goes through the normal
/// check-permission-then-execute-or-pause path, which may itself pause again on a *different*
/// block — handled identically to the very first pause (see nomi-turn's resume path, which calls
/// this same function again when that happens).
#[allow(clippy::too_many_arguments)]
pub async fn resolve_tool_batch(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    tool_use_blocks: &[ContentBlock],
    conversation_so_far: &[LlmMessage],
    already_decided: Option<(&str, bool)>,
) -> Result<ToolBatchOutcome, TurnError> {
    for (index, block) in tool_use_blocks.iter().enumerate() {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            if already_decided.map(|(decided_id, _)| decided_id == id.as_str()).unwrap_or(false) {
                continue;
            }
            if !is_gateable(name) {
                continue;
            }
            let decision = permissions::check_tool_permission(conn, user_id, name, input).await;
            if !matches!(decision, permissions::PermissionDecision::Ask) {
                continue;
            }

            let description = describe_pending_action(name, input);
            let approval_block = crate::content_block::ContentBlock::ApprovalRequest {
                id: Uuid::new_v4(),
                tool_name: name.clone(),
                description: description.clone(),
                input: input.clone(),
                status: crate::content_block::ApprovalStatus::Pending,
                decided_at: None,
            };
            let content_blocks = serde_json::json!([approval_block]);
            let message_id: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(format!("⏳ {description}"))
            .bind(&content_blocks)
            .fetch_one(&mut **conn)
            .await
            .ok();

            let Some(message_id) = message_id else {
                return Err(TurnError::ToolLoopExceeded);
            };

            if let Some((publisher, _)) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::SessionActivity { session_id }).await;
            }

            // Only the not-yet-resolved blocks from this point on need to survive into the next
            // resume — everything before `index` is already resolved by the time this is reached
            // (either during THIS call's own execution pass below, on a later resume, or never
            // executed at all on the very first pass, where nothing runs until the pre-scan
            // finds no more "Ask" blocks).
            let state_patch = serde_json::json!({
                "paused_for_approval": true,
                "pending_approval_message_id": message_id.to_string(),
                "pending_tool_use_id": id,
                "tool_use_blocks": &tool_use_blocks[index..],
                "messages": conversation_so_far,
            });
            let _ = sqlx::query("UPDATE agent_sessions SET state = state || $1 WHERE id = $2")
                .bind(&state_patch)
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await;

            return Ok(ToolBatchOutcome::AwaitingApproval { message_id });
        }
    }

    let mut tool_results = Vec::new();
    for block in tool_use_blocks {
        if let ContentBlock::ToolUse { id, name, input, .. } = block {
            let (result_text, is_error, rich_block) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false, None)
            } else if name.as_str() == DELEGATE_TOOL_NAME {
                let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                let rejection = registry.find(target_agent).and_then(|t| t.validate_delegation_task(task).err());
                let (text, err) = match rejection {
                    Some(reason) => (reason, true),
                    None => match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type(), target_agent, task, user_id).await {
                        Ok(ack) => (ack, false),
                        Err(err) => (err, true),
                    },
                };
                (text, err, None)
            } else if name.as_str() == SHOW_TABLE_TOOL_NAME {
                match parse_table_input(input) {
                    Ok((variant, columns, rows)) => {
                        let text = table_display_text(&variant, rows.len());
                        let block = crate::content_block::ContentBlock::Table { variant, columns, rows };
                        (text, false, Some(block))
                    }
                    Err(err) => (err, true, None),
                }
            } else if name.as_str() == UPDATE_TODOS_TOOL_NAME {
                match parse_todo_items(input) {
                    Ok(items) => match upsert_todo_list(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, items).await {
                        Ok(text) => (text, false, None),
                        Err(err) => (err, true, None),
                    },
                    Err(err) => (err, true, None),
                }
            } else if already_decided.map(|(decided_id, _)| decided_id == id.as_str()).unwrap_or(false) {
                let approved = already_decided.unwrap().1;
                if approved {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(outcome) => (outcome.display_text, false, outcome.block),
                        Err(err) => (describe_tool_error(name, input, &err), true, None),
                    }
                } else {
                    ("Denied by your permission rules.".to_string(), true, None)
                }
            } else if matches!(permissions::check_tool_permission(conn, user_id, name, input).await, permissions::PermissionDecision::Deny) {
                ("Denied by your permission rules.".to_string(), true, None)
            } else {
                match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                    Ok(outcome) => (outcome.display_text, false, outcome.block),
                    Err(err) => (describe_tool_error(name, input, &err), true, None),
                }
            };

            log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

            let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME);
            if should_post {
                post_activity_message(conn, mqtt.map(|(p, _)| p), session_id, &result_text, rich_block.as_ref()).await;
            }

            if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
                let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                return Ok(ToolBatchOutcome::Completed { status, summary });
            }

            tool_results.push(ContentBlock::ToolResult { tool_use_id: id.clone(), content: result_text, is_error });
        }
    }

    Ok(ToolBatchOutcome::Resolved(tool_results))
}
```

Delete the old inline pre-scan and execution-chain code that Steps of Tasks 3–5 built directly inside `run_agent_turn`'s loop body — it now all lives in `resolve_tool_batch` above, called from the one `match` in `run_agent_turn`.

- [ ] **Step 2: Add `TurnError::ApprovalNoLongerPending`**

```rust
// backend/crates/nomi-agent-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("llm call failed: {0}")]
    LlmCallFailed(#[from] nomi_llm::LlmError),
    #[error("tool-calling loop exceeded its turn limit without completing")]
    ToolLoopExceeded,
    #[error("this action is no longer pending approval")]
    ApprovalNoLongerPending,
}
```

- [ ] **Step 3: Re-export the two new public items**

```rust
// backend/crates/nomi-agent-core/src/lib.rs
pub use engine::{run_agent_turn, resolve_tool_batch, LoopOutcome, ToolBatchOutcome, COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME};
```
(was `pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME};` — adds `resolve_tool_batch` and `ToolBatchOutcome`)

- [ ] **Step 4: Run existing tests to verify the refactor is behavior-preserving**

Run: `cargo test -p nomi-agent-core --test engine`
Expected: PASS — every test from Tasks 3–5 still passes unchanged (this refactor moves code, it doesn't change behavior).

- [ ] **Step 5: Write the `approval_resumes` queue module**

```rust
// backend/crates/nomi-turn/src/approval.rs
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use nomi_agent_core::TurnError;

pub struct ClaimedApprovalResume {
    pub id: Uuid,
    pub message_id: Uuid,
    pub decision: String,
    pub remember: bool,
}

pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<Uuid, TurnError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO approval_resumes (message_id, decision, remember) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(message_id)
    .bind(decision)
    .bind(remember)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('approval_resumes_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedApprovalResume>, TurnError> {
    let row: Option<(Uuid, Uuid, String, bool)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM approval_resumes \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE approval_resumes SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, message_id, decision, remember",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, message_id, decision, remember)| ClaimedApprovalResume { id, message_id, decision, remember }))
}

pub async fn mark_completed(pool: &PgPool, id: Uuid) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'completed', completed_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}
```

```rust
// backend/crates/nomi-turn/src/lib.rs — add near the top with the other `pub mod` lines
pub mod approval;
```

- [ ] **Step 6: Thread a real `message_id` through `TurnOutcome`, and extract `finish_agent_turn`**

```rust
// backend/crates/nomi-turn/src/lib.rs
#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
    pub message_id: Option<Uuid>,
}
```
(was `pub struct TurnOutcome { pub session_id: Uuid, pub reply: String }` — update both existing `Ok(TurnOutcome { session_id, reply })` construction sites in `handle_inbound_message`/`process_turn` to `Ok(TurnOutcome { session_id, reply, message_id })`, threading the new return value from `run_locked_turn` described below.)

Replace `run_subagent_turn`'s body (the `match outcome { LoopOutcome::Reply { ... } => { ... } LoopOutcome::Completed { ... } => { ... } }` block) with a call to a new shared helper, and change `run_subagent_turn`'s (and therefore `run_locked_turn`'s) return type from `Result<String, TurnError>` to `Result<(String, Option<Uuid>), TurnError>`:

```rust
async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    agent: &dyn nomi_agent_core::SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<(String, Option<Uuid>), TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = nomi_agent_core::run_agent_turn(
        conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id, messages, SUBAGENT_MAX_TOKENS,
    )
    .await?;

    finish_agent_turn(conn, session_id, agent_session_id, agent, outcome).await
}

/// Persists a `LoopOutcome` (insert the final reply / completion message, record bookkeeping
/// events, update `last_activity_at`) and returns the reply text plus the persisted message's
/// id (`None` for `AwaitingApproval`, which already inserted its own approval message inside
/// `resolve_tool_batch` — nothing more to persist here). Shared by the fresh-turn path
/// (`run_subagent_turn`) and the resume path (`resume_paused_turn`) below, since both end up
/// with a `LoopOutcome` to finish the same way.
#[allow(clippy::too_many_arguments)]
async fn finish_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent: &dyn nomi_agent_core::SubAgent,
    outcome: nomi_agent_core::LoopOutcome,
) -> Result<(String, Option<Uuid>), TurnError> {
    match outcome {
        nomi_agent_core::LoopOutcome::Reply { text: reply_text, memory_ids_used, input_tokens, output_tokens } => {
            let mut tx = conn.begin().await?;
            let reply_message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
            )
            .bind(session_id)
            .bind(&reply_text)
            .fetch_one(&mut *tx)
            .await?;
            for memory_id in &memory_ids_used {
                sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
                    .bind(reply_message_id)
                    .bind(memory_id)
                    .execute(&mut *tx)
                    .await?;
            }
            let agent_session_id_for_event = if agent_session_id == session_id { None } else { Some(agent_session_id) };
            sqlx::query(
                "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'AgentReplied', $4)",
            )
            .bind(session_id)
            .bind(agent_session_id_for_event)
            .bind(agent.agent_type())
            .bind(serde_json::json!({"input_tokens": input_tokens, "output_tokens": output_tokens}))
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE agent_sessions SET last_activity_at = now() WHERE id = $1")
                .bind(agent_session_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok((reply_text, Some(reply_message_id)))
        }
        nomi_agent_core::LoopOutcome::Completed { status, summary } => {
            let mut tx = conn.begin().await?;
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
            )
            .bind(session_id)
            .bind(&summary)
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type(), &status, &summary).await?;

            Ok((summary, Some(message_id)))
        }
        nomi_agent_core::LoopOutcome::AwaitingApproval { .. } => Ok(("Waiting for approval.".to_string(), None)),
    }
}
```

Update `run_locked_turn`'s return type from `Result<String, TurnError>` to `Result<(String, Option<Uuid>), TurnError>` and both of its `run_subagent_turn(...)` call sites (they already just `return`/tail-call `run_subagent_turn(...).await`, so no other change needed there — the type flows through).

Update `handle_inbound_message` and `process_turn`'s `match result { Ok(reply) => ... }` arms — `reply` is now `(String, Option<Uuid>)`:

```rust
        Ok((reply, message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id })
        }
```
(Apply this same change to both `handle_inbound_message` and `process_turn` — each has one `Ok(reply) => { ...; Ok(TurnOutcome { session_id, reply }) }` arm today.)

- [ ] **Step 7: Write `resume_paused_turn`**

```rust
// backend/crates/nomi-turn/src/lib.rs
use nomi_llm::ContentBlock as LlmContentBlock;

#[allow(clippy::too_many_arguments)]
pub async fn resume_paused_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<TurnOutcome, TurnError> {
    let session_id: Uuid = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(pool)
        .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = resume_locked(&mut conn, mqtt, provider, embedding_provider, registry, session_id, message_id, decision, remember).await;

    match result {
        Ok((reply, resumed_message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id: resumed_message_id })
        }
        Err(err) => {
            let _ = sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)")
                .bind(session_id)
                .bind(serde_json::json!({"error": err.to_string()}))
                .execute(&mut *conn)
                .await;
            let _ = mqtt.publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id: Uuid::nil(), error: err.to_string() }).await;
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn resume_locked(
    conn: &mut PoolConnection<Postgres>,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<(String, Option<Uuid>), TurnError> {
    let row: Option<(Uuid, String, serde_json::Value, Uuid)> = sqlx::query_as(
        "SELECT id, agent_type, state, sender_channel_identity_id FROM agent_sessions \
         WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true",
    )
    .bind(message_id.to_string())
    .fetch_optional(&mut **conn)
    .await?;

    let Some((agent_session_id, agent_type, state, sender_channel_identity_id)) = row else {
        return Err(TurnError::ApprovalNoLongerPending);
    };

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(sender_channel_identity_id)
        .fetch_one(&mut **conn)
        .await?;

    let agent = registry.find(&agent_type).ok_or(TurnError::ApprovalNoLongerPending)?;

    let tool_use_blocks: Vec<LlmContentBlock> =
        serde_json::from_value(state["tool_use_blocks"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let messages: Vec<LlmMessage> =
        serde_json::from_value(state["messages"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let pending_tool_use_id = state["pending_tool_use_id"].as_str().unwrap_or_default().to_string();

    if remember {
        if let Some(LlmContentBlock::ToolUse { name, input, .. }) =
            tool_use_blocks.iter().find(|b| matches!(b, LlmContentBlock::ToolUse { id, .. } if *id == pending_tool_use_id))
        {
            let path = input.get("path").and_then(|v| v.as_str());
            let rule_decision = if decision == "approve" { "allow" } else { "deny" };
            let _ = nomi_agent_core::permissions::remember_decision(conn, user_id, name, path, rule_decision).await;
        }
    }

    // Clear the paused-state keys before resolving — resolving may pause again on a different
    // block in the same batch, in which case it writes fresh paused keys right back.
    let _ = sqlx::query(
        "UPDATE agent_sessions SET state = state - 'paused_for_approval' - 'pending_approval_message_id' - 'pending_tool_use_id' - 'tool_use_blocks' - 'messages' WHERE id = $1",
    )
    .bind(agent_session_id)
    .execute(&mut **conn)
    .await?;

    let batch_outcome = nomi_agent_core::resolve_tool_batch(
        conn,
        Some((mqtt, Uuid::nil())),
        registry,
        agent,
        session_id,
        agent_session_id,
        user_id,
        &tool_use_blocks,
        &messages,
        Some((pending_tool_use_id.as_str(), decision == "approve")),
    )
    .await?;

    match batch_outcome {
        nomi_agent_core::ToolBatchOutcome::AwaitingApproval { .. } => Ok(("Waiting for another approval.".to_string(), None)),
        nomi_agent_core::ToolBatchOutcome::Completed { status, summary } => {
            finish_agent_turn(conn, session_id, agent_session_id, agent, nomi_agent_core::LoopOutcome::Completed { status, summary }).await
        }
        nomi_agent_core::ToolBatchOutcome::Resolved(tool_results) => {
            let mut full_messages = messages;
            full_messages.push(LlmMessage { role: LlmRole::User, content: tool_results });

            let outcome = nomi_agent_core::run_agent_turn(
                conn, Some((mqtt, Uuid::nil())), provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id,
                full_messages, SUBAGENT_MAX_TOKENS,
            )
            .await?;

            finish_agent_turn(conn, session_id, agent_session_id, agent, outcome).await
        }
    }
}
```

- [ ] **Step 8: Add the resume claim loop to `worker::run`**

```rust
// backend/crates/nomi-server/src/worker.rs
const APPROVAL_NOTIFY_CHANNEL: &str = "approval_resumes_channel";
```

Right after the existing `listener.listen(NOTIFY_CHANNEL).await` block (the one for `turn_jobs_channel`), add a second listener:

```rust
    let mut approval_listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "worker: failed to connect approval-resume LISTEN client; worker loop not started");
            return;
        }
    };
    if let Err(e) = approval_listener.listen(APPROVAL_NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "worker: failed to LISTEN on approval_resumes_channel; worker loop not started");
        return;
    }
```

Change the outer `loop`'s wake condition to wake on *either* channel (`tokio::select!` instead of a single timeout), and add a second drain loop claiming `approval_resumes` right after the existing `turn_jobs` drain loop:

```rust
    loop {
        tokio::select! {
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()) => {}
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, approval_listener.recv()) => {}
        }

        loop {
            // ... existing turn_jobs claim-and-process loop, unchanged ...
        }

        loop {
            let claimed = match nomi_turn::approval::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "worker: failed to claim next approval resume");
                    break;
                }
            };

            let session_id: Result<Uuid, sqlx::Error> = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
                .bind(claimed.message_id)
                .fetch_one(&pool)
                .await;
            let session_id = match session_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, resume_id = %claimed.id, "worker: failed to resolve session_id for approval resume");
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };
            let user_id: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
                "SELECT ci.user_id FROM agent_sessions ags JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
                 WHERE ags.state->>'pending_approval_message_id' = $1",
            )
            .bind(claimed.message_id.to_string())
            .fetch_one(&pool)
            .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, resume_id = %claimed.id, "worker: failed to resolve user_id for approval resume");
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };
            let _ = session_id; // resolved above only to fail fast with a clear error if the message/session no longer exists

            let provider = build_llm_provider_for_user(&pool, user_id, &settings_key, http_client.clone()).await;
            let embedding_provider = build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let result = nomi_turn::resume_paused_turn(
                &pool, &mqtt, provider.as_ref(), embedding_provider.as_ref(), &registry,
                claimed.message_id, &claimed.decision, claimed.remember,
            )
            .await;

            match result {
                Ok(outcome) => {
                    let _ = nomi_turn::approval::mark_completed(&pool, claimed.id).await;
                    let _ = mqtt
                        .publish(
                            outcome.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: Uuid::nil(), message_id: outcome.message_id.unwrap_or(Uuid::nil()) },
                        )
                        .await;
                    tracing::info!(resume_id = %claimed.id, "worker: approval resume completed");
                }
                Err(e) => {
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    tracing::warn!(resume_id = %claimed.id, error = %e, "worker: approval resume failed");
                }
            }
        }
    }
```

Also update the existing `turn_jobs` success arm (inside the first drain loop, unchanged in shape but now has a real id available) to use the real message id instead of `Uuid::nil()`:

```rust
                    let _ = mqtt
                        .publish(
                            claimed.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: outcome.message_id.unwrap_or(Uuid::nil()) },
                        )
                        .await;
```
(was `message_id: Uuid::nil()` unconditionally — `outcome` here is `process_turn`'s returned `TurnOutcome`, which now carries a real `message_id` per Step 6 above.)

- [ ] **Step 9: Add the HTTP endpoint**

```rust
// backend/crates/nomi-server/src/routes/sessions.rs
#[derive(Deserialize)]
pub struct ApprovalRequest {
    pub decision: String,
    #[serde(default)]
    pub remember: bool,
}

pub async fn resolve_approval(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ApprovalRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;

    if req.decision != "approve" && req.decision != "deny" {
        return Err((StatusCode::BAD_REQUEST, "decision must be 'approve' or 'deny'"));
    }

    let pending: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_sessions WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true)",
    )
    .bind(message_id.to_string())
    .fetch_one(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to check approval status"))?;

    if !pending {
        return Err((StatusCode::CONFLICT, "this action is no longer pending"));
    }

    let mut tx = state.pool.begin().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to start transaction"))?;
    nomi_turn::approval::enqueue(&mut tx, message_id, &req.decision, req.remember)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to queue approval decision"))?;
    tx.commit().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to queue approval decision"))?;

    Ok(StatusCode::ACCEPTED)
}
```

```rust
// backend/crates/nomi-server/src/app.rs — add alongside the existing feedback route
        .route(
            "/api/sessions/:id/messages/:message_id/approval",
            put(sessions_routes::resolve_approval),
        )
```

- [ ] **Step 10: Write an integration test for the endpoint's fencing check**

```rust
// backend/crates/nomi-server/tests/approval_routes.rs
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
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value) {
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
    json_request(router.clone(), "POST", "/api/auth/register", json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Acme"}}), None).await;
    let (_, login_body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn resolving_an_approval_that_is_not_pending_returns_409(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user@example.com").await;

    // Create a session and a plain message with no pending approval state on any agent_sessions row.
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "approve"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_decision_value_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user2@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "maybe"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 11: Run test to verify it passes, then the whole workspace**

Run: `cargo test -p nomi-server --test approval_routes`
Expected: PASS (2 tests)

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all tests pass. Fix any remaining call sites the compiler flags (e.g. anywhere else in the codebase that constructs a `TurnOutcome` or calls `run_locked_turn`/`run_subagent_turn` directly and expected the old `Result<String, TurnError>` shape).

- [ ] **Step 12: Commit**

```bash
git add backend/crates/nomi-agent-core/src/error.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-core/src/lib.rs backend/crates/nomi-turn/src/approval.rs \
        backend/crates/nomi-turn/src/lib.rs backend/crates/nomi-server/src/worker.rs \
        backend/crates/nomi-server/src/routes/sessions.rs backend/crates/nomi-server/src/app.rs \
        backend/crates/nomi-server/tests/approval_routes.rs
git commit -m "feat: resume a paused turn after an approval decision"
```

---

## Task 7: Realtime — `MessageCreated`/`MessageUpdated`, single-message GET endpoint

**Files:**
- Modify: `backend/crates/nomi-realtime/src/lib.rs` (`StreamEnvelope`)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (every publish call site)
- Modify: `backend/crates/nomi-turn/src/lib.rs` (`resume_locked` flips the approval message's status and notifies)
- Modify: `backend/crates/nomi-server/src/routes/sessions.rs` (`MessageItem.content_blocks`, `to_message_item`, `list_messages` SQL, new `get_message` handler)
- Modify: `backend/crates/nomi-server/src/app.rs` (route wiring)

**Interfaces:**
- Produces: `StreamEnvelope::{MessageCreated, MessageUpdated}` (`SessionActivity` removed), `GET /api/sessions/:id/messages/:message_id` — Task 12 (frontend `ChatThread.svelte`) consumes both.

- [ ] **Step 1: Replace `SessionActivity` with `MessageCreated`/`MessageUpdated`**

```rust
// backend/crates/nomi-realtime/src/lib.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
    AgentDelegationUpdated { delegation_id: Uuid },
    /// A brand-new message landed — the frontend fetches it and appends it to the conversation
    /// instead of refetching the whole thing.
    MessageCreated { message_id: Uuid },
    /// An existing message's content_blocks changed in place (a todo list step flipped, an
    /// approval was decided) — the frontend fetches it and replaces its existing entry.
    MessageUpdated { message_id: Uuid },
}
```

- [ ] **Step 2: Update every publish site in `engine.rs`**

`post_activity_message` now returns the inserted message's id and publishes `MessageCreated`:

```rust
async fn post_activity_message(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    content: &str,
    block: Option<&crate::content_block::ContentBlock>,
) -> Option<Uuid> {
    let content_blocks = block.map(|b| serde_json::json!([b]));
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(content)
    .bind(&content_blocks)
    .fetch_one(&mut **conn)
    .await
    .ok();

    if let (Some(id), Some(publisher)) = (message_id, mqtt) {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id: id }).await;
    }
    message_id
}
```
(was `-> ()`, publishing `SessionActivity { session_id }` unconditionally on success. Its three existing call sites — the 🧠 thinking post, the 💭 thought post, and the bottom-of-`resolve_tool_batch` post — are all bare statements with no `let` binding today; that's still valid now that the function returns `Option<Uuid>` instead of `()`, no call-site changes needed there.)

In `resolve_tool_batch`'s pre-scan (the approval-message insert), change the publish from `SessionActivity` to `MessageCreated`:

```rust
                if let Some((publisher, _)) = mqtt {
                    let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
                }
```
(was `&StreamEnvelope::SessionActivity { session_id }` — only the envelope construction changes)

In `upsert_todo_list`, publish `MessageUpdated` from the `Some(message_id)` (update) arm and `MessageCreated` from the `None` (insert) arm, replacing the single shared `SessionActivity` publish after the `match`:

```rust
    match existing_id {
        Some(message_id) => {
            sqlx::query("UPDATE messages SET content = $1, content_blocks = $2 WHERE id = $3")
                .bind(&display_text)
                .bind(&content_blocks)
                .bind(message_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(publisher) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::MessageUpdated { message_id }).await;
            }
        }
        None => {
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(&display_text)
            .bind(&content_blocks)
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query("UPDATE agent_sessions SET state = state || jsonb_build_object('todo_message_id', $1::text) WHERE id = $2")
                .bind(message_id.to_string())
                .bind(agent_session_id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(publisher) = mqtt {
                let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
            }
        }
    }

    Ok("todo list updated".to_string())
}
```
(replaces the whole `match existing_id { ... }` block plus the trailing shared publish that Task 4 originally wrote — the function's signature and everything above the `match` are unchanged.)

- [ ] **Step 3: Flip the approval message's status during resume, and notify**

In `resume_locked` (`backend/crates/nomi-turn/src/lib.rs`), right after the `remember` block and before clearing the paused-state keys, add:

```rust
    let new_status = if decision == "approve" { "approved" } else { "denied" };
    let _ = sqlx::query(
        "UPDATE messages SET content_blocks = jsonb_set(jsonb_set(content_blocks, '{0,status}', to_jsonb($1::text)), '{0,decided_at}', to_jsonb(now())) WHERE id = $2",
    )
    .bind(new_status)
    .bind(message_id)
    .execute(&mut **conn)
    .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::MessageUpdated { message_id }).await;
```

- [ ] **Step 4: Add `content_blocks` to `MessageItem` and every query that builds one**

```rust
// backend/crates/nomi-server/src/routes/sessions.rs
#[derive(Serialize)]
pub struct MessageItem {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub content_blocks: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub my_feedback: Option<String>,
}

fn to_message_item(
    (id, sender, content, created_at, content_blocks, my_feedback): (Uuid, Option<Uuid>, String, DateTime<Utc>, Option<serde_json::Value>, Option<String>),
) -> MessageItem {
    MessageItem {
        id,
        sender: if sender.is_some() { "user".to_string() } else { "assistant".to_string() },
        content,
        content_blocks,
        created_at,
        my_feedback,
    }
}
```

Update `list_messages`'s two query bodies (`Some(before_id)` and `None` branches) to select the new column, in this exact position (matching `to_message_item`'s tuple order above):

```sql
SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating
FROM messages m
LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $4
WHERE m.session_id = $1 AND m.created_at < (SELECT created_at FROM messages WHERE id = $2)
ORDER BY m.created_at DESC LIMIT $3
```
(the `Some(before_id)` branch — `$4` was already `claims.sub`, unchanged, only the SELECT list gained `m.content_blocks`) and:
```sql
SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating
FROM messages m
LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $3
WHERE m.session_id = $1
ORDER BY m.created_at DESC LIMIT $2
```
(the `None` branch). Update both `let rows: Vec<(Uuid, Option<Uuid>, String, DateTime<Utc>, Option<String>)>` type annotations to `Vec<(Uuid, Option<Uuid>, String, DateTime<Utc>, Option<serde_json::Value>, Option<String>)>`.

- [ ] **Step 5: Add the single-message GET endpoint**

```rust
// backend/crates/nomi-server/src/routes/sessions.rs
pub async fn get_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MessageItem>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let row: (Uuid, Option<Uuid>, String, DateTime<Utc>, Option<serde_json::Value>, Option<String>) = sqlx::query_as(
        "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating \
         FROM messages m \
         LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $3 \
         WHERE m.id = $1 AND m.session_id = $2",
    )
    .bind(message_id)
    .bind(session_id)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch message"))?
    .ok_or((StatusCode::NOT_FOUND, "message not found"))?;

    Ok(Json(to_message_item(row)))
}
```

```rust
// backend/crates/nomi-server/src/app.rs — add right after the existing
// "/api/sessions/:id/messages" route
        .route("/api/sessions/:id/messages/:message_id", get(sessions_routes::get_message))
```

- [ ] **Step 6: Build and test the whole workspace**

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all tests pass — fix any remaining `SessionActivity` reference the compiler flags, and any test asserting the old 5-tuple `MessageItem`/query shape.

- [ ] **Step 7: Commit**

```bash
git add backend/crates/nomi-realtime/src/lib.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-turn/src/lib.rs backend/crates/nomi-server/src/routes/sessions.rs \
        backend/crates/nomi-server/src/app.rs
git commit -m "feat: MessageCreated/MessageUpdated realtime events; single-message GET endpoint"
```

---

## Task 8: Frontend TS types + `@codemirror/merge` dependency

**Files:**
- Modify: `frontend/package.json` (new dependency, via `npm install`)
- Modify: `frontend/src/lib/types.ts`

**Interfaces:**
- Produces: `ContentBlock` TS discriminated union (mirrors Task 2's Rust enum exactly — same `kind` tag values, same field names), `MessageItem.content_blocks: ContentBlock[] | null`, `RenderedMessage` inherits it — every later frontend task imports these types.

- [ ] **Step 1: Install the diff-view dependency**

Run: `cd frontend && npm install @codemirror/merge`

Verify: `frontend/package.json`'s `dependencies` now includes `"@codemirror/merge": "^..."` and `frontend/package-lock.json` is updated (this repo's Docker/CI build uses `npm ci` against `package-lock.json` — always run `npm install` here, never `pnpm`, so the lockfile stays in sync).

- [ ] **Step 2: Add the `ContentBlock` union and wire it into `MessageItem`**

```typescript
// frontend/src/lib/types.ts — add near the top, before `MessageItem`

export interface TodoItem {
	id: string;
	text: string;
	status: 'pending' | 'in_progress' | 'done';
}

export interface TableColumn {
	key: string;
	label: string;
}

export type ContentBlock =
	| { kind: 'file_write'; project_id: string; path: string; content: string; previous_content: string | null }
	| { kind: 'file_delete'; project_id: string; path: string }
	| { kind: 'todo_list'; items: TodoItem[] }
	| { kind: 'table'; variant: 'data' | 'comparison'; columns: TableColumn[]; rows: Record<string, unknown>[] }
	| {
			kind: 'approval_request';
			id: string;
			tool_name: string;
			description: string;
			input: Record<string, unknown>;
			status: 'pending' | 'approved' | 'denied';
			decided_at: string | null;
	  };
```

```typescript
// frontend/src/lib/types.ts — MessageItem gains one field
export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	content_blocks: ContentBlock[] | null;
	created_at: string;
	my_feedback: 'up' | 'down' | null;
}
```
(was missing `content_blocks` — `RenderedMessage extends MessageItem` picks it up automatically, no change needed there.)

- [ ] **Step 3: Type-check**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors (the new fields are additive; nothing currently reads `MessageItem` exhaustively by key).

- [ ] **Step 4: Commit**

```bash
git add frontend/package.json frontend/package-lock.json frontend/src/lib/types.ts
git commit -m "feat: add ContentBlock frontend types and @codemirror/merge dependency"
```

---

## Task 9: `ContentBlockView` dispatcher + `FileWriteBlock`/`FileDeleteBlock`

**Files:**
- Create: `frontend/src/lib/components/blocks/ContentBlockView.svelte`
- Create: `frontend/src/lib/components/blocks/FileWriteBlock.svelte`
- Create: `frontend/src/lib/components/blocks/FileDeleteBlock.svelte`

**Interfaces:**
- Consumes: `ContentBlock` from Task 8.
- Produces: `<ContentBlockView block={block} />` — Task 11 (`MessageBubble.svelte`) renders one of these per entry in `message.content_blocks`.

- [ ] **Step 1: Write `FileDeleteBlock.svelte`** (the simplest block — do this first to establish the pattern)

```svelte
<!-- frontend/src/lib/components/blocks/FileDeleteBlock.svelte -->
<script lang="ts">
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'file_delete' }> } = $props();
</script>

<div class="m3-block-card m3-block-card--delete">
	<span class="m3-block-card__icon" aria-hidden="true">🗑️</span>
	<span class="md-body-medium">Deleted <code>{block.path}</code></span>
</div>

<style>
	.m3-block-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-block-card--delete {
		background: color-mix(in srgb, var(--md-sys-color-error) 6%, var(--md-sys-color-surface-container-low));
	}
	.m3-block-card__icon {
		font-size: 1.1em;
	}
</style>
```

- [ ] **Step 2: Write `FileWriteBlock.svelte`** (diff view via `@codemirror/merge` when `previous_content` is present)

```svelte
<!-- frontend/src/lib/components/blocks/FileWriteBlock.svelte -->
<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { EditorView, basicSetup } from 'codemirror';
	import { MergeView } from '@codemirror/merge';
	import { css } from '@codemirror/lang-css';
	import { html } from '@codemirror/lang-html';
	import { javascript } from '@codemirror/lang-javascript';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'file_write' }> } = $props();

	let container: HTMLDivElement | undefined = $state();

	function languageExtension(path: string) {
		const ext = path.split('.').pop()?.toLowerCase();
		if (ext === 'html' || ext === 'htm') return html();
		if (ext === 'css') return css();
		if (ext === 'js' || ext === 'mjs' || ext === 'jsx' || ext === 'ts' || ext === 'tsx') return javascript();
		return null;
	}

	$effect(() => {
		if (!container) return;
		const lang = languageExtension(block.path);
		const readOnlyExtensions = [basicSetup, EditorView.editable.of(false), ...(lang ? [lang] : [])];

		let view: MergeView | EditorView;
		if (block.previous_content !== null) {
			view = new MergeView({
				a: { doc: block.previous_content, extensions: readOnlyExtensions },
				b: { doc: block.content, extensions: readOnlyExtensions },
				parent: container,
			});
		} else {
			view = new EditorView({ state: EditorState.create({ doc: block.content, extensions: readOnlyExtensions }), parent: container });
		}

		return () => view.destroy();
	});
</script>

<div class="m3-block-card m3-block-card--write">
	<div class="m3-block-card__header">
		<span aria-hidden="true">{block.previous_content !== null ? '📝' : '✨'}</span>
		<span class="md-body-medium">{block.previous_content !== null ? 'Updated' : 'Created'} <code>{block.path}</code></span>
	</div>
	<div class="m3-block-card__editor" bind:this={container}></div>
</div>

<style>
	.m3-block-card {
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		overflow: hidden;
	}
	.m3-block-card--write {
		background: var(--md-sys-color-surface-container-low);
	}
	.m3-block-card__header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-block-card__editor {
		max-height: 320px;
		overflow: auto;
		font-size: 0.85em;
	}
</style>
```

- [ ] **Step 3: Write the dispatcher (stub `TodoListBlock`/`TableBlock`/`ApprovalCard` for now — Tasks 10 fills them in)**

```svelte
<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import FileWriteBlock from './FileWriteBlock.svelte';
	import FileDeleteBlock from './FileDeleteBlock.svelte';
	import TodoListBlock from './TodoListBlock.svelte';
	import TableBlock from './TableBlock.svelte';
	import ApprovalCard from './ApprovalCard.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, messageId }: { block: ContentBlock; messageId: string } = $props();
</script>

{#if block.kind === 'file_write'}
	<FileWriteBlock {block} />
{:else if block.kind === 'file_delete'}
	<FileDeleteBlock {block} />
{:else if block.kind === 'todo_list'}
	<TodoListBlock {block} />
{:else if block.kind === 'table'}
	<TableBlock {block} />
{:else if block.kind === 'approval_request'}
	<ApprovalCard {block} {messageId} />
{/if}
```

(`TodoListBlock.svelte`, `TableBlock.svelte`, `ApprovalCard.svelte` don't exist yet — this file won't compile until Task 10 creates them. That's expected; Task 10 is the very next task and this dispatcher is the shared contract both tasks build against.)

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/blocks/ContentBlockView.svelte \
        frontend/src/lib/components/blocks/FileWriteBlock.svelte \
        frontend/src/lib/components/blocks/FileDeleteBlock.svelte
git commit -m "feat: add ContentBlockView dispatcher, FileWriteBlock diff view, FileDeleteBlock"
```

---

## Task 10: `TodoListBlock`, `TableBlock`, `ApprovalCard` + the `resolveApproval` action

**Files:**
- Create: `frontend/src/lib/components/blocks/TodoListBlock.svelte`
- Create: `frontend/src/lib/components/blocks/TableBlock.svelte`
- Create: `frontend/src/lib/components/blocks/ApprovalCard.svelte`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` (new `resolveApproval` action)
- Modify: `frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts` (same action — `ChatThread`/`MessageBubble` are shared between both pages, so both need it, matching how `sendMessage`/`feedback` are already duplicated across both files)

**Interfaces:**
- Consumes: `ContentBlockView` from Task 9 (now compiles, since these three components fill in the last of its `{#if}` branches); `DataTable`/`Card`/`Button`/`Checkbox` from `$lib/components/m3/*` (all pre-existing).
- Produces: `?/resolveApproval` form action, called by `ApprovalCard`.

- [ ] **Step 1: Write `TodoListBlock.svelte`**

```svelte
<!-- frontend/src/lib/components/blocks/TodoListBlock.svelte -->
<script lang="ts">
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'todo_list' }> } = $props();
</script>

<div class="m3-block-card m3-block-card--todo">
	<p class="md-label-large" style="color: var(--md-sys-color-on-surface-variant)">To-do list</p>
	<ul>
		{#each block.items as item (item.id)}
			<li class="m3-todo-item m3-todo-item--{item.status}">
				<span class="m3-todo-item__mark" aria-hidden="true">
					{#if item.status === 'done'}✓{:else if item.status === 'in_progress'}◐{:else}○{/if}
				</span>
				<span>{item.text}</span>
			</li>
		{/each}
	</ul>
</div>

<style>
	.m3-block-card--todo {
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
	}
	.m3-block-card--todo ul {
		list-style: none;
		margin: 8px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.m3-todo-item {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		color: var(--md-sys-color-on-surface);
	}
	.m3-todo-item--done {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.m3-todo-item__mark {
		width: 18px;
		text-align: center;
		color: var(--md-sys-color-primary);
	}
</style>
```

- [ ] **Step 2: Write `TableBlock.svelte`** (`data` variant reuses the existing `DataTable` shell; `comparison` renders side-by-side cards instead, since a comparison reads better as cards than as a wide row-oriented table)

```svelte
<!-- frontend/src/lib/components/blocks/TableBlock.svelte -->
<script lang="ts">
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'table' }> } = $props();
</script>

{#if block.variant === 'data'}
	<DataTable columns={block.columns}>
		{#each block.rows as row, i (i)}
			<tr>
				{#each block.columns as column (column.key)}
					<td>{row[column.key] ?? ''}</td>
				{/each}
			</tr>
		{/each}
	</DataTable>
{:else}
	<div class="m3-comparison-grid">
		{#each block.rows as row, i (i)}
			<Card variant="outlined" class="p-3">
				{#each block.columns as column (column.key)}
					<div class="m3-comparison-field">
						<span class="md-label-small" style="color: var(--md-sys-color-on-surface-variant)">{column.label}</span>
						<span class="md-body-medium">{row[column.key] ?? ''}</span>
					</div>
				{/each}
			</Card>
		{/each}
	</div>
{/if}

<style>
	.m3-comparison-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 8px;
	}
	.m3-comparison-field {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 4px 0;
	}
</style>
```

- [ ] **Step 3: Write `ApprovalCard.svelte`**

```svelte
<!-- frontend/src/lib/components/blocks/ApprovalCard.svelte -->
<script lang="ts">
	import { deserialize } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		messageId,
	}: { block: Extract<ContentBlock, { kind: 'approval_request' }>; messageId: string } = $props();

	let remember = $state(false);
	let submitting = $state(false);
	let error = $state<string | null>(null);

	async function resolve(decision: 'approve' | 'deny') {
		submitting = true;
		error = null;
		const body = new FormData();
		body.set('messageId', messageId);
		body.set('decision', decision);
		body.set('remember', remember ? 'true' : 'false');

		const response = await fetch('?/resolveApproval', { method: 'POST', body });
		const result = deserialize(await response.text());
		submitting = false;
		if (result.type === 'failure') {
			error = (result.data?.error as string) ?? 'Failed to record your decision.';
		}
		// On success the card's own status flips via the MessageUpdated WS event landing shortly
		// after (Task 12) — no local optimistic update needed here.
	}
</script>

<div class="m3-block-card m3-block-card--approval">
	<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{block.description}</p>
	<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">Tool: <code>{block.tool_name}</code></p>

	{#if block.status === 'pending'}
		{#if error}
			<p class="md-body-small" style="color: var(--md-sys-color-error)">{error}</p>
		{/if}
		<label class="m3-approval-remember">
			<Checkbox bind:checked={remember} />
			<span class="md-body-small">Remember this decision for next time</span>
		</label>
		<div class="m3-block-card__actions">
			<Button type="button" variant="filled" disabled={submitting} onclick={() => resolve('approve')}>Approve</Button>
			<Button type="button" variant="outlined" disabled={submitting} onclick={() => resolve('deny')}>Deny</Button>
		</div>
	{:else}
		<p
			class="md-label-large"
			style="color: {block.status === 'approved' ? 'var(--md-sys-color-primary)' : 'var(--md-sys-color-error)'}"
		>
			{block.status === 'approved' ? '✓ Approved' : '✗ Denied'}
		</p>
	{/if}
</div>

<style>
	.m3-block-card--approval {
		padding: 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.m3-approval-remember {
		display: flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
	}
	.m3-block-card__actions {
		display: flex;
		gap: 8px;
	}
</style>
```

- [ ] **Step 4: Add the `resolveApproval` action to both pages that render chat messages**

```typescript
// frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts — add inside the `actions` object,
// alongside the existing `feedback` action
	resolveApproval: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const decision = data.get('decision');
		const remember = data.get('remember') === 'true';

		if (typeof messageId !== 'string' || (decision !== 'approve' && decision !== 'deny')) {
			return fail(400, { error: 'Invalid approval decision.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages/${messageId}/approval`, {
			method: 'PUT',
			body: JSON.stringify({ decision, remember }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to record your decision.' });
		}

		return { success: true };
	},
```

Add the exact same action (unchanged) to `frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts`'s `actions` object.

- [ ] **Step 5: Type-check**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors — `ContentBlockView.svelte` (Task 9) now resolves all five of its imports.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/lib/components/blocks/TodoListBlock.svelte \
        frontend/src/lib/components/blocks/TableBlock.svelte \
        frontend/src/lib/components/blocks/ApprovalCard.svelte \
        "frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts" \
        "frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts"
git commit -m "feat: add TodoListBlock, TableBlock, ApprovalCard, and the resolveApproval action"
```

---

## Task 11: `MessageBubble.svelte` renders blocks; citation-link styling

**Files:**
- Modify: `frontend/src/lib/components/MessageBubble.svelte`

**Interfaces:**
- Consumes: `ContentBlockView` from Tasks 9–10, `message.content_blocks` from Task 8's types.

- [ ] **Step 1: Render blocks when present, fall back to `{@html}` otherwise**

```svelte
<!-- frontend/src/lib/components/MessageBubble.svelte -->
<script lang="ts">
	import { deserialize } from '$app/forms';
	import ContentBlockView from './blocks/ContentBlockView.svelte';
	import IconCheck from './icons/IconCheck.svelte';
	// ... (keep every other existing import unchanged)
```
(only the new `ContentBlockView` import is added, right after the existing `deserialize` import)

Replace:
```svelte
	<div
		bind:this={bubbleEl}
		class="message-bubble md-body-large max-w-md px-4 py-2"
		style={...}
	>
		{@html message.content_html}
	</div>
```
with:
```svelte
	<div
		bind:this={bubbleEl}
		class="message-bubble md-body-large max-w-md px-4 py-2"
		style={...}
	>
		{#if message.content_blocks && message.content_blocks.length > 0}
			<div class="flex flex-col gap-2">
				{#each message.content_blocks as block, i (i)}
					<ContentBlockView {block} messageId={message.id} />
				{/each}
			</div>
		{:else}
			{@html message.content_html}
		{/if}
	</div>
```
(the `style={...}` attribute — the existing `message.sender === 'user' ? ... : ...` ternary building the bubble's background/border-radius — is unchanged, only the element's inner content changes)

- [ ] **Step 2: Add citation-link styling alongside the existing code-block enhancement**

```javascript
	// A reference-style citation ("[1] Attention is all you need — [1]: https://arxiv.org/...")
	// renders through `marked` as a plain `<a href="...">1</a>` — no different from any other
	// link. The heuristic: a citation's visible text is exactly its numeric label; a normal
	// inline link's text is a real phrase. Not airtight (a link whose text happens to be a bare
	// number also matches), but cheap and correct for the actual citation format this is for.
	function enhanceCitations(root: HTMLElement) {
		const links = root.querySelectorAll<HTMLAnchorElement>('a[href]');
		links.forEach((link) => {
			const text = link.textContent?.trim() ?? '';
			if (/^\d+$/.test(text)) {
				link.classList.add('citation-chip');
			}
		});
	}
```
(add this function right after `enhanceCodeBlocks`)

Update the existing `$effect` to also call it:
```javascript
	$effect(() => {
		void message.content_html;
		if (bubbleEl) {
			enhanceCodeBlocks(bubbleEl);
			enhanceCitations(bubbleEl);
		}
	});
```
(was `if (bubbleEl) enhanceCodeBlocks(bubbleEl);` — now calls both)

Add the chip styling to the `<style>` block, alongside the other `.message-bubble :global(...)` rules:
```css
	.message-bubble :global(a.citation-chip) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.2em;
		height: 1.2em;
		padding: 0 0.3em;
		margin: 0 0.1em;
		border-radius: var(--md-sys-shape-corner-full);
		background: color-mix(in srgb, var(--md-sys-color-primary) 15%, transparent);
		color: var(--md-sys-color-primary);
		font-size: 0.7em;
		font-weight: 600;
		text-decoration: none;
		vertical-align: super;
	}
```

- [ ] **Step 3: Type-check and live-verify**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors.

Live-verify (dev server running): open a chat that has produced a `file_write`/`file_delete`/`todo_list`/`table` message (or seed one directly via `docker exec backend-postgres-1 psql ... -c "UPDATE messages SET content_blocks = '[{\"kind\":\"file_delete\",\"project_id\":\"<uuid>\",\"path\":\"a.txt\"}]' WHERE id = '<id>'"` against a throwaway test message) and confirm the right widget renders instead of plain text; confirm a plain-text message with a `[1](url)`-style reference link still renders with the citation chip styling; confirm an ordinary message with no blocks still renders exactly as before.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/MessageBubble.svelte
git commit -m "feat: render content blocks in MessageBubble; style citation links"
```

---

## Task 12: `ChatThread.svelte` — granular `MessageCreated`/`MessageUpdated` patching

**Files:**
- Create: `frontend/src/lib/server/fetchRenderedMessage.ts`
- Create: `frontend/src/routes/(app)/chat/[sessionId]/message/[messageId]/+server.ts`
- Create: `frontend/src/routes/(app)/projects/session/[sessionId]/message/[messageId]/+server.ts`
- Modify: `frontend/src/lib/components/ChatThread.svelte`

**Interfaces:**
- Consumes: `StreamEnvelope::{MessageCreated, MessageUpdated}` (Task 7's backend, now flowing over the existing WS relay unchanged — the relay is a dumb byte pass-through, no relay-side changes needed), `GET /api/sessions/:id/messages/:message_id` (Task 7).

- [ ] **Step 1: Write the shared server-side render-one-message helper**

```typescript
// frontend/src/lib/server/fetchRenderedMessage.ts
import { error } from '@sveltejs/kit';
import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { MessageItem, RenderedMessage } from '$lib/types';

/** Fetches one message from the backend and server-renders its markdown (skipped when it
 * carries content_blocks — those render client-side via ContentBlockView, not {@html}). Used by
 * the two `message/[messageId]/+server.ts` proxy routes (chat and project-workspace pages) so
 * ChatThread.svelte can patch a single message instead of refetching the whole conversation. */
export async function fetchRenderedMessage(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	messageId: string,
): Promise<RenderedMessage> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/messages/${messageId}`);
	if (!response.ok) {
		throw error(response.status, 'Could not load this message.');
	}
	const message = (await response.json()) as MessageItem;
	const content_html =
		message.content_blocks && message.content_blocks.length > 0 ? '' : await renderMarkdown(message.content);
	return { ...message, content_html };
}
```

- [ ] **Step 2: Add the two proxy routes**

```typescript
// frontend/src/routes/(app)/chat/[sessionId]/message/[messageId]/+server.ts
import { json } from '@sveltejs/kit';
import { fetchRenderedMessage } from '$lib/server/fetchRenderedMessage';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const message = await fetchRenderedMessage(fetch, cookies, params.sessionId, params.messageId);
	return json(message);
};
```

```typescript
// frontend/src/routes/(app)/projects/session/[sessionId]/message/[messageId]/+server.ts
import { json } from '@sveltejs/kit';
import { fetchRenderedMessage } from '$lib/server/fetchRenderedMessage';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const message = await fetchRenderedMessage(fetch, cookies, params.sessionId, params.messageId);
	return json(message);
};
```

- [ ] **Step 3: Give `ChatThread.svelte` local, patchable message state**

Replace the top-level `messages` usage with a local copy, and add the fetch-and-upsert helper:

```javascript
	let localMessages = $state(messages);

	// Resync whenever the page's own `messages` prop changes — navigating to a different
	// session, or a full invalidateAll() (still used for AgentDelegationUpdated and on
	// WS-reconnect-after-drop, see below).
	$effect(() => {
		localMessages = messages;
	});

	const NIL_UUID = '00000000-0000-0000-0000-000000000000';

	async function fetchAndUpsertMessage(id: string) {
		try {
			const response = await fetch(`message/${id}`);
			if (!response.ok) return;
			const message: RenderedMessage = await response.json();
			const index = localMessages.findIndex((m) => m.id === message.id);
			if (index === -1) {
				localMessages = [...localMessages, message];
			} else {
				localMessages = [...localMessages.slice(0, index), message, ...localMessages.slice(index + 1)];
			}
		} catch {
			// Best-effort — a dropped connection triggers a full invalidateAll() on reconnect
			// (see the socket 'open' handler below), which reconciles anything missed here.
		}
	}
```
(place `localMessages`/the `$effect`/`NIL_UUID`/`fetchAndUpsertMessage` right after the existing `let pendingReply = $state(false);` block of state declarations)

- [ ] **Step 4: Update the WS message handler**

```javascript
			socket.addEventListener('message', (event) => {
				let envelope: { kind: string; message_id?: string };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				if (envelope.kind === 'Delta') {
					pendingReply = true;
				} else if (envelope.kind === 'TurnCompleted') {
					pendingReply = false;
					turnError = false;
					if (envelope.message_id && envelope.message_id !== NIL_UUID) {
						fetchAndUpsertMessage(envelope.message_id);
					}
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				} else if (envelope.kind === 'AgentDelegationUpdated') {
					invalidateAll();
				} else if (envelope.kind === 'MessageCreated' || envelope.kind === 'MessageUpdated') {
					if (envelope.message_id) fetchAndUpsertMessage(envelope.message_id);
				}
			});
```
(was: `TurnCompleted` called `invalidateAll()`; `SessionActivity` called `invalidateAll()` — both replaced as shown. `Delta`, `TurnFailed`, `AgentDelegationUpdated`, and everything in the `open`/`close` handlers are unchanged.)

- [ ] **Step 5: Point the template at `localMessages`**

```svelte
			{#each localMessages as message, i (message.id)}
				<MessageBubble {message} chained={isChained(localMessages, i)} first={i === 0} />
			{/each}
```
(was `{#each messages as message, i (message.id)}` / `isChained(messages, i)` — the two other references to the `messages` prop, in the component's own `messages: RenderedMessage[]` prop type declaration, stay as-is; only these render-time usages switch to `localMessages`.)

- [ ] **Step 6: Type-check**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors.

- [ ] **Step 7: Live-verify the full flow end to end**

With the dev servers running: open a project chat, ask the coding agent to write a file, and confirm (a) the `FileWrite` block/diff appears live without a full-page flicker/scroll-jump, (b) asking it to delete a file (with no permission rule set) produces a pending `ApprovalCard`, (c) clicking Approve flips the card to "✓ Approved" in place and a follow-up `FileDelete` block appears, (d) clicking Deny on a fresh gated call flips it to "✗ Denied" and the agent's next reply acknowledges it didn't happen. Clean up any test project/session data created during this pass per the project's usual practice.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/lib/server/fetchRenderedMessage.ts \
        "frontend/src/routes/(app)/chat/[sessionId]/message/[messageId]/+server.ts" \
        "frontend/src/routes/(app)/projects/session/[sessionId]/message/[messageId]/+server.ts" \
        frontend/src/lib/components/ChatThread.svelte
git commit -m "feat: patch individual messages in place via MessageCreated/MessageUpdated"
```

---

## Self-Review

**Spec coverage:**
- File diff / "writing file" → Task 3 (`FileWrite` block, diff capture), Task 9 (`FileWriteBlock` diff view). ✓
- "Deleting file" → Task 3 (`FileDelete` block), Task 9 (`FileDeleteBlock`). ✓
- Code block → untouched, confirmed unaffected by this plan (no task modifies `enhanceCodeBlocks` itself, only adds `enhanceCitations` alongside it). ✓
- Inline citations → Task 11 (pure client-side styling, no backend change, per spec's explicit resolution). ✓
- Todo list → Task 4 (`update_todos` engine tool, upsert), Task 10 (`TodoListBlock`). ✓
- Datatable / comparison table → Task 4 (`show_table` engine tool), Task 10 (`TableBlock`, both variants). ✓
- Approval card → Task 5 (permission pre-scan, pause), Task 6 (resume subsystem), Task 10 (`ApprovalCard` UI). ✓
- `tool_permission_rules` + "remember this decision" → Task 1 (schema), Task 5 (`check_tool_permission`), Task 6 (`remember_decision` called from resume), Task 10 (checkbox on the card). ✓
- `MessageCreated`/`MessageUpdated` replacing `SessionActivity` → Task 7 (backend), Task 12 (frontend). ✓
- `messages.content` stays populated as a fallback → every block-producing code path (Tasks 3–5) always sets both `content` and `content_blocks` together, never one without the other. ✓

**Placeholder scan:** no TBD/TODO markers; every code block above is complete, runnable code, not a description of what to write.

**Type consistency check:** `ContentBlock`'s Rust `kind` tag values (`file_write`, `file_delete`, `todo_list`, `table`, `approval_request` — from `#[serde(tag = "kind", rename_all = "snake_case")]` on `FileWrite`/`FileDelete`/`TodoList`/`Table`/`ApprovalRequest`) match the TS union's `kind` literals exactly (Task 8). `ToolOutcome`, `LoopOutcome::AwaitingApproval`, `ToolBatchOutcome`, `TurnOutcome.message_id` are each defined once (Tasks 2, 5, 6, 6) and referenced with the same field names everywhere they're later consumed (Tasks 6, 7, 12). `SHOW_TABLE_TOOL_NAME`/`UPDATE_TODOS_TOOL_NAME` (Task 4) are the same constants `is_gateable` (Task 5) and `resolve_tool_batch` (Task 6) check against.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-09-07-rich-content-blocks-implementation.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
