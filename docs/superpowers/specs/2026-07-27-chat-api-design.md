# Chat REST API

Date: 2026-07-27
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-25-frontend-chat-design.md` (this plan builds the API surface that doc assumes already exists), `docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md` (`handle_inbound_message`, wrapped by this plan's `POST .../messages` endpoint, untouched), `docs/superpowers/specs/2026-07-22-cross-channel-identity-design.md` (the `channel_identities` model this plan extends to a `"web"` channel), `docs/superpowers/plans/2026-07-26-auth-claims-layer.md` (the JWT/`AuthClaims`/`authorize_org_action` machinery this plan reuses)

## Purpose

The frontend chat UI design (`2026-07-25-frontend-chat-design.md`) assumes a REST API — `GET /api/sessions`, `GET /api/sessions/:id/messages`, `POST /api/sessions/:id/messages` — plus a WebSocket stream bridging an internal MQTT bus. None of this exists: the backend currently only exposes auth routes (`/api/auth/*`, `/api/whoami`). This document designs the REST layer only, deferring WebSocket/MQTT streaming (the frontend polls or re-fetches for now, consistent with how this project has deferred MQTT in every prior plan). This gives the frontend something real to render against.

The central design problem this plan resolves: `handle_inbound_message`'s existing bootstrap logic (`bootstrap_identity_and_session`) was built for channel adapters (Telegram/WhatsApp) where `(channel, channel_user_id)` is the *only* way to know who's messaging, and it always creates a brand-new `users` row on first contact. A web REST caller already has an authenticated `user_id` from their JWT (via the existing `/api/auth/login`) — reusing that bootstrap path blindly risks creating a duplicate user. This plan bridges the two by treating `"web"` as just another channel, with an idempotent helper that ties a web `channel_identities` row to the caller's *already-authenticated* `user_id` before `bootstrap_identity_and_session` ever runs — no changes to that function, or to any turn-loop code, are needed.

## 1. Architecture

Two new pieces, plus wiring into the existing router:

- **`backend/src/web_identity.rs`** — `ensure_web_channel_identity(pool, user_id) -> Result<Uuid, sqlx::Error>`, an idempotent, race-safe resolve-or-create for a `channel_identities` row (`channel = 'web'`, `channel_user_id = user_id.to_string()`) explicitly tied to the given `user_id` — never creates a new user.
- **`backend/src/routes/sessions.rs`** — four new endpoints (below), wired into `app.rs` alongside the existing auth routes.

No changes to any other turn-loop/auth code from prior plans. One narrow exception discovered during implementation: `bootstrap_identity_and_session` (and `handle_inbound_message`, which calls it internally) assumed every identity belongs to a personal org — true for Telegram/WhatsApp bot-first-contact users, false for web-registered users, who only ever get named/team orgs. Both functions gained a trailing `org_id_hint: Option<Uuid>` parameter, consulted only in the existing-identity branch; the first-contact branch and every pre-existing (Telegram-context) call site are unchanged, passing `None`. The web path passes `Some(claims.active_org_id)`.

## 2. Endpoints

All four require the existing `AuthClaims` extractor (JWT) and re-validate org membership against current DB state via the existing `authorize_org_action` — never trusting the token alone for authorization, matching the frontend design's own stated security principle.

### `POST /api/sessions`
Creates a new web chat session (the "New Chat" action). Generates a fresh `chat_id` (a UUID string — there's no external platform id for a web-originated chat), calls `ensure_web_channel_identity`, then `bootstrap_identity_and_session(pool, "web", "dm", &chat_id, &user_id.to_string())` to create the session row.

```
Response: 201 { "session_id": "<uuid>" }
```

Group chat creation is out of scope — `chat_type` is always `"dm"` here.

### `GET /api/sessions`
Lists the caller's current org's (`claims.active_org_id`) sessions, most-recent-activity first.

```
Response: 200 {
  "sessions": [
    {
      "id": "<uuid>",
      "channel": "web",
      "chat_type": "dm",
      "chat_id": "<uuid-string>",
      "last_message": { "content": "...", "created_at": "<rfc3339>" } | null,
      "agent_active": true | false,
      "updated_at": "<rfc3339>"
    }
  ]
}
```

`updated_at` is the last message's `created_at`, or the session's own `created_at` if it has no messages yet. `agent_active` is a plain boolean — an active `agent_sessions` row exists for any speaker in this session. This is raw data only: no derived display "title" — a web session has no inherent name (unlike a Telegram/WhatsApp DM's participant name), so title/label derivation is left to the frontend, which can build one from `last_message.content`.

### `GET /api/sessions/:id/messages?before=<message_id>&limit=<n>`
Message history for one session, oldest-first within the returned page.

```
Response: 200 {
  "messages": [
    { "id": "<uuid>", "sender": "user" | "assistant", "content": "...", "created_at": "<rfc3339>" }
  ]
}
```

`limit` defaults to 50, capped at 100. `before` (a message id) pages further back; omitted, fetches the most recent page. `sender` is `"user"` when the row's `sender_channel_identity_id IS NOT NULL`, `"assistant"` when `NULL` — same convention already used internally by chitchat/sub-agent history fetches.

### `POST /api/sessions/:id/messages`
Sends a message. Resolves the target session's `(channel, chat_id)`, calls `handle_inbound_message` exactly as it exists today, then fetches the two most-recently-persisted messages for the session (the inbound one and the assistant's reply) to return with real ids and timestamps.

```
Request:  { "text": "..." }
Response: 200 {
  "user_message": { "id": "<uuid>", "content": "...", "created_at": "<rfc3339>" },
  "assistant_message": { "id": "<uuid>", "content": "...", "created_at": "<rfc3339>" }
}
```

Since no WebSocket exists yet, this HTTP response *is* how the caller learns the assistant's reply. On a `TurnError`, responds `502 { "error": "<message>" }` — the inbound message is already durable server-side (Txn A, per the turn-loop design), so the frontend must treat this as "no reply yet," not "message lost," and may safely retry or re-fetch history.

## 3. Web Identity Bridging (exact mechanics)

```rust
// backend/src/web_identity.rs
use sqlx::PgPool;
use uuid::Uuid;

pub async fn ensure_web_channel_identity(pool: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM channel_identities WHERE channel = 'web' AND channel_user_id = $1",
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) \
         ON CONFLICT (channel, channel_user_id) DO NOTHING RETURNING id",
    )
    .bind(user_id)
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    match inserted {
        Some(id) => Ok(id),
        None => {
            // Lost a race against a concurrent call for the same user (e.g. two browser tabs
            // opened at once) — re-fetch the winner. Same pattern already proven in
            // bootstrap_identity_and_session's own concurrent-first-contact handling.
            sqlx::query_scalar(
                "SELECT id FROM channel_identities WHERE channel = 'web' AND channel_user_id = $1",
            )
            .bind(user_id.to_string())
            .fetch_one(pool)
            .await
        }
    }
}
```

`channel_user_id = user_id.to_string()` — the user's own id, stringified, stands in for "the external platform's id" since a web session has no such thing. After this identity exists (created on first call, idempotent thereafter), `bootstrap_identity_and_session` always hits its "existing identity" branch for this user — it is never given the chance to create a duplicate `users` row.

## 4. Authorization

- `GET /api/sessions`: filters directly by `sessions.org_id = claims.active_org_id` — no per-row DB re-check needed beyond that equality, since the JWT's `active_org_id` is itself re-issued on org switch and expires normally.
- The three session-scoped endpoints (`POST /api/sessions/:id/messages`, `GET /api/sessions/:id/messages`, and implicitly the session created by `POST /api/sessions`): look up the session's `org_id`, then call the existing `authorize_org_action(pool, claims.sub, session_org_id, &["owner", "admin", "member"])` — re-validated against current membership state in the DB, not just trusted from the token (a membership could have been revoked since the token was issued).
- A session whose `org_id` doesn't match an org the caller currently belongs to → `404`, not `403` — avoids confirming to an unauthorized caller that a given session id even exists.

## 5. Error Handling

- `TurnError` from `POST .../messages` → `502 { "error": "<message>" }`. The inbound message is durable regardless (Txn A) — the frontend must not treat this as data loss.
- An unrecognized `session_id` in the URL, or one belonging to another org → `404`.
- Malformed request body (missing `text`, empty string) → `400`.
- All other unexpected `sqlx::Error`s → `500`, logged server-side (no stack trace or internal detail leaked in the response body).

## 6. Testing Approach

- Route tests (matching the existing `auth_routes.rs` pattern: `axum::body::Body`/`http::Request` against the real router, real Postgres, `tower::ServiceExt::oneshot`): create a session via `POST /api/sessions`, confirm it appears in `GET /api/sessions`; send a message via `POST .../messages`, confirm both `user_message`/`assistant_message` come back with real ids and the history endpoint reflects them afterward; a caller from a different org gets `404` on someone else's session.
- `ensure_web_channel_identity`: idempotent across repeated calls for the same user; a genuine concurrency test (two simultaneous calls for the same `user_id`, matching `bootstrap_identity_and_session`'s own concurrency-test pattern) never violates the `(channel, channel_user_id)` unique constraint and both calls resolve to the same identity id.
- Pagination: seed more than the default page size of messages, confirm the default page and `before`-cursor paging both return the correct, non-overlapping slices in the right order.
- Turn failure: wire a `FakeLlmProvider::failure` through (reusing the existing test double) and confirm `POST .../messages` returns `502` while the inbound message is still queryable via `GET .../messages` afterward.

## Out of Scope

- WebSocket/real-time streaming and any MQTT bridge — deferred; the frontend polls or re-fetches for now.
- Unread-message counts — would require new per-user read-tracking schema (e.g. a `last_read_message_id`), not justified without a concrete UI need yet.
- Group chat creation, `/org/:id/members` (member management/invites), and the platform-admin unfiltered endpoint — all separate, later slices per the frontend design doc's own scoping.
- Session title/name generation — the backend returns raw last-message content only; deriving a display title/label is a frontend concern.
- The login/identity-linking flow itself — this plan assumes the caller already holds a valid JWT from the existing `/api/auth/login`.
