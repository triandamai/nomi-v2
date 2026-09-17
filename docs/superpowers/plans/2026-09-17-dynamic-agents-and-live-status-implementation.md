# Dynamic Agents & Live Agent Status Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let admins create new agents entirely from the web (identity, system prompt, a curated
set of existing tool capabilities), running through the exact same turn engine as every hand-coded
agent, and give admins and users live visibility into what any agent — built-in or admin-created —
is doing right now on any active session.

**Architecture:** A new `dynamic_agents` table holds admin-authored agent definitions. A new
`DynamicAgent` struct (nomi-agent-core) implements the existing `SubAgent` trait, built fresh from
a DB row whenever needed. A new `ToolCatalog` (generic map, nomi-agent-core) lets a `DynamicAgent`
call the same Rust functions built-in agents already use, via adapters registered in nomi-server
(which already depends on every agent crate — this avoids a circular dependency from
nomi-agent-core back onto the agent crates). `SubAgent`'s `&'static str`-returning methods become
`Cow<'static, str>`. `AgentRegistry` moves from `Vec<Box<dyn SubAgent>>` to `Vec<Arc<dyn SubAgent>>`
internal storage so built-in and dynamic agents share one return type at every call site. A new
`current_phase`/`current_phase_detail` pair on `agent_sessions`, updated at four points already
inside the shared turn loop, gives every agent live status for free, pushed over the existing
MQTT/WS relay.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, the existing MQTT/WS realtime relay (`nomi-realtime`).

**Spec:** `docs/superpowers/specs/2026-09-17-dynamic-agents-and-live-status-design.md`

## Global Constraints

- Every DB write for phase-tracking or dynamic-agent classification is best-effort — never fails
  the user's turn (matches the codebase's existing convention, e.g. `post_activity_message`).
- Admin CRUD write-time (`POST`/`PUT`) validates every `granted_tools` name against
  `ToolCatalog::known_tool_names()` and rejects (400) unknown names. `ToolCatalog::definitions_for`
  (read/runtime path) tolerates unknown names by skipping them.
- No hard delete for `dynamic_agents` — `is_active` is the only lifecycle transition.
- A `DynamicAgent`'s catalog adapter always receives `user_id` from the real turn context, never
  from the `dynamic_agents` row — granting a tool changes *what* it can do, never *whose* data.
- `uses_memory()` and `uses_personality()` are hardcoded `false` for every `DynamicAgent` (v1 scope
  cut, see spec's Out of Scope).
- Dynamic agents are never valid `delegate_to_agent` targets in v1 (`is_delegation_target` stays
  the trait default `true`, but `delegatable_agent_types` only ever iterates the registry's
  built-in agents, so this is moot — no code needs to special-case it).
- All backend crate paths below are relative to `backend/`; all frontend paths relative to
  `frontend/`.

---

### Task 1: Migration — `dynamic_agents` table and `agent_sessions` phase columns

**Files:**
- Create: `migrations/0025_dynamic_agents_and_phase.sql`

**Interfaces:**
- Produces: `dynamic_agents` table (columns per spec), `agent_sessions.current_phase` (`TEXT NOT
  NULL DEFAULT 'waiting'`), `agent_sessions.current_phase_detail` (`TEXT`, nullable). Every later
  task reads/writes these.

- [ ] **Step 1: Write the migration**

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

CREATE INDEX idx_dynamic_agents_active ON dynamic_agents (is_active);

ALTER TABLE agent_sessions ADD COLUMN current_phase TEXT NOT NULL DEFAULT 'waiting';
ALTER TABLE agent_sessions ADD COLUMN current_phase_detail TEXT;
```

- [ ] **Step 2: Apply and verify**

Run: `sqlx migrate run` (from `backend/`, with `DATABASE_URL` set) or let the server's own
`sqlx::migrate!` boot path apply it. Verify: `\d dynamic_agents` and `\d agent_sessions` in `psql`
show the new table/columns.

- [ ] **Step 3: Commit**

```bash
git add migrations/0025_dynamic_agents_and_phase.sql
git commit -m "feat: add dynamic_agents table and agent_sessions phase columns"
```

---

### Task 2: `SubAgent` trait → `Cow<'static, str>`, `AgentRegistry` → `Arc<dyn SubAgent>`

Mechanical, behavior-neutral change across 6 agent crates plus `nomi-agent-core`'s registry. No
new behavior — every existing test must still pass unchanged in assertions, only call-site shapes
change.

**Files:**
- Modify: `crates/nomi-agent-core/src/subagent.rs`
- Modify: `crates/nomi-agent-core/src/registry.rs`
- Modify: `crates/nomi-agent-chitchat/src/lib.rs`
- Modify: `crates/nomi-agent-money/src/lib.rs`
- Modify: `crates/nomi-agent-personality/src/lib.rs`
- Modify: `crates/nomi-agent-supervisor/src/lib.rs`
- Modify: `crates/nomi-agent-planning/src/lib.rs`
- Modify: `crates/nomi-agent-coding/src/lib.rs`
- Modify: `crates/nomi-agent-core/src/engine.rs` (one comparison + one delegate-targets type)
- Test: `crates/nomi-agent-core/tests/registry.rs` (existing tests, unchanged assertions — only
  compile fallout, see Step 5)

**Interfaces:**
- Produces: `SubAgent::agent_type/intent_label/intent_description/system_prompt` all return
  `Cow<'static, str>`. `AgentRegistry::find`, `find_by_intent_label`, `default_agent` all return
  `Arc<dyn SubAgent>`. `AgentRegistry::delegatable_agent_types` returns `Vec<String>` (was
  `Vec<&'static str>`). New `AgentRegistry::classification_prompt_with_extra(&self, extra_labels:
  &[String], extra_options: &[String]) -> String`; `classification_prompt()` becomes a thin
  wrapper calling it with two empty slices.
- Consumes: nothing new.

- [ ] **Step 1: Rewrite `subagent.rs`'s trait signature**

Replace every `&'static str` return type on `agent_type`, `system_prompt`, `intent_label`,
`intent_description` with `Cow<'static, str>`, and add the import. Keep every doc comment and every
default-method body byte-identical — only these four signatures change.

```rust
use std::borrow::Cow;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> Cow<'static, str>;
    fn system_prompt(&self) -> Cow<'static, str>;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String>;

    /// Fed verbatim into the intent classifier's prompt. Never called for the agent that
    /// returns `true` from `is_default()` — that agent is the fallback, not something the
    /// classifier picks between.
    fn intent_label(&self) -> Cow<'static, str>;
    fn intent_description(&self) -> Cow<'static, str>;

    /// Exactly one registered agent must return `true`. See `AgentRegistry::new`.
    fn is_default(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` retrieves relevant memories before the first LLM call
    /// (folded into the system prompt) and extracts+stores new memories after a `Reply`
    /// outcome (never after `Completed` — a completed task summary isn't a conversational
    /// reply worth remembering facts from).
    fn uses_memory(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` folds the user's currently stored personality (if any)
    /// into the system prompt for this turn. See `nomi_agent_core::personality`.
    fn uses_personality(&self) -> bool {
        false
    }

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

    /// When true, run_agent_turn posts a chat message for every tool call this agent makes
    /// (and for any text it writes alongside one, treated as its "thinking out loud") — so the
    /// user watching a long-running build sees each step as it happens, not just a final
    /// summary. Off by default: most agents' tool calls are internal bookkeeping the user
    /// never needs to see.
    fn surfaces_activity(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent the engine-level `update_todos` tool for
    /// maintaining a live multi-step task checklist. Off by default — most agents don't run
    /// long enough multi-step builds to need one.
    fn supports_todos(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent the engine-level `write_plan` tool for writing
    /// a durable, versioned plan before or during a multi-step task — see
    /// docs/superpowers/specs/2026-09-17-agent-plan-artifacts-design.md. Off by default, same
    /// reasoning as supports_todos(): most agents don't do work substantial enough to plan first.
    fn supports_plans(&self) -> bool {
        false
    }

    /// Called on the delegation *target* before `delegate_to_agent` creates anything, with the
    /// exact task string the delegating agent wrote. Return `Err(reason)` to reject the
    /// delegation outright — no delegation row is created, and `reason` is fed straight back to
    /// the delegating agent as a tool error, giving it a chance to self-correct in the same turn
    /// (e.g. actually create a project before delegating to the coding agent) instead of a
    /// malformed delegation reaching a target agent that has no way to act on it. Default
    /// accepts anything: only targets with a real structural precondition on the task string
    /// need to override this.
    fn validate_delegation_task(&self, _task: &str) -> Result<(), String> {
        Ok(())
    }
}
```

- [ ] **Step 2: Rewrite `registry.rs`**

```rust
use std::sync::Arc;

use crate::subagent::SubAgent;

pub struct AgentRegistry {
    agents: Vec<Arc<dyn SubAgent>>,
    default_index: usize,
}

impl AgentRegistry {
    /// Panics if zero or more than one agent returns `true` from `is_default()` — a
    /// misconfigured registry is a startup-time programmer error, not something to degrade
    /// gracefully from at runtime. Still takes owned `Box`es at the call site (every existing
    /// `AgentRegistry::new(vec![Box::new(Agent), ...])` call keeps compiling unchanged) —
    /// converted to `Arc` once here, so every built-in agent is still constructed exactly once.
    pub fn new(agents: Vec<Box<dyn SubAgent>>) -> Self {
        let agents: Vec<Arc<dyn SubAgent>> = agents.into_iter().map(Arc::from).collect();

        let default_indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.is_default())
            .map(|(i, _)| i)
            .collect();

        match default_indices.as_slice() {
            [index] => Self { agents, default_index: *index },
            [] => panic!("AgentRegistry::new: no registered agent returns true from is_default()"),
            _ => panic!(
                "AgentRegistry::new: more than one registered agent returns true from is_default(): {:?}",
                default_indices.iter().map(|&i| agents[i].agent_type().into_owned()).collect::<Vec<_>>()
            ),
        }
    }

    pub fn default_agent(&self) -> Arc<dyn SubAgent> {
        self.agents[self.default_index].clone()
    }

    pub fn find(&self, agent_type: &str) -> Option<Arc<dyn SubAgent>> {
        self.agents.iter().find(|a| a.agent_type().as_ref() == agent_type).cloned()
    }

    /// Finds the agent whose `intent_label()` matches, ignoring case/whitespace (matching
    /// how the classifier's raw LLM text response is normalized before lookup). Despite the
    /// classifier prompt asking for exactly one word, some models don't reliably comply —
    /// a trailing period, a quoted label, or a whole sentence ("This is about planning.")
    /// would otherwise fail an exact-string match and silently misroute to the default agent
    /// with no visible error. Falling back to a whole-word search inside the response is far
    /// cheaper than that failure mode: exact match is tried first (the common, well-behaved
    /// case), then each whitespace/punctuation-separated token is checked for an exact match
    /// against a registered label.
    pub fn find_by_intent_label(&self, label: &str) -> Option<Arc<dyn SubAgent>> {
        let normalized = label.trim().to_lowercase();
        if let Some(agent) = self.agents.iter().find(|a| a.intent_label().as_ref() == normalized) {
            return Some(agent.clone());
        }
        self.agents
            .iter()
            .find(|a| normalized.split(|c: char| !c.is_alphanumeric()).any(|word| word == a.intent_label().as_ref()))
            .cloned()
    }

    /// Builds the classifier's prompt from every non-default registered agent's
    /// intent_label/intent_description. The default agent is the fallback, not a
    /// classification target (matching today's behavior, where chitchat only wins via
    /// explicit classification OR fallback, never by having its own enum variant picked
    /// exclusively for it).
    pub fn classification_prompt(&self) -> String {
        self.classification_prompt_with_extra(&[], &[])
    }

    /// Same as `classification_prompt`, with `extra_labels`/`extra_options` (same "label:
    /// description" shape, one entry per active dynamic agent) merged in after the built-in
    /// agents — used by `nomi_turn::routing::classify_intent` so dynamic agents are routable
    /// through the identical classifier prompt, without this registry needing to know anything
    /// about the `dynamic_agents` table itself.
    pub fn classification_prompt_with_extra(&self, extra_labels: &[String], extra_options: &[String]) -> String {
        let default_agent = self.default_agent();

        let mut labels: Vec<String> =
            self.agents.iter().filter(|a| !a.is_default()).map(|a| a.intent_label().into_owned()).collect();
        let mut options: Vec<String> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| format!("{}: {}", a.intent_label(), a.intent_description()))
            .collect();
        labels.extend(extra_labels.iter().cloned());
        options.extend(extra_options.iter().cloned());

        crate::prompts::INTENT_CLASSIFICATION_PROMPT_TEMPLATE
            .replace("{labels}", &labels.join(", "))
            .replace("{default_label}", default_agent.intent_label().as_ref())
            .replace("{options}", &options.join("\n"))
    }

    /// agent_type() of every non-default, delegation-eligible agent other than `excluding` —
    /// the valid target list for a delegate_to_agent tool call from that agent. Only ever
    /// built-in agents: dynamic agents are never delegation targets in v1 (see design spec's
    /// Out of Scope), so this never needs to consult the `dynamic_agents` table.
    pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<String> {
        self.agents
            .iter()
            .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type().as_ref() != excluding)
            .map(|a| a.agent_type().into_owned())
            .collect()
    }
}
```

- [ ] **Step 3: Fix the six agent crates' trait impls**

For each of `nomi-agent-chitchat`, `nomi-agent-money`, `nomi-agent-personality`,
`nomi-agent-supervisor`, `nomi-agent-planning`, `nomi-agent-coding`, change every `fn agent_type(&self)
-> &'static str { X }` to `fn agent_type(&self) -> Cow<'static, str> { Cow::Borrowed(X) }`, same
shape for `system_prompt`, `intent_label`, `intent_description`, and add `use std::borrow::Cow;` to
each file's imports. Concretely, for every one of these six files, the diff is:

```rust
// before
fn agent_type(&self) -> &'static str {
    MONEY_AGENT_TYPE
}
// after
fn agent_type(&self) -> Cow<'static, str> {
    Cow::Borrowed(MONEY_AGENT_TYPE)
}
```

applied identically to `system_prompt`, `intent_label`, `intent_description` in each file (the
literal each one currently returns stays exactly the same — only wrapped in `Cow::Borrowed`).
Run this transform with sed on all six files at once (safe here because every occurrence of the
`-> &'static str {` pattern in these six trait impls is one of exactly these four methods — verify
with `grep -n "fn \(agent_type\|system_prompt\|intent_label\|intent_description\)" <file>` first
if unsure):

```bash
for f in crates/nomi-agent-chitchat/src/lib.rs crates/nomi-agent-money/src/lib.rs \
         crates/nomi-agent-personality/src/lib.rs crates/nomi-agent-supervisor/src/lib.rs \
         crates/nomi-agent-planning/src/lib.rs crates/nomi-agent-coding/src/lib.rs; do
  sed -i '' -E 's/fn (agent_type|system_prompt|intent_label|intent_description)\(&self\) -> &.static str \{/fn \1(\&self) -> Cow<'"'"'static, str> {/' "$f"
done
```

This changes the signature line only; each method body still returns a bare `&'static str`
constant/literal, which no longer satisfies the new return type. Fix each body by hand next —
wrap every returned expression in `Cow::Borrowed(...)`. For `nomi-agent-chitchat/src/lib.rs`, that's:

```rust
fn agent_type(&self) -> Cow<'static, str> {
    Cow::Borrowed(CHITCHAT_AGENT_TYPE)
}
fn system_prompt(&self) -> Cow<'static, str> {
    Cow::Borrowed(CHITCHAT_SYSTEM_PROMPT)
}
// ...
fn intent_label(&self) -> Cow<'static, str> {
    Cow::Borrowed(CHITCHAT_AGENT_TYPE)
}
fn intent_description(&self) -> Cow<'static, str> {
    Cow::Borrowed("General conversation, questions, or anything not covered by another agent")
}
```

Apply the same `Cow::Borrowed(...)` wrap to each of the other five crates' four method bodies
(their return expressions are unchanged from what they are today — only wrapped). Add `use
std::borrow::Cow;` near the top of each of the six files. Do this step for all six crates before
attempting to compile — the crate graph won't build with only some done.

- [ ] **Step 4: Fix the two call sites in `engine.rs` that reference delegate targets**

`delegate_tool_definition` currently takes `targets: &[&str]`; `run_agent_turn` currently does
`registry.delegatable_agent_types(agent.agent_type())` where `agent.agent_type()` used to be
`&str` and is now `Cow<'static, str>`. Fix both:

```rust
// engine.rs — delegate_tool_definition's signature
fn delegate_tool_definition(targets: &[String]) -> ToolDefinition {
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

And in `run_agent_turn`:

```rust
if agent.can_delegate() {
    let targets = registry.delegatable_agent_types(agent.agent_type().as_ref());
    if !targets.is_empty() {
        tools.push(delegate_tool_definition(&targets));
    }
}
```

- [ ] **Step 5: Compile, fix remaining fallout, run the full suite**

Run: `cargo build --workspace` from `backend/`. Fix every remaining error mechanically — they will
all be one of: (a) a `.bind(agent.agent_type())` needing `.bind(agent.agent_type().as_ref())` (sqlx
does not implement `Encode` for `Cow<str>`, only `&str`/`String`), (b) a place holding `&dyn
SubAgent` from `registry.find`/`find_by_intent_label`/`default_agent` that now receives `Arc<dyn
SubAgent>` and needs `.as_ref()` where a `&dyn SubAgent` is required downstream, or (c) a
`delegatable_agent_types`/`agent_type()` comparison against `&str` needing `.as_ref()`. Do not
change any test's assertions — only add `.as_ref()`/`.clone()` as needed to satisfy the compiler.
This task does not yet touch `nomi-turn` or `nomi-server` (Task 6 handles those, which also depend
on the tool catalog from Tasks 3-4) — for now, get `nomi-agent-core` and all six agent crates
compiling and their own test suites green; leave `nomi-turn`/`nomi-server` compile errors for
Task 6 (expected at this point — do not attempt to fix them here).

Run: `cargo test -p nomi-agent-core -p nomi-agent-chitchat -p nomi-agent-money -p
nomi-agent-personality -p nomi-agent-supervisor -p nomi-agent-planning -p nomi-agent-coding`
Expected: every test in these 7 crates passes, same count as before this task.

- [ ] **Step 6: Commit**

```bash
git add crates/nomi-agent-core/src/subagent.rs crates/nomi-agent-core/src/registry.rs \
        crates/nomi-agent-core/src/engine.rs crates/nomi-agent-chitchat/src/lib.rs \
        crates/nomi-agent-money/src/lib.rs crates/nomi-agent-personality/src/lib.rs \
        crates/nomi-agent-supervisor/src/lib.rs crates/nomi-agent-planning/src/lib.rs \
        crates/nomi-agent-coding/src/lib.rs
git commit -m "refactor: SubAgent identity methods return Cow<str>, AgentRegistry stores Arc<dyn SubAgent>"
```

---

### Task 3: `ToolCatalog` scaffolding + non-coding tool adapters

`ToolCatalog`/`CatalogTool` are generic and live in `nomi-agent-core` (no dependency on the agent
crates). The actual adapters live in `nomi-server`, which already depends on every agent crate —
this avoids a circular `nomi-agent-core → nomi-agent-money` dependency. This task covers every
grantable tool except the coding ones (Task 4 refactors `CodingAgent` first, then adds those).

**Files:**
- Create: `crates/nomi-agent-core/src/tool_catalog.rs`
- Modify: `crates/nomi-agent-core/src/lib.rs` (register the module + re-exports)
- Modify: `crates/nomi-agent-money/src/lib.rs` (extract tool definitions, add `pub` to the two
  free functions)
- Modify: `crates/nomi-agent-personality/src/lib.rs` (same)
- Modify: `crates/nomi-agent-supervisor/src/lib.rs` (same)
- Modify: `crates/nomi-agent-planning/src/lib.rs` (same)
- Create: `crates/nomi-server/src/tool_catalog_adapters.rs` (adapter structs for the 7 non-coding
  tools)
- Modify: `crates/nomi-server/src/lib.rs` (add `mod tool_catalog_adapters;` — `build_tool_catalog`
  itself is finished in Task 4 once coding adapters exist)

**Interfaces:**
- Produces: `nomi_agent_core::{CatalogTool, ToolCatalog}`; `ToolCatalog::new(HashMap<&'static str,
  (ToolDefinition, Box<dyn CatalogTool>)>)`, `.known_tool_names() -> Vec<&'static str>`,
  `.definitions_for(&[String]) -> Vec<ToolDefinition>`, `.execute(name, conn, session_id,
  agent_session_id, user_id, input) -> Result<ToolOutcome, String>`, `ToolCatalog::empty()` (test
  convenience — no entries).
  `nomi_agent_money::{list_transactions_tool_definition, list_transactions, summarize_budget_tool_definition, summarize_budget}`
  (now `pub`), same pattern for personality's 3 tools, supervisor's 1, planning's `create_project`.
- Consumes: `nomi_llm::ToolDefinition`, `nomi_agent_core::ToolOutcome`.

- [ ] **Step 1: Write `tool_catalog.rs`**

```rust
use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;

/// One grantable tool's execution behavior — an adapter over an existing free function that
/// already lives in one of the built-in agent crates. Implementors ignore whichever of
/// `session_id`/`agent_session_id`/`input` the underlying function doesn't need.
#[async_trait]
pub trait CatalogTool: Send + Sync {
    async fn execute(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        input: Value,
    ) -> Result<ToolOutcome, String>;
}

/// A fixed, Rust-reviewed catalog of tools a `DynamicAgent` can be granted — never arbitrary
/// code, never admin-configurable beyond picking from this set. Built once at startup (see
/// `nomi_server::build_tool_catalog`) and shared behind an `Arc` everywhere it's needed.
pub struct ToolCatalog {
    entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)>,
}

impl ToolCatalog {
    pub fn new(entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)>) -> Self {
        Self { entries }
    }

    /// No entries — for tests that need a `&ToolCatalog`/`Arc<ToolCatalog>` but never exercise
    /// dynamic-agent tool execution.
    pub fn empty() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Every registered tool name — admin CRUD write-time validation rejects any
    /// `granted_tools` entry not in this list before it's ever saved.
    pub fn known_tool_names(&self) -> Vec<&'static str> {
        self.entries.keys().copied().collect()
    }

    /// Tolerates unknown names by skipping them (unlike write-time validation, which rejects
    /// them) — covers a tool being retired from the Rust catalog after an agent was already
    /// granted it: that agent should degrade gracefully, not break. See the design spec's
    /// "Write-time validation, runtime tolerance" note.
    pub fn definitions_for(&self, granted: &[String]) -> Vec<ToolDefinition> {
        granted.iter().filter_map(|name| self.entries.get(name.as_str())).map(|(def, _)| def.clone()).collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        match self.entries.get(name) {
            Some((_, tool)) => tool.execute(conn, session_id, agent_session_id, user_id, input).await,
            None => Err(format!("tool not granted: {name}")),
        }
    }
}
```

- [ ] **Step 2: Register the module in `nomi-agent-core/src/lib.rs`**

Add only the `tool_catalog` module line and its re-export in this task — `dynamic_agent` doesn't
exist yet (Task 5 creates it and adds its own `pub mod`/`pub use` lines then):

```rust
pub mod content_block;
pub mod delegation;
pub mod engine;
pub mod error;
pub mod memory;
pub mod permissions;
pub mod personality;
pub mod prompts;
pub mod registry;
pub mod subagent;
pub mod tool_catalog;

pub use content_block::{ApprovalStatus, ContentBlock, TableColumn, TableVariant, TodoItem, TodoStatus, ToolOutcome};
pub use engine::{
    resolve_tool_batch, run_agent_turn, LoopOutcome, ToolBatchOutcome, COMPLETE_TASK_TOOL_NAME,
    DELEGATE_TOOL_NAME, SHOW_TABLE_TOOL_NAME, UPDATE_TODOS_TOOL_NAME, WRITE_PLAN_TOOL_NAME,
};
pub use error::TurnError;
pub use registry::AgentRegistry;
pub use subagent::SubAgent;
pub use tool_catalog::{CatalogTool, ToolCatalog};
```

- [ ] **Step 3: Extract tool definitions + add `pub` in `nomi-agent-money`**

```rust
// nomi-agent-money/src/lib.rs
pub fn list_transactions_tool_definition() -> ToolDefinition {
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
    }
}

pub fn summarize_budget_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "summarize_budget".to_string(),
        description: "Summarize the user's spending totals grouped by category, optionally since a given date.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "since": {"type": "string", "description": "ISO 8601 date; omit for all-time"}
            }
        }),
    }
}
```

Replace `MoneyAgent::tools()`'s body with:

```rust
fn tools(&self) -> Vec<ToolDefinition> {
    vec![list_transactions_tool_definition(), summarize_budget_tool_definition()]
}
```

Change `async fn list_transactions(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input:
Value) -> Result<String, String>` to `pub async fn list_transactions(...)` (add `pub`, no other
change), same for `summarize_budget`.

- [ ] **Step 4: Same pattern for `nomi-agent-personality`**

```rust
pub fn set_personality_tool_definition() -> ToolDefinition {
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
    }
}

pub fn list_personality_versions_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_personality_versions".to_string(),
        description: "List your past personality descriptions, most recent first, so you can decide what to roll back to.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn rollback_personality_tool_definition() -> ToolDefinition {
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
    }
}
```

Replace `PersonalityAgent::tools()`'s body with:

```rust
fn tools(&self) -> Vec<ToolDefinition> {
    vec![
        set_personality_tool_definition(),
        list_personality_versions_tool_definition(),
        rollback_personality_tool_definition(),
    ]
}
```

Add `pub` to `set_personality`, `list_personality_versions`, `rollback_personality` (no other
change to their bodies or signatures).

- [ ] **Step 5: Same pattern for `nomi-agent-supervisor`**

```rust
pub fn list_recent_agent_activity_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_recent_agent_activity".to_string(),
        description: "List recent and currently active background delegations for this chat session.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}
```

Replace `SupervisorAgent::tools()`'s body with `vec![list_recent_agent_activity_tool_definition()]`.
Add `pub` to `list_recent_agent_activity`.

- [ ] **Step 6: Same pattern for `nomi-agent-planning`'s `create_project`**

```rust
pub fn create_project_tool_definition() -> ToolDefinition {
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
    }
}
```

Replace `PlanningAgent::tools()`'s body with `vec![create_project_tool_definition()]`. Add `pub`
to `create_project` (signature stays `pub async fn create_project(conn: &mut
PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, input: Value) -> Result<String,
String>`).

- [ ] **Step 7: Write the non-coding adapters in `nomi-server`**

```rust
// crates/nomi-server/src/tool_catalog_adapters.rs
use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{CatalogTool, ToolOutcome};
use nomi_llm::ToolDefinition;

struct ListTransactions;
#[async_trait]
impl CatalogTool for ListTransactions {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_money::list_transactions(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct SummarizeBudget;
#[async_trait]
impl CatalogTool for SummarizeBudget {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_money::summarize_budget(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct CreateProject;
#[async_trait]
impl CatalogTool for CreateProject {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_planning::create_project(conn, session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct SetPersonality;
#[async_trait]
impl CatalogTool for SetPersonality {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::set_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListPersonalityVersions;
#[async_trait]
impl CatalogTool for ListPersonalityVersions {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::list_personality_versions(conn, user_id).await.map(ToolOutcome::text)
    }
}

struct RollbackPersonality;
#[async_trait]
impl CatalogTool for RollbackPersonality {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::rollback_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListRecentAgentActivity;
#[async_trait]
impl CatalogTool for ListRecentAgentActivity {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_supervisor::list_recent_agent_activity(conn, session_id).await.map(ToolOutcome::text)
    }
}

/// Every non-coding entry for `build_tool_catalog` — coding's four entries are added by that
/// function directly (Task 4), since they need a `LocalFsStore` this function doesn't have.
pub fn non_coding_entries() -> HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert("list_transactions", (nomi_agent_money::list_transactions_tool_definition(), Box::new(ListTransactions)));
    entries.insert("summarize_budget", (nomi_agent_money::summarize_budget_tool_definition(), Box::new(SummarizeBudget)));
    entries.insert("create_project", (nomi_agent_planning::create_project_tool_definition(), Box::new(CreateProject)));
    entries.insert("set_personality", (nomi_agent_personality::set_personality_tool_definition(), Box::new(SetPersonality)));
    entries.insert(
        "list_personality_versions",
        (nomi_agent_personality::list_personality_versions_tool_definition(), Box::new(ListPersonalityVersions)),
    );
    entries.insert(
        "rollback_personality",
        (nomi_agent_personality::rollback_personality_tool_definition(), Box::new(RollbackPersonality)),
    );
    entries.insert(
        "list_recent_agent_activity",
        (nomi_agent_supervisor::list_recent_agent_activity_tool_definition(), Box::new(ListRecentAgentActivity)),
    );
    entries
}
```

- [ ] **Step 8: Register the module in `nomi-server/src/lib.rs`**

Add `mod tool_catalog_adapters;` near the top of `crates/nomi-server/src/lib.rs`, alongside the
existing `pub mod` lines (this one stays private — only `build_tool_catalog`, added in Task 4, is
public).

- [ ] **Step 9: Compile and test**

Run: `cargo build -p nomi-agent-core -p nomi-agent-money -p nomi-agent-personality -p
nomi-agent-supervisor -p nomi-agent-planning -p nomi-server`
Expected: `nomi-agent-core` through `nomi-agent-planning` build clean. `nomi-server` will still
fail (nothing calls `tool_catalog_adapters` yet, and it isn't `pub`, so an "unused" warning is
expected, not an error — if it errors, `mod tool_catalog_adapters;` was placed somewhere requiring
it to resolve fully; move it next to the other `mod`/`pub mod` declarations in `lib.rs`).
Run: `cargo test -p nomi-agent-money -p nomi-agent-personality -p nomi-agent-supervisor -p
nomi-agent-planning` — expected: identical pass count to before this task (tool definitions moved,
not changed; `execute_tool` dispatch unchanged).

- [ ] **Step 10: Commit**

```bash
git add crates/nomi-agent-core/src/tool_catalog.rs crates/nomi-agent-core/src/lib.rs \
        crates/nomi-agent-money/src/lib.rs crates/nomi-agent-personality/src/lib.rs \
        crates/nomi-agent-supervisor/src/lib.rs crates/nomi-agent-planning/src/lib.rs \
        crates/nomi-server/src/tool_catalog_adapters.rs crates/nomi-server/src/lib.rs
git commit -m "feat: add ToolCatalog scaffolding and adapters for money/personality/supervisor/planning tools"
```

---

### Task 4: `CodingAgent` free-function refactor + coding catalog adapters + `build_tool_catalog`

**Files:**
- Modify: `crates/nomi-agent-coding/src/lib.rs`
- Modify: `crates/nomi-server/src/tool_catalog_adapters.rs`
- Modify: `crates/nomi-server/src/lib.rs` (finish `build_tool_catalog`)

**Interfaces:**
- Produces: `nomi_agent_coding::{write_file, read_file, delete_file}` as `pub async fn(conn, storage:
  &LocalFsStore, user_id, input)` free functions (were `&self` methods); `write_file_tool_definition`,
  `read_file_tool_definition`, `list_files_tool_definition`, `delete_file_tool_definition`.
  `nomi_server::build_tool_catalog(project_storage: LocalFsStore) -> Arc<nomi_agent_core::ToolCatalog>`.
- Consumes: `nomi_storage::LocalFsStore` (already a dependency of `nomi-agent-coding`).

- [ ] **Step 1: Extract tool definitions in `nomi-agent-coding`**

```rust
pub fn write_file_tool_definition() -> ToolDefinition {
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
    }
}

pub fn read_file_tool_definition() -> ToolDefinition {
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
    }
}

pub fn list_files_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_files".to_string(),
        description: "List every file path currently in the project.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"project_id": {"type": "string"}},
            "required": ["project_id"]
        }),
    }
}

pub fn delete_file_tool_definition() -> ToolDefinition {
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
    }
}
```

Replace `CodingAgent::tools()`'s body with:

```rust
fn tools(&self) -> Vec<ToolDefinition> {
    vec![write_file_tool_definition(), read_file_tool_definition(), list_files_tool_definition(), delete_file_tool_definition()]
}
```

- [ ] **Step 2: Turn `write_file`/`read_file`/`delete_file` into free functions taking `storage`**

Delete the `impl CodingAgent { async fn write_file(&self, ...) ... async fn delete_file(&self, ...) }`
block entirely (the one containing `write_file`, `read_file`, `delete_file` as `&self` methods).
Replace it with three free functions at module scope (same level as the existing `list_files` free
function), each taking `storage: &LocalFsStore` as an explicit new second parameter in place of
`&self`, otherwise byte-identical bodies:

```rust
pub async fn write_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &LocalFsStore,
    user_id: Uuid,
    input: Value,
) -> Result<nomi_agent_core::ToolOutcome, String> {
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
    let previous_content = storage.get_object(&key).await.map_err(|e| e.to_string())?;

    storage.put_object(&key, content, content_type).await.map_err(|e| e.to_string())?;

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

pub async fn read_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &LocalFsStore,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let project_id = parse_project_id(&input)?;
    if !owns_project(conn, project_id, user_id).await? {
        return Err("project not found".to_string());
    }
    let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
    validate_path(path)?;

    match storage.get_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())? {
        Some(content) => Ok(content),
        None => Ok("file not found".to_string()),
    }
}

pub async fn delete_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &LocalFsStore,
    user_id: Uuid,
    input: Value,
) -> Result<nomi_agent_core::ToolOutcome, String> {
    let project_id = parse_project_id(&input)?;
    if !owns_project(conn, project_id, user_id).await? {
        return Err("project not found".to_string());
    }
    let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
    validate_path(path)?;

    storage.delete_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())?;
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

`parse_project_id`, `owns_project`, and the existing free `list_files` function are untouched.

- [ ] **Step 3: Update `CodingAgent::execute_tool` to call the free functions with `&self.storage`**

```rust
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
        "write_file" => write_file(conn, &self.storage, user_id, input).await,
        "read_file" => read_file(conn, &self.storage, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
        "list_files" => list_files(conn, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
        "delete_file" => delete_file(conn, &self.storage, user_id, input).await,
        other => Err(format!("unknown tool: {other}")),
    }
}
```

- [ ] **Step 4: Run `nomi-agent-coding`'s existing tests unchanged**

Run: `cargo test -p nomi-agent-coding`
Expected: same pass count as before this task — `validate_path`/`validate_delegation_task` tests
are untouched (they never called `write_file`/`read_file`/`delete_file`).

- [ ] **Step 5: Add the coding adapters and finish `build_tool_catalog`**

Append to `crates/nomi-server/src/tool_catalog_adapters.rs`:

```rust
use nomi_storage::LocalFsStore;

struct WriteFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for WriteFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::write_file(conn, &self.storage, user_id, input).await
    }
}

struct ReadFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for ReadFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::read_file(conn, &self.storage, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListFiles;
#[async_trait]
impl CatalogTool for ListFiles {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::list_files(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct DeleteFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for DeleteFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::delete_file(conn, &self.storage, user_id, input).await
    }
}

/// Coding's four entries, needing `project_storage` — combined with `non_coding_entries()` by
/// `build_tool_catalog` in `lib.rs`.
pub fn coding_entries(project_storage: LocalFsStore) -> HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert(
        "write_file",
        (nomi_agent_coding::write_file_tool_definition(), Box::new(WriteFile { storage: project_storage.clone() })),
    );
    entries.insert(
        "read_file",
        (nomi_agent_coding::read_file_tool_definition(), Box::new(ReadFile { storage: project_storage.clone() })),
    );
    entries.insert("list_files", (nomi_agent_coding::list_files_tool_definition(), Box::new(ListFiles)));
    entries.insert(
        "delete_file",
        (nomi_agent_coding::delete_file_tool_definition(), Box::new(DeleteFile { storage: project_storage })),
    );
    entries
}
```

Note `list_files`'s definition function (`nomi_agent_coding::list_files_tool_definition`) is the
same one just extracted in Step 1 above; the free function `nomi_agent_coding::list_files` already
existed with no `&self` — no signature change needed for it.

Add to `crates/nomi-server/src/lib.rs`, alongside `build_agent_registry`:

```rust
/// The one place a new grantable tool gets wired into `ToolCatalog`: implement (or reuse) a
/// free function in the owning agent crate, add its `CatalogTool` adapter to
/// `tool_catalog_adapters`, and add one entry here.
pub fn build_tool_catalog(project_storage: nomi_storage::LocalFsStore) -> std::sync::Arc<nomi_agent_core::ToolCatalog> {
    let mut entries = tool_catalog_adapters::non_coding_entries();
    entries.extend(tool_catalog_adapters::coding_entries(project_storage));
    std::sync::Arc::new(nomi_agent_core::ToolCatalog::new(entries))
}
```

Change `mod tool_catalog_adapters;` to `mod tool_catalog_adapters;` (stays private — only
`build_tool_catalog` needs to be `pub`, which it now is).

- [ ] **Step 6: Compile**

Run: `cargo build -p nomi-agent-coding -p nomi-server`
Expected: `nomi-agent-coding` builds clean. `nomi-server` may still fail to build as a whole (Task
6 hasn't wired `catalog` through `nomi-turn` yet) — if the only remaining errors are in
`worker.rs`/`delegation_worker.rs`/routes calling `nomi_turn::process_turn`/`resume_paused_turn`
with the old argument count, that's expected; leave them for Task 6. If there are any errors
inside `tool_catalog_adapters.rs` or `build_tool_catalog` itself, fix those now — this task's
scope ends at those two units compiling standalone.

- [ ] **Step 7: Commit**

```bash
git add crates/nomi-agent-coding/src/lib.rs crates/nomi-server/src/tool_catalog_adapters.rs \
        crates/nomi-server/src/lib.rs
git commit -m "refactor: CodingAgent file tools become free functions; finish build_tool_catalog"
```

---

### Task 5: `DynamicAgent` struct + DB fetch helpers

**Files:**
- Create: `crates/nomi-agent-core/src/dynamic_agent.rs`
- Modify: `crates/nomi-agent-core/src/lib.rs` (add the module + re-exports, per Task 3 Step 2's
  note)
- Test: `crates/nomi-agent-core/tests/dynamic_agent.rs` (new)

**Interfaces:**
- Produces: `nomi_agent_core::{DynamicAgent, DynamicAgentRow}`;
  `dynamic_agent::fetch_active_dynamic_agents(conn) -> Result<Vec<DynamicAgentRow>, sqlx::Error>`;
  `dynamic_agent::find_dynamic_agent_by_id(conn, id: Uuid) -> Result<Option<DynamicAgentRow>,
  sqlx::Error>`; `DynamicAgent::from_row(row: DynamicAgentRow, catalog: Arc<ToolCatalog>) -> Self`.
- Consumes: `nomi_agent_core::{SubAgent, ToolCatalog}` (Tasks 2-3), the `dynamic_agents` table
  (Task 1).

- [ ] **Step 1: Write `dynamic_agent.rs`**

```rust
use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;
use crate::subagent::SubAgent;
use crate::tool_catalog::ToolCatalog;

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicAgentRow {
    pub id: Uuid,
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
}

type DynamicAgentSqlRow = (Uuid, String, String, String, String, Vec<String>, bool, bool, bool);

fn row_from_sql(row: DynamicAgentSqlRow) -> DynamicAgentRow {
    let (id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate) = row;
    DynamicAgentRow { id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate }
}

const DYNAMIC_AGENT_COLUMNS: &str =
    "id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate";

/// Every currently-routable dynamic agent — fetched fresh on every call, no cache. Used by
/// `nomi_turn::routing::classify_intent` to build the combined classification prompt and to
/// resolve a classified label into a `DynamicAgent`.
pub async fn fetch_active_dynamic_agents(conn: &mut PoolConnection<Postgres>) -> Result<Vec<DynamicAgentRow>, sqlx::Error> {
    let rows: Vec<DynamicAgentSqlRow> = sqlx::query_as(&format!(
        "SELECT {DYNAMIC_AGENT_COLUMNS} FROM dynamic_agents WHERE is_active = true"
    ))
    .fetch_all(&mut **conn)
    .await?;
    Ok(rows.into_iter().map(row_from_sql).collect())
}

/// Looks up a dynamic agent by id regardless of `is_active` — used to resume a turn already
/// mid-conversation with a dynamic agent even if it was soft-disabled since the turn started
/// (soft-disable only excludes an agent from *future* classification, not turns already
/// underway; see the design spec's Error Handling section).
pub async fn find_dynamic_agent_by_id(conn: &mut PoolConnection<Postgres>, id: Uuid) -> Result<Option<DynamicAgentRow>, sqlx::Error> {
    let row: Option<DynamicAgentSqlRow> = sqlx::query_as(&format!(
        "SELECT {DYNAMIC_AGENT_COLUMNS} FROM dynamic_agents WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&mut **conn)
    .await?;
    Ok(row.map(row_from_sql))
}

/// An admin-authored agent, built fresh from a `dynamic_agents` row — never cached, never held
/// in `AgentRegistry`. `agent_type()` is the row's id (stringified), which is how
/// `nomi_turn::lib::resolve_agent` recognizes and re-fetches a dynamic agent when resuming a
/// turn or continuing an active `agent_sessions` row.
pub struct DynamicAgent {
    id: Uuid,
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
    pub fn from_row(row: DynamicAgentRow, catalog: Arc<ToolCatalog>) -> Self {
        Self {
            id: row.id,
            system_prompt: row.system_prompt,
            intent_label: row.intent_label,
            intent_description: row.intent_description,
            granted_tools: row.granted_tools,
            supports_todos_flag: row.supports_todos,
            supports_plans_flag: row.supports_plans,
            can_delegate_flag: row.can_delegate,
            catalog,
        }
    }
}

#[async_trait]
impl SubAgent for DynamicAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Owned(self.id.to_string())
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Owned(self.system_prompt.clone())
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        self.catalog.definitions_for(&self.granted_tools)
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        // Belt-and-suspenders beyond `tools()` only offering granted tools to the LLM: the
        // catalog itself doesn't know which agent is calling, so an ungranted-but-catalog-known
        // tool name (hallucinated, or left over from a stale tool list) is rejected here too.
        if !self.granted_tools.iter().any(|t| t == name) {
            return Err(format!("tool '{name}' is not granted to this agent"));
        }
        self.catalog.execute(name, conn, session_id, agent_session_id, user_id, input).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Owned(self.intent_label.clone())
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Owned(self.intent_description.clone())
    }

    // v1 scope cut: both features assume a single, well-known prompt shape tuned per built-in
    // agent — extending them to admin-authored agents is a real design question of its own,
    // deliberately deferred rather than half-implemented. See the design spec's Out of Scope.
    fn uses_memory(&self) -> bool {
        false
    }

    fn uses_personality(&self) -> bool {
        false
    }

    fn can_delegate(&self) -> bool {
        self.can_delegate_flag
    }

    // Matches the built-in specialist agents (planning, coding) that do substantial background
    // work — a dynamic agent's tool calls are worth surfacing to the user by default.
    fn surfaces_activity(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        self.supports_todos_flag
    }

    fn supports_plans(&self) -> bool {
        self.supports_plans_flag
    }

    fn is_default(&self) -> bool {
        false
    }
}
```

- [ ] **Step 2: Register the module in `nomi-agent-core/src/lib.rs`**

```rust
pub mod content_block;
pub mod delegation;
pub mod dynamic_agent;
pub mod engine;
pub mod error;
pub mod memory;
pub mod permissions;
pub mod personality;
pub mod prompts;
pub mod registry;
pub mod subagent;
pub mod tool_catalog;

pub use content_block::{ApprovalStatus, ContentBlock, TableColumn, TableVariant, TodoItem, TodoStatus, ToolOutcome};
pub use dynamic_agent::{DynamicAgent, DynamicAgentRow};
pub use engine::{
    resolve_tool_batch, run_agent_turn, LoopOutcome, ToolBatchOutcome, COMPLETE_TASK_TOOL_NAME,
    DELEGATE_TOOL_NAME, SHOW_TABLE_TOOL_NAME, UPDATE_TODOS_TOOL_NAME, WRITE_PLAN_TOOL_NAME,
};
pub use error::TurnError;
pub use registry::AgentRegistry;
pub use subagent::SubAgent;
pub use tool_catalog::{CatalogTool, ToolCatalog};
```

- [ ] **Step 3: Write the failing tests**

```rust
// crates/nomi-agent-core/tests/dynamic_agent.rs
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{CatalogTool, DynamicAgent, DynamicAgentRow, SubAgent, ToolCatalog, ToolOutcome};
use nomi_llm::ToolDefinition;
use nomi_test_support::test_pool;

struct Echo;
#[async_trait]
impl CatalogTool for Echo {
    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        Ok(ToolOutcome::text(input.to_string()))
    }
}

fn catalog_with_echo() -> Arc<ToolCatalog> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert(
        "echo",
        (ToolDefinition { name: "echo".to_string(), description: "echoes input".to_string(), input_schema: json!({"type": "object"}) }, Box::new(Echo)),
    );
    Arc::new(ToolCatalog::new(entries))
}

fn sample_row() -> DynamicAgentRow {
    DynamicAgentRow {
        id: Uuid::new_v4(),
        name: "Echo Bot".to_string(),
        system_prompt: "You echo things.".to_string(),
        intent_label: "echo_intent".to_string(),
        intent_description: "the user wants something echoed".to_string(),
        granted_tools: vec!["echo".to_string()],
        supports_todos: true,
        supports_plans: false,
        can_delegate: false,
    }
}

#[tokio::test]
async fn constructs_correctly_from_a_row() {
    let row = sample_row();
    let id = row.id;
    let agent = DynamicAgent::from_row(row, catalog_with_echo());

    assert_eq!(agent.agent_type(), id.to_string());
    assert_eq!(agent.intent_label(), "echo_intent");
    assert_eq!(agent.intent_description(), "the user wants something echoed");
    assert_eq!(agent.system_prompt(), "You echo things.");
    assert!(agent.supports_todos());
    assert!(!agent.supports_plans());
    assert!(!agent.can_delegate());
    assert!(!agent.uses_memory());
    assert!(!agent.uses_personality());
}

#[tokio::test]
async fn tools_only_lists_granted_tools() {
    let agent = DynamicAgent::from_row(sample_row(), catalog_with_echo());
    let tools = agent.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
}

#[sqlx::test]
async fn execute_tool_runs_a_granted_tool(pool: sqlx::PgPool) {
    let agent = DynamicAgent::from_row(sample_row(), catalog_with_echo());
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "echo", json!({"x": 1}))
        .await
        .unwrap();
    assert_eq!(result.display_text, json!({"x": 1}).to_string());
}

#[sqlx::test]
async fn execute_tool_rejects_an_ungranted_tool_name(pool: sqlx::PgPool) {
    let mut row = sample_row();
    row.granted_tools = vec![]; // granted nothing, even though the catalog knows "echo"
    let agent = DynamicAgent::from_row(row, catalog_with_echo());
    let mut conn = pool.acquire().await.unwrap();
    let result = agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "echo", json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not granted"));
}

#[sqlx::test]
async fn fetch_active_dynamic_agents_excludes_inactive_rows(pool: sqlx::PgPool) {
    let user_id = nomi_test_support::seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Active', 'p', 'active_label', 'd', '{}', true, $1), ('Inactive', 'p', 'inactive_label', 'd', '{}', false, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let rows = nomi_agent_core::dynamic_agent::fetch_active_dynamic_agents(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].intent_label, "active_label");
}

#[sqlx::test]
async fn find_dynamic_agent_by_id_finds_inactive_rows_too(pool: sqlx::PgPool) {
    let user_id = nomi_test_support::seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Disabled', 'p', 'disabled_label', 'd', '{}', false, $1) RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await
    .unwrap();

    let found = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(&mut conn, id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().intent_label, "disabled_label");
}

#[sqlx::test]
async fn find_dynamic_agent_by_id_returns_none_for_unknown_id(pool: sqlx::PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let found = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(&mut conn, Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}
```

Check `crates/nomi-test-support/src/lib.rs` for the exact name of its user-seeding helper (used
elsewhere in this workspace's `#[sqlx::test]` tests, e.g. `nomi-agent-planning`'s tests) before
using `nomi_test_support::seed_user` verbatim — if the actual helper has a different name or
signature, use that one instead; the fixture's job is just "insert one row into `users` (and
whatever `web_credentials`/`user_profiles` rows a foreign key requires) and return its id."

- [ ] **Step 4: Run to verify they fail**

Run: `cargo test -p nomi-agent-core --test dynamic_agent`
Expected: compile failure (nothing wired up yet is not the case — Steps 1-2 already created the
module) or, if it compiles, every test failing/erroring until the migration from Task 1 has been
applied to the test database.

- [ ] **Step 5: Apply the migration to the test DB and re-run**

Run: `sqlx migrate run` against the test database (or however this workspace's `#[sqlx::test]`
suite provisions its schema — check `crates/nomi-test-support` for a migration-application step
already wired into test setup; if so, nothing extra is needed once Task 1's migration file exists).
Run: `cargo test -p nomi-agent-core --test dynamic_agent`
Expected: all 7 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/nomi-agent-core/src/dynamic_agent.rs crates/nomi-agent-core/src/lib.rs \
        crates/nomi-agent-core/tests/dynamic_agent.rs
git commit -m "feat: add DynamicAgent and dynamic_agents DB fetch helpers"
```

---

### Task 6: Routing integration — classification, active-session continuation, resume

The largest mechanical task: `AgentRegistry`'s Arc-based methods (Task 2) and `ToolCatalog`
(Tasks 3-4) now exist; this task threads a `catalog: &Arc<ToolCatalog>` parameter through
`nomi-turn`'s public entrypoints, adds dynamic-agent fallback resolution, and fixes every call
site across `nomi-turn`, `nomi-server`, and both crates' test suites.

**Files:**
- Modify: `crates/nomi-turn/src/routing.rs`
- Modify: `crates/nomi-turn/src/lib.rs`
- Modify: `crates/nomi-server/src/worker.rs`
- Modify: `crates/nomi-server/src/delegation_worker.rs`
- Modify (mechanical, add one arg per call site): `crates/nomi-turn/tests/turn_handle_inbound_message.rs`,
  `crates/nomi-turn/tests/turn_process.rs`, `crates/nomi-turn/tests/turn_intent_classification.rs`,
  `crates/nomi-turn/tests/turn_subagent_state_machine.rs`, `crates/nomi-turn/tests/turn_resume_paused_turn.rs`,
  `crates/nomi-server/tests/session_ws.rs`
- Test: `crates/nomi-turn/tests/turn_dynamic_agent_routing.rs` (new)

**Interfaces:**
- Produces: `nomi_turn::routing::classify_intent(conn, provider, registry, catalog: &Arc<ToolCatalog>,
  text) -> Arc<dyn SubAgent>` (was `(provider, registry, text) -> &dyn SubAgent`).
  `nomi_turn::{handle_inbound_message, process_turn, resume_paused_turn}` each gain a new
  `catalog: &Arc<nomi_agent_core::ToolCatalog>` parameter, inserted immediately after `registry`.
- Consumes: `nomi_agent_core::{AgentRegistry, DynamicAgent, SubAgent, ToolCatalog,
  dynamic_agent::{fetch_active_dynamic_agents, find_dynamic_agent_by_id}}`.

- [ ] **Step 1: Rewrite `classify_intent` in `routing.rs`**

```rust
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, DynamicAgent, SubAgent, TurnError};
use nomi_agent_core::ToolCatalog;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

// Generous for a single-word reply: some models don't reliably follow "reply with only one
// word" and add a short sentence around the label instead — find_by_intent_label's whole-word
// fallback handles that, but only if the label isn't truncated out of the response first.
const INTENT_CLASSIFICATION_MAX_TOKENS: u32 = 20;

/// Classifies `text` against every built-in agent in `registry` plus every currently-active
/// dynamic agent (fetched fresh from the DB, no cache), returning the matching agent — built-in
/// or dynamic — or the registry's default agent on no match, a parse failure, or an LLM error.
pub async fn classify_intent(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    registry: &AgentRegistry,
    catalog: &Arc<ToolCatalog>,
    text: &str,
) -> Arc<dyn SubAgent> {
    let dynamic_rows = nomi_agent_core::dynamic_agent::fetch_active_dynamic_agents(conn).await.unwrap_or_default();
    let extra_labels: Vec<String> = dynamic_rows.iter().map(|r| r.intent_label.clone()).collect();
    let extra_options: Vec<String> =
        dynamic_rows.iter().map(|r| format!("{}: {}", r.intent_label, r.intent_description)).collect();

    let request = LlmRequest {
        system: Some(registry.classification_prompt_with_extra(&extra_labels, &extra_options)),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }],
        tools: vec![],
        max_tokens: INTENT_CLASSIFICATION_MAX_TOKENS,
        enable_reasoning: false,
    };

    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(_) => return registry.default_agent(),
    };

    let label_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    if let Some(agent) = registry.find_by_intent_label(&label_text) {
        return agent;
    }

    let normalized = label_text.trim().to_lowercase();
    let matched = dynamic_rows
        .iter()
        .find(|r| r.intent_label == normalized)
        .or_else(|| dynamic_rows.iter().find(|r| normalized.split(|c: char| !c.is_alphanumeric()).any(|w| w == r.intent_label)));

    match matched {
        Some(row) => Arc::new(DynamicAgent::from_row(row.clone(), catalog.clone())) as Arc<dyn SubAgent>,
        None => registry.default_agent(),
    }
}
```

Everything below `classify_intent` in this file (`find_active_agent_session`,
`load_active_agent_session_details`, `is_stale`, `mark_expired`, `spawn_agent_session`,
`complete_agent_session`) is unchanged — copy them through verbatim from the current file.

- [ ] **Step 2: Add `resolve_agent` and thread `catalog` through `lib.rs`**

Add near the top of `crates/nomi-turn/src/lib.rs`, after the existing `use` block:

```rust
use std::sync::Arc;
```

Add this private helper (placed after `run_subagent_turn`, before `finish_agent_turn`):

```rust
/// Resolves `agent_type` to a live `Arc<dyn SubAgent>` — a built-in agent from `registry` if
/// one matches, otherwise a `DynamicAgent` freshly built from the `dynamic_agents` table (its
/// row id, stringified, is what a dynamic agent's own `agent_type()` returns — see
/// `DynamicAgent::agent_type`). `None` when neither matches, e.g. a stale `agent_sessions` row
/// left over from a dynamic agent that's since been deleted... there is no delete in v1, so in
/// practice this only happens for a genuinely unrecognized `agent_type` string.
async fn resolve_agent(
    conn: &mut PoolConnection<Postgres>,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    agent_type: &str,
) -> Option<Arc<dyn nomi_agent_core::SubAgent>> {
    if let Some(agent) = registry.find(agent_type) {
        return Some(agent);
    }
    let id: Uuid = agent_type.parse().ok()?;
    let row = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(conn, id).await.ok()??;
    Some(Arc::new(nomi_agent_core::DynamicAgent::from_row(row, catalog.clone())) as Arc<dyn nomi_agent_core::SubAgent>)
}
```

Update `handle_inbound_message`'s signature to add `catalog: &Arc<nomi_agent_core::ToolCatalog>`
right after `registry: &AgentRegistry,`, and pass it through to `run_locked_turn`:

```rust
pub async fn handle_inbound_message(
    pool: &PgPool,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    let result = run_locked_turn(&mut conn, None, s3, provider, embedding_provider, registry, catalog, session_id, sender_channel_identity_id, user_id, text).await;

    match result {
        Ok((reply, message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id })
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
```

Same insertion for `process_turn` (add the param, pass through to `run_locked_turn`):

```rust
#[allow(clippy::too_many_arguments)]
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = run_locked_turn(
        &mut conn,
        Some((mqtt, turn_job_id)),
        s3,
        provider,
        embedding_provider,
        registry,
        catalog,
        session_id,
        sender_channel_identity_id,
        user_id,
        text,
    )
    .await;

    match result {
        Ok((reply, message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            let _ = mqtt
                .publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id, error: err.to_string() })
                .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}
```

Rewrite `run_locked_turn` to accept `catalog` and use `resolve_agent`:

```rust
#[allow(clippy::too_many_arguments)]
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<(String, Option<Uuid>), TurnError> {
    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    enum RoutingOutcome {
        Continue { agent: Arc<dyn nomi_agent_core::SubAgent>, agent_session_id: Uuid },
        NeedsClassification,
    }

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else {
                match resolve_agent(conn, registry, catalog, &details.agent_type).await {
                    // The active session's agent_type isn't registered anymore (e.g. an
                    // agent crate was removed, or a dynamic agent row was somehow deleted) —
                    // fall back to classifying fresh, same as an unrecognized/stale session.
                    Some(agent) => RoutingOutcome::Continue { agent, agent_session_id },
                    None => RoutingOutcome::NeedsClassification,
                }
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue { agent, agent_session_id } => {
            run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, agent.as_ref(), session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => {
            let agent = routing::classify_intent(conn, provider, registry, catalog, text).await;

            if agent.agent_type() == registry.default_agent().agent_type() {
                // The default agent (chitchat) never gets a persistent agent_sessions row —
                // matching today's behavior, where chitchat has no agent_session_id at all.
                // agent_session_id == session_id here purely as a stand-in for logging
                // (see nomi-agent-chitchat's own comment on this at its call site's origin).
                run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, agent.as_ref(), session_id, session_id, user_id).await
            } else {
                let agent_session_id =
                    routing::spawn_agent_session(conn, session_id, sender_channel_identity_id, agent.agent_type().as_ref()).await?;
                run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, agent.as_ref(), session_id, agent_session_id, user_id).await
            }
        }
    }
}
```

`run_subagent_turn` itself needs no signature change (still takes `agent: &dyn
nomi_agent_core::SubAgent`) — leave it exactly as-is, byte-identical to today.

Fix `finish_agent_turn`'s two `agent.agent_type()` uses (sqlx has no `Encode` for `Cow<str>`, and
`complete_agent_session` takes `&str`):

```rust
// in the Reply arm:
.bind(agent.agent_type().as_ref())
// ...
// in the Completed arm:
routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type().as_ref(), &status, &summary).await?;
```

Update `resume_paused_turn` to accept and thread `catalog`:

```rust
#[allow(clippy::too_many_arguments)]
pub async fn resume_paused_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<TurnOutcome, TurnError> {
    let session_id: Uuid = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(pool)
        .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = resume_locked(&mut conn, mqtt, s3, provider, embedding_provider, registry, catalog, session_id, message_id, decision, remember).await;

    match result {
        Ok((reply, resumed_message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id: resumed_message_id })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
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
```

Update `resume_locked` to accept `catalog`, resolve via `resolve_agent`, and pass `agent.as_ref()`
everywhere the old `agent: &dyn SubAgent` local was used:

```rust
#[allow(clippy::too_many_arguments)]
async fn resume_locked(
    conn: &mut PoolConnection<Postgres>,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
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

    let agent = resolve_agent(conn, registry, catalog, &agent_type).await.ok_or(TurnError::ApprovalNoLongerPending)?;

    let tool_use_blocks: Vec<LlmContentBlock> =
        serde_json::from_value(state["tool_use_blocks"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let messages: Vec<LlmMessage> =
        serde_json::from_value(state["messages"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let pending_tool_use_id = state["pending_tool_use_id"].as_str().unwrap_or_default().to_string();
    let mut decided_tool_use_ids: Vec<(String, bool)> =
        serde_json::from_value(state["decided_tool_use_ids"].clone()).unwrap_or_default();

    if remember {
        if let Some(LlmContentBlock::ToolUse { name, input, .. }) =
            tool_use_blocks.iter().find(|b| matches!(b, LlmContentBlock::ToolUse { id, .. } if *id == pending_tool_use_id))
        {
            let path = input.get("path").and_then(|v| v.as_str());
            let rule_decision = if decision == "approve" { "allow" } else { "deny" };
            let _ = nomi_agent_core::permissions::remember_decision(conn, user_id, name, path, rule_decision).await;
        }
    }

    let new_status = if decision == "approve" { "approved" } else { "denied" };
    let _ = sqlx::query(
        "UPDATE messages SET content_blocks = jsonb_set(jsonb_set(content_blocks, '{0,status}', to_jsonb($1::text)), '{0,decided_at}', to_jsonb(now())) WHERE id = $2",
    )
    .bind(new_status)
    .bind(message_id)
    .execute(&mut **conn)
    .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::MessageUpdated { message_id }).await;

    let _ = sqlx::query(
        "UPDATE agent_sessions SET state = state - 'paused_for_approval' - 'pending_approval_message_id' - 'pending_tool_use_id' - 'tool_use_blocks' - 'messages' - 'decided_tool_use_ids' WHERE id = $1",
    )
    .bind(agent_session_id)
    .execute(&mut **conn)
    .await?;

    decided_tool_use_ids.push((pending_tool_use_id.clone(), decision == "approve"));

    let batch_outcome = nomi_agent_core::resolve_tool_batch(
        conn,
        Some((mqtt, Uuid::nil())),
        s3,
        registry,
        agent.as_ref(),
        session_id,
        agent_session_id,
        user_id,
        &tool_use_blocks,
        &messages,
        &decided_tool_use_ids,
    )
    .await?;

    match batch_outcome {
        nomi_agent_core::ToolBatchOutcome::AwaitingApproval { .. } => Ok(("Waiting for another approval.".to_string(), None)),
        nomi_agent_core::ToolBatchOutcome::Completed { status, summary } => {
            finish_agent_turn(conn, session_id, agent_session_id, agent.as_ref(), nomi_agent_core::LoopOutcome::Completed { status, summary }).await
        }
        nomi_agent_core::ToolBatchOutcome::Resolved(tool_results) => {
            let mut full_messages = messages;
            full_messages.push(LlmMessage { role: LlmRole::User, content: tool_results });

            let outcome = nomi_agent_core::run_agent_turn(
                conn, Some((mqtt, Uuid::nil())), s3, provider, embedding_provider, registry, agent.as_ref(), session_id, agent_session_id, user_id,
                full_messages, SUBAGENT_MAX_TOKENS,
            )
            .await?;

            finish_agent_turn(conn, session_id, agent_session_id, agent.as_ref(), outcome).await
        }
    }
}
```

(The tail of this function past `run_agent_turn` — the final `finish_agent_turn` call — was
already visible above; keep whatever exact trailing lines exist in the current file, just
changing every bare `agent` reference used as `&dyn SubAgent` to `agent.as_ref()`.)

- [ ] **Step 3: Fix `nomi-server/src/worker.rs`**

```rust
let registry = crate::build_agent_registry(project_storage.clone());
let catalog = crate::build_tool_catalog(project_storage);
```

(replacing the current single `let registry = crate::build_agent_registry(project_storage);`
line). Then add `&catalog` to both calls in this file:

```rust
let result = nomi_turn::process_turn(
    &pool,
    &mqtt,
    s3.as_ref(),
    provider.as_ref(),
    embedding_provider.as_ref(),
    &registry,
    &catalog,
    claimed.id,
    claimed.session_id,
    claimed.sender_channel_identity_id,
    user_id,
    &claimed.text,
)
.await;
```

and

```rust
let result = nomi_turn::resume_paused_turn(
    &pool, &mqtt, s3.as_ref(), provider.as_ref(), embedding_provider.as_ref(), &registry, &catalog,
    claimed.message_id, &claimed.decision, claimed.remember,
)
.await;
```

- [ ] **Step 4: Fix `nomi-server/src/delegation_worker.rs`**

No `catalog` needed (dynamic agents are never delegation targets in v1) — only the `Arc`-return
fallout from Task 2. Change:

```rust
let Some(agent) = registry.find(&claimed.target_agent_type) else {
```

stays identical (now returns `Option<Arc<dyn SubAgent>>` instead of `Option<&dyn SubAgent>` —
no source change needed at the `let Some(agent) = ... else` line itself). Update the
`run_agent_turn` call to pass `agent.as_ref()` instead of `agent`:

```rust
let outcome = nomi_agent_core::run_agent_turn(
    &mut conn,
    Some((&mqtt, claimed.id)),
    s3.as_ref(),
    provider.as_ref(),
    embedding_provider.as_ref(),
    &registry,
    agent.as_ref(),
    claimed.session_id,
    claimed.session_id,
    claimed.user_id,
    messages,
    DELEGATED_MAX_TOKENS,
)
.await;
```

- [ ] **Step 5: Compile `nomi-turn` and `nomi-server` (excluding tests)**

Run: `cargo build -p nomi-turn -p nomi-server`
Expected: clean build. Fix anything unexpected the same way as Task 2 Step 5 (an `.as_ref()` or
`.clone()` short of matching a signature) — do not change any behavior.

- [ ] **Step 6: Fix test call sites — add `catalog`/`&catalog` argument**

Every test file below constructs `AgentRegistry::new(vec![Box::new(...), ...])` and then calls one
of `handle_inbound_message`/`process_turn`/`resume_paused_turn`/`classify_intent`. For each file,
add one line near each registry construction:

```rust
let catalog: Arc<nomi_agent_core::ToolCatalog> = Arc::new(nomi_agent_core::ToolCatalog::empty());
```

(adding `use std::sync::Arc;` to the file's imports if not already present), then add `&catalog` as
a new argument immediately after `&registry` in every call to `handle_inbound_message`,
`process_turn`, or `resume_paused_turn` in that file. For `turn_intent_classification.rs`
specifically, its direct `routing::classify_intent(provider, &registry, text)` calls become
`routing::classify_intent(&mut conn, provider, &registry, &catalog, text).await` — check that
file's exact current call shape (it may already pass a `conn`; if so, only add `&catalog` in the
right position) and adjust to match the new `classify_intent` signature from Step 1 exactly.

Files to fix, one at a time — after editing each, run its own test binary before moving to the
next, so a mistake is caught immediately rather than compounding:

1. `crates/nomi-turn/tests/turn_handle_inbound_message.rs` (8 registry constructions) — run
   `cargo test -p nomi-turn --test turn_handle_inbound_message`
2. `crates/nomi-turn/tests/turn_process.rs` (2) — run `cargo test -p nomi-turn --test turn_process`
3. `crates/nomi-turn/tests/turn_intent_classification.rs` (1, plus direct `classify_intent` calls
   — check this file's body for how many, likely more than 1 call even with 1 registry
   construction) — run `cargo test -p nomi-turn --test turn_intent_classification`
4. `crates/nomi-turn/tests/turn_subagent_state_machine.rs` (6) — run `cargo test -p nomi-turn
   --test turn_subagent_state_machine`
5. `crates/nomi-turn/tests/turn_resume_paused_turn.rs` (5) — run `cargo test -p nomi-turn --test
   turn_resume_paused_turn`
6. `crates/nomi-server/tests/session_ws.rs` (1) — run `cargo test -p nomi-server --test
   session_ws`

Expected after each: same pass count as before this task — no test's assertions change, only the
extra `catalog` argument threading.

`crates/nomi-turn/tests/turn_delegation` equivalents and `crates/nomi-agent-core/tests/{registry,delegation,engine}.rs`
were already fixed in Task 2 (they only touch `AgentRegistry`/`run_agent_turn`/`resolve_tool_batch`
directly, never `classify_intent`/`process_turn`/`handle_inbound_message`/`resume_paused_turn`, so
they need no `catalog` argument) — if `cargo test --workspace` surfaces anything unexpected there,
it means Task 2 Step 5 left something unfixed; fix it now rather than leaving it for later.

- [ ] **Step 7: Write and run the new dynamic-agent routing test**

```rust
// crates/nomi-turn/tests/turn_dynamic_agent_routing.rs
use std::sync::Arc;

use nomi_agent_core::AgentRegistry;
use nomi_turn::routing::classify_intent;
use nomi_llm::FakeLlmProvider;
use nomi_test_support::seed_user;

mod support;
use support::ChitchatAgent;

#[sqlx::test]
async fn classifies_into_a_dynamic_agent_by_intent_label(pool: sqlx::PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Weather Bot', 'You report the weather.', 'weather', 'the user is asking about the weather', '{}', true, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog = Arc::new(nomi_agent_core::ToolCatalog::empty());
    let provider = FakeLlmProvider::with_text_response("weather");

    let agent = classify_intent(&mut conn, &provider, &registry, &catalog, "what's the weather like?").await;
    assert_eq!(agent.intent_label(), "weather");
}

#[sqlx::test]
async fn a_soft_disabled_dynamic_agent_is_excluded_from_classification(pool: sqlx::PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Disabled Bot', 'p', 'disabled_intent', 'd', '{}', false, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog = Arc::new(nomi_agent_core::ToolCatalog::empty());
    let provider = FakeLlmProvider::with_text_response("disabled_intent");

    let agent = classify_intent(&mut conn, &provider, &registry, &catalog, "anything").await;
    // Falls back to the default agent — the classifier prompt never even offered
    // "disabled_intent" as an option, and the post-classification fallback match against
    // dynamic_rows only searches ACTIVE rows (fetch_active_dynamic_agents), so a model that
    // somehow still emits "disabled_intent" also can't match it.
    assert_eq!(agent.agent_type(), registry.default_agent().agent_type());
}
```

Check `crates/nomi-turn/tests/` for an existing shared `ChitchatAgent` test stub (several existing
test files in this directory already define minimal in-file `SubAgent` test doubles — reuse
whichever pattern `turn_intent_classification.rs` already uses for its own chitchat-equivalent
default agent, rather than introducing a new `mod support` if one doesn't already exist; adjust
these two tests' imports to match). Check `nomi_llm`'s actual fake/test provider name and
constructor (`FakeLlmProvider::with_text_response` is illustrative — use whatever this workspace's
other classification tests, e.g. the existing `turn_intent_classification.rs`, already use to stub
a one-word classifier response).

Run: `cargo test -p nomi-turn --test turn_dynamic_agent_routing`
Expected: both tests pass.

- [ ] **Step 8: Full workspace build and test**

Run: `cargo build --workspace && cargo test --workspace`
Expected: clean build, every test passes, total count equal to (pre-Task-2 count) + 7 (Task 5's
`dynamic_agent.rs` tests) + 2 (this task's routing tests) + any catalog/tool-adapter tests added
along the way in Tasks 3-4 (none were specified as new tests there beyond reusing existing
coverage — if `cargo test --workspace`'s count differs from this expectation in a way that isn't
explained by a test file this plan touched, investigate before moving on).

- [ ] **Step 9: Commit**

```bash
git add crates/nomi-turn/src/routing.rs crates/nomi-turn/src/lib.rs \
        crates/nomi-server/src/worker.rs crates/nomi-server/src/delegation_worker.rs \
        crates/nomi-turn/tests/turn_handle_inbound_message.rs crates/nomi-turn/tests/turn_process.rs \
        crates/nomi-turn/tests/turn_intent_classification.rs crates/nomi-turn/tests/turn_subagent_state_machine.rs \
        crates/nomi-turn/tests/turn_resume_paused_turn.rs crates/nomi-server/tests/session_ws.rs \
        crates/nomi-turn/tests/turn_dynamic_agent_routing.rs
git commit -m "feat: route turns to dynamic agents alongside built-in agents"
```

---

### Task 7: Live status tracking — `update_agent_phase` + 4 call sites + `AgentPhaseChanged`

**Files:**
- Modify: `crates/nomi-realtime/src/lib.rs`
- Modify: `crates/nomi-agent-core/src/engine.rs`
- Test: `crates/nomi-agent-core/tests/engine.rs` (add phase-tracking assertions to existing test
  helpers, or a new dedicated test file — see Step 3)

**Interfaces:**
- Produces: `StreamEnvelope::AgentPhaseChanged { agent_session_id: Uuid, phase: String, detail:
  Option<String> }`. `agent_sessions.current_phase`/`current_phase_detail` are kept live-updated
  through every turn.

- [ ] **Step 1: Add the `StreamEnvelope` variant**

```rust
// crates/nomi-realtime/src/lib.rs
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
    /// An agent's live status changed — thinking, calling a tool, writing a reply, or back to
    /// waiting. `detail` carries the tool name for `calling_tool`, `None` otherwise. Never sent
    /// for the default (chitchat) agent's turns, which have no real `agent_sessions` row.
    AgentPhaseChanged { agent_session_id: Uuid, phase: String, detail: Option<String> },
}
```

- [ ] **Step 2: Add `update_agent_phase` and its 4 call points in `engine.rs`**

Add phase constants near the top, alongside the existing tool-name consts:

```rust
const PHASE_THINKING: &str = "thinking";
const PHASE_CALLING_TOOL: &str = "calling_tool";
const PHASE_WRITING_REPLY: &str = "writing_reply";
const PHASE_WAITING: &str = "waiting";
```

Add the helper, placed near `post_activity_message`:

```rust
/// Best-effort (never fails the turn, matches `post_activity_message`'s convention): updates
/// `agent_sessions.current_phase`/`current_phase_detail` and, when `mqtt` is available, pushes
/// the same change live over `StreamEnvelope::AgentPhaseChanged`. Silently a no-op for the
/// default agent's turns, where `agent_session_id` is the `session_id` sentinel (see
/// `nomi_turn::run_locked_turn`'s call site comment) — there is no real `agent_sessions` row
/// with that id, so the UPDATE affects zero rows and nothing is published.
async fn update_agent_phase(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    phase: &str,
    detail: Option<&str>,
) {
    let result = sqlx::query("UPDATE agent_sessions SET current_phase = $1, current_phase_detail = $2 WHERE id = $3")
        .bind(phase)
        .bind(detail)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await;

    if let (Ok(outcome), Some(publisher)) = (&result, mqtt) {
        if outcome.rows_affected() > 0 {
            let envelope = StreamEnvelope::AgentPhaseChanged {
                agent_session_id,
                phase: phase.to_string(),
                detail: detail.map(|d| d.to_string()),
            };
            let _ = publisher.publish(session_id, &envelope).await;
        }
    }
}
```

Call site 1 — before each LLM call, top of the `for _ in 0..MAX_TOOL_TURNS` loop in
`run_agent_turn`, right before `let request = LlmRequest { ... }`:

```rust
for _ in 0..MAX_TOOL_TURNS {
    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_THINKING, None).await;

    let request = LlmRequest {
```

Call site 2 (writing_reply) and the first "waiting" terminal point — inside the `if
response.stop_reason != StopReason::ToolUse` block:

```rust
if response.stop_reason != StopReason::ToolUse {
    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WRITING_REPLY, None).await;

    let input_tokens = response.input_tokens;
    let output_tokens = response.output_tokens;
    let reply_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    if agent.uses_memory() {
        let last_user_text = messages
            .iter()
            .rev()
            .find(|m| m.role == LlmRole::User)
            .and_then(|m| m.content.iter().find_map(|b| match b {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            }))
            .unwrap_or_default();
        memory::extract_and_store_memory(conn, provider, embedding_provider, user_id, &last_user_text, &reply_text).await;
    }

    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;

    return Ok(LoopOutcome::Reply {
        text: reply_text,
        memory_ids_used: memories.iter().map(|m| m.id).collect(),
        input_tokens,
        output_tokens,
    });
}
```

Call site 3 (calling_tool) — in `resolve_tool_batch`'s execution loop (the second `for block in
tool_use_blocks` loop, after the pre-scan permission loop), as the first statement inside the `if
let ContentBlock::ToolUse { id, name, input, .. } = block` body:

```rust
let mut tool_results = Vec::new();
for block in tool_use_blocks {
    if let ContentBlock::ToolUse { id, name, input, .. } = block {
        update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_CALLING_TOOL, Some(name)).await;

        let (result_text, is_error, rich_block) = if name.as_str() == COMPLETE_TASK_TOOL_NAME {
```

Call site 4a (waiting, on completion) — in the same loop, right before `return
Ok(ToolBatchOutcome::Completed { status, summary });`:

```rust
if name.as_str() == COMPLETE_TASK_TOOL_NAME {
    let status = input.get("status").and_then(|v| v.as_str()).unwrap_or("completed").to_string();
    let summary = input.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;
    return Ok(ToolBatchOutcome::Completed { status, summary });
}
```

Call site 4b (waiting, on approval pause) — in the pre-scan permission loop, right before `return
Ok(ToolBatchOutcome::AwaitingApproval { message_id });`:

```rust
if let Some((publisher, _)) = mqtt {
    let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
}

update_agent_phase(conn, mqtt.map(|(p, _)| p), session_id, agent_session_id, PHASE_WAITING, None).await;

// The FULL batch must survive into the next resume, ...
let state_patch = serde_json::json!({
```

(insert the `update_agent_phase` call between the existing `MessageCreated` publish and the
existing `// The FULL batch must survive...` comment — every line before and after stays exactly
as it is today).

- [ ] **Step 3: Add phase-tracking test coverage**

Add to `crates/nomi-agent-core/tests/engine.rs` (alongside its existing `run_agent_turn` tests,
using whatever `TestAgent`/fake-provider fixtures that file already defines):

```rust
#[sqlx::test]
async fn run_agent_turn_sets_phase_to_thinking_then_waiting_on_a_plain_reply(pool: sqlx::PgPool) {
    let user_id = nomi_test_support::seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let agent_session_id = nomi_test_support::seed_agent_session(&pool, user_id, "test").await; // adjust to this file's existing session-seeding helper

    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    let provider = /* this file's existing fake provider returning a plain text reply */;
    let embedding_provider = /* this file's existing fake embedding provider */;

    nomi_agent_core::run_agent_turn(
        &mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent,
        Uuid::new_v4(), agent_session_id, user_id, vec![/* one user message */], 1024,
    )
    .await
    .unwrap();

    let (phase, detail): (String, Option<String>) =
        sqlx::query_as("SELECT current_phase, current_phase_detail FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(phase, "waiting");
    assert!(detail.is_none());
}
```

Adapt this test's setup calls (`seed_agent_session`, the fake provider/embedding-provider
constructions, `TestAgent`'s exact type) to whatever fixtures `engine.rs`'s existing tests already
use — this file already has multiple `run_agent_turn` tests with a `TestAgent` fixture and a
`registry`; reuse that exact pattern rather than inventing new fixtures. The assertion (`current_phase
== "waiting"` after a completed Reply turn) is the only new thing.

Run: `cargo test -p nomi-agent-core --test engine`
Expected: this new test passes alongside every existing test in the file, unchanged.

- [ ] **Step 4: Full workspace test**

Run: `cargo build --workspace && cargo test --workspace`
Expected: clean build, all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/nomi-realtime/src/lib.rs crates/nomi-agent-core/src/engine.rs \
        crates/nomi-agent-core/tests/engine.rs
git commit -m "feat: track and broadcast live agent phase (thinking/calling_tool/writing_reply/waiting)"
```

---

### Task 8: Admin CRUD API for `dynamic_agents`

**Files:**
- Create: `crates/nomi-server/src/routes/dynamic_agents.rs`
- Modify: `crates/nomi-server/src/routes/mod.rs` (or wherever route modules are declared — check
  how `admin_dashboard`/`llm_models` are declared and mirror it)
- Modify: `crates/nomi-server/src/app.rs`
- Test: `crates/nomi-server/tests/dynamic_agents_routes.rs` (new)

**Interfaces:**
- Produces: `GET /api/admin/dynamic-agents`, `POST /api/admin/dynamic-agents`, `PUT
  /api/admin/dynamic-agents/:id`, `POST /api/admin/dynamic-agents/:id/toggle-active`.

- [ ] **Step 1: Write the route handlers**

```rust
// crates/nomi-server/src/routes/dynamic_agents.rs
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct DynamicAgentResponse {
    pub id: Uuid,
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
    pub is_active: bool,
}

type DynamicAgentRow = (Uuid, String, String, String, String, Vec<String>, bool, bool, bool, bool);

fn to_response(row: DynamicAgentRow) -> DynamicAgentResponse {
    let (id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active) = row;
    DynamicAgentResponse { id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active }
}

const SELECT_COLUMNS: &str =
    "id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active";

pub async fn list_dynamic_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<DynamicAgentResponse>>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<DynamicAgentRow> = sqlx::query_as(&format!("SELECT {SELECT_COLUMNS} FROM dynamic_agents ORDER BY created_at DESC"))
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list dynamic agents");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list dynamic agents")
        })?;

    Ok(Json(rows.into_iter().map(to_response).collect()))
}

#[derive(Deserialize)]
pub struct DynamicAgentRequest {
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
}

fn validate_request(req: &DynamicAgentRequest, catalog: &nomi_agent_core::ToolCatalog) -> Result<(), (StatusCode, &'static str)> {
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required"));
    }
    if req.system_prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "system_prompt is required"));
    }
    if req.intent_label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "intent_label is required"));
    }
    if req.intent_description.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "intent_description is required"));
    }
    let known = catalog.known_tool_names();
    for tool in &req.granted_tools {
        if !known.contains(&tool.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "granted_tools contains an unrecognized tool name"));
        }
    }
    Ok(())
}

pub async fn create_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<DynamicAgentRequest>,
) -> Result<(StatusCode, Json<DynamicAgentResponse>), (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    validate_request(&req, &state.tool_catalog)?;

    let row: DynamicAgentRow = sqlx::query_as(&format!(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {SELECT_COLUMNS}"
    ))
    .bind(&req.name)
    .bind(&req.system_prompt)
    .bind(&req.intent_label)
    .bind(&req.intent_description)
    .bind(&req.granted_tools)
    .bind(req.supports_todos)
    .bind(req.supports_plans)
    .bind(req.can_delegate)
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to create dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create dynamic agent — intent_label may already be in use")
    })?;

    Ok((StatusCode::CREATED, Json(to_response(row))))
}

pub async fn update_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<DynamicAgentRequest>,
) -> Result<Json<DynamicAgentResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    validate_request(&req, &state.tool_catalog)?;

    let row: Option<DynamicAgentRow> = sqlx::query_as(&format!(
        "UPDATE dynamic_agents SET name = $1, system_prompt = $2, intent_label = $3, intent_description = $4, \
         granted_tools = $5, supports_todos = $6, supports_plans = $7, can_delegate = $8, updated_at = now() \
         WHERE id = $9 RETURNING {SELECT_COLUMNS}"
    ))
    .bind(&req.name)
    .bind(&req.system_prompt)
    .bind(&req.intent_label)
    .bind(&req.intent_description)
    .bind(&req.granted_tools)
    .bind(req.supports_todos)
    .bind(req.supports_plans)
    .bind(req.can_delegate)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to update dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to update dynamic agent — intent_label may already be in use")
    })?;

    row.map(|r| Json(to_response(r))).ok_or((StatusCode::NOT_FOUND, "dynamic agent not found"))
}

pub async fn toggle_active_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<DynamicAgentResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let row: Option<DynamicAgentRow> = sqlx::query_as(&format!(
        "UPDATE dynamic_agents SET is_active = NOT is_active, updated_at = now() WHERE id = $1 RETURNING {SELECT_COLUMNS}"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to toggle dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to toggle dynamic agent")
    })?;

    row.map(|r| Json(to_response(r))).ok_or((StatusCode::NOT_FOUND, "dynamic agent not found"))
}
```

- [ ] **Step 2: Add `tool_catalog` to `AppState`**

```rust
// crates/nomi-server/src/app.rs — add this field
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
    pub s3: Option<S3Config>,
    pub project_storage: LocalFsStore,
    pub tool_catalog: std::sync::Arc<nomi_agent_core::ToolCatalog>,
}
```

Update `crates/nomi-server/src/main.rs`'s `AppState { ... }` construction to add
`tool_catalog: nomi_server::build_tool_catalog(project_storage.clone()),` (project_storage is
already cloned multiple times in that constructor for the worker/delegation-worker spawns —
follow the same `.clone()` pattern so the original `project_storage` local is still available for
the `AppState { ..., project_storage, ... }` field itself).

- [ ] **Step 3: Declare the route module and wire routes in `app.rs`**

Add `pub mod dynamic_agents;` (or the module-declaration pattern this crate's `routes` uses —
check `crates/nomi-server/src/routes.rs` or `routes/mod.rs` for how `admin_dashboard`/`llm_models`
are declared and match it exactly).

```rust
// app.rs — add the import
use crate::routes::dynamic_agents as dynamic_agents_routes;
```

```rust
// app.rs — add these routes, near the other /api/admin/* routes
.route(
    "/api/admin/dynamic-agents",
    get(dynamic_agents_routes::list_dynamic_agents).post(dynamic_agents_routes::create_dynamic_agent),
)
.route("/api/admin/dynamic-agents/:id", put(dynamic_agents_routes::update_dynamic_agent))
.route("/api/admin/dynamic-agents/:id/toggle-active", post(dynamic_agents_routes::toggle_active_dynamic_agent))
```

- [ ] **Step 4: Write integration tests**

```rust
// crates/nomi-server/tests/dynamic_agents_routes.rs
// Follow this workspace's existing admin-route integration test pattern exactly — check
// crates/nomi-server/tests/ for the LLM-models admin route test file (per the design spec's
// Testing section: "Admin CRUD routes get integration tests matching the existing LLM-models
// admin route tests' pattern") and mirror its request-building/auth-header/app-setup helpers.

#[sqlx::test]
async fn create_then_list_dynamic_agents(pool: sqlx::PgPool) {
    // build the app/state (mirror the LLM-models test's setup helper), authenticate as an
    // admin user (has admin/system_config/manage permission), POST /api/admin/dynamic-agents
    // with a valid body (granted_tools: [] to avoid depending on any specific catalog tool
    // being registered), assert 201 and the response shape, then GET the list and assert the
    // created row appears.
}

#[sqlx::test]
async fn create_rejects_an_unrecognized_granted_tool(pool: sqlx::PgPool) {
    // POST with granted_tools: ["not_a_real_tool"], assert 400.
}

#[sqlx::test]
async fn non_admin_cannot_list_or_create(pool: sqlx::PgPool) {
    // Authenticate as a user without admin/system_config/manage, assert GET and POST both 403.
}

#[sqlx::test]
async fn toggle_active_flips_is_active(pool: sqlx::PgPool) {
    // Create one (is_active defaults true), POST toggle-active, assert is_active is now false in
    // the response; toggle again, assert true.
}
```

Write out each test body fully (this plan intentionally leaves the exact `apiFetch`-equivalent
Rust test-client setup to mirror whatever the existing LLM-models admin route test file already
does in this crate — read that file first, then write these four tests using the identical
app-building and auth pattern, not a new one).

Run: `cargo test -p nomi-server --test dynamic_agents_routes`
Expected: all 4 tests pass.

- [ ] **Step 5: Full workspace build and test, then commit**

Run: `cargo build --workspace && cargo test --workspace`

```bash
git add crates/nomi-server/src/routes/dynamic_agents.rs crates/nomi-server/src/app.rs \
        crates/nomi-server/src/main.rs crates/nomi-server/tests/dynamic_agents_routes.rs
git commit -m "feat: add admin CRUD API for dynamic_agents"
```

(Also `git add` whatever file declares route modules, per Step 3, if it's a separate file from
`app.rs` in this codebase.)

---

### Task 9: Admin CRUD UI — `/admin/dynamic-agents`

**Files:**
- Create: `frontend/src/routes/admin/(protected)/dynamic-agents/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/dynamic-agents/+page.svelte`
- Modify: `frontend/src/lib/types.ts` (add `DynamicAgent` type)
- Modify: the admin layout/nav (wherever `/admin/agents` and `/admin/settings/llm` are linked —
  find it via `grep -rn "admin/agents" frontend/src/routes/admin` and add a sibling link)

**Interfaces:**
- Produces: an admin page mirroring the LLM-models admin page's list + `BottomSheet` create/edit
  pattern, with a tool-picker checkbox list grouped by source agent.

- [ ] **Step 1: Add the `DynamicAgent` type**

```typescript
// frontend/src/lib/types.ts — add
export interface DynamicAgent {
	id: string;
	name: string;
	system_prompt: string;
	intent_label: string;
	intent_description: string;
	granted_tools: string[];
	supports_todos: boolean;
	supports_plans: boolean;
	can_delegate: boolean;
	is_active: boolean;
}
```

- [ ] **Step 2: Write `+page.server.ts`**

```typescript
import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { DynamicAgent } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/dynamic-agents');
	if (!response.ok) {
		return { agents: [] as DynamicAgent[] };
	}
	const agents = (await response.json()) as DynamicAgent[];
	return { agents };
};

function readForm(data: FormData) {
	const name = data.get('name');
	const systemPrompt = data.get('system_prompt');
	const intentLabel = data.get('intent_label');
	const intentDescription = data.get('intent_description');
	const grantedTools = data.getAll('granted_tools').filter((v): v is string => typeof v === 'string');
	return {
		name: typeof name === 'string' ? name : '',
		systemPrompt: typeof systemPrompt === 'string' ? systemPrompt : '',
		intentLabel: typeof intentLabel === 'string' ? intentLabel : '',
		intentDescription: typeof intentDescription === 'string' ? intentDescription : '',
		grantedTools,
		supportsTodos: data.get('supports_todos') === 'true',
		supportsPlans: data.get('supports_plans') === 'true',
		canDelegate: data.get('can_delegate') === 'true',
	};
}

function toBody(fields: ReturnType<typeof readForm>) {
	return JSON.stringify({
		name: fields.name,
		system_prompt: fields.systemPrompt,
		intent_label: fields.intentLabel,
		intent_description: fields.intentDescription,
		granted_tools: fields.grantedTools,
		supports_todos: fields.supportsTodos,
		supports_plans: fields.supportsPlans,
		can_delegate: fields.canDelegate,
	});
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const fields = readForm(await request.formData());
		if (!fields.name || !fields.systemPrompt || !fields.intentLabel || !fields.intentDescription) {
			return fail(400, { error: 'Name, system prompt, intent label, and intent description are required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/dynamic-agents', { method: 'POST', body: toBody(fields) });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to create agent.' });
		}
		return { success: true };
	},

	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const fields = readForm(data);
		if (typeof id !== 'string' || !fields.name || !fields.systemPrompt || !fields.intentLabel || !fields.intentDescription) {
			return fail(400, { error: 'Name, system prompt, intent label, and intent description are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/dynamic-agents/${id}`, { method: 'PUT', body: toBody(fields) });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update agent.' });
		}
		return { success: true };
	},

	toggleActive: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid agent.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/dynamic-agents/${id}/toggle-active`, { method: 'POST' });
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to toggle agent.' });
		}
		return { success: true };
	},
};
```

- [ ] **Step 3: Write `+page.svelte`**

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';
	import type { DynamicAgent } from '$lib/types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Grouped by source agent, matching the design spec's admin tool-picker requirement.
	const TOOL_GROUPS: { label: string; tools: { name: string; description: string }[] }[] = [
		{
			label: 'Money',
			tools: [
				{ name: 'list_transactions', description: 'List recent transactions' },
				{ name: 'summarize_budget', description: 'Summarize spending by category' },
			],
		},
		{ label: 'Planning', tools: [{ name: 'create_project', description: 'Create a new project' }] },
		{
			label: 'Personality',
			tools: [
				{ name: 'set_personality', description: 'Set personality' },
				{ name: 'list_personality_versions', description: 'List personality history' },
				{ name: 'rollback_personality', description: 'Roll back personality' },
			],
		},
		{ label: 'Supervisor', tools: [{ name: 'list_recent_agent_activity', description: 'List recent agent activity' }] },
		{
			label: 'Coding',
			tools: [
				{ name: 'write_file', description: 'Write a project file' },
				{ name: 'read_file', description: 'Read a project file' },
				{ name: 'list_files', description: 'List project files' },
				{ name: 'delete_file', description: 'Delete a project file' },
			],
		},
	];

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let name = $state('');
	let systemPrompt = $state('');
	let intentLabel = $state('');
	let intentDescription = $state('');
	let grantedTools = $state<string[]>([]);
	let supportsTodos = $state(false);
	let supportsPlans = $state(false);
	let canDelegate = $state(false);

	function openCreate() {
		editingId = null;
		name = '';
		systemPrompt = '';
		intentLabel = '';
		intentDescription = '';
		grantedTools = [];
		supportsTodos = false;
		supportsPlans = false;
		canDelegate = false;
		sheetOpen = true;
	}

	function openEdit(agent: DynamicAgent) {
		editingId = agent.id;
		name = agent.name;
		systemPrompt = agent.system_prompt;
		intentLabel = agent.intent_label;
		intentDescription = agent.intent_description;
		grantedTools = [...agent.granted_tools];
		supportsTodos = agent.supports_todos;
		supportsPlans = agent.supports_plans;
		canDelegate = agent.can_delegate;
		sheetOpen = true;
	}

	function toggleTool(toolName: string, checked: boolean) {
		grantedTools = checked ? [...grantedTools, toolName] : grantedTools.filter((t) => t !== toolName);
	}
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Dynamic agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Agents defined here run through the same engine as built-in agents, with a curated set of tools.
</p>

<div class="mt-6 space-y-3">
	{#each data.agents as agent (agent.id)}
		<Card variant="outlined" class="p-4">
			<div class="flex items-center justify-between">
				<div>
					<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">
						{agent.name}
						{#if !agent.is_active}
							<span
								class="md-label-medium ml-2 rounded-full px-2 py-0.5"
								style="background: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container)"
							>
								Disabled
							</span>
						{/if}
					</p>
					<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
						{agent.intent_label} · {agent.granted_tools.length} tool(s)
					</p>
				</div>
				<div class="flex items-center gap-2">
					<Button type="button" variant="text" onclick={() => openEdit(agent)}>Edit</Button>
					<form method="POST" action="?/toggleActive" use:enhance>
						<input type="hidden" name="id" value={agent.id} />
						<Button type="submit" variant="text">{agent.is_active ? 'Disable' : 'Enable'}</Button>
					</form>
				</div>
			</div>
		</Card>
	{/each}
</div>

<div class="mt-6">
	<Button type="button" variant="outlined" onclick={openCreate}>+ Add agent</Button>
</div>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">
		{editingId ? 'Edit agent' : 'Add agent'}
	</h2>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action={editingId ? '?/update' : '?/create'}
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
				sheetOpen = false;
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		{#if editingId}
			<input type="hidden" name="id" value={editingId} />
		{/if}
		<TextField id="name" name="name" label="Name" bind:value={name} required />
		<TextField id="intent_label" name="intent_label" label="Intent label (one word, unique)" bind:value={intentLabel} required />
		<TextField
			id="intent_description"
			name="intent_description"
			label="Intent description (fed to the classifier)"
			bind:value={intentDescription}
			required
		/>
		<label class="md-body-medium flex flex-col gap-1" style="color: var(--md-sys-color-on-surface)">
			System prompt
			<textarea
				name="system_prompt"
				bind:value={systemPrompt}
				required
				rows="6"
				class="rounded-md border px-3 py-2"
				style="border-color: var(--md-sys-color-outline); background: var(--md-sys-color-surface)"
			></textarea>
		</label>

		<p class="md-label-medium mt-2" style="color: var(--md-sys-color-on-surface-variant)">Granted tools</p>
		{#each TOOL_GROUPS as group (group.label)}
			<p class="md-label-medium mt-1" style="color: var(--md-sys-color-on-surface)">{group.label}</p>
			{#each group.tools as tool (tool.name)}
				<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
					<input
						type="checkbox"
						name="granted_tools"
						value={tool.name}
						checked={grantedTools.includes(tool.name)}
						onchange={(e) => toggleTool(tool.name, (e.target as HTMLInputElement).checked)}
					/>
					{tool.name} — {tool.description}
				</label>
			{/each}
		{/each}

		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={supportsTodos} />
			Supports a live to-do checklist
		</label>
		<input type="hidden" name="supports_todos" value={supportsTodos} />
		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={supportsPlans} />
			Supports writing versioned plans
		</label>
		<input type="hidden" name="supports_plans" value={supportsPlans} />
		<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
			<input type="checkbox" bind:checked={canDelegate} />
			Can delegate to other agents
		</label>
		<input type="hidden" name="can_delegate" value={canDelegate} />

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{editingId ? 'Save' : 'Add agent'}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>Cancel</Button>
		</div>
	</form>
</BottomSheet>
```

- [ ] **Step 4: Add a nav link**

Run `grep -rn "admin/agents" frontend/src/routes/admin/\(protected\)/+layout.svelte` (or wherever
the admin nav lives — check `+layout.svelte` in `admin/(protected)/`) and add a sibling entry for
`/admin/dynamic-agents` next to the existing "Running agents" link, matching that file's existing
link markup exactly (same list-item/icon pattern).

- [ ] **Step 5: `svelte-check` and live verification**

Run: `npm run check` (from `frontend/`) — expected: no new type errors.
Start the dev server, log in as an admin user, navigate to `/admin/dynamic-agents`, create an
agent with a couple of granted tools, edit it, toggle it disabled/enabled, and verify each action
reflects immediately in the list without a full page reload glitch (the `use:enhance` pattern
already used by the LLM-models page). If a live dev server isn't reachable in this environment,
state that explicitly rather than claiming this was verified.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/routes/admin/\(protected\)/dynamic-agents frontend/src/lib/types.ts \
        "frontend/src/routes/admin/(protected)/+layout.svelte"
git commit -m "feat: add admin UI for creating and managing dynamic agents"
```

---

### Task 10: Admin live-status extension — `get_agents` + `/admin/agents` page

**Files:**
- Modify: `crates/nomi-server/src/routes/admin_dashboard.rs`
- Modify: `frontend/src/lib/types.ts` (extend `RunningAgentItem`/whatever the existing
  `AgentsResponse` type is called)
- Modify: `frontend/src/routes/admin/(protected)/agents/+page.svelte`
- Test: extend whatever existing integration test covers `get_agents` (find it via `grep -rln
  "get_agents\|/api/admin/agents" crates/nomi-server/tests`)

**Interfaces:**
- Produces: `RunningAgentItem` gains `current_phase: String`, `current_phase_detail: Option<String>`.

- [ ] **Step 1: Extend `RunningAgentItem` and `get_agents`'s query**

```rust
// admin_dashboard.rs
#[derive(Serialize)]
pub struct RunningAgentItem {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub channel: String,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub current_phase: String,
    pub current_phase_detail: Option<String>,
}
```

```rust
pub async fn get_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<AgentsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<(Uuid, Option<String>, String, String, Uuid, String, DateTime<Utc>, DateTime<Utc>, String, Option<String>)> = sqlx::query_as(
        "SELECT \
             u.id, wc.email, ci.channel, ci.channel_user_id, \
             ags.id, ags.agent_type, ags.started_at, ags.last_activity_at, ags.current_phase, ags.current_phase_detail \
         FROM agent_sessions ags \
         JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
         JOIN users u ON u.id = ci.user_id \
         LEFT JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE ags.status = 'active' \
         ORDER BY u.id, ags.started_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load running agents");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load running agents")
    })?;

    let mut groups: Vec<UserAgentGroup> = Vec::new();
    for (user_id, email, channel, channel_user_id, agent_session_id, agent_type, started_at, last_activity_at, current_phase, current_phase_detail) in rows {
        let agent = RunningAgentItem { agent_session_id, agent_type, channel: channel.clone(), started_at, last_activity_at, current_phase, current_phase_detail };
        match groups.last_mut() {
            Some(group) if group.user_id == user_id => group.agents.push(agent),
            _ => {
                let label = email.unwrap_or_else(|| format!("{channel}:{channel_user_id}"));
                groups.push(UserAgentGroup { user_id, label, agents: vec![agent] });
            }
        }
    }

    Ok(Json(AgentsResponse { users: groups }))
}
```

(`get_dashboard` above it in the same file is untouched.)

- [ ] **Step 2: Extend the frontend type and page**

Find `RunningAgentItem`/`AgentsResponse` in `frontend/src/lib/types.ts` and add the two new
fields to match the backend struct exactly (`current_phase: string; current_phase_detail: string |
null;`).

In `frontend/src/routes/admin/(protected)/agents/+page.svelte`, extend the `Row` type and table:

```typescript
type Row = {
	agent_session_id: string;
	user_label: string;
	agent_type: string;
	channel: string;
	started_at: string;
	last_activity_at: string;
	current_phase: string;
	current_phase_detail: string | null;
};

const rows: Row[] = $derived(
	data.agents.users.flatMap((group) =>
		group.agents.map((agent) => ({
			agent_session_id: agent.agent_session_id,
			user_label: group.label,
			agent_type: agent.agent_type,
			channel: agent.channel,
			started_at: agent.started_at,
			last_activity_at: agent.last_activity_at,
			current_phase: agent.current_phase,
			current_phase_detail: agent.current_phase_detail,
		})),
	),
);

const columns = [
	{ key: 'user_label', label: 'User', sortable: true },
	{ key: 'agent_type', label: 'Agent Type', sortable: true },
	{ key: 'channel', label: 'Channel', sortable: true },
	{ key: 'current_phase', label: 'Status', sortable: true },
	{ key: 'started_at', label: 'Started', sortable: true },
	{ key: 'last_activity_at', label: 'Last Activity', sortable: true },
];

function phaseLabel(row: Row): string {
	if (row.current_phase === 'calling_tool' && row.current_phase_detail) {
		return `calling tool: ${row.current_phase_detail}`;
	}
	return row.current_phase.replace('_', ' ');
}
```

Add a `<td>{phaseLabel(row)}</td>` cell in the existing `{#each sortedRows as row (row.agent_session_id)}`
block, positioned to match the `columns` array's new `current_phase` entry's position (between
`channel` and `started_at`).

- [ ] **Step 3: Extend the existing `get_agents` test**

Find the existing integration test covering `get_agents`/`/api/admin/agents` and add an assertion
that the response includes `current_phase`/`current_phase_detail` for a seeded active
`agent_sessions` row (the migration's `DEFAULT 'waiting'` means an untouched row already has a
phase to assert against — seed one, assert `current_phase == "waiting"` and
`current_phase_detail` is `null`).

Run: `cargo test -p nomi-server` (the specific test file/name found above)
Expected: passes.

- [ ] **Step 4: `svelte-check`, full backend test, commit**

Run: `npm run check` (frontend) and `cargo test --workspace` (backend).

```bash
git add crates/nomi-server/src/routes/admin_dashboard.rs frontend/src/lib/types.ts \
        "frontend/src/routes/admin/(protected)/agents/+page.svelte"
# plus whatever existing test file was extended in Step 3
git commit -m "feat: show live agent phase on the admin running-agents page"
```

---

### Task 11: User-facing live status — endpoint + `AgentPhaseChanged` frontend consumption

**Files:**
- Modify: `crates/nomi-server/src/routes/sessions.rs`
- Modify: `crates/nomi-server/src/app.rs`
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`
- Modify: `frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts` (mirror the
  chat route's change — check this file's current shape first; it already mirrors
  `agentActivity` per the existing pattern grep found)
- Modify: `frontend/src/lib/components/ChatThread.svelte`
- Test: extend whatever existing test covers `list_agent_activity` (same file, same pattern) with
  a new one for the status endpoint

**Interfaces:**
- Produces: `GET /api/sessions/:id/agent-status` → `{ agent_session_id, agent_type, current_phase,
  current_phase_detail }` or `null` if no active agent session for this session.
- Consumes: `authorize_session_access` (existing), `StreamEnvelope::AgentPhaseChanged` (Task 7).

- [ ] **Step 1: Add the `agent-status` handler**

```rust
// sessions.rs — add near list_agent_activity
#[derive(Serialize)]
pub struct AgentStatusResponse {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub current_phase: String,
    pub current_phase_detail: Option<String>,
}

pub async fn get_agent_status(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Option<AgentStatusResponse>>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let row: Option<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, agent_type, current_phase, current_phase_detail FROM agent_sessions \
         WHERE session_id = $1 AND status = 'active' ORDER BY started_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load agent status");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load agent status")
    })?;

    Ok(Json(row.map(|(agent_session_id, agent_type, current_phase, current_phase_detail)| AgentStatusResponse {
        agent_session_id,
        agent_type,
        current_phase,
        current_phase_detail,
    })))
}
```

- [ ] **Step 2: Wire the route**

```rust
// app.rs
.route("/api/sessions/:id/agent-status", get(sessions_routes::get_agent_status))
```

(add next to the existing `/api/sessions/:id/agent-activity` route.)

- [ ] **Step 3: Add the frontend type**

```typescript
// frontend/src/lib/types.ts
export interface AgentStatus {
	agent_session_id: string;
	agent_type: string;
	current_phase: string;
	current_phase_detail: string | null;
}
```

- [ ] **Step 4: Fetch initial status in both session loaders**

In `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`'s `load`, alongside the existing
`agentActivityResponse` fetch:

```typescript
const agentStatusResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-status`);
const agentStatus: AgentStatus | null = agentStatusResponse.ok ? await agentStatusResponse.json() : null;

return { messages, models, personality, agentActivity, agentStatus };
```

(add `AgentStatus` to the file's existing `import type { ... } from '$lib/types';` line). Apply
the identical change to `frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts` —
read its current `load` function first (it already mirrors the chat route's `agentActivity` fetch
per this plan's earlier research) and add the same `agentStatus` fetch in the same shape.

- [ ] **Step 5: Pass `agentStatus` into `ChatThread` and handle it live**

In both `+page.svelte` files (chat and projects/session — mirror whichever prop names the other
one already uses for `agentActivity`), add `agentStatus={data.agentStatus}` to the `<ChatThread
... >` invocation.

In `ChatThread.svelte`, add the prop, local state, and WS handling:

```svelte
<script lang="ts">
	// ... existing imports
	import type { AgentStatus, RenderedMessage } from '$lib/types';

	// ... existing interface AgentActivityItem unchanged

	let {
		sessionId,
		messages,
		agentActivity,
		agentStatus = null,
		sendError = null,
		extraControls,
	}: {
		sessionId: string;
		messages: RenderedMessage[];
		agentActivity: AgentActivityItem[];
		agentStatus?: AgentStatus | null;
		sendError?: string | null;
		extraControls?: Snippet;
	} = $props();

	// ... existing state
	let currentPhase = $state<{ phase: string; detail: string | null } | null>(
		agentStatus ? { phase: agentStatus.current_phase, detail: agentStatus.current_phase_detail } : null,
	);

	function phaseText(phase: string, detail: string | null): string {
		if (phase === 'thinking') return 'Nomi is thinking…';
		if (phase === 'writing_reply') return 'Nomi is writing a reply…';
		if (phase === 'calling_tool') return detail ? `Nomi is using ${detail}…` : 'Nomi is using a tool…';
		return 'Nomi is working…';
	}
</script>
```

Add one branch to the existing `socket.addEventListener('message', ...)` handler, alongside the
existing `MessageCreated`/`MessageUpdated` branch:

```typescript
} else if (envelope.kind === 'AgentPhaseChanged') {
    const phaseEnvelope = envelope as { kind: string; phase?: string; detail?: string | null };
    if (typeof phaseEnvelope.phase === 'string') {
        currentPhase = phaseEnvelope.phase === 'waiting' ? null : { phase: phaseEnvelope.phase, detail: phaseEnvelope.detail ?? null };
    }
}
```

(this requires widening the existing `let envelope: { kind: string; message_id?: string };`
declaration to also allow `phase`/`detail` fields — change it to `let envelope: { kind: string;
message_id?: string; phase?: string; detail?: string | null };`).

Update the "Nomi is working…" indicator to prefer the live phase text when available:

```svelte
{#if isWorking}
	<div class="mt-4 flex justify-start">
		<div
			class="md-body-large flex items-center gap-2 max-w-md px-4 py-2"
			style="background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface-variant); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)"
		>
			<span class="m3-spinner" aria-hidden="true"></span>
			{currentPhase ? phaseText(currentPhase.phase, currentPhase.detail) : 'Nomi is working…'}
		</div>
	</div>
{/if}
```

(replace the existing hardcoded `Nomi is working…` text at that spot — everything else in this
block, and the rest of the file below it, is unchanged.)

- [ ] **Step 6: Extend backend tests and add a frontend check**

Extend the existing `list_agent_activity` integration test file (found in Task 10's research) with
one more test for `get_agent_status`:

```rust
#[sqlx::test]
async fn agent_status_returns_none_when_no_active_agent_session(pool: sqlx::PgPool) {
    // seed a session with no active agent_sessions row, GET agent-status, assert body is `null`.
}

#[sqlx::test]
async fn agent_status_returns_the_active_agent_sessions_phase(pool: sqlx::PgPool) {
    // seed a session with one active agent_sessions row (current_phase defaults to "waiting"),
    // GET agent-status, assert the response matches.
}
```

Run: `cargo test -p nomi-server` (the specific file)
Expected: passes.

Run: `npm run check` (frontend) — expected: no new type errors.

Start the dev server, open a chat session, send a message to a specialist agent (e.g. planning —
"build me a todo app"), and watch the "Nomi is working…" bubble's text change as the turn
progresses (thinking → calling tool: create_project → writing a reply, etc.) instead of staying
static. If a live dev server isn't reachable in this environment, state that explicitly rather
than claiming this was verified.

- [ ] **Step 7: Full workspace build/test and commit**

Run: `cargo build --workspace && cargo test --workspace`

```bash
git add crates/nomi-server/src/routes/sessions.rs crates/nomi-server/src/app.rs \
        frontend/src/lib/types.ts \
        "frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts" \
        "frontend/src/routes/(app)/chat/[sessionId]/+page.svelte" \
        "frontend/src/routes/(app)/projects/session/[sessionId]/+page.server.ts" \
        "frontend/src/routes/(app)/projects/session/[sessionId]/+page.svelte" \
        frontend/src/lib/components/ChatThread.svelte
# plus whatever existing test file was extended in Step 6
git commit -m "feat: surface live agent status to users in the chat view"
```

---

## Post-plan note

`backend/.cargo/config.toml` must never be staged or committed at any point in this plan's
execution — it is intentionally left out of every `git add` above and must stay that way in every
task's commit. If any frontend task triggers `rtk`'s known `pnpm install` side effect (see prior
session history), run `rm -rf node_modules && npm install` from `frontend/` and `git checkout --
pnpm-lock.yaml` to discard the drift before committing.
