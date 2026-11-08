# Sandbox Runtime (boat.dev) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a Nomi project run its dev server, installs and Koda's commands in a server-side boat.dev sandbox instead of the user's browser (WebContainer), from first prototype to a production rollout with cost controls, with WebContainer kept as the default and the fallback.

**Architecture:** `ProjectStore` stays the source of truth for files; the sandbox is a disposable cache that a backend reconciler starts, syncs, serves and stops. The existing `project_runs` queue stays the contract between Koda and "something that runs commands": for sandbox projects a server worker claims the runs and executes them over boat's REST API, where today the open browser page does. The orchestrator (Koda, agents, LLM keys, Postgres) never moves into the sandbox; the sandbox holds no Nomi credentials.

**Tech Stack:** Rust (axum, sqlx, reqwest, wiremock, rumqttc), a new `nomi-sandbox` crate, SvelteKit 5 (`frontend/`, Paraglide i18n), boat.dev REST API `https://boat.dev/api/v1`.

**Spec:** this document, section "Design decisions" (research: boat.dev docs, pricing and limits pages, read 2026-11-07/08, summarised in the section "What the research established"). There is no separate spec file.

## Global Constraints

- **Admin on/off switch, default OFF.** Admin → Configuration → Sandbox has an `enabled` toggle, off by default. When off, every project, including ones with `runtime = 'sandbox'`, runs on the existing WebContainer runtime exactly as today, and the sandbox worker starts nothing. Turning it on affects new starts only; turning it off lets running sandboxes idle out.
- Never edit `frontend-new/`; extend `frontend/` only (memory: frontend-new is hands-off).
- No AI attribution anywhere: no Co-Authored-By lines, no "Generated with" lines in commits, PRs or issues. Keep AI tooling folders gitignored (memory: no AI attribution; this overrides any tooling default).
- All LLM prompt text lives in `backend/crates/nomi-agent-core/src/prompts.rs`; every numeric literal gets a named `const` with a one-line doc comment (memory: no hardcoded prompts or magic numbers).
- Realtime is MQTT. Push sandbox state by extending the existing `project/{id}/stream` envelope in `backend/crates/nomi-server/src/project_events.rs`; never build a parallel relay (memory: use the existing stack).
- The boat API key is read server-side only, kept encrypted with the settings key like Relay's control key; it never reaches a page, a sandbox, a log line or a model.
- Tests that touch a live DB or the live provider clean up after themselves and keep the three seed accounts `admin.full@nomi.test`, `user.noaccess@nomi.test`, `user.partial@nomi.test` (memory: test data cleanup).
- Migration number: the next free one at execution time (latest in the tree is `0105`, so `0106`); re-check `ls backend/migrations` first.
- Work on a dedicated branch `feat/sandbox-runtime` created from an up-to-date `main` (`git fetch origin && git worktree add -b feat/sandbox-runtime ../nomi-v2-sandbox origin/main`), not on `feat/project-toolbar-split-share` (it has unrelated uncommitted changes).
- Execution: subagent-driven, token-lean: each implementer gets only its own task text, the Global Constraints and the Interfaces it consumes, not the whole plan or research; reviewers check the diff against that task only; one whole-branch review at the end.
- boat.dev limits that bind the design: waiting commands max 600 s (default 30 s); one file per `PUT /sandboxes/{id}/files`, 5 MiB each, only under `/home/user` or `/tmp`; create/fork/resume each count as a start and are never queued by boat (retry after `nextStartAt`, send `Idempotency-Key` on create); detached processes do not survive stop/resume; resume changes the sandbox IP; stopped disks are kept at least 14 days; Vite needs `server.allowedHosts: [".on.boat.dev"]`.

## What the research established (and what it did not)

Known from the docs: stop and resume exist in the API; background commands stream NDJSON; public URLs are `https://<sandbox-subdomain>-<port>.on.boat.dev` with a `_token`, and WebSockets work for `--localhost` apps; typical platform spend is $1–5 per user-month, the $20 plan covers 555 `default`-hours with 12 starts/min and 60/hour.

Not established, and gated by Task 0: how long a resume really takes, whether HMR works through an iframe from Nomi's origin, whether the preview URL's token can be minted/revoked per view, whether a dev server runs reliably as a systemd service, whether the tarball path (upload URL + `tar`) works, and whether the page's `Cross-Origin-Embedder-Policy: credentialless` header (set for project pages in `frontend/src/hooks.server.ts`) blocks an iframe of `on.boat.dev`.

## Design decisions

1. **Per-project runtime.** `projects.runtime` is `'browser'` (default, today's behaviour) or `'sandbox'`. Everything below applies only to `'sandbox'` projects; `'browser'` code paths are untouched.
2. **Desired-state reconciler.** A row per sandbox project holds `desired` (`running`|`stopped`) and `state`. Page open, a Koda command or a heartbeat set `desired = running`; the idle sweeper sets `desired = stopped`. A worker drives `state` toward `desired`, so rate limits and capacity errors become "retry later", not failures.
3. **Backend-driven, no agent in the sandbox.** The backend calls boat's REST API. There is no `nomi-sandbox-agent` binary and no orchestrator crate inside the sandbox. If REST exec proves too slow in Task 0, revisit with a dial-out agent as a separate plan.
4. **The sandbox is a cache.** Files come from `ProjectStore`; `node_modules` lives only in the sandbox. A lost or expired sandbox is recreated, re-synced and reinstalled, never repaired. No saved snapshots per project (they cost $0.056/day each); at most one shared base template (Task 20).
5. **Heartbeat reuses the existing check-in.** For sandbox projects the page keeps calling `GET /api/projects/:id/runtime`, which marks the project online via `touch_runner` but no longer claims runs (the worker does).
6. **One sandbox per project**, shared by every member who opens it; any member's heartbeat keeps it alive.
7. **Fallback is automatic.** If the sandbox feature is off, over quota, or failing, the page starts the WebContainer runtime instead.
8. **Provider behind a trait.** `SandboxProvider` has a boat implementation and an in-memory fake, so Daytona/E2B/Cloudflare can be swapped in later and every lifecycle test runs without network.

## Review Focus

- Two members open the same sandbox project at once, or the same project twice: one sandbox is created, not two (idempotency key plus row lock).
- boat answers 429 `rate_limited` / 503 `out_of_capacity` during a burst of resumes: the project shows "waiting", is retried after the provider's wait time, and nothing is marked failed.
- The page closes mid-`npm install` or the server restarts mid-sync: a run is never left `running` forever, sync restarts from `synced_version` and converges.
- A resume lands on a new IP and the dev server is gone: the reconciler restarts it and re-exposes the port before reporting `running`.
- Koda queues a command while the sandbox is stopped: the command waits for the start (bounded) instead of returning "project isn't open", and returns a clear message when the start fails.
- A project is deleted (or its owner is) while its sandbox exists: the sandbox is deleted at the provider, and a leaked one is found by the reaper.
- The preview URL carries a secret token: it is returned only to project members, never logged, and a stopped project returns no URL.
- Over-quota or provider-down users keep working on the browser runtime rather than hitting an error page.

---

## Phase 0: Prove the unknowns (go/no-go)

### Task 0: Spike against the real boat API

**Files:**
- Create (throwaway, scratchpad only, not committed): a shell/Node script exercising the API with a throwaway key.
- Create: `docs/superpowers/specs/2026-11-08-sandbox-spike-findings.md`

**Interfaces:**
- Produces: a findings file with a measured value or a yes/no for each line of the checklist below; later tasks read it for their constants (`RESUME_READY_TIMEOUT`, dev-server start method, upload method, iframe/COEP decision).

- [ ] **Step 1: Create a scoped API key with the `ci` preset (`boat api-key create nomi-spike --preset ci`) and a `small` sandbox with `ttlSeconds: 3600`; note the exact create-request fields, including any name/tag/metadata field usable to label a sandbox with its project id.**
- [ ] **Step 2: Measure: create→ready, stop→stopped, resume→ready (3 runs each, with and without `node_modules` on disk). Record seconds.**
- [ ] **Step 3: Push a real SvelteKit project from `ProjectStore` (take the export of a test project) three ways and time each: per-file `PUT /files`; tar.gz via `POST /files/upload-url` then `exec tar -xzf`; `git clone`. Pick one.**
- [ ] **Step 4: Run `npm install` as a detached command, poll `GET /commands/{processId}` with offsets and read the NDJSON `/stream`; record the reconnect behaviour and the 64 KiB/s per-process log cap's effect on a verbose install.**
- [ ] **Step 5: Start the Vite dev server (a) as a detached command and (b) as a systemd user service; stop and resume; record which one is up after resume and how to start it.**
- [ ] **Step 6: Expose port 5173; open the URL in an iframe from a local page served with and without `Cross-Origin-Embedder-Policy: credentialless`. Edit a file in the sandbox and confirm HMR updates the iframe. Record: does the page need the COEP header removed; is the `_token` re-mintable; does `X-Frame-Options`/CSP block framing.**
- [ ] **Step 7: Check what `GET /sandboxes/{id}/usage` returns (billable seconds?) and whether the stop endpoint is `POST .../stop` with a pollable "Get a stop" resource; record exact paths.**
- [ ] **Step 8: Write the findings file ending with a decision: GO, GO with changes (list them), or NO-GO. Delete the spike sandbox and revoke the key (`boat api-key revoke`).**
- [ ] **Step 9: Review gate.** Stop here and show the findings to the user before any repo code is written. If NO-GO or a Daytona/E2B detail looks better, the plan is revised, not continued.

---

## Phase 1: Foundations

### Task 1: `nomi-sandbox` crate: types, trait, errors, fake

**Files:**
- Create: `backend/crates/nomi-sandbox/Cargo.toml`, `src/lib.rs` (trait + types), `src/error.rs`, `src/fake.rs`
- Modify: `backend/Cargo.toml` (workspace member), `backend/crates/nomi-server/Cargo.toml` (dependency)
- Test: `backend/crates/nomi-sandbox/src/fake.rs` (`#[cfg(test)]`) and `tests/fake_provider.rs`

**Interfaces:**
- Produces (`nomi_sandbox`):
  - `struct SandboxId(pub String)`; `enum Size { Small, Default, Large }` with `fn as_api(self) -> &'static str`.
  - `struct CreateSpec { size: Size, ttl_seconds: Option<u64>, from_snapshot: Option<String>, idempotency_key: String, label: String }`; `struct ResumeSpec { ttl_seconds: Option<u64> }`.
  - `enum SandboxState { Starting, Running, Stopping, Stopped, Error, Gone }`; `struct SandboxInfo { id: SandboxId, state: SandboxState }`.
  - `struct ExecOutput { exit_code: Option<i32>, stdout: String, stderr: String }`; `struct ProcessHandle { process_id: String }`; `struct ProcessStatus { running: bool, exit_code: Option<i32>, stdout: String, stderr: String, stdout_next: u64, stderr_next: u64 }`.
  - `struct PreviewUrl { url: String }` (secret: `Debug` prints `[redacted]`).
  - `#[async_trait] trait SandboxProvider: Send + Sync` with `create`, `get`, `resume`, `stop(&self, &SandboxId, force: bool)`, `delete`, `list`, `exec(&self, &SandboxId, command: &str, timeout: Duration)`, `exec_detached`, `process_status(&self, &SandboxId, &ProcessHandle, stdout_from: u64, stderr_from: u64)`, `kill_process`, `write_file(&self, &SandboxId, path: &str, bytes: &[u8])`, `delete_file`, `upload_archive(&self, &SandboxId, dest_dir: &str, tar_gz: Vec<u8>)`, `expose_port(&self, &SandboxId, port: u16)`; all return `Result<_, SandboxError>`.
  - `enum SandboxError { RateLimited { retry_after: Duration }, OutOfCapacity { retry_after: Duration }, BillingRequired, SnapshotExpired, NotFound, Refused(String), Unavailable(String), Other(String) }` with `fn is_retryable(&self) -> bool` and `fn retry_after(&self) -> Option<Duration>`.
  - `FakeProvider` (public, in-memory): records calls, holds files per sandbox, and exposes `fail_next(SandboxError)`, `set_state(&SandboxId, SandboxState)`, `files(&SandboxId) -> BTreeMap<String, Vec<u8>>`, `calls() -> Vec<String>`.

- [ ] **Step 1: Write failing tests in `tests/fake_provider.rs`: `create_then_stop_then_resume_keeps_files`, `detached_process_does_not_survive_resume`, `fail_next_returns_the_queued_error_once`, `retryable_errors_are_exactly_rate_limited_capacity_unavailable`, `preview_url_debug_is_redacted`.**
- [ ] **Step 2: Run `cargo test -p nomi-sandbox`; expect compile failure (crate missing).**
- [ ] **Step 3: Implement the crate as specified; `FakeProvider` mirrors boat's documented behaviour (files persist across stop/resume; processes do not; resume issues a new preview host).**
- [ ] **Step 4: Run `cargo test -p nomi-sandbox`; expect PASS. Run `cargo clippy -p nomi-sandbox -- -D warnings`.**
- [ ] **Step 5: Commit** `feat(sandbox): provider trait, error taxonomy and in-memory fake`.

### Task 2: `BoatProvider` over boat's REST API

**Files:**
- Create: `backend/crates/nomi-sandbox/src/boat.rs`
- Modify: `backend/crates/nomi-sandbox/src/lib.rs` (re-export), `Cargo.toml` (reqwest, wiremock dev-dep)
- Test: `backend/crates/nomi-sandbox/tests/boat_provider.rs`

**Interfaces:**
- Consumes: Task 1 trait/types; exact endpoint paths and bodies from the Task 0 findings file.
- Produces: `struct BoatConfig { base_url: String, api_key: String }`; `struct BoatProvider` with `BoatProvider::new(BoatConfig, reqwest::Client) -> Self` implementing `SandboxProvider`. Error mapping: 429 `rate_limited`/`limit_reached` → `RateLimited` using `nextStartAt`/`retryAfterSeconds`; 503 `out_of_capacity`/`no_ready_machine` → `OutOfCapacity` (120 s / 5 s); 402 → `BillingRequired`; 409 `snapshot_data_expired` → `SnapshotExpired`; 404 → `NotFound`; 5xx otherwise → `Unavailable`.

- [ ] **Step 1: Write failing wiremock tests: `create_sends_bearer_and_idempotency_key`, `rate_limited_maps_retry_after`, `out_of_capacity_maps_120s`, `exec_uses_timeout_seconds_clamped_1_to_600`, `write_file_sends_base64_and_rejects_over_5mib_before_the_request`, `upload_archive_uses_upload_url_then_extracts`, `detached_exec_returns_process_id_and_status_uses_offsets`, `snapshot_expired_maps_409`.**
- [ ] **Step 2: Run them; expect FAIL.**
- [ ] **Step 3: Implement `BoatProvider`; timeouts and the 600 s / 5 MiB limits are named consts; the API key is never part of any error string.**
- [ ] **Step 4: Run `cargo test -p nomi-sandbox`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): boat.dev provider`.

### Task 3: Configuration (admin-managed, with kill switch)

**Files:**
- Modify: `backend/crates/nomi-settings/src/config.rs` (new `SANDBOX` section, mirroring `XENDIT`/`DOMAINS`), `backend/crates/nomi-server/src/configuration.rs`, `backend/crates/nomi-server/src/routes/admin_configuration.rs`
- Test: `backend/crates/nomi-server/tests/admin_configuration_routes.rs` (extend)

**Interfaces:**
- Produces: an `enabled` boolean in the section, default `false`, shown as an on/off switch in Admin → Configuration; `pub async fn sandbox_config() -> Option<SandboxConfig>` in `configuration.rs` returning `None` when `enabled` is false or the key is missing (every caller treats `None` as "use WebContainer"); `struct SandboxConfig { api_base: String, api_key: String, size: Size, idle_stop_secs: u64, rollout: Rollout, starts_per_minute: u32, starts_per_hour: u32, monthly_minutes_per_user: u32 }`; `enum Rollout { Off, Allowlist(Vec<Uuid>), Percent(u8), All }`. Env fallbacks `BOAT_API_KEY`, `BOAT_API_BASE`. The secret field is stored encrypted like other secrets in that section.

- [ ] **Step 1: Failing tests `sandbox_section_saves_encrypts_key_and_hides_it_on_read`, `enabled_defaults_to_false`, `disabled_config_returns_none`, `admin_can_toggle_enabled_and_the_next_read_sees_it`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement the section and `sandbox_config()`; defaults: size `Small`, idle stop 600 s, starts 10/min and 50/hour (headroom under the plan's 12/60), rollout `Off`.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test admin_configuration_routes`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): admin configuration with kill switch`.

### Task 4: Migration and store

**Files:**
- Create: `backend/migrations/0106_project_sandboxes.sql`, `backend/crates/nomi-server/src/sandbox_store.rs`
- Modify: `backend/crates/nomi-server/src/lib.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_store.rs`

**Interfaces:**
- Produces (migration): `ALTER TABLE projects ADD COLUMN runtime TEXT NOT NULL DEFAULT 'browser' CHECK (runtime IN ('browser','sandbox'))`; `CREATE TABLE project_sandboxes (project_id UUID PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE, provider TEXT NOT NULL, sandbox_id TEXT, desired TEXT NOT NULL DEFAULT 'stopped' CHECK (desired IN ('running','stopped')), state TEXT NOT NULL DEFAULT 'none' CHECK (state IN ('none','creating','starting','running','stopping','stopped','error','gone')), synced_version BIGINT NOT NULL DEFAULT 0, preview_url TEXT, retry_at TIMESTAMPTZ, attempts INT NOT NULL DEFAULT 0, last_error TEXT, last_active_at TIMESTAMPTZ, created_at, updated_at)`; `CREATE TABLE sandbox_sessions (id UUID PK, project_id UUID, user_id UUID, size TEXT, started_at TIMESTAMPTZ, stopped_at TIMESTAMPTZ)` for metering; an index on `(desired, state, retry_at)`.
- Produces (store): `async fn want_running(conn, project_id) -> Result<()>`, `want_stopped`, `claim_due(conn, limit: i64) -> Result<Vec<SandboxRow>>` (`FOR UPDATE SKIP LOCKED`, honours `retry_at`), `set_state(conn, project_id, State, Option<&str>)`, `record_attempt_failure(conn, project_id, retry_at)`, `idle_candidates(conn, idle_secs: u64) -> Result<Vec<Uuid>>` (desired running, `runner_seen_at` stale, no pending/running `project_runs`).

- [ ] **Step 1: Failing tests (follow the `#[sqlx::test]` style of `tests/coverage_project_relay.rs`): `want_running_creates_row_once`, `claim_due_skips_rows_locked_by_another_claimer`, `claim_due_honours_retry_at`, `idle_candidates_excludes_projects_with_pending_runs`, `idle_candidates_excludes_recent_runner_seen_at`, `deleting_a_project_cascades_the_row`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Write the migration and store; run `cargo sqlx prepare` if the repo uses offline query data (check for `.sqlx/`).**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_store`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): per-project runtime and sandbox state`.

---

## Phase 2: Lifecycle

### Task 5: Pure planners

**Files:**
- Create: `backend/crates/nomi-sandbox/src/plan.rs`
- Test: same file (`#[cfg(test)]`)

**Interfaces:**
- Produces: `struct StartLimiter` with `new(per_minute: u32, per_hour: u32)` and `fn try_acquire(&mut self, now: Instant) -> Result<(), Duration>` (Err carries the wait); `enum SyncAction { Write(String), Delete(String), Archive }` and `fn plan_sync(known: &BTreeMap<String, String>, manifest: &[ManifestEntry], archive_threshold: usize) -> Vec<SyncAction>` (full `Archive` when `known` is empty or more than `archive_threshold` files changed; otherwise per-file writes/deletes; same changed/removed semantics as `diffManifest` in `frontend/src/lib/projectRuntime.ts`); `fn build_archive(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, ArchiveError>` (tar.gz, paths rejected if absolute or containing `..`); `fn backoff(attempts: u32, base: Duration, cap: Duration) -> Duration`.

- [ ] **Step 1: Failing tests: `limiter_denies_the_eleventh_start_in_a_minute_with_wait`, `limiter_recovers_after_the_window`, `plan_sync_full_archive_when_nothing_known`, `plan_sync_writes_only_changed_and_deletes_removed`, `plan_sync_archives_above_threshold`, `archive_rejects_parent_traversal_and_absolute_paths`, `backoff_is_monotonic_and_capped`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-sandbox`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): start limiter, sync planner, archive builder`.

### Task 6: Reconcile one project

**Files:**
- Create: `backend/crates/nomi-server/src/sandbox_manager.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_manager.rs`

**Interfaces:**
- Consumes: Task 1 `SandboxProvider`/`FakeProvider`, Task 4 store, Task 5 `StartLimiter`/`backoff`.
- Produces: `struct SandboxManager { provider: Arc<dyn SandboxProvider>, limiter: Mutex<StartLimiter>, pool: PgPool, storage: ProjectStore, cfg: SandboxConfig }` and `async fn reconcile(&self, row: SandboxRow) -> ReconcileOutcome` where `enum ReconcileOutcome { Settled, RetryAt(DateTime<Utc>), Failed(String) }`. State walk for `desired = running`: none→creating (create with `idempotency_key = format!("nomi-project-{project_id}-{n}")`) → starting (resume if stopped) → wait ready (`RESUME_READY_TIMEOUT` const from Task 0) → sync (Task 8) → dev server (Task 9) → running. For `desired = stopped`: running→stopping (`stop(force=false)`, retry on refusal) → stopped; stop also closes the open `sandbox_sessions` row.

- [ ] **Step 1: Failing tests with `FakeProvider`: `first_start_creates_syncs_and_runs`, `stopped_sandbox_is_resumed_not_recreated`, `snapshot_expired_recreates_and_resyncs`, `rate_limited_start_schedules_retry_and_does_not_fail`, `stop_refused_is_retried_without_force`, `two_reconciles_of_the_same_row_create_one_sandbox`, `provider_gone_marks_row_none_and_recreates_next_time`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement `reconcile`; every transition writes `set_state` before the provider call that follows so a crash resumes mid-walk; limiter wait becomes `RetryAt`.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_manager`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): desired-state reconciler`.

### Task 7: Worker loop, idle sweeper, heartbeat semantics

**Files:**
- Create: `backend/crates/nomi-server/src/sandbox_worker.rs`
- Modify: `backend/crates/nomi-server/src/main.rs` and `src/bin/` (spawn like `issue_worker`/`scheduler_worker`; read how they are started), `backend/crates/nomi-agent-coding/src/runtime.rs`, `backend/crates/nomi-server/src/routes/project_runtime.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_worker.rs`, `backend/crates/nomi-agent-coding/src/runtime.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `pub async fn run(state: WorkerState, shutdown: CancellationToken)`; `async fn sweep_idle(&self)` sets `want_stopped` for `idle_candidates(cfg.idle_stop_secs)`; `runtime::check_in` gains behaviour: when `projects.runtime = 'sandbox'` it calls `touch_runner` and `want_running`, and returns the version with an empty run list (never claims).

- [ ] **Step 1: Failing tests: `check_in_for_a_sandbox_project_never_claims_runs`, `check_in_marks_desired_running`, `sweep_stops_only_idle_projects`, `worker_reconciles_due_rows_and_skips_not_yet_due`, `worker_survives_a_provider_error_and_continues`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement. Poll interval, batch size and concurrency are named consts; the worker takes `claim_due` rows in batches and reconciles with bounded concurrency.**
- [ ] **Step 4: Run both test targets; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): worker loop and idle stop`.

### Task 8: File sync

**Files:**
- Modify: `backend/crates/nomi-server/src/sandbox_manager.rs`
- Create: `backend/crates/nomi-server/src/sandbox_sync.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_sync.rs`

**Interfaces:**
- Consumes: `plan_sync`, `build_archive`, `ProjectStore` listing (see how `routes::project_runtime::manifest` and `snapshot` read it), `project_file_key`.
- Produces: `async fn sync_project(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<i64, SyncError>` returning the version synced and updating `synced_version`; target directory const `PROJECT_DIR = "/home/user/project"`; files over 5 MiB go through the archive path; a manifest `updated_at` map is kept per sandbox (rebuilt from `synced_version = 0` after a recreate).
- Open item to resolve in this task (do not guess): find how changes made *inside* the browser runtime today (`package.json`/lockfile after `npm install <pkg>`) reach `ProjectStore`, by reading `frontend/src/lib/projectRuntime.svelte.ts` and the files routes; mirror it by reading `package.json` and the lockfile back from the sandbox after any run whose command is `npm install|add|remove|uninstall` and writing them to `ProjectStore` (bumping `files_version`).

- [ ] **Step 1: Failing tests: `initial_sync_uploads_one_archive_with_every_file`, `incremental_sync_writes_only_changed_and_deletes_removed`, `file_over_5mib_goes_via_archive`, `sync_is_idempotent_at_the_same_version`, `files_changed_during_sync_trigger_another_pass`, `npm_install_run_writes_lockfile_back_to_storage`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_sync`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): sync project files into the sandbox`.

### Task 9: Dev server, readiness, preview URL

**Files:**
- Modify: `backend/crates/nomi-server/src/sandbox_manager.rs`
- Create: `backend/crates/nomi-server/src/sandbox_devserver.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_devserver.rs`

**Interfaces:**
- Produces: `async fn ensure_dev_server(&self, sandbox: &SandboxId, stack: Stack) -> Result<PreviewUrl, DevServerError>`: installs when `node_modules` is missing or the lockfile hash changed, starts the dev server using the method chosen in Task 0 (command line per stack lives in a const table, port `DEV_PORT = 5173`), waits for the port (`READY_TIMEOUT` const), exposes it, and stores `preview_url`. Environment passed to the dev server mirrors what the browser runtime passes (`NOMI_PREVIEW=1`, `NODE_NO_WARNINGS=1`, plus the Relay preview variables built by `previewEnv` in `frontend/src/lib/projectRuntime.ts`; port that logic server-side, reusing `project_relays.rs`/`relay_ops.rs` for the values). The generated Vite config needs `allowedHosts` for `.on.boat.dev`: handled by the SvelteKit starter/guides in `nomi-agent-coding/src/guides.rs`, not by editing user files at runtime.
- [ ] **Step 1: Failing tests with `FakeProvider`: `starts_dev_server_after_install_and_exposes_port`, `skips_install_when_lockfile_unchanged`, `restarts_dev_server_after_resume`, `readiness_timeout_marks_row_error_with_log_tail`, `preview_env_includes_relay_variables_for_projects_with_a_backend`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement; the dev server's log tail is kept (bounded by `LOG_KEPT` const) in memory per project for Task 12.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_devserver`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): dev server, readiness and preview url`.

---

## Phase 3: Koda and commands

### Task 10: Run executor for sandbox projects

**Files:**
- Create: `backend/crates/nomi-server/src/sandbox_runs.rs`
- Modify: `backend/crates/nomi-server/src/sandbox_worker.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_runs.rs`

**Interfaces:**
- Consumes: `runtime::claim_runs`, `runtime::finish_run`, `runtime::parse_command` (the allow-list `npm|npx|node` and no shell operators stay as they are).
- Produces: `async fn execute_pending(&self, project_id: Uuid, sandbox: &SandboxId)`: claims `pending` runs, runs each one at a time in `PROJECT_DIR` via `exec` (≤ 600 s) or `exec_detached` + `process_status` polling when the timeout exceeds the exec limit, then `finish_run`. A run claimed by a worker that died is requeued after `RUN_LEASE` const (new column or `started_at` age check; pick the smaller change).

- [ ] **Step 1: Failing tests: `runs_execute_in_order_and_finish_with_exit_code_and_output`, `output_is_tailed_to_output_kept`, `long_command_uses_detached_and_polls_until_exit`, `abandoned_running_run_is_requeued_after_lease`, `command_failure_is_finished_as_failed`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_runs`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): execute Koda's commands in the sandbox`.

### Task 11: Koda's `run_command` for sandbox projects

**Files:**
- Modify: `backend/crates/nomi-agent-coding/src/runtime.rs` (`run_command`, `runner_online`), `backend/crates/nomi-agent-core/src/prompts.rs` (tool/NOT_OPEN wording for sandbox), `backend/crates/nomi-server/src/sandbox_manager.rs` (first-start install+check → `record_check`)
- Test: `backend/crates/nomi-agent-coding/src/runtime.rs` (`#[cfg(test)]`), `backend/crates/nomi-server/tests/sandbox_runs.rs` (extend)

**Interfaces:**
- Produces: for `runtime = 'sandbox'`, `run_command` calls `sandbox_store::want_running`, waits up to `SANDBOX_START_WAIT` const for `state = 'running'`, then queues the run exactly as today; if the wait expires it returns a message (text in `prompts.rs`) saying the sandbox is starting and Koda should finish and complete the task. The install-and-check that the page runs on opening today runs once per `files_version` at first start and calls `record_check`, so a failed check still becomes a new task for Koda.

- [ ] **Step 1: Failing tests: `sandbox_project_wakes_a_stopped_sandbox_and_runs_the_command`, `start_wait_expiry_returns_the_starting_message_not_not_open`, `browser_projects_behave_exactly_as_before`, `failed_first_check_creates_one_fix_request_per_version`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement; keep the browser path byte-for-byte unchanged.**
- [ ] **Step 4: Run `cargo test -p nomi-agent-coding -p nomi-server`; expect PASS.**
- [ ] **Step 5: Commit** `feat(coding): Koda runs commands in the project's sandbox`.

---

## Phase 4: API and frontend

### Task 12: Sandbox API and realtime

**Files:**
- Create: `backend/crates/nomi-server/src/routes/project_sandbox.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs`, `src/app.rs` (routes), `src/project_events.rs`, `backend/crates/nomi-i18n/locales/en.yml` and `id.yml`
- Test: `backend/crates/nomi-server/tests/project_sandbox_routes.rs`

**Interfaces:**
- Produces: `GET /api/projects/:id/sandbox` → `{ runtime, available: bool, state, phase, preview_url: Option<String>, error: Option<String> }` (members only; `preview_url` only when `state = running`); `POST /api/projects/:id/sandbox/start` and `/stop` (owner/members, set desired); `GET /api/projects/:id/sandbox/logs?offset=N` → `{ text, next_offset }`; `project_events::sandbox_changed(project_id, phase)` publishing `{"kind":"SandboxChanged","phase":...}` on the existing `project/{id}/stream` topic, called on every `set_state`. `available` is false when the config is `None`, rollout excludes the user, the user is over quota (Task 15) or the provider errored recently.

- [ ] **Step 1: Failing tests: `non_member_gets_404`, `preview_url_absent_unless_running`, `start_sets_desired_running_and_returns_phase`, `available_false_when_kill_switch_off`, `sandbox_project_reports_webcontainer_when_switch_off`, `start_is_refused_when_switch_off`, `logs_endpoint_pages_by_offset`, `phase_changes_publish_a_sandbox_changed_event` (use the existing publisher test seam).**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement; new projects choose `runtime` from `Rollout` at creation (find the create route in `routes/projects.rs`); existing projects stay `'browser'`.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test project_sandbox_routes`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): sandbox status API and realtime events`.

### Task 13: Frontend runtime for sandbox projects

**Files:**
- Create: `frontend/src/lib/sandboxRuntime.svelte.ts`, `frontend/src/lib/sandboxRuntime.svelte.test.ts`
- Modify: `frontend/src/lib/components/ProjectPreview.svelte`, `frontend/src/routes/(app)/projects/session/[sessionId]/+page.svelte`, `frontend/src/hooks.server.ts`, `frontend/project.inlang` messages (en + id: `project_phase_sandbox_waking`, `project_phase_sandbox_syncing`, `project_phase_sandbox_starting`, `project_sandbox_unavailable_fallback`)
- Test: `frontend/src/lib/sandboxRuntime.svelte.test.ts`, `frontend/src/lib/components/assorted2.svelte.test.ts` (extend)

**Interfaces:**
- Produces: `class SandboxRuntime` with the same public surface the page uses on `ProjectRuntime` (`phase: RuntimePhase`, `previewUrl: string | null`, `log: string`, `error: string | null`, `files`, `start()`, `stop()`, `restart()`, `writeFile()`/`removeFile()` as no-ops because the backend already has the file); `RuntimePhase` gains `'waking' | 'syncing'`; the heartbeat is the existing `GET /api/projects/:id/runtime`; phase changes come from `SandboxChanged` on `project/{id}/stream`, with a slow poll of `GET /sandbox` as backstop; logs from `/sandbox/logs?offset`. `createRuntime(projectId)` picks `SandboxRuntime` when `/sandbox` says `runtime = 'sandbox' && available`, else `ProjectRuntime`. `hooks.server.ts` stops setting COEP/COOP for sandbox projects only if Task 0 showed COEP blocks the iframe (otherwise unchanged).
- Do not touch `frontend-new/`; wrap existing design-system components rather than changing their props.

- [ ] **Step 1: Failing tests: `picks_sandbox_runtime_when_available`, `falls_back_to_browser_runtime_when_unavailable`, `maps_backend_phases_to_runtime_phases`, `heartbeat_continues_while_open_and_stops_on_stop`, `writeFile_and_removeFile_do_not_call_the_api`, `preview_url_is_not_logged_or_kept_in_the_dom_text`.**
- [ ] **Step 2: Run `pnpm --dir frontend vitest run src/lib/sandboxRuntime.svelte.test.ts`; expect FAIL.**
- [ ] **Step 3: Implement; reuse `previewPhase.ts`'s `PHASE_LABEL` pattern for the new labels.**
- [ ] **Step 4: Run `pnpm --dir frontend vitest run` and `pnpm --dir frontend check`; expect PASS.**
- [ ] **Step 5: Commit** `feat(projects): run sandbox projects through the backend sandbox`.

### Task 14: Preview UX states and manual verification

**Files:**
- Modify: `frontend/src/lib/components/project/ProjectWaitScreen.svelte`, `frontend/src/lib/components/ProjectPreview.svelte`
- Test: `frontend/src/lib/components/assorted2.svelte.test.ts` (extend)

**Interfaces:**
- Consumes: Task 13 phases.
- Produces: user-visible states for waking (with elapsed time), syncing, installing, starting, running, waiting for capacity ("busy, retrying"), failed (log tail and a Retry button that calls `/sandbox/start`), and a one-click "Run in my browser instead" that forces the WebContainer runtime for this page session.

- [ ] **Step 1: Failing component tests for each state's label and for the Retry and fallback buttons.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement with Paraglide messages (en + id).**
- [ ] **Step 4: Start the dev server (`preview_start`) with a fake provider enabled by env, open a sandbox project, verify each state, a stop/resume cycle and an edit-to-HMR round trip in the preview; check the console and network for errors and that no request URL contains the token.**
- [ ] **Step 5: Commit** `feat(projects): sandbox preview states and browser fallback`.

---

## Phase 5: Production hardening

### Task 15: Quota and metering

**Files:**
- Create: `backend/crates/nomi-server/src/sandbox_usage.rs`
- Modify: `backend/crates/nomi-server/src/sandbox_manager.rs`, `routes/admin_usage.rs` or the closest admin-usage route, `backend/crates/nomi-usage/src/plans.rs` (only if plan-level allowance is wanted; otherwise config-level `monthly_minutes_per_user`)
- Test: `backend/crates/nomi-server/tests/sandbox_usage.rs`

**Interfaces:**
- Produces: `async fn minutes_this_month(conn, user_id) -> i64` summing `sandbox_sessions` (open sessions counted to now); `async fn within_allowance(conn, user_id, cfg) -> bool` used by `available` in Task 12; a nightly reconcile compares our total to the provider's per-sandbox usage and logs the drift (no billing impact in v1).

- [ ] **Step 1: Failing tests: `open_session_counts_up_to_now`, `over_allowance_makes_sandbox_unavailable_but_running_sandbox_is_not_killed`, `admin_usage_lists_minutes_per_user`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_usage`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): usage metering and monthly allowance`.

### Task 16: Resilience

**Files:**
- Modify: `backend/crates/nomi-server/src/sandbox_manager.rs`, `sandbox_worker.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_resilience.rs`

**Interfaces:**
- Produces: a watchdog that moves any row stuck in `creating|starting|stopping` longer than `STUCK_AFTER` const back to a retryable state; after `MAX_ATTEMPTS` consecutive failures the row becomes `error` and the page falls back (Task 12 `available=false` for `PROVIDER_COOLDOWN`); a daily job that handles sandboxes stopped longer than `EXPIRY_WARN` const (10 days) by marking them `gone` proactively and deleting the provider copy, since the next start rebuilds from `ProjectStore`.

- [ ] **Step 1: Failing tests with `FakeProvider::fail_next`: `stuck_creating_is_retried`, `five_failures_mark_error_and_set_cooldown`, `provider_outage_does_not_hot_loop` (assert call count under a fixed fake clock), `long_stopped_sandbox_is_deleted_and_next_start_rebuilds`, `server_restart_mid_sync_converges`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_resilience`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): watchdog, cooldown and expiry handling`.

### Task 17: Deletion and reaper

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/projects.rs` (delete path), `sandbox_worker.rs`
- Test: `backend/crates/nomi-server/tests/sandbox_cleanup.rs`

**Interfaces:**
- Produces: deleting a project (and an account purge) deletes its sandbox at the provider before the row cascades (the DB row is removed only after success, otherwise the project row is soft-held for retry); `reap_orphans()` lists the provider's sandboxes and deletes any whose label `nomi-project-<uuid>` has no `project_sandboxes` row, with a dry-run flag defaulting on for the first week (config).

- [ ] **Step 1: Failing tests: `deleting_a_project_deletes_its_sandbox`, `provider_failure_on_delete_keeps_a_retry_record`, `reaper_deletes_only_unreferenced_nomi_labelled_sandboxes`, `reaper_dry_run_deletes_nothing`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run `cargo test -p nomi-server --test sandbox_cleanup`; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): delete sandboxes with their projects and reap orphans`.

### Task 18: Security review

**Files:**
- Modify: whatever the review finds; add `docs/sandbox-runtime.md` section "Security model"

- [ ] **Step 1: Audit and fix, with a test for each: API key absent from logs/errors/`Debug`; preview URL returned only to members and absent from logs and from `SandboxChanged` events; sandbox environment contains no Nomi secrets, JWT secret, DB URL or LLM keys; command allow-list and `parse_command` still enforced server-side (not just by the tool schema); archive extraction cannot escape `PROJECT_DIR`; outbound firewall decision recorded (default open for npm; document the choice and whether boat's firewall rules should block internal ranges).**
- [ ] **Step 2: Run the repo's security review (`/security-review`) on the branch and resolve findings.**
- [ ] **Step 3: Commit** `fix(sandbox): security review findings`.

### Task 19: Observability and admin controls

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/admin_dashboard.rs`, `sandbox_manager.rs`, `sandbox_worker.rs`
- Test: `backend/crates/nomi-server/tests/admin_dashboard_routes.rs` (extend)

**Interfaces:**
- Produces: structured `tracing` fields on every transition (`project_id`, `sandbox_id`, `from`, `to`, `elapsed_ms`, `provider_error_kind`; never the key or URL); admin dashboard counts of `running`, `starting`, `error`, rate-limited retries in the last hour, and minutes this month; the Task 3 `enabled` toggle stops new starts immediately and lets running sandboxes idle out.

- [ ] **Step 1: Failing tests: `dashboard_reports_sandbox_counts`, `kill_switch_stops_new_starts_but_not_running_ones`.**
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run the extended tests; expect PASS.**
- [ ] **Step 5: Commit** `feat(sandbox): observability and admin kill switch`.

### Task 20 (optional optimisation): shared base template

**Files:**
- Modify: `backend/crates/nomi-server/src/sandbox_manager.rs`, `docs/sandbox-runtime.md`

- [ ] **Step 1: Only if Task 0 measured a first-start install above the target (set by the user). Build one saved snapshot (`nomi-base-sveltekit`) with Node and the starter's dependencies preinstalled, via a one-off admin action; `CreateSpec.from_snapshot` uses it; refresh it monthly.**
- [ ] **Step 2: Failing test `first_start_forks_from_base_template_when_configured`, then implement; the plan's snapshot allowance (10 on the $20 plan) is the cap.**
- [ ] **Step 3: Commit** `feat(sandbox): start new projects from a prepared template`.

### Task 21: Load, soak and live end-to-end

**Files:**
- Create: `backend/crates/nomi-server/tests/sandbox_load.rs` (FakeProvider), `docs/sandbox-runtime.md` section "Test results"

- [ ] **Step 1: Test `burst_of_200_wakes_respects_start_limits_and_all_settle` with the fake provider and a fake clock: peak starts per minute never exceed the configured limit, every project ends `running`.**
- [ ] **Step 2: Live run against the real provider with a throwaway key and a small `ttlSeconds`: with the three seed accounts, create a sandbox project, let Koda build a SvelteKit page, stop, resume, edit, confirm HMR; record timings and cost. Then delete test sandboxes, projects and the key (keep the three seed accounts).**
- [ ] **Step 3: 24-hour soak with 10 sandboxes on a cycle of use/idle; confirm no leaked sandboxes, no stuck rows, usage matches the provider's within the agreed drift.**
- [ ] **Step 4: Record results in the doc. Commit** `test(sandbox): load and live verification notes`.

### Task 22: Rollout and runbook

**Files:**
- Create/finish: `docs/sandbox-runtime.md` (architecture, config reference, runbook: provider outage, rate limits, stuck rows, orphaned sandboxes, key rotation, cost spike, how to kill-switch; rollback)
- Modify: nothing else; rollout is configuration.

- [ ] **Step 1: Production-readiness checklist, each item ticked with evidence in the doc: all tests green (`cargo test --workspace`, `pnpm --dir frontend vitest run`, `pnpm --dir frontend check`); migration applied on a staging copy; key rotated into Admin → Configuration; provider plan limits above configured starts/min and starts/hour; alerts or a daily check on the dashboard counters; reaper dry-run reviewed then enabled; rollback rehearsed (kill switch → pages fall back to WebContainer).**
- [ ] **Step 2: Staged rollout through `Rollout`: `Off` → `Allowlist` (the team) for one week → `Percent(10)` → `Percent(50)` → `All` for new projects; watch error rate, resume latency p95, minutes per user and spend against the plan. Existing projects stay on the browser runtime until the user opts in.**
- [ ] **Step 3: Review with the user after the allowlist week and decide on `Percent`; do not advance automatically.**

---

## Self-review notes

- **Spec coverage:** reach the sandbox (Tasks 1–2, 6, 10); orchestrator stays on the server, sandbox holds no credentials (Design 3, Task 18); dynamic start/stop (Tasks 4, 6, 7, 16); preview and hot reload (Tasks 0, 9, 12–14); cost and quota (Tasks 3, 15, 19); production readiness (Tasks 16–22).
- **Type consistency:** `SandboxProvider`, `SandboxId`, `SandboxState`, `SandboxConfig`, `Rollout`, `StartLimiter`, `plan_sync` and `sandbox_store` function names are the ones defined in Tasks 1, 3, 4, 5 and used unchanged afterwards.
- **Open items deliberately not decided here:** the dev-server start method, upload method and iframe/COEP handling (decided by Task 0 findings); how in-sandbox `package.json`/lockfile changes are written back (resolved in Task 8 by reading the existing browser code); whether a second provider is worth building (decided after the Task 0 findings).
- **Known cost of the plan:** about 22 tasks; Tasks 6, 8, 9 and 13 are the large ones. Task 20 is optional.
