use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
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

pub async fn get_dashboard(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<DashboardResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to count users"))?;

    let running_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to count running agents"))?;

    let (tokens_all_time, tokens_today): (i64, i64) = sqlx::query_as(
        "SELECT \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint), 0)::bigint AS all_time, \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint) \
                       FILTER (WHERE created_at >= date_trunc('day', now())), 0)::bigint AS today \
         FROM agent_events WHERE event_type = 'AgentReplied'",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to sum token usage"))?;

    Ok(Json(DashboardResponse { total_users, tokens_today, tokens_all_time, running_agents }))
}

#[derive(Serialize)]
pub struct RunningAgentItem {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub channel: String,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
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

pub async fn get_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<AgentsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<(Uuid, Option<String>, String, String, Uuid, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT \
             u.id, wc.email, ci.channel, ci.channel_user_id, \
             ags.id, ags.agent_type, ags.started_at, ags.last_activity_at \
         FROM agent_sessions ags \
         JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
         JOIN users u ON u.id = ci.user_id \
         LEFT JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE ags.status = 'active' \
         ORDER BY u.id, ags.started_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load running agents"))?;

    // Rows are ORDER BY u.id, so every row for the same user is contiguous — grouping by
    // checking the last-pushed group's user_id needs no HashMap or second pass.
    let mut groups: Vec<UserAgentGroup> = Vec::new();
    for (user_id, email, channel, channel_user_id, agent_session_id, agent_type, started_at, last_activity_at) in rows {
        let agent = RunningAgentItem { agent_session_id, agent_type, channel: channel.clone(), started_at, last_activity_at };
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
