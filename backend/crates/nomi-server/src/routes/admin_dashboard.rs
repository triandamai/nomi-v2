use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct DashboardResponse {
    pub total_users: i64,
    pub tokens_today: i64,
    pub tokens_all_time: i64,
    pub running_agents: i64,
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_dashboard(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<DashboardResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to count users");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to count users")
        })?;

    let running_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to count running agents");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to count running agents")
        })?;

    let (tokens_all_time, tokens_today): (i64, i64) = sqlx::query_as(
        "SELECT \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint), 0)::bigint AS all_time, \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint) \
                       FILTER (WHERE created_at >= date_trunc('day', now())), 0)::bigint AS today \
         FROM agent_events WHERE event_type = 'AgentReplied'",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to sum token usage");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to sum token usage")
    })?;

    Ok(Json(DashboardResponse { total_users, tokens_today, tokens_all_time, running_agents }))
}

#[derive(Serialize)]
pub struct RunningAgentItem {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub channel: String,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub current_phase: String,
    pub current_phase_detail: Option<String>,
}

#[derive(Serialize)]
pub struct UserAgentGroup {
    pub user_id: Uuid,
    pub label: String,
    pub agents: Vec<RunningAgentItem>,
}

#[derive(Serialize)]
pub struct AgentsResponse {
    pub users: Vec<UserAgentGroup>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<AgentsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<(Uuid, Option<String>, String, String, Uuid, String, DateTime<Utc>, DateTime<Utc>, String, Option<String>)> = sqlx::query_as(
        "SELECT \
             u.id, wc.email, ci.channel, ci.channel_user_id, \
             ags.id, ags.agent_type, ags.started_at, ags.last_activity_at, ags.current_phase, ags.current_phase_detail \
         FROM agent_sessions ags \
         JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
         JOIN users u ON u.id = ci.user_id \
         LEFT JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE ags.status = 'active' \
         ORDER BY u.id, ags.started_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load running agents");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load running agents")
    })?;

    // Rows are ORDER BY u.id, so every row for the same user is contiguous — grouping by
    // checking the last-pushed group's user_id needs no HashMap or second pass.
    let mut groups: Vec<UserAgentGroup> = Vec::new();
    for (user_id, email, channel, channel_user_id, agent_session_id, agent_type, started_at, last_activity_at, current_phase, current_phase_detail) in rows {
        let agent = RunningAgentItem { agent_session_id, agent_type, channel: channel.clone(), started_at, last_activity_at, current_phase, current_phase_detail };
        match groups.last_mut() {
            Some(group) if group.user_id == user_id => group.agents.push(agent),
            _ => {
                let label = email.unwrap_or_else(|| format!("{channel}:{channel_user_id}"));
                groups.push(UserAgentGroup { user_id, label, agents: vec![agent] });
            }
        }
    }

    Ok(Json(AgentsResponse { users: groups }))
}

/// The wire shape forwarded to an admin's browser — deliberately narrower than
/// `StreamEnvelope`: only these three kinds ever reach this connection, and only with the
/// fields named here. `AgentPhaseChanged`'s `detail` is a tool *name* (safe); nothing here can
/// carry message content — that boundary is enforced in `admin_frame_for` below, not by the
/// frontend choosing not to render something it already received.
// Every variant is deliberately prefixed "Agent" — this enum is the allow-list of
// agent-lifecycle events forwarded to admins, and the prefix carries wire-format meaning:
// frontend/src/lib/types.ts's AdminStreamFrame union matches on these exact "kind" strings.
#[allow(clippy::enum_variant_names)]
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

const AGENT_EVENT_TYPES: [&str; 6] =
    ["AgentSpawned", "AgentCompleted", "AgentCancelled", "AgentExpired", "ToolCalled", "AgentReplied"];

#[derive(Debug, Deserialize)]
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
