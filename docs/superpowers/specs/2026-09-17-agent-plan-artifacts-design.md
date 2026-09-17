# Agent Plan Artifacts & Side Sheet — Design

**Goal:** Give any agent a generic, opt-in way to write a durable, versioned plan before executing
a multi-step task, surface that plan as a small indicator on its chat message, and let the user
open a slide-from-right side sheet to read the full plan (and page through earlier versions).

**Architecture:** A new `agent_plans` table holds one row per plan version, scoped to a single
`agent_session_id`. A new engine-level `write_plan` tool (gated behind a `supports_plans()` trait
flag, mirroring `supports_todos()`) inserts a new version on every call and posts a normal chat
message carrying a light `ContentBlock::Plan` reference block — the message's own plain-text
`content` always carries the full plan body, so it stays in the LLM's own context on later turns
with no extra plumbing. A new session-scoped endpoint returns the full version history for one
agent_session; a new `SideSheet.svelte` component (slide-in from the right — new, nothing like it
exists yet) fetches and renders it.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, the existing `nomi-storage` S3 client (`S3Config`, already used for
avatar uploads) for optional plan-content offload.

## Context

Today, only `PlanningAgent` can write a plan, via a `write_plan` tool that saves markdown into a
`projects.plan` column tied to a project row (`create_project` must run first). That column is
never rendered anywhere in the frontend — the plan is invisible to the user once written. This
project replaces that narrow, project-coupled mechanism with a generic one any agent can use,
independent of whether a project exists at all, with the plan actually visible and revisitable in
chat.

The motivating use case: when a task is too big for an agent to execute in one turn, it should be
able to write a plan first, then work from that plan across subsequent tool calls — the same way a
human breaks down a large task before starting on it.

## Data Model

```sql
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

- Exactly one of `content`/`content_s3_key` is ever set (enforced by the `CHECK` constraint) —
  never both, never neither.
- `agent_session_id` is **not** a foreign key. This matches the existing precedent in this
  codebase (`agent_events.agent_session_id` is similarly unconstrained) — the sentinel
  `agent_session_id == session_id` convention used by the default/chitchat agent (which has no
  real `agent_sessions` row) means a strict FK would reject that case. Chitchat has no tools and
  will never call `write_plan` in practice (same reasoning already applied to the approval
  subsystem in the rich-content-blocks project), so this is a non-issue in practice, just a schema
  constraint that would be actively wrong to add.
- No `project_id` — plans are fully decoupled from projects. `create_project`/`projects.plan`
  remain as they are today for the project-workspace flow; the `write_plan` tool on
  `PlanningAgent` is removed and replaced by this generic mechanism (see below).
- No `org_id` column — like `messages` and `agent_sessions`, isolation is enforced through
  `session_id` → `sessions.org_id`, checked at the API layer (see Isolation, below), matching this
  codebase's existing convention rather than introducing per-table org columns.
- `version` starts at 1 per `agent_session_id` and increments by 1 on each `write_plan` call for
  that same agent_session — computed as `COALESCE(MAX(version), 0) + 1` in the same statement that
  inserts the row (see Engine Tool, below), not tracked anywhere else.

## Backend: `S3Config` Threading

`write_plan` needs access to the same `S3Config` `main.rs` already builds via
`nomi_storage::build_from_env()` for avatar uploads (`Option<S3Config>` — `None` when `S3_BUCKET`
isn't set). Today, `run_agent_turn`/`resolve_tool_batch` accept `mqtt: Option<(&MqttPublisher,
Uuid)>` and `registry: &AgentRegistry` as shared dependencies threaded through every call site;
`S3Config` access needs the same treatment: a new `s3: Option<&S3Config>` parameter added to both
functions, following the exact same optional-shared-resource convention already established for
`mqtt`.

This touches every call site: `run_agent_turn`'s own recursive loop, `resolve_tool_batch`'s two
call sites (inside `run_agent_turn`, and inside `resume_locked`), `run_subagent_turn`,
`resume_paused_turn`, `worker.rs`'s two claim loops, `delegation_worker.rs`, and every existing
test in `nomi-agent-core/tests/engine.rs` and `nomi-turn/tests/*.rs` that constructs one of these
calls directly. Mechanical but wide — the same shape of change as Task 3's `ToolOutcome`
migration in the rich-content-blocks project, not a new pattern.

## Backend: `supports_plans()` and the `write_plan` Tool

```rust
// SubAgent trait — new default method, alongside supports_todos()
fn supports_plans(&self) -> bool {
    false
}
```

Enabled initially on `PlanningAgent` (replacing its old project-scoped `write_plan`) and
`CodingAgent` (already has `supports_todos()`; most likely to hit "task too big for one turn").
Other agents opt in later by flipping the flag, same as `supports_todos()`.

```rust
// Tool definition — registered in run_agent_turn alongside show_table/update_todos, only when
// agent.supports_plans() is true
ToolDefinition {
    name: "write_plan",
    description: "Write a new version of your plan for this task, before or during a multi-step \
                   build. Call this again whenever the plan changes significantly — each call is \
                   a new, separately viewable version, not an edit to the last one.",
    input_schema: json!({
        "type": "object",
        "properties": {
            "title": {"type": "string", "description": "Short label for this plan version"},
            "content": {"type": "string", "description": "The plan, in markdown"}
        },
        "required": ["title", "content"]
    }),
}
```

Dispatch logic (new branch in the `resolve_tool_batch` chain, alongside `SHOW_TABLE_TOOL_NAME`/
`UPDATE_TODOS_TOOL_NAME`):

1. Compute the next version: `SELECT COALESCE(MAX(version), 0) + 1 FROM agent_plans WHERE
   agent_session_id = $1`.
2. If `s3` is `Some`, attempt `s3.put_object(&format!("plans/{agent_session_id}/{version}.md"),
   content, "text/markdown")` — keyed by `agent_session_id`/`version`, both already known at this
   point, rather than the row's own `id` (which only exists after the insert below, and would
   otherwise force an awkward pre-generated-UUID insert just to name the S3 object). On success,
   insert the row with `content_s3_key` set and `content` NULL. On failure (or when `s3` is
   `None`), insert the row with `content` set inline and `content_s3_key` NULL — plan storage
   never fails just because S3 is unavailable or erroring.
3. Post a new message (via `post_activity_message`, same as `show_table`) whose plain-text
   `content` is the **full plan body** (not a summary — this is what keeps the plan in the LLM's
   own context on later turns) prefixed with a short glyph line (e.g. `📋 {title}\n\n{content}`),
   and whose `content_blocks` carries `ContentBlock::Plan { plan_id, title, version }`.
4. Publish `MessageCreated` (reusing the Task 7 realtime mechanism — no new event kind needed).

```rust
// New ContentBlock variant, alongside FileWrite/FileDelete/TodoList/Table/ApprovalRequest
Plan {
    plan_id: Uuid,
    title: String,
    version: i32,
},
```

Deliberately light — unlike `Table`/`FileWrite`, the block never carries plan content itself. The
side sheet always fetches from the version-history endpoint below, whether it was opened from the
version-1 bubble or the version-5 bubble; there is exactly one fetch path, not two.

## API: Version History Endpoint

```
GET /api/sessions/:sessionId/agent-plans/:agentSessionId
```

- `authorize_session_access(pool, claims.sub, session_id)` first — the same helper already used by
  `get_message`/`resolve_approval` in the rich-content-blocks project, already proven to correctly
  scope access by org membership.
- Query filters by **both** `session_id` and `agent_session_id` (not `agent_session_id` alone) —
  defense in depth against a guessed/cross-session `agentSessionId`, matching how `get_message`
  guards against a cross-session `messageId`.
- For each row, resolve `content`: return it directly if set, otherwise fetch it from S3 via
  `content_s3_key`. If the S3 fetch fails (object missing, S3 misconfigured after the fact),
  return that version with an explicit `content: null` and let the frontend show an inline "could
  not load this version" state rather than failing the whole list.
- Response: an array of `{ id, title, version, content: string | null, created_at }`, ordered by
  `version ASC`.

## Frontend

**`PlanBlock.svelte`** (new, alongside `TodoListBlock`/`TableBlock`/`ApprovalCard` in
`frontend/src/lib/components/blocks/`) — renders as a small clickable chip in the message bubble:
`📋 {title} · v{version}`. Clicking sets `sideSheetOpen = true` with the block's `plan_id` (or
just `agent_session_id`, since the endpoint is keyed by that) passed to `SideSheet`.

**`SideSheet.svelte`** (new, `frontend/src/lib/components/m3/`) — a slide-in-from-the-right MD3
sheet. `BottomSheet.svelte` is the closest existing structural reference (native `<dialog>`,
`showModal()`/`close()`, backdrop click to dismiss, drag-to-dismiss) but re-oriented: it slides
in from the right edge rather than up from the bottom, and drag-to-dismiss (if kept) would track
horizontal drag instead of vertical. Content: a small version switcher (tabs or a chip row, one
per version) and the active version's markdown body, rendered the same way message content is
rendered elsewhere (reuse whatever markdown-rendering path `MessageBubble`/`FileWriteBlock`
already use, rather than inventing a second one).

`ContentBlockView.svelte`'s dispatch gains one more branch: `{:else if block.kind === 'plan'}
<PlanBlock {block} agentSessionId={message.agent_session_id} />` — **note:** `MessageItem`/
`RenderedMessage` does not currently carry `agent_session_id` at all (it's not needed by any
existing block type). This is a small, additive extension to the message-fetching SQL/type
(`to_message_item`, `list_messages`, `get_message`) needed specifically for `PlanBlock` to know
which agent_session's history to fetch — everything else about those query paths stays as today.

## Isolation

Every layer of this feature enforces org isolation the same way the rest of the codebase already
does, deliberately not introducing a new pattern:

- **DB:** no `org_id` column on `agent_plans` — scoped through `session_id`, like `messages`.
- **API:** `authorize_session_access` (org-membership check) plus a `session_id` match in the
  query itself.
- **Storage:** S3 keys are opaque (`plans/{agent_session_id}/{version}.md`, no org prefix) — matching
  `project_file_key`'s existing convention of relying on DB-layer authorization rather than
  key-namespacing, since the bucket is never exposed to users directly.

## Error Handling

- S3 write failure at `write_plan` time → fall back to inline Postgres storage, plan is never
  lost.
- S3 read failure at fetch time → that one version returns `content: null`, the rest of the list
  is unaffected.
- `write_plan` called with empty `content` → rejected the same way `show_table`'s malformed input
  is today (an error string, `is_error: true`, no block, no message posted with a block — the LLM
  sees the rejection and can retry).

## Testing

Same approach as the rich-content-blocks project: `#[sqlx::test]` integration tests for the new
tool (version increments correctly per agent_session, S3-configured vs. not-configured paths, the
fallback-on-S3-failure path) and the new endpoint (returns ordered versions, 404/403 on
cross-session access, resolves both inline and S3-backed content correctly). No component test
harness exists for `.svelte` files in this repo — `SideSheet.svelte`/`PlanBlock.svelte` are
verified via `svelte-check` plus live browser verification, same standing exception already
established.

## Out of Scope

- **Retrofitting `FileWrite`'s inline content storage to the same S3-optional pattern.** The
  review that surfaced this idea was specifically about `agent_plans`; `FileWrite`'s content
  already lives inline in `content_blocks` JSONB with no size limit, which has the same underlying
  concern, but changing already-shipped block semantics is out of scope for this spec. Worth a
  fast-follow once this pattern is proven.
- **The dynamic/DB-defined agent framework and live agent-status UI** raised alongside this
  request is a separate, larger project — sequenced to be brainstormed fresh after this spec's
  plan is written, per the explicit decision made during this design.
- **Other agents opting into `supports_plans()`** beyond `PlanningAgent`/`CodingAgent` — trivial
  one-line addition later, not blocking this spec.
