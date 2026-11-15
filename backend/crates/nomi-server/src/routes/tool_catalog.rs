//! The tools a crew member can be given, for the tool picker (Settings › Crew) and the `@` menu
//! in soul.md. Admin → Tools switches apply: a tool switched off isn't offered.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use nomi_agent_core::tools::ToolSwitches;
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, ApiError};

#[derive(Deserialize)]
pub struct CatalogQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct CatalogTool {
    pub name: String,
    pub description: String,
    pub group: &'static str,
    pub keywords: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct CatalogResponse {
    pub tools: Vec<CatalogTool>,
}

/// `GET /api/tools/catalog?q=&limit=`: the catalog tools matching `q` (name, keyword, then
/// description; `ToolCatalog::search`), best first; every tool when `q` is empty, and all matches
/// without a `limit`.
pub async fn catalog(
    State(state): State<AppState>,
    AuthClaims(_claims): AuthClaims,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<CatalogResponse>, ApiError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(internal("failed to list the tools"))?;
    let switches = ToolSwitches::load(&mut conn).await;
    let tools = state
        .tool_catalog
        .search(query.q.as_deref().unwrap_or_default(), usize::MAX)
        .into_iter()
        .filter(|info| switches.allows(&info.name))
        .take(query.limit.unwrap_or(usize::MAX))
        .map(|info| CatalogTool {
            name: info.name,
            description: info.description,
            group: info.group,
            keywords: info.keywords,
        })
        .collect();
    Ok(Json(CatalogResponse { tools }))
}
