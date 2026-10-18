# Nomi Native Apps (Desktop + Mobile, Kotlin Multiplatform) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Nomi as native apps on macOS, Windows, Linux, Android and iOS from one Kotlin Multiplatform + Compose Multiplatform codebase, styled with the Nomi Expressive design system on M3 Expressive, with an always-present companion surface on every platform (a Dynamic-Island-style pill on desktop, a Messenger-style chat-head bubble on Android, a Live Activity / Dynamic Island on iOS) fed by one realtime connection per app.

**Architecture:** The backend gains one user-scoped WebSocket gateway (`GET /api/realtime/ws`) that multiplexes the existing MQTT topics (`chat/{id}/stream`, `project/{id}/stream`) plus a new per-user topic (`user/{id}/stream`) over a single socket, and a push channel (FCM / APNs) for when no socket can stay open. Each native app holds exactly one `RealtimeHub` (Koin singleton) that owns that socket, ref-counts topic subscriptions, and fans events out to listeners; ViewModels register listeners (or collect the `Flow` adapter), and every event ends in "fetch from REST, upsert into Room", so the UI only ever observes Room. Navigation 3 drives the opened app; the companion surfaces (island, bubble) are separate windows/overlays that read the same hub.

**Tech Stack:** Kotlin 2.2+, Compose Multiplatform 1.9+ (Material 3 Expressive APIs, `@ExperimentalMaterial3ExpressiveApi`), Koin 4.1+ (`koin-compose-viewmodel`), Room KMP 2.8+ with `BundledSQLiteDriver`, JetBrains `lifecycle-viewmodel-compose`, JetBrains Navigation 3 (`navigation3-ui`), Ktor client 3.x (WebSockets, Auth, ContentNegotiation), kotlinx.serialization, kotlinx.coroutines, JNA 5.x (desktop native window tweaks), SwiftUI + ActivityKit (iOS Live Activity extension only), Roborazzi (screenshot tests). Backend side: existing Rust/axum/rumqttc stack.

**Spec:** This plan is its own spec (no separate design doc). Source of truth for visual identity: `docs/superpowers/specs/2026-10-05-nomi-expressive-redesign-design.md`, `frontend/src/lib/styles/material3.css`, `frontend/src/lib/components/m3/shapes.ts`, `frontend/CLAUDE.md`. Source of truth for realtime: `docs/superpowers/specs/2026-08-11-websocket-bridge-design.md`, `backend/crates/nomi-realtime/src/lib.rs`.

---

## 0. Platform reality check (read before anything else)

The request asks for the same "always there" behaviour on five platforms. They do not allow the same things. This table is what the plan actually builds:

| Platform | Always running? | Companion surface | Cursor / hover | Realtime when app closed |
|---|---|---|---|---|
| macOS | Yes — login item, no Dock icon (`LSUIElement`), menu-bar tray | Notch-hugging pill window above the menu bar (`NSStatusWindowLevel`), joins all Spaces, expands on hover | Global cursor position via `MouseInfo` (no permission needed); hover expands island; Nomi's face looks toward the cursor | Live socket in the background process |
| Windows | Yes — `HKCU\…\Run` autostart, tray icon | Top-center pill, topmost, `WS_EX_TOOLWINDOW \| WS_EX_NOACTIVATE` (no taskbar entry, never steals focus) | Same as macOS | Live socket |
| Linux X11 | Yes — `~/.config/autostart/nomi.desktop`, tray (StatusNotifier) | Top-center pill, `_NET_WM_WINDOW_TYPE_DOCK`, always on top | Same | Live socket |
| Linux Wayland | Yes, same autostart | **No island** — Wayland forbids clients from positioning windows or reading the global cursor. Falls back to tray + desktop notifications | Not available | Live socket |
| Android | Yes — foreground service (`specialUse`), restarts on boot | Chat-head bubble drawn with `SYSTEM_ALERT_WINDOW` (user grants "Display over other apps"); fallback: Android Bubbles API on notifications when overlay is denied | Touch drag / snap / fling-to-dismiss | Socket in the foreground service; FCM when the OS kills it |
| iOS | **No** — iOS does not let apps keep sockets or draw over other apps | Live Activity on Lock Screen + Dynamic Island (ActivityKit, SwiftUI widget extension), updated by APNs `liveactivity` pushes; in-app floating bubble while the app is open | Not applicable | APNs only |

"Access screen cursor" is implemented as **cursor position + hover**. Reading screen *contents* (Screen Recording permission on macOS) is out of scope; it can be a later plan.

**Realtime decision (flip it now if you disagree):** "single WS MQTT" is implemented as one WebSocket from each app to the Rust gateway, which speaks MQTT to EMQX on the app's behalf. The alternative — apps speaking MQTT-over-WebSocket straight to EMQX — needs EMQX JWT auth plus per-session ACLs that EMQX cannot express from a JWT alone (session access depends on org membership in Postgres), so it is not chosen.

## Phases and how this document is used

Each phase produces working, testable software on its own. Phases A–C are written at full task granularity below and can be executed directly. Phases D–H list their tasks, files, interfaces and the non-obvious code; **before executing D–H, expand that phase into its own plan file** (`docs/superpowers/plans/2026-10-xx-native-<phase>.md`) using the same writing-plans format, because each depends on decisions made while building A–C.

| Phase | Delivers | Depends on |
|---|---|---|
| A | Backend: realtime gateway, user topic, device push | — |
| B | `native/` KMP project, Nomi design system in Compose | — (parallel with A) |
| C | Network, auth, `RealtimeHub`, Room, Koin, Navigation 3 shell | A, B |
| D | Screens: auth, home, chats, chat (streaming + content blocks), notifications, settings | C |
| E | Desktop: background tray app + Dynamic Island + cursor hover | C (D for deep links) |
| F | Android: foreground service + chat-head bubble | C |
| G | iOS: app shell + Live Activity / Dynamic Island + APNs | A (push), C |
| H | CI, signing, packaging, release | E, F, G |

## Global Constraints

- All native code lives under `native/` at the repo root. Backend changes stay in `backend/`. Nothing in `frontend/` changes except where a task says so.
- Package root: `ai.nomi.app` (Android `applicationId` `ai.nomi.app`, iOS bundle id `ai.nomi.app`, desktop `packageName` `Nomi`).
- Targets: `androidTarget` (minSdk 26, compile/target 36), `iosArm64`, `iosSimulatorArm64`, `jvm("desktop")` (JDK 21).
- Colours, gradients, motion and shapes come **only** from `frontend/src/lib/styles/material3.css` and `frontend/src/lib/components/m3/shapes.ts` via the generator in Task B2 — never hand-typed hex in Kotlin.
- Fonts: Bricolage Grotesque (display/headline, weight 800 available), Instrument Sans (title/body/label), JetBrains Mono (meta). Bundled as Compose resources.
- `frontend/CLAUDE.md` UI rules apply to native screens too: no accent stripes on cards, no uppercase eyebrow labels, no glowing/pulsing status dots, no gradient text, no emoji as icons; at most one gradient button per screen; 44dp minimum touch target (48dp on Android per M3); every screen checked in dark mode and at 390dp wide.
- Motion: spatial changes use M3 Expressive spatial springs (`MotionScheme.expressive()`), colour/opacity use effects springs; shapes only spin/morph while something is working; respect reduced motion (Android `ANIMATOR_DURATION_SCALE == 0`, iOS `UIAccessibility.isReduceMotionEnabled`, desktop: setting in app).
- Every numeric literal in production code is a named `const` (repo rule — see memory "No hardcoded prompts/magic numbers"). LLM prompts never live in native code.
- No AI attribution in commits/PRs in this repo (repo rule). Commit messages are plain.
- Wire format: JSON via kotlinx.serialization with `ignoreUnknownKeys = true`; enums decode unknown values to an `Unknown` case — the backend adds `StreamEnvelope` variants often.
- Realtime is never the source of truth: a reconnect never replays; it triggers REST refetch (same rule as the web, websocket-bridge spec §3).
- Auth for native: `POST /api/auth/login` returns `{access_token, refresh_token}` (`TokenPairResponse`, `backend/crates/nomi-server/src/routes/auth.rs:44`); refresh via `POST /api/auth/refresh`. Native apps call the Rust API directly with `Authorization: Bearer` (no SvelteKit proxy).

## Review Focus

1. **Socket drops while a turn is streaming** (laptop sleep, Wi-Fi switch, Android doze): the user expects the reply to appear complete once back online, not half-streamed forever. Pinned by Task C3 test `reconnect_resubscribes_and_emits_resynced` and Task C5 test `resynced_event_refetches_open_chat`.
2. **Token expires while the socket is open or during reconnect**: the hub must refresh the token and reconnect, never loop on 401. Pinned by Task C3 test `unauthorized_upgrade_refreshes_token_once_then_stops`.
3. **Two screens listening to the same chat; one leaves**: the other must keep receiving events (ref-counting). Pinned by Task C3 test `unsubscribe_only_when_last_listener_removed`.
4. **A listener throws**: other listeners on the same topic still get the event and the hub keeps running. Pinned by Task C3 test `throwing_listener_does_not_break_others`.
5. **A client asks for someone else's chat**: denied, and no events leak. Pinned by Task A2 test `subscribe_to_foreign_chat_is_denied_and_silent`.

---

## Repository layout (created across phases)

```
native/
  settings.gradle.kts
  build.gradle.kts
  gradle.properties
  gradle/libs.versions.toml
  scripts/gen-tokens.mjs                 # B2: material3.css + shapes.ts → Kotlin
  shared/
    core/            # models, Result, dispatchers, time            (commonMain)
    designsystem/    # NomiTheme, tokens (generated), shapes, components
    network/         # Ktor client, auth, token storage (expect/actual), REST APIs
    realtime/        # RealtimeHub, wire frames, transport
    database/        # Room DB, entities, DAOs
    data/            # repositories: REST + realtime → Room
    feature/
      auth/ home/ chats/ chat/ notifications/ settings/
    companion/       # island/bubble shared state machine + UI (CompanionState)
    app/             # Koin root, Navigation 3 NavDisplay, NomiApp()
  androidApp/        # MainActivity, NomiPresenceService, BubbleOverlay
  desktopApp/        # main(), tray, island window, native interop (JNA)
  iosApp/            # Xcode project: SwiftUI host, NomiLiveActivity extension
```

Each `shared/*` module is a KMP library with `commonMain`, `commonTest`, and only the platform source sets it needs.

---

# Phase A — Backend realtime gateway and push

### Task A1: Gateway wire protocol types

**Files:**
- Create: `backend/crates/nomi-realtime/src/gateway.rs`
- Modify: `backend/crates/nomi-realtime/src/lib.rs` (add `pub mod gateway;` and re-exports)
- Test: inline `#[cfg(test)] mod tests` in `gateway.rs`

**Interfaces:**
- Produces: `Topic { User(Uuid), Chat(Uuid), Project(Uuid) }` with `mqtt_topic()`, `wire()`, `parse_client(&str) -> Option<Topic>`, `from_mqtt(&str) -> Option<Topic>`; `ClientFrame`, `ServerFrame` (serde). Wire strings: `"user"`, `"chat:<uuid>"`, `"project:<uuid>"`.

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_topic_round_trips_between_wire_and_mqtt() {
        let id = Uuid::new_v4();
        let topic = Topic::parse_client(&format!("chat:{id}")).unwrap();
        assert_eq!(topic, Topic::Chat(id));
        assert_eq!(topic.mqtt_topic(), format!("chat/{id}/stream"));
        assert_eq!(Topic::from_mqtt(&topic.mqtt_topic()), Some(topic));
        assert_eq!(topic.wire(), format!("chat:{id}"));
    }

    #[test]
    fn client_cannot_name_the_user_topic() {
        assert_eq!(Topic::parse_client("user"), None);
        assert_eq!(Topic::parse_client(&format!("user:{}", Uuid::new_v4())), None);
    }

    #[test]
    fn garbage_topics_are_rejected() {
        assert_eq!(Topic::parse_client("chat:not-a-uuid"), None);
        assert_eq!(Topic::parse_client("chat"), None);
        assert_eq!(Topic::from_mqtt("chat/x/stream"), None);
    }

    #[test]
    fn user_topic_wire_is_just_user() {
        let id = Uuid::new_v4();
        assert_eq!(Topic::User(id).wire(), "user");
        assert_eq!(Topic::from_mqtt(&format!("user/{id}/stream")), Some(Topic::User(id)));
    }

    #[test]
    fn client_frames_parse() {
        let f: ClientFrame = serde_json::from_str(r#"{"op":"subscribe","topic":"chat:00000000-0000-0000-0000-000000000001"}"#).unwrap();
        assert!(matches!(f, ClientFrame::Subscribe { .. }));
        let f: ClientFrame = serde_json::from_str(r#"{"op":"ping"}"#).unwrap();
        assert_eq!(f, ClientFrame::Ping);
    }

    #[test]
    fn server_event_frame_serializes_with_type_tag() {
        let frame = ServerFrame::Event { topic: "user".into(), payload: serde_json::json!({"kind":"X"}) };
        assert_eq!(serde_json::to_value(&frame).unwrap(), serde_json::json!({"type":"event","topic":"user","payload":{"kind":"X"}}));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test -p nomi-realtime gateway`
Expected: FAIL — `gateway` module / `Topic` not found.

- [ ] **Step 3: Implement**

```rust
//! The one socket a native app keeps open: it asks for chat/project topics by name, always gets
//! its own user topic, and receives every event wrapped with the topic it came from.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    User(Uuid),
    Chat(Uuid),
    Project(Uuid),
}

impl Topic {
    pub fn mqtt_topic(&self) -> String {
        match self {
            Topic::User(id) => format!("user/{id}/stream"),
            Topic::Chat(id) => format!("chat/{id}/stream"),
            Topic::Project(id) => format!("project/{id}/stream"),
        }
    }

    /// What the client sees. The user topic carries no id: a socket only ever has its own.
    pub fn wire(&self) -> String {
        match self {
            Topic::User(_) => "user".to_string(),
            Topic::Chat(id) => format!("chat:{id}"),
            Topic::Project(id) => format!("project:{id}"),
        }
    }

    /// Topics a client may ask for. The user topic is never asked for — it is implicit.
    pub fn parse_client(s: &str) -> Option<Topic> {
        let (kind, id) = s.split_once(':')?;
        let id = Uuid::parse_str(id).ok()?;
        match kind {
            "chat" => Some(Topic::Chat(id)),
            "project" => Some(Topic::Project(id)),
            _ => None,
        }
    }

    pub fn from_mqtt(s: &str) -> Option<Topic> {
        let mut parts = s.split('/');
        let (kind, id, tail) = (parts.next()?, parts.next()?, parts.next()?);
        if tail != "stream" || parts.next().is_some() {
            return None;
        }
        let id = Uuid::parse_str(id).ok()?;
        match kind {
            "user" => Some(Topic::User(id)),
            "chat" => Some(Topic::Chat(id)),
            "project" => Some(Topic::Project(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientFrame {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerFrame {
    Ready { user_id: Uuid },
    Subscribed { topic: String },
    Unsubscribed { topic: String },
    Denied { topic: String },
    Event { topic: String, payload: serde_json::Value },
    Error { message: String },
    Pong,
}
```

In `lib.rs` add `pub mod gateway;` and `pub use gateway::{ClientFrame, ServerFrame, Topic};`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p nomi-realtime gateway`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-realtime/src/gateway.rs backend/crates/nomi-realtime/src/lib.rs
git commit -m "Realtime gateway wire types: topics and client/server frames"
```

### Task A2: `GET /api/realtime/ws` multiplexing gateway

**Files:**
- Create: `backend/crates/nomi-server/src/routes/realtime.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs` (add `pub mod realtime;`)
- Modify: `backend/crates/nomi-server/src/app.rs` (add `.route("/api/realtime/ws", get(crate::routes::realtime::realtime_stream))` next to `/api/sessions/:id/ws` at line ~105)
- Test: `backend/crates/nomi-server/tests/realtime_ws.rs`

**Interfaces:**
- Consumes: `Topic`, `ClientFrame`, `ServerFrame` (A1); `authorize_session_access(pool, user_id, session_id)` (`routes/sessions.rs:122`, `pub(crate)`); `project_role(pool, project_id, user_id)` (`routes/deploy.rs:58`, `pub(crate)`).
- Produces: the endpoint every native client uses. Behaviour: after upgrade sends `{"type":"ready","user_id":…}`; always subscribed to `user/{sub}/stream`; `subscribe` → authorize → `subscribed` or `denied`; `unsubscribe` → `unsubscribed`; `ping` → `pong`; malformed JSON → `error` (socket stays open); more than `MAX_TOPICS_PER_SOCKET` (64) → `denied`; broker loss → close code 1011 so the client reconnects.

- [ ] **Step 1: Write the failing tests**

Copy the helpers `SECRET`, `test_state`, `spawn_app`, `register_and_login`, `create_session` verbatim from `backend/crates/nomi-server/tests/session_ws.rs` (lines 1–70) into the new file (integration test files are separate crates; the repo duplicates these per file). Then:

```rust
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use nomi_realtime::MqttPublisher;

const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
const PUBLISH_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const SILENCE_WINDOW: Duration = Duration::from_millis(800);

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(ws_base: &str, token: Option<&str>) -> Result<Ws, tokio_tungstenite::tungstenite::Error> {
    let mut req = format!("{ws_base}/api/realtime/ws").into_client_request().unwrap();
    if let Some(t) = token {
        req.headers_mut().insert("Authorization", format!("Bearer {t}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(req).await.map(|(ws, _)| ws)
}

async fn next_json(ws: &mut Ws) -> Value {
    loop {
        let msg = tokio::time::timeout(FRAME_TIMEOUT, ws.next()).await.expect("timed out").unwrap().unwrap();
        if let WsMessage::Text(t) = msg { return serde_json::from_str(&t).unwrap(); }
    }
}

async fn send(ws: &mut Ws, v: Value) {
    ws.send(WsMessage::Text(v.to_string().into())).await.unwrap();
}

/// The gateway's MQTT SUBSCRIBE is asynchronous, so publish repeatedly until the event shows up.
async fn publish_until_event(ws: &mut Ws, mqtt_topic: &str, payload: Value) -> Value {
    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, &format!("test-pub-{}", Uuid::new_v4()));
    let deadline = tokio::time::Instant::now() + FRAME_TIMEOUT;
    loop {
        publisher.try_publish_to(mqtt_topic, &payload).unwrap();
        if let Ok(Some(Ok(WsMessage::Text(t)))) = tokio::time::timeout(PUBLISH_RETRY_INTERVAL, ws.next()).await {
            let v: Value = serde_json::from_str(&t).unwrap();
            if v["type"] == "event" { return v; }
        }
        assert!(tokio::time::Instant::now() < deadline, "event never arrived on {mqtt_topic}");
    }
}

async fn assert_silent(ws: &mut Ws) {
    if let Ok(Some(Ok(WsMessage::Text(t)))) = tokio::time::timeout(SILENCE_WINDOW, ws.next()).await {
        panic!("expected silence, got {t}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn upgrade_without_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    assert!(connect(&ws_base, None).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn ready_then_user_topic_events_arrive(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-user@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    let ready = next_json(&mut ws).await;
    assert_eq!(ready["type"], "ready");
    let user_id = Uuid::parse_str(ready["user_id"].as_str().unwrap()).unwrap();

    let ev = publish_until_event(&mut ws, &format!("user/{user_id}/stream"), json!({"kind":"Ping"})).await;
    assert_eq!(ev["topic"], "user");
    assert_eq!(ev["payload"]["kind"], "Ping");
}

#[sqlx::test(migrations = "../../migrations")]
async fn subscribe_to_own_chat_relays_its_events(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-chat@nomi.test").await;
    let session = create_session(&client, &base, &token).await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await; // ready

    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{session}")})).await;
    let ack = next_json(&mut ws).await;
    assert_eq!(ack, json!({"type":"subscribed","topic":format!("chat:{session}")}));

    let ev = publish_until_event(&mut ws, &format!("chat/{session}/stream"), json!({"kind":"PersonalityChanged"})).await;
    assert_eq!(ev["topic"], format!("chat:{session}"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn subscribe_to_foreign_chat_is_denied_and_silent(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let owner = register_and_login(&client, &base, "rt-owner@nomi.test").await;
    let foreign_session = create_session(&client, &base, &owner).await;
    let intruder = register_and_login(&client, &base, "rt-intruder@nomi.test").await;

    let mut ws = connect(&ws_base, Some(&intruder)).await.unwrap();
    next_json(&mut ws).await; // ready
    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{foreign_session}")})).await;
    assert_eq!(next_json(&mut ws).await["type"], "denied");

    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, "test-pub-foreign");
    publisher.try_publish_to(&format!("chat/{foreign_session}/stream"), &json!({"kind":"Leak"})).unwrap();
    assert_silent(&mut ws).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn unsubscribe_stops_events(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-unsub@nomi.test").await;
    let session = create_session(&client, &base, &token).await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await;
    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{session}")})).await;
    next_json(&mut ws).await;
    publish_until_event(&mut ws, &format!("chat/{session}/stream"), json!({"kind":"A"})).await;

    send(&mut ws, json!({"op":"unsubscribe","topic":format!("chat:{session}")})).await;
    // drain until the unsubscribed ack (late events from the retry loop may precede it)
    loop { if next_json(&mut ws).await["type"] == "unsubscribed" { break; } }
    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, "test-pub-unsub");
    publisher.try_publish_to(&format!("chat/{session}/stream"), &json!({"kind":"B"})).unwrap();
    assert_silent(&mut ws).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn malformed_frame_gets_error_and_socket_survives(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-bad@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await;
    ws.send(WsMessage::Text("{not json".into())).await.unwrap();
    assert_eq!(next_json(&mut ws).await["type"], "error");
    send(&mut ws, json!({"op":"ping"})).await;
    assert_eq!(next_json(&mut ws).await, json!({"type":"pong"}));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && docker compose up -d && cargo test -p nomi-server --test realtime_ws`
Expected: FAIL — 404 on upgrade / route missing.

- [ ] **Step 3: Implement `routes/realtime.rs`**

```rust
//! `GET /api/realtime/ws`: one socket per native app. It always carries the person's own
//! `user/{id}/stream` and, on request, any chat or project they can open — each event wrapped
//! with the topic it came from, so the app can hand it to whoever is listening.

use std::collections::HashSet;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_realtime::{ClientFrame, ServerFrame, Topic};

use crate::app::AppState;

const MAX_TOPICS_PER_SOCKET: usize = 64;
const MQTT_KEEP_ALIVE: Duration = Duration::from_secs(30);
const MQTT_CHANNEL_CAPACITY: usize = 64;
const CLOSE_BROKER_LOST: u16 = 1011;

pub async fn realtime_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| run(socket, state, claims.sub))
}

async fn may_open(state: &AppState, user_id: Uuid, topic: Topic) -> bool {
    match topic {
        Topic::User(id) => id == user_id,
        Topic::Chat(session_id) => crate::routes::sessions::authorize_session_access(&state.pool, user_id, session_id).await.is_ok(),
        Topic::Project(project_id) => crate::routes::deploy::project_role(&state.pool, project_id, user_id).await.is_ok(),
    }
}

async fn send(socket: &mut WebSocket, frame: &ServerFrame) -> bool {
    let text = serde_json::to_string(frame).expect("server frames always serialize");
    socket.send(Message::Text(text)).await.is_ok()
}

async fn run(mut socket: WebSocket, state: AppState, user_id: Uuid) {
    let mut options = MqttOptions::new(format!("realtime-gw-{}", Uuid::new_v4()), state.mqtt_broker_host.clone(), state.mqtt_broker_port);
    options.set_keep_alive(MQTT_KEEP_ALIVE);
    let (client, mut eventloop) = AsyncClient::new(options, MQTT_CHANNEL_CAPACITY);

    let user_topic = Topic::User(user_id);
    if client.subscribe(user_topic.mqtt_topic(), QoS::AtMostOnce).await.is_err() {
        return;
    }
    let mut topics: HashSet<Topic> = HashSet::from([user_topic]);
    if !send(&mut socket, &ServerFrame::Ready { user_id }).await {
        return;
    }

    loop {
        tokio::select! {
            event = eventloop.poll() => match event {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    let Some(topic) = Topic::from_mqtt(&p.topic) else { continue };
                    if !topics.contains(&topic) { continue; } // a late message after unsubscribe
                    let Ok(payload) = serde_json::from_slice(&p.payload) else { continue };
                    if !send(&mut socket, &ServerFrame::Event { topic: topic.wire(), payload }).await { break; }
                }
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!(error = %e, %user_id, "realtime gateway lost the broker");
                    let _ = socket.send(Message::Close(Some(CloseFrame { code: CLOSE_BROKER_LOST, reason: "broker unavailable".into() }))).await;
                    break;
                }
            },
            incoming = socket.recv() => {
                let Some(Ok(msg)) = incoming else { break };
                let Message::Text(text) = msg else { continue };
                let reply = match serde_json::from_str::<ClientFrame>(&text) {
                    Err(_) => ServerFrame::Error { message: "unreadable frame".into() },
                    Ok(ClientFrame::Ping) => ServerFrame::Pong,
                    Ok(ClientFrame::Subscribe { topic: name }) => match Topic::parse_client(&name) {
                        Some(topic) if topics.contains(&topic) => ServerFrame::Subscribed { topic: name },
                        Some(topic) if topics.len() < MAX_TOPICS_PER_SOCKET && may_open(&state, user_id, topic).await => {
                            if client.subscribe(topic.mqtt_topic(), QoS::AtMostOnce).await.is_err() { break; }
                            topics.insert(topic);
                            ServerFrame::Subscribed { topic: name }
                        }
                        _ => ServerFrame::Denied { topic: name },
                    },
                    Ok(ClientFrame::Unsubscribe { topic: name }) => {
                        if let Some(topic) = Topic::parse_client(&name).filter(|t| topics.remove(t)) {
                            let _ = client.unsubscribe(topic.mqtt_topic()).await;
                        }
                        ServerFrame::Unsubscribed { topic: name }
                    }
                };
                if !send(&mut socket, &reply).await { break; }
            }
        }
    }
}
```

Note the `topics.contains` filter on incoming publishes: it is what makes `unsubscribe_stops_events` deterministic even if the broker delivers one more message after UNSUBSCRIBE.

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test -p nomi-server --test realtime_ws`
Expected: PASS (6 tests). Also run `cargo test -p nomi-server --test session_ws` — still PASS (old per-session endpoint untouched; the web keeps using it).

- [ ] **Step 5: Commit**

```bash
git add backend/crates/nomi-server/src/routes/realtime.rs backend/crates/nomi-server/src/routes/mod.rs backend/crates/nomi-server/src/app.rs backend/crates/nomi-server/tests/realtime_ws.rs
git commit -m "Add /api/realtime/ws: one socket multiplexing user, chat and project topics"
```

### Task A3: Per-user topic — notifications and chat replies

**Files:**
- Modify: `backend/crates/nomi-realtime/src/lib.rs` (add `UserEvent`)
- Create: `backend/crates/nomi-server/src/user_events.rs` (mirror of `project_events.rs`)
- Modify: `backend/crates/nomi-server/src/lib.rs` (`pub mod user_events;`)
- Modify: `backend/crates/nomi-server/src/notifications.rs` (`insert` publishes after the row is written)
- Modify: `backend/crates/nomi-server/src/worker.rs` (both `TurnCompleted` publish sites, lines ~158 and ~229)
- Modify: `backend/crates/nomi-server/src/main.rs:44` and `backend/crates/nomi-server/src/bin/worker.rs` (call `user_events::init`)
- Test: `backend/crates/nomi-server/tests/user_events.rs`

**Interfaces:**
- Produces (wire, `kind`-tagged like `StreamEnvelope`):
  - `NotificationCreated { notification_id: Uuid, kind: String, title: String, body: String, link: Option<String> }`
  - `ChatReply { session_id: Uuid, message_id: Uuid, preview: String }` — `preview` is the first `CHAT_REPLY_PREVIEW_CHARS` (140) chars of the reply, cut on a char boundary.
  - `user_events::publish(user_id: Uuid, event: &UserEvent)`, `user_events::topic(user_id) -> String`, `user_events::preview(text: &str) -> String`.

- [ ] **Step 1: Failing tests**

```rust
// tests/user_events.rs
use nomi_realtime::UserEvent;
use nomi_server::user_events;

#[test]
fn preview_cuts_on_char_boundary() {
    let long = "é".repeat(500);
    let p = user_events::preview(&long);
    assert_eq!(p.chars().count(), user_events::CHAT_REPLY_PREVIEW_CHARS);
}

#[test]
fn preview_keeps_short_text() {
    assert_eq!(user_events::preview("hi there"), "hi there");
}

#[test]
fn notification_event_wire_shape() {
    let id = uuid::Uuid::nil();
    let v = serde_json::to_value(UserEvent::NotificationCreated { notification_id: id, kind: "quota".into(), title: "T".into(), body: "B".into(), link: None }).unwrap();
    assert_eq!(v["kind"], "NotificationCreated");
    assert_eq!(v["title"], "T");
}
```

Plus an end-to-end check appended to `tests/realtime_ws.rs`: after `register_and_login`, call `user_events::init(TEST_MQTT_BROKER_HOST, TEST_MQTT_BROKER_PORT)` and `nomi_server::notifications::send_to_inbox(&pool, user_id, Notice { kind: Kind::Account, title: "Hello".into(), body: "B".into(), link: None })` in a retry loop (same as `publish_until_event`), asserting an event with `payload.kind == "NotificationCreated"` and `payload.title == "Hello"` arrives on topic `"user"`.

- [ ] **Step 2: Run, expect FAIL** — `cargo test -p nomi-server --test user_events` → `UserEvent` not found.

- [ ] **Step 3: Implement**

In `nomi-realtime/src/lib.rs`:

```rust
/// Events on `user/{id}/stream`: what a person's apps hear about that isn't tied to a chat they
/// have open — the inbox, and replies that landed in any of their chats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum UserEvent {
    NotificationCreated { notification_id: Uuid, kind: String, title: String, body: String, link: Option<String> },
    ChatReply { session_id: Uuid, message_id: Uuid, preview: String },
}
```

`nomi-server/src/user_events.rs`:

```rust
//! A person's own topic, `user/{id}/stream`: their native apps keep it open all the time and
//! show new inbox items and chat replies in the island / bubble / Live Activity.

use std::sync::OnceLock;

use nomi_realtime::{MqttPublisher, UserEvent};
use uuid::Uuid;

pub const CHAT_REPLY_PREVIEW_CHARS: usize = 140;

static PUBLISHER: OnceLock<MqttPublisher> = OnceLock::new();

pub fn init(broker_host: &str, broker_port: u16) {
    let _ = PUBLISHER.get_or_init(|| MqttPublisher::connect(broker_host, broker_port, &format!("nomi-user-events-{}", Uuid::new_v4())));
}

pub fn topic(user_id: Uuid) -> String {
    format!("user/{user_id}/stream")
}

pub fn preview(text: &str) -> String {
    text.chars().take(CHAT_REPLY_PREVIEW_CHARS).collect()
}

pub fn publish(user_id: Uuid, event: &UserEvent) {
    let Some(publisher) = PUBLISHER.get() else { return };
    if let Err(e) = publisher.try_publish_to(&topic(user_id), event) {
        tracing::debug!(error = %e, %user_id, "couldn't tell the person's apps about an event");
    }
}
```

In `notifications.rs::insert`, after `fetch_one` succeeds, call `crate::user_events::publish(user_id, &UserEvent::NotificationCreated { notification_id: id, kind: notice.kind.as_str().into(), title: notice.title.clone(), body: notice.body.clone(), link: notice.link.clone() })` and then `crate::push::dispatch(...)` (added in A4 — leave that line for A4).

In `worker.rs`, right after each `TurnCompleted` publish, add (the worker already resolved `user_id` and has its own publisher):

```rust
if let Some(message_id) = outcome.message_id {
    let _ = turns.mqtt.try_publish_to(
        &crate::user_events::topic(user_id),
        &UserEvent::ChatReply { session_id: claimed.session_id, message_id, preview: crate::user_events::preview(&outcome.reply) },
    );
}
```

(Use `pool`/`mqtt`/`session_id` names that exist at the first site, ~line 158 — that site uses `outcome.session_id` and a local `mqtt`.)

Add `nomi_server::user_events::init(&mqtt_broker_host, mqtt_broker_port);` beside `project_events::init` in `main.rs` and after the MQTT connect in `bin/worker.rs`.

- [ ] **Step 4: Run** `cargo test -p nomi-server --test user_events --test realtime_ws` → PASS.

- [ ] **Step 5: Commit** — `git commit -m "Publish inbox items and chat replies on the person's user topic"`

### Task A4: Device registration and push (FCM + APNs)

**Files:**
- Create: `backend/migrations/0071_devices.sql`
- Create: `backend/crates/nomi-server/src/push.rs` (trait + FCM + APNs + noop)
- Create: `backend/crates/nomi-server/src/routes/devices.rs`
- Modify: `app.rs` (routes `POST /api/devices`, `DELETE /api/devices/:id`), `AppState` (add `push: std::sync::Arc<dyn crate::push::PushSender>`), every `test_state` in `backend/crates/nomi-server/tests/*.rs` (add `push: Arc::new(nomi_server::push::NoopPush)` — do it with one `sed` across files), `main.rs` (build sender from env), `notifications.rs` / `worker.rs` (dispatch).
- Modify: `backend/.env.example` (add `FCM_PROJECT_ID`, `FCM_SERVICE_ACCOUNT_JSON`, `APNS_KEY_ID`, `APNS_TEAM_ID`, `APNS_PRIVATE_KEY_P8`, `APNS_TOPIC=ai.nomi.app`, `APNS_PRODUCTION=false`)
- Test: `backend/crates/nomi-server/tests/devices_routes.rs`, unit tests in `push.rs`

**Migration:**

```sql
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (platform IN ('android', 'ios', 'macos', 'windows', 'linux')),
    push_token TEXT,
    -- iOS only: ActivityKit push-to-start token, so the server can start a Live Activity.
    live_activity_start_token TEXT,
    app_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, push_token)
);
CREATE INDEX devices_user_id_idx ON devices (user_id);
```

**Interfaces:**
- `POST /api/devices` body `{platform, push_token?, live_activity_start_token?, app_version}` → `200 {id}` (upsert on `(user_id, push_token)`, bumps `last_seen_at`). `DELETE /api/devices/:id` → `204` (only own device; else `404`). Called on sign-in and token refresh; deleted on sign-out.
- `trait PushSender: Send + Sync { async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError>; }` with `PushMessage { title, body, data: BTreeMap<String,String>, kind: PushKind }`, `PushKind { Alert, LiveActivityUpdate { content_state: serde_json::Value }, LiveActivityStart { attributes: serde_json::Value, content_state: serde_json::Value } }`.
- `push::dispatch(pool, sender, user_id, message)` — loads the user's `android`/`ios` devices, sends to each, deletes rows whose token the provider reports as invalid (FCM `UNREGISTERED`, APNs `410`/`BadDeviceToken`).
- Desktop platforms register without a push token (used for "last seen" only); they rely on the live socket.

- [ ] **Step 1: Failing tests** — `devices_routes.rs`: register returns id; re-register same token returns same id; delete foreign device is 404; `push.rs` unit tests: FCM request body builder produces `{"message":{"token":…,"notification":{"title":…,"body":…},"data":{…},"android":{"priority":"HIGH"}}}`; APNs alert payload `{"aps":{"alert":{"title":…,"body":…},"sound":"default"}}`; APNs live activity payload `{"aps":{"timestamp":<unix>,"event":"update","content-state":{…}}}` with header `apns-push-type: liveactivity` and topic `ai.nomi.app.push-type.liveactivity`; `dispatch` with a `RecordingPush` fake that returns `Invalid` for one token deletes that row.
- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement.** FCM: HTTP v1 `POST https://fcm.googleapis.com/v1/projects/{project}/messages:send`, OAuth2 access token minted from the service-account JSON (RS256 JWT → `https://oauth2.googleapis.com/token`, cache until `expires_in − 60s`). APNs: HTTP/2 `POST https://api.push.apple.com/3/device/{token}` (sandbox host when `APNS_PRODUCTION=false`), ES256 provider JWT from the `.p8` key cached for 50 minutes (Apple rejects tokens older than 60 and refreshes more often than 20). Use the existing `reqwest` client (enable `http2` feature if absent) and the `jsonwebtoken` crate already used by `nomi-auth`. If neither provider is configured, `main.rs` installs `NoopPush`.
- [ ] **Step 4: Wire dispatch** — in `notifications::insert` after the user-topic publish: `tokio::spawn(push::dispatch(pool.clone(), push_sender(), user_id, PushMessage::alert(&notice)))` where `push_sender()` reads a `OnceLock<Arc<dyn PushSender>>` set in `main.rs`/`bin/worker.rs` (same pattern as the mailer `OnceLock` in `notifications.rs:59`). In `worker.rs` after the `ChatReply` publish: alert push with title = the chat's title (or "Nomi"), body = preview, data `{session_id, message_id, kind:"chat_reply"}`, plus an iOS `LiveActivityUpdate` with `content_state = {"phase":"replied","preview":…}`.
- [ ] **Step 5: Run** `cargo test -p nomi-server` (whole crate — `AppState` changed) → PASS.
- [ ] **Step 6: Commit** — `git commit -m "Register devices and push inbox items and chat replies via FCM and APNs"`

---

# Phase B — KMP project and the Nomi design system in Compose

### Task B1: Gradle multi-module skeleton that builds on all targets

**Files:**
- Create: `native/settings.gradle.kts`, `native/build.gradle.kts`, `native/gradle.properties`, `native/gradle/libs.versions.toml`, `native/gradle/wrapper/*` (via `gradle wrapper --gradle-version 9.1`), `native/.gitignore`
- Create: `native/build-logic/convention/` with plugins `nomi.kmp.library` (applies `kotlin("multiplatform")`, `com.android.kotlin.multiplatform.library`, sets the three target families, JDK 21, `explicitApi()` off) and `nomi.kmp.compose` (adds `org.jetbrains.compose` + compose compiler)
- Create: every `native/shared/*/build.gradle.kts` listed in the layout, each with one `commonMain` source file (`Module.kt` containing `internal const val MODULE_NAME = "<name>"`) so the module compiles
- Create: `native/shared/app/src/commonMain/kotlin/ai/nomi/app/NomiApp.kt` (`@Composable fun NomiApp() { Text("Nomi") }`)
- Create: `native/androidApp` (`MainActivity` calling `setContent { NomiApp() }`), `native/desktopApp/src/main/kotlin/ai/nomi/desktop/Main.kt` (`application { Window(::exitApplication, title = "Nomi") { NomiApp() } }`), `native/iosApp` (Xcode project from the JetBrains KMP wizard template; `ContentView` hosts `MainViewControllerKt.MainViewController()` which returns `ComposeUIViewController { NomiApp() }`)
- Modify: root `.gitignore` (add `native/.gradle/`, `native/**/build/`, `native/local.properties`, `native/iosApp/**/xcuserdata/`)
- Test: `native/shared/core/src/commonTest/kotlin/ai/nomi/core/SmokeTest.kt`

`libs.versions.toml` (verify each is the latest stable — or the latest alpha where noted — on the day of execution and bump; these are floors):

```toml
[versions]
kotlin = "2.2.20"
agp = "8.13.0"
compose-multiplatform = "1.9.0"
androidx-lifecycle = "2.9.4"          # org.jetbrains.androidx.lifecycle
navigation3 = "1.0.0-alpha04"         # org.jetbrains.androidx.navigation3 (alpha is expected)
koin = "4.1.1"
room = "2.8.1"
sqlite = "2.6.1"
ksp = "2.2.20-2.0.3"
ktor = "3.3.0"
coroutines = "1.10.2"
serialization = "1.9.0"
datetime = "0.7.1"
jna = "5.17.0"
roborazzi = "1.48.0"
turbine = "1.2.1"

[libraries]
compose-material3 = { module = "org.jetbrains.compose.material3:material3", version = "1.9.0" }
compose-adaptive = { module = "org.jetbrains.compose.material3.adaptive:adaptive", version = "1.2.0" }
lifecycle-viewmodel-compose = { module = "org.jetbrains.androidx.lifecycle:lifecycle-viewmodel-compose", version.ref = "androidx-lifecycle" }
lifecycle-runtime-compose = { module = "org.jetbrains.androidx.lifecycle:lifecycle-runtime-compose", version.ref = "androidx-lifecycle" }
navigation3-ui = { module = "org.jetbrains.androidx.navigation3:navigation3-ui", version.ref = "navigation3" }
lifecycle-viewmodel-navigation3 = { module = "org.jetbrains.androidx.lifecycle:lifecycle-viewmodel-navigation3", version = "2.10.0-alpha04" }
koin-core = { module = "io.insert-koin:koin-core", version.ref = "koin" }
koin-compose-viewmodel = { module = "io.insert-koin:koin-compose-viewmodel", version.ref = "koin" }
koin-android = { module = "io.insert-koin:koin-android", version.ref = "koin" }
koin-test = { module = "io.insert-koin:koin-test", version.ref = "koin" }
room-runtime = { module = "androidx.room:room-runtime", version.ref = "room" }
room-compiler = { module = "androidx.room:room-compiler", version.ref = "room" }
sqlite-bundled = { module = "androidx.sqlite:sqlite-bundled", version.ref = "sqlite" }
ktor-client-core = { module = "io.ktor:ktor-client-core", version.ref = "ktor" }
ktor-client-websockets = { module = "io.ktor:ktor-client-websockets", version.ref = "ktor" }
ktor-client-auth = { module = "io.ktor:ktor-client-auth", version.ref = "ktor" }
ktor-client-content-negotiation = { module = "io.ktor:ktor-client-content-negotiation", version.ref = "ktor" }
ktor-serialization-json = { module = "io.ktor:ktor-serialization-kotlinx-json", version.ref = "ktor" }
ktor-client-okhttp = { module = "io.ktor:ktor-client-okhttp", version.ref = "ktor" }
ktor-client-darwin = { module = "io.ktor:ktor-client-darwin", version.ref = "ktor" }
ktor-client-mock = { module = "io.ktor:ktor-client-mock", version.ref = "ktor" }
coroutines-core = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-core", version.ref = "coroutines" }
coroutines-swing = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-swing", version.ref = "coroutines" }
coroutines-test = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-test", version.ref = "coroutines" }
serialization-json = { module = "org.jetbrains.kotlinx:kotlinx-serialization-json", version.ref = "serialization" }
datetime = { module = "org.jetbrains.kotlinx:kotlinx-datetime", version.ref = "datetime" }
jna-platform = { module = "net.java.dev.jna:jna-platform", version.ref = "jna" }
turbine = { module = "app.cash.turbine:turbine", version.ref = "turbine" }

[plugins]
kotlin-multiplatform = { id = "org.jetbrains.kotlin.multiplatform", version.ref = "kotlin" }
kotlin-serialization = { id = "org.jetbrains.kotlin.plugin.serialization", version.ref = "kotlin" }
compose-compiler = { id = "org.jetbrains.kotlin.plugin.compose", version.ref = "kotlin" }
compose-multiplatform = { id = "org.jetbrains.compose", version.ref = "compose-multiplatform" }
android-application = { id = "com.android.application", version.ref = "agp" }
android-kmp-library = { id = "com.android.kotlin.multiplatform.library", version.ref = "agp" }
ksp = { id = "com.google.devtools.ksp", version.ref = "ksp" }
room = { id = "androidx.room", version.ref = "room" }
roborazzi = { id = "io.github.takahirom.roborazzi", version.ref = "roborazzi" }
```

- [ ] **Step 1: Write the smoke test** — `class SmokeTest { @Test fun moduleNameIsCore() = assertEquals("core", MODULE_NAME) }`
- [ ] **Step 2: Run** `cd native && ./gradlew :shared:core:allTests` → FAIL (no build yet), then create the skeleton.
- [ ] **Step 3: Verify every target builds**

```bash
cd native && ./gradlew :shared:core:allTests :androidApp:assembleDebug :desktopApp:run --dry-run :shared:app:linkDebugFrameworkIosSimulatorArm64
```
Expected: BUILD SUCCESSFUL. Then `./gradlew :desktopApp:run` shows a window reading "Nomi"; `xcodebuild -project iosApp/iosApp.xcodeproj -scheme iosApp -destination 'platform=iOS Simulator,name=iPhone 17' build` succeeds.

- [ ] **Step 4: Commit** — `git commit -m "Scaffold native/ Kotlin Multiplatform project for Android, iOS and desktop"`

### Task B2: Token generator — `material3.css` + `shapes.ts` → Kotlin

**Files:**
- Create: `native/scripts/gen-tokens.mjs` (Node 20, no dependencies)
- Create (generated, committed): `native/shared/designsystem/src/commonMain/kotlin/ai/nomi/designsystem/generated/NomiTokens.kt`
- Create: `native/scripts/gen-tokens.test.mjs` (`node --test`)
- Modify: `.github/workflows/test.yml` (job step: `node native/scripts/gen-tokens.mjs --check` fails when the committed file is stale)

**Interfaces:**
- Produces in `NomiTokens.kt`:
  - `object NomiPalettes { val all: Map<String, NomiPalette> }` keyed by accent (`"green"` default, `"coral-reef"`, … every `data-color` in the CSS), `NomiPalette(light: NomiRoles, dark: NomiRoles)`; `NomiRoles` holds every `--md-sys-color-*` role as `Long` ARGB plus `ground`, `stage`, `onStage`, `action1..3`, `onAction`.
  - `object NomiGradients { val glow/ember/tide/sky/bloom/dusk/citrus/slate: GradientSpec }` with `GradientSpec(angleDeg: Float, stops: List<Pair<Float, Long>>, onColor: Long)`.
  - `object NomiMotionTokens { spatialFast/spatialDefault/spatialSlow/effectsFast/effectsDefault: MotionSpec }` with `MotionSpec(durationMs: Int, x1, y1, x2, y2: Float)`.
  - `object NomiShapeSpecs { val all: Map<String, PolarSpec> }`, `PolarSpec(lobes: Int, amplitude: Float, phase: Float)`, plus `const val SHAPE_POINTS = 120`, `SHAPE_RADIUS = 20f`, `SHAPE_CENTER = 24f`; and `agentShape`/`agentTone` maps (Nomi→cookie9/glow, money→sunny8/ember, coding→cookie6/tide, planning→clover4/sky, personality→flower5/bloom).
- `color-mix(in srgb, A p%, B)` values are resolved by the script (linear sRGB channel mix, rounded), so Kotlin never sees CSS functions.

- [ ] **Step 1: Failing tests** (`gen-tokens.test.mjs`):

```js
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseCss, resolveColorMix, parseShapes } from './gen-tokens.mjs';

test('default light primary is the seed scheme value', () => {
  const t = parseCss(`:root { --md-sys-color-primary: #0b6b4a; }`);
  assert.equal(t.green.light['primary'], 0xff0b6b4a);
});
test('color-mix 3% primary into white', () => {
  assert.equal(resolveColorMix('#0b6b4a', 3, '#ffffff'), 0xfff8fbfa);
});
test('gradient stops parse with positions', () => {
  const t = parseCss(`:root { --nomi-gradient-glow: linear-gradient(135deg, #e4f76a 0%, #5be08f 55%, #1db9a0 100%); }`);
  assert.deepEqual(t.gradients.glow.stops, [[0, 0xffe4f76a], [0.55, 0xff5be08f], [1, 0xff1db9a0]]);
  assert.equal(t.gradients.glow.angle, 135);
});
test('shape specs parse from shapes.ts', () => {
  const s = parseShapes(`const SPECS = { cookie9: { lobes: 9, amplitude: 0.09 }, clover3: { lobes: 3, amplitude: 0.19, phase: -Math.PI / 2 } };`);
  assert.deepEqual(s.cookie9, { lobes: 9, amplitude: 0.09, phase: 0 });
  assert.ok(Math.abs(s.clover3.phase + Math.PI / 2) < 1e-9);
});
```

(Compute the exact expected `color-mix` value once with the implementation and pin it; the number above is the arithmetic for 3% `#0b6b4a` over white: r = 0.03·11+0.97·255 = 247.68→248 (f8), g = 0.03·107+0.97·255 = 250.56→251 (fb), b = 0.03·74+0.97·255 = 249.57→250 (fa).)

- [ ] **Step 2: Run** `node --test native/scripts/` → FAIL.
- [ ] **Step 3: Implement** the parser (regex over `:root`, `@media (prefers-color-scheme: dark) :root:not(...)`, `:root[data-theme='dark']`, `:root[data-color='X']`, `:root[data-color='X'][data-theme='dark']` blocks; `var(--x)` resolved within the same scheme), the shapes reader, and a Kotlin emitter. `--check` mode regenerates to memory and exits 1 with a diff hint if it differs from the committed file.
- [ ] **Step 4: Run** `node --test native/scripts/ && node native/scripts/gen-tokens.mjs` → PASS; generated file compiles: `./gradlew :shared:designsystem:compileKotlinDesktop`.
- [ ] **Step 5: Commit** — `git commit -m "Generate Compose design tokens from the web's material3.css and shapes.ts"`

### Task B3: `NomiTheme` on `MaterialExpressiveTheme`

**Files:**
- Create: `native/shared/designsystem/src/commonMain/kotlin/ai/nomi/designsystem/theme/{NomiTheme.kt, NomiColors.kt, NomiTypography.kt, NomiMotion.kt, NomiGradient.kt}`
- Create: `native/shared/designsystem/src/commonMain/composeResources/font/{bricolage_grotesque_variable.ttf, instrument_sans_variable.ttf, jetbrains_mono_variable.ttf}` (download from Google Fonts' GitHub `google/fonts` repo, OFL — commit the OFL.txt files alongside)
- Test: `native/shared/designsystem/src/commonTest/kotlin/ai/nomi/designsystem/theme/NomiColorsTest.kt`, desktop screenshot test `src/desktopTest/.../ThemeScreenshotTest.kt`

**Interfaces:**
- `@Composable fun NomiTheme(accent: String = "green", dark: Boolean = isSystemInDarkTheme(), reducedMotion: Boolean = false, content: @Composable () -> Unit)`
- `fun NomiRoles.toColorScheme(): ColorScheme`
- `object NomiTheme { val gradients: NomiGradientSet @Composable get; val extendedColors: NomiExtendedColors @Composable get }` — `NomiExtendedColors(ground, stage, onStage, actionBrush: Brush, onAction)`.
- `fun GradientSpec.toBrush(size: Size): Brush` (angle → start/end offsets, same convention as CSS `linear-gradient(135deg, …)`: 0deg points up, angles clockwise).
- `MaterialExpressiveTheme(colorScheme, motionScheme = if (reducedMotion) MotionScheme.standard() else MotionScheme.expressive(), typography, shapes)`.
- Typography: display*/headline* = Bricolage Grotesque (ExtraBold for display), title*/body*/label* = Instrument Sans; `NomiTheme.metaStyle` = JetBrains Mono `labelSmall` metrics.

- [ ] **Step 1: Failing tests**

```kotlin
class NomiColorsTest {
    @Test fun defaultLightPrimaryMatchesWeb() {
        val scheme = NomiPalettes.all.getValue("green").light.toColorScheme()
        assertEquals(Color(0xFF0B6B4A), scheme.primary)
    }
    @Test fun everyAccentHasLightAndDark() {
        NomiPalettes.all.forEach { (name, p) -> assertNotEquals(p.light.primary, p.dark.primary, name) }
    }
    @Test fun gradient135degRunsTopLeftToBottomRight() {
        val (start, end) = gradientEndpoints(angleDeg = 135f, size = Size(100f, 100f))
        assertTrue(start.x < end.x && start.y < end.y)
    }
}
```

- [ ] **Step 2: Run** `./gradlew :shared:designsystem:desktopTest` → FAIL.
- [ ] **Step 3: Implement**, then add a Roborazzi desktop capture of a `ThemePreview` (Display/Headline/Title/Body/Meta text, primary/secondary/tertiary buttons, one gradient button, cards on ground) for `green` light+dark → `src/desktopTest/snapshots/theme_{light,dark}.png`; review the images by eye against the web app.
- [ ] **Step 4: Run** `./gradlew :shared:designsystem:desktopTest :shared:designsystem:recordRoborazziDesktop` → PASS.
- [ ] **Step 5: Commit** — `git commit -m "NomiTheme: M3 Expressive theme from generated Nomi tokens"`

### Task B4: Agent shapes, morphing and `AgentShape`

**Files:**
- Create: `native/shared/designsystem/src/commonMain/kotlin/ai/nomi/designsystem/shape/{PolarShape.kt, AgentIdentity.kt, AgentShape.kt, MorphingShape.kt, NomiFace.kt}`
- Test: `.../commonTest/.../shape/PolarShapeTest.kt`, screenshot `AgentShapeScreenshotTest.kt`

**Interfaces:**
- `fun polarPoints(spec: PolarSpec, points: Int = SHAPE_POINTS): FloatArray` — interleaved x,y in the 48×48 box, formula `r(θ) = R·(1 + a·cos(lobes·θ + phase))` exactly as `frontend/src/lib/components/m3/shapes.ts`.
- `fun lerpPoints(a: FloatArray, b: FloatArray, t: Float): FloatArray`
- `class PolarShape(spec: PolarSpec) : androidx.compose.ui.graphics.Shape` (scales the 48-box outline into the layout size).
- `data class AgentIdentity(val shape: String, val tone: String, val motion: ShapeMotion)`; `fun agentIdentity(agentType: String, dynamic: AgentIdentity? = null): AgentIdentity` (fallback = Nomi).
- `@Composable fun AgentShape(identity: AgentIdentity, size: Dp, working: Boolean = false, face: Boolean = false, lookAt: Offset? = null, modifier: Modifier = Modifier)` — fills with the tone gradient; when `working`, runs `identity.motion` (spin = infinite rotation, wobble/bounce/pulse/orbit per `SHAPE_MOTIONS`), disabled under reduced motion; `face` draws Nomi's eyes, offset toward `lookAt` (normalised −1..1) — used by the desktop island (E3) and Android bubble (F2).
- `@Composable fun MorphingShape(from: PolarSpec, to: PolarSpec, progress: Float, brush: Brush, modifier: Modifier)`.

- [ ] **Step 1: Failing tests**

```kotlin
class PolarShapeTest {
    @Test fun everyShapeHasSamePointCount() {
        NomiShapeSpecs.all.values.forEach { assertEquals(SHAPE_POINTS * 2, polarPoints(it).size) }
    }
    @Test fun circleIsRadiusFromCenter() {
        val p = polarPoints(NomiShapeSpecs.all.getValue("circle"))
        for (i in p.indices step 2) {
            val r = hypot(p[i] - SHAPE_CENTER, p[i + 1] - SHAPE_CENTER)
            assertEquals(SHAPE_RADIUS, r, 1e-3f)
        }
    }
    @Test fun firstPointMatchesWebFormula() {
        // shapes.ts: θ0 = 0 → x = 24 + 20·(1 + 0.09), y = 24
        val p = polarPoints(NomiShapeSpecs.all.getValue("cookie9"))
        assertEquals(24f + 20f * 1.09f, p[0], 1e-3f); assertEquals(24f, p[1], 1e-3f)
    }
    @Test fun lerpHalfwayIsMidpoint() {
        val a = floatArrayOf(0f, 0f); val b = floatArrayOf(10f, 20f)
        assertContentEquals(floatArrayOf(5f, 10f), lerpPoints(a, b, 0.5f))
    }
    @Test fun unknownAgentFallsBackToNomi() {
        assertEquals("cookie9", agentIdentity("does-not-exist").shape)
    }
}
```

Before writing `firstPointMatchesWebFormula`, read `shapes.ts` lines 80–140 to confirm the start angle and sampling direction; adjust the expected point if the web starts at −π/2.

- [ ] **Step 2: Run → FAIL. Step 3: Implement. Step 4: Run → PASS**, record screenshots of all 15 shapes × 8 tones grid and a working-state frame.
- [ ] **Step 5: Commit** — `git commit -m "Agent shapes in Compose: polar outlines, morphing, AgentShape with face"`

### Task B5: Nomi components on M3 Expressive

**Files:**
- Create under `native/shared/designsystem/src/commonMain/kotlin/ai/nomi/designsystem/component/`: `NomiButton.kt` (adds `gradient` variant over M3 `Button` with `ButtonDefaults.shapes()` press-morph), `SendButton.kt`, `NomiLoadingIndicator.kt`, `NomiWavyProgress.kt`, `NomiCard.kt`, `NomiChip.kt`, `NomiButtonGroup.kt`, `NomiSplitButton.kt`, `NomiSnackbar.kt`, `NomiTextField.kt`, `Composer.kt` (floating auto-growing input, Enter sends on desktop / Shift+Enter newline; IME action on mobile)
- Test: desktop screenshot tests for each + `SendButtonTest.kt` (state machine)

Mapping (use the M3 Expressive Compose component where one exists; wrap only to apply Nomi tokens):

| Web (`frontend/src/lib/components/m3`) | Compose |
|---|---|
| `Button` (S/M/L/XL, gradient) | `Button`/`FilledTonalButton` + `ButtonDefaults.shapes()`; sizes via `ButtonDefaults.{Extra}*ContainerHeight`; gradient = container transparent + `Modifier.background(actionBrush, shape)` |
| `IconButton selected` | `IconToggleButton` with `IconButtonDefaults.shapes()` (round → square) |
| `LoadingIndicator` | `LoadingIndicator(polygons = listOf(cookie9, sunny8, cookie6, clover4, flower5).map { it.toRoundedPolygon() })` — add `PolarSpec.toRoundedPolygon()` using `androidx.graphics.shapes.RoundedPolygon(vertices)` |
| `WavyProgress` | `LinearWavyProgressIndicator` |
| `ButtonGroup` | `ButtonGroup` / `ToggleButton` with `ButtonGroupDefaults.connected*Shapes()` |
| `SplitButton` | `SplitButtonLayout` |
| `SendButton` | custom: `AgentShape(nomi, 56.dp)` button; when `sending`, becomes `NomiLoadingIndicator` |
| `Chip` | `AssistChip` / `FilterChip` / `SuggestionChip` |
| `Snackbar` | `Snackbar` with `inverseSurface` |
| `BottomSheet` / `SideSheet` / `Dialog` / `Menu` / `DatePicker` / `TimePicker` | M3 equivalents, no wrapper |

- [ ] **Step 1: Failing test** — `SendButtonTest`: `sendButtonState(text = "", sending = false) == Disabled`; `("hi", false) == Ready`; `("hi", true) == Sending`; `("", true) == Sending`.
- [ ] **Step 2–4:** implement, screenshot every component light/dark at 390dp width, review against the web.
- [ ] **Step 5: Commit** — `git commit -m "Nomi components on M3 Expressive for Compose"`

---

# Phase C — Network, auth, RealtimeHub, Room, Koin, Navigation 3

### Task C1: API client and token storage

**Files:**
- Create: `native/shared/network/src/commonMain/kotlin/ai/nomi/network/{NomiHttpClient.kt, ApiConfig.kt, TokenStore.kt, AuthApi.kt, dto/AuthDto.kt}`
- Create actuals: `androidMain/.../TokenStore.android.kt` (DataStore + Tink AEAD, keyset in Android Keystore), `iosMain/.../TokenStore.ios.kt` (Keychain `kSecClassGenericPassword`, service `ai.nomi.app.tokens`), `desktopMain/.../TokenStore.desktop.kt` (`com.github.javakeyring:java-keyring:1.0.4` → macOS Keychain / Windows Credential Manager / libsecret; when unavailable fall back to a 0600 file in the app data dir and log a warning)
- Test: `commonTest/.../NomiHttpClientTest.kt` with `ktor-client-mock`

**Interfaces:**
- `interface TokenStore { suspend fun read(): Tokens?; suspend fun write(tokens: Tokens); suspend fun clear() }`, `@Serializable data class Tokens(val access: String, val refresh: String)`
- `fun nomiHttpClient(engine: HttpClientEngine, config: ApiConfig, tokens: TokenStore, onSignedOut: () -> Unit): HttpClient` — installs ContentNegotiation(json), WebSockets(`pingInterval = 25.seconds`), Auth bearer: `loadTokens` from store; `refreshTokens` posts `/api/auth/refresh {refresh_token}` → `{access_token}`; on refresh 401 clears tokens and calls `onSignedOut`.
- `class AuthApi(client) { suspend fun login(email, password): SignInResult; suspend fun logout() }` — `SignInResult` mirrors `SignInResponse` (tokens or email-code challenge, `routes/auth.rs`).
- `ApiConfig(baseUrl: String)`; `baseUrl.toWebSocketUrl()` maps `https→wss`, `http→ws`.

- [ ] **Step 1: Failing tests:** (a) a 401 on `GET /api/home` triggers exactly one `/api/auth/refresh` then retries with the new token; (b) a 401 from refresh calls `onSignedOut` once and clears the store; (c) `ApiConfig("https://nomi.example").toWebSocketUrl("/api/realtime/ws") == "wss://nomi.example/api/realtime/ws"`.
- [ ] **Step 2–4:** run `./gradlew :shared:network:allTests` FAIL → implement → PASS.
- [ ] **Step 5: Commit** — `git commit -m "Native API client with bearer auth, refresh and secure token storage"`

### Task C2: Realtime wire model (Kotlin)

**Files:**
- Create: `native/shared/realtime/src/commonMain/kotlin/ai/nomi/realtime/{Topic.kt, Frames.kt, Events.kt}`
- Test: `commonTest/.../FramesTest.kt`

**Interfaces:**
- `sealed interface Topic { val wire: String; data object User; data class Chat(val sessionId: String); data class Project(val projectId: String); companion object { fun parse(wire: String): Topic? } }`
- `@Serializable sealed interface ClientFrame` (`@SerialName("subscribe") Subscribe(topic)`, `unsubscribe`, `ping`; class discriminator `"op"`)
- `@Serializable sealed interface ServerFrame` (`ready`, `subscribed`, `unsubscribed`, `denied`, `event(topic, payload: JsonObject)`, `error`, `pong`; discriminator `"type"`)
- `sealed interface RealtimeEvent { val topic: Topic }` decoded from `event.payload["kind"]`: `ChatDelta(sessionId, turnJobId, event: JsonObject)`, `TurnCompleted(sessionId, turnJobId, messageId)`, `TurnFailed`, `MessageCreated`, `MessageUpdated`, `SessionRenamed`, `PersonalityChanged`, `AgentPhaseChanged(agentSessionId, phase, detail)`, `AgentSessionStarted`, `AgentSessionEnded`, `AgentDelegationUpdated`, `DeployChanged(projectId, event)`, `NotificationCreated(...)`, `ChatReply(sessionId, messageId, preview)`, `Resynced(topic)` (synthetic, emitted by the hub after reconnect), `AccessLost(topic)` (synthetic, from a `denied` frame), `Unknown(topic, kind, raw)`.
- `fun decodeEvent(topic: Topic, payload: JsonObject): RealtimeEvent`

- [ ] **Step 1: Failing tests:** each `StreamEnvelope` variant from `backend/crates/nomi-realtime/src/lib.rs` decodes from a JSON literal copied from a serde round-trip (write the literals by hand from the enum definition, e.g. `{"kind":"TurnCompleted","turn_job_id":"…","message_id":"…"}`); an unknown `kind` yields `Unknown`; `Topic.parse("chat:abc")` → `Chat("abc")`, `Topic.parse("user")` → `User`, `Topic.parse("x")` → null.
- [ ] **Step 2–4:** FAIL → implement → PASS (`./gradlew :shared:realtime:allTests`).
- [ ] **Step 5: Commit** — `git commit -m "Kotlin realtime wire model matching the gateway and StreamEnvelope"`

### Task C3: `RealtimeHub` — the single socket, listeners and Flow adapters

**Files:**
- Create: `native/shared/realtime/src/commonMain/kotlin/ai/nomi/realtime/{RealtimeHub.kt, RealtimeTransport.kt, KtorRealtimeTransport.kt, Backoff.kt}`
- Test: `commonTest/.../RealtimeHubTest.kt`, `FakeTransport.kt`

**Interfaces:**

```kotlin
fun interface RealtimeListener { fun onEvent(event: RealtimeEvent) }

interface ListenerHandle { fun remove() }

sealed interface ConnectionState {
    data object Idle : ConnectionState            // signed out / stopped
    data object Connecting : ConnectionState
    data class Connected(val userId: String) : ConnectionState
    data class Waiting(val retryInMs: Long, val attempt: Int) : ConnectionState
}

interface RealtimeHub {
    val state: StateFlow<ConnectionState>
    fun start()                                    // idempotent; called on sign-in / app start
    fun stop()                                     // sign-out; drops all subscriptions server-side
    fun addListener(topic: Topic, listener: RealtimeListener): ListenerHandle
    fun events(topic: Topic): Flow<RealtimeEvent>  // cold; adds a listener on collect, removes on cancel
}

/** One connection attempt; the hub owns retries. */
interface RealtimeTransport {
    suspend fun connect(): RealtimeSession        // throws UnauthorizedException on 401
}
interface RealtimeSession {
    val incoming: Flow<ServerFrame>               // completes when the socket closes
    suspend fun send(frame: ClientFrame)
    suspend fun close()
}

class DefaultRealtimeHub(
    private val transport: RealtimeTransport,
    private val refreshToken: suspend () -> Boolean,   // from network module; false = signed out
    private val scope: CoroutineScope,
    private val backoff: Backoff = Backoff(),
) : RealtimeHub
```

Behaviour (each line is a test):
- First listener on a topic sends `Subscribe`; the second does not. Removing one of two listeners sends nothing; removing the last sends `Unsubscribe`. `User` topic is never subscribed/unsubscribed (implicit).
- Events dispatch to listeners of their topic, in arrival order, on `scope`'s dispatcher (Main for UI). A listener that throws is logged and skipped; others still receive the event.
- On socket close: state → `Waiting(backoff)`; on reconnect, re-sends `Subscribe` for every topic with ≥1 listener, then emits `Resynced(topic)` to each topic's listeners (and to `User`).
- `UnauthorizedException` on connect → call `refreshToken()` once; if true, retry immediately; if false, `stop()` and state → `Idle`. Two consecutive 401s after a successful refresh → `Idle` (no loop).
- `Denied` frame → listeners of that topic get `RealtimeEvent.AccessLost(topic)` so a chat screen can show "You no longer have access"; the topic stays registered locally but is not re-sent on reconnect until a listener is added again.
- Client sends `Ping` every `PING_INTERVAL_MS = 25_000`; if no frame arrives for `STALE_AFTER_MS = 60_000`, the session is closed and the reconnect path runs.
- `Backoff`: exponential `500ms · 2^attempt`, cap `30_000ms`, full jitter, reset after a connection stays up `STABLE_AFTER_MS = 10_000`.
- Thread-safety: all hub state is confined to one actor coroutine (a `Channel<Command>`); public methods only enqueue.

- [ ] **Step 1: Write failing tests** with `FakeTransport` (records sent frames, lets the test push `ServerFrame`s and close the socket) and `runTest` + `StandardTestDispatcher`:

```kotlin
class RealtimeHubTest {
    private val chat = Topic.Chat("s1")

    @Test fun subscribes_once_for_two_listeners() = runTest {
        val (hub, t) = connectedHub()
        hub.addListener(chat) {}; hub.addListener(chat) {}
        advanceUntilIdle()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
    }

    @Test fun unsubscribe_only_when_last_listener_removed() = runTest {
        val (hub, t) = connectedHub()
        val a = hub.addListener(chat) {}; val b = hub.addListener(chat) {}
        advanceUntilIdle(); a.remove(); advanceUntilIdle()
        assertTrue(t.sent.none { it is ClientFrame.Unsubscribe })
        b.remove(); advanceUntilIdle()
        assertEquals(ClientFrame.Unsubscribe("chat:s1"), t.sent.last())
    }

    @Test fun throwing_listener_does_not_break_others() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { error("boom") }
        hub.addListener(chat) { got += it }
        advanceUntilIdle()
        t.push(eventFrame("chat:s1", """{"kind":"SessionRenamed","title":"x"}"""))
        t.push(eventFrame("chat:s1", """{"kind":"SessionRenamed","title":"y"}"""))
        advanceUntilIdle()
        assertEquals(listOf("x", "y"), got.map { (it as RealtimeEvent.SessionRenamed).title })
    }

    @Test fun reconnect_resubscribes_and_emits_resynced() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { got += it }
        advanceUntilIdle(); t.sent.clear()
        t.dropConnection(); advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext(userId = "u1"); advanceUntilIdle()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
        assertTrue(got.any { it is RealtimeEvent.Resynced })
    }

    @Test fun unauthorized_upgrade_refreshes_token_once_then_stops() = runTest {
        var refreshes = 0
        val t = FakeTransport(failWith401Times = Int.MAX_VALUE)
        val hub = DefaultRealtimeHub(t, refreshToken = { refreshes++; true }, scope = backgroundScope)
        hub.start(); advanceUntilIdle()
        assertEquals(1, refreshes)
        assertEquals(ConnectionState.Idle, hub.state.value)
    }

    @Test fun stale_socket_is_recycled() = runTest {
        val (hub, t) = connectedHub()
        advanceTimeBy(DefaultRealtimeHub.STALE_AFTER_MS + 1); runCurrent()
        assertTrue(t.closedCount >= 1)
        assertTrue(hub.state.value is ConnectionState.Waiting || hub.state.value is ConnectionState.Connecting)
    }

    @Test fun flow_adapter_removes_listener_on_cancel() = runTest {
        val (hub, t) = connectedHub()
        val job = launch { hub.events(chat).collect {} }
        advanceUntilIdle(); job.cancel(); advanceUntilIdle()
        assertEquals(ClientFrame.Unsubscribe("chat:s1"), t.sent.last())
    }

    @Test fun denied_emits_access_lost() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { got += it }; advanceUntilIdle()
        t.push(ServerFrame.Denied("chat:s1")); advanceUntilIdle()
        assertEquals(RealtimeEvent.AccessLost(chat), got.single())
    }
}
```

(`connectedHub()` builds the hub with a `FakeTransport`, calls `start()`, pushes `ServerFrame.Ready("u1")`, `advanceUntilIdle()`, returns both. `FakeTransport` lives in `commonTest`.)

- [ ] **Step 2: Run** `./gradlew :shared:realtime:desktopTest` → FAIL.
- [ ] **Step 3: Implement** `DefaultRealtimeHub` (actor loop) and `KtorRealtimeTransport(client: HttpClient, config: ApiConfig)` using `client.webSocketSession(config.toWebSocketUrl("/api/realtime/ws"))`, mapping a 401 `WebSocketException`/`ClientRequestException` to `UnauthorizedException`, decoding text frames with the C2 serializers.
- [ ] **Step 4: Run** → PASS. Manual check: `./gradlew :desktopApp:run` with a debug overlay printing `hub.state` while stopping/starting the local backend.
- [ ] **Step 5: Commit** — `git commit -m "RealtimeHub: one socket, ref-counted topics, listeners and reconnect"`

### Task C4: Room database

**Files:**
- Create: `native/shared/database/src/commonMain/kotlin/ai/nomi/database/{NomiDatabase.kt, entity/*.kt, dao/*.kt, DatabaseFactory.kt}`; `androidMain/desktopMain/iosMain` `DatabaseFactory` actuals (`Room.databaseBuilder<NomiDatabase>(name = <platform data dir>/nomi.db)`, `.setDriver(BundledSQLiteDriver())`, `.setQueryCoroutineContext(Dispatchers.IO)`)
- Test: `desktopTest/.../DaoTest.kt` (in-memory `Room.inMemoryDatabaseBuilder`)

**Entities (v1):** `SessionEntity(id PK, title, lastMessagePreview, updatedAt, unread: Boolean)`, `MessageEntity(id PK, sessionId idx, role, agentType, agentDisplayName, contentJson, createdAt, status)`, `StreamingDraftEntity(sessionId PK, turnJobId, text, phase, updatedAt)` (in-flight reply, deleted on `TurnCompleted`), `NotificationEntity(id PK, kind, title, body, link, createdAt, readAt)`, `AgentActivityEntity(agentSessionId PK, sessionId, agentType, phase, detail, updatedAt)`.

**DAOs:** `SessionDao.observeAll(): Flow<List<SessionEntity>>`, `upsert`, `markRead`; `MessageDao.observe(sessionId): Flow<List<MessageEntity>>`, `upsert(list)`, `delete(id)`; `StreamingDraftDao.observe(sessionId)`, `appendDelta(sessionId, turnJobId, text)` (`@Transaction`), `clear(sessionId)`; `NotificationDao.observeUnreadCount(): Flow<Int>`, `observeAll()`, `upsert`; `AgentActivityDao.observeActive(): Flow<List<AgentActivityEntity>>`.

- [ ] Steps: failing DAO tests (upsert then observe emits; `appendDelta` concatenates across calls; `clear` removes; unread count updates) → implement → `./gradlew :shared:database:desktopTest` PASS → commit `"Room database for sessions, messages, drafts, notifications and agent activity"`.

### Task C5: Repositories — realtime events into Room

**Files:**
- Create: `native/shared/data/src/commonMain/kotlin/ai/nomi/data/{SessionRepository.kt, ChatRepository.kt, NotificationRepository.kt, ActivityRepository.kt, RealtimeSync.kt}`, REST APIs in `network` (`SessionsApi`, `MessagesApi`, `NotificationsApi`) mirroring `/api/sessions`, `/api/sessions/:id/messages`, `/api/notifications`
- Test: `commonTest/.../ChatRepositoryTest.kt`, `RealtimeSyncTest.kt` with fake APIs + in-memory DB (desktopTest)

**Interfaces:**
- `class ChatRepository(api, db, hub) { fun observe(sessionId): Flow<ChatState>; fun bind(sessionId): ListenerHandle; suspend fun send(sessionId, text) }` — `bind` registers a hub listener on `Topic.Chat(sessionId)`: `ChatDelta` → `StreamingDraftDao.appendDelta`; `TurnCompleted`/`MessageCreated`/`MessageUpdated` → fetch message by id (`/api/sessions/:id/messages/:mid` — the rendered-message endpoint used by `frontend/src/lib/server/fetchRenderedMessage.ts`) → upsert, clear draft; `Resynced` → refetch latest page; `AgentPhaseChanged` → `AgentActivityDao`.
- `class RealtimeSync(hub, sessionRepo, notificationRepo, activityRepo)` — app-wide listener on `Topic.User` (registered once at start): `NotificationCreated` → upsert notification; `ChatReply` → update session preview + `unread = true` unless that chat is currently open (`ForegroundChat` StateFlow set by the chat screen); `Resynced` → refetch sessions + notifications. Exposes `val companionSignals: SharedFlow<CompanionSignal>` consumed by island/bubble (E/F): `NewReply(sessionId, preview, agentType)`, `Notification(title, body, link)`, `Working(agentType, phase)`, `Idle`.
- `ChatState(messages: List<Message>, draft: Draft?, activeAgent: AgentActivity?)`.

- [ ] Steps: failing tests — `resynced_event_refetches_open_chat`, `delta_then_completed_leaves_one_message_and_no_draft`, `chat_reply_for_open_chat_is_not_unread`, `notification_created_increments_unread` → implement → PASS → commit `"Repositories: realtime events refetch into Room"`.

### Task C6: Koin graph, ViewModels and the Navigation 3 shell

**Files:**
- Create: `native/shared/app/src/commonMain/kotlin/ai/nomi/app/{di/AppModules.kt, nav/NomiRoutes.kt, nav/NomiNavDisplay.kt, NomiApp.kt}`; platform `initKoin(platformModule: Module)` called from `Application.onCreate` (Android), desktop `main`, iOS `MainViewController`
- Test: `commonTest/.../KoinGraphTest.kt` (`verify()` the modules with `koin-test`), `desktopTest/.../NavigationTest.kt` (compose UI test: back stack push/pop)

**Interfaces:**

```kotlin
@Serializable sealed interface NomiRoute : NavKey
@Serializable data object SignIn : NomiRoute
@Serializable data object Home : NomiRoute
@Serializable data object Chats : NomiRoute
@Serializable data class Chat(val sessionId: String) : NomiRoute
@Serializable data object Notifications : NomiRoute
@Serializable data object Settings : NomiRoute
```

- `val coreModule = module { single { Json { ignoreUnknownKeys = true } }; single { nomiHttpClient(get(), get(), get(), onSignedOut = { get<SessionGate>().signOut() }) }; single<RealtimeTransport> { KtorRealtimeTransport(get(), get()) }; single<RealtimeHub> { DefaultRealtimeHub(get(), refreshToken = { get<AuthRepository>().refresh() }, scope = get(named("app"))) }; single { RealtimeSync(get(), get(), get(), get()) }; … }`
- `val featureModule = module { viewModelOf(::HomeViewModel); viewModel { (id: String) -> ChatViewModel(id, get(), get()) }; … }`
- `NomiNavDisplay(backStack: SnapshotStateList<NomiRoute>)` using `NavDisplay(backStack, onBack = { backStack.removeLastOrNull() }, entryDecorators = listOf(rememberSaveableStateHolderNavEntryDecorator(), rememberViewModelStoreNavEntryDecorator()), entryProvider = entryProvider { entry<Home> { HomeScreen(koinViewModel()) }; entry<Chat> { key -> ChatScreen(koinViewModel { parametersOf(key.sessionId) }) }; … })`. On wide windows (`currentWindowAdaptiveInfo().windowSizeClass` ≥ medium) use a list-detail scene strategy so Chats + Chat show side by side (desktop/tablet).
- Deep links: `fun NomiRoute.Companion.fromLink(link: String): NomiRoute?` (`/chat/{id}` → `Chat`, `/notifications` → `Notifications`, others → `Home`), used by island click, bubble tap, push tap.
- ViewModel ↔ realtime: `ChatViewModel.init { handle = chatRepository.bind(sessionId) }`, `onCleared { handle.remove() }` — the ViewModel store per nav entry guarantees the listener lives exactly as long as the screen is on the back stack.

- [ ] Steps: failing `KoinGraphTest` + `NavigationTest` (`fromLink("/chat/abc") == Chat("abc")`; pushing Chat then back returns to Home; a `ChatViewModel` is cleared → its listener removed, asserted via a fake hub) → implement → PASS → commit `"Koin graph, ViewModels and Navigation 3 shell"`.

---

# Phase D — Screens (expand into its own plan before executing)

Parity target is the web app's routes under `frontend/src/routes`. Order and per-screen deliverable:

1. **Sign in** (`feature/auth`): email + password, email-code challenge, Google via system browser + `ai.nomi.app://auth` redirect (Android App Link / iOS ASWebAuthenticationSession / desktop loopback `http://127.0.0.1:<port>/callback`). Uses `AuthShell` look: dark `stage` ground with the agent crew shapes.
2. **Home**: greeting in the user's timezone, composer that creates the chat and sends the first message (`newChat` with `text`), suggestion chips, "Your crew" card, recent chats — honour the configurable home layouts (`frontend/src/lib/homeLayouts.ts`, `/api/home` layout field, migration `0070_home_layout.sql`).
3. **Chats** list: rows, unread state from Room, pull-to-refresh on mobile.
4. **Chat**: editorial assistant turns with `AgentShape` avatars, primary user bubbles with tight corner, streaming draft rendering, morphing loader while working, `AgentPhaseChanged` status line, floating `Composer`, attachments (system picker → `/api/attachments`). **Content blocks** (`docs/superpowers/specs/2026-09-07-rich-content-blocks-design.md`): approval (ember), to-do (wavy progress + check marks), plan, file, and a generic fallback card for unknown block types. Markdown via `com.mikepenz:multiplatform-markdown-renderer-m3`.
5. **Notifications** inbox with mark-read / mark-all-read.
6. **Settings**: accent + light/dark (ButtonGroup + morphing swatches, same as web Preferences), reduced motion, companion toggles (island on/off, bubble on/off), sign out (deletes device via `DELETE /api/devices/:id`).

Each screen task: ViewModel unit tests (state from fake repos), Roborazzi screenshots light/dark at 390dp and 1280dp, `frontend/CLAUDE.md` checklist review.

---

# Phase E — Desktop: always-on background app + Dynamic Island (expand into its own plan)

### E1: Background process, tray, autostart
- `desktopApp/Main.kt`: `application(exitProcessOnExit = false)` with `Tray(icon = nomiTrayIcon, menu = { Item("Open Nomi"); Item("Pause island"); Separator(); Item("Quit") })`; main window `Window(visible = mainVisible, onCloseRequest = { mainVisible = false })` — closing hides, Quit exits. Single instance via a lock file + loopback socket that forwards "show" to the running instance.
- macOS: `compose.desktop.application.nativeDistributions.macOS.infoPlist { extraKeysRawXml = "<key>LSUIElement</key><true/>" }` (no Dock icon; Dock icon appears only while the main window is open via `NSApp.setActivationPolicy` through JNA). Autostart: `SMAppService.mainApp.register()` via JNA ObjC runtime (`objc_msgSend`); fallback LaunchAgent plist in `~/Library/LaunchAgents/ai.nomi.app.plist`.
- Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Nomi = "<install>\Nomi.exe" --background` via `Advapi32Util.registrySetStringValue`.
- Linux: write `~/.config/autostart/ai.nomi.app.desktop` with `X-GNOME-Autostart-enabled=true`.
- Tests: `AutostartTest` per OS with a temp HOME / fake registry adapter (`interface RegistryAdapter`).

### E2: The island window
- `IslandWindow`: `Window(undecorated = true, transparent = true, alwaysOnTop = true, resizable = false, focusable = false)` sized to the *expanded* bounds; content draws the pill at the current animated size; the rest is transparent and click-through.
- Placement (`IslandGeometry.kt`, pure + unit-tested): top-center of the screen that contains the cursor (`GraphicsEnvironment.screenDevices` bounds); on a Mac with a notch, collapsed width = notch width + `ISLAND_NOTCH_MARGIN_DP` on each side and y = 0 (read `NSScreen.auxiliaryTopLeftArea/auxiliaryTopRightArea` via JNA; no notch → y = menu bar height + `ISLAND_TOP_GAP_DP`). Re-place on display change (`GraphicsEnvironment` listener poll every `DISPLAY_POLL_MS`).
- `MacWindowInterop` (JNA → ObjC on `ComposeWindow.windowHandle`→ `NSView.window`): `setLevel: NSStatusWindowLevel+1` (above the menu bar), `setCollectionBehavior: canJoinAllSpaces | stationary | fullScreenAuxiliary | ignoresCycle`, `setHasShadow: NO`, `setIgnoresMouseEvents:` toggled by hover (E3).
- `WindowsWindowInterop`: `SetWindowLongPtr(GWL_EXSTYLE, WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_LAYERED)`, toggle `WS_EX_TRANSPARENT` for click-through.
- `LinuxWindowInterop`: X11 → `_NET_WM_WINDOW_TYPE_DOCK`, `_NET_WM_STATE_ABOVE`, `_NET_WM_STATE_STICKY`, input shape via XShape for click-through; Wayland (`XDG_SESSION_TYPE == "wayland"` or `WAYLAND_DISPLAY` set) → island disabled, tray + notifications (`notify-send`/`org.freedesktop.Notifications` over D-Bus) instead; Settings shows why.
- Tests: `IslandGeometryTest` (notch, no notch, two monitors with cursor on the second, scaled 150% display), `PlatformCapabilitiesTest` (Wayland detection).

### E3: Cursor tracking, hover and island states
- `CursorTracker(scope)`: polls `MouseInfo.getPointerInfo()?.location` (null on Wayland → tracker reports `Unavailable`) at `CURSOR_POLL_NEAR_MS = 16` when within `NEAR_DISTANCE_DP = 160` of the island, else `CURSOR_POLL_FAR_MS = 100`; emits `StateFlow<CursorSample(x, y, insideIsland, distance)>`. No permission is needed on any OS for position.
- Hover: entering the pill's hit rect → expand (spring `MotionScheme.expressive().defaultSpatialSpec()`); leaving + `COLLAPSE_DELAY_MS = 400` → collapse. Click-through is enabled whenever the cursor is outside the pill's current rect, so the transparent window never blocks the menu bar.
- Nomi's face looks toward the cursor while it is within `NEAR_DISTANCE_DP` (`AgentShape(lookAt = …)` from B4) — the "interactive when mouse hovering" moment.
- `CompanionStateMachine` (in `shared/companion`, pure, shared with Android bubble): inputs = `CompanionSignal` (C5) + hover/tap; states `Idle` (collapsed, Nomi shape breathing), `Working(agentType, phase)` (agent shape spinning, phase text), `NewReply(sessionId, preview)` (expanded for `PEEK_MS = 4000`, then badge), `Notification(title, body, link)`, `Expanded(content)` (hover); `Snoozed` from the tray's "Pause island". Unit tests cover every transition, including "reply arrives while hovered" (stays expanded, content swaps) and "working ends with reply" (Working → NewReply).
- Expanded content: latest reply preview with "Open" / quick reply composer (sends via `ChatRepository.send`), or notification with its link; click opens the main window at `NomiRoute.fromLink`.

---

# Phase F — Android: always-on service + chat-head bubble (expand into its own plan)

### F1: `NomiPresenceService`
- Foreground service, `android:foregroundServiceType="specialUse"` with `<property android:name="android.app.PROPERTY_SPECIAL_USE_FGS_SUBTYPE" android:value="Keeps the Nomi chat head and live replies connected"/>`; permissions `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_SPECIAL_USE`, `POST_NOTIFICATIONS`, `RECEIVE_BOOT_COMPLETED`, `SYSTEM_ALERT_WINDOW`. Play Console needs the special-use declaration and a video — note in the release checklist (H).
- Started from `Application.onCreate` when signed in and "Chat head" is on, and from `BootReceiver`. Holds no socket of its own: it calls `get<RealtimeHub>().start()` — the same Koin singleton the Activity uses (one process), so the app and bubble share **one** socket.
- Ongoing notification is low-importance, channel `presence`, shows the Nomi shape icon and "Nomi is around"; tapping opens the app.
- When the OS kills the service (doze, OEM killers), FCM data messages (A4) post normal notifications and restart the service on the next app open. `FirebaseMessagingService.onNewToken` → `POST /api/devices`.

### F2: The bubble overlay
- `BubbleController` adds a `ComposeView` to `WindowManager` with `TYPE_APPLICATION_OVERLAY`, `FLAG_NOT_FOCUSABLE | FLAG_LAYOUT_NO_LIMITS`, `WRAP_CONTENT`. The view gets a hand-rolled `OverlayLifecycleOwner : LifecycleOwner, SavedStateRegistryOwner, ViewModelStoreOwner` set via `setViewTreeLifecycleOwner` / `setViewTreeSavedStateRegistryOwner` / `setViewTreeViewModelStoreOwner` (Compose crashes without them outside an Activity).
- Head: `AgentShape(nomi, BUBBLE_SIZE_DP = 60, face = true, working = state is Working)` with an unread badge. Drag updates `WindowManager.LayoutParams.x/y` via `updateViewLayout`; release → fling with `SpringSpec` to the nearest screen edge (Messenger behaviour); dragging shows a dismiss target at the bottom center — dropping on it hides the bubble until the next new event (or turns it off if held `LONG_DISMISS_MS`).
- New reply / notification: a speech-bubble card animates out beside the head (`AnimatedVisibility` + expressive spatial spring) showing preview text for `PEEK_MS`; tap → opens a compact chat panel overlay (focusable window, `FLAG_NOT_FOCUSABLE` cleared while the panel is open so the keyboard works) with the last messages + `Composer`; "Open in app" → `MainActivity` with the deep link.
- Hidden while the app is in the foreground (`ProcessLifecycleOwner` observer), re-shown on background.
- Pure logic tested on JVM: `BubbleSnapTest` (edge choice, clamping to insets/cutouts), `BubbleDismissTest`, reuse `CompanionStateMachine` tests.

### F3: Permission and fallback
- Onboarding sheet explains the chat head and sends the user to `Settings.ACTION_MANAGE_OVERLAY_PERMISSION` for `package:ai.nomi.app`; re-checks `Settings.canDrawOverlays` on resume.
- Denied → Android Bubbles API: notifications for chat replies carry `Notification.BubbleMetadata` with a dynamic shortcut per chat and a `BubbleActivity` hosting the compact chat panel. Same visuals, but only appears on new messages (OS rule).

---

# Phase G — iOS: app, Live Activity / Dynamic Island, push (expand into its own plan)

- **App**: `iosApp` SwiftUI `@main` hosts `ComposeUIViewController { NomiApp() }`; `UNUserNotificationCenter` permission; `registerForRemoteNotifications` → APNs token → `POST /api/devices {platform:"ios", push_token}`; tap on notification → deep link into `NomiRoute.fromLink`. The `RealtimeHub` runs only while the app is active (`scenePhase == .active` → `start()`, `.background` → `stop()`).
- **Live Activity extension** (`iosApp/NomiLiveActivity/`, SwiftUI — ActivityKit has no Compose path): `NomiActivityAttributes { sessionId, agentType }`, `ContentState { phase: String, preview: String?, updatedAt: Date }`. Dynamic Island: compact leading = Nomi shape (SwiftUI `Shape` port of the polar formula with `NomiShapeSpecs` values emitted by the B2 generator as a Swift file `NomiShapes.generated.swift` — extend `gen-tokens.mjs` with a `--swift` emitter and a parity test that the first point matches the Kotlin value), compact trailing = phase glyph, expanded = preview + "Open". Lock Screen view = same with the stage ground.
- **Lifecycle**: when the user sends a message from the app, the app starts a Live Activity locally (`Activity.request(pushType: .token)`) and posts its update token to `POST /api/devices` (`live_activity_start_token` column also stores the push-to-start token from `Activity<…>.pushToStartTokenUpdates`, iOS 17.2+). The backend (A4 `PushKind::LiveActivityUpdate`) pushes `AgentPhaseChanged` → phase, `ChatReply` → preview + `event: "end"` with `dismissal-date` = now + `LIVE_ACTIVITY_LINGER_SECONDS` (900). Notifications from the user topic with no running activity use push-to-start (`PushKind::LiveActivityStart`).
- **In-app bubble**: while the app is open on a screen other than the chat that replied, show an in-app floating `AgentShape` head (shared `companion` UI) with the same peek card — gives the Messenger feel inside the app.
- Tests: Swift unit tests for `ContentState` decoding of the backend payload; Kotlin tests for the scenePhase → hub start/stop bridge.

---

# Phase H — CI, packaging, release (expand into its own plan)

- `.github/workflows/native.yml`: `ubuntu-latest` → `./gradlew allTests :androidApp:assembleRelease :desktopApp:packageDeb verifyRoborazziDesktop` + `node native/scripts/gen-tokens.mjs --check`; `macos-latest` → `:desktopApp:packageDmg`, iOS simulator build + `xcodebuild test`; `windows-latest` → `:desktopApp:packageMsi`.
- Signing: Android upload key (secrets), macOS Developer ID + notarization (`notarize` in compose desktop `macOS.notarization`), Windows code signing cert, iOS via fastlane match. Desktop auto-update via a signed update feed (separate plan).
- Store checklist: Play special-use FGS declaration + `SYSTEM_ALERT_WINDOW` justification; App Store Live Activity + push entitlements (`aps-environment`, `com.apple.developer.usernotifications.time-sensitive` not needed); macOS Login Item approval UX.

---

## Execution notes

- Phases A and B have no shared files and can run in parallel sessions; C needs both.
- Within C, C1/C2 are independent; C3 needs C1 (`UnauthorizedException`, client) and C2; C4 is independent of C1–C3; C5 needs C3+C4; C6 needs all.
- After each phase: run the full backend suite (`cd backend && cargo test`) when backend changed, and `cd native && ./gradlew allTests` for native.
