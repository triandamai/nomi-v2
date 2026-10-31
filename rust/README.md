# Nomi Rust SDK

The client engine shared by the Android, iOS and desktop apps under `native/`. Rust owns networking,
local data, sync, domain logic and events; Kotlin keeps Compose UI, ViewModels, navigation, Koin and
platform glue, and reaches Rust only through the generated UniFFI API.

This is a Cargo workspace of its own. It doesn't share crates with `backend/`, and nothing here
touches the backend.

```
rust/
├── Cargo.toml                    workspace, pinned dependency versions
└── crates/nomi-sdk/
    ├── kotlin-bindings.toml      gobley-uniffi-bindgen settings (package ai.nomi.sdk)
    └── src/
        ├── lib.rs                exported free functions, module wiring
        ├── client.rs             NomiSdk: initialize (new) → start → shutdown, Diagnostics
        ├── config.rs             SdkConfig, LogLevel, validation
        ├── error.rs              NomiError (NomiException in Kotlin)
        ├── event.rs              SdkEvent, SdkState, EventBus, EventSubscription
        ├── logging.rs            tracing → SdkEvent::Log
        └── storage/
            ├── mod.rs            Accounts: one database per account, open/switch/remove
            ├── database.rs       connection, transactions, per-table change counters
            ├── migrations.rs     ordered schema migrations (PRAGMA user_version)
            ├── models.rs         Session, Message, Draft, NotificationItem, AgentActivity, …
            ├── repositories/     SQL per table
            ├── observation.rs    snapshot-plus-change streams
            └── store.rs          AccountStore: the API Kotlin reads and writes through
native/shared/sdk/                Kotlin module: generated bindings, NomiClient / NativeNomiClient,
                                  AccountStore flows
```

## Bindings

UniFFI's own Kotlin generator targets the JVM only, but the apps share Kotlin across Android, iOS
and desktop. [gobley-uniffi-bindgen](https://github.com/gobley/gobley) generates Kotlin
Multiplatform bindings instead: JNA on Android and the desktop, cinterop on Kotlin/Native (iOS).
gobley 0.3.7 supports UniFFI 0.29.4, so `uniffi` is pinned to `=0.29.4`. Upgrade the two together.

Only the generator is used. gobley's Gradle plugins expect `androidTarget()` from the
`com.android.library` plugin, and this project uses AGP 9's `com.android.kotlin.multiplatform.library`.
So `native/shared/sdk/build.gradle.kts` runs cargo and the generator itself:

| Task | Does |
| --- | --- |
| `installBindgen` | `cargo install gobley-uniffi-bindgen@0.3.7` into `rust/target/tools` |
| `cargoBuildHost` | builds the cdylib for this machine |
| `generateUniffiBindings` | generates bindings from that library into `build/generated/uniffi/bindings` |
| `stageDesktopLibrary` | copies the library to `<jna-prefix>/` on the desktop classpath |
| `cargoBuildIosArm64`, `cargoBuildIosSimulatorArm64` | build the staticlib that cinterop embeds |

Add `-Pnomi.rust.release=true` for an optimized build. The default is a debug build.

## Working on it

```bash
cd rust && cargo test && cargo clippy --all-targets -- -D warnings
```

```bash
cd native && ./gradlew :shared:sdk:desktopTest :shared:sdk:iosSimulatorArm64Test
```

The iOS tasks need `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.

API rules for the FFI boundary:

- Export stable SDK models, never internal types. The engine validates `SdkConfig` into its own
  `ValidatedConfig`.
- Every engine call that will do I/O is `async`. Futures exported to Kotlin don't depend on a tokio
  context. They wait on channels, or on tasks spawned onto the SDK's own runtime, so Kotlin's executor
  can poll them.
- Error fields are named `reason`, because Kotlin's `Throwable` already has `message`. No method is
  named `close`, because generated objects already have `close()` (`AutoCloseable`), which frees the
  native handle.
- Lists cross the boundary as pages or snapshots, never one call per item.

## Milestones

**1 — Native SDK (done).** Kotlin creates the engine, starts it, collects typed events as a `Flow`,
and shuts it down. This is verified by 22 Rust tests, plus 6 Kotlin tests that run against the real
library on the desktop JVM and on the iOS simulator. Lifecycle rules:

- `ShutDown` is terminal.
- `start` and `shutdown` are idempotent.
- Event streams drain, then end on shutdown.
- A slow subscriber gets `EventsDropped { count }` instead of blocking the engine.
- Dropping a running SDK doesn't panic, even inside an async context.

Still open from Milestone 1:

- **Android packaging.** No `.so` is bundled yet: that needs cargo-ndk and the Android Rust targets,
  which is Phase 6 work. Android host tests compile but skip the native calls.
- **App wiring.** `:shared:sdk` isn't wired into the app or Koin yet. That happens in Milestone 5,
  when screens move over.

**2 — Local engine (done).** Rust owns the local data the apps kept in Room: sessions, messages,
streaming drafts, notifications and agent activity. This is verified by 56 Rust tests, plus 13
Kotlin tests on the desktop JVM and the iOS simulator.

- **Storage:** SQLite (rusqlite, bundled) in WAL mode, with ordered migrations. A database from a
  newer SDK is refused, and a failed migration rolls back.
- **Accounts:** each account gets its own database at `<data_dir>/accounts/<account_id>/nomi.db`.
  Only one is open at a time. Opening another closes the previous `AccountStore`, so its calls fail
  with `AccountClosed` and its observations end. `remove_account` (sign-out) and `reset_storage`
  delete files.
- **Writes:** every write is one immediate transaction. Observers hear about it only after it
  commits, and only for the tables it touched.
- **Observations:** a snapshot first, then a new snapshot after each relevant commit. Unchanged
  answers are skipped and bursts coalesce. Nothing polls.
- **Kotlin:** `AccountStore.sessions(...)`, `messages(...)`, `draft(...)` and the other observations
  are `Flow`s, like Room's.
- **Paging:** messages page with a `(created_at, id)` cursor. Every query is capped at 500 rows.

The write methods (`upsert_*`, `append_draft`, …) stand in for sync until Milestones 3 and 4. They
let the current Kotlin repositories store server data through Rust instead of Room.

**3 — Connected engine.** reqwest and tokio-tungstenite against the existing REST and realtime
contracts (`native/shared/network`, `native/shared/realtime`, `backend/crates/nomi-realtime`), with
token refresh, reconnect and heartbeat.

**4 — Reliable engine.** Durable outbox with idempotency keys, cursors, deduplication, retry with
backoff, and replay.

**5 — Full migration.** Screens move to `NomiClient`, then Room, Ktor Client and the Kotlin
repositories are removed.

**6 — Production ready.** Android, desktop Linux and Windows artifacts, plus recovery and integration
tests on every platform.
