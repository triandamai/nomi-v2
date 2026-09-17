# Agent Plan Artifacts & Side Sheet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give any agent a generic, opt-in `write_plan` tool that persists a versioned plan and
surfaces it as a clickable chip on its chat message; clicking opens a new slide-from-right side
sheet showing the full version history for that agent's task.

**Architecture:** A new `agent_plans` table holds one row per version, scoped to one
`agent_session_id`. A new engine-level `write_plan` tool (gated behind `SubAgent::supports_plans()`,
mirroring `supports_todos()`) inserts a new version on every call, stores its content in S3 when
configured (falling back to inline Postgres storage otherwise or on any S3 failure), and posts a
normal chat message whose plain-text `content` carries the full plan body — that's what keeps the
plan in the LLM's own context on later turns, no extra pointer/state plumbing needed. A new
session-scoped endpoint returns every version for one agent_session; a new `SideSheet.svelte`
component fetches and renders it.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, `nomi-storage`'s `S3Config` (already used for avatar uploads).

**Spec:** `docs/superpowers/specs/2026-09-17-agent-plan-artifacts-design.md`

## Global Constraints

- `messages.content` stays `NOT NULL` and always populated — the posted plan message's `content`
  carries the full plan body (glyph + title + body), never just a summary.
- `agent_plans.content`/`agent_plans.content_s3_key` are mutually exclusive — exactly one is ever
  set, enforced by a `CHECK` constraint, never both, never neither.
- No `org_id` column anywhere new — isolation is enforced via `session_id` → `sessions.org_id`,
  checked through the existing `authorize_session_access` helper, matching how `messages` and
  `agent_sessions` are already isolated. Never introduce a new isolation pattern.
- S3 keys for plan content are opaque (`plans/{agent_session_id}/{version}.md`) — no org prefix,
  matching `project_file_key`'s existing convention of relying on DB-layer authorization, not
  key-namespacing.
- S3 is optional infrastructure — `write_plan` must never fail just because `S3_BUCKET` isn't
  configured or an S3 call errors; it always falls back to inline Postgres storage.
- Package manager for `frontend/` is npm, never pnpm.
- Every new Rust file/function needs `#[cfg(test)]` unit tests or an integration test in `tests/`;
  every new pure TS/JS function needs a vitest test; `.svelte` components are verified via
  `svelte-check` + live browser verification (no component test harness in this repo).
- **Correction from the spec, found during planning:** the spec's frontend section proposed adding
  `agent_session_id` to `MessageItem`/`list_messages`/`get_message`. That's not viable as written —
  `messages` has no `agent_session_id` column at all (only `session_id`), and retrofitting one onto
  a core, high-traffic table is out of scope. Instead, `ContentBlock::Plan` itself carries
  `agent_session_id` (the `write_plan` tool already has it at write time) — `MessageItem` is
  untouched by this plan entirely.

---

## Task 1: Migration — `agent_plans` table

**Files:**
- Create: `backend/migrations/0024_agent_plans.sql`

**Interfaces:**
- Produces: `agent_plans` table — every later backend task binds against these exact
  column names.

- [ ] **Step 1: Write the migration**

First run `ls backend/migrations/` to confirm `0024` is in fact the next free number (as of
writing this plan, `0023_content_blocks_and_permissions.sql` is the latest) — create the
next-numbered file instead if `0024` is already taken by the time you run this.

```sql
-- backend/migrations/0024_agent_plans.sql

CREATE TABLE agent_plans (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id),
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    title             TEXT NOT NULL,
    content           TEXT,
    content_s3_key    TEXT,
    version           INT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((content IS NOT NULL) <> (content_s3_key IS NOT NULL))
);

CREATE INDEX agent_plans_lookup_idx ON agent_plans (agent_session_id, version);
```

(`agent_session_id` is deliberately not a foreign key — matching `agent_events.agent_session_id`,
already unconstrained in this codebase for the same reason: the sentinel
`agent_session_id == session_id` convention used by the default/chitchat agent, which has no real
`agent_sessions` row. Chitchat has zero tools and will never call `write_plan`, so this never
actually arises — this is a schema note, not a gap to work around.)

- [ ] **Step 2: Run migrations against the dev database**

Run: `cd backend && sqlx migrate run` (or let the next `cargo test` apply it automatically — this
repo's tests use `#[sqlx::test(migrations = "../../migrations")]`, applying every migration to a
fresh throwaway DB per test).

Verify: `docker exec backend-postgres-1 psql -U nomi -d nomi -c "\d agent_plans"` shows all 8
columns and the `CHECK` constraint.

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/0024_agent_plans.sql
git commit -m "feat: add agent_plans table for versioned agent-written plans"
```

---

## Task 2: `ContentBlock::Plan` + `SubAgent::supports_plans()`

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/content_block.rs`
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces: `nomi_agent_core::ContentBlock::Plan { plan_id, agent_session_id, title, version }` —
  Task 4 (the `write_plan` tool) constructs this; Task 7 (frontend types) mirrors its exact field
  names/shape. `SubAgent::supports_plans() -> bool` (default `false`) — Task 4 checks this to
  decide whether to register the tool; Task 5 flips it to `true` on two agents.

- [ ] **Step 1: Write the failing test**

```rust
// backend/crates/nomi-agent-core/src/content_block.rs — add to the existing `mod tests` block

#[test]
fn plan_serializes_with_a_kind_tag_and_snake_case_fields() {
    let block = ContentBlock::Plan {
        plan_id: uuid::Uuid::nil(),
        agent_session_id: uuid::Uuid::nil(),
        title: "Build the login page".to_string(),
        version: 2,
    };
    let json = serde_json::to_value(&block).unwrap();
    assert_eq!(json["kind"], "plan");
    assert_eq!(json["title"], "Build the login page");
    assert_eq!(json["version"], 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nomi-agent-core content_block`
Expected: FAIL — `ContentBlock::Plan` doesn't exist yet (compile error).

- [ ] **Step 3: Add the `Plan` variant**

```rust
// backend/crates/nomi-agent-core/src/content_block.rs — add as a new variant inside the
// existing `ContentBlock` enum, alongside FileWrite/FileDelete/TodoList/Table/ApprovalRequest
    Plan {
        plan_id: Uuid,
        agent_session_id: Uuid,
        title: String,
        version: i32,
    },
```

(No other line in the enum's `#[derive(...)]`/`#[serde(...)]` attributes changes — `Plan` picks up
the same `tag = "kind", rename_all = "snake_case"` as every other variant automatically.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nomi-agent-core content_block`
Expected: PASS (5 tests — the 4 existing plus this new one)

- [ ] **Step 5: Add `supports_plans()` to the `SubAgent` trait**

```rust
// backend/crates/nomi-agent-core/src/subagent.rs — added directly after the existing
// supports_todos() default method

    /// When true, run_agent_turn gives this agent the engine-level `write_plan` tool for writing
    /// a durable, versioned plan before or during a multi-step task — see
    /// docs/superpowers/specs/2026-09-17-agent-plan-artifacts-design.md. Off by default, same
    /// reasoning as supports_todos(): most agents don't do work substantial enough to plan first.
    fn supports_plans(&self) -> bool {
        false
    }
```

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-agent-core/src/content_block.rs backend/crates/nomi-agent-core/src/subagent.rs
git commit -m "feat: add ContentBlock::Plan and SubAgent::supports_plans()"
```

---

## Task 3: Thread `S3Config` through the turn engine

**Files:**
- Modify: `backend/crates/nomi-agent-core/Cargo.toml` (add `nomi-storage` dependency)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (`run_agent_turn`, `resolve_tool_batch`
  signatures + their 2 internal call sites)
- Modify: `backend/crates/nomi-turn/Cargo.toml` (add `nomi-storage` dependency)
- Modify: `backend/crates/nomi-turn/src/lib.rs` (`handle_inbound_message`, `process_turn`,
  `run_locked_turn`, `run_subagent_turn`, `resume_paused_turn`, `resume_locked` signatures + their
  internal call sites)
- Modify: `backend/crates/nomi-server/src/worker.rs` (`run` signature + its 2 call sites)
- Modify: `backend/crates/nomi-server/src/delegation_worker.rs` (`run` signature + its call site)
- Modify: `backend/crates/nomi-server/src/main.rs` (thread `s3` into both worker spawns)
- Modify every test in `backend/crates/nomi-agent-core/tests/engine.rs` and
  `backend/crates/nomi-turn/tests/*.rs` that calls one of the above functions directly.

**Interfaces:**
- Consumes: `nomi_storage::S3Config` (already exists — `put_object`/`get_object`, `Clone`).
- Produces: `run_agent_turn`/`resolve_tool_batch` now take `s3: Option<&S3Config>` as their new
  3rd parameter (immediately after `mqtt`); `handle_inbound_message`/`process_turn`/
  `resume_paused_turn` now take `s3: Option<&S3Config>` as their new 2nd parameter (immediately
  after `pool`/before `mqtt` in `process_turn`'s/`resume_paused_turn`'s case — see exact signatures
  below). Task 4 uses this parameter; no task before Task 4 gives it real behavior.

This task is pure plumbing — zero new behavior. Every production call site passes through
whatever `s3` value it received (or `None`, at the very top of the chain in `main.rs` when S3
isn't configured); every test call site passes `None` (no test needs real S3 behavior yet — Task 4
adds the tests that actually exercise `write_plan` with and without S3 configured).

- [ ] **Step 1: Add the `nomi-storage` dependency to `nomi-agent-core` and `nomi-turn`**

```toml
# backend/crates/nomi-agent-core/Cargo.toml — add to [dependencies]
nomi-storage = { path = "../nomi-storage" }
```

```toml
# backend/crates/nomi-turn/Cargo.toml — add to [dependencies]
nomi-storage = { path = "../nomi-storage" }
```

- [ ] **Step 2: Change `run_agent_turn`'s and `resolve_tool_batch`'s signatures**

```rust
// backend/crates/nomi-agent-core/src/engine.rs
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
```

(Only the parameter list changes — `s3` inserted as the 3rd parameter, immediately after `mqtt`.
The function body is unchanged except for its one `resolve_tool_batch(...)` call site, below.)

```rust
// backend/crates/nomi-agent-core/src/engine.rs — the one call site inside run_agent_turn's loop
        match resolve_tool_batch(conn, mqtt, s3, registry, agent, session_id, agent_session_id, user_id, &pending_tool_use_blocks, &messages, &[]).await? {
```
(was `resolve_tool_batch(conn, mqtt, registry, ...)` — only `s3` inserted, same position.)

```rust
// backend/crates/nomi-agent-core/src/engine.rs
#[allow(clippy::too_many_arguments)]
pub async fn resolve_tool_batch(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    registry: &AgentRegistry,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    tool_use_blocks: &[ContentBlock],
    conversation_so_far: &[LlmMessage],
    already_decided: &[(String, bool)],
) -> Result<ToolBatchOutcome, TurnError> {
```
(Same treatment — `s3` inserted as the 3rd parameter. The function body is otherwise unchanged in
this task; Task 4 adds the one new branch that actually reads `s3`.)

- [ ] **Step 3: Update every `run_agent_turn`/`resolve_tool_batch` call site in
      `nomi-agent-core/tests/engine.rs`**

Every call in that file currently starts `run_agent_turn(&mut conn, None, &provider, ...)` (`None`
is the `mqtt` argument). Insert `None` as the new 3rd positional argument (immediately after the
existing `mqtt` argument) in **every** such call in this file — there are 16. Do not change any
other argument. `resolve_tool_batch` is never called directly from this file (only through
`run_agent_turn`), so no separate fix is needed for it here.

Run: `cargo build -p nomi-agent-core --tests` after each batch of fixes if it helps you track
progress — the compiler's "expected N arguments, found N-1" errors point at every remaining call
site precisely; when the crate builds clean, every call site in this file is fixed.

- [ ] **Step 4: Change `nomi-turn`'s function signatures**

```rust
// backend/crates/nomi-turn/src/lib.rs
pub async fn handle_inbound_message(
    pool: &PgPool,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<TurnOutcome, TurnError> {
```
(`s3` inserted as the 2nd parameter, immediately after `pool`. Its one internal call site,
`run_locked_turn(&mut conn, None, provider, ...)`, becomes
`run_locked_turn(&mut conn, None, s3, provider, ...)` — `None` there is the existing `mqtt`
argument this function always passes; `s3` is inserted immediately after it.)

```rust
// backend/crates/nomi-turn/src/lib.rs
#[allow(clippy::too_many_arguments)]
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
```
(`s3` inserted as the 3rd parameter, immediately after `mqtt`. Its internal
`run_locked_turn(&mut conn, Some((mqtt, turn_job_id)), provider, ...)` call site becomes
`run_locked_turn(&mut conn, Some((mqtt, turn_job_id)), s3, provider, ...)`.)

```rust
// backend/crates/nomi-turn/src/lib.rs
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<(String, Option<Uuid>), TurnError> {
```
(`s3` inserted as the 3rd parameter. Its two `run_subagent_turn(conn, mqtt, provider, ...)` call
sites — one in the `RoutingOutcome::Continue` arm, one in the `RoutingOutcome::NeedsClassification`
→ default-agent arm, one more in the non-default-agent arm (three total) — each become
`run_subagent_turn(conn, mqtt, s3, provider, ...)`, `s3` inserted immediately after `mqtt` in each,
no other argument changed.)

```rust
// backend/crates/nomi-turn/src/lib.rs
#[allow(clippy::too_many_arguments)]
async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
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
        conn, mqtt, s3, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id, messages, SUBAGENT_MAX_TOKENS,
    )
    .await?;

    finish_agent_turn(conn, session_id, agent_session_id, agent, outcome).await
}
```
(`s3` inserted as the 3rd parameter; its one `run_agent_turn(...)` call site gets `s3` inserted
immediately after `mqtt`, matching Step 2's new `run_agent_turn` signature exactly.)

```rust
// backend/crates/nomi-turn/src/lib.rs
#[allow(clippy::too_many_arguments)]
pub async fn resume_paused_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<TurnOutcome, TurnError> {
```
(`s3` inserted as the 3rd parameter, immediately after `mqtt`. Its one `resume_locked(&mut conn,
mqtt, provider, ...)` call site becomes `resume_locked(&mut conn, mqtt, s3, provider, ...)`.)

```rust
// backend/crates/nomi-turn/src/lib.rs
#[allow(clippy::too_many_arguments)]
async fn resume_locked(
    conn: &mut PoolConnection<Postgres>,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<(String, Option<Uuid>), TurnError> {
```
(`s3` inserted as the 3rd parameter. Its two call sites both get `s3` inserted immediately after
`mqtt`/the `mqtt`-derived `Some((mqtt, Uuid::nil()))` argument:
`resolve_tool_batch(conn, Some((mqtt, Uuid::nil())), s3, registry, agent, ...)` and
`run_agent_turn(conn, Some((mqtt, Uuid::nil())), s3, provider, embedding_provider, registry, agent, ...)`
— every other argument in both calls is unchanged.)

- [ ] **Step 5: Update every `handle_inbound_message`/`process_turn`/`resume_paused_turn` call site
      in `nomi-turn/tests/*.rs`**

Files affected: `turn_handle_inbound_message.rs`, `turn_process.rs`, `turn_resume_paused_turn.rs`,
`turn_subagent_state_machine.rs`. Insert `None` as the new argument in the exact position Step 4
puts it in each function's signature (2nd argument for `handle_inbound_message`, 3rd for
`process_turn`/`resume_paused_turn`) in every call in these files. Same mechanical, compiler-guided
process as Step 3 — no other argument changes.

- [ ] **Step 6: Update `worker.rs` and `delegation_worker.rs`**

```rust
// backend/crates/nomi-server/src/worker.rs
pub async fn run(pool: PgPool, mqtt: MqttPublisher, s3: Option<nomi_storage::S3Config>, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: nomi_storage::LocalFsStore) {
```
(`s3` inserted as the 3rd parameter, immediately after `mqtt` — owned `Option<S3Config>`, not a
reference, matching how `mqtt`/`project_storage` are already owned at this layer. Its two call
sites become:)

```rust
            let result = nomi_turn::process_turn(
                &pool,
                &mqtt,
                s3.as_ref(),
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                claimed.id,
                claimed.session_id,
                claimed.sender_channel_identity_id,
                user_id,
                &claimed.text,
            )
            .await;
```
(was missing the `s3.as_ref(),` line — inserted immediately after `&mqtt,`.)

```rust
            let result = nomi_turn::resume_paused_turn(
                &pool, &mqtt, s3.as_ref(), provider.as_ref(), embedding_provider.as_ref(), &registry,
                claimed.message_id, &claimed.decision, claimed.remember,
            )
            .await;
```
(was `&pool, &mqtt, provider.as_ref(), ...` — `s3.as_ref(),` inserted immediately after `&mqtt,`.)

```rust
// backend/crates/nomi-server/src/delegation_worker.rs
pub async fn run(pool: PgPool, mqtt: MqttPublisher, s3: Option<nomi_storage::S3Config>, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: nomi_storage::LocalFsStore) {
```
(same treatment — `s3` inserted as the 3rd parameter. Its one `nomi_agent_core::run_agent_turn(...)`
call site gets `s3.as_ref()` inserted immediately after its `mqtt`-derived argument, matching
Step 2's `run_agent_turn` signature.)

- [ ] **Step 7: Thread `s3` from `main.rs` into both worker spawns**

```rust
// backend/crates/nomi-server/src/main.rs — inside the `if run_worker_inline` block
        let worker_mqtt_client_id = format!("nomi-orchestrator-worker-{}", uuid::Uuid::new_v4());
        let worker_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &worker_mqtt_client_id);
        let worker_pool = pool.clone();
        let worker_s3 = s3.clone();
        let worker_http_client = http_client.clone();
        let worker_database_url = database_url.clone();
        let worker_project_storage = project_storage.clone();
        tokio::spawn(async move {
            nomi_server::worker::run(worker_pool, worker_mqtt, worker_s3, settings_key, worker_http_client, worker_database_url, worker_project_storage).await;
        });

        let delegation_mqtt_client_id = format!("nomi-orchestrator-delegation-worker-{}", uuid::Uuid::new_v4());
        let delegation_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &delegation_mqtt_client_id);
        let delegation_pool = pool.clone();
        let delegation_s3 = s3.clone();
        let delegation_http_client = http_client.clone();
        let delegation_database_url = database_url.clone();
        let delegation_project_storage = project_storage.clone();
        tokio::spawn(async move {
            nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, delegation_s3, settings_key, delegation_http_client, delegation_database_url, delegation_project_storage).await;
        });
```
(was missing the `worker_s3`/`delegation_s3` `let` bindings and their corresponding arguments in
each `::run(...)` call — both inserted, everything else unchanged. `s3` here refers to the existing
`let s3 = nomi_storage::build_from_env().await;` binding already present earlier in `main`, from
the avatar-storage setup — `S3Config` is `Clone`, so `.clone()` on an `Option<S3Config>` works
directly.)

- [ ] **Step 8: Build and test the whole workspace**

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, every test passes with the **same pass count as before this task** — this
is a pure signature-threading change with zero new behavior, so no test's outcome should differ,
only its call site's argument list.

- [ ] **Step 9: Commit**

```bash
git add backend/crates/nomi-agent-core/Cargo.toml backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-core/tests/engine.rs backend/crates/nomi-turn/Cargo.toml \
        backend/crates/nomi-turn/src/lib.rs backend/crates/nomi-turn/tests/turn_handle_inbound_message.rs \
        backend/crates/nomi-turn/tests/turn_process.rs backend/crates/nomi-turn/tests/turn_resume_paused_turn.rs \
        backend/crates/nomi-turn/tests/turn_subagent_state_machine.rs backend/crates/nomi-server/src/worker.rs \
        backend/crates/nomi-server/src/delegation_worker.rs backend/crates/nomi-server/src/main.rs
git commit -m "feat: thread S3Config through the turn engine for plan storage"
```

---

## Task 4: The `write_plan` tool

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (tool definition, dispatch branch,
  `write_agent_plan` helper, `is_gateable`/`should_post` exclusions)
- Test: `backend/crates/nomi-agent-core/tests/engine.rs`

**Interfaces:**
- Consumes: `ContentBlock::Plan`, `SubAgent::supports_plans()` from Task 2; `s3: Option<&S3Config>`
  from Task 3; `agent_plans` table from Task 1.
- Produces: `WRITE_PLAN_TOOL_NAME` constant (`pub` from `nomi_agent_core`) — no later task in this
  plan needs it directly, but it follows `SHOW_TABLE_TOOL_NAME`/`UPDATE_TODOS_TOOL_NAME`'s existing
  `pub` convention for consistency.

- [ ] **Step 1: Write the failing tests**

Add to `backend/crates/nomi-agent-core/tests/engine.rs`. First, give `TestAgent` the flag (find its
`impl SubAgent for TestAgent` block from the existing `supports_todos()` override and add):

```rust
    fn supports_plans(&self) -> bool {
        true
    }
```

Then add these three tests:

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_is_only_available_to_agents_that_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // PersonalityAwareTestAgent does not override supports_plans(), so it stays false —
    // calling write_plan should come back as an unrecognized tool, same as calling any tool an
    // agent never registered.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "x", "content": "y"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &PersonalityAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    // The fake provider only queues 2 responses and PersonalityAwareTestAgent never actually
    // calls write_plan for real (the tool was never in its `tools` list, so a well-behaved LLM
    // wouldn't call it) — this test instead asserts no `write_plan` row appears in agent_plans
    // for an agent that never opted in, proving the tool registration itself is gated correctly.
    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
    let plan_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_plans WHERE agent_session_id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plan_count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_creates_a_new_version_each_call_and_posts_a_message_with_the_full_body(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "Build the login page", "content": "1. Add form\n2. Wire auth"})),
        text_response("Wrote the plan.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let (title, version, content, content_s3_key): (String, i32, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT title, version, content, content_s3_key FROM agent_plans WHERE agent_session_id = $1",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(title, "Build the login page");
    assert_eq!(version, 1);
    assert_eq!(content.as_deref(), Some("1. Add form\n2. Wire auth"));
    assert!(content_s3_key.is_none(), "no S3 configured in this test, so content must be stored inline");

    let (message_content, content_blocks): (String, serde_json::Value) = sqlx::query_as(
        "SELECT content, content_blocks FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"plan\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(message_content.contains("Build the login page"));
    assert!(message_content.contains("1. Add form\n2. Wire auth"), "the posted message's plain-text content must carry the FULL plan body, so it stays in the LLM's own context on later turns");
    assert_eq!(content_blocks[0]["kind"], "plan");
    assert_eq!(content_blocks[0]["version"], 1);
    assert_eq!(content_blocks[0]["agent_session_id"], agent_session_id.to_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_write_plan_call_creates_version_two_not_a_second_version_one(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "v1", "content": "first draft"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let provider2 = FakeLlmProvider::sequence(vec![
        tool_use_response("t2", "write_plan", serde_json::json!({"title": "v2", "content": "revised"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    run_agent_turn(&mut conn, None, None, &provider2, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM agent_plans WHERE agent_session_id = $1 ORDER BY version")
        .bind(agent_session_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(versions, vec![1, 2]);

    let plan_message_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"plan\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plan_message_count, 2, "unlike update_todos, each write_plan call posts a NEW message — history, not an upsert");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p nomi-agent-core --test engine write_plan`
Expected: FAIL — `write_plan` isn't a registered tool yet, `agent_plans` dispatch doesn't exist.

- [ ] **Step 3: Add the tool name constant and definition**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — alongside SHOW_TABLE_TOOL_NAME/UPDATE_TODOS_TOOL_NAME
pub const WRITE_PLAN_TOOL_NAME: &str = "write_plan";

fn write_plan_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: WRITE_PLAN_TOOL_NAME.to_string(),
        description: "Write a new version of your plan for this task, before or during a \
                       multi-step build. Call this again whenever the plan changes significantly \
                       — each call is a new, separately viewable version, not an edit to the last \
                       one.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Short label for this plan version"},
                "content": {"type": "string", "description": "The plan, in markdown"}
            },
            "required": ["title", "content"]
        }),
    }
}
```

- [ ] **Step 4: Register the tool in `run_agent_turn`, and exclude it from gating/generic posting**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — right after the existing
// `if agent.supports_todos() { tools.push(update_todos_tool_definition()); }` block
    if agent.supports_plans() {
        tools.push(write_plan_tool_definition());
    }
```

```rust
// backend/crates/nomi-agent-core/src/engine.rs — is_gateable: add one more exclusion
fn is_gateable(tool_name: &str) -> bool {
    tool_name != COMPLETE_TASK_TOOL_NAME
        && tool_name != DELEGATE_TOOL_NAME
        && tool_name != SHOW_TABLE_TOOL_NAME
        && tool_name != UPDATE_TODOS_TOOL_NAME
        && tool_name != WRITE_PLAN_TOOL_NAME
}
```

- [ ] **Step 5: Add the `write_agent_plan` helper**

```rust
// backend/crates/nomi-agent-core/src/engine.rs — new function, anywhere below resolve_tool_batch,
// alongside upsert_todo_list/parse_table_input

/// Inserts a new plan version (S3-backed when `s3` is configured and the upload succeeds, inline
/// in Postgres otherwise — storage never fails just because S3 is unavailable or erroring) and
/// posts a new chat message for it. Unlike `upsert_todo_list`, this always creates a NEW message
/// per call (a version history, not an in-place edit) — matching what the plan design calls for.
/// The posted message's plain-text `content` carries the FULL plan body, not a summary, since
/// that's what keeps the plan in the LLM's own context on later turns via `fetch_recent_messages`.
async fn write_agent_plan(
    conn: &mut PoolConnection<Postgres>,
    s3: Option<&nomi_storage::S3Config>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    input: &serde_json::Value,
) -> Result<String, String> {
    let title = input.get("title").and_then(|v| v.as_str()).ok_or("title is required")?.to_string();
    let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?.to_string();
    if content.trim().is_empty() {
        return Err("content must not be empty".to_string());
    }

    let version: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM agent_plans WHERE agent_session_id = $1")
        .bind(agent_session_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    let s3_key = format!("plans/{agent_session_id}/{version}.md");
    let stored_in_s3 = match s3 {
        Some(s3) => s3.put_object(&s3_key, &content, "text/markdown").await.is_ok(),
        None => false,
    };

    let plan_id: Uuid = if stored_in_s3 {
        sqlx::query_scalar(
            "INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content_s3_key, version) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(session_id)
        .bind(agent_session_id)
        .bind(user_id)
        .bind(&title)
        .bind(&s3_key)
        .bind(version)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_scalar(
            "INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(session_id)
        .bind(agent_session_id)
        .bind(user_id)
        .bind(&title)
        .bind(&content)
        .bind(version)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    let block = crate::content_block::ContentBlock::Plan { plan_id, agent_session_id, title: title.clone(), version };
    let content_blocks = serde_json::json!([block]);
    let message_text = format!("📋 {title}\n\n{content}");

    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks) VALUES ($1, NULL, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(&message_text)
    .bind(&content_blocks)
    .fetch_one(&mut **conn)
    .await
    .ok();

    if let (Some(id), Some(publisher)) = (message_id, mqtt) {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id: id }).await;
    }

    Ok(format!("plan v{version} saved"))
}
```

- [ ] **Step 6: Wire the new branch into the dispatch chain, and exclude it from generic posting**

In `resolve_tool_batch`'s execution loop, insert one more branch between the existing
`UPDATE_TODOS_TOOL_NAME` arm and the `already_decided` arm:

```rust
            } else if name.as_str() == WRITE_PLAN_TOOL_NAME {
                match write_agent_plan(conn, s3, mqtt.map(|(p, _)| p), session_id, agent_session_id, user_id, input).await {
                    Ok(text) => (text, false, None),
                    Err(err) => (err, true, None),
                }
            } else if let Some((_, approved)) = already_decided.iter().find(|(decided_id, _)| decided_id == id) {
```
(the `already_decided` arm and everything after it is unchanged — only the new `WRITE_PLAN_TOOL_NAME`
arm is inserted immediately before it.)

```rust
// backend/crates/nomi-agent-core/src/engine.rs — should_post: add one more exclusion, matching
// UPDATE_TODOS_TOOL_NAME's treatment (write_agent_plan already posted its own message above)
            let should_post = name.as_str() == SHOW_TABLE_TOOL_NAME
                || (agent.surfaces_activity()
                    && name.as_str() != COMPLETE_TASK_TOOL_NAME
                    && name.as_str() != DELEGATE_TOOL_NAME
                    && name.as_str() != UPDATE_TODOS_TOOL_NAME
                    && name.as_str() != WRITE_PLAN_TOOL_NAME);
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p nomi-agent-core --test engine`
Expected: PASS (all tests, including the 3 new ones)

- [ ] **Step 8: Commit**

```bash
git add backend/crates/nomi-agent-core/src/engine.rs backend/crates/nomi-agent-core/tests/engine.rs
git commit -m "feat: add the write_plan engine tool"
```

---

## Task 5: Enable `supports_plans()` on `PlanningAgent` and `CodingAgent`

**Files:**
- Modify: `backend/crates/nomi-agent-planning/src/lib.rs` (remove the old `write_plan` tool/
  function, add `supports_plans() -> true`)
- Modify: `backend/crates/nomi-agent-coding/src/lib.rs` (add `supports_plans() -> true`)
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (remove the now-dead `"write_plan"` arm
  from `describe_tool_error`)
- Test: `backend/crates/nomi-agent-planning/tests/planning_agent.rs`

**Interfaces:**
- Consumes: `supports_plans()` from Task 2, the generic `write_plan` tool from Task 4.
- Produces: nothing new for later tasks — this is a leaf task.

- [ ] **Step 1: Remove `PlanningAgent`'s old `write_plan` tool and function**

```rust
// backend/crates/nomi-agent-planning/src/lib.rs — in `tools()`, remove the entire
// ToolDefinition { name: "write_plan".to_string(), ... } entry, leaving only create_project:
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
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
        }]
    }
```

```rust
// backend/crates/nomi-agent-planning/src/lib.rs — in execute_tool, remove the "write_plan" match
// arm, leaving only create_project:
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
            other => Err(format!("unknown tool: {other}")),
        }
    }
```

Delete the entire `write_plan` function (the one taking `conn, user_id, input` and updating
`projects.plan` via `UPDATE projects SET plan = $1, ...`) — nothing else in the file calls it once
the two edits above land. `create_project` is untouched.

```rust
// backend/crates/nomi-agent-core/src/engine.rs — describe_tool_error: remove the now-dead
// "write_plan" arm (the new write_plan tool handles its own error formatting directly in its
// dispatch branch — see Task 4 — so this function can never be reached with tool_name ==
// "write_plan" again once PlanningAgent's old tool is gone)
fn describe_tool_error(tool_name: &str, input: &serde_json::Value, result: &str) -> String {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("the file");
    match tool_name {
        "write_file" => format!("⚠️ Couldn't write `{path}` — {result}"),
        "read_file" => format!("⚠️ Couldn't read `{path}` — {result}"),
        "delete_file" => format!("⚠️ Couldn't delete `{path}` — {result}"),
        "list_files" => format!("⚠️ Couldn't list project files — {result}"),
        "create_project" => format!("⚠️ Couldn't create the project — {result}"),
        other => format!("⚠️ `{other}` failed — {result}"),
    }
}
```
(only the `"write_plan" => format!("⚠️ Couldn't save the plan — {result}"),` line is removed —
every other arm is unchanged.)

- [ ] **Step 2: Add `supports_plans()` to both agents**

```rust
// backend/crates/nomi-agent-planning/src/lib.rs — inside `impl SubAgent for PlanningAgent`,
// alongside the existing supports_todos()
    fn supports_plans(&self) -> bool {
        true
    }
```

```rust
// backend/crates/nomi-agent-coding/src/lib.rs — inside `impl SubAgent for CodingAgent`,
// alongside the existing supports_todos()
    fn supports_plans(&self) -> bool {
        true
    }
```

- [ ] **Step 3: Delete the two now-dead tests in `planning_agent.rs`**

`backend/crates/nomi-agent-planning/tests/planning_agent.rs` tests every tool by calling
`agent.execute_tool(...)` directly (no `run_agent_turn`, no engine dispatch — this file has no
`FakeLlmProvider`/`AgentRegistry` usage at all, unlike `engine.rs`'s tests). Now that `write_plan`
is dispatched by the engine (`resolve_tool_batch`) BEFORE it would ever reach
`PlanningAgent::execute_tool`, calling `execute_tool(..., "write_plan", ...)` directly against this
agent can never exercise the real mechanism again — `PlanningAgent::execute_tool`'s `match` no
longer has a `"write_plan"` arm at all after Step 1, so both calls would just hit the `other =>
Err(...)` catch-all.

Delete these two tests entirely — they test a code path that no longer exists in this file's
tested unit:
- `write_plan_updates_an_existing_project` (lines 96-129)
- `write_plan_for_another_users_project_is_rejected` (lines 131-157)

The generic `write_plan` mechanism itself is already thoroughly covered by Task 4's tests in
`nomi-agent-core/tests/engine.rs` (opt-in gating, version increment, message posting via
`run_agent_turn`) — `PlanningAgent`/`CodingAgent` opting in is a one-line flag flip with no new
logic of its own, so it needs no separate test here. `create_project_succeeds_and_stamps_the_
calling_session`, `create_project_renames_a_pre_existing_placeholder_instead_of_duplicating`, and
`unknown_tool_name_returns_an_error` are all untouched — `create_project` and the unknown-tool
rejection path are unrelated to this change.

- [ ] **Step 4: Run tests, then the whole workspace**

Run: `cargo test -p nomi-agent-planning && cargo test -p nomi-agent-coding`
Expected: PASS

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all tests pass.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-agent-planning/src/lib.rs backend/crates/nomi-agent-planning/tests/planning_agent.rs \
        backend/crates/nomi-agent-coding/src/lib.rs backend/crates/nomi-agent-core/src/engine.rs
git commit -m "feat: PlanningAgent and CodingAgent opt into the generic write_plan tool"
```

---

## Task 6: Version-history API endpoint

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/sessions.rs` (new `AgentPlanItem`,
  `ListAgentPlansResponse`, `list_agent_plans` handler)
- Modify: `backend/crates/nomi-server/src/app.rs` (route wiring)
- Test: `backend/crates/nomi-server/tests/sessions_routes.rs`

**Interfaces:**
- Consumes: `agent_plans` table from Task 1; `AppState.s3: Option<S3Config>` (already exists,
  built in `main.rs`, used today for avatar routes).
- Produces: `GET /api/sessions/:sessionId/agent-plans/:agentSessionId` — Task 7's frontend proxy
  route calls this exact path.

- [ ] **Step 1: Write the failing tests**

Add to `backend/crates/nomi-server/tests/sessions_routes.rs` (follow this file's existing
`test_state`/`register_and_login`/`json_request` helpers, already used by `get_message`'s tests):

```rust
#[sqlx::test(migrations = "../../migrations")]
async fn list_agent_plans_returns_every_version_ordered_oldest_first(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "plans@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let session_uuid = uuid::Uuid::parse_str(session_id).unwrap();
    let agent_session_id = uuid::Uuid::new_v4();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1").fetch_one(&pool).await.unwrap();

    sqlx::query("INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, $2, $3, 'v1', 'first', 1), ($1, $2, $3, 'v2', 'second', 2)")
        .bind(session_uuid)
        .bind(agent_session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_id}/agent-plans/{agent_session_id}"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let plans = body["plans"].as_array().unwrap();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0]["version"], 1);
    assert_eq!(plans[0]["content"], "first");
    assert_eq!(plans[1]["version"], 2);
    assert_eq!(plans[1]["content"], "second");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_agent_plans_rejects_access_from_a_user_outside_the_session_org(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let owner_token = register_and_login(router.clone(), "owner@example.com").await;
    let outsider_token = register_and_login(router.clone(), "outsider@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&owner_token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let agent_session_id = uuid::Uuid::new_v4();

    let (status, _) = json_request(
        router,
        "GET",
        &format!("/api/sessions/{session_id}/agent-plans/{agent_session_id}"),
        Value::Null,
        Some(&outsider_token),
    )
    .await;
    // authorize_session_access returns 404, not 403, for cross-org access — deliberately not
    // leaking "this session exists but you can't see it" (see its own source: a failed
    // authorize_org_action maps to StatusCode::NOT_FOUND, same as a session that doesn't exist
    // at all).
    assert_eq!(status, StatusCode::NOT_FOUND);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p nomi-server --test sessions_routes list_agent_plans`
Expected: FAIL — route doesn't exist yet (404s instead of the expected status).

- [ ] **Step 3: Add the response types and handler**

```rust
// backend/crates/nomi-server/src/routes/sessions.rs
#[derive(Serialize)]
pub struct AgentPlanItem {
    pub id: Uuid,
    pub title: String,
    pub version: i32,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListAgentPlansResponse {
    pub plans: Vec<AgentPlanItem>,
}

pub async fn list_agent_plans(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, agent_session_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ListAgentPlansResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let rows: Vec<(Uuid, String, i32, Option<String>, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, title, version, content, content_s3_key, created_at FROM agent_plans \
         WHERE session_id = $1 AND agent_session_id = $2 ORDER BY version ASC",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch plans"))?;

    let mut plans = Vec::with_capacity(rows.len());
    for (id, title, version, content, content_s3_key, created_at) in rows {
        let resolved_content = match (content, content_s3_key) {
            (Some(inline), _) => Some(inline),
            (None, Some(key)) => match &state.s3 {
                Some(s3) => s3.get_object(&key).await.ok().flatten(),
                None => None,
            },
            (None, None) => None,
        };
        plans.push(AgentPlanItem { id, title, version, content: resolved_content, created_at });
    }

    Ok(Json(ListAgentPlansResponse { plans }))
}
```

(`state.s3` — `AppState` already has `pub s3: Option<S3Config>`, used today by the avatar routes;
no `AppState` change is needed.)

- [ ] **Step 4: Wire the route**

```rust
// backend/crates/nomi-server/src/app.rs — add alongside the existing
// "/api/sessions/:id/messages/:message_id" route
        .route(
            "/api/sessions/:id/agent-plans/:agent_session_id",
            get(sessions_routes::list_agent_plans),
        )
```

- [ ] **Step 5: Run tests, then the whole workspace**

Run: `cargo test -p nomi-server --test sessions_routes`
Expected: PASS (all tests, including the 2 new ones)

Run: `cd backend && cargo build --workspace && cargo test --workspace`
Expected: builds clean, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-server/src/routes/sessions.rs backend/crates/nomi-server/src/app.rs \
        backend/crates/nomi-server/tests/sessions_routes.rs
git commit -m "feat: add the agent-plans version-history endpoint"
```

---

## Task 7: Frontend types, `buildAgentPlansFetchUrl`, and the proxy routes

**Files:**
- Modify: `frontend/src/lib/types.ts` (`ContentBlock` gains the `plan` variant, new
  `AgentPlanItem`/`AgentPlansResponse` types)
- Create: `frontend/src/lib/buildAgentPlansFetchUrl.ts`
- Create: `frontend/src/lib/buildAgentPlansFetchUrl.test.ts`
- Create: `frontend/src/lib/server/fetchAgentPlans.ts`
- Create: `frontend/src/routes/(app)/chat/[sessionId]/agent-plans/[agentSessionId]/+server.ts`
- Create: `frontend/src/routes/(app)/projects/session/[sessionId]/agent-plans/[agentSessionId]/+server.ts`

**Interfaces:**
- Consumes: `ContentBlock::Plan`'s exact field names from Task 2; the endpoint from Task 6.
- Produces: `ContentBlock` (TS) gains `{ kind: 'plan'; plan_id: string; agent_session_id: string;
  title: string; version: number }`; `AgentPlanItem { id, title, version, content: string | null,
  created_at }`; `buildAgentPlansFetchUrl(pathname, agentSessionId) -> string` — Task 9's
  `PlanBlock.svelte` calls both the proxy route these tasks create and this URL builder.

- [ ] **Step 1: Add the TypeScript types**

```typescript
// frontend/src/lib/types.ts — add to the ContentBlock union, alongside the other 5 variants
	| { kind: 'plan'; plan_id: string; agent_session_id: string; title: string; version: number }
```

```typescript
// frontend/src/lib/types.ts — new types, anywhere near MessageItem
export interface AgentPlanItem {
	id: string;
	title: string;
	version: number;
	content: string | null;
	created_at: string;
}

export interface AgentPlansResponse {
	plans: AgentPlanItem[];
}
```

- [ ] **Step 2: Write the failing test for `buildAgentPlansFetchUrl`**

```typescript
// frontend/src/lib/buildAgentPlansFetchUrl.test.ts
import { describe, expect, it } from 'vitest';
import { buildAgentPlansFetchUrl } from './buildAgentPlansFetchUrl';

describe('buildAgentPlansFetchUrl', () => {
	it('builds the session-scoped path under the /chat/:sessionId route', () => {
		expect(buildAgentPlansFetchUrl('/chat/abc123', 'agent-1')).toBe('/chat/abc123/agent-plans/agent-1');
	});

	it('builds the session-scoped path under the /projects/session/:sessionId route', () => {
		expect(buildAgentPlansFetchUrl('/projects/session/abc123', 'agent-1')).toBe(
			'/projects/session/abc123/agent-plans/agent-1',
		);
	});

	it('does not drop the session ID the way WHATWG relative-URL resolution would', () => {
		// Same regression class buildMessageFetchUrl.ts guards against — plain concatenation
		// from the current page's own pathname, never `new URL(relative, base)` resolution,
		// since neither route renders with a trailing slash.
		const result = buildAgentPlansFetchUrl('/chat/abc123', 'agent-1');
		expect(result).toContain('/chat/abc123/');
		expect(result).not.toBe('/chat/agent-plans/agent-1');
	});
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd frontend && npx vitest run src/lib/buildAgentPlansFetchUrl.test.ts`
Expected: FAIL — module doesn't exist yet.

- [ ] **Step 4: Write `buildAgentPlansFetchUrl`**

```typescript
// frontend/src/lib/buildAgentPlansFetchUrl.ts
/** Builds the absolute path for fetching one agent_session's plan version history, from the
 * current page's own pathname — same reasoning as buildMessageFetchUrl.ts: ChatThread.svelte
 * (and, transitively, PlanBlock.svelte) is mounted under two different route trees
 * (`/chat/[sessionId]` and `/projects/session/[sessionId]`), each with its own sibling
 * `agent-plans/[agentSessionId]` proxy route, and neither renders with a trailing slash — so
 * this is plain string concatenation, never `new URL(relative, base)` resolution. */
export function buildAgentPlansFetchUrl(pathname: string, agentSessionId: string): string {
	return `${pathname}/agent-plans/${agentSessionId}`;
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd frontend && npx vitest run src/lib/buildAgentPlansFetchUrl.test.ts`
Expected: PASS (3 tests)

- [ ] **Step 6: Write the shared server-side fetch-and-render helper**

```typescript
// frontend/src/lib/server/fetchAgentPlans.ts
import { error } from '@sveltejs/kit';
import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { AgentPlansResponse } from '$lib/types';

/** Fetches every plan version for one agent_session and server-renders each version's markdown
 * body — mirrors fetchRenderedMessage.ts's pattern (raw content from the backend, HTML rendered
 * here before it ever reaches the client). A version whose content is `null` (an S3 read failure
 * on the backend) keeps `content_html: null` — the side sheet shows an inline "could not load
 * this version" state for that one entry rather than failing the whole list. */
export async function fetchAgentPlans(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	agentSessionId: string,
): Promise<{ id: string; title: string; version: number; content_html: string | null; created_at: string }[]> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/agent-plans/${agentSessionId}`);
	if (!response.ok) {
		throw error(response.status, 'Could not load this plan.');
	}
	const { plans } = (await response.json()) as AgentPlansResponse;
	return Promise.all(
		plans.map(async (plan) => ({
			id: plan.id,
			title: plan.title,
			version: plan.version,
			content_html: plan.content !== null ? await renderMarkdown(plan.content) : null,
			created_at: plan.created_at,
		})),
	);
}
```

- [ ] **Step 7: Add the two proxy routes**

```typescript
// frontend/src/routes/(app)/chat/[sessionId]/agent-plans/[agentSessionId]/+server.ts
import { json } from '@sveltejs/kit';
import { fetchAgentPlans } from '$lib/server/fetchAgentPlans';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const plans = await fetchAgentPlans(fetch, cookies, params.sessionId, params.agentSessionId);
	return json({ plans });
};
```

```typescript
// frontend/src/routes/(app)/projects/session/[sessionId]/agent-plans/[agentSessionId]/+server.ts
import { json } from '@sveltejs/kit';
import { fetchAgentPlans } from '$lib/server/fetchAgentPlans';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const plans = await fetchAgentPlans(fetch, cookies, params.sessionId, params.agentSessionId);
	return json({ plans });
};
```

- [ ] **Step 8: Type-check**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors.

- [ ] **Step 9: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/lib/buildAgentPlansFetchUrl.ts \
        frontend/src/lib/buildAgentPlansFetchUrl.test.ts frontend/src/lib/server/fetchAgentPlans.ts \
        "frontend/src/routes/(app)/chat/[sessionId]/agent-plans/[agentSessionId]/+server.ts" \
        "frontend/src/routes/(app)/projects/session/[sessionId]/agent-plans/[agentSessionId]/+server.ts"
git commit -m "feat: add agent-plans frontend types, URL builder, and proxy routes"
```

---

## Task 8: `SideSheet.svelte`

**Files:**
- Create: `frontend/src/lib/components/m3/SideSheet.svelte`

**Interfaces:**
- Consumes: nothing new.
- Produces: `<SideSheet bind:open>{@render children()}</SideSheet>` — Task 9's `PlanBlock.svelte`
  wraps its version-viewer content in this.

- [ ] **Step 1: Write the component**

Modeled directly on `BottomSheet.svelte`'s open/close mechanics (native `<dialog>`,
`showModal()`/`close()`, backdrop-click-to-dismiss) but anchored to the right edge instead of the
bottom, and without the drag-to-dismiss gesture (not requested for this component — `BottomSheet`'s
pointer-drag handlers are deliberately not carried over).

```svelte
<!-- frontend/src/lib/components/m3/SideSheet.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		children,
		class: extraClass = '',
	}: {
		open?: boolean;
		children: Snippet;
		class?: string;
	} = $props();

	let dialogEl: HTMLDialogElement | undefined = $state();

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
		} else if (!open && dialogEl.open) {
			dialogEl.close();
		}
	});

	function handleClose() {
		open = false;
	}

	function handleDialogClick(event: MouseEvent) {
		if (event.target === dialogEl) open = false;
	}
</script>

<dialog
	bind:this={dialogEl}
	class="m3-side-sheet {extraClass}"
	onclose={handleClose}
	onclick={handleDialogClick}
>
	<div class="m3-side-sheet__panel">
		<div class="m3-side-sheet__body">
			{@render children()}
		</div>
	</div>
</dialog>

<style>
	.m3-side-sheet {
		margin: 0 0 0 auto;
		padding: 0;
		border: none;
		width: 100%;
		max-width: 420px;
		height: 100%;
		max-height: 100%;
		background: transparent;
	}

	.m3-side-sheet::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 48%, transparent);
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	@starting-style {
		.m3-side-sheet::backdrop {
			background-color: transparent;
		}
		.m3-side-sheet {
			transform: translateX(100%);
		}
	}

	.m3-side-sheet__panel {
		height: 100%;
		border-radius: var(--md-sys-shape-corner-extra-large) 0 0 var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
		display: flex;
		flex-direction: column;
		transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-side-sheet__body {
		overflow-y: auto;
		padding: 24px;
		flex: 1;
	}
</style>
```

(`margin: 0 0 0 auto` anchors the `<dialog>` to the right edge of the viewport, the MD3 side-sheet
equivalent of `BottomSheet`'s `margin: auto auto 0 auto`; the `@starting-style` block gives the
panel itself a `translateX(100%)` entry transition, the horizontal equivalent of what
`BottomSheet`'s backdrop-only starting-style already does vertically via the dialog's default
top-layer behavior.)

- [ ] **Step 2: Type-check**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/m3/SideSheet.svelte
git commit -m "feat: add the SideSheet MD3 component"
```

---

## Task 9: `PlanBlock.svelte` + `ContentBlockView` wiring

**Files:**
- Create: `frontend/src/lib/components/blocks/PlanBlock.svelte`
- Modify: `frontend/src/lib/components/blocks/ContentBlockView.svelte`

**Interfaces:**
- Consumes: `SideSheet` from Task 8; `buildAgentPlansFetchUrl`/`AgentPlanItem`-shaped response from
  Task 7.
- Produces: nothing later in this plan depends on — this is the final, leaf task.

- [ ] **Step 1: Write `PlanBlock.svelte`**

```svelte
<!-- frontend/src/lib/components/blocks/PlanBlock.svelte -->
<script lang="ts">
	import { page } from '$app/state';
	import SideSheet from '$lib/components/m3/SideSheet.svelte';
	import { buildAgentPlansFetchUrl } from '$lib/buildAgentPlansFetchUrl';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'plan' }> } = $props();

	type PlanVersion = { id: string; title: string; version: number; content_html: string | null; created_at: string };

	let sheetOpen = $state(false);
	let loading = $state(false);
	let versions = $state<PlanVersion[]>([]);
	let activeVersion = $state<number | null>(null);
	let loadError = $state<string | null>(null);

	async function openSheet() {
		sheetOpen = true;
		if (versions.length > 0) return;
		loading = true;
		loadError = null;
		try {
			const response = await fetch(buildAgentPlansFetchUrl(page.url.pathname, block.agent_session_id));
			if (!response.ok) throw new Error('failed to load');
			const data = (await response.json()) as { plans: PlanVersion[] };
			versions = data.plans;
			activeVersion = block.version;
		} catch {
			loadError = 'Could not load this plan.';
		} finally {
			loading = false;
		}
	}

	const active = $derived(versions.find((v) => v.version === activeVersion) ?? null);
</script>

<button type="button" class="m3-block-card m3-block-card--plan" onclick={openSheet}>
	<span aria-hidden="true">📋</span>
	<span class="md-body-medium">{block.title} · v{block.version}</span>
</button>

<SideSheet bind:open={sheetOpen}>
	{#if loading}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Loading…</p>
	{:else if loadError}
		<p class="md-body-medium" style="color: var(--md-sys-color-error)">{loadError}</p>
	{:else}
		<div class="m3-plan-versions">
			{#each versions as v (v.id)}
				<button
					type="button"
					class="m3-plan-versions__chip"
					class:m3-plan-versions__chip--active={v.version === activeVersion}
					onclick={() => (activeVersion = v.version)}
				>
					v{v.version}
				</button>
			{/each}
		</div>
		{#if active}
			<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{active.title}</h2>
			{#if active.content_html !== null}
				<div class="md-body-medium">{@html active.content_html}</div>
			{:else}
				<p class="md-body-medium" style="color: var(--md-sys-color-error)">Could not load this version.</p>
			{/if}
		{/if}
	{/if}
</SideSheet>

<style>
	.m3-block-card--plan {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		cursor: pointer;
		font: inherit;
		color: inherit;
		text-align: left;
	}

	.m3-plan-versions {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 16px;
	}

	.m3-plan-versions__chip {
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		cursor: pointer;
	}

	.m3-plan-versions__chip--active {
		background: var(--md-sys-color-primary);
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
</style>
```

(`page.url.pathname` from `$app/state` — the exact same reactive-current-path pattern
`ChatThread.svelte`'s `fetchAndUpsertMessage` already uses for `buildMessageFetchUrl`, verified
correct there across both route trees.)

- [ ] **Step 2: Wire it into the dispatcher**

```svelte
<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import FileWriteBlock from './FileWriteBlock.svelte';
	import FileDeleteBlock from './FileDeleteBlock.svelte';
	import TodoListBlock from './TodoListBlock.svelte';
	import TableBlock from './TableBlock.svelte';
	import ApprovalCard from './ApprovalCard.svelte';
	import PlanBlock from './PlanBlock.svelte';
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
{:else if block.kind === 'plan'}
	<PlanBlock {block} />
{/if}
```
(only the new import and the final `{:else if}` branch are added — every existing branch is
untouched.)

- [ ] **Step 3: Type-check and live-verify**

Run: `cd frontend && npx svelte-check --output human`
Expected: 0 errors.

Live-verify (dev server running, per this repo's established practice for `.svelte` components
with no test harness): seed a throwaway `agent_plans` row plus a message whose `content_blocks` is
`[{"kind":"plan","plan_id":"<uuid>","agent_session_id":"<uuid>","title":"Test plan","version":1}]`
directly via Postgres, load the chat, confirm the chip renders, click it, confirm the side sheet
slides in from the right and shows the plan body. Clean up the throwaway row/message afterward.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/blocks/PlanBlock.svelte frontend/src/lib/components/blocks/ContentBlockView.svelte
git commit -m "feat: add PlanBlock and wire it into ContentBlockView"
```
