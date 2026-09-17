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
}

type DynamicAgentRow = (Uuid, String, String, String, String, Vec<String>, bool, bool, bool, bool);

fn to_response(row: DynamicAgentRow) -> DynamicAgentResponse {
    let (id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active) = row;
    DynamicAgentResponse { id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active }
}

const SELECT_COLUMNS: &str =
    "id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, is_active";

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
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {SELECT_COLUMNS}"
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
         granted_tools = $5, supports_todos = $6, supports_plans = $7, can_delegate = $8, updated_at = now() \
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
