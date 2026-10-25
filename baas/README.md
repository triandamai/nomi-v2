# Nomi BaaS: a ready backend for every app Nomi builds

**Status:** design only. No code lives here yet. When it's built, this folder holds the BaaS
service, its SDK and its docs. It stays separate from `backend/` (Nomi itself) and `frontend/`.

**Goal:** every project Nomi builds starts with a working backend: a database for development
and one for production, sign-in, file storage and live updates. Koda stops writing that plumbing
for each app. He writes only:

1. **migrations**: the schema, plus the rules for who can read and write each row;
2. **backend logic**: the app's own server code (what happens when an order is placed);
3. **the UI**, which calls the backend through a typed client.

Everything else is the same for every app and comes ready, so it's always right.

## Why

Today each app carries its own backend, written fresh by Koda every time:

- The database setup lives in the app's code (`guides/database.md`): Drizzle, plus a
  two-database switch, because the preview runs in the browser with no database server. The
  preview runs PGlite or libsql-wasm; production runs real Postgres or SQLite. So the preview and
  production can behave differently.
- Shipyard creates the production database at first publish. Nomi finds it by reading the
  app's files (`manifest::detect_database`). The development database Shipyard keeps is used only
  by the admin's SQL tool.
- Sign-in, sessions, password reset, uploads and live updates get written by hand in each app,
  or left out. This is where generated apps break most often, and the breakage is hardest to see
  in a preview.
- A plain page or a single-page app (`static`, `react`, `vue`) has no server, so it can't keep
  shared data at all.

## What every project gets

| Piece | What it is | Who uses it |
|---|---|---|
| Database | Postgres, one per environment: `dev` (preview) and `production` | Migrations, the REST API, the app's server |
| Data API | REST over the tables at `https://<app>.api.<apps-domain>/rest/<table>`, guarded by row-level security | The browser (with the public key), the server (with the service key) |
| Auth | Users, sessions, email codes, Google sign-in. Each project has its own signing key | The browser and the server |
| Storage | Buckets of files, with signed URLs and per-bucket rules | Uploads and avatars |
| Realtime | Change feeds on chosen tables, over WebSocket | Live lists and chat-like apps |
| Functions (later) | Hosted server code for apps without a server | Static and single-page apps |

## Architecture

```
            Nomi backend (backend/)                         App (preview or published)
    ┌───────────────────────────────┐                ┌──────────────────────────────┐
    │ Koda's tools, project sheets  │                │ UI + its own server code     │
    │ publish, secrets              │                │ @nomi/baas client            │
    └──────────────┬────────────────┘                └──────────────┬───────────────┘
                   │ control API (service-to-service)               │ public API (keys)
            ┌──────▼────────────────────────────────────────────────▼──────┐
            │                        BaaS (baas/)                          │
            │  control plane │ gateway: rest · auth · storage · realtime   │
            └──────┬─────────────────────────┬───────────────────────┬─────┘
                   │                         │                       │
         Postgres per project/env      S3 prefix per project    LISTEN/NOTIFY
         (Shipyard databases)
```

- **One service, many tenants.** It's a Rust/axum service, the same stack as `backend/`, with
  its own crates under `baas/crates/`. Every project shares it, and the hostname picks the
  project. Shipyard runs it as an ordinary app.
- **A database per project and environment.** This uses the databases Shipyard already makes:
  the app's `dev` database and its production environment database. The BaaS keeps its own
  tables in a `nomi` schema inside each one: `nomi.users`, `nomi.sessions`, `nomi.migrations`
  and `nomi.buckets`. A separate database per project keeps one app's data and load away from the
  others. It also means deleting a project deletes exactly its data.
- **The control plane** is called only by the Nomi backend, with a service credential:
  - create or delete a project's backend;
  - apply migrations to an environment;
  - read the schema;
  - rotate keys;
  - set auth and storage settings.
- **The gateway** is what apps call. It checks the key or the user's token, sets the Postgres
  role and claims for the request (`SET LOCAL role`, `request.jwt.claims`), and lets row-level
  security decide what the request can see. The REST layer translates query parameters into
  parameterized SQL.
  - Decision to make: embed PostgREST, or write our own (see open questions).

## What Koda writes

A project's backend is a folder in its files:

```
nomi/
  backend.json          # which auth methods, buckets and realtime tables it uses
  migrations/
    0001_init.sql       # plain SQL, numbered, forward only
    0002_orders.sql
  functions/            # later: hosted code for apps without a server
```

The schema, row-level security, indexes, triggers and SQL functions all go in the migrations:

```sql
-- nomi/migrations/0001_init.sql
create table task (
  id bigint generated always as identity primary key,
  owner uuid not null default nomi.uid() references nomi.users(id) on delete cascade,
  title text not null,
  done boolean not null default false,
  created_at timestamptz not null default now()
);
alter table task enable row level security;
create policy "own tasks" on task using (owner = nomi.uid()) with check (owner = nomi.uid());
```

The app uses the client from the SDK, `baas/sdk/js`, published as `@nomi/baas`:

```ts
import { createClient } from '@nomi/baas';
import type { Database } from '$lib/nomi.types'; // written by Nomi after each migration

export const nomi = createClient<Database>(import.meta.env.PUBLIC_NOMI_URL, import.meta.env.PUBLIC_NOMI_KEY);

await nomi.auth.signInWithCode(email);
const { data } = await nomi.from('task').select('*').eq('done', false).order('created_at');
await nomi.storage.from('avatars').upload(`${user.id}.png`, file);
nomi.realtime.on('task', (change) => refresh());
```

Server code, such as SvelteKit endpoints, uses the same client with the service key, read from
the environment. Logic that has to hold no matter who calls it goes in SQL: a policy, a trigger,
or a function the API exposes at `/rest/rpc/<name>`.

## How it fits Nomi

**Creating a project.** `create_project` also calls the control plane. It makes the dev
database if it's missing (`ensure_dev_database`), sets up the `nomi` schema there, and creates
the keys. The keys are stored the way project secrets are now (`project_secrets`): only Shipyard
holds the values.
- `NOMI_URL` and `NOMI_ANON_KEY` are public.
- `NOMI_SERVICE_KEY` is for the server only.

**Koda's tools and brief.**
- A new guide, `guides/baas.md`, replaces `database.md` for new projects.
- A new `apply_migrations` tool runs the migrations against `dev` and reports SQL errors back
  to Koda, the same way `npm run check` output does now. Writing a file under
  `nomi/migrations/` triggers it too.
- A new `describe_backend` tool lists the tables, columns, policies and buckets, so Koda checks
  the real schema instead of remembering it.
- The brief lists the backend's tables and its secrets by name. It never includes their values.
- After each migration, Nomi writes `src/lib/nomi.types.ts` from the schema, so `npm run check`
  catches a query against a column that doesn't exist.

**Migrations are safe by construction.**
- They are applied in order, each in its own transaction, and recorded in `nomi.migrations` with
  a checksum.
- Editing a migration that was already applied is refused; Koda writes a new one instead.
- A linter rejects:
  - a public table without row-level security, unless it's marked `-- nomi: public`;
  - `drop table` or `drop column` without `-- nomi: destructive`;
  - statements that reach outside the project's own schemas.

**Preview.** The app in the WebContainer calls the BaaS over HTTPS, so the preview runs on a
real Postgres. That's the same engine as production, and the PGlite switch goes away.
- The dev key only works against `dev`.
- CORS allows Nomi's preview origins.

**Publish.**
1. Before the new version goes live, `publish` applies any migrations production doesn't have
   yet.
2. If a migration fails, the publish stops with the error, and the running version keeps
   serving.
3. The manifest binds `NOMI_*` by name, the way it binds project secrets now.
4. Rollback brings back the old code, but never reverses migrations. Migrations only go forward,
   so a migration that breaks the old code needs a new migration to fix it. Koda's guide says so.

**The project sheets.** The Database sheet (`project_database.rs`) grows into the backend's
console:
- tables and the SQL tool, as now;
- users, with sign-in methods and the option to remove someone;
- buckets and their files;
- migrations, with when each was applied to each environment;
- request logs.

**Apps without a server.** Static, React and Vue apps get data, sign-in and storage through the
REST API and row-level security alone. Hosted functions come later, for logic that can't be
expressed in SQL.

## Security

- Each project has its own JWT signing key. A token from one app means nothing to another.
- The anon key can do only what row-level security allows. The service key bypasses it and is
  never sent to the browser, so the linter flags `PUBLIC_` variables that hold it.
- Every request runs as a role scoped to its own database, with a statement timeout and limits
  on rows returned and request size.
- Per-project rate limits and plan quotas cover rows, storage, requests and realtime
  connections, alongside Nomi's existing plans.
- Control-plane calls and service-key use are audited.
- Users of an app are that app's users, in its own database. They aren't Nomi users and can't
  reach Nomi.

## Existing projects

Projects built on Drizzle keep working as they are: `detect_database` and the current manifest
stay. The BaaS is the default for new projects. An existing project can switch over when Koda
moves its schema into `nomi/migrations/` and its queries onto the client. The data stays where
it is, because it's the same Shipyard database.

## Phases

1. **Control plane and migrations.** The service skeleton in `baas/`, a backend per project,
   the `nomi` schema, `apply_migrations` against `dev` and at publish, the linter, generated
   types, and Koda's `baas.md` guide.
2. **Data API, auth and SDK.** The REST gateway with row-level security, email-code and Google
   sign-in, `@nomi/baas`, and the preview on the real dev database.
3. **Storage and realtime.**
4. **The console** in the project sheets.
5. **Hosted functions** for apps without a server.

Each phase ships on its own and is useful by itself. Phase 1 alone fixes the preview-versus-
production drift and gives Koda a checked migration loop.

## Open questions

- **REST layer:** embed PostgREST (proven, but a Haskell binary to run and keep up to date), or
  write our own in Rust (one stack and our own limits, but more work)? Leaning toward our own,
  covering only the query features the SDK uses.
- **Realtime transport:** Postgres `LISTEN/NOTIFY` fanned out by the gateway over WebSocket, or
  reuse Nomi's MQTT broker (already run, already scales to many subscribers)?
- **Cost of a database per project** on Shipyard for many small free projects. Should free
  projects share a cluster, one schema each, and move to their own database on upgrade?
- **Hosted functions runtime:** Deno, workerd, or running them as a second service in the app's
  own Shipyard environment.
