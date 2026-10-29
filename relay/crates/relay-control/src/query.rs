//! SQL against one environment's tables, for the project's console in Nomi and for Iris (reads
//! only). It runs over a connection logged in as the project's owner role, never as Relay's own
//! user: Relay's user may switch to every project's roles, the owner's login to none.

use std::time::{Duration, Instant};

use relay_core::keys::role_password;
use relay_core::names::{role, RoleKind};
use relay_core::Env;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, Row};
use uuid::Uuid;

use crate::backend::quoted;
use crate::{Relay, RelayError};

/// Rows a read answers with; `truncated` says when more matched.
pub const MAX_ROWS: usize = 500;
const STATEMENT_TIMEOUT_SECS: u64 = 15;
const CONNECT_TIMEOUT_SECS: u64 = 10;
/// Logins the owner role may hold at once: a console and Iris, never a pool.
const OWNER_CONNECTION_LIMIT: u32 = 5;
/// Postgres' "invalid password".
const INVALID_PASSWORD: &str = "28P01";

/// The statements a console may run, by their first word. Anything else (`set`, `reset`, `do`,
/// `copy`, transaction control) is refused before it reaches the database.
const READS: &[&str] = &["select", "with", "table", "values"];
const WRITES: &[&str] = &["insert", "update", "delete", "merge", "create", "alter", "drop", "truncate", "comment", "grant", "revoke"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    ReadWrite,
    /// Iris: a read-only transaction, so not even a function called from a select can write.
    ReadOnly,
}

#[derive(Debug, Default, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub truncated: bool,
    /// For statements that change data.
    pub rows_affected: Option<u64>,
    pub took_ms: u128,
}

fn refused(message: impl Into<String>) -> RelayError {
    RelayError::rejected("QUERY_FAILED", message)
}

fn sql_error(e: sqlx::Error) -> RelayError {
    match e.as_database_error() {
        Some(db) => refused(db.message()),
        None => RelayError::Internal(e.to_string()),
    }
}

fn first_word(sql: &str) -> String {
    sql.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("--"))
        .flat_map(|l| l.split(|c: char| c.is_whitespace() || c == '('))
        .find(|w| !w.is_empty())
        .unwrap_or_default()
        .to_lowercase()
}

/// A connection as `project`'s owner. The login is turned on the first time it's needed.
async fn owner_connection(relay: &Relay, project: Uuid, env: Env) -> Result<(PgConnection, String), RelayError> {
    let placement = relay.placement(project, env).await?;
    let owner = role(project, RoleKind::Owner);
    let password = role_password(&relay.encryption_key, &owner);
    let options = (*placement.pool.connect_options()).clone().username(&owner).password(&password);
    let connect = || async {
        tokio::time::timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS), PgConnection::connect_with(&options))
            .await
            .map_err(|_| RelayError::Internal("the project's database didn't answer".into()))
    };
    let conn = match connect().await? {
        Ok(conn) => conn,
        Err(e) => {
            let not_yet = e.as_database_error().and_then(|d| d.code()).is_some_and(|c| c == INVALID_PASSWORD)
                || !sqlx::query_scalar::<_, bool>("SELECT rolcanlogin FROM pg_roles WHERE rolname = $1").bind(&owner).fetch_one(&placement.pool).await?;
            if !not_yet {
                return Err(e.into());
            }
            sqlx::query(&format!("ALTER ROLE {} LOGIN PASSWORD '{password}' CONNECTION LIMIT {OWNER_CONNECTION_LIMIT}", quoted(&owner)))
                .execute(&placement.pool)
                .await?;
            connect().await??
        }
    };
    Ok((conn, placement.schema))
}

/// Runs one statement in `project`'s `env` as its owner (row-level security doesn't apply to the
/// tables' owner). Reads answer up to `MAX_ROWS` rows; statements time out after 15 seconds.
pub async fn run(relay: &Relay, project: Uuid, env: Env, sql: &str, access: Access) -> Result<QueryResult, RelayError> {
    let sql = sql.trim().trim_end_matches(';').trim();
    if sql.is_empty() {
        return Err(refused("write a query first"));
    }
    let word = first_word(sql);
    let reads = READS.contains(&word.as_str());
    if access == Access::ReadOnly && !reads {
        return Err(refused("only reads are allowed here (SELECT, WITH, TABLE or VALUES)"));
    }
    if !reads && !WRITES.contains(&word.as_str()) {
        return Err(refused(format!("{word} statements can't be run here")));
    }

    let (mut conn, schema) = owner_connection(relay, project, env).await?;
    let started = Instant::now();
    let mut tx = conn.begin().await?;
    tx.execute(format!("SET LOCAL search_path = {}", quoted(&schema)).as_str()).await?;
    tx.execute(format!("SET LOCAL statement_timeout = '{STATEMENT_TIMEOUT_SECS}s'").as_str()).await?;
    if access == Access::ReadOnly {
        tx.execute("SET TRANSACTION READ ONLY").await?;
    }
    let mut result = QueryResult::default();
    // Both forms go over the extended protocol, which takes exactly one statement.
    if reads {
        let described = (&mut *tx).describe(sql).await.map_err(sql_error)?;
        result.columns = described.columns().iter().map(|c| sqlx::Column::name(c).to_string()).collect();
        let wrapped = format!("SELECT row_to_json(t)::text FROM ({sql}) t LIMIT {}", MAX_ROWS + 1);
        let rows = sqlx::query(&wrapped).fetch_all(&mut *tx).await.map_err(sql_error)?;
        result.truncated = rows.len() > MAX_ROWS;
        for row in rows.into_iter().take(MAX_ROWS) {
            let text: String = row.try_get(0)?;
            let object: serde_json::Map<String, Value> = serde_json::from_str(&text).unwrap_or_default();
            result.rows.push(result.columns.iter().map(|c| object.get(c).cloned().unwrap_or(Value::Null)).collect());
        }
    } else {
        let done = sqlx::query(sql).execute(&mut *tx).await.map_err(sql_error)?;
        result.rows_affected = Some(done.rows_affected());
    }
    tx.commit().await.map_err(sql_error)?;
    result.took_ms = started.elapsed().as_millis();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::first_word;

    #[test]
    fn the_first_word_skips_comments_and_parentheses() {
        assert_eq!(first_word("-- recent\n  WITH x as (select 1) select * from x"), "with");
        assert_eq!(first_word("(select 1)"), "select");
        assert_eq!(first_word("RESET role"), "reset");
    }
}
