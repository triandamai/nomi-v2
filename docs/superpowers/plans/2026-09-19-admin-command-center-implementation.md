# Admin Command Center Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the static `/admin/agents` page with a live-updating dashboard: a table of
active agents that adds/updates/removes rows in real time, a scrolling activity feed, and a
per-agent metadata drill-down — with message content never reaching the admin browser.

**Architecture:** A new admin-only WebSocket subscribes to the wildcard MQTT topic `chat/+/stream`
(every session's existing per-session topic, in one subscription) and forwards only a
server-side allow-listed set of event kinds carrying metadata (agent identity, phase, timestamps)
— never conversation content. Two new `StreamEnvelope` variants (`AgentSessionStarted`/
`AgentSessionEnded`) close the one real gap in today's realtime coverage: nothing is published
when an agent starts or finishes, only during an in-flight turn. The existing `agent_events`
table backs both the feed's history and the drill-down's per-agent history — no new storage.

**Tech Stack:** Rust/axum/sqlx backend (`backend/crates/*`), SvelteKit 2/Svelte 5 frontend
(`frontend/`), Postgres, the existing MQTT/WS realtime relay (`nomi-realtime`, `rumqttc`), the
existing Node WS proxy (`frontend/ws-proxy/`, `frontend/server.js`, `frontend/vite-plugins/`).

**Spec:** `docs/superpowers/specs/2026-09-19-admin-command-center-design.md`

## Global Constraints

- Message content (`Delta`, `MessageCreated`, `MessageUpdated`, `AgentDelegationUpdated`) must
  never be forwarded to the admin WebSocket — enforced server-side in the relay, not the frontend.
- No new tables. `agent_events` (existing) backs the feed's history and the drill-down.
- Every new MQTT publish is best-effort (never fails the turn), matching every existing publish
  call in this codebase.
- `/admin/agents` is rewritten in place — same URL, same nav entry, no new page alongside it.
- `reason` on `AgentSessionEnded` is one of `"completed"` | `"cancelled"` | `"expired"`, matching
  `agent_sessions.status`'s possible non-active values exactly.

---

### Task 1: `StreamEnvelope` — session lifecycle events

**Files:**
- Modify: `crates/nomi-realtime/src/lib.rs`

**Interfaces:**
- Produces: `StreamEnvelope::AgentSessionStarted { agent_session_id, session_id, agent_type,
  agent_display_name, channel, sender_label }`, `StreamEnvelope::AgentSessionEnded {
  agent_session_id, session_id, reason }`.

- [ ] **Step 1: Add the two variants**

```rust
// crates/nomi-realtime/src/lib.rs — add to the existing StreamEnvelope enum, after AgentPhaseChanged
/// An `agent_sessions` row was created — an agent started working. Every field the admin
/// command center needs to render a new table row is resolved once here (at spawn time,
/// mirroring how `messages.agent_display_name` is resolved at insert time) so the dashboard
/// never needs a follow-up lookup per event.
AgentSessionStarted {
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: String,
    agent_display_name: String,
    channel: String,
    sender_label: String,
},
/// An `agent_sessions` row was closed. `reason` is one of "completed" | "cancelled" |
/// "expired", matching `agent_sessions.status`'s possible non-active values exactly.
AgentSessionEnded { agent_session_id: Uuid, session_id: Uuid, reason: String },
```

- [ ] **Step 2: Compile**

Run: `cargo build -p nomi-realtime`
Expected: clean build (no existing code constructs `StreamEnvelope` exhaustively via a `match`
without a wildcard arm — adding variants to an enum only breaks callers that pattern-match it
exhaustively; if this fails, fix whatever `match` needs a new arm before moving on).

- [ ] **Step 3: Commit**

```bash
cd backend
git add crates/nomi-realtime/src/lib.rs
git commit -m "feat: add AgentSessionStarted/AgentSessionEnded to StreamEnvelope"
```

---

### Task 2: Publish session start/end from `routing.rs`

**Files:**
- Modify: `crates/nomi-turn/src/routing.rs`
- Modify: `crates/nomi-turn/src/lib.rs`

**Interfaces:**
- Consumes: `StreamEnvelope::{AgentSessionStarted, AgentSessionEnded}` (Task 1).
- Produces: `routing::spawn_agent_session(conn, mqtt, session_id, sender_channel_identity_id,
  agent_type, agent_display_name) -> Result<Uuid, TurnError>` (gained `mqtt` and
  `agent_display_name` params). `routing::complete_agent_session(conn, mqtt, agent_session_id,
  session_id, agent_type, status, summary) -> Result<(), TurnError>` (gained `mqtt`).
  `routing::mark_expired(conn, mqtt, agent_session_id, session_id, agent_type) -> Result<(),
  TurnError>` (gained `mqtt`). All three take `mqtt: Option<(&MqttPublisher, Uuid)>`, the exact
  type already used throughout this file's callers.

- [ ] **Step 1: Add the import**

```rust
// crates/nomi-turn/src/routing.rs — add near the top, alongside the existing use block
use nomi_realtime::{MqttPublisher, StreamEnvelope};
```

- [ ] **Step 2: Rewrite `mark_expired`**

```rust
pub async fn mark_expired(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = 'expired', ended_at = now() WHERE id = $1")
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentExpired')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    if let Some((publisher, _)) = mqtt {
        let envelope = StreamEnvelope::AgentSessionEnded { agent_session_id, session_id, reason: "expired".to_string() };
        let _ = publisher.publish(session_id, &envelope).await;
    }

    Ok(())
}
```

- [ ] **Step 3: Rewrite `spawn_agent_session`**

```rust
pub async fn spawn_agent_session(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    agent_type: &str,
    agent_display_name: &str,
) -> Result<Uuid, TurnError> {
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(agent_type)
    .fetch_one(&mut **conn)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentSpawned')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    if let Some((publisher, _)) = mqtt {
        // Best-effort, and only resolved when there's actually a publisher to send it to —
        // mirrors this codebase's existing "MQTT is optional infrastructure" convention.
        let row: Option<(Option<String>, String, String)> = sqlx::query_as(
            "SELECT wc.email, ci.channel, ci.channel_user_id \
             FROM channel_identities ci \
             LEFT JOIN web_credentials wc ON wc.user_id = ci.user_id \
             WHERE ci.id = $1",
        )
        .bind(sender_channel_identity_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten();

        if let Some((email, channel, channel_user_id)) = row {
            let sender_label = email.unwrap_or_else(|| format!("{channel}:{channel_user_id}"));
            let envelope = StreamEnvelope::AgentSessionStarted {
                agent_session_id,
                session_id,
                agent_type: agent_type.to_string(),
                agent_display_name: agent_display_name.to_string(),
                channel,
                sender_label,
            };
            let _ = publisher.publish(session_id, &envelope).await;
        }
    }

    Ok(agent_session_id)
}
```

- [ ] **Step 4: Rewrite `complete_agent_session`**

```rust
pub async fn complete_agent_session(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
    status: &str,
    summary: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = $1, ended_at = now() WHERE id = $2")
        .bind(status)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    let event_type = if status == "cancelled" { "AgentCancelled" } else { "AgentCompleted" };

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .bind(event_type)
        .bind(serde_json::json!({"summary": summary}))
        .execute(&mut **conn)
        .await?;

    if let Some((publisher, _)) = mqtt {
        let reason = if status == "cancelled" { "cancelled" } else { "completed" };
        let envelope = StreamEnvelope::AgentSessionEnded { agent_session_id, session_id, reason: reason.to_string() };
        let _ = publisher.publish(session_id, &envelope).await;
    }

    Ok(())
}
```

- [ ] **Step 5: Update `nomi-turn/src/lib.rs`'s three call sites and `finish_agent_turn`**

`finish_agent_turn` gains an `mqtt: Option<(&MqttPublisher, Uuid)>` parameter (inserted right
after `conn`, matching this file's existing parameter ordering convention):

```rust
async fn finish_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent: &dyn nomi_agent_core::SubAgent,
    outcome: nomi_agent_core::LoopOutcome,
) -> Result<(String, Option<Uuid>), TurnError> {
```

Its `Completed` arm's call to `complete_agent_session` becomes:

```rust
routing::complete_agent_session(conn, mqtt, agent_session_id, session_id, agent.agent_type().as_ref(), &status, &summary).await?;
```

`mark_expired`'s call site (inside `run_locked_turn`) becomes:

```rust
routing::mark_expired(conn, mqtt, agent_session_id, session_id, &details.agent_type).await?;
```

`spawn_agent_session`'s call site (inside `run_locked_turn`, in the `NeedsClassification` branch)
becomes:

```rust
let agent_session_id =
    routing::spawn_agent_session(conn, mqtt, session_id, sender_channel_identity_id, agent.agent_type().as_ref(), agent.display_name().as_ref()).await?;
```

`finish_agent_turn`'s three call sites each gain the `mqtt` argument, using whatever `mqtt` value
is already in scope at each:

- `run_subagent_turn`'s call: `finish_agent_turn(conn, mqtt, session_id, agent_session_id, agent, outcome).await` (that function's own `mqtt: Option<(&MqttPublisher, Uuid)>` parameter, passed straight through).
- `resume_locked`'s `Completed` branch: `finish_agent_turn(conn, Some((mqtt, Uuid::nil())), session_id, agent_session_id, agent.as_ref(), nomi_agent_core::LoopOutcome::Completed { status, summary }).await` (matching the exact `Some((mqtt, Uuid::nil()))` wrapping already used for the `resolve_tool_batch`/`run_agent_turn` calls a few lines above it in that same function — `resume_locked`'s own `mqtt` parameter is a bare `&MqttPublisher`, not an `Option`).
- `resume_locked`'s `Resolved` branch (after the `run_agent_turn` call): `finish_agent_turn(conn, Some((mqtt, Uuid::nil())), session_id, agent_session_id, agent.as_ref(), outcome).await`.

- [ ] **Step 6: Build**

Run: `cargo build -p nomi-turn`
Expected: clean build.

- [ ] **Step 7: Run the full workspace test suite**

Run: `export DATABASE_URL="postgres://nomi:nomi@localhost:5432/nomi" && cargo test --workspace`
(from `backend/`)
Expected: same pass count as before this task — no existing test constructs `spawn_agent_session`/
`complete_agent_session`/`mark_expired` directly (they're only exercised indirectly through
`handle_inbound_message`/`process_turn`/`resume_paused_turn`), so no test file needs a
signature-matching update; a passing count identical to the pre-task baseline confirms nothing
broke.

- [ ] **Step 8: Commit**

```bash
git add crates/nomi-turn/src/routing.rs crates/nomi-turn/src/lib.rs
git commit -m "feat: publish AgentSessionStarted/AgentSessionEnded when agent sessions spawn or close"
```

---

### Task 3: Admin WebSocket relay

**Files:**
- Modify: `crates/nomi-server/src/routes/admin_dashboard.rs`
- Modify: `crates/nomi-server/src/app.rs`
- Test: `crates/nomi-server/tests/admin_agents_stream_ws.rs` (new)

**Interfaces:**
- Consumes: `StreamEnvelope` (Task 1), `require_system_config_permission` (existing,
  `crate::routes::settings`).
- Produces: `GET /api/admin/agents/ws` (WebSocket upgrade), forwarding a JSON-serialized
  `AdminStreamFrame` per allow-listed MQTT publish.

- [ ] **Step 1: Add imports to `admin_dashboard.rs`**

```rust
// crates/nomi-server/src/routes/admin_dashboard.rs — add to the existing use block
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
```

- [ ] **Step 2: Add the frame type, handler, and relay function**

```rust
// crates/nomi-server/src/routes/admin_dashboard.rs — append
/// The wire shape forwarded to an admin's browser — deliberately narrower than
/// `StreamEnvelope`: only these three kinds ever reach this connection, and only with the
/// fields named here. `AgentPhaseChanged`'s `detail` is a tool *name* (safe); nothing here can
/// carry message content — that boundary is enforced in `admin_frame_for` below, not by the
/// frontend choosing not to render something it already received.
#[derive(Serialize)]
#[serde(tag = "kind")]
enum AdminStreamFrame {
    AgentSessionStarted {
        agent_session_id: Uuid,
        session_id: Uuid,
        agent_type: String,
        agent_display_name: String,
        channel: String,
        sender_label: String,
    },
    AgentSessionEnded { agent_session_id: Uuid, session_id: Uuid, reason: String },
    AgentPhaseChanged { agent_session_id: Uuid, session_id: Uuid, phase: String, detail: Option<String> },
}

pub async fn admin_agents_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    let broker_host = state.mqtt_broker_host.clone();
    let broker_port = state.mqtt_broker_port;
    Ok(ws.on_upgrade(move |socket| relay_admin_agents_stream(socket, broker_host, broker_port)))
}

/// Extracts the session id from a `chat/{session_id}/stream` topic — the only way this relay
/// learns which session an `AgentPhaseChanged` event (which doesn't carry `session_id` itself)
/// belongs to, since every session's topic arrives on one wildcard subscription.
fn session_id_from_topic(topic: &str) -> Option<Uuid> {
    topic.strip_prefix("chat/")?.strip_suffix("/stream")?.parse().ok()
}

/// The content boundary: deserializes the raw MQTT payload and maps only the three allow-listed
/// `StreamEnvelope` kinds to `AdminStreamFrame`; every other kind (`Delta`, `MessageCreated`,
/// `MessageUpdated`, `AgentDelegationUpdated`, `TurnCompleted`, `TurnFailed`) returns `None` and
/// is silently dropped by the caller — message content is never even considered for forwarding.
fn admin_frame_for(topic_session_id: Uuid, payload: &[u8]) -> Option<AdminStreamFrame> {
    let envelope: nomi_realtime::StreamEnvelope = serde_json::from_slice(payload).ok()?;
    match envelope {
        nomi_realtime::StreamEnvelope::AgentSessionStarted { agent_session_id, session_id, agent_type, agent_display_name, channel, sender_label } => {
            Some(AdminStreamFrame::AgentSessionStarted { agent_session_id, session_id, agent_type, agent_display_name, channel, sender_label })
        }
        nomi_realtime::StreamEnvelope::AgentSessionEnded { agent_session_id, session_id, reason } => {
            Some(AdminStreamFrame::AgentSessionEnded { agent_session_id, session_id, reason })
        }
        nomi_realtime::StreamEnvelope::AgentPhaseChanged { agent_session_id, phase, detail } => {
            Some(AdminStreamFrame::AgentPhaseChanged { agent_session_id, session_id: topic_session_id, phase, detail })
        }
        _ => None,
    }
}

async fn relay_admin_agents_stream(mut socket: WebSocket, broker_host: String, broker_port: u16) {
    let mut options = MqttOptions::new(format!("admin-agents-ws-bridge-{}", Uuid::new_v4()), broker_host, broker_port);
    options.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(options, 16);
    // Single-level MQTT wildcard: matches chat/{any session_id}/stream in one subscription —
    // this is what makes the relay admin-wide instead of per-session.
    if client.subscribe("chat/+/stream", QoS::AtMostOnce).await.is_err() {
        return; // socket closes on drop
    }

    loop {
        tokio::select! {
            event = eventloop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let Some(session_id) = session_id_from_topic(&publish.topic) else { continue };
                        let Some(frame) = admin_frame_for(session_id, &publish.payload) else { continue };
                        let Ok(json) = serde_json::to_string(&frame) else { continue };
                        if socket.send(Message::Text(json)).await.is_err() {
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

- [ ] **Step 3: Wire the route in `app.rs`**

```rust
// crates/nomi-server/src/app.rs — add next to the existing /api/admin/agents route
.route("/api/admin/agents/ws", get(admin_dashboard_routes::admin_agents_stream))
```

- [ ] **Step 4: Build**

Run: `cargo build -p nomi-server`
Expected: clean build.

- [ ] **Step 5: Write the failing tests**

```rust
// crates/nomi-server/tests/admin_agents_stream_ws.rs
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_realtime::MqttPublisher;
use nomi_server::app::{build_router, AppState};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::{process_turn, queue};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: Arc::new(ToolCatalog::empty()),
    }
}

/// Binds a real TCP listener and serves the app on it — a WebSocket upgrade is a persistent
/// duplex connection, which `tower::oneshot` (used by this crate's other route tests) cannot
/// drive. Returns (http_base_url, ws_base_url).
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

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn try_connect_admin_ws(
    ws_base: &str,
    token: Option<&str>,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Error,
> {
    let url = format!("{ws_base}/api/admin/agents/ws");
    let mut request = url.into_client_request().unwrap();
    if let Some(token) = token {
        request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(request).await.map(|(ws, _)| ws)
}

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

/// Reads WS text frames for up to `timeout`, parsing each as JSON and returning every frame
/// received — used both to assert specific frames arrived (non-empty) and that none did (empty).
async fn collect_frames_within(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    timeout: Duration,
) -> Vec<Value> {
    let mut frames = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, ws.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    frames.push(value);
                }
            }
            Ok(Some(Ok(_))) => continue,
            _ => break,
        }
    }
    frames
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_without_a_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let err = try_connect_admin_ws(&ws_base, None).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => assert_eq!(response.status().as_u16(), 401),
        other => panic!("expected a 401 HTTP rejection, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_from_a_non_admin_is_rejected(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "regular@example.com").await;

    let err = try_connect_admin_ws(&ws_base, Some(&token)).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => assert_eq!(response.status().as_u16(), 403),
        other => panic!("expected a 403 HTTP rejection, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn agent_session_started_and_ended_are_forwarded_live(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let admin_token = register_and_login(&client, &http_base, "admin@example.com").await;
    make_platform_admin(&pool, "admin@example.com").await;
    let mut admin_ws = try_connect_admin_ws(&ws_base, Some(&admin_token)).await.unwrap();

    // A money-intent turn that immediately completes via complete_task — this spawns and then
    // closes an agent_sessions row in one turn, exercising both new envelope kinds.
    let ingested = nomi_turn::ingest::ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        canned_response("money"),
        LlmResponse {
            content: vec![ContentBlock::ToolUse {
                id: "t1".to_string(),
                name: "complete_task".to_string(),
                input: json!({"status": "completed", "summary": "Done"}),
                thought_signature: None,
            }],
            stop_reason: StopReason::ToolUse,
            input_tokens: 1,
            output_tokens: 1,
        },
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect(
        nomi_test_support::TEST_MQTT_BROKER_HOST,
        nomi_test_support::TEST_MQTT_BROKER_PORT,
        &format!("test-command-center-{}", Uuid::new_v4()),
    );

    process_turn(
        &pool, &mqtt, None, &provider, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "how much did I spend?",
    )
    .await
    .unwrap();

    let frames = collect_frames_within(&mut admin_ws, Duration::from_secs(5)).await;
    let started = frames.iter().find(|f| f["kind"] == "AgentSessionStarted").expect("expected an AgentSessionStarted frame");
    assert_eq!(started["agent_type"], "money");
    assert_eq!(started["agent_display_name"], "Money");
    assert_eq!(started["session_id"], ingested.session_id.to_string());

    let ended = frames.iter().find(|f| f["kind"] == "AgentSessionEnded").expect("expected an AgentSessionEnded frame");
    assert_eq!(ended["reason"], "completed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn message_content_never_reaches_the_admin_socket(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let admin_token = register_and_login(&client, &http_base, "admin2@example.com").await;
    make_platform_admin(&pool, "admin2@example.com").await;
    let mut admin_ws = try_connect_admin_ws(&ws_base, Some(&admin_token)).await.unwrap();

    // A plain chitchat turn: publishes Delta chunks and MessageCreated on the per-session topic
    // (and never AgentPhaseChanged, since chitchat has no real agent_sessions row — see
    // engine.rs's update_agent_phase, a no-op UPDATE when agent_session_id == session_id). None
    // of that should ever cross onto the admin-wide relay.
    let ingested = nomi_turn::ingest::ingest_inbound_message(&pool, "telegram", "dm", "chat-2", "tg-2", "hello", None).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("Hi there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect(
        nomi_test_support::TEST_MQTT_BROKER_HOST,
        nomi_test_support::TEST_MQTT_BROKER_PORT,
        &format!("test-command-center-{}", Uuid::new_v4()),
    );

    process_turn(
        &pool, &mqtt, None, &provider, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "hello",
    )
    .await
    .unwrap();

    let frames = collect_frames_within(&mut admin_ws, Duration::from_secs(3)).await;
    assert!(frames.is_empty(), "expected no frames on the admin socket for a chitchat turn, got: {frames:?}");
}
```

Check `crates/nomi-turn/src/ingest.rs`'s exact `ingest_inbound_message` signature and its
returned struct's field names (`turn_job_id`, `session_id`, `sender_channel_identity_id`) before
using them verbatim — this plan's earlier tasks this session already established this exact shape
is used the same way in `nomi-server/tests/session_ws.rs`'s `run_one_claimed_turn` helper; mirror
that file's usage precisely if any field name here doesn't match.

- [ ] **Step 6: Run to verify the tests fail correctly, then pass**

Run: `export DATABASE_URL="postgres://nomi:nomi@localhost:5432/nomi" && cargo test -p nomi-server --test admin_agents_stream_ws`
Expected: all 4 tests pass. (They can't "fail then pass" in the usual TDD sense since the handler
already exists from Steps 1-3 of this task — this is the verification step for code that was
written test-first at the design level but implemented in one pass; run once and confirm green.)

- [ ] **Step 7: Commit**

```bash
git add crates/nomi-server/src/routes/admin_dashboard.rs crates/nomi-server/src/app.rs \
        crates/nomi-server/tests/admin_agents_stream_ws.rs
git commit -m "feat: add admin-wide live agent WebSocket relay"
```

---

### Task 4: Agent-events history endpoint

**Files:**
- Modify: `crates/nomi-server/src/routes/admin_dashboard.rs`
- Modify: `crates/nomi-server/src/app.rs`
- Test: `crates/nomi-server/tests/admin_agent_events_routes.rs` (new)

**Interfaces:**
- Produces: `GET /api/admin/agent-events?limit=<i64>&session_id=<uuid>` (both query params
  optional) → `Vec<AgentEventItem>`.

- [ ] **Step 1: Add imports**

```rust
// crates/nomi-server/src/routes/admin_dashboard.rs — add to the existing use block
use axum::extract::Query;
use serde::Deserialize;
```

(`serde::Serialize` is already imported in this file — add `Deserialize` alongside it, e.g.
change `use serde::Serialize;` to `use serde::{Deserialize, Serialize};`.)

- [ ] **Step 2: Add the query type, response type, and handler**

```rust
// crates/nomi-server/src/routes/admin_dashboard.rs — append
const AGENT_EVENT_TYPES: [&str; 6] =
    ["AgentSpawned", "AgentCompleted", "AgentCancelled", "AgentExpired", "ToolCalled", "AgentReplied"];

#[derive(Deserialize)]
pub struct AgentEventsQuery {
    pub limit: Option<i64>,
    pub session_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct AgentEventItem {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub agent_session_id: Option<Uuid>,
    pub agent_type: Option<String>,
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    /// Only ever a tool *name* — never the tool's `input`/`result`, which is where a tool
    /// call's actual data (file contents, transaction rows) lives. This is the same content
    /// boundary as the live relay, enforced at the same layer: the SQL projection below only
    /// ever reads `payload->>'tool_name'`/`payload->>'is_error'`, never `payload` itself.
    pub tool_name: Option<String>,
    pub is_error: Option<bool>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_agent_events(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<AgentEventsQuery>,
) -> Result<Json<Vec<AgentEventItem>>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let rows: Vec<(Uuid, Option<Uuid>, Option<Uuid>, Option<String>, String, DateTime<Utc>, Option<String>, Option<bool>)> =
        match query.session_id {
            Some(session_id) => sqlx::query_as(
                "SELECT id, session_id, agent_session_id, agent_type, event_type, created_at, \
                        payload->>'tool_name', (payload->>'is_error')::boolean \
                 FROM agent_events \
                 WHERE session_id = $1 AND event_type = ANY($2) \
                 ORDER BY created_at DESC LIMIT $3",
            )
            .bind(session_id)
            .bind(&AGENT_EVENT_TYPES[..])
            .bind(limit)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to list agent events");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agent events")
            })?,
            None => sqlx::query_as(
                "SELECT id, session_id, agent_session_id, agent_type, event_type, created_at, \
                        payload->>'tool_name', (payload->>'is_error')::boolean \
                 FROM agent_events \
                 WHERE event_type = ANY($1) \
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(&AGENT_EVENT_TYPES[..])
            .bind(limit)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to list agent events");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agent events")
            })?,
        };

    let items = rows
        .into_iter()
        .map(|(id, session_id, agent_session_id, agent_type, event_type, created_at, tool_name, is_error)| AgentEventItem {
            id, session_id, agent_session_id, agent_type, event_type, created_at, tool_name, is_error,
        })
        .collect();

    Ok(Json(items))
}
```

- [ ] **Step 3: Wire the route**

```rust
// crates/nomi-server/src/app.rs — add next to /api/admin/agents/ws
.route("/api/admin/agent-events", get(admin_dashboard_routes::list_agent_events))
```

- [ ] **Step 4: Build**

Run: `cargo build -p nomi-server`
Expected: clean build.

- [ ] **Step 5: Write the failing tests**

```rust
// crates/nomi-server/tests/admin_agent_events_routes.rs
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_via_api(router: axum::Router, email: &str) {
    router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
    let (_, body) = json_request_with_body(
        router,
        "POST",
        "/api/auth/login",
        None,
        json!({ "email": email, "password": "correct-password" }),
    )
    .await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn json_request_with_body(router: axum::Router, method: &str, uri: &str, bearer: Option<&str>, body: Value) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/agent-events", Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_recent_events_never_including_tool_input_or_result(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c1') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload) \
         VALUES ($1, 'money', 'ToolCalled', $2)",
    )
    .bind(session_id)
    .bind(json!({"tool_name": "list_transactions", "input": {"limit": 10}, "result": "sensitive data here", "is_error": false}))
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/agent-events", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["tool_name"], "list_transactions");
    assert_eq!(items[0]["is_error"], false);
    assert!(items[0].get("input").is_none());
    assert!(items[0].get("result").is_none());
    assert!(!body.to_string().contains("sensitive data here"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_id_filters_to_one_session(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_a: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'a') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_b: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'b') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, 'money', 'AgentSpawned')")
        .bind(session_a)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, 'money', 'AgentSpawned')")
        .bind(session_b)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(router, "GET", &format!("/api/admin/agent-events?session_id={session_a}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["session_id"], session_a.to_string());
}
```

- [ ] **Step 6: Run to verify they pass**

Run: `export DATABASE_URL="postgres://nomi:nomi@localhost:5432/nomi" && cargo test -p nomi-server --test admin_agent_events_routes`
Expected: all 3 tests pass.

- [ ] **Step 7: Run the full workspace test suite**

Run: `cargo test --workspace` (from `backend/`, with `DATABASE_URL` set)
Expected: baseline count + 4 (Task 3) + 3 (this task) new tests, all passing.

- [ ] **Step 8: Commit**

```bash
git add crates/nomi-server/src/routes/admin_dashboard.rs crates/nomi-server/src/app.rs \
        crates/nomi-server/tests/admin_agent_events_routes.rs
git commit -m "feat: add GET /api/admin/agent-events history endpoint"
```

---

### Task 5: Generalize the Node WS proxy

**Files:**
- Modify: `frontend/ws-proxy/session-stream-proxy.js` → rename to `frontend/ws-proxy/ws-proxy.js`
- Modify: `frontend/vite-plugins/dev-ws-proxy.ts`
- Modify: `frontend/server.js`

**Interfaces:**
- Produces: `attachWsProxy(server: http.Server, options?: ProxyOptions)` — replaces
  `attachSessionStreamProxy`, matching every browser-facing WS path in `ROUTES` below instead of
  hardcoding one.

**Context:** The browser's `WebSocket()` constructor cannot attach an `Authorization` header, so
every WS route goes through this proxy: it reads the `access_token` cookie off the incoming
browser upgrade request, connects to the Rust backend with a `Bearer` header attached, and relays
bidirectionally with reconnect-with-backoff. Today this only matches `/chat/:sessionId/ws`. This
task adds a second matcher for `/admin/agents/ws` without duplicating the ~150 lines of
relay/backoff logic.

- [ ] **Step 1: Rename the file and generalize the route matching**

```bash
cd frontend
git mv ws-proxy/session-stream-proxy.js ws-proxy/ws-proxy.js
```

Replace the top of the file (everything through `toUpstreamUrl`/`connectUpstream`) with a
route-table version. The `readCookie` helper is unchanged — keep it exactly as-is. Replace
`SESSION_WS_PATH`, `toUpstreamUrl`, and `connectUpstream` with:

```js
/**
 * @typedef {Object} WsRoute
 * @property {RegExp} pattern
 * @property {(match: RegExpMatchArray) => string} upstreamPath
 */

/** @type {WsRoute[]} */
const ROUTES = [
	{
		pattern: /^\/chat\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\/ws$/,
		upstreamPath: (match) => `/api/sessions/${match[1]}/ws`,
	},
	{
		pattern: /^\/admin\/agents\/ws$/,
		upstreamPath: () => '/api/admin/agents/ws',
	},
];

/**
 * @param {string} pathname
 * @returns {WsRoute & { match: RegExpMatchArray } | undefined}
 */
function matchRoute(pathname) {
	for (const route of ROUTES) {
		const match = pathname.match(route.pattern);
		if (match) return { ...route, match };
	}
	return undefined;
}

/**
 * @param {ProxyOptions} options
 * @returns {string}
 */
function resolveApiUrl(options) {
	return options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080';
}

/**
 * @param {string} upstreamPath
 * @param {ProxyOptions} options
 * @returns {string}
 */
function toUpstreamUrl(upstreamPath, options) {
	const wsBase = resolveApiUrl(options).replace(/^http/, 'ws');
	return `${wsBase}${upstreamPath}`;
}

/**
 * @param {string} upstreamPath
 * @param {string | undefined} accessToken
 * @param {ProxyOptions} options
 * @returns {WebSocket}
 */
function connectUpstream(upstreamPath, accessToken, options) {
	return new WebSocket(toUpstreamUrl(upstreamPath, options), {
		headers: accessToken ? { authorization: `Bearer ${accessToken}` } : {}
	});
}
```

- [ ] **Step 2: Thread `upstreamPath` through instead of `sessionId`**

Every remaining function in the file (`attachWsProxy` — renamed from `attachSessionStreamProxy`,
`relay`, `attemptReconnect`) currently takes/threads a `sessionId: string` parameter purely to
pass it to `connectUpstream`/`toUpstreamUrl` on (re)connect — nothing else about the relay logic
is session-specific. Rename every occurrence of the parameter `sessionId` to `upstreamPath` in
those three functions' signatures and bodies (a mechanical rename — the retry/backoff/piping
logic itself is byte-identical, it just forwards a different string). `attachWsProxy`'s body
becomes:

```js
export function attachWsProxy(server, options = {}) {
	if (!server) return;
	const wss = new WebSocketServer({ noServer: true });

	server.on('upgrade', (request, socket, head) => {
		const url = new URL(request.url ?? '', 'http://internal');
		const route = matchRoute(url.pathname);
		if (!route) return; // not ours; let it fall through untouched

		const upstreamPath = route.upstreamPath(route.match);
		const accessToken = readCookie(request.headers.cookie, 'access_token');
		const upstream = connectUpstream(upstreamPath, accessToken, options);

		const cleanup = () => {
			upstream.off('open', onOpen);
			upstream.off('unexpected-response', onUnexpectedResponse);
			upstream.off('error', onError);
		};
		const onOpen = () => {
			cleanup();
			wss.handleUpgrade(request, socket, head, (browserWs) => {
				relay(browserWs, upstream, upstreamPath, accessToken, options);
			});
		};
		const onUnexpectedResponse = (_req, res) => {
			cleanup();
			const code = res.statusCode === 401 ? 4401 : res.statusCode === 404 ? 4404 : 1011;
			acceptThenClose(request, socket, head, wss, code);
		};
		const onError = () => {
			cleanup();
			acceptThenClose(request, socket, head, wss, 1011);
		};

		upstream.once('open', onOpen);
		upstream.once('unexpected-response', onUnexpectedResponse);
		upstream.once('error', onError);
	});
}
```

`acceptThenClose` is unchanged. `relay` and `attemptReconnect` keep their exact existing bodies —
only the parameter name `sessionId` → `upstreamPath` changes (it's already opaque to the rest of
the logic; `attemptReconnect`'s `connectUpstream(sessionId, accessToken, options)` call becomes
`connectUpstream(upstreamPath, accessToken, options)`, same for `relay`'s own
`wireUpstream`/reconnect setup).

- [ ] **Step 3: Update the two call sites**

```typescript
// frontend/vite-plugins/dev-ws-proxy.ts
import type { Plugin } from 'vite';
import { attachWsProxy } from '../ws-proxy/ws-proxy.js';

export function devWsProxy(): Plugin {
	return {
		name: 'dev-ws-proxy',
		configureServer(server) {
			attachWsProxy(server.httpServer);
		},
		configurePreviewServer(server) {
			attachWsProxy(server.httpServer);
		}
	};
}
```

```javascript
// frontend/server.js
import { createServer } from 'node:http';
import { handler } from './build/handler.js';
import { attachWsProxy } from './ws-proxy/ws-proxy.js';

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

const server = createServer(handler);
attachWsProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
```

- [ ] **Step 4: Verify the existing chat WS flow still works**

Run: `npm run check` (from `frontend/`) — expected: 0 new errors (this is plain JS with JSDoc
types, not type-checked by `svelte-check`, but a broken import here would surface as a runtime
failure, not a type error — the real check is functional).

Start the dev server (`npm run dev`), open an existing chat session, send a message, and confirm
the reply still streams in live (the existing `/chat/:id/ws` path must keep working unchanged —
this task only adds a second route, it must not regress the first). If a live dev server isn't
reachable in this environment, state that explicitly rather than claiming this was verified.

- [ ] **Step 5: Commit**

```bash
git add ws-proxy/ws-proxy.js vite-plugins/dev-ws-proxy.ts server.js
git commit -m "refactor: generalize the Node WS proxy to a route table, add the admin agents route"
```

(Git will record the rename from `session-stream-proxy.js` automatically if the content
similarity is high enough — `git add` both the new path and confirm the old path is gone via
`git status` before committing; if `git mv` wasn't used in Step 1, `git rm` the old file
explicitly here.)

---

### Task 6: Frontend types

**Files:**
- Modify: `frontend/src/lib/types.ts`

**Interfaces:**
- Produces: `AdminStreamFrame` (discriminated union matching `AdminStreamFrame` from Task 3
  exactly), `AgentEventItem` (matching Task 4's response shape exactly).

- [ ] **Step 1: Add the types**

```typescript
// frontend/src/lib/types.ts — add
export type AdminStreamFrame =
	| {
			kind: 'AgentSessionStarted';
			agent_session_id: string;
			session_id: string;
			agent_type: string;
			agent_display_name: string;
			channel: string;
			sender_label: string;
	  }
	| { kind: 'AgentSessionEnded'; agent_session_id: string; session_id: string; reason: string }
	| { kind: 'AgentPhaseChanged'; agent_session_id: string; session_id: string; phase: string; detail: string | null };

export interface AgentEventItem {
	id: string;
	session_id: string | null;
	agent_session_id: string | null;
	agent_type: string | null;
	event_type: string;
	created_at: string;
	tool_name: string | null;
	is_error: boolean | null;
}
```

- [ ] **Step 2: Type-check**

Run: `npm run check` (from `frontend/`)
Expected: 0 new errors (nothing consumes these types yet — this step just confirms the syntax is
valid TypeScript).

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat: add frontend types for the admin command center's live stream and event history"
```

---

### Task 7: Live table + activity feed

**Files:**
- Modify: `frontend/src/routes/admin/(protected)/agents/+page.server.ts`
- Modify: `frontend/src/routes/admin/(protected)/agents/+page.svelte`

**Interfaces:**
- Consumes: `AdminStreamFrame`, `AgentEventItem` (Task 6); existing `GET /api/admin/agents`
  (unchanged, still the table's initial snapshot); `GET /api/admin/agent-events` (Task 4, the
  feed's initial snapshot); the `/admin/agents/ws` browser-facing path (Task 5).

**Note on the WS URL:** the browser connects to `new WebSocket('/admin/agents/ws')` — a plain
relative path, NOT `/api/admin/agents/ws`. The Node proxy (Task 5) intercepts this exact path at
the raw HTTP server's `'upgrade'` event, before SvelteKit's own router ever sees it (the same way
`/chat/:id/ws` already works for the existing per-session chat WS — see
`frontend/src/lib/components/ChatThread.svelte`'s own `new WebSocket(\`/chat/${sessionId}/ws\`)`
call for the precedent this follows).

- [ ] **Step 1: Extend the loader to fetch the feed's initial snapshot**

```typescript
// frontend/src/routes/admin/(protected)/agents/+page.server.ts
import { apiFetch } from '$lib/server/api';
import type { AgentEventItem, AgentsResponse } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const agentsResponse = await apiFetch(fetch, cookies, '/api/admin/agents');
	const agents: AgentsResponse = agentsResponse.ok ? ((await agentsResponse.json()) as AgentsResponse) : { users: [] };

	const eventsResponse = await apiFetch(fetch, cookies, '/api/admin/agent-events');
	const events: AgentEventItem[] = eventsResponse.ok ? ((await eventsResponse.json()) as AgentEventItem[]) : [];

	return { agents, events };
};
```

- [ ] **Step 2: Rewrite the page**

```svelte
<!-- frontend/src/routes/admin/(protected)/agents/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import type { AdminStreamFrame, AgentEventItem } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	type Row = {
		agent_session_id: string;
		session_id: string;
		user_label: string;
		agent_type: string;
		channel: string;
		current_phase: string;
		current_phase_detail: string | null;
		started_at: string;
		last_activity_at: string;
	};

	let rows = $state<Row[]>(
		data.agents.users.flatMap((group) =>
			group.agents.map((agent) => ({
				agent_session_id: agent.agent_session_id,
				session_id: '', // the initial snapshot doesn't carry session_id per row (see admin_dashboard::get_agents) — live updates below always carry it, and this stays unused until a live event replaces the row
				user_label: group.label,
				agent_type: agent.agent_type,
				channel: agent.channel,
				current_phase: agent.current_phase,
				current_phase_detail: agent.current_phase_detail,
				started_at: agent.started_at,
				last_activity_at: agent.last_activity_at,
			})),
		),
	);

	let feed = $state<AgentEventItem[]>(data.events);

	function feedLine(item: AgentEventItem): string {
		const who = item.agent_type ?? 'An agent';
		if (item.event_type === 'AgentSpawned') return `${who} started`;
		if (item.event_type === 'AgentCompleted') return `${who} finished (completed)`;
		if (item.event_type === 'AgentCancelled') return `${who} finished (cancelled)`;
		if (item.event_type === 'AgentExpired') return `${who} expired`;
		if (item.event_type === 'ToolCalled') return `${who} called ${item.tool_name ?? 'a tool'}${item.is_error ? ' (failed)' : ''}`;
		if (item.event_type === 'AgentReplied') return `${who} replied`;
		return `${who}: ${item.event_type}`;
	}

	function phaseLabel(row: Row): string {
		if (row.current_phase === 'calling_tool' && row.current_phase_detail) {
			return `calling tool: ${row.current_phase_detail}`;
		}
		return row.current_phase.replace('_', ' ');
	}

	const columns = [
		{ key: 'user_label', label: 'User', sortable: true },
		{ key: 'agent_type', label: 'Agent Type', sortable: true },
		{ key: 'channel', label: 'Channel', sortable: true },
		{ key: 'current_phase', label: 'Status', sortable: true },
		{ key: 'started_at', label: 'Started', sortable: true },
		{ key: 'last_activity_at', label: 'Last Activity', sortable: true },
	];

	let sortKey = $state<string | undefined>('user_label');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	const sortedRows = $derived.by(() => {
		if (!sortKey) return rows;
		const key = sortKey as keyof Row;
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...rows].sort((a, b) => {
			const av = a[key] ?? '';
			const bv = b[key] ?? '';
			if (av < bv) return -1 * direction;
			if (av > bv) return 1 * direction;
			return 0;
		});
	});

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;
	const MAX_FEED_ITEMS = 200;

	function applyFrame(frame: AdminStreamFrame) {
		if (frame.kind === 'AgentSessionStarted') {
			rows = [
				...rows,
				{
					agent_session_id: frame.agent_session_id,
					session_id: frame.session_id,
					user_label: frame.sender_label,
					agent_type: frame.agent_type,
					channel: frame.channel,
					current_phase: 'waiting',
					current_phase_detail: null,
					started_at: new Date().toISOString(),
					last_activity_at: new Date().toISOString(),
				},
			];
			feed = [
				{
					id: crypto.randomUUID(),
					session_id: frame.session_id,
					agent_session_id: frame.agent_session_id,
					agent_type: frame.agent_type,
					event_type: 'AgentSpawned',
					created_at: new Date().toISOString(),
					tool_name: null,
					is_error: null,
				},
				...feed,
			].slice(0, MAX_FEED_ITEMS);
		} else if (frame.kind === 'AgentSessionEnded') {
			const ended = rows.find((r) => r.agent_session_id === frame.agent_session_id);
			rows = rows.filter((r) => r.agent_session_id !== frame.agent_session_id);
			feed = [
				{
					id: crypto.randomUUID(),
					session_id: ended?.session_id ?? frame.session_id,
					agent_session_id: frame.agent_session_id,
					agent_type: ended?.agent_type ?? null,
					event_type: frame.reason === 'cancelled' ? 'AgentCancelled' : frame.reason === 'expired' ? 'AgentExpired' : 'AgentCompleted',
					created_at: new Date().toISOString(),
					tool_name: null,
					is_error: null,
				},
				...feed,
			].slice(0, MAX_FEED_ITEMS);
		} else if (frame.kind === 'AgentPhaseChanged') {
			rows = rows.map((r) =>
				r.agent_session_id === frame.agent_session_id
					? { ...r, current_phase: frame.phase, current_phase_detail: frame.detail, last_activity_at: new Date().toISOString() }
					: r,
			);
			if (frame.phase === 'calling_tool' && frame.detail) {
				const source = rows.find((r) => r.agent_session_id === frame.agent_session_id);
				feed = [
					{
						id: crypto.randomUUID(),
						session_id: frame.session_id,
						agent_session_id: frame.agent_session_id,
						agent_type: source?.agent_type ?? null,
						event_type: 'ToolCalled',
						created_at: new Date().toISOString(),
						tool_name: frame.detail,
						is_error: null,
					},
					...feed,
				].slice(0, MAX_FEED_ITEMS);
			}
		}
	}

	onMount(() => {
		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;

		function connect() {
			socket = new WebSocket('/admin/agents/ws');

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
			});

			socket.addEventListener('message', (event) => {
				let frame: AdminStreamFrame;
				try {
					frame = JSON.parse(event.data);
				} catch {
					return;
				}
				applyFrame(frame);
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) return; // 401/404 — not an admin, or the route vanished; don't retry
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Command center</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every active agent, live.
</p>

<div class="mt-6">
	{#if sortedRows.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		<DataTable {columns} bind:sortKey bind:sortDirection>
			{#each sortedRows as row (row.agent_session_id)}
				<tr>
					<td>{row.user_label}</td>
					<td>{row.agent_type}</td>
					<td>{row.channel}</td>
					<td>{phaseLabel(row)}</td>
					<td>{new Date(row.started_at).toLocaleString()}</td>
					<td>{new Date(row.last_activity_at).toLocaleString()}</td>
				</tr>
			{/each}
		</DataTable>
	{/if}
</div>

<div class="mt-8">
	<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface)">Activity</h2>
	<div class="mt-2 flex flex-col gap-1 max-h-96 overflow-y-auto">
		{#each feed as item (item.id)}
			<div class="md-body-medium flex items-center justify-between px-2 py-1" style="border-bottom: 1px solid var(--md-sys-color-outline-variant)">
				<span style="color: var(--md-sys-color-on-surface)">{feedLine(item)}</span>
				<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{new Date(item.created_at).toLocaleTimeString()}</span>
			</div>
		{/each}
	</div>
</div>
```

Row-drill-down (clicking a row to open the `SideSheet`) is added in Task 8, which extends this
same file rather than duplicating it — the table/feed logic above is already the full, working
page on its own.

- [ ] **Step 3: Type-check**

Run: `npm run check` (from `frontend/`)
Expected: 0 new errors.

- [ ] **Step 4: Live verification**

Start the backend and frontend dev servers, log in as an admin, open `/admin/agents`, and (in
another tab/session) trigger a turn that reaches a specialist agent (e.g. ask about spending, or
build something via the planning agent). Confirm: a row appears without a page reload, its status
column updates as the agent progresses, a feed entry appears for each phase/tool-call/completion,
and the row disappears when the agent finishes. If a live dev server isn't reachable in this
environment, state that explicitly rather than claiming this was verified.

- [ ] **Step 5: Commit**

```bash
git add "src/routes/admin/(protected)/agents/+page.server.ts" "src/routes/admin/(protected)/agents/+page.svelte"
git commit -m "feat: replace the static admin agents page with a live table and activity feed"
```

---

### Task 8: Drill-down

**Files:**
- Modify: `frontend/src/routes/admin/(protected)/agents/+page.svelte`

**Interfaces:**
- Consumes: `SideSheet` (`frontend/src/lib/components/m3/SideSheet.svelte`, existing,
  `open`/`children` props), `GET /api/admin/agent-events?session_id=...` (Task 4).

- [ ] **Step 1: Add drill-down state and a fetch helper**

```typescript
// frontend/src/routes/admin/(protected)/agents/+page.svelte — add to the <script> block
import SideSheet from '$lib/components/m3/SideSheet.svelte';

let drillDownOpen = $state(false);
let drillDownRow = $state<Row | null>(null);
let drillDownEvents = $state<AgentEventItem[]>([]);
let drillDownLoading = $state(false);

async function openDrillDown(row: Row) {
	drillDownRow = row;
	drillDownOpen = true;
	drillDownLoading = true;
	try {
		const response = await fetch(`/admin/agents/${row.agent_session_id}/events?sessionId=${row.session_id}`);
		drillDownEvents = response.ok ? await response.json() : [];
	} finally {
		drillDownLoading = false;
	}
}
```

- [ ] **Step 2: Make table rows clickable**

```svelte
<!-- replace the existing {#each sortedRows as row (row.agent_session_id)} block's <tr> with: -->
{#each sortedRows as row (row.agent_session_id)}
	<tr onclick={() => openDrillDown(row)} style="cursor: pointer;">
		<td>{row.user_label}</td>
		<td>{row.agent_type}</td>
		<td>{row.channel}</td>
		<td>{phaseLabel(row)}</td>
		<td>{new Date(row.started_at).toLocaleString()}</td>
		<td>{new Date(row.last_activity_at).toLocaleString()}</td>
	</tr>
{/each}
```

- [ ] **Step 3: Render the sheet**

```svelte
<!-- append at the end of the file, alongside the existing markup -->
<SideSheet bind:open={drillDownOpen}>
	{#snippet children()}
		{#if drillDownRow}
			<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface); margin: 0 0 4px;">
				{drillDownRow.agent_type}
			</h2>
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 16px;">
				Session {drillDownRow.session_id || '(unknown until a live event arrives)'} · {phaseLabel(drillDownRow)}
			</p>
			{#if drillDownLoading}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Loading…</p>
			{:else if drillDownEvents.length === 0}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No recent activity for this agent.</p>
			{:else}
				{#each drillDownEvents as item (item.id)}
					<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
						<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">{feedLine(item)}</p>
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{new Date(item.created_at).toLocaleString()}</p>
					</div>
				{/each}
			{/if}
		{/if}
	{/snippet}
</SideSheet>
```

- [ ] **Step 4: Add the thin proxy route the drill-down fetches from**

The drill-down's `fetch` call above hits a same-origin SvelteKit route (not the Rust backend
directly — the browser has no bearer token to attach, same reason every other frontend data fetch
goes through a `+server.ts`/`+page.server.ts` that holds the cookie-derived token server-side).

```typescript
// frontend/src/routes/admin/(protected)/agents/[agentSessionId]/events/+server.ts (new)
import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, url, cookies, fetch }) => {
	const sessionId = url.searchParams.get('sessionId');
	const query = sessionId ? `?session_id=${sessionId}` : '';
	const response = await apiFetch(fetch, cookies, `/api/admin/agent-events${query}`);
	if (!response.ok) {
		return json([], { status: response.status });
	}
	const events = await response.json();
	// Task 4's endpoint doesn't filter by agent_session_id (only session_id) — narrow to this
	// one agent here, since a session can in principle have had more than one agent over time.
	const filtered = Array.isArray(events) ? events.filter((e) => e.agent_session_id === params.agentSessionId) : [];
	return json(filtered);
};
```

- [ ] **Step 5: Type-check**

Run: `npm run check` (from `frontend/`)
Expected: 0 new errors.

- [ ] **Step 6: Live verification**

With both dev servers running, open `/admin/agents`, click a running agent's row, and confirm the
side sheet opens showing that agent's session id, phase, and its own activity history (tool
calls, not conversation text). If a live dev server isn't reachable in this environment, state
that explicitly rather than claiming this was verified.

- [ ] **Step 7: Commit**

```bash
git add "src/routes/admin/(protected)/agents/+page.svelte" \
        "src/routes/admin/(protected)/agents/[agentSessionId]/events/+server.ts"
git commit -m "feat: add per-agent drill-down to the command center"
```

---

## Post-plan note

`backend/.cargo/config.toml` must never be staged or committed at any point in this plan's
execution — exclude it from every `git add` in every task's commit. If any frontend task triggers
`rtk`'s known `pnpm install` side effect, run `rm -rf node_modules && npm install` from
`frontend/` and `git checkout -- pnpm-lock.yaml` to discard the drift before committing.
