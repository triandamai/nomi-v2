# Multi-Agent Supervisor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the default agent (chitchat) delegate a task to a specialist agent to run in the background, acknowledging immediately and continuing the conversation; when the delegated work finishes, a supervisor agent (same personality, distinct job) delivers the result asynchronously into the open chat. The supervisor is also directly reachable for cross-agent status queries. Chat UI gets a small activity indicator that expands into the existing `BottomSheet` component.

**Architecture:** `run_agent_turn` gains a `registry: &AgentRegistry` parameter so its tool-list construction can offer a generic `delegate_to_agent` tool (built from the live registry, not hand-maintained) to any agent whose `can_delegate() == true`. Calling that tool does a fast, non-blocking insert into a new `agent_delegations` table (mirroring the existing `turn_jobs` queue's `SKIP LOCKED`/`LISTEN`-`NOTIFY` pattern) and returns immediately — never runs the delegated agent's turn inline, never blocks the conversational session's advisory lock. A new background worker (structurally identical to `worker.rs`) claims delegations, runs the target agent's own `run_agent_turn` directly (no real `agent_sessions` row — that table's uniqueness is scoped to conversational turn-taking, which a background delegation explicitly isn't part of), phrases the result via a new `nomi-agent-supervisor` crate, and delivers it by inserting a `messages` row and publishing over the same MQTT topic the open chat's WebSocket already subscribes to (proven, already-decoupled transport — no new plumbing needed there).

**Tech Stack:** Rust (`nomi-agent-core`, `nomi-turn`, `nomi-server`, `nomi-realtime`, `nomi-agent-chitchat`, new `nomi-agent-supervisor` crates), sqlx/Postgres (`LISTEN`/`NOTIFY`, `SKIP LOCKED`), MQTT (`rumqttc`, already in use), SvelteKit/Svelte 5 (frontend).

**Spec:** docs/superpowers/specs/2026-08-29-multi-agent-supervisor-design.md

## Global Constraints

- Delegated execution never acquires the conversational session's advisory lock (`lock::acquire_session_lock` is never called by the delegation worker) and never creates a real `agent_sessions` row — both would interfere with the user's live, concurrent conversation. Delegated `run_agent_turn` calls pass `agent_session_id = session_id` (the same sentinel value already used for the default agent elsewhere in this codebase).
- `mqtt: None` for delegated `run_agent_turn` calls — no live token-by-token streaming of a delegated agent's own thinking in this pass; only the final result is delivered.
- Only `ChitchatAgent` gets `can_delegate() = true` in this pass. Only `SupervisorAgent` gets `is_delegation_target() = false`.
- Every new provider/worker file mirrors the shape of its existing sibling exactly (`turn_jobs`'s `queue.rs`/`worker.rs` for the new delegation queue/worker; `nomi-agent-money`'s `Cargo.toml`/trait-impl shape for the new `nomi-agent-supervisor` crate).
- No compile-time-checked `sqlx::query_as!` macro anywhere in this plan — this codebase consistently uses runtime `sqlx::query_as::<_, (tuple)>(...)` with manual struct mapping (see `nomi-server/src/routes/settings.rs`, `nomi-agent-core/src/memory.rs`, every other query site touched this session); stay consistent with that, not the macro form.
- Frontend: reuse the existing `BottomSheet`/`ListItem` components from `$lib/components/m3/` — no new components.

---

### Task 1: `SubAgent`/`AgentRegistry` delegation primitives

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/subagent.rs`
- Modify: `backend/crates/nomi-agent-core/src/registry.rs`
- Modify: `backend/crates/nomi-agent-chitchat/src/lib.rs`
- Create: `backend/crates/nomi-agent-core/tests/registry.rs`

**Interfaces:**
- Produces (used by Task 3): `SubAgent::can_delegate(&self) -> bool` (default `false`), `SubAgent::is_delegation_target(&self) -> bool` (default `true`), `AgentRegistry::delegatable_agent_types(&self, excluding: &str) -> Vec<&'static str>`.

- [ ] **Step 1: Add the two new `SubAgent` default methods**

In `backend/crates/nomi-agent-core/src/subagent.rs`, add inside the `trait SubAgent` block, after the existing `uses_personality` method:

```rust
    /// When true, run_agent_turn gives this agent an extra `delegate_to_agent` tool that hands
    /// a task to another registered specialist to run in the background.
    fn can_delegate(&self) -> bool {
        false
    }

    /// When false, this agent is never offered as a delegation target — used by the supervisor
    /// agent, which delivers/reports on delegated work rather than being delegated to itself.
    fn is_delegation_target(&self) -> bool {
        true
    }
```

- [ ] **Step 2: Add `AgentRegistry::delegatable_agent_types`**

In `backend/crates/nomi-agent-core/src/registry.rs`, add after the existing `classification_prompt` method:

```rust
    /// agent_type() of every non-default, delegation-eligible agent other than `excluding` —
    /// the valid target list for a delegate_to_agent tool call from that agent.
    pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<&'static str> {
        self.agents
            .iter()
            .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type() != excluding)
            .map(|a| a.agent_type())
            .collect()
    }
```

- [ ] **Step 3: `ChitchatAgent` opts into delegation**

In `backend/crates/nomi-agent-chitchat/src/lib.rs`, add inside the `impl SubAgent for ChitchatAgent` block, after the existing `uses_personality` method:

```rust
    fn can_delegate(&self) -> bool {
        true
    }
```

- [ ] **Step 4: Write tests for `delegatable_agent_types`**

Create `backend/crates/nomi-agent-core/tests/registry.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, SubAgent};
use nomi_llm::ToolDefinition;

struct DefaultAgent;

#[async_trait]
impl SubAgent for DefaultAgent {
    fn agent_type(&self) -> &'static str {
        "default"
    }
    fn system_prompt(&self) -> &'static str {
        "default"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "default"
    }
    fn intent_description(&self) -> &'static str {
        "default"
    }
    fn is_default(&self) -> bool {
        true
    }
    fn can_delegate(&self) -> bool {
        true
    }
}

struct SpecialistAgent;

#[async_trait]
impl SubAgent for SpecialistAgent {
    fn agent_type(&self) -> &'static str {
        "specialist"
    }
    fn system_prompt(&self) -> &'static str {
        "specialist"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "specialist"
    }
    fn intent_description(&self) -> &'static str {
        "specialist"
    }
}

struct NonTargetableAgent;

#[async_trait]
impl SubAgent for NonTargetableAgent {
    fn agent_type(&self) -> &'static str {
        "non-targetable"
    }
    fn system_prompt(&self) -> &'static str {
        "non-targetable"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "non-targetable"
    }
    fn intent_description(&self) -> &'static str {
        "non-targetable"
    }
    fn is_delegation_target(&self) -> bool {
        false
    }
}

#[test]
fn delegatable_agent_types_excludes_default_non_targetable_and_the_requester() {
    let registry = AgentRegistry::new(vec![
        Box::new(DefaultAgent),
        Box::new(SpecialistAgent),
        Box::new(NonTargetableAgent),
    ]);

    let targets = registry.delegatable_agent_types("default");

    assert_eq!(targets, vec!["specialist"]);
}

#[test]
fn delegatable_agent_types_excludes_the_named_requester_even_if_delegation_eligible() {
    let registry = AgentRegistry::new(vec![Box::new(DefaultAgent), Box::new(SpecialistAgent)]);

    let targets = registry.delegatable_agent_types("specialist");

    assert!(targets.is_empty());
}
```

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-agent-core -p nomi-agent-chitchat 2>&1 | tail -60
```

Expected: clean build, all tests pass (including the 2 new ones).

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-agent-core/src/subagent.rs backend/crates/nomi-agent-core/src/registry.rs \
        backend/crates/nomi-agent-chitchat/src/lib.rs backend/crates/nomi-agent-core/tests/registry.rs
git commit -m "feat: add delegation primitives to SubAgent and AgentRegistry"
```

---

### Task 2: `StreamEnvelope::AgentDelegationUpdated`

**Files:**
- Modify: `backend/crates/nomi-realtime/src/lib.rs`

**Interfaces:**
- Produces (used by Tasks 4 and 7): `StreamEnvelope::AgentDelegationUpdated { delegation_id: Uuid }`.

- [ ] **Step 1: Add the variant**

Replace `backend/crates/nomi-realtime/src/lib.rs` in full:

```rust
pub mod mqtt;

pub use mqtt::{MqttError, MqttPublisher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_llm::StreamEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
    AgentDelegationUpdated { delegation_id: Uuid },
}
```

- [ ] **Step 2: Verify**

```bash
cd backend && cargo build -p nomi-realtime 2>&1 | tail -20
```

Expected: no errors. (This crate has no existing test file for `StreamEnvelope` itself — its serialization is exercised indirectly wherever it's consumed; no new test needed for a data-only enum variant addition.)

- [ ] **Step 3: Commit**

```bash
git add backend/crates/nomi-realtime/src/lib.rs
git commit -m "feat: add AgentDelegationUpdated to StreamEnvelope"
```

---

### Task 3: `run_agent_turn` takes the registry; `delegate_to_agent` tool is offered

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/engine.rs`
- Modify: `backend/crates/nomi-turn/src/lib.rs`
- Modify: `backend/crates/nomi-agent-core/tests/engine.rs`
- Modify: `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs`

**Interfaces:**
- Consumes: `SubAgent::can_delegate()`, `AgentRegistry::delegatable_agent_types()` (Task 1).
- Produces (used by Task 4): `run_agent_turn`'s new signature (an `registry: &AgentRegistry` parameter inserted after `embedding_provider`, before `agent`); `pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";` in `engine.rs`, re-exported from `nomi-agent-core`'s `lib.rs` alongside the existing `COMPLETE_TASK_TOOL_NAME` re-export.

This is the highest-risk task in the plan: it changes an existing, heavily-tested function's signature. Every one of the 9 existing call sites below must be updated in this same task — a partial update leaves the workspace non-compiling, which is expected and fine *within* this task's own work, but the task's own Verify step (Step 7) must show a fully clean build across the whole workspace, not just the crate you're editing.

- [ ] **Step 1: Add `DELEGATE_TOOL_NAME` and `delegate_tool_definition`, and thread `registry` through `run_agent_turn`**

In `backend/crates/nomi-agent-core/src/engine.rs`:

Add near the top, alongside the existing `MAX_TOOL_TURNS`/`COMPLETE_TASK_TOOL_NAME` consts:

```rust
pub const DELEGATE_TOOL_NAME: &str = "delegate_to_agent";
```

Add a new function, near `complete_task_tool_definition`:

```rust
fn delegate_tool_definition(targets: &[&str]) -> ToolDefinition {
    ToolDefinition {
        name: DELEGATE_TOOL_NAME.to_string(),
        description: format!(
            "Hand off a task to a specialist agent to work on in the background, and immediately \
             tell the user you'll follow up — do NOT wait for the result before replying. Use this \
             only when the request genuinely needs a specialist; answer anything else yourself. \
             Available specialists: {}.",
            targets.join(", "),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "target_agent": {"type": "string", "enum": targets, "description": "Which specialist to delegate to"},
                "task": {"type": "string", "description": "What to ask the specialist to do, in your own words"}
            },
            "required": ["target_agent", "task"]
        }),
    }
}
```

Add `use nomi_agent_core::AgentRegistry` — actually this is already the `nomi-agent-core` crate itself, so add `use crate::AgentRegistry;` (or `use crate::registry::AgentRegistry;` — match whichever import style `engine.rs` already uses for its other `crate::` imports; it currently has `use crate::error::TurnError;`, `use crate::memory;`, `use crate::subagent::SubAgent;` — follow that same `use crate::<module>::<Type>;` style: `use crate::registry::AgentRegistry;`).

Change `run_agent_turn`'s signature (currently lines 42-53) — insert `registry: &AgentRegistry` after `embedding_provider: &dyn EmbeddingProvider,` and before `agent: &dyn SubAgent,`:

```rust
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
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

Change the tool-list construction (currently lines 54-55, `let mut tools = agent.tools(); tools.push(complete_task_tool_definition());`) to:

```rust
    let mut tools = agent.tools();
    tools.push(complete_task_tool_definition());
    if agent.can_delegate() {
        let targets = registry.delegatable_agent_types(agent.agent_type());
        if !targets.is_empty() {
            tools.push(delegate_tool_definition(&targets));
        }
    }
```

(The `!targets.is_empty()` guard avoids offering a tool whose schema has an empty `enum` — meaningless to any real provider — when an agent that opts into delegation happens to be the only delegation-eligible agent registered, e.g. in a single-agent test registry.)

Do **not** add the `DELEGATE_TOOL_NAME` special-case branch to the tool-execution loop yet — that lands in Task 4, once `create_delegation` (which it calls) exists. For this task, the tool is offered to the LLM but never actually invoked by any test; that's fine and expected.

- [ ] **Step 2: Re-export `DELEGATE_TOOL_NAME`**

In `backend/crates/nomi-agent-core/src/lib.rs`, change:

```rust
pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME};
```

to:

```rust
pub use engine::{run_agent_turn, LoopOutcome, COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME};
```

- [ ] **Step 3: Thread `registry` through `nomi-turn`'s `run_subagent_turn`**

In `backend/crates/nomi-turn/src/lib.rs`:

Change `run_subagent_turn`'s signature (currently lines 189-199) to add `registry: &AgentRegistry` — insert it right after `embedding_provider: &dyn EmbeddingProvider,` and before `agent: &dyn nomi_agent_core::SubAgent,`:

```rust
#[allow(clippy::too_many_arguments)]
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
) -> Result<String, TurnError> {
```

Inside its body, change the call to `nomi_agent_core::run_agent_turn` (currently lines 202-214) to pass `registry` in the same new position:

```rust
    let outcome = nomi_agent_core::run_agent_turn(
        conn,
        mqtt,
        provider,
        embedding_provider,
        registry,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        SUBAGENT_MAX_TOKENS,
    )
    .await?;
```

Update `run_subagent_turn`'s two call sites, both inside `run_locked_turn` (currently lines 169 and 179/183) — `registry` is already a parameter of `run_locked_turn` itself, so this is purely adding one argument at each call:

```rust
        RoutingOutcome::Continue { agent, agent_session_id } => {
            run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => {
            let agent = routing::classify_intent(provider, registry, text).await;

            if agent.agent_type() == registry.default_agent().agent_type() {
                run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, session_id, user_id).await
            } else {
                let agent_session_id =
                    routing::spawn_agent_session(conn, session_id, sender_channel_identity_id, agent.agent_type()).await?;
                run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id).await
            }
        }
```

- [ ] **Step 4: Update all 7 call sites in `nomi-agent-core/tests/engine.rs`**

This file's `TestAgent` and `PersonalityAwareTestAgent` structs each need `fn is_default(&self) -> bool { true }` added to their `impl SubAgent` blocks (each is used alone in its own single-agent registry in this file — being "the default" is a registry-construction requirement, not a semantic claim about the test).

Add near the top of the file: `use nomi_agent_core::AgentRegistry;` (alongside the existing `use nomi_agent_core::{run_agent_turn, LoopOutcome, SubAgent, COMPLETE_TASK_TOOL_NAME};`).

In each of these 7 test functions, immediately before the `run_agent_turn(...)` call, insert a registry construction, then add `&registry` as a new argument to the call, positioned after `&embedding_provider,` and before the agent reference:

- `end_turn_without_any_tool_use_returns_a_plain_reply` (line ~147): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`
- `a_tool_use_is_executed_and_its_result_fed_back` (line ~180): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`
- `complete_task_terminates_the_loop_with_a_completed_outcome` (line ~214): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`
- `exceeding_the_turn_cap_returns_tool_loop_exceeded` (line ~246): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`
- `every_tool_call_is_logged_as_a_tool_called_event` (line ~279): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`
- `a_stored_personality_is_folded_into_the_system_prompt_when_uses_personality_is_true` (line ~320): `let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);`
- `a_stored_personality_is_not_folded_in_for_an_agent_that_does_not_opt_in` (line ~355): `let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);`

Example of the resulting call shape (for the first test function; apply the same pattern — insert the `let registry = ...;` line, then add `&registry,` to the call — at each of the other 6 sites, using the correct agent type per the list above):

```rust
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();
```

Neither `TestAgent` nor `PersonalityAwareTestAgent` overrides `can_delegate()` (both stay at the default `false`), so none of these 7 tests will see a `delegate_to_agent` tool in their requests — no assertion in any of them needs to change beyond the call-site signature update.

- [ ] **Step 5: Update the 1 call site in `nomi-agent-chitchat/tests/chitchat_agent.rs`**

Add `use nomi_agent_core::AgentRegistry;` to the imports. Immediately before the `run_agent_turn(...)` call (around line 58), insert:

```rust
    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
```

(`ChitchatAgent` already has `is_default() == true`, no trait changes needed here.) Add `&registry,` to the call, in the same new position (after `&embedder,`, before `&ChitchatAgent,`). Because this registry contains only `ChitchatAgent` itself, `delegatable_agent_types("chitchat")` returns an empty list, so the `!targets.is_empty()` guard from Step 1 means no `delegate_to_agent` tool is added here either — this existing test's assertions are unaffected.

- [ ] **Step 6: Self-review the full call-site list**

Re-run the grep that scoped this task and confirm every hit was addressed:

```bash
grep -rn "run_agent_turn(" backend/crates --include="*.rs"
```

Expected: every call site now has 11 positional arguments (was 10), with the new one being a `&registry` (or `registry`) reference in the 5th position.

- [ ] **Step 7: Verify**

```bash
cd backend && cargo build --workspace 2>&1 | tail -60
cd backend && cargo test --workspace 2>&1 | tail -100
```

Expected: clean build, and every existing test across the whole workspace still passes (this task changes no runtime behavior for any agent that doesn't have `can_delegate() == true` and more than one delegation-eligible sibling registered — which, after this task, is none, since the `DELEGATE_TOOL_NAME` branch isn't wired into the execution loop until Task 4).

- [ ] **Step 8: Commit**

```bash
git add backend/crates/nomi-agent-core/src/engine.rs backend/crates/nomi-agent-core/src/lib.rs \
        backend/crates/nomi-turn/src/lib.rs backend/crates/nomi-agent-core/tests/engine.rs \
        backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs
git commit -m "feat: thread AgentRegistry through run_agent_turn and offer delegate_to_agent"
```

---

### Task 4: `agent_delegations` table, `create_delegation`, and wiring the tool into the loop

**Files:**
- Create: `backend/migrations/0016_agent_delegations.sql`
- Create: `backend/crates/nomi-agent-core/src/delegation.rs`
- Modify: `backend/crates/nomi-agent-core/src/lib.rs`
- Modify: `backend/crates/nomi-agent-core/src/engine.rs`
- Create: `backend/crates/nomi-agent-core/tests/delegation.rs`

**Interfaces:**
- Consumes: `DELEGATE_TOOL_NAME` (Task 3), `StreamEnvelope::AgentDelegationUpdated` (Task 2).
- Produces (used by Tasks 5, 6, 7): the `agent_delegations` table; `nomi_agent_core::delegation::create_delegation(...)`.

- [ ] **Step 1: Add the migration**

Create `backend/migrations/0016_agent_delegations.sql`:

```sql
CREATE TABLE agent_delegations (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id             UUID NOT NULL REFERENCES sessions(id),
    user_id                UUID NOT NULL REFERENCES users(id),
    requesting_agent_type  TEXT NOT NULL,
    target_agent_type      TEXT NOT NULL,
    task                   TEXT NOT NULL,
    status                 TEXT NOT NULL DEFAULT 'pending',
    result                 TEXT,
    error                  TEXT,
    claimed_at             TIMESTAMPTZ,
    completed_at           TIMESTAMPTZ,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX agent_delegations_pending_idx ON agent_delegations (created_at) WHERE status = 'pending';
CREATE INDEX agent_delegations_session_idx ON agent_delegations (session_id, created_at DESC);
```

- [ ] **Step 2: `create_delegation`**

Create `backend/crates/nomi-agent-core/src/delegation.rs`:

```rust
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_realtime::{MqttPublisher, StreamEnvelope};

pub async fn create_delegation(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    requesting_agent_type: &str,
    target_agent_type: &str,
    task: &str,
    user_id: Uuid,
) -> Result<String, String> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(requesting_agent_type)
    .bind(target_agent_type)
    .bind(task)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| format!("failed to create delegation: {e}"))?;

    let _ = sqlx::query("SELECT pg_notify('agent_delegations_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **conn)
        .await;

    if let Some((publisher, _)) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: id }).await;
    }

    Ok(format!(
        "Delegated to {target_agent_type}. Tell the user you'll follow up once it's done — do not wait for the result now."
    ))
}
```

- [ ] **Step 3: Wire the module and the tool-execution branch**

In `backend/crates/nomi-agent-core/src/lib.rs`, add `pub mod delegation;` alongside the existing `pub mod` lines (e.g. after `pub mod engine;`).

In `backend/crates/nomi-agent-core/src/engine.rs`, inside the tool-execution loop's `if/else` chain (currently: `if name.as_str() == COMPLETE_TASK_TOOL_NAME { ... } else { match agent.execute_tool(...) ... }`), add a new branch between them:

```rust
                let (result_text, is_error) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
                    (input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(), false)
                } else if name.as_str() == DELEGATE_TOOL_NAME {
                    let target_agent = input.get("target_agent").and_then(|v| v.as_str()).unwrap_or_default();
                    let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default();
                    match crate::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type(), target_agent, task, user_id).await {
                        Ok(ack) => (ack, false),
                        Err(err) => (err, true),
                    }
                } else {
                    match agent.execute_tool(conn, session_id, agent_session_id, user_id, name, input.clone()).await {
                        Ok(text) => (text, false),
                        Err(err) => (err, true),
                    }
                };
```

- [ ] **Step 4: Write the delegation-creation test**

Create `backend/crates/nomi-agent-core/tests/delegation.rs`. This needs a delegating agent (`can_delegate() == true`) and a delegation target registered together, run through the full `run_agent_turn`, with the fake LLM actually calling `delegate_to_agent`, then asserts a real `agent_delegations` row was created and the tool result fed back into the loop was a non-error acknowledgment (not an error string).

```rust
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_agent_core::{run_agent_turn, AgentRegistry, LoopOutcome, SubAgent};

use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

struct DelegatingAgent;

#[async_trait]
impl SubAgent for DelegatingAgent {
    fn agent_type(&self) -> &'static str {
        "delegator"
    }
    fn system_prompt(&self) -> &'static str {
        "test"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "delegator"
    }
    fn intent_description(&self) -> &'static str {
        "test"
    }
    fn is_default(&self) -> bool {
        true
    }
    fn can_delegate(&self) -> bool {
        true
    }
}

struct TargetAgent;

#[async_trait]
impl SubAgent for TargetAgent {
    fn agent_type(&self) -> &'static str {
        "target"
    }
    fn system_prompt(&self) -> &'static str {
        "test"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "target"
    }
    fn intent_description(&self) -> &'static str {
        "test"
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

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn delegate_tool_call_response() -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse {
            id: "t1".to_string(),
            name: "delegate_to_agent".to_string(),
            input: serde_json::json!({"target_agent": "target", "task": "look into it"}),
            thought_signature: None,
        }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn calling_delegate_to_agent_creates_a_pending_delegation_row(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let registry = AgentRegistry::new(vec![Box::new(DelegatingAgent), Box::new(TargetAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        delegate_tool_call_response(),
        text_response("I'll check and get back to you!"),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &DelegatingAgent,
        session_id,
        session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply {
            text: "I'll check and get back to you!".to_string(),
            memory_ids_used: vec![],
            input_tokens: 1,
            output_tokens: 1,
        }
    );

    let (status, target_agent_type, task): (String, String, String) = sqlx::query_as(
        "SELECT status, target_agent_type, task FROM agent_delegations WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");
    assert_eq!(target_agent_type, "target");
    assert_eq!(task, "look into it");

    // The tool result fed back into the second LLM call must be a real acknowledgment, not an
    // error string — proving create_delegation's Ok path (not its Err path) was taken.
    let requests = provider.received_requests.lock().unwrap();
    let second_request = &requests[1];
    let tool_result_text = second_request
        .messages
        .iter()
        .flat_map(|m| &m.content)
        .find_map(|b| match b {
            ContentBlock::ToolResult { content, is_error, .. } => {
                assert!(!is_error);
                Some(content.clone())
            }
            _ => None,
        })
        .expect("expected a tool result in the second request");
    assert!(tool_result_text.contains("Delegated to target"));
}
```

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-agent-core 2>&1 | tail -80
```

Expected: clean build, all tests pass including the new one.

- [ ] **Step 6: Commit**

```bash
git add backend/migrations/0016_agent_delegations.sql backend/crates/nomi-agent-core/src/delegation.rs \
        backend/crates/nomi-agent-core/src/lib.rs backend/crates/nomi-agent-core/src/engine.rs \
        backend/crates/nomi-agent-core/tests/delegation.rs
git commit -m "feat: add agent_delegations table and wire delegate_to_agent into the tool loop"
```

---

### Task 5: `GET /api/sessions/:id/agent-activity`

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/sessions.rs`
- Modify: `backend/crates/nomi-server/src/app.rs`
- Create: `backend/crates/nomi-server/tests/agent_activity_routes.rs`

**Interfaces:**
- Consumes: `agent_delegations` table (Task 4).
- Produces (used by Task 8): `GET /api/sessions/:id/agent-activity` → `Vec<AgentActivityItem>` JSON.

- [ ] **Step 1: Add the handler**

In `backend/crates/nomi-server/src/routes/sessions.rs`, add near the other session-scoped handlers (e.g. after `session_stream`). Check the file's existing imports first — it should already import `Json`, `StatusCode`, `AuthClaims`, `Path`, `State`, `AppState`, and `authorize_session_access` for reuse here; add `chrono::{DateTime, Utc}` and `serde::Serialize` if not already imported.

```rust
#[derive(Serialize)]
pub struct AgentActivityItem {
    pub id: Uuid,
    pub target_agent_type: String,
    pub task: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn list_agent_activity(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Vec<AgentActivityItem>>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let rows: Vec<(Uuid, String, String, String, Option<String>, Option<String>, DateTime<Utc>, Option<DateTime<Utc>>)> =
        sqlx::query_as(
            "SELECT id, target_agent_type, task, status, result, error, created_at, completed_at \
             FROM agent_delegations WHERE session_id = $1 ORDER BY created_at DESC LIMIT 20",
        )
        .bind(session_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list agent activity");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agent activity")
        })?;

    let items = rows
        .into_iter()
        .map(|(id, target_agent_type, task, status, result, error, created_at, completed_at)| AgentActivityItem {
            id, target_agent_type, task, status, result, error, created_at, completed_at,
        })
        .collect();

    Ok(Json(items))
}
```

- [ ] **Step 2: Register the route**

In `backend/crates/nomi-server/src/app.rs`, add alongside the existing `.route("/api/sessions/:id/ws", get(sessions_routes::session_stream))`:

```rust
        .route("/api/sessions/:id/agent-activity", get(sessions_routes::list_agent_activity))
```

- [ ] **Step 3: Write the route test**

Create `backend/crates/nomi-server/tests/agent_activity_routes.rs`. Read `backend/crates/nomi-server/tests/settings_routes.rs` first for the exact `test_state`/`json_request`/`register_via_api`/`login_via_api` helper shapes already established in this codebase's route tests, and reuse that same pattern here (do not invent a different test-harness style).

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use nomi_test_support::TEST_SETTINGS_KEY;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
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

async fn org_id_for(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT m.org_id FROM memberships m \
         JOIN web_credentials w ON w.user_id = m.user_id \
         WHERE w.email = $1 AND m.status = 'active'",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn user_id_for(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_empty_when_no_delegations_exist_for_the_session(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-empty@example.com").await;
    let org_id = org_id_for(&pool, "activity-empty@example.com").await;

    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, body) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn lists_delegations_for_a_session_the_caller_owns(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-owner@example.com").await;
    let org_id = org_id_for(&pool, "activity-owner@example.com").await;
    let user_id = user_id_for(&pool, "activity-owner@example.com").await;

    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c2') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result) \
         VALUES ($1, $2, 'chitchat', 'money', 'check spending', 'completed', 'spent $42 on groceries')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["target_agent_type"], "money");
    assert_eq!(items[0]["status"], "completed");
    assert_eq!(items[0]["result"], "spent $42 on groceries");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_belonging_to_a_different_org_is_not_accessible(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-outsider@example.com").await;

    let other_org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Someone Else') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c3') RETURNING id",
    )
    .bind(other_org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, _) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
```

- [ ] **Step 4: Verify**

```bash
cd backend && cargo test -p nomi-server agent_activity 2>&1 | tail -60
```

Expected: clean build, all 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-server/src/routes/sessions.rs backend/crates/nomi-server/src/app.rs \
        backend/crates/nomi-server/tests/agent_activity_routes.rs
git commit -m "feat: add GET /api/sessions/:id/agent-activity"
```

---

### Task 6: `nomi-agent-supervisor` crate

**Files:**
- Create: `backend/crates/nomi-agent-supervisor/Cargo.toml`
- Create: `backend/crates/nomi-agent-supervisor/src/lib.rs`
- Modify: `backend/crates/nomi-server/src/lib.rs`
- Modify: `backend/crates/nomi-server/Cargo.toml`
- Modify: `backend/Cargo.toml` (workspace members list, if it enumerates crates explicitly — check first; skip this file if the workspace uses a glob pattern like `crates/*`)
- Create: `backend/crates/nomi-agent-supervisor/tests/supervisor_agent.rs`

**Interfaces:**
- Consumes: `agent_delegations` table (Task 4), `nomi_agent_core::personality::get_current_personality` (existing, already `pub`).
- Produces (used by Task 7): `nomi_agent_supervisor::SupervisorAgent`, `nomi_agent_supervisor::phrase_delegation_result(...)`.

- [ ] **Step 1: Create the crate manifest**

Read `backend/crates/nomi-agent-money/Cargo.toml` first to mirror its exact dependency versions (do not guess version numbers — copy them). Create `backend/crates/nomi-agent-supervisor/Cargo.toml`:

```toml
[package]
name = "nomi-agent-supervisor"
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

(Match `nomi-agent-money`'s actual dependency list exactly — the `chrono` dependency shown there is specific to money's own needs and may not be needed here; include only what this crate's code in Step 2 actually uses, following the same version pins.)

- [ ] **Step 2: `SupervisorAgent` and `phrase_delegation_result`**

Create `backend/crates/nomi-agent-supervisor/src/lib.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const SUPERVISOR_AGENT_TYPE: &str = "supervisor";

const SUPERVISOR_SYSTEM_PROMPT: &str =
    "You are the coordinator among a small team of specialist agents. When asked what the team \
     is doing, or for a status report, use list_recent_agent_activity and summarize it plainly — \
     what was asked, of whom, and the outcome if it finished. You never do the specialist work \
     yourself; you only report on it.";

pub struct SupervisorAgent;

#[async_trait]
impl SubAgent for SupervisorAgent {
    fn agent_type(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        SUPERVISOR_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "list_recent_agent_activity".to_string(),
            description: "List recent and currently active background delegations for this chat session.".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<String, String> {
        match name {
            "list_recent_agent_activity" => list_recent_agent_activity(conn, session_id).await,
            other => Err(format!("supervisor has no tool named {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "The user is asking what the other agents are doing, wants a status update, or explicitly asks for a report across multiple specialists"
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn is_delegation_target(&self) -> bool {
        false
    }
}

async fn list_recent_agent_activity(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<String, String> {
    let rows: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT target_agent_type, task, status, result, error FROM agent_delegations \
         WHERE session_id = $1 ORDER BY created_at DESC LIMIT 10",
    )
    .bind(session_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No background delegations for this session yet.".to_string());
    }

    let mut out = String::new();
    for (target, task, status, result, error) in rows {
        out.push_str(&format!("- {target} ({status}): asked to \"{task}\""));
        if let Some(r) = result {
            out.push_str(&format!(" — result: {r}"));
        }
        if let Some(e) = error {
            out.push_str(&format!(" — error: {e}"));
        }
        out.push('\n');
    }
    Ok(out)
}

/// One-shot, tool-free completion that phrases a completed delegation's raw result in the
/// supervisor's voice, folding in the user's personality the same way run_agent_turn does for
/// any agent with uses_personality() == true (duplicated here, not shared, because this path
/// intentionally doesn't go through run_agent_turn's full tool loop — this is a deliberate,
/// documented design decision, not an oversight; see the design spec).
pub async fn phrase_delegation_result(
    provider: &dyn nomi_llm::LlmProvider,
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    target_agent_type: &str,
    task: &str,
    raw_result: &str,
) -> Result<String, String> {
    let mut system = format!(
        "{SUPERVISOR_SYSTEM_PROMPT}\n\nDeliver this result from the {target_agent_type} agent to the user, in your \
         own voice, as if reporting back after they asked you to look into something. Be natural and brief.\n\n\
         What was asked: {task}\nRaw result: {raw_result}"
    );
    if let Some(p) = nomi_agent_core::personality::get_current_personality(conn, user_id).await {
        system = format!("{system}\n\nAdopt this personality in your reply: {p}");
    }

    let request = nomi_llm::LlmRequest {
        system: Some(system),
        messages: vec![nomi_llm::LlmMessage {
            role: nomi_llm::LlmRole::User,
            content: vec![nomi_llm::ContentBlock::Text { text: "Report back.".to_string() }],
        }],
        tools: vec![],
        max_tokens: 256,
    };
    let response = nomi_llm::complete(provider, request).await.map_err(|e| e.to_string())?;
    Ok(response
        .content
        .into_iter()
        .find_map(|block| match block {
            nomi_llm::ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or(raw_result.to_string()))
}
```

Note the `messages` field is **not** empty (unlike the spec's original sketch) — it carries a single placeholder user message (`"Report back."`). Every other call site of `nomi_llm::LlmRequest`/`complete`/`complete_stream` in this codebase (`classify_intent`, memory extraction) always supplies at least one message; do not assume an empty `messages: vec![]` works against every provider implementation without verifying — using a placeholder message here avoids the question entirely and costs nothing.

- [ ] **Step 3: Write tests**

Create `backend/crates/nomi-agent-supervisor/tests/supervisor_agent.rs`:

```rust
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_supervisor::SupervisorAgent;

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

async fn seed_delegation(pool: &PgPool, session_id: Uuid, user_id: Uuid, target: &str, status: &str, result: Option<&str>) {
    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result) \
         VALUES ($1, $2, 'chitchat', $3, 'do a thing', $4, $5)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(target)
    .bind(status)
    .bind(result)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_recent_agent_activity_reports_no_delegations_when_none_exist(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "list_recent_agent_activity", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result, "No background delegations for this session yet.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_recent_agent_activity_summarizes_completed_and_pending_delegations(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    seed_delegation(&pool, session_id, user_id, "money", "completed", Some("spent $42 on groceries")).await;
    seed_delegation(&pool, session_id, user_id, "personality", "pending", None).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "list_recent_agent_activity", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.contains("money"));
    assert!(result.contains("completed"));
    assert!(result.contains("spent $42 on groceries"));
    assert!(result.contains("personality"));
    assert!(result.contains("pending"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_tool_name_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "not_a_real_tool", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
```

- [ ] **Step 4: Wire into `build_agent_registry` and `Cargo.toml`s**

In `backend/crates/nomi-server/Cargo.toml`, add `nomi-agent-supervisor = { path = "../nomi-agent-supervisor" }` alongside the existing `nomi-agent-money`/`nomi-agent-chitchat`/`nomi-agent-personality` lines.

Check `backend/Cargo.toml`'s `[workspace] members` — if it lists each crate path explicitly (rather than a glob like `crates/*`), add `"crates/nomi-agent-supervisor"` to that list.

In `backend/crates/nomi-server/src/lib.rs`, change `build_agent_registry`:

```rust
pub fn build_agent_registry() -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_personality::PersonalityAgent),
        Box::new(nomi_agent_supervisor::SupervisorAgent),
    ])
}
```

- [ ] **Step 5: Verify**

```bash
cd backend && cargo test -p nomi-agent-supervisor -p nomi-server 2>&1 | tail -100
```

Expected: clean build across the workspace (the new crate compiles, `nomi-server` picks up the new registry entry cleanly), all tests pass including the 3 new ones. `AgentRegistry::new` must not panic — confirm exactly one agent (`ChitchatAgent`) still returns `true` from `is_default()`; adding `SupervisorAgent` must not change that.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-agent-supervisor backend/crates/nomi-server/src/lib.rs \
        backend/crates/nomi-server/Cargo.toml backend/Cargo.toml
git commit -m "feat: add the supervisor agent crate and register it"
```

(Omit `backend/Cargo.toml` from the `git add` if Step 4 determined the workspace uses a glob member pattern and no edit was needed there.)

---

### Task 7: Background delegation worker

**Files:**
- Create: `backend/crates/nomi-server/src/delegation_worker.rs`
- Modify: `backend/crates/nomi-server/src/lib.rs`
- Modify: `backend/crates/nomi-server/src/main.rs`

**Interfaces:**
- Consumes: `run_agent_turn` (Task 3), `create_delegation`'s `agent_delegations` table (Task 4), `nomi_agent_supervisor::phrase_delegation_result` (Task 6), `StreamEnvelope::AgentDelegationUpdated` (Task 2).

No automated test for the worker's own `run()` loop shell in this task — matching this codebase's existing convention where `worker.rs`'s own `run()` loop has no direct unit test either (its correctness rests on the already-tested pieces it calls: `run_agent_turn`, `phrase_delegation_result`, the claim/mark SQL, all covered by Tasks 3/4/6's own tests). This task's Verify step is a clean build plus careful self-review against the design's stated constraints (no advisory lock, no real `agent_sessions` row).

- [ ] **Step 1: Read `worker.rs` and `queue.rs` once more immediately before writing this file**

Both are short (under 110 lines each) — re-read `backend/crates/nomi-server/src/worker.rs` and `backend/crates/nomi-turn/src/queue.rs` directly before writing Step 2, to catch any drift between what's shown below and the actual current file (this plan's authoring already read them once; a final direct comparison immediately before writing catches anything this plan's transcription got subtly wrong).

- [ ] **Step 2: Write the worker**

Create `backend/crates/nomi-server/src/delegation_worker.rs`:

```rust
use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::LoopOutcome;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};

const NOTIFY_CHANNEL: &str = "agent_delegations_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);
const DELEGATED_MAX_TOKENS: u32 = 1024;

struct ClaimedDelegation {
    id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    target_agent_type: String,
    task: String,
}

async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedDelegation>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM agent_delegations \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE agent_delegations SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, user_id, target_agent_type, task",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, session_id, user_id, target_agent_type, task)| ClaimedDelegation {
        id,
        session_id,
        user_id,
        target_agent_type,
        task,
    }))
}

async fn fail_and_notify(pool: &PgPool, mqtt: &MqttPublisher, delegation_id: Uuid, session_id: Uuid, error: &str) {
    let _ = sqlx::query("UPDATE agent_delegations SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(delegation_id)
        .bind(error)
        .execute(pool)
        .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id }).await;
}

/// Runs the delegation-processing worker loop forever: claims pending `agent_delegations` (via
/// LISTEN/NOTIFY with a polling fallback, mirroring worker.rs's turn_jobs loop exactly), runs
/// each through nomi_agent_core::run_agent_turn directly, phrases the result via the supervisor
/// agent, and delivers it. Deliberately does NOT take the conversational session's advisory
/// lock (see the design spec) — this must never block a user's live conversation.
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String) {
    let mut listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "delegation worker: failed to connect LISTEN client; not started");
            return;
        }
    };
    if let Err(e) = listener.listen(NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "delegation worker: failed to LISTEN on agent_delegations_channel; not started");
        return;
    }
    tracing::info!("delegation worker: listening for new agent delegations");

    let registry = crate::build_agent_registry();

    loop {
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

        loop {
            let claimed = match claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "delegation worker: failed to claim next delegation");
                    break;
                }
            };

            let Some(agent) = registry.find(&claimed.target_agent_type) else {
                tracing::warn!(delegation_id = %claimed.id, target = %claimed.target_agent_type, "delegation worker: unknown target agent");
                fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, "unknown target agent").await;
                continue;
            };

            let provider = build_llm_provider_for_user(&pool, claimed.user_id, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let mut conn = match pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                    continue;
                }
            };

            let messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: claimed.task.clone() }] }];

            let outcome = nomi_agent_core::run_agent_turn(
                &mut conn,
                None,
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                agent,
                claimed.session_id,
                claimed.session_id,
                claimed.user_id,
                messages,
                DELEGATED_MAX_TOKENS,
            )
            .await;

            match outcome {
                Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
                    let phrased = nomi_agent_supervisor::phrase_delegation_result(
                        provider.as_ref(),
                        &mut conn,
                        claimed.user_id,
                        &claimed.target_agent_type,
                        &claimed.task,
                        &text,
                    )
                    .await
                    .unwrap_or_else(|_| text.clone());

                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&phrased)
                        .execute(&mut *conn)
                        .await;

                    let _ = sqlx::query(
                        "UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1",
                    )
                    .bind(claimed.id)
                    .bind(&text)
                    .execute(&pool)
                    .await;

                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;

                    tracing::info!(delegation_id = %claimed.id, "delegation worker: delegation completed");
                }
                Err(e) => {
                    tracing::warn!(delegation_id = %claimed.id, error = %e, "delegation worker: delegated turn failed");
                    let sorry = format!("I wasn't able to get an answer from the {} agent — {}.", claimed.target_agent_type, e);
                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&sorry)
                        .execute(&mut *conn)
                        .await;
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                }
            }
        }
    }
}
```

Add `use nomi_agent_supervisor;` is not needed as a separate import line since it's referenced via its full path (`nomi_agent_supervisor::phrase_delegation_result`) — but `nomi-server`'s `Cargo.toml` already gained this dependency in Task 6, so this compiles without further manifest changes.

- [ ] **Step 3: Wire the module**

In `backend/crates/nomi-server/src/lib.rs`, add `pub mod delegation_worker;` alongside the existing `pub mod worker;`.

- [ ] **Step 4: Spawn it inline from `main.rs`**

In `backend/crates/nomi-server/src/main.rs` (or wherever the binary's `main` function lives — confirm the exact path first if it's not literally `nomi-server/src/main.rs`), inside the existing `if run_worker_inline { ... }` block, after the existing turn-jobs worker's `tokio::spawn`, add a second spawn for the delegation worker:

```rust
        let delegation_mqtt_client_id = format!("nomi-orchestrator-delegation-worker-{}", uuid::Uuid::new_v4());
        let delegation_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &delegation_mqtt_client_id);
        let delegation_pool = pool.clone();
        let delegation_http_client = http_client.clone();
        let delegation_database_url = database_url.clone();
        tokio::spawn(async move {
            nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, settings_key, delegation_http_client, delegation_database_url).await;
        });
```

Both workers share the same `RUN_WORKER_INLINE` flag — no new environment variable.

- [ ] **Step 5: Verify**

```bash
cd backend && cargo build --workspace 2>&1 | tail -60
```

Expected: clean build, zero errors, across the entire workspace including the new binary wiring.

- [ ] **Step 6: Self-review against the two hard constraints**

Re-read the finished `delegation_worker.rs` and confirm, in writing in the task report:
1. `lock::acquire_session_lock` (or any function from `nomi_turn::lock`) is never called anywhere in this file.
2. Every call to `run_agent_turn` in this file passes `claimed.session_id` for BOTH `session_id` and `agent_session_id` — never a value fetched from or written into a real `agent_sessions` row.

- [ ] **Step 7: Commit**

```bash
git add backend/crates/nomi-server/src/delegation_worker.rs backend/crates/nomi-server/src/lib.rs \
        backend/crates/nomi-server/src/main.rs
git commit -m "feat: add the background delegation worker and spawn it inline by default"
```

---

### Task 8: Chat UI — activity indicator and BottomSheet

**Files:**
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/sessions/:id/agent-activity` (Task 5), `StreamEnvelope::AgentDelegationUpdated`'s `"kind": "AgentDelegationUpdated"` JSON shape (Task 2, reaching the frontend verbatim over the existing WS relay — no frontend-side type needs to know the Rust enum, only the string tag), `BottomSheet` (already built, `frontend/src/lib/components/m3/BottomSheet.svelte`).

- [ ] **Step 1: Fetch agent activity in `load()`**

Read the current `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` first to see its exact existing `load()` shape (it currently fetches messages, models, and personality data — mirror that same `apiFetch` + tolerant-empty-on-non-ok pattern for the new call, matching this codebase's established convention rather than introducing a different error-handling style).

Add one more fetch to the existing `load()` function:

```typescript
const agentActivityResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-activity`);
const agentActivity = agentActivityResponse.ok ? await agentActivityResponse.json() : [];
```

And include `agentActivity` in the object `load()` returns, alongside the existing `messages`/`models`/`personality` keys.

- [ ] **Step 2: Handle the new WS event and add the indicator + BottomSheet**

In `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`:

Add the import: `import BottomSheet from '$lib/components/m3/BottomSheet.svelte';`

Add new state near the existing `menuOpen`/`menuView` declarations:

```typescript
let activitySheetOpen = $state(false);

const activeDelegationCount = $derived(
	data.agentActivity.filter((d: { status: string }) => d.status === 'pending' || d.status === 'processing').length,
);
```

In the WS message handler (the existing `if (envelope.kind === 'Delta') { ... } else if (envelope.kind === 'TurnCompleted') { ... } else if (envelope.kind === 'TurnFailed') { ... }` chain), add one more branch:

```typescript
} else if (envelope.kind === 'AgentDelegationUpdated') {
    invalidateAll();
}
```

In the markup, in the same row as the existing chat-settings `Menu` trigger (`<div class="mb-2 flex justify-end">`), add the indicator before that `Menu`, changing the row to hold both:

```svelte
<div class="mb-2 flex items-center justify-between">
	{#if activeDelegationCount > 0}
		<button
			type="button"
			class="md-label-medium"
			style="color: var(--md-sys-color-primary); background: none; border: none; cursor: pointer; padding: 4px 8px;"
			onclick={() => (activitySheetOpen = true)}
		>
			{activeDelegationCount === 1 ? '1 agent working…' : `${activeDelegationCount} agents working…`}
		</button>
	{:else}
		<span></span>
	{/if}
	<Menu bind:open={menuOpen}>
		<!-- unchanged existing Menu content -->
	</Menu>
</div>
```

(The empty `<span></span>` in the `else` branch keeps the `justify-between` layout stable whether or not the indicator is showing — the `Menu` trigger button should stay right-aligned either way.)

Add the `BottomSheet`, anywhere at the top level of the template (e.g. right after the closing `</div>` of the outermost `flex h-full flex-col` container, alongside where `Dialog`/other overlay-style components would live — it renders via `<dialog>`, so its position in the markup doesn't affect layout):

```svelte
<BottomSheet bind:open={activitySheetOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 12px;">Agent activity</h2>
		{#if data.agentActivity.length === 0}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No background activity yet.</p>
		{:else}
			{#each data.agentActivity as item (item.id)}
				<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
					<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">
						{item.target_agent_type} — {item.status}
					</p>
					<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.task}</p>
					{#if item.result}
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.result}</p>
					{/if}
					{#if item.error}
						<p class="md-body-small" style="color: var(--md-sys-color-error)">{item.error}</p>
					{/if}
				</div>
			{/each}
		{/if}
	{/snippet}
</BottomSheet>
```

This matches `BottomSheet`'s actual contract exactly (`{ open?: boolean (bindable); children: Snippet; class?: string }`, from `frontend/src/lib/components/m3/BottomSheet.svelte`, built earlier this session) — `bind:open` plus a single `{#snippet children()}` block is the only way it's used anywhere else in this codebase.

- [ ] **Step 3: Verify**

```bash
cd frontend && npm run check
```

Expected: 0 errors (the same pre-existing `Menu.svelte` a11y warning is fine, unrelated to this task).

- [ ] **Step 4: Self-review**

Confirm, by reading the finished page: the indicator only appears when `activeDelegationCount > 0`; clicking it opens the `BottomSheet`; the WS handler's new branch calls `invalidateAll()` the same way `TurnCompleted` already does (proven pattern, not a new one); the existing chat-settings `Menu` (model/personality picker) is completely untouched in content, only its containing row's layout changed to accommodate the new indicator alongside it.

- [ ] **Step 5: Commit**

```bash
git add "frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts" "frontend/src/routes/(app)/chat/[sessionId]/+page.svelte"
git commit -m "feat: add background agent activity indicator and BottomSheet to the chat page"
```

---

## Task Ordering Note

Tasks 1 and 2 are independent of each other and of everything else; both must land before Task 3 (which consumes Task 1's `can_delegate`/`delegatable_agent_types` and, indirectly through Task 4, Task 2's `StreamEnvelope` variant — though Task 3 itself only needs Task 1). Task 3 is the highest-risk task (an existing function's signature changes across 9 call sites) and must fully land, with a clean whole-workspace build and test pass, before Task 4 (which adds the tool's actual execution behavior on top of Task 3's plumbing). Task 4 depends on Tasks 2 and 3. Task 5 depends only on Task 4's migration. Task 6 depends on Task 1 (trait shape) and Task 4 (the table `list_recent_agent_activity` queries) but not on Task 5. Task 7 depends on Tasks 3, 4, and 6 together (it calls `run_agent_turn` with the registry parameter, reads/writes `agent_delegations`, and calls `phrase_delegation_result`). Task 8 depends on Task 5 (the endpoint it fetches) and transitively on Task 7 (nothing publishes `AgentDelegationUpdated` for it to react to until the worker exists, though the frontend code itself only directly needs Task 2's wire format and Task 5's endpoint to compile/render — full end-to-end behavior needs Task 7 too).
