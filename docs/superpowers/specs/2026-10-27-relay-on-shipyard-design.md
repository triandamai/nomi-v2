# Relay on Shipyard: one-button development Relay, and a Relay stack per published app

Date: 2026-10-27
Status: Draft, waiting for approval
Related: `relay/README.md` (Relay's design), `docs/superpowers/plans/2026-10-26-relay-service-and-koda.md`
(how Relay was built), `backend/crates/nomi-shipyard/` (the Shipyard client).

## Why

Today an admin sets Relay up by hand: they run Relay somewhere, then type its control address,
control key, public domain, both shared database URLs, both PostgREST addresses and the
functions runtime into Admin → Integrations → Relay. That's a server job, and the Configuration
page (#91) exists to stop sending admins back to the server.

There's also a gap in how Relay was meant to work. Its README says paid projects get their own
database and PostgREST. But in practice every published app shares one production Relay.

What the user asked for, in their words: "each user will have their own db and rest, but for
development use shared". In practice:

- **Development is shared.** One Relay that Koda builds against while a project is being made.
  The admin sets it up with one button, and Nomi provisions it through Shipyard.
- **Production is per app.** When someone publishes, Nomi sends Shipyard the app *and* a full
  Relay stack (Postgres, PostgREST, Relay, functions) in the same manifest. Each published app
  owns its data and its API.

## Goals

1. Admin → Integrations → Relay has one button that sets up the development Relay on
   Shipyard. Pressed again, it checks everything and repairs whatever is missing. No addresses
   or keys are typed.
2. The first publish of a Relay project creates its own Relay stack in its Shipyard
   environment. Later publishes update that stack, then the app.
3. Koda, Iris, the Relay console and the data sheet keep working without knowing which Relay
   they talk to. Nomi picks the right one: the shared one for `dev`, the app's own for
   `production`.
4. Nothing already published breaks.

## Non-goals

- **Moving apps already published on the shared production Relay to their own stack.** They
  keep running where they are. Moving one needs a data copy; that's a later piece of work.
- **A per-project development stack.** Development stays shared.
- **Plans, tiers and billing** for the extra containers.
- **Running Relay anywhere but Shipyard.** The manual fields stay for that case (see below).

## Anti-goals (what would make this a failure even if it works)

- A publish that loses or resets an app's production data.
- A model, a log line or the browser ever seeing a control key, a database password or a
  signing secret.
- Publishing gets noticeably slower on every publish. Only the first publish of a Relay app
  may take longer.
- An admin has to understand PostgREST to press the button.

---

## Part 1: the development Relay, set up with one button

### What the admin sees

On Admin → Integrations → Relay, a **Development Relay** card shows one of four states:

| State | What it says | Action |
|---|---|---|
| Not set up | "Koda needs a development Relay to build apps that keep data." | **Set up on Shipyard** |
| Setting up | The steps below, each ticking off as it finishes | none (it keeps going if they leave the page) |
| Ready | Its address, Relay's version and "All checks passed" | **Check again** |
| Needs attention | Which step or check failed, in Relay's or Shipyard's own words | **Fix it** (runs the same setup again) |

- Needs `integrations` at `manage`. Viewing needs `view`.
- Without Shipyard configured, the button is off and the card says Shipyard must be set up
  first.
- The manual fields move under **Use a Relay you run yourself**, collapsed. They're kept for
  self-hosting and for setups already saved by hand. A connection saved by hand is never
  overwritten by the button without the admin confirming.

### What the button does

Every step is idempotent: it checks, then creates only what's missing. That makes "Fix it" and
"Check again" the same code path.

1. **Platform org and app on Shipyard.** `put_org(PLATFORM_ORG, "Nomi platform")`, then
   `put_app(PLATFORM_ORG, "relay-dev", "Relay (development)", dev_database: false)`.
   `PLATFORM_ORG` is a constant (`nomi-platform`).
2. **Secrets.** Nomi generates and stores these as Shipyard secrets on that app, but only when
   they're missing:
   - `RELAY_CONTROL_KEY` (32 random bytes, hex);
   - `RELAY_FUNCTIONS_SECRET`;
   - `RELAY_PGRST_PASSWORD`.

   Nomi keeps the control key and functions secret sealed in its own database
   (`relay_connection`, as today). It never shows them.
3. **Publish the stack** to `production` of that app, with the manifest in "The stack
   manifest" below. The database is `{ "engine": "postgres" }`. Relay keeps its own state and
   the shared development data in it, in separate schemas (`relay`, `nomi`, `app_*`), which
   Relay already supports.
4. **Wait** for the publish, following it the way project publishes are followed (webhook,
   with polling as the fallback). Then read the `relay` service's URL from `get_env`.
5. **Save the connection.** Control URL = that service's URL; control key from step 2;
   `source = "shipyard"`.
6. **Save Relay's settings** through the existing `PUT /control/settings`:
   - the public URL;
   - the default preview origins;
   - the functions runtime's internal address and the secret from step 2;
   - from Nomi's Configuration (#91), when set: the SMTP sender (Relay's shared sign-in code
     sender) and the Storage bucket (where project files go).

   The database URLs and the PostgREST address are *not* sent from Nomi. Relay gets them from
   its own environment (see "Relay changes").
7. **Run Relay's checks** (they already exist) and show them on the card.

Steps run in a background job (the same worker that runs turns), so a closed tab doesn't stop
them. The card reads the job's progress.

### Settings that leave the admin page

Once the button is used, these are filled by Shipyard and no longer typed:

- the control address and key;
- the public domain;
- the shared dev database URL;
- the dev PostgREST address and its login role;
- the functions address and secret.

These are still set by the admin, on the Relay page:

- preview origins (prefilled);
- per-project limits, when they come.

The **shared production** fields stay visible only while some project still uses the shared
production Relay (see Part 2, "Apps already published").

---

## Part 2: a Relay stack in every published app

### The rule

A project with `backend = 'relay'` whose first publish happens after this ships gets its own
stack in its own Shipyard environment. Its `production` environment in Relay's sense now lives
there, not on the shared Relay.

Nomi remembers this per project in a new table:

```
project_relay_stacks (
  project_id      UUID PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  shipyard_env    TEXT NOT NULL,               -- 'production'
  control_url     TEXT,                        -- the stack's relay service URL, once known
  control_key     BYTEA NOT NULL,              -- sealed with SETTINGS_ENCRYPTION_KEY
  relay_version   TEXT NOT NULL,               -- the stack's images, see "Versions"
  state           TEXT NOT NULL,               -- 'creating' | 'ready' | 'failed'
  error           TEXT,
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
)
```

### Picking the Relay

One function decides which Relay a call goes to:

```rust
relay_for(project, env) -> RelayTarget
```

| | `dev` | `production` |
|---|---|---|
| Project has a `project_relay_stacks` row in state `ready` | shared development Relay | the project's own stack |
| Otherwise | shared development Relay | shared production Relay (apps already published) |

Every current caller of the shared `RelayConfig` goes through `relay_for` instead:

- Koda's tools (`relay_ops.rs`);
- `apply_before_publish` and `functions_on_rollback`;
- the Relay console and data sheet (`routes/project_relay.rs`);
- Iris's queries;
- preview access.

Nothing above `relay_ops` changes.

### First publish of a Relay project

The app's `web` service needs `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` when it starts,
and those come from the stack. So the first publish runs in two halves:

1. **Secrets.** Nomi generates `RELAY_CONTROL_KEY`, `RELAY_FUNCTIONS_SECRET` and
   `RELAY_PGRST_PASSWORD` and stores them on the project's Shipyard app, via the existing
   `project_secrets::store`. These three are marked Nomi-managed, and the project's Secrets
   sheet shows them as such: listed, not editable, not deletable. A row goes into
   `project_relay_stacks` with state `creating`.
2. **Stack first.** Publish to `production` a source zip holding only the stack's folders
   (`.nomi/stack/…`, see below), with the stack manifest and no `web` service. Wait for it.
   Read the `relay` service's URL into `control_url`.
3. **Make the backend in it.** Through that stack's control API:
   - create the project's backend, so the stack runs exactly one project ("Relay changes:
     standalone");
   - set its auth from `nomi/backend.json`;
   - set storage and the shared sender from Nomi's Configuration;
   - apply `nomi/migrations/`;
   - set realtime;
   - deploy functions.

   These are the same calls `apply_before_publish` makes today, aimed at the new target. Then
   store `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` from its answer as project secrets,
   exactly as `ensure_backend` does today for the shared Relay.
4. **Then the app.** Publish the full source plus `.nomi/stack/…`, with the full manifest (stack
   services and `web`). The stack services are unchanged, so Shipyard updates them in place;
   `web` starts with its secrets bound. Nomi then sets the stack's allowed origins to the
   app's URL (and again whenever the app's domain changes), and the row becomes `ready`.

Production starts with an empty database and the schema from the migrations, as it does today:
development data is never copied to production.

A failure in any half:

- leaves the row in `failed` with the error;
- tells the publisher in chat and by notification, as publish failures do today (#90);
- creates no `web` service.

Publishing again picks up from the first unfinished step.

### Every later publish

The same as today with a different target. `apply_before_publish` runs against the project's
stack (auth, storage, migrations, realtime, functions; a failure stops the publish and the
running version keeps serving). Then one publish with the full manifest.

If Nomi's pinned Relay version is newer than the stack's `relay_version`, the stack's
Dockerfiles in the zip name the new version, so the stack upgrades in the same publish. The row
records the new version once the publish succeeds.

### The stack manifest

These services go next to the app's own `web` service. They're added by
`nomi_shipyard::manifest::with_relay_stack(manifest, RelayStack { version, has_functions })`.

| Service | Kind | Built from | Environment |
|---|---|---|---|
| `relay` | `web`, port 8080 | `.nomi/stack/relay/Dockerfile` (`FROM <relay image>:<version>`) | `RELAY_DATABASE_URL = {database: url}`, `RELAY_ENCRYPTION_KEY = {generate: hex32}`, `RELAY_CONTROL_KEY = {secret}`, `RELAY_STANDALONE = "true"`, `RELAY_POSTGREST_URL = http://postgrest:3000`, `RELAY_PGRST_PASSWORD = {secret}`, `RELAY_FUNCTIONS_URL = http://functions:8000` and `RELAY_FUNCTIONS_SECRET = {secret}` when it has functions |
| `postgrest` | `worker` | `.nomi/stack/postgrest/Dockerfile` (`FROM postgrest/postgrest:<pinned>`) | `PGRST_DB_URI = "postgresql://"`, `PGHOST = {database: host}`, `PGPORT = {database: port}`, `PGDATABASE = {database: name}`, `PGUSER = "authenticator"`, `PGPASSWORD = {secret: RELAY_PGRST_PASSWORD}` (libpq fills the URI from these, so no URL has to be assembled) |
| `functions` | `worker`, only when the app has `nomi/functions/` | `.nomi/stack/functions/Dockerfile` (`FROM <functions image>:<version>`) | `RELAY_FUNCTIONS_SECRET = {secret}` |

- Plus `database: { engine: "postgres" }`. The app's tables, the `nomi` schema and Relay's own
  state share it.
- The app's `web` service gets `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY` bound by name.
- It does *not* get `DATABASE_URL`. A Relay app talks to its data through Relay, never directly.

The development Relay (Part 1) uses the same three services. The differences:

- no `RELAY_STANDALONE`;
- `RELAY_SHARED_DEV_URL = {database: url}`;
- `functions` always present.

### Versions

The `relay` and `functions` images are built and published by `relay.yml` on every merge under
`relay/`, tagged with the same version as the npm packages:

- `relay-server` from `relay/Dockerfile`;
- `functions-runtime` from `relay/functions-runtime/Dockerfile`.

Nomi pins the version it was built against in one constant (`RELAY_STACK_VERSION`). Both the
development Relay and app stacks use that constant. The PostgREST tag is pinned the same way.

### Deleting, transferring and rolling back

- Deleting the project deletes its Shipyard app, as today, and the stack and its data go with
  it. The `project_relay_stacks` row goes with the project.
- Transferring the app to another Shipyard org moves the stack with it. Nomi keeps the control
  key, so Koda, Iris and the console still reach it.
- Rolling back redeploys `web` and the functions from the earlier publish. Migrations never go
  back, as today. The stack services stay on their current version.

### Apps already published

Projects already published on the shared production Relay keep using it. `relay_for` sends
them there because they have no `project_relay_stacks` row. The admin page keeps showing the
shared production fields and checks while any such project exists, with a count. Moving them
is a separate piece of work.

---

## Relay changes (`relay/`)

1. **Settings from the environment.** At start, Relay reads `RELAY_SHARED_DEV_URL`,
   `RELAY_SHARED_PRODUCTION_URL`, `RELAY_POSTGREST_URL`, `RELAY_PGRST_PASSWORD`,
   `RELAY_FUNCTIONS_URL` and `RELAY_FUNCTIONS_SECRET`. The Dockerfile's comment already lists
   some of these, but `config.rs` doesn't read them yet.
   - A value set in the environment wins over the saved setting, and the settings view marks
     it as coming from the environment. This mirrors Nomi's Configuration page.
   - Relay creates the `authenticator` login role with `RELAY_PGRST_PASSWORD` when it's
     missing. That's the PostgREST role it already configures with `ALTER ROLE … SET pgrst.*`.
2. **Standalone mode** (`RELAY_STANDALONE=true`).
   - The instance holds exactly one project and only a `production` environment, in its own
     database (the `dedicated` tier's `public` schema).
   - Every request on its host is that project, so there's no subdomain per project.
   - `PUT /control/projects/:id` is accepted once; a second, different project is refused.
   - The project's public URL is the instance's own URL.
3. **Projects by path on a shared host.** Shipyard gives the development Relay one hostname,
   not a wildcard. So the gateway also accepts `https://<host>/p/<slug>-dev/…`, and the public
   URL it reports for a project is that form when no wildcard public domain is set. Subdomains
   keep working where a wildcard domain is configured. The SDK already joins paths onto
   `NOMI_URL`; this needs a test proving it does so with a path prefix.
4. **Images published** by `relay.yml` (see Versions).

## Nomi changes (`backend/`, `frontend/`)

- `nomi-shipyard`:
  - `manifest::with_relay_stack` and `manifest::stack_files(version, has_functions)`, which
    write the `.nomi/stack/*/Dockerfile` files into the zip;
  - the `put_secret` call if it isn't there yet.
- `nomi-server`:
  - `relay_dev_setup.rs`: the button's job, its steps and progress;
  - `relay_stacks.rs`: the table, `relay_for`, and the two-half first publish wired into
    `routes/deploy.rs::start_publish`;
  - `relay_ops` and `project_relay` go through `relay_for`;
  - Nomi-managed secrets are refused by the secrets routes.
- Migration `00xx_project_relay_stacks.sql`.
- Frontend:
  - the Development Relay card and the collapsed manual fields on the Integrations page;
  - the Secrets sheet shows Nomi-managed secrets as locked;
  - the Deployments sheet shows the stack's services next to `web` (it already lists services).
- Koda's guide (`guides/relay.md`), the Publishing section: the first publish also creates the
  app's own backend and takes longer.
- Iris: no change (she reads through `relay_for`).

## Order of work

Each step is its own PR and leaves everything working.

1. **Relay:** settings from the environment, the `authenticator` role, path-based projects,
   standalone mode, and image publishing.
2. **Nomi:** the development Relay button (Part 1).
3. **Nomi:** `relay_for` and the table, with no behaviour change (no rows yet).
4. **Nomi:** the stack on first publish, and upgrades on later publishes (Part 2).

## Testing

- **Relay:**
  - standalone mode refuses a second project;
  - settings read from the environment win and show as such;
  - path-prefixed requests reach the right project and can't reach another;
  - the SDK works with a path-prefixed `NOMI_URL`.
- **Nomi**, with a fake Shipyard (wiremock, as the existing deploy tests do):
  - the button's steps are idempotent: run twice, nothing is created twice;
  - a failed step resumes from where it stopped;
  - a hand-saved connection isn't overwritten without confirmation;
  - first publish runs the stack half before the app half, stores the three `NOMI_*` secrets,
    and a failure leaves no `web` service;
  - `relay_for` picks the right target in each of the four cases;
  - Nomi-managed secrets can't be edited or deleted;
  - a later publish with a newer `RELAY_STACK_VERSION` writes the new version into the stack
    Dockerfiles.
- **By hand, on a real Shipyard:**
  - press the button on an empty Nomi and build a data app with Koda;
  - publish it, check the app's data in the Relay console, publish again;
  - check that the second app's production can't see the first's.

## Questions only the user can answer (they own Shipyard)

These decide whether the manifest above works as written. Each has a fallback, so none of them
blocks Part 1.

1. **Can services in one environment reach each other by service name** (`http://postgrest:3000`)?
   Shipyard's docs mention a private network but not the names.
   - Fallback: none that's clean. Shipyard would need to document this, or add an env value
     form like `{ "service": "postgrest", "value": "internal_url" }`.
2. **Can Shipyard's database user create roles (`CREATEROLE`)?** Relay creates roles for every
   project: owner, anon, authenticated, and the `authenticator` login.
   - Fallback: a manifest option for it (`database: { engine: "postgres", create_role: true }`).
3. **Can a manifest service run a prebuilt image** (an `image` field)? This design uses
   one-line Dockerfiles (`FROM …`) in the zip, which works with what's documented today. An
   `image` field would skip the build step on every first publish.
4. **Does every `web` service in an environment get its own public URL**, and is the
   `domain.service` choice what makes `web` (not `relay`) the app's main address?
