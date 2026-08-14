# WebSocket Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Rust WebSocket endpoint, `GET /api/sessions/:id/ws`, that authorizes the caller the same way `send_message`/`list_messages` do, then relays a session's `chat/{session_id}/stream` MQTT topic to the socket verbatim, for the life of the connection.

**Architecture:** Reuse the existing `AuthClaims` + `authorize_session_access` gate in `backend/src/routes/sessions.rs`. On successful upgrade, each connection opens its own `rumqttc::AsyncClient`/`EventLoop` pair (mirroring `MqttPublisher::connect`), subscribes to exactly one topic, and forwards every `Publish` payload as a WS text frame in a `tokio::select!` loop that also watches for the client closing the socket. No connection registry, no replay buffer.

**Tech Stack:** `axum` 0.7 `ws` feature (extractor `WebSocketUpgrade`, currently NOT enabled — must be added), `rumqttc` (already a dependency), `tokio-tungstenite` as a new dev-dependency for tests (a real WS client standing in for the future SvelteKit proxy).

## Global Constraints

- No new auth mechanism: the handler takes `AuthClaims` + `Path<Uuid>` + `WebSocketUpgrade` as normal extractors and calls the existing `authorize_session_access(&state.pool, claims.sub, session_id)` before upgrading — same 401/404 semantics as `send_message`/`list_messages`.
- Each WebSocket connection opens its own MQTT broker connection (no shared/multiplexed subscriber) — accepted limitation, not in scope to optimize.
- QoS 0 (`QoS::AtMostOnce`) on subscribe, matching `MqttPublisher::publish`'s QoS on the publish side.
- The MQTT payload (already `StreamEnvelope` JSON bytes) is forwarded verbatim as a WS text frame — no reparsing or re-serialization in the relay itself.
- No replay/catch-up buffering on (re)connect — a client only sees events published after it connects. This is intentional per the spec (§3) and out of scope to change.
- `MQTT_BROKER_HOST`/`MQTT_BROKER_PORT` default to `localhost`/`1883`, read once at process startup (mirroring `backend/src/bin/worker.rs:26-30`) and stored on `AppState`, not re-read per connection.
- The connection stays open for the life of the session view (not scoped to one turn) — it must relay events from a second, later turn without the client reconnecting (tested explicitly in Task 3).

---

## File Structure

- Modify `backend/Cargo.toml`: enable axum's `ws` feature; add `tokio-tungstenite` as a dev-dependency.
- Modify `backend/src/app.rs`: add `mqtt_broker_host: String` and `mqtt_broker_port: u16` to `AppState`; add the new route.
- Modify `backend/src/routes/sessions.rs`: add `session_stream` handler + `relay_session_stream` relay function.
- Modify `backend/src/main.rs`: read `MQTT_BROKER_HOST`/`MQTT_BROKER_PORT` env vars, pass into `AppState`.
- Modify `backend/tests/support/mod.rs`: add `TEST_MQTT_BROKER_HOST`/`TEST_MQTT_BROKER_PORT` constants (mirrors the existing `TEST_SETTINGS_KEY` convention) so every test file's `AppState` literal has one place to source these two new fields from.
- Modify `backend/tests/sessions_routes.rs`, `backend/tests/settings_routes.rs`, `backend/tests/auth_extractor.rs`, `backend/tests/auth_routes.rs`: these four files each construct an `AppState` literal directly — add the two new fields to each, sourced from `support::TEST_MQTT_BROKER_HOST`/`support::TEST_MQTT_BROKER_PORT`.
- Create `backend/tests/session_ws.rs`: integration tests for the new endpoint (auth gating in Task 2, relay behavior in Task 3), using a real bound `TcpListener` + `axum::serve` (not `tower::oneshot`, which can't drive a persistent duplex WebSocket) and `tokio-tungstenite` as the client.

---

### Task 1: Wire MQTT broker config through `AppState`

**Files:**
- Modify: `backend/Cargo.toml`
- Modify: `backend/src/app.rs`
- Modify: `backend/src/main.rs`
- Modify: `backend/tests/support/mod.rs`
- Modify: `backend/tests/sessions_routes.rs:13-20`, `backend/tests/settings_routes.rs:14-21`, `backend/tests/auth_extractor.rs:14-19`, `backend/tests/auth_routes.rs:12-19`

**Interfaces:**
- Produces: `AppState { pool, jwt_secret, http_client, settings_key, mqtt_broker_host: String, mqtt_broker_port: u16 }` — Task 2 and Task 3's test helpers construct this literal.
- Produces: `support::TEST_MQTT_BROKER_HOST: &str = "localhost"`, `support::TEST_MQTT_BROKER_PORT: u16 = 1883` — used by every test file's `test_state()`.

This task is pure plumbing (no new runtime behavior yet), so its "test" is that the whole workspace still compiles with the new required `AppState` fields threaded everywhere they're needed.

- [ ] **Step 1: Enable axum's `ws` feature and add the `tokio-tungstenite` dev-dependency**

Edit `backend/Cargo.toml`:

```toml
axum = { version = "0.7", features = ["ws"] }
```

(replaces the current bare `axum = "0.7"` line)

Add to `[dev-dependencies]`:

```toml
tokio-tungstenite = "0.24"
```

- [ ] **Step 2: Add the two MQTT fields to `AppState`**

Edit `backend/src/app.rs`, add to the `AppState` struct:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
}
```

- [ ] **Step 3: Read the env vars in `main.rs` and populate `AppState`**

Edit `backend/src/main.rs`, mirroring `backend/src/bin/worker.rs:26-30`:

```rust
let mqtt_broker_host = var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
let mqtt_broker_port: u16 = var("MQTT_BROKER_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(1883);
```

(add near the existing `database_url`/`jwt_secret`/`settings_key` reads; `main.rs` already has `use std::env::var;`)

Then add the two fields to the `AppState { ... }` literal further down:

```rust
let state = nomi_orchestrator::app::AppState {
    pool,
    jwt_secret,
    http_client,
    settings_key,
    mqtt_broker_host,
    mqtt_broker_port,
};
```

- [ ] **Step 4: Add shared test constants**

Edit `backend/tests/support/mod.rs`, add near `TEST_SETTINGS_KEY`:

```rust
pub const TEST_MQTT_BROKER_HOST: &str = "localhost";
pub const TEST_MQTT_BROKER_PORT: u16 = 1883;
```

- [ ] **Step 5: Update every existing `AppState` literal to include the new fields**

In each of `backend/tests/sessions_routes.rs`, `backend/tests/settings_routes.rs`, `backend/tests/auth_routes.rs` (all three have a `fn test_state(pool: PgPool) -> AppState { AppState { ... } }`), add:

```rust
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
```

right after the existing `settings_key: ...,` line (before the closing `}`).

In `backend/tests/auth_extractor.rs:14-19` (inline `let state = AppState { ... };`), add the same two lines in the same place.

- [ ] **Step 6: Verify the workspace compiles**

Run: `cd backend && cargo build --tests`
Expected: builds cleanly, 0 errors. (There is no `session_stream`/`relay_session_stream` yet, and no route added yet — this step only proves the config plumbing didn't break anything that already existed.)

- [ ] **Step 7: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/app.rs backend/src/main.rs backend/tests/support/mod.rs backend/tests/sessions_routes.rs backend/tests/settings_routes.rs backend/tests/auth_extractor.rs backend/tests/auth_routes.rs
git commit -m "chore: wire MQTT broker config through AppState, enable axum ws feature"
```

---

### Task 2: Add the WebSocket endpoint with auth gating

**Files:**
- Modify: `backend/src/routes/sessions.rs`
- Modify: `backend/src/app.rs`
- Test: `backend/tests/session_ws.rs` (new)

**Interfaces:**
- Consumes: `AppState` from Task 1 (`state.mqtt_broker_host`, `state.mqtt_broker_port`), `authorize_session_access(&state.pool, claims.sub, session_id) -> Result<(), (StatusCode, &'static str)>` (`backend/src/routes/sessions.rs:99-117`, unchanged), `AuthClaims` extractor (`backend/src/auth/extractor.rs`, unchanged).
- Produces: `pub async fn session_stream(...)`, wired into the router as `GET /api/sessions/:id/ws` — Task 3's tests connect to this route and drive real turns through it.

This task proves the auth gate: an upgrade attempt with no token, or with a token for a session in a different org, is rejected before any MQTT subscription happens — mirrors `send_message`'s existing 401/404 tests (`backend/tests/sessions_routes.rs:124-129`, `190-208`).

- [ ] **Step 1: Write the failing tests**

Create `backend/tests/session_ws.rs`:

```rust
mod support;

use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use nomi_orchestrator::app::{build_router, AppState};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
    }
}

/// Binds a real TCP listener and serves the app on it — a WebSocket upgrade is a persistent
/// duplex connection, which `tower::oneshot` (used by the other route tests) cannot drive.
/// Returns (http_base_url, ws_base_url).
async fn spawn_app(pool: PgPool) -> (String, String) {
    let app = build_router(test_state(pool));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), format!("ws://{addr}"))
}

async fn register_and_login(client: &reqwest::Client, base: &str, email: &str) -> String {
    client
        .post(format!("{base}/api/auth/register"))
        .json(&json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }))
        .send()
        .await
        .unwrap();
    let login: Value = client
        .post(format!("{base}/api/auth/login"))
        .json(&json!({ "email": email, "password": "correct-password" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    login["access_token"].as_str().unwrap().to_string()
}

async fn create_session(client: &reqwest::Client, base: &str, token: &str) -> Uuid {
    let body: Value = client
        .post(format!("{base}/api/sessions"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    Uuid::parse_str(body["session_id"].as_str().unwrap()).unwrap()
}

async fn try_connect_ws(
    ws_base: &str,
    session_id: Uuid,
    token: Option<&str>,
) -> Result<
    (
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    let url = format!("{ws_base}/api/sessions/{session_id}/ws");
    let mut request = url.into_client_request().unwrap();
    if let Some(token) = token {
        request
            .headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(request).await
}

fn assert_rejected_with(err: tokio_tungstenite::tungstenite::Error, expected_status: u16) {
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => {
            assert_eq!(response.status().as_u16(), expected_status);
        }
        other => panic!("expected an HTTP rejection with status {expected_status}, got {other:?}"),
    }
}

#[sqlx::test]
async fn ws_upgrade_without_a_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let err = try_connect_ws(&ws_base, Uuid::new_v4(), None).await.unwrap_err();
    assert_rejected_with(err, 401);
}

#[sqlx::test]
async fn ws_upgrade_for_another_orgs_session_is_rejected(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token_a = register_and_login(&client, &http_base, "a@example.com").await;
    let token_b = register_and_login(&client, &http_base, "b@example.com").await;
    let session_id = create_session(&client, &http_base, &token_a).await;

    let err = try_connect_ws(&ws_base, session_id, Some(&token_b)).await.unwrap_err();
    assert_rejected_with(err, 404);
}

#[sqlx::test]
async fn ws_upgrade_with_a_valid_token_for_the_callers_own_session_succeeds(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "c@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (ws, response) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();
    assert_eq!(response.status().as_u16(), 101);
    drop(ws);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test session_ws`
Expected: compile failure — `session_stream` doesn't exist yet, and `/api/sessions/:id/ws` isn't a route (`build_router` won't reference it). This confirms the tests are exercising code that doesn't exist yet.

- [ ] **Step 3: Implement the handler and relay function**

Edit `backend/src/routes/sessions.rs`, add near the top:

```rust
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
```

Add at the end of the file:

```rust
pub async fn session_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let broker_host = state.mqtt_broker_host.clone();
    let broker_port = state.mqtt_broker_port;
    Ok(ws.on_upgrade(move |socket| relay_session_stream(socket, session_id, broker_host, broker_port)))
}

async fn relay_session_stream(mut socket: WebSocket, session_id: Uuid, broker_host: String, broker_port: u16) {
    let mut options = MqttOptions::new(format!("ws-bridge-{}", Uuid::new_v4()), broker_host, broker_port);
    options.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(options, 16);
    let topic = format!("chat/{session_id}/stream");
    if client.subscribe(&topic, QoS::AtMostOnce).await.is_err() {
        return; // socket closes on drop
    }

    loop {
        tokio::select! {
            event = eventloop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        if socket.send(Message::Text(String::from_utf8_lossy(&publish.payload).into_owned())).await.is_err() {
                            break; // client disconnected
                        }
                    }
                    Ok(_) => continue,
                    Err(_) => break, // broker connection lost; let the client reconnect
                }
            }
            incoming = socket.recv() => {
                if incoming.is_none() { break; } // client closed the socket
            }
        }
    }
}
```

Edit `backend/src/app.rs`, add the route (alongside the existing `/api/sessions/:id/messages` route):

```rust
        .route("/api/sessions/:id/ws", get(sessions_routes::session_stream))
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test session_ws`
Expected: all three tests pass. (Requires a running local Postgres + EMQX, same as every other `sqlx::test` in this suite — see `backend/docker-compose.yml`.)

- [ ] **Step 5: Commit**

```bash
git add backend/src/routes/sessions.rs backend/src/app.rs backend/tests/session_ws.rs
git commit -m "feat: add GET /api/sessions/:id/ws WebSocket endpoint with auth gating"
```

---

### Task 3: Relay MQTT `StreamEnvelope` events over the WebSocket

**Files:**
- Test: `backend/tests/session_ws.rs` (extend from Task 2)

**Interfaces:**
- Consumes: `session_stream`/`relay_session_stream` from Task 2 (unchanged); `nomi_orchestrator::turn::queue::claim_next(&PgPool) -> Result<Option<ClaimedJob>, TurnError>` where `ClaimedJob { id, session_id, sender_channel_identity_id, text, org_id_hint }` (`backend/src/turn/queue.rs:6-12,40-63`); `nomi_orchestrator::turn::process_turn(pool, mqtt, provider, embedding_provider, turn_job_id, session_id, sender_channel_identity_id, user_id, text) -> Result<TurnOutcome, TurnError>` (`backend/src/turn/mod.rs:77-125`); `nomi_orchestrator::realtime::{MqttPublisher, StreamEnvelope}`; `nomi_orchestrator::llm::{StreamEvent, PartialBlock, StopReason}`; `support::{FakeLlmProvider, FakeEmbeddingProvider, dummy_embedding}` (`backend/tests/support/mod.rs`).
- Produces: nothing further downstream — this is the last task, proving the endpoint end-to-end.

This proves the two behaviors the design spec calls out in §4: the exact ordered sequence of `StreamEnvelope` frames for one turn (`Delta` events matching `FakeLlmProvider`'s canned stream, then `TurnCompleted`), and that a connection which stays open across two separate turns on the same session receives both turns' events without reconnecting (proving the session-scoped, not per-turn, lifecycle from §3).

- [ ] **Step 1: Write the failing test for a single turn**

Append to `backend/tests/session_ws.rs`:

```rust
use std::time::Duration as StdDuration;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, PartialBlock, StopReason, StreamEvent};
use nomi_orchestrator::realtime::{MqttPublisher, StreamEnvelope};
use nomi_orchestrator::turn::{process_turn, queue};

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Claims the next pending turn job, resolves its user_id the same way
/// backend/src/bin/worker.rs does, runs it through process_turn with a fake provider, and
/// publishes TurnCompleted afterwards — mirroring the worker's own success path
/// (backend/src/bin/worker.rs:91-104), since process_turn itself only publishes on failure.
async fn run_one_claimed_turn(pool: &PgPool, reply_text: &str) -> Uuid {
    let claimed = queue::claim_next(pool).await.unwrap().unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(claimed.sender_channel_identity_id)
        .fetch_one(pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response(reply_text));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let mqtt = MqttPublisher::connect(
        support::TEST_MQTT_BROKER_HOST,
        support::TEST_MQTT_BROKER_PORT,
        &format!("test-ws-relay-{}", Uuid::new_v4()),
    );

    process_turn(
        pool,
        &mqtt,
        &provider,
        &embedder,
        claimed.id,
        claimed.session_id,
        claimed.sender_channel_identity_id,
        user_id,
        &claimed.text,
    )
    .await
    .unwrap();

    mqtt.publish(
        claimed.session_id,
        &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: Uuid::nil() },
    )
    .await
    .unwrap();

    claimed.id
}

fn expected_frames_for(turn_job_id: Uuid, reply_text: &str) -> Vec<StreamEnvelope> {
    vec![
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text },
        },
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::TextDelta { index: 0, text: reply_text.to_string() },
        },
        StreamEnvelope::Delta { turn_job_id, event: StreamEvent::ContentBlockDone { index: 0 } },
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 10, output_tokens: 5 },
        },
        StreamEnvelope::TurnCompleted { turn_job_id, message_id: Uuid::nil() },
    ]
}

async fn recv_n_frames(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    n: usize,
) -> Vec<StreamEnvelope> {
    let mut received = Vec::new();
    for _ in 0..n {
        let msg = tokio::time::timeout(StdDuration::from_secs(5), ws.next())
            .await
            .expect("timed out waiting for a frame")
            .expect("stream ended")
            .unwrap();
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            received.push(serde_json::from_str::<StreamEnvelope>(&text).unwrap());
        }
    }
    received
}

#[sqlx::test]
async fn relays_delta_and_turn_completed_events_for_a_single_turn(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "d@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (mut ws, _) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "hello" }))
        .send()
        .await
        .unwrap();

    let turn_job_id = run_one_claimed_turn(&pool, "hi there").await;

    let received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(received, expected_frames_for(turn_job_id, "hi there"));
}

#[sqlx::test]
async fn relays_events_across_two_turns_without_reconnecting(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "e@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (mut ws, _) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "first" }))
        .send()
        .await
        .unwrap();
    let first_turn_job_id = run_one_claimed_turn(&pool, "first reply").await;
    let first_received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(first_received, expected_frames_for(first_turn_job_id, "first reply"));

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "second" }))
        .send()
        .await
        .unwrap();
    let second_turn_job_id = run_one_claimed_turn(&pool, "second reply").await;
    let second_received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(second_received, expected_frames_for(second_turn_job_id, "second reply"));

    assert_ne!(first_turn_job_id, second_turn_job_id);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test session_ws relays_`
Expected: FAIL (or hang until the 5s per-frame timeout panics) — nothing is publishing to `chat/{session_id}/stream` yet via a fully-wired path at this point in a fresh checkout... actually by Task 2's end the endpoint exists, so this step is really about confirming the *new* test code itself is correct by watching it run against the real implementation. If Task 2 was done correctly this may already pass; if so, treat this run as the "make it pass" step directly and skip to Step 4. If it fails, debug per Step 3 below before concluding the relay is broken.

- [ ] **Step 3: Fix forward, if needed**

If Step 2 fails, the most likely causes are: (a) `relay_session_stream`'s `client.subscribe` not yet acked before the test's `POST /messages` + `run_one_claimed_turn` publish — `rumqttc` queues the publish either way and the broker will deliver once the SubAck lands, so this should not race in practice, but if it does, the fix is on the test side (add a short `tokio::time::sleep` after the WS connects and before triggering the turn), not the relay implementation, since the spec explicitly does not want the relay itself to synchronize on SubAck. (b) A mismatch between the expected frame sequence and `response_to_stream`'s actual output in `backend/src/llm/mod.rs` — re-read it and adjust `expected_frames_for` to match, don't change production code to match a wrong test expectation.

- [ ] **Step 4: Run the full test file to verify everything passes**

Run: `cd backend && cargo test --test session_ws`
Expected: all 5 tests pass (3 from Task 2, 2 from this task).

- [ ] **Step 5: Run the full backend test suite to confirm no regressions**

Run: `cd backend && cargo test`
Expected: all tests pass, including the four files touched in Task 1.

- [ ] **Step 6: Commit**

```bash
git add backend/tests/session_ws.rs
git commit -m "test: prove the WebSocket bridge relays ordered StreamEnvelope frames across turns"
```

---

## Out of Scope (unchanged from the spec)

- The SvelteKit-side WebSocket proxy and any browser-facing rendering (sub-project 5).
- Replay/catch-up buffering for reconnects — REST (`GET /api/sessions/:id/messages`) remains the source of truth for missed history.
- A shared/multiplexed MQTT subscriber for scaling beyond today's expected connection counts.
- Reconnect/retry policy on the client side of this endpoint.
