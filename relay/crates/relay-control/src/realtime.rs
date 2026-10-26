//! A project's realtime tables per environment, as `nomi/backend.json`'s `realtime` says: each
//! gets a `nomi_realtime` trigger announcing its changes on the `nomi_realtime` channel, which the
//! gateway listens to; tables taken off the list lose theirs.

use relay_core::backend::realtime_problems;
use relay_core::names::{role, RoleKind};
use relay_core::Env;
use uuid::Uuid;

use crate::backend::{install_nomi_schema, quoted};
use crate::storage::invalid;
use crate::{state, Relay, RelayError};

/// The channel changes are announced on.
pub const CHANNEL: &str = "nomi_realtime";
const TRIGGER: &str = "nomi_realtime";

/// Replaces `env`'s realtime tables. Every table must exist and have a primary key; when one
/// doesn't, nothing changes.
pub async fn save(relay: &Relay, project: Uuid, env: Env, tables: &[String]) -> Result<(), RelayError> {
    let problems = realtime_problems(tables);
    if !problems.is_empty() {
        return Err(invalid(problems));
    }
    state::environment(relay, project, env).await?.ok_or_else(|| RelayError::NotFound("no backend for this project".into()))?;
    let placement = relay.placement(project, env).await?;
    install_nomi_schema(&placement.pool).await?;

    let mut keys = Vec::new();
    let mut problems = Vec::new();
    for table in tables {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = $1 AND tablename = $2)")
            .bind(&placement.schema)
            .bind(table)
            .fetch_one(&placement.pool)
            .await?;
        if !exists {
            problems.push(format!("no table {table} yet (write its migration first)"));
            continue;
        }
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT a.attname::text FROM pg_index i JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) \
             WHERE i.indrelid = (quote_ident($1) || '.' || quote_ident($2))::regclass AND i.indisprimary ORDER BY array_position(i.indkey, a.attnum)",
        )
        .bind(&placement.schema)
        .bind(table)
        .fetch_all(&placement.pool)
        .await?;
        if columns.is_empty() {
            problems.push(format!("{table} has no primary key, which realtime needs to say which row changed"));
        } else {
            keys.push((table, columns));
        }
    }
    if !problems.is_empty() {
        return Err(invalid(problems));
    }

    let schema = quoted(&placement.schema);
    let mut tx = placement.pool.begin().await?;
    // As the project's owner, who owns its tables.
    sqlx::query(&format!("SET LOCAL ROLE {}", quoted(&role(project, RoleKind::Owner)))).execute(&mut *tx).await?;
    let current: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE t.tgname = $1 AND n.nspname = $2",
    )
    .bind(TRIGGER)
    .bind(&placement.schema)
    .fetch_all(&mut *tx)
    .await?;
    for table in current {
        sqlx::query(&format!("DROP TRIGGER IF EXISTS {TRIGGER} ON {schema}.{}", quoted(&table))).execute(&mut *tx).await?;
    }
    for (table, columns) in &keys {
        let arguments: Vec<String> = std::iter::once(project.to_string()).chain(columns.iter().cloned()).map(|a| format!("'{}'", a.replace('\'', "''"))).collect();
        sqlx::query(&format!(
            "CREATE TRIGGER {TRIGGER} AFTER INSERT OR UPDATE OR DELETE ON {schema}.{} FOR EACH ROW EXECUTE FUNCTION nomi.realtime_notify({})",
            quoted(table),
            arguments.join(", ")
        ))
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    sqlx::query("INSERT INTO relay.realtime_tables (project_id, env, tables) VALUES ($1, $2, $3) ON CONFLICT (project_id, env) DO UPDATE SET tables = EXCLUDED.tables")
        .bind(project)
        .bind(env.as_str())
        .bind(tables)
        .execute(&relay.state)
        .await?;
    Ok(())
}

/// `env`'s realtime tables; none until `backend.json` lists some.
pub async fn load(relay: &Relay, project: Uuid, env: Env) -> Result<Vec<String>, RelayError> {
    let tables: Option<Vec<String>> = sqlx::query_scalar("SELECT tables FROM relay.realtime_tables WHERE project_id = $1 AND env = $2")
        .bind(project)
        .bind(env.as_str())
        .fetch_optional(&relay.state)
        .await?;
    Ok(tables.unwrap_or_default())
}
