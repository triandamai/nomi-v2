# Frontend Chat UI Design

Date: 2026-07-25
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-22-sub-agent-lifecycle-design.md`, `docs/superpowers/specs/2026-07-22-cross-channel-identity-design.md`, `docs/superpowers/specs/2026-07-25-multi-tenant-rbac-design.md` (amends §1 and §2 below — role model and admin routing)

## Purpose

The backend orchestrator (`plans/initial.md`) holds one persistent conversation thread per user per channel, with sub-agents spawning/completing mid-thread and every step audited via MQTT + `agent_events`. This document designs the web frontend that makes that thread visible and usable: a chat list, a conversation view with visible agent reasoning, and an input bar that can address the agent layer directly (slash commands, per-turn mode) rather than only sending plain text.

## 1. Audience & Roles

*Superseded by `2026-07-25-multi-tenant-rbac-design.md` §2 and §6* — kept here in summary form so this doc still reads standalone:

One SvelteKit app serves every user from the same UI, scoped by permission-string claims rather than a flat role:

- **Regular member** — sees their own org's sessions (DM or group, across whichever channels they're linked to via `channel_identities`), scoped to whichever org is currently active in the org switcher.
- **Org owner/admin** — same view as a member, plus access to `/org/:id/members` for managing that org's membership and invites.
- **Platform admin** (`nomi:admin:*`) — sees every org/session on the platform via the separate `/admin` route, for oversight and debugging.

**Authorization is a backend-issued claims array, not a frontend construct.** The session token carries permission strings (`nomi:<scope>:<role>:[actions]`); the frontend's route guards (SvelteKit `load` redirects) are a UX nicety only — the real enforcement is that the backend checks these same permission strings server-side on every request, and re-validates against current DB state (not just the token) for sensitive writes. This must hold even if someone hits the API directly, bypassing the UI entirely.

## 2. Screens & Navigation

- **Left rail — chat list**: flat list of the *current org's* sessions, sorted by most recent activity (not grouped by channel — recency ordering matters more than provenance, and provenance is shown per-row via a channel badge instead). Each row: avatar, name (or group name), channel badge (Telegram/WhatsApp/Slack/web), last-message preview, timestamp, unread count, and an agent-active pill when a sub-agent currently owns that thread (or, in a group, owns any participant's turn within it).
- **Right pane — conversation view**: header (participant/group name, channel, a persistent agent-state badge), scrollable message bubbles, input bar pinned to the bottom. Selecting a chat updates the URL (`/chat/[sessionId]`) so it's linkable and survives a refresh.
- **Org switcher**: visible whenever the user has more than one active, non-personal org membership; switching re-scopes claims and reloads the chat list for the newly-selected org only (no unified cross-org inbox in this iteration).
- **`/org/:id/members`** — org owner/admin only; member list, role management, invite-link generation, and the "add to group" conversation-participant picker (all per the RBAC design's §4 and §6).
- **`/admin`** — platform-superuser-only (`nomi:admin:*`); the same `ChatList`/`ConversationView` components as the main shell, but unfiltered across every org/session on the platform, each row additionally showing the owning org and user identity. Structurally distinct from `/org/:id/members` — this is the rare, ops-only surface; the org-scoped one is what most org admins actually use day to day.
- **Onboarding** (create-or-join): shown post-registration, before first chat — previously out of scope, now in scope per the RBAC design's §4.

Out of scope for this spec: the underlying login/identity-linking mechanism (how a Telegram/WhatsApp/web identity authenticates into a browser session in the first place), and admin *intervention* actions (force-cancelling an agent, editing memory weights) — both remain natural follow-up specs.

## 3. Components

- **`ChatListItem`** — avatar, name, channel badge, preview text, timestamp, unread count, `AgentBadge` pill. Admin variant additionally renders the owning identity.
- **`ConversationView`** — header with participant/group name, channel, and a persistent `AgentBadge`; scrollable `MessageBubble` list; pinned `InputBar`.
- **`MessageBubble`** — user bubbles are plain text/image. Assistant bubbles render an **always-expanded reasoning timeline** above the final reply text: a thinking line, then one line per tool call (name, args, result), then the reply — matching the backend's real tool-call/reasoning trace rather than summarizing it away. This is deliberately verbose (see Trade-offs below).
- **`InputBar`** — a single pill (text field only by default, no separate icon row). "+" opens a sheet for image/file/location attachments. Typing "/" opens a command palette listing available agent tools as commands (e.g. `/booking`, `/memory`) — these map to backend tool invocations under the hood but are presented as commands, not a raw tool list. A small "mode" chip (e.g. Auto-route / force chitchat) sends a per-turn routing hint alongside the message. Display-only preferences (e.g. a compact bubble view) are a separate, view-level setting and do not live in this bar, since they're not something sent to the backend.
- **`AgentBadge`** — shared pill component (used in both `ChatListItem` and `ConversationView`'s header) mapping `agent_sessions.status` (per the sub-agent lifecycle design: active/completed/cancelled/expired) to color/label: active = amber/pulsing, completed = muted then disappears after a short grace period, cancelled/expired = hidden.

## 4. Data Flow & Real-Time Updates

1. Loading `/` fetches the session list via `GET /api/sessions?org=<currentOrgId>` (org-scoped, enforced server-side); `/admin` instead calls the unfiltered platform-admin endpoint. Either way the result populates a `sessions` Svelte store.
2. Opening a conversation fetches recent history via `GET /api/sessions/:id/messages`, then opens a WebSocket (`WS /api/sessions/:id/stream`) for that session. The backend bridges its internal MQTT event bus to this per-session WebSocket — browsers don't speak MQTT directly.
3. Three event kinds flow over the socket, mirroring the backend's audited event model:
   - `MessageAppended` — a user or assistant message; assistant messages can arrive with reasoning/tool-call lines progressively, then the final reply text, giving the timeline a natural streaming reveal instead of popping in whole.
   - `AgentStateChanged` — spawned/completed/cancelled/expired, updates the `agentState` store and therefore every `AgentBadge` referencing that session/participant.
   - `TurnFailed` — surfaces the backend's `AgentTurnFailed` audit event as an inline per-turn error affordance, not a lost message (the user's inbound message is already durable per the backend's two-transaction design).
4. Sending a message: optimistic-append the user bubble locally, `POST /api/sessions/:id/messages`, then rely on the socket for the assistant's reply.
5. Admin's unfiltered chat list additionally subscribes to a lightweight cross-session `AgentStateChanged` feed so pills update live across many rows without opening one socket per session.

## 5. Error Handling

- **WebSocket drop**: reconnect with backoff; on reconnect, re-fetch recent history via REST to reconcile anything missed while disconnected. Safe because the backend's `messages`/`agent_events` tables are the durable source of truth — the socket is only a live-push convenience.
- **Turn failure**: show a small inline "no reply yet" state on that turn, consistent with the backend's own framing ("a missing reply reads as the assistant not having answered yet"), rather than a generic error banner.
- **Send failure (network)**: the optimistically-appended bubble is marked "not sent" with a retry action — never silently dropped.

## 6. Testing Approach

- Component tests: `MessageBubble` reasoning-timeline rendering states (in progress / complete / tool-call formatting), `AgentBadge` status→style mapping, `InputBar` command palette and mode chip behavior.
- Integration test: a mocked WebSocket feed exercising the reconnect-and-reconcile path.
- Route-guard test: confirms `/admin` redirects anyone without `nomi:admin:*` client-side, and `/org/:id/members` redirects anyone without `owner`/`admin` in *that specific* org — **and** companion backend tests confirm both API endpoints reject the equivalent unauthorized tokens server-side, including a valid owner/admin of a *different* org. The redirect alone is not the security boundary.

## Trade-offs Accepted

- **Always-expanded reasoning timeline** (chosen over a collapsed "▸ Thinking..." summary) makes every assistant bubble bulkier and will read as noisy in long conversations. Accepted deliberately for transparency; if this proves too dense in practice, a future iteration could add a per-conversation or per-user "compact reasoning" display toggle without changing the underlying data model (the toggle would only affect client-side rendering, since the full trace is always sent).
- **Flat, recency-sorted chat list** (chosen over grouping by channel) means channel provenance is only a small per-row badge, not a section header — acceptable since recency is the more common way people scan a chat app, and the badge is still visible per row.

## Open Questions (deferred, not blocking this spec)

- Login/identity-linking flow (how a channel identity becomes an authenticated browser session).
- Admin intervention actions (cancel/override a running sub-agent from the UI).
- Whether a "compact reasoning" display toggle is needed once real usage shows the always-expanded timeline's actual noise level.
