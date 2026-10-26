# Project sheets redesign: deployments, monitor, presence and activity

**Status:** agreed in conversation on 2026-10-26; ready to plan.

## Why

A project's tool sheets (Deploy, Monitor, Database, Members, Issues) open over the project's
chat. Today they work, but they read as a first pass: a failed publish is hard to see and explain,
Monitor only knows what it sampled while open, and a teammate working in the same project is
invisible until their messages land. The owner wants every sheet to look like it belongs to Nomi's
design system and to answer the question each one exists for:

- **Deployments:** what's live, what went wrong, and how to get back to a version that worked.
- **Monitor:** how the live app is being used and whether it's healthy.
- **Members:** who's here and what's happening in the project right now.

## Goals

1. Deployments sheet rebuilt to the agreed mockup, with failed publishes shown in place, a details
   sheet with steps and logs, and rollback that works end to end.
2. When a publish fails, the crew knows about it (Koda first), without anyone starting work.
3. Monitor rebuilt to the agreed mockup on what Shipyard gives Nomi today, plus a written to-do
   list for what Shipyard has to add.
4. Typing indicators in the project chat, and a live Activity tab with "Here now" in Members.

## Non-goals (this round)

- **The Relay sheet.** The Database sheet becomes the Relay console (Relay's control plane, the
  dev and prod backends by privilege, tables, data, queries) only once the Relay service exists.
  Until then the Database sheet stays exactly as it is. No interim redesign.
- **Issues as a board.** The Kanban with a fixed "Ready to pick up" column that the crew takes work
  from comes later, as its own design.
- **Shipyard changes.** This round writes the Shipyard to-do list; it doesn't change Shipyard.
- Koda starting a fix by himself after a failed publish.

## Anti-goals (a failure even if it works)

- Anything that reads as AI-generated, per `frontend/CLAUDE.md`: accent stripes on cards,
  uppercase eyebrow labels with icons, glowing or pulsing status dots, gradient text, emoji icons,
  `#fff` surfaces.
- Numbers on screen that Nomi can't actually back. A metric Shipyard doesn't provide is not
  faked: it isn't shown, or shown as unavailable, until Shipyard provides it.
- A failed publish disappearing from the list, or showing only "failed" with no reason.
- A rollback button that can fail in a way the person can't understand.

## Constraints

- Svelte 5 + SvelteKit frontend. Sheets live in `frontend/src/lib/components/project/` and open
  in `routes/(app)/projects/session/[sessionId]/+page.svelte` inside `BottomSheet` +
  `SheetFrame`. Keep that mechanism (and `?sheet=` deep links).
- Colours, type and shapes only from `frontend/src/lib/styles/material3.css`: pages on
  `--nomi-color-ground`, cards on `--md-sys-color-surface-container-lowest`, things inside cards
  on `surface-container-low`, theme roles for status (primary/error containers); brand type
  `--md-ref-typeface-brand` for titles. Single-series charts use `--nomi-chart`.
- Every interactive element has hover and `focus-visible` states and a 44px target on phones.
  Every screen is checked at 390px wide and in dark mode.
- All copy goes through paraglide messages (`frontend/messages/*.json`) like the rest of the
  app; no hardcoded strings.
- Backend: Rust/axum `nomi-server`; live page updates use the existing project topic
  (`project/{id}/stream`, `project_events`) and the chat topic. LLM prompts live in
  `nomi-agent-core/src/prompts.rs`; numeric literals get named constants.
- Each part ships as its own PR, with tests, leaving everything else working.
- On phones the sheets are full screen; the details sheet stacks over the list with a back arrow.
  The mockups show desktop width; the phone layout follows the same order, one column.

## 1. Deployments

Agreed mockup: [`assets/2026-10-26-project-sheets/deployments.html`](assets/2026-10-26-project-sheets/deployments.html)
(screenshots: [light, approved](assets/2026-10-26-project-sheets/deployments-light.png),
[dark](assets/2026-10-26-project-sheets/deployments-dark.png)). Summary of what it shows:

**The Deployments sheet**
- Title bar: "Deployments", the project name and environment as subtitle, a filled **Publish**
  button, close.
- The meta line says who started a publish: the person, or the agent when it came from an agent's
  `publish_project` ("Koda"). Today `project_publishes.created_by` only records the person; add
  what's needed to tell the two apart.
- Environment tabs (filter chips) when the project has more than one environment.
- A **Live** card: "Live since <relative time>", the app's address (opens in a new tab), "Version
  N · published by <person>", an **Open** button. When nothing is live: an empty state that says
  so and offers Publish.
- The list of publishes, newest first, as rows (not cards) inside one container. Each row: a
  rounded status square (running: neutral/yellow container with "…", failed: error container with
  "!", succeeded: primary container with "✓"), "Version N" with a small tag (Running / Failed /
  Live), and a meta line: when, who started it (a person, or Koda), and duration or the step it
  broke at.
- **A failed publish keeps its row** and shows, inside the row, where it broke ("Build › npm run
  build") and Shipyard's error message in an error-container box. This is the "failed
  placeholder": a failure is never just a gap or a bare "failed".
- A running publish updates live (it already can: the page hears `DeployChanged` on the project
  topic).
- **Rollback** is offered on every earlier *successful* publish of that environment (a row action
  and in the details sheet), never on the live one or a failed one. It asks for confirmation:
  "Go back to version N? The app's code returns to version N; its secrets stay as they are now."

**The publish details sheet** (opens over the list, back arrow returns)
- Title "Version N", subtitle status · time · environment.
- For a failed publish, first: an error callout with where it broke and Shipyard's message, plus
  "Version M is still live" (when true) and "Koda has been told in the project chat". Buttons:
  **Ask Koda to fix it** (sends the existing fix suggestion into the project chat) and **Publish
  again**.
- The steps, grouped by phase (validate, build, configure, deploy, finalize), each with its
  status square, name and duration; skipped steps shown as skipped. The failed step is
  highlighted and its log is open below by default; tapping another step shows that step's log.
- The log in a dark monospace block; error lines in the error colour.
- For a succeeded publish: the address it went live at, and **Roll back to this version** when
  it's not the live one.
- Dark mode: the error containers are toned down from the mockup (the mockup's dark red is too
  heavy); use the theme's error-container roles and check contrast.

**Data.** Already available: `GET /api/projects/:id/deploy` (overview), `…/deploy/publishes/:id`
(publish + its deployments), `…/deploy/deployments/:deployment_id` (Shipyard's deployment with
steps and logs), `project_publishes.failure` (step, phase, message, log tail, runtime logs),
`created_by`, `rollback_of`. The builder adds what's missing for the meta line (e.g. the name of
who started a publish, its duration) to the overview response rather than fetching per row.

**Rollback must work end to end.** `POST …/deploy/publishes/:id/rollback` re-publishes the old
publish's kept source with today's secrets. The builder:
- covers it with tests (a successful rollback creates a publish with `rollback_of` set and the
  old source; a publish whose source wasn't kept answers a clear, translated reason; only admins
  can roll back; a failed publish can't be rolled back to);
- tries it against a real Shipyard environment once and fixes what breaks;
- makes the UI show the rollback as a new publish in the list ("Rolled back to version N") that
  succeeds or fails like any other.

**Koda is told about a failed publish.** Today a failure already posts an ordinary message in the
project chat (where it broke, the end of the log, a "fix it" suggestion button). Add: until a
later publish to that environment succeeds, Koda's brief (`nomi-agent-coding/src/brief.rs`)
carries a short "The last publish to <env> failed at <phase › step>: <message>" line, so any build
he starts sees it first. Nothing starts on its own.

## 2. Monitor

Agreed mockup: [`assets/2026-10-26-project-sheets/monitor.html`](assets/2026-10-26-project-sheets/monitor.html)
([screenshot](assets/2026-10-26-project-sheets/monitor-light.png)). The mockup's numbers are
made up to show the layout.

- Title bar: "Monitor", subtitle "<project> · <environment> · running · up <duration>" (or the
  service's real state), a 1h / 24h / 7d segmented switch, close.
- Four number tiles: Visitors, Requests, Errors, Response time, each with a one-line comparison
  or detail. **Shipyard doesn't provide these today**, so until it does they're not shown as
  numbers; the builder decides between hiding the row and showing it quietly as "available once
  Shipyard reports it" (no fake values; see anti-goals).
- Visitors per hour (bar chart). Also needs Shipyard.
- CPU and memory over the selected period, side by side (stacked on phones), with "now" and
  "peak". Today Shipyard gives a live snapshot per container; the sheet keeps sampling while open
  as it does now, and the history chart covers only what it sampled until Shipyard gives history.
  Several containers: one line each, or a container picker. Builder's call.
- Events: crashes, resource alerts, versions going live (from `…/deploy/events`), as rows with a
  status square, sentence and time.
- Logs: All / Warnings / Errors filter, search, **Follow live**, monospace dark block (from
  `…/deploy/logs`).

**Shipyard to-do list.** Write `docs/shipyard-todo.md` (or add to an existing Shipyard notes
file) listing, for each missing metric, what Nomi needs: unique visitors per hour and per day;
request count, error count (5xx, and 4xx separately), p50/p95 response time per bucket; CPU and
memory history per container at 1-minute resolution for 24h and 5-minute for 7d; the API shape
Nomi would call (`GET …/envs/:env/metrics?range=24h&step=1h`) and how the webhook/alerts relate.
When Shipyard ships them, the tiles and charts light up without a redesign.

## 3. Presence, typing and activity

**Typing in the project chat.** While another person is writing in the project's chat, everyone
else with that chat open sees "<Name> is typing…" just above the composer, where the crew's
progress shows today. It goes away a few seconds after they stop or when they send. Two or more:
"<A> and <B> are typing…". A person never sees their own typing.

**Members → Activity.** The Members sheet gets two tabs: **People** (today's content) and
**Activity**.
- **Here now** at the top of Activity: avatars and names of the people who have the project open
  right now.
- Below, a live feed, newest first, of what happens in the project, in plain sentences with who,
  what and when:
  - <person> asked <agent> to … (a prompt in the project chat, shortened)
  - Koda started building / is building (version N) / finished / stopped
  - Koda changed N files (from file-change events)
  - <person or Koda> published version N to <env>: live, or failed at <phase › step>
  - <person> rolled back <env> to version N
  - <person> joined or left the project; access changed
- It loads the recent history (say the last 50 items, with "show more") and appends live.

**Here now in the toolbar (proposed, owner to confirm while reading).** A small stack of the
avatars of others who have the project open, in the project's toolbar; tapping it opens
Members → Activity.

**Mechanics (builder's call within these rules).** Presence and typing are short-lived: they must
not be written to messages, and must clear themselves when a tab closes or a phone sleeps (a
heartbeat with an expiry, not an explicit "left" only). They travel on the project's live topic
so only people who can open the project hear them. The activity feed may be derived from
existing tables (messages, delegations, publishes, file changes, members) or recorded in a new
table at the moment each thing happens; whichever keeps it accurate and cheap to load.

## 4. Later (designed separately)

- **Relay sheet** replacing Database once Relay's service runs: Relay's control plane for the
  project, dev and production backends shown to those with the privilege (production for admins),
  tables, data and a query runner.
- **Issues board**: Kanban with one fixed column, "Ready to pick up", that the crew takes issues
  from automatically.

## Order of work

1. Deployments (sheet, details sheet, rollback end to end, Koda's brief line).
2. Monitor (sheet on today's data, Shipyard to-do doc).
3. Presence and activity (typing, Here now, Activity tab).

Each is its own PR.

## Calls made without the owner (check while reading)

- Rollback lives on earlier successful publishes only, with a confirmation that says secrets stay
  as they are now.
- Koda is "told" through his brief plus the existing chat message; he doesn't start a fix.
- Metrics Shipyard doesn't provide are hidden or marked unavailable, never shown as numbers.
- The avatar stack in the toolbar (proposed above).
- The Database sheet is untouched until Relay exists.
