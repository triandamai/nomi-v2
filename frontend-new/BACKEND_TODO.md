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
- **Still open:** the Settings › Data page itself, in both apps.

Found while planning the working v2 (sign in, chat and build against the real API, 2026-11-10).

### Sign in or create with an email code, no password — done
- **Needed by:** *Sign in or create* (`vDaAQ`) and *Enter code* (`pUDXz`).
- **Built:** `POST /api/auth/start {email, language?, invite_code?}` → `{verification}` for an existing or a new address alike (400 `invalid_email` / `invalid_invite`).
  - An existing account gets a sign-in code (`purpose: "login"`). A new address gets a code with no account yet (`purpose: "register"`); the account is made only when the code is entered.
  - `POST /api/auth/verify` now also returns `is_new`. A new account has no password (`password_login` off), its email confirmed, and a personal workspace ("{name}'s space", `is_personal`), or the invite's workspace.
  - Password `login` and `register` work as before. Migration `0109_email_code_sign_in.sql`.
  - `frontend/`: the login page has "Email me a code instead"; `/verify` finishes it and sends new accounts to first run.

### Change the password while signed in — done
- **Needed by:** *Settings / General*: "Password · Not set. Add one to sign in without Google".
- **Built:** `PUT /api/auth/password {current_password?, new_password}` → 204. `current_password` is required when a password is set (403 `wrong_password`; 400 `too_short` under 8 characters). Setting one turns password sign-in on; other devices stay signed in.
  - The emailed reset now turns password sign-in on too.
  - `frontend/`: Account › Sign-in methods has a set/change password form.

### Switch the active workspace — done
- **Needed by:** *Workspace switcher* (`pKiO2`) and the account menu.
- **Built:** `PUT /api/auth/active-org {org_id}` → `{access_token}` carrying that org (404 `not_a_member`).
  - The choice is kept (`users.active_org_id`, migration `0110_active_org.sql`): later sign-ins and refreshes keep it while the person is still an active member, else fall back to their first workspace.
  - `frontend/`: Organizations marks the active workspace and offers "Switch"; the new token is stored like sign-in's.

### A project's versions, listed and restorable
- **Needed by:** the Build flows' version line ("version N by Koda"), and restoring a version that was never published.
- **Why:** `projects.version` moves once per Koda reply (migration `0107`), but it's only exposed on publishes (`PublishRow.version`) and activity items.
- **What exists today:**
  - `GET /api/projects/:id/activity` shows versions as they happened.
  - `POST .../deploy/publishes/:pid/rollback` restores a published one.
- **Proposed contract:** `GET /api/projects/:id/versions` → `{versions: [{version, by, at, summary}]}`. `POST /api/projects/:id/versions/:v/restore` → 202.
- **Until then:** versions appear in the activity and on publishes. Rollback is offered for publishes only.

Found while building the app shell (App / Home, the account menu and the workspace switcher, 2026-11-10).

### Search chats, memory and files
- **Needed by:** *App / Home* (`Vr4nn`), the top bar's Search (`u8pcLQ`): "Search chats, memory, files ⌘K".
- **Why:** people can't find an old chat, a remembered fact or a file without scrolling through each list.
- **What exists today:**
  - Only the lists themselves: `GET /api/sessions` (every chat, no query), `GET /api/memory`, and a project's files.
  - The only search routes are an admin one (`/api/admin/tools/web_search/test`) and `GET /api/projects/:id/transfer/orgs`.
- **Proposed contract:** `GET /api/search?q=&kinds=chat,memory,file&limit=20` → `{ results: [{ kind: "chat"|"memory"|"file", id, title, snippet, session_id?, project_id?, updated_at }] }`: the person's own things (and their projects' files) only, best match first.
- **Until then:** the top bar has no search box. ⌘K opens a new chat, as the sidebar shows.

### A person's pending workspace invites — done
- **Built:** `GET /api/invites` → `{ invites: [{ code, org_id, org_name, inviter, role, projects, expires_at, project: { id, name } | null }] }`: open invites to any of the person's addresses, for workspaces they aren't in yet, newest first.
  - `frontend/`: Organizations lists them under Invitations with Join and Decline.

### Say which workspace is personal — done
- **Built:** `is_personal` on each workspace in `GET /api/orgs` and `POST /api/orgs` (and on `GET /api/orgs/:id`).
  - `frontend/`: Organizations shows a "Personal" tag.

Found while building sign in and create account (Phase 1, 2026-11-10).

### Sign up from a project invite puts the person on the project too — done
- **Built:** `place_in_org` (Join) now adds the `project_members` row with the invite's project role, records `used_by` and `invited_by`, and the project chat announces the join. It covers password sign-up, Google sign-up and new email-code accounts (all go through `place_in_org`).

### Who shared a link, when, and whether the viewer is its owner
- **Needed by:** *Flow / Chat — Shared link* (b8Flw8), steps 1–2: the meta line "Shared by M. · 22 Oct 2026" and the owner's banner ("This is how others see your link. 3 views so far.", Copy link, Stop sharing).
- **Why:** a viewer can't see who shared it or when; the owner can't tell their own preview apart, see how many opened it, or stop sharing from the page.
- **What exists today:** `GET /api/shared/:token` answers `{ title, scope, masked, messages, expires_at }`.
- **Proposed contract:** add `shared_by` (initials when masked), `shared_at`, and, when the bearer is the owner, `own: true`, `share_id` and `views`.
- **Until then:** the meta line shows the chat's title and expiry; no owner banner.

### Reporting a shared page
- **Needed by:** *Flow / Chat — Shared link* (b8Flw8): "Report this page" in the footer.
- **Why:** a viewer has no way to flag a shared page.
- **What exists today:** nothing.
- **Proposed contract:** `POST /api/shared/:token/report` with `{ reason, note? }`, open without a sign-in, rate-limited.
- **Until then:** the footer has no report link.

### Push for reminders, and quiet hours
- **Needed by:** *App / Inbox* (BFoiZ) → How it reaches you: "Push for reminders — On your phone and desktop, at the exact time." and "Quiet hours 22:00–06:00 — Learned from your routines. Reminders wait until morning.", each with a switch.
- **Why:** a person can't turn reminder pushes off without turning off notifications for the whole app on the phone, and there's no way to hold reminders back overnight.
- **What exists today:** `GET|PUT /api/notifications/preferences` holds only `email_account` and `email_promos`; phones get pushes for every inbox item and reply (`crate::push`), with no preference.
- **Proposed contract:** add `push_reminders: bool` and `quiet_hours: { start: "22:00", end: "06:00" } | null` (in the person's zone) to the preferences; the reminders worker holds a reminder due inside quiet hours until they end.
- **Until then:** the Inbox shows the two email settings only.

Found while building the Inbox (2026-11-12).

### Profile insights and when the person is active
- **Needed by:** *App / Settings / Profile* (U9Gnrg): the Insights panel ("Save ~18% on Nomi — gemini-flash answered 9 of your last 10 quick questions just as well", "You work late — 68% of your tokens land after 21:00") and the heatmap's note "Most active Tue · 21:00–23:00".
- **Why:** nothing works out a cheaper model that would have done, or the hours someone uses Nomi; the profile can only say which weekday is busiest.
- **What exists today:** `GET /api/profile` gives tokens per day (`activity`), the top models of the last 30 days, and the crew's chat counts.
- **Proposed contract:** add `active_hours: { from: "21:00", to: "23:00" } | null` (in the person's zone) and `insights: [{ kind: "cheaper_model" | "late_hours", title, body, action?: { label, href } }]`.
- **Until then:** no Insights panel; the note names the weekday only.

Found while building Settings › Profile (2026-11-12).

### Account settings General can't keep yet
- **Needed by:** *App / Settings / General* (m7lvY1): Email "Change", Formats (region), the Chat group ("New chats start with", "Enter sends", "Show tokens under replies") and Sessions' "Delete account".
- **Why:** a person can't move their account to a new address, pick number and date formats apart from the language, keep how chats start and send, or close their account.
- **What exists today:** `GET|PUT /api/preferences` keeps theme, time zone, language and drawer/home settings; the email can only be set at sign-up; nothing deletes an account.
- **Proposed contract:** `POST /api/account/email` (sends a code to the new address, then `POST /api/account/email/verify`); `formats`, `default_agent`, `enter_sends` and `show_reply_tokens` in the preferences; `DELETE /api/account` with the password or a fresh code, after which every device is signed out.
- **Until then:** General leaves those rows out.

Found while building Settings › General (2026-11-12).

### Changing plan, and what was paid
- **Needed by:** *App / Settings / Plan & usage* (C0og49): "Join Pro waitlist" / "Join the waitlist", Billing's payment method, and spend shown as "Rp 1.800 ≈ $0.11 · you paid Rp 0".
- **Why:** a person can see the plans but can't move to one or ask to; nothing says how they pay or what they paid; spend is only in dollars.
- **What exists today:** `GET /api/plans` lists the plans with the person's current one; an admin sets a subscription (`PUT /api/admin/users/:id/subscription`); Xendit is configured in the admin, but nothing starts a payment for the person.
- **Proposed contract:** `POST /api/plans/:id/checkout` → `{ url }` (Xendit), or `POST /api/plans/:id/waitlist`; `payment_method` and `paid_idr` on `GET /api/usage`; a `usd_to_idr` rate with the usage.
- **Until then:** Compare plans has no buttons and says plans are changed by Nomi's team; spend is in dollars.

### A provider key's balance
- **Needed by:** *App / Settings / Plan & usage* (C0og49) → Your own API keys: "Balance empty" and "Use Nomi allowance".
- **Why:** when a key runs dry, the person learns of it from a failed reply instead of here.
- **What exists today:** `GET /api/llm/connections` gives each key's provider, label and masked key.
- **Proposed contract:** `status: "ok" | "balance_empty" | "rejected"` and `checked_at` on each connection, from the last call made with it.
- **Until then:** the row shows the key only.

Found while building Settings › Plan & usage (2026-11-12).

### What each model can do
- **Needed by:** *App / Settings / Models & providers* (RbTgo): the Context column and the Fast / Reasoning / Vision / Code tags on each model, and "2 recommended selected" in Connect (voUxB).
- **Why:** people pick models blind: nothing in the picker's list says how much a model reads or what it's good at, or which of a new provider's models to start with.
- **What exists today:** `GET /api/llm/model-options` gives each model's key, label, id, whether it's free and whether it's shown; `GET /api/llm/providers/:id/models` has catalog metadata (context, inputs, cost) for catalog providers only.
- **Proposed contract:** `context`, `capabilities: ["fast" | "reasoning" | "vision" | "code"]` and `recommended: bool` on each model in model-options and in `POST /api/llm/connections/:id/models`.
- **Until then:** no Context column or tags; a new connection's models start switched off.

Found while building Settings › Models & providers (2026-11-13).

### Memory settings, the profile and who learned what
- **Needed by:** *App / Settings / Memory* (hANTE): the "You" profile row (the summary Nomi keeps of the person, its size), Learned by on each topic, and the settings column (Learn from chats, Keep a profile, Search past chats, the Memory and Profile budgets with their use, Storage).
- **Why:** people can see and correct single memories, but not the profile the crew works from, which agent learned something, or switch learning off.
- **What exists today:** `GET /api/memory` lists memories (content, weight, kind, uses, last used, needs_check); `PUT`/`DELETE /api/memory/:id`, `POST /api/memory/:id/confirm`.
- **Proposed contract:** `GET|PUT /api/memory/settings` with `learn_from_chats`, `keep_profile`, `search_past_chats`, `memory_budget`, `profile_budget`; `GET|PUT /api/memory/profile` with `{ text, tokens, updated_at }`; `learned_by` (the agent type) on each memory.
- **Until then:** Memory & context lists and edits memories only.

Found while building Settings › Memory & context (2026-11-13).


### More apps to connect, MCP servers, and a connection's health
- **Needed by:** *App / Settings / Connections* (OBmgS): "Search 120+ apps" and the Add more catalog with its categories (For your crew, Work, Building, Health, Money, Home & media) and SUGGESTED FOR YOUR CREW; "Add MCP server" (top bar and the custom row); access per service on each chip ("Gmail · read", "Docs · write · ask"); "Needs you" with Reconnect when a sign-in has lapsed; who used a connection and when, for Google; and Manage's own page.
- **Why:** the crew can work in three apps; everything else the pen offers (Notion, Slack, Strava…) and any MCP server has nothing to connect to, and a lapsed Google or Hevy sign-in is only found when a reply fails.
- **What exists today:** `GET /api/connections` (Google: configured, email, services; Hevy: account, connected, last used), `GET|PUT|DELETE /api/connections/github`, Google and GitHub start/callback, `PUT|DELETE /api/connections/hevy`.
- **Proposed contract:** `GET /api/connections/catalog` → `[{ id, name, category, for_agent, access: ["read" | "write" | "ask"], available }]`; `GET|POST|DELETE /api/connections/mcp` with `{ url | command, name }`; on each connection `status: "ok" | "expired"`, `expired_at`, `last_used_at` and `used_by: [agent]`, and per Google service an access level.
- **Until then:** Connections shows Google Workspace, GitHub and Hevy, connected or to add; who uses each is the crew member it's for.

Found while building Settings › Connections (2026-11-13).

### A person's own crew: members, switching one off, permissions, soul.md, activity
- **Needed by:** *App / Settings / Crew* (vVUeh, New member u377E4, Templates u2auTN): "Add crew member" and New member, Templates, the On switch on each member (and "paused" in the roster), Permissions per member (Allow / Ask first / Never), soul.md with its versions and Edit, and Activity · N.
- **Why:** people can see who's on the crew and set Nomi's and Koda's models, but can't shape the crew: make a member of their own, turn one off, or limit what one may do without asking.
- **What exists today:** `GET /api/agents` (name, role, state, status, custom or not); custom members are made by admins (`/api/admin/dynamic-agents`); Nomi's chat model and Koda's coding model; the personality's versions (`GET /api/personality/history`, `POST /api/personality/rollback`).
- **Proposed contract:** `PUT /api/agents/:type` `{ enabled }`; `GET|PUT /api/agents/:type/permissions` `[{ key, label, level: "allow" | "ask" | "never" }]`; `GET|PUT /api/agents/:type/soul` `{ text, version }` with `/versions`; `GET /api/agents/:type/activity`; `POST /api/agents` (a person's own member, from scratch or a template) and `GET /api/agents/templates`.
- **Until then:** Crew lists the members, Nomi's and Koda's models, what each is doing, and the personality's versions with Restore.

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

### Projects: renaming, "Needs you", community apps
- **Needed by:** *App / Projects* (JyWON) and its flows: Rename and Duplicate in the card menu, the Needs attention tab and the Needs answer card (Koda waiting on questions), the failed and down states, and Community (apps to remix).
- **Why:** a project keeps the name "New project" forever, and people can't see which projects are waiting on them without opening each.
- **What exists today:** `GET /api/projects` (status `planning` / `building` / `ready`, `deploy_url`, counts), `POST /api/projects`, `DELETE /api/projects/:id`, `/showcase` per project (no listing).
- **Proposed contract:** `PATCH /api/projects/:id` `{ name, description }`; `POST /api/projects/:id/duplicate`; on each summary `waiting_on_you: number` (open questions), `last_deploy: { status: "ok" | "failed", at }` and `live_status: "up" | "down"`; `GET /api/showcase` (public apps to remix).
- **Until then:** cards are live, building or draft; Needs attention stays empty.

Found while building Projects (2026-11-14).

### A project's own domain (DNS)
- **Needed by:** *Flow / Build — Workspace tools* (U16wU) steps 8–9, Screen / Domain (J2hPJ) and Domain connected (Dd60u): a domain the person owns (warungkopibusiti.id) beside the app's address, the records to set at their registrar (A, CNAME, TXT, each found or waiting), "last checked · every 5 min", Check now, HTTPS issued, Make primary and Remove domain.
- **Why:** a business's app lives at Nomi's address only; people can't put it on their own domain.
- **What exists today:** `GET|PUT /api/projects/:id/domain` and `/domain/check`: the app's name under Nomi's apps domain only.
- **Proposed contract:** `POST /api/projects/:id/domains` `{ hostname }` → `{ hostname, records: [{ type, name, value, status: "found" | "waiting" }], checked_at, https: "pending" | "issued" }`; `POST /api/projects/:id/domains/:hostname/check`; `PUT /api/projects/:id/domains/:hostname/primary`; `DELETE /api/projects/:id/domains/:hostname`.
- **Until then:** Domain shows the app's address (and where it moves on the next publish) and renames it.

Found while building the workspace's Domain (2026-11-14).

### A project's database: a dump, and editing rows
- **Needed by:** *Flow / Build — Workspace tools* (U16wU) step 11, Screen / Database (xYgNi): Download dump and Edit rows.
- **Why:** people can see their app's data but not take a copy of it, or fix a wrong row without asking the crew.
- **What exists today:** `GET /api/projects/:id/database/tables`, `POST /database/query` (SQL as the owner), `POST /database/credentials`.
- **Proposed contract:** `GET /api/projects/:id/database/dump?source=` (a `pg_dump` file); row edits by primary key, `PATCH /api/projects/:id/database/tables/:schema/:table/rows` `{ key, changes }`, so the browser never sends SQL.
- **Until then:** Database reads tables and rows; changes go through Koda.

Found while building the workspace's Database (2026-11-14).

### A site's visits: where they came from, form sends, uptime
- **Needed by:** *Flow / Build — Landing page* (WUiji) step 5, Monitor for a site: "Where visits come from" (Instagram, Google search, WhatsApp links, Direct), Form sends with their share of visits, and Uptime.
- **Why:** a studio's site is judged by who finds it and who gets in touch, not by requests and errors.
- **What exists today:** `GET /api/projects/:id/deploy/metrics?range=`: visitors, requests, 4xx/5xx, p50/p95 per bucket. No referrers, form submissions or uptime.
- **Proposed contract:** add to the metrics answer `sources: [{ name, share }]` (from the referrer, grouped), `form_sends: number | null` (POSTs answered 2xx to the site's own forms) and `uptime: { share, since } | null`.
- **Until then:** a site's Monitor leads with visits and page load, its bars count visits, and the rest is left out.

Found while building the Landing page flow (2026-11-14).

### A message read on its own names its author
- **Needed by:** *Flow / Build — Invite & build together* (TnwQT) step 6: a teammate's message in the project chat, in their colour and name.
- **Why:** the list (`GET /api/sessions/:id/messages`) names someone else's message (`author_name`), but the single read a live update uses (`GET /api/sessions/:id/messages/:message_id`) doesn't, so a teammate's new message would look like the person's own.
- **What exists today:** `attach_authors` runs for the list only.
- **Proposed contract:** run `attach_authors` in `get_message` too.
- **Until then:** v2's server reads the newest page of the list for a person's message read on its own and takes the author from there.

Found while building Invite & build together (2026-11-14).
