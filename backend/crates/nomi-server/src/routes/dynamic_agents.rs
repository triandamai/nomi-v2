use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct DynamicAgentResponse {
    pub id: Uuid,
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
    pub is_active: bool,
    pub shape: String,
    pub tone: String,
    pub motion: String,
}

type DynamicAgentRow = (Uuid, String, String, String, String, Vec<String>, bool, bool, bool, bool, String, String, String);

fn to_response(row: DynamicAgentRow) -> DynamicAgentResponse {
    let (id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active, shape, tone, motion) =
        row;
    DynamicAgentResponse {
        id,
        name,
        system_prompt,
        intent_label,
        intent_description,
        granted_tools,
        supports_todos,
        supports_plans,
        can_delegate,
        is_active,
        shape,
        tone,
        motion,
    }
}

const SELECT_COLUMNS: &str =
    "id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active, shape, tone, motion";

pub async fn list_dynamic_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<DynamicAgentResponse>>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<DynamicAgentRow> = sqlx::query_as(&format!("SELECT {SELECT_COLUMNS} FROM dynamic_agents ORDER BY created_at DESC"))
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list dynamic agents");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list dynamic agents")
        })?;

    Ok(Json(rows.into_iter().map(to_response).collect()))
}

#[derive(Deserialize)]
pub struct DynamicAgentRequest {
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
    /// The agent's look; omitted fields keep the defaults (Nomi's cookie in the glow gradient).
    #[serde(default = "default_shape")]
    pub shape: String,
    #[serde(default = "default_tone")]
    pub tone: String,
    #[serde(default = "default_motion")]
    pub motion: String,
}

fn default_shape() -> String {
    "cookie9".to_string()
}
fn default_tone() -> String {
    "glow".to_string()
}
fn default_motion() -> String {
    "spin".to_string()
}

fn validate_request(req: &DynamicAgentRequest, catalog: &nomi_agent_core::ToolCatalog) -> Result<(), (StatusCode, &'static str)> {
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required"));
    }
    if req.system_prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "system_prompt is required"));
    }
    if req.intent_label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "intent_label is required"));
    }
    if req.intent_description.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "intent_description is required"));
    }
    if !crate::routes::agents::SHAPES.contains(&req.shape.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "shape is not a known shape"));
    }
    if !crate::routes::agents::TONES.contains(&req.tone.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "tone is not a known gradient"));
    }
    if !crate::routes::agents::MOTIONS.contains(&req.motion.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "motion is not a known motion"));
    }
    let known = catalog.known_tool_names();
    for tool in &req.granted_tools {
        if !known.contains(&tool.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "granted_tools contains an unrecognized tool name"));
        }
    }
    Ok(())
}

pub async fn create_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<DynamicAgentRequest>,
) -> Result<(StatusCode, Json<DynamicAgentResponse>), (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    validate_request(&req, &state.tool_catalog)?;

    let row: DynamicAgentRow = sqlx::query_as(&format!(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, created_by, shape, tone, motion) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING {SELECT_COLUMNS}"
    ))
    .bind(&req.name)
    .bind(&req.system_prompt)
    .bind(&req.intent_label)
    .bind(&req.intent_description)
    .bind(&req.granted_tools)
    .bind(req.supports_todos)
    .bind(req.supports_plans)
    .bind(req.can_delegate)
    .bind(claims.sub)
    .bind(&req.shape)
    .bind(&req.tone)
    .bind(&req.motion)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to create dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create dynamic agent — intent_label may already be in use")
    })?;

    Ok((StatusCode::CREATED, Json(to_response(row))))
}

pub async fn update_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<DynamicAgentRequest>,
) -> Result<Json<DynamicAgentResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    validate_request(&req, &state.tool_catalog)?;

    let row: Option<DynamicAgentRow> = sqlx::query_as(&format!(
        "UPDATE dynamic_agents SET name = $1, system_prompt = $2, intent_label = $3, intent_description = $4, \
         granted_tools = $5, supports_todos = $6, supports_plans = $7, can_delegate = $8, \
         shape = $10, tone = $11, motion = $12, updated_at = now() \
         WHERE id = $9 RETURNING {SELECT_COLUMNS}"
    ))
    .bind(&req.name)
    .bind(&req.system_prompt)
    .bind(&req.intent_label)
    .bind(&req.intent_description)
    .bind(&req.granted_tools)
    .bind(req.supports_todos)
    .bind(req.supports_plans)
    .bind(req.can_delegate)
    .bind(id)
    .bind(&req.shape)
    .bind(&req.tone)
    .bind(&req.motion)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to update dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to update dynamic agent — intent_label may already be in use")
    })?;

    row.map(|r| Json(to_response(r))).ok_or((StatusCode::NOT_FOUND, "dynamic agent not found"))
}

pub async fn toggle_active_dynamic_agent(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<DynamicAgentResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let row: Option<DynamicAgentRow> = sqlx::query_as(&format!(
        "UPDATE dynamic_agents SET is_active = NOT is_active, updated_at = now() WHERE id = $1 RETURNING {SELECT_COLUMNS}"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to toggle dynamic agent");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to toggle dynamic agent")
    })?;

    row.map(|r| Json(to_response(r))).ok_or((StatusCode::NOT_FOUND, "dynamic agent not found"))
}
