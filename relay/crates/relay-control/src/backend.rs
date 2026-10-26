//! Making and deleting a project's backend: its roles, its schema (its own in a shared
//! database, or `public` in a dedicated one), the `nomi` schema next to it, the grants that keep
//! each project to itself, and its API keys.

use std::collections::BTreeMap;

use relay_core::keys::{api_key, ApiRole};
use relay_core::names::{public_url, role, schema_for, RoleKind};
use relay_core::{Env, Tier};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{postgrest, state, Relay, RelayError};

const NOMI_SCHEMA_SQL: &str = include_str!("nomi_schema.sql");
/// Postgres' "duplicate object", raised when two calls create the same role at once.
const DUPLICATE_OBJECT: &str = "42710";

/// A dedicated project's own databases.
#[derive(Debug, Clone, Deserialize)]
pub struct Databases {
    pub dev: String,
    pub production: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentKeys {
    pub url: String,
    pub anon_key: String,
    pub service_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendKeys {
    pub id: Uuid,
    pub slug: String,
    pub tier: &'static str,
    /// Keyed by `dev` and `production`.
    pub environments: BTreeMap<String, EnvironmentKeys>,
}

/// Identifiers Relay makes are always plain (`app_<hex>_<kind>`, `public`); quoting is still done.
fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

async fn install_nomi_schema(pool: &PgPool) -> Result<(), RelayError> {
    sqlx::raw_sql(NOMI_SCHEMA_SQL).execute(pool).await?;
    Ok(())
}

async fn ensure_role(pool: &PgPool, name: &str) -> Result<(), RelayError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)").bind(name).fetch_one(pool).await?;
    if !exists {
        if let Err(e) = sqlx::query(&format!("CREATE ROLE {} NOLOGIN", quoted(name))).execute(pool).await {
            let duplicate = e.as_database_error().and_then(|d| d.code()).is_some_and(|code| code == DUPLICATE_OBJECT);
            if !duplicate {
                return Err(e.into());
            }
        }
    }
    // Relay switches to the project's roles (SET ROLE) to run its migrations and requests.
    sqlx::query(&format!("GRANT {} TO CURRENT_USER", quoted(name))).execute(pool).await?;
    Ok(())
}

/// Roles, schema and grants for `project` in one database.
async fn provision(pool: &PgPool, project: Uuid, tier: Tier) -> Result<(), RelayError> {
    install_nomi_schema(pool).await?;
    for kind in RoleKind::ALL {
        ensure_role(pool, &role(project, kind)).await?;
    }
    let owner = quoted(&role(project, RoleKind::Owner));
    let users = [RoleKind::Anon, RoleKind::Authenticated, RoleKind::Service].map(|k| quoted(&role(project, k))).join(", ");
    let schema = quoted(&schema_for(project, tier));
    let statements = match tier {
        Tier::Free => vec![format!("CREATE SCHEMA IF NOT EXISTS {schema} AUTHORIZATION {owner}")],
        Tier::Dedicated => vec![format!("GRANT USAGE, CREATE ON SCHEMA {schema} TO {owner}")],
    }
    .into_iter()
    .chain([
        format!("GRANT USAGE ON SCHEMA {schema} TO {users}"),
        format!("ALTER DEFAULT PRIVILEGES FOR ROLE {owner} IN SCHEMA {schema} GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO {users}"),
        format!("ALTER DEFAULT PRIVILEGES FOR ROLE {owner} IN SCHEMA {schema} GRANT USAGE, SELECT ON SEQUENCES TO {users}"),
        format!("ALTER DEFAULT PRIVILEGES FOR ROLE {owner} IN SCHEMA {schema} GRANT EXECUTE ON FUNCTIONS TO {users}"),
        format!("GRANT USAGE ON SCHEMA nomi TO {owner}, {users}"),
        // The project's tables point at its users; row-level security keeps the rows its own.
        format!("GRANT SELECT, REFERENCES ON nomi.users TO {owner}, {users}"),
        format!("GRANT SELECT ON nomi.migrations TO {owner}"),
    ])
    .collect::<Vec<_>>();
    for statement in statements {
        sqlx::query(&statement).execute(pool).await?;
    }
    Ok(())
}

/// Makes `project`'s backend in both environments, or does nothing if it exists. Answers its
/// address and keys per environment (the same ones every time).
pub async fn ensure_backend(relay: &Relay, project: Uuid, name: &str, tier: Tier, databases: Option<Databases>) -> Result<BackendKeys, RelayError> {
    if tier == Tier::Dedicated && databases.is_none() {
        return Err(RelayError::rejected("BAD_REQUEST", "a dedicated backend needs its dev and production databases"));
    }
    if let Some(existing) = state::environment(relay, project, Env::Dev).await? {
        if existing.tier != tier {
            return Err(RelayError::rejected("CONFLICT", "this project's backend is already on another tier; moving it is a separate step"));
        }
    }
    state::record(relay, project, name, tier, databases.as_ref().map(|d| (d.dev.as_str(), d.production.as_str()))).await?;

    let mut environments = BTreeMap::new();
    let mut slug = String::new();
    for env in Env::ALL {
        let row = state::environment(relay, project, env).await?.ok_or_else(|| RelayError::Internal("environment wasn't recorded".into()))?;
        let pool = match &row.database_url {
            Some(url) => relay.dedicated(url).await?,
            None => relay.shared(env)?,
        };
        provision(&pool, project, row.tier).await?;
        if let Some(authenticator) = &relay.postgrest_role {
            postgrest::open_project(&pool, authenticator, project).await?;
            if row.tier == Tier::Free {
                postgrest::refresh_schemas(&pool, authenticator).await?;
            }
        }
        environments.insert(
            env.as_str().to_string(),
            EnvironmentKeys {
                url: public_url(&relay.public_domain, &row.slug, env),
                anon_key: api_key(&row.signing_secret, project, env, ApiRole::Anon, row.keys_issued_at),
                service_key: api_key(&row.signing_secret, project, env, ApiRole::Service, row.keys_issued_at),
            },
        );
        slug = row.slug;
    }
    Ok(BackendKeys { id: project, slug, tier: tier.as_str(), environments })
}

/// Drops exactly `project`'s database objects, its rows in `nomi`, its roles and Relay's record.
pub async fn delete_backend(relay: &Relay, project: Uuid) -> Result<(), RelayError> {
    let mut pools = Vec::new();
    for env in Env::ALL {
        if let Ok(placement) = relay.placement(project, env).await {
            pools.push(placement);
        }
    }
    if pools.is_empty() {
        return Err(RelayError::NotFound("no backend for this project".into()));
    }
    let roles: Vec<String> = RoleKind::ALL.iter().map(|k| role(project, *k)).collect();
    for placement in &pools {
        if placement.tier == Tier::Free {
            sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", quoted(&placement.schema))).execute(&placement.pool).await?;
        }
        for table in ["sessions", "identities", "users", "migrations", "buckets"] {
            sqlx::query(&format!("DELETE FROM nomi.{table} WHERE project_id = $1")).bind(project).execute(&placement.pool).await?;
        }
        for name in &roles {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)").bind(name).fetch_one(&placement.pool).await?;
            if exists {
                // What the role still owns or was granted in this database (a dedicated project's tables, grants on nomi).
                sqlx::query(&format!("DROP OWNED BY {}", quoted(name))).execute(&placement.pool).await?;
            }
        }
    }
    for name in &roles {
        sqlx::query(&format!("DROP ROLE IF EXISTS {}", quoted(name))).execute(&pools[0].pool).await?;
    }
    if let Some(authenticator) = &relay.postgrest_role {
        for placement in pools.iter().filter(|p| p.tier == Tier::Free) {
            postgrest::refresh_schemas(&placement.pool, authenticator).await?;
        }
    }
    state::forget(relay, project).await
}
