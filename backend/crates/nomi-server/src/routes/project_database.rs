//! A project's databases on Shipyard, for its admins: the development database Shipyard keeps
//! for the app, and the database of its published production environment. Nomi connects with
//! the owner login (revealed server-side, never shown to a model) to list tables and run SQL;
//! the login itself is only shown to an admin who asks for it.

use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Connection, Executor, Row};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::project_ops::ShipyardAccess;
use crate::routes::deploy::{client, internal, require_admin, shipyard_error, ApiError};

const MAX_ROWS: usize = 500;
const STATEMENT_TIMEOUT_SECS: u64 = 15;

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    #[default]
    Development,
    Production,
}

#[derive(Deserialize)]
pub struct SourceQuery {
    #[serde(default)]
    pub source: Source,
}

#[derive(Serialize)]
pub struct DatabaseInfo {
    pub source: &'static str,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub user: Option<String>,
}

/// The owner login of the chosen database. The development one is created when missing.
pub(crate) async fn owner_url(state: &impl ShipyardAccess, project_id: Uuid, source: Source) -> Result<String, ApiError> {
    let client = client(state)?;
    let target = crate::shipyard::load_target(state.pool(), project_id)
        .await
        .map_err(internal("failed to load project"))?
        .ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))?;
    match source {
        Source::Development => {
            let app = crate::shipyard::ensure_app(state.pool(), &client, &target).await.map_err(|e| match e {
                crate::shipyard::EnsureError::Shipyard(e) => shipyard_error(e),
                crate::shipyard::EnsureError::Db(e) => internal("failed to prepare the database")(e),
            })?;
            let dev = client.ensure_dev_database(&target.org_key(), &app).await.map_err(shipyard_error)?;
            let owner_ref = dev
                .get("owner_url_ref")
                .and_then(Value::as_str)
                .ok_or((StatusCode::BAD_GATEWAY, "Shipyard didn't return the database login".to_string()))?;
            client.reveal(owner_ref).await.map_err(shipyard_error)
        }
        Source::Production => {
            let app = target
                .shipyard_app
                .as_deref()
                .ok_or((StatusCode::NOT_FOUND, "this project hasn't been published yet".to_string()))?;
            // The environment's own login, kept by Shipyard as its DATABASE_URL secret.
            let secrets = client.list_secrets(&target.org_key(), app).await.map_err(shipyard_error)?;
            let secret_ref = secrets
                .get("secrets")
                .and_then(Value::as_array)
                .or_else(|| secrets.as_array())
                .and_then(|list| {
                    list.iter().find(|s| {
                        s.get("name").and_then(Value::as_str) == Some("DATABASE_URL")
                            && s.get("environment").and_then(Value::as_str) == Some("production")
                    })
                })
                .and_then(|s| s.get("ref").and_then(Value::as_str))
                .ok_or((StatusCode::NOT_FOUND, "the published app has no database".to_string()))?;
            client.reveal(secret_ref).await.map_err(shipyard_error)
        }
    }
}

fn describe(url: &str, source: Source) -> DatabaseInfo {
    let parsed = reqwest::Url::parse(url).ok();
    DatabaseInfo {
        source: if source == Source::Development { "development" } else { "production" },
        host: parsed.as_ref().and_then(|u| u.host_str().map(str::to_string)),
        port: parsed.as_ref().and_then(|u| u.port()).or(Some(5432)),
        database: parsed.as_ref().map(|u| u.path().trim_start_matches('/').to_string()).filter(|d| !d.is_empty()),
        user: parsed.as_ref().map(|u| u.username().to_string()).filter(|u| !u.is_empty()),
    }
}

pub(crate) async fn connect(url: &str) -> Result<sqlx::PgConnection, ApiError> {
    match tokio::time::timeout(Duration::from_secs(10), sqlx::PgConnection::connect(url)).await {
        Ok(Ok(conn)) => Ok(conn),
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "couldn't connect to a project database");
            Err((StatusCode::BAD_GATEWAY, "Nomi couldn't connect to this database".to_string()))
        }
        Err(_) => Err((StatusCode::GATEWAY_TIMEOUT, "the database didn't answer".to_string())),
    }
}

/// `GET /api/projects/:id/database?source=`: where the database is (no password).
pub async fn info(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<DatabaseInfo>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let url = owner_url(&state, project_id, query.source).await?;
    Ok(Json(describe(&url, query.source)))
}

#[derive(Serialize)]
pub struct Credentials {
    pub url: String,
}

/// `POST /api/projects/:id/database/credentials?source=`: the full login, for the admin's own
/// database client.
pub async fn credentials(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<Credentials>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    tracing::info!(%project_id, user = %claims.sub, "database login revealed");
    Ok(Json(Credentials { url: owner_url(&state, project_id, query.source).await? }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Table {
    pub schema: String,
    pub name: String,
    /// Postgres' estimate; -1 before the table was first analyzed.
    pub rows_estimate: i64,
}

/// `GET /api/projects/:id/database/tables?source=`
pub async fn tables(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<Vec<Table>>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let url = owner_url(&state, project_id, query.source).await?;
    Ok(Json(list_tables(&url).await?))
}

/// The tables in the database at `url`.
pub(crate) async fn list_tables(url: &str) -> Result<Vec<Table>, ApiError> {
    let mut conn = connect(url).await?;
    let tables: Vec<Table> = sqlx::query_as(
        "SELECT n.nspname AS schema, c.relname AS name, c.reltuples::bigint AS rows_estimate \
         FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE c.relkind IN ('r', 'p', 'v', 'm') AND n.nspname NOT IN ('pg_catalog', 'information_schema') \
           AND n.nspname NOT LIKE 'pg_toast%' \
         ORDER BY n.nspname, c.relname",
    )
    .fetch_all(&mut conn)
    .await
    .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;
    Ok(tables)
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,
    #[serde(default)]
    pub source: Source,
}

#[derive(Serialize, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    /// More rows matched than are shown.
    pub truncated: bool,
    /// For statements that change data.
    pub rows_affected: Option<u64>,
    pub took_ms: u128,
}

/// Whether the statement returns rows and can be wrapped to read them as JSON.
fn reads_rows(sql: &str) -> bool {
    let first = sql
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("--"))
        .collect::<Vec<_>>()
        .join(" ");
    let word = first.split(|c: char| c.is_whitespace() || c == '(').find(|w| !w.is_empty()).unwrap_or_default().to_lowercase();
    matches!(word.as_str(), "select" | "with" | "table" | "values")
}

/// `POST /api/projects/:id/database/query`: runs SQL as the database's owner. Reads return up
/// to 500 rows; statements time out after 15 seconds.
pub async fn run_query(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResult>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let sql = req.sql.trim().trim_end_matches(';').trim().to_string();
    if sql.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "write a query first".to_string()));
    }
    let url = owner_url(&state, project_id, req.source).await?;
    Ok(Json(execute_sql(&url, &sql, Access::ReadWrite).await?))
}

/// Whether a statement may change data. Agents only ever read: the database refuses writes in a
/// read-only transaction, whatever the statement is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    ReadWrite,
    ReadOnly,
}

/// Runs `sql` on the database at `url`: reads return up to `MAX_ROWS` rows, statements time out
/// after `STATEMENT_TIMEOUT_SECS`.
pub(crate) async fn execute_sql(url: &str, sql: &str, access: Access) -> Result<QueryResult, ApiError> {
    if access == Access::ReadOnly && !reads_rows(sql) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "only reads are allowed here (SELECT, WITH, TABLE or VALUES)".to_string()));
    }
    let mut conn = connect(url).await?;
    let started = Instant::now();
    let sql_error = |e: sqlx::Error| match e {
        sqlx::Error::Database(db) => (StatusCode::UNPROCESSABLE_ENTITY, db.message().to_string()),
        other => (StatusCode::UNPROCESSABLE_ENTITY, other.to_string()),
    };
    let mut tx = conn.begin().await.map_err(internal("failed to start the query"))?;
    tx.execute(format!("SET LOCAL statement_timeout = '{STATEMENT_TIMEOUT_SECS}s'").as_str())
        .await
        .map_err(sql_error)?;
    if access == Access::ReadOnly {
        tx.execute("SET TRANSACTION READ ONLY").await.map_err(sql_error)?;
    }
    let mut result = QueryResult::default();
    if reads_rows(sql) {
        let described = (&mut *tx).describe(sql).await.map_err(sql_error)?;
        result.columns = described.columns().iter().map(|c| sqlx::Column::name(c).to_string()).collect();
        let wrapped = format!("SELECT row_to_json(t)::text FROM ({sql}) t LIMIT {}", MAX_ROWS + 1);
        let rows = sqlx::query(&wrapped).fetch_all(&mut *tx).await.map_err(sql_error)?;
        result.truncated = rows.len() > MAX_ROWS;
        for row in rows.into_iter().take(MAX_ROWS) {
            let text: String = row.try_get(0).map_err(sql_error)?;
            let object: serde_json::Map<String, Value> = serde_json::from_str(&text).unwrap_or_default();
            result.rows.push(result.columns.iter().map(|c| object.get(c).cloned().unwrap_or(Value::Null)).collect());
        }
    } else {
        let done = tx.execute(sql).await.map_err(sql_error)?;
        result.rows_affected = Some(done.rows_affected());
    }
    tx.commit().await.map_err(sql_error)?;
    result.took_ms = started.elapsed().as_millis();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::reads_rows;

    #[test]
    fn tells_reads_from_writes() {
        assert!(reads_rows("SELECT * FROM task"));
        assert!(reads_rows("-- recent\n  with x as (select 1) select * from x"));
        assert!(reads_rows("(select 1)"));
        assert!(!reads_rows("insert into task (title) values ('a')"));
        assert!(!reads_rows("DELETE FROM task"));
        assert!(!reads_rows("create table t (id int)"));
    }
}
