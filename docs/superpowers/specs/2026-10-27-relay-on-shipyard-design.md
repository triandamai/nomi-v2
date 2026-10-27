# Relay on Shipyard: every project gets its own database and Relay

Date: 2026-10-27
Status: Draft, waiting for approval
Related:
- `relay/README.md`: Relay's design.
- `docs/superpowers/plans/2026-10-26-relay-service-and-koda.md`: how Relay was built.
- `backend/crates/nomi-shipyard/`: the Shipyard client.

## Why

Today one shared Relay serves every project, and an admin sets it up by hand. They run Relay
somewhere, then type its control address, control key, public domain, both shared database
URLs, both PostgREST addresses and the functions runtime into Admin → Integrations → Relay.
That's a server job, and the Configuration page (#91) exists to stop sending admins back to the
server.

The user's direction, in their words: forget the Relay in admin; create a new database for each
project; "once its done and published nomi ask shipyard for deployment and setup db and the
relay". That means:

- **Making a project** with a backend: Nomi asks Shipyard for a development environment for it.
  That environment holds a new Postgres and a Relay set up on it. Koda builds against that.
- **Publishing it:** Nomi asks Shipyard to deploy the app to production. Production gets its own
  Postgres and its own Relay, set up the same way.
- **No shared Relay, and nothing for an admin to set up.** Shipyard being configured is enough.

## Goals

1. Creating a Relay project makes its development backend on Shipyard by itself: database,
   PostgREST, Relay and functions. No admin step.
2. The first publish makes its production backend the same way, then deploys the app next to it.
3. Koda, Iris, the Relay console, the data sheet and the preview keep working, now pointed at
   the project's own Relay for each environment.
4. The Relay section of Admin → Integrations goes away.

## Non-goals

- Moving projects made on the shared Relay before this ships (see "Projects made before this").
- Plans, tiers or billing for the extra containers.
- Running Relay anywhere but Shipyard.

## Anti-goals (a failure even if it works)

- A publish that loses or resets an app's production data.
- A model, a log line or the browser ever seeing a control key, a database password or a signing
  secret.
- Making a project, or Koda's first message, waiting on a slow setup. The backend comes up in
  the background while Rena plans; Koda waits only if it truly isn't ready yet.
- One project able to reach another's data or Relay.

---

## How it works

### One stack, two environments

Every Relay project is one Shipyard app (it already is: `put_app` at project creation) with two
environments:

| Shipyard environment | When it's made | What runs in it |
|---|---|---|
| `dev` | When the project is created with `backend = 'relay'` | the stack: `relay`, `postgrest`, `functions`, and Postgres |
| `production` | At the first publish | the same stack plus the app's `web` service |

Both run the same stack, built from the same manifest code. A Relay in a stack holds exactly one
project and one environment: it's the only Relay this project's calls go to for that environment.

### What Nomi remembers

A new table, one row per project and environment:

```
project_relays (
  project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  env           TEXT NOT NULL CHECK (env IN ('dev', 'production')),
  url           TEXT,                       -- the relay service's public URL, once known
  control_key   BYTEA NOT NULL,             -- sealed with SETTINGS_ENCRYPTION_KEY
  version       TEXT NOT NULL,              -- the stack's Relay version
  state         TEXT NOT NULL CHECK (state IN ('creating', 'ready', 'sleeping', 'waking', 'failed')),
  last_used_at  TIMESTAMPTZ NOT NULL DEFAULT now(),  -- dev only: see "Sleeping development backends"
  step          TEXT,                       -- the step it's on or failed at
  error         TEXT,
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (project_id, env)
)
```

`relay_for(project, env)` reads this row and gives back the client for that project's Relay
(its URL and control key). Every current caller of the shared `RelayConfig` goes through it
instead:

- Koda's tools in `relay_ops.rs`;
- `apply_before_publish` and `functions_on_rollback`;
- the Relay console and data sheet (`routes/project_relay.rs`);
- Iris's queries;
- preview access.

When the row isn't `ready`, it answers the way "Relay isn't set up" answers today, but with what
is happening ("The project's backend is still being set up", or the failed step).

### Sleeping development backends

The user's rule: when development is inactive, shut down the database and Relay.

- **Using it.** Every call that `relay_for(project, 'dev')` serves sets `last_used_at`. That
  covers Koda's backend tools, the preview's access, the Relay console, the data sheet and
  Iris.
- **Going to sleep.** A worker job runs every `IDLE_CHECK_EVERY` (about 5 minutes). It finds
  `dev` rows that are `ready` and unused for `DEV_IDLE_AFTER` (a named constant, 1 hour to
  start), and asks Shipyard to stop that environment. Stopping keeps the database's data (see
  question 5). The row becomes `sleeping`.
- **Waking up.** `relay_for` on a `sleeping` row starts the environment and marks it `waking`.
  It waits until Relay answers, up to `BACKEND_READY_WAIT`, then marks it `ready` and answers
  the call. Opening the project in Nomi wakes it ahead of time: opening the session, the
  preview or the Relay console. So the first call usually doesn't wait.
- **What people see.** The preview and the Relay console show "Waking the backend…" while it
  starts. Koda's tools wait quietly, and answer "still waking" only if it takes longer than
  `BACKEND_READY_WAIT`.
- **Production never sleeps.**

### Making the development backend (when a project is created)

`ensure_backend` (called from `create_project` when Rena picks `relay`) stops calling the shared
Relay. It writes the row with state `creating` and queues a job on the worker. The job runs these
steps. Each one checks first and does only what's missing, so the job can run again after any
failure and pick up where it stopped.

1. **Secrets.** Nomi generates `RELAY_CONTROL_KEY`, `RELAY_FUNCTIONS_SECRET` and
   `RELAY_PGRST_PASSWORD` and stores them on the project's Shipyard app. They're scoped to
   `dev`, through the existing `project_secrets::store`. They are Nomi-managed: the project's
   Secrets sheet lists them as locked, and the secrets routes refuse to change or delete them.
   The control key is also kept sealed in the row.
2. **Deploy the stack.** Nomi publishes to `dev` a zip holding only the stack's Dockerfiles
   (`.nomi/stack/…`) with the stack manifest (below). It then follows that publish the way
   project publishes are followed (webhook, with polling as the fallback).
3. **Find it.** Nomi reads the `relay` service's URL from `get_env` into the row.
4. **Set it up.** Through that Relay's control API:
   - create the project's backend;
   - set the allowed origins to Nomi's preview origins;
   - set the sign-in code sender and file storage from Nomi's Configuration (SMTP and Storage,
     when set).
5. **Hand Koda the keys.** Nomi writes the dev URL and anon key where Koda's preview reads them,
   as it does today, and marks the row `ready`.

When it fails:
- the row says which step failed and why;
- the project's chat says the backend couldn't be set up, in Shipyard's or Relay's own words,
  with a **Try again** button that runs the job again;
- Koda's backend tools answer with the same message, so it can tell the user instead of guessing.

Koda doesn't wait for this to start planning or writing UI. The first tool that needs the
backend (`apply_migrations`, `describe_backend`) waits up to `BACKEND_READY_WAIT` (a named
constant, about 2 minutes) for `ready`. After that it answers "still being set up".

### Publishing for the first time

The app's `web` service needs `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` when it starts,
and those come from the production Relay. So the first publish deploys the stack first and the
app second:

1. **Secrets** for `production`, as in step 1 above. The row goes in with state `creating`.
2. **Stack first.** Nomi publishes the stack-only zip and manifest to `production`, waits for it,
   and reads the `relay` URL.
3. **Set up the backend in it**, through its control API, with the same calls
   `apply_before_publish` makes today:
   - create the backend;
   - auth from `nomi/backend.json`;
   - storage and the sign-in sender from Configuration;
   - apply `nomi/migrations/`;
   - realtime;
   - deploy the functions.

   Then Nomi stores `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` from Relay's answer as
   project secrets scoped to `production`, as `ensure_backend` does today.
4. **Then the app.** Nomi publishes the full source plus `.nomi/stack/…` with the full manifest:
   the stack services and `web`. The stack services haven't changed, so Shipyard keeps them as
   they are. `web` starts with its secrets bound.
5. **Allow the app.** Nomi sets the production Relay's allowed origins to the app's URL. It does
   this again whenever the app's domain changes. Then it marks the row `ready`.

When the first publish fails:
- the row records the step;
- the publisher is told in chat and by notification, as publish failures are today (#90);
- no `web` service is created.

Publishing again picks up from the first unfinished step.

Production starts empty, with the schema from the migrations, as it does today. Development data
is never copied to production.

### Every later publish

This is today's flow, aimed at the project's own production Relay. `apply_before_publish` runs
first: auth, storage, migrations, realtime, then functions. A failure stops the publish, and the
running version keeps serving. Then one publish with the full manifest.

### The stack manifest

`nomi_shipyard::manifest::relay_stack(version, has_functions)` gives these services.
`manifest::stack_files(version, has_functions)` gives the Dockerfiles that go into the zip.

| Service | Kind | Built from | Environment |
|---|---|---|---|
| `relay` | `web`, port 8080 | `.nomi/stack/relay/Dockerfile`: `FROM <relay image>:<version>` | See below |
| `postgrest` | `worker` | `.nomi/stack/postgrest/Dockerfile`: `FROM postgrest/postgrest:<pinned>` | See below |
| `functions` | `worker` | `.nomi/stack/functions/Dockerfile`: `FROM <functions image>:<version>` | `RELAY_FUNCTIONS_SECRET = {secret}` |

**`relay`'s environment:**
- `RELAY_DATABASE_URL = {database: url}`
- `RELAY_ENCRYPTION_KEY = {generate: hex32}`
- `RELAY_CONTROL_KEY = {secret}`
- `RELAY_PROJECT = <project id>` (standalone mode, below)
- `RELAY_ENV = dev | production`
- `RELAY_POSTGREST_URL = http://postgrest:3000`
- `RELAY_PGRST_PASSWORD = {secret}`
- `RELAY_FUNCTIONS_URL = http://functions:8000`
- `RELAY_FUNCTIONS_SECRET = {secret}`

**`postgrest`'s environment:**
- `PGRST_DB_URI = "postgresql://"`
- `PGHOST = {database: host}`
- `PGPORT = {database: port}`
- `PGDATABASE = {database: name}`
- `PGUSER = "authenticator"`
- `PGPASSWORD = {secret: RELAY_PGRST_PASSWORD}`

PostgREST's database library (libpq) fills in the URI from the `PG*` variables, so no URL is
pieced together.

**The rest of the manifest:**
- `database: { engine: "postgres" }`. The app's tables, the `nomi` schema and Relay's own state
  share that one database, in their own schemas.
- In `production`, the app's `web` service gets `NOMI_URL`, `NOMI_ANON_KEY` and
  `NOMI_SERVICE_KEY` bound by name. It doesn't get `DATABASE_URL`: a Relay app reaches its data
  through Relay.
- `domain.service = "web"` in production, so the app keeps its address. The `relay` service gets
  its own.
- In `dev`, there's no `web` service. The preview runs in the browser (WebContainers) and calls
  the dev Relay.

### Versions and upgrades

- `relay.yml` builds and publishes two images on every merge under `relay/`, tagged with the
  same version as the npm packages:
  - `relay-server`, from `relay/Dockerfile`;
  - `functions-runtime`, from `relay/functions-runtime/Dockerfile`.
- Nomi pins the version it was built against in one constant, `RELAY_STACK_VERSION`. The
  PostgREST tag is pinned the same way.
- **Production** upgrades when a publish sees a stack older than `RELAY_STACK_VERSION`: the
  publish's stack Dockerfiles name the new version, so the stack upgrades in the same publish.
  The row records the version once the publish succeeds.
- **Development** upgrades the next time Koda starts a turn on the project and the stack is
  older: the stack-only zip is published again to `dev` (the data stays in its database).

### Deleting, transferring, rolling back

- **Deleting a project** deletes its Shipyard app, as today. Both environments, both databases
  and both Relays go with it, and the rows go with the project.
- **Transferring the app** to another Shipyard org moves both environments. Nomi keeps the
  control keys, so Koda, Iris and the console still reach them.
- **Rolling back** redeploys `web` and the functions from the earlier publish. Migrations never
  go back, as today. The stack keeps its current version.

### Projects made before this

Projects already on the shared Relay keep working there. `relay_for` falls back to the shared
connection for a project that has no `project_relays` row. The shared connection is the one
saved today, read from the database, no longer editable in admin.

When no project uses the shared Relay any more, that fallback and `relay_connection` are
deleted. That's a follow-up, along with moving those projects' data.

---

## Relay changes (`relay/`)

1. **Standalone mode** (`RELAY_PROJECT`, `RELAY_ENV`):
   - The instance serves exactly one project in one environment, in its own database, using the
     `dedicated` tier's `public` schema.
   - Every request to its host is that project. There are no subdomains and no project lookup.
   - The project's public URL is the instance's own URL.
   - `PUT /control/projects/:id` is accepted for that id only.
   - Settings that only made sense for the shared Relay are ignored: shared database URLs,
     public domain and per-database PostgREST URLs.
2. **Settings from the environment.** At start, Relay reads `RELAY_POSTGREST_URL`,
   `RELAY_PGRST_PASSWORD`, `RELAY_FUNCTIONS_URL` and `RELAY_FUNCTIONS_SECRET`. It creates the
   `authenticator` login role with that password when the role is missing, and keeps
   configuring it with `ALTER ROLE … SET pgrst.*`, as it does now. In standalone mode Relay
   counts as configured from its environment alone, with no `PUT /control/settings` needed. The
   allowed origins, the sender and storage are still set through the control API.
3. **Images published** by `relay.yml` (see Versions).

The multi-project code stays in Relay for now. It still serves projects made before this.

## Nomi changes (`backend/`, `frontend/`)

- **`nomi-shipyard`:**
  - `manifest::relay_stack` and `manifest::stack_files`;
  - a `put_secret` call for env-scoped secrets, if it isn't there yet.
- **`nomi-server`:**
  - `project_relays.rs`: the table, `relay_for`, the backend job and its steps, and the
    two-step first publish wired into `routes/deploy.rs::start_publish`;
  - `relay_ops`, `project_relay` and preview access go through `relay_for`;
  - the secrets routes refuse Nomi-managed secrets;
  - `ensure_backend` queues the job instead of calling the shared Relay;
  - the chat message and **Try again** button when the backend job fails.
- **Migration** `00xx_project_relays.sql`.
- **Frontend:**
  - the Relay section leaves Admin → Integrations;
  - the Secrets sheet shows Nomi-managed secrets as locked;
  - the Relay console shows while the backend is being set up, and shows a failure with
    **Try again**;
  - the Deployments sheet lists the stack's services next to `web` (it already lists services);
  - the preview and the Relay console show "Waking the backend…" while a sleeping `dev`
    starts.
- **Koda's guide** (`guides/relay.md`): the backend can take a minute to come up after the
  project is made. The first publish also creates the production backend and takes longer.

## Order of work

Each step is its own PR, and each leaves everything working.

1. **Relay:** standalone mode, settings from the environment, the `authenticator` role, and
   images published.
2. **Nomi:** the table, `relay_for` (falling back to the shared Relay for everything), and the
   stack manifest code with its tests. No behaviour change yet.
3. **Nomi:** new Relay projects get their development stack on creation, and the Relay
   section leaves admin.
4. **Nomi:** idle development stacks sleep and wake (needs question 5 answered).
5. **Nomi:** first publish creates the production stack, and later publishes upgrade it.

## Testing

**Relay:**
- standalone mode serves only its project and refuses another id;
- a standalone instance counts as configured from its environment;
- the `authenticator` role is created once.

**Nomi**, with a fake Shipyard (wiremock, as the deploy tests use):
- the backend job runs every step in order;
- run twice, it creates nothing twice;
- a failure resumes from the failed step;
- `relay_for` picks the project's Relay, falls back for older projects, and explains
  "creating" and "failed";
- the first publish deploys the stack before the app, stores the three `NOMI_*` secrets, and
  leaves no `web` service when it fails;
- a later publish with a newer `RELAY_STACK_VERSION` writes the new version into the stack
  Dockerfiles;
- Nomi-managed secrets can't be changed or deleted;
- the stack manifest matches the tables above (snapshot);
- an idle `dev` row is stopped after `DEV_IDLE_AFTER`, a used one isn't, `production` never is;
- a call on a sleeping row starts the environment, waits for Relay, then answers.

**By hand, on a real Shipyard:**
- make a data app with Koda on a fresh project;
- see its dev data in the Relay console;
- publish it and publish it again;
- check that a second project can't reach the first one's data.

## Questions only the user can answer (they own Shipyard)

These decide whether the manifest works as written.

1. **Can services in one environment reach each other by service name**, for example
   `http://postgrest:3000`? Shipyard's docs mention a private network but not the names. If
   they can't, Shipyard needs to document the internal address, or add an env value form like
   `{ "service": "postgrest", "value": "internal_url" }`.
2. **Can Shipyard's database user create roles (`CREATEROLE`)?** Relay creates roles for each
   project: owner, anon, authenticated, and the `authenticator` login.
3. **Can a manifest service run a prebuilt image** (an `image` field)? This design uses one-line
   `FROM …` Dockerfiles in the zip, which works with what's documented today. An `image` field
   would skip that build step.
4. **Does every `web` service get its own public URL**, with `domain.service` keeping `web` as
   the app's main address?
5. **Can Nomi stop and start an environment, keeping its database's data?** That's for
   sleeping development backends. Today the client has `restart_env` and `delete_env`, and
   `replicas` can't go below 1. Nomi needs a stop and a start (`POST …/envs/{env}/stop` and
   `…/start`, or `replicas: 0`) that leave the database volume in place.
