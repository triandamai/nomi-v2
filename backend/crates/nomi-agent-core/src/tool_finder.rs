//! Finding tools and making crew members with them: the `find_tools` tool every agent has, the
//! `create_agent` tool Nomi and the supervisor have, and the `tool_catalog_entries` table that
//! lets `find_tools` match a tool by meaning (embeddings), not only by its words.
//!
//! `tool_catalog_entries` mirrors the Rust `ToolCatalog` (synced at startup). It is also the seam
//! for tools defined without Rust later: they would be listed there too. Nothing reads such rows
//! yet, and the sync below still removes rows the Rust catalog doesn't have.

use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::pool::PoolConnection;
use sqlx::{Connection, PgConnection, PgPool, Postgres};
use uuid::Uuid;

use nomi_embedding::EmbeddingProvider;

use crate::personal_crew::{self, NewMember};
use crate::registry::AgentRegistry;
use crate::subagent::SubAgent;
use crate::tool_catalog::{ToolCatalog, ToolInfo};
use crate::tools::ToolSwitches;

/// How many tools `find_tools` returns when the agent doesn't say.
pub const FIND_TOOLS_DEFAULT_LIMIT: usize = 8;
/// The most tools one `find_tools` call returns.
pub const FIND_TOOLS_MAX_LIMIT: usize = 20;
/// A tool further than this (cosine distance) from the query isn't about it, however few matched.
pub const SIMILAR_TOOL_MAX_DISTANCE: f64 = 0.6;

/// The text a tool is embedded from, and what its `content_hash` covers.
pub fn embedding_text(info: &ToolInfo) -> String {
    let keywords = info.keywords.join(", ");
    format!("{}: {} ({}; {})", info.name, info.description, info.group, keywords)
}

/// Changes whenever the tool's name, description, group or keywords do: its embedding is redone.
pub fn content_hash(info: &ToolInfo) -> String {
    format!("{:x}", Sha256::digest(embedding_text(info).as_bytes()))
}

/// Which embedding model made a stored vector: vectors of different models aren't comparable.
pub fn model_tag(provider: &dyn EmbeddingProvider) -> String {
    format!("{}/{}", provider.provider_name(), provider.model_id())
}

/// Brings `tool_catalog_entries` in line with `catalog`: new and changed tools are written (a
/// changed one loses its embedding), tools no longer in the catalog are removed.
pub async fn sync_catalog_table(conn: &mut PgConnection, catalog: &ToolCatalog) -> Result<(), sqlx::Error> {
    let entries = catalog.entries();
    let mut tx = conn.begin().await?;
    for info in &entries {
        let keywords: Vec<String> = info.keywords.iter().map(|k| k.to_string()).collect();
        sqlx::query(
            "INSERT INTO tool_catalog_entries (name, description, tool_group, keywords, content_hash) VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description, tool_group = EXCLUDED.tool_group, \
                 keywords = EXCLUDED.keywords, content_hash = EXCLUDED.content_hash, embedding = NULL, embedding_model = NULL, updated_at = now() \
             WHERE tool_catalog_entries.content_hash <> EXCLUDED.content_hash",
        )
        .bind(&info.name)
        .bind(&info.description)
        .bind(info.group)
        .bind(&keywords)
        .bind(content_hash(info))
        .execute(&mut *tx)
        .await?;
    }
    let names: Vec<String> = entries.into_iter().map(|info| info.name).collect();
    sqlx::query("DELETE FROM tool_catalog_entries WHERE NOT (name = ANY($1))").bind(&names).execute(&mut *tx).await?;
    tx.commit().await
}

/// Embeds every catalog row without an embedding from `provider`'s model. Stops at the first
/// failure (the provider is down or not set up): the rows stay as they are, and `find_tools`
/// matches by words alone until a later startup fills them. Returns how many were embedded.
pub async fn embed_catalog_entries(pool: &PgPool, provider: &dyn EmbeddingProvider) -> usize {
    let tag = model_tag(provider);
    let rows: Vec<(String, String, String, Vec<String>, String)> = match sqlx::query_as(
        "SELECT name, description, tool_group, keywords, content_hash FROM tool_catalog_entries \
         WHERE embedding IS NULL OR embedding_model IS DISTINCT FROM $1",
    )
    .bind(&tag)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "tool catalog: couldn't read the rows to embed");
            return 0;
        }
    };
    let mut embedded = 0;
    for (name, description, group, keywords, hash) in rows {
        let text = format!("{name}: {description} ({group}; {})", keywords.join(", "));
        let vector = match provider.embed(&text).await {
            Ok(v) if v.len() == crate::memory::STORED_DIMENSIONS => v,
            Ok(v) => {
                tracing::warn!(model = %tag, got = v.len(), expected = crate::memory::STORED_DIMENSIONS, "tool catalog: embedding model has the wrong width; tools stay unembedded");
                break;
            }
            Err(e) => {
                tracing::warn!(error = %e, model = %tag, "tool catalog: embedding failed; tools stay unembedded for now");
                break;
            }
        };
        let saved = sqlx::query(
            "UPDATE tool_catalog_entries SET embedding = $2::vector, embedding_model = $3, updated_at = now() WHERE name = $1 AND content_hash = $4",
        )
        .bind(&name)
        .bind(crate::memory::to_vector_literal(&vector))
        .bind(&tag)
        .bind(&hash)
        .execute(pool)
        .await;
        match saved {
            Ok(_) => embedded += 1,
            Err(e) => {
                tracing::warn!(error = %e, tool = %name, "tool catalog: couldn't save an embedding");
                break;
            }
        }
    }
    embedded
}

/// The catalog tools nearest `query` by meaning, nearest first, within `SIMILAR_TOOL_MAX_DISTANCE`.
async fn similar_tool_names(conn: &mut PgConnection, provider: &dyn EmbeddingProvider, query: &str, limit: usize) -> Result<Vec<String>, String> {
    let vector = provider.embed_for_query(query).await.map_err(|e| e.to_string())?;
    if vector.len() != crate::memory::STORED_DIMENSIONS {
        return Err(format!("embedding has {} dimensions", vector.len()));
    }
    sqlx::query_scalar(
        "SELECT name FROM tool_catalog_entries \
         WHERE embedding IS NOT NULL AND embedding_model = $2 AND (embedding <=> $1::vector) <= $3 \
         ORDER BY embedding <=> $1::vector LIMIT $4",
    )
    .bind(crate::memory::to_vector_literal(&vector))
    .bind(model_tag(provider))
    .bind(SIMILAR_TOOL_MAX_DISTANCE)
    .bind(i64::try_from(limit).unwrap_or(i64::MAX))
    .fetch_all(conn)
    .await
    .map_err(|e| e.to_string())
}

/// `find_tools`' `{query, limit?}`: the query, trimmed, and the limit clamped to 1..=max.
pub fn parse_find_tools_input(input: &Value) -> Result<(String, usize), String> {
    let query = input.get("query").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if query.is_empty() {
        return Err("query is required: a few words about the task".to_string());
    }
    let limit = match input.get("limit") {
        None | Some(Value::Null) => FIND_TOOLS_DEFAULT_LIMIT,
        Some(v) => v.as_u64().and_then(|n| usize::try_from(n).ok()).ok_or("limit is a whole number")?,
    };
    Ok((query.to_string(), limit.clamp(1, FIND_TOOLS_MAX_LIMIT)))
}

/// The word matches first, in their order, then the meaning matches not already in, up to `limit`.
pub fn merge_ranked(by_words: Vec<String>, by_meaning: Vec<String>, limit: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    by_words.into_iter().chain(by_meaning).filter(|name| seen.insert(name.clone())).take(limit).collect()
}

/// Tool name → the members (display names, in `members`' order) that have it.
pub fn carried_by(members: &[(String, Vec<String>)]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (member, tools) in members {
        for tool in tools {
            let carriers = map.entry(tool.clone()).or_default();
            if !carriers.contains(member) {
                carriers.push(member.clone());
            }
        }
    }
    map
}

/// The person's crew other than `caller`, each with its tool names: the built-in members and the
/// dynamic ones they see, without those they switched off.
async fn crew_tools(conn: &mut PoolConnection<Postgres>, registry: &AgentRegistry, caller: &str, user_id: Uuid) -> Vec<(String, Vec<String>)> {
    let off = personal_crew::disabled_agent_types(conn, user_id).await;
    let mut members: Vec<(String, Vec<String>)> = registry
        .agents()
        .iter()
        .filter(|a| a.agent_type() != caller && !off.contains(a.agent_type().as_ref()))
        .map(|a| (a.display_name().into_owned(), a.tools().into_iter().map(|t| t.name).collect()))
        .collect();
    match crate::dynamic_agent::fetch_visible_dynamic_agents(conn, user_id).await {
        Ok(rows) => members.extend(
            rows.into_iter()
                .filter(|r| r.id.to_string() != caller && !off.contains(&r.id.to_string()))
                .map(|r| (r.name, r.granted_tools)),
        ),
        Err(e) => tracing::warn!(error = %e, "find_tools: couldn't read the person's own crew"),
    }
    members
}

/// `find_tools`: the catalog tools that fit `query` (words first, then meaning when an embedding
/// provider is at hand; a failed similarity search leaves the word matches), without the ones an
/// admin switched off. Each says whether `agent` has it and which crew members do. Informs only:
/// it never grants a tool.
#[allow(clippy::too_many_arguments)]
pub async fn find_tools(
    conn: &mut PoolConnection<Postgres>,
    registry: &AgentRegistry,
    catalog: &ToolCatalog,
    embedding: Option<&dyn EmbeddingProvider>,
    switches: &ToolSwitches,
    agent: &dyn SubAgent,
    user_id: Uuid,
    input: &Value,
) -> Result<String, String> {
    let (query, limit) = parse_find_tools_input(input)?;
    let by_words: Vec<String> =
        catalog.search(&query, usize::MAX).into_iter().filter(|info| switches.allows(&info.name)).map(|info| info.name).collect();
    let by_meaning = match embedding {
        Some(provider) if by_words.len() < limit => match similar_tool_names(conn, provider, &query, limit).await {
            Ok(names) => names,
            Err(e) => {
                tracing::debug!(error = %e, "find_tools: similarity search failed; matching by words only");
                Vec::new()
            }
        },
        _ => Vec::new(),
    };
    let entries: HashMap<String, ToolInfo> = catalog.entries().into_iter().map(|info| (info.name.clone(), info)).collect();
    let by_meaning: Vec<String> = by_meaning.into_iter().filter(|name| entries.contains_key(name) && switches.allows(name)).collect();
    // A generic tool (records, reminders, …) the agent can't have is left out, so search never
    // shows it a way round its gates. Area tools stay findable: who carries them is the point.
    let eligible = |name: &String| {
        catalog.get(name).map_or(true, |t| t.auto_offer() == nomi_tool::AutoOffer::GrantOnly || crate::engine::offered_to(t, agent))
    };
    let by_words: Vec<String> = by_words.into_iter().filter(eligible).collect();
    let by_meaning: Vec<String> = by_meaning.into_iter().filter(eligible).collect();
    let names = merge_ranked(by_words, by_meaning, limit);

    let own: HashSet<String> = agent.tools().into_iter().map(|t| t.name).collect();
    let carriers = carried_by(&crew_tools(conn, registry, agent.agent_type().as_ref(), user_id).await);
    let tools: Vec<Value> = names
        .iter()
        .filter_map(|name| entries.get(name))
        .map(|info| {
            json!({
                "name": info.name,
                "description": info.description,
                "group": info.group,
                "granted": switches.allows(&info.name) && (own.contains(&info.name) || catalog.get(&info.name).is_some_and(|t| crate::engine::offered_to(t, agent))),
                "carried_by": carriers.get(&info.name).cloned().unwrap_or_default(),
            })
        })
        .collect();
    Ok(json!({ "tools": tools, "note": crate::prompts::FIND_TOOLS_NOTE }).to_string())
}

/// `create_agent`'s input.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateAgentInput {
    pub name: String,
    pub role: String,
    pub soul: String,
    /// Once each, in the order given.
    pub tools: Vec<String>,
}

/// `create_agent`'s `{name, role, soul?, tools}`.
pub fn parse_create_agent_input(input: &Value) -> Result<CreateAgentInput, String> {
    let text = |key: &str| input.get(key).and_then(Value::as_str).map(|v| v.trim().to_string()).unwrap_or_default();
    let tools: Vec<String> = match input.get("tools") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_str().map(|s| s.trim().to_string()).ok_or("tools is a list of tool names"))
            .collect::<Result<Vec<String>, _>>()?,
        None | Some(Value::Null) => return Err("tools is required: the tool names from find_tools (may be empty)".to_string()),
        Some(_) => return Err("tools is a list of tool names".to_string()),
    };
    let tools = personal_crew::dedupe_tool_names(tools);
    Ok(CreateAgentInput { name: text("name"), role: text("role"), soul: text("soul"), tools })
}

/// `create_agent`: a member of `user_id`'s own crew, made exactly as Settings › Crew makes one
/// (`personal_crew::create_member`: tool names checked, the plan's limit, unique names).
pub async fn create_agent(
    conn: &mut PoolConnection<Postgres>,
    registry: &AgentRegistry,
    catalog: &ToolCatalog,
    user_id: Uuid,
    input: &Value,
) -> Result<String, String> {
    let input = parse_create_agent_input(input)?;
    let crew_names: Vec<String> = registry.agents().iter().map(|a| a.display_name().into_owned()).collect();
    let member = NewMember {
        name: input.name,
        role: input.role,
        soul: input.soul,
        tools: input.tools,
        template_key: None,
        shape: personal_crew::DEFAULT_SHAPE.to_string(),
        tone: personal_crew::DEFAULT_TONE.to_string(),
        motion: personal_crew::DEFAULT_MOTION.to_string(),
    };
    let created = personal_crew::create_member(conn, user_id, member, catalog, &crew_names).await.map_err(|e| {
        if let personal_crew::CreateMemberError::Db(err) = &e {
            tracing::error!(error = %err, "create_agent: failed to add the member");
        }
        e.message()
    })?;
    let tools_text = if created.tools.is_empty() { "no tools".to_string() } else { format!("the tools {}", created.tools.join(", ")) };
    let message = crate::prompts::CREATE_AGENT_DONE.replace("{name}", &created.name).replace("{tools}", &tools_text);
    Ok(json!({ "agent_type": created.agent_type, "name": created.name, "tools": created.tools, "message": message }).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_LIMIT: usize = 3;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn word_matches_come_first_then_new_meaning_matches_up_to_the_limit() {
        let merged = merge_ranked(strings(&["a", "b"]), strings(&["b", "c", "d"]), SMALL_LIMIT);
        assert_eq!(merged, strings(&["a", "b", "c"]));
        assert_eq!(merge_ranked(strings(&["a", "b", "c", "d"]), strings(&["e"]), SMALL_LIMIT), strings(&["a", "b", "c"]));
        assert_eq!(merge_ranked(Vec::new(), strings(&["x"]), SMALL_LIMIT), strings(&["x"]));
    }

    #[test]
    fn find_tools_input_needs_a_query_and_clamps_the_limit() {
        assert_eq!(parse_find_tools_input(&json!({"query": " budget "})).unwrap(), ("budget".to_string(), FIND_TOOLS_DEFAULT_LIMIT));
        assert_eq!(parse_find_tools_input(&json!({"query": "x", "limit": 0})).unwrap().1, 1);
        assert_eq!(parse_find_tools_input(&json!({"query": "x", "limit": 500})).unwrap().1, FIND_TOOLS_MAX_LIMIT);
        assert!(parse_find_tools_input(&json!({"query": "  "})).is_err());
        assert!(parse_find_tools_input(&json!({})).is_err());
        assert!(parse_find_tools_input(&json!({"query": "x", "limit": "many"})).is_err());
    }

    #[test]
    fn carried_by_lists_each_member_once_per_tool() {
        let members = vec![
            ("Dana".to_string(), strings(&["list_budgets", "set_budget"])),
            ("Coach".to_string(), strings(&["list_budgets", "list_budgets"])),
        ];
        let map = carried_by(&members);
        assert_eq!(map["list_budgets"], strings(&["Dana", "Coach"]));
        assert_eq!(map["set_budget"], strings(&["Dana"]));
        assert!(!map.contains_key("gmail_send"));
    }

    #[test]
    fn create_agent_input_needs_a_tools_list_and_dedupes_it() {
        let parsed = parse_create_agent_input(&json!({"name": " Chef ", "role": "cooks", "tools": ["a", "b", "a", " "]})).unwrap();
        assert_eq!(parsed, CreateAgentInput { name: "Chef".into(), role: "cooks".into(), soul: String::new(), tools: strings(&["a", "b"]) });
        assert!(parse_create_agent_input(&json!({"name": "Chef", "role": "cooks"})).is_err());
        assert!(parse_create_agent_input(&json!({"name": "Chef", "role": "cooks", "tools": "a"})).is_err());
        assert!(parse_create_agent_input(&json!({"name": "Chef", "role": "cooks", "tools": [1]})).is_err());
        assert_eq!(parse_create_agent_input(&json!({"name": "C", "role": "r", "soul": "hi", "tools": []})).unwrap().soul, "hi");
    }

    #[test]
    fn content_hash_follows_the_tool_text() {
        let info = ToolInfo { name: "set_budget".into(), description: "Set a budget.".into(), group: "money", keywords: vec!["limit"] };
        let same = info.clone();
        let changed = ToolInfo { description: "Set a monthly budget.".into(), ..info.clone() };
        assert_eq!(content_hash(&info), content_hash(&same));
        assert_ne!(content_hash(&info), content_hash(&changed));
        assert!(embedding_text(&info).contains("limit"));
    }
}
