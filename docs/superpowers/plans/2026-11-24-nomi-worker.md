# nomi-worker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move every background worker loop out of `nomi-server` into a `nomi-worker` crate, started by one `spawn_all`, with both binaries in a thin `nomi-app` crate.

**Architecture:**
- `nomi-worker` (lib) depends on `nomi-server` (now library-only) and holds the loops plus `spawn_all`.
- `nomi-app` holds the `nomi-orchestrator` and `worker` binaries.
- `nomi-server` never depends on `nomi-worker`, not even as a dev-dependency.

**Tech Stack:** Rust workspace (`backend/crates/*`), tokio, sqlx 0.7, cargo-llvm-cov floors.

**Spec:** `docs/superpowers/specs/2026-11-24-nomi-worker-design.md`

## Global Constraints

- Loop behaviour unchanged: same queries, intervals, MQTT envelopes, log lines.
- Binary names stay `nomi-orchestrator` and `worker`; `RUN_WORKER_INLINE` keeps its meaning.
- `nomi-server` gets no dependency on `nomi-worker`, including dev-dependencies.
- No AI attribution in commits or the PR. Commits use Conventional Commits.
- Every numeric literal gets a named const. Prompts stay in `nomi-agent-core/src/prompts.rs`.
- Build with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`. Tests need
  `DATABASE_URL=postgres://nomi:nomi@localhost:55432/nomi` and an MQTT broker on 1883.

## Review Focus

1. **Standalone `worker` binary:** it now runs all 10 loops. The ones it gains (delegation, scheduler, crew reminders, billing) need the setup they had in the server binary. Expect each loop to start without a missing-init panic. Test in Task 2.
2. **Inline mode:** with `RUN_WORKER_INLINE` unset, the server still starts every loop, each with its own MQTT client id, as today. Test in Task 2.
3. **`RUN_WORKER_INLINE=false`:** the server starts no loops. Test in Task 3.
4. **Server routes that used worker items:** `sandbox_check_in` and `PROVIDER_NAME` behave the same after moving inside the server. Existing route tests in Task 1.
5. **Coverage:** `nomi-server` and `nomi-worker` stay at or above their floors after the move. Checked in Task 3.

---

### Task 1: Create nomi-worker and move the loops

**Files:**
- Create `backend/crates/nomi-worker/{Cargo.toml,src/lib.rs}`.
- `git mv` from `nomi-server/src` into `nomi-worker/src`:
  - `worker.rs`, `delegation_worker.rs`, `scheduler_worker.rs`, `reminders_worker.rs`;
  - `memory_worker.rs`, `issue_worker.rs`, `billing_worker.rs`, `attachment_worker.rs`;
  - `background_worker.rs`, `sandbox_worker.rs`, `sandbox_cleanup.rs`.
- Modify `nomi-server/src/lib.rs`: drop those `mod` lines; make public what the moved code uses.
- Create `backend/crates/nomi-app/{Cargo.toml,src/bin/nomi-orchestrator.rs,src/bin/worker.rs}`, `git mv`'d from `nomi-server/src/main.rs` and `src/bin/worker.rs`, calling `nomi_worker::…`. Drop `[[bin]]` from `nomi-server/Cargo.toml`. The binaries must leave now: once they call `nomi-worker`, keeping them in `nomi-server` would be a dependency cycle.
- Move `PROVIDER_NAME` and `sandbox_check_in` (and anything `sandbox_manager` / routes use from `sandbox_cleanup`) into the server's `sandbox_manager.rs` or `sandbox_ops.rs`.
- `git mv` the worker tests from `nomi-server/tests` to `nomi-worker/tests`. Those are the files that reference a moved module:
  - `account_settings_routes`, `attachments_routes`, `billing_routes`;
  - `coverage_worker_loops`, `coverage_workers`, `issue_worker`;
  - `sandbox_cleanup`, `sandbox_load`, `sandbox_resilience`, `sandbox_usage`, `sandbox_worker`;
  - `scheduler_worker`.
  - If a moved test file also holds pure route tests, keep the file whole in `nomi-worker`; don't split it.

**Interfaces:**
- Produces `nomi_worker::{worker, delegation_worker, scheduler_worker, reminders_worker, memory_worker, issue_worker, billing_worker, attachment_worker, background_worker, sandbox_worker, sandbox_cleanup}`, each `run` with today's signature.
- `nomi_server::sandbox_manager::PROVIDER_NAME` (or `sandbox_ops`) and the server-side `sandbox_check_in`.

- [ ] **Step 1:** Move the tests first. Rewrite `nomi_server::<loop module>` to `nomi_worker::<loop module>` in them. Add `nomi-worker` dev-deps by copying `nomi-server`'s dev-deps that the moved tests use.
- [ ] **Step 2:** Run `cargo test -p nomi-worker --no-run`. Expected: FAIL (the crate has no modules yet).
- [ ] **Step 3:** Move the modules and rewrite paths:
  - `crate::<server item>` → `nomi_server::<server item>`;
  - `crate::<moved module>` stays `crate::`.
  - Make the server items they reach `pub`, as few as possible.
  - Move both binaries into `nomi-app` and point their calls at `nomi_worker::…`. `sqlx::migrate!("../../migrations")` still resolves from `crates/nomi-app`.
- [ ] **Step 4:** Run `cargo build --workspace && cargo build --bin nomi-orchestrator --bin worker && cargo test -p nomi-worker && cargo test -p nomi-server`. Expected: PASS.
- [ ] **Step 5:** Commit `refactor: worker loops into nomi-worker, binaries into nomi-app`.

### Task 2: One spawn_all for both binaries

**Files:** `nomi-worker/src/spawn.rs` (new); `nomi-worker/tests/spawn_all.rs` (new).

**Interfaces:**
```rust
pub struct WorkerContext {
    pub pool: PgPool,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    /// "nomi-orchestrator" inline, "nomi-worker" standalone: each loop's MQTT client id is
    /// "<prefix>-<loop>-<uuid>".
    pub client_id_prefix: &'static str,
    pub s3: Option<nomi_storage::S3Config>,
    pub settings_key: [u8; 32],
    pub http_client: reqwest::Client,
    pub database_url: String,
    pub project_storage: nomi_storage::ProjectStore,
}
/// Every loop, by name, ready to spawn: built without polling anything.
pub fn loops(ctx: WorkerContext) -> Vec<(&'static str, BoxFuture<'static, ()>)>;
/// Spawns every loop and returns their names, in order.
pub fn spawn_all(ctx: WorkerContext) -> Vec<&'static str>;
pub const LOOPS: [&str; 10] = ["turns", "delegation", "background", "scheduler", "reminders", "memory", "issues", "billing", "sandboxes", "attachments"];
```
The order matches today's `main.rs`. The scheduler gets `Arc::new(LogOnlyDelivery)` as now, and the sandbox loop gets `ShipyardConfig::from_env()`.

- [ ] **Step 1: Tests.**
  - `every_loop_is_spawned`: `loops(ctx)` names equal `LOOPS`. Build the ctx with `PgPoolOptions::connect_lazy` and MQTT on `localhost`, and never poll the futures.
  - `every_worker_module_is_a_loop`: every `nomi-worker/src/*.rs` that defines `pub async fn run(` maps to one `LOOPS` entry. Keep the name map in the test.
- [ ] **Step 2:** Run. Expected: FAIL (`loops` missing).
- [ ] **Step 3:** Implement `spawn.rs`.
  - Replace the inline block in `main.rs` with `nomi_worker::spawn_all(ctx)`, keeping the `RUN_WORKER_INLINE` check and its log lines.
  - Replace the six spawns plus the `worker::run` call in `bin/worker.rs` with `spawn_all`, then wait forever (`std::future::pending::<()>().await`).
  - The standalone binary also calls `nomi_server::project_events::init`, which `issue_worker` publishes through, as the server binary does.
- [ ] **Step 4:** Run `cargo test -p nomi-worker --test spawn_all && cargo build --workspace`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(worker): one spawn_all starts every loop; the standalone worker runs all of them`.

### Task 3: Inline switch, verification, PR

**Files:**
- `nomi-app/src/lib.rs`.
- Update the Dockerfile comment pointing at `crates/nomi-server/src/main.rs`.
- `backend/coverage-floors.json`.

- [ ] **Step 1: Test** `inline_off_starts_no_loops`: `nomi_app::worker_inline(Some("false"))` is `false`; for `None` and `Some("true")` it is `true`. Extract that one-line check into `nomi-app/src/lib.rs`, used by the orchestrator binary.
- [ ] **Step 2:** Run. Expected: FAIL.
- [ ] **Step 3:** Add the lib function and use it in the orchestrator binary.
- [ ] **Step 4:** Run:
  - `cargo build --bin nomi-orchestrator --bin worker && cargo test --workspace`
  - `cargo llvm-cov --workspace --no-report`, then the floor check.
  - Add a `nomi-worker` floor one point under what it reaches, and a `nomi-app` one if it falls under the default. If `nomi-server` drops under its floor, record a ruling, because its share of loop code moved out.
  - Expected: PASS.
- [ ] **Step 5:** Commit `test(app): pin the RUN_WORKER_INLINE switch; coverage floors for the new crates`.
- [ ] **Step 6:** Push `refactor/nomi-worker` and open a PR against main.
