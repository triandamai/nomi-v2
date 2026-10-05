//! The crew: every agent a user can talk to, for the Home crew card and the chat crew panel.

use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

/// The looks a dynamic agent can pick. Must match `ShapeName`, `GradientTone` and `ShapeMotion`
/// in frontend/src/lib/components/m3/shapes.ts.
pub const SHAPES: &[&str] = &[
    "cookie9", "sunny8", "cookie6", "clover4", "flower5", "circle", "sunny12", "clover3", "puffy7", "burst16", "wave10",
    "soft-square", "soft-triangle", "pentagon", "pill",
];
pub const TONES: &[&str] = &["glow", "ember", "tide", "sky", "bloom", "dusk", "citrus", "slate"];
pub const MOTIONS: &[&str] = &["spin", "wobble", "bounce", "pulse", "orbit"];

#[derive(Serialize)]
pub struct CrewMember {
    /// What `messages.agent_display_name` and `agent_delegations.target_agent_type` refer to it by.
    pub agent_type: String,
    pub name: String,
    pub role: String,
    pub is_dynamic: bool,
    /// Set for dynamic agents; built-ins wear the looks fixed in the frontend's `agentLook`.
    pub shape: Option<String>,
    pub tone: Option<String>,
    pub motion: Option<String>,
    /// What it's doing for this user right now: "working", "waiting" (on the user's approval),
    /// "done" (finished something in the last day) or "idle".
    pub state: String,
    /// One line to go with `state`, e.g. "Finished · Summarize March spending".
    pub status: String,
}

#[derive(Serialize)]
pub struct CrewResponse {
    pub agents: Vec<CrewMember>,
}

/// One-line roles for the built-ins. Their intent descriptions are written for the router, not
/// for people.
fn built_in_role(agent_type: &str) -> Option<&'static str> {
    Some(match agent_type {
        "chitchat" => "Talks with you and routes the work",
        "money" => "Transactions, budgets, subscriptions",
        "coding" => "Builds and edits project files",
        "planning" => "Plans, to-dos and reminders",
        "personality" => "Keeps Nomi sounding like you want",
        "supervisor" => "Keeps track of the crew, and stops it when you ask",
        _ => return None,
    })
}

fn first_sentence(text: &str) -> String {
    let sentence = text.split(['.', '\n']).next().unwrap_or(text).trim();
    let mut chars = sentence.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn clip(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let mut clipped: String = flat.chars().take(max - 1).collect();
    clipped.push('…');
    clipped
}

/// Each agent's live state for `user_id`, keyed by agent type. Agents not in the map are idle.
async fn crew_states(pool: &sqlx::PgPool, user_id: Uuid, default_agent_type: &str) -> Result<HashMap<String, (String, String)>, sqlx::Error> {
    let mut states: HashMap<String, (String, String)> = HashMap::new();

    // Lowest priority first; later inserts overwrite: done < working < waiting.
    let delegations: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT DISTINCT ON (target_agent_type) target_agent_type, status, task FROM agent_delegations \
         WHERE user_id = $1 AND created_at > now() - interval '1 day' ORDER BY target_agent_type, created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    for (agent_type, status, task) in delegations {
        match status.as_str() {
            "completed" => {
                states.insert(agent_type, ("done".into(), format!("Finished · {}", clip(&task, 48))));
            }
            "pending" | "processing" => {
                states.insert(agent_type, ("working".into(), clip(&task, 56)));
            }
            _ => {}
        }
    }

    let sessions: Vec<(String, String, Option<bool>)> = sqlx::query_as(
        "SELECT a.agent_type, a.current_phase, (a.state->>'paused_for_approval')::boolean FROM agent_sessions a \
         JOIN channel_identities ci ON ci.id = a.sender_channel_identity_id \
         WHERE ci.user_id = $1 AND a.status IN ('active', 'awaiting_approval')",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    for (agent_type, phase, paused) in &sessions {
        if *paused == Some(true) {
            states.insert(agent_type.clone(), ("waiting".into(), "Waiting on you · approval needed".into()));
        } else if phase != "waiting" && states.get(agent_type).is_none_or(|(s, _)| s != "waiting") {
            states.insert(agent_type.clone(), ("working".into(), "Working…".into()));
        }
    }

    // Nomi itself is mid-turn when one of the user's turn jobs is processing.
    let nomi_busy: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM turn_jobs tj JOIN channel_identities ci ON ci.id = tj.sender_channel_identity_id \
         WHERE ci.user_id = $1 AND tj.status = 'processing')",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if nomi_busy && !states.contains_key(default_agent_type) {
        states.insert(default_agent_type.to_string(), ("working".into(), "Working…".into()));
    }
    Ok(states)
}

pub async fn list_crew(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<CrewResponse>, (StatusCode, &'static str)> {
    let registry = crate::build_agent_registry(state.project_storage.clone());
    let mut agents: Vec<CrewMember> = registry
        .agents()
        .iter()
        .map(|agent| {
            let agent_type = agent.agent_type().into_owned();
            CrewMember {
                role: built_in_role(&agent_type).map(str::to_string).unwrap_or_else(|| first_sentence(&agent.intent_description())),
                name: agent.display_name().into_owned(),
                agent_type,
                is_dynamic: false,
                shape: None,
                tone: None,
                motion: None,
                state: "idle".into(),
                status: String::new(),
            }
        })
        .collect();
    // Nomi leads the crew.
    if let Some(index) = agents.iter().position(|a| a.agent_type == registry.default_agent().agent_type()) {
        let nomi = agents.remove(index);
        agents.insert(0, nomi);
    }

    let dynamic: Vec<(Uuid, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, intent_description, shape, tone, motion FROM dynamic_agents WHERE is_active = true ORDER BY created_at",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list dynamic agents for the crew");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agents")
    })?;
    agents.extend(dynamic.into_iter().map(|(id, name, description, shape, tone, motion)| CrewMember {
        agent_type: id.to_string(),
        name,
        role: first_sentence(&description),
        is_dynamic: true,
        shape: Some(shape),
        tone: Some(tone),
        motion: Some(motion),
        state: "idle".into(),
        status: String::new(),
    }));

    let default_type = registry.default_agent().agent_type().into_owned();
    let states = crew_states(&state.pool, claims.sub, &default_type).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load the crew's live states");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agents")
    })?;
    for agent in &mut agents {
        match states.get(&agent.agent_type) {
            Some((live_state, status)) => {
                agent.state = live_state.clone();
                agent.status = status.clone();
            }
            None => {
                agent.status = if agent.agent_type == default_type { "Ready when you are".into() } else { "Standing by".into() };
            }
        }
    }

    Ok(Json(CrewResponse { agents }))
}
