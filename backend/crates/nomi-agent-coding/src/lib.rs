//! Koda, the coding agent: builds projects with Nomi's stack (Vite, TypeScript, Tailwind;
//! SvelteKit first) from his guides, writes the code into the project's storage, and runs
//! commands in the project's live preview in the user's browser.

pub mod files;
pub mod guides;
pub mod runtime;

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::CODING_SYSTEM_PROMPT;
use nomi_agent_core::{SubAgent, ToolOutcome};
use nomi_llm::ToolDefinition;
use nomi_storage::ProjectStore;

pub use files::{
    delete_file, delete_file_tool_definition, edit_file, edit_file_tool_definition, guess_content_type, list_files,
    list_files_tool_definition, project_file_key, read_file, read_file_tool_definition, search_files, search_files_tool_definition,
    validate_path, write_file, write_file_tool_definition, write_files, write_files_tool_definition,
};

pub const CODING_AGENT_TYPE: &str = "coding";

/// Every tool Koda has, by name (also used by the tool catalog for custom agents).
pub const TOOL_NAMES: &[&str] =
    &["read_guide", "list_files", "read_file", "search_files", "write_files", "write_file", "edit_file", "delete_file", "run_command"];

pub fn tool_definition(name: &str) -> Option<ToolDefinition> {
    Some(match name {
        "read_guide" => guides::read_guide_tool_definition(),
        "list_files" => list_files_tool_definition(),
        "read_file" => read_file_tool_definition(),
        "search_files" => search_files_tool_definition(),
        "write_files" => write_files_tool_definition(),
        "write_file" => write_file_tool_definition(),
        "edit_file" => edit_file_tool_definition(),
        "delete_file" => delete_file_tool_definition(),
        "run_command" => runtime::run_command_tool_definition(),
        _ => return None,
    })
}

/// Runs one of Koda's tools.
pub async fn execute(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, user_id: Uuid, name: &str, input: Value) -> Result<ToolOutcome, String> {
    match name {
        "read_guide" => guides::read_guide(&input).map(ToolOutcome::text),
        "list_files" => list_files(conn, user_id, input).await.map(ToolOutcome::text),
        "read_file" => read_file(conn, storage, user_id, input).await.map(ToolOutcome::text),
        "search_files" => search_files(conn, storage, user_id, input).await.map(ToolOutcome::text),
        "write_files" => write_files(conn, storage, user_id, input).await,
        "write_file" => write_file(conn, storage, user_id, input).await,
        "edit_file" => edit_file(conn, storage, user_id, input).await,
        "delete_file" => delete_file(conn, storage, user_id, input).await,
        "run_command" => runtime::run_command(conn, user_id, input).await.map(ToolOutcome::text),
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Checks whether `task` starts with the "Project <uuid>: ..." prefix CODING_SYSTEM_PROMPT
/// requires — used by validate_delegation_task to reject a delegation before it's created,
/// rather than let this agent hallucinate a project_id when the planning agent skipped
/// create_project/write_plan and delegated without ever making a project (a real failure mode:
/// the model doesn't always follow the create-then-delegate sequence its own prompt asks for).
fn has_project_prefix(task: &str) -> bool {
    task.strip_prefix("Project ")
        .and_then(|rest| rest.split_once(':'))
        .map(|(id, _)| id.trim().parse::<Uuid>().is_ok())
        .unwrap_or(false)
}

pub struct CodingAgent {
    storage: ProjectStore,
}

impl CodingAgent {
    pub fn new(storage: ProjectStore) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl SubAgent for CodingAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            guides::read_guide_tool_definition(),
            list_files_tool_definition(),
            read_file_tool_definition(),
            search_files_tool_definition(),
            write_files_tool_definition(),
            write_file_tool_definition(),
            edit_file_tool_definition(),
            delete_file_tool_definition(),
            runtime::run_command_tool_definition(),
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        execute(conn, &self.storage, user_id, name, input).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("writing or editing code for a project — not reachable directly, only via delegation from planning")
    }

    /// Koda's tool results (command output, file listings, guides) aren't posted as messages:
    /// they're steps in his reply's activity, with command output there to expand. File changes
    /// still show as their own cards (rich blocks always post).
    fn surfaces_activity(&self) -> bool {
        false
    }

    fn supports_todos(&self) -> bool {
        true
    }

    /// Koda's tools only touch the project's own files and run commands in the person's own
    /// browser preview: building shouldn't stop for an approval at every file.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        !TOOL_NAMES.contains(&tool_name)
    }

    fn supports_plans(&self) -> bool {
        true
    }

    /// Building a project takes a while: it runs in the background and reports back.
    fn works_in_background(&self) -> bool {
        true
    }

    /// Setting up, installing, checking and fixing a project takes many rounds of tools.
    fn max_tool_turns(&self) -> u32 {
        60
    }

    fn validate_delegation_task(&self, task: &str) -> Result<(), String> {
        if has_project_prefix(task) {
            Ok(())
        } else {
            Err(
                "This task has no 'Project <uuid>: ...' prefix, so there's no project to write \
                 files into. Call create_project (and write_plan) first, then delegate again \
                 with a task formatted exactly as 'Project <the real project id>: <summary>'."
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_koda_has_is_named_and_defined() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let names: Vec<String> = agent.tools().into_iter().map(|t| t.name).collect();
        assert_eq!(names, TOOL_NAMES);
        assert!(TOOL_NAMES.iter().all(|n| tool_definition(n).is_some()));
    }

    #[test]
    fn rejects_a_delegation_task_with_no_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let result = agent.validate_delegation_task("Build an advanced calculator app with trig functions.");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_delegation_task_with_a_malformed_project_id() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        assert!(agent.validate_delegation_task("Project not-a-real-uuid: build it").is_err());
    }

    #[test]
    fn accepts_a_delegation_task_with_a_real_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let task = format!("Project {}: build a calculator", Uuid::new_v4());
        assert!(agent.validate_delegation_task(&task).is_ok());
    }
}
