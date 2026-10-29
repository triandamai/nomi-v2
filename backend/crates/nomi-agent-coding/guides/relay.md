# Building on Nomi Relay

This project's backend is Nomi Relay: a real Postgres database for `dev` (the preview) and one for
production, sign-in, file storage and live changes are already there. You write three things:

1. **migrations**: the schema and who may read and write each row;
2. **the app's server logic**, only where SQL can't express it;
3. **the UI**, calling the backend through the `nomi-relay` client.

Never write a database connection, sign-in, sessions, password handling or uploads yourself, and
never add Drizzle or another ORM to a Relay project.

## Layout

```
nomi/
  migrations/
    0001_init.sql      # plain SQL, numbered, applied in order, once
    0002_orders.sql
src/lib/nomi.ts        # the client (you write it, once)
src/lib/nomi.types.ts  # written by Nomi after every migration: never edit it
```

## Migrations

Write a file in `nomi/migrations/` and it's applied to the dev database right away: the result
comes back with the write. A failure says the file and line; fix that file if it never applied,
otherwise write the next migration. `src/lib/nomi.types.ts` is rewritten after each one.

- Name files `0001_name.sql`, `0002_name.sql`, … (lowercase, digits, underscores).
- **Never edit a migration that was applied.** Relay refuses it. Write the next one instead
  (`alter table …`).
- Every table gets row-level security and policies **in the same migration that creates it**.
  A table that's really public (a list of countries) is marked `-- nomi: public` on the line
  above `create table`.
- Dropping a table or a column loses data: mark it `-- nomi: destructive` on the line above,
  and only when the plan says so.
- Only define things in the project's own schema: no `nomi.` tables, no schemas, roles,
  `set role` or `search_path`.
- Use `bigint generated always as identity` or `uuid default gen_random_uuid()` for ids,
  `timestamptz` for times.

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
create policy "own tasks" on task
  using (owner = nomi.uid())
  with check (owner = nomi.uid());
```

In SQL, `nomi.uid()` is the signed-in user's id (null when nobody is), `nomi.role()` is `anon`,
`authenticated` or `service`, and `nomi.users(id)` is what to reference for "who owns this".
Keep app-specific profile fields in your own table keyed by `nomi.users(id)`.

Logic that must hold whoever calls goes in SQL too: a policy, a check constraint, a trigger, or a
function (`create function …`), which the app calls with `nomi.rpc('name', args)`.

Check what the database really has with `describe_backend` instead of remembering it, and apply
again with `apply_migrations` if you need to.

## The client

```ts
// src/lib/nomi.ts
import { createClient } from 'nomi-relay';
import type { Database } from '$lib/nomi.types';
import { PUBLIC_NOMI_URL, PUBLIC_NOMI_KEY } from '$env/static/public';

export const nomi = createClient<Database>(PUBLIC_NOMI_URL, PUBLIC_NOMI_KEY);
```

Add `nomi-relay` to `package.json` dependencies (`"nomi-relay": "^0.1.0"`).

```ts
await nomi.auth.signInWithCode(email);              // emails a six-digit code
await nomi.auth.verifyCode(email, code);
await nomi.auth.signInWithOAuth('github');          // google, apple too

const { data, error } = await nomi.from('task').select('*').eq('done', false).order('created_at', { ascending: false });
await nomi.from('task').insert({ title });
await nomi.from('task').update({ done: true }).eq('id', id);
await nomi.from('task').delete().eq('id', id);

await nomi.storage.from('avatars').upload(`${userId}.png`, file);
nomi.realtime.on('task', (change) => (change.event === 'resync' ? reload() : apply(change)));
```

Every call returns `{ data, error }` and never throws for a failed request: check `error`.

## Sign-in

Email codes work with no setup. Other methods are switched on in `nomi/backend.json`; writing it
saves them to dev right away (publishing saves them to production), and the result says which
secrets are still missing.

```json
{
  "auth": {
    "methods": ["email_code", "google", "github"],
    "google": { "client_id": "1234.apps.googleusercontent.com" },
    "github": { "client_id": "Iv1.abc123" },
    "redirect_urls": ["https://myapp.example.com"]
  }
}
```

- Methods: `email_code`, `google`, `github`, `apple` (`"apple": { "client_id", "team_id",
  "key_id" }`), and company logins as `oidc:<id>` with `"oidc": [{ "id", "issuer", "client_id" }]`.
- Client IDs go in the file; **secrets never do**. Ask the person to add them in the project's
  Secrets: `NOMI_OAUTH_GOOGLE_SECRET`, `NOMI_OAUTH_GITHUB_SECRET`, `NOMI_OAUTH_APPLE_KEY` (the
  `.p8` file's text), `NOMI_OAUTH_OIDC_<ID>_SECRET`. You never see their values.
- The provider's callback URL is Relay's, one per provider:
  `<PUBLIC_NOMI_URL>/auth/callback/<provider>`. Tell the person to register that exact URL with
  the provider (the dev one and the production one).
- `redirect_urls` are the app's own addresses OAuth may send people back to; the preview and
  `localhost` are always allowed on dev.
- A user who signs in with a provider that verified their email joins the account that already
  has that email. `nomi.auth.linkIdentity('github')` adds a method to the signed-in user.
- To send codes from the app's own domain, add `"smtp": { "host", "port", "user", "from" }` and
  ask for `NOMI_SMTP_PASSWORD`.

## Files

Buckets are declared in `nomi/backend.json`; writing it saves them to dev, publishing to
production. Never store uploads in a table or on the app's own server.

```json
{
  "storage": {
    "buckets": [
      { "name": "avatars", "public": true, "write": "owner", "max_size_mb": 5, "types": ["image/*"] },
      { "name": "invoices", "read": "owner", "write": "service" }
    ]
  }
}
```

- `read` and `write` say who may: `anyone` (any caller with the app's key), `signed_in` (the
  default), `owner` (a signed-in user, only under a folder named by their id) or `service` (only
  server code with the service key, which passes every rule).
- With `owner`, put each user's files under their id: `` `${user.id}/avatar.png` ``.
- `public: true` also serves files to anyone at `getPublicUrl(path)`, with no request: use it for
  images shown on public pages. For a private file in an `<img>` or a download link, use
  `createSignedUrl(path, seconds)`.
- `max_size_mb` defaults to 50 (100 at most); `types` limits the content types an upload may have.
- An upload never replaces a file unless it says `{ upsert: true }`.

```ts
const { error } = await nomi.storage.from('avatars').upload(`${user.id}/avatar.png`, file, { upsert: true });
const { publicUrl } = nomi.storage.from('avatars').getPublicUrl(`${user.id}/avatar.png`);
const { data } = await nomi.storage.from('invoices').createSignedUrl(`${user.id}/may.pdf`, 600);
const { data: files } = await nomi.storage.from('invoices').list(user.id);
```

## Live changes

List the tables whose changes the app shows live (chat messages, a shared board), and the app
gets each insert, update and delete as it happens, only for rows its user may read.

```json
{ "realtime": ["message"] }
```

A listed table needs a primary key, and its migration first (Relay says so if it's missing, and
turns it on as soon as the migration applies).

```ts
const stop = nomi.realtime.on<Message>('message', (change) => {
  if (change.event === 'resync') return reload();          // reconnected or missed changes: refetch
  if (change.event === 'delete') messages = messages.filter((m) => m.id !== change.key.id);
  else messages = [...messages.filter((m) => m.id !== change.row!.id), change.row!];
});
// stop() when the page goes away (onDestroy / the effect's cleanup)
```

Load the list with a normal query first, then apply changes; on `resync`, load it again.

## Functions

Trusted server code SQL can't express (charging a card, sending email, a webhook from another
service) goes in a hosted function: one file per function in `nomi/functions/`, helpers in
`nomi/functions/_shared/`. It's the only server code a static or single-page app has. Writing a
file there deploys every function to dev, checked first: a type error comes back with its file
and line, and the running functions stay as they were. Publishing deploys them to production.

```ts
// nomi/functions/place-order.ts: called as nomi.functions.invoke('place-order', { body: { cart } })
import { defineFunction } from 'nomi-relay/functions';

export default defineFunction(async (req, ctx) => {
  if (!ctx.user) return new Response('sign in first', { status: 401 });
  const { cart } = await req.json();
  const charge = await fetch('https://api.stripe.com/v1/charges', {
    method: 'POST',
    headers: { authorization: `Bearer ${ctx.env.STRIPE_SECRET}` },
    body: new URLSearchParams({ amount: String(cart.total), currency: 'usd' })
  });
  if (!charge.ok) return new Response('payment failed', { status: 402 });
  const { data, error } = await ctx.nomi.from('order').insert({ cart }).select().single();
  return error ? Response.json(error, { status: 400 }) : Response.json(data);
});
```

- `ctx.user` is the caller, already checked; `ctx.nomi` acts as them (row-level security
  applies); `ctx.admin` uses the service key, only for a step that must bypass it; `ctx.env` has
  the project's secrets by name (ask the person to add them in the project's Secrets).
- It runs on Deno: `fetch`, `Request`, `Response`, Web Crypto. Imports are `nomi-relay`,
  `npm:` packages (`import Stripe from 'npm:stripe@17'`) and files in `_shared/`. No file system,
  no subprocesses, no `Deno.env` (use `ctx.env`).
- Keep the top of each file light: it runs whenever the functions wake.
- Settings go in `backend.json`; every function is for signed-in callers by default:

```json
{
  "functions": {
    "stripe-webhook": { "auth": "public", "timeout": 10 },
    "nightly-report": { "schedule": "0 3 * * *" }
  }
}
```

  `auth: "public"` lets anyone call it, even without the app's key: only for webhooks, which must
  check the provider's signature themselves. `timeout` is in seconds (30 by default, 60 at most).
  `schedule` is a cron expression in UTC; the function is then called with the service role
  (`ctx.user` is null, `ctx.claims.role` is `service`).
- Call them from the app with `nomi.functions.invoke(name, { body })`, which returns
  `{ data, error }` like every other call. A function's address is
  `<PUBLIC_NOMI_URL>/functions/<name>` (give that to a webhook provider).

Server code (SvelteKit `+page.server.ts`, endpoints) uses the same client with the service key,
read from the environment, never sent to the browser:

```ts
import { env } from '$env/dynamic/private';
import { createClient } from 'nomi-relay';
const admin = createClient(env.NOMI_URL, env.NOMI_SERVICE_KEY);
```

The service key bypasses row-level security: use it only for what a user must not be able to
do themselves, and never put it in a `PUBLIC_` variable.

## Your backend: its own database and Relay

Every project has its own development database and Relay. Nomi makes them in the background when
the project is made, so the first backend tool can answer "still being set up" for a minute:
wait and try again rather than working around it. If setup fails, the person sees why in the
project's Relay sheet and can try again; tell them, and don't retry in a loop. After an hour
nobody uses it, the development backend sleeps; your next backend tool wakes it and waits, so a
call can take a few seconds longer than usual.

The first publish makes the app's own production database and Relay the same way, then deploys
the app, so it takes a few minutes longer than later ones. `publish_project` then answers that
publishing starts by itself: don't publish again and don't give a link; Nomi posts in the chat
when the app is live or why it couldn't be. Development data never goes to production: production
starts empty with the schema from the migrations.

## Publishing

Publishing goes in this order: sign-in settings, buckets, then production's new migrations, live
tables and functions, and only then the new code. If Relay refuses a migration, the publish stops
before anything changes: the version that was live keeps running, and Nomi says the file and line
in the chat and notifies the project's admins. Fix it with a new migration (never edit one that
production already has), then publish again. When the publish goes live, Nomi posts the address
in the chat and notifies the admins too.

## Seeing the data

The project's admins see the backend in the project's **Relay** sheet: each environment's tables
with their columns and policies, buckets, live tables, functions and sign-in, and a SQL console
that runs as the project's owner. Iris reads the same tables (reads only) when someone asks
what's in them. Neither needs anything from you: don't build an admin page just to look at data.
