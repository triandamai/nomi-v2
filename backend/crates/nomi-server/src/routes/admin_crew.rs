//! Admin → Agents: every built-in crew member, and whether it is switched on for everyone. A
//! member that is off is in nobody's crew: it isn't listed, routed to, mentioned, delegated to or
//! asked, exactly like one a person switched off themselves. Nomi (the default agent) can't be
//! switched off, since every chat falls back to it. Custom agents keep their own page.

use std::collections::BTreeSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use nomi_auth::extractor::AuthClaims;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::routes::admin_access::require_admin;
use crate::routes::agents::{built_in_role, first_sentence};

#[derive(Debug, Serialize, PartialEq)]
pub struct AdminCrewMember {
    pub agent_type: String,
    pub name: String,
    /// What it does, in a sentence.
    pub role: String,
    /// Switched on for everyone.
    pub enabled: bool,
    /// It can't be switched off: Nomi, the agent every chat falls back to.
    pub locked: bool,
}

#[derive(Serialize)]
pub struct AdminCrewResponse {
    pub agents: Vec<AdminCrewMember>,
}

/// A built-in member as the registry describes it.
pub struct Registered {
    pub agent_type: String,
    pub name: String,
    pub role: String,
    pub is_default: bool,
}

/// The crew as the page lists it: Nomi first, then the rest in registry order, each with whether
/// it is on (`off` holds the members switched off).
pub fn crew_view(registered: Vec<Registered>, off: &BTreeSet<String>) -> Vec<AdminCrewMember> {
    let mut members: Vec<AdminCrewMember> = registered
        .into_iter()
        .map(|r| AdminCrewMember { enabled: r.is_default || !off.contains(&r.agent_type), locked: r.is_default, agent_type: r.agent_type, name: r.name, role: r.role })
        .collect();
    members.sort_by_key(|m| !m.locked);
    members
}

/// Whether `agent_type` may be switched: it is a built-in member, and not the default one.
pub fn switchable(members: &[AdminCrewMember], agent_type: &str) -> Result<(), (StatusCode, &'static str)> {
    match members.iter().find(|m| m.agent_type == agent_type) {
        None => Err((StatusCode::NOT_FOUND, "no such agent")),
        Some(member) if member.locked => Err((StatusCode::CONFLICT, "Nomi can't be switched off")),
        Some(_) => Ok(()),
    }
}

async fn members(state: &AppState, user_id: uuid::Uuid) -> Result<Vec<AdminCrewMember>, (StatusCode, &'static str)> {
    let locale = {
        let mut conn = state.pool.acquire().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agents"))?;
        nomi_agent_core::user_locale(&mut conn, user_id).await
    };
    let registry = crate::build_agent_registry(&state.pool, state.project_storage.clone(), state.tool_catalog.clone());
    let registered = registry
        .agents()
        .iter()
        .map(|agent| {
            let agent_type = agent.agent_type().into_owned();
            Registered {
                role: built_in_role(&agent_type, locale).unwrap_or_else(|| first_sentence(&agent.intent_description())),
                name: agent.display_name().into_owned(),
                is_default: agent.is_default(),
                agent_type,
            }
        })
        .collect();
    Ok(crew_view(registered, &nomi_settings::config::agents_off_now().await))
}

/// `GET /api/admin/crew`: every built-in member and whether it is on.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<AdminCrewResponse>, (StatusCode, &'static str)> {
    require_admin(&claims, "agents", "view")?;
    Ok(Json(AdminCrewResponse { agents: members(&state, claims.sub).await? }))
}

#[derive(Deserialize)]
pub struct SetEnabled {
    pub enabled: bool,
}

/// `PUT /api/admin/crew/:agent_type`: switches a member on or off for everyone.
pub async fn set_enabled(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(agent_type): Path<String>,
    Json(input): Json<SetEnabled>,
) -> Result<Json<AdminCrewResponse>, (StatusCode, &'static str)> {
    require_admin(&claims, "agents", "write")?;
    switchable(&members(&state, claims.sub).await?, &agent_type)?;
    // Read what is saved now, not what this process cached a few seconds ago.
    nomi_settings::config::forget();
    nomi_settings::config::set_agent_on(&state.pool, &agent_type, input.enabled, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, %agent_type, "admin crew: the switch couldn't be saved");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save")
    })?;
    Ok(Json(AdminCrewResponse { agents: members(&state, claims.sub).await? }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered() -> Vec<Registered> {
        let member = |agent_type: &str, name: &str, is_default: bool| Registered { agent_type: agent_type.into(), name: name.into(), role: format!("{name}'s job"), is_default };
        vec![member("money", "Dana", false), member("chitchat", "Nomi", true), member("design", "Sora", false)]
    }

    #[test]
    fn every_member_is_listed_on_unless_switched_off_with_nomi_first() {
        let all_on = crew_view(registered(), &BTreeSet::new());
        assert_eq!(all_on.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["Nomi", "Dana", "Sora"]);
        assert!(all_on.iter().all(|m| m.enabled), "nothing saved: Sora is on like everyone else");

        let off: BTreeSet<String> = ["design".to_string(), "chitchat".to_string()].into();
        let some_off = crew_view(registered(), &off);
        assert!(!some_off.iter().find(|m| m.agent_type == "design").unwrap().enabled);
        let nomi = some_off.iter().find(|m| m.agent_type == "chitchat").unwrap();
        assert!(nomi.enabled && nomi.locked, "Nomi stays on whatever is saved");
    }

    #[test]
    fn only_a_built_in_member_other_than_nomi_can_be_switched() {
        let members = crew_view(registered(), &BTreeSet::new());
        assert_eq!(switchable(&members, "design"), Ok(()));
        assert_eq!(switchable(&members, "chitchat").unwrap_err().0, StatusCode::CONFLICT);
        assert_eq!(switchable(&members, "made_up").unwrap_err().0, StatusCode::NOT_FOUND);
    }
}
