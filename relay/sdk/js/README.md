# nomi-relay

The client for Nomi Relay: the backend every app Nomi builds gets ready-made. One client covers
the database (through a REST API guarded by row-level security), sign-in, file storage, live
updates and hosted functions.

```sh
npm install nomi-relay
```

```ts
import { createClient } from 'nomi-relay';
import type { Database } from '$lib/nomi.types'; // written by Nomi after each migration

export const nomi = createClient<Database>(import.meta.env.PUBLIC_NOMI_URL, import.meta.env.PUBLIC_NOMI_KEY);
```

Every call resolves to `{ data, error }` and never throws for a failed request: check `error`
first.

## Data

```ts
const { data, error } = await nomi.from('task').select('*').eq('done', false).order('created_at', { ascending: false });
await nomi.from('task').insert({ title: 'Buy milk' });
const { data: task } = await nomi.from('task').update({ done: true }).eq('id', 3).select().single();
await nomi.from('task').delete().eq('id', 3);
const { data: open } = await nomi.rpc('count_open_tasks', { owner: userId });
```

Filters: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `like`, `ilike`, `is`, `in`, `contains`,
`textSearch`, `not`, `or`, `match`, `filter`. Modifiers: `order`, `limit`, `range`, `single`,
`maybeSingle`, and `select(columns, { count: 'exact' })` for a total in `count`.

## Auth

```ts
await nomi.auth.signInWithCode('ada@example.com');      // emails a six-digit code
await nomi.auth.verifyCode('ada@example.com', '123456');

await nomi.auth.signInWithOAuth('github', { redirectTo: location.origin }); // google, apple, or an OIDC provider id

const { data } = await nomi.auth.getSession();
nomi.auth.onAuthStateChange((event, session) => { /* SIGNED_IN, SIGNED_OUT, TOKEN_REFRESHED… */ });
await nomi.auth.signOut();
```

The page at `redirectTo` finishes an OAuth sign-in by itself. Access tokens are refreshed in the
background. Server code made with the service key can manage users with
`nomi.auth.admin.listUsers / createUser / updateUserById / deleteUser`.

## Storage

```ts
await nomi.storage.from('avatars').upload(`${user.id}.png`, file, { upsert: true });
const { data } = await nomi.storage.from('avatars').createSignedUrl(`${user.id}.png`, 600);
const { publicUrl } = nomi.storage.from('public').getPublicUrl('logo.svg');
```

## Realtime

```ts
const stop = nomi.realtime.on('task', (change) => {
  if (change.event === 'resync') return refetch(); // reconnected: changes may have been missed
  apply(change);                                   // insert / update / delete, row-level security applied
});
```

## Functions

Call a function from the app:

```ts
const { data, error } = await nomi.functions.invoke('place-order', { body: { cart } });
```

Write one in `nomi/functions/place-order.ts`:

```ts
import { defineFunction } from 'nomi-relay/functions';

export default defineFunction(async (req, ctx) => {
  if (!ctx.user) return new Response('sign in first', { status: 401 });
  const { data } = await ctx.nomi.from('order').insert({ owner: ctx.user.id }).select();
  return Response.json(data);
});
```

`ctx.nomi` acts as the caller (row-level security applies), `ctx.admin` uses the service key,
and `ctx.env` holds the project's secrets.

## Runtimes

ESM only, with no runtime dependencies. Works in browsers, Node 20+, Deno and Bun.

Use the command line, [`nomi-relay-cli`](https://www.npmjs.com/package/nomi-relay-cli), for
migrations and functions.
