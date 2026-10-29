//! Opening projects to PostgREST: it logs in as one role (the authenticator) and switches to the
//! project's role per request, so the authenticator is granted each project's roles; it serves
//! the schemas listed in its `pgrst.db_schemas` setting for that database, which Relay keeps
//! equal to the free projects' schemas there.

use relay_core::names::{role, RoleKind};
use sqlx::PgPool;
use uuid::Uuid;

use crate::RelayError;

/// The longest a single app request may run in the database.
pub const REQUEST_STATEMENT_TIMEOUT: &str = "8s";
/// Serialises rewrites of `pgrst.db_schemas`, so two projects made at once both end up listed.
const SCHEMAS_LOCK: i64 = 0x7265_6c61_7900_0001;
/// A free project's schema: `app_` and its id without dashes.
const FREE_SCHEMA_PATTERN: &str = "^app_[0-9a-f]{32}$";
/// Postgres' "duplicate object", raised when two starts create the role at once.
const DUPLICATE_OBJECT: &str = "42710";
/// What Postgres says when two sessions change the same role at once (roles are shared by the
/// whole cluster, so Relays started together against one cluster can meet here).
const CONCURRENT_ROLE_UPDATE: &str = "tuple concurrently updated";
/// How many times a role change is tried, and how long to wait between tries.
const ROLE_CHANGE_ATTEMPTS: u32 = 10;
const ROLE_CHANGE_RETRY: std::time::Duration = std::time::Duration::from_millis(50);

fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// The login PostgREST uses next to a standalone Relay (`relay_core::settings::AUTHENTICATOR`),
/// made when missing and given `password` every time, so it matches `RELAY_PGRST_PASSWORD`.
/// NOINHERIT: it only becomes a project's role when a request's token says so.
pub async fn ensure_authenticator(pool: &PgPool, password: &str) -> Result<(), RelayError> {
    let mut attempt = 1;
    loop {
        match make_authenticator(pool, password).await {
            Err(RelayError::Internal(message)) if message.contains(CONCURRENT_ROLE_UPDATE) && attempt < ROLE_CHANGE_ATTEMPTS => {
                attempt += 1;
                tokio::time::sleep(ROLE_CHANGE_RETRY).await;
            }
            result => return result,
        }
    }
}

async fn make_authenticator(pool: &PgPool, password: &str) -> Result<(), RelayError> {
    let name = quoted(relay_core::settings::AUTHENTICATOR);
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
        .bind(relay_core::settings::AUTHENTICATOR)
        .fetch_one(pool)
        .await?;
    if !exists {
        if let Err(e) = sqlx::query(&format!("CREATE ROLE {name} LOGIN NOINHERIT")).execute(pool).await {
            let duplicate = e.as_database_error().and_then(|d| d.code()).is_some_and(|code| code == DUPLICATE_OBJECT);
            if !duplicate {
                return Err(e.into());
            }
        }
    }
    sqlx::query(&format!("ALTER ROLE {name} LOGIN NOINHERIT PASSWORD {}", literal(password))).execute(pool).await?;
    Ok(())
}

async fn require_role(pool: &PgPool, authenticator: &str) -> Result<(), RelayError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)").bind(authenticator).fetch_one(pool).await?;
    if exists {
        Ok(())
    } else {
        Err(RelayError::Internal(format!("PostgREST's role {authenticator} doesn't exist in this database's cluster")))
    }
}

/// Lets PostgREST switch to `project`'s roles and limits how long their requests may run.
pub async fn open_project(pool: &PgPool, authenticator: &str, project: Uuid) -> Result<(), RelayError> {
    require_role(pool, authenticator).await?;
    for kind in [RoleKind::Anon, RoleKind::Authenticated, RoleKind::Owner] {
        sqlx::query(&format!("GRANT {} TO {}", quoted(&role(project, kind)), quoted(authenticator))).execute(pool).await?;
    }
    for kind in [RoleKind::Anon, RoleKind::Authenticated] {
        sqlx::query(&format!("ALTER ROLE {} SET statement_timeout = '{REQUEST_STATEMENT_TIMEOUT}'", quoted(&role(project, kind)))).execute(pool).await?;
    }
    Ok(())
}

/// Sets the authenticator's `pgrst.db_schemas` in this database to every free project's schema
/// here, and tells PostgREST to reload.
pub async fn refresh_schemas(pool: &PgPool, authenticator: &str) -> Result<(), RelayError> {
    require_role(pool, authenticator).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(SCHEMAS_LOCK).execute(&mut *tx).await?;
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&mut *tx).await?;
    let schemas: Vec<String> = sqlx::query_scalar("SELECT nspname::text FROM pg_namespace WHERE nspname ~ $1 ORDER BY nspname")
        .bind(FREE_SCHEMA_PATTERN)
        .fetch_all(&mut *tx)
        .await?;
    let target = format!("ROLE {} IN DATABASE {}", quoted(authenticator), quoted(&database));
    let statement = if schemas.is_empty() {
        format!("ALTER {target} RESET pgrst.db_schemas")
    } else {
        // Schema names match FREE_SCHEMA_PATTERN, so they're safe inside the literal.
        format!("ALTER {target} SET pgrst.db_schemas = '{}'", schemas.join(", "))
    };
    sqlx::query(&statement).execute(&mut *tx).await?;
    sqlx::query("NOTIFY pgrst, 'reload config'").execute(&mut *tx).await?;
    sqlx::query("NOTIFY pgrst, 'reload schema'").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
