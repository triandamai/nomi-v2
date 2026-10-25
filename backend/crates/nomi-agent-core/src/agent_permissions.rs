//! What an agent may do on an organization's projects (`agent_permissions`): one row per
//! `<agent>.<org>.<project>.<permission>`, scoped allow or denied. A grant for the project beats
//! one for the whole organization; no row means the agent hasn't been allowed.

use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Allow,
    Denied,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Allow => "allow",
            Scope::Denied => "denied",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "allow" => Some(Scope::Allow),
            "denied" => Some(Scope::Denied),
            _ => None,
        }
    }
}

/// The dotted key of a permission (`project` None: every project in the org).
pub fn key(agent_type: &str, org_id: Uuid, project_id: Option<Uuid>, permission: &str) -> String {
    let project = project_id.map(|id| id.to_string()).unwrap_or_else(|| "*".to_string());
    format!("{agent_type}.{org_id}.{project}.{permission}")
}

/// The scope that applies to `agent_type` doing `permission` on `project_id`: the project's own
/// row, else the organization-wide one, else None (not allowed yet).
pub async fn scope_for(conn: &mut PgConnection, agent_type: &str, project_id: Uuid, permission: &str) -> Result<Option<Scope>, sqlx::Error> {
    let scope: Option<String> = sqlx::query_scalar(
        "SELECT ap.scope FROM agent_permissions ap JOIN projects p ON p.org_id = ap.org_id \
         WHERE p.id = $1 AND ap.agent_type = $2 AND ap.permission = $3 AND (ap.project_id = p.id OR ap.project_id IS NULL) \
         ORDER BY ap.project_id IS NULL LIMIT 1",
    )
    .bind(project_id)
    .bind(agent_type)
    .bind(permission)
    .fetch_optional(conn)
    .await?;
    Ok(scope.as_deref().and_then(Scope::parse))
}

/// Records `scope` for `agent_type` doing `permission` on `project_id` (in the project's org),
/// replacing what was decided before.
pub async fn set_for_project(
    conn: &mut PgConnection,
    agent_type: &str,
    project_id: Uuid,
    permission: &str,
    scope: Scope,
    decided_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO agent_permissions (agent_type, org_id, project_id, permission, scope, decided_by) \
         SELECT $1, p.org_id, p.id, $3, $4, $5 FROM projects p WHERE p.id = $2 AND p.org_id IS NOT NULL \
         ON CONFLICT (key) DO UPDATE SET scope = EXCLUDED.scope, decided_by = EXCLUDED.decided_by, updated_at = now()",
    )
    .bind(agent_type)
    .bind(project_id)
    .bind(permission)
    .bind(scope.as_str())
    .bind(decided_by)
    .execute(conn)
    .await?;
    Ok(())
}

/// Whether `user_id` runs the project (whoever started it, its admins, the organization's owners
/// and admins): only they decide what agents may do on it.
pub async fn can_decide(conn: &mut PgConnection, project_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let admin: Option<bool> = sqlx::query_scalar(
        "SELECT (p.user_id = $2 \
                 OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = p.id AND pm.user_id = $2 AND pm.role = 'admin') \
                 OR EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = p.org_id AND m.user_id = $2 AND m.status = 'active' AND m.role IN ('owner', 'admin'))) \
         FROM projects p WHERE p.id = $1",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(conn)
    .await?;
    Ok(admin == Some(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_read_agent_org_project_permission() {
        let org = Uuid::nil();
        let project = Uuid::from_u128(1);
        assert_eq!(key("inspector", org, Some(project), "read_data"), format!("inspector.{org}.{project}.read_data"));
        assert_eq!(key("inspector", org, None, "read_logs"), format!("inspector.{org}.*.read_logs"));
        assert_eq!(Scope::parse("denied"), Some(Scope::Denied));
        assert_eq!(Scope::parse("maybe"), None);
    }
}
