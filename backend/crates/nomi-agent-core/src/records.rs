//! Private record storage for agents without tables of their own (every dynamic agent, and any
//! built-in that opts in through `SubAgent::uses_records`). Each record lives in a named
//! collection ("trips", "contacts", ...) and is scoped to the calling agent and user: these
//! functions take the agent type from the engine, never from the model's input, so one agent can
//! neither see nor change another's records.

use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

pub const SAVE_RECORD_TOOL_NAME: &str = "save_record";
pub const LIST_RECORDS_TOOL_NAME: &str = "list_records";
pub const UPDATE_RECORD_TOOL_NAME: &str = "update_record";
pub const DELETE_RECORD_TOOL_NAME: &str = "delete_record";

pub const RECORD_TOOL_NAMES: [&str; 4] = [SAVE_RECORD_TOOL_NAME, LIST_RECORDS_TOOL_NAME, UPDATE_RECORD_TOOL_NAME, DELETE_RECORD_TOOL_NAME];

const LIST_LIMIT: i64 = 50;
const MAX_RECORD_BYTES: usize = 16 * 1024;

pub fn record_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: SAVE_RECORD_TOOL_NAME.to_string(),
            description: "Save a record in your own private storage for this user, in a named collection \
                          (e.g. \"trips\", \"contacts\"). Use it for anything you'll need again later."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Short lowercase name, e.g. trips"},
                    "data": {"type": "object", "description": "The record's fields"}
                },
                "required": ["collection", "data"]
            }),
        },
        ToolDefinition {
            name: LIST_RECORDS_TOOL_NAME.to_string(),
            description: "List the records you saved in one collection, newest first.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"collection": {"type": "string"}},
                "required": ["collection"]
            }),
        },
        ToolDefinition {
            name: UPDATE_RECORD_TOOL_NAME.to_string(),
            description: "Change fields of one of your records (the given fields replace those keys).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "data": {"type": "object"}},
                "required": ["id", "data"]
            }),
        },
        ToolDefinition {
            name: DELETE_RECORD_TOOL_NAME.to_string(),
            description: "Delete one of your records.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
        },
    ]
}

fn collection_of(input: &Value) -> Result<String, String> {
    let collection = input.get("collection").and_then(|v| v.as_str()).unwrap_or_default().trim().to_lowercase();
    if collection.is_empty() || collection.len() > 64 {
        return Err("collection must be a short name".to_string());
    }
    Ok(collection)
}

fn data_of(input: &Value) -> Result<Value, String> {
    let data = input.get("data").filter(|d| d.is_object()).cloned().ok_or("data must be an object")?;
    if data.to_string().len() > MAX_RECORD_BYTES {
        return Err("record is too large (16 KB max)".to_string());
    }
    Ok(data)
}

fn id_of(input: &Value) -> Result<Uuid, String> {
    input.get("id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or("id must be a record id".to_string())
}

/// Runs one record tool for `agent_type`. Returns `None` when `name` isn't a record tool.
pub async fn execute(conn: &mut PgConnection, agent_type: &str, user_id: Uuid, name: &str, input: &Value) -> Option<Result<String, String>> {
    let result = match name {
        SAVE_RECORD_TOOL_NAME => save(conn, agent_type, user_id, input).await,
        LIST_RECORDS_TOOL_NAME => list(conn, agent_type, user_id, input).await,
        UPDATE_RECORD_TOOL_NAME => update(conn, agent_type, user_id, input).await,
        DELETE_RECORD_TOOL_NAME => delete(conn, agent_type, user_id, input).await,
        _ => return None,
    };
    Some(result)
}

async fn save(conn: &mut PgConnection, agent_type: &str, user_id: Uuid, input: &Value) -> Result<String, String> {
    let collection = collection_of(input)?;
    let data = data_of(input)?;
    let id: Uuid = sqlx::query_scalar("INSERT INTO agent_records (agent_type, user_id, collection, data) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(agent_type)
        .bind(user_id)
        .bind(&collection)
        .bind(&data)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Saved to {collection} (id {id})."))
}

async fn list(conn: &mut PgConnection, agent_type: &str, user_id: Uuid, input: &Value) -> Result<String, String> {
    let collection = collection_of(input)?;
    let rows: Vec<(Uuid, Value)> = sqlx::query_as(
        "SELECT id, data FROM agent_records WHERE agent_type = $1 AND user_id = $2 AND collection = $3 ORDER BY updated_at DESC LIMIT $4",
    )
    .bind(agent_type)
    .bind(user_id)
    .bind(&collection)
    .bind(LIST_LIMIT)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Ok(format!("No records in {collection} yet."));
    }
    Ok(rows.iter().map(|(id, data)| format!("{id}: {data}")).collect::<Vec<_>>().join("\n"))
}

async fn update(conn: &mut PgConnection, agent_type: &str, user_id: Uuid, input: &Value) -> Result<String, String> {
    let id = id_of(input)?;
    let data = data_of(input)?;
    let updated = sqlx::query(
        "UPDATE agent_records SET data = data || $4, updated_at = now() WHERE id = $1 AND agent_type = $2 AND user_id = $3",
    )
    .bind(id)
    .bind(agent_type)
    .bind(user_id)
    .bind(&data)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if updated == 0 {
        return Err("no record of yours with that id".to_string());
    }
    Ok("Updated.".to_string())
}

async fn delete(conn: &mut PgConnection, agent_type: &str, user_id: Uuid, input: &Value) -> Result<String, String> {
    let id = id_of(input)?;
    let deleted = sqlx::query("DELETE FROM agent_records WHERE id = $1 AND agent_type = $2 AND user_id = $3")
        .bind(id)
        .bind(agent_type)
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();
    if deleted == 0 {
        return Err("no record of yours with that id".to_string());
    }
    Ok("Deleted.".to_string())
}
