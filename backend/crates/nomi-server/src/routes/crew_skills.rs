//! A crew member's skills (Settings › Crew): the built-in ones it came with, and the ones a person
//! added — typed, uploaded (a SKILL.md or a .zip of a skill folder) or imported from a link —
//! switched on or off, edited, refreshed from their link and removed. The member reads them with
//! `read_skill` (`nomi_tools_skills`).
//!
//! Errors are `400 {"error": <code>, "detail": <what went wrong>}` so the app can word each one;
//! an unknown member or skill is a 404.

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_skills::{frontmatter, validate, Reference, SkillError};
use nomi_tools_skills::import::{self, ImportError, Imported};
use nomi_tools_skills::store::{self, Origin, SkillRow, StoreError};

use crate::app::AppState;
use crate::routes::crew_settings::member;

/// The env var holding a GitHub token for link imports (optional; raises GitHub's rate limit).
const GITHUB_TOKEN_VAR: &str = "NOMI_GITHUB_TOKEN";

/// Why a skill request failed, as the app sees it.
#[derive(Debug, PartialEq, Eq)]
pub enum SkillApiError {
    /// 400 with a code and a detail.
    Bad(&'static str, String),
    /// 404: no such member or skill for this person.
    NotFound(String),
    /// 500.
    Internal(String),
}

impl IntoResponse for SkillApiError {
    fn into_response(self) -> Response {
        match self {
            SkillApiError::Bad(code, detail) => (StatusCode::BAD_REQUEST, Json(json!({ "error": code, "detail": detail }))).into_response(),
            SkillApiError::NotFound(what) => (StatusCode::NOT_FOUND, what).into_response(),
            SkillApiError::Internal(what) => (StatusCode::INTERNAL_SERVER_ERROR, what).into_response(),
        }
    }
}

fn internal<E: std::fmt::Display>(what: &'static str) -> impl FnOnce(E) -> SkillApiError {
    move |e| {
        tracing::error!(error = %e, "{what}");
        SkillApiError::Internal(what.to_string())
    }
}

fn skill_not_found() -> SkillApiError {
    SkillApiError::NotFound("skill not found".to_string())
}

/// A SKILL.md that broke a rule.
pub fn skill_error(e: SkillError) -> SkillApiError {
    match e {
        SkillError::TooLarge => SkillApiError::Bad("too_large", e.detail()),
        other => SkillApiError::Bad("invalid_skill", other.detail()),
    }
}

/// An upload or link that didn't give a skill.
pub fn import_error(e: ImportError) -> SkillApiError {
    match e {
        ImportError::UnsupportedFile => SkillApiError::Bad("unsupported_file", "send a SKILL.md or a .zip of a skill folder".to_string()),
        ImportError::NotASkill(detail) => SkillApiError::Bad("not_a_skill", detail),
        ImportError::Invalid(e) => skill_error(e),
        ImportError::TooLarge => SkillApiError::Bad("too_large", "that's more than one skill may be".to_string()),
        ImportError::FetchFailed(detail) => SkillApiError::Bad("fetch_failed", detail),
        ImportError::PrivateAddress => SkillApiError::Bad("private_address", "links to private or local addresses can't be imported".to_string()),
        ImportError::RateLimited => SkillApiError::Bad("rate_limited", "GitHub is turning requests away for now; try again later".to_string()),
    }
}

fn store_error(e: StoreError) -> SkillApiError {
    match e {
        StoreError::NameTaken => SkillApiError::Bad("name_taken", "this member already has a skill with that name".to_string()),
        StoreError::TooMany => SkillApiError::Bad(
            "too_many_skills",
            format!("a member can have at most {} skills of yours", nomi_skills::MAX_SKILLS_PER_MEMBER),
        ),
        StoreError::NotFound => skill_not_found(),
        StoreError::Db(e) => internal("failed to save the skill")(e),
    }
}

/// The member's key, when this person has it in their crew.
async fn member_type(state: &AppState, user_id: Uuid, agent_type: &str) -> Result<(String, &'static [nomi_skills::BuiltinSkill]), SkillApiError> {
    let member = member(state, user_id, agent_type).await.map_err(|(_, what)| SkillApiError::NotFound(what))?;
    Ok((member.agent_type, member.agent.builtin_skills()))
}

async fn conn(state: &AppState) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, SkillApiError> {
    state.pool.acquire().await.map_err(internal("failed to reach the database"))
}

#[derive(Serialize)]
pub struct BuiltinSummary {
    pub name: String,
    pub description: String,
    pub references: Vec<String>,
}

/// One of the person's skills in the list: everything but its text.
#[derive(Serialize)]
pub struct SkillSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub license: String,
    pub source: String,
    pub enabled: bool,
    pub origin: Origin,
    pub source_url: Option<String>,
    pub references: Vec<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<SkillRow> for SkillSummary {
    fn from(row: SkillRow) -> Self {
        SkillSummary {
            id: row.id,
            name: row.name,
            description: row.description,
            license: row.license,
            source: row.source,
            enabled: row.enabled,
            origin: row.origin,
            source_url: row.source_url,
            references: row.reference_names,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct SkillsResponse {
    pub builtin: Vec<BuiltinSummary>,
    pub mine: Vec<SkillSummary>,
}

/// `GET /api/agents/:type/skills`: the skills it came with, and this person's own.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(agent_type): Path<String>) -> Result<Json<SkillsResponse>, SkillApiError> {
    let (agent_type, builtin) = member_type(&state, claims.sub, &agent_type).await?;
    let mine = store::list(&mut *conn(&state).await?, claims.sub, &agent_type).await.map_err(internal("failed to load the skills"))?;
    let builtin = builtin
        .iter()
        .map(|s| BuiltinSummary {
            name: s.name.to_string(),
            description: frontmatter(s.text).map(|(m, _)| m.description).unwrap_or_default(),
            references: s.references.iter().map(|(name, _)| name.to_string()).collect(),
        })
        .collect();
    Ok(Json(SkillsResponse { builtin, mine: mine.into_iter().map(SkillSummary::from).collect() }))
}

#[derive(Serialize)]
pub struct SkillResponse {
    #[serde(flatten)]
    pub skill: SkillRow,
    pub references: Vec<ReferenceOut>,
}

#[derive(Serialize)]
pub struct ReferenceOut {
    pub name: String,
    pub text: String,
}

/// `GET /api/agents/:type/skills/:id`: one skill whole.
pub async fn get_one(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((agent_type, id)): Path<(String, Uuid)>) -> Result<Json<SkillResponse>, SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let (skill, references) = store::get(&mut *conn(&state).await?, claims.sub, &agent_type, id)
        .await
        .map_err(internal("failed to load the skill"))?
        .ok_or_else(skill_not_found)?;
    let references = references.into_iter().map(|r| ReferenceOut { name: r.name, text: r.text }).collect();
    Ok(Json(SkillResponse { skill, references }))
}

/// A saved skill, and what an upload or import left out.
#[derive(Serialize)]
pub struct SavedSkill {
    #[serde(flatten)]
    pub skill: SkillRow,
    pub skipped: Vec<String>,
}

#[derive(Deserialize)]
pub struct ReferenceBody {
    pub name: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct TypedSkill {
    pub text: String,
    #[serde(default)]
    pub references: Vec<ReferenceBody>,
}

fn references_of(body: Vec<ReferenceBody>) -> Vec<Reference> {
    body.into_iter().map(|r| Reference { name: r.name, text: r.text }).collect()
}

async fn save(state: &AppState, user_id: Uuid, agent_type: &str, imported: Imported, origin: Origin, source_url: Option<&str>) -> Result<(StatusCode, Json<SavedSkill>), SkillApiError> {
    let skill = store::insert(&mut *conn(state).await?, user_id, agent_type, &imported.skill, origin, source_url).await.map_err(store_error)?;
    Ok((StatusCode::CREATED, Json(SavedSkill { skill, skipped: imported.skipped })))
}

/// `POST /api/agents/:type/skills` `{text, references?}`: a skill the person wrote.
pub async fn create(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(body): Json<TypedSkill>,
) -> Result<(StatusCode, Json<SavedSkill>), SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let skill = validate(&body.text, references_of(body.references)).map_err(skill_error)?;
    save(&state, claims.sub, &agent_type, Imported { skill, skipped: vec![] }, Origin::Typed, None).await
}

#[derive(Deserialize)]
pub struct UploadQuery {
    pub filename: String,
}

/// `POST /api/agents/:type/skills/upload?filename=<name>` with the file as the body: a SKILL.md
/// (`.md`) or a skill folder (`.zip`).
pub async fn upload(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Query(query): Query<UploadQuery>,
    body: Bytes,
) -> Result<(StatusCode, Json<SavedSkill>), SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let filename = query.filename.to_ascii_lowercase();
    let imported = if filename.ends_with(".md") {
        import::from_markdown(&body)
    } else if filename.ends_with(".zip") {
        import::from_zip(&body)
    } else {
        Err(ImportError::UnsupportedFile)
    }
    .map_err(import_error)?;
    save(&state, claims.sub, &agent_type, imported, Origin::Upload, None).await
}

#[derive(Deserialize)]
pub struct ImportBody {
    pub url: String,
}

fn github_token() -> Option<String> {
    std::env::var(GITHUB_TOKEN_VAR).ok().filter(|t| !t.trim().is_empty())
}

/// `POST /api/agents/:type/skills/import` `{url}`: a skill from a GitHub folder or file, or any
/// `https` link to a `.md`. The text is copied now; only a refresh fetches it again.
pub async fn import_link(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(body): Json<ImportBody>,
) -> Result<(StatusCode, Json<SavedSkill>), SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let url = body.url.trim().to_string();
    let imported = import::from_link(&url, github_token().as_deref()).await.map_err(import_error)?;
    save(&state, claims.sub, &agent_type, imported, Origin::Link, Some(&url)).await
}

#[derive(Deserialize)]
pub struct UpdateSkill {
    pub text: Option<String>,
    pub references: Option<Vec<ReferenceBody>>,
    pub enabled: Option<bool>,
}

/// `PUT /api/agents/:type/skills/:id` `{text?, references?, enabled?}`: edits a skill (a new name
/// is taken when free; references stay unless given) or switches it on or off.
pub async fn update(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((agent_type, id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateSkill>,
) -> Result<Json<SkillRow>, SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let mut conn = conn(&state).await?;
    let (current, current_references) = store::get(&mut conn, claims.sub, &agent_type, id).await.map_err(internal("failed to load the skill"))?.ok_or_else(skill_not_found)?;
    if body.text.is_some() || body.references.is_some() {
        let text = body.text.unwrap_or(current.text);
        let references = body.references.map(references_of).unwrap_or(current_references);
        let skill = validate(&text, references).map_err(skill_error)?;
        // An edited link skill is the person's own now: a refresh would overwrite the edit.
        let origin = (current.origin == Origin::Link).then_some(Origin::Typed);
        store::replace(&mut conn, claims.sub, &agent_type, id, &skill, None, origin).await.map_err(store_error)?;
    }
    if let Some(enabled) = body.enabled {
        store::set_enabled(&mut conn, claims.sub, &agent_type, id, enabled).await.map_err(internal("failed to save the skill"))?;
    }
    let (skill, _) = store::get(&mut conn, claims.sub, &agent_type, id).await.map_err(internal("failed to load the skill"))?.ok_or_else(skill_not_found)?;
    Ok(Json(skill))
}

/// `POST /api/agents/:type/skills/:id/refresh`: fetches a link skill again and replaces it.
pub async fn refresh(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((agent_type, id)): Path<(String, Uuid)>) -> Result<Json<SavedSkill>, SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let current = {
        // Given back before the fetch, which can take a while: no connection waits on the network.
        let mut conn = conn(&state).await?;
        store::get(&mut conn, claims.sub, &agent_type, id).await.map_err(internal("failed to load the skill"))?.ok_or_else(skill_not_found)?.0
    };
    let Some(url) = current.source_url.filter(|_| current.origin == Origin::Link) else {
        return Err(SkillApiError::Bad("not_a_link_skill", "only a skill imported from a link can be updated from it".to_string()));
    };
    let imported = import::from_link(&url, github_token().as_deref()).await.map_err(import_error)?;
    let skill = store::replace(&mut *conn(&state).await?, claims.sub, &agent_type, id, &imported.skill, Some(&url), None).await.map_err(store_error)?;
    Ok(Json(SavedSkill { skill, skipped: imported.skipped }))
}

/// `DELETE /api/agents/:type/skills/:id`.
pub async fn delete(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((agent_type, id)): Path<(String, Uuid)>) -> Result<StatusCode, SkillApiError> {
    let (agent_type, _) = member_type(&state, claims.sub, &agent_type).await?;
    let removed = store::delete(&mut *conn(&state).await?, claims.sub, &agent_type, id).await.map_err(internal("failed to remove the skill"))?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(skill_not_found())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_import_error_has_its_code() {
        let code = |e: ImportError| match import_error(e) {
            SkillApiError::Bad(code, _) => code,
            other => panic!("{other:?}"),
        };
        assert_eq!(code(ImportError::UnsupportedFile), "unsupported_file");
        assert_eq!(code(ImportError::NotASkill("x".into())), "not_a_skill");
        assert_eq!(code(ImportError::Invalid(SkillError::BadName)), "invalid_skill");
        assert_eq!(code(ImportError::Invalid(SkillError::TooLarge)), "too_large");
        assert_eq!(code(ImportError::TooLarge), "too_large");
        assert_eq!(code(ImportError::FetchFailed("x".into())), "fetch_failed");
        assert_eq!(code(ImportError::PrivateAddress), "private_address");
        assert_eq!(code(ImportError::RateLimited), "rate_limited");
    }
}
