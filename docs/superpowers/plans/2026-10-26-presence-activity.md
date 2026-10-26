# Presence and activity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Part 3 of the project sheets redesign: "<Name> is typing…" in the project chat, who has the project open now (Members → Activity, and a small avatar stack in the toolbar), and a live feed of what happens in the project.

**Architecture:** Presence and typing are short-lived and never stored as messages. The project page sends a heartbeat (`POST /api/projects/:id/presence`) every 20 s and a typing ping while the person types; the server keeps the last heartbeat per person in `project_presence` (expiring after 45 s) and publishes `PresenceChanged` / `Typing` on the project's live topic (`project/{id}/stream`), which only people who can open the project can hear. Leaving sends `DELETE` (and the expiry covers tabs that close without it). The activity feed is derived from tables Nomi already writes (messages, coding delegations and the file changes inside them, publishes and rollbacks, members): no new event table, so it's accurate for history from before this ships. The page's `ProjectDeploy` stops reloading on every project-topic message and only reloads on `DeployChanged`.

**Tech Stack:** Rust/axum + sqlx, MQTT via `project_events`, Svelte 5, paraglide, vitest.

**Spec:** `docs/superpowers/specs/2026-10-26-project-sheets-redesign-design.md` §3.

## Global Constraints

As parts 1–2. Plus: presence and typing travel only on the project topic; nothing about them is written to `messages`; a person never sees their own typing or themselves in "Here now".

## Review Focus

1. Two tabs of the same person: they appear once in Here now; closing one tab doesn't remove them while the other still beats.
2. A tab that closes without saying goodbye (phone sleeps): gone from Here now within the expiry.
3. Someone who lost access: their heartbeat is refused and they never appear.
4. Fast typing: pings are throttled (at most one every few seconds), and the indicator clears a few seconds after the last ping or when the message is sent.
5. A big project history: the feed pages (`before`, 50 at a time) and never loads all of it.

---

### Task 1: Presence and typing (backend)

**Files:** Create `backend/migrations/0083_project_presence.sql`, `backend/crates/nomi-server/src/routes/project_presence.rs`; modify `backend/crates/nomi-server/src/routes/mod.rs`, `app.rs`; test `backend/crates/nomi-server/tests/project_presence_routes.rs`.

**Produces:**
- Table `project_presence (project_id, user_id, last_seen, PRIMARY KEY (project_id, user_id))`.
- `POST /api/projects/:id/presence` → 204; upserts `last_seen`; publishes `{"kind":"PresenceChanged"}` when the person wasn't already present.
- `DELETE /api/projects/:id/presence` → 204; removes the row; publishes `PresenceChanged`.
- `GET /api/projects/:id/presence` → `{ "people": [{ "user_id", "name", "avatar_url" }] }`, everyone seen in the last 45 s except the caller.
- `POST /api/projects/:id/typing` → 204; publishes `{"kind":"Typing","user_id","name"}`.
- All four need `project_role` (can open the project).

- [ ] Failing tests: heartbeat then another member's GET lists them, the caller never sees themself; a row older than the expiry isn't listed; DELETE removes them; a stranger gets 404 on every route; typing answers 204 for a member, 404 for a stranger.
- [ ] Implement; `cargo test --workspace`; commit.

### Task 2: Project activity feed (backend)

**Files:** Create `backend/crates/nomi-server/src/routes/project_activity.rs`; modify `app.rs`; test `backend/crates/nomi-server/tests/project_activity_routes.rs`.

**Produces:** `GET /api/projects/:id/activity?before=<rfc3339>&limit=50` → `{ "items": [{ "kind", "at", "actor": { "name", "avatar_url", "agent" } | null, "text"?, "version"?, "environment"?, "status"?, "phase"?, "step"?, "files"?, "rollback_of_version"? }], "has_more" }`, newest first, merged from:
- `prompt`: a person's message in the project chat (text shortened to 140 chars);
- `build_started` / `build_finished` / `build_failed`: coding delegations for the project's chat (`claimed_at` / `completed_at`, status);
- `files_changed`: per finished coding delegation, the count of `file_write` + `file_writes[].files` + `file_delete` blocks posted in the chat between its claim and completion (only when > 0);
- `published` / `publish_failed` / `rolled_back`: from `project_publishes`;
- `joined`: from `project_members.created_at`.
Needs `project_role`.

- [ ] Failing tests: each kind appears from seeded rows with the right fields; newest first; `before` + `limit` page with `has_more`; a stranger gets 404.
- [ ] Implement; tests; commit.

### Task 3: Typing line, heartbeat, toolbar avatars (frontend)

**Files:** Create `frontend/src/lib/projectPresence.svelte.ts` (+ test for the pure parts in `frontend/src/lib/presenceView.ts`/`.test.ts`); modify `projectDeploy.svelte.ts` (reload only on `DeployChanged`; expose the socket's other messages to listeners), `ChatThread.svelte` (props `othersTyping?: string[]`, `ontyping?: () => void`), the project page (heartbeat lifecycle, typing ping, avatar stack in the toolbar opening Members → Activity).

**Produces:** `presenceView.ts`: `typingLine(names: string[]): string | null` inputs for the message (0 → null, 1, 2, 3+ → "A, B and 2 others"), `throttle(lastSentAt: number, now: number, everyMs: number): boolean`, `expireTyping(map: Record<string, number>, now: number, ttlMs: number)`.

- [ ] Tests for the helpers; implement; `npm run check` + `vitest`; commit.

### Task 4: Members → Activity (frontend)

**Files:** Modify `MembersPanel.svelte` (People / Activity tabs; prop `initialTab`), create `ProjectActivity.svelte`; messages.

- [ ] Here now row (avatars + names, or "Only you right now"), feed rows (avatar or agent mark, sentence, time), "Show more" paging, live refresh on project-topic events and every 15 s while open; commit.

### Task 5: Check it with two people

- [ ] Two browsers signed in as two members of one project: Here now shows each other, typing shows and clears, the feed shows prompts/builds/publishes; screenshots; PR stacked on part 2.
