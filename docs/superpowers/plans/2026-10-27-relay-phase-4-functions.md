# Relay Phase 4: hosted functions

**Goal.** Any app gets trusted server code by Koda writing one file in `nomi/functions/`: Relay
checks it, runs it, and the gateway serves it at `/functions/<name>`.

**Spec.** `relay/README.md` (Functions) and phase 4 of
`docs/superpowers/plans/2026-10-26-relay-service-and-koda.md`, with one change decided on
2026-10-27: **Relay runs the functions itself** instead of a Shipyard service per project
(Shipyard gives an environment one public URL and Relay can't reach an app's other services).

## Decisions

- **`relay/functions-runtime/`**: a Deno service run next to Relay (its own container, its own
  secret). It keeps each project environment's current build on disk and runs it in its own Deno
  worker with only network access and read access to that build's folder: no environment, files,
  subprocesses or FFI. A worker idle for 10 minutes is stopped (the "sleep"); the next call starts
  it again.
- **The runtime's API** (every call carries `x-relay-runtime: <secret>`):
  - `GET /health`
  - `PUT /builds/<project>/<env>/<version>` `{ files, env, functions }`: writes the files,
    type-checks them with `deno check`, starts the worker and checks each file's default export is
    a function. Answers `{ ok: true }` or `{ ok: false, error }` with the file and line.
  - `* /run/<project>/<env>/<version>/<name>[/…]`: one call, with `x-nomi-claims`,
    `x-relay-timeout` and the caller's own `authorization`. `412 NOT_LOADED` when that build isn't
    on disk (after a restart).
- **Imports**: `npm:` specifiers and files in `_shared/`. `nomi-relay` and `nomi-relay/functions`
  are rewritten to the `npm:nomi-relay@<version>` the runtime pins.
- **`ctx.env`** is the project's secrets plus `NOMI_URL`, `NOMI_ANON_KEY` and `NOMI_SERVICE_KEY`,
  handed to the worker by the runtime (the worker can't read the process environment). Nomi
  forwards every project secret to Relay when an admin sets it (sealed, as sign-in secrets are);
  secrets saved before this ships need entering once more.
- **Builds** live in Relay's state (`relay.function_builds`), numbered per project environment.
  A deploy that fails its check never replaces the running build. Rollback makes the previous
  good build current. When the runtime answers `412`, the gateway sends the current build again
  and retries the call once; a load that takes over 10 seconds answers `503` with
  `Retry-After: 2`.
- **Settings per function** from `backend.json`'s `functions`:
  `{ "place-order": { "auth": "required" | "public", "timeout": 30, "schedule": "0 * * * *" } }`.
  `auth` defaults to `required` (a signed-in user or the service key); timeout defaults to 30
  seconds, at most 60.
- **Schedules**: the gateway checks every minute, claims each due run in `relay.function_runs`
  (so two Relay instances never both run it) and calls the function with the service role.
- **Relay settings** (Nomi's admin): the runtime's URL and secret; saving checks `/health`.
  Without them `/functions` answers `503 RELAY_NO_FUNCTIONS`.
- **Nomi**: Koda gets `deploy_functions`; writing under `nomi/functions/` deploys to dev and the
  result comes back with the write. Publishing deploys to production after migrations; rolling a
  publish back rolls the functions back.
- **CLI**: `functions serve` runs `nomi/functions/` locally on Node with the same context.
  `functions deploy` and `secrets set` wait for the CLI's `login`/`link` (phase 2c).

## Tasks

1. Runtime (Deno): host, worker, import rewriting, check, idle stop, timeouts; `deno test`.
2. Relay: settings (`functions_url`, `functions_secret`), state migration, secrets for every name,
   `relay-control::functions` (deploy, rollback, current), control routes, `describe`.
3. Gateway: `/functions/<name>` proxy with auth, claims, reload-on-412, wake hold.
4. Schedules.
5. Nomi: tool, write trigger, publish and rollback, secrets forwarding, guide, admin fields.
6. CLI `functions serve`.
7. End to end with the real SDK and the runtime in Docker.

## Review focus

- A function that never answers: the call gets `504` at its timeout and the worker is restarted.
- A function that loops forever on import: the deploy fails its check instead of hanging.
- Claims can't be forged: only the gateway's runtime secret makes the runtime trust `x-nomi-claims`.
- A file outside the function's build (`../`, `/etc`) can't be imported or read.
- Two Relay instances: a scheduled function still runs once a minute.
