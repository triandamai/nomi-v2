# How Nomi runs your project (the preview)

The project runs in the user's browser in a WebContainer: a Node.js environment compiled to
WebAssembly. When the user has the project open:

1. Nomi copies the project's files in, runs `npm install`, then `npm run check` (when the script
   exists), and starts `npm run dev` with `NOMI_PREVIEW=1` set. The dev server's URL is shown in
   the preview pane.
2. Every file you write is copied into the running container straight away, so Vite reloads it.
3. `run_command` runs a command there and gives you its output. You can run `npm install`,
   `npm run check`, `npm run build`, `npm run db:generate`, `npx ...` and `node ...`. Don't run
   `npm run dev` (Nomi manages the dev server) or anything that never exits.

When the project isn't open, `run_command` can't run: write the files anyway and finish. The next
time it's opened, Nomi installs and checks it, and if that fails, the errors come back to you as
a new task.

What doesn't work in the preview:
- Native modules (`better-sqlite3`, `bcrypt`, `sharp`, `pg-native`, anything with a `binding.gyp`
  or a platform-specific binary). Use pure-JS or WebAssembly packages instead: for password
  hashing, `node:crypto`'s `scrypt` (built in) or `bcryptjs`.
- Database servers and Docker: use the in-browser databases from the database guide.
- Long-running background processes besides the dev server.
- Outbound network calls are limited to `fetch` over HTTPS (no raw TCP).

Rules for every project:
- TypeScript everywhere, strict mode on. No `any` unless unavoidable.
- Vite as the build tool (SvelteKit, React, Vue and Astro all use it).
- Tailwind CSS v4 for all styling: `@import 'tailwindcss';` in the main CSS file and the
  `@tailwindcss/vite` plugin. No `tailwind.config.js`, no `postcss.config.js`; customise with
  `@theme { --color-brand: #0f766e; }` in CSS. No other CSS frameworks or component kits.
- package.json must have `dev`, `build` and `check` scripts.
- Don't commit generated folders (`node_modules`, `.svelte-kit`, `dist`, `build`, `.data`).
