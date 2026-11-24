//! A person's own crew (Settings › Crew): switching a member off, what each member may do
//! without asking, each member's soul.md and its versions, what a member has been doing, and
//! members of their own, from scratch or from a template. The router and the tool loop honour all
//! of it through `nomi_agent_core::personal_crew`.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_agent_core::personal_crew::{self, CreateMemberError, CrewTemplate, Level, SoulVersion, SOUL_MAX_CHARS};
use nomi_agent_core::SubAgent;
use nomi_auth::extractor::AuthClaims;
use nomi_llm::ToolDefinition;

use crate::app::AppState;
use crate::routes::deploy::{internal, ApiError};

/// How many soul.md versions the history lists.
const SOUL_VERSIONS_LIMIT: i64 = 50;
/// How many entries Activity lists.
const ACTIVITY_LIMIT: i64 = 50;
/// How far back Activity looks.
const ACTIVITY_WINDOW_DAYS: i32 = 30;

/// A crew member as these routes see it: who it is, and the tools a person can set levels for.
pub(crate) struct Member {
    pub(crate) agent_type: String,
    name: String,
    is_default: bool,
    /// Its own (a person's own member), so it can be deleted.
    is_own: bool,
    tools: Vec<ToolDefinition>,
    pub(crate) agent: Arc<dyn SubAgent>,
}

fn not_found() -> ApiError {
    (StatusCode::NOT_FOUND, "crew member not found".to_string())
}

/// The member `agent_type` names, when `user_id` has it in their crew: a built-in one, an admin's
/// member for everyone, or one of their own.
pub(crate) async fn member(state: &AppState, user_id: Uuid, agent_type: &str) -> Result<Member, ApiError> {
    let registry = crate::build_agent_registry(&state.pool, state.project_storage.clone(), state.tool_catalog.clone());
    if let Some(agent) = registry.find(agent_type) {
        let mut tools = agent.tools();
        // The engine runs it for them, under the same permission levels as their own tools.
        if agent.creates_crew_members() {
            tools.push(nomi_agent_core::engine::create_agent_tool_definition());
        }
        return Ok(Member {
            agent_type: agent_type.to_string(),
            name: agent.display_name().into_owned(),
            is_default: agent.is_default(),
            is_own: false,
            tools,
            agent,
        });
    }
    let id: Uuid = agent_type.parse().map_err(|_| not_found())?;
    let row: Option<bool> = sqlx::query_scalar(
        "SELECT owner_user_id IS NOT NULL FROM dynamic_agents WHERE id = $1 AND is_active = true AND (owner_user_id IS NULL OR owner_user_id = $2)",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to load the crew member"))?;
    let is_own = row.ok_or_else(not_found)?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to load the crew member"))?;
    let row = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(&mut conn, id)
        .await
        .map_err(internal("failed to load the crew member"))?
        .ok_or_else(not_found)?;
    let agent: Arc<dyn SubAgent> = Arc::new(nomi_agent_core::DynamicAgent::from_row(
        row,
        state.tool_catalog.clone(),
    ));
    Ok(Member {
        agent_type: agent_type.to_string(),
        name: agent.display_name().into_owned(),
        is_default: false,
        is_own,
        tools: agent.tools(),
        agent,
    })
}

#[derive(Deserialize)]
pub struct UpdateMember {
    #[serde(default)]
    pub enabled: Option<bool>,
    /// The member's tools, replacing the ones it has: only for a member of the person's own.
    #[serde(default)]
    pub tools: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct UpdateMemberResponse {
    pub agent_type: String,
    pub enabled: bool,
    pub tools: Vec<String>,
}

/// `PUT /api/agents/:type` `{enabled?, tools?}`: switches a member on or off for this person
/// (Nomi itself stays on, 400) and sets the tools of a member of their own (404 for anyone
/// else's, 400 naming an unknown tool).
pub async fn update_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(req): Json<UpdateMember>,
) -> Result<Json<UpdateMemberResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    if member.is_default && req.enabled == Some(false) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("{} can't be switched off", member.name),
        ));
    }
    if req.tools.is_some() && !member.is_own {
        return Err(not_found());
    }
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to update the member"))?;
    let tools = match req.tools {
        Some(tools) => {
            let tools = personal_crew::dedupe_tool_names(tools);
            let switches = nomi_agent_core::tools::ToolSwitches::load(&mut conn).await;
            personal_crew::check_tool_names(&state.tool_catalog, &switches, &tools).map_err(bad)?;
            Some(tools)
        }
        None => None,
    };
    if let Some(enabled) = req.enabled {
        personal_crew::set_enabled(&mut conn, claims.sub, &member.agent_type, enabled)
            .await
            .map_err(internal("failed to switch the member"))?;
    }
    let tools = match tools {
        Some(tools) => {
            let id: Uuid = member.agent_type.parse().map_err(|_| not_found())?;
            let saved = personal_crew::set_member_tools(&mut conn, claims.sub, id, &tools)
                .await
                .map_err(internal("failed to save the tools"))?;
            if !saved {
                return Err(not_found());
            }
            tools
        }
        None => member.tools.iter().map(|t| t.name.clone()).collect(),
    };
    let enabled = personal_crew::is_enabled(&mut conn, claims.sub, &member.agent_type).await;
    Ok(Json(UpdateMemberResponse {
        agent_type: member.agent_type,
        enabled,
        tools,
    }))
}

#[derive(Serialize)]
pub struct PermissionItem {
    /// The tool's name.
    pub key: String,
    pub label: String,
    /// What the tool does, from its description.
    pub description: String,
    pub level: Level,
    /// What it is without a decision of the person's.
    pub default_level: Level,
}

/// "list_transactions" → "List transactions".
fn tool_label(name: &str) -> String {
    let words = name.replace('_', " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

async fn permission_items(
    state: &AppState,
    user_id: Uuid,
    member: &Member,
) -> Result<Vec<PermissionItem>, ApiError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to load the permissions"))?;
    let levels = personal_crew::member_levels(&mut conn, user_id, &member.agent_type).await;
    let mut items: Vec<PermissionItem> = member
        .tools
        .iter()
        .filter(|tool| nomi_agent_core::engine::is_gateable(Some(state.tool_catalog.as_ref()), &tool.name))
        .map(|tool| {
            let default_level = if member.agent.tool_needs_approval(&tool.name) {
                Level::Ask
            } else {
                Level::Allow
            };
            PermissionItem {
                key: tool.name.clone(),
                label: tool_label(&tool.name),
                description: crate::routes::agents::first_sentence(&tool.description),
                level: levels.get(&tool.name).copied().unwrap_or(default_level),
                default_level,
            }
        })
        .collect();
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.key == b.key);
    Ok(items)
}

/// `GET /api/agents/:type/permissions`: each of the member's tools with what it may do.
pub async fn get_permissions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
) -> Result<Json<Vec<PermissionItem>>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    Ok(Json(permission_items(&state, claims.sub, &member).await?))
}

#[derive(Deserialize)]
pub struct SetPermission {
    pub key: String,
    pub level: String,
}

/// `PUT /api/agents/:type/permissions` `[{key, level}]`: sets the levels named (others keep
/// theirs). 400 for a tool the member doesn't have or an unknown level.
pub async fn set_permissions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(req): Json<Vec<SetPermission>>,
) -> Result<Json<Vec<PermissionItem>>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let current = permission_items(&state, claims.sub, &member).await?;
    let mut levels = Vec::with_capacity(req.len());
    for item in req {
        if !current.iter().any(|c| c.key == item.key) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("{} has no {} permission", member.name, item.key),
            ));
        }
        let level = Level::parse(&item.level).ok_or((
            StatusCode::BAD_REQUEST,
            "level is allow, ask or never".to_string(),
        ))?;
        levels.push((item.key, level));
    }
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to save the permissions"))?;
    personal_crew::set_levels(&mut conn, claims.sub, &member.agent_type, &levels)
        .await
        .map_err(internal("failed to save the permissions"))?;
    drop(conn);
    Ok(Json(permission_items(&state, claims.sub, &member).await?))
}

#[derive(Serialize)]
pub struct SoulResponse {
    pub text: String,
    /// 0 before the person writes one.
    pub version: i32,
}

/// `GET /api/agents/:type/soul`: the member's soul.md for this person.
pub async fn get_soul(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
) -> Result<Json<SoulResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to load the soul"))?;
    let soul = personal_crew::current_soul(&mut conn, claims.sub, &member.agent_type)
        .await
        .map_err(internal("failed to load the soul"))?;
    let (text, version) = soul.unwrap_or_default();
    Ok(Json(SoulResponse { text, version }))
}

#[derive(Deserialize)]
pub struct SetSoul {
    pub text: String,
}

/// `PUT /api/agents/:type/soul` `{text}`: saves it as the next version, in use from the next turn.
pub async fn set_soul(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(req): Json<SetSoul>,
) -> Result<Json<SoulResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let text = req.text.trim().to_string();
    if text.chars().count() > SOUL_MAX_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("soul.md is at most {SOUL_MAX_CHARS} characters"),
        ));
    }
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to save the soul"))?;
    let version = personal_crew::save_soul(&mut conn, claims.sub, &member.agent_type, &text)
        .await
        .map_err(internal("failed to save the soul"))?;
    Ok(Json(SoulResponse { text, version }))
}

#[derive(Serialize)]
pub struct SoulVersionsResponse {
    pub versions: Vec<SoulVersion>,
}

/// `GET /api/agents/:type/soul/versions`: every saved version, newest first.
pub async fn soul_versions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
) -> Result<Json<SoulVersionsResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to load the versions"))?;
    let versions = personal_crew::soul_versions(
        &mut conn,
        claims.sub,
        &member.agent_type,
        SOUL_VERSIONS_LIMIT,
    )
    .await
    .map_err(internal("failed to load the versions"))?;
    Ok(Json(SoulVersionsResponse { versions }))
}

#[derive(Deserialize)]
pub struct RestoreSoul {
    pub version: i32,
}

/// `POST /api/agents/:type/soul/rollback` `{version}`: puts an earlier version back in use, like
/// the personality's rollback. 404 for a version that isn't there.
pub async fn rollback_soul(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(req): Json<RestoreSoul>,
) -> Result<Json<SoulResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to restore the soul"))?;
    let restored =
        personal_crew::restore_soul(&mut conn, claims.sub, &member.agent_type, req.version)
            .await
            .map_err(internal("failed to restore the soul"))?;
    if !restored {
        return Err((StatusCode::NOT_FOUND, "version not found".to_string()));
    }
    let (text, version) = personal_crew::current_soul(&mut conn, claims.sub, &member.agent_type)
        .await
        .map_err(internal("failed to restore the soul"))?
        .unwrap_or_default();
    Ok(Json(SoulResponse { text, version }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ActivityItem {
    /// "task" (work handed to it), "tool" (a tool it used), "started" or "finished" (a
    /// conversation it had).
    pub kind: String,
    /// The task, the tool's name, or how it finished.
    pub text: String,
    /// For a task: pending, processing, completed, failed or cancelled. For a tool: ok or error.
    pub status: Option<String>,
    pub session_id: Option<Uuid>,
    pub chat_title: Option<String>,
    pub at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ActivityResponse {
    pub items: Vec<ActivityItem>,
}

/// `GET /api/agents/:type/activity`: what the member did for this person lately, newest first,
/// from its delegations and the turn events it recorded in their chats.
pub async fn activity(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
) -> Result<Json<ActivityResponse>, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    let items: Vec<ActivityItem> = sqlx::query_as(
        "SELECT * FROM ( \
            SELECT 'task' AS kind, d.task AS text, d.status AS status, d.session_id, s.title AS chat_title, d.created_at AS at \
            FROM agent_delegations d LEFT JOIN sessions s ON s.id = d.session_id \
            WHERE d.user_id = $1 AND d.target_agent_type = $2 AND d.created_at > now() - make_interval(days => $3) \
          UNION ALL \
            SELECT CASE e.event_type WHEN 'ToolCalled' THEN 'tool' WHEN 'AgentSpawned' THEN 'started' ELSE 'finished' END, \
                   CASE e.event_type WHEN 'ToolCalled' THEN COALESCE(e.payload->>'tool_name', '') ELSE COALESCE(e.payload->>'summary', '') END, \
                   CASE e.event_type WHEN 'ToolCalled' THEN CASE WHEN (e.payload->>'is_error')::boolean THEN 'error' ELSE 'ok' END \
                        WHEN 'AgentCancelled' THEN 'cancelled' ELSE NULL END, \
                   e.session_id, s.title, e.created_at \
            FROM agent_events e JOIN sessions s ON s.id = e.session_id \
            WHERE s.user_id = $1 AND e.agent_type = $2 \
              AND e.event_type IN ('ToolCalled', 'AgentSpawned', 'AgentCompleted', 'AgentCancelled') \
              AND e.created_at > now() - make_interval(days => $3) \
         ) a ORDER BY at DESC LIMIT $4",
    )
    .bind(claims.sub)
    .bind(&member.agent_type)
    .bind(ACTIVITY_WINDOW_DAYS)
    .bind(ACTIVITY_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load the activity"))?;
    Ok(Json(ActivityResponse { items }))
}

#[derive(Serialize)]
pub struct TemplatesResponse {
    pub templates: &'static [CrewTemplate],
}

/// `GET /api/agents/templates`: the starting points for a member of one's own.
pub async fn templates(AuthClaims(_claims): AuthClaims) -> Json<TemplatesResponse> {
    Json(TemplatesResponse {
        templates: personal_crew::TEMPLATES,
    })
}

#[derive(Deserialize)]
pub struct NewMember {
    /// A template's key; its name, role, soul, tools and look fill whatever isn't given.
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub soul: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub shape: Option<String>,
    #[serde(default)]
    pub tone: Option<String>,
    #[serde(default)]
    pub motion: Option<String>,
}

#[derive(Serialize)]
pub struct NewMemberResponse {
    pub agent_type: String,
    pub name: String,
    pub role: String,
    pub soul_version: i32,
    pub tools: Vec<String>,
}

fn bad(message: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, message.into())
}

/// `POST /api/agents`: a member of the person's own, from scratch (`name`, `role`) or from a
/// `template`. Only they see it, talk to it and can route to it; its soul.md starts at version 1.
pub async fn create_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<NewMember>,
) -> Result<(StatusCode, Json<NewMemberResponse>), ApiError> {
    let template = match req.template.as_deref() {
        Some(key) => Some(personal_crew::template(key).ok_or_else(|| bad("unknown template"))?),
        None => None,
    };
    let pick = |given: Option<String>, from_template: Option<&str>| {
        given
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or(from_template.map(str::to_string))
    };
    let name = pick(req.name, template.map(|t| t.name)).ok_or_else(|| bad("name is required"))?;
    let role = pick(req.role, template.map(|t| t.role)).ok_or_else(|| bad("role is required"))?;
    let soul = pick(req.soul, template.map(|t| t.soul)).unwrap_or_default();
    let tools: Vec<String> = req.tools.unwrap_or_else(|| {
        template
            .map(|t| t.tools.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    });
    let shape = pick(req.shape, template.map(|t| t.shape))
        .unwrap_or_else(|| personal_crew::DEFAULT_SHAPE.to_string());
    let tone = pick(req.tone, template.map(|t| t.tone))
        .unwrap_or_else(|| personal_crew::DEFAULT_TONE.to_string());
    let motion = pick(req.motion, template.map(|t| t.motion))
        .unwrap_or_else(|| personal_crew::DEFAULT_MOTION.to_string());
    if !crate::routes::agents::SHAPES.contains(&shape.as_str())
        || !crate::routes::agents::TONES.contains(&tone.as_str())
        || !crate::routes::agents::MOTIONS.contains(&motion.as_str())
    {
        return Err(bad("unknown look"));
    }
    let registry = crate::build_agent_registry(&state.pool, state.project_storage.clone(), state.tool_catalog.clone());
    let crew_names: Vec<String> = registry
        .agents()
        .iter()
        .map(|a| a.display_name().into_owned())
        .collect();
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to add the member"))?;
    let new_member = personal_crew::NewMember {
        name,
        role,
        soul,
        tools,
        template_key: template.map(|t| t.key),
        shape,
        tone,
        motion,
    };
    let created = personal_crew::create_member(
        &mut conn,
        claims.sub,
        new_member,
        &state.tool_catalog,
        &crew_names,
    )
    .await
    .map_err(|e| match e {
        CreateMemberError::Invalid(m) => bad(m),
        CreateMemberError::Taken(m) | CreateMemberError::TooMany(m) => (StatusCode::CONFLICT, m),
        CreateMemberError::PlanLimit(m) => (StatusCode::PAYMENT_REQUIRED, m),
        CreateMemberError::Db(err) => internal("failed to add the member")(err),
    })?;
    Ok((
        StatusCode::CREATED,
        Json(NewMemberResponse {
            agent_type: created.agent_type,
            name: created.name,
            role: created.role,
            soul_version: created.soul_version,
            tools: created.tools,
        }),
    ))
}

/// `DELETE /api/agents/:type`: removes a member of the person's own, with its settings. 404 for
/// anyone else's (the built-ins and the admins' members are switched off instead).
pub async fn delete_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
) -> Result<StatusCode, ApiError> {
    let member = member(&state, claims.sub, &agent_type).await?;
    if !member.is_own {
        return Err(not_found());
    }
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to remove the member"))?;
    for table in [
        "user_agent_settings",
        "user_agent_permissions",
        "user_agent_souls",
        "user_agent_soul_versions",
        // Its skills' references go with them (ON DELETE CASCADE).
        "user_agent_skills",
    ] {
        sqlx::query(&format!(
            "DELETE FROM {table} WHERE user_id = $1 AND agent_type = $2"
        ))
        .bind(claims.sub)
        .bind(&member.agent_type)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to remove the member"))?;
    }
    sqlx::query("UPDATE agent_sessions SET status = 'completed', ended_at = now() WHERE agent_type = $1 AND status IN ('active', 'awaiting_approval')")
        .bind(&member.agent_type)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to remove the member"))?;
    sqlx::query("DELETE FROM dynamic_agents WHERE id = $1 AND owner_user_id = $2")
        .bind(member.agent_type.parse::<Uuid>().map_err(|_| not_found())?)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to remove the member"))?;
    tx.commit()
        .await
        .map_err(internal("failed to remove the member"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_labels_read_as_words() {
        assert_eq!(tool_label("list_transactions"), "List transactions");
        assert_eq!(tool_label(""), "");
    }
}
