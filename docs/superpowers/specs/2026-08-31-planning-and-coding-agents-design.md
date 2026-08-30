# Planning & Coding Agents — Design

## Goal

A user asks nomi to build something ("build me a todo app"). Chitchat delegates to a new **planning** agent, which produces a plan the user can see. The plan hands off (same delegation mechanism) to a new **coding** agent, which writes real files into a project the user can browse, read, and edit inside nomi's own web app — file tree, code editor, and a live preview for simple (static) projects. Everything the user interacts with is nomi; no other system is named or exposed.

**Explicitly out of scope for this spec** (deferred, per direction from the project owner — "focus on nomi first"):
- **Publishing to Shipyard.** Shipyard (a separate self-hosted PaaS, same owner) can build-and-deploy a `git`-type service and has an API to trigger/poll deploys, but no API to *create* a new project/service yet — that's a Shipyard-side gap to close later. When it's ready, Publish becomes: push the project's files to a git repo, trigger a Shipyard deploy, poll for status, report back in chat. Nothing in this spec's data model blocks that later.
- **Running the generated project** (installing dependencies, dev servers, tests). Phase 1 preview is static-file-only (see §7). Anything needing real execution is a separate, later design — it's the one piece that would need actual sandboxing, and it's deliberately not being taken on here.

## Current State (verified against the code)

### Agent architecture

- Every agent implements `SubAgent` (`nomi-agent-core/src/subagent.rs`): `agent_type()`, `system_prompt()`, `tools() -> Vec<ToolDefinition>`, `execute_tool(conn, session_id, agent_session_id, user_id, name, input) -> Result<String, String>`, `intent_label()`/`intent_description()` (classifier prompt text), plus opt-in flags defaulting to `false`/`true`: `is_default()`, `uses_memory()`, `uses_personality()`, `can_delegate()`, `is_delegation_target()`.
- `ToolDefinition { name: String, description: String, input_schema: serde_json::Value }` (`nomi-llm/src/types.rs:29`) — a raw JSON Schema object, provider-agnostic.
- `AgentRegistry::new` (`nomi-agent-core/src/registry.rs`) panics unless exactly one registered agent has `is_default() == true`. `build_agent_registry()` (`nomi-server/src/lib.rs`) is the single wiring point — currently `ChitchatAgent` (default), `MoneyAgent`, `PersonalityAgent`, `SupervisorAgent`. Adding an agent is: implement `SubAgent` in a new crate, add it as a dependency, add one line here.
- `run_agent_turn` (`nomi-agent-core/src/engine.rs`) drives up to `MAX_TOOL_TURNS = 10` LLM round-trips. Every agent automatically gets a `complete_task` tool (short-circuits the loop with `LoopOutcome::Completed { status, summary }`). If `agent.can_delegate()`, it also gets `delegate_to_agent` (target enum built from `registry.delegatable_agent_types(excluding: self)`).
- **Delegation is already fully built** (`2026-08-29-multi-agent-supervisor-design.md`): calling `delegate_to_agent` inserts a row into `agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result, error, created_at, claimed_at, completed_at)` and returns immediately — the delegating agent's own turn continues (it doesn't block). A background worker (`nomi-server/src/delegation_worker.rs`) claims pending rows (`FOR UPDATE SKIP LOCKED`, `LISTEN`/`NOTIFY` + 5s poll fallback), runs the target agent through the *same* `run_agent_turn` loop with a single-message conversation (`[User: task]`), phrases the raw result in the supervisor agent's voice (`nomi_agent_supervisor::phrase_delegation_result`), and inserts it as a new `messages` row in the original session — which the open WebSocket picks up via MQTT, same as any other reply.
- The `task` field on `agent_delegations` is **plain text** — there is no structured "context" column threaded from requester to target today. `MoneyAgent`/`PersonalityAgent` don't need one; this design does (see §3).
- `nomi-agent-money` (`crates/nomi-agent-money/src/lib.rs`) is the cleanest minimal example of a tool-bearing, non-delegating, non-memory agent to model the new agents' crate shape on.

### Storage & frontend patterns worth reusing

- `nomi-server/src/s3.rs` already wires up an optional, gracefully-degrading S3 client (`AppState.s3: Option<S3Config>`, `None` when `S3_BUCKET` is unset — the app boots and runs fine without it). Today it exposes exactly one operation, `presign_put(key, content_type) -> PresignedUpload { upload_url, public_url }`, used only for avatar images: the browser uploads bytes *directly* to S3 via a presigned URL, so the Rust backend never touches the image bytes. That shape fits a browser-originated binary upload; it doesn't fit this feature, where the *writer* is either the coding agent (server-side Rust, already holding the text content mid-tool-call) or a user editing text in the browser and POSTing it to nomi's own API like every other write in this app. Both need direct, server-side `PutObject`/`GetObject`/`DeleteObject` calls — see §2.
- SvelteKit route-driven `BottomSheet` (`frontend/src/lib/components/m3/BottomSheet.svelte`) and the MD3 component set (`Card`, `List`/`ListItem`, `Button`, `TextField`, `IconButton`, `Menu`/`MenuItem`) are the established UI vocabulary — no new design system, no external component library.
- The app's routing under `(app)` is one route per feature area (`chat/[sessionId]`, `profile`, `preferences`, `account`), each with its own `+page.server.ts` doing `apiFetch` calls against `nomi-server`. A new `projects` area follows the same shape.

## Design

### 1. New crates: `nomi-agent-planning`, `nomi-agent-coding`

Both follow the `nomi-agent-money` shape exactly: one `lib.rs`, a `SubAgent` impl, plain-`match` `execute_tool`.

```rust
// nomi-agent-planning
pub const PLANNING_AGENT_TYPE: &str = "planning";
// intent_label/description: routes messages like "build me an app", "I want to create a project"
// can_delegate() -> true   (hands off to coding once a plan exists)
// uses_personality() -> true (it's still nomi talking)
// tools: create_project, write_plan

// nomi-agent-coding
pub const CODING_AGENT_TYPE: &str = "coding";
// is_delegation_target() -> true (default) — planning delegates to it; nothing delegates to
//   planning itself from coding (no back-and-forth loop in v1)
// tools: write_file, read_file, list_files, delete_file
```

Registered in `build_agent_registry()` (`nomi-server/src/lib.rs`) alongside the existing four.

**One real plumbing change, not just two lines:** `SubAgent::execute_tool` has no `S3Config` parameter today, and adding one to the trait would touch every existing agent (money, personality, chitchat, supervisor) for a capability only these two new agents need. Instead, `PlanningAgent`/`CodingAgent` hold their own `Option<S3Config>` at construction (`S3Config` is already `Clone`) — `build_agent_registry()` becomes `build_agent_registry(s3: Option<S3Config>) -> AgentRegistry`, and its two existing call sites, `worker::run` and `delegation_worker::run` (both already take explicit params like `pool`/`mqtt`/`settings_key` rather than a full `AppState` — see `nomi-server/src/worker.rs:19`, `delegation_worker.rs:63` — so this is one more parameter of the same shape, not a new plumbing pattern), gain an `s3: Option<S3Config>` parameter threaded from `main.rs`, which already builds this value for `AppState`.

### 2. Data model — file content lives in S3, Postgres holds metadata only

File bytes do **not** go in a Postgres column. `project_files` is an index — path, size, content type, timestamps — that makes the file tree a single fast query; the actual content lives at a deterministic S3 key, `projects/{project_id}/{path}`, computed from columns already in the row rather than stored redundantly.

```sql
-- migrations/0020_projects_and_files.sql

CREATE TABLE projects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT,
    plan        TEXT,                          -- latest plan, markdown; NULL until planning agent writes one
    status      TEXT NOT NULL DEFAULT 'planning'
                CHECK (status IN ('planning', 'building', 'ready')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE project_files (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path         TEXT NOT NULL,                -- e.g. "index.html", "src/app.js" — relative, no leading slash
    content_type TEXT NOT NULL DEFAULT 'text/plain',
    size_bytes   INTEGER NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, path)
);

CREATE INDEX project_files_project_id_idx ON project_files (project_id);
CREATE INDEX projects_session_id_idx ON projects (session_id);
```

`status` is a coarse, agent-and-UI-facing signal (`planning` until a plan exists, `building` while the coding agent is actively writing, `ready` once it calls `complete_task`) — driven by the tool handlers in §3/§4, not a separate state machine.

One plan per project, overwritten on each `write_plan` call — no revision history in v1 (YAGNI; add a `project_plan_revisions` table later if "show me what changed" is ever asked for).

**This feature requires S3 to be configured** — unlike avatars, there's no meaningful degraded mode for "the coding agent can't store any files." §3 covers the exact failure path: `create_project` checks `state.s3.is_some()` up front and fails clearly (to the agent, which relays it to the user) rather than letting the feature partially work and break later on the first `write_file`.

**`S3Config` gains three direct (non-presigned) methods** in `nomi-server/src/s3.rs`, alongside the existing `presign_put`:

```rust
pub async fn put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), S3Error>;
pub async fn get_object(&self, key: &str) -> Result<Option<String>, S3Error>;  // None on 404 / NoSuchKey
pub async fn delete_object(&self, key: &str) -> Result<(), S3Error>;
```

Write ordering is always **S3 first, then the Postgres metadata row** (`write_file` in §4, and the frontend's file-save route in §6): if the Postgres upsert fails after a successful S3 write, the object is merely orphaned (harmless — the next write to that path overwrites it, and it's invisible without a matching metadata row); the reverse order risks the file tree listing a path whose content was never actually written.

### 3. Planning agent tools

```rust
ToolDefinition {
    name: "create_project",
    description: "Create a new project for the app the user wants built. Call this once, \
                   before writing a plan.",
    input_schema: json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "description": "Short project name"},
            "description": {"type": "string", "description": "One-sentence description of what it does"}
        },
        "required": ["name", "description"]
    }),
}

ToolDefinition {
    name: "write_plan",
    description: "Write or replace the build plan for a project — the files you intend to \
                   create and the approach. The user sees this before building starts.",
    input_schema: json!({
        "type": "object",
        "properties": {
            "project_id": {"type": "string", "description": "The project ID from create_project"},
            "plan": {"type": "string", "description": "The plan, in markdown"}
        },
        "required": ["project_id", "plan"]
    }),
}
```

`execute_tool`: `create_project` first checks `state.s3.is_some()` — if `None`, returns `Err("Project creation isn't available right now — code storage isn't configured.".to_string())` as a normal tool error (same shape as any other `execute_tool` failure; the LLM sees it and explains to the user, no special-casing needed in the engine). Otherwise it inserts a row (`user_id`/`session_id` come from `run_agent_turn`'s own parameters, already threaded through every tool call — no new plumbing) and returns the new `project_id` as its tool-result text, so the LLM has it for the next call. `write_plan` does `UPDATE projects SET plan = $1, updated_at = now() WHERE id = $2 AND user_id = $3` (the `user_id` check is the ownership guard — same pattern as every other per-user row in this app).

System prompt instructs: after `write_plan`, call `delegate_to_agent` with `target_agent: "coding"` and a `task` string that **includes the project ID verbatim** (e.g. `"Project <uuid>: <plan summary>"`) — this is how the coding agent learns which project it's working on, since `agent_delegations.task` is plain text with no structured field for it (see Current State). Documented here as a deliberate v1 simplification: a dedicated `context_id UUID` column on `agent_delegations` would be the cleaner fix if a second use case ever needs structured delegation context, but one column for one field isn't worth a migration yet.

### 4. Coding agent tools

```rust
ToolDefinition {
    name: "write_file",
    description: "Create or overwrite a file in the project.",
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
ToolDefinition { name: "read_file", /* project_id, path -> current content or "not found" */ }
ToolDefinition { name: "list_files", /* project_id -> newline-separated paths */ }
ToolDefinition { name: "delete_file", /* project_id, path */ }
```

`write_file`: `self.s3.put_object(&format!("projects/{project_id}/{path}"), &content, &guess_content_type(&path)).await` (S3 write first, per §2's ordering rule), then `INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()` — same upsert shape already used in `user_profiles`/`user_preferences` (`profile.rs`), now over metadata instead of content. Also does `UPDATE projects SET status = 'building' WHERE id = $1 AND status = 'planning'` (idempotent — only transitions forward, so repeated writes are cheap no-ops after the first).

`read_file`/`delete_file` mirror this: `read_file` does `self.s3.get_object(&key)`, returning the text (or `"file not found"` on `None`) as the tool result — the coding agent needs the actual content inline to reason about edits, not a URL. `delete_file` calls `self.s3.delete_object(&key)` then `DELETE FROM project_files WHERE project_id = $1 AND path = $2`.

`projects.status = 'ready'` is set by the delegation worker, not a coding-agent tool: `delegation_worker.rs`'s existing match on `Ok(LoopOutcome::Completed { .. }) | Ok(LoopOutcome::Reply { .. })` is generic across every delegation target today (money, personality, ...). This design adds one targeted branch there, gated on `claimed.target_agent_type == CODING_AGENT_TYPE`: parse the project UUID out of `claimed.task` (the same "Project `<uuid>`: ..." convention §3 mandates planning use — a plain UUID-pattern extraction, not a new column) and run `UPDATE projects SET status = 'ready' WHERE id = $1 AND status = 'building'`. A malformed/missing UUID in the task text is a no-op, not an error — the project simply stays `building` and the next `list_files`/UI poll shows it as such; this is deliberately tolerant rather than failing the whole delegation over a status cosmetic. `session_id` alone isn't a safe lookup key here since one chat session can hold multiple projects over time.

Every tool call re-validates `project_id` belongs to the delegation's `user_id` (`WHERE id = $1 AND user_id = $2`), same ownership-guard pattern as planning's tools.

### 5. Delegation wiring

No changes to the delegation mechanism itself — it's generic today (`create_delegation`/`delegation_worker.rs` don't know or care what `target_agent_type` means). Two registry entries:

```rust
// nomi-server/src/lib.rs
Box::new(nomi_agent_planning::PlanningAgent),
Box::new(nomi_agent_coding::CodingAgent),
```

`ChitchatAgent` needs no change — `can_delegate() == true` already, and `delegatable_agent_types()` picks up `planning` automatically (any non-default, `is_delegation_target()`-true agent). `PlanningAgent::can_delegate() -> true` makes `coding` available to it the same way.

### 6. Frontend — Projects area

New route group mirroring `chat`/`profile`'s shape:

- `(app)/projects/+page.svelte` — list of the user's projects (`Card` per project: name, description, status badge), linking into each.
- `(app)/projects/[projectId]/+page.svelte` — three-pane layout: file tree (left, `List`/`ListItem`), code editor (center), plan + preview toggle (right or a tab). Loads via `+page.server.ts` → `GET /api/projects/:id` (files + plan + status).
- Sidebar gains a "Projects" nav entry (same place as the existing chat-session list), showing projects tied to the current session inline or as a separate top-level list — **left as an implementation-time call, not a spec-blocking decision**, since it's pure layout and doesn't affect the data model or agent design.
- Code editor: recommend **CodeMirror 6** (`@codemirror/*` packages) over Monaco — much smaller bundle, no web-worker bundling complexity, and matches this codebase's existing preference for lean, hand-integrated pieces over heavy pre-built UI kits (e.g. the MD3 components here are hand-built rather than pulling in `@material/web` wholesale).
- New backend routes: `GET /api/projects`, `GET /api/projects/:id` (metadata + file list from Postgres — no S3 call), `GET /api/projects/:id/files/:path` (S3 `get_object`, returned as the response body — the file tree lists paths from Postgres, content is fetched per-file on open, not bulk-loaded), `PUT /api/projects/:id/files/:path` (lets the *user* edit a file directly, not just the agent — same S3-then-Postgres write path as §4's `write_file`), `DELETE /api/projects/:id/files/:path`. All gated on `claims.sub == projects.user_id`, no new permission model needed (this is personal data, like sessions/messages already are). Content is proxied through nomi's backend rather than presigned S3 URLs — these are small text files edited in-browser, not the large-binary case presigned uploads exist for, so there's no CORS/bucket-policy setup to add and every write stays server-validated like the rest of this app's writes.

### 7. Preview — static files only, no execution

`GET /api/projects/:id/preview/*path` looks up the `project_files` row for `path` (for its stored `content_type`), fetches the bytes from S3 (`get_object`), and serves them with that `Content-Type`, rendered in the frontend inside a **sandboxed iframe** (`sandbox="allow-scripts"`, no `allow-same-origin`, so previewed content can never read nomi's own cookies/storage). `path` defaults to `index.html`.

This covers static/vanilla-JS projects end-to-end with zero new execution infrastructure — the biggest class of "simple app" a coding agent would plausibly one-shot. Anything needing a build step or a dev server (React/Vue/Svelte with npm, a backend process) shows "Preview isn't available for this project yet" instead of attempting to run it. That gap — real execution — is real infrastructure work (subprocess or container lifecycle, resource limits, port allocation) and is intentionally a separate future design, not bundled in here.

## Summary of what ships vs. what's explicitly deferred

| Ships in this design | Deferred |
|---|---|
| `planning` and `coding` agents, reusing the existing delegation mechanism | Publishing to Shipyard (blocked on Shipyard's own create-project/service API) |
| `projects` / `project_files` data model | Running/building generated projects (real execution/sandboxing) |
| File read/write/list/delete tools | Multi-file build tooling (npm install, bundlers) |
| In-app file tree + code editor, user-editable | Plan revision history |
| Static-file preview (sandboxed iframe) | Dev-server preview for non-static projects |
