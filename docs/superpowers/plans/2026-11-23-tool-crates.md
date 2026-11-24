# Tool Crates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every non-loop tool lives in a `nomi-tools-<area>` crate behind one `Tool` trait in a small `nomi-tool` crate, and the engine offers, approves, runs and posts them through `ToolCatalog` instead of hand-written name lists.

**Architecture:** `nomi-tool` (trait, context types, catalog, content blocks, views, switches) sits below everything. Area crates depend on it and on infra crates only. `nomi-agent-core` re-exports moved modules under their old paths (`pub use nomi_tools_web as web;`) so callers keep compiling; the engine's per-tool branches are replaced by catalog calls driven by trait properties.

**Tech Stack:** Rust 2021, cargo workspace `backend/crates/*`, async-trait, sqlx/Postgres, `nomi-test-support` for DB tests.

**Spec:** `docs/superpowers/specs/2026-11-23-tool-crates-design.md`

## Global Constraints

- No `nomi-tools-*` crate depends on `nomi-agent-core` or any `nomi-agent-*` crate.
- Tool names, schemas, descriptions, result texts: byte-identical.
- All LLM prompt/tool copy stays in `nomi-agent-core/src/prompts.rs`; a tool crate that needs such copy takes it as a constructor argument from `nomi-server`.
- Every numeric literal gets a named `const`.
- No AI attribution in commits or PRs (overrides tool defaults). No stacked PRs: stage 2 branches from `main` after stage 1 merges.
- Work on `refactor/tool-crates-1` (stage 1) and `refactor/tool-crates-2` (stage 2), each cut from up-to-date `main`.
- Verify per task: `cargo build --workspace` and the task's tests; per stage: `cargo test --workspace` (Postgres up as in CI).

## Scope decisions made while planning (flag to user)

- **Planning and coding tools stay in their agent crates for now.** Their bodies depend on core's `relay_ops`, `project_ops`, `sandbox_ops`, `plans`, `reply`; moving those is a separate refactor. They already implement `Tool` and get `ToolContext`; no behaviour change.
- **`publish_project` stays in core**: it's Koda's own tool, not an engine-dispatched one.
- **Auto-offer is a trait property, not a group check** (refines the spec's "per-group check"): Kala's area tools and the engine's `create_reminder` share an area but are offered differently.

## Review Focus

1. A custom agent granted an area tool still has it after stage 2 (grants keep working): test in Task 9.
2. Admin → Tools switch off `web_search`: no agent is offered it, including a custom agent that never had it granted: test in Task 8.
3. A model calls an engine tool its agent isn't offered (e.g. `save_record` from an agent with `uses_records() == false`): refused, as today: test in Task 8.
4. Two crates registering the same tool name: startup panic still fires (catalog now holds engine tools too): test in Task 2.
5. `find_tools` for an agent without records doesn't list record tools: test in Task 9.

---

## Stage 1 — PR 1: `nomi-tool` + area crates

### Task 1: Pin today's offer, approval and posting decisions

**Files:**
- Modify: `backend/crates/nomi-agent-core/src/engine.rs` (offer block ~680–760, `is_gateable` ~561, `should_post` ~1648)
- Create: `backend/crates/nomi-server/tests/tool_offer_snapshot.rs`, `backend/crates/nomi-server/tests/snapshots/tool_offer.json`

**Interfaces:**
- Produces (in `engine.rs`, `pub`):
  - `pub struct OfferInputs { pub off: HashSet<String>, pub switches: ToolSwitches, pub web_search_ready: bool, pub voice_ready: bool, pub mentions_files: bool }`
  - `pub fn engine_tools_for(agent: &dyn SubAgent, registry: &AgentRegistry, inputs: &OfferInputs) -> Vec<ToolDefinition>` — everything the offer block adds after the agent's own/chat tools, before the chat-project defaulting, same order.
  - `pub fn is_quiet(name: &str) -> bool` — the name list inside `should_post` (true = don't post for surfacing agents).
- `run_agent_turn_in_reply` calls `engine_tools_for` with inputs it loads exactly as today; pure refactor.

- [ ] **Step 1:** Cut `refactor/tool-crates-1` from up-to-date `main`.
- [ ] **Step 2:** Extract `OfferInputs`, `engine_tools_for`, `is_quiet`; `cargo test -p nomi-agent-core` passes unchanged.
- [ ] **Step 3: Write the snapshot test** `offered_tools_match_snapshot`: build the production registry (as `nomi-server` does), and for each built-in agent × each of 4 inputs (`all_ready` = nothing off, web+voice ready, files mentioned; `nothing_ready`; `web_search_off` via `ToolSwitches::with_off(["web_search"])`; `voice_off`), record `agent.tools()` names + `engine_tools_for` names. Also record, for every name in `crew_tool_definitions` ∪ catalog, `is_gateable(name)` and `is_quiet(name)`; and catalog `(name, group)` pairs. Serialize as sorted JSON; compare to `snapshots/tool_offer.json`. When the env var `UPDATE_SNAPSHOTS=1` is set, write the file instead.
- [ ] **Step 4:** Run `UPDATE_SNAPSHOTS=1 cargo test -p nomi-server --test tool_offer_snapshot`, then without the var: PASS. Inspect the JSON: Dana has the money tools, Nomi has `web_search` only in `all_ready`.
- [ ] **Step 5:** Commit `test: snapshot each agent's offered tools before the tool-crates split`.

### Task 2: `nomi-tool` crate

**Files:**
- Create: `backend/crates/nomi-tool/{Cargo.toml,src/lib.rs,src/catalog.rs,src/context.rs}`
- Move (git mv) from `nomi-agent-core/src/`: `content_block.rs`, `views.rs`, `locale.rs`, `tools.rs` (switches/settings rows), `tool_catalog.rs` → `catalog.rs`; `crew_name`/`display_name_for` from `crew.rs` → `nomi-tool/src/crew_names.rs`.
- Modify: `nomi-agent-core/src/lib.rs` (re-exports), every `impl Tool for` (money, personality, planning, supervisor, coding, reminders, workspace, health, core tests).

**Interfaces:**
- Produces:
  ```rust
  pub struct ToolContext<'a> {
      pub session_id: Uuid, pub agent_session_id: Uuid, pub user_id: Uuid,
      pub agent_type: &'a str, pub agent_name: &'a str,
      pub mqtt: Option<(&'a MqttPublisher, Uuid)>,
  }
  pub struct OfferContext<'a> { pub reminder_targets: &'a [String], pub mentions_files: bool }
  pub enum Approval { Checked, Never }
  pub enum AutoOffer { GrantOnly, Everyone, IfUsesRecords, IfSupportsReminders }

  #[async_trait] pub trait Tool: Send + Sync {
      fn definition(&self) -> ToolDefinition;
      fn group(&self) -> &'static str;
      fn keywords(&self) -> &'static [&'static str] { &[] }
      fn offer(&self, _ctx: &OfferContext<'_>) -> Option<ToolDefinition> { Some(self.definition()) }
      async fn ready(&self, _conn: &mut PgConnection) -> bool { true }
      fn approval(&self) -> Approval { Approval::Checked }
      fn auto_offer(&self) -> AutoOffer { AutoOffer::GrantOnly }
      fn posts_activity(&self) -> bool { true }
      async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String>;
  }
  ```
  `ToolCatalog::execute(&self, name, conn, ctx: &ToolContext<'_>, input)`; new `ToolCatalog::get(&self, name) -> Option<&dyn Tool>` and `ToolCatalog::tools(&self) -> impl Iterator<Item = &dyn Tool>` (sorted by name).
- Core keeps paths: `pub use nomi_tool::{content_block, views, locale, tools, catalog as tool_catalog};` and `crew::{crew_name, display_name_for}` re-exported from `nomi_tool::crew_names`.
- Deps: `nomi-llm`, `nomi-i18n`, `nomi-realtime`, `nomi-settings` (only if `tools.rs` needs it), async-trait, serde, serde_json, uuid, chrono, sqlx.

- [ ] **Step 1: Tests in `nomi-tool/src/lib.rs`:** `default_trait_properties` (a minimal fake tool: `approval() == Checked`, `auto_offer() == GrantOnly`, `posts_activity()`, `offer` returns its definition); `catalog_get_and_tools_sorted`; move the existing catalog tests; keep `register_twice_panics` (`#[should_panic(expected = "registered twice")]`).
- [ ] **Step 2:** Run `cargo test -p nomi-tool`: FAIL (crate missing).
- [ ] **Step 3:** Create the crate, move the modules, add `ToolContext` etc.; switch every `Tool` impl and caller (each `SubAgent::execute_tool` builds a `ToolContext` from its args; `DynamicAgent` and `ToolCatalog::execute` callers likewise).
- [ ] **Step 4:** `cargo build --workspace && cargo test -p nomi-tool -p nomi-agent-core && cargo test -p nomi-server --test tool_offer_snapshot`: PASS, snapshot unchanged.
- [ ] **Step 5:** Commit `refactor: move the Tool trait and its types into nomi-tool`.

### Task 3: Dependency guard + `nomi-tools-money` and `nomi-tools-reminders`

**Files:**
- Create: `backend/crates/nomi-server/tests/tool_crate_deps.rs`; `nomi-tools-money/`, `nomi-tools-reminders/`
- Move: money tool definitions, `Item`/`items_of`/`Logged`, data fns (`list_transactions`, `summarize_budget`, `log_transaction(_outcome)`, `set_budget`, `budgets_with_spending`, `list_budgets`, month start), `period.rs`, `tools.rs` → `nomi-tools-money`. Reminders: `ReminderTool`, `tool_definitions`, `run_tool`, `keywords_for` and the data fns they call → `nomi-tools-reminders`.
- Modify: `nomi-agent-money`, `nomi-agent-reminders` (depend on their tools crate, keep `SubAgent` + prompt), `nomi-server/src/lib.rs::build_tool_catalog`, other callers (grep `nomi_agent_money::`, `nomi_agent_reminders::`).

**Interfaces:**
- Produces: `nomi_tools_money::tools() -> Vec<Box<dyn Tool>>`, `nomi_tools_reminders::tools() -> Vec<Box<dyn Tool>>` (replace `catalog_tools`). Data fns keep names/signatures at the crate root.

- [ ] **Step 1: Write `no_tool_crate_depends_on_agent_crates`:** glob `../nomi-tools-*/Cargo.toml` from `CARGO_MANIFEST_DIR`; assert at least 1 found; parse `[dependencies]` and `[dev-dependencies]` with `toml`; assert no key equals `nomi-agent-core` or starts with `nomi-agent-`.
- [ ] **Step 2:** Run it: FAIL (no crates found).
- [ ] **Step 3:** Create both crates; move code with `git mv` where whole files move; tests move with their code.
- [ ] **Step 4:** `cargo build --workspace && cargo test -p nomi-tools-money -p nomi-tools-reminders -p nomi-agent-money -p nomi-agent-reminders && cargo test -p nomi-server --test tool_crate_deps --test tool_offer_snapshot`: PASS.
- [ ] **Step 5:** Commit `refactor: money and reminder tools into their own crates`.

### Task 4: `nomi-tools-workspace` and `nomi-tools-health`

**Files:** Create `nomi-tools-workspace/`, `nomi-tools-health/`; modify both agent crates and `build_tool_catalog`.

**Interfaces:**
- The HTTP/Google/Hevy client half of `WorkspaceAgent`/`HealthAgent` (fields, `from_env`, `run`, connection/google/hevy/format modules) moves as `WorkspaceClient`/`HealthClient` with the same `run(conn, user_id, name, input) -> Result<ToolOutcome, String>`. `nomi_tools_workspace::tools(&WorkspaceClient)`, `nomi_tools_health::tools(&HealthClient)`.
- The agent crates keep `WorkspaceAgent { client: WorkspaceClient }` / `HealthAgent { client: HealthClient }` implementing `SubAgent`, with `from_env()` delegating.

- [ ] **Step 1:** Moved tests are the tests; add nothing new.
- [ ] **Step 2:** Move; update callers (`nomi-server` routes for connections use the client types).
- [ ] **Step 3:** `cargo build --workspace && cargo test -p nomi-tools-workspace -p nomi-tools-health -p nomi-agent-workspace -p nomi-agent-health && cargo test -p nomi-server --test tool_crate_deps --test tool_offer_snapshot`: PASS.
- [ ] **Step 4:** Commit `refactor: workspace and health tools into their own crates`.

### Task 5: `nomi-tools-personality`

**Files:** Create `nomi-tools-personality/`; move `nomi-agent-core/src/personality.rs` and `nomi-agent-personality/src/tools.rs` + its tool definitions into it; core gets `pub use nomi_tools_personality::store as personality;`.

**Interfaces:**
- Personality store fns return `Result<_, sqlx::Error>` instead of `TurnError`; core callers map with `?` (check `TurnError: From<sqlx::Error>`; add the impl if missing).
- `nomi_tools_personality::tools() -> Vec<Box<dyn Tool>>`.

- [ ] **Step 1:** Moved tests are the tests.
- [ ] **Step 2:** Move; fix callers (`engine.rs`, supervisor, server routes).
- [ ] **Step 3:** `cargo build --workspace && cargo test -p nomi-tools-personality -p nomi-agent-personality -p nomi-agent-core && cargo test -p nomi-server --test tool_crate_deps --test tool_offer_snapshot`: PASS.
- [ ] **Step 4:** Commit `refactor: personality tools and store into nomi-tools-personality`.

### Task 6: Stage 1 verification and PR

- [ ] **Step 1:** `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`: PASS.
- [ ] **Step 2:** `git diff main -- backend/crates/nomi-server/tests/snapshots/tool_offer.json`: only the file's addition.
- [ ] **Step 3:** Push and open PR "refactor: nomi-tool trait crate and per-area tool crates (1/2)" against `main`; body states the planning/coding deferral.

---

## Stage 2 — PR 2: engine tools through the catalog

Start after PR 1 merges: cut `refactor/tool-crates-2` from up-to-date `main`.

### Task 7: Engine tools into crates (records, web, attachments, voice)

**Files:** Create `nomi-tools-records/`, `nomi-tools-web/`, `nomi-tools-attachments/`, `nomi-tools-voice/`. Move `records.rs`, `web.rs`, `voice.rs` whole; from `attachments.rs` only `read_attachment_tool_definition`, `read_attachment`, `attachment_bytes`, `mentions_files`, `has_attachments` (the rest stays). Core: `pub use nomi_tools_records as records;` etc.

**Interfaces (each a `Tool` impl, `approval() == Never`):**

| Tool(s) | `auto_offer` | `posts_activity` | `ready` / `offer` |
|---|---|---|---|
| save/list/update/delete_record | `IfUsesRecords` | false | — (executes with `ctx.agent_type`) |
| web_search | `Everyone` | false | `ready`: `search_setup(conn).is_some()` |
| read_web_page | `Everyone` | false | — |
| read_attachment | `Everyone` | true | `offer`: `None` unless `ctx.mentions_files` |
| speak | `Everyone` | true | `ready`: `voice::is_ready(conn)` |

- `nomi_tools_voice::tools(copy: SpeakCopy)` with `pub struct SpeakCopy { pub description: &'static str, pub done: &'static str }`; `nomi-server` passes `prompts::SPEAK_TOOL_DESCRIPTION` / `SPEAK_TOOL_DONE`. `voice.rs` keeps no `prompts` reference.
- `nomi-server::build_tool_catalog` registers all four crates.

- [ ] **Step 1: Tests** in each crate: `<tool>_properties` asserting the table row above; `read_attachment_hidden_without_files` (`offer` with `mentions_files: false` is `None`).
- [ ] **Step 2:** Run: FAIL (crates missing).
- [ ] **Step 3:** Move and implement. The engine still hand-dispatches these in this task (paths resolve through the re-exports).
- [ ] **Step 4:** `cargo build --workspace && cargo test -p nomi-tools-records -p nomi-tools-web -p nomi-tools-attachments -p nomi-tools-voice && cargo test -p nomi-server --test tool_crate_deps --test tool_offer_snapshot`: PASS (catalog section of the snapshot gains these names. Re-record with `UPDATE_SNAPSHOTS=1` and check the diff is only catalog additions).
- [ ] **Step 5:** Commit `refactor: records, web, attachment and voice tools into their own crates`.

### Task 8: Background, engine reminder tools, supervisor; catalog-driven engine

**Files:**
- Create `nomi-tools-background/` (whole `background.rs`; `crew::display_name_for` now from `nomi_tool::crew_names`), `nomi-tools-supervisor/` (supervisor `tools.rs` + `stop.rs` module from core; `stop.rs` from supervisor crate stays if it is agent logic).
- Move `create/list/cancel_reminder` definitions (from `engine.rs`) and bodies (`scheduled_jobs.rs`) into `nomi-tools-reminders` as `Tool`s: `approval() == Never`, `auto_offer() == IfSupportsReminders`, `posts_activity() == false`, `offer` builds `create_reminder` from `ctx.reminder_targets`.
- Background tools: `approval() == Never`, `auto_offer() == Everyone`, `posts_activity() == false`; `run_background_task` returns `ToolOutcome { display_text, block: Some(card) }` and the engine's `posted_text` comes from `card_text(&block)` as today.
- Modify `engine.rs`:
  - `engine_tools_for` → offers, from `registry.tool_catalog()`, every tool whose `auto_offer` matches the agent (`Everyone`; `IfUsesRecords` && `agent.uses_records()`; `IfSupportsReminders` && `agent.supports_reminders()`), then `switches.allows`, then not in `inputs.not_ready`, then `offer(&OfferContext)`. `OfferInputs` swaps `web_search_ready`/`voice_ready` for `pub not_ready: HashSet<String>` (names whose `ready(conn)` returned false, computed by the engine before the call), so `engine_tools_for` stays sync and DB-free; the snapshot test maps `all_ready` → empty set and `nothing_ready` → `{"web_search", "speak"}`.
  - `is_gateable(name)`: loop-control names as today, else `catalog.get(name).map_or(true, |t| t.approval() == Approval::Checked)`.
  - `is_quiet(name)`: loop-control names as today, else `catalog.get(name).is_some_and(|t| !t.posts_activity())`.
  - Execute chain: generic branches replaced by one branch: tool is offered to this agent (same eligibility as offer) and `approval() == Never` → `catalog.execute(name, conn, &ctx, input)`. An engine tool not offered to this agent → `"unknown tool: {name}"`, error, as today.
  - `PERSONALITY_TOOL_NAMES` → `catalog.get(name).is_some_and(|t| t.group() == nomi_tools_personality::GROUP)`.
  - `crew_tool_definitions` lists catalog tools with `auto_offer != GrantOnly` instead of the hand list.

- [ ] **Step 1: Tests** (core, with a fake registry + catalog): `records_not_offered_without_uses_records`; `records_call_refused_without_uses_records` (result is the unknown-tool error); `web_search_off_offered_to_nobody` (switch off → absent for a built-in and a custom agent); `reminder_targets_reach_create_reminder` (the schema lists the targets passed in).
- [ ] **Step 2:** Run: FAIL.
- [ ] **Step 3:** Implement the moves and engine changes above. `grep -n "_TOOL_NAME" engine.rs` afterwards lists only loop-control names.
- [ ] **Step 4:** `cargo build --workspace && cargo test -p nomi-agent-core && cargo test -p nomi-server --test tool_crate_deps --test tool_offer_snapshot`: PASS **with the stage-1 snapshot for every built-in agent unchanged** (do not re-record).
- [ ] **Step 5:** Commit `refactor: engine offers, approves and runs its tools through the catalog`.

### Task 9: Custom agents: automatic + grants; `find_tools` eligibility

**Files:** `nomi-agent-core/src/dynamic_agent.rs`, `engine.rs` (offer), `tool_finder.rs`; tests in `nomi-agent-core/tests/dynamic_agent.rs`, `tests/tool_finder.rs`.

**Interfaces:**
- `DynamicAgent` offer = automatic engine tools (by `auto_offer`, as any agent) ∪ `granted_tools`; a granted tool is offered even when its `auto_offer` would exclude this agent (e.g. `create_reminder` with `supports_reminders() == false`).
- `DynamicAgent::execute_tool` accepts a name if granted **or** automatically offered.
- `tool_finder::find_tools` drops results the calling agent is neither automatically offered nor granted.

- [ ] **Step 1: Tests:** `granted_area_tool_still_offered` (custom agent granted `list_transactions` is offered and runs it); `granted_create_reminder_offered_without_flag`; `custom_agent_keeps_automatic_web_and_records`; `find_tools_hides_records_from_agent_without_records`.
- [ ] **Step 2:** Run: FAIL on the reminder grant and `find_tools` cases.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test -p nomi-agent-core && cargo test -p nomi-server --test tool_offer_snapshot --test dynamic_agents_routes --test coverage_tool_catalog`: PASS.
- [ ] **Step 5:** Commit `feat: custom agents can be granted the crew's generic tools`.

### Task 10: Stage 2 verification and PR

- [ ] **Step 1:** `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`: PASS.
- [ ] **Step 2:** `git diff main -- backend/crates/nomi-server/tests/snapshots/tool_offer.json`: built-in agents' offers and approvals unchanged; catalog gains engine tools only.
- [ ] **Step 3:** Push; open PR "refactor: engine tools through the catalog; grantable to custom agents (2/2)" against `main`.
