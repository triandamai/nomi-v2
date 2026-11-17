//! A project's token usage and spend, a month at a time (`nomi_usage::project`). Anyone who can
//! open the project sees its totals and models; who spent what is for the project's admins.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, ApiError};
use crate::routes::usage::{usd_to_idr, UsageQuery};
use nomi_auth::extractor::AuthClaims;
use nomi_usage::project::ProjectUsage;

/// The role that sees each person's part.
const SEES_PEOPLE: &str = "admin";

#[derive(Serialize)]
pub struct ProjectUsageView {
    #[serde(flatten)]
    pub usage: ProjectUsage,
    /// Whether `by_person` was filled in for this viewer.
    pub shows_people: bool,
    /// Rupiah to the dollar, to show `spend_usd` in rupiah.
    pub usd_to_idr: u64,
}

/// Leaves out who spent what unless `role` may see it.
pub fn for_role(mut usage: ProjectUsage, role: &str) -> (ProjectUsage, bool) {
    let shows_people = role == SEES_PEOPLE;
    if !shows_people {
        usage.by_person.clear();
    }
    (usage, shows_people)
}

/// `GET /api/projects/:id/usage?month=`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>, Query(query): Query<UsageQuery>) -> Result<Json<ProjectUsageView>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let usage = nomi_usage::project::project_month(&state.pool, project_id, claims.sub, query.month.as_deref())
        .await
        .map_err(internal("failed to read the project's usage"))?;
    let (usage, shows_people) = for_role(usage, role);
    Ok(Json(ProjectUsageView { usage, shows_people, usd_to_idr: usd_to_idr().await }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_usage::project::PersonUsage;

    fn usage() -> ProjectUsage {
        ProjectUsage {
            month: "2026-11".into(),
            current_month: "2026-11".into(),
            timezone: "Asia/Jakarta".into(),
            counted_since: None,
            tokens: 10,
            input_tokens: 8,
            output_tokens: 2,
            cached_input_tokens: 0,
            own_key_tokens: 0,
            calls: 1,
            spend_usd: 0.5,
            by_model: vec![],
            by_person: vec![PersonUsage { user_id: Uuid::nil(), name: Some("Budi Santoso".into()), tokens: 10, calls: 1, spend_usd: 0.5 }],
        }
    }

    #[test]
    fn only_a_projects_admins_see_who_spent_what() {
        let (seen, shows) = for_role(usage(), "admin");
        assert!(shows);
        assert_eq!(seen.by_person.len(), 1);
        for role in ["member", "viewer"] {
            let (seen, shows) = for_role(usage(), role);
            assert!(!shows);
            assert!(seen.by_person.is_empty());
            assert_eq!(seen.tokens, 10);
        }
    }
}
