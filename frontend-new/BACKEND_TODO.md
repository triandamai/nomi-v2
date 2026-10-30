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

### Dislike reasons and a note
- **Needed by:** *Chat responses / Feedback*: DislikeFeedback.
- **Why:** the person picks any number of reasons and can add a note. They can also attach the chat.
- **What exists today:** `PUT /api/sessions/:id/messages/:message_id/feedback`.
  - Body: `{ rating, reason? }`, with one reason from `wrong_memory | not_relevant | too_long | other`.
  - No note.
- **Proposed contract:** the same route.
  - Body: `{ rating: "down", reasons: ("inaccurate"|"ignored"|"wrong_crew"|"too_long"|"outdated"|"unsafe"|"other")[], note?: string, include_chat?: bool }`.
  - The old single `reason` is still accepted.
- **Until then:** one reason is sent and the note is left out.

### Regenerate a reply, and its versions
- **Needed by:** *Chat responses / Actions*: RegenerateMenu and VersionSwitcher.
- **Why:** the person asks for the reply again, with the same model or another one, and moves between the versions.
- **What exists today:**
  - `POST /api/sessions/:id/messages/:message_id/retry` only re-runs a failed run (`RunFailed` block).
  - Messages have no versions.
- **Proposed contract:**
  - `POST /api/sessions/:id/messages/:message_id/regenerate` with `{ model_id? }` returns `202 { version, total }`.
  - The reply streams over the session WS as usual.
  - `GET …/messages/:message_id/versions` returns `[{ version, created_at, model }]`.
  - `GET …/messages/:message_id?version=n` returns that version.
- **Until then:** no regenerate or version switcher.

### Edit a sent message
- **Needed by:** *Chat responses / Edit*: MessageEditor, and MessageUser `edited` / `version`.
- **Why:** the person fixes what they asked, and the reply after it runs again.
- **What exists today:** nothing. A message can only be deleted (`DELETE …/messages/:message_id`).
- **Proposed contract:**
  - `PATCH /api/sessions/:id/messages/:message_id` with `{ text }` returns `{ version, edited_at }`.
  - It keeps the earlier text as a version and queues a new turn from that message.
- **Until then:** no edit.

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

### Share a chat by link
- **Needed by:** *Chat / Share*: ShareDialog and SharePanel.
- **Why:** a read-only link with an expiry and three settings.
  - `ShareSettings` = `{ scope: "reply" | "chat", mask, anyone }`: share one reply or the whole chat, mask personal details, and anyone with the link or only signed-in people.
- **What exists today:** nothing.
- **Proposed contract:**
  - `POST /api/sessions/:id/shares` with `{ scope: "reply"|"chat", message_id?, mask: bool, anyone: bool, expires_in_days?: number|null }` returns `{ id, url, expires_at }`.
  - `DELETE /api/sessions/:id/shares/:share_id` removes it.
  - `GET /api/shared/:token` is public and returns the messages.
- **Until then:** no share item.

### Export a project to GitHub
- **Needed by:** *Build / Export*: ExportPanel push.
- **What exists today:**
  - `POST /api/projects/:id/import/github` (the other direction).
  - `GET /api/projects/:id/snapshot` (download, which already works).
- **Proposed contract:**
  - `POST /api/projects/:id/export/github` with `{ repo: "owner/name", create?: bool, private?: bool }` returns `{ url, commit }`.
  - It needs a GitHub connection (a new `/api/connections/github`), since import today reads a public zip and holds no token.
- **Until then:** download only.

### Publish requests
- **Needed by:** *Build / Together*: PublishRequest and RequestQueue.
- **Why:** an editor asks to publish, and the owner sees the queue and approves or declines.
- **What exists today:** `POST /api/projects/:id/deploy/publish`, which publishes straight away for whoever is allowed to.
- **Proposed contract:**
  - `POST /api/projects/:id/deploy/requests` with `{ note? }` creates a request.
  - `GET /api/projects/:id/deploy/requests` lists them.
  - `POST …/requests/:rid` with `{ action: "approve" | "decline" }` resolves one; approve runs the publish.
  - Each change goes out on the project WS.
- **Until then:** only people who may publish see the button, and nobody else gets a request.

### What a connect prompt says
- **Needed by:** *Chat responses / Connect Prompt*: ConnectPrompt.
- **Why:** the prompt says which crew member needs the tool and why, and exactly what the tool can read, before the person connects it.
- **What exists today:** the `connector_connect` and `workspace_connect` content blocks.
  - `connector_connect` carries only `connector` and `reason`.
  - `workspace_connect` carries `reason` and `services`.
- **Proposed contract:** both blocks also carry `agent_type` (who asked), `why` (one line, in the person's language) and `reads` (what the connection can read).
- **Until then:** `frontend/` keeps its own connect cards; ConnectPrompt isn't used.

### A table's title and source
- **Needed by:** *Chat responses / Table*: DataTable.
- **Why:** the table is named above its rows (and in its downloaded file), and says where its numbers came from underneath ("Rp · from Dana · 142 transactions").
- **What exists today:** the `table` content block carries `variant`, `columns` and `rows`, but no title or source.
- **Proposed contract:** the block also carries `title` (one line, in the person's language) and `source` (optional, one line).
- **Until then:** `frontend/` keeps its own table block.

