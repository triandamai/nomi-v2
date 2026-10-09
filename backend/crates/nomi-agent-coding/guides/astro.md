# Astro 7 (content sites, with islands)

Use Astro for content-first sites: landing pages, blogs, docs, portfolios. Interactive parts are
islands written in Svelte (preferred) or React. Server endpoints and SSR need the Node adapter.

`package.json`:
```json
{
  "name": "app",
  "type": "module",
  "private": true,
  "scripts": { "dev": "astro dev", "build": "astro check && astro build", "check": "astro check", "preview": "astro preview" },
  "dependencies": {
    "astro": "^7.3.8", "@astrojs/node": "^11.1.7", "@astrojs/svelte": "^9.0.1", "svelte": "^5.57.0",
    "tailwindcss": "^4.3.0", "@tailwindcss/vite": "^4.3.0"
  },
  "devDependencies": { "@astrojs/check": "^0.9.10", "typescript": "^6.0.3" }
}
```
(Leave out `@astrojs/node` and `output: 'server'` for a fully static site.)

`astro.config.mjs`:
```js
// @ts-check
import { defineConfig } from 'astro/config';
import node from '@astrojs/node';
import svelte from '@astrojs/svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  output: 'server',
  adapter: node({ mode: 'standalone' }),
  integrations: [svelte()],
  vite: { plugins: [tailwindcss()] },
});
```

`tsconfig.json`: `{ "extends": "astro/tsconfigs/strict", "include": [".astro/types.d.ts", "**/*"], "exclude": ["dist"] }`

`src/styles/global.css`: `@import 'tailwindcss';`, imported from the layout's frontmatter.

Pages are `src/pages/*.astro` (frontmatter between `---` runs on the server); API routes are
`src/pages/api/*.ts`:
```ts
import type { APIRoute } from 'astro';

export const GET: APIRoute = () => Response.json({ hello: 'world' });
```
Islands: `<Counter client:load />` (Svelte 5 component with runes). Shared layout in
`src/layouts/Layout.astro` with `<slot />`. With a database, follow the database guide, but read
`DATABASE_URL` and `NOMI_PREVIEW` from `process.env` (there's no `$app/env/private` in Astro) and
call `migrateDb()` at the top of `src/middleware.ts` or the first endpoint that needs it.
