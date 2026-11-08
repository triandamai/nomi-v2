# Project issue board

A project's issues as a kanban board. People can add their own columns. Two columns are fixed:
**Backlog** and **Ready to dev**. A new issue starts in Backlog. Moving an issue to Ready to dev
hands it to the crew, which picks it up and starts working on it.

## What's there today

- `project_issues` (migration 0055): `number`, `title`, `body`, `kind` (feature | enhancement |
  bug), `severity` (low | medium | high | critical), `status` (open | closed), `created_by`,
  `closed_at`.
- `routes/project_issues.rs`: list (filter by status/kind/severity), get, create, update, delete.
  Includes counts for the toolbar badge (`open_issues`).
- `frontend/src/lib/components/project/IssuesPanel.svelte`: a list with filters. "Ask Koda" posts
  the issue into the project's chat as a message (`m.issues_koda_message`).

The board extends all of this. It does not replace it.

## Data (migration `0104_issue_board.sql`)

```sql
CREATE TABLE project_issue_columns (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 40),
    position   INTEGER NOT NULL,
    -- 'backlog' | 'ready' for the two fixed columns; NULL for the ones people add.
    fixed      TEXT CHECK (fixed IN ('backlog', 'ready')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, fixed)
);

ALTER TABLE project_issues
    ADD COLUMN column_id   UUID REFERENCES project_issue_columns(id) ON DELETE SET NULL,
    ADD COLUMN position    DOUBLE PRECISION NOT NULL DEFAULT 0,  -- order inside the column
    ADD COLUMN start_on    DATE,
    ADD COLUMN due_on      DATE,                                  -- the deadline
    ADD COLUMN points      SMALLINT CHECK (points > 0),
    -- what the crew is doing with it: NULL until it reaches Ready to dev
    ADD COLUMN crew_state  TEXT CHECK (crew_state IN ('queued', 'working', 'done', 'failed')),
    -- who works it: the crew member the person assigned (default 'coding', Koda), changed to
    -- whoever the assignee delegates it to
    ADD COLUMN crew_agent  TEXT NOT NULL DEFAULT 'coding',
    ADD COLUMN assigned_by_agent TEXT,                            -- set when the crew reassigned it
    ADD COLUMN picked_up_at TIMESTAMPTZ,
    ADD CONSTRAINT issue_dates CHECK (start_on IS NULL OR due_on IS NULL OR start_on <= due_on);
```

- **Type becomes bug | task | enhancement.** Existing `feature` rows become `task`. Update the
  `kind` check and `KINDS`.
- **Every project gets the two fixed columns.** Backfill existing projects. New projects get them
  when the board is first read (`ensure_fixed_columns`, idempotent through `UNIQUE (project_id,
  fixed)`). Open issues with no column go to Backlog.
- **Ordering:** `position` is a float, so a move only rewrites the one row (it takes the midpoint
  of its new neighbours). Renumber the column if the gap gets smaller than
  `MIN_POSITION_GAP`.
- **Points:** any positive number, with Fibonacci values suggested in the picker
  (`POINT_SUGGESTIONS = [1, 2, 3, 5, 8, 13]`).

## API (in `routes/project_issues.rs`)

| Route | What it does |
|---|---|
| `GET /api/projects/:id/board` | Columns in order, each with its open issues in order |
| `POST /api/projects/:id/board/columns` | Add a column (name). It goes after the last column |
| `PATCH /api/projects/:id/board/columns/:col` | Rename or reorder. Fixed columns refuse both (409) |
| `DELETE /api/projects/:id/board/columns/:col` | Its issues go back to Backlog. Fixed columns refuse (409) |
| `POST /api/projects/:id/issues/:n/move` | `{column_id, before?, after?}`. Moving into Ready to dev queues it for the crew |

- **Create/update** gain `start_on`, `due_on` and `points`. Create always lands in Backlog.
- **Limits** are named consts: `MAX_COLUMNS`, `MAX_COLUMN_NAME_CHARS`.
- **Fixed columns:** the two fixed columns always stay first (Backlog, then Ready to dev). Added
  columns can be ordered only after them.
- **Who can do what:** the same rules as issues today. Members can move issues; admins can add,
  rename or delete columns.
- **Live updates:** every change calls `project_events::changed(project_id, "issues.changed")`.
  This uses the existing envelope, with no new relay. Open boards refetch.

## The crew picks it up

Moving an issue into **Ready to dev** sets `crew_state = 'queued'`. A small worker takes it from
there, built like `memory_worker` (`issue_worker.rs` in nomi-server):

1. **Claim.** Every `ISSUE_PICKUP_EVERY` (5s), claim queued issues per project, in Ready to dev
   order, with `FOR UPDATE SKIP LOCKED`. A project runs at most **`issues_per_project`**
   issues at once (see the setting below; default 1). With 1, an issue also waits while the
   crew is running a turn in the project's chat, since they'd edit the same files.
2. **Start.** Set `working` and `picked_up_at`. Post a message to the project's chat with a new
   `issue` content block (number, title, type, severity, points, deadline, assignee) and the
   issue as the text, addressed to the assigned crew member. This runs the turn through the same
   pipeline as "Ask Koda", so the work shows in the chat, the pinned activity card (with its
   hand-over steps) and the logs. With more than one running, each issue gets its own agent
   session so their turns and activity cards don't mix. The instruction for how to treat an
   issue goes in `prompts.rs` (`ISSUE_PICKUP_PROMPT`): "work the issue, say what changed, don't
   publish unless asked; if another crew member is better suited, delegate it".
3. **Delegating.** The assigned crew member can hand the issue to someone better suited (Koda
   finds it's a data question and gives it to Dana) through the existing `delegate_to_agent`.
   When a delegation comes from an issue's session, the worker updates `crew_agent` to the new
   crew member and sets `assigned_by_agent`. The card then reads "Dana is working (from Koda)",
   and `issues.changed` goes out.
4. **Finish.** When the turn ends, set `done`, or `failed` if the run failed or was stopped, and
   publish `issues.changed`. **A done issue stays in Ready to dev, marked Done.** It isn't closed
   or moved; the person moves or closes it when they've checked it. A failed issue stays there
   too, marked failed, with a "Try again" action that sets it back to queued.
5. **Moving it back.** If someone moves a queued issue out of Ready to dev, it's no longer
   queued. A working one keeps going; stopping it is the chat's Stop button.

Log each step: `issue board: picked up #12 for Koda`, `issue board: Koda handed #12 to Dana`,
`issue board: #12 done`, and so on.

### The parallel setting (Admin → Configuration)

- A field in the existing Agent loop section of `nomi-settings/src/config.rs`, next to
  "Turns at once": `saved_only("issues_per_project", Kind::Number)`, stored in `app_config` like the other
  fields, shown on the existing Configuration page.
- Default `DEFAULT_PARALLEL_ISSUES = 1`, clamped to `1..=MAX_PARALLEL_ISSUES` (5).
- **Caveat at more than 1:** parallel issues work on the same project files at the same time.
  The pick-up prompt tells each crew member which other issues are in progress so they stay out
  of each other's files. Separate branches or workspaces per issue are out of scope here, and
  the admin page says so beside the field.

## Frontend (`frontend/` only)

- **IssuesPanel** gets a Board / List switch. The board is the default view, and the list stays
  as it is.
- **Board:**
  - Columns scroll sideways inside the panel, never the page.
  - On a phone (390px) it shows one column at a time, with a column picker at the top.
  - The two fixed columns have no rename/delete menu. An "Add column" button comes after the
    last column.
- **Card:** title, one line (type · severity · points), and one quiet meta line (deadline, or
  "Koda is working" / "Dana is working (from Koda)" / "Done" / "Couldn't finish"), with the
  assignee's crew avatar. There are no accent stripes and no status
  dots (frontend/CLAUDE.md).
- **Moving:**
  - Drag and drop with the pointer.
  - From the keyboard and on touch, a "Move to…" menu on each card, which also shows the
    hover/focus-visible states and 44px targets.
- **Issue form:** title, description, type (bug, task, enhancement), severity, points, start
  date, deadline and assignee (a crew picker, Koda by default). A deadline before the start date is refused, both client-side and by the
  database.
- **In the chat:** the new `issue` block shows the issue (`ContentBlockView`), and its number
  opens it on the board.
- **Strings:** new strings go in `messages/en.json` and `id.json`.

## Tests

- **Backend (`tests/project_issues.rs`):**
  - Fixed columns are created once and can't be renamed, moved or deleted.
  - Deleting a column sends its issues to Backlog.
  - A new issue lands in Backlog.
  - A move between two issues orders it between them.
  - A deadline before the start date is refused.
  - `feature` becomes `task`.
- **Worker:**
  - Moving to Ready to dev queues the issue.
  - One issue at a time per project.
  - Ready-to-dev order is respected.
  - Moving back out unqueues it.
  - A failed run marks the issue failed and can be retried.
  - A finished issue stays in Ready to dev, marked done, still open.
  - A delegation from the issue's session reassigns it to the new crew member.
  - With `issues_per_project = 2`, two issues run and the third waits; with the field unset,
    one at a time.
  - The pick-up opens the issue's own chat with the issue addressed to its crew member.
- **Frontend:**
  - The board renders columns in order, and fixed columns have no menu.
  - "Move to…" moves a card.
  - The phone layout shows one column at a time.
  - The form validates dates.

## Order of work

1. Migration, the type rename, fixed columns and backfill. Board routes and tests.
2. Move route, the worker, delegation reassigning and the parallel setting, with their tests.
3. The issue's chat: `chat_project()` maps it to the project, and every agent there gets the
   issue's context in its prompt (`issue_chat.rs`).
4. Board UI, form fields, phone layout and strings. Check it at 390px and in dark mode.

## Decisions

- **A finished issue is marked Done.** It stays in Ready to dev, still open, until a person
  moves or closes it.
- **The person picks the crew member** (Koda by default). That crew member can delegate it to
  another, and the issue's assignee follows.
- **One issue at a time per project by default.** Admins can raise it in Admin → Configuration
  (`agent_loop.issues_per_project`, up to 5).

## As built

- **One chat per issue.** Turns run one at a time per chat, so the worker opens a new chat for each
  issue (as the person who queued it, titled "#12 · title") instead of using the project's chat.
  That is what makes `issues_per_project` above 1 run in parallel. The `chat_project()` SQL
  function maps an issue chat back to its project, for the crew's tools and for who can open it.
- **The crew member is picked by addressing them.** The chat's first message starts "@Koda
  Issue #12: …", which routes the turn to them.
- **No `issue` content block.** The first message carries the issue as text, and the board links
  to the chat ("Open its chat").
