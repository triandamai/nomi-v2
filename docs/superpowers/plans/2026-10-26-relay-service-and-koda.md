# Relay service and Koda on Relay: implementation plan

**Goal.** Every app Koda builds that keeps data, signs people in, stores files or needs trusted
server code uses Nomi Relay (`relay/`) instead of a backend written fresh each time. Koda writes
migrations, backend logic and the UI; Relay is the rest. The design is `relay/README.md`; this
plan is how to build it, in the order the README's phases give, with what "done" means for each.

**Today.** The client (`nomi-relay`, `relay/sdk/js`) and the CLI (`nomi-relay-cli`, `relay/cli`:
`init`, `migrate new|lint|status`, `functions new`) are published. The service doesn't exist.
Koda builds data apps with Drizzle and the PGlite/libsql preview switch
(`backend/crates/nomi-agent-coding/guides/database.md`), and Shipyard creates the production
database at first publish (`nomi_shipyard::manifest::detect_database`).

**Rule for the whole plan.** Each phase ships on its own, behind `projects.backend`
(`'drizzle' | 'relay'`, default `'drizzle'` until phase 2 is live), and leaves every existing
project working. Nothing here edits an app's data in place.

---

## Phase 0: the switch and the seams (in `backend/`)

Small, first, so later phases plug in without touching Koda's loop again.

1. **Migration `00xx_project_backend.sql`.** `projects.backend TEXT NOT NULL DEFAULT 'drizzle'
   CHECK (backend IN ('drizzle', 'relay'))`, plus `relay_project_id UUID` (null until the
   control plane makes one).
2. **`RelayOps` next to `ProjectOps`** (`nomi-agent-core/src/project_ops.rs` already shows the
   pattern: the agent crate sees a trait, the server implements it). Methods: `ensure_backend`,
   `apply_migrations(env)`, `describe_backend(env)`, `deploy_functions(env)`, `types(env)`.
   `NoRelayOps` answers "Relay isn't set up on this Nomi" so a Nomi without Relay keeps building
   Drizzle apps.
3. **Server client:** `backend/crates/nomi-relay-client` (control-plane HTTP calls with the
   service credential `RELAY_CONTROL_URL` / `RELAY_CONTROL_KEY`), the same shape as
   `nomi-shipyard`'s client, with its errors mapped to `problem` codes.
4. **Rena chooses the backend.** `create_project` gets `backend` from the plan: `relay` for
   anything that keeps data on a server, signs people in, stores files or calls paid APIs;
   `drizzle` never for new projects once phase 2 ships. Prompt change in
   `prompts::PLANNING_APP_PLAYBOOK`.

**Done when** the flag exists, Koda's tool list depends on it, and with no Relay configured
everything behaves exactly as today.

---

## Phase 1: control plane and migrations (`relay/crates/`)

The service: Rust/axum like `backend/`, its own Cargo workspace in `relay/`, run on Shipyard as
an ordinary app.

| Crate | What's in it |
|---|---|
| `relay-core` | Project, environment, plan tier, keys; the `nomi` schema SQL |
| `relay-control` | Control-plane routes (service credential only) |
| `relay-migrate` | Apply, record (`nomi.migrations` with checksums), lint (the same rules as `relay/cli/src/lint.ts`, ported, with the CLI's tests as fixtures), generate TypeScript types |
| `relay-server` | The binary: control plane now, gateway from phase 2 |

1. **Backends.** `PUT /control/projects/:id` makes the backend: paid projects use Shipyard's dev
   database and production environment database (`ensure_dev_database`); free projects get
   `app_<project>` schemas and roles (`_anon`, `_authenticated`, `_service`) in the shared
   databases. Installs the `nomi` schema (`users`, `identities`, `sessions`, `migrations`,
   `buckets`; `nomi.uid()`, `nomi.role()`, `nomi.claims()`), with `project_id` row-level
   security. Creates the signing key and the anon and service keys, and stores them as project
   secrets on Shipyard (`NOMI_URL`, `NOMI_ANON_KEY`, `NOMI_SERVICE_KEY`). `DELETE` drops exactly
   that project's database objects and rows.
2. **Migrations.** `POST /control/projects/:id/environments/:env/migrations` takes the files of
   `nomi/migrations/`, lints them, refuses an edited applied migration (checksum), applies the
   rest in order, one transaction each, as the project's owner role with a statement timeout, and
   answers what ran or the exact SQL error with file and line. Then `NOTIFY pgrst`.
3. **Types.** `GET .../types` answers `src/lib/nomi.types.ts` from the catalog.
4. **In Nomi:**
   - `create_project` with `backend = 'relay'` calls `ensure_backend`.
   - Koda gets `apply_migrations` and `describe_backend` (no approval: they act on the
     project's own dev backend), and writing under `nomi/migrations/` runs `apply_migrations`
     against `dev`, its result going back like `npm run check` output
     (`nomi-agent-coding/src/runtime.rs`).
   - After each successful apply, Nomi writes `src/lib/nomi.types.ts` into the project's files.
   - `publish` (`routes/deploy.rs::start_publish`) applies pending production migrations before
     sending the source; a failure stops the publish with the error and the running version
     keeps serving. The manifest binds `NOMI_*` by name like other secrets.
   - Koda's brief (`brief.rs`) lists the backend's tables and the secret names.
5. **Koda's guide `guides/relay.md`** replaces `database.md` for relay projects: the folder
   layout, migrations with row-level security from the first table, `nomi.uid()` in policies,
   the client calls, and "never edit an applied migration, write the next one".

**Tests.** Lint fixtures shared with the CLI; migrate/apply/refuse-edit against a throwaway
Postgres (`sqlx::test`); a free and a paid backend side by side proving one can't see the other.

**Done when** a new project's Koda writes `nomi/migrations/0001_init.sql`, sees it applied to
`dev` or gets its error, and publishing applies it to production.

---

## Phase 2: data API, auth, preview on real Postgres

1. **Gateway** in `relay-server`: hostname → project, key or user token checked against the
   project's signing key, then:
   - `/rest/*` → PostgREST with a short-lived internal token carrying the role and claims; for
     free projects `Accept-Profile`/`Content-Profile` set by the gateway, the client's dropped;
   - statement timeout, `db-max-rows` and request-size limits from the plan;
   - CORS: the app's origins, and Nomi's preview origins against `dev` only.
2. **PostgREST:** one per paid project environment (a Shipyard service next to its database),
   one per shared database for free projects.
3. **Auth** (`relay-auth`): email codes (shared sender with strict per-project limits, or the
   project's SMTP with `NOMI_SMTP_PASSWORD`), Google, GitHub, Apple (client secret signed from
   the `.p8`), any OIDC issuer; PKCE and `state`; the one callback URL per provider; linking by
   verified email; 15-minute access tokens and rotating refresh tokens with reuse detection;
   `auth.admin.*` with the service key.
4. **Preview.** For relay projects the WebContainer preview calls Relay `dev` over HTTPS; the
   PGlite switch isn't used. `guides/preview.md` and `guides/sveltekit.md` say so.
5. **Templates.** Koda's stack guides give relay projects `nomi-relay` (pinned minor), the client
   in `src/lib/nomi.ts`, and server code using the service key from the environment.
6. **CLI:** `login`, `link`, `migrate apply`, `types` (through the Nomi backend with a personal
   access token; the control plane stays service-to-service).
7. **Flip the default:** Rena creates every new data app with `backend = 'relay'`.

**Done when** Koda builds a sign-in + CRUD app with no server code of its own, it works the same
in the preview and in production, and the Data API refuses another user's rows.

---

## Phase 3: storage and realtime

- **Storage:** buckets from `backend.json`, an S3 prefix per project, signed URLs, per-bucket
  rules checked by the gateway with the caller's claims.
- **Realtime:** for each table in `backend.json`'s `realtime`, a trigger calling
  `pg_notify('nomi_realtime', …)` with project, table, op and primary key; one `LISTEN` per
  database in the gateway; per-subscriber row reads through the same role switch; at-most-once
  delivery with `resync` on reconnect and on bursts over quota.

**Done when** a chat-style app Koda builds updates live in two browsers, and a file upload is
only readable by whom the bucket rule says.

---

## Phase 4: hosted functions

- Bundle `nomi/functions/` into one Deno entry (Relay's router plus every handler), run as one
  Shipyard service per project and environment, sleeping when idle.
- The gateway proxies `/functions/<name>` after checking the key or token, holds calls while the
  service wakes, and calls cron `schedule`s with the service role.
- Koda gets `deploy_functions` (writing under `nomi/functions/` triggers it on `dev`); publish
  deploys them after migrations; rollback brings back the previous build.
- CLI: `functions serve`, `functions deploy`, `secrets set`.

**Done when** a static or SPA app gets trusted server code (a payment call) with Koda writing one
file.

---

## Phase 5: the console in the project sheets

The Database sheet (`frontend/src/lib/components/project/DatabasePanel.svelte`,
`routes/project_database.rs`) becomes the backend's console: tables and SQL (as now), users and
their sign-in methods, auth providers and their secrets (set, never shown), buckets and files,
functions with logs and deploys, migrations per environment, request logs. Iris
(`nomi-agent-inspector`) reads the same sources through `ProjectOps`, blind by default as she is
now.

---

## Koda: making Relay the default for every app that needs a backend

| Where | Change |
|---|---|
| `prompts::PLANNING_APP_PLAYBOOK` | The plan names the backend: Relay for any server-side data, sign-in, files or paid API; browser-only storage stays allowed for tools that need nothing shared |
| `create_project` | Takes `backend`; `relay` calls `ensure_backend` |
| `CODING_SYSTEM_PROMPT` | "Data on a server goes through Relay: migrations in `nomi/migrations/`, the `nomi-relay` client, functions for trusted code. Never write auth, sessions, uploads or a database connection yourself." Drizzle only for projects whose `backend` is `drizzle` |
| `guides.rs` | `relay` guide; `database` guide kept for drizzle projects only |
| `brief.rs` | Backend summary (tables, policies, buckets, functions, missing provider secrets) from `describe_backend` |
| Tools | `apply_migrations`, `describe_backend`, later `deploy_functions` (no approval: own dev backend) |
| `report.rs` | A failed migration or functions build counts as a failed check, so the build isn't "verified" |
| Iris | Relay request logs and auth events as more `ProjectOps` sources |

**Existing projects** keep `backend = 'drizzle'` and work as now. Moving one over is a Koda task
on request: write `nomi/migrations/` from the Drizzle schema, switch queries to the client; the
data stays in the same Shipyard database.

---

## Open questions, with a recommendation each

- **Free projects per shared database:** cap at 200 schemas per shared database (PostgREST
  schema-cache reloads stay well under a second at that size; measure in phase 1 and adjust).
  Open the next shared database automatically when one is full; a project never moves between
  shared databases.
- **Function wake timeout:** hold for 10 seconds, then `503` with `Retry-After: 2`; the SDK's
  `functions.invoke` retries once on that.
- **Where Relay's own state lives:** its own small Postgres (projects, keys, quotas, audit),
  separate from Nomi's, so the service can run and be backed up on its own.

## Verification at every phase

- `cargo test` for `relay/` (with a Postgres in CI, as `backend/` has), the CLI's lint fixtures
  run against the Rust linter too.
- One end-to-end script per phase: create a project through Nomi, have Koda build the phase's
  reference app, publish, and check it from outside (curl against the Data API with the anon key
  and with a user's token).
- Nothing reaches production users until the phase's reference app passes in both `dev` and
  production.
