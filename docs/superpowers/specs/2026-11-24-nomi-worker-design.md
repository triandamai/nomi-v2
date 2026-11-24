# nomi-worker — design

Date: 2026-11-24
Scope: project 2 of 3 (worker → its own crate). Project 3 (custom agents with skills.md) follows.

## Intent

The background loops live in their own crate, so `nomi-server` is about serving HTTP and the code
is easier to find. Organization is the goal; build speed and deploying separately are not.

## Goals

1. A `nomi-worker` crate holds every background worker loop now in `nomi-server`.
2. One `nomi_worker::spawn_all(...)` starts them, used by both binaries. The standalone worker
   then runs every loop. Today it skips delegation, scheduler, crew reminders and billing.
3. Behaviour of each loop is unchanged.

## Non-goals

- Moving shared services (registry assembly, push, sandbox code, billing) out of `nomi-server`
  (approach B, possible later).
- The server-side loops tied to HTTP features: presence, Shipyard catch-up and webhook, relay
  sleep. They stay in `nomi-server`.

## Approaches

- **A. Workers on top of the server library; binaries in a thin crate (chosen).** `nomi-worker`
  depends on `nomi-server`'s lib; `nomi-app` holds both binaries. Mostly moves; low risk.
  Trade-off: `nomi-worker` still builds all of `nomi-server`.
- **B. Full untangle.** Shared pieces move down so the server depends on the workers. Cleanest,
  about three times the work, mostly the sandbox family.

**Chosen: A** (user's pick, 2026-11-24).

## Design

- **`nomi-worker`** (new lib) gets these modules from `nomi-server/src`, unchanged:
  - `worker` (turns), `delegation_worker`, `scheduler_worker`, `reminders_worker`;
  - `memory_worker`, `issue_worker`, `billing_worker`, `attachment_worker`, `background_worker`;
  - `sandbox_worker`, `sandbox_cleanup`.
  - `crate::…` paths into the server become `nomi_server::…`. Anything they use that is
    private becomes `pub` in `nomi-server`.
- **Server code that uses a worker module** keeps what it needs inside the server:
  - `sandbox_worker::PROVIDER_NAME` and `sandbox_check_in` move into the server's sandbox modules.
  - The `sandbox_cleanup` pieces that `sandbox_manager` uses (if any) move the same way.
  - The worker modules import them from there.
- **`spawn_all`.** `nomi_worker::spawn_all(ctx: WorkerContext)` spawns every loop, as `main.rs`
  does inline today, in the same order.
  - `WorkerContext` holds: pool, MQTT publisher, S3, settings key, HTTP client, database URL,
    project storage, Shipyard config.
- **`nomi-app`** (new, binaries only) holds `nomi-orchestrator` (today's `main.rs`) and
  `worker` (today's `bin/worker.rs`).
  - Both call `spawn_all`; the server only when `RUN_WORKER_INLINE` isn't false, as now.
  - The worker binary keeps its own setup (migrations, settings, storage, mailer, push…).
  - Binary names stay, so the Dockerfile, CI and `scripts/dev.sh` (`--bin nomi-orchestrator`,
    `--bin worker`) keep working.
- **`nomi-server`** becomes library-only: no `[[bin]]`, no `main.rs`.
- **Tests.** The worker tests under `nomi-server/tests` move to `nomi-worker/tests`.
  - That includes route tests that run a worker pass.
  - They use `nomi_server::…` for everything else.
  - `nomi-server` never depends on `nomi-worker`, not even as a dev-dependency, which would
    build two copies of `nomi-server`.

## Testing

- Moved tests pass unchanged apart from paths.
- New: a test that `spawn_all` starts every loop, so the standalone worker can't drift again.
  It checks a list of the loops it spawned against the worker modules.
- `cargo build --workspace`, `cargo test --workspace`, coverage floors. `nomi-worker` gets a
  floor one point under what it reaches; `nomi-server`'s floor is re-checked after the move.

## Left to the builder

- `WorkerContext`'s exact shape, and how `spawn_all` reports what it started.
- Which private server items need `pub`: as few as possible.
