# Sub-Agent Spawning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the loop described in `docs/superpowers/specs/2026-07-26-sub-agent-spawning-design.md`: a generic multi-turn tool-calling loop, a `SubAgent` trait, one concrete money agent, and the `agent_sessions` state-machine wiring (spawn/continue/complete/expire) that replaces `routing.rs`'s current always-`None` stub.

**Architecture:** Four new modules under `backend/src/turn/` (`tools.rs`, `subagent.rs`, `money_agent.rs`, plus additions to the existing `routing.rs`), and a rewritten `handle_inbound_message` body in `backend/src/turn/mod.rs` — its public signature does **not** change, so no existing caller/test needs updating for that reason alone. `routing::find_active_agent_session` (from the turn-loop plan) is left completely untouched; new, purely-additive functions sit alongside it.

**Tech Stack:** Rust, `sqlx` (Postgres), the existing `LlmProvider`/`ToolDefinition`/`ContentBlock` types from `backend/src/llm/`. **No new Cargo.toml dependencies.**

## Global Constraints

- No new Cargo.toml dependencies.
- The tool-calling loop caps at **10 tool-turns** per single inbound message; exceeding it returns `TurnError::ToolLoopExceeded` — a real error, not a silent truncation.
- The shared `complete_task(status: "completed" | "cancelled", summary: String)` tool is added to every sub-agent's tool list by the loop itself, and intercepted by the loop — never delegated to `SubAgent::execute_tool`.
- Every tool call (including `complete_task`) is logged as a `ToolCalled` `agent_events` row **immediately as it happens** — not batched with the turn's final persist step — so a loop that hits the cap still leaves a durable record of every step attempted.
- Intent classification (`chitchat` vs `money`) reuses the same `provider: &dyn LlmProvider` already passed into `handle_inbound_message` — no second model/provider parameter. On any classification failure or unrecognized response text, it fails open to `chitchat`.
- Agent expiry timeout is **24 hours** since `last_activity_at`, per the existing `docs/superpowers/specs/2026-07-22-sub-agent-lifecycle-design.md`.
- `mock_transactions` is a stub table — no seed data ships in its migration; tests seed their own rows.
- `agent_sessions.state` (JSONB) is not read or written by anything in this plan — the money agent's tools are stateless per turn, and nothing yet needs the scratch-data mechanism the column exists for. It stays at its schema default (`'{}'`) for the lifetime of every `agent_sessions` row this plan creates.
- No live MQTT publishing — `agent_events` remains the durable audit record, as in every prior plan in this project.
- `Postgres SUM(bigint_column)` returns `NUMERIC`, not `BIGINT` — every aggregate query in this plan casts explicitly (`SUM(amount_cents)::bigint`), verified against this project's real Postgres instance before being written into this plan (an uncast `SUM` fails at runtime with `ColumnDecode { ... mismatched types ... SQL type NUMERIC }`, not a compile error).

## File Structure

- `backend/src/turn/subagent.rs` — the `SubAgent` trait every concrete sub-agent implements.
- `backend/src/turn/tools.rs` — the generic tool-calling loop (`run_tool_calling_loop`, `LoopOutcome`, `COMPLETE_TASK_TOOL_NAME`).
- `backend/src/turn/money_agent.rs` — the concrete `MoneyAgent` (`list_transactions`, `summarize_budget`, its system prompt).
- `backend/migrations/0009_mock_transactions.sql` — the money agent's stub data table.
- `backend/src/turn/routing.rs` — gains `classify_intent`/`Intent` (Task 3) and `ActiveAgentSessionDetails`/`load_active_agent_session_details`/`is_stale`/`mark_expired`/`spawn_agent_session`/`complete_agent_session` (Task 4), all additive; `find_active_agent_session` is untouched.
- `backend/src/turn/types.rs` — gains `TurnError::ToolLoopExceeded` (Task 1) and `TurnError::UnknownAgentType(String)` (Task 4).
- `backend/src/turn/mod.rs` — `handle_inbound_message`'s body is rewritten in Task 4 (its public signature is unchanged); gains `run_subagent_turn` and `fetch_recent_messages` helpers.
- `backend/tests/support/mod.rs` — gains a `FakeOutcome::Sequence` variant and `FakeLlmProvider::sequence(...)` constructor (Task 1), purely additive — `FakeLlmProvider::success`/`failure`/`with_delay` and `FakeEmbeddingProvider` are untouched.
- New test files: `backend/tests/turn_tools.rs`, `backend/tests/turn_money_agent.rs`, `backend/tests/turn_intent_classification.rs`, `backend/tests/turn_subagent_state_machine.rs`.

---

### Task 1: Tool-calling loop + `SubAgent` trait

**Files:**
- Create: `backend/src/turn/subagent.rs`
- Create: `backend/src/turn/tools.rs`
- Modify: `backend/src/turn/types.rs` (add `ToolLoopExceeded`)
- Modify: `backend/src/turn/mod.rs` (add `pub mod subagent; pub mod tools;`)
- Modify: `backend/tests/support/mod.rs` (add `FakeOutcome::Sequence` + `FakeLlmProvider::sequence`)
- Test: `backend/tests/turn_tools.rs`

**Interfaces:**
- Produces: `trait SubAgent: Send + Sync { fn agent_type(&self) -> &'static str; fn system_prompt(&self) -> &'static str; fn tools(&self) -> Vec<ToolDefinition>; async fn execute_tool(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, name: &str, input: Value) -> Result<String, String>; }`, `enum LoopOutcome { Reply(String), Completed { status: String, summary: String } }`, `const COMPLETE_TASK_TOOL_NAME: &str`, `async fn run_tool_calling_loop(conn: &mut PoolConnection<Postgres>, provider: &dyn LlmProvider, agent: &dyn SubAgent, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, messages: Vec<LlmMessage>, max_tokens: u32) -> Result<LoopOutcome, TurnError>`. Task 2 implements `SubAgent` for `MoneyAgent`. Task 4 calls `run_tool_calling_loop`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/support/mod.rs — FULL FILE (existing content plus the additions below)
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nomi_orchestrator::embedding::{EmbeddingError, EmbeddingProvider};
use nomi_orchestrator::llm::{LlmError, LlmProvider, LlmRequest, LlmResponse};

enum FakeOutcome {
    Success(LlmResponse),
    Failure(String),
    Sequence(Mutex<VecDeque<LlmResponse>>),
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

    pub fn sequence(responses: Vec<LlmResponse>) -> Self {
        Self {
            outcome: FakeOutcome::Sequence(Mutex::new(responses.into())),
            delay: None,
            received_requests: Mutex::new(Vec::new()),
        }
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
            FakeOutcome::Sequence(queue) => queue
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| LlmError::ProviderError("sequence exhausted".to_string())),
        }
    }
}

enum FakeEmbeddingOutcome {
    Success(Vec<f32>),
    Failure(String),
}

pub struct FakeEmbeddingProvider {
    outcome: FakeEmbeddingOutcome,
}

impl FakeEmbeddingProvider {
    pub fn success(vector: Vec<f32>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Success(vector) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Failure(message.into()) }
    }
}

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        match &self.outcome {
            FakeEmbeddingOutcome::Success(vector) => Ok(vector.clone()),
            FakeEmbeddingOutcome::Failure(message) => Err(EmbeddingError::ProviderError(message.clone())),
        }
    }
}

pub fn dummy_embedding() -> Vec<f32> {
    vec![0.0; 1536]
}
```

```rust
// backend/tests/turn_tools.rs
mod support;

use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_orchestrator::turn::subagent::SubAgent;
use nomi_orchestrator::turn::tools::{run_tool_calling_loop, LoopOutcome, COMPLETE_TASK_TOOL_NAME};
use nomi_orchestrator::turn::TurnError;

use support::FakeLlmProvider;

struct TestAgent;

#[async_trait::async_trait]
impl SubAgent for TestAgent {
    fn agent_type(&self) -> &'static str {
        "test"
    }
    fn system_prompt(&self) -> &'static str {
        "test prompt"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "echo".to_string(),
            description: "Echoes back its input".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        }]
    }
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<String, String> {
        match name {
            "echo" => Ok(format!("echoed: {input}")),
            other => Err(format!("unknown tool: {other}")),
        }
    }
}

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
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'test', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, agent_session_id)
}

fn text_response(text: &str, stop_reason: StopReason) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test]
async fn end_turn_without_any_tool_use_returns_a_plain_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply("Hello!".to_string()));
}

#[sqlx::test]
async fn a_tool_use_is_executed_and_its_result_fed_back(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done!", StopReason::EndTurn),
    ]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply("Done!".to_string()));
}

#[sqlx::test]
async fn complete_task_terminates_the_loop_with_a_completed_outcome(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        COMPLETE_TASK_TOOL_NAME,
        COMPLETE_TASK_TOOL_NAME,
        serde_json::json!({"status": "completed", "summary": "All done"}),
    )]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Completed { status: "completed".to_string(), summary: "All done".to_string() }
    );
}

#[sqlx::test]
async fn exceeding_the_turn_cap_returns_tool_loop_exceeded(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let responses: Vec<LlmResponse> =
        (0..11).map(|i| tool_use_response(&format!("t{i}"), "echo", serde_json::json!({}))).collect();
    let provider = FakeLlmProvider::sequence(responses);

    let result = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100).await;

    assert!(matches!(result, Err(TurnError::ToolLoopExceeded)));
}

#[sqlx::test]
async fn every_tool_call_is_logged_as_a_tool_called_event(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        tool_use_response(
            COMPLETE_TASK_TOOL_NAME,
            COMPLETE_TASK_TOOL_NAME,
            serde_json::json!({"status": "completed", "summary": "done"}),
        ),
    ]);

    run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND agent_session_id = $2 AND event_type = 'ToolCalled'",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 2);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_tools`
Expected: FAIL to compile — `nomi_orchestrator::turn::tools`/`subagent` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/subagent.rs
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use crate::llm::ToolDefinition;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String>;
}
```

```rust
// backend/src/turn/tools.rs
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};

use super::subagent::SubAgent;
use super::types::TurnError;

const MAX_TOOL_TURNS: u32 = 10;
pub const COMPLETE_TASK_TOOL_NAME: &str = "complete_task";

fn complete_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: COMPLETE_TASK_TOOL_NAME.to_string(),
        description: "Call this when you are done helping with this task, whether it succeeded or the user wants to stop.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["completed", "cancelled"]},
                "summary": {"type": "string"}
            },
            "required": ["status", "summary"]
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopOutcome {
    Reply(String),
    Completed { status: String, summary: String },
}

pub async fn run_tool_calling_loop(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    mut messages: Vec<LlmMessage>,
    max_tokens: u32,
) -> Result<LoopOutcome, TurnError> {
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());

    for _ in 0..MAX_TOOL_TURNS {
        let request = LlmRequest {
            system: Some(agent.system_prompt().to_string()),
            messages: messages.clone(),
            tools: tools.clone(),
            max_tokens,
        };

        let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;

        messages.push(LlmMessage { role: LlmRole::Assistant, content: response.content.clone() });

        if response.stop_reason != StopReason::ToolUse {
            let reply_text = response
                .content
                .into_iter()
                .find_map(|block| match block {
                    ContentBlock::Text { text } => Some(text),
                    _ => None,
                })
                .unwrap_or_default();
            return Ok(LoopOutcome::Reply(reply_text));
        }

        let mut tool_results = Vec::new();
        for block in &response.content {
            if let ContentBlock::ToolUse { id, name, input } = block {
                let (result_text, is_error) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false)
                } else {
                    match agent.execute_tool(conn, user_id, name, input.clone()).await {
                        Ok(text) => (text, false),
                        Err(err) => (err, true),
                    }
                };

                log_tool_call(conn, session_id, agent_session_id, agent.agent_type(), name, input, &result_text, is_error).await;

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
            }
        }

        messages.push(LlmMessage { role: LlmRole::User, content: tool_results });
    }

    Err(TurnError::ToolLoopExceeded)
}

async fn log_tool_call(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_type: &str,
    tool_name: &str,
    input: &serde_json::Value,
    result: &str,
    is_error: bool,
) {
    let _ = sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'ToolCalled', $4)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(agent_type)
    .bind(serde_json::json!({"tool_name": tool_name, "input": input, "result": result, "is_error": is_error}))
    .execute(&mut **conn)
    .await;
}
```

```rust
// backend/src/turn/types.rs — full file replacement
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
    #[error("tool-calling loop exceeded its turn limit without completing")]
    ToolLoopExceeded,
}
```

```rust
// backend/src/turn/mod.rs — add these two lines among the existing pub mod declarations
pub mod subagent;
pub mod tools;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_tools`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/subagent.rs backend/src/turn/tools.rs backend/src/turn/types.rs backend/src/turn/mod.rs backend/tests/support/mod.rs backend/tests/turn_tools.rs
git commit -m "feat: add generic tool-calling loop and SubAgent trait"
```

---

### Task 2: Money agent

**Files:**
- Create: `backend/migrations/0009_mock_transactions.sql`
- Create: `backend/src/turn/money_agent.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod money_agent;`)
- Test: `backend/tests/turn_money_agent.rs`

**Interfaces:**
- Consumes: `SubAgent` trait (Task 1).
- Produces: `pub const MONEY_AGENT_TYPE: &str = "money"`, `pub struct MoneyAgent;` implementing `SubAgent`. Task 4 constructs `Box::new(money_agent::MoneyAgent)` and dispatches on `money_agent::MONEY_AGENT_TYPE`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_money_agent.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::money_agent::MoneyAgent;
use nomi_orchestrator::turn::subagent::SubAgent;

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn seed_transaction(
    pool: &PgPool,
    user_id: Uuid,
    occurred_at: chrono::DateTime<chrono::Utc>,
    amount_cents: i64,
    category: &str,
    description: &str,
) {
    sqlx::query(
        "INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(occurred_at)
    .bind(amount_cents)
    .bind(category)
    .bind(description)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn list_transactions_returns_recent_transactions_for_the_user(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let other_user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, other_user_id, now, 9999, "food", "someone else's lunch").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "list_transactions", serde_json::json!({"limit": 10}))
        .await
        .unwrap();

    assert!(result.contains("lunch"));
    assert!(!result.contains("someone else's lunch"));
}

#[sqlx::test]
async fn list_transactions_filters_by_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, user_id, now, 5000, "rent", "monthly rent").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "list_transactions", serde_json::json!({"limit": 10, "category": "food"}))
        .await
        .unwrap();

    assert!(result.contains("lunch"));
    assert!(!result.contains("monthly rent"));
}

#[sqlx::test]
async fn list_transactions_with_no_matches_says_so(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "list_transactions", serde_json::json!({"limit": 10}))
        .await
        .unwrap();
    assert_eq!(result, "No transactions found.");
}

#[sqlx::test]
async fn summarize_budget_groups_totals_by_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let now = chrono::Utc::now();
    seed_transaction(&pool, user_id, now, 1500, "food", "lunch").await;
    seed_transaction(&pool, user_id, now, 2500, "food", "dinner").await;
    seed_transaction(&pool, user_id, now, 5000, "rent", "monthly rent").await;

    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "summarize_budget", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.contains("food: 40.00"));
    assert!(result.contains("rent: 50.00"));
}

#[sqlx::test]
async fn summarize_budget_respects_since_filter(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let old = chrono::Utc::now() - chrono::Duration::days(60);
    let recent = chrono::Utc::now();
    seed_transaction(&pool, user_id, old, 1000, "food", "old lunch").await;
    seed_transaction(&pool, user_id, recent, 2000, "food", "recent lunch").await;

    let mut conn = pool.acquire().await.unwrap();
    let since = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "summarize_budget", serde_json::json!({"since": since}))
        .await
        .unwrap();

    assert!(result.contains("food: 20.00"));
    assert!(!result.contains("food: 30.00"));
}

#[sqlx::test]
async fn summarize_budget_with_an_invalid_since_date_returns_an_error(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent
        .execute_tool(&mut conn, user_id, "summarize_budget", serde_json::json!({"since": "not-a-date"}))
        .await;
    assert!(result.is_err());
}

#[sqlx::test]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let result = MoneyAgent.execute_tool(&mut conn, user_id, "transfer_money", serde_json::json!({})).await;
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_money_agent`
Expected: FAIL — migration table missing and/or `nomi_orchestrator::turn::money_agent` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```sql
-- backend/migrations/0009_mock_transactions.sql
CREATE TABLE mock_transactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    occurred_at  TIMESTAMPTZ NOT NULL,
    amount_cents BIGINT NOT NULL,
    category     TEXT NOT NULL,
    description  TEXT NOT NULL
);
```

```rust
// backend/src/turn/money_agent.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use crate::llm::ToolDefinition;

use super::subagent::SubAgent;

pub const MONEY_AGENT_TYPE: &str = "money";

const MONEY_SYSTEM_PROMPT: &str =
    "You are a financial assistant. You can list the user's recent transactions and summarize \
     their spending by category. You are strictly read-only and advisory: you cannot move money, \
     make payments, or modify any transaction. If asked to do anything beyond listing or \
     summarizing, explain that you can only advise, not act. When you have fully answered the \
     user's question (or they want to stop), call complete_task.";

pub struct MoneyAgent;

#[async_trait]
impl SubAgent for MoneyAgent {
    fn agent_type(&self) -> &'static str {
        MONEY_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        MONEY_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "list_transactions".to_string(),
                description: "List the user's most recent transactions, optionally filtered by category.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {"type": "integer", "description": "Max number of transactions to return"},
                        "category": {"type": "string", "description": "Optional category filter"}
                    },
                    "required": ["limit"]
                }),
            },
            ToolDefinition {
                name: "summarize_budget".to_string(),
                description: "Summarize the user's spending totals grouped by category, optionally since a given date.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "since": {"type": "string", "description": "ISO 8601 date; omit for all-time"}
                    }
                }),
            },
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
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
}

async fn list_transactions(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
    let category = input.get("category").and_then(|v| v.as_str());

    let rows: Vec<(DateTime<Utc>, i64, String, String)> = if let Some(category) = category {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description FROM mock_transactions \
             WHERE user_id = $1 AND category = $2 ORDER BY occurred_at DESC LIMIT $3",
        )
        .bind(user_id)
        .bind(category)
        .bind(limit)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT occurred_at, amount_cents, category, description FROM mock_transactions \
             WHERE user_id = $1 ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    if rows.is_empty() {
        return Ok("No transactions found.".to_string());
    }

    let lines: Vec<String> = rows
        .iter()
        .map(|(occurred_at, amount_cents, category, description)| {
            format!(
                "{} | {} | {:.2} | {}",
                occurred_at.format("%Y-%m-%d"),
                category,
                *amount_cents as f64 / 100.0,
                description
            )
        })
        .collect();

    Ok(lines.join("\n"))
}

async fn summarize_budget(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let since = input.get("since").and_then(|v| v.as_str());

    let rows: Vec<(String, i64)> = if let Some(since) = since {
        let since: DateTime<Utc> = since.parse().map_err(|_| "invalid 'since' date, expected ISO 8601".to_string())?;
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM mock_transactions \
             WHERE user_id = $1 AND occurred_at >= $2 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as(
            "SELECT category, SUM(amount_cents)::bigint FROM mock_transactions \
             WHERE user_id = $1 GROUP BY category ORDER BY category",
        )
        .bind(user_id)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?
    };

    if rows.is_empty() {
        return Ok("No transactions found.".to_string());
    }

    let lines: Vec<String> = rows
        .iter()
        .map(|(category, total_cents)| format!("{}: {:.2}", category, *total_cents as f64 / 100.0))
        .collect();

    Ok(lines.join("\n"))
}
```

```rust
// backend/src/turn/mod.rs — add this line among the existing pub mod declarations
pub mod money_agent;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_money_agent`
Expected: PASS (7 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/migrations/0009_mock_transactions.sql backend/src/turn/money_agent.rs backend/src/turn/mod.rs backend/tests/turn_money_agent.rs
git commit -m "feat: add MoneyAgent (read-only list_transactions/summarize_budget)"
```

---

### Task 3: Intent classification

**Files:**
- Modify: `backend/src/turn/routing.rs` (add `Intent`, `classify_intent`)
- Test: `backend/tests/turn_intent_classification.rs`

**Interfaces:**
- Produces: `#[derive(Debug, Clone, Copy, PartialEq)] pub enum Intent { Chitchat, Money }`, `pub async fn classify_intent(provider: &dyn LlmProvider, text: &str) -> Intent`. Task 4 calls this when no agent is currently active.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_intent_classification.rs
mod support;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::routing::{classify_intent, Intent};

use support::FakeLlmProvider;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[tokio::test]
async fn classifies_money_intent() {
    let provider = FakeLlmProvider::success(text_response("money"));
    let intent = classify_intent(&provider, "how much did I spend on food this month?").await;
    assert_eq!(intent, Intent::Money);
}

#[tokio::test]
async fn classifies_chitchat_intent() {
    let provider = FakeLlmProvider::success(text_response("chitchat"));
    let intent = classify_intent(&provider, "how's the weather today?").await;
    assert_eq!(intent, Intent::Chitchat);
}

#[tokio::test]
async fn is_case_and_whitespace_insensitive() {
    let provider = FakeLlmProvider::success(text_response("  Money  \n"));
    let intent = classify_intent(&provider, "budget please").await;
    assert_eq!(intent, Intent::Money);
}

#[tokio::test]
async fn unrecognized_text_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::success(text_response("something else entirely"));
    let intent = classify_intent(&provider, "anything").await;
    assert_eq!(intent, Intent::Chitchat);
}

#[tokio::test]
async fn provider_failure_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::failure("provider down");
    let intent = classify_intent(&provider, "anything").await;
    assert_eq!(intent, Intent::Chitchat);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_intent_classification`
Expected: FAIL to compile — `classify_intent`/`Intent` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/routing.rs — ADD these imports and this code (existing find_active_agent_session is untouched)
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

const INTENT_CLASSIFICATION_SYSTEM_PROMPT: &str =
    "Classify the user's message as exactly one of: chitchat, money. Reply with only that single word, nothing else.";
const INTENT_CLASSIFICATION_MAX_TOKENS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Chitchat,
    Money,
}

pub async fn classify_intent(provider: &dyn LlmProvider, text: &str) -> Intent {
    let request = LlmRequest {
        system: Some(INTENT_CLASSIFICATION_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }],
        tools: vec![],
        max_tokens: INTENT_CLASSIFICATION_MAX_TOKENS,
    };

    let response = match provider.complete(request).await {
        Ok(r) => r,
        Err(_) => return Intent::Chitchat,
    };

    let text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    match text.trim().to_lowercase().as_str() {
        "money" => Intent::Money,
        _ => Intent::Chitchat,
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_intent_classification`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/routing.rs backend/tests/turn_intent_classification.rs
git commit -m "feat: add LLM-based chitchat/money intent classification"
```

---

### Task 4: State machine wiring

**Files:**
- Modify: `backend/src/turn/routing.rs` (add `ActiveAgentSessionDetails`, `load_active_agent_session_details`, `is_stale`, `mark_expired`, `spawn_agent_session`, `complete_agent_session` — all additive; `find_active_agent_session` and `classify_intent`/`Intent` are untouched)
- Modify: `backend/src/turn/types.rs` (add `UnknownAgentType`)
- Modify: `backend/src/turn/mod.rs` (rewrite `handle_inbound_message`'s body — its public signature is unchanged; add `run_subagent_turn`, `fetch_recent_messages`)
- Test: `backend/tests/turn_subagent_state_machine.rs`

**Interfaces:**
- Consumes: `tools::run_tool_calling_loop`/`LoopOutcome` (Task 1), `money_agent::{MoneyAgent, MONEY_AGENT_TYPE}` (Task 2), `routing::{classify_intent, Intent}` (Task 3), `routing::find_active_agent_session` (already exists, untouched).
- Produces: `handle_inbound_message`'s existing public signature is unchanged — no existing caller or test needs updating for that reason. Internally, `handle_inbound_message` now spawns/continues/completes/expires money-agent sessions instead of always falling through to chitchat.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_subagent_state_machine.rs
mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::handle_inbound_message;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

async fn seed_speaker(pool: &PgPool, channel_user_id: &str) -> (Uuid, Uuid) {
    // Returns (user_id, channel_identity_id) — does not create a session or agent_sessions row.
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(user_id)
    .bind(channel_user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, identity_id)
}

#[sqlx::test]
async fn money_intent_with_no_active_agent_spawns_and_runs_the_money_agent(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("Here's your spending"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Here's your spending");

    let (agent_type, status): (String, String) =
        sqlx::query_as("SELECT agent_type, status FROM agent_sessions WHERE session_id = $1")
            .bind(outcome.session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent_type, "money");
    assert_eq!(status, "active");

    let spawned_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND event_type = 'AgentSpawned'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(spawned_count, 1);

    let tool_called_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tool_called_count, 1);
}

#[sqlx::test]
async fn an_active_agent_is_continued_without_reclassifying_intent(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id).bind(identity_id).execute(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![text_response("Sure, here's more info")]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "and rent?")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Sure, here's more info");

    let agent_session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(agent_session_count, 1); // no new row — the existing one was reused

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(requests.len(), 1); // classification was skipped entirely
    // "financial assistant" only ever appears in the money agent's own system prompt — never in
    // the classify_intent prompt — so this proves the single call was the money-agent turn, not
    // a repeated intent classification.
    assert!(requests[0].system.as_ref().unwrap().contains("financial assistant"));
}

#[sqlx::test]
async fn complete_task_marks_the_agent_session_completed(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active') RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        "complete_task",
        "complete_task",
        serde_json::json!({"status": "completed", "summary": "All set!"}),
    )]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "thanks, that's all")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "All set!");

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "completed");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentCompleted'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload, serde_json::json!({"summary": "All set!"}));
}

#[sqlx::test]
async fn complete_task_with_cancelled_status_marks_the_agent_session_cancelled(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active') RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        "complete_task",
        "complete_task",
        serde_json::json!({"status": "cancelled", "summary": "Nevermind, no problem!"}),
    )]);

    handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "actually nevermind")
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentCancelled'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 1);
}

#[sqlx::test]
async fn a_stale_active_agent_is_expired_and_the_message_falls_through_to_chitchat(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let stale_time = chrono::Utc::now() - chrono::Duration::hours(25);
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, last_activity_at) VALUES ($1, $2, 'money', 'active', $3) RETURNING id",
    )
    .bind(session_id).bind(identity_id).bind(stale_time).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![
        text_response("chitchat"),
        text_response("Hi there!"),
        text_response("NONE"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hello again")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi there!");

    let status: String = sqlx::query_scalar("SELECT status FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "expired");

    let expired_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'AgentExpired'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expired_count, 1);
}

#[sqlx::test]
async fn after_completion_a_new_money_intent_message_spawns_a_fresh_second_agent_session(pool: PgPool) {
    let (_user_id, identity_id) = seed_speaker(&pool, "tg-1").await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id).fetch_one(&pool).await.unwrap();
    let old_agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'money', 'completed', now()) RETURNING id",
    )
    .bind(session_id).bind(identity_id).fetch_one(&pool).await.unwrap();

    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![
        text_response("money"),
        tool_use_response("t1", "list_transactions", serde_json::json!({"limit": 10})),
        text_response("Here's your spending again"),
    ]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "what about now?")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Here's your spending again");

    let total_agent_sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total_agent_sessions, 2);

    let new_active_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM agent_sessions WHERE session_id = $1 AND status = 'active'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(new_active_id, old_agent_session_id);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_subagent_state_machine`
Expected: FAIL to compile — the new routing.rs functions and mod.rs's rewritten dispatch logic don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/routing.rs — ADD these imports and this code (existing find_active_agent_session, classify_intent, and Intent are untouched)
use chrono::{DateTime, Utc};

const AGENT_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveAgentSessionDetails {
    pub agent_type: String,
    pub last_activity_at: DateTime<Utc>,
}

pub async fn load_active_agent_session_details(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
) -> Result<ActiveAgentSessionDetails, TurnError> {
    let (agent_type, last_activity_at): (String, DateTime<Utc>) =
        sqlx::query_as("SELECT agent_type, last_activity_at FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&mut **conn)
            .await?;
    Ok(ActiveAgentSessionDetails { agent_type, last_activity_at })
}

pub fn is_stale(last_activity_at: DateTime<Utc>) -> bool {
    Utc::now() - last_activity_at > chrono::Duration::hours(AGENT_EXPIRY_HOURS)
}

pub async fn mark_expired(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = 'expired', ended_at = now() WHERE id = $1")
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentExpired')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(())
}

pub async fn spawn_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    agent_type: &str,
) -> Result<Uuid, TurnError> {
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(agent_type)
    .fetch_one(&mut **conn)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentSpawned')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(agent_session_id)
}

pub async fn complete_agent_session(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
    status: &str,
    summary: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = $1, ended_at = now() WHERE id = $2")
        .bind(status)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    let event_type = if status == "cancelled" { "AgentCancelled" } else { "AgentCompleted" };

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .bind(event_type)
        .bind(serde_json::json!({"summary": summary}))
        .execute(&mut **conn)
        .await?;

    Ok(())
}
```

```rust
// backend/src/turn/types.rs — full file replacement
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
    #[error("tool-calling loop exceeded its turn limit without completing")]
    ToolLoopExceeded,
    #[error("unknown sub-agent type: {0}")]
    UnknownAgentType(String),
}
```

```rust
// backend/src/turn/mod.rs — full file replacement
pub mod bootstrap;
pub mod chitchat;
pub mod lock;
pub mod memory;
pub mod money_agent;
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

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    let active = routing::find_active_agent_session(&mut conn, session_id, sender_channel_identity_id).await?;

    let active_agent = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(&mut conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(&mut conn, agent_session_id, session_id, &details.agent_type).await?;
                None
            } else {
                Some((agent_session_id, details.agent_type))
            }
        }
        None => None,
    };

    let result = match active_agent {
        Some((agent_session_id, agent_type)) => {
            run_subagent_turn(&mut conn, provider, session_id, agent_session_id, &agent_type, user_id, text).await
        }
        None => match routing::classify_intent(provider, text).await {
            routing::Intent::Money => {
                let agent_session_id = routing::spawn_agent_session(
                    &mut conn,
                    session_id,
                    sender_channel_identity_id,
                    money_agent::MONEY_AGENT_TYPE,
                )
                .await?;
                run_subagent_turn(
                    &mut conn,
                    provider,
                    session_id,
                    agent_session_id,
                    money_agent::MONEY_AGENT_TYPE,
                    user_id,
                    text,
                )
                .await
            }
            routing::Intent::Chitchat => {
                chitchat::run_chitchat_turn(&mut conn, provider, embedding_provider, session_id, user_id, text).await
            }
        },
    };

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

async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_type: &str,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let agent: Box<dyn subagent::SubAgent> = match agent_type {
        money_agent::MONEY_AGENT_TYPE => Box::new(money_agent::MoneyAgent),
        other => return Err(TurnError::UnknownAgentType(other.to_string())),
    };

    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = tools::run_tool_calling_loop(
        conn,
        provider,
        agent.as_ref(),
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

            routing::complete_agent_session(conn, agent_session_id, session_id, agent_type, &status, &summary).await?;

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

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_subagent_state_machine`
Expected: PASS (6 tests)

- [ ] **Step 5: Run the full test suite**

Run: `cd backend && cargo test`
Expected: PASS, no regressions — in particular, confirm the pre-existing `turn_handle_inbound_message.rs` (5 tests) and `turn_routing.rs` (4 tests) suites still pass completely unmodified, proving this task's changes were genuinely additive to those areas.

- [ ] **Step 6: Commit**

```bash
git add backend/src/turn/routing.rs backend/src/turn/types.rs backend/src/turn/mod.rs backend/tests/turn_subagent_state_machine.rs
git commit -m "feat: wire agent_sessions state machine (spawn/continue/complete/expire) into handle_inbound_message"
```
