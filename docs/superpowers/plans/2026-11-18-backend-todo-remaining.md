# Backend TODOs remaining — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build every backend entry still open in `frontend-new/BACKEND_TODO.md`, so the new frontend can stop leaving those steps out.

**Architecture:** Each task adds what one entry's *Proposed contract* asks for, to the existing axum routes in `backend/crates/nomi-server/src/routes/`, the content blocks in `nomi-agent-core`, and new numbered migrations. Contracts are additive: existing fields, paths and statuses keep working for the old `frontend/` and native apps.

**Tech Stack:** Rust (axum, sqlx/Postgres, tokio), MQTT realtime (`nomi-realtime`), existing crates only unless a task names a new one.

**Spec:** `frontend-new/BACKEND_TODO.md` on main — each task names its entry; that entry's *Proposed contract* is the spec (a suggestion, refined where the code shows a better fit; record the refinement as a ruling).

## Global Constraints

- LLM system prompts live only in `backend/crates/nomi-agent-core/src/prompts.rs`.
- Every numeric literal gets a named `const`.
- Migrations: new files in `backend/migrations/`, numbered from `0157_` upward, one per task that needs one; never edit an existing migration.
- Additive only: never remove or rename an existing response field, route or status.
- Realtime is the existing MQTT topics and envelopes; extend them, never add a parallel relay.
- Never edit `frontend-new/` or `frontend/` in these tasks (the TODO file is updated once at the end).
- Commits: Conventional Commits, scoped, no AI attribution of any kind.
- Tests: `cd backend && cargo test -p <crate> <filter>` against the local Postgres (`DATABASE_URL=postgres://nomi:nomi@localhost:5432/nomi`) and EMQX; don't change their config or data beyond what the tests create.

## Review Focus

- Old clients: a request without the new fields behaves exactly as before.
- Authorization: every new route checks the session/project/attachment belongs to the caller, like its neighbours.
- Time windows (undo deny) measured on the server clock, not trusted from the client.
- Secrets: signed file URLs only when a share allows files; GitHub tokens never in responses.
- Idempotence: retrying a cancel/undo/discard/summarize request doesn't double-apply.

---

### Task 1: Attachment error codes, accepted types, "sent without"
Entries: *Upload failure codes (G3)*, *The accepted types (G4)*, *"Sent without" kept on the message (G6)*.
Files: `routes/attachments.rs`, `nomi-attachments`, `routes/sessions.rs` (message send), migration if `left_out` needs a column.
- [ ] Failing tests: 413 body `{error:"too_large", message, max_bytes}`; 507 `storage_full`; 415 `{error:"unsupported_type", extension}`; interrupted → `interrupted`; `GET /api/attachments/limits` has `accepted.{kinds,extensions,mimes}`; `POST /api/sessions/:id/messages` with `left_out` keeps it and returns it on the message, and the crew sees a line naming the left-out file.
- [ ] Implement; tests green; commit `feat(attachments): coded upload errors, accepted types, sent-without notes`.

### Task 2: Who answered each chat; when an ask started
Entries: *Who last answered in each chat*, *When an ask started running* (the `started_at` part).
- [ ] Failing tests: `GET /api/sessions` rows have `last_message.agent_type` (null for the person) and `agents: [..]`; `GET /api/sessions/:id/queue` running item has `started_at`.
- [ ] Implement (+ migration for `started_at`); commit `feat(sessions): last answering agent, crew list, ask start time`.

### Task 3: Question recommendations and helper; rich run_failed
Entries: *Questions: a recommended answer, and a helper line*, *A failed turn's status, provider and the provider's words*.
- [ ] Failing tests: `Questions` serialises `options[].recommended` and `questions[].helper` (defaults false/null, old JSON still parses); the ask-questions tool schema accepts them; `run_failed` carries `kind, status, provider, provider_message, model`, with `kind:"content_filter"` for a provider content-filter error.
- [ ] Implement; commit `feat(agent-core): recommended answers, helper lines, detailed run failures`.

### Task 4: Approvals — edits, result link, scoped remember, undo, discard
Entries: *An approval sent with the person's edits*, *Undo a deny, and discard after a stop*, *"Don't ask again" scoped to a recipient*.
- [ ] Failing tests: approve with `input` runs the tool on the edited input (validated against the tool's schema); `result_url` on the block after a tool returns one; `remember:{scope:"recipient",value}` only auto-approves that recipient; `remember:true` still works; `POST …/approval/undo` within the window → pending, after → 409; `POST /api/sessions/:id/stopped/:message_id/discard` drops partial work, twice is a no-op.
- [ ] Implement (+ migration); commit `feat(approvals): edited input, scoped remember, undo deny, discard stopped work`.

### Task 5: Structured plan steps; run or research
Entries: *A plan's steps*, *Run a proposed plan, or only research it*.
- [ ] Failing tests: plan versions return `steps[{text, owner_agent_type, status, status_detail, started_at, finished_at}]` (parsed from the checklist when no structured steps exist) and `estimate_seconds`; `POST …/agent-plans/:id/run {mode}` → 202 and starts the turn in that mode; bad mode → 400.
- [ ] Implement (+ migration); commit `feat(plans): structured steps, estimates, run or research`.

### Task 6: Summarise and continue
Entry: *Summarise older messages and carry on*.
- [ ] Failing tests: `POST /api/sessions/:id/summarize` → 202, writes a summary message, later turns send the summary instead of the summarised messages, the failed turn is retried; twice while running → 409.
- [ ] Implement (prompt in prompts.rs); commit `feat(sessions): summarise older messages and continue`.

### Task 7: Files in a shared link
Entry: *Files in a shared link (G16)*.
- [ ] Failing tests: `files_open` on create/update (default false); with `mask` → 400 `mask_and_files_open`; `GET /api/shared/:token` has `files_open` and `files[]`, `url` only when open.
- [ ] Implement (+ migration); commit `feat(shares): open a chat's files from a shared link`.

### Task 8: Web Push
Entry: *Web Push for a phone's browser*.
- [ ] Failing tests: `POST /api/devices/web-push` stores the subscription (upsert by endpoint); needs-you, approval and plan-finished pushes are sent to web-push subscriptions through the same dispatcher as FCM/APNs; a 404/410 from the push service deletes the subscription.
- [ ] Implement (VAPID keys from env, `.env.example` documented; + migration); commit `feat(push): web push for phone browsers`.

### Task 9: Read results and pre-send estimate
Entries: *A read result per file (G9)*, *A pre-send estimate (G7)*.
- [ ] Failing tests: `GET /api/attachments/:id` has `read{result,read?,total?,skipped?}` (locked/no_text/damaged/partial/ok from extraction) and `replaces`; `POST /api/attachments/estimate` returns per-file pages/tokens, total, payer, provider_label, `cost_idr` when credits, `threshold_tokens`.
- [ ] Implement (+ migration); commit `feat(attachments): read results and a pre-send estimate`.

### Task 10: Resumable uploads
Entry: *Resumable uploads (G3, M1)*.
- [ ] Failing tests: create → `{upload_id, offset:0}`; PATCH with Content-Range appends and returns offset; wrong offset → 409 with current offset; HEAD returns offset; last chunk returns `AttachmentResponse`; size limits enforced across chunks; expired uploads cleaned.
- [ ] Implement (+ migration); commit `feat(attachments): resumable uploads`.

### Task 11: Project card build progress, deploys, allowance, move approval
Entry: *Projects: a card's progress, a build's allowance, moving with approval*.
- [ ] Failing tests: summaries carry `build` and `last_deploy`; `DELETE /api/projects/:id/deploy/publishes/:id` cancels a running publish (409 when finished); `GET /api/me/plan` has the build allowance; transfer to a workspace the caller doesn't own creates a pending request the owner approves/declines.
- [ ] Implement (+ migration); commit `feat(projects): build progress, last deploy, cancel publish, move approval`.

### Task 12: Ask built event, request's change
Entry: *When an ask started running, and what a change to publish holds* (rest of it).
- [ ] Failing tests: `AskBuilt` chat event published when a coding turn finishes with files changed; publish requests carry the ask they follow and the files changed since the live deploy.
- [ ] Implement (+ migration); commit `feat(deploy): ask-built events and what a publish request holds`.

### Task 13: GitHub repos, private import, import review
Entry: *Import from GitHub*.
- [ ] Failing tests: `GET /api/connections/github/repos?q=` lists with the connected token (mocked GitHub); import uses the token for private repos; `GET /api/projects/:id/import/review` returns the review shape from a detector over the imported tree.
- [ ] Implement; commit `feat(github): list repos, import private ones, review an import`.

### Task 14: Files Maya makes
Entry: *Files Maya makes (G11)*.
- [ ] Failing tests: file events `file_generating/file_ready/file_failed`; content, preview, delete (keeps `deleted_at`), remake routes; storage-full 402 body `{error:"storage_full"}`.
- [ ] Implement (+ migration); commit `feat(files): files the crew makes in a chat`.

### Task 15: Citations to a file's pages
Entry: *Citations to a file's page and rows (G12)*.
- [ ] Failing tests: file citations carry `file_id, pages, spans`; `GET /api/attachments/:id/pages/:n` renders a page image (404 past the last page).
- [ ] Implement; commit `feat(attachments): cite a file's pages and rows`.

### Final
- [ ] Whole-branch review; mark each built entry done in `frontend-new/BACKEND_TODO.md` with a **Built:** line (the file's existing convention); push; open the PR.
