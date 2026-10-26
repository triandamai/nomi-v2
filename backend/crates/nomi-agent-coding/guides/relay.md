# Building on Nomi Relay

This project's backend is Nomi Relay: a real Postgres database for `dev` (the preview) and one for
production, sign-in, file storage and live updates are already there. You write three things:

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

Server code (SvelteKit `+page.server.ts`, endpoints) uses the same client with the service key,
read from the environment, never sent to the browser:

```ts
import { env } from '$env/dynamic/private';
import { createClient } from 'nomi-relay';
const admin = createClient(env.NOMI_URL, env.NOMI_SERVICE_KEY);
```

The service key bypasses row-level security: use it only for what a user must not be able to
do themselves, and never put it in a `PUBLIC_` variable.
