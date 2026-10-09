# SvelteKit 3 + Svelte 5 (Nomi's first-class stack)

Use this for any app with pages and a server: forms, auth, a database, API routes. It's one
project: pages, server code and the database layer live together.

SvelteKit 3 is NOT SvelteKit 2. Things you may remember that are now wrong:

| SvelteKit 2 (don't)                          | SvelteKit 3 (do)                                              |
|----------------------------------------------|---------------------------------------------------------------|
| `svelte.config.js`                           | No such file. Kit options go in `sveltekit({...})` in `vite.config.ts` |
| `$lib/...`                                   | `#lib/...` WITH the file extension: `#lib/server/db/index.ts`, `#lib/Button.svelte` |
| `$env/static/private`, `$env/dynamic/private`| Declare in `src/env.ts`, import from `$app/env/private` (or `$app/env/public`) |
| `$app/environment`                           | `$app/env` (`browser`, `dev`, `building`)                     |
| `import type { ServerInit } from '@sveltejs/kit'` | Hook types come from `@sveltejs/kit/hooks`              |
| tsconfig extends `./.svelte-kit/tsconfig.json` | `"extends": "$app/tsconfig"`                                |
| `export let data`                            | `let { data }: PageProps = $props();`                         |

## Files to start with

`package.json`:
```json
{
	"name": "app",
	"private": true,
	"version": "0.0.1",
	"type": "module",
	"scripts": {
		"dev": "vite dev",
		"build": "vite build",
		"preview": "vite preview",
		"prepare": "svelte-kit sync || echo ''",
		"check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
		"db:generate": "drizzle-kit generate"
	},
	"devDependencies": {
		"@sveltejs/adapter-node": "^6.0.0",
		"@sveltejs/kit": "^3.0.1",
		"@sveltejs/vite-plugin-svelte": "^7.3.0",
		"@tailwindcss/vite": "^4.3.0",
		"@types/node": "^22",
		"svelte": "^5.57.0",
		"svelte-check": "^4.7.0",
		"tailwindcss": "^4.3.0",
		"typescript": "^6.0.3",
		"vite": "^8.3.0"
	},
	"imports": {
		"#lib": "./src/lib/index.js",
		"#lib/*": "./src/lib/*"
	}
}
```
Add the database packages from the database guide when the app needs one. Don't add `.npmrc`.

`vite.config.ts`:
```ts
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
			},
			adapter: adapter()
		})
	]
});
```

`tsconfig.json`:
```json
{
	"extends": "$app/tsconfig",
	"compilerOptions": { "strict": true, "types": ["$app/types", "node"] },
	"include": ["src", "vite.config.ts", "drizzle.config.ts"]
}
```

`src/app.html`:
```html
<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1" />
		%sveltekit.head%
	</head>
	<body data-sveltekit-preload-data="hover">
		<div style="display: contents">%sveltekit.body%</div>
	</body>
</html>
```

`src/app.d.ts`:
```ts
declare global {
	namespace App {
		// interface Locals {}
	}
}
export {};
```

`src/routes/layout.css`: `@import 'tailwindcss';` (plus `@theme { ... }` for brand colours/fonts).

`src/routes/+layout.svelte`:
```svelte
<script lang="ts">
	import './layout.css';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

{@render children()}
```

`src/lib/index.ts`: can be empty (`export {};`). `.gitignore`: `node_modules`, `/.svelte-kit`,
`/build`, `.env`, `.data`, `*.db`.

## Environment variables

Every variable the app reads is declared in `src/env.ts`. A variable that may be missing needs a
`schema` that returns `undefined` (without one, it's required):
```ts
import { defineEnvVars } from '@sveltejs/kit/env';

export const variables = defineEnvVars({
	DATABASE_URL: { description: 'Database connection string.', schema: (v) => v },
	NOMI_PREVIEW: { description: 'Set by the Nomi preview.', schema: (v) => v }
});
```
Read them with `import { DATABASE_URL } from '$app/env/private';` (server-only files). Use
`public: true` and `$app/env/public` only for values safe to show in the browser.

## Pages, loading and forms

- `src/routes/+page.svelte` (UI), `+page.server.ts` (`load` + `actions`, runs on the server),
  `+page.ts` (universal load, rarely needed), `+layout.svelte`, `+server.ts` (API endpoints:
  `export const GET: RequestHandler = ...`), `+error.svelte`.
- Types come from `./$types`: `PageServerLoad`, `Actions`, `PageProps`, `LayoutProps`,
  `RequestHandler`.
- Server-only code goes in `src/lib/server/...` (importing it from the browser is a build error).

`src/routes/+page.server.ts`:
```ts
import { fail } from '@sveltejs/kit';
import { db } from '#lib/server/db/index.ts';
import { task } from '#lib/server/db/schema.ts';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async () => ({ tasks: await db.select().from(task).orderBy(task.id) });

export const actions: Actions = {
	add: async ({ request }) => {
		const title = String((await request.formData()).get('title') ?? '').trim();
		if (!title) return fail(400, { error: 'Give the task a title.' });
		await db.insert(task).values({ title });
	}
};
```

`src/routes/+page.svelte`:
```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<form method="POST" action="?/add" use:enhance class="flex gap-2">
	<input name="title" class="flex-1 rounded-lg border px-3 py-2" />
	<button class="rounded-lg bg-zinc-900 px-4 py-2 text-white">Add</button>
</form>
{#if form?.error}<p class="text-red-600">{form.error}</p>{/if}
<ul>
	{#each data.tasks as t (t.id)}<li>{t.title}</li>{/each}
</ul>
```

Hooks (`src/hooks.server.ts`): `init` runs once before the first request, `handle` wraps each:
```ts
import type { Handle, ServerInit } from '@sveltejs/kit/hooks';
import { migrateDb } from '#lib/server/db/index.ts';

export const init: ServerInit = async () => {
	await migrateDb();
};

export const handle: Handle = ({ event, resolve }) => resolve(event);
```
Other imports: `redirect`, `error`, `fail` from `@sveltejs/kit`; `goto`, `invalidateAll` from
`$app/navigation`; `page` from `$app/state` (`page.url`, `page.params`; not a store, no `$`).

## Svelte 5 (runes only)

- State: `let count = $state(0);` (objects and arrays are deeply reactive). Derived:
  `const doubled = $derived(count * 2);` or `$derived.by(() => ...)`. Effects: `$effect(() => ...)`
  only for syncing with the outside world (not for derived values).
- Props: `let { title, onsave, children }: { title: string; onsave?: () => void; children?: Snippet } = $props();`
  Two-way: `let { value = $bindable('') } = $props();`
- Events are attributes: `onclick={...}`, `oninput={...}`, `onsubmit={...}` (never `on:click`).
- Children and slots are snippets: `{@render children?.()}`; named ones via `{#snippet header()}...{/snippet}`
  passed as props. No `<slot>`, no `createEventDispatcher`, no `$:`, no `export let`.
- Shared state lives in `.svelte.ts` files: `export const cart = $state({ items: [] as Item[] });`
- Lists: always key them: `{#each items as item (item.id)}`.

## Database

Read the `database` guide. In short: schema in `src/lib/server/db/schema.ts`, connection in
`src/lib/server/db/index.ts` that uses the in-browser database when `NOMI_PREVIEW` is set,
migrations generated with `npm run db:generate` and applied by `migrateDb()` in the `init` hook.

## Before you finish

Run `npm install` (after changing package.json), then `npm run check`, then `npm run build`.
Fix every error they report.
