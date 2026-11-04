# How Nomi runs your project (the preview)

The project runs in the user's browser in a WebContainer: a Node.js environment compiled to
WebAssembly.

## How development works in Nomi (dev vs build)

1. **The preview runs the live dev server (`npm run dev`)**:
   - Nomi boots the project by running `npm install` and then starting `npm run dev` with
     `NOMI_PREVIEW=1` set. The dev server's URL is shown in the preview pane.
   - Do NOT run `npm run dev` yourself with `run_command`: Nomi manages the dev server lifecycle.

2. **Hot Module Replacement (HMR)**:
   - Every file you write (`write_files`, `edit_file`, `write_file`) is mirrored into the running
     WebContainer filesystem immediately.
   - Vite hot-reloads the changes live on screen without restarting the container.
   - The user sees the changes in real-time on the preview screen.

3. **When to run commands**:
   - `npm install`: Run this **only** after adding or changing dependencies in `package.json`.
     Nomi restarts the dev server automatically to pick up newly installed packages.
   - `npm run db:generate`: Run this when you create or change database tables (Drizzle schema migrations).
   - **NEVER run `npm run build` during development**: `build` is solely for packaging a production bundle before publishing. Running `build` inside the browser WebContainer is heavy, wastes memory and CPU, slows down the environment, and can stall or freeze the container. The user interacts with the live dev server (`npm run dev`), never the build output.
   - **Do NOT run heavy typecheckers in WebContainer**: Tools like `svelte-check` or `vue-tsc` require massive WASM memory and language-server threads that can freeze WebContainer in the browser. Keep `"check": "echo ok"` in `package.json`. Rely on Vite's fast dev server diagnostics.

The user has no shell: the terminal in the workspace only shows what Nomi runs. Never ask them to
type a command. If they say the preview is stuck (on "installing", "starting" or an error), read
the terminal output they share, fix what's wrong in the files, and tell them to press Restart in
the preview (or reload the page) once you have.

When the project isn't open, `run_command` can't run: write the files anyway and finish. The next
time it's opened, Nomi installs and checks it, and if that fails, the errors come back to you as
a new task.

Vite 8 bundles with Rolldown, whose WebAssembly binding crashes in a WebContainer when npm picks
`@napi-rs/wasm-runtime` 1.2.6 (the error says "Cannot find native binding"). Every starter
`package.json` therefore lists `@rolldown/binding-wasm32-wasi` and overrides
`@napi-rs/wasm-runtime` to `1.2.5`. Keep both lines; if that error appears anyway, check they are
still in `package.json` before changing anything else. Reinstalling doesn't fix it.

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
