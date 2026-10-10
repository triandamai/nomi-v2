//! What the crew did on the way to each reply: the tools they used (searching the web, reading
//! a page, writing a file, handing over to someone else) and the pages they read, so the chat can
//! show a reply's steps and sources next to its thinking. Read from the `ToolCalled` events the
//! engine logs; each event belongs to the first reply after it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::sessions::MessageItem;

/// Engine bookkeeping that isn't a step anyone would want to see.
const HIDDEN_TOOLS: &[&str] = &["complete_task", "update_todos", "write_plan", "show_table", "list_files", "read_guide"];
const MAX_STEPS: usize = 80;
/// How much of a command's output a step keeps (its end, where errors are).
const MAX_OUTPUT: usize = 4000;
const MAX_SOURCES: usize = 8;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Step {
    /// The tool's name; the app words it ("Searched the web", "Wrote a file").
    pub tool: String,
    /// What it was used on: the search, the page, the file, the command, who it went to.
    pub detail: Option<String>,
    pub ok: bool,
    /// What a command printed (its end), to read under the step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Source {
    pub title: Option<String>,
    pub url: String,
}

fn clip(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_string();
    }
    format!("{}…", text.chars().take(max).collect::<String>().trim_end())
}

/// The part of a tool's input worth showing.
pub fn step_detail(tool: &str, input: &Value) -> Option<String> {
    let field = |name: &str| input.get(name).and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty());
    let detail = match tool {
        "delegate_to_agent" => field("target_agent").map(str::to_string),
        "write_files" => input.get("files").and_then(|f| f.as_array()).map(|files| {
            let paths: Vec<&str> = files.iter().filter_map(|f| f.get("path").and_then(|p| p.as_str())).collect();
            match paths.as_slice() {
                [] => String::new(),
                [one] => one.to_string(),
                [first, rest @ ..] => format!("{first} +{}", rest.len()),
            }
        }),
        _ => ["query", "url", "path", "command", "title", "name", "description", "category"].iter().find_map(|k| field(k)).map(str::to_string),
    };
    detail.filter(|d| !d.is_empty()).map(|d| clip(&d, 90))
}

/// The end of what a command printed, without the "$ command" line the tool puts first.
pub fn step_output(tool: &str, result: &str) -> Option<String> {
    if tool != "run_command" {
        return None;
    }
    let body = result.strip_prefix('$').and_then(|r| r.split_once('\n')).map_or(result, |(_, rest)| rest).trim();
    if body.is_empty() {
        return None;
    }
    let chars = body.chars().count();
    Some(if chars > MAX_OUTPUT { format!("…{}", body.chars().skip(chars - MAX_OUTPUT).collect::<String>()) } else { body.to_string() })
}

/// The pages a step looked at: the page it read, or the results a search came back with.
pub fn step_sources(tool: &str, input: &Value, result: &str) -> Vec<Source> {
    match tool {
        "read_web_page" => input.get("url").and_then(|v| v.as_str()).map(|url| vec![Source { title: None, url: url.to_string() }]).unwrap_or_default(),
        // web_search results read "1. Title\n   https://…" (see nomi_agent_core::web::format_hits).
        "web_search" => {
            let mut sources = Vec::new();
            let mut title: Option<String> = None;
            for line in result.lines() {
                let trimmed = line.trim();
                if let Some((number, rest)) = trimmed.split_once(". ") {
                    if number.chars().all(|c| c.is_ascii_digit()) && !number.is_empty() {
                        title = Some(rest.trim().to_string());
                        continue;
                    }
                }
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    let title = title.take().filter(|t| t != trimmed);
                    sources.push(Source { title, url: trimmed.to_string() });
                }
            }
            sources
        }
        _ => Vec::new(),
    }
}

/// Fills in each assistant message's steps and sources.
pub async fn attach_steps(pool: &PgPool, session_id: Uuid, messages: &mut [MessageItem]) -> Result<(), sqlx::Error> {
    let Some(last) = messages.iter().rev().find(|m| m.sender == "assistant") else { return Ok(()) };
    let until = last.created_at;
    let first = messages[0].created_at;
    // Steps before the first message shown belong to it only if nothing came between.
    let since: Option<DateTime<Utc>> =
        sqlx::query_scalar("SELECT max(created_at) FROM messages WHERE session_id = $1 AND created_at < $2").bind(session_id).bind(first).fetch_one(pool).await?;
    let events: Vec<(DateTime<Utc>, Value)> = sqlx::query_as(
        "SELECT created_at, payload FROM agent_events WHERE session_id = $1 AND event_type = 'ToolCalled' \
         AND created_at > COALESCE($2, '-infinity'::timestamptz) AND created_at <= $3 ORDER BY created_at",
    )
    .bind(session_id)
    .bind(since)
    .bind(until)
    .fetch_all(pool)
    .await?;

    let mut index = 0;
    for (at, payload) in events {
        while index < messages.len() && (messages[index].created_at < at || messages[index].sender != "assistant") {
            index += 1;
        }
        let Some(message) = messages.get_mut(index) else { break };
        let tool = payload.get("tool_name").and_then(|v| v.as_str()).unwrap_or_default();
        if tool.is_empty() || HIDDEN_TOOLS.contains(&tool) {
            continue;
        }
        let input = payload.get("input").cloned().unwrap_or(Value::Null);
        let result = payload.get("result").and_then(|v| v.as_str()).unwrap_or_default();
        let ok = !payload.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
        if message.steps.len() < MAX_STEPS {
            message.steps.push(Step { tool: tool.to_string(), detail: step_detail(tool, &input), ok, output: step_output(tool, result) });
        }
        if ok {
            for source in step_sources(tool, &input, result) {
                if message.sources.len() < MAX_SOURCES && !message.sources.iter().any(|s| s.url == source.url) {
                    message.sources.push(source);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn details_show_what_each_tool_worked_on() {
        assert_eq!(step_detail("web_search", &json!({"query": "best cafes in Bandung"})).as_deref(), Some("best cafes in Bandung"));
        assert_eq!(step_detail("run_command", &json!({"project_id": "x", "command": "npm run check"})).as_deref(), Some("npm run check"));
        assert_eq!(step_detail("delegate_to_agent", &json!({"target_agent": "coding", "task": "build"})).as_deref(), Some("coding"));
        assert_eq!(
            step_detail("write_files", &json!({"files": [{"path": "a.ts"}, {"path": "b.ts"}, {"path": "c.ts"}]})).as_deref(),
            Some("a.ts +2")
        );
        assert_eq!(step_detail("whatever", &json!({})), None);
    }

    #[test]
    fn commands_keep_the_end_of_their_output() {
        assert_eq!(step_output("run_command", "$ npm run check\nexit code 1\nsrc/a.ts:3 error").as_deref(), Some("exit code 1\nsrc/a.ts:3 error"));
        assert_eq!(step_output("web_search", "anything"), None);
        let long = format!("$ npm install\n{}", "x".repeat(MAX_OUTPUT + 50));
        let out = step_output("run_command", &long).unwrap();
        assert!(out.starts_with('…') && out.chars().count() == MAX_OUTPUT + 1);
    }

    #[test]
    fn search_results_and_read_pages_become_sources() {
        let result = "Results for \"x\" (cite the links you use):\n\n1. Kopi Nako\n   https://kopinako.id\n   A café.\n\n2. https://bare.example\n   https://bare.example\n";
        assert_eq!(
            step_sources("web_search", &json!({"query": "x"}), result),
            vec![
                Source { title: Some("Kopi Nako".into()), url: "https://kopinako.id".into() },
                Source { title: None, url: "https://bare.example".into() },
            ]
        );
        assert_eq!(step_sources("read_web_page", &json!({"url": "https://a.example/p"}), "page text"), vec![Source { title: None, url: "https://a.example/p".into() }]);
        assert!(step_sources("write_file", &json!({}), "").is_empty());
    }
}
