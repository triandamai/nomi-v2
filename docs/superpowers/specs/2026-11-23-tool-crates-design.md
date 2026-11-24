# Tool crates — design

Date: 2026-11-23
Scope: project 1 of 3 (tools → crates). Project 2 (worker → `nomi-worker`) and project 3
(custom agents with skills.md / soul.md) get their own designs after this lands.

## Intent

Every tool an agent can call should live in a crate of its own area and run through the one
`Tool` trait, so that:

- tools are found, granted, switched and executed the same way whichever agent calls them;
- custom agents (`dynamic_agents`) can be granted the generic tools (web, records, reminders,
  background tasks…), not only the area tools that are in `ToolCatalog` today;
- `nomi-agent-core` stops being the home of tool bodies and keeps only the turn loop;
- the tool crates build in parallel and don't pull in all of core.

## Today

- `Tool` (`nomi-agent-core/src/tool_catalog.rs`) has `definition`, `group`, `keywords`,
  `execute(conn, session_id, agent_session_id, user_id, input)`. `ToolCatalog` collects the
  catalog tools; `nomi-server/src/lib.rs:96` assembles it from each agent crate's
  `catalog_tools()`.
- Area tools implementing `Tool`: money, personality, planning, supervisor, coding.
  Reminders, workspace and health have their own `*Tool` structs.
- Engine tools are hand-dispatched in `engine.rs`: offered in two lists (~513 and ~698), skipped
  from approval in two lists (~565 and ~1657), executed in an `if/else` chain (~1500–1610).
  They need more than the trait passes today: the agent's type (records, reminders,
  background), the MQTT publisher (background), and they can return a content block
  (background card, voice).

## Goals

1. A small `nomi-tool` crate holding the trait and what it needs; nothing else.
2. One `nomi-tools-<area>` crate per area, holding that area's tool definitions, the data
   functions behind them, and the `Tool` impls.
3. The engine runs every non-loop tool through the catalog: no per-tool names in `engine.rs`'s
   offer, approval or execute paths.
4. Behaviour unchanged: same tool names, schemas, descriptions, outputs, approval behaviour,
   per-agent gating and admin switches.

## Non-goals

- Changing any tool's schema, prompt text or result text.
- The worker move and custom-agent skills (projects 2 and 3).
- Reworking `ToolCatalog` search / `find_tools` ranking.

## Anti-goals (failure even if it compiles)

- A tool crate depending on `nomi-agent-core` (it would defeat the split and invite cycles).
- A tool that ran without an approval card now asking for one, or the reverse.
- An agent offered a tool it isn't offered today (e.g. records for an agent with
  `uses_records() == false`).

## Approaches considered

- **A. Trait crate + area crates, engine tools stay in core.** Real split, mostly mechanical.
- **B. Trait stays in core, only move area tools.** Smallest; no build gain, core can't use the
  crates.
- **C. A, plus the generic engine tools move to crates and run through the catalog.**
  Cleanest result; custom agents can be granted generic tools; touches `engine.rs`.

**Chosen: C** (user's pick, 2026-11-23).

## Design

### `nomi-tool` (new)

Depends only on `nomi-llm` (for `ToolDefinition`), `async-trait`, `serde`, `serde_json`,
`uuid`, `chrono`, `sqlx`, and `nomi-realtime` (for the publisher handle).

Moves here from core: `Tool`, `ToolOutcome` (and the content-block types it carries),
`ToolCatalog`, `ToolInfo`, `tool_definitions`, `find_tool`. Core re-exports them so existing
`nomi_agent_core::Tool` paths keep compiling during the move; the re-exports are removed once
nothing uses them.

The trait's `execute` takes a context instead of loose ids:

```rust
pub struct ToolContext<'a> {
    pub session_id: Uuid,
    pub agent_session_id: Uuid,
    pub user_id: Uuid,
    pub agent_type: &'a str,
    pub mqtt: Option<&'a MqttPublisher>,
}

async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value)
    -> Result<ToolOutcome, String>;
```

New trait methods, with defaults matching today's area tools:

- `fn approval(&self) -> Approval` — `Approval::Checked` (default: goes through the permission
  levels / approval card as area tools do now) or `Approval::Never` (runs straight away, as the
  engine tools do now).
- `fn readiness(&self, conn) -> bool` (async, default `true`) — for tools offered only when
  configured (web search provider set, voice ready).

### Area crates

| Crate | From | Tools |
|---|---|---|
| `nomi-tools-money` | `nomi-agent-money` | list/summarize/log transactions, budgets, month start |
| `nomi-tools-personality` | `nomi-agent-personality` | set/reset/list/rollback personality |
| `nomi-tools-planning` | `nomi-agent-planning` | create_project |
| `nomi-tools-supervisor` | `nomi-agent-supervisor` | recent activity, stop agents |
| `nomi-tools-coding` | `nomi-agent-coding` | coding tools |
| `nomi-tools-reminders` | `nomi-agent-reminders` (stage 1) + core `scheduled_jobs` reminder tools (stage 2) | reminder agent tools; create/list/cancel_reminder |
| `nomi-tools-workspace` | `nomi-agent-workspace` | workspace tools (moved onto `Tool`) |
| `nomi-tools-health` | `nomi-agent-health` | health tools (moved onto `Tool`) |
| `nomi-tools-web` | core `web.rs` | web_search, read_web_page, link previews |
| `nomi-tools-records` | core `records.rs` | save/list/update/delete_record |
| `nomi-tools-background` | core `background.rs` | run/list/cancel_background_task |
| `nomi-tools-attachments` | core `attachments.rs` (tool part) | read_attachment |
| `nomi-tools-voice` | core `voice.rs` (tool part) | speak |
| `nomi-tools-publish` | core `project_ops.rs` (tool part) | publish_project |

Each exposes `pub fn tools() -> Vec<Box<dyn Tool>>`. The `nomi-agent-<area>` crate keeps the
`SubAgent` impl and its prompt, and depends on its tools crate. The data functions (e.g.
`money::list_transactions`) move with the tools; other callers (server routes, worker) import
them from the tools crate.

A tool crate may depend on `nomi-tool`, `nomi-llm`, infrastructure crates (`nomi-storage`,
`nomi-settings`, `nomi-usage`, `nomi-embedding`, `nomi-attachments`, `nomi-realtime`) — never on
`nomi-agent-core` or another `nomi-agent-*` crate. Where an engine tool body calls into core
today, the needed piece moves down into the tool crate or an infra crate; if a piece can't move
without dragging the loop with it, that tool stays in core (see "Left to the builder").

### What stays in core

Loop-control tools that change the turn's own state: `complete_task`, `delegate_to_agent`,
`rename_chat`, `update_todos`, `read_plan`/`write_plan`, `ask_user`, `find_tools`,
`create_agent`, `ask_agent`, `post_crew_note`, `show`. They keep their hand dispatch.

### Engine changes

- `ToolCatalog` is assembled once (in `nomi-server`, as today) from every tools crate, and
  passed to the engine via the registry as today.
- Offering: the two hand lists are replaced by one function that, per agent, takes the agent's
  own tools plus the generic ones it is gated for. Gating stays on `SubAgent`
  (`uses_records`, `supports_reminders`, …) and is expressed as a per-group check
  (`group == "records"` needs `uses_records()`), then `ToolSwitches::allows` and `readiness`.
- Approval: the name lists are replaced by `tool.approval() == Approval::Never`.
- Execution: the generic branches of the `if/else` chain are replaced by one catalog lookup;
  `ToolOutcome.block` replaces the special-cased card/voice blocks, and `posted_text` is taken
  from the outcome the same way the background card does today.

### Offer-time context *(builder's call, settled in design)*

Some tools depend on the conversation: `create_reminder`'s schema lists the agents a reminder
can target, and `read_attachment` is offered only when the chat mentions files. The trait gets
plain data at offer time:

```rust
pub struct OfferContext<'a> {
    pub reminder_targets: &'a [String],
    pub mentions_files: bool,
}

/// The definition to offer, or `None` to hide the tool this turn. Default: `Some(definition())`.
fn offer(&self, ctx: &OfferContext<'_>) -> Option<ToolDefinition>;
```

The engine builds `OfferContext` from the registry and messages; it's plain data, so there's
no registry ↔ catalog cycle.

### Other per-tool name lists *(design call)*

`engine.rs` has further name lists beyond offer/approval/execute: `should_post` (~1640) skips
activity posts for reminder, background, web, record and `find_tools` calls, and the
personality MQTT publish keys off `PERSONALITY_TOOL_NAMES`. These become trait properties:

- `fn posts_activity(&self) -> bool` (default `true`; `false` for the tools in today's skip
  list);
- the personality publish keys off `group() == "personality"`.

No per-tool name lists remain in `engine.rs` for tools that moved out.

### Catalog and `find_tools` *(design call)*

Generic tools join `ToolCatalog` (needed for grants, the admin picker and the
`tool_catalog_entries` sync). `find_tools` filters out tools the calling agent isn't eligible
for (same gating as the offer), so search can never be a path around anti-goal 3.

### Custom agents *(user's decision, 2026-11-23)*

**Automatic + grants.** A custom agent keeps every generic tool it gets automatically today
(web, records, background, speak, read_attachment — gated exactly as now), and its
`granted_tools` *add* to that set: an explicit grant (e.g. `create_reminder`) is offered even
when the agent's flag (`supports_reminders() == false`) wouldn't offer it automatically. No
existing custom agent loses a tool; no migration needed. Built-in agents have no grants, so
their offers are unchanged.

`DynamicAgent::execute_tool` already goes through the catalog; its "is granted" check widens
to "is granted or automatically offered". The admin/crew tool picker reads the catalog and
lists the generic tools with no extra work.

## Testing

- Existing unit and integration tests move with their code and keep passing unchanged.
- New: a test in `nomi-server` that lists every tool name offered to each built-in agent before
  and after (snapshot), and every tool's `approval()` — so anti-goals 2 and 3 are checked.
- New: a test that no `nomi-tools-*` crate's `Cargo.toml` depends on `nomi-agent-core` or any
  `nomi-agent-*` crate.
- `cargo build --workspace`, `cargo test --workspace` (Postgres-backed tests as today).

## Delivery

On a branch from `main`, as one PR per stage (no stacking; each based on `main` after the
previous merges):

1. `nomi-tool` crate + `ToolContext` + area crates (money, personality, planning, supervisor,
   coding, reminders, workspace, health).
2. Engine tools (including core's reminder tools) → crates + catalog-driven
   offer/approval/execute/activity in `engine.rs`; custom-agent grants widened.

Each PR is behaviour-identical on its own; the offered-tools snapshot is captured on `main`
before stage 1 and must match after each stage, except for the intended custom-agent grant
change in stage 2.

The trait (`ToolContext`, `OfferContext`, `offer`, `approval`, `posts_activity`) is frozen in
stage 1 so stage 2 adds no signature changes.

## Refinements made while planning

- Auto-offer is a trait property (`auto_offer() -> AutoOffer { GrantOnly, Everyone, IfUsesRecords,
  IfSupportsReminders }`), not a check on `group()`: Kala's area tools and the engine's
  `create_reminder` share the reminders area but are offered differently.
- Planning and coding tools stay in their agent crates for now (they depend on core's
  `relay_ops`, `project_ops`, `sandbox_ops`, `plans`, `reply`); `publish_project` stays in core
  (Koda's own tool, not engine-dispatched). Supervisor moves in stage 2, after background.

## Left to the builder

- Exact module layout inside each crate.
- Whether `readiness` needs `conn` or a settings handle.
- If an engine tool body is too entangled with the loop to move cleanly, keep it in core and say
  so in the PR — don't drag loop types into a tool crate.
