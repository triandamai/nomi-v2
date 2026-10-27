# Relay on Shipyard: every project gets its own database and Relay

Date: 2026-10-27
Status: Draft, waiting for approval
Related: `relay/README.md` (Relay's design), `docs/superpowers/plans/2026-10-26-relay-service-and-koda.md`
(how Relay was built), `backend/crates/nomi-shipyard/` (the Shipyard client),
https://shipyard.trian.space/docs/nomi (Shipyard's integration API).

## Why

Today one shared Relay serves every project, and an admin sets it up by hand. They run Relay
somewhere, then type its control address, control key, public domain, both shared database
URLs, both PostgREST addresses and the functions runtime into Admin → Integrations → Relay.
That's a server job. The Configuration page (#91) exists so admins don't have to go back to the
server for this kind of thing.

The user's direction, in their words: forget the Relay in admin, create a new database for each
project, and "once its done and published nomi ask shipyard for deployment and setup db and the
relay". In practice:

- **Making a project** with a backend: Nomi asks Shipyard for the project's development
  backend, a new Postgres with a Relay set up on it. Koda builds against that.
- **Publishing it:** Nomi asks Shipyard for the project's production backend, set up the same
  way, then deploys the app pointed at it.
- **When development goes idle,** its database and Relay are shut down, and they wake up the
  next time the project is used.
- **No shared Relay, and nothing for an admin to set up.** Shipyard being configured is enough.

## Goals

1. Creating a Relay project makes its development backend on Shipyard by itself: database,
   PostgREST, Relay and functions. No admin step.
2. The first publish makes its production backend the same way, then deploys the app against it.
3. An idle development backend scales to zero and wakes when the project is used again.
4. Koda, Iris, the Relay console, the data sheet and the preview keep working, now pointed at
   the project's own Relay for each environment.
5. The Relay section leaves Admin → Integrations.

## Non-goals

- Moving projects made on the shared Relay before this ships (see "Projects made before this").
- Plans, tiers or billing for the extra containers.
- Running Relay anywhere but Shipyard.

## Anti-goals (a failure even if it works)

- A publish or a sleep that loses or resets a project's data.
- A model, a log line or the browser ever seeing a control key, a database password or a
  signing secret.
- Making a project, or Koda's first message, waiting on a slow setup. The backend comes up in
  the background while Rena plans; Koda waits only if it truly isn't ready yet.
- One project able to reach another's data or Relay.

---

## How it works

### Two Shipyard apps per Relay project

Shipyard gives each environment one address, and that address goes to one web service. The
app's `web` service needs it, and so does Relay. So a Relay project has two Shipyard apps in
its organization:

| Shipyard app | Environments | What runs in it | Address (Shipyard generates it) |
|---|---|---|---|
| `<app>` (as today) | `production`, at first publish | the app's `web` service | `https://<app>.<apps-domain>` |
| `<app>-backend` (new) | `dev`, when the project is made; `production`, at first publish | `relay`, `postgrest`, `functions`, and Postgres | `https://<app>-backend-dev.<apps-domain>` and `https://<app>-backend.<apps-domain>` |

- Each backend environment has its own database, created by the manifest
  (`database: { engine: "postgres" }`).
- Shipyard gives the database user free rein over its own database. That covers the roles
  Relay creates for each project: owner, anon, authenticated, and the `authenticator` login.
- The services in one environment reach each other by name (`http://postgrest:3000`) on
  Shipyard's private network.
- A Relay in a backend environment serves exactly one project and one environment.

### What Nomi remembers

A new table, one row per project and environment:

```
project_relays (
  project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  env           TEXT NOT NULL CHECK (env IN ('dev', 'production')),
  url           TEXT,                       -- the backend environment's address, once known
  control_key   BYTEA NOT NULL,             -- sealed with SETTINGS_ENCRYPTION_KEY
  version       TEXT NOT NULL,              -- the stack's Relay version
  state         TEXT NOT NULL CHECK (state IN ('creating', 'ready', 'sleeping', 'waking', 'failed')),
  step          TEXT,                       -- the step it's on, or failed at
  error         TEXT,
  last_used_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (project_id, env)
)
```

`relay_for(project, env)` reads this row and gives back the client for that project's Relay:
its address and control key.

Every current caller of the shared `RelayConfig` goes through it instead:
- Koda's tools (`relay_ops.rs`);
- `apply_before_publish` and `functions_on_rollback`;
- the Relay console and data sheet (`routes/project_relay.rs`);
- Iris's queries;
- preview access.

When the row isn't `ready`, it explains:
- "The project's backend is still being set up" (`creating`);
- "Waking the backend…" (`sleeping` or `waking`, see below);
- or the failed step, in Shipyard's or Relay's own words (`failed`).

### The backend manifest

`nomi_shipyard::manifest::relay_backend(version, env)` builds the manifest for a backend
environment. Every service runs a prebuilt image, so nothing is built and no source is sent.

| Service | Kind | Image |
|---|---|---|
| `relay` | `web`, port 8080 (the environment's address) | `<registry>/nomi-relay:<version>` |
| `postgrest` | `worker` | `postgrest/postgrest:<pinned>` |
| `functions` | `worker` | `<registry>/nomi-relay-functions:<version>` |

**`relay`'s environment:**
- `RELAY_DATABASE_URL = {database: url}`
- `RELAY_ENCRYPTION_KEY = {secret: RELAY_ENCRYPTION_KEY}`
- `RELAY_CONTROL_KEY = {secret: RELAY_CONTROL_KEY}`
- `RELAY_PROJECT = <project id>` and `RELAY_ENV = dev | production` (standalone mode, below)
- `RELAY_POSTGREST_URL = http://postgrest:3000`
- `RELAY_PGRST_PASSWORD = {secret: RELAY_PGRST_PASSWORD}`
- `RELAY_FUNCTIONS_URL = http://functions:8000`
- `RELAY_FUNCTIONS_SECRET = {secret: RELAY_FUNCTIONS_SECRET}`

**`postgrest`'s environment:**
- `PGRST_DB_URI = "postgresql://"`
- `PGHOST = {database: host}`
- `PGPORT = {database: port}`
- `PGDATABASE = {database: name}`
- `PGUSER = "authenticator"`
- `PGPASSWORD = {secret: RELAY_PGRST_PASSWORD}`

PostgREST's database library (libpq) fills the URI from these variables, so no URL is pieced
together.

**`functions`'s environment:** `RELAY_FUNCTIONS_SECRET = {secret: RELAY_FUNCTIONS_SECRET}`.

**Secrets.** The backend app's secrets are scoped to their environment (`PUT …/secrets/{name}`
with `environment`):
- `RELAY_CONTROL_KEY`: Nomi sets the value, because Nomi calls Relay with it. It's also kept
  sealed in the row.
- `RELAY_ENCRYPTION_KEY`, `RELAY_PGRST_PASSWORD` and `RELAY_FUNCTIONS_SECRET`: made by Shipyard
  (`{ "generate": "hex32" }`). Nomi never needs their values.

**The app's `web` service** in `<app>` gets `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY`
bound by name, as secrets scoped to `production`, exactly as today. It doesn't get
`DATABASE_URL`: a Relay app reaches its data through Relay.

### Setting up a backend environment

`ensure_backend` (called from `create_project` when Rena picks `relay`) stops calling the shared
Relay. Instead it writes the `dev` row as `creating` and queues the backend job on the worker.
The first publish queues the same job for `production`.

Each step checks first and does only what's missing. So after any failure the job can run again
and continue where it stopped.

1. **App.** `put_app(org, "<app>-backend", "<name> backend", dev_database: false)`.
2. **Secrets** for this environment, as listed above.
3. **Deploy.** Publish the backend manifest to the environment, with no source. Then follow it
   the way project publishes are followed: the webhook, with polling as the fallback.
4. **Address.** Read the environment's URL from `get_env` into the row.
5. **Set up Relay** through its control API:
   - create the project's backend;
   - set the allowed origins (Nomi's preview origins for `dev`, the app's address for
     `production`);
   - set the sign-in code sender and file storage from Nomi's Configuration (SMTP and Storage,
     when set).
6. **Keys.**
   - For `dev`: give the dev address and anon key to Koda's preview, as today.
   - For `production`: store `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` as `<app>`'s
     production secrets, as `ensure_backend` does today.

   Then mark the row `ready`.

When it fails, the row says which step failed and why. For `dev`:
- the project's chat says the backend couldn't be set up, with a **Try again** button that runs
  the job again;
- Koda's backend tools answer with the same message, so Koda can tell the user rather than
  guess.

Koda doesn't wait for this to start planning or writing UI. The first tool that needs the
backend (`apply_migrations`, `describe_backend`) waits up to `BACKEND_READY_WAIT` (a named
constant, about 2 minutes) for `ready`. After that it answers "still being set up".

### Publishing

**The first time:**

1. Run the backend job for `production` and wait for it.
2. Set up the production backend through its control API. These are the same calls
   `apply_before_publish` makes today:
   - auth, from `nomi/backend.json`;
   - storage;
   - apply `nomi/migrations/`;
   - realtime;
   - deploy the functions.
3. Publish the app to `<app>`, as today. Its `web` service starts with the three `NOMI_*`
   secrets bound.

If any step fails:
- the publish stops there;
- the publisher is told in chat and by notification, as publish failures are today (#90);
- no `web` service is created.

Publishing again picks up from the first unfinished step.

Production starts empty, with the schema from the migrations, as it does today. Development
data is never copied to production.

**Every later publish** is today's flow, aimed at the project's own production Relay:
1. `apply_before_publish`: auth, storage, migrations, realtime, then functions. A failure stops
   the publish, and the running version keeps serving.
2. Publish the app.

### Sleeping development backends

The user's rule: when development is inactive, shut down the database and Relay. Shipyard runs
on Swarm, so sleeping is scaling to zero.

- **What counts as use.** Every call `relay_for(project, 'dev')` serves sets `last_used_at`.
  That covers Koda's backend tools, the preview's access, the Relay console, the data sheet and
  Iris.
- **Going to sleep.**
  - A worker job runs every `IDLE_CHECK_EVERY` (about 5 minutes).
  - It finds `dev` rows that are `ready` and haven't been used for `DEV_IDLE_AFTER` (a named
    constant, 1 hour to start).
  - It scales every service of that backend environment to 0 replicas, the database included.
    The data stays on its volume. The row becomes `sleeping`.
- **Waking up.**
  - `relay_for` on a `sleeping` row scales the services back to 1 and marks the row `waking`.
  - It waits until Relay answers, up to `BACKEND_READY_WAIT`, then marks the row `ready` and
    answers the call.
  - Opening the project in Nomi wakes it ahead of time: the session, the preview or the Relay
    console. So the first real call usually doesn't wait.
- **What people see.** The preview and the Relay console show "Waking the backend…" while it
  starts. Koda's tools wait quietly, and answer "still waking" only if it takes longer than
  `BACKEND_READY_WAIT`.
- **Production never sleeps.**

### Versions and upgrades

- `relay.yml` builds and pushes the `nomi-relay` and `nomi-relay-functions` images on every
  merge under `relay/`. They're tagged with the same version as the npm packages.
- Nomi pins the version it was built against in one constant, `RELAY_STACK_VERSION`. The
  PostgREST tag is pinned the same way.
- When a backend's row is older than `RELAY_STACK_VERSION`, the backend's manifest is published
  again with the new tags:
  - for `production`, at its next publish, before `apply_before_publish`;
  - for `dev`, when it next wakes or Koda next starts a turn on the project.

  The data stays in its database. The row records the new version once the publish succeeds.

### Deleting, transferring, rolling back

- **Deleting** a project deletes both Shipyard apps: both environments, both databases and both
  Relays. The rows go with the project.
- **Transferring** the app to another Shipyard org transfers `<app>-backend` with it. Nomi keeps
  the control keys, so Koda, Iris and the console still reach it.
- **Rolling back** redeploys `web` and the functions from the earlier publish. Migrations never
  go back, as today. The backend keeps its current version.

### Projects made before this

Projects already on the shared Relay keep working there. `relay_for` falls back to the shared
connection (the one saved today, read from the database, no longer editable in admin) for a
project with no `project_relays` row.

Removing that fallback and `relay_connection`, and moving those projects' data, is a follow-up
for once nothing uses the shared Relay.

---

## Relay changes (`relay/`)

1. **Standalone mode** (`RELAY_PROJECT`, `RELAY_ENV`). The instance serves exactly one project
   in one environment, in its own database (the `dedicated` tier's `public` schema).
   - Every request to its host is that project: no subdomains, no project lookup.
   - Its public URL is the instance's own.
   - `PUT /control/projects/:id` is accepted for that id only.
   - The shared-Relay settings (shared database URLs, public domain, per-database PostgREST
     URLs) don't apply.
2. **Settings from the environment.** Relay reads `RELAY_POSTGREST_URL`,
   `RELAY_PGRST_PASSWORD`, `RELAY_FUNCTIONS_URL` and `RELAY_FUNCTIONS_SECRET` at start.
   - It creates the `authenticator` login role with that password when the role is missing,
     and keeps configuring it with `ALTER ROLE … SET pgrst.*` as it does now.
   - In standalone mode it counts as configured from its environment alone.
   - Allowed origins, the sender and storage are still set through the control API.
3. **Images** pushed by `relay.yml` (see Versions).

The multi-project code stays for now. It still serves projects made before this.

## Nomi changes (`backend/`, `frontend/`)

- **`nomi-shipyard`:**
  - `manifest::relay_backend(version, env)`;
  - `put_secret` with `environment` and `generate`;
  - `scale_env(org, app, env, replicas)`.
- **`nomi-server`:**
  - `project_relays.rs`: the table, `relay_for`, the backend job and its steps, and the sleep
    and wake job;
  - `routes/deploy.rs::start_publish` runs the production backend job first;
  - `relay_ops`, `project_relay` and preview access go through `relay_for`;
  - `ensure_backend` queues the job;
  - project deletion and transfer include `<app>-backend`;
  - the chat message and **Try again** when the backend job fails.
- **Migration** `00xx_project_relays.sql`.
- **Frontend:**
  - the Relay section leaves Admin → Integrations;
  - the Relay console and the preview show "Setting up the backend…", "Waking the backend…",
    or a failure with **Try again**;
  - the Deployments sheet shows the backend's services under the app's.
- **Koda's guide** (`guides/relay.md`): the backend can take a minute to come up after the
  project is made or after it's been idle, and the first publish also creates the production
  backend.

## Order of work

Each step is its own PR and leaves everything working.

1. **Relay:** standalone mode, settings from the environment, the `authenticator` role, images
   pushed.
2. **Nomi:** the table, `relay_for` (falling back to the shared Relay for everything), and the
   manifest and Shipyard client calls with their tests. No behaviour change yet.
3. **Nomi:** new Relay projects get their development backend on creation, and the Relay
   section leaves admin.
4. **Nomi:** idle development backends sleep and wake.
5. **Nomi:** the first publish creates the production backend, and later publishes upgrade it.

## Testing

**Relay:**
- standalone mode serves only its project and refuses another id;
- a standalone instance is configured from its environment;
- the `authenticator` role is created once.

**Nomi**, with a fake Shipyard (wiremock, as the deploy tests use):
- the backend job runs every step in order;
- run twice, it creates nothing twice;
- a failure resumes from the failed step;
- `relay_for` picks the project's Relay, falls back for older projects, and explains
  creating, waking and failed;
- the first publish sets up the production backend before the app, stores the three `NOMI_*`
  secrets, and leaves no `web` service when it fails;
- an idle `dev` is scaled to 0 after `DEV_IDLE_AFTER`, a used one isn't, and `production` never
  is;
- a call on a sleeping backend scales it back up, waits for Relay, then answers;
- a backend older than `RELAY_STACK_VERSION` is republished with the new tags;
- the backend manifest matches the tables above (snapshot).

**By hand, on a real Shipyard:**
- make a data app with Koda;
- see its dev data in the Relay console;
- leave it idle and come back;
- publish it, and publish again;
- check that a second project can't reach the first one's data.

## Shipyard: what the integration API needs

The user confirmed that Shipyard does all of this. The integration docs
(https://shipyard.trian.space/docs/nomi) don't show two of the pieces yet, so Nomi needs them
there:

1. **A prebuilt image in a manifest service.** The design assumes `services[].image` (no
   `path` or `dockerfile`, and no source needed). If the field has another name, only
   `manifest::relay_backend` changes.
2. **Scaling an environment's services to 0 and back, the database included.** The docs say
   "scale replicas to 0 via the dashboard", and manifest `replicas` starts at 1. Nomi would
   call something like `PATCH …/envs/{env}/scale` with `{ "replicas": 0 }`, which keeps
   volumes. If it's per service instead, `scale_env` loops over the services.
