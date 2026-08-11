# WebSocket Bridge

Date: 2026-08-11
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-11-mqtt-turn-queue-design.md` (sub-projects 2+3 — the `turn_jobs` queue, worker binary, EMQX broker, and `StreamEnvelope`/`StreamEvent` wire types this design subscribes to, all already merged). Sub-project 4 of 5 in the realtime chat effort — see the final one (frontend realtime rendering), specced separately.

## Purpose

Sub-projects 2+3 gave the worker a way to publish live token deltas onto `chat/{session_id}/stream` in EMQX, but nothing outside the backend can see them — MQTT is a backend-internal protocol, and the browser has no way to speak it or reach the broker. This design adds the missing link: a WebSocket endpoint on the existing Rust HTTP server that subscribes to a session's MQTT topic and relays every event to whoever is connected, in order, as JSON.

**Frontend/browser topology is a load-bearing constraint, not a detail.** The browser never talks to the Rust backend directly today — every API call goes through SvelteKit's own server (Node), which holds the JWT in an httpOnly cookie and calls the Rust API server-side with `Authorization: Bearer`. This design keeps that shape: the browser's WebSocket connects to SvelteKit, and SvelteKit's server-side code opens a second WebSocket to this new Rust endpoint, proxying bytes between the two. **The SvelteKit-side proxy and any browser-facing rendering are explicitly out of scope for this sub-project** — sub-project 5 owns them. This sub-project delivers and proves, in isolation, exactly one thing: a Rust WebSocket endpoint that, given a valid session and a live turn in progress, relays that session's `StreamEnvelope` events over the wire, tested with a raw WebSocket client standing in for the future SvelteKit proxy.

## 1. Endpoint and Auth

`GET /api/sessions/:id/ws`, added to `backend/src/routes/sessions.rs` (alongside `send_message`/`list_messages`, so it can reuse the existing private `authorize_session_access` helper without changing its visibility) and wired into `backend/src/app.rs`'s router.

Because this connection is initiated server-side by SvelteKit's Node process — not the browser — it can set arbitrary headers on the WebSocket upgrade request, exactly like every other API call today. **No new auth mechanism is needed**: the handler takes `AuthClaims` (the existing `Authorization: Bearer` extractor) as a normal parameter alongside `axum::extract::ws::WebSocketUpgrade`, and `authorize_session_access(pool, claims.sub, session_id)` gates it the same way `send_message`/`list_messages` already do. A missing/invalid token or unauthorized session rejects the upgrade with the same status codes those endpoints already use.

```rust
pub async fn session_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    Ok(ws.on_upgrade(move |socket| relay_session_stream(socket, session_id)))
}
```

(`axum`'s `ws` feature is expected to already be enabled via its default features — `axum = "0.7"` with no `default-features = false` in `backend/Cargo.toml` — confirm at implementation time; add `features = ["ws"]` explicitly if `WebSocketUpgrade` doesn't resolve.)

## 2. Per-Connection MQTT Subscription

Each accepted WebSocket connection opens its **own** `rumqttc::AsyncClient`/`EventLoop` pair — mirroring `MqttPublisher::connect`'s existing pattern from sub-project 3, just per-connection instead of per-process — subscribes to exactly `chat/{session_id}/stream` (QoS 0, matching the publisher side), and forwards every received publish payload as a WebSocket text frame, verbatim (the MQTT payload is already the `StreamEnvelope` JSON bytes the worker published — no reparsing or re-serialization needed, just pass the bytes through).

```rust
async fn relay_session_stream(mut socket: WebSocket, session_id: Uuid) {
    let mut options = MqttOptions::new(format!("ws-bridge-{}", Uuid::new_v4()), MQTT_BROKER_HOST, MQTT_BROKER_PORT);
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

`MQTT_BROKER_HOST`/`MQTT_BROKER_PORT` (defaulting to `localhost`/`1883`, mirroring `worker.rs`'s existing env var reads from sub-project 2/3) are new to the HTTP server binary — it never needed MQTT config before this. They're read once in `main.rs` at startup (same place `jwt_secret`/`settings_key` are read) and added as two new `String`/`u16` fields on `AppState`, so the handler in `sessions.rs` reads them from `state` rather than re-reading env vars per connection.

No connection registry, no fan-out, no wildcard subscription. Each WebSocket connection is fully isolated — N concurrent connections means N broker connections, which is the right trade-off at today's scale (accepted limitation, same spirit as the worker's single-instance-for-now design in sub-project 2/3): a shared subscriber multiplexing many session topics through one broker connection is a legitimate future optimization if this ever needs to handle thousands of concurrent tabs, not needed now.

## 3. Lifecycle

The connection stays open for the life of the chat session view, not scoped to a single turn — it relays every subsequent turn's deltas without reconnecting, matching how a real chat UI behaves (connect once when the page opens, see all future replies live). It closes when either side disconnects (browser navigates away, tab closes, network drop) or the broker connection is lost (the `Err(_) => break` arm above) — in the broker-lost case, the client (the future SvelteKit proxy, sub-project 5) is expected to reconnect; this sub-project does not implement its own reconnect/retry loop, since that policy belongs to whoever is consuming the connection.

**No replay or catch-up on (re)connect.** A client that connects — or reconnects after a drop — only sees events published *after* that point; it does not see what it missed. Recovering "what happened while I was disconnected" is REST's job (`GET /api/sessions/:id/messages`, already built), not this endpoint's — this is the standard "WebSocket for live deltas, REST for source of truth" split, and keeping it means this sub-project doesn't need any buffering/replay logic at all.

## 4. Testing Approach

A new dev-dependency, `tokio-tungstenite`, gives tests a real WebSocket client to stand in for the future SvelteKit proxy (which doesn't exist yet — sub-project 5). Tests spin up the app with a real Postgres + EMQX (same `docker-compose` services sub-project 2/3 already added), authenticate with a real JWT (reusing whatever test helper already mints one for the existing authenticated-route tests), connect the WS client to `GET /api/sessions/:id/ws`, then drive a real chitchat turn through the already-built worker (`turn::process_turn` directly, or the full ingest→worker path) and assert the exact ordered sequence of `StreamEnvelope` JSON frames arrives over the socket — `Delta` events matching `FakeLlmProvider`'s canned stream, followed by `TurnCompleted`.

Additional cases: an unauthenticated/wrong-session upgrade attempt is rejected before any MQTT subscription happens (mirrors `send_message`'s existing 401/404 tests); a WebSocket that stays open across two separate turns on the same session receives both turns' events without reconnecting (proves the session-scoped lifecycle from §3, not per-turn).

## Out of Scope

- The SvelteKit-side WebSocket proxy and any browser-facing rendering (sub-project 5).
- Replay/catch-up buffering for reconnects (§3) — deferred; REST remains the source of truth.
- A shared/multiplexed MQTT subscriber for scaling beyond today's expected connection counts (§2) — future hardening, not needed now.
- Reconnect/retry policy on the client side of this endpoint — belongs to whoever consumes it (sub-project 5).
