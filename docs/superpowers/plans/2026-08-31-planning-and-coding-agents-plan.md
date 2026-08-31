# Planning & Coding Agents Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `planning` agent and a `coding` agent to nomi's existing multi-agent system, backed by a new `projects`/`project_files` data model (file content in S3, metadata in Postgres), so a user can ask nomi to build something, see the plan, watch real files get written, and browse/edit/preview them in nomi's own web app.

**Architecture:** Two new `SubAgent` crates (`nomi-agent-planning`, `nomi-agent-coding`) plug into the existing delegation mechanism with zero changes to it. File content lives in a new `nomi-storage` crate (extracted from `nomi-server` to avoid a dependency cycle, since agent crates sit below `nomi-server`) wrapping direct S3 `put_object`/`get_object`/`delete_object`. The frontend gets a new `projects` route area with a CodeMirror-based editor and a sandboxed-iframe static preview.

**Tech Stack:** Rust/Axum/sqlx/Postgres backend, aws-sdk-s3, SvelteKit 2/Svelte 5 frontend, CodeMirror 6.

**Spec:** `docs/superpowers/specs/2026-08-31-planning-and-coding-agents-design.md`

## Global Constraints

- File content is never stored in Postgres — only in S3, at key `projects/{project_id}/{path}`. Postgres holds metadata only (path, content_type, size_bytes).
- Every project/file DB query is scoped by `user_id` ownership — no new permission model, matches how sessions/messages are already scoped.
- S3 writes always happen before the corresponding Postgres metadata write (see spec §2) — never the reverse.
- No changes to the generic delegation mechanism (`create_delegation`, `agent_delegations` table, the `delegate_to_agent` tool) — the two new agents are ordinary registry entries.
- `agent_delegations.task` stays plain text; the project ID is embedded in that text as `"Project <uuid>: ..."` — no new column on that table.

---

### Task 1: Extract `nomi-storage` crate from `nomi-server`'s `s3.rs`

**Files:**
- Create: `backend/crates/nomi-storage/Cargo.toml`
- Create: `backend/crates/nomi-storage/src/lib.rs`
- Delete: `backend/crates/nomi-server/src/s3.rs`
- Modify: `backend/crates/nomi-server/src/lib.rs:1-7` (remove `pub mod s3;`)
- Modify: `backend/crates/nomi-server/src/app.rs:14` (`use crate::s3::S3Config;` → `use nomi_storage::S3Config;`)
- Modify: `backend/crates/nomi-server/src/main.rs:34` (`nomi_server::s3::build_from_env()` → `nomi_storage::build_from_env()`)
- Modify: `backend/crates/nomi-server/Cargo.toml` (remove `aws-config`/`aws-sdk-s3`, add `nomi-storage = { path = "../nomi-storage" }`)

**Interfaces:**
- Produces: `nomi_storage::S3Config` (Clone), `nomi_storage::build_from_env() -> Option<S3Config>`, `nomi_storage::PresignError`, `nomi_storage::S3Error`, `nomi_storage::PresignedUpload { upload_url, public_url }`, and `S3Config` methods `presign_put(&self, key: &str, content_type: &str) -> Result<PresignedUpload, PresignError>`, `put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), S3Error>`, `get_object(&self, key: &str) -> Result<Option<String>, S3Error>`, `delete_object(&self, key: &str) -> Result<(), S3Error>`.

- [ ] **Step 1: Create the crate**

`backend/crates/nomi-storage/Cargo.toml`:

```toml
[package]
name = "nomi-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
aws-config = { version = "1", features = ["behavior-version-latest"] }
aws-sdk-s3 = "1"
thiserror = "1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

`backend/crates/nomi-storage/src/lib.rs` — this is `nomi-server/src/s3.rs`'s current content, unchanged, plus three new methods and a new `S3Error` type:

```rust
use std::time::Duration;

use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

const PRESIGNED_UPLOAD_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct S3Config {
    client: Client,
    bucket: String,
    public_url_base: String,
}

/// Optional infrastructure — a fresh install with no S3 credentials configured should boot and
/// run everything else normally, not panic at startup. Returns `None` (not an error) whenever
/// `S3_BUCKET` is unset; every other required var missing while `S3_BUCKET` IS set is treated as
/// a real misconfiguration and does panic, since that means someone intended to enable this and
/// got it wrong.
pub async fn build_from_env() -> Option<S3Config> {
    let bucket = std::env::var("S3_BUCKET").ok()?;
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let access_key = std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set when S3_BUCKET is set");
    let secret_key =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set when S3_BUCKET is set");
    let endpoint_url = std::env::var("S3_ENDPOINT_URL").ok();
    let public_url_base = std::env::var("S3_PUBLIC_URL_BASE")
        .unwrap_or_else(|_| format!("https://{bucket}.s3.{region}.amazonaws.com"));

    let credentials = Credentials::new(access_key, secret_key, None, None, "env");
    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region))
        .credentials_provider(credentials)
        .load()
        .await;

    let mut s3_builder = S3ConfigBuilder::from(&sdk_config);
    if let Some(endpoint) = endpoint_url {
        // S3-compatible services (Shipyard, R2, MinIO, ...) are usually addressed with
        // path-style URLs (host/bucket/key) rather than AWS's virtual-hosted-style
        // (bucket.host/key) — force it whenever a custom endpoint is in play.
        s3_builder = s3_builder.endpoint_url(endpoint).force_path_style(true);
    }

    Some(S3Config { client: Client::from_conf(s3_builder.build()), bucket, public_url_base })
}

#[derive(Debug, thiserror::Error)]
pub enum PresignError {
    #[error("failed to presign upload url: {0}")]
    Presign(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error(transparent)]
    Config(#[from] aws_sdk_s3::presigning::PresigningConfigError),
}

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("failed to put object: {0}")]
    Put(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to get object: {0}")]
    Get(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to delete object: {0}")]
    Delete(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::delete_object::DeleteObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to read object body: {0}")]
    Body(#[from] aws_sdk_s3::primitives::ByteStreamError),
    #[error("object content was not valid utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub struct PresignedUpload {
    pub upload_url: String,
    pub public_url: String,
}

impl S3Config {
    /// `key` is the full object key (path within the bucket) to presign a PUT for — callers
    /// build it (e.g. `avatars/{user_id}/{uuid}.{ext}`), this just talks to S3.
    pub async fn presign_put(&self, key: &str, content_type: &str) -> Result<PresignedUpload, PresignError> {
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(PresigningConfig::expires_in(PRESIGNED_UPLOAD_TTL)?)
            .await?;

        Ok(PresignedUpload {
            upload_url: presigned.uri().to_string(),
            public_url: format!("{}/{key}", self.public_url_base.trim_end_matches('/')),
        })
    }

    /// Writes `content` directly from the server — for writers that already hold the bytes
    /// in-process (an agent tool call, a browser-submitted body proxied through our own API),
    /// as opposed to `presign_put`'s browser-uploads-directly-to-S3 shape.
    pub async fn put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), S3Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(content.as_bytes().to_vec()))
            .send()
            .await?;
        Ok(())
    }

    /// `Ok(None)` when the object doesn't exist (S3's `NoSuchKey`) — callers that want "not
    /// found" to be a normal, expected outcome rather than an error path get that for free.
    pub async fn get_object(&self, key: &str) -> Result<Option<String>, S3Error> {
        let result = self.client.get_object().bucket(&self.bucket).key(key).send().await;
        let output = match result {
            Ok(output) => output,
            Err(err) => {
                if let aws_sdk_s3::error::SdkError::ServiceError(service_err) = &err {
                    if service_err.err().is_no_such_key() {
                        return Ok(None);
                    }
                }
                return Err(S3Error::Get(err));
            }
        };
        let bytes = output.body.collect().await?.into_bytes();
        Ok(Some(String::from_utf8(bytes.to_vec())?))
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), S3Error> {
        self.client.delete_object().bucket(&self.bucket).key(key).send().await?;
        Ok(())
    }
}
```

- [ ] **Step 2: Delete the old module and update `nomi-server`**

```bash
rm backend/crates/nomi-server/src/s3.rs
```

`backend/crates/nomi-server/src/lib.rs` — remove the `pub mod s3;` line (the file listed the modules alphabetically: `app`, `bootstrap`, `delegation_worker`, `routes`, `s3`, `web_identity`, `worker` — delete the `s3` one, leave the rest and the `build_agent_registry` function untouched for now, Task 5 changes it).

`backend/crates/nomi-server/src/app.rs` — change:
```rust
use crate::s3::S3Config;
```
to:
```rust
use nomi_storage::S3Config;
```

`backend/crates/nomi-server/src/main.rs` — change line 34:
```rust
let s3 = nomi_server::s3::build_from_env().await;
```
to:
```rust
let s3 = nomi_storage::build_from_env().await;
```

`backend/crates/nomi-server/Cargo.toml` — remove these two lines:
```toml
aws-config = { version = "1", features = ["behavior-version-latest"] }
aws-sdk-s3 = "1"
```
add, alongside the other `nomi-*` path deps:
```toml
nomi-storage = { path = "../nomi-storage" }
```

- [ ] **Step 3: Verify it builds and existing tests still pass**

Run: `cd backend && cargo build --workspace`
Expected: clean build, no errors. (`profile.rs` never named `S3Config`/`PresignError` explicitly — it only calls methods on `state.s3` — so it needs no changes.)

Run: `cd backend && cargo test -p nomi-server --test profile_routes`
Expected: all existing tests pass unchanged (they set `s3: None` as a struct field, which doesn't care about the type's import path).

- [ ] **Step 4: Commit**

```bash
git add backend/crates/nomi-storage backend/crates/nomi-server/Cargo.toml backend/crates/nomi-server/src/lib.rs backend/crates/nomi-server/src/app.rs backend/crates/nomi-server/src/main.rs
git rm backend/crates/nomi-server/src/s3.rs
git commit -m "refactor: extract S3Config into a new nomi-storage crate

Agent crates sit below nomi-server in the dependency graph; the coding
agent needs direct S3 access, and S3Config living in nomi-server would
make that a cycle. Also adds direct put/get/delete methods alongside the
existing presign_put, for writers that already hold content in-process."
```

---

### Task 2: `projects` and `project_files` migration

**Files:**
- Create: `backend/migrations/0020_projects_and_files.sql`

**Interfaces:**
- Produces: `projects` table (`id, user_id, session_id, name, description, plan, status, created_at, updated_at`), `project_files` table (`id, project_id, path, content_type, size_bytes, created_at, updated_at`, unique on `(project_id, path)`).

- [ ] **Step 1: Write the migration**

```sql
CREATE TABLE projects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT,
    plan        TEXT,
    status      TEXT NOT NULL DEFAULT 'planning'
                CHECK (status IN ('planning', 'building', 'ready')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE project_files (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path         TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text/plain',
    size_bytes   INTEGER NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, path)
);

CREATE INDEX project_files_project_id_idx ON project_files (project_id);
CREATE INDEX projects_session_id_idx ON projects (session_id);
CREATE INDEX projects_user_id_idx ON projects (user_id);
```

- [ ] **Step 2: Verify it applies cleanly**

Run: `cd backend && cargo test -p nomi-server --test profile_routes -- --test-threads=1 2>&1 | head -5`
Expected: `sqlx::test` runs the full migration set (including the new file) against a fresh ephemeral database for its first test — a failure here means the SQL itself is broken. A pass (even just the first test starting) confirms the migration applied.

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/0020_projects_and_files.sql
git commit -m "feat: add projects and project_files tables

File content lives in S3 (see nomi-storage); these tables are the
Postgres-side index — ownership, plan text, build status, and per-file
metadata (path, content type, size) for listing a project's file tree
in one query."
```

---

### Task 3: `nomi-agent-planning` crate

**Files:**
- Create: `backend/crates/nomi-agent-planning/Cargo.toml`
- Create: `backend/crates/nomi-agent-planning/src/lib.rs`
- Test: `backend/crates/nomi-agent-planning/tests/planning_agent.rs`

**Interfaces:**
- Consumes: `nomi_agent_core::SubAgent`, `nomi_storage::S3Config`.
- Produces: `nomi_agent_planning::PlanningAgent::new(s3: Option<S3Config>) -> PlanningAgent`, `nomi_agent_planning::PLANNING_AGENT_TYPE: &str = "planning"`. Tools: `create_project`, `write_plan`.

- [ ] **Step 1: Create the crate**

`backend/crates/nomi-agent-planning/Cargo.toml`:

```toml
[package]
name = "nomi-agent-planning"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
nomi-storage = { path = "../nomi-storage" }
async-trait = "0.1"
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
nomi-test-support = { path = "../nomi-test-support" }
```

`backend/crates/nomi-agent-planning/src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;
use nomi_storage::S3Config;

pub const PLANNING_AGENT_TYPE: &str = "planning";

const PLANNING_SYSTEM_PROMPT: &str =
    "You help the user plan an app or script they want built. When they describe what they want, \
     call create_project with a short name and one-sentence description, then call write_plan \
     with the project_id it returns and a concise markdown plan: the files you intend to create \
     and the approach. The user will see this plan before anything gets built. After write_plan \
     succeeds, call delegate_to_agent with target_agent 'coding' and a task string that includes \
     the project ID verbatim, formatted exactly as 'Project <project_id>: <short summary of the \
     plan>' — the coding agent has no other way to know which project to write files into. Then \
     tell the user you'll let them know once it's built, and call complete_task. If project \
     creation fails because storage isn't configured, tell the user plainly that building apps \
     isn't available right now — don't retry.";

pub struct PlanningAgent {
    s3: Option<S3Config>,
}

impl PlanningAgent {
    pub fn new(s3: Option<S3Config>) -> Self {
        Self { s3 }
    }
}

#[async_trait]
impl SubAgent for PlanningAgent {
    fn agent_type(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        PLANNING_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "create_project".to_string(),
                description: "Create a new project for the app the user wants built. Call this once, before writing a plan.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Short project name"},
                        "description": {"type": "string", "description": "One-sentence description of what it does"}
                    },
                    "required": ["name", "description"]
                }),
            },
            ToolDefinition {
                name: "write_plan".to_string(),
                description: "Write or replace the build plan for a project. The user sees this before building starts.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The project ID from create_project"},
                        "plan": {"type": "string", "description": "The plan, in markdown"}
                    },
                    "required": ["project_id", "plan"]
                }),
            },
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "create_project" => create_project(conn, self.s3.is_some(), session_id, user_id, input).await,
            "write_plan" => write_plan(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "the user wants to build, create, or plan an app, website, or script"
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn can_delegate(&self) -> bool {
        true
    }
}

async fn create_project(
    conn: &mut PoolConnection<Postgres>,
    s3_configured: bool,
    session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    if !s3_configured {
        return Err("project creation isn't available right now — code storage isn't configured".to_string());
    }

    let name = input.get("name").and_then(|v| v.as_str()).ok_or("name is required")?;
    let description = input.get("description").and_then(|v| v.as_str());

    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .bind(name)
    .bind(description)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(project_id.to_string())
}

async fn write_plan(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id_str = input.get("project_id").and_then(|v| v.as_str()).ok_or("project_id is required")?;
    let project_id: Uuid = project_id_str.parse().map_err(|_| "project_id is not a valid UUID".to_string())?;
    let plan = input.get("plan").and_then(|v| v.as_str()).ok_or("plan is required")?;

    let updated = sqlx::query("UPDATE projects SET plan = $1, updated_at = now() WHERE id = $2 AND user_id = $3")
        .bind(plan)
        .bind(project_id)
        .bind(user_id)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    if updated.rows_affected() == 0 {
        return Err("project not found".to_string());
    }

    Ok("plan saved".to_string())
}
```

- [ ] **Step 2: Write the failing tests**

`backend/crates/nomi-agent-planning/tests/planning_agent.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_planning::PlanningAgent;

async fn seed_user_and_session(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_project_without_s3_configured_returns_an_error(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "create_project",
            serde_json::json!({"name": "Todo app", "description": "A simple todo list"}),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not available"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_updates_an_existing_project(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "1. index.html\n2. app.js"}),
        )
        .await
        .unwrap();
    assert_eq!(result, "plan saved");

    let stored_plan: Option<String> = sqlx::query_scalar("SELECT plan FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_plan.as_deref(), Some("1. index.html\n2. app.js"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_for_another_users_project_is_rejected(pool: PgPool) {
    let (owner_id, session_id) = seed_user_and_session(&pool).await;
    let (other_user_id, _) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(owner_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            other_user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "malicious overwrite"}),
        )
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "delete_everything", serde_json::json!({})).await;
    assert!(result.is_err());
}
```

Note: `create_project`'s happy path (S3 configured) isn't covered here since that requires a real/mocked S3 endpoint this crate has no infrastructure for — matches this codebase's existing precedent (`nomi-server`'s own avatar-upload tests only cover the `s3: None` path too, never real S3 wire behavior). Task 6's route-level tests exercise a fuller flow using a directly-seeded project.

- [ ] **Step 3: Run and verify**

Run: `cd backend && cargo test -p nomi-agent-planning`
Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add backend/crates/nomi-agent-planning
git commit -m "feat: add the planning agent

Two tools — create_project, write_plan — following the nomi-agent-money
crate shape. create_project fails cleanly when S3 isn't configured
rather than letting the feature half-work and break on the first file
write."
```

---

### Task 4: `nomi-agent-coding` crate

**Files:**
- Create: `backend/crates/nomi-agent-coding/Cargo.toml`
- Create: `backend/crates/nomi-agent-coding/src/lib.rs`
- Test: `backend/crates/nomi-agent-coding/tests/coding_agent.rs`

**Interfaces:**
- Consumes: `nomi_agent_core::SubAgent`, `nomi_storage::S3Config`.
- Produces: `nomi_agent_coding::CodingAgent::new(s3: Option<S3Config>) -> CodingAgent`, `nomi_agent_coding::CODING_AGENT_TYPE: &str = "coding"`. Tools: `write_file`, `read_file`, `list_files`, `delete_file`. Also exports `pub fn s3_key(project_id: Uuid, path: &str) -> String` and `pub fn guess_content_type(path: &str) -> &'static str` — Task 6's HTTP routes need the exact same key/content-type logic to read what this agent wrote, so it's a public function here rather than a private duplicate in `nomi-server`.

- [ ] **Step 1: Create the crate**

`backend/crates/nomi-agent-coding/Cargo.toml`:

```toml
[package]
name = "nomi-agent-coding"
version = "0.1.0"
edition = "2021"

[dependencies]
nomi-agent-core = { path = "../nomi-agent-core" }
nomi-llm = { path = "../nomi-llm" }
nomi-storage = { path = "../nomi-storage" }
async-trait = "0.1"
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
nomi-test-support = { path = "../nomi-test-support" }
```

`backend/crates/nomi-agent-coding/src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;
use nomi_storage::S3Config;

pub const CODING_AGENT_TYPE: &str = "coding";

const CODING_SYSTEM_PROMPT: &str =
    "You write real files for a project the user asked to have built, following the plan you \
     were given. The task you were delegated includes a line like 'Project <uuid>: ...' — use \
     that UUID as project_id in every tool call. Use write_file to create or overwrite files, \
     read_file to check existing content before editing it, list_files to see what's there \
     already, and delete_file to remove something you no longer need. Use relative paths with no \
     leading slash (e.g. 'index.html', 'src/app.js'). When you've finished building everything \
     the plan calls for, call complete_task with a short summary of what you built.";

/// The S3 key every file for a project lives at — shared with nomi-server's HTTP routes so both
/// sides agree on where content is, without either duplicating the format string.
pub fn s3_key(project_id: Uuid, path: &str) -> String {
    format!("projects/{project_id}/{path}")
}

/// Coarse content-type guess from a file extension — good enough for both the S3 object's
/// Content-Type and the static preview route; not a full MIME database.
pub fn guess_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "md" => "text/markdown",
        _ => "text/plain",
    }
}

pub struct CodingAgent {
    s3: Option<S3Config>,
}

impl CodingAgent {
    pub fn new(s3: Option<S3Config>) -> Self {
        Self { s3 }
    }

    fn s3(&self) -> Result<&S3Config, String> {
        self.s3.as_ref().ok_or_else(|| "code storage isn't configured".to_string())
    }
}

#[async_trait]
impl SubAgent for CodingAgent {
    fn agent_type(&self) -> &'static str {
        CODING_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        CODING_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Create or overwrite a file in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string", "description": "Relative path, e.g. 'src/index.html'"},
                        "content": {"type": "string"}
                    },
                    "required": ["project_id", "path", "content"]
                }),
            },
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the current content of a file in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string"}
                    },
                    "required": ["project_id", "path"]
                }),
            },
            ToolDefinition {
                name: "list_files".to_string(),
                description: "List every file path currently in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {"project_id": {"type": "string"}},
                    "required": ["project_id"]
                }),
            },
            ToolDefinition {
                name: "delete_file".to_string(),
                description: "Delete a file from the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string"}
                    },
                    "required": ["project_id", "path"]
                }),
            },
        ]
    }

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
            "write_file" => self.write_file(conn, user_id, input).await,
            "read_file" => self.read_file(conn, user_id, input).await,
            "list_files" => list_files(conn, user_id, input).await,
            "delete_file" => self.delete_file(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        CODING_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "writing or editing code for a project — not reachable directly, only via delegation from planning"
    }
}

fn parse_project_id(input: &Value) -> Result<Uuid, String> {
    input
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or("project_id is required")?
        .parse()
        .map_err(|_| "project_id is not a valid UUID".to_string())
}

async fn owns_project(conn: &mut PoolConnection<Postgres>, project_id: Uuid, user_id: Uuid) -> Result<bool, String> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(exists)
}

impl CodingAgent {
    async fn write_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let s3 = self.s3()?;
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?;
        let content_type = guess_content_type(path);

        s3.put_object(&s3_key(project_id, path), content, content_type).await.map_err(|e| e.to_string())?;

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

        Ok(format!("wrote {path}"))
    }

    async fn read_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let s3 = self.s3()?;
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;

        match s3.get_object(&s3_key(project_id, path)).await.map_err(|e| e.to_string())? {
            Some(content) => Ok(content),
            None => Ok("file not found".to_string()),
        }
    }

    async fn delete_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let s3 = self.s3()?;
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;

        s3.delete_object(&s3_key(project_id, path)).await.map_err(|e| e.to_string())?;
        sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
            .bind(project_id)
            .bind(path)
            .execute(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("deleted {path}"))
    }
}

async fn list_files(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = parse_project_id(&input)?;
    if !owns_project(conn, project_id, user_id).await? {
        return Err("project not found".to_string());
    }

    let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    if paths.is_empty() {
        return Ok("no files yet".to_string());
    }
    Ok(paths.join("\n"))
}
```

- [ ] **Step 2: Write the failing tests**

`backend/crates/nomi-agent-coding/tests/coding_agent.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::{guess_content_type, CodingAgent};
use nomi_agent_core::SubAgent;

async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, project_id)
}

#[test]
fn guess_content_type_covers_common_extensions() {
    assert_eq!(guess_content_type("index.html"), "text/html");
    assert_eq!(guess_content_type("styles.css"), "text/css");
    assert_eq!(guess_content_type("app.js"), "application/javascript");
    assert_eq!(guess_content_type("data.json"), "application/json");
    assert_eq!(guess_content_type("README"), "text/plain");
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_file_without_s3_configured_returns_an_error(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_file_for_a_project_you_do_not_own_is_rejected(pool: PgPool) {
    let (_owner_id, project_id) = seed_project(&pool).await;
    let other_user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            other_user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "project not found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_files_on_an_empty_project_says_so(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_files", serde_json::json!({"project_id": project_id.to_string()}))
        .await
        .unwrap();
    assert_eq!(result, "no files yet");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "run_shell", serde_json::json!({"project_id": project_id.to_string()}))
        .await;
    assert!(result.is_err());
}
```

Same note as Task 3: the S3-configured happy paths for `write_file`/`read_file`/`delete_file` aren't covered here (no S3 test double in this codebase); Task 6's route tests seed `project_files` rows directly to test the metadata side, and manual verification (Task 6, Step 4) exercises the real S3 round trip end-to-end.

- [ ] **Step 3: Run and verify**

Run: `cd backend && cargo test -p nomi-agent-coding`
Expected: 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add backend/crates/nomi-agent-coding
git commit -m "feat: add the coding agent

Four tools — write_file, read_file, list_files, delete_file — backed by
S3 for content and project_files for metadata. Exports s3_key() and
guess_content_type() so nomi-server's HTTP routes read files the same
way this agent writes them."
```

---

### Task 5: Wire both agents into the registry; thread S3 through the workers

**Files:**
- Modify: `backend/crates/nomi-server/Cargo.toml` (add `nomi-agent-planning`, `nomi-agent-coding` deps)
- Modify: `backend/crates/nomi-server/src/lib.rs` (`build_agent_registry` signature + two new entries)
- Modify: `backend/crates/nomi-server/src/worker.rs` (`run()` signature, pass `s3` to `build_agent_registry`)
- Modify: `backend/crates/nomi-server/src/delegation_worker.rs` (`run()` signature, pass `s3` to `build_agent_registry`, add the coding-specific `status = 'ready'` branch)
- Modify: `backend/crates/nomi-server/src/bin/worker.rs` (build its own `S3Config`, pass to `worker::run`)
- Modify: `backend/crates/nomi-server/src/main.rs` (pass `s3.clone()` into both worker spawns)

**Interfaces:**
- Consumes: `nomi_agent_planning::PlanningAgent`, `nomi_agent_coding::{CodingAgent, CODING_AGENT_TYPE}`, `nomi_storage::S3Config`.
- Produces: `build_agent_registry(s3: Option<S3Config>) -> AgentRegistry`.

- [ ] **Step 1: Add the crate dependencies**

`backend/crates/nomi-server/Cargo.toml` — add alongside the other `nomi-agent-*` deps:

```toml
nomi-agent-planning = { path = "../nomi-agent-planning" }
nomi-agent-coding = { path = "../nomi-agent-coding" }
```

- [ ] **Step 2: Update `build_agent_registry`**

`backend/crates/nomi-server/src/lib.rs` — replace the whole function:

```rust
pub fn build_agent_registry(s3: Option<nomi_storage::S3Config>) -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_personality::PersonalityAgent),
        Box::new(nomi_agent_supervisor::SupervisorAgent),
        Box::new(nomi_agent_planning::PlanningAgent::new(s3.clone())),
        Box::new(nomi_agent_coding::CodingAgent::new(s3)),
    ])
}
```

- [ ] **Step 3: Thread `s3` through `worker.rs`**

`backend/crates/nomi-server/src/worker.rs` — change the `run` signature (line 19) from:
```rust
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String) {
```
to:
```rust
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, s3: Option<nomi_storage::S3Config>) {
```
and change the registry line (currently `let registry = crate::build_agent_registry();`) to:
```rust
let registry = crate::build_agent_registry(s3);
```

- [ ] **Step 4: Thread `s3` through `delegation_worker.rs`, and add the ready-status branch**

Change the `run` signature the same way:
```rust
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, s3: Option<nomi_storage::S3Config>) {
```
and:
```rust
let registry = crate::build_agent_registry(s3);
```

Then, in the `match outcome` block's `Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. })` arm (the one that inserts the phrased message and marks the delegation completed), add the coding-specific status update right after the existing `INSERT INTO agent_delegations ... SET status = 'completed'` call:

```rust
if claimed.target_agent_type == nomi_agent_coding::CODING_AGENT_TYPE {
    if let Some(project_id) = extract_project_id(&claimed.task) {
        let _ = sqlx::query("UPDATE projects SET status = 'ready' WHERE id = $1 AND status = 'building'")
            .bind(project_id)
            .execute(&pool)
            .await;
    }
}
```

Add the helper function (near the bottom of the file, alongside `fail_and_notify`):

```rust
/// Planning's delegation task is always formatted "Project <uuid>: ..." (see
/// nomi-agent-planning's system prompt) — this is the only place that convention needs parsing,
/// since it's just used to know which project to mark ready, a UI/status nicety. A malformed or
/// missing UUID here is silently ignored, not an error: the project simply stays "building" and
/// nothing about the delegation itself fails over a status cosmetic.
fn extract_project_id(task: &str) -> Option<Uuid> {
    task.strip_prefix("Project ")?.split_once(':').map(|(id, _)| id.trim()).and_then(|id| id.parse().ok())
}
```

Add `use uuid::Uuid;` to the top of `delegation_worker.rs` if it isn't already imported (it is — `ClaimedDelegation.id: Uuid` already requires it).

- [ ] **Step 5: Update the standalone worker binary**

`backend/crates/nomi-server/src/bin/worker.rs` — after the existing `let mqtt = ...` line and before the final `nomi_server::worker::run(...)` call, add:

```rust
let s3 = nomi_storage::build_from_env().await;
```

and change the final call from:
```rust
nomi_server::worker::run(pool, mqtt, settings_key, http_client, database_url).await;
```
to:
```rust
nomi_server::worker::run(pool, mqtt, settings_key, http_client, database_url, s3).await;
```

- [ ] **Step 6: Update `main.rs`**

Both `tokio::spawn` blocks call `nomi_server::worker::run(...)` and `nomi_server::delegation_worker::run(...)` respectively. Since `s3` is used again afterward to build `AppState` (line ~79), clone it for each spawn. Change:

```rust
tokio::spawn(async move {
    nomi_server::worker::run(worker_pool, worker_mqtt, settings_key, worker_http_client, worker_database_url).await;
});
```
to:
```rust
let worker_s3 = s3.clone();
tokio::spawn(async move {
    nomi_server::worker::run(worker_pool, worker_mqtt, settings_key, worker_http_client, worker_database_url, worker_s3).await;
});
```

and:
```rust
tokio::spawn(async move {
    nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, settings_key, delegation_http_client, delegation_database_url).await;
});
```
to:
```rust
let delegation_s3 = s3.clone();
tokio::spawn(async move {
    nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, settings_key, delegation_http_client, delegation_database_url, delegation_s3).await;
});
```

(Place both `let ..._s3 = s3.clone();` lines before their respective `tokio::spawn`, same style as the existing `worker_pool`/`delegation_pool` clones right above each spawn.)

- [ ] **Step 7: Verify the whole workspace builds and existing tests pass**

Run: `cd backend && cargo build --workspace`
Expected: clean build. This is the step most likely to surface a missed call site — `build_agent_registry`/`worker::run`/`delegation_worker::run` signature changes are exactly the kind of change the compiler catches everywhere it's called.

Run: `cd backend && cargo test --workspace`
Expected: all existing tests still pass (352+ from before this plan, plus the 9 new ones from Tasks 3-4).

- [ ] **Step 8: Commit**

```bash
git add backend/crates/nomi-server
git commit -m "feat: register the planning and coding agents

Threads Option<S3Config> through build_agent_registry() and both
workers (the embedded ones spawned by main.rs and the standalone
bin/worker.rs) so the new agents can reach S3. The delegation worker
also marks a project 'ready' once its coding delegation completes,
parsed from the task text's 'Project <uuid>: ...' convention."
```

---

### Task 6: Backend HTTP routes for projects

**Files:**
- Create: `backend/crates/nomi-server/src/routes/projects.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs` (add `pub mod projects;`)
- Modify: `backend/crates/nomi-server/src/app.rs` (register routes)
- Test: `backend/crates/nomi-server/tests/projects_routes.rs`

**Interfaces:**
- Consumes: `nomi_agent_coding::{s3_key, guess_content_type}`, `AppState`.
- Produces: `GET /api/projects`, `GET /api/projects/:id`, `GET /api/projects/:id/files/:path`, `PUT /api/projects/:id/files/:path`, `DELETE /api/projects/:id/files/:path`, `GET /api/projects/:id/preview/*path`.

- [ ] **Step 1: Write the route handlers**

`backend/crates/nomi-server/src/routes/projects.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use nomi_agent_coding::{guess_content_type, s3_key};
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<ProjectSummary>>, (StatusCode, &'static str)> {
    let projects: Vec<ProjectSummary> = sqlx::query_as(
        "SELECT id, name, description, status, created_at, updated_at FROM projects \
         WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list projects");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list projects")
    })?;

    Ok(Json(projects))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectFileSummary {
    pub path: String,
    pub content_type: String,
    pub size_bytes: i32,
}

#[derive(Serialize)]
pub struct ProjectDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub plan: Option<String>,
    pub status: String,
    pub files: Vec<ProjectFileSummary>,
}

async fn load_owned_project(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, Option<String>, Option<String>, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT name, description, plan, status FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, &'static str)> {
    let row = load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;
    let (name, description, plan, status) = row.ok_or((StatusCode::NOT_FOUND, "project not found"))?;

    let files: Vec<ProjectFileSummary> = sqlx::query_as(
        "SELECT path, content_type, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list project files");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse { id: project_id, name, description, plan, status, files }))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content = s3.get_object(&s3_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to read file".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}

#[derive(serde::Deserialize)]
pub struct PutFileRequest {
    pub content: String,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
    Json(req): Json<PutFileRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content_type = guess_content_type(&path);
    s3.put_object(&s3_key(project_id, &path), &req.content, content_type).await.map_err(|e| {
        tracing::error!(error = %e, "failed to write project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    })?;

    sqlx::query(
        "INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()",
    )
    .bind(project_id)
    .bind(&path)
    .bind(content_type)
    .bind(req.content.len() as i32)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save project file metadata");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state, claims))]
pub async fn delete_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    s3.delete_object(&s3_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to delete project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
    })?;
    sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
        .bind(project_id)
        .bind(&path)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to delete project file metadata");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
        })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state, claims))]
pub async fn preview_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, Option<String>)>,
) -> Result<Response, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    let path = path.unwrap_or_else(|| "index.html".to_string());

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content = s3.get_object(&s3_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file for preview");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preview".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}
```

- [ ] **Step 2: Wire up modules and routes**

`backend/crates/nomi-server/src/routes/mod.rs` — add:
```rust
pub mod projects;
```
(alphabetically, between `personality` and `profile`).

`backend/crates/nomi-server/src/app.rs` — add the import near the other route imports:
```rust
use crate::routes::projects as projects_routes;
```
and these routes, placed after the `/api/preferences` route (the last one before `.layer(TraceLayer::new_for_http())`):
```rust
.route("/api/projects", get(projects_routes::list_projects))
.route("/api/projects/:id", get(projects_routes::get_project))
.route(
    "/api/projects/:id/files/*path",
    get(projects_routes::get_project_file).put(projects_routes::put_project_file).delete(projects_routes::delete_project_file),
)
.route("/api/projects/:id/preview", get(projects_routes::preview_project_file))
.route("/api/projects/:id/preview/*path", get(projects_routes::preview_project_file))
```

Note the two preview routes: axum's `:id/files/*path` wildcard requires a non-empty tail, so a bare `/preview` (no path — defaulting to `index.html`) needs its own route registered separately, both pointing at the same handler. `preview_project_file`'s `Path` extractor is `(Uuid, Option<String>)` to accept either.

- [ ] **Step 3: Write the tests**

`backend/crates/nomi-server/tests/projects_routes.rs`:

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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
    let (_, body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn seed_project(pool: &PgPool, user_id: Uuid) -> Uuid {
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id").bind(user_id).fetch_one(pool).await.unwrap();
    sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, description, plan) VALUES ($1, $2, 'Todo app', 'a simple list', '1. index.html') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user_id_by_email(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn listing_projects_only_returns_the_caller_owns(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    seed_project(&pool, user_id).await;

    let other_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let other_session: Uuid = sqlx::query_scalar("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id").bind(other_id).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Someone elses app')").bind(other_id).bind(other_session).execute(&pool).await.unwrap();

    let (status, body) = json_request(router, "GET", "/api/projects", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let projects = body.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["name"], "Todo app");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_project_includes_plan_and_files(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    let project_id = seed_project(&pool, user_id).await;
    sqlx::query("INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, 'index.html', 'text/html', 20)")
        .bind(project_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(router, "GET", &format!("/api/projects/{project_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["plan"], "1. index.html");
    assert_eq!(body["files"].as_array().unwrap().len(), 1);
    assert_eq!(body["files"][0]["path"], "index.html");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_project_for_another_users_project_is_not_found(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "attacker@example.com").await;

    let owner_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let project_id = seed_project(&pool, owner_id).await;

    let (status, _) = json_request(router, "GET", &format!("/api/projects/{project_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn getting_a_file_without_s3_configured_is_service_unavailable(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    let project_id = seed_project(&pool, user_id).await;

    let (status, _) = json_request(router, "GET", &format!("/api/projects/{project_id}/files/index.html"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unauthenticated_requests_are_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let (status, _) = json_request(router, "GET", "/api/projects", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 4: Run and verify**

Run: `cd backend && cargo test -p nomi-server --test projects_routes`
Expected: 5 tests pass.

Run: `cd backend && cargo test --workspace`
Expected: everything still green.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-server
git commit -m "feat: add HTTP routes for browsing and editing projects

GET/PUT/DELETE on individual files (proxied through the backend to S3,
not presigned — these are small text edits, not large binary uploads),
a project list/detail pair, and a preview route that serves a file's
raw content for the frontend's sandboxed-iframe preview."
```

---

### Task 7: Frontend — types and the projects list page

**Files:**
- Modify: `frontend/src/lib/types.ts` (add `ProjectSummary`, `ProjectFileSummary`, `ProjectDetail`)
- Create: `frontend/src/routes/(app)/projects/+page.server.ts`
- Create: `frontend/src/routes/(app)/projects/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/projects` (Task 6).
- Produces: `/projects` route rendering a list of the user's projects.

- [ ] **Step 1: Add types**

`frontend/src/lib/types.ts` — append:

```ts
export interface ProjectSummary {
	id: string;
	name: string;
	description: string | null;
	status: 'planning' | 'building' | 'ready';
	created_at: string;
	updated_at: string;
}

export interface ProjectFileSummary {
	path: string;
	content_type: string;
	size_bytes: number;
}

export interface ProjectDetail {
	id: string;
	name: string;
	description: string | null;
	plan: string | null;
	status: 'planning' | 'building' | 'ready';
	files: ProjectFileSummary[];
}
```

- [ ] **Step 2: List page**

`frontend/src/routes/(app)/projects/+page.server.ts`:

```ts
import { apiFetch } from '$lib/server/api';
import type { ProjectSummary } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/projects');
	const projects: ProjectSummary[] = response.ok ? await response.json() : [];
	return { projects };
};
```

`frontend/src/routes/(app)/projects/+page.svelte`:

```svelte
<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const STATUS_LABEL: Record<string, string> = {
		planning: 'Planning',
		building: 'Building…',
		ready: 'Ready',
	};
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Projects</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Apps nomi has built or is building for you. Ask nomi to build something in chat to start a new one.
</p>

<div class="mt-6 space-y-3">
	{#each data.projects as project (project.id)}
		<a href="/projects/{project.id}" class="block">
			<Card variant="outlined" class="p-4">
				<div class="flex items-center justify-between">
					<div>
						<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{project.name}</p>
						{#if project.description}
							<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{project.description}</p>
						{/if}
					</div>
					<span
						class="md-label-medium rounded-full px-3 py-1"
						style="background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container)"
					>
						{STATUS_LABEL[project.status] ?? project.status}
					</span>
				</div>
			</Card>
		</a>
	{/each}
	{#if data.projects.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No projects yet.</p>
	{/if}
</div>
```

- [ ] **Step 3: Verify**

Run: `cd frontend && npm run check`
Expected: 0 errors (new warnings matching the existing `state_referenced_locally` pattern elsewhere in this codebase are fine — see Global Constraints in every prior plan this session; this page doesn't introduce any since `data` isn't captured into `$state` here).

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/types.ts "frontend/src/routes/(app)/projects/+page.server.ts" "frontend/src/routes/(app)/projects/+page.svelte"
git commit -m "feat: add the projects list page"
```

---

### Task 8: Frontend — project detail page (file tree, editor, plan, preview)

**Files:**
- Create: `frontend/src/routes/(app)/projects/[projectId]/+page.server.ts`
- Create: `frontend/src/routes/(app)/projects/[projectId]/+page.svelte`
- Create: `frontend/src/routes/(app)/projects/[projectId]/preview/[...path]/+server.ts`
- Create: `frontend/src/lib/components/CodeEditor.svelte`
- Modify: `frontend/package.json` (add CodeMirror deps)

**Interfaces:**
- Consumes: `GET /api/projects/:id`, `GET /api/projects/:id/files/:path`, `PUT /api/projects/:id/files/:path`, `GET /api/projects/:id/preview[/*path]` (Task 6).
- Produces: `/projects/[projectId]` route with a three-pane layout; `/projects/[projectId]/preview/[...path]` as a same-origin proxy the iframe actually loads.

**Why a proxy route, not a direct iframe `src` to nomi-server:** `AuthClaims` (`nomi-auth/src/extractor.rs`) only ever reads a `Bearer` token from the `Authorization` header — never a cookie. An `<iframe src="http://nomi-server-host/...">` is a plain browser navigation with no way to attach a custom header, so it would 401 unconditionally; this isn't a CORS question; it never gets far enough to hit CORS. Every other authenticated call in this app goes through a SvelteKit server route that reads the `access_token` cookie and calls `apiFetch` (`frontend/src/lib/server/api.ts`) to attach the header server-side — the preview iframe needs the exact same proxy, just one that streams a response body through instead of returning JSON.

- [ ] **Step 1: Add CodeMirror**

```bash
cd frontend && npm install codemirror @codemirror/lang-javascript @codemirror/lang-html @codemirror/lang-css
```

- [ ] **Step 2: `CodeEditor` component**

`frontend/src/lib/components/CodeEditor.svelte` — a thin Svelte wrapper: CodeMirror manages its own DOM subtree once mounted, so this component creates one `EditorView` on mount and tears it down on unmount, reacting to `path`/`value` prop changes by resetting the document rather than trying to diff it reactively (CodeMirror's own state is the source of truth while the user is typing).

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { EditorView, basicSetup } from 'codemirror';
	import { javascript } from '@codemirror/lang-javascript';
	import { html } from '@codemirror/lang-html';
	import { css } from '@codemirror/lang-css';
	import { EditorState } from '@codemirror/state';

	let {
		path,
		value,
		onSave,
	}: {
		path: string;
		value: string;
		onSave: (content: string) => void;
	} = $props();

	let container: HTMLDivElement | undefined = $state();
	let view: EditorView | undefined;

	function languageFor(path: string) {
		if (path.endsWith('.html')) return html();
		if (path.endsWith('.css')) return css();
		if (path.endsWith('.js') || path.endsWith('.mjs')) return javascript();
		return [];
	}

	function createEditor(path: string, value: string) {
		view?.destroy();
		if (!container) return;
		view = new EditorView({
			state: EditorState.create({ doc: value, extensions: [basicSetup, languageFor(path)] }),
			parent: container,
		});
	}

	onMount(() => {
		createEditor(path, value);
		return () => view?.destroy();
	});

	// Re-create the document when the user switches files — CodeMirror owns the live text while
	// editing one file, so this only fires on a genuine path change, not on every keystroke.
	let currentPath = $state(path);
	$effect(() => {
		if (path !== currentPath) {
			currentPath = path;
			createEditor(path, value);
		}
	});

	export function getContent(): string {
		return view?.state.doc.toString() ?? value;
	}
</script>

<div class="flex h-full flex-col">
	<div bind:this={container} class="min-h-0 flex-1 overflow-auto" style="background: var(--md-sys-color-surface-container-lowest)"></div>
	<div class="flex justify-end p-2" style="border-top: 1px solid var(--md-sys-color-outline-variant)">
		<button
			type="button"
			class="md-label-large rounded-full px-4 py-2"
			style="background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); border: none; cursor: pointer"
			onclick={() => onSave(getContent())}
		>
			Save
		</button>
	</div>
</div>
```

- [ ] **Step 3: Detail page server load + actions**

`frontend/src/routes/(app)/projects/[projectId]/+page.server.ts`:

```ts
import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProjectDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}`);
	if (!response.ok) {
		throw error(response.status === 404 ? 404 : 500, 'Failed to load project.');
	}
	const project: ProjectDetail = await response.json();
	return { project };
};

export const actions: Actions = {
	loadFile: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const path = data.get('path');
		if (typeof path !== 'string' || !path) {
			return fail(400, { error: 'Invalid path.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/files/${path}`);
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to load file.' });
		}
		const content = await response.text();
		return { success: true, path, content };
	},

	saveFile: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const path = data.get('path');
		const content = data.get('content');
		if (typeof path !== 'string' || !path || typeof content !== 'string') {
			return fail(400, { error: 'Invalid file.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/files/${path}`, {
			method: 'PUT',
			body: JSON.stringify({ content }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to save file.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 4: Preview proxy route**

`frontend/src/routes/(app)/projects/[projectId]/preview/[...path]/+server.ts` — SvelteKit's `[...path]` rest parameter matches zero or more segments, so this one route handles both a bare `/preview/` (path empty, backend defaults to `index.html`) and `/preview/styles.css` etc.:

```ts
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const suffix = params.path ? `/${params.path}` : '';
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/preview${suffix}`);
	const contentType = response.headers.get('content-type') ?? 'text/plain';
	return new Response(response.body, { status: response.status, headers: { 'content-type': contentType } });
};
```

- [ ] **Step 5: Detail page component**

`frontend/src/routes/(app)/projects/[projectId]/+page.svelte`:

```svelte
<script lang="ts">
	import { deserialize } from '$app/forms';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let activePath = $state<string | null>(data.project.files[0]?.path ?? null);
	let activeContent = $state('');
	let saveError = $state<string | null>(null);
	let view = $state<'code' | 'plan' | 'preview'>(data.project.plan ? 'plan' : 'code');

	async function openFile(path: string) {
		activePath = path;
		view = 'code';
		const body = new FormData();
		body.set('path', path);
		const response = await fetch('?/loadFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type === 'success' && typeof result.data?.content === 'string') {
			activeContent = result.data.content;
		} else {
			activeContent = '';
		}
	}

	async function saveFile(content: string) {
		if (!activePath) return;
		saveError = null;
		const body = new FormData();
		body.set('path', activePath);
		body.set('content', content);
		const response = await fetch('?/saveFile', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			saveError = (result.type === 'failure' && (result.data?.error as string)) || 'Failed to save.';
		}
	}
</script>

<div class="flex h-full">
	<aside class="w-56 shrink-0 overflow-y-auto p-3" style="border-right: 1px solid var(--md-sys-color-outline-variant)">
		<h2 class="md-title-medium mb-2" style="color: var(--md-sys-color-on-surface)">{data.project.name}</h2>
		<div class="mb-3 flex gap-1">
			<button
				type="button"
				class="md-label-small rounded-full px-3 py-1"
				style="border: 1px solid var(--md-sys-color-outline); background: {view === 'plan' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
				onclick={() => (view = 'plan')}
			>
				Plan
			</button>
			<button
				type="button"
				class="md-label-small rounded-full px-3 py-1"
				style="border: 1px solid var(--md-sys-color-outline); background: {view === 'preview' ? 'var(--md-sys-color-secondary-container)' : 'transparent'}; color: var(--md-sys-color-on-surface); cursor: pointer"
				onclick={() => (view = 'preview')}
			>
				Preview
			</button>
		</div>
		<List>
			{#each data.project.files as file (file.path)}
				<button
					type="button"
					class="w-full text-left"
					style="background: none; border: none; padding: 0; cursor: pointer"
					onclick={() => openFile(file.path)}
				>
					<ListItem headline={file.path} selected={activePath === file.path && view === 'code'} />
				</button>
			{/each}
		</List>
		{#if data.project.files.length === 0}
			<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">No files yet.</p>
		{/if}
	</aside>

	<main class="min-w-0 flex-1">
		{#if saveError}
			<p class="md-body-medium p-2" style="color: var(--md-sys-color-error)">{saveError}</p>
		{/if}
		{#if view === 'plan'}
			<div class="h-full overflow-y-auto p-6">
				<pre class="md-body-medium whitespace-pre-wrap" style="color: var(--md-sys-color-on-surface)">{data.project.plan ?? 'No plan yet.'}</pre>
			</div>
		{:else if view === 'preview'}
			<iframe
				title="Project preview"
				src={`/projects/${data.project.id}/preview/`}
				sandbox="allow-scripts"
				class="h-full w-full"
				style="border: none; background: white"
			></iframe>
		{:else if activePath}
			{#key activePath}
				<CodeEditor path={activePath} value={activeContent} onSave={saveFile} />
			{/key}
		{:else}
			<div class="flex h-full items-center justify-center">
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Select a file to edit.</p>
			</div>
		{/if}
	</main>
</div>
```

- [ ] **Step 6: Verify**

Run: `cd frontend && npm run check`
Expected: 0 errors.

Run: `cd frontend && npm run build`
Expected: clean build (confirms CodeMirror's dependencies bundle correctly).

- [ ] **Step 7: Commit**

```bash
git add frontend/package.json frontend/package-lock.json frontend/src/lib/components/CodeEditor.svelte "frontend/src/routes/(app)/projects/[projectId]"
git commit -m "feat: add the project detail page — file tree, editor, plan, preview

CodeMirror 6 for editing (smaller bundle than Monaco, no web-worker
setup). Preview is a sandboxed iframe pointed at a same-origin SvelteKit
proxy route, not the backend directly — nomi-server's AuthClaims only
reads a Bearer header, which a bare iframe src can't attach."
```

---

### Task 9: Sidebar navigation entry

**Files:**
- Modify: `frontend/src/lib/components/Sidebar.svelte`

**Interfaces:**
- Consumes: none new.
- Produces: a "Projects" link in the sidebar, next to "New Chat".

- [ ] **Step 1: Add the link**

In `Sidebar.svelte`, immediately after the "+ New Chat" form block (the `<form method="POST" action="/?/newChat" ...>...</form>` — same indentation level, still inside the collapsible-aware layout), add a plain link matching the existing `m3-nav-link`-style pattern used in the admin sidebar (`frontend/src/routes/admin/(protected)/+layout.svelte`) for visual consistency — but since this file doesn't currently define that class, use the same inline-style approach already used elsewhere in this component:

```svelte
<a
	href="/projects"
	class="mx-3 mt-2 flex items-center gap-2 rounded-full px-3 py-2"
	class:justify-center={collapsed}
	style="color: var(--md-sys-color-on-surface-variant); text-decoration: none;"
>
	{#if collapsed}
		<IconAgents size={20} />
	{:else}
		<span class="md-body-medium">Projects</span>
	{/if}
</a>
```

Reuses the existing `IconAgents` icon (already imported in this file for the collapsed chat-bubble state — reusing it here for "Projects" avoids adding a new icon file for a single nav entry; swap for a dedicated `IconFolder` later if the visual distinction matters).

- [ ] **Step 2: Verify manually**

Start both dev servers (backend + frontend, per this session's established pattern — worktree or main tree as appropriate) and confirm the "Projects" link appears in the sidebar and navigates to `/projects` without errors, both collapsed and expanded.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/Sidebar.svelte
git commit -m "feat: add a Projects link to the sidebar"
```

---

## Final Verification (after all tasks)

- [ ] `cd backend && cargo test --workspace` — full suite green.
- [ ] `cd frontend && npm run check && npm run test:unit -- --run && npm run build` — all green.
- [ ] Live walkthrough: log in, open a chat, ask nomi to plan and build a tiny static app (e.g. "build me a one-page hello world site"), confirm delegation flows through planning → coding, a project appears at `/projects`, its plan and files are visible and editable, and the preview iframe renders the generated `index.html`. Clean up any test project data created during this pass per this session's established test-data-cleanup practice.
