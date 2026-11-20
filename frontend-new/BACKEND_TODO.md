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
- **Still open:** masking hides amounts ("Rp •••"), email addresses and phone numbers. SharePanel's help text also promises names as initials; the server has no reliable way to tell a name from other words, so names aren't masked yet. File names in a shared message aren't masked either (they can carry a name: UXP-001, M3).

### Export a project to GitHub — done
- **Needed by:** *Build / Export*: ExportPanel push.
- **Built:** migration `0095_github_connections.sql`, `backend/crates/nomi-server/src/github.rs` and routes in `backend/crates/nomi-server/src/routes/github.rs`.
  - **Connecting:** `GET /api/connections/github` returns `{ oauth, connection: { login, kind, created_at, last_used_at } | null }`. `oauth` says whether Nomi has an OAuth app (Admin › Configuration › GitHub, or `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`).
    - With one: `POST /api/connections/github/start` returns `{ url }` to send the person to; the callback page posts `{ code, state }` to `POST /api/connections/github/callback`.
    - Always: `PUT /api/connections/github` with `{ token }` connects with a token the person made (repository access). It's checked with GitHub first.
    - `DELETE /api/connections/github` forgets it.
  - **Pushing:** `POST /api/projects/:id/export/github` with `{ repo: "owner/name" (or a github.com link), create?: bool, private?: bool (default true), message? }` returns `{ url, commit, branch, created }`.
    - One commit of the project's files on the default branch, as the whole tree, never forced. `.env` files, build output and local databases stay out, as when publishing.
    - Without a connection it's `409`; a missing repository without `create` is `404`.
- **Still open:** the connect and push screens (a GitHub row in Connections, the callback page at `/connections/github/callback`, ExportPanel's push), in both apps.

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
  - `frontend/`: Account › Data (`/account#data`) lists them with Revoke.
- **Still open:** the Settings › Data page in v2.

Found while planning the working v2 (sign in, chat and build against the real API, 2026-11-10).

### Sign in or create with an email code, no password — done
- **Needed by:** *Sign in or create* (`vDaAQ`) and *Enter code* (`pUDXz`).
- **Built:** `POST /api/auth/start {email, language?, invite_code?}` → `{verification}` for an existing or a new address alike (400 `invalid_email` / `invalid_invite`).
  - An existing account gets a sign-in code (`purpose: "login"`). A new address gets a code with no account yet (`purpose: "register"`); the account is made only when the code is entered.
  - `POST /api/auth/verify` now also returns `is_new`. A new account has no password (`password_login` off), its email confirmed, and a personal workspace ("{name}'s space", `is_personal`), or the invite's workspace.
  - Password `login` and `register` work as before. Migration `0109_email_code_sign_in.sql`.
  - `frontend/`: the login page has "Email me a code instead"; `/verify` finishes it and sends new accounts to first run.
  - v2: `/sign-in` starts with the email alone; "Use a password instead" keeps the password form.

### Change the password while signed in — done
- **Needed by:** *Settings / General*: "Password · Not set. Add one to sign in without Google".
- **Built:** `PUT /api/auth/password {current_password?, new_password}` → 204. `current_password` is required when a password is set (403 `wrong_password`; 400 `too_short` under 8 characters). Setting one turns password sign-in on; other devices stay signed in.
  - The emailed reset now turns password sign-in on too.
  - `frontend/`: Account › Sign-in methods has a set/change password form.
  - v2: Settings › General's password row opens a dialog to set or change it.

### Switch the active workspace — done
- **Needed by:** *Workspace switcher* (`pKiO2`) and the account menu.
- **Built:** `PUT /api/auth/active-org {org_id}` → `{access_token}` carrying that org (404 `not_a_member`).
  - The choice is kept (`users.active_org_id`, migration `0110_active_org.sql`): later sign-ins and refreshes keep it while the person is still an active member, else fall back to their first workspace.
  - `frontend/`: Organizations marks the active workspace and offers "Switch"; the new token is stored like sign-in's.

### A project's versions, listed and restorable — done
- **Built:** `GET /api/projects/:id/versions?before=&limit=` → `{ versions: [{ version, by: { name, agent }, at, summary, files, restorable, restored_from, current }], current, has_more }`, newest first; `summary` is the prompt Koda's version answered.
  - `POST /api/projects/:id/versions/:v/restore` → 202 `{ version, restored_from }`: puts the files back as a new version made by the person (builders only; 409 for a version from before files were kept).
  - Migration `0120_project_version_files.sql`: `project_versions` and `project_version_files`; every file's bytes are kept once per content (`versions/<project>/<sha256>` in project storage, `project_files.blob_sha`), recorded on each version bump. Older files are kept on the project's next change.
  - `frontend/`: the project's Members sheet has a Versions tab with Restore.

### Search chats, memory and files — done
- **Built:** `GET /api/search?q=&kinds=chat,memory,file&limit=20` → `{ results: [{ kind, id, title, snippet, session_id?, project_id?, updated_at }] }`: case-insensitive matching (ILIKE, wildcards literal) over the person's own chats in the active workspace (title and messages), their memory, and file paths in that workspace's projects they can open. Title and file-name hits first, then newest.
  - `frontend/`: a search dialog in the app shell (⌘K / Ctrl+K, Search in the drawer and the phone's app bar).
- **Still open:** file contents aren't searched (paths only).

Found while building the app shell (App / Home, the account menu and the workspace switcher, 2026-11-10).

### A person's pending workspace invites — done
- **Built:** `GET /api/invites` → `{ invites: [{ code, org_id, org_name, inviter, role, projects, expires_at, project: { id, name } | null }] }`: open invites to any of the person's addresses, for workspaces they aren't in yet, newest first.
  - `frontend/`: Organizations lists them under Invitations with Join and Decline.

### Say which workspace is personal — done
- **Built:** `is_personal` on each workspace in `GET /api/orgs` and `POST /api/orgs` (and on `GET /api/orgs/:id`).
  - `frontend/`: Organizations shows a "Personal" tag.

Found while building sign in and create account (Phase 1, 2026-11-10).

### Sign up from a project invite puts the person on the project too — done
- **Built:** `place_in_org` (Join) now adds the `project_members` row with the invite's project role, records `used_by` and `invited_by`, and the project chat announces the join. It covers password sign-up, Google sign-up and new email-code accounts (all go through `place_in_org`).

### Who shared a link, when, and whether the viewer is its owner — done
- **Built:** migration `0117_shared_page_views_and_reports.sql` (`chat_shares.views`), in `routes/shares.rs`.
  - `GET /api/shared/:token` adds `shared_by` (the sharer's name, or its initials "M. P." when masked) and `shared_at`.
  - When the bearer made the link it also answers `own: true`, `share_id`, `session_id` (to stop sharing with `DELETE /api/sessions/:session_id/shares/:share_id`) and `views`.
  - Every open by someone other than the owner counts as a view.
  - `frontend/`: the shared page shows the meta line and the owner's banner (views, Copy link, Stop sharing).

### Reporting a shared page — done
- **Built:** `POST /api/shared/:token/report` with `{ reason: "spam"|"abuse"|"personal_info"|"illegal"|"other", note? }` → `201`, in `routes/shares.rs`; table `shared_page_reports` (migration 0117).
  - Open without a sign-in (a link only for signed-in people still needs one); a revoked, expired or unknown link is a 404.
  - Rate-limited in the database: 5 reports an hour per signed-in person, or per hashed `X-Forwarded-For` address, then `429`. Notes keep their first 1000 characters.
  - Admins (support, view) list them with `GET /api/admin/shared-reports` → `{ reports: [{ id, share_id, token, session_id, title, shared_by, reason, note, revoked, created_at }] }`, newest first.
  - `frontend/`: "Report this page" in the shared page's footer, with the reasons and a note.

### Push for reminders, and quiet hours — done
- **Built:** migration `0124_reminder_push_and_quiet_hours.sql`; `GET|PUT /api/notifications/reminder-preferences` returns `{ push_reminders: bool, quiet_hours: { start: "22:00", end: "06:00" } | null }` (local times in the person's zone; a start after the end runs over midnight). `GET|PUT /api/notifications/preferences` keeps answering with just the two email fields, as before; both PUTs change only the fields sent and accept any of the four (`quiet_hours: null` turns them off).
  - The reminders worker pushes each reminder as it goes off (`kind: "reminder"`, opens its chat) unless `push_reminders` is off.
  - A reminder due inside quiet hours is held (`reminders.held_until`) until they end, then goes off; its schedule (`due_at`) doesn't move. Changing quiet hours lets held reminders go by the new ones.
  - `frontend/`: the Inbox's settings have "Push for reminders" and "Quiet hours" with From and Until.
  - v2: Settings › General's Notifications has Reminders and Quiet hours.

Found while building the Inbox (2026-11-12).

### Profile insights and when the person is active — done
- **Built:** `GET /api/profile` (own profile only, never the public one) adds `active_hours: { from, to } | null` and `insights: [{ kind, title, body, action? }]`, in `routes/profile_insights.rs`, worked out from `llm_usage` over 30 days with no model call. Titles and bodies come in the person's language.
  - `active_hours`: the two hours with the most tokens in the person's time zone (null under 1,000 tokens).
  - `late_hours`: when at least half the tokens land between 21:00 and 04:00 ("68% of your tokens land after 21:00").
  - `cheaper_model`: when Nomi's cheapest priced chat model would have answered at least 5 of the last 10 quick questions (replies of 300 tokens or fewer) for less, and that saves at least 5% of the month's spend; the action opens `/models`. It says "for less", not "just as well": nothing judges the answers.
  - Also fixed: the profile's top models read a column that doesn't exist, so they were always empty.
  - `frontend/`: Profile shows an Insights panel, and the heatmap's note says "Most active Tue · 21:00–23:00".

Found while building Settings › Profile (2026-11-12).

### Account settings General can't keep yet — done
- **Built:** migration `0125_account_settings_and_deletion.sql`, in `routes/account.rs` and `routes/profile.rs`.
  - `POST /api/account/email` with `{ email }` sends a code to the new address (`409 email_taken`, `400 same_email`) and answers like sign-in (`challenge_id`, masked `email`, `expires_in`…); `POST /api/account/email/verify` with `{ challenge_id, code }` moves the account to it → `{ email }`. These codes never sign anyone in.
  - `GET|PUT /api/preferences` adds `formats` (a region like "en-GB"; null follows the language), `default_agent` (an `agent_type`; null is Nomi), `enter_sends` (default on) and `show_reply_tokens` (default off). An empty string clears the first two.
  - `DELETE /api/account` with `{ password }` or `{ challenge_id, code }` (from `POST /api/account/delete/code`) → `204`. The account is kept as a row (chats, projects and usage others share point at it), with `users.deleted_at` set and its sign-in, refresh tokens, devices (each hears `DeviceSignedOut`), memberships, profile, memories, reminders, connections and own keys removed; the address is free again. A workspace only they're in closes with it; one they're the only owner of with other people in it answers `409 { error: "sole_owner", orgs: [{ id, name }] }` until it's handed over or closed.
  - `frontend/`: Account has Change email (with the code step), Formats and the chat rows (the composer follows Enter sends), and Delete account with a confirmation and the password or an emailed code.
  - **Still open:** the old frontend shows no tokens under replies, so `show_reply_tokens` is kept but nothing reads it there; a closed account's live apps aren't taken down.
  - v2: General's Chat group has "New chats start with" (Home's To) and "Enter sends" (every composer). Formats and tokens under replies wait for v2 to follow them.
  - v2: General's Email row changes the address with the code; Delete account closes it with the password or an emailed code, naming workspaces to hand over.
  - **Still open (backend):** the turn's routing doesn't read `default_agent`, so a chat started elsewhere (a phone app, the API) still starts with Nomi.

Found while building Settings › General (2026-11-12).

### Changing plan, and what was paid — done
- **Built:** migration `0128_plan_checkout_and_waitlist.sql` (`plans.price_idr`, `plan_payments`, `plan_waitlist`), in `routes/plan_checkout.rs`.
  - `POST /api/plans/:id/checkout` → `{ url }`: a Xendit invoice for a month of the plan (`price_idr`, set in Admin → Plans), returning to `/billing`. `409 already_on_plan`, `409 not_for_sale` (no price: join its waitlist), `503 payments_not_configured` while Xendit has no secret key, `502 gateway_error`.
  - `POST /api/plans/:id/waitlist` → `{ on_waitlist: true }` (asking again changes nothing). `GET /api/plans` adds `waitlist` (plan ids) and `checkout_available`; each plan has `price_idr`.
  - `POST /api/payments/xendit/webhook`: Xendit's invoice callback, only with the webhook token set (`x-callback-token`) and only for the amount asked. Paid moves the person onto the plan (in the change history), tells them, and takes them off its waitlist; expired or failed is kept; repeats change nothing.
  - `GET /api/usage` adds `payment_method` (the last one used, "BANK_TRANSFER · BCA" or null), `paid_idr` (paid in that month) and `usd_to_idr` (Admin → Payment's "Rupiah per dollar", `USD_TO_IDR`, else 16,000).
  - `frontend/`: Billing's plan cards have Upgrade (to Xendit) or Join the waitlist / On the waitlist; spend reads "Rp … ≈ $…" with "You paid Rp …", and a Payment method tile. Admin → Plans sets the price in rupiah.

Found while building Settings › Plan & usage (2026-11-12).

### A provider key's balance — done
- **Built:** migration `0129_provider_key_status.sql`. Every connection in `GET /api/llm/connections` (and the admin list) has `status: "ok" | "balance_empty" | "rejected"` and `checked_at` (null until a call is made with it).
  - Each model call made through a connection (a person's own, or the instance's behind Nomi's models) records it as it ends: a reply is `ok`; a 402 / `insufficient_quota` / "credit balance is too low" is `balance_empty`; a 401 / 403 / invalid key is `rejected` (`LlmError::key_health` in nomi-llm, the same reading the chat's error messages use). Outages and rate limits leave it as it was. Changing the key, address or headers starts it over at `ok`.
  - `frontend/`: Providers shows "Balance empty" or "Key rejected" on the row, with a line to pick one of Nomi's models meanwhile.

Found while building Settings › Plan & usage (2026-11-12).

### What each model can do — done
- **Built:** every model in `GET /api/llm/model-options` and in `POST /api/llm/connections/:id/models` (and the admin one) has `context` (tokens, from the catalog; null when it isn't there), `capabilities` (`fast`, `reasoning`, `vision`, `code`) and `recommended`.
  - Vision comes from what the catalog (or, for Nomi's models, the admin's media inputs) says the model reads; the rest from the model's name (`nomi_llm::catalog::capabilities`), since models.dev's data in the catalog carries no such flags. `recommended` is a short hand-kept list of current models (`RECOMMENDED_MODELS`), matched wherever they come through ("anthropic/claude-sonnet-5.5" too).
  - `frontend/`: Manage models shows each model's context and tags and "Recommended"; the chat's model picker shows the context.
  - **Still open:** the catalog has no reasoning/tool flags, so the tags beyond vision are read from names; regenerating it from models.dev with its `reasoning` field would make them exact.

Found while building Settings › Models & providers (2026-11-13).

### Memory settings, the profile and who learned what — done
- **Built:** migration `0132_memory_settings_and_profile.sql` (`memory_settings`, `memory_profiles`, `memory_items.learned_by`, `memory_jobs.agent_type`), in `routes/memory.rs` and `nomi-agent-core/src/memory_settings.rs`.
  - `GET|PUT /api/memory/settings` → `{ learn_from_chats, keep_profile, search_past_chats, memory_budget, profile_budget, memory_used, profile_used }` (budgets and use in tokens, ~4 characters each; defaults on, 8,000 and 1,000). PUT changes only the fields sent; `400 invalid_budget` outside 500–100,000 / 100–4,000.
  - The crew honours them: `learn_from_chats` off queues nothing and drops what was queued; `search_past_chats` off leaves the "recently" note from other chats out of every prompt and stops writing it; recall never hands an agent more than `memory_budget`, and saving past it (or lowering it) archives the weakest memories; the profile is read and written within `profile_budget`.
  - `GET|PUT /api/memory/profile` → `{ text, tokens, updated_at }`. Nothing kept a summary of the person before (personality is how Nomi talks), so it's new: every agent that uses memory reads it while `keep_profile` is on, the memory worker rewrites it about daily from the profile so far and the strongest memories (prompt in `prompts.rs`, so the person's own edits carry over), and PUT is the person's wording (`400 too_long` past the budget; empty clears it).
  - Each memory in `GET /api/memory` has `learned_by` (the agent type whose reply it came from; null for older ones).
  - `frontend/`: Memory has a "You" profile editor with its size, and Memory settings (the three switches, both budgets with a use bar); each memory says "Learned by Raga".
  - v2: Memory & context has You (the profile, edited in place), Learned by on each topic and memory, and the Learning and Budget column.

Found while building Settings › Memory & context (2026-11-13).


### More apps to connect, MCP servers, and a connection's health — done
- **Built:** migration `0133_connection_health_and_mcp.sql`, in `routes/connections_catalog.rs` and `routes/connections.rs`.
  - `GET /api/connections/catalog` → `[{ id, name, category, for_agent, access, available }]`, a list kept in code (16 apps across work, building, health, money, home_media). `available` is true only for Hevy, GitHub and, when Google sign-in is set up, Google Workspace.
  - `GET /api/connections/mcp` → `{ servers: [{ id, name, url, created_at }], callable: false }`; `POST` with `{ name, url }` → `201` (`400 missing_name | missing_url | invalid_url | command_not_supported`, `409 already_added | too_many` past 20); `DELETE /api/connections/mcp/:id` → `204` (the id is in the path). Only http/https addresses are kept; a `command` is refused, never stored or run.
  - `GET /api/connections`: `google` adds `status` (`ok` | `expired`), `expired_at`, `last_used_at`, `used_by: ["workspace"]` and `service_access: [{ service, access }]` (Gmail and Calendar `read · write · ask`, Docs and Sheets `read · write`, Drive `read`); `hevy` adds `status`, `expired_at`, `used_by: ["health"]`. `GET /api/connections/github`'s connection adds `status`, `expired_at`, `used_by: []` (the person pushes, not the crew).
  - A lapsed sign-in is kept and marked `expired` instead of being forgotten: Google when it refuses a refresh or a call, Hevy when it refuses the key (cleared when Hevy takes it again), GitHub when it refuses a push. Connecting again clears it. Google records `last_used_at` on each use.
  - `frontend/`: Connections lists the catalog (coming soon unless it connects), GitHub with its login, per-service access under Google, "Needs you" with Reconnect on a lapsed one, who uses each, and an MCP servers section with an Add form.
  - **Still open:** nothing in the crew can call MCP tools yet, so servers are only stored (`callable: false`); the catalog's other apps need their own connectors.

Found while building Settings › Connections (2026-11-13).

### A person's own crew: members, switching one off, permissions, soul.md, activity — done
- **Built:** `PUT /api/agents/:type {enabled}` → `{agent_type, enabled}` (400 for Nomi, 404 for a member not in their crew). `GET /api/agents` adds `enabled`, `is_own`, `is_default`, and `state: "paused"` for a switched-off member. The router never offers a switched-off member to the classifier, an `@mention` of one posts "… is switched off …" from Nomi and Nomi answers, an ongoing conversation with one ends, and delegation / `ask_agent` to one is refused with a reason.
  - `GET|PUT /api/agents/:type/permissions` `[{key, label, description, level, default_level}]` (PUT takes `[{key, level}]`), by tool. The turn honours them: `never` refuses the call, `ask` always brings the existing approval card (even over an "always allow" rule), `allow` runs without one.
  - `GET|PUT /api/agents/:type/soul` `{text, version}`, `GET /soul/versions`, `POST /soul/rollback {version}` (like the personality's). The soul.md joins the member's instructions on every turn.
  - `GET /api/agents/:type/activity` → `{items: [{kind: task|tool|started|finished, text, status, session_id, chat_title, at}]}` from delegations and turn events, last 30 days.
  - `POST /api/agents` `{template?, name?, role?, soul?, tools?, shape?, tone?, motion?}` → 201 `{agent_type, name, role, soul_version}`: a dynamic agent only that person sees and is routed to (409 for a name already in their crew, at most 20). `DELETE /api/agents/:type` removes one's own. `GET /api/agents/templates` (four, in code; their soul text in `prompts.rs`). Admin dynamic agents no longer list people's own.
  - Migration `0136_personal_crew.sql`. `frontend/`: Crew links each member to its page (On switch, Permissions, soul.md with Edit and versions, Activity · N, Remove for one's own), shows "Paused", and New member (from a template or from scratch).

Found while building Settings › Crew (2026-11-13).

### Workspaces: declining an invitation, activity, templates, where Nomi opens — done
- **Built:** `POST /api/invites/:code/decline` → 204 (403 another address, 409 already a member, 410 used or expired, 400 for a link). A declined invite leaves both lists; `GET /api/invites/:code` says `state: "declined"`; resending brings it back.
  - Each org in `GET /api/orgs`: `member_avatars` (up to 3, `{name, avatar_url}`, owners first), `last_activity: { kind: "published" | "worked_on", project, by, at } | null` (the newest publish or project-chat message; the client writes the line, rather than the proposed `text`), and `is_personal` (in place of `personal`).
  - `PUT /api/preferences/open-org {org_id | null}` (null is last used) → 204, 404 when not a member. `GET /api/orgs` returns `open_org_id`. Each fresh sign-in (password, code, Google) opens there while the person is still in it, by setting `users.active_org_id`; refreshes and switches keep the active one.
  - Migration `0113_workspace_pages.sql`. `frontend/`: Organizations shows faces, the last activity line and an "Open Nomi in" setting.
- **Still open:** `templates`: Nomi has no shared crew templates to count.

Found while building Settings › Workspaces (2026-11-13).

### A workspace's own page: who made it, each member's work, resending, deleting — done
- **Built:** `GET /api/orgs/:id` adds `is_personal`, `created_by: {id, name} | null` (`organizations.created_by`, earlier ones take their first owner) and `created_at`; each member adds `projects` (started or added to, in this workspace) and `last_active_at` (latest sign-in refresh, open project or device check-in).
  - `POST /api/orgs/:id/invites/:code/resend` → the invite, emailed (and in the inbox) again with another 7 days (owners and admins; 404 for links).
  - `POST /api/orgs/:id/invite-links {role?}` → 201 `{code, email: null, role, link: true, …}`: a link anyone signed in (or signing up) can join with, many times, for 30 days; a new one replaces the open link for that role, and it's revoked like any invite. Open invites carry `link`.
  - `DELETE /api/orgs/:id` → 204, owners only (403). 409 while it still has projects (move or delete them first), for a personal workspace, or when it's the only workspace one of its members has. Each member's own chats in it move to their next workspace; open invites and links stop working; everyone leaves it (kept as `deleted_at`).
  - `frontend/`: the workspace page shows who made it and when, each member's projects and activity, Resend, "Copy invite link" (for the chosen role) and, for owners, Delete.

Found while building Settings › Workspace (2026-11-13).

### Projects: renaming, "Needs you", community apps — done
- **Built:** `PATCH /api/projects/:id` `{ name?, description? }` (builders); `POST /api/projects/:id/duplicate` `{ name?, org_id? }` → 201 `{ session_id, project_id }` (files, stack, description and plan; not its publishing, data or members).
  - Summaries carry `waiting_on_you` (unanswered `questions` cards and pending approvals in its chats), `last_deploy: { status: "ok" | "failed", at }` and `live_status: "up" | "down"` (down after a `service.crashed` in the last 30 minutes with no publish since; null when not live).
  - `GET /api/showcase` already listed the apps on show; added `POST /api/showcase/:id/remix` → 201 `{ session_id, project_id }`, a copy into the person's workspace for projects.
  - `frontend/`: Rename and Duplicate in the card menu, a "Needs you" badge and the Needs attention tab, Community lists apps on show with Remix, and the showcase page has Remix.

Found while building Projects (2026-11-14).

### Projects: a card's progress, a build's allowance, moving with approval — done
- **Needed by:** *Flow / Projects — Act from the card* (syolc) and *Card menu* (M9O0iU): "3 of 6" and "writing routes/menu/+page.svelte" on a building card, the Deploy dialog's "Uses · 1 build · 26 of 30 left in Oct", Stop on a deploying card, the failed card's "build error in src/lib/pdf.ts · #5 still live", and "Waiting on the owner" after Move.
- **Why:** the list says a project is building, not how far along it is; publishes can't be cancelled; and a move happens at once, with the target workspace's owner taking the project.
- **Built:** project summaries carry `build: { step, steps, doing } | null` (only while a turn is running in the project's chat) and `last_deploy: { status, error, at, number }`; `DELETE /api/projects/:id/deploy/publishes/:id` marks a running publish `cancelled` (409 once finished; Shipyard can't abort, so a cancelled publish may still finish, and the row stays cancelled); `GET /api/me/plan` gives the build allowance (`plans.max_builds_month`, null = unlimited); `POST /api/projects/:id/transfer` to a workspace the caller doesn't own answers 202 with a pending request its owner approves or declines (migrations 0163–0164).
- **Still open:** the old `frontend/` still treats a pending move as done.

Found while building the Projects page's card actions (2026-11-15).

### Import from GitHub: a person's repos, private ones, and what Koda found — done
- **Needed by:** *Flow / Projects — Import from GitHub* (qJxWu): Connect GitHub (once), Pick a repo (the person's repos newest first, ones Nomi can't see with Grant access), Koda reads it, What Koda found (app, tests, data, secrets it couldn't find, what it will set up), Set up & deploy.
- **Why:** people with an app already on GitHub can only bring it in as a public repo by name, and nothing says what Koda made of it before it's built.
- **Built:** `GET /api/connections/github/repos?q=` (newest first, `accessible`; `q` filters the first 500 repos); `import/github` uses the connected token for private repos (public ones still work without); `GET /api/projects/:id/import/review` → `{ stack, build, tests, database, secrets_missing, plan }`, read from the imported files (the plan's steps are English only).

Found while building the Projects page's card actions (2026-11-15).

### A project's own domain (DNS) — done
- **Built:** `GET|POST /api/projects/:id/domains` (`{hostname}` → 201), `POST /api/projects/:id/domains/:hostname/check`, `PUT …/:hostname/primary`, `DELETE …/:hostname` (204); migration `0140_project_domains.sql`.
  - A domain is `{ hostname, records: [{ type, name, value, status: "found" | "waiting" }], checked_at, verified_at, primary, https: "pending" | "issued", created_at }`. Records: a CNAME to the app's live address (a bare domain like `warung.co.id` gets A records to the same ingress's addresses instead) and a TXT `_nomi-verify.<hostname>` = `nomi-verify=<token>`. Looked up with hickory-resolver when added, on Check now, and every 5 minutes by the scheduler loop; the page hears `domain.*` on the project stream.
  - Admins add, make primary (only once every record is found) and remove; anyone on the project lists and checks. The app must be live first (409); a hostname is on one project only (409); one under Nomi's own apps domain is refused.
  - `frontend/`: Domain lists the domains with their records (found or waiting), last checked · every 5 min, HTTPS, Check now, Make primary and Remove, and adds one.
  - **Still open:** Shipyard serves one address per environment (`PUT …/domain`), with no extra hostnames, so a domain's HTTPS stays `pending` and making it primary is kept by Nomi only, until Shipyard can route and certify more than one hostname.

Found while building the workspace's Domain (2026-11-14).

### A project's database: a dump, and editing rows — done
- **Built:** `GET /api/projects/:id/database/dump?source=` streams a `.sql` file (`<project>-<source>-<date>.sql`). The server image has no `pg_dump`, so it's written from the catalog in one REPEATABLE READ snapshot: extensions, schemas, enum types, sequences, tables with defaults, identity, generated columns and constraints, the rows as INSERTs, then indexes, foreign keys, sequence positions and views (it restores into an empty database; functions and triggers aren't included).
  - `PATCH /api/projects/:id/database/tables/:schema/:table/rows` `{ key, changes, source? }` → `{ row }`: the table and columns are checked against the catalog, the key must be exactly the primary key's columns, generated columns are refused, values are bound as parameters and cast to each column's type; 404 for a missing table or row, 409 for a table without a primary key, 422 with the database's reason for a bad value.
  - `GET …/database/tables` now gives each table's `primary_key`.
  - `frontend/`: Database has Download dump, and a browsed table's cells edit in place (Enter saves, Esc cancels).

Found while building the workspace's Database (2026-11-14).

### A site's visits: where they came from, form sends, uptime — done
- **Built:** `GET /api/projects/:id/deploy/metrics` (production) adds `sources: [{ name, share }] | null`, `form_sends: number | null` and `uptime: { share, since } | null`, where Shipyard's answer doesn't carry them. Migrations `0141_site_visits.sql`, `0142_site_uptime.sql`.
  - Shipyard's metrics have no referrers, so a production publish puts a small script before `</head>` in the site's HTML pages that reports each arrival's referrer and each form sent to `POST /api/hooks/sites/:id` (forwarded by `frontend/` at `/hooks/sites/:id`); moving between the site's own pages isn't counted. Referrers group as Instagram, Google search, WhatsApp, Facebook, X, TikTok, LinkedIn, Direct, or the host.
  - Uptime: the scheduler loop opens each live app's address every 5 minutes; `share` is the looks that got an answer below 500 over the range, `since` the first look. Reports and looks are kept 30 days.
  - `frontend/`: Monitor shows where visits come from, form sends with their share of visitors, and uptime, each "Not measured" until there's data.
  - **Still open:** a form send is counted when the form is submitted, not from the answer's status (the request log with statuses is Shipyard's); sites without HTML pages of their own (server-rendered without a `</head>` in the source) report nothing until Shipyard gives referrers.

Found while building the Landing page flow (2026-11-14).

### A message read on its own names its author — done
- **Built:** `GET /api/sessions/:id/messages/:message_id` runs `attach_authors`, so a teammate's message carries `author_name` there too.
  - `frontend/`: live message updates already read it; a teammate's message no longer chains onto the person's own.

Found while building Invite & build together (2026-11-14).

### When an ask started running, and what a change to publish holds — done
- **Needed by:** *Flow / Build — Invite & build together* (TnwQT) step 9 ("Mario · running · 00:21") and step 10 (the event "Sari's ask built in the preview · 2 files · tests pass" and the request's "Makanan section · 2 files · built and tested in the preview").
- **Why:** the line can't say how long the running ask has run, and a request to publish can't say which change it is, which files it touches or whether its tests passed.
- **Built:** a running ask in `GET /api/sessions/:id/queue` has `started_at`; the chat event `AskBuilt { job_id, files, tests }` when a coding ask finishes with files changed (`tests` from the turn's test run, else null); a publish request carries `ask: { job_id, text, asked_at }` and `files` changed since the live deploy, snapshotted when asked (migration 0165).
- **Still open:** no `AskBuilt` for a turn resumed after an approval; deleted files aren't counted.

Found while building Invite & build together, steps 9–10 (2026-11-15).

### Who last answered in each chat — done
- **Needed by:** *Flow / Mobile Chat — Start & answer* (FvlVs) step 1, Chats on a phone: each row's avatar is the crew member who last answered, in their tint, its line starts with their name ("Dana: Rp 2,1 jt on food…"), and the filters above the list (All, Dana, Kala, Tara) narrow the list to the chats each one answered in.
- **Built:** `GET /api/sessions` rows carry `last_message.agent_type` (null for the person's own) and `agents: string[]` in order of first reply.

Found while building the phone's Chats (2026-11-16).


### A project's workspace on a phone: the line a teammate is on, cancelling a publish, web push
- **Needed by:** *Flow / Mobile Build* — Invite & build together (L8dgcS) step 8 (following draws a "Sari" tag on the line she's on); Workspace tools (tJh5G) steps 3–4 and Sign in → live URL (zM0xh) step 5 (a publish that's running, and a build, offer Stop); Sign in → live URL step 6 (the lock-screen notification "warung-kopi-menu is live").
- **Why:** following on a phone (and the desktop) can go to the file a teammate has open but not the line; a publish, once started, can't be stopped; and "We'll notify you" reaches a phone's lock screen only in the native apps.
- **What exists today:** presence (`POST/GET /api/projects/:id/presence`) carries `view` and `path`, not a line; there's no route to cancel a running publish (`project_publishes` goes from `running` to `succeeded`/`failed` only); a published app's notice goes to the Inbox and by email (`notify_admins("published")`), and pushes only to native iOS and Android devices through FCM/APNs (`push.rs`).
- **Proposed contract:** `line: number | null` (and optionally `to`) on a person's presence; `DELETE /api/projects/:id/deploy/publishes/:id` to cancel a running publish (already proposed under "Projects: a card's progress"); Web Push as proposed under "Web Push for a phone's browser" below, also sending the published notice (`notify_admins("published")`) the native apps get.
- **Until then:** following has no line tag; the phone's publishing pill shows the time with no Stop; the web app says "We'll notify you" (true through the Inbox and email) and draws no lock screen.

Found while building the workspace on a phone (2026-11-18).

### How many messages each chat holds
- **Needed by:** *Flow / Mobile Chat — Manage chat* (W3a8j) step 1, the chat's sheet (G8cFz: "14 messages · Dana"), opened from the Chats list's long press as well as from inside the chat.
- **Why:** from the Chats list the sheet can't say how big the chat is, and inside a long chat (older messages not loaded yet) neither can the chat's own sheet.
- **What exists today:** `GET /api/sessions` gives `last_message` without a count; `GET /api/sessions/:id/messages` pages with `has_more`, no total. A share link carries `chat_messages`, only once one is made.
- **Proposed contract:** `message_count` on each item of `GET /api/sessions` (and on the session itself).
- **Until then:** the list's sheet says when the chat last moved, as before; the chat's own sheet says "N messages · Dana" once every message is loaded, and only who answers while older ones aren't.

Found while matching the phone's chat screens to the pen (phone-fidelity, 2026-11-18).

### How long a reply thought
- **Needed by:** *Flow / Mobile Chat — Start & answer* (FvlVs) step 6 and the desktop's Ask → think → answer: a finished reply's reasoning row reads "Thought for 8s" (k8JkX2).
- **Why:** after a reload the app can't say how long the crew thought, so the row reads "Thinking" on a reply that's done.
- **What exists today:** a message's `reasoning` content block is `{ kind: "reasoning", text }`; the live feed's phase changes give the time only while the turn runs (the app shows "Thought for Ns" for a reply it watched).
- **Proposed contract:** `duration_ms` on the `reasoning` block (from the first thinking delta to the first reply delta).
- **Until then:** "Thought for Ns" on a reply timed live; "Thinking" on one loaded later.

Found while matching the phone's chat screens to the pen (phone-fidelity, 2026-11-18).

### Save a reply to memory
- **Needed by:** *Flow / Mobile Chat — Like & dislike* (M0VaJC) step 1, the reply's sheet: "Save to memory" (bookmark-plus, XPHtc).
- **Why:** a person can't keep what a reply said as a memory of their own; memories only come from what the crew learns.
- **What exists today:** `GET /api/memory` lists, `PUT`/`DELETE /api/memory/:id` edit or forget, `POST /api/memory/:id/confirm` confirms; `GET /api/sessions/:id/messages/:message_id/memories` lists what a message taught. Nothing creates one.
- **Proposed contract:** `POST /api/sessions/:id/messages/:message_id/memories` → `201 { memory }` (the crew sums the reply up as one memory, with its kind), or `POST /api/memory` with `{ text, kind }`.
- **Until then:** the reply's sheet leaves Save to memory out.

Found while matching the phone's chat screens to the pen (phone-fidelity, 2026-11-18).

### Bring a deleted reply back
- **Needed by:** *Flow / Mobile Chat — Share & delete* (yoVD6) step 5: "Reply deleted · Undo within 10 seconds", and the placeholder's Undo and Regenerate.
- **Why:** `DELETE` removes a reply and what its cards made at once and for good, so Undo can only work by holding the delete back; a tab closed in those ten seconds deletes it straight away (the app sends it on `pagehide`), and a reply that's gone can't be asked again in its place.
- **What exists today:** `GET /api/sessions/:id/messages/:message_id/delete` (what goes with it) and `DELETE /api/sessions/:id/messages/:message_id`; `POST …/regenerate` works on a reply that still exists.
- **Proposed contract:** a soft delete (`deleted_at` on the message and what it lists) with `POST /api/sessions/:id/messages/:message_id/restore` inside a window, and `POST /api/sessions/:id/messages/:message_id/rerun` (on the person's message) to answer it again once its reply is gone.
- **Until then:** the app waits ten seconds before it deletes, with Undo on the toast and the placeholder; the placeholder has no Regenerate.

Found while matching the phone's chat screens to the pen (phone-fidelity, 2026-11-18).

### Chat files: what the composer and the reply still need (UXP-001, audit-chat-files) — done
Checked against `backend/crates/nomi-server/src/app.rs`, `routes/attachments.rs`, `routes/shares.rs` and
`nomi-agent-core/src/views.rs` on 16 Nov 2026. The UX contract is
`docs/ux/work/audit-chat-files/40-ux-contract.md`; each part below is left out of the UI until it lands.

- **Upload failure codes (G3) — done.**
  - **Needed by:** *Flow / Chat — Attachments, voice & limits*, step 8 (D8 · M9).
  - **What exists today:** `POST /api/attachments` answers plain strings (`413 file is too big`, `507 storage is full`, `400 upload was interrupted`, `415 programs can't be attached`). The chip says why from the status.
  - **Proposed contract:** errors as `{ "error": "too_large" | "storage_full" | "unsupported_type" | "interrupted" | "server_error", "message": "…" }`, with `max_bytes` on `too_large`.
  - **Built:** errors are `{ error, message }` with `too_large` (+ `max_bytes`), `storage_full`, `unsupported_type` (+ `extension`), `interrupted`, `server_error`; an empty file is `empty_file`.
- **Resumable uploads (G3, M1) — done.**
  - **Needed by:** step 9 (D9 · M11), "Resuming · 41%" and "picks up at 41%".
  - **What exists today:** one streamed request per file; a retry starts again.
  - **Proposed contract:** `POST /api/attachments/uploads` → `{ upload_id, offset: 0 }`; `PATCH /api/attachments/uploads/:upload_id` with `Content-Range: bytes <from>-<to>/<total>` → `{ offset }`; `HEAD` the same to read the offset after a drop; the last chunk answers the usual `AttachmentResponse`.
  - **Until then:** the restart copy ("Uploading again · 0% · 0 of 5 MB", "starts the upload again").
  - **Built:** as proposed (`POST /api/attachments/uploads`, `PATCH` with `Content-Range`, `HEAD`; a wrong offset is 409 `offset_mismatch` with the current one; unfinished uploads expire after 24 h) (migration 0163). Chunks wait on the server's disk, so several backend servers need a shared temp folder or sticky routing.
- **The accepted types (G4) — done.**
  - **Needed by:** step 10 (D10 · M12) and Errors D6 · M5: the type check before any upload, and the helper "Maya reads PDF, images, sheets and zip."
  - **What exists today:** `GET /api/attachments/limits` gives sizes and counts; the server takes any file but programs (415).
  - **Proposed contract:** `GET /api/attachments/limits` adds `accepted: { kinds: ["pdf","image","spreadsheet","archive",…], extensions: [".pdf",".jpg",…], mimes: [...] }`, and a refused upload answers `415 { "error": "unsupported_type", "extension": ".dmg" }`.
  - **Until then:** a file is refused only when the server refuses it (the Rejected chip, after its upload); no helper line.
  - **Built:** `GET /api/attachments/limits` adds `accepted: { kinds, extensions, mimes }` (the composer's check; the server still refuses only programs), and a refused upload answers `415 { error: "unsupported_type", extension }`.
- **A pre-send estimate (G7, and `Read as images`) — done.**
  - **Needed by:** steps 15–16 (D15–D16 · M17–M18), Errors D18 · M20.
  - **Proposed contract:** `POST /api/attachments/estimate` `{ attachment_ids: [...], model_key }` → `{ files: [{ id, pages, tokens }], tokens, payer: "credits" | "byok", provider_label, cost_idr?, plan: { used_pct, after_pct }? , threshold_tokens }`.
  - **Until then:** no estimate row; nothing shown.
  - **Built:** `POST /api/attachments/estimate` as proposed (a picture or scanned page counts 1,600 tokens; `threshold_tokens` is half the model's context, else 50,000).
  - **Still open:** it doesn't check that the model is allowed on the person's plan.
- **A read result per file (G9) — done.**
  - **Needed by:** steps 17–20 (D17–D20 · M19–M22), Errors D7 · M6.
  - **What exists today:** `status` (`pending | ready | failed`) and `details.scanned`; a scanned PDF is read by the files model, so "couldn't read" would be false.
  - **Proposed contract:** on `GET /api/attachments/:id` and in the reply's content blocks: `read: { result: "ok" | "locked" | "no_text" | "damaged" | "partial", read?: number, total?: number, skipped?: [{ name, reason }] }`, and `replaces: <attachment id>` on an upload sent as a replacement.
  - **Built:** `read` on `GET /api/attachments/:id` (null while it's being read) and `replaces`, set by sending `X-Replaces: <attachment id>` with the upload (migration 0162). Not in the reply's content blocks.
- **Files Maya makes (G11) — done.**
  - **Needed by:** steps 21–26 (D21–D26 · M23–M28).
  - **Proposed contract:** reply events `file_generating { file_id, name, step, started_at }`, `file_ready { file_id, name, mime, size, rows?, summary }`, `file_failed { file_id, code: "write_failed" | "stopped" }`; `GET /api/sessions/:id/files/:file_id/content` (signed or auth'd download), `…/preview?rows=20`, `DELETE …/:file_id` (keeps `deleted_at`, and whether its sources still exist), `POST …/:file_id/remake`; saving goes through `PUT /api/projects/:id/files/*path`, whose 402 should carry `{ "error": "storage_full" }`.
  - **Built:** Maya's `make_file` tool (CSV, XLSX); the events `file_generating`, `file_ready`, `file_failed`; `GET …/files/:file_id/content` (signed in, not signed), `…/preview?rows=`, `DELETE` (keeps the row with `deleted_at`, then 410), `POST …/remake` (202, a new file with `replaces`); the project files' 402 is `{ error: "storage_full", message }` (migration 0166).
  - **Still open:** `file_failed` with `stopped` is never sent; only the file's owner can open it.
- **Citations to a file's page and rows (G12) — done.**
  - **Needed by:** step 27 (D27 · M29): `[3]` opens the file at the cited page with the rows highlighted.
  - **What exists today:** the `sources` view numbers citations `{ title, url?, detail?, kind: web | file }`; `GET /api/attachments/:id/content` serves the whole file.
  - **Proposed contract:** a file citation adds `{ file_id, pages: [3, 8], spans: [{ page, rects: [[x,y,w,h]…], label, total_idr? }] }`; `GET /api/attachments/:id/pages/:n` → a rendered page image (so the viewer can draw highlights over it).
  - **Until then:** a sent PDF opens at page 1 in the browser's own reader, with no cite bar or highlights.
  - **Built:** file citations carry `file_id`, `pages` and `spans` (Maya fills them when she knows the page; `rects` stay empty); `GET /api/attachments/:id/pages/:n` (404 past the last page).
  - **Still open:** no PDF renderer: page 1 is the stored preview and other pages answer `501 { error: "page_render_unavailable" }`.
- **Files in a shared link (G16) — done.**
  - **Needed by:** *Flow / Chat — Share & delete*, steps 6–8 (Share D6–D8 · M6–M8); *Shared link*, steps 6–7 (Link D6–D7).
  - **What exists today:** shares keep `mask`, `anyone` and `expires_in_days`; `GET /api/shared/:token` returns the messages, whose reference tags name the files. No file can be opened from a link.
  - **Proposed contract:** `POST`/`PUT /api/sessions/:id/shares` take `files_open: bool` (default false), refused with `400 { "error": "mask_and_files_open" }` when `mask` is also true; `GET /api/shared/:token` adds `files_open` and `files: [{ message_index, name, kind, size, pages?, made_by_crew: bool, url? }]`, with signed `url`s only when `files_open`.
  - **Until then:** the dialog has no files row (its switch would do nothing); the shared page names each file with a lock, "private".
  - **Built:** as proposed (migration 0160), files Maya made included; a file's `url` is the link's own `/api/shared/:token/files/:id` (nothing signs file URLs), served only while `files_open`. A `PUT` without `files_open` sets it off.
- **"Sent without" kept on the message (G6) — done.**
  - **Needed by:** step 14 (D14 · M16): the bubble's note `Sent without struk-belanja-nov.jpg · deepseek-v3 can't see images`.
  - **Proposed contract:** `POST /api/sessions/:id/messages` takes `left_out: [{ name, reason: "model_cannot_see", model_label }]`, kept on the message and passed to the crew (so Maya can say she didn't get the photo).
  - **Until then:** the note shows on this visit only.
  - **Built:** `left_out` is kept on the message (migration 0157), returned with it, and each file is named to the crew in the message's history.

### Questions: a recommended answer, and a helper line — done
- **Needed by:** *Flow / Mobile Chat — Clarify → plan* (GCvfD) steps 1 and 3: an option tagged RECOMMENDED and picked to start; Skip takes it as a DEFAULT ("You skipped the weekend, so I picked 1–2 Nov."); a line under the question ("You and Sari, from Jakarta.").
- **Built:** `options[].recommended` and `questions[].helper` on the `questions` block, which the crew can now set.

### A plan's steps: who does each, how it's going, how long it'll take — done
- **Needed by:** *Flow / Mobile Chat — Plan → approve → act* (G1tK6L) steps 1, 2 and 6: each step's owner, a per-step line ("running · 00:12", "will ask you first"), the foot "~4 min · Maya, Dana, Kala", and "Finished in 3m 40s".
- **Built:** each version has `steps[{ text, owner_agent_type, status, status_detail, started_at, finished_at }]` and the plan `estimate_seconds`.
- **Still open:** steps are read from the checklist, so the owner is the plan's own agent, times are null, and the estimate is a minute per open step.

### Run a proposed plan, or only research it — done
- **Needed by:** *Flow / Mobile Chat — Clarify → plan* step 7 ("Run this plan?": Run it, ask before anything leaves Nomi / Research only / Keep planning).
- **Built:** `POST /api/sessions/:id/agent-plans/:agent_session_id/run { mode: "run" | "research" }` → 202 (400 for another mode).

### An approval sent with the person's edits, and a link to what was sent — done
- **Needed by:** *Flow / Mobile Chat — Approvals & background* (xvntx) steps 1–2 (Edit the draft, Sent with your edits) and *Plan → approve → act* step 6 (View in Gmail).
- **Built:** `{ decision: "approve", input }` runs the tool on the edited input (checked against the tool's schema, 400 otherwise); `result_url` on the approval block once the tool returns a link.

### Undo a deny, and discard after a stop — done
- **Needed by:** *Flow / Mobile Chat — Approvals & background* step 3 (Undo on "Not sent") and step 6 (Discard on the Supervisor's report).
- **Built:** `POST …/messages/:mid/approval/undo` within 10 s of a deny (409 after; the crew carries on only once the window ends); `POST /api/sessions/:id/stopped/:message_id/discard` drops the stopped tasks' partial output and marks the block `discarded` (a second call does nothing; files already written aren't reverted) (migration 0158).

### "Don't ask again" scoped to a recipient — done
- **Needed by:** *Flow / Mobile Chat — Plan → approve → act* step 5 ("Don't ask again for Bu Sri").
- **Built:** `remember: { scope: "recipient", value }` allows that tool for that recipient only; `remember: true` still allows the whole tool.

### A failed turn's status, provider and the provider's words; a content-filter kind — done
- **Needed by:** *Flow / Mobile Chat — Errors & recovery* (AyrHG): the card's code badge (402, 429…), "DEEPSEEK SAID · Insufficient Balance", and scenario 8.5 (Blocked by provider, with Rephrase).
- **Built:** `run_failed { retry, failure_kind, status, provider, provider_message, model }` (`kind` is the block's own tag, so the failure's is `failure_kind`; values include `content_filter`).
- **Still open:** `model` is always null: the turn doesn't pass it to the failure yet.

### Summarise older messages and carry on — done
- **Needed by:** *Flow / Mobile Chat — Errors & recovery* scenario 3 (Chat too long → Summarise & continue, Summarised).
- **Built:** `POST /api/sessions/:id/summarize` → 202 (409 `already_summarizing` or `nothing_to_summarise`): a `summary` message replaces all but the newest 6 messages for the crew (they stay in the chat), then the failed turn is retried; a failure shows as `run_failed` (migration 0159).

### Web Push for a phone's browser — done
- **Needed by:** *Flow / Mobile Chat — Approvals & background* step 5 (the lock screen: "Tara needs your OK"), and plans finishing while the person is away.
- **Built:** `GET /api/devices/web-push/key` (the VAPID public key; 404 when push is off) and `POST /api/devices/web-push { endpoint, keys }`; needs-you, approval and plan-finished pushes go to it too (migration 0161; keys in `.env.example`).

Found while building the phone's chat flows (2026-11-18).

### A chat's hand-off, and a code change to apply from a reply
- **Needed by:** *Flow / Mobile Chat — Crew, memory & connections* (YuWzO) step 1 ("Nomi handed this to Dana · Ask Nomi" over the reply), and *Rich replies* (XO7DI) step 4 (a diff card with `+3 −1`, its hunk, Reject and Apply change), on both layouts (hiAY1, y2Qxt2).
- **What exists today:** a reply names who answered (`agent_display_name`), but nothing says Nomi routed the question there, or why. A reply can carry a `code` block (a file and its lines), but no diff, and there's no route to apply or reject one.
- **Proposed contract:**
  - A `handoff` content block on the reply that took over: `{ kind: 'handoff', from: 'Nomi', to: 'Dana', reason?: string }`. "Ask Nomi" would send the next message to Nomi as it does today.
  - A `diff` block: `{ kind: 'diff', file, added, removed, hunks: [{ header, lines: [{ old?, new?, kind: 'add'|'remove'|'same', text }] }], change_id }`, with `POST /api/sessions/:id/changes/:change_id` taking `{ decision: 'apply'|'reject' }`.
- **Until then:** the reply shows who answered without the hand-off line, and code shows as a code block with no Apply or Reject.

Found while building the rest of a chat on a phone (2026-11-19).

### A public waitlist
- **Needed by:** the landing page (Figma, Nomi Builder → "Landing v2 · Playful (ID)" 170:2766), its Waitlist CTA (170:2948): "Jadi yang pertama punya kru sendiri", an email field and "Gabung waitlist", with "Cuma 1 email pas rilis" under it.
- **Why:** a visitor can't leave an address to hear when Nomi opens without making an account.
- **What exists today:** `POST /api/plans/:id/waitlist` (`plan_checkout::join_waitlist`), which needs a signed-in person and a plan; nothing for someone without an account.
- **Proposed contract:** `POST /api/waitlist` `{ email: string, locale?: 'en' | 'id' }` → `202 {}` (the same answer whether or not the address was already on it), rate-limited by address and IP, with one email sent when access opens and a way to leave the list.
- **Until then:** the landing page leaves the waitlist section out (and its nav link); Try it free goes to register.

Found while building the landing page (2026-11-21).
