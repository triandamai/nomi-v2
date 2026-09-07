# Rich Content Blocks & Tool Approval — Design

## Goal

Today every chat message — a real reply, a tool-call activity note, a "thinking" trace — is one opaque markdown string rendered as `{@html message.content_html}`. This design adds **structured content blocks** so specific tool/action states render as real widgets instead of emoji-prefixed text: a file write shows a diff (or a "created" card), a file delete shows a delete card, a multi-step task shows a live-updating todo list, tabular data shows a real table (plain or side-by-side comparison), and text can carry footnote-style citations. It also adds a **tool approval subsystem** (Claude-Code-style allow/deny rules, with an in-chat approval card for anything not covered by a rule) — the one piece here that isn't just richer rendering, since it requires the turn loop to genuinely pause before a gated tool runs and resume later.

**Eight requested items, mapped to five block kinds + one subsystem** (established during design, not a scope cut — see "Content block schema" below for why):
- File diff, "writing file" → `FileWrite` block (diff is `FileWrite` with `previous_content: Some(..)`, "created" is `None`)
- "Deleting file" → `FileDelete` block
- Code block → already exists (shiki + `enhanceCodeBlocks`, untouched by this design)
- Inline citations → **no new backend mechanism at all** — `marked` (already in use) natively parses standard Markdown reference-style links (`[1]` in text + a trailing `[1]: https://arxiv.org/... "Attention is all you need"` footer), which is exactly the format requested. An agent can write this today; the only work is a client-side presentational pass (style `[n]` as a superscript chip), same pattern as today's `enhanceCodeBlocks` post-process. Not a `ContentBlock` kind — final reply text never flows through `content_blocks` in this design (only tool/action activity messages do; see §3).
- Todo list → `TodoList` block (mutable, upserted in place)
- Datatable, comparison table → one `Table` block with a presentation-only `variant` flag
- Approval card → `ApprovalRequest` block (mutable) + the permission/pause/resume subsystem

**Explicitly out of scope**: a permissions-management settings page (the `tool_permission_rules` table and "always allow/deny" from the approval card are in scope; a dedicated UI to browse/edit/delete saved rules later is not). Error-flavored activity messages stay plain text — no block type for tool failures in v1. Multi-turn negotiation on a denied action (the LLM arguing back) is whatever the LLM naturally does with the denial `ToolResult`; no new UI for that.

## Current State (verified against the code)

- `messages` (`migrations/0004_sessions.sql`): `id, session_id, sender_channel_identity_id, content TEXT NOT NULL, created_at`. No structured/JSON column. `sender_channel_identity_id` NULL ⇒ assistant/system message, non-NULL ⇒ real user message (`sessions.rs` `to_message_item`).
- `agent_events` (`migrations/0006_memory_and_events.sql`) has a `payload JSONB` column populated by `log_tool_call` (`engine.rs`) with `{tool_name, input, result, is_error}` — but it is **write-only**: no route reads it back. The richest structured tool-call data that exists today is invisible to the frontend.
- `agent_sessions.state JSONB` (`migrations/0005_agent_sessions.sql`) exists, defaults to `{}`, and is **currently unused by anything** — this design repurposes it for both the todo-list-message pointer and paused-turn state (see below).
- Every activity message users see today is produced by `describe_tool_activity` (`engine.rs`), a hardcoded `match tool_name { ... }` that builds emoji strings (`"📝 Wrote \`{path}\`"`, `"🗑️ Deleted \`{path}\`"`, etc.) by reaching into the tool's `input`/`result` — a layering issue this design fixes by moving that knowledge into the tool that owns it.
- `post_activity_message` (`engine.rs`) inserts the string as a new `messages` row and publishes `StreamEnvelope::SessionActivity { session_id }` — a bare signal. The frontend (`ChatThread.svelte`) reacts to `SessionActivity`, `TurnCompleted`, and `AgentDelegationUpdated` identically: call `invalidateAll()`, which reruns `+page.server.ts`'s `load()` and refetches + re-renders **the entire message list**. `Delta` events (streamed tokens) are only used as a boolean "something is streaming" flag — the actual token text is discarded client-side.
- `SubAgent::execute_tool` (`nomi-agent-core/src/subagent.rs`) returns `Result<String, String>`.
- Coding agent tools (`nomi-agent-coding/src/lib.rs`): `write_file` (full overwrite, no before-content captured, returns `"wrote {path}"`), `read_file`, `list_files`, `delete_file` (returns `"deleted {path}"`). `validate_path` rejects absolute paths / `..` segments. `project_file_key(project_id, path)` = `"{project_id}/{path}"` in `LocalFsStore`.
- `run_agent_turn` (`engine.rs`): `for _ in 0..MAX_TOOL_TURNS` (= 10), one LLM call per iteration, executes every `ToolUse` block in the response immediately and unconditionally, no gate of any kind. Stops on `stop_reason != ToolUse`, `complete_task`, or hitting the cap (`TurnError::ToolLoopExceeded`). `complete_task`/`delegate_to_agent` are **engine-level tools** — appended to every agent's tool list by `run_agent_turn` itself (not defined per-agent-crate) and dispatched by name in the same match block, a pattern this design reuses for two new engine-level tools.
- No pause/approval/human-in-the-loop mechanism exists anywhere in the backend (confirmed by grep — zero hits). `agent_delegations.status` is only `pending → processing → completed|failed`; the delegation worker's `FOR UPDATE SKIP LOCKED` claim is for avoiding double-claims, not user-input coordination.
- `agent_sessions_one_active_per_speaker`: `UNIQUE (session_id, sender_channel_identity_id) WHERE status = 'active'` — a new plain-text message from the same identity always reuses the existing active agent session row rather than spawning a new one. Relevant to the concurrency note in §4.
- Frontend: `MessageBubble.svelte` renders one `{@html message.content_html}` per message; `RenderedMessage = MessageItem & { content_html: string }`, no per-block structure. `DataTable.svelte` already exists as a headless shell (search/sort/pagination chrome, caller supplies `<tbody>` rows via a snippet) — a solid base for the `Table` block. `CodeEditor.svelte` has no diff capability; `@codemirror/merge` is not an installed dependency.

## Design

### 1. Data model

```sql
-- messages: additive only, nothing removed or renamed
ALTER TABLE messages ADD COLUMN content_blocks JSONB;
```
`content` stays `NOT NULL` and always populated — a plain-text summary (unchanged from today's emoji-string convention) used as a fallback for anything that reads `content` directly (search, memory extraction's `last_user_text`, notifications) and for clients that don't render blocks. When `content_blocks` is non-null, the frontend renders those instead of `content_html`. `content_blocks` is an array (usually length 1) rather than a single object, so a future block combination (e.g. text + trailing table) isn't forced into two message rows.

```sql
CREATE TABLE tool_permission_rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    tool_name    TEXT NOT NULL,
    path_pattern TEXT,              -- glob; NULL matches any input for this tool_name
    decision     TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
```
Looked up by `(user_id, tool_name)`: fetch all rules for that `tool_name`, glob-match `path_pattern` in application code (not a SQL-level match — Postgres has no glob operator), most-specific match wins, no match ⇒ "ask" (an `ApprovalRequest` block is created). Rows are written implicitly via "always allow/deny" on an approval card — no rule-management UI in this design (see Goal's out-of-scope note).

`agent_sessions.state` (already exists, currently `{}` everywhere) becomes a small **merge-patched** state bag — engine code reads the existing object, updates only the key(s) it owns, and writes back, since the two use cases below must be able to coexist on the same row without clobbering each other:
```json
{
  "todo_message_id": "<uuid>",                    // set once update_todos has been called at least once
  "paused_for_approval": true,                     // present only while a turn is suspended
  "pending_approval_message_id": "<uuid>",
  "pending_tool_use_id": "<string>",
  "tool_use_blocks": [ /* verbatim ToolUse content blocks from the paused response */ ],
  "messages": [ /* full LlmMessage history up to and including that response */ ]
}
```

### 2. Content block schema

One `#[serde(tag = "kind", rename_all = "snake_case")]` enum, defined once in `nomi-agent-core` and mirrored as a TS discriminated union in `frontend/src/lib/types.ts`:

```rust
pub enum ContentBlock {
    FileWrite {
        project_id: Uuid,
        path: String,
        content: String,
        previous_content: Option<String>,      // Some => diff view; None => "created" view
    },
    FileDelete {
        project_id: Uuid,
        path: String,
    },
    TodoList {
        items: Vec<TodoItem>,                  // [{id, text, status: pending|in_progress|done}]
    },
    Table {
        variant: TableVariant,                 // Data | Comparison — presentation hint, same shape
        columns: Vec<TableColumn>,             // [{key, label}]
        rows: Vec<serde_json::Value>,
    },
    ApprovalRequest {
        id: Uuid,
        tool_name: String,
        description: String,                   // human-readable, e.g. "Delete src/old.js"
        input: serde_json::Value,               // the real tool call args, for an expandable detail view
        status: ApprovalStatus,                 // Pending | Approved | Denied
        decided_at: Option<DateTime<Utc>>,
    },
}
```
- **Diff is not a separate kind** — `FileWrite` with `previous_content: Some(..)`. The `write_file` tool now reads the file's existing content before overwriting (same storage call `read_file` already uses) and passes it through.
- **Comparison table is not a separate kind** — same `Table` shape (e.g. `columns: [{key:"item"},{key:"price"},{key:"speed"}]`, one row per compared item); `variant` only changes whether the frontend renders a row grid or side-by-side cards.
- **Citations aren't a block at all** — see the Goal section; standard Markdown reference-links already carry this, no schema needed.
- `TodoList` and `ApprovalRequest` are the two **mutable** kinds — their message row is updated in place, never re-inserted. Everything else is written once.

### 3. Tool execution & block creation

`SubAgent::execute_tool` changes from `Result<String, String>` to `Result<ToolOutcome, String>`:
```rust
pub struct ToolOutcome {
    pub display_text: String,           // the plain-text `content` mirror
    pub block: Option<ContentBlock>,    // structured block, when this call warrants one
}
```
Errors stay a plain `String` (v1 keeps failures as today's `⚠️ ...` text-only activity message — no block type for errors). This touches every agent crate's `execute_tool` signature; for money/personality/supervisor it's mechanical (`ToolOutcome { display_text: <unchanged>, block: None }`), no behavior change there.

Block-building logic moves **into the tool that owns the domain data** rather than staying in `engine.rs`'s `describe_tool_activity`, which today reconstructs it by pattern-matching `input`/`result` strings across a crate boundary:
- `write_file` (coding agent): reads existing content first, returns `ToolOutcome { display_text: "📝 Wrote \`{path}\`", block: Some(FileWrite { project_id, path, content, previous_content }) }`.
- `delete_file`: `block: Some(FileDelete { project_id, path })`.
- `read_file`/`list_files` and everything outside the coding agent: `block: None`, `display_text` unchanged from today.

Two new **engine-level** tools, added the same way `complete_task`/`delegate_to_agent` are (appended to an agent's tool list by `run_agent_turn`, dispatched by name in its match block, not routed through `agent.execute_tool`):
- **`show_table(variant, columns, rows)`** — added unconditionally to every agent. Always inserts a fresh message (no upsert; independent tables in one conversation are all independently meaningful).
- **`update_todos(items)`** — added only for agents with a new capability flag `fn supports_todos(&self) -> bool { false }` (same opt-in pattern as `uses_memory`/`can_delegate`); coding and planning agents override it `true`. **Upserts**: if `agent_sessions.state->>'todo_message_id'` is set, `UPDATE`s that message's `content_blocks` in place; otherwise inserts a new message and stores its id into `state` (merge-patched, not overwritten).

### 4. Approval & permission subsystem

Permission check runs right before executing each `ToolUse` block, against `tool_permission_rules(user_id, tool_name[, path_pattern])`:
- **Allow** (explicit rule) → execute immediately, as today.
- **Deny** (explicit rule) → don't execute; synthesize `ToolResult { is_error: true, content: "Denied by your permission rules." }`, continue the loop in-turn — no pause.
- **No rule matches** → pause.

**Pause invariant: nothing in a response executes until every block in it is resolved.** A single assistant response can contain several `ToolUse` blocks; rather than executing some and pausing mid-batch (a real correctness risk if resume ever replayed an already-completed side effect), the engine pre-scans all of a response's tool blocks for permission *before executing any of them*. If any need approval, none run yet — the loop pauses on the first one needing a decision, persisting:
```json
{
  "paused_for_approval": true,
  "pending_approval_message_id": "<uuid>",
  "pending_tool_use_id": "<string>",
  "tool_use_blocks": [ /* every ToolUse block from this response, none executed */ ],
  "messages": [ /* LlmMessage history through this response */ ]
}
```
An `ApprovalRequest` block (`Pending`) is created; `run_agent_turn` returns a new `LoopOutcome::AwaitingApproval` — the caller leaves the turn open (no failure, no completion).

**Resume is a fresh call into `run_agent_turn`, not a special continuation function.** `POST /api/sessions/:id/messages/:messageId/approval` (`{decision: approve|deny, remember: bool}`):
1. Fencing check: `state.paused_for_approval == true AND state.pending_approval_message_id == messageId` — false ⇒ 409 ("no longer pending"). Protects against resuming a stale approval card after the conversation has moved on (see concurrency note below).
2. If `remember`: upsert into `tool_permission_rules`.
3. Flip the message's block `Pending → Approved/Denied` in place → `MessageUpdated`.
4. Re-run permission-check-then-execute over every block in `state.tool_use_blocks`, in order: the just-decided block uses its known outcome (approve ⇒ execute for real now, building its own follow-up activity message exactly as an auto-allowed call would; deny ⇒ synthetic denial result); every other block is re-evaluated against current rules (if one of *those* also needs approval, it just pauses again with a fresh state blob — same mechanism, no special-casing, naturally handles a second gated call in the same batch).
5. Once every block resolves, append the combined `ToolResult`s and call `run_agent_turn` again with the reconstructed history — a normal, non-paused invocation, so `MAX_TOOL_TURNS` gets a fresh 10-iteration budget. Deliberate: approval waits can be minutes to hours, and counting them against a budget sized for LLM-speed back-and-forth doesn't make sense.
6. Runs through the same turn-job/worker mechanism regular inbound messages already use, not inline in the HTTP handler.

**Concurrency**: if the user sends a new plain-text message instead of resolving a pending approval, `agent_sessions_one_active_per_speaker` means it reuses the same active agent session and just runs a normal fresh turn — nothing destructive happened yet, so the stale `ApprovalRequest` card sitting inertly is safe; the fencing check in step 1 stops it from ever being resumed against outdated state. No new `agent_sessions.status` value, no new locking.

### 5. Realtime delivery

`StreamEnvelope` gains `MessageCreated { message_id }` / `MessageUpdated { message_id }`, replacing `SessionActivity` (retired — nothing else depends on it). Every `post_activity_message` call site switches from publishing `SessionActivity` to `MessageCreated`; the two upsert-in-place paths (todo list, approval status) publish `MessageUpdated`.

New endpoint `GET /api/sessions/:id/messages/:messageId` returns one message (with `content_blocks`), so the frontend patches a single row instead of `invalidateAll()`'s full-conversation refetch. `TurnCompleted` already carries a `message_id` — it moves onto the same single-message fetch-and-append path for free consistency. `AgentDelegationUpdated` is untouched. Frontend: `MessageCreated` → fetch, push onto the array end (creation order is monotonic); `MessageUpdated` → fetch, replace the existing entry by `id` (the `{#each messages as message (message.id)}` keying already in place makes this a clean, non-disruptive Svelte update).

### 6. Frontend rendering

New dispatcher `ContentBlockView.svelte`, switching on `block.kind` across five components: `FileWriteBlock` (real diff via a new `@codemirror/merge` dependency when `previous_content` is present, else a plain syntax-highlighted "created" card), `FileDeleteBlock` (path + icon card), `TodoListBlock` (checklist, re-renders on `MessageUpdated`), `TableBlock` (built on the existing `DataTable.svelte` shell; `variant` switches row-grid vs. side-by-side), `ApprovalCard` (pending: description + Approve/Deny + "always allow/deny" checkbox; resolved: static outcome, no buttons).

Separately (not part of the block dispatcher — this applies to ordinary `content_html` rendering, every message, block or not): `renderMarkdown`/`enhanceCodeBlocks`'s client-side post-process gains a small pass that finds Markdown reference-style links (`marked` already resolves `[1]`/`[1]: url "title"` into a plain `<a href>`) and re-styles them as a superscript citation chip, matching the `[1] Attention is all you need - arxiv.org` format from the Goal section.

`MessageBubble.svelte`: `content_blocks` present ⇒ `{#each content_blocks as block}<ContentBlockView {block} />{/each}`; otherwise today's `{@html message.content_html}` unchanged — full backward compatibility for every existing message and every tool that doesn't build a block. The approval card's buttons submit a new `?/resolveApproval` form action (same shape as the existing `selectAdminModel` action) proxying to the new endpoint; the card's visible state flip is driven by the `MessageUpdated` event, consistent with everything else in this design.

## Testing

- **Backend**: unit tests per block-producing tool (`write_file` diff-vs-create, `delete_file`, `show_table`, `update_todos` upsert-vs-insert) asserting the exact `ContentBlock` shape returned. Engine tests: pre-scan pauses correctly with multiple `ToolUse` blocks and none execute; resume with `approve` executes the gated tool and continues the loop; resume with `deny` never executes it; the fencing check rejects a resume against stale/superseded state; `remember: true` writes a rule that a subsequent call auto-resolves without a new approval block.
- **Frontend**: unit tests for the citation-link re-styling post-process (same shape as existing `renderMarkdown` tests) and the block dispatcher's fallback to `content_html` when `content_blocks` is absent. Live browser verification of each of the five block components rendering correctly, plus the approve/deny round trip end-to-end (click Approve → card flips to Approved via `MessageUpdated` → gated action's own follow-up message appears).
