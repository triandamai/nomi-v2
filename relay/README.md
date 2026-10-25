# Nomi Relay: a ready backend for every app Nomi builds

Relay is Nomi's backend-as-a-service (BaaS).

**Status:** the client (`sdk/js`, published as `nomi-relay`) and the CLI (`cli`, published as
`nomi-relay-cli`) are started; the service itself isn't built yet. This folder holds the Relay
service, its SDK, its CLI and its docs. It stays separate from `backend/` (Nomi itself) and
`frontend/`.

**Goal:** every project Nomi builds starts with a working backend: a database for development
and one for production, sign-in, file storage, live updates and hosted functions. Koda stops
writing that plumbing for each app. He writes only:

1. **migrations**: the schema, plus the rules for who can read and write each row;
2. **backend logic**: the app's own server code (what happens when an order is placed), either
   in the app's server or as hosted functions;
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
  shared data or run trusted code at all.

## What every project gets

| Piece | What it is | Who uses it |
|---|---|---|
| Database | Postgres for `dev` (preview) and `production`: its own database per environment on paid plans, its own schema in a shared database on free | Migrations, the REST API, the app's server |
| Data API | REST over the tables at `https://<app>.api.<apps-domain>/rest/<table>`, guarded by row-level security | The browser (with the public key), the server (with the service key) |
| Auth | Users, sessions, email codes, and OAuth sign-in: Google, GitHub, Apple, or any OpenID Connect provider. Each project has its own signing key | The browser and the server |
| Storage | Buckets of files, with signed URLs and per-bucket rules | Uploads and avatars |
| Realtime | Change feeds on chosen tables, over WebSocket | Live lists and chat-like apps |
| Functions | TypeScript handlers in `nomi/functions/`, built and run by Shipyard as a Deno service per environment | Any app, and the only server code a static or single-page app has |

## Architecture

```
            Nomi backend (backend/)                         App (preview or published)
    ┌───────────────────────────────┐                ┌──────────────────────────────┐
    │ Koda's tools, project sheets  │                │ UI + its own server code     │
    │ publish, secrets              │                │ nomi-relay client            │
    └──────────────┬────────────────┘                └──────────────┬───────────────┘
                   │ control API (service-to-service)               │ public API (keys)
            ┌──────▼────────────────────────────────────────────────▼──────┐
            │                        Relay (relay/)                        │
            │  control plane │ gateway: rest · auth · storage · realtime   │
            │                │          · functions (proxy)                │
            └──────┬───────────────────┬──────────────────┬──────────┬─────┘
                   │                   │                  │          │
         PostgREST ──► Postgres    S3 prefix per    LISTEN/NOTIFY  Functions service
         (own DB: paid,            project                         per project/env
          shared DB: free)                                         (Shipyard, Deno)
```

- **One service, many tenants.** It's a Rust/axum service, the same stack as `backend/`, with
  its own crates under `relay/crates/`. Every project shares it, and the hostname picks the
  project. Shipyard runs it as an ordinary app.
- **Paid projects get a database per environment.** This uses the databases Shipyard already
  makes: the app's `dev` database and its production environment database. A separate database
  keeps one app's data and load away from the others, and deleting the project deletes exactly
  its data.
- **Free projects share a database.** There is one shared `dev` database and one shared
  production database (more as they fill up; see open questions). Each free project gets its
  own schema in them, `app_<project>`, and its own Postgres roles, whose `search_path` is that
  schema. Migrations are the same SQL either way: `create table task` lands in `public` on a
  paid project and in `app_<project>` on a free one. Grants keep each project's roles inside
  their own schema.
- **The `nomi` schema** holds Relay's own tables: `nomi.users`, `nomi.identities`,
  `nomi.sessions`, `nomi.migrations` and `nomi.buckets`. Every row has a `project_id`. In a
  paid project's database it's always the same value; in a shared database, row-level security
  on these tables (`project_id = nomi.project_id()`) keeps each project to its own users. User
  IDs are UUIDs, so `references nomi.users(id)` works the same in both.
- **Upgrading** a free project to paid copies its schema and its `nomi` rows into its own new
  database (`public` schema), during a short write pause, then points the gateway at it.
  Deleting a free project drops its schema and deletes its `nomi` rows.
- **The control plane** is called only by the Nomi backend, with a service credential:
  - create or delete a project's backend;
  - apply migrations to an environment;
  - read the schema;
  - rotate keys;
  - set auth, OAuth provider and storage settings;
  - deploy functions to an environment.
- **The gateway** is what apps call. It checks the key or the user's token against the
  project's signing key, then passes the request on. Calls to `/functions/<name>` go to the
  project's functions service; calls to `/rest/...` go to PostgREST.
- **PostgREST is the REST layer.** We run it rather than write our own: its query syntax,
  row-level security handling and schema cache are proven, and the SDK's query builder speaks
  its syntax.
  - A paid project gets a PostgREST per environment, run by Shipyard next to its database.
  - Each shared database gets one PostgREST for all its free projects, with every project's
    schema in `db-schemas`. The gateway sets `Accept-Profile` and `Content-Profile` to the
    project's schema, and drops any the client sent, so a request can't pick another schema.
  - PostgREST only trusts tokens from the gateway. After checking the app's token, the gateway
    signs a short-lived internal token with the PostgREST instance's key, carrying the project's
    role (`app_<project>_authenticated`, `_anon` or `_service`) and the user's claims. PostgREST
    sets the role and `request.jwt.claims`, and row-level security decides what the request can
    see.
  - After each migration, and when a free project is added to or removed from a shared database,
    the control plane sends `NOTIFY pgrst, 'reload schema'` (and `'reload config'` when the
    schema list changes).
  - Every PostgREST runs with a statement timeout, `db-max-rows`, and request-size limits set
    from the project's plan.

## What Koda writes

A project's backend is a folder in its files:

```
nomi/
  backend.json          # auth methods, OAuth providers, buckets, realtime tables, functions
  migrations/
    0001_init.sql       # plain SQL, numbered, forward only
    0002_orders.sql
  functions/
    place-order.ts      # one file per function, served at /functions/place-order
    _shared/            # helpers imported by functions; not served
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

The app uses the client from the SDK, `relay/sdk/js`, published on npm as `nomi-relay`:

```ts
import { createClient } from 'nomi-relay';
import type { Database } from '$lib/nomi.types'; // written by Nomi after each migration

export const nomi = createClient<Database>(import.meta.env.PUBLIC_NOMI_URL, import.meta.env.PUBLIC_NOMI_KEY);

await nomi.auth.signInWithCode(email);
await nomi.auth.signInWithOAuth('github', { redirectTo: location.origin });
const { data } = await nomi.from('task').select('*').eq('done', false).order('created_at');
await nomi.storage.from('avatars').upload(`${user.id}.png`, file);
nomi.realtime.on('task', (change) => refresh());
const { data: order } = await nomi.functions.invoke('place-order', { body: { cart } });
```

Server code, such as SvelteKit endpoints, uses the same client with the service key, read from
the environment. Logic that has to hold no matter who calls it goes in SQL: a policy, a trigger,
or a function the API exposes at `/rest/rpc/<name>`. Logic SQL can't express (calling a payment
provider, sending email, a webhook from another service) goes in a hosted function.

## Auth

Each project's users live in its own database, in `nomi.users`. One user can have several
sign-in methods; each is a row in `nomi.identities` (`provider`, `provider_user_id`, `email`).

**Sign-in methods**, switched on per project in `backend.json`:

```json
{
  "auth": {
    "methods": ["email_code", "google", "github", "apple"],
    "oidc": [{ "id": "acme", "issuer": "https://login.acme.com", "client_id": "nomi-app" }],
    "redirect_urls": ["https://myapp.<apps-domain>", "http://localhost:5173"],
    "session_ttl_days": 30,
    "smtp": { "host": "smtp.resend.com", "port": 587, "user": "resend", "from": "My App <hi@myapp.com>" }
  }
}
```

- **Email code.** A six-digit code, sent by Relay. It works with no setup, so it's the
  default for every new project.
  - **Sender.** By default, codes go out through Nomi's shared sender, from a Nomi address with
    the app's name. A project can set its own SMTP server in `auth.smtp` instead, to send from
    its own domain. The SMTP password is a project secret (`NOMI_SMTP_PASSWORD`), never in
    `backend.json`. Saving the settings sends a test email; if it fails, the project keeps
    using the shared sender and the sheet shows why.
  - The shared sender has stricter per-project rate limits than a custom SMTP server, to keep
    its reputation clean for everyone.
- **Google, GitHub, Apple.** Built-in providers, each with the right scopes and profile mapping.
  Apple's client secret is a short-lived JWT; Relay signs it from the project's `.p8` key, so
  nobody has to rotate it by hand.
- **Any OpenID Connect provider.** Configured by issuer URL; Relay reads its discovery
  document. This covers Microsoft, GitLab, Auth0, Keycloak and company logins without new code.

**Provider credentials are project secrets.** The client ID can sit in `backend.json`; the
client secret (or Apple's key) is stored the way `project_secrets` are now, by name, e.g.
`NOMI_OAUTH_GITHUB_SECRET`. Koda sees the names, never the values. A provider that's switched
on but missing its secret shows up in `describe_backend` and in the project sheet, so the user
is asked for it instead of finding out at sign-in.

**The OAuth flow.**
1. `signInWithOAuth('github')` sends the browser to
   `https://<app>.api.<apps-domain>/auth/authorize/github`, with PKCE and a `state` value.
2. Every provider's callback URL is Relay's own,
   `https://<app>.api.<apps-domain>/auth/callback/<provider>`. The user registers that one URL
   with GitHub, Google or Apple, once, and it never changes, even when the preview's origin does.
3. Relay swaps the code for the provider's tokens, finds or creates the user, and sends the
   browser back to the app's `redirectTo`, which must match `redirect_urls` (Nomi's preview
   origins are always allowed against `dev`). The app gets a one-time code and the SDK trades it
   for a session.

**Linking accounts.** A new sign-in whose provider says the email is verified joins the user who
already has that verified email. An unverified email never links; it makes a new user. A
signed-in user can add or remove methods with `nomi.auth.linkIdentity` and `unlinkIdentity`, but
can't remove the last one.

**Sessions.** A short-lived access token (a JWT signed with the project's key, 15 minutes) and a
refresh token stored in `nomi.sessions`. Each refresh rotates the refresh token; reusing an old
one ends the whole session family, because it means the token was stolen. Signing out, or
removing a user from the project sheet, revokes their sessions at once.

**In SQL and functions.** `nomi.uid()` is the signed-in user's ID, `nomi.role()` is `anon`,
`authenticated` or `service`, and `nomi.claims()` is the whole token. Apps add their own roles
and profile fields in their own tables, keyed by `nomi.users(id)`, not by editing `nomi.users`.

**Admin.** With the service key, server code and functions can list, create, update and delete
users (`nomi.auth.admin.*`), for invites and back-office tools.

## Realtime

Realtime runs on Postgres `LISTEN/NOTIFY`, fanned out by the gateway over WebSocket. Nothing
else to run or keep up.

- **Which tables.** `backend.json` lists them (`"realtime": ["task", "message"]`). For each,
  the control plane adds a trigger that calls `pg_notify('nomi_realtime', ...)` after insert,
  update or delete.
- **What's sent.** The notification carries the project, table, operation and primary key, not
  the row: `NOTIFY` payloads stop at 8000 bytes, and the row has to pass row-level security for
  each subscriber anyway.
- **Fan-out.** The gateway holds one `LISTEN` connection per database, so a shared database is
  one connection for all its free projects. For each change it reads the row once per
  subscriber's role and claims, batched, through the same role switch PostgREST uses, and sends
  it only to subscribers who are allowed to see it. Deletes send just the primary key.
- **Delivery.** At most once. A client that reconnects gets a `resync` event and refetches, so
  the SDK's live lists never drift.
- **Limits.** Connections per project and changes per second are plan quotas. A burst beyond
  them collapses into a single `resync` instead of dropping events silently.

## Functions

Hosted functions give every app trusted server code, including apps with no server of their own.

**Writing one.** Each file in `nomi/functions/` is one function. It exports a handler that takes
a standard `Request` and returns a `Response`:

```ts
// nomi/functions/place-order.ts
import { defineFunction } from 'nomi-relay/functions';

export default defineFunction(async (req, ctx) => {
  if (!ctx.user) return new Response('sign in first', { status: 401 });
  const { cart } = await req.json();
  const charge = await fetch('https://api.stripe.com/v1/charges', {
    method: 'POST',
    headers: { authorization: `Bearer ${ctx.env.STRIPE_SECRET}` },
    body: new URLSearchParams({ amount: String(cart.total), currency: 'usd' })
  });
  const { data } = await ctx.nomi.from('order').insert({ owner: ctx.user.id, cart }).select();
  return Response.json(data);
});
```

- `ctx.user` is the caller, already checked by the gateway, or `null` for an anonymous call.
- `ctx.nomi` is a client acting as the caller, so row-level security still applies.
  `ctx.admin` is a client with the service key, for the rare step that must bypass it.
- `ctx.env` holds the project's secrets, by name.
- Imports are npm packages (`npm:` specifiers) or files in `_shared/`.

**How it runs.** Shipyard runs functions the way it runs any app service, on the Deno runtime:

1. Nomi bundles `nomi/functions/` into one Deno entry point: a small router Relay provides,
   plus every handler. One service per project and environment, not one per function, so a
   project with ten functions costs the same as one with one.
2. Shipyard builds and runs that bundle as a service in the project's environment, with the
   project's secrets and the `NOMI_*` values bound by name.
3. The gateway proxies `https://<app>.api.<apps-domain>/functions/<name>` to it, after checking
   the key or token. Only the gateway can reach the service; it isn't exposed on its own.

**Sleeping and waking.** A functions service doesn't stay running while nobody calls it.
Shipyard stops the Deno runtime when it's idle, and the next call wakes it, which takes a
couple of seconds. Calls after that are fast until it goes idle again.
- The gateway holds a call while the service wakes instead of failing it, up to a wake timeout.
  Calls that arrive during the wake wait for the same start.
- The SDK's `functions.invoke` allows for that first slow call, so apps don't need their own
  retry.
- A cron `schedule` wakes the service like any other call.
- Koda's guide says to keep a function's top-level code light (no big setup on import), since
  it runs on every wake.

**Settings per function** in `backend.json`: whether a signed-in user is required
(`"auth": "required"`, the default, or `"public"` for webhooks), a timeout (default 30 seconds,
plan-capped) and an optional cron `schedule` (`"0 * * * *"`), which the gateway calls with the
service role.

**Preview.** Writing a file under `nomi/functions/` redeploys the `dev` functions service, the
way writing a migration applies it to `dev`. Build errors and type errors go back to Koda like
`npm run check` output. Logs from the service show in the project sheet and through
`describe_backend`.

**Publish.** `publish` deploys the functions to production after migrations and before the new
app version goes live. If the build fails, the publish stops and the running version keeps
serving. Rollback brings back the previous functions build along with the old code.

## The npm packages and the CLI

Two public packages, published from this repo under the owner's npm account:

| Package | What it is |
|---|---|
| `nomi-relay` | The client: `createClient`, auth, data, storage, realtime, `functions.invoke`, plus `nomi-relay/functions` for writing handlers. ESM, typed, no runtime dependencies, works in browsers, Node, Deno and Bun |
| `nomi-relay-cli` | The command line, installed as the `nomi-relay` command (`npx nomi-relay-cli …`) |

The names are unscoped because the `@nomi` org on npm belongs to someone else. (A free npm
account can publish public scoped packages under its own user scope too, e.g.
`@<username>/relay`, if we'd rather group them later.)

**What the CLI does**, so Relay is usable outside Nomi as well as inside it:

```
nomi-relay login                       # opens Nomi in the browser, stores a personal access token
nomi-relay link <project>              # ties this folder to a Nomi project
nomi-relay migrate new <name>          # writes nomi/migrations/000N_<name>.sql
nomi-relay migrate apply [--env dev]   # lints and applies pending migrations
nomi-relay migrate status              # what each environment has applied
nomi-relay types > src/lib/nomi.types.ts
nomi-relay functions serve             # runs nomi/functions locally on Deno, against dev
nomi-relay functions deploy [--env dev]
nomi-relay secrets set NAME            # reads the value from stdin; never echoed
```

The CLI never talks to the control plane directly. It calls the Nomi backend with the user's
personal access token, and the backend checks the user can act on that project before passing
the call on. The control plane stays service-to-service only.

**Versions and releases.**
- Both packages share one semver version. `.github/workflows/relay.yml` tests every pull
  request that touches `relay/`. Every merge to main that touches it publishes both packages with
  npm provenance: the next patch after what npm has, or the version in the package.json files
  when a PR raised it (that's how a minor or major goes out). The workflow then commits the
  version back to main and tags it `relay-v<version>`.
- CI publishes through npm trusted publishing: npm trusts this repo's `relay.yml` workflow
  directly, so there is no long-lived token to leak or rotate. npm only allows that on a package
  that already exists, so the very first version goes out by hand (see *Publishing* below).
  The `NOMI_NPM_TOKEN` secret is a fallback; npm refuses tokens that bypass 2FA for publishing.
- The client sends its version in a header; the gateway keeps old minor versions working, and a
  breaking change is a new major version of the package and a new `/v2` API path.
- Apps Nomi builds pin the package's minor version, and Nomi bumps it when it updates the
  template.

## Publishing

**The first time** (once per package name, from a maintainer's machine, with 2FA):

```sh
npm login
cd relay && npm ci && npm run build
npm publish -w sdk/js --access public
npm publish -w cli --access public
```

Then, on npmjs.com, open each package's settings (`nomi-relay` and `nomi-relay-cli`) and add a
**trusted publisher**: GitHub Actions, repository `triandamai/nomi-v2`, workflow `relay.yml`. After
that, delete the `NOMI_NPM_TOKEN` secret if it exists.

**Every time after that,** merge a change under `relay/` to main. The workflow publishes the next
patch (0.1.0 → 0.1.1) and commits the version back. For a minor or major, set the new version in
both `relay/sdk/js/package.json` and `relay/cli/package.json` in the PR.

**If a release fails** after publishing (the version commit couldn't be pushed), set both
package.json files to the published version by hand; the next run continues from what npm has.

## How it fits Nomi

**Creating a project.** `create_project` also calls the control plane. It makes the dev
database if it's missing (`ensure_dev_database`), sets up the `nomi` schema there, and creates
the keys. The keys are stored the way project secrets are now (`project_secrets`): only Shipyard
holds the values.
- `NOMI_URL` and `NOMI_ANON_KEY` are public.
- `NOMI_SERVICE_KEY` is for the server and functions only.

**Koda's tools and brief.**
- A new guide, `guides/relay.md`, replaces `database.md` for new projects. It covers migrations,
  auth setup, and when to reach for a function instead of SQL.
- A new `apply_migrations` tool runs the migrations against `dev` and reports SQL errors back
  to Koda, the same way `npm run check` output does now. Writing a file under
  `nomi/migrations/` triggers it too.
- A new `deploy_functions` tool builds and deploys `nomi/functions/` to `dev` and reports build
  errors. Writing a file under `nomi/functions/` triggers it too.
- A new `describe_backend` tool lists the tables, columns, policies, buckets, auth providers
  (and which are missing their secret) and functions, so Koda checks the real backend instead of
  remembering it.
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

**Preview.** The app in the WebContainer calls Relay over HTTPS, so the preview runs on a
real Postgres. That's the same engine as production, and the PGlite switch goes away.
- The dev key only works against `dev`.
- CORS, and OAuth redirects, allow Nomi's preview origins against `dev`.

**Publish.**
1. Before the new version goes live, `publish` applies any migrations production doesn't have
   yet, then deploys the functions.
2. If a migration or the functions build fails, the publish stops with the error, and the
   running version keeps serving.
3. The manifest binds `NOMI_*` by name, the way it binds project secrets now.
4. Rollback brings back the old code and the old functions, but never reverses migrations.
   Migrations only go forward, so a migration that breaks the old code needs a new migration to
   fix it. Koda's guide says so.

**The project sheets.** The Database sheet (`project_database.rs`) grows into the backend's
console:
- tables and the SQL tool, as now;
- users, with their sign-in methods and the option to remove someone;
- auth settings: providers, their secrets (set, never shown), redirect URLs;
- buckets and their files;
- functions, with their logs and last deploy per environment;
- migrations, with when each was applied to each environment;
- request logs.

**Apps without a server.** Static, React and Vue apps get data, sign-in and storage through the
REST API and row-level security, and trusted server code through hosted functions.

## Security

- Each project has its own JWT signing key. A token from one app means nothing to another.
- The anon key can do only what row-level security allows. The service key bypasses it and is
  never sent to the browser, so the linter flags `PUBLIC_` variables that hold it.
- OAuth uses PKCE and a `state` value on every sign-in, and redirects only to the project's
  allowed URLs. Provider secrets are project secrets, held only by Shipyard.
- Refresh tokens rotate on every use; reusing one revokes the session family.
- Every request runs as a role scoped to its own project (its database, or its schema in a
  shared database), with a statement timeout and limits on rows returned and request size.
- In a shared database, the gateway alone picks the schema PostgREST uses, and the `nomi`
  tables are guarded by `project_id` row-level security, so one free project can't read
  another's tables or users.
- Each project's functions run in their own Shipyard service, reachable only through the
  gateway, with a timeout and the plan's memory limit. One project's functions can't see another
  project's secrets or database.
- Per-project rate limits and plan quotas cover rows, storage, requests, function time and
  realtime connections, alongside Nomi's existing plans.
- Control-plane calls, CLI calls and service-key use are audited.
- Users of an app are that app's users, in its own database. They aren't Nomi users and can't
  reach Nomi.

## Existing projects

Projects built on Drizzle keep working as they are: `detect_database` and the current manifest
stay. Relay is the default for new projects. An existing project can switch over when Koda
moves its schema into `nomi/migrations/` and its queries onto the client. The data stays where
it is, because it's the same Shipyard database.

## Phases

1. **Control plane and migrations.** The service skeleton in `relay/`, a backend per project
   (own database for paid, a schema in the shared database for free), the `nomi` schema, `apply_migrations` against `dev` and at publish, the linter, generated
   types, and Koda's `relay.md` guide.
2. **Data API, auth and SDK.** The gateway in front of PostgREST, with row-level security,
   email-code sign-in (shared sender and custom SMTP),
   Google, GitHub, Apple and generic OIDC, sessions with rotating refresh tokens, and the preview
   on the real dev database. First release of `nomi-relay` and `nomi-relay-cli` (login, link,
   migrate, types) on npm.
3. **Storage and realtime** (`LISTEN/NOTIFY` triggers and the gateway's WebSocket fan-out).
4. **Hosted functions.** The Deno router and bundler, the per-project Shipyard service, the
   gateway proxy, `deploy_functions`, cron schedules, and `functions serve` / `deploy` in the CLI.
5. **The console** in the project sheets.

Each phase ships on its own and is useful by itself. Phase 1 alone fixes the preview-versus-
production drift and gives Koda a checked migration loop.

## Decisions

- **REST layer: PostgREST.** Proven query syntax, row-level security handling and schema cache,
  instead of writing our own. The cost is a Haskell binary to run and keep up to date.
- **Realtime: Postgres `LISTEN/NOTIFY`**, fanned out by the gateway over WebSocket. No extra
  broker to run.
- **Free projects share a database**, one schema each, and move to their own database when they
  upgrade.
- **Functions sleep when idle.** The first call after a sleep takes a couple of seconds while
  Deno starts.
- **Email codes go through a shared Nomi sender** by default, and each project can set its own
  SMTP server.

## Open questions

- **How many free projects per shared database?** Every project's schema is in the shared
  PostgREST's schema cache, so reloads get slower as it grows. We need a cap, and a rule for
  opening the next shared database when one is full.
- **Wake timeout for functions:** how long the gateway holds a call while the service starts
  before giving up.
