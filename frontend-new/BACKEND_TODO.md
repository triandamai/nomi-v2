# Backend changes the new frontend needs

`frontend-new/` uses the backend API exactly as it is. When a flow in the design needs something
the API doesn't offer, it's written down here instead of being built or faked in the UI. Each
entry is checked against the backend's real routes first, and names the flow and step that need
it.

Nothing here has been implemented: these are requests for the backend.

## Entry format

### Short name
- **Needed by:** flow and step in `Nomi.pen` (e.g. *Flow / Chat — Share & delete*, step 2).
- **Why:** what the person can't do without it.
- **What exists today:** the closest current endpoint or behaviour, with its path.
- **Proposed contract:** method, path, request and response shape (a suggestion, open to change).
- **Until then:** what the frontend shows instead (the step is left out, never faked).

## Entries

Found by the design-system parity audit (`docs/superpowers/specs/2026-10-30-design-system-parity-audit.md`).
The old `frontend/` will use the same endpoints.

### Dislike reasons and a note — done
- **Needed by:** *Chat responses / Feedback*: DislikeFeedback.
- **Built:** `PUT /api/sessions/:id/messages/:message_id/feedback` takes `{ rating, reasons?, note?, include_chat? }` (migration `0091_feedback_reasons.sql`).
  - The reasons are DislikeFeedback's (`inaccurate`, `ignored`, `wrong_crew`, `too_long`, `outdated`, `unsafe`, `other`) and the memory sheet's (`wrong_memory`, `not_relevant`).
  - The single `reason` older apps send still works and lands in `reasons`.
  - A thumbs-up keeps none of them. The note is at most 2000 characters.

### Regenerate a reply, and its versions — done
- **Needed by:** *Chat responses / Actions*: RegenerateMenu and VersionSwitcher.
- **Built:** `backend/crates/nomi-server/src/routes/message_rerun.rs`, migration `0093_message_versions.sql`.
  - `POST /api/sessions/:id/messages/:message_id/regenerate` with `{ style?: "again"|"shorter"|"longer"|"list" }` returns `202 { message_id, version, total, edited_at }`.
    - `message_id` is the person's message whose turn runs again.
    - The style's instruction goes to the crew with the message (`nomi-agent-core/src/prompts.rs`); the message itself doesn't change.
  - `GET …/messages/:message_id/versions` returns `{ message_id, versions: [{ version, text, replies, created_at }], total }`. The live take is number `total`.
  - Messages in the list carry `versions`: how many earlier takes a person's message has.
  - Only the latest turn runs again, and only when nothing is running in the chat (409 otherwise).
  - Before it runs, what the turn made is taken back (replies, transactions, reminders, plans, scheduled tasks), as deleting the message would, so it isn't done twice.
- **Still open:** RegenerateMenu's "with another model" needs the chat's model list in the menu. The app can already set the chat's model first (`PUT /api/sessions/:id/model`), but the old frontend's menu offers styles only for now.

### Edit a sent message — done
- **Needed by:** *Chat responses / Edit*: MessageEditor, and MessageUser `edited` / `version`.
- **Built:** `PATCH /api/sessions/:id/messages/:message_id` with `{ text }` returns `{ message_id, version, total, edited_at }`.
  - It's the same run-again as regenerate, with the new text saved and `edited_at` set.
  - The earlier text and its replies stay as a version. Messages carry `edited_at`.

### Pin a chat — done
- **Needed by:** *Chat / Menu*: ChatMenu pin and unpin, and the pinned group in the sidebar.
- **Built:** `PATCH /api/sessions/:id` with `{ pinned: bool }` (the chat's owner only) returns `{ title, pinned, in_project_id }`.
  - `GET /api/sessions` returns `pinned` and `pinned_at` for each chat; newest pin first is the list's job.
  - Migration `0090_session_pins_and_projects.sql`.

### Move a chat into a project — done
- **Needed by:** *Chat / Menu*: MoveDialog.
- **Built:** `PATCH /api/sessions/:id` with `{ in_project_id: uuid | null }` (the chat's owner only); `null` takes it out.
  - The field is `in_project_id`, since `project_id` in the list already means "this chat is that project's own chat".
  - Any project the person can open will do, since the chat itself stays private.
  - A project's own chat answers 400: it stays with its project.
  - `GET /api/sessions` returns `in_project_id`.

### Share a chat by link — done
- **Needed by:** *Chat / Share*: ShareDialog and SharePanel.
- **Built:** migration `0092_chat_shares.sql`, routes in `backend/crates/nomi-server/src/routes/shares.rs`.
  - `POST /api/sessions/:id/shares` with `{ scope: "reply"|"chat", message_id?, mask?, anyone?, expires_in_days?: 1|7|30 }` returns `201 { id, token, scope, message_id, mask, anyone, expires_at, preview, chat_messages }`.
    - `mask` and `anyone` default to true. Without `expires_in_days` the link never expires.
    - `preview` is what the link opens with, masked when `mask` is on.
  - `PUT /api/sessions/:id/shares/:share_id` with the same body changes the link's settings; `DELETE` revokes it.
  - `GET /api/shared/:token` returns `{ title, scope, masked, messages: [{ sender, agent_display_name, content, created_at }], expires_at }`.
    - It's public when `anyone` is on; otherwise any signed-in person may open it.
    - A revoked, expired or unknown link is a 404.
  - Only the chat's owner shares it.
- **Still open:** masking hides amounts ("Rp •••"), email addresses and phone numbers. SharePanel's help text also promises names as initials; the server has no reliable way to tell a name from other words, so names aren't masked yet.

### Export a project to GitHub
- **Needed by:** *Build / Export*: ExportPanel push.
- **What exists today:**
  - `POST /api/projects/:id/import/github` (the other direction).
  - `GET /api/projects/:id/snapshot` (download, which already works).
- **Proposed contract:**
  - `POST /api/projects/:id/export/github` with `{ repo: "owner/name", create?: bool, private?: bool }` returns `{ url, commit }`.
  - It needs a GitHub connection (a new `/api/connections/github`), since import today reads a public zip and holds no token.
- **Until then:** download only.

### Publish requests — done
- **Needed by:** *Build / Together*: PublishRequest and RequestQueue.
- **Built:** migration `0094_publish_requests.sql`, routes in `backend/crates/nomi-server/src/routes/publish_requests.rs`.
  - `POST /api/projects/:id/deploy/requests` with `{ note?, environment? }` returns `201` and the request. Admins publish themselves (`409`), and a person has one open request at a time (`409`).
  - `GET /api/projects/:id/deploy/requests` returns `{ requests: [{ id, environment, note, status, requested_by, requested_by_name, decided_by_name, decided_at, publish_id, created_at }] }`, open ones first, then newest first.
  - `POST …/requests/:rid` with `{ action: "approve" | "decline" | "cancel" }`. Admins approve or decline; the person who asked cancels. Approve publishes as that admin; if the publish can't start, the request stays open and the error comes back.
  - Each change goes out on the project stream as `DeployChanged` with event `publish_request.changed`. The admins are notified of a request, and the person who asked of the answer.

### What a connect prompt says — done
- **Needed by:** *Chat responses / Connect Prompt*: ConnectPrompt.
- **Built:** `connector_connect` and `workspace_connect` blocks carry `agent_type`: who needs it (optional; older cards have none).
  - The why and reads lines are the frontend's to write, in the person's language, from the agent, the service and `reason`. The backend sends facts, not sentences, as it does everywhere else.
  - Older cards: the connector's own agent (`frontend/src/lib/connectors.ts`), or Tara for Google.

### A table's title and source — done
- **Needed by:** *Chat responses / Table*: DataTable.
- **Built:** the `table` block carries `title` and `source`, one line each (optional; older tables have none).
  - The inspector fills both: "Query results" with its row count, "Log lines" with how much of the log it shows.
  - `frontend/` draws a titled data table with DataTable and keeps its old block for older ones.

### The person's share links, to revoke — done
- **Needed by:** *Chat / Share*: SharePanel's footer says links can be revoked "any time in Settings › Data".
- **Built:** `GET /api/shares` returns `{ shares: [{ id, session_id, title, scope, message_id, mask, anyone, expires_at, created_at }] }`: the person's links that still work, newest first. Revoke one with `DELETE /api/sessions/:session_id/shares/:id`.
- **Still open:** the Settings › Data page itself, in both apps.
