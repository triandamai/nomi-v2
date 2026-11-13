# Sandbox runtime (boat.dev)

An optional server-side runtime for projects: instead of WebContainer in the user's browser, a
project's files, commands and dev server live in a boat.dev sandbox driven by the Nomi server
(`nomi-sandbox` crate, `nomi-server/src/sandbox_*.rs`). It is off unless an admin turns it on in
Admin -> Configuration -> Sandbox and sets a boat API key.

## Architecture

The Nomi server owns every sandbox. A per-project row (`project_sandboxes`) records what the
project wants (`desired`: running or stopped) and where the sandbox is (`state`). A background
worker claims due rows and a reconciler walks each one toward its goal through a provider trait,
so boat.dev is one implementation among possible others and tests use a fake. The sandbox holds
only project files; the agent, the database and all credentials stay on the server. The page
asks to wake the sandbox, shows its phase from MQTT `SandboxChanged` events, and loads the
preview URL in an iframe. With the switch off, every project keeps using WebContainer.

| Piece | Where |
| --- | --- |
| Provider trait, boat client, fake provider, start limiter, sync planning | `backend/crates/nomi-sandbox/` |
| Row store (desired/state, leases, retries, deletion outbox) | `backend/crates/nomi-server/src/sandbox_store.rs` |
| Reconciler (create, resume, stop, limits, backoff, stop-only mode) | `.../sandbox_manager.rs` |
| Worker loop (claims, idle sweep, wind-down, usage drift, reaper schedule) | `.../sandbox_worker.rs` |
| File sync into the sandbox | `.../sandbox_sync.rs` |
| Dev server start, readiness, preview environment | `.../sandbox_devserver.rs` |
| Project steps glue (sync + preview) | `.../sandbox_steps.rs` |
| Koda's commands run inside the sandbox | `.../sandbox_runs.rs` |
| Agent-side wake and wait | `.../sandbox_ops.rs` |
| Metered minutes and monthly allowance | `.../sandbox_usage.rs` |
| Deletion outbox drain and orphan reaper | `.../sandbox_cleanup.rs` |
| HTTP routes `/api/projects/:id/sandbox*` | `.../routes/project_sandbox.rs` |
| Schema | `backend/migrations/0106_project_sandboxes.sql` |
| Settings section and parsing | `backend/crates/nomi-settings/src/config.rs`, `.../nomi-server/src/configuration.rs` |
| Frontend state, runtime choice, preview iframe | `frontend/src/lib/sandboxRuntime.svelte.ts`, `runtimeChooser.ts`, `components/ProjectPreview.svelte` |

Migration 0106 was edited in place while the feature was being developed. Any development
database that already ran an earlier version of it must be reset. It must never be edited after
the branch is merged; further schema changes go in a new migration.

## Configuration reference

Admin -> Configuration -> Sandbox (section `sandbox`). Values are saved in settings; only
`api_base` and `api_key` can also come from the environment when nothing is saved.

| Field | Meaning | Default |
| --- | --- | --- |
| `enabled` | Master switch and kill switch. Anything but `on` keeps projects on WebContainer and puts the worker in stop-only mode. | off |
| `api_base` (env `BOAT_API_BASE`) | boat API address. | `https://boat.dev/api/v1` |
| `api_key` (env `BOAT_API_KEY`) | boat API key, stored encrypted, never returned. Without it the runtime is unavailable. | none |
| `size` | `small`, `default` or `large`. | `small` |
| `idle_stop_secs` | Idle time before a sandbox is stopped. | 600 |
| `auto_stop_secs` | The provider-side auto-stop asked on every create, resume and extension. boat.dev's free trial refuses more than 7200 (`trial_auto_stop_required`); a paid account may take longer. Values under 3600 are raised to 3600. | 7200 |
| `rollout` | Who gets sandboxes for new projects: `off`, `allowlist`, `percent`, `all`. | `all` |
| `rollout_allowlist` | Comma-separated user ids for `allowlist`. | empty |
| `rollout_percent` | 1 to 100 for `percent`; a user stays in as it is raised. | 0 |
| `starts_per_minute` | Create/resume starts per minute (boat allows 12). | 10 |
| `starts_per_hour` | Create/resume starts per hour (boat allows 60). | 50 |
| `monthly_minutes_per_user` | Sandbox minutes per owner per month. | 600 |
| `reaper_mode` | Hourly orphan sweep: `dry_run` logs, `delete` deletes. | `dry_run` |

## Runbook

Where to look first: the admin overview shows sandboxes running, starting, in error, and minutes
this month. Server logs carry `sandbox:` and `sandbox reaper` lines with ids but never URLs or
keys.

| Situation | What to do |
| --- | --- |
| Provider outage | Rows back off and retry on their own; after repeated failures a row is parked in `error` with a redacted `last_error` (its sandbox is stopped and its metering closed; closing the page or the kill switch stops it at once), and after a provider cooldown the worker tries again. Pages show the error and the project can still use WebContainer. If it lasts, turn the switch off (below). Stuck rows clear with the worker's `unstick` pass once the provider is back. |
| Rate limits | Starts are paced by `starts_per_minute` and `starts_per_hour`; over the limit a wake is deferred, not failed. A provider 429 is honoured via its retry-after. If users wait too long, raise the limits only up to the plan's. |
| Stuck rows | A row in `creating`, `starting` or `stopping` for over 20 minutes is released by the worker (`SandboxManager::unstick`). Inspect with `SELECT project_id, state, attempts, last_error, retry_at FROM project_sandboxes WHERE state IN ('error','creating','starting','stopping')`. A row parked in `error` stays held back until the cause is fixed; read `last_error` first. |
| Orphaned sandboxes | The hourly reaper lists provider sandboxes labelled `nomi-project` that no row or outbox entry names. Keep `reaper_mode` at `dry_run` first, read the `would delete` log lines, then set `delete`. Use a boat account dedicated to this deployment. |
| Deletion outbox | A sandbox that could not be deleted is queued in `sandbox_deletions` and retried by the worker; rows with a repeating `last_error` need the provider checked, not the table edited. |
| Key rotation | Create a new key at boat, save it in Admin -> Configuration -> Sandbox, confirm a sandbox still starts, then revoke the old key. (Not yet verified live: whether running sandboxes survive a key change.) |
| Cost spike | Check minutes this month on the overview, lower `idle_stop_secs`, lower `monthly_minutes_per_user`, or move `rollout` back a stage. Usage is compared with the provider's and drift is logged. |
| Kill switch | Set `enabled` to off. The worker enters stop-only mode: it stops and deletes sandboxes, and never creates or resumes one, even if a start races the switch. New pages use WebContainer. Needs the key saved to keep winding down. |
| Install or dev server fails | The sandbox still counts as running (metered, idles out as usual) with no preview and the reason in `last_error`; the failure goes to Koda as a failed check, Koda's commands run in it, and the preview is tried again when the files change or the person presses Retry. |
| Dead running sandbox | Every 2 minutes (at most 25 per pass) the worker asks the provider about running sandboxes; one that stopped, crashed or vanished goes back to `stopped`, its session is closed, and it is resumed or recreated. Sandboxes are created and resumed with the provider auto-stop from `auto_stop_secs`, recorded in `auto_stop_at`; this pass pushes it out again once half of it is left, or 2 hours (`AUTO_STOP_EXTEND_BEFORE`) for an auto-stop over 4 hours. boat documents no extend call, so the extension resends `ttlSeconds` through `POST /sandboxes/{id}/resume` and then saves the deadline boat reports (`archiveAfter`, from the reply or a `GET`); `now + auto_stop_secs` is assumed only when boat reports none. A refused extension, or one whose reported deadline didn't move, is backed off (`auto_stop_failures`, `auto_stop_retry_at`: 10 min doubling to 40 min, or to half the margin when that is shorter), so boat is asked a few times before the deadline, not on every pass; each try takes a start from the limiter, since the contract counts resumes as starts. If the deadline still passes, the sandbox auto-stops and this pass resumes it. The idle sweep stops sandboxes long before; the auto-stop bounds leaks while Nomi is down. |
| Dead dev server in a live sandbox | The same pass runs the readiness check (`curl` on port 5173) against sandboxes that have a preview. One that doesn't answer loses its preview, `devserver_restarted_at` and `devserver_restart_reason` are set, and the next walk restarts it through the preview start. At most one restart per 10 minutes (`DEVSERVER_RESTART_GAP`); a restart that fails shows as a failed preview with Retry. |
| Dependencies change in a running sandbox | When a sync ran `npm install` (the dependency files changed), the dev server is killed and started again and the new preview address saved. |
| Browser runtime beside a sandbox | While the browser runtime's check-in is fresh (`runner_seen_at`, 30 s), the sandbox claims none of Koda's commands and the worker dispatches none; only commands the sandbox ran (`project_runs.ran_in_sandbox`) count as its activity for the idle sweep. "Run in my browser instead" also posts `/sandbox/stop`. |
| Dev server logs | The end of the dev server's output (8 KiB) and its process id are saved on the row, so the API process serves logs written by the worker binary. |
| Rollback | Switch off, wait until the overview shows no running sandboxes, then set `rollout` to `off`. Projects fall back to WebContainer; files are in Nomi's storage and come back from there. |

## Test results

Automated, against `nomi_sandbox::FakeProvider` (no boat account exists yet):

- `cargo test -p nomi-server --test sandbox_load`: `burst_of_200_wakes_respects_start_limits_and_all_settle`
  wakes 200 projects at once with 50 starts per minute. The manager's limiter runs on the real
  clock, so each of the four minute windows is stood in for by a fresh manager; the test asserts
  no window admits more than 50 starts, every project ends `running`, each is started exactly
  once, and no deferral counts as a failed attempt. It runs in seconds.

OWED (needs a boat key; not done): the live run and the 24-hour soak. Run these once a key
exists, using a throwaway key and a small `ttlSeconds`, and record the results here:

- [ ] Create a sandbox project as each seed account; let Koda build a SvelteKit page.
- [ ] Time stop and resume; record cold start, resume and sync times and the cost of the run.
- [ ] Auto-stop extension: on a running sandbox created with a small `ttlSeconds`, call `POST /sandboxes/{id}/resume` with a new `ttlSeconds` and record (a) whether it answers 202 or a 409, (b) whether `archiveAfter` moves to now + the new TTL, (c) whether running processes (the dev server) survive it, and (d) whether it counts against the start limits. Until then Nomi trusts `archiveAfter` as the auto-stop deadline (inferred from the docs' example, `createdAt + ttlSeconds`), and assumes now + 6 h only when boat reports none. If (b) or (c) fails, replace the extension with a different mechanism before rollout.
- [ ] Edit a file and confirm HMR works through the preview iframe.
- [ ] Confirm the preview works under the project page's `Cross-Origin-Embedder-Policy: credentialless`.
- [ ] Check whether boat's `kill_process` kills the whole process tree (dev server children).
- [ ] Check whether `fuser` exists in the image (port cleanup depends on it).
- [ ] Record the real upload-URL host and confirm it passes the SSRF check.
- [ ] Check preview token behaviour across stop and resume (the security model assumes it is stable).
- [ ] 24-hour soak with 10 sandboxes on a use/idle cycle: no leaked sandboxes, no stuck rows,
  usage within the agreed drift of the provider's.
- [ ] Clean up: delete test sandboxes, projects and the key; keep the seed accounts
  `admin.full@nomi.test`, `user.noaccess@nomi.test` and `user.partial@nomi.test`.

## Production-readiness checklist

| Item | Status |
| --- | --- |
| Backend tests green (`cargo test --workspace`) | DONE for the sandbox suites on this branch (see Test results); run the full workspace before merge |
| Frontend tests green (`pnpm --dir frontend vitest run`, `pnpm --dir frontend check`) | OWED: run before merge |
| Migration 0106 applied on a staging copy | OWED |
| Key rotated into Admin -> Configuration | OWED: no boat account yet |
| Provider plan limits above configured starts per minute and hour | OWED: defaults (10 and 50) sit under boat's published 12 and 60; confirm for the actual plan |
| Alerts or a daily check on the dashboard counters | OWED: the counters exist (running, starting, error, minutes) on the admin overview; someone must check them daily |
| Reaper dry-run reviewed, then enabled | OWED: ships in `dry_run`; review a week of logs, then set `delete` |
| Rollback rehearsed (kill switch, pages fall back to WebContainer) | DONE in tests (`kill_switch_*`, `stop_only_never_creates_*` in `tests/sandbox_worker.rs`); OWED on staging |
| Security review | DONE: findings fixed in commits `5f89291b` and `2f8de288`; residual risks listed in Security model |
| Load test against the fake provider | DONE (`tests/sandbox_load.rs`) |
| Live run and 24-hour soak | OWED (checklist above) |

## Staged rollout

Rollout is configuration only, and applies to **new** projects. Existing projects stay on the
browser runtime until their owner opts in.

1. `off`: everyone on WebContainer. Start here with the key saved and the switch on.
2. `allowlist`: the team's user ids, for one week.
3. `percent` 10, then `percent` 50.
4. `all`.

Between stages watch the error count, resume latency (p95), minutes per user and spend against
the plan. Do not advance automatically: review with the product owner after the allowlist week
and decide on the first percentage; each later step is a separate decision. To go back, lower
the stage or use the kill switch.

## Security model

### What the sandbox is trusted with

A sandbox is treated as untrusted, project-owned compute. It runs the project's own code and
Koda's commands, so anything inside it can read anything else inside it. It is given only:

- the project's files, copied from Nomi's storage;
- `NOMI_PREVIEW` and `NODE_NO_WARNINGS`;
- for a project on Relay, the project's own **dev** backend address and anon key, and its dev
  service key (the owner's role is admin, so the owner's browser already holds the same key in
  the WebContainer runtime). These are project-scoped app keys, never Nomi's.

It is never given Nomi's own secrets: the JWT secret, the database URL, the settings encryption
key, LLM keys, Shipyard keys or the boat API key. The environment is built from a fixed list
(`sandbox_devserver::preview_env`), never from the server's process environment; a unit test pins
the list. Variables reach the sandbox as `env NAME='value'` with every value shell-quoted. Note
that the dev service key does travel to boat.dev as part of the start command, so boat is trusted
with it as it is with the project's code.

### boat API key

The key is stored encrypted as a `Secret` settings field and is never returned by the admin API.
`BoatConfig` and `SandboxConfig` redact it in `Debug`. It is only ever sent as the bearer header
to the configured API base, never in a request body, a command, the sandbox, an MQTT event or the
frontend. Every error built from provider text passes through `BoatProvider::scrub`, which removes
the key and URL query strings, user info and fragments (`nomi_sandbox::redact`); network errors
also drop their URL. The API base is admin-configurable; admins are trusted not to point it (and
the key) at a host they do not control.

### Preview URL

A private port's preview URL carries its access token (`?_token=...`), so the URL itself is the
secret. It is stored in `project_sandboxes.preview_url` and returned by
`GET /api/projects/:id/sandbox` only to people who can open the project, and only while the
sandbox is running. `SandboxChanged` MQTT events carry the phase only. `PreviewUrl`'s `Debug` is
redacted and the URL is never logged. Anything saved as `last_error` goes through `redact_urls`
in the store, so a provider or step error that echoes a URL cannot persist its token there. The
provider must return an `https` URL (checked server-side, and again in the frontend before it is
used), and the frontend only ever uses it as the preview iframe's `src`.

Residual risk: boat keeps the same token for a (sandbox, port) pair across stops and resumes, so a
person removed from a project who copied the URL keeps access to that preview until the sandbox
is recreated. Deleting the project deletes the sandbox.

### Commands and paths

Koda's commands are queued as program plus arguments. Before running one the server re-checks it
(`sandbox_runs::validate`): it must survive `parse_command` unchanged, so the program is on the
`npm | npx | node` allow-list, no shell operators are present and dev-server starts are refused;
each argument is then single-quoted. The program name is the only unquoted word and comes from
the allow-list. Other command lines (install, dev server start, readiness check, port cleanup,
archive extraction) are built from constants plus shell-quoted values.

Project paths come from the database and are checked with `validate_path` (relative, no `..`, no
empty segment, no NUL or backslash, not under `node_modules`) before they are written, archived or
deleted; the per-sandbox memory of synced paths is re-checked too, since a remembered path is
deleted recursively. `build_archive` refuses absolute and `..` paths and writes regular files
only, and extraction uses `tar --no-same-owner --no-same-permissions` (GNU tar also strips a
leading `/` and refuses `..` members). Code inside the sandbox can create symlinks in the project
directory, and a later extraction would follow them, but that only reaches other places in the
same sandbox, which that code could already write to. Sandbox and process ids are checked to be
plain names before they go into a boat request path.

### Server-side requests (SSRF)

The server makes outgoing requests only to the configured API base, plus one `PUT` of the project
archive to the signed upload URL boat returns. That URL is checked first: it must be on the API's
own origin, or be `https` on a public host name (no IP literal, no `localhost`, `.internal`,
`.local` and similar names, with or without a trailing dot, no user info). The PUT is sent on a
client that never follows redirects, and a redirect answer fails the upload, so a checked URL
cannot hop to an unchecked host. The preview URL is never fetched by the server; the readiness
check runs `curl` inside the sandbox against `localhost`.

Residual risk: the check looks at the host name only, not at what it resolves to. A public name
that resolves to a private address (for example `127.0.0.1.nip.io`), or DNS rebinding between
the check and the request, is not blocked; that would need a resolver that checks the addresses
it connects to.

### Authorization

Every `/api/projects/:id/sandbox*` route loads the project with `can_open_project` and answers
404 to anyone else (tested for all four routes). Starting also requires the admin switch, the
rollout, the owner's monthly allowance and no cooldown, and the same check
(`routes::project_sandbox::availability`) gates the sandbox page's heartbeat
(`GET /runtime?runtime=sandbox`, sent only by the sandbox page) and Koda's wake. The browser
runtime's check-in (the fallback, or "run in my browser") never wakes the sandbox and claims Koda's
commands as before; Koda uses the sandbox only when no browser runtime is open on the project.
Background tasks of sandbox projects always go through the project runtime, never the host shell.
Stopping is open to members. The sandbox settings
section is behind the admin configuration permission. `sandbox_deletions` and the reaper are
worker-only and have no routes. The reaper only touches sandboxes labelled `nomi-project`, so a
boat account should be dedicated to one Nomi deployment.

### Outbound network from the sandbox

Decision: egress is left open (boat's default), because `npm install`, `npx` and the project's
own Relay backend need the public internet. The sandbox runs on boat's infrastructure, not inside
Nomi's network, so it cannot reach Nomi's database or internal services directly. If boat offers
firewall rules, the recommended tightening is to deny link-local and cloud metadata ranges
(`169.254.0.0/16`, `fd00:ec2::254`) and private ranges (`10.0.0.0/8`, `172.16.0.0/12`,
`192.168.0.0/16`, `100.64.0.0/10`) while keeping public egress open. This is not configured by
Nomi today.
