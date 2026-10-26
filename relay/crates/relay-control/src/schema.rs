//! A project's schema read back from the catalog: a description (tables, columns, row-level
//! security and policies) for Koda's `describe_backend`, and `src/lib/nomi.types.ts` in the shape
//! the `nomi-relay` client expects.

use relay_core::Env;
use serde::Serialize;
use uuid::Uuid;

use crate::{Relay, RelayError};

#[derive(Debug, Serialize)]
pub struct Column {
    pub name: String,
    /// Postgres' name for the type (`text`, `int8`, `_text` for an array of text).
    pub r#type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub identity: bool,
}

#[derive(Debug, Serialize)]
pub struct Policy {
    pub name: String,
    pub command: String,
    pub using: Option<String>,
    pub check: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Table {
    pub name: String,
    pub rls_enabled: bool,
    pub columns: Vec<Column>,
    pub policies: Vec<Policy>,
}

#[derive(Debug, Serialize)]
pub struct Schema {
    pub tables: Vec<Table>,
}

pub async fn describe(relay: &Relay, project: Uuid, env: Env) -> Result<Schema, RelayError> {
    let placement = relay.placement(project, env).await?;
    // Read as the project sees itself: its own tables unqualified, everything else (nomi.uid())
    // with its schema, whatever the connecting user's own search path is.
    let mut tx = placement.pool.begin().await?;
    sqlx::query(&format!("SET LOCAL search_path = \"{}\"", placement.schema.replace('"', "\"\""))).execute(&mut *tx).await?;
    let pool = &mut *tx;
    let tables: Vec<(String, bool)> = sqlx::query_as(
        "SELECT c.relname::text, c.relrowsecurity FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = $1 AND c.relkind IN ('r', 'p') ORDER BY c.relname",
    )
    .bind(&placement.schema)
    .fetch_all(&mut *pool)
    .await?;
    let columns: Vec<(String, String, String, bool, Option<String>, bool)> = sqlx::query_as(
        "SELECT table_name::text, column_name::text, udt_name::text, is_nullable = 'YES', column_default::text, is_identity = 'YES' \
         FROM information_schema.columns WHERE table_schema = $1 ORDER BY table_name, ordinal_position",
    )
    .bind(&placement.schema)
    .fetch_all(&mut *pool)
    .await?;
    let policies: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT tablename::text, policyname::text, cmd::text, qual, with_check FROM pg_policies WHERE schemaname = $1 ORDER BY tablename, policyname",
    )
    .bind(&placement.schema)
    .fetch_all(&mut *pool)
    .await?;
    tx.rollback().await?;
    Ok(Schema {
        tables: tables
            .into_iter()
            .map(|(name, rls_enabled)| Table {
                columns: columns
                    .iter()
                    .filter(|c| c.0 == name)
                    .map(|c| Column { name: c.1.clone(), r#type: c.2.clone(), nullable: c.3, default: c.4.clone(), identity: c.5 })
                    .collect(),
                policies: policies
                    .iter()
                    .filter(|p| p.0 == name)
                    .map(|p| Policy { name: p.1.clone(), command: p.2.clone(), using: p.3.clone(), check: p.4.clone() })
                    .collect(),
                name,
                rls_enabled,
            })
            .collect(),
    })
}

/// The TypeScript type for a Postgres type, as PostgREST sends it in JSON.
fn ts_type(udt: &str) -> String {
    if let Some(element) = udt.strip_prefix('_') {
        return format!("{}[]", ts_type(element));
    }
    match udt {
        "int2" | "int4" | "int8" | "float4" | "float8" | "numeric" | "oid" => "number",
        "bool" => "boolean",
        "json" | "jsonb" => "Json",
        "text" | "varchar" | "bpchar" | "char" | "name" | "citext" | "uuid" | "date" | "time" | "timetz" | "timestamp" | "timestamptz"
        | "interval" | "bytea" | "inet" | "cidr" | "macaddr" => "string",
        _ => "unknown",
    }
    .to_string()
}

fn property(name: &str) -> String {
    let plain = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if plain {
        name.to_string()
    } else {
        format!("\"{}\"", name.replace('"', "\\\""))
    }
}

fn fields(columns: &[Column], optional: impl Fn(&Column) -> bool) -> String {
    let parts: Vec<String> = columns
        .iter()
        .map(|c| {
            let ty = if c.nullable { format!("{} | null", ts_type(&c.r#type)) } else { ts_type(&c.r#type) };
            format!("{}{}: {ty}", property(&c.name), if optional(c) { "?" } else { "" })
        })
        .collect();
    format!("{{ {} }}", parts.join("; "))
}

/// The file Nomi writes to `src/lib/nomi.types.ts`.
pub fn typescript(schema: &Schema) -> String {
    let mut out = String::from(
        "// Written by Nomi Relay from the project's dev database after each migration. Don't edit it:\n\
         // it's rewritten every time. Use it as createClient<Database>(…).\n\n\
         export type Json = string | number | boolean | null | { [key: string]: Json } | Json[];\n\n\
         export interface Database {\n\ttables: {\n",
    );
    for table in &schema.tables {
        out.push_str(&format!("\t\t{}: {{\n", property(&table.name)));
        out.push_str(&format!("\t\t\tRow: {};\n", fields(&table.columns, |_| false)));
        out.push_str(&format!("\t\t\tInsert: {};\n", fields(&table.columns, |c| c.nullable || c.default.is_some() || c.identity)));
        out.push_str(&format!("\t\t\tUpdate: {};\n", fields(&table.columns, |_| true)));
        out.push_str("\t\t};\n");
    }
    out.push_str("\t};\n\tfunctions: Record<string, never>;\n}\n");
    out
}

pub async fn types(relay: &Relay, project: Uuid, env: Env) -> Result<String, RelayError> {
    Ok(typescript(&describe(relay, project, env).await?))
}
