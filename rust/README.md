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
        ├── services/             the account pages: crew, preferences, home, money, reminders,
        │                         memory, connections, usage, projects (online, typed)
        ├── network/
        │   ├── mod.rs            Network: the HTTP client, the session, the realtime hub
        │   ├── tls.rs            rustls on ring + Mozilla roots, shared by HTTP and the socket
        │   ├── http.rs           pooled reqwest client, timeouts, error mapping
        │   ├── auth.rs           CredentialStore (host callback), sign-in, refresh, sign-out
        │   ├── events.rs         RealtimeEvent: realtime payloads decoded into typed events
        │   └── realtime.rs       the realtime socket: topics, ping, watchdog, backoff, re-auth
        └── storage/
            ├── mod.rs            Accounts: one database per account, open/switch/remove
            ├── database.rs       connection, transactions, per-table change counters
            ├── migrations.rs     ordered schema migrations (PRAGMA user_version)
            ├── models.rs         Session, Message, Draft, NotificationItem, AgentActivity, …
            ├── repositories/     SQL per table
            ├── observation.rs    snapshot-plus-change streams
            └── store.rs          AccountStore: the API Kotlin reads and writes through
        └── sync/
            ├── mod.rs            SyncEngine: refreshes, paging cursors, sending, ChatBinding
            ├── api.rs            the chat and inbox endpoints, wire shapes, mapping to models
            ├── realtime.rs       realtime events applied to storage, one ordered lane per topic
            └── outbox.rs         the outbox worker: delivery, retry, echo check, replay
native/shared/sdk/                Kotlin module: generated bindings, NomiClient / NativeNomiClient,
                                  AccountStore flows, topic flows
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
| `installCargoNdk` | `cargo install cargo-ndk@4.1.2` into `rust/target/tools` |
| `cargoBuildAndroid` | builds `libnomi_sdk.so` for arm64-v8a, armeabi-v7a and x86_64 into `build/generated/uniffi/jniLibs`, which `androidApp` packages |

Add `-Pnomi.rust.release=true` for an optimized build. The default is a debug build.

The iOS static library is compiled for iOS 15 (`IPHONEOS_DEPLOYMENT_TARGET`), the oldest any
binary linking it targets. `src/nativeInterop/cinterop/nomi_sdk.def` adds the system frameworks
it needs (`Security`) to every binary that links it.

## Working on it

```bash
cd rust && cargo test && cargo clippy --all-targets -- -D warnings
```

```bash
cd native && ./gradlew :shared:sdk:desktopTest :shared:sdk:iosSimulatorArm64Test
```

The iOS tasks need `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`. The Android build needs
`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android` and an NDK:
`ANDROID_NDK_HOME`, else `<sdk>/ndk/<nomi.android.ndkVersion>`, else the newest NDK in the SDK.

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

Android packaging and app wiring, left open here, were done in Milestone 5.

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

The write methods (`upsert_*`, `append_draft`, …) let the current Kotlin repositories store server
data through Rust instead of Room while screens move over; sync (Milestone 4) writes through the
same repositories.

**3 — Connected engine (done).** Rust talks to the backend as it is today. The endpoints and wire
shapes come from `backend/crates/nomi-server` and `nomi-realtime`, and the behaviour from the Kotlin
`AuthApi` and `RealtimeHub` this replaces. This is verified by 83 Rust tests against a fake backend
(wiremock for REST, and a tokio-tungstenite gateway that also answers the auth endpoints), plus 19
Kotlin tests on the desktop JVM and the iOS simulator.

- **HTTP:** one pooled reqwest client with a 10 s connect timeout and a 30 s request timeout. It
  never follows redirects, so a bearer token can't leak. Failures map to `Network`, `Http { status }`,
  `InvalidResponse`, `Unauthorized` or `NotSignedIn`. Cancelling a Kotlin call aborts its request.
- **TLS:** rustls on `ring` with webpki roots on every platform. `aws-lc-rs` would need cmake to
  cross-compile, and the platform verifier needs JNI setup on Android. One consequence: CAs a user
  installs on the device aren't trusted.
- **Session:** tokens live in the platform's secure store, through the `CredentialStore` callback
  Kotlin implements. Sign-in handles the emailed-code challenge (`EmailCode` errors mirror
  `code_error`). A 401 triggers one shared refresh and a retry. If the backend refuses the refresh,
  the session ends with `SignedOut { SessionExpired }`. A refresh that a newer sign-in overtook
  changes nothing. Sign-out revokes the session best effort.
- **Realtime:**
  - Watched topics are reference-counted into subscriptions. After each `ready`, every live topic
    gets `TopicResynced`. `denied` becomes `TopicAccessLost`, and `unavailable` is retried with
    backoff.
  - A ping goes out every 25 s, and a socket silent for 60 s is recycled. Reconnects use
    full-jitter backoff from 0.5 s up to 30 s, which resets after 10 s of stable connection.
  - A 401 upgrade or close code 4001 triggers one token refresh. If the socket is still refused, the
    session ends with `AccessRevoked`.
  - The socket runs exactly while the SDK is started and someone is signed in.
- **Kotlin:** `NomiClient.user` and `NomiClient.connection` are `StateFlow`s, and
  `topicEvents(topic)` holds a watch for as long as it is collected.

The REST endpoints that feed local data (sessions, messages, notifications) came with Milestone 4,
where sync writes what they return into storage.

**4 — Reliable engine (done).** Rust keeps the open account's data in step with the backend, which
the Kotlin `SessionRepository`, `ChatRepository`, `NotificationRepository`, `ActivityRepository` and
`RealtimeSync` do today. Endpoints are the backend's as they are (`/api/sessions`,
`/api/sessions/:id/messages[/:id[/approval]]`, `/api/notifications[/:id/read | /read-all]`). This is
verified by 125 Rust tests (wiremock for REST, with realtime events put on the event stream), plus 25
Kotlin tests on the desktop JVM and the iOS simulator.

- **Engine:** one per open account, started by `open_account` and stopped when the account closes.
  It only does network work while that account's user is signed in.
- **Reads:** `refresh_sessions` mirrors the server's chat list. The unread flag stays the
  device's, and a chat the server dropped goes unless the outbox still owes it a message.
  `refresh_messages` and `load_older_messages` page with the server's `before` cursor, and a
  message the server deleted inside a page's span is removed. Paging cursors (`has_older_messages`,
  inbox pages) live in `sync_state` and survive restarts.
- **Realtime:** `SdkEvent::Realtime` carries a typed `RealtimeEvent` (Kotlin can also call
  `decode_realtime_event`). Sync applies each topic's events in order on its own lane:
  - Deltas stream into the draft, and a delta after its turn finished is dropped.
  - A finished turn fetches its message and clears only its own draft; a failed one sends
    `ChatTurnFailed`.
  - Agent phases feed agent activity.
  - User events update the inbox and the chat list. `ChatUpdated`, which the Kotlin app ignored,
    moves the chat up or refetches the list.
  - `TopicResynced` and `EventsDropped` refetch, and drop drafts and agents only a live event would
    have ended.
  - `bind_chat` (`syncChat` in Kotlin) holds a chat's topic while its screen is open. The first
    binding resyncs, later ones refresh.
- **Outbox:** sending a message and marking notifications read are written locally at once, then
  queued in SQLite under an idempotency key.
  - Entries in one lane (a chat, or the inbox) go out in order.
  - Network errors, 408, 429 and 5xx are retried with full-jitter backoff (1 s doubling to 5 min,
    8 attempts). Any other refusal fails the entry: its message shows `failed` until
    `retry_outbox` or `discard_outbox`.
  - Coming back online makes waiting entries due at once, and entries left by a previous run are
    replayed.
  - "Mark all read" folds in single reads still waiting. Finished entries stay 24 h as idempotency
    records.
- **No double sends without backend support:** the backend has no idempotency keys, so a message
  whose attempt may have landed (a timeout, an unreadable answer, a crash mid-request) isn't simply
  posted again. Before retrying, the worker looks in the chat's newest page for the person's message
  with the same text, created no more than 5 minutes before it was queued and not claimed by an
  earlier entry. If one is there, it claims it instead of sending.
  - One limitation: if the backend rewrites the text (attachment tags), no match is found and the
    message can be sent twice.
  - Until the backend answers, a realtime echo of the message can briefly show next to its
    `sending` row.

`resolve_approval` and `create_session` are direct calls, not queued: an approval only matters while
the turn waits, and a new chat's id is needed at once.

**5 — The app runs on the engine (first pass done).** The apps use the Rust engine for everything it
covers: the session, chats, messages, notifications, agent activity and realtime. Room
(`:shared:database`) and the Kotlin realtime hub (`:shared:realtime`) are gone, and so are Ktor's
sessions, messages, notifications and auth clients. This is verified by 477 desktop and Android host
tests, 106 iOS simulator tests, the debug APK, the macOS installer, and the iOS app built and launched
in the simulator.

- **Android:** `cargoBuildAndroid` builds the engine for arm64-v8a, armeabi-v7a and x86_64, and the
  app packages it with JNA's own library. `abiFilters` keeps the APK to those three ABIs, so no device
  installs it without the engine.
- **Data layer:** the public APIs of `SessionRepository`, `ChatRepository`, `NotificationRepository`,
  `ActivityRepository`, `RealtimeSync` and `SessionGate` are unchanged, so screens and ViewModels
  barely moved. They now read the engine's store and call `NomiClient`.
  - `NomiEngine` creates the client off the main thread on first use, because creating it reads the
    stored session.
  - `TokenStoreCredentials` is the engine's `CredentialStore` over the existing secure token stores
    (Keystore, Keychain, desktop). An unreadable store counts as signed out, and nothing is thrown
    back into the engine.
  - `SdkAuthRepository` signs in through the engine.
  - `EngineSession` starts the engine at sign-in, opens the user's account and wipes local data at
    sign-out. When the engine ends the session itself, `SessionGate` signs out too.
- **Errors:** errors map to what screens already handle: `ApiException`, `EmailCodeException` and
  `SessionEndedException`.
- **Data on disk:** each platform keeps the engine's data under its private app storage
  (`filesDir/nomi-sdk`, Application Support `nomi-sdk`, `<desktop data dir>/sdk`).
- **CI:** the Android and desktop-installer jobs install Rust; Android also gets its targets and the
  runner's NDK.
- **Account pages (second pass):** crew, preferences, the home summary, money, reminders, memory,
  connections, usage and projects are typed engine calls (`services/`). The existing Kotlin `*Api`
  interfaces are kept and implemented on the engine (`SdkHomeApi`, `SdkMoneyApi`, …), so their
  repositories and screens are unchanged. Ktor, `NomiHttpClient` and its separate token refresh are
  gone: all networking now goes through Rust. The calls follow the backend's routes as they are, which
  fixed four the Kotlin client got wrong:
  - setting a budget is `PUT /api/money/budgets` (was `POST`);
  - a reminder action is `POST /api/reminders/:id` (was `…/:id/action`);
  - cancelling scheduled work is `POST /api/scheduled-tasks/:id/cancel` (was `DELETE /api/reminders/tasks/:id`);
  - money amounts are sent in currency units (`amount`, `monthly_limit`), as the backend parses them
    (were cents), and the two money writes no longer expect a body back.

  This is verified by 136 Rust tests, 432 desktop and Android host tests, 96 iOS simulator tests, the
  debug APK, the macOS installer, and the iOS app launched in the simulator.

## What's left, by milestone

**1 — Native SDK**
- Release builds don't turn on the optimized Rust build by themselves: pass `-Pnomi.rust.release=true`.
  Tie it to the release variant and to the desktop distribution task.
- 32-bit x86 Android (old emulators only) isn't built.
- Linux and Windows desktop libraries are only built by CI's installer jobs, and not run there.

**2 — Local engine**
- Observations cap at 500 rows: a chat with more than 500 stored messages can't show its oldest
  ones. This needs a windowed message observation.
- The old Room database file (`nomi.db` in each platform's app storage) is left on disk. Nothing is
  lost (the server has it all), but it should be deleted once.

**3 — Connected engine**
- Certificate authorities a user installs on the device aren't trusted (webpki roots, not the
  platform verifier).
- Sign-out revokes the session inside `SessionGate`'s lock. Offline, a new sign-in waits up to the
  request timeout.

**4 — Reliable engine**
- The no-double-send check matches on text: a message whose text the backend rewrites (attachment
  tags) can be sent twice. Backend idempotency keys on `POST /messages` would close this.
- Until the backend answers, a realtime echo of a queued message can briefly show next to its
  `sending` row.
- Chats resync from the newest page on every bind: the backend has no "changes since" endpoint.
- The inbox's filters and counts (`counts`, `filter=unread|read|promo`) aren't mirrored.

**5 — The app runs on the engine**
- The account pages are online only, as before: nothing is cached, and their writes aren't queued in
  the outbox. Their timestamps stay RFC 3339 strings.
- Endpoints the apps never called aren't ported (budget deletion, money period, connecting services,
  project details and files, usage by month).
- Show the outbox in the chat screen: a `failed` message with retry and discard
  (`retryOutbox` / `discardOutbox`). Today only the status string changes.
- Move ViewModels from the repository wrappers to `NomiClient` and the `AccountStore` flows directly,
  then drop the duplicate app models and DTOs (`ai.nomi.data.Session`, `MoneySummaryDto`, …) in
  favour of the SDK's.
- Not verified yet: a real sign-in and chat end to end. The debug build points at the hosted API and
  no local backend ran. Running the Android app on an emulator or device.

**6 — Production ready**
- Release profiles everywhere (above); sign and size-check the artifacts (debug `.so` files are
  10–19 MB per ABI).
- Recovery tests: a corrupt database, a full disk, a migration interrupted mid-way. Integration
  tests against a real backend on every platform.

**6 — Production ready.** Android, desktop Linux and Windows artifacts, plus recovery and integration
tests on every platform.
