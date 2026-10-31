# Nomi's API: the contract a frontend builds on

Nomi's backend (`backend/crates/nomi-server`) is the only thing a frontend talks to: the web app
(`frontend/`), the native apps (`native/`) and any new client go through the same HTTP and
WebSocket API. This page is the contract. When a route changes in a way that breaks it, the change
says so here first.

## Basics

- **Base URL.** Everything lives under `/api` on one origin (`API_URL` in the web app's
  environment; `http://localhost:8080` in development).
- **JSON.** Requests and responses are JSON (`Content-Type: application/json`) unless a route
  says otherwise (uploads take the file as the body; previews serve the project's own files).
  Field names are `snake_case`. Ids are UUIDs. Times are RFC 3339 in UTC.
- **Device id.** Every app makes a UUID the first time it runs and keeps it (the SDK in its data
  directory, the web app in the `device_id` cookie), and sends it on every call as
  `X-Device-Id`. It ties sign-ins to the device and names the device's realtime connection.
- **Request ids.** Every response carries `x-request-id`. Send your own (any string) to follow a
  request through the logs; otherwise the server makes one. Quote it when reporting a problem.

## Signing in

| Call | What it does |
|---|---|
| `POST /api/auth/register`, `POST /api/auth/login` | Answer a token pair, or a sign-in code challenge (`challenge_id`, finished at `POST /api/auth/verify`) when email codes are on |
| `POST /api/auth/verify`, `POST /api/auth/resend` | Finish or resend a code challenge |
| `GET /api/auth/google/available`, `POST /api/auth/google/start`, `POST /api/auth/google/callback` | Google sign-in |
| `POST /api/auth/refresh` | `{ refresh_token }` → a new `{ access_token, refresh_token }` |
| `POST /api/auth/logout` | Revokes the refresh token |

Every other call sends `Authorization: Bearer <access_token>`. The access token is short-lived:
on a `401`, refresh once and retry; if the refresh fails too, sign the person out. Refresh tokens
rotate: always keep the newest one.

## Devices

| Call | What it does |
|---|---|
| `POST /api/devices` | `{ device_id, platform, name, brand, model, os_name, os_version, app_version, push_token? }`: adds this device to the person's list or updates it (every app start). `platform` is `android`, `ios`, `macos`, `windows`, `linux` or `web` |
| `GET /api/devices` | The person's devices, the ones that are on first: the fields above plus `presence` (`online`, `away`, `offline`), `last_seen_at`, and `current` (the device making the call) |
| `DELETE /api/devices/:id` | Signs that device out: its sign-ins end, and it hears `DeviceSignedOut` and leaves |

## Errors

A failed call answers a `4xx` or `5xx` status. Branch on the status, never on the wording.

**Ask for problem details.** Send `Accept: application/problem+json` (alongside
`application/json`) and every error, from every route, comes back in one shape
([RFC 9457](https://www.rfc-editor.org/rfc/rfc9457)):

```json
{
  "type": "about:blank",
  "title": "Conflict",
  "status": 409,
  "code": "conflict",
  "detail": "a publish is already running",
  "request_id": "3f6c…"
}
```

- `code` is stable and safe to branch on: `bad_request`, `unauthorized`, `payment_required`,
  `forbidden`, `not_found`, `method_not_allowed`, `conflict`, `gone`, `payload_too_large`,
  `unsupported_media_type`, `unprocessable`, `rate_limited`, `upstream_failed`, `unavailable`,
  `upstream_timeout`, `internal`.
- `detail` is a sentence for the person, already in plain words; show it as it is.
- New clients should always ask for problem details. Without the header, routes answer errors
  as they always have (mostly a plain sentence as the body), which the current web and native
  apps parse; that form is kept for them and isn't extended.

| Status | Means | What a client does |
|---|---|---|
| 400 / 422 | The request was wrong or can't be done as asked | Show `detail` next to what caused it |
| 401 | Not signed in, or the token ran out | Refresh and retry once |
| 403 | Signed in but not allowed (not an admin of that project) | Hide or disable the action |
| 404 | Doesn't exist, or the person can't see it | Treat both the same |
| 409 | Clashes with the current state (busy, already done) | Refresh what's shown; offer to try again |
| 502 / 503 / 504 | Something Nomi depends on failed (a model provider, Shipyard) | Offer to try again |

## Lists

Lists that grow (a chat's messages) page backwards from the newest: `?limit=` and
`?before=<id of the oldest one you have>`. An answer shorter than `limit` means there's nothing
older. Small lists (a project's members, its secrets) come whole.

## Chats

| Call | What it does |
|---|---|
| `POST /api/sessions`, `GET /api/sessions` | Start a chat; list the person's chats |
| `PATCH /api/sessions/:id`, `DELETE /api/sessions/:id` | Rename; delete |
| `GET /api/sessions/:id/messages?limit=&before=` | A page of messages, oldest first |
| `POST /api/sessions/:id/messages` | `{ text }` → `202` with the person's message. The crew answers asynchronously (see *Live updates*) |
| `GET /api/sessions/:id/messages/:message_id` | One message (after a `MessageCreated` or `MessageUpdated` event) |
| `PUT /api/sessions/:id/messages/:message_id/approval` | `{ decision: "approve" \| "deny", remember }` on an approval card |
| `POST /api/sessions/:id/messages/:message_id/retry` | Try a failed run again (a `run_failed` block with `retry`); removes the error bubble |
| `GET .../delete`, `DELETE /api/sessions/:id/messages/:message_id` | What deleting it would remove; delete it |

A message is `{ id, sender: "user" | "assistant", content, content_blocks, created_at,
agent_display_name, … }`. `content` is always readable text. `content_blocks`, when present, is
what to draw instead: a list of blocks, each tagged by `kind`. A client draws the kinds it knows
and falls back to `content` for the rest, so new kinds never break an old client. The kinds
(`backend/crates/nomi-agent-core/src/content_block.rs`, mirrored in `frontend/src/lib/types.ts`):

| `kind` | Draw it as |
|---|---|
| `file_write`, `file_writes`, `file_delete` | A file change (with the previous content for a diff) |
| `todo_list`, `plan` | A checklist; a plan card that opens its versions |
| `table`, `view` | A table; a list, timeline, chart or form |
| `questions` | Questions with ready answers; the answer is sent as the person's next message |
| `approval_request` | Approve / deny (see the approval call above) |
| `suggestions` | Buttons whose text is sent as the next message; hide once the person has replied |
| `run_failed` | An error bubble; Try again when `retry` is set |
| `quota_notice`, `connector_connect`, `workspace_connect` | A card pointing to plans or a setup page |
| `project_link`, `reminder`, `voice`, `reasoning` | A link to a project chat; a reminder; a recording; collapsed thinking |

## Projects

`/api/projects/:id/…` holds a project's files, runtime, issues, members, secrets, database,
deployment (`deploy`, `deploy/publish`, `deploy/logs`, …), domain and what the crew may do on it
(`agent-permissions`). Everyone who can open a project reads; only its admins (whoever started
it, project admins, the organization's owners and admins) change things, and get `403` otherwise.
`GET /api/projects/:id/deploy` says which the person is (`role`).

## Live updates

Nothing polls. A client opens one WebSocket and subscribes to what's on screen:

- `GET /api/realtime/ws` (authenticated like any call). Send
  `{ "op": "subscribe", "topic": "chat/<session_id>" }` (also `project/<project_id>` and
  `user/<user_id>`; `unsubscribe` to stop); the server answers `{ "type": "subscribed" }`,
  `denied` or `unavailable` (try again later), then `{ "type": "event", "topic", "payload" }`
  for each event. `{ "op": "ping" }` → `{ "type": "pong" }`.
- Chat events (`payload.kind`): `MessageCreated` and `MessageUpdated` (fetch that message),
  `Delta` (the crew is writing), `TurnCompleted` / `TurnFailed` (the turn ended; a failure
  also posts its `run_failed` message), `AgentPhaseChanged` (thinking, calling a tool, waiting),
  `AgentDelegationUpdated` (a background job moved on), `SessionRenamed`, `PersonalityChanged`.
- Presence: the socket's broker (MQTT) client id is the device id (`<deviceId>-<tabId>` for a
  browser tab, which sends `X-Tab-Id`). The app sends `{ "op": "presence", "state": "away" }`
  when it goes to the background and `"online"` when it's back; closing the socket makes the
  device offline (the connection's MQTT last will).
- User events: `NotificationCreated`, `ChatReply` (a reply landed in another chat),
  `MessagesDeleted` / `MessageUpdated` with their `session_id` (the same change as on the chat's
  topic, for devices not showing that chat), `DevicePresence` (`device_id`, `presence`,
  `last_seen_at`) and `DeviceSignedOut` (`device_id`: if it's yours, sign out). The
  person's activity feed (`GET /api/me/activity/ws`) sends `ChatActivity`: a chat elsewhere
  finished, failed or needs them.

Events say *what* changed, not the whole new state: fetch the message or reload the list they
name. After a reconnect, reload what's on screen; events missed while disconnected aren't replayed.

## Changing the API

- Adding a field, a route, an event or a block `kind` is not a breaking change: clients ignore
  what they don't know.
- Renaming or removing a field, changing what a status means, or changing a block's shape is
  breaking. Add the new form next to the old one, move the clients, then remove the old form
  (and say which release did).
- Errors from new routes are plain sentences in the person's terms: the problem-details layer
  gives every client the same shape, so a route never invents its own error JSON.
