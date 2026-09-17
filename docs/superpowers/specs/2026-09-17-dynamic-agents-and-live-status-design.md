# Dynamic Agents & Live Agent Status — Design

**Goal:** Let admins create new agents entirely from the web (identity, system prompt, and a
curated set of existing tool capabilities — no Rust code), running through the exact same turn
engine as every hand-coded agent, and give both admins and users live visibility into what any
agent — built-in or admin-created — is doing right now (thinking, calling a tool, writing a
reply) on any active session.

**Architecture:** A new `dynamic_agents` table holds admin-authored agent definitions. A new
`DynamicAgent` struct implements the existing `SubAgent` trait, constructed fresh from a DB row
whenever it's needed — never cached, never mutating the static, compile-time `AgentRegistry`. A
new `ToolCatalog` lets a `DynamicAgent` call the *same* Rust functions built-in agents already use
for a curated, admin-selected subset of tools, with zero duplicated logic. `SubAgent`'s
`&'static str`-returning methods become `Cow<'static, str>` to accommodate DB-generated identity
strings — a mechanical, behavior-neutral change for every existing agent. A new `current_phase`
column on `agent_sessions`, updated at a handful of points already inside the shared turn loop,
gives every agent — built-in or dynamic — live status for free.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, the existing MQTT/WS realtime relay (`nomi-realtime`).

## Context

Today, adding an agent means writing a new crate implementing `SubAgent`, registering it in
`build_agent_registry`, and redeploying. This project adds a second path — admin-authored agents
stored in the database, composed from a fixed, Rust-reviewed catalog of tool capabilities rather
than arbitrary code — while keeping execution flowing through the identical `run_agent_turn`
engine loop every agent already uses. Separately, admins and users currently have no visibility
into what an active agent is doing beyond "it's active since some timestamp" (the existing
`/admin/agents` page) — this project adds real-time phase visibility for every agent.

## Data Model

```sql
CREATE TABLE dynamic_agents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL,
    system_prompt       TEXT NOT NULL,
    intent_label        TEXT NOT NULL UNIQUE,
    intent_description  TEXT NOT NULL,
    granted_tools       TEXT[] NOT NULL DEFAULT '{}',
    supports_todos      BOOLEAN NOT NULL DEFAULT false,
    supports_plans      BOOLEAN NOT NULL DEFAULT false,
    can_delegate        BOOLEAN NOT NULL DEFAULT false,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    created_by          UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE agent_sessions ADD COLUMN current_phase TEXT NOT NULL DEFAULT 'waiting';
ALTER TABLE agent_sessions ADD COLUMN current_phase_detail TEXT;
```

- `granted_tools` is a plain array — the catalog is small and admin-curated, not something needing
  a join table's relational query power.
- `is_active = false` soft-disables an agent (excluded from classification) without breaking
  `agent_sessions` rows already referencing it, and without losing its `dynamic_agents` row's
  history. No hard delete in this spec.
- `current_phase` is one of `waiting` | `thinking` | `calling_tool` | `writing_reply` — a
  separate axis from the existing `status` column (`active`/`expired`/`completed`), which tracks
  session lifecycle, not moment-to-moment activity. `current_phase_detail` carries free text for
  the one case that needs it (`calling_tool` → the tool's name).
- Chitchat's sentinel `agent_session_id == session_id` (no real `agent_sessions` row) means phase
  updates silently no-op for chitchat turns — consistent with how `agent_sessions.state` writes
  already behave for chitchat today (it has zero tools, so this never causes a visible gap).

## `SubAgent` Trait Change

```rust
// backend/crates/nomi-agent-core/src/subagent.rs
use std::borrow::Cow;

pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> Cow<'static, str>;
    fn intent_label(&self) -> Cow<'static, str>;
    fn intent_description(&self) -> Cow<'static, str>;
    fn system_prompt(&self) -> Cow<'static, str>;
    // tools(), execute_tool(), is_default(), uses_memory(), uses_personality(), can_delegate(),
    // is_delegation_target(), surfaces_activity(), supports_todos(), supports_plans(),
    // validate_delegation_task() — all unchanged.
}
```

Every existing implementor's change is mechanical and behavior-neutral: `fn agent_type(&self) ->
&'static str { "money" }` becomes `fn agent_type(&self) -> Cow<'static, str> {
Cow::Borrowed("money") }`, same shape for the other three methods, across all 6 existing agent
crates (chitchat, money, personality, supervisor, planning, coding). `AgentRegistry::find`,
`find_by_intent_label`, and `classification_prompt` need matching signature updates (comparing
`Cow<str>` instead of `&str`) but no logic changes.

## `DynamicAgent`

```rust
// backend/crates/nomi-agent-core/src/dynamic_agent.rs (new)
pub struct DynamicAgent {
    id: Uuid,
    name: String,
    system_prompt: String,
    intent_label: String,
    intent_description: String,
    granted_tools: Vec<String>,
    supports_todos_flag: bool,
    supports_plans_flag: bool,
    can_delegate_flag: bool,
    catalog: Arc<ToolCatalog>,
}

impl DynamicAgent {
    pub fn from_row(row: DynamicAgentRow, catalog: Arc<ToolCatalog>) -> Self { /* field mapping */ }
}

#[async_trait]
impl SubAgent for DynamicAgent {
    fn agent_type(&self) -> Cow<'static, str> { Cow::Owned(self.id.to_string()) }
    fn intent_label(&self) -> Cow<'static, str> { Cow::Owned(self.intent_label.clone()) }
    fn intent_description(&self) -> Cow<'static, str> { Cow::Owned(self.intent_description.clone()) }
    fn system_prompt(&self) -> Cow<'static, str> { Cow::Owned(self.system_prompt.clone()) }
    fn tools(&self) -> Vec<ToolDefinition> { self.catalog.definitions_for(&self.granted_tools) }
    async fn execute_tool(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, name: &str, input: Value) -> Result<ToolOutcome, String> {
        self.catalog.execute(name, conn, session_id, agent_session_id, user_id, input).await
    }
    fn uses_memory(&self) -> bool { false }       // v1: no memory for dynamic agents
    fn uses_personality(&self) -> bool { false }  // v1: no personality folding for dynamic agents
    fn can_delegate(&self) -> bool { self.can_delegate_flag }
    fn supports_todos(&self) -> bool { self.supports_todos_flag }
    fn supports_plans(&self) -> bool { self.supports_plans_flag }
    fn is_default(&self) -> bool { false }
}
```

Constructed fresh wherever it's needed (classification, or resuming a paused turn whose
`agent_type` — the row's UUID — no longer matches any built-in agent). Never held in a static
list, never cached — every lookup is a DB read. `uses_memory`/`uses_personality` are hardcoded
`false` for v1: both features assume a single, well-known personality/memory-extraction prompt
shape tuned per built-in agent; extending them to admin-authored agents is a real design question
of its own, deliberately deferred rather than half-implemented here.

## The Tool Catalog

```rust
// backend/crates/nomi-agent-core/src/tool_catalog.rs (new)
#[async_trait]
pub trait CatalogTool: Send + Sync {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String>;
}

pub struct ToolCatalog {
    entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)>,
}

impl ToolCatalog {
    pub fn definitions_for(&self, granted: &[String]) -> Vec<ToolDefinition> {
        granted.iter().filter_map(|name| self.entries.get(name.as_str())).map(|(def, _)| def.clone()).collect()
    }
    pub async fn execute(&self, name: &str, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        match self.entries.get(name) {
            Some((_, tool)) => tool.execute(conn, session_id, agent_session_id, user_id, input).await,
            None => Err(format!("tool not granted: {name}")),
        }
    }
}
```

Built once at startup, alongside `build_agent_registry` (same function, extended to also return
the catalog, or a sibling `build_tool_catalog(project_storage: LocalFsStore)`). Most existing
tools are already free functions with varying signatures (`list_transactions(conn, user_id,
input)`, `create_project(conn, session_id, user_id, input)`) — each gets a thin adapter struct
implementing `CatalogTool` that calls the real function, ignoring whatever parameters it doesn't
need:

```rust
struct ListTransactions;
#[async_trait]
impl CatalogTool for ListTransactions {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_money::list_transactions(conn, user_id, input).await.map(ToolOutcome::text)
    }
}
```

**`CodingAgent::write_file`/`delete_file` are refactored into free functions** taking `storage:
&LocalFsStore` as an explicit parameter (matching the pattern every other grantable tool already
follows), with `CodingAgent::execute_tool` updated to call them with `&self.storage` — no behavior
change for `CodingAgent` itself. The catalog's adapters for these close over their own cloned
`LocalFsStore` (built once at startup, the same instance `build_agent_registry` already receives).

**V1 catalog contents:** `list_transactions`, `summarize_budget` (money); `create_project`
(planning — `write_plan` itself is already a generic engine tool every `supports_plans()` agent
gets automatically, not something to grant separately); `set_personality`,
`list_personality_versions`, `rollback_personality` (personality); `list_recent_agent_activity`
(supervisor); `write_file`, `read_file`, `list_files`, `delete_file` (coding, post-refactor).

**Isolation, stated explicitly:** every catalog adapter receives `user_id` from the real turn
context (threaded through `execute_tool`'s existing parameters), never from the `dynamic_agents`
row. A dynamic agent granted `list_transactions` only ever sees the *calling* user's own
transactions — identical isolation to `MoneyAgent` today. Granting a tool changes *what* an admin
lets a dynamic agent do, never *whose* data it can touch.

## Routing & Classification

Returning either a built-in agent (today, a borrowed `&dyn SubAgent` into the registry's owned
list) or a freshly-constructed `DynamicAgent` from the same function needs a shared ownership
model. `AgentRegistry`'s internal storage changes from `Vec<Box<dyn SubAgent>>` to
`Vec<Arc<dyn SubAgent>>` — every built-in agent is still constructed exactly once at startup, just
held behind a cheaply-cloneable `Arc` instead of an owned `Box`. `AgentRegistry::find`,
`find_by_intent_label`, and `default_agent` return a cloned `Arc<dyn SubAgent>` (a pointer-and-
refcount bump, not a data copy) instead of `&dyn SubAgent`. A `DynamicAgent` is wrapped in a fresh
`Arc::new(...)` the same way. Every call site that currently holds `&dyn SubAgent` (borrowed from
whatever produced it) keeps working unchanged via `arc.as_ref()`/auto-deref — `run_agent_turn`
and `resolve_tool_batch` themselves need no signature change at all.

```rust
// backend/crates/nomi-turn/src/routing.rs
pub async fn classify_intent(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    registry: &AgentRegistry,
    catalog: Arc<ToolCatalog>,
    text: &str,
) -> Arc<dyn SubAgent> {
    let dynamic_rows = fetch_active_dynamic_agents(conn).await.unwrap_or_default();
    let prompt = build_combined_classification_prompt(registry, &dynamic_rows);
    let label = /* existing LLM classification call, prompt swapped in */;

    if let Some(agent) = registry.find_by_intent_label(&label) {
        return agent;
    }
    if let Some(row) = dynamic_rows.iter().find(|r| r.intent_label == label) {
        return Arc::new(DynamicAgent::from_row(row.clone(), catalog));
    }
    registry.default_agent()
}
```

Dynamic agents are fetched fresh from the DB on every classification call — no cache, no
invalidation to get wrong, no delay between an admin creating an agent and it becoming routable.
This costs one extra indexed query per turn that needs classification (turns continuing with an
already-active `agent_sessions` row skip classification entirely, same as today) — negligible
next to the classification LLM call itself.

## Live Status Tracking

A new best-effort helper (matching `post_activity_message`'s "never fails the turn" convention):

```rust
async fn update_agent_phase(conn: &mut PoolConnection<Postgres>, agent_session_id: Uuid, phase: &str, detail: Option<&str>) {
    let _ = sqlx::query("UPDATE agent_sessions SET current_phase = $1, current_phase_detail = $2 WHERE id = $3")
        .bind(phase).bind(detail).bind(agent_session_id).execute(&mut **conn).await;
}
```

Called from four points already inside `run_agent_turn`/`resolve_tool_batch`:

1. **Before each LLM call** → `thinking`, `current_phase_detail: None`.
2. **Before executing each tool** in the resolved batch → `calling_tool`, `current_phase_detail:
   Some(tool_name)` — one call per tool, so a multi-tool response shows the right name as each
   runs in sequence.
3. **When the LLM response is a final text reply** (not a tool call), right before it's persisted
   → `writing_reply`.
4. **At every turn-ending point** (reply persisted, approval pause, completion) → `waiting`.

Applies uniformly to every agent, built-in or dynamic, since it lives in the shared engine loop.

## API & UI

**Admin CRUD** — `/api/admin/dynamic-agents` (`GET` list, `POST` create, `PUT /:id` update,
`POST /:id/toggle-active`), gated by the same `require_system_config_permission` check
`get_agents` already uses. No hard delete — `is_active` is the only lifecycle transition, to
preserve `agent_sessions` history. Admin UI: a new `/admin/dynamic-agents` page, list + a
`BottomSheet`-based create/edit form (matching the existing LLM-models admin page's established
pattern), with a tool-picker grouping checkboxes by source agent ("Money: list_transactions,
summarize_budget", "Coding: write_file, delete_file, read_file, list_files", etc.).

**Admin live status** — the existing `get_agents`/`RunningAgentItem` gain `current_phase` and
`current_phase_detail`; the existing `/admin/agents` page renders them, e.g. `[Nomi Money Agent]
— session abc123 — calling tool: list_transactions`.

**User-facing live status** — a new endpoint, `GET /api/sessions/:id/agent-status`, authorized via
the existing `authorize_session_access` (so a user only ever sees status for sessions they
actually participate in — no admin-only data crosses over), returning the same
`current_phase`/`current_phase_detail` shape for that session's currently-active agent, if any.
Pushed live via a new `StreamEnvelope::AgentPhaseChanged { agent_session_id, phase, detail }`
variant over the existing MQTT/WS relay — reusing the exact realtime mechanism `Delta`/
`MessageCreated`/`MessageUpdated` already use, not a new transport.

## Error Handling

- Every DB write in this design (`update_agent_phase`, the dynamic-agent classification lookup) is
  best-effort — a failure never fails the user's turn, matching this codebase's existing
  convention for status/activity writes throughout the turn engine.
- **Write-time validation, runtime tolerance — two different rules for two different moments.**
  The admin CRUD `POST`/`PUT` handlers validate every submitted `granted_tools` name against
  `ToolCatalog`'s known entries and reject the request (400) if any name is unrecognized — an
  admin should never end up with a saved agent that silently grants nothing they intended.
  Separately, `ToolCatalog::definitions_for` (used at classification/tool-listing time, not at
  save time) tolerates unknown names by skipping them rather than erroring — this covers the
  narrower case of a tool being removed from the Rust catalog *after* an agent was already saved
  with it (e.g. a backend deploy retires a tool), where the agent should degrade gracefully rather
  than break, not a case an admin can proactively avoid.
- A `DynamicAgent` whose `is_active` flips to `false` mid-conversation: the currently-paused/
  active `agent_sessions` row keeps working (routing only filters *new* classifications, not
  agents already mid-turn) — consistent with `is_active`'s stated "soft-disable for future
  routing" semantics, not an emergency kill switch.

## Testing

Same approach as every other feature built in this session: `#[cfg(test)]`/`#[sqlx::test]`
integration tests for the trait change (verify every existing agent still classifies/executes
correctly post-`Cow` migration), the tool catalog (each grantable tool dispatches to the right
underlying function, an ungranted tool name is rejected), `DynamicAgent` (constructs correctly
from a row, respects `granted_tools`/`supports_todos`/`supports_plans`/`can_delegate` flags),
routing (a dynamic agent's intent label is classified into correctly, a soft-disabled one is
excluded), and the phase-tracking helper (each of the four transition points writes the right
phase). Admin CRUD routes get integration tests matching the existing LLM-models admin route
tests' pattern. Frontend: `svelte-check` + live verification for the new admin pages (no component
test harness in this repo), matching the standing exception already established.

## Out of Scope

- **Memory and personality folding for dynamic agents** — deliberately deferred (see
  `DynamicAgent`'s hardcoded `false`s above); a real design question about what "personality" even
  means for an admin-authored agent, not a gap to paper over.
- **Delegation *targets* being dynamic agents** — `can_delegate` (can this agent delegate *to*
  others) is in scope; whether a dynamic agent can itself be delegated *to* by another agent is a
  smaller follow-on (mechanically: `is_delegation_target()` just needs a matching DB flag) left out
  of v1 to keep the first cut's surface area bounded.
- **A generic "grant any tool, including future ones, automatically"** mechanism — the catalog is
  a deliberately curated, Rust-reviewed allowlist; adding a new grantable tool always means a small
  Rust change (registering the adapter), never something admin-configurable end-to-end. This is a
  deliberate safety boundary, not a limitation to lift later without reconsidering it.
