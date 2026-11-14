//! A project's database for its admins, beyond reading: a copy of it as a SQL file, and fixing
//! one row by its primary key. The browser never sends SQL here: tables and columns are checked
//! against the database's own catalog and quoted by Nomi, values are bound as parameters.
//!
//! The server image has no `pg_dump`, so the dump is written from the catalog and the rows:
//! schemas, enum types, sequences, tables with their defaults and constraints, the rows as
//! INSERTs, then indexes, foreign keys and views, in one REPEATABLE READ snapshot.

use std::collections::BTreeSet;

use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sqlx::{Connection, Executor, PgConnection, Row};
use tokio::sync::mpsc;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, require_admin, ApiError};
use crate::routes::project_database::{connect, owner_url, Source, SourceQuery};

/// How long a row edit may run.
const EDIT_TIMEOUT_SECS: u64 = 15;
/// The most columns one edit may change.
const MAX_CHANGES: usize = 200;
/// How much of the dump is gathered before it's sent on.
const DUMP_CHUNK_BYTES: usize = 64 * 1024;
/// How many chunks may wait for a slow download.
const DUMP_QUEUE: usize = 8;
/// Schemas that belong to Postgres, not the app.
/// Tables and views an extension made: the extension makes them again.
const NOT_FROM_EXTENSION: &str =
    "NOT EXISTS (SELECT 1 FROM pg_depend d WHERE d.objid = c.oid AND d.deptype = 'e')";
/// Types an extension made.
const NOT_FROM_EXTENSION_TYPE: &str =
    "NOT EXISTS (SELECT 1 FROM pg_depend d WHERE d.objid = t.oid AND d.deptype = 'e')";
const SYSTEM_SCHEMAS: &str =
    "n.nspname NOT IN ('pg_catalog', 'information_schema') AND n.nspname NOT LIKE 'pg\\_%'";

/// `"name"`, with any quote inside doubled.
fn ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// `'text'`, with any quote inside doubled.
fn literal(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

fn qualified(schema: &str, name: &str) -> String {
    format!("{}.{}", ident(schema), ident(name))
}

/// `todo-app-development-20261115-0930.sql`
fn dump_file_name(project: &str, source: Source, now: chrono::DateTime<chrono::Utc>) -> String {
    let slug: String = project
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.is_empty() {
        "database".to_string()
    } else {
        slug
    };
    let source = if source == Source::Development {
        "development"
    } else {
        "production"
    };
    format!("{slug}-{source}-{}.sql", now.format("%Y%m%d-%H%M"))
}

/// Gathers the dump's text and sends it on in chunks.
struct Writer {
    buffer: String,
    tx: mpsc::Sender<Result<Bytes, std::io::Error>>,
}

impl Writer {
    async fn line(&mut self, text: &str) -> Result<(), ()> {
        self.buffer.push_str(text);
        self.buffer.push('\n');
        if self.buffer.len() >= DUMP_CHUNK_BYTES {
            self.flush().await?;
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), ()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let chunk = std::mem::take(&mut self.buffer);
        // The download was dropped: stop reading.
        self.tx.send(Ok(Bytes::from(chunk))).await.map_err(|_| ())
    }
}

#[derive(Debug)]
enum DumpError {
    Sql(sqlx::Error),
    Gone,
}

impl From<sqlx::Error> for DumpError {
    fn from(e: sqlx::Error) -> Self {
        Self::Sql(e)
    }
}

impl From<()> for DumpError {
    fn from(_: ()) -> Self {
        Self::Gone
    }
}

struct TableInfo {
    oid: sqlx::postgres::types::Oid,
    schema: String,
    name: String,
}

async fn write_dump(conn: &mut PgConnection, out: &mut Writer) -> Result<(), DumpError> {
    let mut tx = conn.begin().await?;
    tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut *tx)
        .await?;
    out.line(&format!(
        "-- Nomi's dump of {database}, {}",
        chrono::Utc::now().to_rfc3339()
    ))
    .await?;
    out.line("SET client_encoding = 'UTF8';\nSET standard_conforming_strings = on;\nBEGIN;\n")
        .await?;

    let extensions: Vec<String> = sqlx::query_scalar(
        "SELECT extname::text FROM pg_extension WHERE extname <> 'plpgsql' ORDER BY 1",
    )
    .fetch_all(&mut *tx)
    .await?;
    for extension in &extensions {
        out.line(&format!(
            "CREATE EXTENSION IF NOT EXISTS {};",
            ident(extension)
        ))
        .await?;
    }
    let schemas: Vec<String> = sqlx::query_scalar(&format!("SELECT n.nspname FROM pg_namespace n WHERE {SYSTEM_SCHEMAS} AND n.nspname <> 'public' ORDER BY 1"))
        .fetch_all(&mut *tx)
        .await?;
    for schema in &schemas {
        out.line(&format!("CREATE SCHEMA IF NOT EXISTS {};", ident(schema)))
            .await?;
    }

    let enums: Vec<(String, String, Vec<String>)> = sqlx::query_as(&format!(
        "SELECT n.nspname, t.typname, array_agg(e.enumlabel::text ORDER BY e.enumsortorder) \
         FROM pg_type t JOIN pg_enum e ON e.enumtypid = t.oid JOIN pg_namespace n ON n.oid = t.typnamespace \
         WHERE {SYSTEM_SCHEMAS} AND {NOT_FROM_EXTENSION_TYPE} GROUP BY 1, 2 ORDER BY 1, 2"
    ))
    .fetch_all(&mut *tx)
    .await?;
    for (schema, name, labels) in &enums {
        let labels: Vec<String> = labels.iter().map(|l| literal(l)).collect();
        out.line(&format!(
            "CREATE TYPE {} AS ENUM ({});",
            qualified(schema, name),
            labels.join(", ")
        ))
        .await?;
    }

    // Sequences of their own (serial columns'); an identity column's comes with its table.
    let sequences: Vec<(String, String, Option<i64>)> = sqlx::query_as(
        "SELECT s.schemaname::text, s.sequencename::text, s.last_value FROM pg_sequences s \
         JOIN pg_class c ON c.relname = s.sequencename JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = s.schemaname \
         WHERE s.schemaname NOT IN ('pg_catalog', 'information_schema') \
           AND NOT EXISTS (SELECT 1 FROM pg_depend d WHERE d.objid = c.oid AND d.deptype = 'i') ORDER BY 1, 2",
    )
    .fetch_all(&mut *tx)
    .await?;
    for (schema, name, _) in &sequences {
        out.line(&format!(
            "CREATE SEQUENCE IF NOT EXISTS {};",
            qualified(schema, name)
        ))
        .await?;
    }

    let tables: Vec<TableInfo> = sqlx::query(&format!(
        "SELECT c.oid, n.nspname, c.relname FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE c.relkind = 'r' AND NOT c.relispartition AND {SYSTEM_SCHEMAS} AND {NOT_FROM_EXTENSION} ORDER BY 2, 3"
    ))
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|r| TableInfo { oid: r.get(0), schema: r.get(1), name: r.get(2) })
    .collect();

    let mut after: Vec<String> = Vec::new();
    for table in &tables {
        let columns: Vec<(String, String, bool, Option<String>, String, String)> = sqlx::query_as(
            "SELECT a.attname::text, format_type(a.atttypid, a.atttypmod), a.attnotnull, pg_get_expr(d.adbin, d.adrelid), \
                    a.attgenerated::text, a.attidentity::text \
             FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
             WHERE a.attrelid = $1 AND a.attnum > 0 AND NOT a.attisdropped ORDER BY a.attnum",
        )
        .bind(table.oid)
        .fetch_all(&mut *tx)
        .await?;
        let constraints: Vec<(String, String, String)> =
            sqlx::query_as("SELECT conname::text, pg_get_constraintdef(oid), contype::text FROM pg_constraint WHERE conrelid = $1 ORDER BY contype, conname")
                .bind(table.oid)
                .fetch_all(&mut *tx)
                .await?;
        let full = qualified(&table.schema, &table.name);
        let mut parts: Vec<String> = Vec::new();
        for (name, kind, not_null, default, generated, identity) in &columns {
            let mut part = format!("    {} {kind}", ident(name));
            match (generated.as_str(), identity.as_str(), default) {
                ("s", _, Some(expr)) => {
                    part.push_str(&format!(" GENERATED ALWAYS AS ({expr}) STORED"))
                }
                (_, "a", _) => part.push_str(" GENERATED ALWAYS AS IDENTITY"),
                (_, "d", _) => part.push_str(" GENERATED BY DEFAULT AS IDENTITY"),
                (_, _, Some(expr)) => part.push_str(&format!(" DEFAULT {expr}")),
                _ => {}
            }
            if *not_null {
                part.push_str(" NOT NULL");
            }
            parts.push(part);
        }
        for (name, definition, kind) in &constraints {
            match kind.as_str() {
                // Not-null constraints are on their columns already.
                "n" => {}
                "f" => after.push(format!(
                    "ALTER TABLE {full} ADD CONSTRAINT {} {definition};",
                    ident(name)
                )),
                _ => parts.push(format!("    CONSTRAINT {} {definition}", ident(name))),
            }
        }
        out.line(&format!(
            "\nCREATE TABLE {full} (\n{}\n);",
            parts.join(",\n")
        ))
        .await?;

        // The rows, as INSERTs; generated columns are worked out again on restore.
        let stored: Vec<&String> = columns
            .iter()
            .filter(|c| c.4.is_empty())
            .map(|c| &c.0)
            .collect();
        if !stored.is_empty() {
            let names = stored
                .iter()
                .map(|c| ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            let values = stored
                .iter()
                .map(|c| format!("quote_nullable({})", ident(c)))
                .collect::<Vec<_>>()
                .join(", ");
            let select = format!("SELECT concat_ws(', ', {values}) FROM {full}");
            let mut rows = sqlx::query_scalar::<_, String>(&select).fetch(&mut *tx);
            while let Some(row) = rows.next().await {
                out.line(&format!(
                    "INSERT INTO {full} ({names}) OVERRIDING SYSTEM VALUE VALUES ({});",
                    row?
                ))
                .await?;
            }
        }
        for (name, _, _, _, _, identity) in &columns {
            if !identity.is_empty() {
                after.push(format!(
                    "SELECT setval(pg_get_serial_sequence({}, {}), COALESCE((SELECT max({}) FROM {full}), 1));",
                    literal(&full),
                    literal(name),
                    ident(name)
                ));
            }
        }
        let indexes: Vec<String> = sqlx::query_scalar(
            "SELECT pg_get_indexdef(i.indexrelid) FROM pg_index i WHERE i.indrelid = $1 \
             AND NOT EXISTS (SELECT 1 FROM pg_constraint c WHERE c.conindid = i.indexrelid) ORDER BY 1",
        )
        .bind(table.oid)
        .fetch_all(&mut *tx)
        .await?;
        after.extend(indexes.into_iter().map(|i| format!("{i};")));
    }

    out.line("").await?;
    for (schema, name, last_value) in &sequences {
        if let Some(value) = last_value {
            out.line(&format!(
                "SELECT setval({}, {value});",
                literal(&qualified(schema, name))
            ))
            .await?;
        }
    }
    for statement in &after {
        out.line(statement).await?;
    }
    let views: Vec<(String, String, String)> = sqlx::query_as(&format!(
        "SELECT n.nspname, c.relname, pg_get_viewdef(c.oid) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE c.relkind = 'v' AND {SYSTEM_SCHEMAS} AND {NOT_FROM_EXTENSION} ORDER BY c.oid"
    ))
    .fetch_all(&mut *tx)
    .await?;
    for (schema, name, definition) in &views {
        out.line(&format!(
            "CREATE VIEW {} AS {}",
            qualified(schema, name),
            definition.trim()
        ))
        .await?;
    }
    out.line("\nCOMMIT;").await?;
    tx.rollback().await?;
    out.flush().await?;
    Ok(())
}

/// `GET /api/projects/:id/database/dump?source=`: the database as a `.sql` file, streamed.
pub async fn dump(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SourceQuery>,
) -> Result<Response, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let url = owner_url(&state, project_id, query.source).await?;
    let mut conn = connect(&url).await?;
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load project"))?;
    let file = dump_file_name(
        name.as_deref().unwrap_or_default(),
        query.source,
        chrono::Utc::now(),
    );
    tracing::info!(%project_id, user = %claims.sub, "database dump downloaded");

    let (tx, rx) = mpsc::channel(DUMP_QUEUE);
    tokio::spawn(async move {
        let mut out = Writer {
            buffer: String::new(),
            tx: tx.clone(),
        };
        match write_dump(&mut conn, &mut out).await {
            Ok(()) | Err(DumpError::Gone) => {}
            Err(DumpError::Sql(e)) => {
                tracing::warn!(error = %e, "database dump stopped");
                let _ = tx
                    .send(Err(std::io::Error::other("the dump stopped")))
                    .await;
            }
        }
    });
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|chunk| (chunk, rx))
    });
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/sql; charset=utf-8".to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{file}\""),
            ),
        ],
        Body::from_stream(stream),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct RowEdit {
    /// The row's primary key: every key column, nothing else.
    pub key: Map<String, Value>,
    /// The columns to change and their new values (null clears one).
    pub changes: Map<String, Value>,
    #[serde(default)]
    pub source: Source,
}

/// A JSON value as the text Postgres reads into the column's type.
fn as_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

/// `PATCH /api/projects/:id/database/tables/:schema/:table/rows {key, changes, source?}` →
/// `{row}`, the row as it is now.
pub async fn update_row(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, schema, table)): Path<(Uuid, String, String)>,
    Json(edit): Json<RowEdit>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let bad = |message: String| (StatusCode::BAD_REQUEST, message);
    if edit.changes.is_empty() {
        return Err(bad("nothing to change".to_string()));
    }
    if edit.changes.len() > MAX_CHANGES {
        return Err(bad(format!("at most {MAX_CHANGES} columns at once")));
    }
    let url = owner_url(&state, project_id, edit.source).await?;
    let mut conn = connect(&url).await?;
    let sql_error = |e: sqlx::Error| match e {
        sqlx::Error::Database(db) => (StatusCode::UNPROCESSABLE_ENTITY, db.message().to_string()),
        other => (StatusCode::UNPROCESSABLE_ENTITY, other.to_string()),
    };

    let oid: Option<sqlx::postgres::types::Oid> = sqlx::query_scalar(
        "SELECT c.oid FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = $1 AND c.relname = $2 AND c.relkind IN ('r', 'p')",
    )
    .bind(&schema)
    .bind(&table)
    .fetch_optional(&mut conn)
    .await
    .map_err(sql_error)?;
    let oid = oid.ok_or((StatusCode::NOT_FOUND, "there's no such table".to_string()))?;
    // name → (type, can be written)
    let columns: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT attname::text, format_type(atttypid, atttypmod), attgenerated = '' AND attidentity <> 'a' \
         FROM pg_attribute WHERE attrelid = $1 AND attnum > 0 AND NOT attisdropped",
    )
    .bind(oid)
    .fetch_all(&mut conn)
    .await
    .map_err(sql_error)?;
    let primary: BTreeSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT a.attname::text FROM pg_index i JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) \
         WHERE i.indrelid = $1 AND i.indisprimary",
    )
    .bind(oid)
    .fetch_all(&mut conn)
    .await
    .map_err(sql_error)?
    .into_iter()
    .collect();
    if primary.is_empty() {
        return Err((
            StatusCode::CONFLICT,
            "this table has no primary key, so one row can't be picked out".to_string(),
        ));
    }
    let given: BTreeSet<String> = edit.key.keys().cloned().collect();
    if given != primary {
        return Err(bad(format!(
            "the key is the primary key's columns: {}",
            primary.iter().cloned().collect::<Vec<_>>().join(", ")
        )));
    }
    let column = |name: &str| columns.iter().find(|c| c.0 == name);

    let mut values: Vec<Option<String>> = Vec::new();
    let mut sets = Vec::new();
    for (name, value) in &edit.changes {
        let (_, kind, writable) =
            column(name).ok_or_else(|| bad(format!("there's no column {name}")))?;
        if !writable {
            return Err(bad(format!(
                "{name} is worked out by the database and can't be set"
            )));
        }
        values.push(as_text(value));
        sets.push(format!("{} = ${}::text::{kind}", ident(name), values.len()));
    }
    let mut wheres = Vec::new();
    for (name, value) in &edit.key {
        let (_, kind, _) = column(name).ok_or_else(|| bad(format!("there's no column {name}")))?;
        let text = as_text(value).ok_or_else(|| bad(format!("the key's {name} can't be null")))?;
        values.push(Some(text));
        wheres.push(format!("{} = ${}::text::{kind}", ident(name), values.len()));
    }
    let sql = format!(
        "UPDATE {} AS t SET {} WHERE {} RETURNING row_to_json(t)::text",
        qualified(&schema, &table),
        sets.join(", "),
        wheres.join(" AND ")
    );

    let mut tx = conn
        .begin()
        .await
        .map_err(internal("failed to start the edit"))?;
    tx.execute(format!("SET LOCAL statement_timeout = '{EDIT_TIMEOUT_SECS}s'").as_str())
        .await
        .map_err(sql_error)?;
    let mut statement = sqlx::query_scalar::<_, String>(&sql);
    for value in values {
        statement = statement.bind(value);
    }
    let rows = statement.fetch_all(&mut *tx).await.map_err(sql_error)?;
    match rows.len() {
        0 => {
            return Err((
                StatusCode::NOT_FOUND,
                "that row isn't there any more".to_string(),
            ))
        }
        1 => {}
        _ => {
            return Err((
                StatusCode::CONFLICT,
                "more than one row matched; nothing was changed".to_string(),
            ))
        }
    }
    tx.commit().await.map_err(sql_error)?;
    tracing::info!(%project_id, user = %claims.sub, %schema, %table, "database row edited");
    let row: Value = serde_json::from_str(&rows[0]).unwrap_or(Value::Null);
    Ok(Json(json!({ "row": row })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn names_and_text_are_quoted() {
        assert_eq!(ident("a\"b"), "\"a\"\"b\"");
        assert_eq!(literal("it's"), "'it''s'");
        assert_eq!(qualified("public", "task"), "\"public\".\"task\"");
    }

    #[test]
    fn values_are_read_as_text() {
        assert_eq!(as_text(&json!(null)), None);
        assert_eq!(as_text(&json!("a")), Some("a".into()));
        assert_eq!(as_text(&json!(3)), Some("3".into()));
        assert_eq!(as_text(&json!({"a": 1})), Some("{\"a\":1}".into()));
    }

    #[test]
    fn dump_files_are_named_after_the_project() {
        let at = chrono::Utc
            .with_ymd_and_hms(2026, 11, 15, 9, 30, 0)
            .unwrap();
        assert_eq!(
            dump_file_name("Warung Kopi!", Source::Production, at),
            "warung-kopi-production-20261115-0930.sql"
        );
        assert_eq!(
            dump_file_name("", Source::Development, at),
            "database-development-20261115-0930.sql"
        );
    }
}
